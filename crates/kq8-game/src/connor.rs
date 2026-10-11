//! Connor's ground movement: which movement plays, turning, and the step
//! his animation asks the mover for.
//!
//! Rules from the behaviour note `docs/behaviour/connor-movement.md`
//! ("Movement intent", "Which movement plays", "Forward speed comes from
//! the animation", "Turning"). M2 step 2b: no start/stop transitions (Q25),
//! no banking, the loop's step spread evenly over the loop (Q24).

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
}

impl MovementLoop {
    /// Units per second along the loop's step (negative when played back).
    pub fn ground_speed(&self) -> f32 {
        let [x, y, _] = self.step_sum;
        if self.duration > 0.0 { (x * x + y * y).sqrt() / self.duration * self.speed } else { 0.0 }
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
}

impl Walker {
    pub fn new(heading: f32) -> Self {
        Walker { heading, movement: "stop", phase: 0.0 }
    }

    /// One update (1/15 s). `loops` gives a movement's loop by name.
    pub fn update(&mut self, intent: Intent, loops: &dyn Fn(&str) -> Option<MovementLoop>) -> Step {
        let dt = (1.0 / UPDATES_PER_SECOND) as f32;
        let movement = choose_movement(intent);
        let mut changed = None;
        if movement != self.movement {
            // No transition animations yet (Q25): the new loop starts at
            // its beginning.
            changed = Some((self.movement, movement));
            self.movement = movement;
            self.phase = 0.0;
        }
        self.heading += f32::from(intent.turn) * TURN_RATE * dt;
        let Some(l) = loops(movement) else {
            return Step { wanted: [0.0; 3], changed, missing: true };
        };
        if l.duration <= 0.0 {
            return Step { wanted: [0.0; 3], changed, missing: false };
        }
        // Spread the loop's step evenly over its length (Q24).
        let share = dt * l.speed / l.duration;
        self.phase = (self.phase + share).rem_euclid(1.0);
        let [x, y, z] = l.step_sum.map(|v| v * share);
        // Turn the model-space step by the heading (positive = left, CCW).
        let (s, c) = self.heading.sin_cos();
        Step { wanted: [x * c - y * s, x * s + y * c, z], changed, missing: false }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn intent(forward: i8, turn: i8, run: bool) -> Intent {
        Intent { forward, turn, run }
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
        MovementLoop { duration: 1.2667, step_sum: [0.0, -1427.1, 0.0], speed }
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
