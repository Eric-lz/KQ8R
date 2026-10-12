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

    /// Seconds one play takes: the whole loop when it repeats; from the
    /// first frame to the last (`frames − 1` frame times) when it doesn't.
    pub fn play_time(&self) -> f32 {
        if self.repeat || self.frames < 2 {
            self.duration
        } else {
            self.duration * (self.frames - 1) as f32 / self.frames as f32
        }
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
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Free => "free",
            Mode::Fall => "fall",
            Mode::Land => "land",
            Mode::Dying => "dying",
            Mode::Dead => "dead",
        }
    }

    /// The movement this mode plays (`None`: the keys decide).
    fn movement(&self) -> Option<&'static str> {
        match self {
            Mode::Free => None,
            Mode::Fall => Some("jumphold"),
            Mode::Land => Some("jumpend"),
            Mode::Dying | Mode::Dead => Some("jumpdie"),
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
        Walker { heading, movement: "stop", phase: 0.0, mode: Mode::Free }
    }

    /// Switch mode (from the mover's fall events, or a test lift). The new
    /// mode's animation starts at its beginning in the next `update`.
    pub fn set_mode(&mut self, mode: Mode) {
        self.mode = mode;
    }

    /// In the Move state: the keys chose a movement other than standing
    /// (walking, running, stepping back, turning on the spot).
    pub fn moving(&self) -> bool {
        self.mode == Mode::Free && self.movement != "stop"
    }

    /// A head-on hit while moving (connor-physics.md "Walls"): walking or
    /// stepping back switches straight to `stop`, with no stop animation
    /// (measured). Running is left alone until the crash (M2 step 2d-4).
    /// Returns the change, if any.
    pub fn stop_at_wall(&mut self) -> Option<(&'static str, &'static str)> {
        if !self.moving() || self.movement == "run" {
            return None;
        }
        let from = self.movement;
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
                Mode::Land => Some(Mode::Free),
                Mode::Dying => Some(Mode::Dead),
                _ => None,
            };
            if let Some(next) = next {
                mode_changed = Some((self.mode, next));
                self.mode = next;
            }
        }
        let movement = self.mode.movement().unwrap_or_else(|| choose_movement(intent));
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
        if self.mode == Mode::Free {
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
    fn head_on_stops_walking_and_stepping_back_not_running() {
        for (movement, stops) in [("walk", true), ("back", true), ("run", false), ("stop", false)] {
            let mut w = Walker::new(0.0);
            w.movement = movement;
            w.phase = 0.5;
            assert_eq!(w.stop_at_wall().is_some(), stops, "{movement}");
            assert_eq!(w.movement, if stops { "stop" } else { movement });
        }
        let mut w = Walker::new(0.0);
        w.movement = "walk";
        w.set_mode(Mode::Fall);
        assert_eq!(w.stop_at_wall(), None, "not while falling");
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

    /// A one-shot of 11 frames lasting 11/15 s, so 10/15 s to play.
    fn once() -> MovementLoop {
        MovementLoop { duration: 11.0 / 15.0, step_sum: [0.0; 3], speed: 1.0, repeat: false, frames: 11 }
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
