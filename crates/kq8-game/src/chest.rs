//! The chest (`KQChest`, M3 step 3e-4): its keys, its movements, and
//! what each click does.
//!
//! Rules from `docs/behaviour/chests.md` (traced unless marked):
//!
//! - Keys: `invItemTypeName`, `invQuantity` (default 1), `invLocX/Y/Z`,
//!   `invDirX/Y/Z`, and the `Take…` / `Empty…` remarks (six numbers each)
//!   (rule 1). Collision type 0x8021 unless `colliderType` (rule 2).
//! - Movements `closed` and `open`; no `opened`: after `open` the chest
//!   stays on its last frame (rule 3). Played by `crate::movement` (the
//!   phase step and cues of doors).
//! - 1st click: `open` plays; the item appears at `invLoc` (made by the
//!   caller), the chest remembers it and is open, or empty if it holds
//!   nothing (rule 7). 2nd click: the remembered item goes into Connor's
//!   inventory, then the `Take` remark (rule 8). Later clicks: the `Empty`
//!   remark (rule 9). The chest's script is not run on clicks (rule 6).
//!
//! The caller (`crate::active`) makes and moves the item and calls the
//! script (`Init`, `EndLoop <movement>`, `Cue`).

use crate::collide::Placement;
use crate::door::DoorMovement;
use crate::movement::{MovementPlayer, Played};

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

#[derive(Debug, Clone)]
pub struct Chest {
    pub name: String,
    pub state: ChestState,
    pub settings: ChestSettings,
    /// The item object made by the first click, until the second.
    pub remembered: Option<String>,
    /// Its movements (`closed`, `open`).
    pub player: MovementPlayer,
}

impl Chest {
    /// Closed, showing movement `current` at its start (rule 2).
    pub fn new(name: &str, settings: ChestSettings, movements: Vec<DoorMovement>, current: usize, base: Placement) -> Self {
        Chest { name: name.to_string(), state: ChestState::Closed, settings, remembered: None, player: MovementPlayer::new(name, movements, current, base) }
    }

    pub fn movement_name(&self) -> &str {
        self.player.movement_name()
    }

    pub fn play(&mut self, name: &str) -> Result<(), String> {
        self.player.play(name)
    }

    /// One update; a finished play-once movement stays on its last frame
    /// (rule 3), and the caller calls `EndLoop <name>` (rule 6).
    pub fn advance(&mut self, dt: f32) -> Played {
        self.player.advance(dt)
    }

    pub fn frame(&self) -> usize {
        self.player.frame()
    }

    pub fn placement(&self) -> Placement {
        self.player.placement()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A chest like `smlchst3`: `closed` still, `open` over 1 s
    /// (`crate::movement::tests::still_then_open`).
    pub fn test_chest(item: Option<&str>) -> Chest {
        let player = crate::movement::tests::still_then_open("smlchst3");
        let settings = ChestSettings {
            item: item.map(str::to_string),
            quantity: 3,
            item_position: [0.0, 0.0, 301.0],
            item_angles: [0.0, 0.0, -2.5],
            take: None,
            empty: Some(Remark([0, 109, 14, 33, 1, 1])),
        };
        Chest::new("smlchst3", settings, player.movements, 0, player.base)
    }

    #[test]
    fn a_remark_counts_when_its_seq_is_not_0() {
        assert!(Remark([0, 109, 14, 33, 1, 1]).said());
        assert!(!Remark([0, 109, 14, 33, 0, 1]).said());
    }
}
