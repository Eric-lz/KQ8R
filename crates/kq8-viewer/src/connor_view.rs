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
    /// His mouth (M4 4a-6, `talking.md` rule 18): the materials that show
    /// the face texture `baseLip`, and the image of each mouth shape
    /// 0-7 (`None`: not found, the face stays).
    pub face: Vec<Handle<ObjectMaterial>>,
    pub lip_images: Vec<Option<Handle<Image>>>,
    /// The mouth shape shown now (0 = rest).
    pub mouth: u16,
    /// Presses that cut his line short (`talking.md` rule 16, M4 4a-5):
    /// forward, back, a turn key or Jump, not during `handsOff`; taken
    /// by the door simulation.
    pub cut_presses: Vec<String>,
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
    /// The turn keys as events (M4 4a-7, `kq8_core::turn_keys`): the
    /// turn direction changes only on a key going down or up.
    pub turn_keys: kq8_core::turn_keys::TurnKeys,
    /// `--hold`'s turn key has been pressed (and not yet let go).
    hold_turn_down: bool,
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
    /// The frame entity shown now (its children are the model parts; the
    /// shadow reads them, M4 4d-7b-2).
    pub(crate) shown: Option<Entity>,
    /// Sounds his cues started (M3 3h-4), taken by `sound_view`.
    pub sounds: Vec<kq8_core::sound::Start>,
    /// Picks the footstep file (`sound.md` rule 38).
    dice: kq8_core::sound::Dice,
    /// The building his steps sound like (`sound.md` rule 39a).
    step_ground: kq8_core::sound::StepGround,
    /// His play-once movements that ended (plain names), for `getEndLoop`
    /// listeners; taken by `door_view::trap_step` (M3 3i-7).
    pub ended: Vec<String>,
    /// Physical pushables he moved this update (`pushing.md` rule 4):
    /// name and movement; applied by [`push_apply`] (M3 3j-2).
    pub pushed: Vec<(String, [f32; 3])>,
    /// Scripted pushables he pushed this update, whose scripts get `Push`
    /// (`door_view::trap_step`, M3 3j-3).
    pub push_calls: Vec<String>,
    /// Scripted pushables whose script didn't take control: walls for him
    /// from then on (`pushing.md` "Do not reproduce").
    pub not_pushable: Vec<String>,
    /// `handsOff 1` (M3 3k-3, Q89 a stand-in): movement keys and Jump
    /// count as let go.
    pub hands_off: bool,
    /// Scripted turns and walks that ended (M3 3l-2), whose cues
    /// `door_view::trap_step` sends.
    pub auto_ends: Vec<kq8_core::connor_auto::AutoEnd>,
    /// State requests a jump refused (M3 3m-1, `anim-scripts.md` rule
    /// 28): tried again at each later update by `door_view::trap_step`
    /// (STAND-IN: when the original tries again was not traced).
    pub refused: Vec<kq8_core::active::Request>,
    /// The end of a scripted walk being watched (M3 3n-5, Q98: logs only).
    walk_end: Option<WalkEnd>,
}

/// Q98 (M3 3n-5, logs only): from the update a scripted walk ends until
/// its stop transition has ended, what he did.
struct WalkEnd {
    update: u64,
    at: [f32; 3],
    heading: f32,
    /// The stop transition seen playing, and the updates it played in.
    via: Option<&'static str>,
    first: u64,
    last: u64,
    /// Flat distance along his path since the walk ended, and while the
    /// transition played.
    path: f32,
    via_path: f32,
    via_from: [f32; 3],
    previous: [f32; 3],
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

/// `--jump N` (repeatable): a test that presses Jump once, before update
/// N + 1, for each N given.
#[derive(Resource)]
pub struct JumpAt(pub Vec<u64>);

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
    (placement, mut assets): (Placement, ConnorAssets),
    camera_start: Option<[f32; 3]>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let camera_keys = assets.camera_keys.clone();
    let connor_keys = assets.keys.clone();
    // His mouth (M4 4a-6): the face materials are found below.
    let lip_base = assets.lips.as_ref().map(|(b, _)| b.clone());
    let lip_images: Vec<Option<Handle<Image>>> = assets
        .lips
        .take()
        .map(|(_, looks)| {
            looks
                .into_iter()
                .map(|l| match l {
                    Some(crate::load::Look::Texture { width, height, rgba, .. }) => {
                        Some(images.add(crate::object_light::texture_image(width, height, rgba, bevy::image::ImageAddressMode::Repeat)))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let mut face = Vec::new();
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
            .map(|(m, look, l)| {
                // The look's name is where it was found (`vol:…/CF0.BMP`).
                let file = |name: &str| name.rsplit(['/', '\\']).next().unwrap_or(name).to_string();
                let is_face = matches!(&look, crate::load::Look::Texture { name, .. } if lip_base.as_ref().is_some_and(|b| b.eq_ignore_ascii_case(&file(name))));
                let handle = materials.add(object_material(look, l, light, images));
                if is_face {
                    face.push(handle.clone());
                }
                (m, handle, l)
            })
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
        cut_presses: Vec::new(),
        face,
        lip_images,
        mouth: 0,
        jump_start_z: None,
        turn_keys: Default::default(),
        hold_turn_down: false,
        hold_run_set: false,
        repeat: None,
        log,
        last_state: "normal",
        last_down: "terrain".to_string(),
        shown: None,
        sounds: Vec::new(),
        dice: Default::default(),
        step_ground: Default::default(),
        ended: Vec::new(),
        pushed: Vec::new(),
        push_calls: Vec::new(),
        not_pushable: Vec::new(),
        hands_off: false,
        auto_ends: Vec::new(),
        refused: Vec::new(),
        walk_end: None,
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
        // handsOff (anim-scripts.md rule 21): presses are swallowed,
        // releases still count.
        if pressed && sim.hands_off {
            continue;
        }
        if pressed {
            sim.cut_presses.push(format!("{what:?} pressed"));
        }
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
    // The turn keys as events (connor-movement.md "Turning" rule 2, Q104
    // a): releases first, then presses.
    let c = &sim.controls;
    let mut turn_events = Vec::new();
    for (key, dir) in [(c.left, 1i8), (c.right, -1i8)] {
        if key.is_some_and(|k| keys.just_released(k)) {
            turn_events.push((dir, false));
        }
    }
    for (key, dir) in [(c.left, 1i8), (c.right, -1i8)] {
        if key.is_some_and(|k| keys.just_pressed(k)) {
            turn_events.push((dir, true));
        }
    }
    for (dir, down) in turn_events {
        let what = if down { "key" } else { "key released" };
        turn_key_event(&mut sim, dir, down, what, &shared);
    }
    let c = &sim.controls;
    let forward = c.axis(c.forward, c.back);
    let run_pressed = c.run.is_some_and(|k| keys.just_pressed(k));
    if c.jump.is_some_and(|k| keys.just_pressed(k)) {
        sim.jump_pressed = true;
        if !sim.hands_off {
            sim.cut_presses.push("Jump pressed".into());
        }
    }
    let due = jump_at.0.len();
    jump_at.0.retain(|n| sim.updates < *n);
    if jump_at.0.len() < due {
        sim.jump_pressed = true;
        if !sim.hands_off {
            sim.cut_presses.push("test --jump: Jump pressed".into());
        }
        let line = format!("connor update {}: test --jump: Jump pressed", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    sim.intent.forward = forward;
    if let Some(([f, t, r], until)) = hold.0 {
        let held = until.is_none_or(|n| sim.updates < n);
        sim.intent.forward = if held { f } else { 0 };
        // The held turn: pressed at the start, let go at the end (events).
        if t != 0 && held != sim.hold_turn_down {
            sim.hold_turn_down = held;
            turn_key_event(&mut sim, t, held, "test --hold", &shared);
        }
        // The run switch only at the start, so scripts can change it
        // (`KQConner::setRun`, M3 step 3d-3).
        if !sim.hold_run_set {
            sim.hold_run_set = true;
            sim.intent.run = r != 0;
        }
    }
    if let Some((_, t)) = turn_at.0.filter(|&(n, _)| sim.updates >= n) {
        turn_at.0 = None;
        turn_key_event(&mut sim, t, true, "test --turn (pressed and held)", &shared);
        sim.repeat = Some((None, MoveKey::Turn));
        if !sim.hands_off {
            sim.cut_presses.push("test --turn: turn pressed".into());
        }
        let cleared = sim.walker.key_event(MoveKey::Turn, true);
        let side = if t > 0 { "left" } else { "right" };
        let line = format!("connor update {}: test --turn: {side} pressed and held; blocks cleared (forward, back): {cleared:?}", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    sim.intent.turn = sim.turn_keys.turn;
    if run_pressed && sim.hands_off {
        let line = format!("connor update {}: run key pressed: ignored (handsOff)", sim.updates);
        sim.log.detail(line);
    } else if run_pressed {
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

/// One turn key event (+1 left, −1 right; `down` or up) reaching Connor
/// (`connor-movement.md` "Turning" rule 2, Q104 a): sets the turn
/// direction; a down that reaches him (not swallowed by `handsOff`)
/// cancels an automatic turn at his next update. Logged in `connor.log`,
/// with "auto-turn cancelled" only on a down during an automatic turn.
fn turn_key_event(sim: &mut ConnorSim, dir: i8, down: bool, what: &str, shared: &Shared) {
    let hands_off = sim.hands_off;
    let e = if down { sim.turn_keys.press(dir, hands_off) } else { sim.turn_keys.release(dir, hands_off) };
    let turning = sim.walker.auto.as_ref().is_some_and(|a| a.phase == kq8_core::connor_auto::AutoPhase::Turning);
    if e.is_down() {
        sim.walker.turn_press = true;
    }
    let cancel = if e.is_down() && turning { "; auto-turn cancelled (at the next update)" } else if turning { "; the automatic turn goes on" } else { "" };
    sim.intent.turn = sim.turn_keys.turn;
    let line = format!(
        "connor update {}: turn key {} ({what}): {e}; turn direction {:+}{cancel}",
        sim.updates,
        if dir > 0 { "left" } else { "right" },
        sim.turn_keys.turn
    );
    shared.log.lock().unwrap().detail(&line);
    sim.log.detail(line);
}

impl ConnorSim {
    /// He can turn to face a listener (`talking.md` rule 7): in Wait,
    /// Move or the talk state.
    pub fn can_turn_to_listener(&self) -> bool {
        matches!(self.walker.mode, Mode::Free | Mode::Talk)
    }

    /// His state for the turn after a click (`clicking.md` rule 28): Wait
    /// or Move turn (talk, weapon draw, throwing and the tile puzzle are
    /// not built); anything else doesn't. The state's name, for the log.
    pub fn click_turn_state(&self) -> (bool, &'static str) {
        match self.walker.mode {
            Mode::Free if self.walker.moving() => (true, "Move"),
            Mode::Free => (true, "Wait"),
            // Rule 28: the talk state turns too (M4 4e-3).
            Mode::Talk => (true, "Talk"),
            m => (false, m.name()),
        }
    }

    /// The jump he is in (`jump`, `runJump`, `backJump`) for the scripts'
    /// `KQConner::JumpDir` / `endJump` (M3 3m-1), `None` outside the Jump
    /// state.
    pub fn jump_state(&self) -> Option<&'static str> {
        (self.walker.mode == Mode::Jump).then(|| self.walker.kind.jump())
    }

    /// `KQConner::endJump` (`anim-scripts.md` rule 28): the jump finished,
    /// no state change. Returns a line for the script logs.
    pub fn end_jump(&mut self, at: &str) -> String {
        let line = kq8_core::connor_update::finish_jump(&mut self.mover, &mut self.walker)
            .unwrap_or_else(|| "not in the Jump state any more; nothing".into());
        self.log.detail(format!("connor update {}: endJump: {line} ({at})", self.updates));
        format!("endJump: {line}")
    }

    /// A script asks for another state (`anim-scripts.md` rule 28): during
    /// an unfinished jump, with `handsOff` on the jump is finished first;
    /// without it the request is refused (`false`). Logged either way.
    pub fn state_request(&mut self, what: &str, at: &str) -> bool {
        match kq8_core::connor_update::state_request(&mut self.mover, &mut self.walker, self.hands_off) {
            Ok(None) => true,
            Ok(Some(note)) => {
                self.log.detail(format!("connor update {}: {what}: {note} ({at})", self.updates));
                true
            }
            Err(why) => {
                self.log.detail(format!("connor update {}: {what}: refused, {why}; tried again next update (stand-in) ({at})", self.updates));
                false
            }
        }
    }

    /// `KQMonster::setState Connor special <items>` (`anim-scripts.md`
    /// rule 16): the items worked out against the movements loaded (his
    /// own and the Special lists'), then the Special state. Returns a
    /// line for the script logs.
    pub fn start_special(&mut self, items: &str, at: &str) -> String {
        let movements = &self.movements;
        let resolve = |list: Option<&str>, word: &str| -> Option<&'static str> {
            let wanted = match list {
                Some(l) => kq8_core::connor::special_movement_name(l, word),
                None => word.to_string(),
            };
            movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
        };
        let steps = kq8_core::connor::special_steps(items, &resolve);
        let plays: Vec<&str> = steps.iter().filter_map(|s| if let kq8_core::connor::SpecialStep::Play(n, _) = s { Some(*n) } else { None }).collect();
        let line = format!("special {items}: plays {plays:?}");
        let notes = self.walker.start_special(steps);
        self.log.detail(format!("connor update {}: KQMonster::setState Connor special {items} ({at}): {}", self.updates, notes.join("; ")));
        line
    }

    /// `move Connor <flags> x y z rx ry rz` from a script (M3 3k-1,
    /// `objects.md` "(d)"): his position set or added, his heading (rz)
    /// set or added; no collision passes; the mover's state is kept. Then
    /// the move step's own vertical bookkeeping (one gravity step, the
    /// terrain raise; refined Q92), and when the script ran **inside his
    /// update** (a `Push` call, `pushing.md` rule 11) the rest of that
    /// update: the down pass and the ground check (M3 3l-1). Returns a
    /// line for the script logs.
    #[allow(clippy::too_many_arguments)]
    pub fn move_to(&mut self, flags: u32, position: [f32; 3], turn: [f32; 3], at: &str, world: &dyn MoverWorld, inside: bool) -> String {
        let old = (self.mover.position, self.walker.heading);
        self.mover.position = if flags & 1 != 0 { std::array::from_fn(|k| old.0[k] + position[k]) } else { position };
        self.walker.heading = if flags & 2 != 0 { old.1 + turn[2] } else { turn[2] };
        let tilt = if turn[0] != 0.0 || turn[1] != 0.0 { " (his turn about x and y ignored: he only turns about z)" } else { "" };
        let set = self.mover.position;
        let drop = self.mover.after_move(world);
        let mut rest = format!("; the move step: gravity step {drop:.1} -> z {:.1}", self.mover.position[2]);
        if inside {
            let r = self.mover.rest_of_update(world);
            let on = r.hits.iter().find(|h| h.pass == "down").map_or(r.check.to_string(), |h| format!("{} ({})", h.hit.what, h.result));
            rest += &format!(
                "; inside his update (a trap touch or a Push call): the rest of it: drop {:.1}, step-down 325, down pass on {on} -> z {:.1}, on ground {}",
                r.drop, self.mover.position[2], self.mover.on_ground
            );
        } else {
            rest += "; outside his update: he falls from there";
        }
        // No blending from the old place in the drawing.
        self.current = (self.mover.position, self.walker.heading, self.walker.phase, self.walker.bank);
        self.previous = self.current;
        let line = format!(
            "moved Connor from ({:.0}, {:.0}, {:.0}) heading {:.3} to ({:.0}, {:.0}, {:.0}) heading {:.3}{tilt}{rest}",
            old.0[0], old.0[1], old.0[2], old.1, set[0], set[1], set[2], self.walker.heading
        );
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// `handsOff 0` (`anim-scripts.md` rule 22): every movement key still
    /// held is sent to him again as a press (which clears wall blocks as a
    /// press does), so it acts at once. Returns a line for the log.
    pub fn keys_sent_again(&mut self) -> String {
        let mut sent = Vec::new();
        let keys = [(self.intent.forward == 1, MoveKey::Forward), (self.intent.forward == -1, MoveKey::Back)];
        for (held, key) in keys {
            if held {
                let cleared = self.walker.key_event(key, true);
                sent.push(format!("{key:?} (blocks cleared {cleared:?})"));
            }
        }
        // The held turn key goes down again (Q104 a: a down event).
        if let Some(dir) = self.turn_keys.held_last() {
            let e = self.turn_keys.press(dir, false);
            self.walker.turn_press = true;
            self.intent.turn = self.turn_keys.turn;
            let cleared = self.walker.key_event(MoveKey::Turn, true);
            sent.push(format!("Turn: {e} (blocks cleared {cleared:?})"));
        }
        if sent.is_empty() { "no movement key held".into() } else { format!("held keys sent again: {}", sent.join(", ")) }
    }

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

/// Moves the physical pushables Connor pushed this update (M3 3j-2,
/// `pushing.md` rule 4): their colliders (his and the camera's) and their
/// drawn entities. Runs right after `connor_step`.
pub fn push_apply(
    sim: Option<ResMut<ConnorSim>>,
    colliders: Option<ResMut<ObjectColliders>>,
    view: Option<ResMut<crate::terrain_view::ViewBlockerSet>>,
    mut drawn: Query<(&crate::terrain_view::ObjectName, &mut Transform)>,
    shared: Res<Shared>,
) {
    let Some(mut sim) = sim else { return };
    let (mut colliders, mut view) = (colliders, view);
    for (name, d) in std::mem::take(&mut sim.pushed) {
        let mut boxes = 0;
        let mut at = None;
        for list in [colliders.as_mut().map(|c| &mut c.0), view.as_mut().map(|v| &mut v.boxes)].into_iter().flatten() {
            for b in list.iter_mut().filter(|b| b.name == name) {
                for (p, v) in b.placement.position.iter_mut().zip(d) {
                    *p += v;
                }
                at = Some(b.placement.position);
                boxes += 1;
            }
        }
        let shift = Vec3::from(crate::to_bevy(d, [0.0; 3]));
        let mut entities = 0;
        for (_, mut t) in drawn.iter_mut().filter(|(n, _)| n.0 == name) {
            t.translation += shift;
            entities += 1;
        }
        let line = format!(
            "connor update {}: pushed {name} by ({:.1}, {:.1}, {:.1}): {boxes} collider(s), {entities} drawn part(s) moved; now at {}",
            sim.updates,
            d[0],
            d[1],
            d[2],
            at.map_or("?".into(), |p| format!("({:.1}, {:.1}, {:.1})", p[0], p[1], p[2]))
        );
        if sim.updates % 15 == 0 {
            shared.log.lock().unwrap().detail(&line);
        }
        sim.log.detail(line);
    }
}

/// One update; runs in Bevy's fixed-rate schedule (15 per second).
/// Q98 (`anim-scripts.md` rules 24–25, open question 6; logs only): in
/// the update a scripted walk ends, the walk's phase, its stop transitions
/// and which comes next from that phase; then, once the stop transition
/// has played, the updates it took and how far it carried him.
fn watch_walk_end(sim: &mut ConnorSim, ended: bool) {
    let p = sim.mover.position;
    // This update's number (`sim.updates` grows later in `connor_step`).
    let n = sim.updates + 1;
    if ended {
        let movement = sim.walker.movement;
        let phase = sim.walker.phase;
        let lp = sim.movements.iter().find(|m| m.name == movement).map(|m| &m.movement_loop);
        let stops: Vec<String> = lp.map_or(Vec::new(), |l| l.transitions.iter().filter(|t| t.to == "stop").map(|t| format!("{} from {:.2}", t.via, t.from)).collect());
        let next = lp.and_then(|l| l.choose_transition(phase, "stop")).map_or("none".to_string(), |(t, d)| format!("{} from {:.2} ({d:.4} of a loop on)", t.via, t.from));
        sim.log.detail(format!(
            "connor update {n}: Q98: the scripted walk ends at ({:.1}, {:.1}), heading {:.3}: {movement} at phase {phase:.4} after this update's move; {movement}'s stop transitions: [{}]; the next from this phase: {next}",
            p[0],
            p[1],
            sim.walker.heading,
            stops.join(", ")
        ));
        sim.walk_end = Some(WalkEnd { update: n, at: p, heading: sim.walker.heading, via: None, first: 0, last: 0, path: 0.0, via_path: 0.0, via_from: p, previous: p });
        return;
    }
    let Some(w) = sim.walk_end.as_mut() else { return };
    let moved = ((p[0] - w.previous[0]).powi(2) + (p[1] - w.previous[1]).powi(2)).sqrt();
    w.path += moved;
    let playing = sim.walker.playing.map(|t| t.via);
    match (w.via, playing) {
        (None, Some(via)) => {
            // It started this update; its share of this update's move is
            // counted with it (the walk played on to the from point first).
            w.via = Some(via);
            w.first = n;
            w.last = n;
            w.via_from = w.previous;
            w.via_path = moved;
        }
        (Some(v), Some(via)) if v == via => {
            w.last = n;
            w.via_path += moved;
        }
        _ => {}
    }
    w.previous = p;
    let done = w.via.is_some() && playing.is_none();
    if done || n > w.update + 60 {
        let w = sim.walk_end.take().expect("watched");
        let straight = ((p[0] - w.at[0]).powi(2) + (p[1] - w.at[1]).powi(2)).sqrt();
        // The transition's own steps in the data (its frames added up, flat).
        let own = |via: &str| {
            sim.movements.iter().find(|m| m.name == via).map_or("unknown".to_string(), |m| {
                let l = &m.movement_loop;
                let sum = if l.frame_steps.is_empty() { l.step_sum } else { l.frame_steps.iter().fold([0.0; 3], |a, f| [a[0] + f[0], a[1] + f[1], a[2] + f[2]]) };
                format!("{:.1} (left {:.1}, forward {:.1}) over {:.3} s", (sum[0] * sum[0] + sum[1] * sum[1]).sqrt(), sum[0], -sum[1], l.duration)
            })
        };
        let line = match w.via {
            Some(via) => format!(
                "connor update {n}: Q98: {via} (its own steps in the data: {}) played in updates {}-{} ({} updates; the update it started in also played the walk up to its from point): {:.1} along his path from ({:.1}, {:.1}); since the walk ended (update {}): {:.1} along his path, {straight:.1} straight, now at ({:.1}, {:.1}), heading {:.3} -> {:.3}",
                own(via),
                w.first,
                w.last,
                w.last - w.first + 1,
                w.via_path,
                w.via_from[0],
                w.via_from[1],
                w.update,
                w.path,
                p[0],
                p[1],
                w.heading,
                sim.walker.heading
            ),
            None => format!("connor update {n}: Q98: no stop transition seen in 60 updates after the walk ended (update {}); now {}", w.update, sim.walker.movement),
        };
        sim.log.detail(line);
    }
}

pub fn connor_step(
    sim: Option<ResMut<ConnorSim>>,
    lookup: Option<Res<GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    objects: Option<Res<ObjectColliders>>,
    order: Option<Res<crate::terrain_view::ColliderOrder>>,
    footsounds: Option<Res<FootSounds>>,
    shared: Res<Shared>,
) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let sim = &mut *sim;
    let mut scene = MoverScene::new(
        &lookup.0,
        buildings.as_ref().map_or(&[][..], |b| &b.0),
        objects.as_ref().map_or(&[][..], |o| &o.0),
    );
    scene.not_pushable = sim.not_pushable.clone();
    scene.order = order.as_ref().map(|o| &o.0[..]);
    if let Some((_, what)) = sim.repeat.filter(|_| !sim.hands_off) {
        let cleared = sim.walker.key_event(what, true);
        if cleared != (false, false) {
            let line = format!("connor update {}: {what:?} held (repeat): blocks cleared (forward, back): {cleared:?}", sim.updates);
            shared.log.lock().unwrap().detail(&line);
            sim.log.detail(line);
        }
    }
    // handsOff (Q89 a stand-in): the keys count as let go (the run
    // switch stays as it is).
    let intent = if sim.hands_off { Intent { forward: 0, turn: 0, ..sim.intent } } else { sim.intent };
    let jump = std::mem::take(&mut sim.jump_pressed) && !sim.hands_off;
    let z_before = sim.mover.position[2];
    let tick = {
        let ConnorSim { mover, walker, movements, .. } = &mut *sim;
        let loops = |name: &str| movements.iter().find(|m| m.name == name).map(|m| &m.movement_loop);
        kq8_core::connor_update::tick(mover, walker, intent, jump, &loops, &scene)
    };
    if sim.walker.mode == Mode::Jump && sim.jump_start_z.is_none() {
        sim.jump_start_z = Some(z_before);
    }
    let kq8_core::connor_update::Tick { step, report, wall_stop, notes: jump_notes, auto_end, auto_turn_ended } = tick;
    // Q104 a: the end of an automatic turn sets the turn direction to 0;
    // a held turn key is not sent again.
    if auto_turn_ended {
        sim.turn_keys.auto_turn_ended();
        sim.intent.turn = 0;
    }
    // M3 3n-5 (Q98): the end of a scripted walk, logged.
    let walk_ended = auto_end.as_ref().is_some_and(|a| a.why.starts_with("walked") || a.why == "head-on");
    watch_walk_end(sim, walk_ended);
    // M3 3l-2: a scripted turn or walk ended: its cue.
    sim.auto_ends.extend(auto_end);
    // M3 3j-2: physical pushables moved by his move.
    sim.pushed.extend(scene.moved.take());
    // M3 3j-3: scripted pushables pushed: their scripts get Push.
    for t in report.pushes.iter().filter(|t| t.scripted && t.result == kq8_core::connor_mover::PushResult::Push) {
        if !sim.push_calls.contains(&t.object) {
            sim.push_calls.push(t.object.clone());
        }
    }
    // M3 3i-7: a play-once movement ended: for getEndLoop listeners.
    if let Some(e) = step.as_ref().and_then(|s| s.ended) {
        sim.ended.push(kq8_core::connor::plain_movement_name(e).to_string());
    }
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
    // sound.md rule 39a: the building his steps sound like.
    let down = (down_now != "none").then_some(down_now.as_str());
    let is_building = down.is_some_and(|d| footsounds.as_ref().is_some_and(|f| f.0.contains_key(&d.to_ascii_lowercase())));
    if let Some(line) = sim.step_ground.after_update(down, is_building, sim.mover.use_gravity) {
        events.push(line);
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
            format!("head-on into a wall: {from} -> {to} from the next update (turn keys turn him; forward, back and run wait until it ends)")
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

/// A `FootStep` cue (`sound.md` rules 36-39a): the remembered building's
/// `footsoundType`, else the terrain's flags under him now, give the flags,
/// the flags the set, a roll the file; a plain one-shot. Returns the log
/// text.
fn footstep(sim: &mut ConnorSim, terrain: &kq8_data::Terrain, footsounds: Option<&FootSounds>) -> String {
    use kq8_core::sound::{building_floor_flags, footstep_file, footstep_set, Start};
    let [x, y, _] = sim.mover.position;
    let building = sim.step_ground.building().map(str::to_string);
    let t = building.as_ref().and_then(|b| footsounds.and_then(|f| f.0.get(&b.to_ascii_lowercase())));
    let (on, flags, from) = match (building, t) {
        (Some(b), Some(t)) => {
            let from = match t {
                Some(n) => format!("building footsoundType={n}"),
                None => "building without footsoundType: 0, so 0x1 (rule 39)".to_string(),
            };
            (b, building_floor_flags(*t), from)
        }
        _ => ("terrain".to_string(), terrain.flags_at(x, y), "the terrain's envInfo.kq flags under him (rule 39a)".to_string()),
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
