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
    /// Turning; the walker gets no keys (Wait).
    Turned { by: f32, left: f32 },
    /// The turn is over (and the walk, if any, starts now).
    TurnDone,
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
        AutoMove { target, kind, phase: AutoPhase::Turning, cue }
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
                if left.abs() < step {
                    *heading += left;
                    if let AutoKind::Walk { .. } = self.kind {
                        let from = [position[0], position[1]];
                        self.phase = AutoPhase::Walking { from, distance: flat_distance(from, self.target) };
                    }
                    return Before::TurnDone;
                }
                *heading += step * left.signum();
                Before::Turned { by: step * left.signum(), left: left - step * left.signum() }
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
                Before::TurnDone => break,
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
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::TurnDone);
        let mut b = AutoMove::new(AutoKind::Point, [10.0, 0.0], None);
        assert_eq!(b.before([0.0; 3], &mut h, 1, DT), Before::Cancelled);
    }

    #[test]
    fn the_walk_goes_straight_until_the_distance_is_gone() {
        let mut a = AutoMove::new(AutoKind::Walk { run: false }, [0.0, -2740.0], None);
        let mut h = 0.0;
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::TurnDone);
        assert_eq!(a.phase, AutoPhase::Walking { from: [0.0, 0.0], distance: 2740.0 });
        assert_eq!(a.before([0.0; 3], &mut h, 0, DT), Before::Walk(Intent { forward: 1, turn: 0, run: false }));
        assert_eq!(a.walk_done([0.0, -2700.0, 0.0]), None);
        // Pushed off the line: the distance from the start counts.
        assert!(a.walk_done([300.0, -2730.0, 0.0]).is_some());
    }
}
