//! Playing an object's named movements (M3 steps 3e-4, 3e-5): which one
//! is shown, its phase, starting one by name, one update's step, the
//! frame shown and where the object stands.
//!
//! Used by chests and scripted animated objects. Rules as for doors
//! (`docs/behaviour/doors.md` rules 6 and 9, `crate::door`): a movement
//! started in an update first moves in the next; a play-once movement
//! stops at its end and stays on its last frame (`chests.md` rule 3);
//! the object stands where the shown frame's steps put it (as doors).

use kq8_data::shape::Transform;

use crate::collide::Placement;
use crate::door::{chain, identity, DoorMovement};

/// What one update did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Played {
    pub from: f32,
    pub to: f32,
    pub cues: Vec<String>,
    /// The play-once movement that reached its end in this update.
    pub finished: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MovementPlayer {
    /// The object's name (for messages).
    pub name: String,
    pub movements: Vec<DoorMovement>,
    /// Index into `movements` of the one shown.
    pub current: usize,
    pub phase: f32,
    /// The current movement still moves.
    pub playing: bool,
    /// Started in this update: first moves in the next.
    pub started_this_update: bool,
    /// The object's place in the world.
    pub base: Placement,
}

impl MovementPlayer {
    /// Showing movement `current` at its start.
    pub fn new(name: &str, movements: Vec<DoorMovement>, current: usize, base: Placement) -> Self {
        let current = current.min(movements.len().saturating_sub(1));
        let (phase, playing) = movements.get(current).map_or((0.0, false), |m| (m.start, m.speed != 0.0));
        MovementPlayer { name: name.to_string(), movements, current, phase, playing, started_this_update: false, base }
    }

    pub fn movement(&self) -> Option<&DoorMovement> {
        self.movements.get(self.current)
    }

    pub fn movement_name(&self) -> &str {
        self.movement().map_or("none", |m| m.name.as_str())
    }

    /// Starts movement `name` (any capitals) at its start phase. `Err` if
    /// there is none.
    pub fn play(&mut self, name: &str) -> Result<(), String> {
        let i = self
            .movements
            .iter()
            .position(|m| m.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| format!("{} has no movement '{name}'", self.name))?;
        self.current = i;
        self.phase = self.movements[i].start;
        self.playing = self.movements[i].speed != 0.0;
        self.started_this_update = true;
        Ok(())
    }

    /// One update of `dt` seconds.
    pub fn advance(&mut self, dt: f32) -> Played {
        let mut out = Played { from: self.phase, to: self.phase, ..Default::default() };
        if std::mem::take(&mut self.started_this_update) || !self.playing {
            return out;
        }
        let Some(m) = self.movements.get(self.current) else { return out };
        let step = m.step(self.phase, dt);
        self.phase = step.to;
        out.to = step.to;
        out.cues = step.cues;
        if step.done {
            self.playing = false;
            out.finished = Some(m.name.clone());
        }
        out
    }

    pub fn frame(&self) -> usize {
        self.movement().map_or(0, |m| m.frame_at(self.phase))
    }

    /// Where it stands now: the steps of the shown frame, then its place.
    pub fn placement(&self) -> Placement {
        let t = self.movement().map_or_else(identity, |m| m.turn_at_frame(self.frame()));
        let base = Transform { flags: 0, m: self.base.rotation, p: self.base.position };
        let w = chain(&base, &t);
        Placement { rotation: w.m, position: w.p }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::door::Cue;

    /// `stop` still at 0, then `open` 0 -> 1 over 1 s with a cue at 0, 16
    /// frames (made-up numbers in the style of the chests and baskets).
    pub fn still_then_open(name: &str) -> MovementPlayer {
        let steps = vec![identity(); 16];
        let mv = |n: &str, speed: f32, cues: Vec<Cue>| DoorMovement {
            name: n.into(),
            start: 0.0,
            speed,
            repeat: false,
            play_time: 1.0,
            frame_steps: steps.clone(),
            cues,
        };
        MovementPlayer::new(
            name,
            vec![mv("closed", 0.0, vec![]), mv("open", 1.0, vec![Cue { phase: 0.0, action: "playsound chstopen".into() }])],
            0,
            Placement::at([0.0; 3]),
        )
    }

    #[test]
    fn open_plays_once_and_stays_on_its_last_frame() {
        let mut p = still_then_open("smlchst3");
        assert_eq!((p.movement_name(), p.playing), ("closed", false));
        assert!(p.play("OPEN").is_ok());
        assert!(p.play("opened").is_err());
        let first = p.advance(1.0 / 15.0);
        assert_eq!(first.to, 0.0, "started in this update: waits");
        assert_eq!(p.advance(1.0 / 15.0).cues, ["playsound chstopen"]);
        let mut n = 2;
        let finished = loop {
            n += 1;
            if let Some(f) = p.advance(1.0 / 15.0).finished {
                break f;
            }
        };
        assert_eq!((finished.as_str(), n), ("open", 16));
        assert_eq!((p.movement_name(), p.phase, p.frame()), ("open", 1.0, 15), "the last frame stays");
        assert_eq!(p.advance(1.0 / 15.0).finished, None);
    }
}
