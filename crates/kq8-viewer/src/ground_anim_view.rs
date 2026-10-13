//! Ground animators in the ground view (M3 step 3b-3, docs/DESIGN.md;
//! rules: docs/behaviour/emitters.md "Ground animators").
//!
//! - The animators live in `kq8_game::ground_anim` and run at 15 updates
//!   per second; game time = updates / 15.
//! - Each animated square is its own entity, taken out of the normal
//!   ground mesh. When its material changes, it gets that material's
//!   texture, but keeps the orientation of its own (loaded) material
//!   (stand-in, Q50).
//! - The U key's views treat these squares as surface squares (hidden in
//!   "floor only", see-through in "both" where the floor differs).
//!
//! Log: `kq8-artifacts/logs/ground_anim.log`, every change with its update,
//! game time and material.

use std::collections::BTreeMap;

use bevy::image::ImageAddressMode;
use bevy::prelude::*;
use kq8_game::ground_anim::GroundAnimator;

use crate::load::GroundAnimAssets;
use crate::log::Log;
use crate::terrain_view::FloorView;
use crate::{material_for, mesh_from, to_bevy, with_opacity, Shared};

/// Opacity of an animated square over a different floor in the U key's
/// "both" view (the same as the rest of the surface there).
const SURFACE_SEE_THROUGH: f32 = 0.35;

#[derive(Resource)]
pub struct LoadedGroundAnim(pub Option<GroundAnimAssets>);

/// A material's (opaque, see-through) look.
type SquareLook = (Handle<StandardMaterial>, Handle<StandardMaterial>);

/// One square's entity and what it shows now.
struct DrawnSquare {
    entity: Entity,
    shown: Option<u8>,
    floor_differs: bool,
}

#[derive(Resource)]
pub struct GroundAnimSim {
    animators: Vec<GroundAnimator>,
    materials: kq8_data::terrain::Grid,
    squares: BTreeMap<(usize, usize), DrawnSquare>,
    meshes: BTreeMap<((usize, usize), u8), Handle<Mesh>>,
    /// Per material: (opaque, see-through); `None` = not drawn.
    looks: BTreeMap<u8, Option<SquareLook>>,
    /// The U view the squares were last drawn for.
    view: u8,
    updates: u64,
    /// Per animator: the update of its last change, and the gaps seen.
    last_change: Vec<Option<u64>>,
    gaps: Vec<BTreeMap<u64, u64>>,
    log: Log,
}

pub fn setup_ground_anim(
    mut commands: Commands,
    mut loaded: ResMut<LoadedGroundAnim>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(assets) = loaded.0.take() else { return };
    let mut main_log = shared.log.lock().unwrap();
    let mut log = match Log::create("ground_anim") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("ground animation: cannot create ground_anim.log: {e}; not started"));
            return;
        }
    };
    let mut built_meshes = BTreeMap::new();
    for (key, triangles) in assets.triangles {
        let mut positions = Vec::with_capacity(triangles.len() * 3);
        let mut uvs = Vec::with_capacity(triangles.len() * 3);
        for t in &triangles {
            // Clockwise from above (the game's front) -> reversed for Bevy.
            for k in [0, 2, 1] {
                positions.push(to_bevy(t.positions[k], [0.0; 3]));
                uvs.push(t.uvs[k]);
            }
        }
        built_meshes.insert(key, meshes.add(mesh_from(positions, uvs)));
    }
    let mut looks = BTreeMap::new();
    for (m, look) in assets.looks {
        looks.insert(
            m,
            look.map(|look| {
                // One texture covers exactly one square: clamp at the edges.
                let material = material_for(look, ImageAddressMode::ClampToEdge, &mut images);
                let see_through = materials.add(with_opacity(&material, SURFACE_SEE_THROUGH));
                (materials.add(material), see_through)
            }),
        );
    }
    let mut squares = BTreeMap::new();
    for a in &assets.animators {
        for &sq in &a.squares {
            squares.entry(sq).or_insert_with(|| DrawnSquare {
                entity: commands.spawn((Transform::default(), Visibility::Hidden)).id(),
                shown: None,
                floor_differs: assets.floor_differs.contains(&sq),
            });
        }
    }
    let line = format!(
        "ground animation: {} animators, {} squares drawn as their own entities; log in {}",
        assets.animators.len(),
        squares.len(),
        log.path().display()
    );
    main_log.summary(&line);
    log.detail(line);
    for a in &assets.animators {
        log.detail(format!(
            "[{}] {} squares {:?}; cycle {:?} every {} s repeat {} running {} first change due at {:.3} s",
            a.name, a.squares.len(), a.squares, a.cycle, a.delay, a.repeat, a.running, a.due
        ));
    }
    let n = assets.animators.len();
    commands.insert_resource(GroundAnimSim {
        animators: assets.animators,
        materials: assets.materials,
        squares,
        meshes: built_meshes,
        looks,
        view: u8::MAX,
        updates: 0,
        last_change: vec![None; n],
        gaps: vec![BTreeMap::new(); n],
        log,
    });
}

/// One update (15 per second): every animator's changes due by now.
pub fn ground_anim_step(sim: Option<ResMut<GroundAnimSim>>, shared: Res<Shared>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    let now = n as f64 / kq8_game::UPDATES_PER_SECOND;
    for (k, a) in sim.animators.iter_mut().enumerate() {
        for c in a.update(now, &mut sim.materials) {
            let gap = sim.last_change[k].map(|last| n - last);
            if let Some(g) = gap {
                *sim.gaps[k].entry(g).or_default() += 1;
            }
            sim.last_change[k] = Some(n);
            sim.log.detail(format!(
                "update {n} (game time {now:.3} s, due {:.3} s): [{}] entry {} material {} on {} squares{}",
                c.time,
                a.name,
                c.entry,
                c.material,
                a.squares.len(),
                gap.map_or(String::new(), |g| format!(", {g} updates after the last"))
            ));
        }
    }
    // Every 150 updates (10 s): the gaps between changes so far.
    if n % 150 == 0 {
        let summary: Vec<String> = sim
            .animators
            .iter()
            .zip(&sim.gaps)
            .filter(|(a, _)| a.running)
            .map(|(a, g)| format!("{} {g:?}", a.name))
            .collect();
        let line = format!("ground animation: update {n}: updates between changes (gap: count): {}", summary.join(", "));
        shared.log.lock().unwrap().detail(&line);
        sim.log.detail(line);
    }
}

/// Each animated square shows its current material (and follows the U
/// view).
#[allow(clippy::type_complexity)]
pub fn ground_anim_draw(
    sim: Option<ResMut<GroundAnimSim>>,
    view: Res<FloorView>,
    mut query: Query<(&mut Visibility, Option<&mut Mesh3d>, Option<&mut MeshMaterial3d<StandardMaterial>>)>,
    mut commands: Commands,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    let view_changed = sim.view != view.0;
    sim.view = view.0;
    for (&(i, j), sq) in &mut sim.squares {
        let m = sim.materials.get(i, j).unwrap();
        if sq.shown == Some(m) && !view_changed {
            continue;
        }
        sq.shown = Some(m);
        let Ok((mut v, mesh, material)) = query.get_mut(sq.entity) else { continue };
        let look = sim.looks.get(&m).cloned().flatten();
        let (Some((opaque, see_through)), Some(handle)) = (look, sim.meshes.get(&((i, j), m)).cloned()) else {
            *v = Visibility::Hidden;
            continue;
        };
        let wanted = if view.0 == 1 && sq.floor_differs { see_through } else { opaque };
        match (mesh, material) {
            (Some(mut mesh), Some(mut material)) => {
                mesh.0 = handle;
                material.0 = wanted;
            }
            _ => {
                commands.entity(sq.entity).insert((Mesh3d(handle), MeshMaterial3d(wanted)));
            }
        }
        *v = if view.0 == 2 { Visibility::Hidden } else { Visibility::Inherited };
    }
}
