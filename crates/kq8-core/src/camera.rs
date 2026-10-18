//! The third-person camera that follows Connor.
//!
//! Rules from the behaviour note `docs/behaviour/camera.md` ("What the camera
//! looks at", "Distance", "Height above the ground", "Swinging and zooming
//! with the mouse", "Tilt", "Obstacles", "When the camera updates at
//! all"). [`Camera::tick`] runs once per game update (15 per second); a
//! camera update ([`Camera::update_with`]) runs only when the caller has a
//! cause for one (M3 3n-1). Not here yet: the automatic swing behind
//! Connor (Q27 c), first person, shake.
//!
//! Stand-ins until Q27 is answered: what the tilt turns (the caller draws
//! the view looking at the aim point, pitched by `tilt - DEFAULT_TILT`),
//! and in fixed mode Page Up = closer. `mouse_forward` positive = zoom in;
//! the viewer sends mouse-down as positive (the user's check of the
//! original).

use crate::connor_mover::GroundLookup;

/// The camera aims this far above the followed object's base.
pub const AIM_HEIGHT: f32 = 800.0;  // modified manually to what feels more natural, but could be wrong
/// The camera stays at least this far above the terrain under it and above
/// the followed object's base.
pub const MIN_HEIGHT: f32 = 1400.0;
/// Within this of its wanted height the camera jumps to it...
pub const HEIGHT_SNAP: f32 = 100.0;
/// ...otherwise, if the aim point moved more than this (|Δx| + |Δy| +
/// |Δz|) since the camera's previous record of it...
pub const HEIGHT_MOVED: f32 = 20.0;
/// ...it eases this share of the gap per update at 15 updates per second
/// (`height_share`; replaces the hand-tuned 0.5 of before M3 3n-2).
pub const HEIGHT_EASE: f32 = 0.1;
/// One game update (15 per second).
pub const UPDATE_SECONDS: f32 = 1.0 / 15.0;

/// The share of the height gap eased in `seconds` ("Do not reproduce":
/// per second, not per update): 1 − 0.9^(15 × seconds), 10% for one
/// update at 15 per second.
pub fn height_share(seconds: f32) -> f32 {
    1.0 - (1.0 - HEIGHT_EASE).powf(15.0 * seconds)
}

/// Step 1 of the height rule: how the camera was lifted off the ground.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Lift {
    /// Not under the ground + 1,400.
    No,
    /// Put straight at ground + 1,400 (under it by at most the maximum).
    ToGround,
    /// Deeper: put where the line from the update's start to the new spot
    /// meets the ground.
    AlongLine,
    /// Deeper, and that line meets no ground (it starts under it too):
    /// STAND-IN, put at ground + 1,400.
    AlongLineMissed,
}

/// Steps 2–3 of the height rule.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HeightThen {
    /// Within 100 of the wanted height: put on it.
    Snapped,
    /// The aim point moved: the start height + the share of the gap.
    Eased,
    /// Not moved: the height stays as step 1 left it.
    Kept,
    /// Not moved, but the mouse swings and it was lifted earlier: snapped.
    SnappedMouse,
}

/// What the height rule did in one update, for the log.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HeightStep {
    pub lift: Lift,
    pub then: HeightThen,
    /// The aim point's movement since the previous record, |Δx| + |Δy| +
    /// |Δz| (moved if more than 20).
    pub sum: f32,
    pub wanted: f32,
    /// The height at the start of the update and before the height rule.
    pub start: f32,
    pub before: f32,
}

impl HeightStep {
    /// One word or two for the log, e.g. `lifted+kept`.
    pub fn words(&self) -> String {
        let lift = match self.lift {
            Lift::No => "",
            Lift::ToGround => "lifted+",
            Lift::AlongLine => "lifted_along_line+",
            Lift::AlongLineMissed => "lifted_along_line_missed(stand-in)+",
        };
        let then = match self.then {
            HeightThen::Snapped => "snapped",
            HeightThen::Eased => "eased",
            HeightThen::Kept => "kept",
            HeightThen::SnappedMouse => "snapped(mouse)",
        };
        format!("{lift}{then}")
    }
}

/// camera.md "Height above the ground" rule 2, after the leash: the camera
/// went from `start` (where the update began) to `now`; `floor` is the
/// floor grid's height; the wanted height is the higher of the ground
/// under the camera + 1,400 and `base_z` + 1,400.
/// 1. Under the ground + 1,400 by at most `max`: put there at once; deeper:
///    where the line from `start` to `now` meets the ground, the wanted
///    height taken again there.
/// 2. Within 100 of the wanted height: put on it.
/// 3. Otherwise, if `sum` (the aim point's movement) is more than 20: the
///    height at the start of the update + `share` × (wanted − the height
///    after 1–2); if not, kept (snapped only if `mouse_swing` and
///    `lifted_before`).
#[allow(clippy::too_many_arguments)]
pub fn height_rule(
    start: [f32; 3],
    now: [f32; 3],
    base_z: f32,
    max: f32,
    sum: f32,
    share: f32,
    mouse_swing: bool,
    lifted_before: bool,
    floor: &dyn Fn(f32, f32) -> Option<f32>,
) -> ([f32; 3], HeightStep) {
    let mut p = now;
    let mut lift = Lift::No;
    if let Some(g) = floor(p[0], p[1]) {
        let gap = g + MIN_HEIGHT - p[2];
        if gap > 0.0 && gap <= max {
            p[2] = g + MIN_HEIGHT;
            lift = Lift::ToGround;
        } else if gap > max {
            match ground_line_hit(floor, start, now) {
                Some(hit) => {
                    p = hit;
                    lift = Lift::AlongLine;
                }
                None => {
                    p[2] = g + MIN_HEIGHT;
                    lift = Lift::AlongLineMissed;
                }
            }
        }
    }
    let wanted = floor(p[0], p[1]).map_or(base_z, |g| g.max(base_z)) + MIN_HEIGHT;
    let before = now[2];
    let then = if (wanted - p[2]).abs() <= HEIGHT_SNAP {
        p[2] = wanted;
        HeightThen::Snapped
    } else if sum > HEIGHT_MOVED {
        p[2] = start[2] + share * (wanted - p[2]);
        HeightThen::Eased
    } else if mouse_swing && lifted_before {
        p[2] = wanted;
        HeightThen::SnappedMouse
    } else {
        HeightThen::Kept
    };
    (p, HeightStep { lift, then, sum, wanted, start: start[2], before })
}
/// Swing per mouse unit (radians).
pub const SWING_PER_UNIT: f32 = 3.0 * std::f32::consts::FRAC_PI_4;
/// Zoom per mouse unit (normal mode).
pub const ZOOM_PER_UNIT: f32 = 5000.0;
/// Tilt per mouse unit (fixed mode, or normal mode with Shift).
pub const TILT_PER_UNIT: f32 = 0.1 * 7.5;
/// Tilt per update while Page Up/Down is held (0.1 × 3 × 0.05).
pub const TILT_PER_UPDATE: f32 = 0.015;
/// Fixed-mode zoom per update while Page Up/Down is held (0.05 × 2,500).
pub const FIXED_ZOOM_PER_UPDATE: f32 = 125.0;
/// Third-person tilt limits.
pub const TILT_LIMIT: f32 = 1.57;
pub const DEFAULT_TILT: f32 = 0.2;
/// Tilt snaps back once the followed object moved more than this
/// (|Δx| + |Δy|) in one update, in normal mode.
pub const MOVE_RESETS_TILT: f32 = 10.0;
/// Range before any script sets one.
pub const DEFAULT_RANGE: (f32, f32) = (1500.0, 5000.0);
/// A blocked camera is pulled in to this far short of the hit
/// (camera.md "Obstacle numbers").
pub const PULL_IN_MARGIN: f32 = 100.0;
/// ...but never closer than this to the aim point ("Do not reproduce":
/// the original can end up behind Connor).
pub const PULL_IN_MIN: f32 = 1.0;
/// The new direction aims halfway between Connor's last aim point and
/// the hit.
pub const AVOID_MIX: f32 = 0.5;
/// Swing step per update = this × the first turn.
pub const SWING_SHARE: f32 = 0.66;
/// Dragging into a wall: the previous offset shortened to this.
pub const DRAG_SHORTEN: f32 = 0.95;
/// Connor counts as still after this many updates without movement.
pub const STILL_UPDATES: u32 = 3;
/// A move sends "Connor moved" when his position changes by more than
/// this on an axis... (camera.md "When the camera updates at all")
pub const MOVED_POSITION: f32 = 0.1;
/// ...or an entry of his turn matrix by more than this.
pub const MOVED_FACING: f32 = 0.0001;

/// Whether going from `before` to `now` (base, heading) sends a "Connor
/// moved" message: a position change of more than 0.1 on any axis, or a
/// facing change (cos or sin of the heading, the entries of a turn round
/// the vertical, by more than 0.0001).
pub fn connor_moved(before: ([f32; 3], f32), now: ([f32; 3], f32)) -> bool {
    let (sb, cb) = before.1.sin_cos();
    let (sn, cn) = now.1.sin_cos();
    (0..3).any(|k| (now.0[k] - before.0[k]).abs() > MOVED_POSITION) || (sn - sb).abs() > MOVED_FACING || (cn - cb).abs() > MOVED_FACING
}

/// What blocks the view line (camera.md "Obstacles"): the drawn ground,
/// buildings, objects with collision bit 0x8 or 0x1000.
pub trait ViewBlockers {
    /// The nearest point where the line from `from` to `to` is blocked,
    /// and what blocked it (for the log).
    fn first_block(&self, from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], String)>;
}

/// Step along the view line when testing it against the drawn ground.
pub const GROUND_STEP: f32 = 50.0;
/// Halvings of the step that found the ground.
pub const GROUND_REFINE: u32 = 20;

/// Where the line from `from` to `to` first goes from above the ground to
/// below it; `height` gives the drawn ground's height at (x, y) (`None`
/// outside the grid). Stepped every `GROUND_STEP`, then halved
/// `GROUND_REFINE` times (our method; the note gives none). A line that
/// starts below the ground is not blocked until it comes up and goes down
/// again (our choice, e.g. the aim point under deep water).
pub fn ground_line_hit(height: &dyn Fn(f32, f32) -> Option<f32>, from: [f32; 3], to: [f32; 3]) -> Option<[f32; 3]> {
    let at = |t: f32| -> [f32; 3] { std::array::from_fn(|k| from[k] + (to[k] - from[k]) * t) };
    let below = |p: [f32; 3]| height(p[0], p[1]).is_some_and(|h| p[2] < h);
    let steps = (length(sub(to, from)) / GROUND_STEP).ceil().max(1.0) as u32;
    let mut was_below = below(from);
    let mut t0 = 0.0;
    for i in 1..=steps {
        let t1 = i as f32 / steps as f32;
        let now = below(at(t1));
        if now && !was_below {
            let (mut lo, mut hi) = (t0, t1);
            for _ in 0..GROUND_REFINE {
                let mid = 0.5 * (lo + hi);
                if below(at(mid)) { hi = mid } else { lo = mid }
            }
            return Some(at(hi));
        }
        was_below = now;
        t0 = t1;
    }
    None
}

/// The followed object, for the obstacle rules.
#[derive(Debug, Clone, Copy)]
pub struct Followed {
    /// Its base (Connor's feet).
    pub base: [f32; 3],
}

/// An avoidance swing: turn `step` radians round the aim point every
/// update until the direction `target` (an angle round the vertical) is
/// reached or passed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Swing {
    pub step: f32,
    pub target: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// Stays put unless outside the min/max range.
    Normal,
    /// Keeps this distance to the aim point (F11).
    Fixed { distance: f32 },
}

/// This update's input. Mouse values are in the game's mouse units.
#[derive(Debug, Clone, Copy, Default)]
pub struct Input {
    /// Right mouse button held (`MoveCamera`).
    pub mouse_held: bool,
    /// Sideways mouse movement since the last update.
    pub mouse_x: f32,
    /// Forward (positive) / back mouse movement since the last update.
    pub mouse_forward: f32,
    /// Shift held (`ModifyDn`).
    pub shift: bool,
    /// Page Up (+1) / Page Down (−1) held (`RotateX`).
    pub tilt_keys: i8,
    /// Home pressed (`ResetX`).
    pub reset_tilt: bool,
    /// F11 pressed (`CameraChangeMode`).
    pub change_mode: bool,
}

#[derive(Debug, Clone)]
pub struct Camera {
    pub position: [f32; 3],
    pub tilt: f32,
    pub mode: Mode,
    /// (min, max) distance, set by `KQCamera::setRange`.
    pub range: (f32, f32),
    /// Connor's base at the previous camera update (the tilt reset).
    last_base: Option<[f32; 3]>,
    mouse_was_held: bool,
    /// Obstacle avoidance on (it is by default; nothing found switches it
    /// off).
    pub avoid: bool,
    pub swing: Option<Swing>,
    /// The previous update's offset (camera − aim), and the camera's
    /// previous record of the aim point (reset by `tick` after 3 still
    /// updates and by `not_moved`).
    prev_offset: Option<[f32; 3]>,
    prev_aim: Option<[f32; 3]>,
    /// The height rule lifted the camera off the ground and has not eased
    /// since (camera.md "Height above the ground" rule 2).
    lifted: bool,
    /// "Connor is moving" (see `tick`), and game updates without a "Connor
    /// moved" message.
    moving: bool,
    still: u32,
}

/// What one update did, for the log.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub aim: [f32; 3],
    pub distance: f32,
    pub terrain_below: Option<f32>,
    pub events: Vec<String>,
    /// The view test: where and by what the view line was blocked.
    pub blocked: Option<([f32; 3], String)>,
    /// Connor counted as moving for the obstacle rules this update.
    pub moving: bool,
    /// The swing after this update: (step, angle still to go).
    pub swing: Option<(f32, f32)>,
    /// Pulled in to this distance from the aim point.
    pub pulled_in: Option<f32>,
    /// What the height rule did.
    pub height: Option<HeightStep>,
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

/// `follow Connor` after a cutscene (camera.md "Shake and cutscene
/// cameras"): a view more than this far (radians) off Connor turns back
/// gradually; nearer, it looks at him at once.
pub const FOLLOW_TURN_FROM: f32 = 0.4;
/// The turn back snaps onto Connor once the rest is under this (one snap
/// angle on both sides, the note's "Do not reproduce").
pub const FOLLOW_TURN_SNAP: f32 = 0.063;

/// The view direction of `move KQCamera … rx ry rz` (camera.md "Shake and
/// cutscene cameras"): rx = tilt (positive looks up), rz = heading (0 looks
/// along +Y, positive anticlockwise seen from above):
/// (−sin rz × cos rx, cos rz × cos rx, sin rx).
pub fn view_direction(rx: f32, rz: f32) -> [f32; 3] {
    let (sx, cx) = rx.sin_cos();
    let (sz, cz) = rz.sin_cos();
    [-sz * cx, cz * cx, sx]
}

/// The (tilt, heading) of a view direction, the reverse of
/// [`view_direction`].
pub fn view_angles(d: [f32; 3]) -> (f32, f32) {
    let flat = (d[0] * d[0] + d[1] * d[1]).sqrt();
    (d[2].atan2(flat), (-d[0]).atan2(d[1]))
}

/// The heading offset a turn back toward Connor starts with: the whole
/// offset if it is more than [`FOLLOW_TURN_FROM`], else none.
pub fn follow_turn_start(offset: f32) -> f32 {
    let offset = wrap(offset);
    if offset.abs() > FOLLOW_TURN_FROM { offset } else { 0.0 }
}

/// One update of the turn back: a third of what is left, then the snap.
pub fn follow_turn_step(offset: f32) -> f32 {
    let left = offset * (2.0 / 3.0);
    if left.abs() < FOLLOW_TURN_SNAP { 0.0 } else { left }
}

/// The angle of `v` round the vertical (atan2 of y, x).
fn azimuth(v: [f32; 3]) -> f32 {
    v[1].atan2(v[0])
}

/// An angle brought into −π..π.
pub fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let a = a.rem_euclid(t);
    if a > std::f32::consts::PI { a - t } else { a }
}

/// `v` turned by `a` round the vertical.
fn turn_z(v: [f32; 3], a: f32) -> [f32; 3] {
    let (s, c) = a.sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
}

/// `aim + direction(from aim towards camera) × distance`.
fn at_distance(aim: [f32; 3], camera: [f32; 3], distance: f32) -> [f32; 3] {
    let d = sub(camera, aim);
    let len = length(d);
    let dir = if len < 1e-3 { [0.0, 1.0, 0.0] } else { d.map(|v| v / len) };
    std::array::from_fn(|i| aim[i] + dir[i] * distance)
}

impl Camera {
    pub fn new(position: [f32; 3]) -> Self {
        Camera {
            position,
            tilt: DEFAULT_TILT,
            mode: Mode::Normal,
            range: DEFAULT_RANGE,
            last_base: None,
            mouse_was_held: false,
            avoid: true,
            swing: None,
            prev_offset: None,
            prev_aim: None,
            lifted: false,
            moving: false,
            still: 0,
        }
    }

    /// `KQCamera::setRange <min> <max>`: refused unless min < max.
    pub fn set_range(&mut self, min: f32, max: f32) -> bool {
        if min < max {
            self.range = (min, max);
            true
        } else {
            false
        }
    }

    /// One game update (tick) and one camera update following an object
    /// whose base is at `base`, without obstacles; "Connor moved" if the
    /// base changed since the last call (tests).
    pub fn update(&mut self, base: [f32; 3], input: &Input, ground: &dyn GroundLookup) -> Report {
        let moved = self.last_base.is_some_and(|l| connor_moved((l, 0.0), (base, 0.0)));
        self.tick(moved, base, input.mouse_held);
        self.update_with(Followed { base }, moved, input, ground, None)
    }

    /// The aim point for an object whose base is at `base`.
    pub fn aim_for(base: [f32; 3]) -> [f32; 3] {
        [base[0], base[1], base[2] + AIM_HEIGHT]
    }

    /// Once per game update, before any camera update (camera.md "Connor
    /// is moving"): `moved` = a "Connor moved" message came this update.
    /// After 3 updates without one he counts as still and the camera's
    /// previous record of the aim point is reset to where he stands. Also
    /// forgets the mouse button once it is let go (the first update after
    /// pressing uses no movement). Returns true when he has just become
    /// still (for the log).
    pub fn tick(&mut self, moved: bool, base: [f32; 3], mouse_held: bool) -> bool {
        if !mouse_held {
            self.mouse_was_held = false;
        }
        if moved {
            self.still = 0;
            return false;
        }
        self.still += 1;
        if self.still == STILL_UPDATES {
            self.moving = false;
            self.prev_aim = Some(Self::aim_for(base));
            return true;
        }
        false
    }

    /// `follow` (and a flight or cut): the previous record of the aim point
    /// becomes the present one, so this update counts as "not moved".
    pub fn not_moved(&mut self, base: [f32; 3]) {
        self.prev_aim = Some(Self::aim_for(base));
    }

    /// One camera update, with the obstacle rules (camera.md "Obstacles",
    /// steps 6–9) when `view` is given. `moved`: a "Connor moved" message is
    /// among this update's causes.
    pub fn update_with(&mut self, followed: Followed, moved: bool, input: &Input, ground: &dyn GroundLookup, view: Option<&dyn ViewBlockers>) -> Report {
        let base = followed.base;
        let start = self.position;
        let mut report = Report::default();
        // "Connor is moving": from the update after his first "moved"
        // message until 3 updates have passed without one (`tick`).
        let moving_now = self.moving;
        if moved {
            self.moving = true;
        }
        report.moving = moving_now;
        let aim = Self::aim_for(base);
        let (min, max) = self.range;

        if input.change_mode {
            self.mode = match self.mode {
                Mode::Normal => Mode::Fixed { distance: length(sub(self.position, aim)).clamp(min + 1.0, max - 1.0) },
                Mode::Fixed { .. } => Mode::Normal,
            };
            report.events.push(format!("camera mode -> {:?}", self.mode));
        }
        if input.reset_tilt {
            self.tilt = DEFAULT_TILT;
        }
        // Page Up/Down: tilt in normal mode, zoom in fixed mode.
        if input.tilt_keys != 0 {
            match &mut self.mode {
                Mode::Normal => self.tilt += f32::from(input.tilt_keys) * TILT_PER_UPDATE,
                Mode::Fixed { distance } => {
                    *distance = (*distance - f32::from(input.tilt_keys) * FIXED_ZOOM_PER_UPDATE).clamp(min + 1.0, max - 1.0)
                }
            }
        }

        // Mouse, only while the button is held; the first update after
        // pressing uses no movement.
        let (mx, mf) = if input.mouse_held && self.mouse_was_held { (input.mouse_x, input.mouse_forward) } else { (0.0, 0.0) };
        self.mouse_was_held = input.mouse_held;
        if mx != 0.0 {
            // Swing round the aim point about the vertical axis.
            let a = mx * SWING_PER_UNIT;
            let (s, c) = a.sin_cos();
            let [dx, dy, _] = sub(self.position, aim);
            self.position[0] = aim[0] + dx * c - dy * s;
            self.position[1] = aim[1] + dx * s + dy * c;
        }
        if mf != 0.0 {
            let zooms = matches!(self.mode, Mode::Normal) != input.shift;
            if !zooms {
                // Opposite sign to the zoom: mouse down (positive) tilts
                // the way Page Down does (checked by eye by the user
                // against the original, 2026-10-11).
                self.tilt -= mf * TILT_PER_UNIT;
            } else {
                match &mut self.mode {
                    Mode::Normal => {
                        let before = sub(self.position, aim);
                        let new = length(before) - mf * ZOOM_PER_UNIT;
                        // Never past the aim point: then the minimum, same side.
                        let new = if new <= 0.0 { min + 1.0 } else { new };
                        self.position = at_distance(aim, self.position, new);
                    }
                    Mode::Fixed { distance } => {
                        *distance = (*distance - mf * ZOOM_PER_UNIT).clamp(min + 1.0, max - 1.0);
                    }
                }
            }
        }

        // Step 3: the leash (maximum), or the fixed distance.
        let d = length(sub(self.position, aim));
        match self.mode {
            Mode::Normal if d > max => self.position = at_distance(aim, self.position, max - 1.0),
            Mode::Normal => {}
            Mode::Fixed { distance } => self.position = at_distance(aim, self.position, distance),
        }

        // Step 4: the height (camera.md "Height above the ground" rule 2).
        // "Moved" by the sum of sizes, not the original's signed sum ("Do
        // not reproduce").
        let sum = self.prev_aim.map_or(0.0, |p| (0..3).map(|k| (aim[k] - p[k]).abs()).sum());
        let floor = |x: f32, y: f32| ground.ground(x, y).map(|g| g.0);
        let (p, h) = height_rule(start, self.position, base[2], max, sum, height_share(UPDATE_SECONDS), mx != 0.0, self.lifted, &floor);
        self.position = p;
        if h.lift != Lift::No {
            self.lifted = true;
        }
        if h.then == HeightThen::Eased {
            self.lifted = false;
        }
        report.height = Some(h);

        // Step 5: pushed out to the minimum.
        if matches!(self.mode, Mode::Normal) && length(sub(self.position, aim)) < min {
            self.position = at_distance(aim, self.position, min + 1.0);
        }
        report.terrain_below = floor(self.position[0], self.position[1]);

        if let Some(view) = view {
            self.avoid_obstacles(aim, base, mx != 0.0 || mf != 0.0, moving_now, ground, view, &mut report);
        }
        self.prev_offset = Some(sub(self.position, aim));
        self.prev_aim = Some(aim);
        report.swing = self.swing.map(|s| (s.step, wrap(s.target - azimuth(sub(self.position, aim)))));

        // Tilt snaps back once the followed object moves (normal mode).
        if let (Mode::Normal, Some(last)) = (self.mode, self.last_base) {
            let moved = (base[0] - last[0]).abs() + (base[1] - last[1]).abs();
            if moved > MOVE_RESETS_TILT && self.tilt != DEFAULT_TILT && mf == 0.0 {
                self.tilt = DEFAULT_TILT;
                report.events.push("tilt back to default (Connor moved)".into());
            }
        }
        self.last_base = Some(base);
        self.tilt = self.tilt.clamp(-TILT_LIMIT, TILT_LIMIT);
        report.aim = aim;
        report.distance = length(sub(self.position, aim));
        report
    }

    /// camera.md "Obstacles", steps 6–9.
    #[allow(clippy::too_many_arguments)]
    fn avoid_obstacles(
        &mut self,
        aim: [f32; 3],
        base: [f32; 3],
        mouse_used: bool,
        moving: bool,
        ground: &dyn GroundLookup,
        view: &dyn ViewBlockers,
        report: &mut Report,
    ) {
        let (min, _) = self.range;
        // 6. A running swing turns the camera round the aim point, or is
        // cancelled by the mouse.
        if let Some(sw) = self.swing {
            if mouse_used {
                self.swing = None;
                report.events.push("avoidance swing cancelled (mouse)".into());
            } else {
                let offset = turn_z(sub(self.position, aim), sw.step);
                self.position = add(aim, offset);
                let left = wrap(sw.target - azimuth(offset));
                if left == 0.0 || left.signum() != sw.step.signum() {
                    self.swing = None;
                    report.events.push(format!("avoidance swing ended ({:.3} rad past the path he came by)", -left));
                }
            }
        }
        // 7. The view test.
        let Some((hit, what)) = view.first_block(aim, self.position) else { return };
        report.blocked = Some((hit, what));
        // 8.1 Dragged into a wall: back to the previous offset, 5% shorter
        // unless that is under the minimum.
        if mouse_used {
            if let Some(prev) = self.prev_offset {
                let short = prev.map(|v| v * DRAG_SHORTEN);
                self.position = add(aim, if length(short) < min { prev } else { short });
                report.events.push("dragged into a wall: previous offset kept, shortened 5%".into());
            }
        }
        // 8.2 Only when avoidance is on, Connor is moving and his feet are
        // not under the ground (floor grid).
        let feet_ok = ground.ground(base[0], base[1]).is_none_or(|g| base[2] >= g.0);
        if !(self.avoid && moving && feet_ok) {
            return;
        }
        let a = self.prev_aim.unwrap_or(aim);
        let m: [f32; 3] = std::array::from_fn(|k| a[k] + (hit[k] - a[k]) * AVOID_MIX);
        let old = sub(self.position, aim);
        let len = length(old);
        let towards = sub(m, aim);
        if length(towards) < 1e-3 || len < 1e-3 {
            return;
        }
        let new = towards.map(|v| v / length(towards) * len);
        self.position = add(aim, new);
        let first_turn = wrap(azimuth(new) - azimuth(old));
        // The swing ends looking along the path he came by (towards A).
        let back = sub(a, aim);
        self.swing = (back[0].abs() + back[1].abs() > 1e-3).then(|| Swing { step: SWING_SHARE * first_turn, target: azimuth(back) });
        report.events.push(format!("view blocked: turned {first_turn:.3} rad towards the path he came by; swing {:?}", self.swing));
        // Still blocked: pulled in to 100 short of the new hit.
        if let Some((hit2, what2)) = view.first_block(aim, self.position) {
            let d = (length(sub(hit2, aim)) - PULL_IN_MARGIN).max(PULL_IN_MIN);
            self.position = at_distance(aim, self.position, d);
            self.swing = None;
            report.pulled_in = Some(d);
            report.events.push(format!("still blocked by {what2}: pulled in to {d:.1}"));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Flat(f32);
    impl GroundLookup for Flat {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((self.0, [0.0, 0.0, 1.0]))
        }
    }

    #[test]
    fn cutscene_angles_give_the_view_and_back() {
        // Heading 0 looks along +Y, π/2 along −X; negative tilt looks down.
        let d = view_direction(0.0, 0.0);
        assert!((d[1] - 1.0).abs() < 1e-6 && d[0].abs() < 1e-6);
        let d = view_direction(0.0, std::f32::consts::FRAC_PI_2);
        assert!((d[0] + 1.0).abs() < 1e-6, "{d:?}");
        assert!(view_direction(-0.2, 0.0)[2] < 0.0);
        // The watermill shot (wtrmwalk.cs): heading 2.632 points from the
        // camera at Connor's walkTo target (about 2.64).
        let (tilt, heading) = view_angles(view_direction(-0.2, 2.632));
        assert!((tilt + 0.2).abs() < 1e-5 && (heading - 2.632).abs() < 1e-5);
        let to_target = view_angles([190800.0 - 192100.0, 230100.0 - 232500.0, 0.0]).1;
        assert!((wrap(to_target - 2.632)).abs() < 0.05, "{to_target}");
    }

    #[test]
    fn follow_turns_back_by_a_third_per_update_then_snaps() {
        // 0.19 rad off (the bricks' cue 10): looks at Connor at once.
        assert_eq!(follow_turn_start(0.19), 0.0);
        let mut o = follow_turn_start(-1.0);
        assert_eq!(o, -1.0);
        let mut steps = 0;
        while o != 0.0 {
            let next = follow_turn_step(o);
            assert!(next == 0.0 || (next - o * 2.0 / 3.0).abs() < 1e-6);
            o = next;
            steps += 1;
        }
        // 1.0 × (2/3)^n < 0.063 from n = 7.
        assert_eq!(steps, 7);
    }

    fn horizontal(c: &Camera, aim: [f32; 3]) -> f32 {
        ((c.position[0] - aim[0]).powi(2) + (c.position[1] - aim[1]).powi(2)).sqrt()
    }

    /// Where a camera at (0, `y`, MIN_HEIGHT) ends up horizontally when it
    /// is moved along the line to the aim point (Connor at 0, 0, 0) until
    /// it is `d` from it in 3D. The height rule afterwards changes z only.
    fn horizontal_at(y: f32, d: f32) -> f32 {
        let dz = MIN_HEIGHT - AIM_HEIGHT;
        d * y / (y * y + dz * dz).sqrt()
    }

    #[test]
    fn pulled_in_on_a_leash_and_pushed_out() {
        let (min, max) = DEFAULT_RANGE;
        let aim = [0.0, 0.0, AIM_HEIGHT];
        let mut c = Camera::new([0.0, 9000.0, MIN_HEIGHT]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert!((horizontal(&c, aim) - horizontal_at(9000.0, max - 1.0)).abs() < 1.0, "{:?}", c.position);
        let mut c = Camera::new([0.0, 300.0, MIN_HEIGHT]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert!((horizontal(&c, aim) - horizontal_at(300.0, min + 1.0)).abs() < 1.0, "{:?}", c.position);
        // Inside the range, at its wanted height: it stays where it is.
        let mut c = Camera::new([0.0, 3000.0, MIN_HEIGHT]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert_eq!(c.position, [0.0, 3000.0, MIN_HEIGHT]);
    }

    #[test]
    fn height_lifted_at_once_snapped_within_100_and_kept_while_he_stands() {
        // Under the ground + 1,400: lifted at once.
        let mut c = Camera::new([0.0, 3000.0, 400.0]);
        let r = c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert_eq!(c.position[2], MIN_HEIGHT);
        assert_eq!(r.height.unwrap().lift, Lift::ToGround);
        // Within 100: snapped.
        let mut c = Camera::new([0.0, 3000.0, MIN_HEIGHT + 90.0]);
        let r = c.update([0.0; 3], &Input::default(), &Flat(0.0));
        assert_eq!((c.position[2], r.height.unwrap().then), (MIN_HEIGHT, HeightThen::Snapped));
        // Terrain under the camera higher than Connor's base wins.
        let mut c = Camera::new([0.0, 1500.0, 4000.0 + MIN_HEIGHT + 50.0]);
        c.update([0.0; 3], &Input::default(), &Flat(4000.0));
        assert_eq!(c.position[2], 4000.0 + MIN_HEIGHT);
        // Far above it and Connor standing still: kept, however far.
        let mut c = Camera::new([0.0, 3000.0, 3000.0]);
        for _ in 0..5 {
            let r = c.update([0.0; 3], &Input::default(), &Flat(0.0));
            assert_eq!(r.height.unwrap().then, HeightThen::Kept);
        }
        assert_eq!(c.position[2], 3000.0);
    }

    #[test]
    fn height_eases_10_percent_only_when_the_aim_moved_more_than_20() {
        let mut c = Camera::new([0.0, 3000.0, 3000.0]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        // 15: not moved.
        let r = c.update([15.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert_eq!((r.height.unwrap().then, c.position[2]), (HeightThen::Kept, 3000.0));
        // 75: eased 10% of the gap.
        let r = c.update([90.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        let h = r.height.unwrap();
        assert_eq!((h.then, h.sum), (HeightThen::Eased, 75.0));
        assert!((c.position[2] - (3000.0 - 0.1 * (3000.0 - MIN_HEIGHT))).abs() < 0.01, "{}", c.position[2]);
        // +300 in x and −300 in y: the original's signed sum says 0; the
        // sum of sizes says moved ("Do not reproduce").
        let r = c.update([390.0, -300.0, 0.0], &Input::default(), &Flat(0.0));
        assert_eq!(r.height.unwrap().then, HeightThen::Eased);
        assert!((height_share(UPDATE_SECONDS) - 0.1).abs() < 1e-6);
        assert!((height_share(1.0) - (1.0 - 0.9f32.powi(15))).abs() < 1e-5);
    }

    #[test]
    fn height_eases_from_the_start_of_the_update_and_lifts_along_the_line_when_deep() {
        let flat = |_: f32, _: f32| Some(0.0);
        // The leash lowered it to 2,500 this update: eased from 3,000.
        let (p, h) = height_rule([0.0, 0.0, 3000.0], [0.0, 10.0, 2500.0], 0.0, 5000.0, 75.0, 0.1, false, false, &flat);
        assert_eq!(h.then, HeightThen::Eased);
        assert!((p[2] - (3000.0 + 0.1 * (MIN_HEIGHT - 2500.0))).abs() < 0.01, "{p:?}");
        // A cliff 10,000 high from x = 500: the new spot is 10,000 under
        // ground + 1,400, more than the maximum: put where the line meets it.
        let cliff = |x: f32, _: f32| Some(if x < 500.0 { 0.0 } else { 10_000.0 });
        let (p, h) = height_rule([0.0, 0.0, MIN_HEIGHT], [1000.0, 0.0, MIN_HEIGHT], 0.0, 5000.0, 0.0, 0.1, false, false, &cliff);
        assert_eq!(h.lift, Lift::AlongLine);
        assert!((p[0] - 500.0).abs() < 0.1 && p[2] == MIN_HEIGHT, "{p:?}");
    }

    #[test]
    fn swing_keeps_the_distance() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        let held = Input { mouse_held: true, mouse_x: 1.0, ..Input::default() };
        c.update([0.0; 3], &held, &Flat(0.0)); // first update: no movement
        assert_eq!(c.position, [0.0, 3000.0, 1400.0]);
        c.update([0.0; 3], &held, &Flat(0.0));
        assert!((horizontal(&c, [0.0; 3]) - 3000.0).abs() < 0.5);
        assert!(c.position[0].abs() > 1000.0, "{:?}", c.position);
    }

    #[test]
    fn zoom_stays_in_range_and_never_passes_the_aim() {
        let (min, _) = DEFAULT_RANGE;
        let mut c = Camera::new([0.0, 3000.0, MIN_HEIGHT]);
        let zoom_in = Input { mouse_held: true, mouse_forward: 10.0, ..Input::default() };
        c.update([0.0; 3], &zoom_in, &Flat(0.0));
        c.update([0.0; 3], &zoom_in, &Flat(0.0));
        let h = horizontal(&c, [0.0, 0.0, AIM_HEIGHT]);
        assert!((h - horizontal_at(3000.0, min + 1.0)).abs() < 1.0, "{:?}", c.position);
        assert!(c.position[1] > 0.0, "same side: {:?}", c.position);
    }

    #[test]
    fn fixed_mode_keeps_its_distance() {
        // The distance kept is the 3D distance to the aim point when F11
        // is pressed.
        let d = (3000.0f32.powi(2) + (MIN_HEIGHT - AIM_HEIGHT).powi(2)).sqrt();
        let mut c = Camera::new([0.0, 3000.0, MIN_HEIGHT]);
        c.update([0.0; 3], &Input { change_mode: true, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.mode, Mode::Fixed { distance: d });
        // Connor moves 2,000: the camera is put back at that distance, then
        // the height rule moves it up or down, which changes the distance
        // by less than the gap between the aim and the wanted height.
        let r = c.update([2000.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert!((r.distance - d).abs() <= (MIN_HEIGHT - AIM_HEIGHT).abs() + 1.0, "{}", r.distance);
        // Page Up in fixed mode: 125 closer.
        c.update([2000.0, 0.0, 0.0], &Input { tilt_keys: 1, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.mode, Mode::Fixed { distance: d - FIXED_ZOOM_PER_UPDATE });
    }

    #[test]
    fn tilt_keys_and_reset() {
        let mut c = Camera::new([0.0, 3000.0, 1400.0]);
        for _ in 0..15 {
            c.update([0.0; 3], &Input { tilt_keys: 1, ..Input::default() }, &Flat(0.0));
        }
        assert!((c.tilt - (DEFAULT_TILT + 0.225)).abs() < 1e-4, "{}", c.tilt);
        // Connor moves 75 (a walking update): back to default.
        c.update([75.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert_eq!(c.tilt, DEFAULT_TILT);
        c.tilt = 1.0;
        c.update([75.0, 0.0, 0.0], &Input { reset_tilt: true, ..Input::default() }, &Flat(0.0));
        assert_eq!(c.tilt, DEFAULT_TILT);
    }

    /// Upright walls of any height, as 2D segments.
    struct Walls(Vec<([f32; 2], [f32; 2])>);
    impl ViewBlockers for Walls {
        fn first_block(&self, from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], String)> {
            let mut best: Option<f32> = None;
            for (p, q) in &self.0 {
                let (d, e) = ([to[0] - from[0], to[1] - from[1]], [q[0] - p[0], q[1] - p[1]]);
                let den = d[0] * e[1] - d[1] * e[0];
                if den.abs() < 1e-9 {
                    continue;
                }
                let w = [p[0] - from[0], p[1] - from[1]];
                let t = (w[0] * e[1] - w[1] * e[0]) / den;
                let u = (w[0] * d[1] - w[1] * d[0]) / den;
                if (0.0..=1.0).contains(&t) && (0.0..=1.0).contains(&u) && best.is_none_or(|b| t < b) {
                    best = Some(t);
                }
            }
            best.map(|t| (std::array::from_fn(|k| from[k] + (to[k] - from[k]) * t), "wall".into()))
        }
    }

    /// One game update and one camera update with `view`, Connor at (x, y, 0).
    fn step(c: &mut Camera, x: f32, y: f32, input: &Input, view: &dyn ViewBlockers) -> Report {
        let base = [x, y, 0.0];
        let moved = c.last_base.is_some_and(|l| connor_moved((l, 0.0), (base, 0.0)));
        c.tick(moved, base, input.mouse_held);
        c.update_with(Followed { base }, moved, input, &Flat(0.0), Some(view))
    }

    #[test]
    fn standing_still_behind_a_wall_nothing_moves() {
        let wall = Walls(vec![([-5000.0, -1000.0], [5000.0, -1000.0])]);
        let mut c = Camera::new([0.0, -3000.0, MIN_HEIGHT]);
        for _ in 0..5 {
            let r = step(&mut c, 0.0, 0.0, &Input::default(), &wall);
            assert!(r.blocked.is_some() && !r.moving);
        }
        assert_eq!(c.position, [0.0, -3000.0, MIN_HEIGHT]);
    }

    #[test]
    fn walking_with_a_wall_behind_pulls_the_camera_in_100_short() {
        let wall = Walls(vec![([-50000.0, -1000.0], [50000.0, -1000.0])]);
        let mut c = Camera::new([0.0, -3000.0, MIN_HEIGHT]);
        // The first update he moves in doesn't count as moving yet.
        let mut r = step(&mut c, 0.0, 0.0, &Input::default(), &wall);
        for x in [75.0, 150.0] {
            r = step(&mut c, x, 0.0, &Input::default(), &wall);
        }
        assert!(r.moving && r.blocked.is_some());
        let d = r.pulled_in.expect("pulled in");
        // In front of the wall now, 100 short of where the line met it.
        assert!(c.position[1] > -1000.0, "{:?}", c.position);
        assert!(c.swing.is_none());
        assert!((d - r.distance).abs() < 0.01);
    }

    #[test]
    fn the_camera_swings_round_to_look_along_his_path() {
        // A short wall straight between the camera and the aim at update 3.
        let wall = Walls(vec![([-500.0, -1000.0], [500.0, -1000.0])]);
        let mut c = Camera::new([0.0, -3000.0, MIN_HEIGHT]);
        c.set_range(100.0, 50000.0);
        step(&mut c, -2000.0, 0.0, &Input::default(), &wall);
        step(&mut c, -1000.0, 0.0, &Input::default(), &wall);
        let r = step(&mut c, 0.0, 0.0, &Input::default(), &wall);
        assert!(r.blocked.is_some() && r.pulled_in.is_none(), "{r:?}");
        let sw = c.swing.expect("swing started");
        // From straight behind (-90°) towards halfway to A (-1000, 0): -135°, so the
        // swing turns clockwise by 0.66 × 45° per update.
        let az = azimuth(sub(c.position, r.aim));
        assert!((az + 3.0 * std::f32::consts::FRAC_PI_4).abs() < 0.05, "{az}");
        assert!((sw.step + SWING_SHARE * std::f32::consts::FRAC_PI_4).abs() < 0.05, "{sw:?}");
        // He stops; the swing goes on and ends once it reaches or passes
        // the direction he came from (180°).
        let mut updates = 0;
        while c.swing.is_some() && updates < 20 {
            step(&mut c, 0.0, 0.0, &Input::default(), &wall);
            updates += 1;
        }
        assert!(c.swing.is_none() && updates <= 3, "{updates}");
        let az = azimuth(sub(c.position, [0.0, 0.0, AIM_HEIGHT]));
        assert!(wrap(az - std::f32::consts::PI).abs() <= sw.step.abs() + 1e-3, "{az}");
    }

    #[test]
    fn dragging_into_a_wall_creeps_in_5_percent_never_under_the_minimum() {
        let wall = Walls(vec![([-5000.0, -1000.0], [5000.0, -1000.0])]);
        let mut c = Camera::new([0.0, -900.0, MIN_HEIGHT]);
        c.set_range(100.0, 5000.0);
        let zoom_out = Input { mouse_held: true, mouse_forward: -0.2, ..Input::default() };
        step(&mut c, 0.0, 0.0, &zoom_out, &wall); // first update: no movement
        let before = sub(c.position, [0.0, 0.0, AIM_HEIGHT]);
        let r = step(&mut c, 0.0, 0.0, &zoom_out, &wall);
        assert!(r.blocked.is_some());
        let after = sub(c.position, r.aim);
        for k in 0..3 {
            assert!((after[k] - before[k] * DRAG_SHORTEN).abs() < 0.01, "{after:?} vs {before:?}");
        }
        // With the minimum just under the current distance, it stays put.
        c.set_range(length(after) - 1.0, 5000.0);
        let r = step(&mut c, 0.0, 0.0, &zoom_out, &wall);
        assert!(r.blocked.is_some());
        let kept = sub(c.position, r.aim);
        assert!((0..3).all(|k| (kept[k] - after[k]).abs() < 0.01), "{kept:?} vs {after:?}");
    }

    #[test]
    fn a_hit_closer_than_100_keeps_the_camera_in_front() {
        let wall = Walls(vec![([-50000.0, -30.0], [50000.0, -30.0])]);
        let mut c = Camera::new([0.0, -3000.0, MIN_HEIGHT]);
        for x in [0.0, 75.0, 150.0] {
            step(&mut c, x, 0.0, &Input::default(), &wall);
        }
        let d = length(sub(c.position, [150.0, 0.0, AIM_HEIGHT]));
        assert!((d - PULL_IN_MIN).abs() < 0.01, "{d}");
        assert!(c.position[1] < 0.0, "same side as before: {:?}", c.position);
    }

    #[test]
    fn the_ground_blocks_where_the_line_goes_into_it() {
        // A ridge 1,000 high between x = 1,000 and x = 2,000.
        let ridge = |x: f32, _: f32| Some(if (1000.0..2000.0).contains(&x) { 1000.0 } else { 0.0 });
        let hit = ground_line_hit(&ridge, [0.0, 0.0, 800.0], [3000.0, 0.0, 800.0]).expect("blocked");
        assert!((hit[0] - 1000.0).abs() < 0.1 && hit[2] == 800.0, "{hit:?}");
        assert!(ground_line_hit(&ridge, [0.0, 0.0, 1200.0], [3000.0, 0.0, 1200.0]).is_none());
        // Starting under water (below the surface) and rising out: not blocked.
        let lake = |_: f32, _: f32| Some(1000.0);
        assert!(ground_line_hit(&lake, [0.0, 0.0, 800.0], [3000.0, 0.0, 2000.0]).is_none());
        // Outside the grid: no ground.
        assert!(ground_line_hit(&|_, _| None, [0.0, 0.0, 0.0], [3000.0, 0.0, -100.0]).is_none());
    }

    #[test]
    fn moved_needs_more_than_0_1_on_an_axis_or_a_turn() {
        assert!(!connor_moved(([0.0; 3], 0.0), ([0.1, -0.1, 0.1], 0.0)));
        assert!(connor_moved(([0.0; 3], 0.0), ([0.0, 0.0, 0.11], 0.0)));
        assert!(!connor_moved(([0.0; 3], 1.0), ([0.0; 3], 1.00005)));
        assert!(connor_moved(([0.0; 3], 1.0), ([0.0; 3], 1.001)));
    }

    #[test]
    fn still_after_3_updates_without_a_message_and_the_aim_record_reset() {
        let mut c = Camera::new([0.0, 3000.0, MIN_HEIGHT]);
        c.update([0.0; 3], &Input::default(), &Flat(0.0));
        let r = c.update([75.0, 0.0, 0.0], &Input::default(), &Flat(0.0));
        assert!(!r.moving, "the first update of a walk doesn't count as moving");
        assert!(c.moving);
        // He stops: 2 ticks still moving, the 3rd makes him still.
        assert!(!c.tick(false, [75.0, 0.0, 0.0], false));
        assert!(!c.tick(false, [75.0, 0.0, 0.0], false));
        assert!(c.moving);
        assert!(c.tick(false, [75.0, 0.0, 0.0], false));
        assert!(!c.moving);
        assert_eq!(c.prev_aim, Some(Camera::aim_for([75.0, 0.0, 0.0])));
    }

    #[test]
    fn set_range_refuses_min_not_below_max() {
        let mut c = Camera::new([0.0; 3]);
        assert!(!c.set_range(2500.0, 1000.0));
        assert_eq!(c.range, DEFAULT_RANGE);
        assert!(c.set_range(1000.0, 2500.0));
    }
}
