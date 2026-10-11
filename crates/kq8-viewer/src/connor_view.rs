//! Connor in the ground view (M2 steps 2a and 2b, docs/DESIGN.md).
//!
//! - His mover (`kq8_game::mover`, docs/behaviour/connor-physics.md) and his
//!   walking (`kq8_game::connor`, docs/behaviour/connor-movement.md) run at
//!   a fixed 15 updates per second, gravity on.
//! - Keys from `MASK/Connor.sae`; the intent comes from the keys held (the
//!   note's recommendation), so releasing one of two opposite keys gives the
//!   other back.
//! - Every frame of his current movement is drawn in turn; position,
//!   heading and loop position are blended between the last two updates
//!   (display only).
//!
//! Test keys: G lifts him 5,000 (should die on landing), B lifts him 2,000
//! (safe). The camera that follows him is in `game_camera.rs`.

use bevy::image::ImageAddressMode;
use bevy::prelude::*;
use kq8_data::objects::Placement;
use kq8_game::connor::{Intent, MovementLoop, Walker};
use kq8_game::mover::{Mover, State};

use crate::load::ConnorAssets;
use crate::log::Log;
use crate::terrain_view::{bevy_transform, GroundLookup};
use crate::{material_for, mesh_from, to_bevy, Shared};

/// One drawable movement: its loop for the walker and one entity per frame.
pub struct DrawnMovement {
    pub name: &'static str,
    pub movement_loop: MovementLoop,
    pub frames: Vec<Entity>,
}

/// Keys from `Connor.sae`, and which of them are held, oldest first.
pub struct Controls {
    forward: Option<KeyCode>,
    back: Option<KeyCode>,
    left: Option<KeyCode>,
    right: Option<KeyCode>,
    run: Option<KeyCode>,
    held: Vec<KeyCode>,
}

impl Controls {
    /// The value of the pair (`plus` gives +1, `minus` −1) from the keys
    /// held: the one pressed last wins.
    fn axis(&self, plus: Option<KeyCode>, minus: Option<KeyCode>) -> i8 {
        for k in self.held.iter().rev() {
            if Some(*k) == plus {
                return 1;
            }
            if Some(*k) == minus {
                return -1;
            }
        }
        0
    }
}

/// Connor's mover, walker, drawing and logs.
#[derive(Resource)]
pub struct ConnorSim {
    pub mover: Mover,
    pub walker: Walker,
    pub intent: Intent,
    pub controls: Controls,
    pub movements: Vec<DrawnMovement>,
    /// Placement (name, angles) used to build his transform; angle Z is
    /// the walker's heading.
    pub placement: Placement,
    /// Position, heading and loop position after the last two updates.
    previous: ([f32; 3], f32, f32),
    current: ([f32; 3], f32, f32),
    /// Share of the loop played per update (for blending the loop position).
    phase_step: f32,
    pub updates: u64,
    /// One line per update: `kq8-artifacts/logs/connor.log`.
    pub log: Log,
    last_state: &'static str,
    shown: Option<Entity>,
}

#[derive(Component)]
pub struct ConnorBody;

/// `--lift N`: lift Connor this much once at the start (a test, like G/B).
#[derive(Resource)]
pub struct StartLift(pub Option<f32>);

/// `--hold F,T,R`: a test that holds this intent (forward, turn, run 0/1)
/// instead of the keys.
#[derive(Resource)]
pub struct HoldIntent(pub Option<[i8; 3]>);

/// Bevy key for a key name in a `.sae` file. Only the names Connor's
/// movement uses so far, plus letters and function keys.
pub(crate) fn key_code(name: &str) -> Option<KeyCode> {
    Some(match name {
        "up" => KeyCode::ArrowUp,
        "down" => KeyCode::ArrowDown,
        "left" => KeyCode::ArrowLeft,
        "right" => KeyCode::ArrowRight,
        "end" => KeyCode::End,
        "home" => KeyCode::Home,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        "pageup" | "prior" => KeyCode::PageUp,
        "pagedown" | "next" => KeyCode::PageDown,
        "f2" => KeyCode::F2,
        "f11" => KeyCode::F11,
        "shift" => KeyCode::ShiftLeft,
        "control" => KeyCode::ControlLeft,
        "alt" => KeyCode::AltLeft,
        _ => return None,
    })
}

/// Spawns Connor (every frame of every movement, hidden but one) and
/// starts his mover with gravity on (connor-physics.md "Is gravity on at
/// the start?", recommendation).
pub fn spawn_connor(
    commands: &mut Commands,
    (placement, assets): (Placement, ConnorAssets),
    camera_start: Option<[f32; 3]>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let camera_keys = assets.camera_keys.clone();
    let connor_keys = assets.keys.clone();
    let loops: Vec<(f32, [f32; 3])> = assets.animations.iter().map(|a| (a.duration, a.step_sum)).collect();
    // Each animation: one material per look, one entity per frame.
    let mut frame_entities: Vec<Vec<Entity>> = Vec::new();
    let body = commands.spawn((bevy_transform(&placement), Visibility::default(), ConnorBody)).id();
    for anim in assets.animations {
        let looks: Vec<(i32, Handle<StandardMaterial>)> = anim
            .looks
            .into_iter()
            .map(|(m, look)| (m, materials.add(material_for(look, ImageAddressMode::Repeat, images))))
            .collect();
        let mut entities = Vec::new();
        for frame in anim.frames {
            let parent = commands.spawn((Transform::default(), Visibility::Hidden)).id();
            commands.entity(body).add_child(parent);
            for (m, triangles) in frame {
                let mut positions = Vec::with_capacity(triangles.len() * 3);
                let mut uvs = Vec::with_capacity(triangles.len() * 3);
                for t in &triangles {
                    // Clockwise front (the game) -> counter-clockwise (Bevy).
                    for k in [0, 2, 1] {
                        positions.push(to_bevy(t.positions[k], [0.0; 3]));
                        uvs.push(t.uvs[k]);
                    }
                }
                let Some((_, material)) = looks.iter().find(|(i, _)| *i == m) else { continue };
                let part = commands.spawn((Mesh3d(meshes.add(mesh_from(positions, uvs))), MeshMaterial3d(material.clone()))).id();
                commands.entity(parent).add_child(part);
            }
            entities.push(parent);
        }
        frame_entities.push(entities);
    }
    let movements: Vec<DrawnMovement> = assets
        .movements
        .iter()
        .map(|m| {
            let (duration, step_sum) = loops[m.animation];
            DrawnMovement {
                name: m.name,
                movement_loop: MovementLoop { duration, step_sum, speed: m.speed },
                frames: frame_entities[m.animation].clone(),
            }
        })
        .collect();
    for d in &movements {
        main_log.summary(format!(
            "connor: {} moves at {:.1} units/s ({} frames)",
            d.name,
            d.movement_loop.ground_speed(),
            d.frames.len()
        ));
    }

    let mut key = |action: &str, value: f32| {
        kq8_data::action_map::key_for(&assets.keys, action, value).and_then(|name| {
            let code = key_code(name);
            if code.is_none() {
                main_log.summary(format!("connor: key '{name}' for {action} {value} is not known to the viewer"));
            }
            code
        })
    };
    let controls = Controls {
        forward: key("MoveY", 1.0),
        back: key("MoveY", -1.0),
        left: key("RotateZ", 1.0),
        right: key("RotateZ", -1.0),
        run: key("Run", 0.0),
        held: Vec::new(),
    };
    main_log.summary(format!(
        "connor: keys from Connor.sae: forward {:?}, back {:?}, turn left {:?}, turn right {:?}, run on/off {:?}",
        controls.forward, controls.back, controls.left, controls.right, controls.run
    ));

    let mover = Mover::new(placement.position, true);
    let log = match Log::create("connor") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("connor: cannot create connor.log: {e}"));
            return;
        }
    };
    main_log.summary(format!(
        "connor: mover started at ({:.0}, {:.0}, {:.0}), gravity {}, step up/down {}/{}, {} updates per second, run on; per-update log in {}",
        mover.position[0],
        mover.position[1],
        mover.position[2],
        if mover.use_gravity { "on" } else { "off" },
        mover.step_up,
        mover.step_down,
        kq8_game::UPDATES_PER_SECOND,
        log.path().display()
    ));
    let heading = placement.angles[2];
    let start = (mover.position, heading, 0.0);
    let mut sim = ConnorSim {
        mover,
        walker: Walker::new(heading),
        // A freshly made Connor runs (connor-movement.md rule 4).
        intent: Intent { forward: 0, turn: 0, run: true },
        controls,
        movements,
        placement,
        previous: start,
        current: start,
        phase_step: 0.0,
        updates: 0,
        log,
        last_state: "normal",
        shown: None,
    };
    sim.log.detail(
        "update time_s x y z on_ground gravity fall_timer air_updates state drop ground_z ground_normal_z forward turn run movement heading phase wanted_x wanted_y",
    );
    if let Some(cam) = crate::game_camera::make(
        sim.mover.position,
        heading,
        camera_start,
        &camera_keys,
        &connor_keys,
        main_log,
    ) {
        commands.insert_resource(cam);
        commands.insert_resource(crate::game_camera::CameraChoice { game: true });
    }
    commands.insert_resource(sim);
}

/// Keys: Connor's movement keys (held order) and the test keys G and B.
pub fn connor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Option<ResMut<ConnorSim>>,
    shared: Res<Shared>,
    mut start_lift: ResMut<StartLift>,
    hold: Res<HoldIntent>,
) {
    let Some(mut sim) = sim else { return };
    // Movement keys, in the order they were pressed.
    let c = &sim.controls;
    let movement_keys: Vec<KeyCode> = [c.forward, c.back, c.left, c.right].into_iter().flatten().collect();
    for k in &movement_keys {
        if keys.just_pressed(*k) {
            sim.controls.held.retain(|h| h != k);
            sim.controls.held.push(*k);
        }
        if keys.just_released(*k) {
            sim.controls.held.retain(|h| h != k);
        }
    }
    let c = &sim.controls;
    let (forward, turn) = (c.axis(c.forward, c.back), c.axis(c.left, c.right));
    let run_pressed = c.run.is_some_and(|k| keys.just_pressed(k));
    sim.intent.forward = forward;
    sim.intent.turn = turn;
    if let Some([f, t, r]) = hold.0 {
        sim.intent = Intent { forward: f.clamp(-1, 1), turn: t.clamp(-1, 1), run: r != 0 };
    }
    if run_pressed {
        sim.intent.run = !sim.intent.run;
        let line = format!("connor update {}: run switch {}", sim.updates, if sim.intent.run { "on" } else { "off" });
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }

    let start = start_lift.0.take().map(|h| ("--lift".to_string(), h));
    let pressed = [(KeyCode::KeyG, 5000.0), (KeyCode::KeyB, 2000.0)]
        .into_iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(k, h)| (format!("key {k:?}"), h));
    for (what, height) in start.into_iter().chain(pressed) {
        let was = sim.mover.state;
        sim.mover.lift(height);
        let p = sim.mover.position;
        sim.previous.0 = p;
        sim.current.0 = p;
        let line = format!(
            "connor update {}: test {what}: lifted {height} to z={:.0}{}",
            sim.updates,
            p[2],
            if was == State::Dead { " (was dead: alive again)" } else { "" }
        );
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
}

/// One update; runs in Bevy's fixed-rate schedule (15 per second).
pub fn connor_step(sim: Option<ResMut<ConnorSim>>, lookup: Option<Res<GroundLookup>>, shared: Res<Shared>) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let sim = &mut *sim;
    let intent = sim.intent;
    let step = if sim.mover.state == State::Dead {
        None
    } else {
        let movements = &sim.movements;
        let loops = |name: &str| movements.iter().find(|m| m.name == name).map(|m| m.movement_loop);
        let step = sim.walker.update(intent, &loops);
        sim.phase_step = loops(sim.walker.movement).map_or(0.0, |l| {
            if l.duration > 0.0 { (1.0 / kq8_game::UPDATES_PER_SECOND) as f32 * l.speed / l.duration } else { 0.0 }
        });
        Some(step)
    };
    let wanted = step.as_ref().map_or([0.0; 3], |s| s.wanted);
    let report = sim.mover.update(wanted, &lookup.0);
    sim.updates += 1;
    sim.previous = sim.current;
    sim.current = (sim.mover.position, sim.walker.heading, sim.walker.phase);
    let m = &sim.mover;
    let line = format!(
        "{} {:.3} {:.1} {:.1} {:.1} {} {} {:.2} {} {} {:.1} {} {} {} {} {} {} {:.4} {:.3} {:.1} {:.1}",
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
        intent.forward,
        intent.turn,
        intent.run,
        sim.walker.movement,
        sim.walker.heading,
        sim.walker.phase,
        wanted[0],
        wanted[1],
    );
    sim.log.detail(line);
    let mut events = report.events;
    if let Some(s) = &step {
        if let Some((from, to)) = s.changed {
            events.push(format!("movement {from} -> {to}"));
        }
        if s.missing {
            events.push(format!("movement {} has no loop loaded; not moving", sim.walker.movement));
        }
    }
    let state = sim.mover.state.name();
    if state != sim.last_state || !events.is_empty() {
        let mut main = shared.log.lock().unwrap();
        for e in &events {
            let e = format!("connor update {}: {e}", sim.updates);
            main.detail(&e);
            sim.log.detail(e);
        }
        if state != sim.last_state {
            main.summary(format!("connor update {}: state {} -> {state}", sim.updates, sim.last_state));
        }
        sim.last_state = state;
    }
}

/// Draws Connor between his last two updates (display only): position,
/// heading, and the frame of his current movement.
pub fn connor_draw(
    sim: Option<ResMut<ConnorSim>>,
    fixed: Res<Time<Fixed>>,
    mut body: Query<&mut Transform, (With<ConnorBody>, Without<Camera3d>)>,
    mut visibility: Query<&mut Visibility>,
) {
    let Some(mut sim) = sim else { return };
    let a = fixed.overstep_fraction();
    let (p0, h0, _) = sim.previous;
    let (p1, h1, phase) = sim.current;
    let p: [f32; 3] = std::array::from_fn(|i| p0[i] + (p1[i] - p0[i]) * a);
    let heading = h0 + (h1 - h0) * a;
    let mut placement = sim.placement.clone();
    placement.position = p;
    placement.angles[2] = heading;
    let t = bevy_transform(&placement);
    for mut b in &mut body {
        *b = t;
    }
    // The frame: loop position blended forward from the last update.
    let movement = sim.walker.movement;
    let frame = sim.movements.iter().find(|m| m.name == movement).and_then(|m| {
        let n = m.frames.len();
        (n > 0).then(|| {
            let at = (phase + sim.phase_step * a).rem_euclid(1.0);
            m.frames[((at * n as f32) as usize).min(n - 1)]
        })
    });
    if frame != sim.shown {
        if let Some(old) = sim.shown {
            if let Ok(mut v) = visibility.get_mut(old) {
                *v = Visibility::Hidden;
            }
        }
        if let Some(new) = frame {
            if let Ok(mut v) = visibility.get_mut(new) {
                *v = Visibility::Inherited;
            }
        }
        sim.shown = frame;
    }
}
