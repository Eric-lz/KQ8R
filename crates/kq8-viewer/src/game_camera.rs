//! The camera that follows Connor (M2 step 2f, docs/DESIGN.md), using
//! `kq8_core::camera` (rules from docs/behaviour/camera.md). Its logic runs
//! in the fixed 15-per-second schedule right after Connor; drawing blends
//! between its last two updates. F switches to the free flying camera and
//! back.
//!
//! Keys from `MASK/Camera.sae` (F11 mode, Page Up/Down tilt, Home reset)
//! and `MASK/Connor.sae` (right mouse `MoveCamera`, Shift `ModifyDn`).
//! Mouse value = pixels ÷ 600: a stand-in until Q27 (a) is answered.

use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use kq8_core::camera::{connor_moved, ground_line_hit, view_angles, view_direction, wrap, Camera, Followed, Input, Mode, ViewBlockers, DEFAULT_TILT};
use kq8_core::camera_flight::{Flight, Spots, View};
use kq8_core::connor_mover::GroundLookup as _;
use kq8_core::collide::{BoxCollider, BuildingCollider};

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
/// (Clicks used it too until M3 step 3g-2; they now test only clickable
/// shapes, clicking.md rules 1-8.)
pub struct Blockers<'a> {
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

/// The view blockers as they stand now (doors move).
pub fn view_blockers<'a>(
    terrain: &'a kq8_data::Terrain,
    buildings: Option<&'a Buildings>,
    blockers: Option<&'a ViewBlockerSet>,
    doors: Option<&DoorSim>,
) -> Blockers<'a> {
    Blockers {
        terrain,
        buildings: match (buildings, blockers) {
            (Some(b), Some(v)) => b.0.iter().zip(&v.building_types).filter(|(_, t)| *t & VIEW_BLOCKING != 0).map(|(b, _)| b).collect(),
            (Some(b), None) => b.0.iter().collect(),
            _ => Vec::new(),
        },
        boxes: blockers.map_or(&[][..], |v| &v.boxes[..]),
        doors: doors
            .map(|d| {
                let w = &d.active.world;
                // Not while its collider is out (object-lighting.md rule 41).
                w.doors.iter().filter(|a| a.collision_type & VIEW_BLOCKING != 0 && !w.is_switched_off(&a.door.name)).filter_map(|a| a.collider()).collect()
            })
            .unwrap_or_default(),
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
    /// `KQCamera::follow none` (M3 3k-3, `camera.md` rule 1): `false` =
    /// the camera stays where it is.
    pub following: bool,
    /// After `follow Connor` (M3 3l-3): how far the view's heading is
    /// still turned away from looking at Connor (radians, positive =
    /// anticlockwise from above), after the last two updates.
    view_turn: (f32, f32),
    /// `camera.md` "Camera flights and cuts" (M3 3m-4): the spot list,
    /// the flight under way, the view of a flight or cut (with the roll,
    /// logged, not drawn), and the "places" / "aims" switches `follow`
    /// sets.
    spots: Spots,
    flight: Option<Flight>,
    /// The flight was asked for during this update: its first step is the
    /// next update's (the note's timing: watermill cue 5, 30 updates after
    /// cue 4, comes one update before the flight's last).
    flight_fresh: bool,
    view: View,
    places: bool,
    aims: bool,
    /// `camera.md` "When the camera updates at all" (M3 3n-1): Connor's
    /// base and heading at the last tick (for the "Connor moved" test),
    /// causes waiting for this tick's camera update (`follow`, the start),
    /// the scripts' `move Connor` this tick (for the log), and the "turn
    /// back" switch (an update every tick while on).
    last_connor: Option<([f32; 3], f32)>,
    pending: Vec<&'static str>,
    script_moves: Vec<String>,
    turn_back: bool,
}

impl GameCam {
    /// The view direction drawn now (from the camera to its aim point,
    /// turned by what is left of a turn back) as (tilt, heading).
    fn view_now(&self) -> (f32, f32) {
        let (p, _, aim) = self.current;
        let (tilt, heading) = view_angles(std::array::from_fn(|k| aim[k] - p[k]));
        (tilt, heading + self.view_turn.1)
    }

    /// `KQCamera::follow Connor|none` (`camera.md` rule 1, "Shake and
    /// cutscene cameras"). Following Connor again starts from where the
    /// camera is: its next update pulls it in or pushes it out to the
    /// range; a view more than 0.4 rad off Connor turns back gradually
    /// (1/3 per update), otherwise it looks at him at once; the tilt is
    /// the default at once.
    pub fn follow(&mut self, connor: bool, connor_base: [f32; 3]) -> String {
        self.following = connor;
        // Both switches follow `follow` (rule 1 of the flights); following
        // an object switches the obstacle avoidance on again.
        self.places = connor;
        self.aims = connor;
        if !connor {
            let flying = if self.flight.is_some() { "; the flight goes on without aiming" } else { "" };
            return format!("the camera stays where it is (places and aims off){flying}");
        }
        self.cam.avoid = true;
        let ended = match self.flight.take() {
            Some(f) => format!(" (the flight ends at update {} of {}: STAND-IN, the original adds its steps until the end, overruled by the leash)", f.k, f.n),
            None => String::new(),
        };
        let p = self.cam.position;
        let aim = Camera::aim_for(connor_base);
        let towards = view_angles(std::array::from_fn(|k| aim[k] - p[k])).1;
        let (_, heading) = self.view_now();
        let off = wrap(heading - towards);
        let start = kq8_core::camera::follow_turn_start(off);
        self.view_turn = (start, start);
        self.cam.tilt = DEFAULT_TILT;
        // "When the camera updates at all" item 2: one update right after
        // the command, with the aim point counting as not moved.
        self.pending.push("follow");
        // Item 3: the "turn back" switch. A follow that finds the camera
        // farther than the maximum switches it off; one that finds the view
        // more than 0.4 rad off switches it on (our order when both hold).
        let distance = (0..3).map(|k| (p[k] - aim[k]).powi(2)).sum::<f32>().sqrt();
        let was = self.turn_back;
        if distance > self.cam.range.1 {
            self.turn_back = false;
        }
        if start != 0.0 {
            self.turn_back = true;
        }
        let switch = match (was, self.turn_back) {
            (false, true) => "the \"turn back\" switch goes ON: the camera updates every update until a flight ends (the original clears it no other way)".to_string(),
            (true, false) => format!("the \"turn back\" switch goes OFF (camera {distance:.1} from the aim point, farther than the maximum {:.0})", self.cam.range.1),
            (true, true) => "the \"turn back\" switch stays ON".to_string(),
            (false, false) => String::new(),
        };
        if !switch.is_empty() {
            self.log.detail(format!("camera tick {}: {switch}", self.updates));
        }
        format!(
            "the camera follows Connor again, from where it is: view heading {heading:.3}, Connor at {towards:.3} ({off:+.3} off): {}{ended}{}",
            if start != 0.0 { "turns back 1/3 per update" } else { "looks at him at once (0.4 rad or less)" },
            if switch.is_empty() { String::new() } else { format!("; {switch}") }
        )
    }

    /// A script's `move Connor` this tick (for the log: the camera update
    /// it causes comes after the script, in `camera_step`).
    pub fn connor_moved_by_script(&mut self, at: &str) {
        self.script_moves.push(at.to_string());
    }

    /// `KQCamera::addCamLoc` (rule 1).
    pub fn add_spot(&mut self, spot: [f32; 3], newlist: bool) -> String {
        self.spots.add(spot, newlist);
        let line = format!("spot ({:.0}, {:.0}, {:.0}) added; the list has {}", spot[0], spot[1], spot[2], self.spots.0.len());
        self.log.detail(format!("camera update {}: {line}", self.updates));
        line
    }

    /// The view a flight or cut starts from: the one drawn now, with the
    /// roll kept.
    fn start_view(&self) -> View {
        let (tilt, heading) = self.view_now();
        View { tilt, heading, roll: self.view.roll, locked: false }
    }

    /// Draws the view of a flight or cut from where the camera is.
    fn show_view(&mut self) {
        let p = self.cam.position;
        let dir = view_direction(self.view.tilt, self.view.heading);
        self.current = (p, DEFAULT_TILT, std::array::from_fn(|k| p[k] + dir[k] * 1000.0));
        self.view_turn = (0.0, 0.0);
    }

    /// `KQCamera::flyToCamLoc` (rule 4): to the spot nearest the camera, in
    /// N updates from the next camera update on (`camera_step`). An empty
    /// list: nothing, an error ("Do not reproduce").
    pub fn fly(&mut self, aim: [f32; 3], seconds: Option<f32>, end_roll: Option<f32>) -> String {
        let Some(spot) = self.spots.nearest(self.cam.position) else {
            let line = "ERROR flyToCamLoc: no spot in the list (addCamLoc); the camera stays".to_string();
            self.log.detail(format!("camera update {}: {line}", self.updates));
            return line;
        };
        let p = self.cam.position;
        self.view = self.start_view();
        let f = Flight::new(p, self.view.roll, spot, aim, seconds, end_roll);
        self.following = false;
        self.cam.avoid = false;
        self.cam.tilt = DEFAULT_TILT;
        self.places = false;
        let line = format!(
            "flight: N = {} updates from ({:.1}, {:.1}, {:.1}) to the spot ({:.0}, {:.0}, {:.0}), step ({:.2}, {:.2}, {:.2}), roll {:.3} -> {:.3}; aiming at ({:.0}, {:.0}, {:.0}) + 1400; places off, aims {}",
            f.n, p[0], p[1], p[2], spot[0], spot[1], spot[2], f.step[0], f.step[1], f.step[2], self.view.roll, end_roll.unwrap_or(0.0), aim[0], aim[1], aim[2], if self.aims { "on" } else { "off" }
        );
        self.log.detail(format!("camera update {}: {line}", self.updates));
        self.flight = Some(f);
        self.flight_fresh = true;
        line
    }

    /// `KQCamera::cutToCamLoc` (rule 3): on the nearest spot at once, then
    /// one follow update. An empty list: nothing, an error.
    pub fn cut_to(&mut self, aim: [f32; 3], roll: f32, ground: &dyn Fn(f32, f32) -> Option<f32>) -> String {
        let Some(spot) = self.spots.nearest(self.cam.position) else {
            let line = "ERROR cutToCamLoc: no spot in the list (addCamLoc); the camera stays".to_string();
            self.log.detail(format!("camera update {}: {line}", self.updates));
            return line;
        };
        self.view = self.start_view();
        self.flight = None;
        self.following = false;
        self.cam.avoid = false;
        self.cam.tilt = DEFAULT_TILT;
        let (p, from) = kq8_core::camera_flight::cut(spot, aim, roll, &mut self.view, self.places, self.aims, self.cam.range);
        self.cam.position = p;
        self.show_view();
        self.previous = self.current;
        let height = height_note(p, aim, ground);
        let line = format!(
            "cut to the spot ({:.0}, {:.0}, {:.0}){}; at ({:.1}, {:.1}, {:.1}), view heading {:.3}, tilt {:+.3}, roll {:.3} (not drawn); places {}, aims {}{height}",
            spot[0], spot[1], spot[2],
            from.map_or(String::new(), |d| format!(", pulled onto the range from {d:.1}")),
            p[0], p[1], p[2], self.view.heading, self.view.tilt, self.view.roll, self.places, self.aims
        );
        self.log.detail(format!("camera update {}: {line}", self.updates));
        line
    }

    /// One update of the flight under way (`camera_step`).
    fn flight_step(&mut self, ground: &dyn Fn(f32, f32) -> Option<f32>) {
        if std::mem::take(&mut self.flight_fresh) {
            return;
        }
        let Some(f) = self.flight.as_mut() else { return };
        let mut p = self.cam.position;
        let s = f.update(&mut p, &mut self.view, self.aims, self.cam.range, ground);
        let (aim, step) = (f.aim, f.step);
        self.cam.position = p;
        self.show_view();
        let mut line = format!(
            "camera update {}: flight {}/{}: at ({:.1}, {:.1}, {:.1}), step ({:.2}, {:.2}, {:.2}), roll {:.3}, view heading {:.3}, tilt {:+.3}{}; places {}, aims {}",
            self.updates, s.k, s.n, p[0], p[1], p[2], step[0], step[1], step[2], self.view.roll, self.view.heading, self.view.tilt,
            s.aimed.map_or(String::new(), |a| format!(" ({a})")),
            s.k >= s.n || self.places, self.aims
        );
        if s.k >= s.n {
            // Rule 4, the last update: "places" back on, pulled onto the range.
            self.places = true;
            if std::mem::take(&mut self.turn_back) {
                line += "; the \"turn back\" switch goes OFF (a flight ended)";
            }
            line += &format!(
                "; last update: on the spot, places on: {}{}; the flight is over, the camera stays here",
                s.pulled_from.map_or("within the range: stays".to_string(), |d| format!("{d:.1} from the point 1400 above the aim: pulled onto the range ({:.0}, {:.0})", self.cam.range.0, self.cam.range.1)),
                height_note(p, aim, ground)
            );
            self.flight = None;
        }
        self.log.detail(line);
    }

    /// `handsOff 1` (`anim-scripts.md` rule 21): fixed mode (F11) goes back
    /// to normal mode; the mouse is ignored while it is on (`camera_step`).
    pub fn hands_off(&mut self) -> Option<String> {
        if let Mode::Fixed { .. } = self.cam.mode {
            self.cam.mode = Mode::Normal;
            return Some("fixed camera mode back to normal".into());
        }
        None
    }

    /// `move KQCamera <flags> x y z rx ry rz` (`camera.md` "Shake and
    /// cutscene cameras"): flags bit 1 adds the position, bit 2 adds the
    /// angles to the current view; rx = tilt, rz = heading; the camera
    /// takes that place and view at once. ry (roll) is not drawn (logged).
    pub fn move_to(&mut self, flags: u32, position: [f32; 3], turn: [f32; 3]) -> String {
        let old = self.cam.position;
        let (old_tilt, old_heading) = self.view_now();
        self.cam.position = if flags & 1 != 0 { std::array::from_fn(|k| old[k] + position[k]) } else { position };
        let (tilt, heading) = if flags & 2 != 0 { (old_tilt + turn[0], old_heading + turn[2]) } else { (turn[0], turn[2]) };
        let dir = view_direction(tilt, heading);
        let p = self.cam.position;
        // Only the direction matters; the drawing pitches by the tilt's
        // difference from the default, so the default draws it as is.
        let aim = std::array::from_fn(|k| p[k] + dir[k] * 1000.0);
        self.current = (p, DEFAULT_TILT, aim);
        self.previous = self.current;
        self.view_turn = (0.0, 0.0);
        // A flight under way goes on from here; with "aims" on it turns
        // back to its aim at the next update (flights rule 6).
        self.view = View { tilt, heading, roll: if flags & 2 != 0 { self.view.roll + turn[1] } else { turn[1] }, locked: false };
        let roll = if turn[1] != 0.0 { format!("; roll {:.3} not drawn", turn[1]) } else { String::new() };
        let line = format!(
            "camera moved from ({:.0}, {:.0}, {:.0}) to ({:.0}, {:.0}, {:.0}), view tilt {tilt:.3}, heading {heading:.3} (looks along ({:.3}, {:.3}, {:.3})){roll}",
            old[0], old[1], old[2], p[0], p[1], p[2], dir[0], dir[1], dir[2]
        );
        self.log.detail(format!("camera update {}: {line}", self.updates));
        line
    }
}

/// The height rule's wanted height at `p` around `aim` (logged only on a
/// flight's last update and after a cut: STAND-IN, whether it is applied
/// there was not settled).
fn height_note(p: [f32; 3], aim: [f32; 3], ground: &dyn Fn(f32, f32) -> Option<f32>) -> String {
    let floor = ground(p[0], p[1]).map_or(f32::MIN, |g| g + kq8_core::camera::MIN_HEIGHT);
    let wanted = floor.max(aim[2] + kq8_core::camera::MIN_HEIGHT);
    if p[2] >= wanted {
        String::new()
    } else {
        format!("; the height rule would want z {wanted:.1} ({:.1} higher): not applied (stand-in, not settled)", wanted - p[2])
    }
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
    let aim = [connor_at[0], connor_at[1], connor_at[2] + kq8_core::camera::AIM_HEIGHT];
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
        following: true,
        view_turn: (0.0, 0.0),
        spots: Spots::default(),
        flight: None,
        flight_fresh: false,
        view: View { tilt: -DEFAULT_TILT, heading: 0.0, roll: 0.0, locked: false },
        places: true,
        aims: true,
        last_connor: None,
        // STAND-IN (Q27 d): Daventry's start-up script has `KQCamera::follow
        // none`; how the original's camera starts following was not traced.
        // One update on the first tick, as before 3n-1.
        pending: vec!["start (stand-in)"],
        script_moves: Vec::new(),
        turn_back: false,
    };
    g.log.detail("update x y z distance_to_aim terrain_below height_above_terrain tilt mode range_min range_max | connor_moving blocked_by hit swing_step swing_left pulled_in inside_building | causes (one line per camera update; a tick without one has no line) | height_step aim_moved_sum wanted start_z before_z");
    Some(g)
}

/// Gathers mouse movement and key presses between updates; F switches
/// between the game camera and the free one.
// A Bevy system gets everything it uses as arguments; splitting it up
// would not make it clearer.
#[allow(clippy::too_many_arguments)]
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
    g.updates += 1;
    g.previous = g.current;
    g.view_turn.0 = g.view_turn.1;
    // "Connor moved" (camera.md "When the camera updates at all" item 1):
    // his position or facing changed since the last tick. Our camera runs
    // after Connor's update and this tick's scripts, so the update it
    // causes comes after a trap's `move KQCamera` (the note's queue).
    let now = (sim.mover.position, sim.walker.heading);
    let moved = g.last_connor.is_some_and(|l| connor_moved(l, now));
    g.last_connor = Some(now);
    let script_moves = std::mem::take(&mut g.script_moves);
    // `KQCamera::follow none` (camera.md rule 1): it stays where it is,
    // unless a flight is under way (M3 3m-4).
    if !g.following {
        g.mouse_px = Vec2::ZERO;
        g.pending.clear();
        let ground = |x: f32, y: f32| lookup.0.ground(x, y).map(|h| h.0);
        g.flight_step(&ground);
        return;
    }
    // One range for walking and running: the user saw no difference in
    // the original (Q27 e), so the run switch does not change it.
    let held = |k: Option<KeyCode>| choice.game && k.is_some_and(|k| keys.pressed(k));
    // handsOff (anim-scripts.md rule 21): the mouse-look button does
    // nothing; the camera's own keys still work.
    let input = Input {
        mouse_held: choice.game && !sim.hands_off && g.keys.mouse.is_some_and(|b| buttons.pressed(b)),
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
    // Ticks without a "moved" message (Connor is moving, the previous aim
    // point), then the causes of a camera update this tick.
    let base = sim.mover.position;
    if g.cam.tick(moved, base, input.mouse_held) {
        g.log.detail(format!("camera tick {}: 3 updates without \"Connor moved\": still; the previous aim point reset to where he stands", g.updates));
    }
    let mut causes: Vec<String> = Vec::new();
    if moved {
        causes.push(if script_moves.is_empty() { "moved".into() } else { "moved(script)".into() });
    }
    if !script_moves.is_empty() {
        g.log.detail(format!(
            "camera tick {}: script's move Connor ({}): {}",
            g.updates,
            script_moves.join("; "),
            if moved { "\"Connor moved\" after the script: one camera update below, from where the script left the camera" } else { "changed nothing: no message" }
        ));
    }
    let pending = std::mem::take(&mut g.pending);
    if !pending.is_empty() {
        // `follow` (and our start): the aim point counts as not moved.
        g.cam.not_moved(base);
        causes.extend(pending.iter().map(|c| c.replace(' ', "_")));
    }
    if g.turn_back {
        causes.push("turn_back".into());
    }
    if input.mouse_held || input.tilt_keys != 0 || input.reset_tilt || input.change_mode {
        causes.push("player".into());
    }
    if causes.is_empty() {
        return;
    }
    // The view blockers as they stand now (doors move).
    let view = view_blockers(&lookup.0, buildings.as_deref(), blockers.as_deref(), doors.as_deref());
    let report = g.cam.update_with(Followed { base }, moved, &input, &lookup.0, Some(&view));
    // Check from the numbers: is the camera inside a building's solid?
    let inside: Vec<&str> = view.buildings.iter().filter(|b| b.solid_at(g.cam.position)).map(|b| b.name.as_str()).collect();
    g.current = (g.cam.position, g.cam.tilt, report.aim);
    // The turn back toward Connor after `follow Connor` (3l-3).
    if g.view_turn.1 != 0.0 {
        g.view_turn.1 = kq8_core::camera::follow_turn_step(g.view_turn.1);
        let line = format!("camera update {}: view turning back toward Connor: {:.3} left", g.updates, g.view_turn.1);
        g.log.detail(line);
    }
    let p = g.cam.position;
    let line = format!(
        "{} {:.1} {:.1} {:.1} {:.1} {} {} {:.3} {} {} {} | {} {} {} {} {} {} {} | {} | {}",
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
        if inside.is_empty() { "-".to_string() } else { inside.join(",") },
        causes.join(","),
        report.height.map_or("-".to_string(), |h| format!("{} {:.1} {:.1} {:.1} {:.1}", h.words(), h.sum, h.wanted, h.start, h.before))
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
    // What is left of a turn back toward Connor (3l-3), round the
    // vertical (game +Z is Bevy +Y; both turn anticlockwise from above).
    let (o0, o1) = game.view_turn;
    view.rotate_axis(Dir3::Y, o0 + (o1 - o0) * a);
    *t = view;
}
