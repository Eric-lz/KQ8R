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
//! - Rules 13, 14, 16–19, 29 (M5 5c-4): at the `Do` cue the hit test:
//!   the target (Connor, for an alarmed character) within reach (1.25 ×
//!   while his back key is held) and in the attacker's front half (less
//!   than 90° off the way it faces). A hit carries the attacker's hand
//!   damage *D* (`Fire` with a Flame Sword). The roll *R* = round(*D* ×
//!   (0.25 + random × 0.75)) on Connor, at least 1; protection *P* 100 or
//!   *D* 0: no loss; else *P'* = round(*P* × (0.75 + random × 0.5)) ≤ 100
//!   and the loss cut(*R* × (100 − *P'*) / 100), at least 1. Below 0 he
//!   dies. He flinches (`react`) when *R* > 0.9 *D*, his movement can be
//!   cut short and his last flinch ended over 2 s ago, else his ouch
//!   sound at most once a second (the viewer decides, it has his
//!   movement: `Request::ConnorHurt`, `Request::ConnorDies`).
//! - Rules 39–41 (M5 5c-5; the input side is a **stand-in** from the
//!   manual, open question 1): with a hand weapon ready a click swings
//!   (`Attack` / `AttackB` 50/50, Shift `Attack2`, from the weapon's list
//!   in `Connor.kq`'s `ShortAnimList`); at his `Do` cue the same test as
//!   rule 13 with his reach and his target (the clicked character); his
//!   hand damage (Shift: × `minDamageFactor`..`maxDamageFactor`, cut);
//!   `Fire` with the Flame Sword. On a character: the roll 75–125 %
//!   (85–125 % below combat level 1.0), its protection, its health.
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

/// The kind of a hit (rules 14, 30, 32): it picks the die movement and
/// the corpse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HitKind {
    Normal,
    Projectile,
    Fire,
    Ice,
}

impl HitKind {
    pub fn name(self) -> &'static str {
        match self {
            HitKind::Normal => "Normal",
            HitKind::Projectile => "Projectile",
            HitKind::Fire => "Fire",
            HitKind::Ice => "Ice",
        }
    }
}

/// The roll's range on Connor: *a* + random × *b* (rule 16).
pub const ROLL_ON_CONNOR: (f32, f32) = (0.25, 0.75);
/// The roll's range on a character, combat level 1.0 or more (rule 16).
pub const ROLL_ON_CHARACTER: (f32, f32) = (0.75, 0.5);
/// The roll's range on a character, combat level below 1.0 (rule 16).
pub const ROLL_ON_CHARACTER_EASY: (f32, f32) = (0.85, 0.4);
/// The protection's roll (rule 16).
pub const PROTECTION_ROLL: (f32, f32) = (0.75, 0.5);
/// The reach against a Connor holding his back key (rule 13).
pub const BACK_KEY_REACH: f32 = 1.25;
/// `normalReactPercent`'s default (rule 23; Connor's too, rule 29).
pub const REACT_PERCENT: f32 = 0.9;
/// No flinch until the last one ended this long ago (rule 23), seconds.
pub const FLINCH_PAUSE: f64 = 2.0;
/// The ouch sound at most this often (rule 24), seconds.
pub const OUCH_EVERY: f64 = 1.0;
/// Connor's ouch sound when `Connor.kq` gives none (rule 24's default).
pub const DEFAULT_OUCH: &str = "sword.aud";

/// Round to the nearest whole number, halves up (the note's "round").
pub fn round_half_up(x: f32) -> i64 {
    (x + 0.5).floor() as i64
}

/// One hit's damage, worked out (rule 16).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Damage {
    /// The hit's damage *D*.
    pub d: i64,
    /// The roll *R* (at least 1).
    pub roll: i64,
    /// The victim's protection *P*, and *P'* if it was rolled.
    pub protection: i64,
    pub rolled: Option<i64>,
    /// Health lost.
    pub loss: i64,
}

impl Damage {
    /// Rule 16: the roll in `range`, then the protection.
    pub fn roll(d: i64, protection: i64, range: (f32, f32), random: &mut dyn FnMut() -> f32) -> Self {
        let roll = round_half_up(d as f32 * (range.0 + random() * range.1)).max(1);
        if protection == crate::fight::PROTECTION_CAP || d == 0 {
            return Damage { d, roll, protection, rolled: None, loss: 0 };
        }
        let rolled = round_half_up(protection as f32 * (PROTECTION_ROLL.0 + random() * PROTECTION_ROLL.1)).min(PROTECTION_CAP);
        // "Cut": toward zero (integer division).
        let loss = (roll * (100 - rolled) / 100).max(1);
        Damage { d, roll, protection, rolled: Some(rolled), loss }
    }

    /// Rule 23: the roll is high enough for a flinch.
    pub fn flinch_roll(&self, percent: f32) -> bool {
        self.roll as f32 > percent * self.d as f32
    }

    /// For the log: D, R, P', loss.
    pub fn describe(&self) -> String {
        let p = match self.rolled {
            Some(r) => format!("P {} -> P' {r}", self.protection),
            None if self.protection == PROTECTION_CAP => "P 100: no loss".into(),
            None => "D 0: no loss".into(),
        };
        format!("D {}, R {}, {p}, loss {}", self.d, self.roll, self.loss)
    }
}

/// Protection's cap (rule 1).
pub const PROTECTION_CAP: i64 = kq8_data::characters::PROTECTION_CAP;

/// Rule 13's hit test, worked out.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct HitTest {
    pub hit: bool,
    /// Straight 3D distance, feet to feet; the reach used (× 1.25 with
    /// his back key).
    pub distance: f32,
    pub reach: f32,
    /// The angle between the way it faces and the line to the target,
    /// degrees.
    pub angle: f32,
}

impl HitTest {
    /// `facing`: the attacker's forward direction (3D, unit); `from`,
    /// `target`: positions at the feet.
    pub fn new(from: [f32; 3], facing: [f32; 3], target: [f32; 3], reach: f32, back_key: bool) -> Self {
        let v: [f32; 3] = std::array::from_fn(|k| target[k] - from[k]);
        let distance = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        let reach = if back_key { BACK_KEY_REACH * reach } else { reach };
        let dot = v[0] * facing[0] + v[1] * facing[1] + v[2] * facing[2];
        let angle = if distance > 0.0 { (dot / distance).clamp(-1.0, 1.0).acos().to_degrees() } else { 0.0 };
        // In front: strictly less than 90 degrees (exactly 90 is behind).
        HitTest { hit: distance <= reach && dot > 0.0, distance, reach, angle }
    }

    /// Why it missed, for the log (the original's own line says "too far
    /// away" for both).
    pub fn why_missed(&self) -> &'static str {
        if self.distance > self.reach { "out of reach" } else { "not in its front half" }
    }
}

/// The way a rotation faces (its −Y, row vectors: −(second row)).
pub fn facing(r: &[[f32; 3]; 3]) -> [f32; 3] {
    [-r[1][0], -r[1][1], -r[1][2]]
}

/// What `Connor.kq` gives the fight (rules 4, 24, 39–41; M5 5c-4).
#[derive(Debug, Clone, PartialEq)]
pub struct ConnorFight {
    pub ouch_sound: String,
    /// The big swing's factor range (`minDamageFactor`, `maxDamageFactor`).
    pub min_damage_factor: f32,
    pub max_damage_factor: f32,
    /// `ShortAnimList`: (weapon name without `INVITEM_`, `.anm` file).
    pub short_lists: Vec<(String, String)>,
}

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
    /// Rule 13: a fighter's attack passed its `Do` cue: the hit test on its
    /// target (Connor, when it is alarmed), then the hit (rule 14).
    /// Ranged characters shoot instead (5c-9: logged only for now).
    pub(crate) fn character_do(&mut self, i: usize) {
        let a = &self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref() else { return };
        let Some(f) = c.fight.as_ref() else { return };
        let movement = format!("{} at phase {:.4}", a.player.movement_name(), a.player.phase);
        if f.ranged() {
            self.fight_note(&name, format!("Do ({movement}): a ranged character shoots here; the bolt is 5c-9"));
            return;
        }
        let alarmed = c.ai.as_ref().is_some_and(|ai| ai.alarmed);
        let Some(target) = self.connor_at.filter(|_| alarmed) else {
            self.fight_note(&name, format!("Do ({movement}): no target (not alarmed): nothing (rule 13)"));
            return;
        };
        let reach = c.ai.as_ref().map_or(0.0, |ai| ai.settings.attack_dist);
        let t = HitTest::new(a.player.base.position, facing(&a.player.base.rotation), target, reach, self.connor_back_key);
        let back = if self.connor_back_key { " (his back key held: 1.25 x)" } else { "" };
        if !t.hit {
            let why = t.why_missed();
            self.fight_note(&name, format!("Do ({movement}): {name} missing Connor, too far away: miss, {why}: distance {:.0}, reach {:.0}{back}, {:.0} deg off its facing (rule 13)", t.distance, t.reach, t.angle));
            return;
        }
        let d = f.stats.0[1];
        let kind = if f.settings.fire { HitKind::Fire } else { HitKind::Normal };
        self.fight_note(&name, format!("Do ({movement}): hits Connor: distance {:.0}, reach {:.0}{back}, {:.0} deg off its facing; hand damage {d}, {} (rules 13, 14)", t.distance, t.reach, t.angle, kind.name()));
        self.hit_connor(&name, d, kind);
    }

    /// Rules 16–18, 29: a hit of damage `d` on Connor from `from`.
    pub(crate) fn hit_connor(&mut self, from: &str, d: i64, kind: HitKind) {
        if self.connor_dying {
            self.fight_note("Connor", format!("hit by {from}: ignored, he is dying (rule 29)"));
            return;
        }
        let Some(mut c) = self.connor_stats else {
            self.fight_note("Connor", format!("hit by {from}: his stats are unknown (no Connor.kq); nothing"));
            return;
        };
        let rng = &mut self.fight_rng;
        let dmg = Damage::roll(d, c.stats.0[3], ROLL_ON_CONNOR, &mut || rng.fraction());
        let before = c.stats.0[0];
        c.stats.0[0] -= dmg.loss;
        // Rule 29: his health is kept at most 10 × level.
        c.stats.0[0] = c.stats.0[0].min(10 * c.level);
        let after = c.stats.0[0];
        self.connor_stats = Some(c);
        // interface.md rule 24: the bars follow his stats.
        self.connor_stats_events += 1;
        self.fight_note("Connor", format!("hit by {from}, {}: {} (roll 25-100 %, rule 16); health {before} -> {after}", kind.name(), dmg.describe()));
        // Rule 29: two one-off tips (logged only; their texts are another study).
        if !self.connor_tips[0] {
            self.connor_tips[0] = true;
            self.fight_note("Connor", "first hit by a character: a one-off tip would come here (rule 29; not traced further)");
        }
        if !self.connor_tips[1] && after * 2 < 10 * c.level {
            self.connor_tips[1] = true;
            self.fight_note("Connor", "health below half his maximum for the first time: a one-off tip would come here (rule 29; not traced further)");
        }
        let at = format!("hit by {from}");
        if after < 0 {
            self.connor_dying = true;
            self.fight_note("Connor", format!("health {after} is below 0: his dying state, kind {} (rules 18, 29); game over is another study", kind.name()));
            self.requests.push(crate::active::Request::ConnorDies { kind, at });
        } else {
            self.requests.push(crate::active::Request::ConnorHurt { roll_over: dmg.flinch_roll(REACT_PERCENT), at });
        }
    }
}

/// Connor's swings in a hand weapon's list (rule 40): `Attack` or
/// `AttackB` for a plain click, `Attack2` with Shift.
pub const SWINGS: [&str; 3] = ["attack", "attackb", "attack2"];

/// "Can be cut short" at `phase` of a movement: its `loop=` line's value,
/// then each `interruptable <n>` cue at or before the phase, in phase
/// order (`anim-scripts.md` rule 7); an ended movement always can.
pub fn cut_short_at(cut_short: bool, cues: &[(f32, String)], phase: f32) -> bool {
    if phase >= 1.0 {
        return true;
    }
    let mut passed: Vec<&(f32, String)> = cues.iter().filter(|(p, a)| *p <= phase && a.split_whitespace().next().is_some_and(|w| w.eq_ignore_ascii_case("interruptable"))).collect();
    passed.sort_by(|a, b| a.0.total_cmp(&b.0));
    passed.last().map_or(cut_short, |(_, a)| a.split_whitespace().nth(1).and_then(|n| n.parse::<f64>().ok()).is_some_and(|n| n != 0.0))
}

/// The Flame Sword's type (rule 14).
pub const FLAME_SWORD: &str = "INVITEM_FlameSword";

impl crate::active::WorldState {
    /// Rule 41 (M5 5c-5): Connor's swing passed its `Do` cue. `from` and
    /// `heading`: his feet and heading; `reach`: his `attackDist`;
    /// `target`: the character clicked for this swing; `big`: Shift.
    pub fn connor_do(&mut self, from: [f32; 3], heading: f32, reach: f32, target: Option<&str>, big: bool, movement: &str) {
        let Some(j) = target.and_then(|t| self.character_index(t)).filter(|&j| self.anims[j].character.as_deref().is_some_and(|c| c.fight.is_some())) else {
            self.fight_note("Connor", format!("Do ({movement}): no target: he hits dead air (rule 39)"));
            return;
        };
        let name = self.anims[j].player.name.clone();
        let facing = [heading.sin(), -heading.cos(), 0.0];
        let t = HitTest::new(from, facing, self.anims[j].player.base.position, reach, false);
        if !t.hit {
            let why = t.why_missed();
            self.fight_note("Connor", format!("Do ({movement}): Connor missing {name}, too far away: miss, {why}: distance {:.0}, reach {:.0}, {:.0} deg off his facing (rule 41)", t.distance, t.reach, t.angle));
            return;
        }
        let Some(hand) = self.connor_stats.map(|c| c.stats.0[1]) else {
            self.fight_note("Connor", format!("Do ({movement}): his stats are unknown; nothing"));
            return;
        };
        let (d, factor) = match (big, self.connor_fight.as_ref()) {
            (true, Some(f)) => {
                let k = f.min_damage_factor + self.fight_rng.fraction() * (f.max_damage_factor - f.min_damage_factor);
                ((hand as f32 * k) as i64, format!(" x {k:.3} (the big swing, {} to {}, cut)", f.min_damage_factor, f.max_damage_factor))
            }
            _ => (hand, String::new()),
        };
        let kind = if self.weapon_ready.as_deref().is_some_and(|w| w.eq_ignore_ascii_case(FLAME_SWORD)) { HitKind::Fire } else { HitKind::Normal };
        self.fight_note("Connor", format!("Do ({movement}): hits {name}: distance {:.0}, reach {:.0}, {:.0} deg off his facing; hand damage {hand}{factor} = {d}, {} (rule 41)", t.distance, t.reach, t.angle, kind.name()));
        self.hit_character(j, "Connor", d, kind);
    }

    /// Rules 16–18 on a character (M5 5c-5: the damage; its reaction is
    /// 5c-6, dying 5c-7).
    pub(crate) fn hit_character(&mut self, j: usize, from: &str, d: i64, kind: HitKind) {
        let range = if self.combat_level < 1.0 { ROLL_ON_CHARACTER_EASY } else { ROLL_ON_CHARACTER };
        let a = &mut self.anims[j];
        let name = a.player.name.clone();
        let Some(f) = a.character.as_deref_mut().and_then(|c| c.fight.as_mut()) else { return };
        let rng = &mut self.fight_rng;
        let dmg = Damage::roll(d, f.stats.0[3], range, &mut || rng.fraction());
        let before = f.stats.0[0];
        f.stats.0[0] -= dmg.loss;
        let after = f.stats.0[0];
        let dies = if after < 0 { "; below 0: it would die (dying is 5c-7)" } else { "" };
        let line = format!("hit by {from}, {}: {} (roll {:.0}-125 %, rule 16); health {before} -> {after}{dies}", kind.name(), dmg.describe(), range.0 * 100.0);
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

    /// Every roll from `n` evenly spread random numbers in [0, 1).
    fn all_rolls(d: i64, protection: i64, range: (f32, f32), n: usize) -> Vec<Damage> {
        let mut out = Vec::new();
        for i in 0..n {
            for j in 0..n {
                let mut k = 0;
                let r = [i as f32 / n as f32, j as f32 / n as f32];
                out.push(Damage::roll(d, protection, range, &mut || {
                    k += 1;
                    r[k - 1]
                }));
            }
        }
        out
    }

    #[test]
    fn the_worked_examples_of_rule_19() {
        // Connor's fists (10) on goblin22 (protection 0): R 8 to 12, all lost.
        let r = all_rolls(10, 0, ROLL_ON_CHARACTER, 40);
        assert_eq!((r.iter().map(|d| d.loss).min(), r.iter().map(|d| d.loss).max()), (Some(8), Some(12)));
        // goblin22 (2) on Connor (0): 1 or 2.
        let r = all_rolls(2, 0, ROLL_ON_CONNOR, 40);
        assert_eq!((r.iter().map(|d| d.loss).min(), r.iter().map(|d| d.loss).max()), (Some(1), Some(2)));
        // A graveyard zombie (15) on Connor: 4 to 15.
        let r = all_rolls(15, 0, ROLL_ON_CONNOR, 40);
        assert_eq!((r.iter().map(|d| d.loss).min(), r.iter().map(|d| d.loss).max()), (Some(4), Some(15)));
        // Protection 100 or damage 0: nothing lost; otherwise at least 1.
        assert!(all_rolls(50, 100, ROLL_ON_CONNOR, 10).iter().all(|d| d.loss == 0));
        assert!(all_rolls(0, 5, ROLL_ON_CONNOR, 10).iter().all(|d| d.loss == 0 && d.roll == 1));
        assert!(all_rolls(1, 99, ROLL_ON_CONNOR, 10).iter().all(|d| d.loss == 1));
        // The flinch with D 10 on a character: R 10, 11 or 12 (3 in 5).
        let r = all_rolls(10, 0, ROLL_ON_CHARACTER, 100);
        let share = r.iter().filter(|d| d.flinch_roll(REACT_PERCENT)).count() as f32 / r.len() as f32;
        assert!((share - 0.6).abs() < 0.02, "{share}");
        assert_eq!(round_half_up(2.5), 3);
        assert_eq!(round_half_up(-0.5), 0);
    }

    #[test]
    fn the_hit_test_needs_reach_and_the_front_half() {
        // Facing −y (heading 0); target 1,400 in front, reach 1,500.
        let f = [0.0, -1.0, 0.0];
        assert!(HitTest::new([0.0; 3], f, [0.0, -1400.0, 0.0], 1500.0, false).hit);
        // 1,800 away: out of reach, but within 1.25 × with his back key.
        assert!(!HitTest::new([0.0; 3], f, [0.0, -1800.0, 0.0], 1500.0, false).hit);
        assert!(HitTest::new([0.0; 3], f, [0.0, -1800.0, 0.0], 1500.0, true).hit);
        // Exactly 90 degrees counts as behind; 89 is in front.
        let side = HitTest::new([0.0; 3], f, [1000.0, 0.0, 0.0], 1500.0, false);
        assert!(!side.hit && side.why_missed() == "not in its front half");
        assert!(HitTest::new([0.0; 3], f, [1000.0, -20.0, 0.0], 1500.0, false).hit);
        // Behind it.
        assert!(!HitTest::new([0.0; 3], f, [0.0, 500.0, 0.0], 1500.0, false).hit);
    }

    #[test]
    fn interruptable_cues_decide_cut_short_by_phase() {
        // confists.anm's attack: `0 0` in its loop= line, on at 0, off at 0.111.
        let cues = vec![(0.111111, "interruptable 0".to_string()), (0.0, "interruptable 1".to_string()), (0.444444, "do".to_string())];
        assert!(cut_short_at(false, &cues, 0.0));
        assert!(cut_short_at(false, &cues, 0.1));
        assert!(!cut_short_at(false, &cues, 0.2));
        assert!(cut_short_at(false, &cues, 1.0), "ended");
        assert!(cut_short_at(true, &[], 0.5) && !cut_short_at(false, &[], 0.5));
    }

    #[test]
    fn the_first_attack_is_due_at_once_and_attack_ends_count() {
        let f = Fight::new(settings([15, 2, 2, 0, 3], 1.0));
        assert!(f.due(0.0), "rule 6: the attack time starts at 0");
        assert!(is_attack("attack") && is_attack("ATTACK2") && is_attack("idle:attack"));
        assert!(!is_attack("attackb") && !is_attack("react"));
    }
}
