//! The Rope and Hook (M6 6a): the rope places' settings, Connor's rope
//! status and the hook icons.
//!
//! Rules from `docs/behaviour/ropeandhook.md` (session A, 2026-10-21;
//! checked by the user in the original):
//!
//! - Rule 1: a rope place is a pair of trap boxes whose `On` scripts run
//!   `KQConner::setRope Bottom|Top x y z rx ry rz` and whose `Off`
//!   scripts run `KQConner::setRope None`.
//! - Rule 2: `setRope` takes `None`, `Top` or `Bottom` (any case) and 1,
//!   4 or 7 words after the command (place and direction 0 when left
//!   out); anything else prints the usage line and changes nothing. The
//!   place and direction are stored only when the change is accepted
//!   (rule 4); the two `setRopeCam` spots are cleared on every `setRope`.
//! - Rule 3: `setRopeCam x1 y1 z1 x2 y2 z2` (a spot counts as set only
//!   with all three coordinates non-zero), `setRopeCam2 x y z` (the
//!   camera when the rope is taken at the top), `setRopeCamTopLimit z` /
//!   `setRopeCamBottomLimit z` (0 = no limit), `setRopeCamAngle angle
//!   range` (kept, never used: "Do not reproduce", the aim test always
//!   passes). Only `setRope None` clears the limits; the rest is sticky.
//! - Rule 4: the status changes only if Connor carries an item of
//!   category `Rope`; otherwise both hook icons are hidden and nothing
//!   else changes (`None` too).
//! - Rule 5: the game time of each accepted change is noted (nothing
//!   reads it: "Do not reproduce"; kept for the log only).
//! - Rule 6: an accepted `setRope None` also puts a held rope back and
//!   clears both camera limits.
//! - Rule 7: `HookUp` while the status is Bottom, `HookDown` while Top,
//!   none for None or while Connor is on the rope.
//!
//! The record is one plain struct of Connor's, so a saved game can keep
//! it later (rule 32; we have no saved games yet).

use crate::potions::StatusIcon;

/// Connor's rope status (rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    None,
    /// At the top of a rope: the rope can be let down.
    Top,
    /// At the foot of a rope: the rope can be thrown up.
    Bottom,
}

impl Status {
    /// The word as `setRope` takes it, any case.
    pub fn from_word(w: &str) -> Option<Status> {
        match w.to_ascii_lowercase().as_str() {
            "none" => Some(Status::None),
            "top" => Some(Status::Top),
            "bottom" => Some(Status::Bottom),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Status::None => "none",
            Status::Top => "top",
            Status::Bottom => "bottom",
        }
    }
}

/// The two hook icons (rule 7), shown in the status icon places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    /// At the foot: `INVITEM_HookUp` (`hookup.pba`).
    Up,
    /// At the top: `INVITEM_HookDown` (`hookdown.pba`).
    Down,
}

impl Hook {
    /// The item type whose picture the icon shows (`Invinfo.itl`).
    pub fn item_type(self) -> &'static str {
        match self {
            Hook::Up => "INVITEM_HookUp",
            Hook::Down => "INVITEM_HookDown",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Hook::Up => "HookUp",
            Hook::Down => "HookDown",
        }
    }
}

/// The category of the rope's item type (rule 4; `INVITEM_RopeAndHook`).
pub const ROPE_CATEGORY: &str = "Rope";

/// Connor's rope settings (rules 1–6): set by the rope places' scripts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rope {
    pub status: Status,
    /// The rope's place and direction (radians, as an object's
    /// `dirX/dirY/dirZ`), stored only with an accepted change.
    pub place: [f32; 3],
    pub dir: [f32; 3],
    /// `setRopeCam`'s two spots; `None` when not set (a 0 coordinate).
    pub cam: [Option<[f32; 3]>; 2],
    /// `setRopeCam2`: the camera when the rope is taken at the top.
    pub top_cam: Option<[f32; 3]>,
    /// `setRopeCamAngle`: heading and tolerance (unused, "Do not
    /// reproduce").
    pub angle: Option<(f32, f32)>,
    /// `setRopeCamTopLimit` / `BottomLimit`; 0 = no limit.
    pub top_limit: f32,
    pub bottom_limit: f32,
    /// Game time of the last accepted change (rule 5; only logged).
    pub changed_at: Option<f64>,
}

/// What a command did, for the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Done {
    /// The result for `$Console::Return`.
    pub result: &'static str,
    /// The line for `rope.log`.
    pub line: String,
    /// An accepted `setRope None`: a held rope goes back (rule 6).
    pub put_back: bool,
}

fn done(result: &'static str, line: String) -> Done {
    Done { result, line, put_back: false }
}

/// The numbers of `words`, or `None` if one isn't a number.
fn numbers(words: &[String]) -> Option<Vec<f32>> {
    words.iter().map(|w| crate::script::looks_like_number(w).then(|| crate::script::number_value(w) as f32)).collect()
}

fn spot(v: &[f32]) -> Option<[f32; 3]> {
    let s = [v[0], v[1], v[2]];
    s.iter().all(|c| *c != 0.0).then_some(s)
}

fn fmt3(v: [f32; 3]) -> String {
    format!("({:.0}, {:.0}, {:.0})", v[0], v[1], v[2])
}

fn fmt_spot(s: Option<[f32; 3]>) -> String {
    s.map_or("not set".to_string(), fmt3)
}

impl Rope {
    /// One of the six `KQConner::setRope*` commands (`words[0]` is the
    /// command); `carries`: Connor carries an item of category `Rope`;
    /// `now`: game time. `None` for another command.
    pub fn command(&mut self, words: &[String], carries: bool, now: f64) -> Option<Done> {
        let args = &words[1..];
        let usage = |u: &str| done("1", format!("usage: {u}: '{}'; nothing changes", words.join(" ")));
        Some(match words[0].as_str() {
            "KQConner::setRope" => {
                const USAGE: &str = "KQConner::setRope None|Top|Bottom [x y z [rx ry rz]]";
                let Some(status) = args.first().and_then(|w| Status::from_word(w)) else { return Some(usage(USAGE)) };
                if !matches!(args.len(), 1 | 4 | 7) {
                    return Some(usage(USAGE));
                }
                let Some(v) = numbers(&args[1..]) else { return Some(usage(USAGE)) };
                let place = if v.len() >= 3 { [v[0], v[1], v[2]] } else { [0.0; 3] };
                let dir = if v.len() == 6 { [v[3], v[4], v[5]] } else { [0.0; 3] };
                // Rule 2: cleared on every setRope, accepted or not.
                let cleared = self.cam.iter().any(Option::is_some);
                self.cam = [None, None];
                let cleared = if cleared { "; the setRopeCam spots cleared" } else { "" };
                let old = self.status;
                if !carries {
                    // Rule 4: nothing changes; the icons are hidden.
                    return Some(done("0", format!("setRope {}: refused: Connor carries no rope (rule 4); status stays {}, the hook icons hidden{cleared}", status.name(), old.name())));
                }
                self.status = status;
                self.place = place;
                self.dir = dir;
                self.changed_at = Some(now);
                let mut line = format!("status {} -> {} at game time {now:.3} (rule 2), place {}, direction ({:.3}, {:.3}, {:.3}){cleared}", old.name(), status.name(), fmt3(place), dir[0], dir[1], dir[2]);
                let mut put_back = false;
                if status == Status::None {
                    // Rule 6.
                    line += &format!("; the camera limits cleared (were top {:.0}, bottom {:.0}; rule 6)", self.top_limit, self.bottom_limit);
                    self.top_limit = 0.0;
                    self.bottom_limit = 0.0;
                    put_back = true;
                }
                Done { result: "0", line, put_back }
            }
            "KQConner::setRopeCam" => {
                const USAGE: &str = "KQConner::setRopeCam x1 y1 z1 x2 y2 z2";
                let Some(v) = numbers(args).filter(|v| v.len() == 6) else { return Some(usage(USAGE)) };
                self.cam = [spot(&v[0..3]), spot(&v[3..6])];
                done("0", format!("camera spots for the climb: 1 {}, 2 {} (rule 3: set only with no 0 coordinate)", fmt_spot(self.cam[0]), fmt_spot(self.cam[1])))
            }
            "KQConner::setRopeCam2" => {
                const USAGE: &str = "KQConner::setRopeCam2 x y z";
                let Some(v) = numbers(args).filter(|v| v.len() == 3) else { return Some(usage(USAGE)) };
                self.top_cam = Some([v[0], v[1], v[2]]);
                done("0", format!("the camera when the rope is taken at the top: {} (rule 3; kept until set again)", fmt3([v[0], v[1], v[2]])))
            }
            "KQConner::setRopeCamAngle" => {
                const USAGE: &str = "KQConner::setRopeCamAngle angle range";
                let Some(v) = numbers(args).filter(|v| v.len() == 2) else { return Some(usage(USAGE)) };
                self.angle = Some((v[0], v[1]));
                done("0", format!("camera angle {:.3}, range {:.3}: kept, not used (the aim test always passes: \"Do not reproduce\")", v[0], v[1]))
            }
            c @ ("KQConner::setRopeCamTopLimit" | "KQConner::setRopeCamBottomLimit") => {
                let top = c.ends_with("TopLimit");
                let Some(v) = numbers(args).filter(|v| v.len() == 1) else { return Some(usage(&format!("{c} z"))) };
                if top {
                    self.top_limit = v[0];
                } else {
                    self.bottom_limit = v[0];
                }
                done("0", format!("camera {} limit {:.0}{} (rule 3)", if top { "top" } else { "bottom" }, v[0], if v[0] == 0.0 { " (no limit)" } else { "" }))
            }
            _ => return None,
        })
    }

    /// Rule 7: the hook icon wanted now; `carries`: Connor carries the
    /// rope (rule 4); `on_rope`: in the rope state's hanging part.
    pub fn icon(&self, carries: bool, on_rope: bool) -> Option<Hook> {
        if !carries || on_rope {
            return None;
        }
        match self.status {
            Status::None => None,
            Status::Bottom => Some(Hook::Up),
            Status::Top => Some(Hook::Down),
        }
    }
}

/// Rule 7: puts `wanted` in the status icon places (the first free one:
/// STAND-IN, Q147 a) and takes the other hook icon out. Returns the log
/// lines (the places as `IDCTL_TIMER_<n>`).
pub fn sync_icon(places: &mut [Option<StatusIcon>; crate::potions::ICON_PLACES], wanted: Option<Hook>) -> Vec<String> {
    use crate::potions::{free_place, take_place};
    let mut out = Vec::new();
    for h in [Hook::Up, Hook::Down] {
        let icon = StatusIcon::Hook(h);
        let has = places.contains(&Some(icon.clone()));
        if Some(h) == wanted && !has {
            match take_place(places, icon) {
                Some(k) => out.push(format!("{} shown in IDCTL_TIMER_{} (rule 7; the first free place: stand-in, Q147 a)", h.name(), k + 1)),
                None => out.push(format!("{} not shown: all 7 status places are taken", h.name())),
            }
        } else if Some(h) != wanted && has {
            if let Some(k) = free_place(places, &icon) {
                out.push(format!("{} hidden (IDCTL_TIMER_{} free; rule 7)", h.name(), k + 1));
            }
        }
    }
    out
}

/// The movement lists of the rope state (rule 17): Connor's own list in
/// use for his hand mode (`away`, `out`), or `Conrope.anm` (the rest).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Anim {
    Own(&'static str),
    Rope(&'static str),
}

impl Anim {
    pub fn name(self) -> String {
        match self {
            Anim::Own(n) => format!("{n} (his list in use)"),
            Anim::Rope(n) => format!("{n} ({ROPE_LIST})"),
        }
    }
}

/// The rope animation list (rule 13).
pub const ROPE_LIST: &str = "Conrope.anm";
/// Its twelve movements (`Conrope.anm`, observed).
pub const ROPE_MOVEMENTS: [&str; 12] = ["baway", "boff", "bon", "btoss", "climb", "draw", "drawdown", "rope", "taway", "toff", "ton", "ttoss"];

/// Which end the climb started from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum End {
    Bottom,
    Top,
}

/// Where the rope state is (rules 13–26): the movement playing, or what
/// it waits for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    Away,
    Draw,
    Btoss,
    /// The separate rope model's `rope` plays; Connor stands still.
    RopeModel,
    Bon,
    Drawdown,
    Ttoss,
    Ton,
    /// He hangs on the rope; the keys climb (rule 22).
    Hanging,
    Toff,
    Taway,
    Boff,
    Baway,
    Out,
}

/// Rule 23: where he is put when the climb reaches the foot, in his own
/// frame (x his left, y his back, z up).
pub const FOOT_DROP: [f32; 3] = [0.0, 1000.0, -4900.0];

/// Open question 2 (the camera at the end of `btoss`), STAND-IN (Q147 f)
/// from the user's check in the original ("close to Connor's eye level,
/// looking straight up, seeing the rope fly up") and the note's partly
/// read numbers: 250 behind him (the note's (0, 250, …)), at
/// `THROW_CAM_HEIGHT` (ours: the height our camera treats as his chest,
/// `camera.md`), tilted up 1.27 rad, turned half round from the rope's
/// heading (a camera's heading looks the opposite way to Connor's, so
/// this looks the way he faces).
pub const THROW_CAM_BACK: f32 = 250.0;
pub const THROW_CAM_HEIGHT: f32 = 1400.0;
pub const THROW_CAM_TILT: f32 = 1.27;

/// The camera's spot and a point it looks at, for the throw from below.
pub fn throw_camera(place: [f32; 3], dir: [f32; 3]) -> ([f32; 3], [f32; 3]) {
    let spot = add(place, turned(dir, [0.0, THROW_CAM_BACK, THROW_CAM_HEIGHT]));
    let look = crate::camera::view_direction(THROW_CAM_TILT, dir[2] + std::f32::consts::PI);
    (spot, std::array::from_fn(|k| spot[k] + look[k] * 1000.0))
}

/// Rule 18: the lift when he grabs the rope from below.
pub const GRAB_LIFT: f32 = 5000.0;
/// Rules 19, 21: the camera's height range for the climb, and what is
/// kept clear of an obstacle.
pub const CAMERA_RANGE: f32 = 5000.0;
pub const CAMERA_CLEAR: f32 = 250.0;
/// Rule 18: the spots the game makes at the foot, in the rope's frame
/// (x his left, y his back, z up), x taken + and −.
pub const FOOT_SPOT: [f32; 3] = [2500.0, 2500.0, 4800.0];
/// Rule 20: the same at the top.
pub const TOP_SPOT: [f32; 3] = [5000.0, -5000.0, -2000.0];

/// `v` in the rope's frame turned into the world by the rope's direction
/// (its turn about z, as Connor's heading: x = his left, −y = his front;
/// the data's rx and ry are always 0: not used).
pub fn turned(dir: [f32; 3], v: [f32; 3]) -> [f32; 3] {
    let (s, c) = dir[2].sin_cos();
    [v[0] * c - v[1] * s, v[0] * s + v[1] * c, v[2]]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|k| a[k] + b[k])
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()
}

/// Rules 18, 20: the camera spot for the climb: with the place's
/// `setRopeCam` spot 1 set, the nearer to `camera` of spots 1 and 2 (2
/// when equally near; spot 1 alone if 2 is not set); otherwise the nearer
/// of the two the game makes from `offset` (x taken + and −). Returns the
/// spot and why.
pub fn camera_spot(set: [Option<[f32; 3]>; 2], place: [f32; 3], dir: [f32; 3], offset: [f32; 3], camera: [f32; 3]) -> ([f32; 3], String) {
    let (a, b, whose) = match set {
        [Some(a), Some(b)] => (a, b, "the place's setRopeCam spots"),
        [Some(a), None] => return (a, format!("the place's setRopeCam spot 1 {} (spot 2 not set)", fmt3(a))),
        _ => {
            let [x, y, z] = offset;
            (add(place, turned(dir, [x, y, z])), add(place, turned(dir, [-x, y, z])), "the game's own spots")
        }
    };
    let (da, db) = (dist(a, camera), dist(b, camera));
    let (spot, which) = if da < db { (a, 1) } else { (b, 2) };
    (spot, format!("{whose}: 1 {} at {da:.0}, 2 {} at {db:.0} from the camera {}: spot {which}, the nearer (2 when equal)", fmt3(a), fmt3(b), fmt3(camera)))
}

/// Rule 19 (from the foot): the camera's height range: lowest its height
/// now; highest 5,000 above, or the height of something solid straight
/// above (`solid`), less 250, then at most the top limit (0 = none).
pub fn range_up(camera_z: f32, solid: Option<f32>, top_limit: f32) -> (f32, f32) {
    let mut hi = solid.unwrap_or(camera_z + CAMERA_RANGE) - CAMERA_CLEAR;
    if top_limit != 0.0 && hi > top_limit {
        hi = top_limit;
    }
    (camera_z, hi)
}

/// Rule 21 (from the top): highest the camera's height now; lowest 5,000
/// below or the height of something solid straight below, plus 250, then
/// at least the bottom limit (0 = none).
pub fn range_down(camera_z: f32, solid: Option<f32>, bottom_limit: f32) -> (f32, f32) {
    let mut lo = solid.unwrap_or(camera_z - CAMERA_RANGE) + CAMERA_CLEAR;
    if bottom_limit != 0.0 && lo < bottom_limit {
        lo = bottom_limit;
    }
    (lo, camera_z)
}

/// What the rope state needs from the game (the viewer gives it).
pub trait RopeHost {
    fn note(&mut self, line: String);
    /// Starts a movement of the rope state; `false` if it isn't loaded.
    fn play(&mut self, anim: Anim) -> bool;
    fn connor_at(&self) -> [f32; 3];
    fn heading(&self) -> f32;
    fn set_gravity(&mut self, on: bool);
    /// His step-up and step-down heights (`connor-physics.md`).
    fn step_heights(&self) -> (f32, f32);
    fn set_step_heights(&mut self, up: f32, down: f32);
    /// Puts him at `position` with `heading`, at once.
    fn place(&mut self, position: [f32; 3], heading: f32);
    /// His automatic turn to face a point (`walkTo`'s turn).
    fn turn_to(&mut self, point: [f32; 2]);
    fn camera_at(&self) -> [f32; 3];
    /// The camera stops following and stays where it is.
    fn camera_stay(&mut self);
    /// The camera goes to `spot` and looks at `aim`.
    fn camera_to(&mut self, spot: [f32; 3], aim: [f32; 3]);
    /// Where the straight line from `from` to `to` first meets the
    /// terrain or a world object (not Connor): its height and what.
    fn solid_between(&self, from: [f32; 3], to: [f32; 3]) -> Option<(f32, String)>;
    /// Starts the separate rope model (rule 15); `false` if it can't.
    fn start_rope_model(&mut self) -> bool;
    /// handsOff with the pointer kept (rules 13, 19, 21, 23).
    fn hands_off(&mut self, on: bool);
    /// He hangs, the climb at `phase`, frozen (rules 19, 21).
    fn hang(&mut self, phase: f32);
    /// His hand mode becomes empty hands (rule 14).
    fn empty_hands(&mut self);
    /// He hangs on the rope (no hook icon, rule 7).
    fn on_rope(&mut self, on: bool);
    /// The climb that just ended was playing forwards (up) (rule 23).
    fn climbed_to_top(&self) -> bool;
    /// The camera follows Connor again (rules 24, 25).
    fn camera_follow(&mut self);
    /// Rule 25: his hand mode and weapon back as they were; `true` if a
    /// weapon is to be drawn (`out` from its list).
    fn restore_hand(&mut self, mode: crate::weapons::WeaponMode, weapon: Option<String>) -> bool;
    /// Rule 26: standing (`Wait`), the first-person key allowed again, the
    /// rope list let go.
    fn leave(&mut self);
}

/// The rope state (rules 12–26): what was copied at its start, and its
/// progress. One plain record, so a saved game can keep it later (rule
/// 32).
#[derive(Debug, Clone, PartialEq)]
pub struct Climb {
    pub from: End,
    pub stage: Stage,
    pub place: [f32; 3],
    pub dir: [f32; 3],
    /// The place's camera spots and limits (rule 13).
    pub cam: [Option<[f32; 3]>; 2],
    pub top_limit: f32,
    pub bottom_limit: f32,
    /// His hand mode and weapon before (rules 13, 25).
    pub hand_before: crate::weapons::WeaponMode,
    pub weapon_before: Option<String>,
    /// Rule 13: switched out of first person (never: we have none).
    pub first_to_third: bool,
    /// Where the camera looks while he climbs (rules 18, 21).
    pub aim: [f32; 3],
    /// The camera's height range while he climbs (rules 19, 21).
    pub range: (f32, f32),
    /// The step heights saved at the foot (rule 15).
    pub saved_steps: Option<(f32, f32)>,
    /// The rope state is over (rule 26).
    pub done: bool,
}

/// Rule 22: how fast the camera's height and the aim point move while
/// he climbs: the original's 50 per update made per second at 15 updates
/// a second ("Do not reproduce").
pub const CLIMB_CAMERA_SPEED: f32 = 750.0;

/// Rule 22: one update of the climb's camera: `dir` +1 up, −1 down, 0
/// still; the camera's height moves within `range`, the aim point's
/// without limit; x and y stay. Returns the camera's new place and the
/// new aim point.
pub fn climb_camera(camera: [f32; 3], aim: [f32; 3], range: (f32, f32), dir: i8, dt: f32) -> ([f32; 3], [f32; 3]) {
    let dz = CLIMB_CAMERA_SPEED * dt * f32::from(dir);
    let z = match dir {
        1 => (camera[2] + dz).min(range.1).max(camera[2].min(range.1)),
        -1 => (camera[2] + dz).max(range.0).min(camera[2].max(range.0)),
        _ => camera[2],
    };
    ([camera[0], camera[1], z], [aim[0], aim[1], aim[2] + dz])
}

fn solid_text(solid: &Option<(f32, String)>) -> String {
    solid.as_ref().map_or("nothing within 5,000".to_string(), |(z, what)| format!("{what} at {z:.0}"))
}

fn limit_text(l: f32) -> String {
    if l == 0.0 { "none".to_string() } else { format!("{l:.0}") }
}

impl Climb {
    /// Rules 12–13: the rope state starts from the place's settings `r`
    /// with his hand mode `hand` (and weapon); `None` without a status.
    pub fn start(r: &Rope, hand: crate::weapons::WeaponMode, weapon: Option<String>, host: &mut dyn RopeHost) -> Option<Climb> {
        let from = match r.status {
            Status::None => {
                host.note("rope state not started: the status is none".into());
                return None;
            }
            Status::Bottom => End::Bottom,
            Status::Top => End::Top,
        };
        let mut c = Climb {
            from,
            stage: Stage::Away,
            place: r.place,
            dir: r.dir,
            cam: r.cam,
            top_limit: r.top_limit,
            bottom_limit: r.bottom_limit,
            hand_before: hand,
            weapon_before: weapon,
            first_to_third: false,
            aim: [0.0; 3],
            range: (0.0, 0.0),
            saved_steps: None,
            done: false,
        };
        host.camera_stay();
        host.turn_to([r.place[0], r.place[1]]);
        host.hands_off(true);
        let weapon = c.weapon_before.as_deref().map_or(String::new(), |w| format!(" ({w})"));
        host.note(format!(
            "rope state: starts from the {} (rule 13): the camera stops following and stays at {}; he turns to face the rope's place {}; the setRopeCam spots 1 {}, 2 {}, limits top {}, bottom {} copied; third person already (no first person: nothing to switch or restore); handsOff on, the pointer kept; hand mode before: {}{weapon}; {ROPE_LIST} ready",
            if from == End::Bottom { "foot" } else { "top" },
            fmt3(host.camera_at()),
            fmt3(r.place),
            fmt_spot(r.cam[0]),
            fmt_spot(r.cam[1]),
            limit_text(r.top_limit),
            limit_text(r.bottom_limit),
            hand.name(),
        ));
        c.go(Stage::Away, host);
        Some(c)
    }

    /// Rule 26: the end of the rope state.
    fn finish(&mut self, host: &mut dyn RopeHost) {
        host.hands_off(false);
        host.leave();
        self.done = true;
        host.note(format!(
            "rope state: over (rule 26): handsOff off, the first-person key allowed again, {ROPE_LIST} let go, standing (Wait); first person not restored (never left: we have none); Connor at {}",
            fmt3(host.connor_at())
        ));
    }

    /// The movement of `stage`, if it plays one.
    fn anim(stage: Stage) -> Option<Anim> {
        Some(match stage {
            Stage::Away => Anim::Own("away"),
            Stage::Out => Anim::Own("out"),
            Stage::Draw => Anim::Rope("draw"),
            Stage::Btoss => Anim::Rope("btoss"),
            Stage::Bon => Anim::Rope("bon"),
            Stage::Drawdown => Anim::Rope("drawdown"),
            Stage::Ttoss => Anim::Rope("ttoss"),
            Stage::Ton => Anim::Rope("ton"),
            Stage::Toff => Anim::Rope("toff"),
            Stage::Taway => Anim::Rope("taway"),
            Stage::Boff => Anim::Rope("boff"),
            Stage::Baway => Anim::Rope("baway"),
            Stage::RopeModel | Stage::Hanging => return None,
        })
    }

    /// Enters `stage`: its movement starts (a missing one counts as ended
    /// at once, logged).
    fn go(&mut self, stage: Stage, host: &mut dyn RopeHost) {
        self.stage = stage;
        let Some(a) = Self::anim(stage) else { return };
        if host.play(a) {
            host.note(format!("rope state: {} starts", a.name()));
        } else {
            host.note(format!("rope state: {} is not loaded: skipped", a.name()));
            self.ended(host);
        }
    }

    /// The movement of the stage now ended (or the rope model's, in
    /// `RopeModel`): what follows (rules 14–21).
    pub fn ended(&mut self, host: &mut dyn RopeHost) {
        if let Some(a) = Self::anim(self.stage) {
            host.note(format!("rope state: {} ends; Connor at {}", a.name(), fmt3(host.connor_at())));
        }
        match self.stage {
            Stage::Away => {
                // Rule 14.
                host.empty_hands();
                host.note("rope state: empty hands (hand mode 4; rule 14)".into());
                self.go(if self.from == End::Bottom { Stage::Draw } else { Stage::Drawdown }, host);
            }
            Stage::Draw => {
                host.set_gravity(false);
                host.note("rope state: gravity off (rule 15)".into());
                self.go(Stage::Btoss, host);
            }
            Stage::Btoss => {
                let (up, down) = host.step_heights();
                self.saved_steps = Some((up, down));
                host.set_step_heights(0.0, 0.0);
                host.place(self.place, self.dir[2]);
                let model = if host.start_rope_model() { "the rope model starts (rope, Conrope.anm)" } else { "the rope model could not start" };
                // Open question 2 (STAND-IN, Q147 f): close behind him at
                // eye level, looking straight up the rope.
                let (spot, aim) = throw_camera(self.place, self.dir);
                host.camera_to(spot, aim);
                host.note(format!(
                    "rope state: step heights saved (up {up:.0}, down {down:.0}) and set to 0; placed at the rope's place {} heading {:.3}; {model}; the camera close to him at {}, tilted up {THROW_CAM_TILT} rad, looking the way he faces (open question 2: stand-in from the user's check, Q147 f); Connor told to stand still (rule 15)",
                    fmt3(self.place),
                    self.dir[2],
                    fmt3(spot)
                ));
                self.stage = Stage::RopeModel;
                if !host.play(Anim::Own("stop")) {
                    host.note("rope state: stop (his own list) is not loaded".into());
                }
            }
            Stage::RopeModel => {
                // Rule 18: the grab from below.
                self.aim = host.connor_at();
                let camera = host.camera_at();
                let (spot, why) = camera_spot(self.cam, self.place, self.dir, FOOT_SPOT, camera);
                host.camera_to(spot, self.aim);
                host.set_gravity(false);
                host.on_rope(true);
                let p = host.connor_at();
                let lifted = [p[0], p[1], p[2] + GRAB_LIFT];
                host.place(lifted, host.heading());
                host.note(format!(
                    "rope state: the rope model ended: Connor grabs the rope at {} (the aim point); camera spot {} ({why}), aimed at the aim point; gravity off; the hook icons hidden; lifted 5,000 to {} (rule 18)",
                    fmt3(p),
                    fmt3(spot),
                    fmt3(lifted)
                ));
                self.go(Stage::Bon, host);
            }
            Stage::Bon => {
                // Rule 19.
                let cam = host.camera_at();
                let solid = host.solid_between(cam, [cam[0], cam[1], cam[2] + CAMERA_RANGE]);
                self.range = range_up(cam[2], solid.as_ref().map(|s| s.0), self.top_limit);
                host.hang(0.0);
                host.hands_off(false);
                host.note(format!(
                    "rope state: hanging at the climb's first frame (the foot), frozen, turning off, handsOff off; camera height range {:.0}..{:.0} (from {:.0}; above it {}; top limit {}; rule 19)",
                    self.range.0,
                    self.range.1,
                    cam[2],
                    solid_text(&solid),
                    limit_text(self.top_limit)
                ));
                self.stage = Stage::Hanging;
            }
            Stage::Drawdown => {
                // Rules 16, 20.
                host.place(self.place, self.dir[2]);
                let camera = host.camera_at();
                let (spot, why) = camera_spot(self.cam, self.place, self.dir, TOP_SPOT, camera);
                host.camera_to(spot, self.place);
                host.set_gravity(false);
                host.note(format!("rope state: placed at the rope's place {} heading {:.3}; camera spot {} ({why}), aimed at the rope's place; gravity off (rules 16, 20)", fmt3(self.place), self.dir[2], fmt3(spot)));
                self.go(Stage::Ttoss, host);
            }
            Stage::Ttoss => self.go(Stage::Ton, host),
            Stage::Ton => {
                // Rule 21.
                let cam = host.camera_at();
                let solid = host.solid_between(cam, [cam[0], cam[1], cam[2] - CAMERA_RANGE]);
                self.range = range_down(cam[2], solid.as_ref().map(|s| s.0), self.bottom_limit);
                host.set_gravity(false);
                host.on_rope(true);
                let h = host.heading() + std::f32::consts::PI;
                let p = host.connor_at();
                host.place(p, h);
                self.aim = p;
                host.hang(1.0);
                host.hands_off(false);
                host.note(format!(
                    "rope state: gravity off, the hook icons hidden, a half turn to heading {h:.3}; the aim point {}; hanging at the climb's last frame (the top), frozen, turning off, handsOff off; camera height range {:.0}..{:.0} (from {:.0}; below it {}; bottom limit {}; rule 21)",
                    fmt3(p),
                    self.range.0,
                    self.range.1,
                    cam[2],
                    solid_text(&solid),
                    limit_text(self.bottom_limit)
                ));
                self.stage = Stage::Hanging;
            }
            Stage::Hanging => {
                // Rule 23: the climb reached an end.
                let top = host.climbed_to_top();
                host.on_rope(false);
                host.hands_off(true);
                let p = host.connor_at();
                if top {
                    host.note(format!("climb: reached the top end at {}: the hook icon may show again, turning on, handsOff on (pointer kept) (rule 23)", fmt3(p)));
                    self.go(Stage::Toff, host);
                } else {
                    let h = host.heading();
                    let to = add(p, turned([0.0, 0.0, h], FOOT_DROP));
                    host.place(to, h);
                    host.set_gravity(true);
                    host.note(format!(
                        "climb: reached the bottom end at {}: the hook icon may show again, turning on, handsOff on (pointer kept); moved 1,000 toward his back and 4,900 down to {}, gravity on (rule 23)",
                        fmt3(p),
                        fmt3(to)
                    ));
                    self.go(Stage::Boff, host);
                }
            }
            Stage::Toff => {
                // Rule 24.
                host.camera_follow();
                host.note("rope state: the camera follows Connor again (rule 24)".into());
                self.go(Stage::Taway, host);
            }
            Stage::Boff => self.go(Stage::Baway, host),
            Stage::Taway | Stage::Baway => {
                // Rule 25.
                host.set_gravity(true);
                let steps = match self.saved_steps {
                    Some((up, down)) if up != 0.0 || down != 0.0 => {
                        let (u, d) = host.step_heights();
                        host.set_step_heights(if up != 0.0 { up } else { u }, if down != 0.0 { down } else { d });
                        format!("step heights back to up {up:.0}, down {down:.0}")
                    }
                    Some(_) => "the saved step heights are 0: not put back".into(),
                    None => "the step heights were never saved (from the top): unchanged".into(),
                };
                host.camera_follow();
                let draw = host.restore_hand(self.hand_before, self.weapon_before.clone());
                host.note(format!(
                    "rope state: gravity on; {steps}; the camera follows Connor; hand mode back to {}{} (rule 25)",
                    self.hand_before.name(),
                    if draw { ": out is played (he held a weapon)" } else { ": nothing drawn (his weapon was put away; the user's check)" }
                ));
                if draw {
                    self.go(Stage::Out, host);
                } else {
                    self.finish(host);
                }
            }
            Stage::Out => self.finish(host),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn set_rope_needs_the_rope_and_stores_the_place_only_when_accepted() {
        let mut r = Rope::default();
        let d = r.command(&w("KQConner::setRope Bottom 147646 207586 7800 0 0 -1.57"), false, 1.0).unwrap();
        assert!(d.line.contains("refused"), "{}", d.line);
        assert_eq!(r.status, Status::None);
        assert_eq!(r.place, [0.0; 3], "not stored when refused");
        let d = r.command(&w("KQConner::setRope bottom 147646 207586 7800 0 0 -1.57"), true, 2.0).unwrap();
        assert_eq!(d.result, "0");
        assert_eq!(r.status, Status::Bottom, "any case");
        assert_eq!(r.place, [147646.0, 207586.0, 7800.0]);
        assert_eq!(r.dir, [0.0, 0.0, -1.57]);
        assert_eq!(r.changed_at, Some(2.0));
        // Without the rope, None leaves the status as it was (rule 4).
        r.command(&w("KQConner::setRope None"), false, 3.0);
        assert_eq!(r.status, Status::Bottom);
    }

    #[test]
    fn set_rope_takes_one_four_or_seven_words() {
        let mut r = Rope::default();
        assert_eq!(r.command(&w("KQConner::setRope Top 1 2"), true, 0.0).unwrap().result, "1");
        assert_eq!(r.command(&w("KQConner::setRope Sideways"), true, 0.0).unwrap().result, "1");
        assert_eq!(r.status, Status::None);
        r.command(&w("KQConner::setRope Top 1 2 3"), true, 0.0);
        assert_eq!((r.status, r.place, r.dir), (Status::Top, [1.0, 2.0, 3.0], [0.0; 3]));
        r.command(&w("KQConner::setRope Bottom"), true, 0.0);
        assert_eq!((r.status, r.place), (Status::Bottom, [0.0; 3]));
    }

    #[test]
    fn camera_spots_limits_and_their_clearing() {
        let mut r = Rope::default();
        r.command(&w("KQConner::setRopeCam 1 2 3 4 0 6"), true, 0.0);
        assert_eq!(r.cam, [Some([1.0, 2.0, 3.0]), None], "a 0 coordinate: not set");
        r.command(&w("KQConner::setRopeCam2 151584 210996 8520"), true, 0.0);
        r.command(&w("KQConner::setRopeCamTopLimit 9000"), true, 0.0);
        r.command(&w("KQConner::setRopeCamBottomLimit 4000"), true, 0.0);
        // A refused setRope still clears the spots (rule 2).
        r.command(&w("KQConner::setRope Top 1 2 3 0 0 0"), false, 0.0);
        assert_eq!(r.cam, [None, None]);
        r.command(&w("KQConner::setRope Top 1 2 3 0 0 0"), true, 0.0);
        assert_eq!((r.top_limit, r.bottom_limit), (9000.0, 4000.0), "Top/Bottom keep the limits");
        let d = r.command(&w("KQConner::setRope None"), true, 0.0).unwrap();
        assert!(d.put_back);
        assert_eq!((r.top_limit, r.bottom_limit), (0.0, 0.0), "None clears them (rule 6)");
        assert_eq!(r.top_cam, Some([151584.0, 210996.0, 8520.0]), "sticky");
    }

    /// A pretend game: records what the rope state asked.
    #[derive(Default)]
    struct Host {
        played: Vec<Anim>,
        at: [f32; 3],
        heading: f32,
        gravity: bool,
        steps: (f32, f32),
        camera: [f32; 3],
        camera_aim: Option<[f32; 3]>,
        following: bool,
        hands_off: bool,
        hung: Option<f32>,
        on_rope: bool,
        empty: bool,
        model: bool,
        left: bool,
        lines: Vec<String>,
    }

    impl RopeHost for Host {
        fn note(&mut self, line: String) {
            self.lines.push(line);
        }
        fn play(&mut self, anim: Anim) -> bool {
            self.played.push(anim);
            true
        }
        fn connor_at(&self) -> [f32; 3] {
            self.at
        }
        fn heading(&self) -> f32 {
            self.heading
        }
        fn set_gravity(&mut self, on: bool) {
            self.gravity = on;
        }
        fn step_heights(&self) -> (f32, f32) {
            self.steps
        }
        fn set_step_heights(&mut self, up: f32, down: f32) {
            self.steps = (up, down);
        }
        fn place(&mut self, position: [f32; 3], heading: f32) {
            self.at = position;
            self.heading = heading;
        }
        fn turn_to(&mut self, _point: [f32; 2]) {}
        fn camera_at(&self) -> [f32; 3] {
            self.camera
        }
        fn camera_stay(&mut self) {
            self.following = false;
        }
        fn camera_to(&mut self, spot: [f32; 3], aim: [f32; 3]) {
            self.camera = spot;
            self.camera_aim = Some(aim);
        }
        fn solid_between(&self, _from: [f32; 3], _to: [f32; 3]) -> Option<(f32, String)> {
            None
        }
        fn start_rope_model(&mut self) -> bool {
            self.model = true;
            true
        }
        fn hands_off(&mut self, on: bool) {
            self.hands_off = on;
        }
        fn hang(&mut self, phase: f32) {
            self.hung = Some(phase);
        }
        fn empty_hands(&mut self) {
            self.empty = true;
        }
        fn on_rope(&mut self, on: bool) {
            self.on_rope = on;
        }
        fn climbed_to_top(&self) -> bool {
            self.hung == Some(1.0)
        }
        fn camera_follow(&mut self) {
            self.following = true;
        }
        fn restore_hand(&mut self, mode: crate::weapons::WeaponMode, _weapon: Option<String>) -> bool {
            self.empty = false;
            mode != crate::weapons::WeaponMode::None
        }
        fn leave(&mut self) {
            self.left = true;
        }
    }

    fn place(status: Status) -> Rope {
        Rope { status, place: [1000.0, 2000.0, 300.0], dir: [0.0, 0.0, -std::f32::consts::FRAC_PI_2], ..Default::default() }
    }

    #[test]
    fn the_spots_the_game_makes_turn_with_the_rope() {
        // Heading -pi/2: he faces -x; his left is -y... his back +x.
        let d = [0.0, 0.0, -std::f32::consts::FRAC_PI_2];
        let v = turned(d, [0.0, 2500.0, 0.0]);
        assert!((v[0] - 2500.0).abs() < 0.01 && v[1].abs() < 0.01, "y (his back) goes to +x: {v:?}");
        let (spot, why) = camera_spot([None, None], [0.0; 3], d, FOOT_SPOT, [2500.0, -2500.0, 0.0]);
        assert!(why.contains("spot 1"), "{why}");
        assert!((spot[0] - 2500.0).abs() < 0.01 && (spot[1] + 2500.0).abs() < 0.01 && spot[2] == 4800.0, "{spot:?}");
        // Equally near: spot 2.
        let (_, why) = camera_spot([Some([1.0, 0.0, 1.0]), Some([-1.0, 0.0, 1.0])], [0.0; 3], d, FOOT_SPOT, [0.0; 3]);
        assert!(why.contains("spot 2"), "{why}");
    }

    #[test]
    fn the_throw_camera_looks_up_the_way_he_faces() {
        // He faces -x (heading -pi/2): the camera 250 behind (+x), looking
        // toward -x and up.
        let (spot, aim) = throw_camera([0.0; 3], [0.0, 0.0, -std::f32::consts::FRAC_PI_2]);
        assert!((spot[0] - 250.0).abs() < 0.01 && spot[1].abs() < 0.01 && spot[2] == 1400.0, "{spot:?}");
        let d: [f32; 3] = std::array::from_fn(|k| aim[k] - spot[k]);
        assert!(d[0] < 0.0 && d[1].abs() < 0.01 && d[2] > 900.0, "{d:?}");
    }

    #[test]
    fn the_height_ranges_and_their_limits() {
        assert_eq!(range_up(1000.0, None, 0.0), (1000.0, 5750.0));
        assert_eq!(range_up(1000.0, Some(3000.0), 0.0), (1000.0, 2750.0), "an obstacle less 250");
        assert_eq!(range_up(1000.0, None, 4000.0), (1000.0, 4000.0), "the top limit");
        assert_eq!(range_down(9000.0, None, 0.0), (4250.0, 9000.0));
        assert_eq!(range_down(9000.0, None, 6000.0), (6000.0, 9000.0), "a bottom limit only raises the lowest");
        assert_eq!(range_down(9000.0, None, 2000.0), (4250.0, 9000.0));
    }

    #[test]
    fn from_the_foot_to_hanging_in_order() {
        let mut h = Host { at: [990.0, 2010.0, 300.0], steps: (325.0, 325.0), camera: [5000.0, 0.0, 2000.0], following: true, gravity: true, ..Default::default() };
        let mut c = Climb::start(&place(Status::Bottom), crate::weapons::WeaponMode::Hand, Some("INVITEM_Fists".into()), &mut h).unwrap();
        assert!(!h.following && h.hands_off);
        for _ in 0..3 {
            c.ended(&mut h);
        }
        assert!(h.empty && !h.gravity && h.model);
        assert_eq!(h.steps, (0.0, 0.0));
        assert_eq!(c.saved_steps, Some((325.0, 325.0)));
        assert_eq!(h.at, [1000.0, 2000.0, 300.0], "placed at the rope's place");
        assert_eq!(c.stage, Stage::RopeModel);
        c.ended(&mut h);
        assert_eq!(h.at, [1000.0, 2000.0, 5300.0], "lifted 5,000");
        assert_eq!(h.camera_aim, Some([1000.0, 2000.0, 300.0]), "aimed at where he grabbed");
        assert!(h.on_rope);
        c.ended(&mut h);
        assert_eq!(c.stage, Stage::Hanging);
        assert_eq!(h.hung, Some(0.0));
        assert!(!h.hands_off);
        assert_eq!(c.range, (h.camera[2], h.camera[2] + 4750.0));
        let names: Vec<Anim> = h.played.clone();
        assert_eq!(names, [Anim::Own("away"), Anim::Rope("draw"), Anim::Rope("btoss"), Anim::Own("stop"), Anim::Rope("bon")]);
    }

    #[test]
    fn from_the_top_to_hanging_with_the_half_turn() {
        let mut h = Host { at: [990.0, 2010.0, 300.0], camera: [5000.0, 0.0, 2000.0], gravity: true, steps: (325.0, 325.0), ..Default::default() };
        let mut c = Climb::start(&place(Status::Top), crate::weapons::WeaponMode::None, None, &mut h).unwrap();
        for _ in 0..4 {
            c.ended(&mut h);
        }
        assert_eq!(c.stage, Stage::Hanging);
        assert_eq!(h.played, [Anim::Own("away"), Anim::Rope("drawdown"), Anim::Rope("ttoss"), Anim::Rope("ton")]);
        assert_eq!(h.steps, (325.0, 325.0), "the top never changes the step heights");
        assert!((h.heading - std::f32::consts::FRAC_PI_2).abs() < 1e-5, "half a turn from -pi/2: {}", h.heading);
        assert_eq!(h.hung, Some(1.0));
        assert_eq!(h.camera_aim, Some([1000.0, 2000.0, 300.0]), "aimed at the rope's place");
        assert_eq!(c.range.1, h.camera[2]);
    }

    #[test]
    fn up_from_the_foot_and_off_at_the_top_with_the_weapon_drawn() {
        let mut h = Host { at: [990.0, 2010.0, 300.0], steps: (325.0, 325.0), camera: [5000.0, 0.0, 2000.0], gravity: true, ..Default::default() };
        let mut c = Climb::start(&place(Status::Bottom), crate::weapons::WeaponMode::Hand, Some("INVITEM_Fists".into()), &mut h).unwrap();
        for _ in 0..5 {
            c.ended(&mut h);
        }
        assert_eq!(c.stage, Stage::Hanging);
        h.hung = Some(1.0); // climbed to the top
        c.ended(&mut h);
        assert_eq!(c.stage, Stage::Toff);
        assert!(h.hands_off && !h.on_rope);
        assert_eq!(h.at, [1000.0, 2000.0, 5300.0], "no move at the top end");
        c.ended(&mut h);
        assert!(h.following, "the camera follows after toff");
        c.ended(&mut h);
        assert_eq!(c.stage, Stage::Out, "he held a weapon: out");
        assert!(h.gravity);
        assert_eq!(h.steps, (325.0, 325.0), "put back");
        c.ended(&mut h);
        assert!(c.done && h.left && !h.hands_off);
        assert_eq!(&h.played[5..], [Anim::Rope("toff"), Anim::Rope("taway"), Anim::Own("out")]);
    }

    #[test]
    fn down_from_the_top_and_off_at_the_foot_with_nothing_drawn() {
        let mut h = Host { at: [990.0, 2010.0, 5300.0], steps: (325.0, 325.0), camera: [5000.0, 0.0, 2000.0], gravity: true, ..Default::default() };
        let mut c = Climb::start(&place(Status::Top), crate::weapons::WeaponMode::None, None, &mut h).unwrap();
        for _ in 0..4 {
            c.ended(&mut h);
        }
        assert_eq!(c.stage, Stage::Hanging);
        h.hung = Some(0.0); // climbed down
        let before = h.at;
        c.ended(&mut h);
        assert_eq!(c.stage, Stage::Boff);
        assert!(h.gravity, "gravity on for the drop");
        // He faces +pi/2 after the half turn (from -pi/2): his back is -x.
        assert!((h.at[0] - (before[0] - 1000.0)).abs() < 0.1 && (h.at[1] - before[1]).abs() < 0.1 && h.at[2] == before[2] - 4900.0, "{:?}", h.at);
        c.ended(&mut h);
        assert_eq!(c.stage, Stage::Baway);
        c.ended(&mut h);
        assert!(c.done, "no weapon: no out");
        assert!(h.following);
        assert_eq!(h.steps, (325.0, 325.0), "never changed from the top");
    }

    #[test]
    fn the_climb_camera_moves_750_a_second_within_its_range() {
        let dt = 1.0 / 15.0;
        let (c, a) = climb_camera([1.0, 2.0, 1000.0], [0.0, 0.0, 300.0], (1000.0, 1100.0), 1, dt);
        assert!((c[2] - 1050.0).abs() < 1e-3 && (a[2] - 350.0).abs() < 1e-3, "50 an update at 15 a second");
        let (c, a) = climb_camera(c, a, (1000.0, 1100.0), 1, dt);
        let (c, a) = climb_camera(c, a, (1000.0, 1100.0), 1, dt);
        assert_eq!(c[2], 1100.0, "not above the highest");
        assert!((a[2] - 450.0).abs() < 1e-3, "the aim point has no limit");
        let (c2, _) = climb_camera(c, a, (1000.0, 1100.0), 0, dt);
        assert_eq!(c2, c, "no key: still");
        let (c3, _) = climb_camera([1.0, 2.0, 1010.0], a, (1000.0, 1100.0), -1, dt);
        assert_eq!((c3[0], c3[1], c3[2]), (1.0, 2.0, 1000.0), "not below the lowest; x, y kept");
    }

    #[test]
    fn the_hook_icon_follows_the_status() {
        let mut r = Rope { status: Status::Bottom, ..Default::default() };
        assert_eq!(r.icon(true, false), Some(Hook::Up));
        assert_eq!(r.icon(false, false), None, "no rope, no icon (rule 4)");
        assert_eq!(r.icon(true, true), None, "not while on the rope");
        r.status = Status::Top;
        assert_eq!(r.icon(true, false), Some(Hook::Down));
        let mut places: [Option<StatusIcon>; crate::potions::ICON_PLACES] = Default::default();
        places[0] = Some(StatusIcon::Mask);
        assert_eq!(sync_icon(&mut places, Some(Hook::Up)).len(), 1);
        assert_eq!(places[1], Some(StatusIcon::Hook(Hook::Up)), "the first free place");
        sync_icon(&mut places, Some(Hook::Down));
        assert_eq!(places[1], Some(StatusIcon::Hook(Hook::Down)), "the old one goes first");
        sync_icon(&mut places, None);
        assert_eq!(places[1], None);
    }
}
