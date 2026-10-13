//! The camera that follows Connor (M2 step 2f, docs/DESIGN.md), using
//! `kq8_game::camera` (rules from docs/behaviour/camera.md). Its logic runs
//! in the fixed 15-per-second schedule right after Connor; drawing blends
//! between its last two updates. F switches to the free flying camera and
//! back.
//!
//! Keys from `MASK/Camera.sae` (F11 mode, Page Up/Down tilt, Home reset)
//! and `MASK/Connor.sae` (right mouse `MoveCamera`, Shift `ModifyDn`).
//! Mouse value = pixels ÷ 600: a stand-in until Q27 (a) is answered.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use kq8_game::camera::{ground_line_hit, Camera, Followed, Input, Mode, ViewBlockers, DEFAULT_TILT};
use kq8_game::collide::{BoxCollider, BuildingCollider};

use crate::connor_view::{key_code, ConnorSim};
use crate::log::Log;
use crate::door_view::DoorSim;
use crate::load::VIEW_BLOCKING;
use crate::terrain_view::{Buildings, Fly, GroundLookup, ViewBlockerSet};
use crate::{to_bevy, Shared, UNITS_PER_METRE};

/// Stand-in until Q27 (a): this many screen pixels make one mouse unit.
const PIXELS_PER_MOUSE_UNIT: f32 = 800.0; // modified manually to what feels more natural, but could be wrong

/// What blocks the view line now (camera.md "Obstacles"): the drawn
/// ground, the blocking buildings, object boxes and doors as they stand.
struct Blockers<'a> {
    terrain: &'a kq8_data::Terrain,
    buildings: Vec<&'a BuildingCollider>,
    boxes: &'a [BoxCollider],
    doors: Vec<BoxCollider>,
}

impl ViewBlockers for Blockers<'_> {
    fn first_block(&self, from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], String)> {
        let dist = |p: [f32; 3]| (0..3).map(|k| (p[k] - from[k]).powi(2)).sum::<f32>();
        let surface = |x: f32, y: f32| self.terrain.height_at(x, y).map(|g| g.height);
        let mut hits: Vec<([f32; 3], String)> = Vec::new();
        if let Some(h) = ground_line_hit(&surface, from, to) {
            hits.push((h, "the ground".into()));
        }
        for b in &self.buildings {
            if let Some(h) = b.line_hit(from, to) {
                hits.push((h, format!("building {}", b.name)));
            }
        }
        for b in self.boxes.iter().chain(&self.doors) {
            if let Some(h) = b.line_hit(from, to) {
                hits.push((h, format!("box {}", b.name)));
            }
        }
        hits.into_iter().min_by(|a, b| dist(a.0).total_cmp(&dist(b.0)))
    }
}

/// Which camera draws the view: the game camera or the free one.
#[derive(Resource)]
pub struct CameraChoice {
    pub game: bool,
}

struct Keys {
    change_mode: Option<KeyCode>,
    tilt_up: Option<KeyCode>,
    tilt_down: Option<KeyCode>,
    reset: Option<KeyCode>,
    shift: Option<KeyCode>,
    mouse: Option<MouseButton>,
}

impl GameCam {
    /// The camera's position after the last update (game units).
    pub fn position(&self) -> [f32; 3] {
        self.cam.position
    }

    /// `KQCamera::setRange` from a script (M3 step 3d-3); false = refused.
    pub fn set_range(&mut self, min: f32, max: f32) -> bool {
        let old = self.cam.range;
        let ok = self.cam.set_range(min, max);
        self.log.detail(format!(
            "camera update {}: range ({:.0}, {:.0}) -> {} (script)",
            self.updates,
            old.0,
            old.1,
            if ok { format!("({min:.0}, {max:.0})") } else { "refused".into() }
        ));
        ok
    }
}

#[derive(Resource)]
pub struct GameCam {
    cam: Camera,
    /// (position, tilt, aim) after the last two updates.
    previous: ([f32; 3], f32, [f32; 3]),
    current: ([f32; 3], f32, [f32; 3]),
    keys: Keys,
    /// Mouse movement (pixels) and presses gathered between updates.
    mouse_px: Vec2,
    pressed_mode: bool,
    pressed_reset: bool,
    updates: u64,
    log: Log,
}

/// Makes the game camera. `start`: where the world script put `KQCamera`
/// (Q27 d), else behind Connor.
pub fn make(
    connor_at: [f32; 3],
    heading: f32,
    start: Option<[f32; 3]>,
    camera_keys: &[kq8_data::action_map::Binding],
    connor_keys: &[kq8_data::action_map::Binding],
    main_log: &mut Log,
) -> Option<GameCam> {
    use kq8_data::action_map::key_for;
    let position = start.unwrap_or_else(|| {
        let forward = [heading.sin(), -heading.cos()];
        [connor_at[0] - forward[0] * 3000.0, connor_at[1] - forward[1] * 3000.0, connor_at[2] + 1400.0]
    });
    let key = |list: &[kq8_data::action_map::Binding], action: &str, value: f32| key_for(list, action, value).and_then(key_code);
    let mouse = connor_keys
        .iter()
        .find(|b| b.press && b.device == "MOU" && b.action.eq_ignore_ascii_case("MoveCamera"))
        .and_then(|b| match b.key.split_whitespace().next() {
            Some("r") => Some(MouseButton::Right),
            Some("l") => Some(MouseButton::Left),
            _ => None,
        });
    let keys = Keys {
        change_mode: key(camera_keys, "CameraChangeMode", 0.0),
        tilt_up: key(camera_keys, "RotateX", 0.05),
        tilt_down: key(camera_keys, "RotateX", -0.05),
        reset: key(camera_keys, "ResetX", 0.0),
        shift: key(connor_keys, "ModifyDn", 0.0),
        mouse,
    };
    main_log.summary(format!(
        "camera: starts at ({:.0}, {:.0}, {:.0}) ({}); keys: mode {:?}, tilt up {:?}, tilt down {:?}, reset {:?}, modifier {:?}, mouse {:?}",
        position[0],
        position[1],
        position[2],
        if start.is_some() { "the world script's KQCamera" } else { "behind Connor" },
        keys.change_mode,
        keys.tilt_up,
        keys.tilt_down,
        keys.reset,
        keys.shift,
        keys.mouse
    ));
    let log = match Log::create("camera") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("camera: cannot create camera.log: {e}"));
            return None;
        }
    };
    let cam = Camera::new(position);
    let aim = [connor_at[0], connor_at[1], connor_at[2] + kq8_game::camera::AIM_HEIGHT];
    let mut g = GameCam {
        previous: (position, cam.tilt, aim),
        current: (position, cam.tilt, aim),
        cam,
        keys,
        mouse_px: Vec2::ZERO,
        pressed_mode: false,
        pressed_reset: false,
        updates: 0,
        log,
    };
    g.log.detail("update x y z distance_to_aim terrain_below height_above_terrain tilt mode range_min range_max | connor_moving blocked_by hit swing_step swing_left pulled_in inside_building");
    Some(g)
}

/// Gathers mouse movement and key presses between updates; F switches
/// between the game camera and the free one.
pub fn camera_input(
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    motion: Res<AccumulatedMouseMotion>,
    game: Option<ResMut<GameCam>>,
    mut choice: ResMut<CameraChoice>,
    mut fly: ResMut<Fly>,
    camera: Query<&Transform, With<Camera3d>>,
    shared: Res<Shared>,
) {
    let Some(mut game) = game else { return };
    if keys.just_pressed(KeyCode::KeyF) {
        choice.game = !choice.game;
        if !choice.game {
            // The free camera starts where the game camera was looking.
            if let Ok(t) = camera.single() {
                let (yaw, pitch, _) = t.rotation.to_euler(EulerRot::YXZ);
                fly.yaw = yaw;
                fly.pitch = pitch;
            }
        }
        shared.log.lock().unwrap().summary(format!("camera: {}", if choice.game { "game camera (follows Connor)" } else { "free flying camera" }));
    }
    if !choice.game {
        return;
    }
    if game.keys.mouse.is_some_and(|b| buttons.pressed(b)) {
        game.mouse_px += motion.delta;
    }
    if game.keys.change_mode.is_some_and(|k| keys.just_pressed(k)) {
        game.pressed_mode = true;
    }
    if game.keys.reset.is_some_and(|k| keys.just_pressed(k)) {
        game.pressed_reset = true;
    }
}

/// One camera update, right after Connor's (fixed 15 per second).
#[allow(clippy::too_many_arguments)]
pub fn camera_step(
    game: Option<ResMut<GameCam>>,
    sim: Option<Res<ConnorSim>>,
    lookup: Option<Res<GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    blockers: Option<Res<ViewBlockerSet>>,
    doors: Option<Res<DoorSim>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    choice: Res<CameraChoice>,
    shared: Res<Shared>,
) {
    let (Some(mut game), Some(sim), Some(lookup)) = (game, sim, lookup) else { return };
    let g = &mut *game;
    // One range for walking and running: the user saw no difference in
    // the original (Q27 e), so the run switch does not change it.
    let held = |k: Option<KeyCode>| choice.game && k.is_some_and(|k| keys.pressed(k));
    let input = Input {
        mouse_held: choice.game && g.keys.mouse.is_some_and(|b| buttons.pressed(b)),
        // Directions checked by eye by the user against the original
        // (2026-10-11): mouse left swings the camera left, mouse down
        // (towards you; screen y grows downwards) zooms in.
        mouse_x: -g.mouse_px.x / PIXELS_PER_MOUSE_UNIT,
        mouse_forward: g.mouse_px.y / PIXELS_PER_MOUSE_UNIT,
        shift: held(g.keys.shift),
        tilt_keys: i8::from(held(g.keys.tilt_up)) - i8::from(held(g.keys.tilt_down)),
        reset_tilt: g.pressed_reset,
        change_mode: g.pressed_mode,
    };
    g.mouse_px = Vec2::ZERO;
    g.pressed_mode = false;
    g.pressed_reset = false;
    // The view blockers as they stand now (doors move).
    let view = Blockers {
        terrain: &lookup.0,
        buildings: match (&buildings, &blockers) {
            (Some(b), Some(v)) => b.0.iter().zip(&v.building_types).filter(|(_, t)| *t & VIEW_BLOCKING != 0).map(|(b, _)| b).collect(),
            (Some(b), None) => b.0.iter().collect(),
            _ => Vec::new(),
        },
        boxes: blockers.as_ref().map_or(&[][..], |v| &v.boxes[..]),
        doors: doors
            .as_ref()
            .map(|d| d.active.world.doors.iter().filter(|a| a.collision_type & VIEW_BLOCKING != 0).filter_map(|a| a.collider()).collect())
            .unwrap_or_default(),
    };
    let followed = Followed { base: sim.mover.position, heading: sim.walker.heading };
    let report = g.cam.update_with(followed, &input, &lookup.0, Some(&view));
    // Check from the numbers: is the camera inside a building's solid?
    let inside: Vec<&str> = view.buildings.iter().filter(|b| b.solid_at(g.cam.position)).map(|b| b.name.as_str()).collect();
    g.updates += 1;
    g.previous = g.current;
    g.current = (g.cam.position, g.cam.tilt, report.aim);
    let p = g.cam.position;
    let line = format!(
        "{} {:.1} {:.1} {:.1} {:.1} {} {} {:.3} {} {} {} | {} {} {} {} {} {} {}",
        g.updates,
        p[0],
        p[1],
        p[2],
        report.distance,
        report.terrain_below.map_or("none".into(), |t| format!("{t:.1}")),
        report.terrain_below.map_or("none".into(), |t| format!("{:.1}", p[2] - t)),
        g.cam.tilt,
        match g.cam.mode {
            Mode::Normal => "normal".to_string(),
            Mode::Fixed { distance } => format!("fixed({distance:.0})"),
        },
        g.cam.range.0,
        g.cam.range.1,
        report.moving,
        report.blocked.as_ref().map_or("-".to_string(), |(_, w)| w.replace(' ', "_")),
        report.blocked.as_ref().map_or("-".to_string(), |(h, _)| format!("({:.0},{:.0},{:.0})", h[0], h[1], h[2])),
        report.swing.map_or("-".to_string(), |(s, _)| format!("{s:.3}")),
        report.swing.map_or("-".to_string(), |(_, l)| format!("{l:.3}")),
        report.pulled_in.map_or("-".to_string(), |d| format!("{d:.1}")),
        if inside.is_empty() { "-".to_string() } else { inside.join(",") }
    );
    g.log.detail(line);
    if !report.events.is_empty() {
        let mut main = shared.log.lock().unwrap();
        for e in report.events {
            main.detail(format!("camera update {}: {e}", g.updates));
        }
    }
}

/// Draws the view from the game camera, blended between its last two
/// updates, looking at the aim point and pitched by the tilt's difference
/// from the default (stand-in until Q27 b).
pub fn camera_draw(
    game: Option<Res<GameCam>>,
    choice: Res<CameraChoice>,
    fixed: Res<Time<Fixed>>,
    mut camera: Query<&mut Transform, With<Camera3d>>,
) {
    let Some(game) = game else { return };
    if !choice.game {
        return;
    }
    let Ok(mut t) = camera.single_mut() else { return };
    let a = fixed.overstep_fraction();
    let lerp = |x: [f32; 3], y: [f32; 3]| -> [f32; 3] { std::array::from_fn(|i| x[i] + (y[i] - x[i]) * a) };
    let (p0, t0, a0) = game.previous;
    let (p1, t1, a1) = game.current;
    let eye = Vec3::from(to_bevy(lerp(p0, p1), [0.0; 3]));
    let aim = Vec3::from(to_bevy(lerp(a0, a1), [0.0; 3]));
    let tilt = t0 + (t1 - t0) * a;
    if (aim - eye).length() * UNITS_PER_METRE < 1.0 {
        return;
    }
    let mut view = Transform::from_translation(eye).looking_at(aim, Vec3::Y);
    view.rotate_local_x(tilt - DEFAULT_TILT);
    *t = view;
}
