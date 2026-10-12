//! Connor's ground movement: which movement plays, turning, and the step
//! his animation asks the mover for.
//!
//! Rules from the behaviour note `docs/behaviour/connor-movement.md`
//! ("Movement intent", "Which movement plays", "Forward speed comes from
//! the animation", "Turning"). M2 step 2b: no start/stop transitions (Q25),
//! no banking, the loop's step spread evenly over the loop (Q24).
//!
//! The falling animations (M2 step 2c/2e, `connor-physics.md` "Falling off
//! an edge"): `jumphold` while falling, then `jumpend` (safe landing) or
//! `jumpdie` (death), each played once, with the keys ignored meanwhile.

use crate::UPDATES_PER_SECOND;

/// Turning speed while a turn key is held, radians per second.
pub const TURN_RATE: f32 = 1.75;

/// What the player wants: forward/back and turn are −1, 0 or +1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Intent {
    pub forward: i8,
    pub turn: i8,
    pub run: bool,
}

/// The movement name for an intent (the note's table; no water yet).
pub fn choose_movement(intent: Intent) -> &'static str {
    match (intent.forward, intent.turn) {
        (1, _) if intent.run => "run",
        (1, _) => "walk",
        (-1, _) => "back",
        (0, 1) => "left",
        (0, -1) => "right",
        _ => "stop",
    }
}

/// One movement's animation, as far as moving is concerned.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MovementLoop {
    /// Length of the loop in seconds.
    pub duration: f32,
    /// The loop's per-frame steps added up, in model space (Z up; the
    /// model faces −Y).
    pub step_sum: [f32; 3],
    /// Playback speed from `Conner.anm` (`back`: −1).
    pub speed: f32,
    /// True = repeats; false = plays once, first to last frame
    /// (`objects.md` "Animated objects at load", loop rule 2).
    pub repeat: bool,
    /// Number of frames.
    pub frames: usize,
}

impl MovementLoop {
    /// Units per second along the loop's step (negative when played back).
    pub fn ground_speed(&self) -> f32 {
        let [x, y, _] = self.step_sum;
        if self.duration > 0.0 { (x * x + y * y).sqrt() / self.duration * self.speed } else { 0.0 }
    }

    /// Seconds one play takes: the stored loop length, repeating or not.
    /// It already runs from the first frame to the last (`crash`: 8 frames
    /// in 7/15 s, and measured: its whole step in 7 ticks; DESIGN 2d-4).
    pub fn play_time(&self) -> f32 {
        self.duration
    }

    /// The frame shown at `phase` (0..1 of one play).
    pub fn frame_at(&self, phase: f32) -> usize {
        let n = self.frames.max(1);
        if self.repeat {
            ((phase.rem_euclid(1.0) * n as f32) as usize).min(n - 1)
        } else {
            (phase.clamp(0.0, 1.0) * (n - 1) as f32).round() as usize
        }
    }
}

/// What drives Connor's animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// The keys choose the movement.
    Free,
    /// Falling: `jumphold`, repeating.
    Fall,
    /// Safe landing: `jumpend` once, then back to the keys.
    Land,
    /// Fatal landing: `jumpdie` once, then dead.
    Dying,
    /// The Die state: holds the last frame of `jumpdie`.
    Dead,
    /// Ran head-on into a wall: `crash` once (it pushes him back), then
    /// back to the keys.
    Crash,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Free => "free",
            Mode::Fall => "fall",
            Mode::Land => "land",
            Mode::Dying => "dying",
            Mode::Dead => "dead",
            Mode::Crash => "crash",
        }
    }

    /// The movement this mode plays (`None`: the keys decide).
    fn movement(&self) -> Option<&'static str> {
        match self {
            Mode::Free => None,
            Mode::Fall => Some("jumphold"),
            Mode::Land => Some("jumpend"),
            Mode::Dying | Mode::Dead => Some("jumpdie"),
            Mode::Crash => Some("crash"),
        }
    }
}

/// Connor's heading, current movement and place in its loop.
#[derive(Debug, Clone)]
pub struct Walker {
    /// Radians about Z; 0 = facing −Y (the model's own forward).
    pub heading: f32,
    pub movement: &'static str,
    /// Position in the current loop, 0..1.
    pub phase: f32,
    pub mode: Mode,
    /// Wall blocks set by a head-on stop: while either is on he stands
    /// (connor-physics.md "Walk held against a wall", traced, Q38).
    pub blocked_forward: bool,
    pub blocked_back: bool,
}

/// A movement key, for the key events that clear wall blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveKey {
    Forward,
    Back,
    /// Left or right.
    Turn,
}

/// What one update asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Movement change for the mover, world space.
    pub wanted: [f32; 3],
    /// Set when the movement changed this update: (from, to).
    pub changed: Option<(&'static str, &'static str)>,
    /// The movement has no loop in the animation list.
    pub missing: bool,
    /// Set when the mode changed this update: (from, to).
    pub mode_changed: Option<(Mode, Mode)>,
}

impl Walker {
    pub fn new(heading: f32) -> Self {
        Walker { heading, movement: "stop", phase: 0.0, mode: Mode::Free, blocked_forward: false, blocked_back: false }
    }

    /// Switch mode (from the mover's fall events, or a test lift). The new
    /// mode's animation starts at its beginning in the next `update`. Wall
    /// blocks stay as they are (traced: only key events clear them).
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// Either wall block is on.
    pub fn blocked(&self) -> bool {
        self.blocked_forward || self.blocked_back
    }

    /// The wall blocks for the log: `none`, `fwd`, `back` or `both`.
    pub fn blocked_name(&self) -> &'static str {
        match (self.blocked_forward, self.blocked_back) {
            (false, false) => "none",
            (true, false) => "fwd",
            (false, true) => "back",
            (true, true) => "both",
        }
    }

    /// A movement key pressed or released: clears wall blocks as traced.
    /// Forward released → forward; back released → back; forward pressed
    /// → back; back pressed → forward; a turn key pressed → both.
    /// Returns what it cleared (forward, back).
    pub fn key_event(&mut self, key: MoveKey, pressed: bool) -> (bool, bool) {
        let (forward, back) = match (key, pressed) {
            (MoveKey::Forward, false) | (MoveKey::Back, true) => (true, false),
            (MoveKey::Back, false) | (MoveKey::Forward, true) => (false, true),
            (MoveKey::Turn, true) => (true, true),
            (MoveKey::Turn, false) => (false, false),
        };
        let cleared = (forward && self.blocked_forward, back && self.blocked_back);
        self.blocked_forward &= !forward;
        self.blocked_back &= !back;
        cleared
    }

    /// In the Move state: the keys chose a movement other than standing
    /// (walking, running, stepping back, turning on the spot).
    pub fn moving(&self) -> bool {
        self.mode == Mode::Free && self.movement != "stop"
    }

    /// A head-on hit while moving (connor-physics.md "Walls"). Running:
    /// `crash` plays in full from the next update (traced, Q31 e). Walking
    /// or stepping back switches straight to `stop`, with no stop animation
    /// (measured), and sets a wall block: back blocked after `back`,
    /// forward blocked otherwise (traced, Q38). Returns the change, if any.
    pub fn head_on_at_wall(&mut self) -> Option<(&'static str, &'static str)> {
        if !self.moving() {
            return None;
        }
        if self.movement == "run" {
            self.set_mode(Mode::Crash);
            return Some(("run", "crash"));
        }
        let from = self.movement;
        if from == "back" {
            self.blocked_back = true;
        } else {
            self.blocked_forward = true;
        }
        self.movement = "stop";
        self.phase = 0.0;
        Some((from, "stop"))
    }

    /// One update (1/15 s). `loops` gives a movement's loop by name.
    pub fn update(&mut self, intent: Intent, loops: &dyn Fn(&str) -> Option<MovementLoop>) -> Step {
        let dt = (1.0 / UPDATES_PER_SECOND) as f32;
        let mut mode_changed = None;
        // A one-shot animation that finished last update hands over.
        if self.phase >= 1.0 {
            let next = match self.mode {
                Mode::Land | Mode::Crash => Some(Mode::Free),
                Mode::Dying => Some(Mode::Dead),
                _ => None,
            };
            if let Some(next) = next {
                mode_changed = Some((self.mode, next));
                self.mode = next;
            }
        }
        // While a wall block is on he stands, whatever the keys ask.
        let movement = self.mode.movement().unwrap_or_else(|| if self.blocked() { "stop" } else { choose_movement(intent) });
        let mut changed = None;
        if movement != self.movement {
            // No transition animations yet (Q25): the new loop starts at
            // its beginning.
            changed = Some((self.movement, movement));
            self.movement = movement;
            self.phase = 0.0;
        }
        let none = |changed, missing| Step { wanted: [0.0; 3], changed, missing, mode_changed };
        if self.mode == Mode::Dead {
            self.phase = 1.0;
            return none(changed, false);
        }
        // The keys turn him only when they choose the movement.
        if self.mode == Mode::Free && !self.blocked() {
            self.heading += f32::from(intent.turn) * TURN_RATE * dt;
        }
        let Some(l) = loops(movement) else {
            return none(changed, true);
        };
        let time = l.play_time();
        if time <= 0.0 {
            return none(changed, false);
        }
        // Spread the loop's step evenly over its length (Q24).
        // Share of one play this update (negative when played backwards).
        let mut share = dt * l.speed / time;
        if l.repeat {
            self.phase = (self.phase + share).rem_euclid(1.0);
        } else {
            // One-shots run forward and stop on the last frame.
            share = share.abs().min(1.0 - self.phase);
            self.phase += share;
            if self.phase > 1.0 - 1e-4 {
                self.phase = 1.0;
            }
            share *= l.speed.signum();
        }
        let k = share * time / l.duration;
        let [x, y, z] = l.step_sum.map(|v| v * k);
        // Turn the model-space step by the heading (positive = left, CCW).
        let (s, c) = self.heading.sin_cos();
        Step { wanted: [x * c - y * s, x * s + y * c, z], changed, missing: false, mode_changed }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(forward: i8, turn: i8, run: bool) -> Intent {
        Intent { forward, turn, run }
    }

    #[test]
    fn head_on_stops_walking_and_stepping_back_and_crashes_running() {
        for (movement, change, mode) in [
            ("walk", Some(("walk", "stop")), Mode::Free),
            ("back", Some(("back", "stop")), Mode::Free),
            ("run", Some(("run", "crash")), Mode::Crash),
            ("stop", None, Mode::Free),
        ] {
            let mut w = Walker::new(0.0);
            w.movement = movement;
            w.phase = 0.5;
            assert_eq!(w.head_on_at_wall(), change, "{movement}");
            assert_eq!(w.mode, mode, "{movement}");
        }
        let mut w = Walker::new(0.0);
        w.movement = "walk";
        w.set_mode(Mode::Fall);
        assert_eq!(w.head_on_at_wall(), None, "not while falling");
        w.movement = "crash";
        w.set_mode(Mode::Crash);
        assert_eq!(w.head_on_at_wall(), None, "not while crashing");
    }

    /// `crash` as in the data: 8 frames in 7/15 s, 441.5 back (+Y in the
    /// model, which faces −Y).
    fn crash() -> MovementLoop {
        MovementLoop { duration: 7.0 / 15.0, step_sum: [0.0, 441.5, 0.0], speed: 1.0, repeat: false, frames: 8 }
    }

    #[test]
    fn running_head_on_crashes_7_updates_441_5_back_then_runs() {
        let run = MovementLoop { duration: 14.0 / 15.0, step_sum: [0.0, -2350.2, 0.0], speed: 1.0, repeat: true, frames: 15 };
        let loops = |m: &str| match m {
            "run" => Some(run),
            "crash" => Some(crash()),
            _ => None,
        };
        let held = intent(1, 1, true);
        let mut w = Walker::new(0.0);
        w.update(held, &loops);
        assert_eq!(w.head_on_at_wall(), Some(("run", "crash")));
        // Straight back from his facing (he turned a little while running).
        let heading = w.heading;
        let behind = [-heading.sin(), heading.cos()];
        let mut back = 0.0;
        for n in 1..=7 {
            let s = w.update(held, &loops);
            assert_eq!((w.movement, w.mode), ("crash", Mode::Crash), "update {n}");
            let along = s.wanted[0] * behind[0] + s.wanted[1] * behind[1];
            assert!((along - 63.07).abs() < 0.01 && s.wanted[0].hypot(s.wanted[1]) - along < 1e-3, "update {n}: {s:?}");
            assert_eq!(crash().frame_at(w.phase), n, "update {n}: frame");
            back += along;
        }
        assert!((back - 441.5).abs() < 0.01, "{back}");
        // Keys ignored while it played (no turning); then running again.
        assert_eq!(w.heading, heading);
        let s = w.update(held, &loops);
        assert_eq!(s.mode_changed, Some((Mode::Crash, Mode::Free)));
        assert_eq!(w.movement, "run");
        assert!(s.wanted[0] * behind[0] + s.wanted[1] * behind[1] < 0.0, "runs forward: {s:?}");
    }

    fn stop_loops(m: &str) -> Option<MovementLoop> {
        match m {
            "walk" => Some(walk(1.0)),
            "back" => Some(walk(-1.0)),
            "stop" => Some(MovementLoop { step_sum: [0.0; 3], ..walk(1.0) }),
            _ => None,
        }
    }

    #[test]
    fn a_wall_stop_blocks_until_the_key_is_released() {
        for (forward, movement, key) in [(1, "walk", MoveKey::Forward), (-1, "back", MoveKey::Back)] {
            let mut w = Walker::new(0.0);
            w.update(intent(forward, 0, false), &stop_loops);
            assert_eq!(w.head_on_at_wall(), Some((movement, "stop")));
            assert_eq!(w.blocked_name(), if forward > 0 { "fwd" } else { "back" });
            // Key held for 3 s: standing, the idle animation playing on.
            let mut last = w.phase;
            for n in 0..45 {
                let s = w.update(intent(forward, 0, false), &stop_loops);
                assert_eq!((w.movement, s.wanted), ("stop", [0.0; 3]), "{movement} update {n}");
                assert_ne!(w.phase, last, "{movement} update {n}: stop's frame moves on");
                last = w.phase;
            }
            // Pressing the same key again clears nothing; releasing it does.
            assert_eq!(w.key_event(key, true), (false, false));
            assert_eq!(w.key_event(key, false), (forward > 0, forward < 0));
            w.update(intent(0, 0, false), &stop_loops);
            assert_eq!(w.key_event(key, true), (false, false));
            w.update(intent(forward, 0, false), &stop_loops);
            assert_eq!(w.movement, movement);
        }
    }

    #[test]
    fn while_blocked_he_neither_runs_nor_turns_on_the_spot() {
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &stop_loops);
        w.head_on_at_wall();
        for keys in [intent(1, 0, true), intent(1, 1, false), intent(0, -1, false), intent(-1, 0, false)] {
            w.update(keys, &stop_loops);
            assert_eq!((w.movement, w.heading), ("stop", 0.0), "{keys:?}");
        }
        // A mode change (falling, landing) leaves the block on.
        w.set_mode(Mode::Fall);
        w.set_mode(Mode::Free);
        assert!(w.blocked_forward);
    }

    #[test]
    fn which_key_events_clear_which_block() {
        use MoveKey::*;
        let both = || Walker { blocked_forward: true, blocked_back: true, ..Walker::new(0.0) };
        for (key, pressed, cleared) in [
            (Forward, false, (true, false)),
            (Back, false, (false, true)),
            (Forward, true, (false, true)),
            (Back, true, (true, false)),
            (Turn, true, (true, true)),
            (Turn, false, (false, false)),
        ] {
            let mut w = both();
            assert_eq!(w.key_event(key, pressed), cleared, "{key:?} pressed {pressed}");
            assert_eq!((w.blocked_forward, w.blocked_back), (!cleared.0, !cleared.1), "{key:?} pressed {pressed}");
        }
        // Forward held against the wall, a turn key pressed: he walks
        // again, turning as he goes (forward wins over turning).
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &stop_loops);
        w.head_on_at_wall();
        w.key_event(Turn, true);
        w.update(intent(1, 1, false), &stop_loops);
        assert_eq!(w.movement, "walk");
        assert!(w.heading > 0.0);
    }

    #[test]
    fn movement_table() {
        assert_eq!(choose_movement(intent(1, 0, true)), "run");
        assert_eq!(choose_movement(intent(1, 1, false)), "walk");
        assert_eq!(choose_movement(intent(-1, -1, true)), "back");
        assert_eq!(choose_movement(intent(0, 1, true)), "left");
        assert_eq!(choose_movement(intent(0, -1, false)), "right");
        assert_eq!(choose_movement(intent(0, 0, true)), "stop");
    }

    /// The note's walk: 1,427.1 per 1.2667 s loop, along −Y.
    fn walk(speed: f32) -> MovementLoop {
        MovementLoop { duration: 1.2667, step_sum: [0.0, -1427.1, 0.0], speed, repeat: true, frames: 20 }
    }

    /// A one-shot of 11 frames, 10/15 s from the first to the last (as
    /// `jumpend` in the data).
    fn once() -> MovementLoop {
        MovementLoop { duration: 10.0 / 15.0, step_sum: [0.0; 3], speed: 1.0, repeat: false, frames: 11 }
    }

    #[test]
    fn landing_plays_once_then_the_keys_take_over() {
        let mut w = Walker::new(0.0);
        let loops = |m: &str| match m {
            "walk" => Some(walk(1.0)),
            "jumphold" => Some(MovementLoop { repeat: true, ..once() }),
            "jumpend" => Some(once()),
            _ => None,
        };
        w.set_mode(Mode::Fall);
        for _ in 0..3 {
            let s = w.update(intent(1, 1, false), &loops);
            assert_eq!(w.movement, "jumphold");
            assert_eq!(s.wanted, [0.0; 3], "no walking while falling");
        }
        assert_eq!(w.heading, 0.0, "no turning while falling");
        w.set_mode(Mode::Land);
        // 10 updates of jumpend (10/15 s), keys ignored; then walking.
        for n in 1..=10 {
            w.update(intent(1, 0, false), &loops);
            assert_eq!((w.movement, w.mode), ("jumpend", Mode::Land), "update {n}");
        }
        assert_eq!(once().frame_at(w.phase), 10, "last frame shown");
        let s = w.update(intent(1, 0, false), &loops);
        assert_eq!(s.mode_changed, Some((Mode::Land, Mode::Free)));
        assert_eq!(w.movement, "walk");
    }

    #[test]
    fn dying_ends_dead_on_the_last_frame() {
        let mut w = Walker::new(0.0);
        let loops = |m: &str| (m == "jumpdie").then(once);
        w.set_mode(Mode::Dying);
        let mut changes = Vec::new();
        for _ in 0..20 {
            changes.extend(w.update(intent(1, 0, false), &loops).mode_changed);
        }
        assert_eq!(changes, vec![(Mode::Dying, Mode::Dead)]);
        assert_eq!((w.movement, w.mode), ("jumpdie", Mode::Dead));
        assert_eq!(once().frame_at(w.phase), 10);
    }

    #[test]
    fn walking_10_seconds_covers_about_11270() {
        let mut w = Walker::new(0.0);
        let loops = |m: &str| (m == "walk").then(|| walk(1.0));
        let mut y = 0.0;
        for _ in 0..150 {
            y += w.update(intent(1, 0, false), &loops).wanted[1];
        }
        assert!((y + 11_266.6).abs() < 5.0, "{y}");
        assert!((walk(1.0).ground_speed() - 1126.6).abs() < 1.0);
    }

    #[test]
    fn back_moves_the_other_way() {
        let mut w = Walker::new(0.0);
        let loops = |m: &str| (m == "back").then(|| walk(-1.0));
        let s = w.update(intent(-1, 0, false), &loops);
        assert!(s.wanted[1] > 0.0, "{s:?}");
        assert_eq!(s.changed, Some(("stop", "back")));
    }

    #[test]
    fn turning_left_2_seconds_is_3_5_radians() {
        let mut w = Walker::new(0.0);
        let loops = |_: &str| None;
        for _ in 0..30 {
            w.update(intent(0, 1, false), &loops);
        }
        assert!((w.heading - 3.5).abs() < 1e-4, "{}", w.heading);
    }

    #[test]
    fn heading_turns_the_step() {
        // Turned left by 90°: facing −Y becomes facing +X.
        let mut w = Walker::new(std::f32::consts::FRAC_PI_2);
        let loops = |m: &str| (m == "walk").then(|| walk(1.0));
        let s = w.update(intent(1, 0, false), &loops);
        assert!(s.wanted[0] > 0.0 && s.wanted[1].abs() < 1e-3, "{s:?}");
    }
}
