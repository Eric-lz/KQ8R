//! The door (`KQDoor`): its four states, its movements, cues and the turn
//! its animation gives it (M3 step 3a-2).
//!
//! Rules from `docs/behaviour/doors.md` (traced unless marked):
//!
//! - States closed / opening / opened / closing; every door starts closed
//!   (rule 1).
//! - `KQDoorEvent` 1 opens only a closed door, 2 closes only an opened
//!   one, anything else (also no number) toggles; a moving door ignores
//!   it (rules 2–4).
//! - `KQAnimPlayEvent <movement>` starts that movement; the state stays
//!   (rule 6).
//! - When a movement called `open` / `close` (any capitals) finishes, the
//!   door is opened / closed, shows `opened` / `closed`, and its script is
//!   called with `Open` / `Close` (rules 5, 6). The caller makes that call.
//! - A cue at phase c fires when c is in [lower, higher) of the phases
//!   before and after the update, in either direction (rule 9).
//! - The door turns with its animation (rule 24, inferred from data and
//!   the user's check). Open question 1's recommendation: the turn is
//!   worked out from the current phase, never added up: the frame shown
//!   at the phase, and the frame steps 1…k of that frame chained, so it
//!   can't drift.

use kq8_data::shape::Transform;

use crate::collide::Placement;
use crate::named_anim::NamedAnimPlayer;

/// Rule 1.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DoorState {
    Closed,
    Opening,
    Opened,
    Closing,
}

impl DoorState {
    pub fn name(self) -> &'static str {
        match self {
            DoorState::Closed => "closed",
            DoorState::Opening => "opening",
            DoorState::Opened => "opened",
            DoorState::Closing => "closing",
        }
    }
}

/// `cueN=<phase> <action…>` of a movement.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub phase: f32,
    pub action: String,
}

/// One of the door's movements, from its `loop=` line, its cues and its
/// shape's frames.
#[derive(Debug, Clone)]
pub struct DoorMovement {
    pub name: String,
    /// Start phase, 0..1.
    pub start: f32,
    /// 1 forward, −1 backward, 0 still.
    pub speed: f32,
    /// True = repeats (no Daventry door movement does).
    pub repeat: bool,
    /// The 6th field of the `loop=` line: 1 = a new movement may replace
    /// it at once (`anim-scripts.md` rule 4).
    pub cut_short: bool,
    /// Seconds for one play from phase 0 to 1 (the loop's stored length).
    pub play_time: f32,
    /// Each frame's own step (frame 0 first; frame 0's is zero).
    pub frame_steps: Vec<Transform>,
    /// The loop starts at the first frame of its `.kqs` file: entering its
    /// frame 0 forwards adds the last frame's step (`objects.md` "Loops"
    /// rule 6a).
    pub first_in_file: bool,
    /// Each frame's named model parts and their own placements (the
    /// `arrow` part a bolt starts at, `ranged-weapons.md` rule 24; M5
    /// 5f-2; the part a dropped weapon is laid on, `fighting.md` rule
    /// 31a); empty where not needed.
    pub frame_parts: Vec<kq8_data::shape::FrameParts>,
    pub cues: Vec<Cue>,
    /// Its `transitionN` lines (`connor-movement.md` "Starting and
    /// stopping"; played for characters from M4 4d-11).
    pub transitions: Vec<kq8_data::objects::TransitionLine>,
}

impl DoorMovement {
    /// Frames in the loop (at least 1).
    pub fn frames(&self) -> usize {
        self.frame_steps.len().max(1)
    }

    /// Phase a second at speed 1: rule 6's (N − 1)/(N × length) for a
    /// repeating loop of 2 frames or more ([`Self::step_with`]), else
    /// 1/length; 0 with no length.
    pub fn rate(&self) -> f32 {
        if self.play_time <= 0.0 {
            0.0
        } else if self.repeat && self.frame_steps.len() >= 2 {
            repeat_rate(self.frames(), self.play_time)
        } else {
            1.0 / self.play_time
        }
    }

    /// The frame shown at `phase`: floor(phase × frames + 0.001), repeating
    /// or not ([`kq8_data::shape::frame_at_phase`], `objects.md` "Loops"
    /// rule 6, Q153).
    pub fn frame_at(&self, phase: f32) -> usize {
        kq8_data::shape::frame_at_phase(phase, self.frames(), self.repeat)
    }

    /// One update of `dt` seconds from `phase` at the movement's own speed.
    pub fn step(&self, phase: f32, dt: f32) -> Step {
        self.step_with(phase, dt, self.speed)
    }

    /// One update of `dt` seconds from `phase` at `speed`: the new phase,
    /// whether a play-once movement reached its end (1 forward, 0
    /// backward), and the cues passed (rule 9). A repeating loop wraps;
    /// at a wrap the cue ranges are [old, 1.0001) then [0, new) forwards,
    /// [0, old) then [new, 1.0001) backwards (`objects.md` "Loops" rule
    /// 4), so a cue at exactly 1.0 fires once per cycle. Shared by doors,
    /// chests and animated objects.
    pub fn step_with(&self, phase: f32, dt: f32, speed: f32) -> Step {
        // `objects.md` "Loops" rule 6 (Q164): a repeating loop of N ≥ 2
        // frames moves (N − 1)/(N × length) a second and a forward wrap
        // lands on exactly 0, for every `.kqs` animation (doors, chests,
        // animated objects; characters through `step_frames` too). One
        // frame: the old way (ours: the rule's rate would be 0 and its
        // cues would never fire).
        if let Some((step, _)) = self.step_frames(phase, dt, speed) {
            return step;
        }
        if self.play_time <= 0.0 {
            // `anim-scripts.md` rule 10a (Q120): a model loop that lasts 0
            // s never moves and never ends, loop or play-once (the
            // Uglybeast's mouth poses hold).
            return Step { to: phase, done: false, cues: Vec::new() };
        }
        let raw = phase + speed * dt / self.play_time;
        let mut to = raw;
        let mut done = false;
        let mut ranges = Vec::with_capacity(2);
        if self.repeat {
            to = raw.rem_euclid(1.0);
            if raw >= 1.0 {
                ranges.extend([(phase, WRAP_TOP), (0.0, to)]);
            } else if raw < 0.0 {
                ranges.extend([(0.0, phase), (to, WRAP_TOP)]);
            }
        } else if raw >= crate::named_anim::ENDED_FORWARD && speed > 0.0 {
            // "Has ended" (`anim-scripts.md`): 0.9999 or more.
            to = 1.0;
            done = true;
        } else if raw <= 0.0 && speed < 0.0 {
            to = 0.0;
            done = true;
        }
        if ranges.is_empty() {
            ranges.push(if phase <= to { (phase, to) } else { (to, phase) });
        }
        let cues = self
            .cues
            .iter()
            .filter(|c| ranges.iter().any(|&(lo, hi)| c.phase >= lo && c.phase < hi))
            .map(|c| c.action.clone())
            .collect();
        Step { to, done, cues }
    }

    /// Units per second along the ground of a repeating movement at
    /// `speed` by the frame player (`objects.md` "Loops" rule 6a): (sum of
    /// the steps + the last step, forwards from the file's first frame) ×
    /// (N − 1)/(N × length) × |speed|. For the logs.
    pub fn loop_ground_speed(&self, speed: f32) -> f32 {
        let n = self.frames();
        let mut d = self.frame_steps.iter().fold([0.0f32; 3], |a, t| [a[0] + t.p[0], a[1] + t.p[1], a[2] + t.p[2]]);
        if speed >= 0.0 && self.first_in_file && n >= 2 {
            let last = self.frame_steps[n - 1].p;
            d = std::array::from_fn(|i| d[i] + last[i]);
        }
        (d[0] * d[0] + d[1] * d[1]).sqrt() * repeat_rate(n, self.play_time) * speed.abs()
    }

    /// A character's repeating movement played by the frame player
    /// (`objects.md` "Loops" rules 4, 6 and 6a, Q164): the phase moves
    /// (N − 1)/(N × length) per second; a forward wrap sets it to exactly 0;
    /// the cue ranges as [`Self::step_with`]'s. Also the summed step of
    /// the frames entered ([`repeat_advance`]). `None` for a play-once
    /// movement, no length or one frame (then [`Self::step_with`]).
    pub fn step_frames(&self, phase: f32, dt: f32, speed: f32) -> Option<(Step, [f32; 3])> {
        let rate = repeat_rate(self.frames(), self.play_time);
        if !self.repeat || rate <= 0.0 || self.frame_steps.len() < 2 {
            return None;
        }
        let (to, own) = repeat_advance(phase, speed * dt * rate, self.frames(), self.first_in_file, |k| self.frame_steps[k].p);
        let wrapped = if speed >= 0.0 { to < phase } else { to > phase };
        let ranges = match (wrapped, speed >= 0.0) {
            (true, true) => [(phase, WRAP_TOP), (0.0, to)],
            (true, false) => [(0.0, phase), (to, WRAP_TOP)],
            (false, _) => [(phase.min(to), phase.max(to)), (0.0, 0.0)],
        };
        let cues = self.cues.iter().filter(|c| ranges.iter().any(|&(lo, hi)| c.phase >= lo && c.phase < hi)).map(|c| c.action.clone()).collect();
        Some((Step { to, done: false, cues }, own))
    }

    /// The steps of frames 1…`frame` chained: each step is taken in the
    /// door's own axes as they are after the earlier steps. Frame 0 is the
    /// starting pose (no step).
    pub fn turn_at_frame(&self, frame: usize) -> Transform {
        let mut c = identity();
        for s in self.frame_steps.iter().take(frame + 1).skip(1) {
            c = chain(&c, s);
        }
        c
    }
}

/// The top of a cue range at a wrap (`objects.md` "Loops" rule 4).
pub const WRAP_TOP: f32 = 1.0001;

/// Phase per second at speed 1 of a repeating loop of `frames` frames
/// lasting `duration` s: (N − 1)/(N × duration), so one cycle takes
/// N × duration/(N − 1) (`objects.md` "Loops that move the whole object"
/// rule 6, Q164). 0 for no length or a single frame.
pub fn repeat_rate(frames: usize, duration: f32) -> f32 {
    if duration <= 0.0 || frames < 2 {
        return 0.0;
    }
    let n = frames as f32;
    (n - 1.0) / (n * duration)
}

/// One update of a repeating loop moved by `delta` in phase (rules 6 and
/// 6a, Q164): the new phase and the whole-object step of the frames
/// entered. Forwards: frames a + 1 … b are added; reaching frame N wraps to
/// frame 0 with the phase **exactly 0** (the overshoot dropped), and
/// entering frame 0 adds the **last** frame's step when the loop starts at
/// the file's first frame (`first_in_file`), else frame 0's own.
/// Backwards: frames a − 1 … b are taken off; below 0 the phase becomes
/// (N − 1)/N on the last frame; no last-frame swap.
pub fn repeat_advance(phase: f32, delta: f32, frames: usize, first_in_file: bool, step: impl Fn(usize) -> [f32; 3]) -> (f32, [f32; 3]) {
    let n = frames.max(1);
    let frame = |p: f32| kq8_data::shape::frame_at_phase(p, n, true);
    let a = frame(phase);
    let raw = phase + delta;
    let mut out = [0.0f32; 3];
    let mut add = |k: usize, sign: f32| {
        let s = step(k);
        for i in 0..3 {
            out[i] += sign * s[i];
        }
    };
    if delta >= 0.0 {
        let b = (raw * n as f32 + 0.001).floor() as usize;
        if b >= n {
            for k in a + 1..n {
                add(k, 1.0);
            }
            add(if first_in_file { n - 1 } else { 0 }, 1.0);
            return (0.0, out);
        }
        for k in a + 1..=b {
            add(k, 1.0);
        }
        (raw, out)
    } else if (raw * n as f32 + 0.001).floor() < 0.0 {
        // "Below 0" read on the frame, with rule 6's thousandth (ours: a
        // plain `raw < 0` skips frame 0 when float sums land a hair under).
        for k in (0..a).rev() {
            add(k, -1.0);
        }
        add(n - 1, -1.0);
        ((n - 1) as f32 / n as f32, out)
    } else {
        let b = frame(raw);
        for k in (b..a).rev() {
            add(k, -1.0);
        }
        (raw.max(0.0), out)
    }
}

/// What one update of a movement does ([`DoorMovement::step`]).
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    pub to: f32,
    /// A play-once movement reached its end (or has no length).
    pub done: bool,
    pub cues: Vec<String>,
}

/// No change.
pub fn identity() -> Transform {
    Transform { flags: 0, m: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], p: [0.0; 3] }
}

/// `outer` after `step`: a point goes through `step` first, then `outer`
/// (row vectors: m = step.m × outer.m, p = step.p × outer.m + outer.p).
pub fn chain(outer: &Transform, step: &Transform) -> Transform {
    let m = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| step.m[i][k] * outer.m[k][j]).sum()));
    let p = std::array::from_fn(|j| (0..3).map(|k| step.p[k] * outer.m[k][j]).sum::<f32>() + outer.p[j]);
    Transform { flags: 0, m, p }
}

/// Smooth animation (DESIGN S-5, ours; drawing only): a placement `w`
/// (0..1) of the way from `a` to `b`: the place on the straight line, the
/// turn by mixing the two matrices and making the mix a plain turn again
/// (rows of length 1 at right angles, the first row kept in its direction).
/// Good for the small turns between two updates or two frames.
pub fn mix_transform(a: &Transform, b: &Transform, w: f32) -> Transform {
    let lerp = |x: f32, y: f32| x + (y - x) * w;
    let p = std::array::from_fn(|j| lerp(a.p[j], b.p[j]));
    let m: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| lerp(a.m[i][j], b.m[i][j])));
    let len = |v: [f32; 3]| (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
    let unit = |v: [f32; 3], fallback: [f32; 3]| {
        let l = len(v);
        if l > 1e-6 { v.map(|x| x / l) } else { fallback }
    };
    let x = unit(m[0], a.m[0]);
    let y = unit(std::array::from_fn(|k| m[1][k] - dot(m[1], x) * x[k]), a.m[1]);
    let z = [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
    Transform { flags: 0, m: [x, y, z], p }
}

/// The angle a transform turns about Z, in degrees (for the log).
pub fn turn_degrees(t: &Transform) -> f32 {
    t.m[0][1].atan2(t.m[0][0]).to_degrees()
}

/// What one update did to a door.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Advance {
    /// Phase before and after.
    pub from: f32,
    pub to: f32,
    /// Cue actions that fired.
    pub cues: Vec<String>,
    /// `Open` or `Close` when an `open` / `close` movement finished: the
    /// caller calls the script with it.
    pub finished: Option<&'static str>,
    /// The name of the movement that finished, if any.
    pub finished_movement: Option<String>,
    /// A movement needed after finishing (`opened` / `closed`) that the
    /// door doesn't have.
    pub problem: Option<String>,
    /// The waiting movement that started in this update.
    pub started: Option<crate::named_anim::Started>,
}

#[derive(Debug, Clone)]
pub struct Door {
    pub name: String,
    pub state: DoorState,
    /// Its movements, the one shown, its phase and the waiting one
    /// (shared with chests and objects, `anim-scripts.md` rules 1–10).
    pub player: NamedAnimPlayer,
}

impl Door {
    /// A door showing movement `current` at its start phase, closed
    /// whatever that movement is (rule 1).
    pub fn new(name: &str, movements: Vec<DoorMovement>, current: usize, base: Placement) -> Self {
        Door { name: name.to_string(), state: DoorState::Closed, player: NamedAnimPlayer::new(name, movements, current, base) }
    }

    pub fn movement(&self) -> Option<&DoorMovement> {
        self.player.movement()
    }

    pub fn movement_name(&self) -> &str {
        self.player.movement_name()
    }

    /// Starts movement `name` (any capitals) at once at its start phase.
    /// `Err` if the door has no such movement.
    pub fn play(&mut self, name: &str) -> Result<(), String> {
        self.player.play(name)
    }

    /// `KQDoorEvent` (rules 2–4); `None` = no number (toggle). Returns the
    /// state before and a note (what happened, or why it was ignored).
    pub fn door_event(&mut self, value: Option<i64>) -> (DoorState, String) {
        let before = self.state;
        let wants = match value {
            Some(1) => "open",
            Some(2) => "close",
            _ => "toggle",
        };
        let target = match (wants, before) {
            ("open" | "toggle", DoorState::Closed) => Some(("open", DoorState::Opening)),
            ("close" | "toggle", DoorState::Opened) => Some(("close", DoorState::Closing)),
            _ => None,
        };
        let note = match target {
            Some((movement, state)) => match self.play(movement) {
                Ok(()) => {
                    self.state = state;
                    format!("{wants}: starts '{movement}'")
                }
                Err(e) => format!("{wants}: {e}; state unchanged"),
            },
            None => format!("{wants}: ignored while {}", before.name()),
        };
        (before, note)
    }

    /// One update of `dt` seconds: the phase moves, cues fire, a play-once
    /// movement finishes at 1 (forward) or 0 (backward). A waiting
    /// movement starts at the end (`anim-scripts.md` rule 8); then, if the
    /// finished one is `open` / `close`, the door rule: opened / closed,
    /// shows `opened` / `closed` (rule 5), which replaces it.
    pub fn advance(&mut self, dt: f32) -> Advance {
        let p = self.player.advance(dt);
        let mut out = Advance { from: p.from, to: p.to, cues: p.cues, started: p.started, ..Default::default() };
        let Some(name) = p.finished else { return out };
        let next = if name.eq_ignore_ascii_case("open") {
            Some((DoorState::Opened, "opened", "Open"))
        } else if name.eq_ignore_ascii_case("close") {
            Some((DoorState::Closed, "closed", "Close"))
        } else {
            None
        };
        out.finished_movement = Some(name);
        if let Some((state, still, call)) = next {
            self.state = state;
            if let Some(s) = &out.started {
                // Not covered by the notes (no data does it): logged.
                out.problem = Some(format!("the waiting movement '{}' started at the end, then the door rule shows '{still}'", s.name));
            }
            if let Err(e) = self.play(still) {
                out.problem = Some(e);
            }
            // Showing the still movement is not a new start to wait for.
            self.player.started_this_update = false;
            out.finished = Some(call);
        }
        out
    }

    /// The frame shown now.
    pub fn frame(&self) -> usize {
        self.player.frame()
    }

    /// The turn the animation gives the door now (own space).
    pub fn turn(&self) -> Transform {
        self.player.turn()
    }

    /// Where the door stands now: its turn, then its place in the world.
    pub fn placement(&self) -> Placement {
        self.player.placement()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A turn of `deg` about Z round the point (hx, hy), as one frame
    /// step (made-up numbers in the style of `door1.kqs`).
    pub fn hinge_step(deg: f32, hx: f32, hy: f32) -> Transform {
        let (s, c) = deg.to_radians().sin_cos();
        let m = [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]];
        // p such that the hinge stays put: h·m + p = h.
        let p = [hx - (hx * c - hy * s), hy - (hx * s + hy * c), 0.0];
        Transform { flags: 3, m, p }
    }

    /// A door like Daventry's: 16 frames over 1 s, 15 steps adding up to
    /// 165°, the four movements of the `.kq` files.
    pub fn test_door(close_cue: Option<f32>) -> Door {
        let mut steps = vec![identity()];
        steps.extend((1..16).map(|_| hinge_step(11.0, -634.0, -63.0)));
        let mv = |name: &str, start: f32, speed: f32, cues: Vec<Cue>| DoorMovement {
            name: name.into(),
            start,
            speed,
            repeat: false,
            cut_short: true,
            play_time: 1.0,
            frame_steps: steps.clone(),
            first_in_file: true,
            frame_parts: Vec::new(),
            cues,
            transitions: Vec::new(),
        };
        let close_cues = close_cue.map(|phase| Cue { phase, action: "playsound door4.aud".into() }).into_iter().collect();
        Door::new(
            "Condoor",
            vec![
                mv("closed", 0.0, 0.0, vec![]),
                mv("opened", 1.0, 0.0, vec![]),
                mv("open", 0.0, 1.0, vec![Cue { phase: 0.0, action: "playsound door4.aud".into() }]),
                mv("close", 0.93, -1.0, close_cues),
            ],
            0,
            Placement::at([1000.0, 2000.0, 300.0]),
        )
    }

    const DT: f32 = 1.0 / 15.0;

    /// Updates until the door stops moving: (updates moved, advances).
    fn run_out(d: &mut Door) -> (usize, Vec<Advance>) {
        let mut all = Vec::new();
        for _ in 0..100 {
            let a = d.advance(DT);
            let moved = a.from != a.to;
            let finished = a.finished.is_some();
            all.push(a);
            if finished || (!moved && !d.player.playing) {
                break;
            }
        }
        (all.iter().filter(|a| a.from != a.to).count(), all)
    }

    #[test]
    fn mixing_two_placements_halves_the_turn_and_the_move() {
        // A door step of 12 degrees about z with a hinge move.
        let turn = |deg: f32, p: [f32; 3]| {
            let (s, c) = deg.to_radians().sin_cos();
            Transform { flags: 0, m: [[c, s, 0.0], [-s, c, 0.0], [0.0, 0.0, 1.0]], p }
        };
        let (a, b) = (turn(0.0, [0.0; 3]), turn(12.0, [100.0, -20.0, 0.0]));
        let h = mix_transform(&a, &b, 0.5);
        assert!((turn_degrees(&h) - 6.0).abs() < 0.01, "{}", turn_degrees(&h));
        assert_eq!(h.p, [50.0, -10.0, 0.0]);
        // Rows of length 1 at right angles; the ends are the two placements.
        let dot = |u: [f32; 3], v: [f32; 3]| u[0] * v[0] + u[1] * v[1] + u[2] * v[2];
        assert!((dot(h.m[0], h.m[0]) - 1.0).abs() < 1e-5 && dot(h.m[0], h.m[1]).abs() < 1e-5);
        assert!((turn_degrees(&mix_transform(&a, &b, 1.0)) - 12.0).abs() < 1e-3);
        assert_eq!(mix_transform(&a, &b, 0.0).p, a.p);
    }

    #[test]
    fn cues_at_a_wrap_fire_once_including_one_at_exactly_1() {
        let mv = |speed: f32| DoorMovement {
            name: "spin".into(),
            start: 0.0,
            speed,
            repeat: true,
            cut_short: true,
            play_time: 1.0,
            frame_steps: vec![identity(); 16],
            first_in_file: true,
            frame_parts: Vec::new(),
            cues: vec![Cue { phase: 1.0, action: "one".into() }, Cue { phase: 0.0, action: "zero".into() }, Cue { phase: 0.5, action: "half".into() }],
            transitions: Vec::new(),
        };
        // Forwards over the wrap: 1.0 (old..1.0001); the wrap lands on
        // exactly 0 (rule 6), so the 0.0 cue fires in the next update
        // (rule 4, Q158 c).
        let s = mv(1.0).step(0.95, 0.1);
        assert!(s.to == 0.0 && !s.done);
        assert_eq!(s.cues, ["one"]);
        assert_eq!(mv(1.0).step(0.0, 0.1).cues, ["zero"]);
        // Backwards over the wrap: 0 (0..old) and 1.0 (new..1.0001); below
        // 0 lands on the last frame, 15/16.
        let s = mv(-1.0).step(0.05, 0.1);
        assert!((s.to - 15.0 / 16.0).abs() < 1e-5, "{}", s.to);
        assert_eq!(s.cues, ["one", "zero"]);
        // No wrap: the plain range.
        assert_eq!(mv(1.0).step(0.45, 0.1).cues, ["half"]);
        // A repeating loop with no length never ends; the frame rule wraps.
        let mut still = mv(1.0);
        still.play_time = 0.0;
        assert!(!still.step(0.3, 0.1).done);
        assert_eq!(mv(1.0).frame_at(0.99), 15);
        assert_eq!(mv(1.0).frame_at(0.3), 4);
        assert_eq!(mv(1.0).frame_at(1.0), 0);
    }

    #[test]
    fn the_state_table() {
        use DoorState::*;
        // (state, value) -> state after.
        for (state, value, after) in [
            (Closed, Some(1), Opening),
            (Closed, Some(2), Closed),
            (Closed, Some(0), Opening),
            (Closed, None, Opening),
            (Opened, Some(1), Opened),
            (Opened, Some(2), Closing),
            (Opened, Some(7), Closing),
            (Opened, None, Closing),
            (Opening, Some(1), Opening),
            (Opening, Some(2), Opening),
            (Opening, None, Opening),
            (Closing, Some(1), Closing),
            (Closing, Some(2), Closing),
            (Closing, Some(0), Closing),
        ] {
            let mut d = test_door(None);
            d.state = state;
            let (before, note) = d.door_event(value);
            assert_eq!((before, d.state), (state, after), "{state:?} {value:?}: {note}");
        }
    }

    #[test]
    fn open_takes_15_updates_then_calls_open_and_stays_opened() {
        let mut d = test_door(None);
        d.door_event(Some(1));
        assert_eq!((d.state, d.movement_name()), (DoorState::Opening, "open"));
        // The update it was started in: no movement yet.
        let first = d.advance(DT);
        assert_eq!(first.from, first.to);
        let (moved, all) = run_out(&mut d);
        assert_eq!(moved, 15);
        let last = all.last().unwrap();
        assert_eq!((last.finished, last.finished_movement.as_deref()), (Some("Open"), Some("open")));
        assert_eq!((d.state, d.movement_name(), d.player.phase, d.frame()), (DoorState::Opened, "opened", 1.0, 15));
        // A plain door's script sends 1 again: ignored, it stays open.
        d.door_event(Some(1));
        assert_eq!((d.state, d.movement_name()), (DoorState::Opened, "opened"));
        // The open cue at 0.0 fired in the first moving update only.
        let cues: Vec<usize> = all.iter().enumerate().filter(|(_, a)| !a.cues.is_empty()).map(|(i, _)| i).collect();
        assert_eq!(cues, [0]);
    }

    #[test]
    fn close_from_0_93_takes_14_updates_and_a_cue_at_0_95_never_fires() {
        let mut d = test_door(Some(0.95));
        d.door_event(Some(1));
        run_out(&mut d);
        d.advance(DT);
        d.door_event(Some(2));
        assert_eq!((d.state, d.player.phase), (DoorState::Closing, 0.93));
        d.advance(DT);
        let (moved, all) = run_out(&mut d);
        assert_eq!(moved, 14);
        assert!(all.iter().all(|a| a.cues.is_empty()), "Saradoor's close cue at 0.95 must stay silent");
        assert_eq!((all.last().unwrap().finished, d.state, d.movement_name()), (Some("Close"), DoorState::Closed, "closed"));
    }

    #[test]
    fn anim_play_open_from_closed_ends_opened_and_a_door_event_restarts_it() {
        let mut d = test_door(None);
        d.play("OPEN").unwrap();
        assert_eq!(d.state, DoorState::Closed, "rule 6: the state stays");
        d.advance(DT);
        for _ in 0..5 {
            d.advance(DT);
        }
        assert!(d.player.phase > 0.3);
        // Still closed, so KQDoorEvent 1 starts `open` again from the start.
        d.door_event(Some(1));
        assert_eq!((d.state, d.player.phase), (DoorState::Opening, 0.0));
        d.advance(DT);
        let (_, all) = run_out(&mut d);
        assert_eq!((all.last().unwrap().finished, d.state), (Some("Open"), DoorState::Opened));
        assert!(d.play("swing").is_err());
    }

    #[test]
    fn the_turn_follows_the_phase_and_never_drifts() {
        let mut d = test_door(None);
        let closed = d.placement();
        for _ in 0..10 {
            d.door_event(None);
            d.advance(DT);
            run_out(&mut d);
            assert!((turn_degrees(&d.turn()) - 165.0).abs() < 0.01, "{}", turn_degrees(&d.turn()));
            d.door_event(None);
            d.advance(DT);
            run_out(&mut d);
        }
        assert_eq!(d.state, DoorState::Closed);
        assert_eq!(d.frame(), 0);
        let now = d.placement();
        assert_eq!((now.rotation, now.position), (closed.rotation, closed.position));
        // `close` starts at 0.93, which shows frame 14 (with these equal
        // made-up steps 154°; in `door1.kqs` frame 14 is 167.75°, 2.75°
        // past frame 15).
        d.door_event(None);
        d.advance(DT);
        run_out(&mut d);
        d.door_event(None);
        assert_eq!(d.frame(), 14);
        // The hinge stays where it is, in the world too.
        let hinge = d.placement().to_world([-634.0, -63.0, 0.0]);
        assert!((hinge[0] - (1000.0 - 634.0)).abs() < 0.05 && (hinge[1] - (2000.0 - 63.0)).abs() < 0.05, "{hinge:?}");
    }
}
