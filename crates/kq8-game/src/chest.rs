//! The chest (`KQChest`, M3 step 3e-4): its keys, its movements, and
//! what each click does.
//!
//! Rules from `docs/behaviour/chests.md` (traced unless marked):
//!
//! - Keys: `invItemTypeName`, `invQuantity` (default 1), `invLocX/Y/Z`,
//!   `invDirX/Y/Z`, and the `Take…` / `Empty…` remarks (six numbers each)
//!   (rule 1). Collision type 0x8021 unless `colliderType` (rule 2).
//! - Movements `closed` and `open`; no `opened`: after `open` the chest
//!   stays on its last frame (rule 3). Played as doors' are (the phase
//!   step and cues of `crate::door`).
//! - 1st click: `open` plays; the item appears at `invLoc` (made by the
//!   caller), the chest remembers it and is open, or empty if it holds
//!   nothing (rule 7). 2nd click: the remembered item goes into Connor's
//!   inventory, then the `Take` remark (rule 8). Later clicks: the `Empty`
//!   remark (rule 9). The chest's script is not run on clicks (rule 6).
//!
//! The caller (`crate::active`) makes and moves the item and calls the
//! script (`Init`, `EndLoop <movement>`, `Cue`).

use crate::collide::Placement;
use crate::door::{chain, identity, DoorMovement};
use kq8_data::shape::Transform;

/// Rule 2.
pub const CHEST_TYPE: u32 = 0x8021;

/// The chest's own state (`chests.md` rules 7-9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ChestState {
    Closed,
    Open,
    Empty,
}

impl ChestState {
    pub fn name(self) -> &'static str {
        match self {
            ChestState::Closed => "closed",
            ChestState::Open => "open",
            ChestState::Empty => "empty",
        }
    }
}

/// A remark's six numbers: `Msg`, `Noun`, `Verb`, `Case`, `Seq`, `End`
/// (rule 1). The line itself is M4.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Remark(pub [i64; 6]);

impl Remark {
    /// Rule 8: a remark counts when its `Seq` is not 0.
    pub fn said(&self) -> bool {
        self.0[4] != 0
    }
}

/// The chest's keys (rule 1).
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChestSettings {
    /// `invItemTypeName` as written; `None` = holds nothing.
    pub item: Option<String>,
    pub quantity: i64,
    /// `invLoc` (world) and `invDir`.
    pub item_position: [f32; 3],
    pub item_angles: [f32; 3],
    pub take: Option<Remark>,
    pub empty: Option<Remark>,
}

/// What one update did to a chest.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ChestAdvance {
    pub from: f32,
    pub to: f32,
    pub cues: Vec<String>,
    /// The movement that finished in this update: the caller calls the
    /// script with `EndLoop <name>` (rule 6).
    pub finished: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Chest {
    pub name: String,
    pub state: ChestState,
    pub settings: ChestSettings,
    /// The item object made by the first click, until the second.
    pub remembered: Option<String>,
    pub movements: Vec<DoorMovement>,
    pub current: usize,
    pub phase: f32,
    pub playing: bool,
    /// Started in this update: first moves in the next (as doors).
    pub started_this_update: bool,
    pub base: Placement,
}

impl Chest {
    /// Closed, showing movement `current` at its start (rule 2).
    pub fn new(name: &str, settings: ChestSettings, movements: Vec<DoorMovement>, current: usize, base: Placement) -> Self {
        let current = current.min(movements.len().saturating_sub(1));
        let (phase, playing) = movements.get(current).map_or((0.0, false), |m| (m.start, m.speed != 0.0));
        Chest {
            name: name.to_string(),
            state: ChestState::Closed,
            settings,
            remembered: None,
            movements,
            current,
            phase,
            playing,
            started_this_update: false,
            base,
        }
    }

    pub fn movement(&self) -> Option<&DoorMovement> {
        self.movements.get(self.current)
    }

    pub fn movement_name(&self) -> &str {
        self.movement().map_or("none", |m| m.name.as_str())
    }

    /// Starts movement `name` (any capitals). `Err` if there is none.
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

    /// One update of `dt` seconds. A finished play-once movement stays on
    /// its last frame (rule 3).
    pub fn advance(&mut self, dt: f32) -> ChestAdvance {
        let mut out = ChestAdvance { from: self.phase, to: self.phase, ..Default::default() };
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

    /// Where it stands now: the steps of the shown frame, then its place
    /// (as doors; a chest's steps are zero in the data seen so far).
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

    /// A chest like `smlchst3` (made-up frame count): `closed` still at 0,
    /// `open` 0 -> 1 over 1 s with a sound cue at 0.
    pub fn test_chest(item: Option<&str>) -> Chest {
        let steps = vec![identity(); 16];
        let mv = |name: &str, speed: f32, cues: Vec<crate::door::Cue>| DoorMovement {
            name: name.into(),
            start: 0.0,
            speed,
            repeat: false,
            play_time: 1.0,
            frame_steps: steps.clone(),
            cues,
        };
        let settings = ChestSettings {
            item: item.map(str::to_string),
            quantity: 3,
            item_position: [0.0, 0.0, 301.0],
            item_angles: [0.0, 0.0, -2.5],
            take: None,
            empty: Some(Remark([0, 109, 14, 33, 1, 1])),
        };
        Chest::new(
            "smlchst3",
            settings,
            vec![mv("closed", 0.0, vec![]), mv("open", 1.0, vec![crate::door::Cue { phase: 0.0, action: "playsound chstopen".into() }])],
            0,
            Placement::at([0.0; 3]),
        )
    }

    #[test]
    fn open_plays_once_and_stays_on_its_last_frame() {
        let mut c = test_chest(Some("INVITEM_Crystal"));
        assert_eq!((c.movement_name(), c.playing), ("closed", false));
        c.play("open").unwrap();
        let first = c.advance(1.0 / 15.0);
        assert_eq!(first.to, 0.0, "started in this update: waits");
        let second = c.advance(1.0 / 15.0);
        assert_eq!(second.cues, ["playsound chstopen"]);
        let mut n = 2;
        let finished = loop {
            let a = c.advance(1.0 / 15.0);
            n += 1;
            if let Some(f) = a.finished {
                break f;
            }
        };
        assert_eq!((finished.as_str(), n), ("open", 16));
        assert_eq!((c.movement_name(), c.phase, c.frame()), ("open", 1.0, 15), "no `opened`: the last frame stays");
        assert_eq!(c.advance(1.0 / 15.0).finished, None);
    }

    #[test]
    fn a_remark_counts_when_its_seq_is_not_0() {
        assert!(Remark([0, 109, 14, 33, 1, 1]).said());
        assert!(!Remark([0, 109, 14, 33, 0, 1]).said());
    }
}
