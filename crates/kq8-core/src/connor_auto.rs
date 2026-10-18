//! Connor turned or walked by a script (M3 step 3l-2): `KQConner::walkTo`,
//! `KQConner::runTo`, `pointAndCue` and `walkExit`.
//!
//! Rules from `anim-scripts.md` rules 24–26 and `connor-movement.md`
//! "Turning" rule 2: he first turns to face the spot with the automatic
//! turn (3.5 rad/s, the shorter way, exactly onto the heading once the
//! rest is under one update's turn; a player turn cancels it); a walk then
//! goes **straight ahead** in Move, with no more steering, until he has
//! gone the flat distance measured when the walk started. At the end (or
//! at once for a turn already done) the script's object gets
//! `KQCueEvent <c> <d>`. Nothing times out (rule 26).
//!
//! What plays during the turn (rule 24, corrected 2026-10-18, Q94 c): the
//! automatic turn counts as a held turn key for his standing state: the
//! turn's first update `Stop`, the second `Left`/`Right` (Move), from the
//! third `Walk` (`Run`) while the turn finishes; when the turn ends Move
//! starts afresh and the distance is measured again (M3 3m-3).

use crate::connor::Intent;

/// The automatic turn, radians per second (double the key turn).
pub const AUTO_TURN_RATE: f32 = 3.5;

/// The cue a scripted turn or walk sends when it is done: `KQCueEvent
/// <c> <d>` to `who`.
#[derive(Debug, Clone, PartialEq)]
pub struct Cue {
    pub who: String,
    pub c: String,
    pub d: String,
}

/// What was asked.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoKind {
    /// `pointAndCue`: only the turn.
    Point,
    /// `walkTo` (`run` false) or `runTo` (`run` true).
    Walk { run: bool },
}

/// Where it stands.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AutoPhase {
    Turning,
    /// Walking straight ahead from `from` until `distance` (flat) is gone.
    Walking { from: [f32; 2], distance: f32 },
}

/// A scripted turn or walk in progress.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoMove {
    pub target: [f32; 2],
    pub kind: AutoKind,
    pub phase: AutoPhase,
    pub cue: Option<Cue>,
    /// Updates of the automatic turn so far (rule 24: what plays).
    pub turn_updates: u32,
}

/// How a scripted turn or walk ended, for the caller (the cue) and the log.
#[derive(Debug, Clone, PartialEq)]
pub struct AutoEnd {
    pub cue: Option<Cue>,
    pub why: String,
}

/// What the scripted movement does this update, before the walker's own
/// update.
#[derive(Debug, Clone, PartialEq)]
pub enum Before {
    /// Nothing scripted is going on: the player's intent.
    Keys,
    /// Turning (update `n` of the turn): the walker gets `intent`.
    Turned { by: f32, left: f32, n: u32, intent: Intent },
    /// The turn is over in update `n` (and the walk, if any, starts now
    /// afresh): the walker gets `intent` this update.
    TurnDone { n: u32, intent: Intent },
    /// A player turn key cancelled the automatic turn: no cue ever comes.
    Cancelled,
    /// Walking: the walker gets this intent.
    Walk(Intent),
}

/// The heading that faces (x, y) from `from` (heading 0 faces −Y,
/// positive turns left, as `Walker::heading`).
pub fn heading_towards(from: [f32; 3], to: [f32; 2]) -> f32 {
    (to[0] - from[0]).atan2(-(to[1] - from[1]))
}

/// An angle brought into −π..π.
pub fn wrap(a: f32) -> f32 {
    let t = std::f32::consts::TAU;
    let a = a.rem_euclid(t);
    if a > std::f32::consts::PI { a - t } else { a }
}

fn flat_distance(a: [f32; 2], b: [f32; 2]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

impl AutoMove {
    pub fn new(kind: AutoKind, target: [f32; 2], cue: Option<Cue>) -> Self {
        AutoMove { target, kind, phase: AutoPhase::Turning, cue, turn_updates: 0 }
    }

    /// What the walker is asked for in update `n` of the turn, turning
    /// toward `dir` (+1 left, −1 right; rules 24–25): update 1 nothing
    /// (`Stop`, Wait); update 2 the turn direction as a turn key (`Left` /
    /// `Right`, Move); from update 3 a walk (`Walk`, `Run`) with the turn
    /// still going, or for `pointAndCue` `Left` / `Right` to the end. A
    /// wall block keeps him standing (the walker's own rule).
    pub fn turn_intent(&self, n: u32, dir: i8) -> Intent {
        match (n, self.kind) {
            (0 | 1, _) => Intent::default(),
            (2, _) | (_, AutoKind::Point) => Intent { forward: 0, turn: dir, run: false },
            (_, AutoKind::Walk { run }) => Intent { forward: 1, turn: dir, run },
        }
    }

    /// A walk or run is under way: after the turn, or during its third
    /// update on (rule 24: he walks the rest of the turn).
    pub fn walking(&self) -> bool {
        match (self.phase, self.kind) {
            (AutoPhase::Walking { .. }, _) => true,
            (AutoPhase::Turning, AutoKind::Walk { .. }) => self.turn_updates >= 3,
            _ => false,
        }
    }

    /// Before the walker's update: one update's automatic turn (changing
    /// `heading`), or the walk's intent. `player_turn`: the player's turn
    /// key (0 while `handsOff` is on).
    pub fn before(&mut self, position: [f32; 3], heading: &mut f32, player_turn: i8, dt: f32) -> Before {
        match self.phase {
            AutoPhase::Turning => {
                if player_turn != 0 {
                    return Before::Cancelled;
                }
                let left = wrap(heading_towards(position, self.target) - *heading);
                let step = AUTO_TURN_RATE * dt;
                self.turn_updates += 1;
                let n = self.turn_updates;
                let dir = if left >= 0.0 { 1 } else { -1 };
                if left.abs() < step {
                    *heading += left;
                    let intent = match self.kind {
                        AutoKind::Walk { run } => {
                            // Move starts afresh: the distance is measured
                            // again from here.
                            let from = [position[0], position[1]];
                            self.phase = AutoPhase::Walking { from, distance: flat_distance(from, self.target) };
                            Intent { forward: 1, turn: 0, run }
                        }
                        AutoKind::Point => self.turn_intent(n, dir),
                    };
                    return Before::TurnDone { n, intent };
                }
                *heading += step * left.signum();
                Before::Turned { by: step * left.signum(), left: left - step * left.signum(), n, intent: self.turn_intent(n, dir) }
            }
            AutoPhase::Walking { .. } => {
                let run = matches!(self.kind, AutoKind::Walk { run: true });
                Before::Walk(Intent { forward: 1, turn: 0, run })
            }
        }
    }

    /// After the move: whether the walk has gone its distance (measured
    /// from where it started, so a push off the line counts too).
    pub fn walk_done(&self, position: [f32; 3]) -> Option<(f32, f32)> {
        match self.phase {
            AutoPhase::Walking { from, distance } => {
                let gone = flat_distance(from, [position[0], position[1]]);
                (gone >= distance).then_some((gone, distance))
            }
            AutoPhase::Turning => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const DT: f32 = 1.0 / 15.0;

    #[test]
    fn heading_towards_follows_the_walkers_axes() {
        // Heading 0 faces −Y; π/2 faces +X (sin h, −cos h).
        assert!(heading_towards([0.0; 3], [0.0, -10.0]).abs() < 1e-6);
        assert!((heading_towards([0.0; 3], [10.0, 0.0]) - std::f32::consts::FRAC_PI_2).abs() < 1e-6);
    }

    #[test]
    fn the_automatic_turn_is_3_5_per_second_the_short_way_and_lands_exactly() {
        let mut a = AutoMove::new(AutoKind::Point, [0.0, -10.0], None);
        // Facing +X (π/2): the target is 1.5708 to the right.
        let mut h = std::f32::consts::FRAC_PI_2;
        let mut turns = 0;
        loop {
            match a.before([0.0; 3], &mut h, 0, DT) {
                Before::Turned { by, .. } => {
                    assert!((by + AUTO_TURN_RATE * DT).abs() < 1e-6);
                    turns += 1;
                }
                Before::TurnDone { .. } => break,
                other => panic!("{other:?}"),
            }
        }
        // 1.5708 / 0.2333 = 6.7: 6 full steps, then the rest.
        assert_eq!(turns, 6);
        assert!(h.abs() < 1e-6, "{h}");
    }

    #[test]
    fn a_turn_already_done_ends_at_once_and_a_player_turn_cancels() {
        let mut a = AutoMove::new(AutoKind::Point, [0.0, -10.0], None);
        let mut h = 0.0;
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::TurnDone { n: 1, intent: Intent::default() });
        let mut b = AutoMove::new(AutoKind::Point, [10.0, 0.0], None);
        assert_eq!(b.before([0.0; 3], &mut h, 1, DT), Before::Cancelled);
    }

    #[test]
    fn the_walk_goes_straight_until_the_distance_is_gone() {
        let mut a = AutoMove::new(AutoKind::Walk { run: false }, [0.0, -2740.0], None);
        let mut h = 0.0;
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::TurnDone { n: 1, intent: Intent { forward: 1, turn: 0, run: false } });
        assert_eq!(a.phase, AutoPhase::Walking { from: [0.0, 0.0], distance: 2740.0 });
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::Walk(Intent { forward: 1, turn: 0, run: false }));
        assert_eq!(a.walk_done([0.0, -2700.0, 0.0]), None);
        // Pushed off the line: the distance from the start counts.
        assert!(a.walk_done([300.0, -2730.0, 0.0]).is_some());
    }

    #[test]
    fn during_the_turn_stop_then_left_then_walk_and_the_distance_is_measured_again() {
        // Rule 24 (Q94 c): the watermill's 0.576 rad turn takes 3 updates
        // (0.233, 0.233, 0.110): Stop, Left, then Walk while it finishes.
        let mut a = AutoMove::new(AutoKind::Walk { run: false }, [0.0, -1000.0], None);
        let mut h = -0.576f32;
        let walk = |turn| Intent { forward: 1, turn, run: false };
        assert!(matches!(a.before([0.0; 3], &mut h, 0, DT), Before::Turned { n: 1, intent, .. } if intent == Intent::default()));
        assert!(!a.walking());
        assert!(matches!(a.before([0.0; 3], &mut h, 0, DT), Before::Turned { n: 2, intent: Intent { forward: 0, turn: 1, run: false }, .. }));
        // He has walked a little during the turn; measured again at its end.
        assert_eq!(a.before([0.0, 50.0, 0.0], &mut h, 0, DT), Before::TurnDone { n: 3, intent: walk(0) });
        assert_eq!(a.phase, AutoPhase::Walking { from: [0.0, 50.0], distance: 1050.0 });
        // A longer turn walks from its third update, still turning.
        let mut b = AutoMove::new(AutoKind::Walk { run: true }, [0.0, -1000.0], None);
        let mut h = 1.2f32;
        for n in 1..=3 {
            if let Before::Turned { intent, .. } = b.before([0.0; 3], &mut h, 0, DT) {
                assert_eq!(intent, b.turn_intent(n, -1));
            }
        }
        assert_eq!(b.turn_intent(3, -1), Intent { forward: 1, turn: -1, run: true });
        assert!(b.walking());
        // pointAndCue: Left/Right to the end of the turn.
        let p = AutoMove::new(AutoKind::Point, [0.0, -1000.0], None);
        assert_eq!(p.turn_intent(5, 1), Intent { forward: 0, turn: 1, run: false });
    }
}
