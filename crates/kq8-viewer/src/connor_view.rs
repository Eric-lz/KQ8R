//! Connor in the ground view (M2 step 2a, docs/DESIGN.md): drawn in his
//! standing pose, his mover (`kq8_game::mover`, rules from
//! docs/behaviour/connor-physics.md) run at a fixed 15 updates per second
//! with gravity on. Drawn between his last two updates so motion is smooth;
//! that blend is display only.
//!
//! Test keys: G lifts him 5,000 (should die on landing), B lifts him 2,000
//! (safe), C puts the camera behind him.

use bevy::image::ImageAddressMode;
use bevy::prelude::*;
use kq8_data::objects::Placement;
use kq8_game::mover::{Mover, State};

use crate::load::Model;
use crate::log::Log;
use crate::terrain_view::{bevy_transform, Fly, GroundLookup};
use crate::{material_for, mesh_from, to_bevy, Shared};

/// Connor's mover and what is needed to draw and log it.
#[derive(Resource)]
pub struct ConnorSim {
    pub mover: Mover,
    /// Placement (angles, name) used to build his transform.
    pub placement: Placement,
    /// Positions after the last two updates, for the drawing blend.
    pub previous: [f32; 3],
    pub current: [f32; 3],
    pub updates: u64,
    /// One line per update: `kq8-artifacts/logs/connor.log`.
    pub log: Log,
    last_state: &'static str,
    camera_placed: bool,
}

#[derive(Component)]
pub struct ConnorBody;

/// `--lift N`: lift Connor this much once at the start (a test, like G/B).
#[derive(Resource)]
pub struct StartLift(pub Option<f32>);

/// Spawns Connor's model and starts his mover (gravity on:
/// connor-physics.md "Is gravity on at the start?", recommendation).
pub fn spawn_connor(
    commands: &mut Commands,
    (placement, model): (Placement, Model),
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let mut parts = Vec::new();
    for group in model.groups {
        let mut positions = Vec::with_capacity(group.triangles.len() * 3);
        let mut uvs = Vec::with_capacity(group.triangles.len() * 3);
        for t in &group.triangles {
            // Clockwise front (the game) -> counter-clockwise (Bevy).
            for k in [0, 2, 1] {
                positions.push(to_bevy(t.positions[k], [0.0; 3]));
                uvs.push(t.uvs[k]);
            }
        }
        let material = material_for(group.look, ImageAddressMode::Repeat, images);
        parts.push((meshes.add(mesh_from(positions, uvs)), materials.add(material)));
    }
    commands.spawn((bevy_transform(&placement), Visibility::default(), ConnorBody)).with_children(|c| {
        for (mesh, material) in parts {
            c.spawn((Mesh3d(mesh), MeshMaterial3d(material)));
        }
    });
    let mover = Mover::new(placement.position, true);
    let log = match Log::create("connor") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("connor: cannot create connor.log: {e}"));
            return;
        }
    };
    main_log.summary(format!(
        "connor: mover started at ({:.0}, {:.0}, {:.0}), gravity {}, step up/down {}/{}, {} updates per second; per-update log in {}",
        mover.position[0],
        mover.position[1],
        mover.position[2],
        if mover.use_gravity { "on" } else { "off" },
        mover.step_up,
        mover.step_down,
        kq8_game::UPDATES_PER_SECOND,
        log.path().display()
    ));
    let mut sim = ConnorSim {
        current: mover.position,
        previous: mover.position,
        mover,
        placement,
        updates: 0,
        log,
        last_state: "normal",
        camera_placed: false,
    };
    sim.log.detail("update time_s x y z on_ground gravity fall_timer air_updates state drop ground_z ground_normal_z");
    commands.insert_resource(sim);
}

/// One mover update; runs in Bevy's fixed-rate schedule (15 per second).
pub fn connor_step(sim: Option<ResMut<ConnorSim>>, lookup: Option<Res<GroundLookup>>, shared: Res<Shared>) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let report = sim.mover.update([0.0; 3], &lookup.0);
    sim.updates += 1;
    sim.previous = sim.current;
    sim.current = sim.mover.position;
    let m = &sim.mover;
    let line = format!(
        "{} {:.3} {:.1} {:.1} {:.1} {} {} {:.2} {} {} {:.1} {} {}",
        sim.updates,
        sim.updates as f64 / kq8_game::UPDATES_PER_SECOND,
        m.position[0],
        m.position[1],
        m.position[2],
        m.on_ground,
        m.use_gravity,
        m.fall_timer,
        m.air_updates,
        m.state.name(),
        report.drop,
        report.ground.map_or("none".into(), |g| format!("{:.1}", g.0)),
        report.ground.map_or("none".into(), |g| format!("{:.3}", g.1[2])),
    );
    let state = m.state.name();
    sim.log.detail(line);
    if state != sim.last_state || !report.events.is_empty() {
        let mut main = shared.log.lock().unwrap();
        for e in &report.events {
            let e = format!("connor update {}: {e}", sim.updates);
            main.summary(&e);
            sim.log.detail(e);
        }
        if state != sim.last_state {
            main.detail(format!("connor update {}: state {} -> {state}", sim.updates, sim.last_state));
        }
        sim.last_state = state;
    }
}

/// Draws Connor between his last two updates (display only).
pub fn connor_draw(
    sim: Option<Res<ConnorSim>>,
    fixed: Res<Time<Fixed>>,
    mut body: Query<&mut Transform, With<ConnorBody>>,
) {
    let Some(sim) = sim else { return };
    let a = fixed.overstep_fraction();
    let p: [f32; 3] = std::array::from_fn(|i| sim.previous[i] + (sim.current[i] - sim.previous[i]) * a);
    for mut t in &mut body {
        t.translation = Vec3::from(to_bevy(p, [0.0; 3]));
    }
}

/// Test keys G (lift 5,000), B (lift 2,000) and C (camera behind him);
/// the camera is also put behind him once at the start.
pub fn connor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Option<ResMut<ConnorSim>>,
    shared: Res<Shared>,
    mut fly: ResMut<Fly>,
    mut start_lift: ResMut<StartLift>,
    mut camera: Query<&mut Transform, (With<Camera3d>, Without<ConnorBody>)>,
) {
    let Some(mut sim) = sim else { return };
    let start = start_lift.0.take().map(|h| ("--lift".to_string(), h));
    let pressed = [(KeyCode::KeyG, 5000.0), (KeyCode::KeyB, 2000.0)]
        .into_iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(k, h)| (format!("key {k:?}"), h));
    for (what, height) in start.into_iter().chain(pressed) {
        {
            let was = sim.mover.state;
            sim.mover.lift(height);
            sim.previous = sim.mover.position;
            sim.current = sim.mover.position;
            let line = format!(
                "connor update {}: test {what}: lifted {height} to z={:.0}{}",
                sim.updates,
                sim.mover.position[2],
                if was == State::Dead { " (was dead: alive again)" } else { "" }
            );
            shared.log.lock().unwrap().summary(&line);
            sim.log.detail(line);
        }
    }
    if keys.just_pressed(KeyCode::KeyC) || !sim.camera_placed {
        let Ok(mut cam) = camera.single_mut() else { return };
        sim.camera_placed = true;
        // He faces -Y of his model, turned by dirZ.
        let heading = sim.placement.angles[2];
        let forward = [heading.sin(), -heading.cos()];
        let p = sim.current;
        let eye = [p[0] - forward[0] * 3500.0, p[1] - forward[1] * 3500.0, p[2] + 2200.0];
        let target = [p[0], p[1], p[2] + 1400.0];
        let (eye, target) = (Vec3::from(to_bevy(eye, [0.0; 3])), Vec3::from(to_bevy(target, [0.0; 3])));
        let d = target - eye;
        // Matches fly_input: rotation = yaw about Y, then pitch.
        fly.yaw = (-d.x).atan2(-d.z);
        fly.pitch = (d.y / d.length()).asin();
        cam.translation = eye;
        shared.log.lock().unwrap().detail(format!(
            "camera behind Connor at game ({:.0}, {:.0}, {:.0})",
            eye_game(eye)[0],
            eye_game(eye)[1],
            eye_game(eye)[2]
        ));
    }
}

fn eye_game(bevy: Vec3) -> [f32; 3] {
    let p = bevy * crate::UNITS_PER_METRE;
    [p.x, -p.z, p.y]
}
