//! The fight (M5 5c): characters and Connor hurting each other.
//!
//! Rules from `docs/behaviour/fighting.md` (session A, 2026-10-20; rules 6
//! and 31–32 checked by the user in the original):
//!
//! - Rules 1, 2 (M5 5c-1): every fighter carries five stats: health, hand
//!   damage, ranged damage, protection (capped at 100), experience. A
//!   character from the monster list gets them from its type and line
//!   plus the items it holds (`kq8_data::characters::FightSettings`).
//! - Rules 6–12 (M5 5c-3): the **attack time** starts at 0 (the first
//!   attack is due at once) and is never reset by an alarm. Standing (in
//!   Wait, alarmed): it can see Connor, he is closer than its reach (1.25
//!   × for a ranged character) and the attack is due → `Attack`. Moving:
//!   the same with 1 × reach, whatever Connor does. Only when the running
//!   movement can be cut short or has ended. While the attack plays the
//!   attack time is "never" and the chase timer 5 s from its start; its
//!   end (any movement named `Attack` or `Attack2` ending, an idle fidget
//!   too) makes it now + `attackDelay` and the chase timer 5 s; cut short
//!   by something else, the "never" is put right at the next entry into
//!   Wait or Move (or a hit, 5c-6). The attack models have no step: it
//!   stands still while it keeps turning; a moving one's stuck check is
//!   off until its `walk` / `run` plays again.
//! - Rule 11b, **not reproduced** (the user's decision, "Do not
//!   reproduce"): while a moving character's `Attack` plays, the moving
//!   state's stop rule is not applied; it stays in Move until the swing
//!   ends.
//!
//! Everything about the fight is logged to `fight.log` (and
//! `characters.log`) through [`crate::active::WorldState::fight_note`].

use kq8_data::characters::FightSettings;

use crate::stats::Stats;

/// The movement a character asks for to attack (rule 8; any capitals).
pub const ATTACK: &str = "Attack";
/// The attack time while an attack plays (rule 9).
pub const NEVER: f64 = f64::INFINITY;
/// The chase timer at an attack's start and end (rules 9, 10), seconds.
pub const ATTACK_CHASE: f64 = 5.0;
/// A ranged character attacks from standing within this × its reach
/// (rules 7, 35).
pub const RANGED_STANDING: f32 = 1.25;

/// A character's part in the fight (M5 5c).
#[derive(Debug, Clone)]
pub struct Fight {
    /// What its files say (rule 2).
    pub settings: FightSettings,
    /// Its five stats now: health, hand, ranged, protection, experience.
    pub stats: Stats,
    /// When its next attack is due (rule 6): 0 at first, [`NEVER`] while
    /// an attack plays.
    pub attack_time: f64,
}

impl Fight {
    pub fn new(settings: FightSettings) -> Self {
        let stats = Stats(settings.stats);
        Fight { settings, stats, attack_time: 0.0 }
    }

    /// It holds a `LongWeapon` (rule 34).
    pub fn ranged(&self) -> bool {
        self.settings.ranged_weapon.is_some()
    }

    /// Its attack is due now (rule 7).
    pub fn due(&self, now: f64) -> bool {
        now + 1e-6 >= self.attack_time
    }

    /// The attack time as text for the log.
    pub fn attack_time_text(&self) -> String {
        if self.attack_time == NEVER { "never".into() } else { format!("{:.2}", self.attack_time) }
    }
}

/// A movement whose end sets the attack time (rule 10: `Attack` or
/// `Attack2`, any capitals; an idle list's fidget counts too, `Edge cases`).
pub fn is_attack(name: &str) -> bool {
    let n = name.strip_prefix(crate::character_ai::IDLE_PREFIX).unwrap_or(name);
    n.eq_ignore_ascii_case("attack") || n.eq_ignore_ascii_case("attack2")
}

impl crate::active::WorldState {
    /// Rule 13: a fighter's attack passed its `Do` cue. M5 5c-3: logged
    /// only; the hit test is 5c-4.
    pub(crate) fn character_do(&mut self, i: usize) {
        let a = &self.anims[i];
        let name = a.player.name.clone();
        let d = self.connor_at.map(|c| {
            let p = a.player.base.position;
            ((p[0] - c[0]).powi(2) + (p[1] - c[1]).powi(2) + (p[2] - c[2]).powi(2)).sqrt()
        });
        let line = format!(
            "Do cue of '{}' at phase {:.4}: Connor {} away; the hit test comes in 5c-4",
            a.player.movement_name(),
            a.player.phase,
            d.map_or("not known".to_string(), |d| format!("{d:.0}"))
        );
        self.fight_note(&name, line);
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Fight settings with these stats and attack delay, nothing else.
    pub(crate) fn settings(stats: [i64; 5], attack_delay: f32) -> FightSettings {
        FightSettings {
            stats,
            attack_delay,
            ranged_weapon: None,
            projectile: None,
            fire: false,
            drops: Vec::new(),
            attack_more: None,
            dead_dts: None,
            ided_kqs: None,
            projd_kqs: None,
            ouch_sound: None,
            normal_react_percent: None,
            special_react_anim: None,
            special_react_percent: None,
            likely_to_flee: None,
            must_flee: false,
            from: "test".into(),
        }
    }

    #[test]
    fn the_first_attack_is_due_at_once_and_attack_ends_count() {
        let f = Fight::new(settings([15, 2, 2, 0, 3], 1.0));
        assert!(f.due(0.0), "rule 6: the attack time starts at 0");
        assert!(is_attack("attack") && is_attack("ATTACK2") && is_attack("idle:attack"));
        assert!(!is_attack("attackb") && !is_attack("react"));
    }
}
