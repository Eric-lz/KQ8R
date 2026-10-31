//! `matchHeights` in the viewer (batch 2 D-8b; rules:
//! docs/behaviour/terrain.md "`matchHeights`", logic in
//! `kq8_core::terrain_drain`).
//!
//! - Runs on the 15-per-second update, after the scripts (`trap_step`):
//!   drains asked for in an update take their first step in the next one
//!   (ours; the note gives one step every 1/15 s).
//! - Each step writes the live terrain (`GroundLookup`), which water depth,
//!   the camera and shadows read, then lowers the drawn ground in place:
//!   every surface vertex near the changed corners gets its height from the
//!   terrain again, in the normal ground meshes and the animated squares.
//!
//! Log: `kq8-artifacts/logs/ground_drain.log`, per drain the corners, and
//! the surface and floor height at its middle before and after each step.

use bevy::mesh::VertexAttributeValues;
use bevy::prelude::*;
use kq8_core::terrain_drain::{Drain, MAX_RUNNING};
use kq8_data::terrain::Level;

use crate::door_view::DoorSim;
use crate::ground_anim_view::{GroundAnimSim, GroundAsks};
use crate::log::Log;
use crate::terrain_view::{GroundLookup, GroundPart};
use crate::{Shared, UNITS_PER_METRE};

/// The running drains: each with where it came from and its middle point.
#[derive(Resource, Default)]
pub struct DrainSim {
    drains: Vec<(Drain, String, [f32; 2])>,
    log: Option<Log>,
}

// A Bevy system gets everything it uses as arguments.
#[allow(clippy::too_many_arguments)]
pub fn ground_drain_step(
    mut asks: ResMut<GroundAsks>,
    mut sim: ResMut<DrainSim>,
    lookup: Option<ResMut<GroundLookup>>,
    anim: Option<Res<GroundAnimSim>>,
    surfaces: Query<(&Mesh3d, &GroundPart)>,
    mut meshes: ResMut<Assets<Mesh>>,
    door: Option<Res<DoorSim>>,
    shared: Res<Shared>,
) {
    if sim.drains.is_empty() && asks.drains.is_empty() {
        return;
    }
    let update = door.map_or(0, |d| d.active.world.updates);
    let sim = &mut *sim;
    if sim.log.is_none() {
        match Log::create("ground_drain") {
            Ok(l) => sim.log = Some(l),
            Err(e) => shared.log.lock().unwrap().summary(format!("matchHeights: cannot create ground_drain.log: {e}")),
        }
    }
    let mut lines = Vec::new();
    let Some(mut lookup) = lookup else {
        for (words, at) in asks.drains.drain(..) {
            lines.push(format!("update {update}: {at}: matchHeights {words:?}: no ground in this world; nothing"));
        }
        write(sim, &shared, lines);
        return;
    };
    let terrain = &mut lookup.0;
    let size = terrain.settings.square_size();
    let heights = |t: &kq8_data::Terrain, p: [f32; 2]| {
        let h = |l| t.ground_at(l, p[0], p[1]).map_or(f32::NAN, |g| g.height);
        (h(Level::Surface), h(Level::Floor))
    };
    // The steps of the drains asked for in earlier updates.
    let mut changed: Vec<(usize, usize)> = Vec::new();
    for (d, at, mid) in &mut sim.drains {
        let corners = d.step(&mut terrain.heights);
        let (s, f) = heights(terrain, *mid);
        lines.push(format!(
            "update {update}: [{at}] step {}/{}: {} corners changed; at the middle ({:.0}, {:.0}) surface {s:.0}, floor {f:.0}{}",
            d.done,
            d.frames,
            corners.len(),
            mid[0],
            mid[1],
            if d.finished() { "; finished" } else { "" }
        ));
        changed.extend(corners);
    }
    sim.drains.retain(|(d, _, _)| !d.finished());
    // The new ones (terrain.md: up to 32 at once).
    for (words, at) in asks.drains.drain(..) {
        if sim.drains.len() >= MAX_RUNNING {
            // What the original does then: not in the note (ours: refused).
            lines.push(format!("update {update}: {at}: matchHeights {words:?}: {MAX_RUNNING} already running; not started"));
            continue;
        }
        let Some(floor) = terrain.floor.as_ref() else {
            lines.push(format!("update {update}: {at}: matchHeights {words:?}: this world has no floor grid (the surface is the floor); nothing"));
            continue;
        };
        match Drain::new(words, size as i64, &terrain.heights, floor) {
            Ok(d) => {
                let mid = [(words[0] + words[2]) as f32 / 2.0, (words[1] + words[3]) as f32 / 2.0];
                let (s, f) = heights(terrain, mid);
                let (n, differ) = d.counts();
                lines.push(format!(
                    "update {update}: {at}: matchHeights {words:?}: corner columns {:?}, rows {:?} (max not included): {n} corners, {differ} above or below the floor; at the middle ({:.0}, {:.0}) surface {s:.0}, floor {f:.0}; {} steps from the next update",
                    d.columns, d.rows, mid[0], mid[1], d.frames
                ));
                sim.drains.push((d, at, mid));
            }
            Err(why) => lines.push(format!("update {update}: {at}: matchHeights {words:?}: {why}")),
        }
    }
    if !changed.is_empty() {
        lines.push(redraw(terrain, &changed, anim.as_deref(), &surfaces, &mut meshes, update));
    }
    write(sim, &shared, lines);
}

fn write(sim: &mut DrainSim, shared: &Shared, lines: Vec<String>) {
    let mut main = shared.log.lock().unwrap();
    for l in lines {
        main.summary(&l);
        if let Some(log) = sim.log.as_mut() {
            log.detail(l);
        }
    }
}

/// Puts every drawn surface vertex in the squares around the changed
/// corners back on the terrain; returns the log line.
fn redraw(
    terrain: &kq8_data::Terrain,
    changed: &[(usize, usize)],
    anim: Option<&GroundAnimSim>,
    surfaces: &Query<(&Mesh3d, &GroundPart)>,
    meshes: &mut Assets<Mesh>,
    update: u64,
) -> String {
    let size = terrain.settings.square_size();
    let (nx, ny) = terrain.squares();
    let i0 = changed.iter().map(|c| c.0).min().unwrap();
    let i1 = changed.iter().map(|c| c.0).max().unwrap();
    let j0 = changed.iter().map(|c| c.1).min().unwrap();
    let j1 = changed.iter().map(|c| c.1).max().unwrap();
    // Squares touching those corners: columns i0-1 ..= i1, rows likewise.
    let columns = (i0.saturating_sub(1), (i1 + 1).min(nx));
    let rows = (j0.saturating_sub(1), (j1 + 1).min(ny));
    let (x0, x1) = (columns.0 as f32 * size, columns.1 as f32 * size);
    let (y0, y1) = (rows.0 as f32 * size, rows.1 as f32 * size);
    let mut handles: Vec<Handle<Mesh>> = surfaces
        .iter()
        .filter(|(_, p)| matches!(p, GroundPart::SurfaceSame | GroundPart::SurfaceOverFloor { .. }))
        .map(|(m, _)| m.0.clone())
        .collect();
    let animated = anim.map(|a| a.meshes_in(columns, rows)).unwrap_or_default();
    let n_animated = animated.len();
    handles.extend(animated);
    handles.sort_by_key(|h| h.id());
    handles.dedup();
    let (mut touched, mut moved) = (0, 0);
    for h in handles {
        let Some(mut mesh) = meshes.get_mut(&h) else { continue };
        let Some(VertexAttributeValues::Float32x3(positions)) = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION) else { continue };
        let mut any = false;
        for p in positions.iter_mut() {
            // Back from `to_bevy`: game x = bevy x, game y = -bevy z, height = bevy y.
            let (gx, gy) = (p[0] * UNITS_PER_METRE, -p[2] * UNITS_PER_METRE);
            if !(gx >= x0 - 0.5 && gx <= x1 + 0.5 && gy >= y0 - 0.5 && gy <= y1 + 0.5) {
                continue;
            }
            let Some(g) = terrain.ground_at(Level::Surface, gx, gy) else { continue };
            let z = g.height / UNITS_PER_METRE;
            if (z - p[1]).abs() * UNITS_PER_METRE > 0.5 {
                moved += 1;
                any = true;
            }
            p[1] = z;
        }
        if any {
            mesh.compute_flat_normals();
            touched += 1;
        }
    }
    format!(
        "update {update}: drawn ground lowered: squares columns {columns:?} rows {rows:?}: {moved} vertices moved in {touched} meshes ({n_animated} animated-square meshes looked at)"
    )
}
