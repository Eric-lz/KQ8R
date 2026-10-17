//! Connor in the ground view (M2 steps 2a, 2b, 2c and the falls of 2e,
//! docs/DESIGN.md).
//!
//! - His mover (`kq8_core::connor_mover`, docs/behaviour/connor-physics.md) and his
//!   walking (`kq8_core::connor`, docs/behaviour/connor-movement.md) run at
//!   a fixed 15 updates per second, gravity on.
//! - Keys from `MASK/Connor.sae`; the intent comes from the keys held (the
//!   note's recommendation), so releasing one of two opposite keys gives the
//!   other back.
//! - Every frame of his current movement is drawn in turn; position,
//!   heading, lean and loop position are blended between the last two
//!   updates (display only).
//! - Banking (M2 step 2i, connor-movement.md "Turning" rule 3): the
//!   walker's lean is turned in before his heading, about his forward axis
//!   through his feet (pivot: stand-in, Q56 b). Drawing only.
//!
//! - Falling: `jumphold` while the mover is in Fall, then `jumpend` or
//!   `jumpdie` once (connor-physics.md "Falling off an edge").
//! - Walls: walking head-on stops (`stop`), running head-on plays `crash`
//!   once (connor-physics.md "Walls"; M2 steps 2d-3, 2d-3b, 2d-4).
//!
//! Test keys: G lifts him 5,000 (should die on landing), B lifts him 2,000
//! (safe). `--at X,Y,H` starts him somewhere else. The camera that follows
//! him is in `game_camera.rs`.

use bevy::prelude::*;
use kq8_data::objects::Placement;
use kq8_core::connor::{Intent, Mode, MoveKey, MovementLoop, Walker};
use kq8_core::connor_mover::{Mover, MoverWorld, Scene as MoverScene, State};

use crate::load::ConnorAssets;
use crate::log::Log;
use crate::terrain_view::{bevy_transform, bevy_transform_from, Buildings, GroundLookup, ObjectColliders};
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::{model_mesh, Shared};

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
    jump: Option<KeyCode>,
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
    /// Position, heading, loop position and lean after the last two updates.
    previous: ([f32; 3], f32, f32, f32),
    current: ([f32; 3], f32, f32, f32),
    /// Share of the loop played per update (for blending the loop position).
    phase_step: f32,
    pub updates: u64,
    /// The Jump key was pressed since the last update.
    jump_pressed: bool,
    /// Height where the last jump started (for the log).
    jump_start_z: Option<f32>,
    /// `--turn`: the test's turn key held (1 left, −1 right, 0 none).
    test_turn: i8,
    /// `--hold`'s run switch has been set (once, at the start).
    hold_run_set: bool,
    /// Keyboard auto-repeat: the movement key pressed last, while it is
    /// still held (its key, `None` for the `--turn` test). Each update it
    /// counts as pressed again (user's check in the original, 2026-10-12:
    /// a held turn key keeps clearing the wall blocks).
    repeat: Option<(Option<KeyCode>, MoveKey)>,
    /// One line per update: `kq8-artifacts/logs/connor.log`.
    pub log: Log,
    last_state: &'static str,
    /// What the last down-pass contact was (`terrain`, a building, or
    /// `none`), to log only when it changes.
    last_down: String,
    shown: Option<Entity>,
    /// Sounds his cues started (M3 3h-4), taken by `sound_view`.
    pub sounds: Vec<kq8_core::sound::Start>,
    /// Picks the footstep file (`sound.md` rule 38).
    dice: kq8_core::sound::Dice,
}

/// Each building's `footsoundType` by lower-case name (`sound.md` rule 39).
#[derive(Resource, Default)]
pub struct FootSounds(pub std::collections::BTreeMap<String, Option<i64>>);

impl ConnorSim {
    fn movement_loop(&self, name: &str) -> Option<&MovementLoop> {
        self.movements.iter().find(|m| m.name == name).map(|m| &m.movement_loop)
    }
}

#[derive(Component)]
pub struct ConnorBody;

/// `--lift N`: lift Connor this much once at the start (a test, like G/B).
#[derive(Resource)]
pub struct StartLift(pub Option<f32>);

/// `--hold F,T,R[,N]`: a test that holds this intent (forward, turn, run
/// 0/1) instead of the keys; with N, only for the first N updates, then
/// forward and turn are let go (run stays as it was).
#[derive(Resource)]
pub struct HoldIntent(pub Option<([i8; 3], Option<u64>)>);

/// `--jump N`: a test that presses Jump once, before update N + 1.
#[derive(Resource)]
pub struct JumpAt(pub Option<u64>);

/// `--turn N[,T]`: a test that presses a turn key (T = 1 left, the
/// default, or −1 right) before update N + 1 and holds it from then on
/// (with `--hold`, on top of its intent).
#[derive(Resource)]
pub struct TurnAt(pub Option<(u64, i8)>);

/// `--at X,Y,H`: a test that starts Connor on the ground at X, Y facing
/// heading H (radians) instead of where the world script puts him.
#[derive(Resource)]
pub struct StartAt(pub Option<[f32; 3]>);

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
#[allow(clippy::too_many_arguments)]
pub fn spawn_connor(
    commands: &mut Commands,
    (placement, assets): (Placement, ConnorAssets),
    camera_start: Option<[f32; 3]>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let camera_keys = assets.camera_keys.clone();
    let connor_keys = assets.keys.clone();
    // Each animation's loop data; the movements add speed, flags and
    // transitions.
    let loops: Vec<MovementLoop> = assets
        .animations
        .iter()
        .map(|a| MovementLoop {
            duration: a.duration,
            step_sum: a.step_sum,
            frames: a.frames.len(),
            frame_steps: a.frame_steps.clone(),
            ..Default::default()
        })
        .collect();
    // Each animation: one material per look, one entity per frame.
    let mut frame_entities: Vec<Vec<Entity>> = Vec::new();
    let body = commands.spawn((bevy_transform(&placement), Visibility::default(), ConnorBody)).id();
    // His first animation's first frame, for the light log (M1 5b).
    let mut first_frame_normals: Option<Vec<Vec3>> = None;
    for anim in assets.animations {
        let looks: Vec<(i32, Handle<ObjectMaterial>, crate::load::Shading)> = anim
            .looks
            .into_iter()
            .map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)), l))
            .collect();
        let mut entities = Vec::new();
        for frame in anim.frames {
            let parent = commands.spawn((Transform::default(), Visibility::Hidden)).id();
            commands.entity(body).add_child(parent);
            let record = first_frame_normals.is_none();
            let mut normals = Vec::new();
            for (m, triangles) in frame {
                let Some((_, material, shading)) = looks.iter().find(|(i, _, _)| *i == m) else { continue };
                if record {
                    normals.extend(crate::object_light::smooth_normals(&triangles, shading.lighting));
                }
                let mesh = model_mesh(&triangles, shading.lighting, [0.0; 3]);
                let part = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone()))).id();
                commands.entity(parent).add_child(part);
            }
            entities.push(parent);
            if record {
                first_frame_normals = Some(normals);
            }
        }
        frame_entities.push(entities);
    }
    let rotation = bevy_transform(&placement).rotation;
    match first_frame_normals.as_deref().and_then(|n| crate::object_light::light_stats(light, n, rotation)) {
        Some(stats) => main_log.summary(format!("object light: Connor at his start (first frame): {stats}")),
        None => main_log.summary("object light: Connor has no smooth corners in his first frame"),
    }
    let movements: Vec<DrawnMovement> = assets
        .movements
        .iter()
        .map(|m| {
            DrawnMovement {
                name: m.name,
                movement_loop: MovementLoop {
                    speed: m.speed,
                    repeat: m.repeat,
                    cut_short: m.cut_short,
                    transitions: m.transitions.clone(),
                    cues: m.cues.clone(),
                    ..loops[m.animation].clone()
                },
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
        jump: key("Jump", 0.0),
        held: Vec::new(),
    };
    main_log.summary(format!(
        "connor: keys from Connor.sae: forward {:?}, back {:?}, turn left {:?}, turn right {:?}, run on/off {:?}, jump {:?}",
        controls.forward, controls.back, controls.left, controls.right, controls.run, controls.jump
    ));

    let mut mover = Mover::new(placement.position, true);
    if let Some((min, max)) = assets.collision_box {
        mover.balls = kq8_core::collide::BallStack::from_box(min, max);
    }
    main_log.summary(format!("connor: collision balls radius {} at heights {:?}", mover.balls.radius, mover.balls.heights));
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
        kq8_core::UPDATES_PER_SECOND,
        log.path().display()
    ));
    let heading = placement.angles[2];
    let start = (mover.position, heading, 0.0, 0.0);
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
        jump_pressed: false,
        jump_start_z: None,
        test_turn: 0,
        hold_run_set: false,
        repeat: None,
        log,
        last_state: "normal",
        last_down: "terrain".to_string(),
        shown: None,
        sounds: Vec::new(),
        dice: Default::default(),
    };
    sim.log.detail(
        "update time_s x y z on_ground gravity fall_timer air_updates state drop ground_z ground_normal_z began_on_ground up_contact_nz down contact_nx contact_ny contact_nz push check pull_z forward turn run mode movement heading phase wanted_x wanted_y across hits down_what touch_gap blocked played played_from played_to wanted_movement step_left step_forward above_jump_start submerged surface_z bank",
    );
    sim.log.detail(
        "  (played = the animation that moved him this update, with its phase before and after; wanted_movement = asked for and not reached yet; step_left / step_forward = that animation's step in his own axes; movement / phase = where he is after the update)",
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
#[allow(clippy::too_many_arguments)]
pub fn connor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Option<ResMut<ConnorSim>>,
    shared: Res<Shared>,
    mut start_lift: ResMut<StartLift>,
    mut start_at: ResMut<StartAt>,
    hold: Res<HoldIntent>,
    mut jump_at: ResMut<JumpAt>,
    mut turn_at: ResMut<TurnAt>,
    lookup: Option<Res<GroundLookup>>,
) {
    let Some(mut sim) = sim else { return };
    if let (Some([x, y, heading]), Some(lookup)) = (start_at.0, &lookup) {
        start_at.0 = None;
        let line = match lookup.0.floor_at(x, y) {
            Some(g) => {
                let balls = sim.mover.balls.clone();
                sim.mover = Mover::new([x, y, g.height], sim.mover.use_gravity);
                sim.mover.balls = balls;
                sim.walker = Walker::new(heading);
                let p = sim.mover.position;
                sim.previous = (p, heading, 0.0, 0.0);
                sim.current = sim.previous;
                format!(
                    "connor update {}: test --at: put on the ground at ({x:.0}, {y:.0}, {:.0}), heading {heading:.3}, ground normal z {:.3}",
                    sim.updates, g.height, g.normal[2]
                )
            }
            None => format!("connor update {}: test --at ({x:.0}, {y:.0}): no ground there; not moved", sim.updates),
        };
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
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
    // Movement key events clear wall blocks (connor-physics.md "Walk held
    // against a wall").
    let c = &sim.controls;
    let mut key_events = Vec::new();
    for (key, what) in [(c.forward, MoveKey::Forward), (c.back, MoveKey::Back), (c.left, MoveKey::Turn), (c.right, MoveKey::Turn)] {
        let Some(key) = key else { continue };
        if keys.just_pressed(key) {
            key_events.push((what, true));
        }
        if keys.just_released(key) {
            key_events.push((what, false));
        }
    }
    // Only the key pressed last repeats, as in Windows; letting it go
    // ends the repeat.
    for (key, what) in [(c.forward, MoveKey::Forward), (c.back, MoveKey::Back), (c.left, MoveKey::Turn), (c.right, MoveKey::Turn)] {
        let Some(key) = key else { continue };
        if keys.just_released(key) && sim.repeat.is_some_and(|(k, _)| k == Some(key)) {
            sim.repeat = None;
        }
        if keys.just_pressed(key) {
            sim.repeat = Some((Some(key), what));
        }
    }
    for (what, pressed) in key_events {
        let cleared = sim.walker.key_event(what, pressed);
        if cleared != (false, false) {
            let block = match cleared {
                (true, true) => "forward and back blocks",
                (true, false) => "forward block",
                _ => "back block",
            };
            let line = format!("connor update {}: {what:?} {}: {block} cleared", sim.updates, if pressed { "pressed" } else { "released" });
            shared.log.lock().unwrap().detail(&line);
            sim.log.detail(line);
        }
    }
    let c = &sim.controls;
    let (forward, turn) = (c.axis(c.forward, c.back), c.axis(c.left, c.right));
    let run_pressed = c.run.is_some_and(|k| keys.just_pressed(k));
    if c.jump.is_some_and(|k| keys.just_pressed(k)) {
        sim.jump_pressed = true;
    }
    if jump_at.0.is_some_and(|n| sim.updates >= n) {
        jump_at.0 = None;
        sim.jump_pressed = true;
        let line = format!("connor update {}: test --jump: Jump pressed", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    sim.intent.forward = forward;
    sim.intent.turn = turn;
    if let Some(([f, t, r], until)) = hold.0 {
        let held = until.is_none_or(|n| sim.updates < n);
        (sim.intent.forward, sim.intent.turn) = if held { (f, t) } else { (0, 0) };
        // The run switch only at the start, so scripts can change it
        // (`KQConner::setRun`, M3 step 3d-3).
        if !sim.hold_run_set {
            sim.hold_run_set = true;
            sim.intent.run = r != 0;
        }
    }
    if let Some((_, t)) = turn_at.0.filter(|&(n, _)| sim.updates >= n) {
        turn_at.0 = None;
        sim.test_turn = t;
        sim.repeat = Some((None, MoveKey::Turn));
        let cleared = sim.walker.key_event(MoveKey::Turn, true);
        let side = if t > 0 { "left" } else { "right" };
        let line = format!("connor update {}: test --turn: {side} pressed and held; blocks cleared (forward, back): {cleared:?}", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    if sim.test_turn != 0 {
        sim.intent.turn = sim.test_turn;
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
        sim.walker.set_mode(Mode::Free);
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

impl ConnorSim {
    /// `KQConner::setRun` from a script (M3 step 3d-3).
    pub fn set_run(&mut self, run: bool, at: &str) {
        let was = self.intent.run;
        self.intent.run = run;
        self.log.detail(format!(
            "connor update {}: run switch {} -> {} ({at})",
            self.updates,
            if was { "on" } else { "off" },
            if run { "on" } else { "off" }
        ));
    }
}

/// One update; runs in Bevy's fixed-rate schedule (15 per second).
pub fn connor_step(
    sim: Option<ResMut<ConnorSim>>,
    lookup: Option<Res<GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    objects: Option<Res<ObjectColliders>>,
    footsounds: Option<Res<FootSounds>>,
    shared: Res<Shared>,
) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let sim = &mut *sim;
    let scene = MoverScene::new(
        &lookup.0,
        buildings.as_ref().map_or(&[][..], |b| &b.0),
        objects.as_ref().map_or(&[][..], |o| &o.0),
    );
    if let Some((_, what)) = sim.repeat {
        let cleared = sim.walker.key_event(what, true);
        if cleared != (false, false) {
            let line = format!("connor update {}: {what:?} held (repeat): blocks cleared (forward, back): {cleared:?}", sim.updates);
            shared.log.lock().unwrap().detail(&line);
            sim.log.detail(line);
        }
    }
    let intent = sim.intent;
    let jump = std::mem::take(&mut sim.jump_pressed);
    let z_before = sim.mover.position[2];
    let tick = {
        let ConnorSim { mover, walker, movements, .. } = &mut *sim;
        let loops = |name: &str| movements.iter().find(|m| m.name == name).map(|m| &m.movement_loop);
        kq8_core::connor_update::tick(mover, walker, intent, jump, &loops, &scene)
    };
    if sim.walker.mode == Mode::Jump && sim.jump_start_z.is_none() {
        sim.jump_start_z = Some(z_before);
    }
    let kq8_core::connor_update::Tick { step, report, wall_stop, notes: jump_notes } = tick;
    sim.phase_step = sim.movement_loop(sim.walker.movement).map_or(0.0, |l| {
        let time = l.play_time();
        if time > 0.0 && sim.walker.mode != Mode::Dead { (1.0 / kq8_core::UPDATES_PER_SECOND) as f32 * l.speed / time } else { 0.0 }
    });
    let wanted = step.as_ref().map_or([0.0; 3], |s| s.wanted);
    sim.updates += 1;
    sim.previous = sim.current;
    let bank_before = sim.current.3;
    sim.current = (sim.mover.position, sim.walker.heading, sim.walker.phase, sim.walker.bank);
    let m = &sim.mover;
    let down_hit = report.hits.iter().find(|h| h.pass == "down");
    // The drawn ground (the water's surface where there is water).
    let surface_z = scene.water(m.position[0], m.position[1]).map_or("none".into(), |(s, _)| format!("{s:.1}"));
    let normal = |n: Option<[f32; 3]>, i: usize| n.map_or("none".into(), |n| format!("{:.3}", n[i]));
    let line = format!(
        "{} {:.3} {:.1} {:.1} {:.1} {} {} {:.2} {} {} {:.1} {} {} {} {} {} {} {} {} {:.1} {} {:.3} {} {} {} {} {} {:.4} {:.3} {:.1} {:.1} {} {} {} {} {} {} {} {} {} {:.2} {:.2} {} {} {} {:.3}",
        sim.updates,
        sim.updates as f64 / kq8_core::UPDATES_PER_SECOND,
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
        report.began_on_ground,
        normal(report.up_contact, 2),
        report.down.name(),
        normal(report.down_normal, 0),
        normal(report.down_normal, 1),
        normal(report.down_normal, 2),
        report.push,
        report.check,
        m.pull[2],
        intent.forward,
        intent.turn,
        intent.run,
        sim.walker.mode.name(),
        sim.walker.movement,
        sim.walker.heading,
        sim.walker.phase,
        wanted[0],
        wanted[1],
        report.across,
        report.hits.len(),
        down_hit.map_or("none".into(), |h| h.hit.what.replace(' ', "_")),
        down_hit.map_or("none".into(), |h| format!("{:.1}", h.hit.position[2] - h.hit.touch[2])),
        sim.walker.blocked_name(),
        step.as_ref().map_or("none", |s| s.played),
        step.as_ref().map_or("none".into(), |s| format!("{:.4}", s.played_from)),
        step.as_ref().map_or("none".into(), |s| format!("{:.4}", s.played_to)),
        sim.walker.wanted.unwrap_or("none"),
        step.as_ref().map_or(0.0, |s| s.own_step[0]),
        step.as_ref().map_or(0.0, |s| -s.own_step[1]),
        sim.jump_start_z.map_or("none".into(), |z| format!("{:.1}", sim.mover.position[2] - z)),
        m.submerged,
        surface_z,
        sim.walker.bank,
    );
    if sim.walker.mode == Mode::Free {
        sim.jump_start_z = None;
    }
    sim.log.detail(line);
    let mut events = jump_notes;
    events.extend(report.events.iter().cloned());
    if report.up_contact.is_some() {
        events.push(format!("up pass hit the terrain (normal z {})", normal(report.up_contact, 2)));
    }
    if let Some(s) = &step {
        if let Some((from, to)) = s.mode_changed {
            events.push(format!("animation mode {} -> {}", from.name(), to.name()));
            if to == Mode::Dead {
                events.push("jumpdie finished: Die state".into());
            }
        }
        events.extend(s.notes.iter().cloned());
        if let Some((from, to)) = s.changed {
            events.push(format!("movement {from} -> {to}"));
        }
        if s.missing {
            events.push(format!("movement {} has no loop loaded; not moving", sim.walker.movement));
        }
    }
    let bank = sim.walker.bank;
    if bank_before == 0.0 && bank != 0.0 {
        events.push(format!("lean starts: {bank:.3} rad ({})", if bank > 0.0 { "left" } else { "right" }));
    } else if bank_before != 0.0 && bank == 0.0 {
        events.push(format!("lean back upright (was {bank_before:.3} rad)"));
    }
    let down_now = down_hit.map_or("none".to_string(), |h| h.hit.what.clone());
    if down_now != sim.last_down && down_now != "none" {
        events.push(format!("standing on {down_now} (was {})", sim.last_down));
    }
    if down_now != "none" {
        sim.last_down = down_now;
    }
    for h in &report.hits {
        // Floor landings are in the columns; log the other building and
        // object hits.
        if h.hit.what != "terrain" && h.result != "landing" {
            events.push(format!(
                "{} pass hit {} ({}): normal ({:.3}, {:.3}, {:.3}), angle {}, put at ({:.1}, {:.1}, {:.1}), touched at ({:.1}, {:.1}, {:.1})",
                h.pass,
                h.hit.what,
                h.result,
                h.hit.normal[0],
                h.hit.normal[1],
                h.hit.normal[2],
                h.angle.map_or("none".into(), |a| format!("{:.1} deg", a.to_degrees())),
                h.hit.position[0],
                h.hit.position[1],
                h.hit.position[2],
                h.hit.touch[0],
                h.hit.touch[1],
                h.hit.touch[2]
            ));
        }
    }
    if let Some((from, to)) = wall_stop {
        events.push(if to == "crash" {
            format!("head-on into a wall: {from} -> {to} from the next update (keys ignored until it ends)")
        } else {
            format!("head-on into a wall: movement {from} -> {to}; blocked: {}", sim.walker.blocked_name())
        });
    }
    // M3 3h-4: the cues his movement passed this update (doors.md rule 9,
    // sound.md rules 34-39).
    if let Some(s) = &step {
        let fired: Vec<String> =
            sim.movement_loop(s.played).map(|l| l.cues_between(s.played_from, s.played_to).into_iter().map(String::from).collect()).unwrap_or_default();
        for cue in fired {
            let mut words = cue.split_whitespace();
            let action = words.next().unwrap_or("").to_ascii_lowercase();
            let what = match action.as_str() {
                "footstep" => footstep(sim, &lookup.0, footsounds.as_deref()),
                "playsound" => match words.next() {
                    Some(name) => {
                        let at = sim.mover.position;
                        sim.sounds.push(kq8_core::sound::Start::placed_one_shot(name, at, format!("Connor {} cue", s.played)));
                        "sound started".to_string()
                    }
                    None => "playsound without a file name; nothing".to_string(),
                },
                _ => "cue action not done for Connor".to_string(),
            };
            events.push(format!("cue '{cue}' of {} fires (phase {:.4} -> {:.4}): {what}", s.played, s.played_from, s.played_to));
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

/// A `FootStep` cue (`sound.md` rules 36-39): the ground he stands on (this
/// update's down contact, else the last one: Q81 d) gives the flags, the
/// flags the set, a roll the file; a plain one-shot. Returns the log text.
fn footstep(sim: &mut ConnorSim, terrain: &kq8_data::Terrain, footsounds: Option<&FootSounds>) -> String {
    use kq8_core::sound::{building_floor_flags, footstep_file, footstep_set, Start};
    let [x, y, _] = sim.mover.position;
    let on = sim.last_down.clone();
    let (flags, from) = if on == "terrain" {
        (terrain.flags_at(x, y), "its square's envInfo.kq flags".to_string())
    } else if let Some(t) = footsounds.and_then(|f| f.0.get(&on.to_ascii_lowercase())) {
        let from = match t {
            Some(n) => format!("building footsoundType={n}"),
            None => "building without footsoundType: 0x1 (stand-in, Q81 a)".to_string(),
        };
        (building_floor_flags(*t), from)
    } else {
        (terrain.flags_at(x, y), "an object: the terrain's flags under him (stand-in, Q81 b)".to_string())
    };
    let Some(set) = footstep_set(flags) else {
        return format!("footstep on {on} ({from}), flags {flags:#x}: no step sound (rule 37)");
    };
    let roll = sim.dice.roll4();
    let file = footstep_file(set, roll);
    sim.sounds.push(Start::plain_one_shot(&file, format!("Connor's footstep on {on}, flags {flags:#x} -> {set}, roll {roll}")));
    format!("footstep on {on} ({from}), flags {flags:#x} -> {set}, roll {roll} -> {file}")
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
    let (p0, h0, _, b0) = sim.previous;
    let (p1, h1, phase, b1) = sim.current;
    let p: [f32; 3] = std::array::from_fn(|i| p0[i] + (p1[i] - p0[i]) * a);
    let heading = h0 + (h1 - h0) * a;
    let bank = b0 + (b1 - b0) * a;
    let mut placement = sim.placement.clone();
    placement.position = p;
    placement.angles[2] = heading;
    // The lean first, about the model's forward axis (−Y; a turn about Y,
    // positive tips his head towards +X, his left), then his placement.
    let lean = kq8_data::objects::rotation([0.0, bank, 0.0]);
    let r = placement.transform().m;
    let m: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| lean[i][k] * r[k][j]).sum()));
    let t = bevy_transform_from(m, p);
    for mut b in &mut body {
        *b = t;
    }
    // The frame: loop position blended forward from the last update.
    let movement = sim.walker.movement;
    let frame = sim.movements.iter().find(|m| m.name == movement).and_then(|m| {
        let n = m.frames.len();
        (n > 0).then(|| m.frames[m.movement_loop.frame_at(phase + sim.phase_step * a).min(n - 1)])
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
