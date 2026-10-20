//! The fight (M5 5c): characters and Connor hurting each other.
//!
//! Rules from `docs/behaviour/fighting.md` (session A, 2026-10-20; rules 6
//! and 31–32 checked by the user in the original):
//!
//! - Rules 1, 2 (M5 5c-1): every fighter carries five stats: health, hand
//!   damage, ranged damage, protection (capped at 100), experience. A
//!   character from the monster list gets them from its type and line
//!   plus the items it holds (`kq8_data::characters::FightSettings`).
//!
//! Everything about the fight is logged to `fight.log` (and
//! `characters.log`) through [`crate::active::WorldState::fight_note`].

use kq8_data::characters::FightSettings;

use crate::stats::Stats;

/// A character's part in the fight (M5 5c).
#[derive(Debug, Clone)]
pub struct Fight {
    /// What its files say (rule 2).
    pub settings: FightSettings,
    /// Its five stats now: health, hand, ranged, protection, experience.
    pub stats: Stats,
}

impl Fight {
    pub fn new(settings: FightSettings) -> Self {
        let stats = Stats(settings.stats);
        Fight { settings, stats }
    }
}
