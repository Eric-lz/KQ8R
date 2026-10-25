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

/// The chase timer after the alarm with "run" (rule 26), seconds.
pub const RUN_ALARM_CHASE: f64 = 10.0;
/// `likelyToFlee`'s default; above this a hit makes it flee (rule 22.5).
pub const LIKELY_TO_FLEE: f32 = 0.5;

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
    /// `LongAnimList`, the same for the ranged weapons
    /// (`ranged-weapons.md` rule 10; M5 5f-4).
    pub long_lists: Vec<(String, String)>,
    /// `shootOffsets`: (item type, offset in his own space) (rule 17).
    pub shoot_offsets: Vec<(String, [f32; 3])>,
    /// `projectileFile` and `shootOffsetX/Y/Z`: start values the first
    /// ranged item replaces (rule 21).
    pub start_projectile: Option<String>,
    pub start_offset: [f32; 3],
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
    /// The last hit's kind (rule 22.1; picks the die movement, rule 30).
    pub last_kind: Option<HitKind>,
    /// It must flee (rules 22.5, 37): `mustFlee`, or hit with
    /// `likelyToFlee` above 0.5.
    pub must_flee: bool,
    /// When its last flinch ended, and its last ouch sound (rules 23, 24).
    pub flinch_end: f64,
    pub last_ouch: f64,
    /// Its flinch movement playing now (`react` or its special one).
    pub flinching: Option<String>,
    /// Dying (rule 30): its die movement (`None` with `Die now` or an
    /// unknown kind: nothing plays).
    pub dying: Option<Option<String>>,
    /// It was loaded while `applyCombatLevel` was on (rule 3): its
    /// health and damage get the combat level's factor when it enters the
    /// world (M5 5c-10), set by the loader.
    pub combat_scaled: bool,
    /// Where its bolts start, in its own space: the bolt's model part found
    /// at a shot, then kept (`ranged-weapons.md` rule 24; M5 5f-2).
    pub shoot_offset: Option<[f32; 3]>,
    /// The drop entry named like a part of its pose at the dying start
    /// (its weapon), kept for the die movement's end (rules 31, 31a; C-2):
    /// the type as written and the quantity.
    pub kept_drop: Option<(String, i64)>,
}

/// A weapon laid where a death left it (rule 31a.2; C-2), waiting for
/// its line test, which needs the collision scene.
#[derive(Debug, Clone, PartialEq)]
pub struct LaidCheck {
    pub item: String,
    pub kind: String,
    pub quantity: i64,
    /// The dying character: its feet (where a blocked item is thrown
    /// from) and its name (never tested).
    pub feet: [f32; 3],
    pub owner: String,
}

/// Rule 31a.1: with no part of that name in the last frame, the item is
/// put at the character's place and turn, this much higher.
pub const LAID_FALLBACK_UP: f32 = 750.0;
/// Rule 31a.2: the line test runs from this far above the item to this
/// far above the character's feet.
pub const LAID_LINE_UP: f32 = 1000.0;
/// Rule 31a.2: what blocks the line: the terrain and "any solid object
/// other than the dying character". Which collision types count as
/// solid was not given: **stand-in**, every type (asked: QUESTIONS
/// Q170).
pub const LAID_LINE_MASK: u32 = u32::MAX;

/// A drop entry as compared with the model's part names (rule 31): the
/// quotes and an `INVITEM_` prefix taken off.
pub fn drop_part_name(entry: &str) -> &str {
    let e = entry.trim().trim_matches('"');
    match e.get(..8) {
        Some(p) if p.eq_ignore_ascii_case("INVITEM_") => &e[8..],
        _ => e,
    }
}

/// `setCombatLevel 1|2|3` (rule 3): the factor.
pub fn combat_factor(level: i64) -> Option<f32> {
    match level {
        1 => Some(0.66),
        2 => Some(1.0),
        3 => Some(1.5),
        _ => None,
    }
}

/// A bolt in the air (`fighting.md` rule 36, M5 5c-9; its flight as
/// `ranged-weapons.md` rules 26–31, M5 5f-1).
#[derive(Debug, Clone, PartialEq)]
pub struct Bolt {
    /// `bolt<n>`, for the log and the viewer.
    pub name: String,
    pub shooter: String,
    pub shape: Option<String>,
    pub position: [f32; 3],
    /// Rule 27: the way it points, set once when it enters the world
    /// (unit length), and that as heading (our usual: 0 = −y) and pitch.
    pub direction: [f32; 3],
    pub heading: f32,
    pub pitch: f32,
    /// `velocity`, units per second.
    pub speed: f32,
    /// How far it has gone, and how far it may go (`maxDistance`).
    pub travelled: f32,
    pub max: f32,
    /// The shooter's ranged damage at the shot (rule 22).
    pub damage: i64,
    /// `Projectile`, or `Fire` / `Ice` by `weaponType` (rule 30).
    pub kind: HitKind,
    /// `colliderMask` (rule 29).
    pub mask: u32,
    /// Its own shape: its `BOUNDS.KQ` box as a ball stack (rule 29;
    /// `connor-collision.md` rule 18; Q140 a); `None` without a box: a
    /// point.
    pub balls: Option<crate::collide::BallStack>,
    /// Updates flown, for the log.
    pub updates: u32,
    /// Azriel's Hammer (a `KQHammer` file): it flies back instead of
    /// being removed (`azriel-hammer.md` rules 7–9; M6 6b-2).
    pub hammer: Option<HammerFlight>,
}

/// Azriel's Hammer in flight (`azriel-hammer.md` rules 6–9).
#[derive(Debug, Clone, PartialEq)]
pub struct HammerFlight {
    /// On its way back (rule 9): only Connor stops it.
    pub back: bool,
    /// Its file's `modifier`, `minDamage`, `maxDamage` (rule 6).
    pub modifier: i64,
    pub min_damage: i64,
    pub max_damage: i64,
}

/// `azriel-hammer.md` rule 8: the hammer's hit: `modifier` + the ranged
/// damage, capped at `maxDamage`, then raised to at least `minDamage`
/// (in that order: "Do not reproduce", kept).
pub fn hammer_damage(modifier: i64, ranged: i64, min: i64, max: i64) -> i64 {
    (modifier + ranged).min(max).max(min)
}

/// Rule 8: the Dead City objects whose touch runs `lever1.cs` with
/// `lever1 ConnorAction` instead of a hit (the lever and its tumbler).
pub const HAMMER_LEVER_OBJECTS: [&str; 2] = ["lever1", "tmblrcyl"];
/// The object whose script that is.
pub const HAMMER_LEVER: &str = "lever1";

impl Bolt {
    /// Rule 27: its model turned to its direction (its model's front is
    /// +y, the game's heading 0 faces −y: turned half round), pitch first.
    pub fn rotation(&self) -> [[f32; 3]; 3] {
        kq8_data::objects::rotation([self.pitch, 0.0, self.heading + std::f32::consts::PI])
    }
}

/// Rule 30: the hit's kind by the bolt file's `weaponType`.
pub fn bolt_kind(weapon_type: Option<i64>) -> HitKind {
    match weapon_type {
        Some(2) => HitKind::Fire,
        Some(3) => HitKind::Ice,
        _ => HitKind::Projectile,
    }
}

/// Rule 27: heading (0 = −y, as everywhere in ours) and pitch (asin of the
/// height difference over the straight distance) from `from` to `to`, and
/// the unit direction. `None` when the two points are the same.
pub fn aim_direction(from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], f32, f32)> {
    let d: [f32; 3] = std::array::from_fn(|k| to[k] - from[k]);
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
    if len < 1e-3 {
        return None;
    }
    let pitch = (d[2] / len).clamp(-1.0, 1.0).asin();
    let heading = d[0].atan2(-d[1]);
    let flat = pitch.cos();
    Some(([flat * heading.sin(), -flat * heading.cos(), pitch.sin()], heading, pitch))
}

/// Rule 25: a character aims at its target raised by this share of the
/// top of the target's collision box (Connor: his `BOUNDS.KQ` box, top
/// 1,300 → 975; the note says 1,600 → 1,200: Q140 b).
pub const AIM_SHARE: f32 = 0.75;
/// Rule 30: a victim whose collision type has this bit gets 0 damage and
/// `ProjImmune`.
pub const IMMUNE_TO_PROJECTILES: u32 = 0x4000;

/// Rule 36: where to aim so that a bolt of speed `speed` from `from`
/// meets a target at `at` moving at `v`: the first time t > 0 with
/// |at + v t − from| = speed × t; straight at it if there is none.
pub fn lead(from: [f32; 3], at: [f32; 3], v: [f32; 3], speed: f32) -> ([f32; 3], Option<f32>) {
    let d: [f32; 3] = std::array::from_fn(|k| at[k] - from[k]);
    let dot = |a: [f32; 3], b: [f32; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
    let (qa, qb, qc) = (dot(v, v) - speed * speed, 2.0 * dot(v, d), dot(d, d));
    let t = if qa.abs() < 1e-6 {
        (qb.abs() > 1e-6).then(|| -qc / qb).filter(|t| *t > 0.0)
    } else {
        let disc = qb * qb - 4.0 * qa * qc;
        (disc >= 0.0).then(|| {
            let s = disc.sqrt();
            let (t1, t2) = ((-qb - s) / (2.0 * qa), (-qb + s) / (2.0 * qa));
            [t1, t2].into_iter().filter(|t| *t > 0.0).fold(f32::MAX, f32::min)
        }).filter(|t| *t < f32::MAX)
    };
    match t {
        Some(t) => (std::array::from_fn(|k| at[k] + v[k] * t), Some(t)),
        None => (at, None),
    }
}

/// A body left where a character died (rule 32; M5 5c-7), drawn by the
/// viewer.
#[derive(Debug, Clone, PartialEq)]
pub struct Corpse {
    /// The character's name (the body is `<name>_corpse` in the logs).
    pub name: String,
    /// Its shape (`deadDts`, `projdKqs`, `idedKqs`).
    pub shape: String,
    pub position: [f32; 3],
    pub rotation: [[f32; 3]; 3],
    /// Made by a script's `loadObject` (`anim-scripts.md` rule 34): drawn
    /// under its own name (so `deleteObject` finds it), not `<name>_corpse`.
    pub loaded: bool,
}

impl Corpse {
    /// The name its drawn object goes by.
    pub fn drawn_name(&self) -> String {
        if self.loaded { self.name.clone() } else { format!("{}_corpse", self.name) }
    }
}

/// Types whose death gives no first-kill tip (rule 33, with the Do
/// instead: `chicken2`, not the original's typo `checken2`).
pub const NO_KILL_TIP: [&str; 3] = ["bat", "chicken", "chicken2"];
/// The first-kill tip comes this long after the death (rule 33), seconds.
pub const KILL_TIP_DELAY: f64 = 1.5;

impl Fight {
    pub fn new(settings: FightSettings) -> Self {
        let stats = Stats(settings.stats);
        let must_flee = settings.must_flee;
        Fight { settings, stats, attack_time: 0.0, last_kind: None, must_flee, flinch_end: f64::NEG_INFINITY, last_ouch: f64::NEG_INFINITY, flinching: None, dying: None, combat_scaled: false, shoot_offset: None, kept_drop: None }
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
            self.shoot(i, &movement);
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
        if self.god_mode {
            self.fight_note("Connor", format!("hit by {from}, {}, damage {d}: nothing, god mode is on (test switch, ours)", kind.name()));
            return;
        }
        if self.connor_dying {
            self.fight_note("Connor", format!("hit by {from}: ignored, he is dying (rule 29)"));
            return;
        }
        if self.connor_roping {
            // ropeandhook.md rule 27 (M6 6a-6): no damage, sound or reaction.
            self.fight_note("Connor", format!("hit by {from}, {}, damage {d}: ignored completely, he is in the rope state (ropeandhook.md rule 27)", kind.name()));
            self.rope_note(format!("hit by {from} ({} damage): ignored (rule 27)", d));
            return;
        }
        let Some(mut c) = self.connor_stats else {
            self.fight_note("Connor", format!("hit by {from}: his stats are unknown (no Connor.kq); nothing"));
            return;
        };
        // potions.md rule 19: in the last temple the shield gives half.
        let shield = self.potions.iter().find(|r| r.effect == crate::potions::Effect::Shield).map(|r| r.added);
        let (protection, temple) = match shield {
            Some(added) if self.connor_in_temple4 => {
                let p = crate::potions::temple4_protection(c.stats.0[3], added);
                (p, format!("; in the last temple with the shield: protection {} used as {p} (potions.md rule 19)", c.stats.0[3]))
            }
            _ => (c.stats.0[3], String::new()),
        };
        let rng = &mut self.fight_rng;
        let dmg = Damage::roll(d, protection, ROLL_ON_CONNOR, &mut || rng.fraction());
        let before = c.stats.0[0];
        c.stats.0[0] -= dmg.loss;
        // Rule 29: his health is kept at most 10 × level.
        c.stats.0[0] = c.stats.0[0].min(10 * c.level);
        let after = c.stats.0[0];
        self.connor_stats = Some(c);
        // interface.md rule 24: the bars follow his stats.
        self.connor_stats_events += 1;
        self.fight_note("Connor", format!("hit by {from}, {}: {} (roll 25-100 %, rule 16); health {before} -> {after}{temple}", kind.name(), dmg.describe()));
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
            if let Some(at) = target.and_then(|t| self.object_at(t)) {
                return self.connor_hits_object(target.unwrap_or_default(), at, from, heading, reach, big, movement);
            }
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

    /// Where an object with a script stands (a box object, a door or an
    /// animated one), for Connor's hit on it (M5 5c fix 5; doors:
    /// `doors.md` rules 18-20, `ranged-weapons.md` rule 32).
    fn object_at(&self, name: &str) -> Option<[f32; 3]> {
        if let Some(c) = self.clickables.iter().find(|c| !c.gone && c.script.is_some() && c.name.eq_ignore_ascii_case(name)) {
            return Some(c.placement.position);
        }
        if let Some(d) = self.doors.iter().find(|d| !d.gone && d.script.is_some() && d.door.name.eq_ignore_ascii_case(name)) {
            return Some(d.door.player.placement().position);
        }
        self.objects.iter().find(|o| !o.gone && o.script.is_some() && o.name().eq_ignore_ascii_case(name)).map(|o| o.player.placement().position)
    }

    /// Connor's swing on an object (M5 5c fix 5; the barrels): rule 13's
    /// test with his reach, then `doors.md` rules 19-20 on its stats
    /// (`stats=`, all 0 without one, `anim-scripts.md` rule 19): damage
    /// rolled as on a character (rule 16's 75-125 % and its protection,
    /// the 4th stat; **stand-in**, doors.md says only "random, at least
    /// 1"), taken off its health; below 0 its script gets `Die <kind>`
    /// (every hit while below 0 again).
    #[allow(clippy::too_many_arguments)]
    fn connor_hits_object(&mut self, name: &str, at: [f32; 3], from: [f32; 3], heading: f32, reach: f32, big: bool, movement: &str) {
        let facing = [heading.sin(), -heading.cos(), 0.0];
        let t = HitTest::new(from, facing, at, reach, false);
        if !t.hit {
            let why = t.why_missed();
            self.fight_note("Connor", format!("Do ({movement}): Connor missing {name}, too far away: miss, {why}: distance {:.0}, reach {:.0}, {:.0} deg off his facing (rule 41)", t.distance, t.reach, t.angle));
            return;
        }
        let Some(hand) = self.connor_stats.map(|c| c.stats.0[1]) else {
            self.fight_note("Connor", format!("Do ({movement}): his stats are unknown; nothing"));
            return;
        };
        let d = match (big, self.connor_fight.as_ref()) {
            (true, Some(f)) => (hand as f32 * (f.min_damage_factor + self.fight_rng.fraction() * (f.max_damage_factor - f.min_damage_factor))) as i64,
            _ => hand,
        };
        let kind = if self.weapon_ready.as_deref().is_some_and(|w| w.eq_ignore_ascii_case(FLAME_SWORD)) { HitKind::Fire } else { HitKind::Normal };
        let line = self.object_takes(name, d, kind);
        self.fight_note("Connor", format!("Do ({movement}): hits object {name}: distance {:.0}, reach {:.0}, {:.0} deg off his facing; {line}", t.distance, t.reach, t.angle));
    }

    /// A hit of damage `d` on object `name` (M5 5c fix 5, **stand-in**,
    /// Q139 e; a bolt's hit too, M5 5f-1): `doors.md` rules 19-20 on its
    /// stats (`stats=`, all 0 without one), the damage rolled as on a
    /// character (75-125 %, its protection the 4th stat); below 0 its
    /// script gets `Die <kind>`. Returns the log text.
    fn object_takes(&mut self, name: &str, d: i64, kind: HitKind) -> String {
        let key = name.to_ascii_lowercase();
        let known = self.stats.contains_key(&key);
        let s = *self.stats.entry(key.clone()).or_default();
        let rng = &mut self.fight_rng;
        let dmg = Damage::roll(d, s.0[3].min(PROTECTION_CAP), ROLL_ON_CHARACTER, &mut || rng.fraction());
        let mut after = s;
        after.0[0] -= dmg.loss;
        self.stats.insert(key, after);
        let start = if known { "" } else { " (no stats= line: all 0)" };
        if after.0[0] < 0 {
            self.fight_note(name, format!("health below 0: its script gets Die {} (doors.md rule 20)", kind.name()));
            self.die_calls.push((name.to_string(), kind.name()));
        }
        format!("{} (75-125 %, stand-in); health {} -> {}{start} (doors.md rule 19)", dmg.describe(), s.0[0], after.0[0])
    }

    /// Rules 16–18, 20–28 on a character (M5 5c-6), in rule 22's order:
    /// the kind; active; chase timer 5 s; the hit count; must flee; the
    /// damage (dying: 5c-7); the attack time put right; the flinch or the
    /// ouch sound; where Connor is; the alarm with "run"; then rule 28 if
    /// it is still not alarmed. No knock-back (rule 20).
    pub(crate) fn hit_character(&mut self, j: usize, from: &str, d: i64, kind: HitKind) {
        let now = self.now();
        let range = if self.combat_level < 1.0 { ROLL_ON_CHARACTER_EASY } else { ROLL_ON_CHARACTER };
        let name = self.anims[j].player.name.clone();
        if self.anims[j].character.as_deref().is_none_or(|c| c.fight.is_none()) {
            return;
        }
        let mut notes = Vec::new();
        // 22.2: active if it was not (if there is room).
        if !self.anims[j].character.as_deref().and_then(|c| c.ai.as_ref()).is_some_and(|ai| ai.active) {
            self.join_active_pub(j, now);
        }
        let connor = self.connor_at;
        let a = &mut self.anims[j];
        let c = a.character.as_deref_mut().unwrap();
        let had_target = c.ai.as_ref().is_some_and(|ai| ai.alarmed);
        let standing = c.state == crate::character::State::Wait;
        if let Some(ai) = c.ai.as_mut() {
            // 22.3, 22.4.
            ai.chase_until = now + ATTACK_CHASE;
            ai.hit_count += 1;
            notes.push(format!("chase timer 5 s, hit count {} (rule 22)", ai.hit_count));
        }
        let f = c.fight.as_mut().unwrap();
        // 22.1, 22.5.
        f.last_kind = Some(kind);
        if f.settings.likely_to_flee.unwrap_or(LIKELY_TO_FLEE) > LIKELY_TO_FLEE && !f.must_flee {
            f.must_flee = true;
            notes.push("likelyToFlee above 0.5: it must flee (rule 22.5)".into());
        }
        // 22.6: the damage.
        let rng = &mut self.fight_rng;
        let dmg = Damage::roll(d, f.stats.0[3], range, &mut || rng.fraction());
        let before = f.stats.0[0];
        f.stats.0[0] -= dmg.loss;
        let after = f.stats.0[0];
        let line = format!("hit by {from}, {}: {} (roll {:.0}-125 %, rule 16); health {before} -> {after}", kind.name(), dmg.describe(), range.0 * 100.0);
        if f.dying.is_some() {
            // Rule 21: health still taken, others still alarmed; nothing else.
            self.fight_note(&name, format!("{line}; it is dying: nothing else (rule 21)"));
            let told = self.run_alarm(&name, now);
            if !told.is_empty() {
                self.fight_note(&name, format!("the alarm with run (rule 26): {}", told.join(", ")));
            }
            return;
        }
        if after < 0 {
            self.fight_note(&name, format!("{line}; below 0: it dies, the rest of the hit is skipped (rules 18, 22.6)"));
            self.character_dies(j, kind, false);
            return;
        }
        // 22.7: an attack under way: its next is due attackDelay from now.
        if f.attack_time == NEVER {
            f.attack_time = now + f64::from(f.settings.attack_delay);
            notes.push(format!("its attack was under way: next attack due at {:.2} (rule 22.7)", f.attack_time));
        }
        // 22.8: the flinch (rule 23) or the ouch sound (rule 24).
        let percent = f.settings.normal_react_percent.unwrap_or(REACT_PERCENT);
        let special = f.settings.special_react_anim.clone().filter(|_| f.settings.special_react_percent.is_some_and(|p| dmg.flinch_roll(p)));
        let can_cut = a.player.cut_short || a.player.has_ended();
        let paused = now < f.flinch_end + FLINCH_PAUSE;
        let why_not = if special.is_none() && !dmg.flinch_roll(percent) {
            Some(format!("R {} is not above {percent} x D", dmg.roll))
        } else if !can_cut {
            Some(format!("'{}' can't be cut short", a.player.movement_name()))
        } else if paused {
            Some(format!("its last flinch ended {:.2} s ago (2 s pause)", now - f.flinch_end))
        } else {
            None
        };
        match why_not {
            None => {
                let m = special.unwrap_or_else(|| "React".to_string());
                let how = match a.player.request(&m) {
                    crate::named_anim::Request::AtOnce(st) if !st.unknown => "starts at once".to_string(),
                    crate::named_anim::Request::AtOnce(_) => "starts at once (not in its list: movement 0)".to_string(),
                    _ => format!("waits for '{}'", a.player.movement_name()),
                };
                f.flinching = Some(m.clone());
                f.last_ouch = now;
                notes.push(format!("flinches: '{m}' {how} (rule 23)"));
            }
            Some(why) => {
                if now + 1e-6 >= f.last_ouch + OUCH_EVERY {
                    f.last_ouch = now;
                    let ouch = f.settings.ouch_sound.clone().unwrap_or_else(|| DEFAULT_OUCH.to_string());
                    let at = a.player.base.position;
                    self.sounds.push(crate::sound::Start::placed_one_shot(&ouch, at, format!("{name}'s ouch sound (fighting.md rule 24)")));
                    notes.push(format!("no flinch ({why}): ouch sound {ouch} (rule 24)"));
                } else {
                    notes.push(format!("no flinch ({why}); no ouch sound, the last was {:.2} s ago (rule 24)", now - f.last_ouch));
                }
            }
        }
        // 22.9: it learns where Connor is now.
        if let Some(ai) = self.anims[j].character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
            ai.last_seen = connor;
        }
        self.fight_note(&name, format!("{line}; {}", notes.join("; ")));
        // 22.10: the alarm with "run" (rule 26).
        let told = self.run_alarm(&name, now);
        // Rule 25: no Daventry character has Surprise lines (not built).
        // Rule 28: it had no target when hit and is still not alarmed.
        let still = self.anims[j].character.as_deref().and_then(|c| c.ai.as_ref()).is_some_and(|ai| !ai.alarmed);
        if !had_target && standing && still {
            self.hit_unalarmed(j, now);
        }
        if !told.is_empty() {
            self.fight_note(&name, format!("the alarm with run (rule 26): {}", told.join(", ")));
        }
    }

    /// Rule 26: after every hit, every active character with an alarm
    /// radius above 0 that can see Connor (or whose team has a member that
    /// can): alarmed if it had no target (rule 23, its reaction delay and
    /// team); then in every case hit count 2, `run` at once if moving,
    /// chase timer 10 s. Returns who, for the log.
    fn run_alarm(&mut self, hit: &str, now: f64) -> Vec<String> {
        let mut told = Vec::new();
        for k in 0..self.anims.len() {
            if self.anims[k].gone {
                continue;
            }
            let Some(ai) = self.anims[k].character.as_deref().and_then(|c| c.ai.as_ref()) else { continue };
            if !ai.active || ai.settings.alarm_radius <= 0.0 || ai.sinking {
                continue;
            }
            let team = ai.settings.team.clone();
            let sees = ai.sees || self.team_sees_pub(k, team.as_deref());
            if !sees {
                continue;
            }
            let name = self.anims[k].player.name.clone();
            if !ai.alarmed {
                self.alarm_pub(k, &format!("the alarm with run after {hit} was hit (fighting.md rule 26)"), now);
            }
            let a = &mut self.anims[k];
            let c = a.character.as_deref_mut().unwrap();
            let moving = c.state == crate::character::State::Move;
            let run = if moving { format!(", {}", c.set_moving(&mut a.player, true)) } else { String::new() };
            let ai = c.ai.as_mut().unwrap();
            ai.hit_count = ai.hit_count.max(2);
            ai.chase_until = now + RUN_ALARM_CHASE;
            told.push(format!("{name} (hit count 2, chase timer 10 s{run})"));
        }
        told
    }

    /// Rule 28: hit while not alarmed and standing. No team and a chase
    /// radius above 0: it starts moving at `run` with no target (the hunt,
    /// `character_ai`). Otherwise: alarmed at once, no reaction delay.
    fn hit_unalarmed(&mut self, j: usize, now: f64) {
        if self.ai_off {
            let name = self.anims[j].player.name.clone();
            self.fight_note(&name, "hit while not alarmed: no hunt, no alarm: the AI is switched off (test switch, ours)");
            return;
        }
        let a = &mut self.anims[j];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_ref() else { return };
        if ai.settings.team.is_none() && ai.settings.chase_radius > 0.0 {
            c.ai.as_mut().unwrap().hunting = true;
            let line = c.enter_move(&mut a.player, now, true, "hit while not alarmed, no team: runs with no target, fighting.md rule 28");
            self.fight_note(&name, line);
            return;
        }
        self.alarm_pub(j, "hit while not alarmed (a team or a chase radius of 0): alarmed at once (fighting.md rule 28)", now);
        if let Some(ai) = self.anims[j].character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
            ai.react_at = now;
        }
    }
}

impl crate::active::WorldState {
    /// Rule 30 (M5 5c-7): it enters the dying state. Health 0; its die
    /// movement by the last hit's kind (`now`: none, straight to rule 32);
    /// its drops (5c-8); off the active list; gravity off; not solid
    /// (collision type and mask 0); its state never changes again.
    pub(crate) fn character_dies(&mut self, j: usize, kind: HitKind, now_flag: bool) {
        let now = self.now();
        let a = &mut self.anims[j];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(f) = c.fight.as_mut() else { return };
        if f.dying.is_some() {
            return;
        }
        f.stats.0[0] = 0;
        f.last_kind = Some(kind);
        let has = |n: &str| a.player.movements.iter().any(|m| m.name.eq_ignore_ascii_case(n));
        let movement = if now_flag {
            None
        } else {
            Some(match kind {
                HitKind::Normal => "Die",
                HitKind::Projectile if has("projdie") => "projdie",
                HitKind::Projectile => "Die",
                HitKind::Fire => "firedie",
                HitKind::Ice => "icedie",
            })
        };
        let how = match movement {
            Some(m) => {
                let st = a.player.start(m);
                format!("'{m}' starts at once{}", if st.unknown { " (not in its list: movement 0)" } else { "" })
            }
            None => "Die now: no movement, straight to its end (rule 30)".to_string(),
        };
        f.dying = Some(movement.map(str::to_string));
        f.flinching = None;
        c.state = crate::character::State::Die;
        c.gravity = false;
        c.collision_type = 0;
        c.collision_mask = 0;
        c.turn_to = None;
        let mut was_active = false;
        if let Some(ai) = c.ai.as_mut() {
            was_active = ai.active;
            ai.active = false;
            ai.alarmed = false;
            ai.hunting = false;
            ai.on_way_home = false;
        }
        self.active_list.retain(|n| !n.eq_ignore_ascii_case(&name));
        let talk = if self.conversation.is_some() { "; a conversation it was in would end (not built: none in Daventry)" } else { "" };
        self.fight_note(
            &name,
            format!(
                "dies (rule 30): kind {}, {how}; health 0; {}; gravity off; not solid (collision type and mask 0); its state never changes again{talk}",
                kind.name(),
                if was_active { "off the active list" } else { "it was not on the active list" }
            ),
        );
        self.drop_items(j);
        if now_flag {
            self.death_ended(j, now);
        }
    }

    /// The viewer console's `kill` (ours, not in the original): the
    /// character dies as from a normal hit (rule 30), without the hit's
    /// alarm to the others. `Err` with the reason when it can't.
    pub fn console_kill(&mut self, name: &str) -> Result<String, String> {
        let j = self.character_index(name).ok_or_else(|| format!("no character '{name}'"))?;
        let a = &self.anims[j];
        let Some(f) = a.character.as_deref().and_then(|c| c.fight.as_ref()) else {
            return Err(format!("{} never fights (no monster-list line): it can't die", a.player.name));
        };
        if f.dying.is_some() {
            return Err(format!("{} is already dying or dead", a.player.name));
        }
        let name = a.player.name.clone();
        self.fight_note(&name, "console kill: dies as from a normal hit (ours)");
        self.character_dies(j, HitKind::Normal, false);
        Ok(name)
    }

    /// Rule 31 (M5 5c-8; C-2): its drops, when the dying starts: the
    /// line's `invDrop`, then the type's. Each entry is compared (quotes
    /// and `INVITEM_` off, capitals ignored) with the part names of the
    /// pose shown now (the die movement's first frame): one that names a
    /// part is kept for the end of the die movement (rule 31a; the last
    /// such wins, earlier ones are lost); every other one is a new item at
    /// its feet (offset 0 in Daventry), thrown as `KQObject::tossItems`
    /// (`items.md` rule 12).
    pub(crate) fn drop_items(&mut self, j: usize) {
        let a = &self.anims[j];
        let name = a.player.name.clone();
        let at = a.player.base.position;
        let Some(f) = a.character.as_deref().and_then(|c| c.fight.as_ref()) else { return };
        let drops = f.settings.drops.clone();
        if drops.is_empty() {
            self.fight_note(&name, "drops: nothing (rule 31)");
            return;
        }
        let frame = a.player.frame();
        let parts: Vec<String> = a.player.movement().and_then(|m| m.frame_parts.get(frame)).map(|p| p.iter().map(|(n, _)| n.clone()).collect()).unwrap_or_default();
        let pose = format!("'{}' frame {frame}", a.player.movement_name());
        let mut kept: Option<(String, i64)> = None;
        let mut lines = Vec::new();
        for d in &drops {
            let key = drop_part_name(&d.name);
            if parts.iter().any(|p| p.eq_ignore_ascii_case(key)) {
                if let Some((lost, _)) = kept.replace((d.name.clone(), d.quantity)) {
                    lines.push(format!("{lost}: also names a part, replaced by {}: lost (rule 31)", d.name));
                }
                lines.push(format!("{} x{}: names a part of {pose}: kept for the die movement's end, not thrown (rule 31a)", d.name, d.quantity));
            } else {
                let before = self.items.objects.len();
                lines.push(self.toss(&d.name, d.quantity, at));
                if self.items.objects.len() > before {
                    self.new_inv_item = self.items.objects.last().map(|o| o.name.clone());
                }
            }
        }
        if let Some(f) = self.anims[j].character.as_deref_mut().and_then(|c| c.fight.as_mut()) {
            f.kept_drop = kept;
        }
        let line = format!("drops (rule 31; parts of {pose}: {parts:?}): {}", lines.join("; "));
        self.item_note(format!("{name}: {line}"));
        self.fight_note(&name, line);
    }

    /// Rule 31a (C-2), first of the die movement's end steps: the kept
    /// weapon made with its `body` part on the part of the same name in
    /// the last frame (no such part: its place and turn, 750 up); its line
    /// test waits for the collision scene ([`Self::laid_checks_step`]).
    fn lay_kept_drop(&mut self, j: usize) {
        let a = &mut self.anims[j];
        let dying = a.character.as_deref().and_then(|c| c.fight.as_ref()).and_then(|f| f.dying.clone());
        let now_flag = dying == Some(None);
        let Some((kind, quantity)) = a.character.as_deref_mut().and_then(|c| c.fight.as_mut()).and_then(|f| f.kept_drop.take()) else { return };
        let name = a.player.name.clone();
        // The die movement's last frame, even when a waiting movement has
        // just started in its place (rule 8.1); `Die now`: the pose shown.
        let die = dying.flatten().and_then(|d| a.player.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&d)));
        let (movement, frame) = match die {
            Some(m) => (Some(m), m.frames() - 1),
            None => (a.player.movement(), a.player.frame()),
        };
        let base = kq8_data::shape::Transform { flags: 0, m: a.player.base.rotation, p: a.player.base.position };
        let owner = crate::door::chain(&base, &movement.map_or_else(crate::door::identity, |m| m.turn_at_frame(frame)));
        let place = crate::collide::Placement { rotation: owner.m, position: owner.p };
        let pose = format!("'{}' frame {frame}", movement.map_or("(none)", |m| m.name.as_str()));
        let part = movement.and_then(|m| m.frame_parts.get(frame)).and_then(|p| p.iter().find(|(n, _)| n.eq_ignore_ascii_case(drop_part_name(&kind)))).map(|(_, t)| *t);
        let body = self.items.body(&kind);
        let (laid, how) = match part {
            Some(t) => {
                let on = crate::door::chain(&owner, &t);
                let laid = crate::item::laid_on(&on, body.as_ref());
                let what = if body.is_some() { "its body part" } else { "its own origin (its model has no body part)" };
                (laid, format!("{what} on part '{}' of {pose} (the part at ({:.0}, {:.0}, {:.0}); scale {:.3} {:.3} {:.3})", drop_part_name(&kind), on.p[0], on.p[1], on.p[2], laid.scale[0], laid.scale[1], laid.scale[2]))
            }
            None => {
                let p = place.position;
                let laid = crate::item::Laid { position: [p[0], p[1], p[2] + LAID_FALLBACK_UP], angles: kq8_data::objects::angles_of(place.rotation), scale: [1.0; 3] };
                (laid, format!("no part '{}' in {pose}: its place and turn, {LAID_FALLBACK_UP} up", drop_part_name(&kind)))
            }
        };
        let now = if now_flag { "; Die now: whether a part is matched with no die movement is not sure (rule 31a), ours: the pose shown" } else { "" };
        let made = self.items.make(&kind, quantity, crate::item::Place::World { position: laid.position, angles: laid.angles }).map(|o| o.name.clone());
        let line = match made {
            Ok(item) => {
                self.known.insert(item.to_ascii_lowercase());
                self.new_inv_item = Some(item.clone());
                self.laid_checks.push(LaidCheck { item: item.clone(), kind: kind.clone(), quantity, feet: place.position, owner: name.clone() });
                let (p, r) = (laid.position, laid.angles);
                format!("{item} ({kind} x{quantity}) laid at ({:.0}, {:.0}, {:.0}) turned ({:.3}, {:.3}, {:.3}): {how}; not thrown; line test next (rule 31a){now}", p[0], p[1], p[2], r[0], r[1], r[2])
            }
            Err(e) => format!("{kind}: {e}; not made"),
        };
        self.item_note(format!("{name}: kept weapon (rule 31a): {line}"));
        self.fight_note(&name, format!("kept weapon (rule 31a): {line}"));
    }

    /// Rule 31a.2 (C-2): each weapon laid since the last update: the line
    /// from 1,000 above it to 1,000 above the dying character's feet. If
    /// the terrain or a solid object other than that character cuts it,
    /// the item is thrown from the feet instead, like the rule 31 items;
    /// otherwise it stays exactly there (no throw, no fall). Called with
    /// the thrown items' scene, before their step.
    pub fn laid_checks_step(&mut self, ground: &dyn crate::connor_mover::GroundLookup, buildings: &[crate::collide::BuildingCollider], objects: &[crate::collide::BoxCollider]) {
        if self.laid_checks.is_empty() {
            return;
        }
        use crate::connor_mover::MoverWorld;
        let characters = self.character_colliders.clone();
        let mut scene = crate::connor_mover::Scene::new(ground, buildings, objects);
        scene.characters = &characters;
        for c in std::mem::take(&mut self.laid_checks) {
            let Some(at) = self.items.get(&c.item).and_then(|o| o.placement()).map(|p| p.position) else {
                self.item_note(format!("{}: {} no longer in the world: no line test (rule 31a)", c.owner, c.item));
                continue;
            };
            let from = [at[0], at[1], at[2] + LAID_LINE_UP];
            let to = [c.feet[0], c.feet[1], c.feet[2] + LAID_LINE_UP];
            let line = match scene.line_blocked(from, to, LAID_LINE_MASK, &c.owner.to_ascii_lowercase()) {
                None => format!("line test free ({:.0}, {:.0}, {:.0}) to ({:.0}, {:.0}, {:.0}): it stays there, no throw, no fall (rule 31a.3)", from[0], from[1], from[2], to[0], to[1], to[2]),
                Some(what) => {
                    if let Some(i) = self.items.index(&c.item) {
                        self.items.objects[i].place = crate::item::Place::World { position: c.feet, angles: [0.0; 3] };
                    }
                    let thrown = self.throw_made(&c.item, &c.kind, c.quantity, c.feet);
                    format!("line test blocked by {what}: thrown instead (rule 31a.2): {thrown}")
                }
            };
            self.item_note(format!("{}: {}: {line}", c.owner, c.item));
            self.fight_note(&c.owner, format!("kept weapon {}: {line}", c.item));
        }
    }

    /// `items.md` rule 12: makes one item of `kind` x `quantity` at `at`
    /// and throws it (`thrown-items.md` rules 2, 3: no turn, then the
    /// launch and the tumble). Returns the log text.
    pub(crate) fn toss(&mut self, kind: &str, quantity: i64, at: [f32; 3]) -> String {
        let made = self.items.make(kind, quantity, crate::item::Place::World { position: at, angles: [0.0; 3] }).map(|o| o.name.clone());
        match made {
            Ok(item) => {
                self.known.insert(item.to_ascii_lowercase());
                self.throw_made(&item, kind, quantity, at)
            }
            Err(e) => format!("{kind}: {e}; not made"),
        }
    }

    /// Throws item `item` (already made, lying at `at`) as [`Self::toss`]
    /// does; also a laid weapon whose line test is blocked (rule 31a.2).
    /// Returns the log text.
    fn throw_made(&mut self, item: &str, kind: &str, quantity: i64, at: [f32; 3]) -> String {
        let rng = &mut self.fight_rng;
        let r = [rng.fraction(), rng.fraction(), rng.fraction()];
        let spin = rng.fraction();
        let shape = self.items.get(item).and_then(|o| self.items.click_box(o));
        let fl = crate::item::Flight::launch(item, at, r, shape).with_tumble(spin);
        let v = fl.velocity;
        self.flights.push(fl);
        format!(
            "{item} ({kind} x{quantity}) thrown from ({:.0}, {:.0}, {:.0}): velocity ({:.0}, {:.0}, {:.0}), heading {:.0} deg, {:.1} deg up, {:.0} units/s",
            at[0],
            at[1],
            at[2],
            v[0],
            v[1],
            v[2],
            r[0] * 360.0,
            crate::item::TOSS_ELEVATION.0 + r[1] * (crate::item::TOSS_ELEVATION.1 - crate::item::TOSS_ELEVATION.0),
            (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
        )
    }

    /// `thrown-items.md` (M5 5e-1): each thrown item flies one update,
    /// after the characters' movers. Its move meets the floor grid, the
    /// buildings and the objects whose type has bit 0x1 or 0x100 (rule
    /// 11); not characters, Connor or other items. Its land sound plays at
    /// its first touch (rule 12.2). Picked up (or gone) in the air: the
    /// flight ends.
    pub fn flights_step(&mut self, ground: &dyn crate::connor_mover::GroundLookup, buildings: &[crate::collide::BuildingCollider], objects: &[crate::collide::BoxCollider]) {
        if self.flights.is_empty() {
            return;
        }
        let dt = (1.0 / crate::UPDATES_PER_SECOND) as f32;
        let blocking: Vec<crate::collide::BoxCollider> = objects.iter().filter(|o| o.kind.collision_type & crate::item::ITEM_OBSTACLE_BITS != 0).cloned().collect();
        let shaped = crate::connor_mover::Scene::new(ground, buildings, &blocking);
        let bare = crate::connor_mover::Scene::new(ground, &[], &[]);
        // Rule 12.3: what throws it again: anything with a script, and any
        // animated object (doors, chests and the rest, `anim-scripts.md`
        // rule 8; the play-once ones too), lower case.
        let mut again: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
        again.extend(self.doors.iter().map(|d| d.door.name.to_ascii_lowercase()));
        again.extend(self.chests.iter().map(|c| c.chest.name.to_ascii_lowercase()));
        again.extend(self.objects.iter().map(|o| o.player.name.to_ascii_lowercase()));
        again.extend(self.anims.iter().filter(|a| a.character.is_none()).map(|a| a.player.name.to_ascii_lowercase()));
        again.extend(self.clickables.iter().filter(|c| c.script.is_some()).map(|c| c.name.to_ascii_lowercase()));
        again.extend(self.buildings.iter().filter(|b| b.script.is_some()).map(|b| b.name.to_ascii_lowercase()));
        let throws_again = |name: &str| again.contains(&name.to_ascii_lowercase());
        let mut flights = std::mem::take(&mut self.flights);
        flights.retain_mut(|f| {
            let Some(i) = self.items.index(&f.item) else { return false };
            let crate::item::Place::World { position, .. } = self.items.objects[i].place.clone() else {
                self.item_log.push(format!("update {}: {}: no longer in the world: its flight ends", self.updates, f.item));
                return false;
            };
            let rng = &mut self.fight_rng;
            let s = f.update(position, dt, if f.shaped { &shaped } else { &bare }, &throws_again, &mut || rng.fraction());
            let p = s.position;
            let kind = self.items.objects[i].kind.clone();
            let mut line = match s.wanted {
                Some(w) => format!("t {:.3} wanted ({:.0}, {:.0}, {:.0}) at ({:.0}, {:.0}, {:.0})", f.clock, w[0], w[1], w[2], p[0], p[1], p[2]),
                None => format!("t {:.3} settling at ({:.0}, {:.0}, {:.0})", f.clock, p[0], p[1], p[2]),
            };
            if f.coming_down {
                line += ", coming down";
            }
            if !s.touches.is_empty() {
                line += &format!("; touches: {}", s.touches.join(", "));
            }
            if let Some(what) = &s.first_touch {
                let sound = self.items.kind(&kind).and_then(|t| t.land_sound.clone());
                if let Some(snd) = &sound {
                    self.sounds.push(crate::sound::Start::placed_one_shot(snd, p, format!("{} touched {what} (thrown-items.md rule 12.2)", f.item)));
                }
                line += &format!("; first touch: land sound {}", sound.as_deref().unwrap_or("none"));
            }
            for t in &s.thrown_again {
                line += &format!("; thrown again (rule 12.3) by {t}");
            }
            if let Some(why) = &s.arc_ended {
                line += &format!("; the arc ends: {why}, ordinary gravity on");
            }
            // Rules 9, 10: the tumble about its own x axis; no turn once
            // the flight has ended.
            if s.ended.is_none() {
                line += &format!(", turned {:.2} rad", s.angles[0]);
            }
            self.items.objects[i].place = crate::item::Place::World { position: p, angles: s.angles };
            if let Some(why) = s.ended {
                let flat = ((p[0] - f.from[0]).powi(2) + (p[1] - f.from[1]).powi(2)).sqrt();
                let line = format!(
                    "update {}: {} ({kind}) flight ends ({why}, rule 16) at ({:.0}, {:.0}, {:.0}) after {} updates, {flat:.0} from where it was last thrown",
                    self.updates, f.item, p[0], p[1], p[2], f.updates
                );
                self.item_log.push(line.clone());
                self.fight_log.push(line);
                return false;
            }
            self.item_log.push(format!("update {}: {} flight {}: {line}", self.updates, f.item, f.updates));
            true
        });
        self.flights.extend(flights);
    }

    /// Rule 32: its die movement ended (or `Die now`): Connor gets its
    /// experience (a stats event: his levels apply); a corpse where it
    /// stood (`deadDts`; `Projectile`: `projdKqs` or `deadDts`; `Ice`:
    /// `idedKqs` or none; `Fire`: none); its script gets `Die <kind>`
    /// (queued for the caller, which runs scripts); it is removed. Rule 33:
    /// the first-kill tip (logged only).
    pub(crate) fn death_ended(&mut self, j: usize, now: f64) {
        // Rule 31a: the kept weapon first, before the experience and the
        // corpse.
        self.lay_kept_drop(j);
        let a = &mut self.anims[j];
        let name = a.player.name.clone();
        let (position, rotation) = (a.player.base.position, a.player.base.rotation);
        let Some(f) = a.character.as_deref().and_then(|c| c.fight.as_ref()) else { return };
        let kind = f.last_kind.unwrap_or(HitKind::Normal);
        let exp = f.stats.0[4];
        let s = &f.settings;
        let shape = match kind {
            HitKind::Normal => s.dead_dts.clone(),
            HitKind::Projectile => s.projd_kqs.clone().or_else(|| s.dead_dts.clone()),
            HitKind::Ice => s.ided_kqs.clone(),
            HitKind::Fire => None,
        };
        let type_name = s.type_name.clone();
        a.gone = true;
        // 32.1: experience to Connor, whoever killed it.
        let xp = match self.connor_stats.as_mut() {
            Some(c) => {
                let before = *c;
                let level = c.change(crate::stats::Stats([0, 0, 0, 0, exp]));
                let after = *c;
                self.connor_stats_events += 1;
                format!(
                    "Connor gets its experience {exp}: {:?} level {} -> {:?} level {}{}",
                    before.stats.0,
                    before.level,
                    after.stats.0,
                    after.level,
                    level.map_or(String::new(), |l| format!("; {l}"))
                )
            }
            None => format!("Connor's stats are unknown: its experience {exp} is lost"),
        };
        // 32.2: the corpse.
        let corpse = match shape {
            Some(shape) => {
                let line = format!("a corpse {shape} where it stood ({:.0}, {:.0}, {:.0}), turned the same way, no shadow, not solid", position[0], position[1], position[2]);
                self.corpses.push(Corpse { name: name.clone(), shape, position, rotation, loaded: false });
                line
            }
            None => format!("no corpse ({})", if kind == HitKind::Fire { "Fire: burnt away" } else { "no shape for this kind" }),
        };
        self.fight_note(&name, format!("its death ended (rule 32): {xp}; {corpse}; its script gets Die {}; removed from the world", kind.name()));
        // 32.3: queued for the caller; 32.4: removed.
        self.die_calls.push((name.clone(), kind.name()));
        self.active_list.retain(|n| !n.eq_ignore_ascii_case(&name));
        self.requests.push(crate::active::Request::Delete { name: name.clone(), at: format!("{name} died (fighting.md rule 32)") });
        // Rule 33.
        if !self.kill_tip && !NO_KILL_TIP.iter().any(|t| t.eq_ignore_ascii_case(&type_name)) {
            self.kill_tip = true;
            self.fight_note(&name, format!("first kill (type {type_name}): a one-off tip would come at {:.2} (rule 33; not followed further)", now + KILL_TIP_DELAY));
        }
    }
}

impl crate::active::WorldState {
    /// `fighting.md` rule 36 (M5 5c-9): a ranged character's `Do`: it
    /// shoots its weapon's projectile at its target (Connor, when
    /// alarmed); no reach or front test. `ranged-weapons.md` (M5 5f-1,
    /// 5f-2): it leaves from the bolt's model part (`meshName`) in the
    /// frame playing, found at a shot and then kept (rule 24; fallback its
    /// `shootOffsets`: none in Daventry, 0, 0, 0), turned with it; aimed
    /// at his feet raised by 0.75 × the top of his box, led for his
    /// movement (rule 25).
    fn shoot(&mut self, i: usize, movement: &str) {
        let dt = (1.0 / crate::UPDATES_PER_SECOND) as f32;
        let a = &self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref() else { return };
        let Some(f) = c.fight.as_ref() else { return };
        let Some(target) = self.connor_at.filter(|_| c.ai.as_ref().is_some_and(|ai| ai.alarmed)) else {
            self.fight_note(&name, format!("Do ({movement}): no target (not alarmed): no shot (rule 36)"));
            return;
        };
        let file = f.settings.projectile.clone().unwrap_or_default();
        let Some(p) = self.projectiles.get(&file.to_ascii_lowercase()).cloned() else {
            self.fight_note(&name, format!("Do ({movement}): projectile file {file} not read: no shot"));
            return;
        };
        let part = p.mesh_name.clone().unwrap_or_default();
        let (offset, found, how) = match f.shoot_offset {
            Some(o) => (o, None, "kept from an earlier shot".to_string()),
            None => {
                // ranged-weapons.md rule 24 (Q140 c): the frame after this
                // update's step, where the movement is now (the queued `do`
                // is handled after every object moved): `spbattck` at
                // 0.1667, frame 3, not frame 2 at the cue's own 0.125.
                let frame = a.player.movement().map_or(0, |m| m.frame_at(a.player.phase));
                let place = a.player.movement().and_then(|m| m.frame_parts.get(frame)).and_then(|parts| parts.iter().find(|(n, _)| n.eq_ignore_ascii_case(&part))).map(|(_, p)| p.p);
                match place {
                    Some(o) => (o, Some(o), format!("its part '{part}' in frame {frame} of '{}' (kept from now on)", a.player.movement_name())),
                    None => ([0.0; 3], None, format!("no part '{part}' in frame {frame} of '{}': its shootOffsets (none read, none in Daventry: 0, 0, 0)", a.player.movement_name())),
                }
            }
        };
        let pos = a.player.base.position;
        let r = a.player.base.rotation;
        let from: [f32; 3] = std::array::from_fn(|k| pos[k] + offset[0] * r[0][k] + offset[1] * r[1][k] + offset[2] * r[2][k]);
        let raise = AIM_SHARE * crate::connor_mover::CONNOR_BOX.1[2];
        let at = [target[0], target[1], target[2] + raise];
        let v = self.connor_was.map_or([0.0; 3], |w| std::array::from_fn(|k| (target[k] - w[k]) / dt));
        let (aim, t) = lead(from, at, v, p.speed);
        let lead_text = match t {
            Some(t) => format!("led for his speed ({:.0}, {:.0}, {:.0}): meeting in {t:.2} s", v[0], v[1], v[2]),
            None => "straight at him (no meeting point)".to_string(),
        };
        let damage = f.stats.0[2];
        if let Some(o) = found {
            if let Some(f) = self.anims[i].character.as_deref_mut().and_then(|c| c.fight.as_mut()) {
                f.shoot_offset = Some(o);
            }
        }
        let why = format!(
            "Do ({movement}): start offset ({:.0}, {:.0}, {:.0}): {how} (rule 24); aim: Connor + {raise:.0} (0.75 x his box top, rule 25), {lead_text}; ranged damage {damage} (rule 22)",
            offset[0], offset[1], offset[2]
        );
        self.launch_bolt(&name, &file, &p, from, aim, damage, &why);
    }

    /// `ranged-weapons.md` rules 26, 27 (M5 5f-1): a bolt of projectile
    /// file `file` enters the world at `from`, turned once to point at
    /// `aim` (heading and pitch, no roll), carrying `damage`. `why`: the
    /// start of the log line.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn launch_bolt(&mut self, shooter: &str, file: &str, p: &kq8_data::characters::ProjectileKeys, from: [f32; 3], aim: [f32; 3], damage: i64, why: &str) {
        let Some((direction, heading, pitch)) = aim_direction(from, aim) else {
            self.fight_note(shooter, format!("{why}; the aim point is the start point: no shot"));
            return;
        };
        self.bolts_shot += 1;
        let balls = p.bounds.map(|(min, max)| crate::collide::BallStack::from_box(min, max));
        let hammer = p.hammer.then_some(HammerFlight { back: false, modifier: p.modifier, min_damage: p.min_damage, max_damage: p.max_damage });
        let bolt = Bolt {
            name: format!("{}{}", if hammer.is_some() { "hammer" } else { "bolt" }, self.bolts_shot),
            shooter: shooter.to_string(),
            shape: p.shape.clone(),
            position: from,
            direction,
            heading,
            pitch,
            speed: p.speed,
            travelled: 0.0,
            max: p.max_distance,
            damage,
            // azriel-hammer.md rule 8: the hammer's hits are `Projectile`.
            kind: if hammer.is_some() { HitKind::Projectile } else { bolt_kind(p.weapon_type) },
            mask: p.collider_mask,
            balls,
            updates: 0,
            hammer,
        };
        let shape = match &bolt.balls {
            Some(b) => format!("box {:?} as {} ball(s) of radius {:.0} at {:?} above it (Q140 a)", p.bounds.unwrap_or_default(), b.heights.len(), b.radius, b.heights),
            None => "no box: a point".into(),
        };
        self.fight_note(
            shooter,
            format!(
                "{why}; shoots {} ({file}, {} units/s = {:.0} per update, range {}, mask 0x{:X}, {}{}) from ({:.0}, {:.0}, {:.0}) at ({:.0}, {:.0}, {:.0}): heading {:.1} deg, pitch {:.1} deg; {shape} (ranged-weapons.md rules 26, 27)",
                bolt.name,
                p.speed,
                p.speed / crate::UPDATES_PER_SECOND as f32,
                p.max_distance,
                bolt.mask,
                bolt.kind.name(),
                match &bolt.hammer {
                    Some(h) => format!(
                        "; Azriel's Hammer: a hit is {} (modifier {} + ranged damage {damage}, capped at {}, at least {}; azriel-hammer.md rule 8)",
                        hammer_damage(h.modifier, damage, h.min_damage, h.max_damage),
                        h.modifier,
                        h.max_damage,
                        h.min_damage
                    ),
                    None => String::new(),
                },
                from[0],
                from[1],
                from[2],
                aim[0],
                aim[1],
                aim[2],
                heading.to_degrees(),
                pitch.to_degrees()
            ),
        );
        self.bolts.push(bolt);
    }

    /// `ranged-weapons.md` rules 28–30 (M5 5f-1): each update every bolt
    /// moves speed / 15 straight along its direction (no gravity). Its
    /// move is swept with its ball stack against what its mask covers:
    /// the buildings (bit 0x8), the objects and characters whose type
    /// shares a bit with it, Connor (type 4); never its shooter; a start
    /// inside counts as a hit (`connor-collision.md` rule 19). At the
    /// first touch it hits (rule 30) and is removed; otherwise, at or below
    /// the floor grid it is removed, and past `maxDistance` too.
    ///
    /// Azriel's Hammer (`azriel-hammer.md` rules 7–9; M6 6b-2): on its way
    /// out the same move (never its thrower); at the first touch (rule 8's
    /// hit), at or below the floor grid, or past `maxDistance` it turns
    /// back instead of being removed. On the way back only Connor stops it;
    /// touching him, passing its range or reaching the floor grid it is
    /// removed and its thrower is told it came back (`hammers_back`).
    pub(crate) fn bolts_step(&mut self, ground: &dyn crate::connor_mover::GroundLookup, buildings: &[crate::collide::BuildingCollider], objects: &[crate::collide::BoxCollider]) {
        use crate::collide::{BallStack, CylinderCollider, ObjectHit};
        let dt = (1.0 / crate::UPDATES_PER_SECOND) as f32;
        let bolts = std::mem::take(&mut self.bolts);
        let mut kept = Vec::new();
        for mut b in bolts {
            let back = b.hammer.as_ref().is_some_and(|h| h.back);
            let from = b.position;
            let step = b.speed * dt;
            let to: [f32; 3] = std::array::from_fn(|k| from[k] + b.direction[k] * step);
            let point = BallStack { radius: 0.0, heights: vec![0.0] };
            let balls = b.balls.as_ref().unwrap_or(&point);
            let dist = |p: [f32; 3]| ((p[0] - from[0]).powi(2) + (p[1] - from[1]).powi(2) + (p[2] - from[2]).powi(2)).sqrt();
            let skip = |n: &str| n.eq_ignore_ascii_case(&b.shooter);
            // What it touches first: (distance, what, touch point).
            let mut best: Option<(f32, Touched, [f32; 3])> = None;
            let mut keep = |p: [f32; 3], what: Touched| {
                let d = dist(p);
                if best.as_ref().is_none_or(|(bd, _, _)| d < *bd) {
                    best = Some((d, what, p));
                }
            };
            // azriel-hammer.md rule 9: on its way back only Connor.
            if b.mask & 0x8 != 0 && !back {
                for w in buildings.iter().filter(|w| !self.is_switched_off(&w.name)) {
                    if let Some(c) = w.sweep(balls, from, to) {
                        keep(c.touch, Touched::Building(w.name.clone()));
                    }
                }
            }
            for o in objects.iter().filter(|o| !back && o.kind.collision_type & b.mask != 0 && !skip(&o.name) && !self.is_switched_off(&o.name)) {
                match o.sweep(balls, from, to) {
                    Some(ObjectHit::Contact(c)) => keep(c.touch, Touched::Object(o.name.clone(), o.kind.collision_type)),
                    // connor-collision.md rule 19: projectiles count a start inside as a hit.
                    Some(ObjectHit::StartedInside { .. }) => keep(from, Touched::Object(o.name.clone(), o.kind.collision_type)),
                    None => {}
                }
            }
            for (j, a) in self.anims.iter().enumerate() {
                let Some(c) = a.character.as_deref() else { continue };
                if back || a.gone || c.unborn || c.collision_type & b.mask == 0 || skip(&a.player.name) || !self.characters_solid.contains(&a.player.name.to_ascii_lowercase()) {
                    continue;
                }
                let Some((min, max)) = c.bounds else { continue };
                let cyl = CylinderCollider::from_box(&a.player.name, a.player.base.position, min, max, c.collision_type, c.collision_mask);
                let s = cyl.sweep(balls, from, to);
                if !s.inside.is_empty() {
                    keep(from, Touched::Character(j));
                } else if let Some(k) = s.contact {
                    keep(k.touch, Touched::Character(j));
                }
            }
            if let (true, Some(at)) = (back || (b.mask & crate::collide::CONNOR_TYPE != 0 && !skip("Connor")), self.connor_at) {
                let (min, max) = crate::connor_mover::CONNOR_BOX;
                let cyl = CylinderCollider::from_box("Connor", at, min, max, crate::collide::CONNOR_TYPE, 0);
                let s = cyl.sweep(balls, from, to);
                if !s.inside.is_empty() {
                    keep(from, Touched::Connor);
                } else if let Some(k) = s.contact {
                    keep(k.touch, Touched::Connor);
                }
            }
            b.updates += 1;
            if b.hammer.is_some() {
                if let Some(b) = self.hammer_step(b, best, from, to, step, ground) {
                    kept.push(b);
                }
                continue;
            }
            let Some((d, what, p)) = best else {
                b.travelled += step;
                b.position = to;
                let floor = ground.ground(to[0], to[1]).map(|(h, _)| h);
                let line = format!("{} update {}: at ({:.0}, {:.0}, {:.0}), {:.0} travelled", b.name, b.updates, to[0], to[1], to[2], b.travelled);
                if let Some(h) = floor.filter(|h| to[2] <= *h) {
                    self.fight_note(&b.shooter, format!("{line}; at or below the floor grid ({h:.0}): removed, no damage (rule 29)"));
                } else if b.travelled > b.max {
                    self.fight_note(&b.shooter, format!("{line}; more than its range {:.0}: removed (rule 28)", b.max));
                } else {
                    self.fight_note(&b.shooter, line);
                    kept.push(b);
                }
                continue;
            };
            b.travelled += d;
            b.position = p;
            let at = format!("{} update {}: touches {{}} at ({:.0}, {:.0}, {:.0}), {:.0} travelled; removed there (rule 30)", b.name, b.updates, p[0], p[1], p[2], b.travelled);
            self.bolt_hits(&b, what, &at);
        }
        self.bolts.extend(kept);
    }

    /// Rule 30: what a bolt's touch does. `at`: the log line, with `{}`
    /// for what it touched.
    fn bolt_hits(&mut self, b: &Bolt, what: Touched, at: &str) {
        let shooter = b.shooter.clone();
        match what {
            Touched::Building(n) => self.fight_note(&shooter, at.replace("{}", &format!("building {n}")) + ": no damage"),
            Touched::Connor => {
                self.fight_note(&shooter, at.replace("{}", "Connor") + &format!(": a hit of {}, {} (fighting.md rule 29)", b.damage, b.kind.name()));
                self.hit_connor(&shooter, b.damage, b.kind);
            }
            Touched::Character(j) => {
                let who = self.anims[j].player.name.clone();
                let t = self.anims[j].character.as_deref().map_or(0, |c| c.collision_type);
                if t & IMMUNE_TO_PROJECTILES != 0 {
                    self.fight_note(&shooter, at.replace("{}", &who) + &format!(": its type 0x{t:X} has the immune bit 0x4000: a hit of 0, its script gets ProjImmune"));
                    self.hit_character(j, &shooter, 0, b.kind);
                    self.immune_calls.push(who);
                } else {
                    self.fight_note(&shooter, at.replace("{}", &who) + &format!(": a hit of {}, {} (fighting.md rules 16-33)", b.damage, b.kind.name()));
                    self.hit_character(j, &shooter, b.damage, b.kind);
                }
            }
            Touched::Object(n, t) => {
                if t & IMMUNE_TO_PROJECTILES != 0 {
                    self.fight_note(&shooter, at.replace("{}", &format!("object {n}")) + &format!(": its type 0x{t:X} has the immune bit 0x4000: a hit of 0, its script gets ProjImmune"));
                    self.immune_calls.push(n);
                } else if self.object_at(&n).is_some() {
                    self.fight_note(&shooter, at.replace("{}", &format!("object {n}")) + &format!(": a hit of {}, {}", b.damage, b.kind.name()));
                    let line = self.object_takes(&n, b.damage, b.kind);
                    self.fight_note(&shooter, format!("{}: {line}", b.name));
                } else {
                    self.fight_note(&shooter, at.replace("{}", &format!("object {n}")) + ": no script: nothing (how objects take damage: Q139 e)");
                }
            }
        }
    }
}

impl crate::active::WorldState {
    /// `azriel-hammer.md` rules 7–9 (M6 6b-2): the hammer's update after
    /// its sweep (`best`: its first touch). Way out: a touch (its hit,
    /// rule 8), the floor grid or its range turns it back: put back at
    /// `from` (its position at the start of the update), direction
    /// reversed, distance 0, range = the count (which includes the
    /// turn-back update's step) − one step = the distance from its start
    /// to `from`, so it flies back to its start point (Q148 a); only
    /// Connor in its mask.
    /// Way back: touching Connor, passing its range or the floor grid
    /// ends it: removed, and `hammers_back` tells the thrower. Returns it
    /// while it flies.
    fn hammer_step(&mut self, mut b: Bolt, best: Option<(f32, Touched, [f32; 3])>, from: [f32; 3], to: [f32; 3], step: f32, ground: &dyn crate::connor_mover::GroundLookup) -> Option<Bolt> {
        let h = b.hammer.clone()?;
        let way = if h.back { "back" } else { "out" };
        let thrower = b.shooter.clone();
        let end = match best {
            Some((d, what, p)) => {
                let line = format!("{} update {} ({way}): touches {{}} at ({:.0}, {:.0}, {:.0}), {:.0} travelled", b.name, b.updates, p[0], p[1], p[2], b.travelled + d);
                if h.back {
                    self.fight_note(&thrower, line.replace("{}", "Connor"));
                    "touching Connor".to_string()
                } else {
                    self.hammer_hits(&b, &h, what, &line);
                    "its touch".to_string()
                }
            }
            None => {
                let travelled = b.travelled + step;
                self.fight_note(&thrower, format!("{} update {} ({way}): at ({:.0}, {:.0}, {:.0}), {travelled:.0} travelled", b.name, b.updates, to[0], to[1], to[2]));
                if let Some(g) = ground.ground(to[0], to[1]).map(|(g, _)| g).filter(|g| to[2] <= *g) {
                    format!("at or below the floor grid ({g:.0})")
                } else if travelled > b.max {
                    format!("more than its range {:.0}", b.max)
                } else {
                    b.travelled = travelled;
                    b.position = to;
                    return Some(b);
                }
            }
        };
        if h.back {
            self.fight_note(&thrower, format!("{}: the way back ends ({end}): removed; {thrower} is told it came back, {} updates after the throw (rule 9)", b.name, b.updates));
            self.hammers_back.push((thrower, format!("{} came back ({end}) after {} updates", b.name, b.updates)));
            return None;
        }
        let out = b.travelled;
        let direction = b.direction.map(|v| -v);
        b.direction = direction;
        b.heading = direction[0].atan2(-direction[1]);
        b.pitch = direction[2].clamp(-1.0, 1.0).asin();
        b.position = from;
        b.travelled = 0.0;
        b.max = out;
        b.mask = crate::collide::CONNOR_TYPE;
        b.hammer = Some(HammerFlight { back: true, ..h });
        self.fight_note(
            &thrower,
            format!(
                "{}: turns back ({end}) at ({:.0}, {:.0}, {:.0}), its place at the start of the update; {out:.0} flown out, range back {:.0} (its count {:.0} - one step {step:.0}: back to its start point); heading {:.1} deg, pitch {:.1} deg; only Connor stops it now (rule 9)",
                b.name,
                from[0],
                from[1],
                from[2],
                b.max,
                out + step,
                b.heading.to_degrees(),
                b.pitch.to_degrees()
            ),
        );
        Some(b)
    }

    /// `azriel-hammer.md` rule 8: what the hammer's touch on its way out
    /// does. `line`: the log line, with `{}` for what it touched.
    fn hammer_hits(&mut self, b: &Bolt, h: &HammerFlight, what: Touched, line: &str) {
        let thrower = b.shooter.clone();
        let d = hammer_damage(h.modifier, b.damage, h.min_damage, h.max_damage);
        let parts = format!("modifier {} + ranged damage {} = {}, capped at {}, at least {}: {d}", h.modifier, b.damage, h.modifier + b.damage, h.max_damage, h.min_damage);
        match what {
            Touched::Building(n) => self.fight_note(&thrower, line.replace("{}", &format!("building {n}")) + ": no hit (rule 8)"),
            Touched::Connor => self.fight_note(&thrower, line.replace("{}", "Connor") + ": ignored on the way out (rule 7)"),
            Touched::Character(j) => {
                let who = self.anims[j].player.name.clone();
                let t = self.anims[j].character.as_deref().map_or(0, |c| c.collision_type);
                if t & IMMUNE_TO_PROJECTILES != 0 {
                    self.fight_note(&thrower, line.replace("{}", &who) + &format!(": its type 0x{t:X} has the immune bit 0x4000: a hit of 0, no ProjImmune (rule 8)"));
                    self.hit_character(j, &thrower, 0, b.kind);
                } else {
                    self.fight_note(&thrower, line.replace("{}", &who) + &format!(": a hit of {d} ({parts}), {} (rule 8; fighting.md rules 16-33)", b.kind.name()));
                    self.hit_character(j, &thrower, d, b.kind);
                }
            }
            Touched::Object(n, t) => {
                if HAMMER_LEVER_OBJECTS.iter().any(|l| l.eq_ignore_ascii_case(&n)) {
                    self.fight_note(&thrower, line.replace("{}", &format!("object {n}")) + &format!(": no damage; {HAMMER_LEVER}'s script runs with {HAMMER_LEVER} ConnorAction (rule 8)"));
                    self.hammer_calls.push(HAMMER_LEVER.to_string());
                } else if t & IMMUNE_TO_PROJECTILES != 0 {
                    self.fight_note(&thrower, line.replace("{}", &format!("object {n}")) + &format!(": its type 0x{t:X} has the immune bit 0x4000: a hit of 0, no ProjImmune (rule 8)"));
                } else if self.object_at(&n).is_some() {
                    self.fight_note(&thrower, line.replace("{}", &format!("object {n}")) + &format!(": a hit of {d} ({parts}), {} (rule 8)", b.kind.name()));
                    let taken = self.object_takes(&n, d, b.kind);
                    self.fight_note(&thrower, format!("{}: {taken}", b.name));
                } else {
                    self.fight_note(&thrower, line.replace("{}", &format!("object {n}")) + ": no script: nothing (how objects take damage: Q139 e)");
                }
            }
        }
    }
}

/// What a bolt touched (rule 30).
#[derive(Debug, Clone, PartialEq)]
enum Touched {
    Building(String),
    /// Name and collision type.
    Object(String, u32),
    Character(usize),
    Connor,
}

impl Fight {
    /// Rule 3: health, hand and ranged damage × the combat level's factor,
    /// cut to whole numbers (protection and experience are not scaled).
    /// Returns the log text.
    pub fn apply_combat_level(&mut self, factor: f32) -> String {
        let before = self.stats.0;
        for k in 0..3 {
            self.stats.0[k] = (self.stats.0[k] as f32 * factor) as i64;
        }
        format!("combat level x {factor} (rule 3): {before:?} -> {:?}", self.stats.0)
    }
}

impl crate::active::WorldState {
    /// `setCombatLevel 1|2|3` (rule 3; M5 5c-10): 0.66 / 1.0 / 1.5 for the
    /// characters that enter the world from now on, and the roll on
    /// characters. Returns the result for `$Console::Return`.
    pub(crate) fn set_combat_level(&mut self, words: &[String], at: &str) -> String {
        match words.get(1).and_then(|w| w.parse::<i64>().ok()).and_then(combat_factor) {
            Some(f) => {
                self.combat_level = f;
                self.fight_note("game", format!("{at}: setCombatLevel {}: factor {f} (rule 3)", words[1]));
                "0".into()
            }
            None => {
                self.note(format!("{at}: usage: setCombatLevel 1|2|3 (3 is hardest); nothing"));
                "1".into()
            }
        }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Fight settings with these stats and attack delay, nothing else.
    pub(crate) fn settings(stats: [i64; 5], attack_delay: f32) -> FightSettings {
        FightSettings {
            type_name: "goblin2".into(),
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
    fn the_combat_level_scales_health_and_damage_cut() {
        // Rule 3: 0.66 / 1.0 / 1.5; health, hand, ranged; cut toward zero.
        assert_eq!((combat_factor(1), combat_factor(2), combat_factor(3), combat_factor(4)), (Some(0.66), Some(1.0), Some(1.5), None));
        let mut f = Fight::new(settings([130, -3, 2, 15, 15], 1.0));
        f.apply_combat_level(1.5);
        assert_eq!(f.stats.0, [195, -4, 3, 15, 15]);
        let mut g = Fight::new(settings([15, 2, 2, 0, 3], 1.0));
        g.apply_combat_level(0.66);
        assert_eq!(g.stats.0, [9, 1, 1, 0, 3]);
    }

    #[test]
    fn the_shot_leads_a_moving_target() {
        // Rule 36: a target 8,000 off moving across at 2,000 a second.
        let (aim, t) = lead([0.0; 3], [8000.0, 0.0, 0.0], [0.0, 2000.0, 0.0], 8000.0);
        let t = t.unwrap();
        // 8,000² + (2,000 t)² = (8,000 t)²: t = 1.0328.
        assert!((t - 1.0328).abs() < 1e-3, "{t}");
        assert!((aim[1] - 2000.0 * t).abs() < 0.5);
        // Standing still: straight at it, 1 s.
        let (aim, t) = lead([0.0; 3], [8000.0, 0.0, 0.0], [0.0; 3], 8000.0);
        assert_eq!((aim, t), ([8000.0, 0.0, 0.0], Some(1.0)));
        // Running away faster than the bolt: no meeting, straight at it.
        assert_eq!(lead([0.0; 3], [8000.0, 0.0, 0.0], [9000.0, 0.0, 0.0], 8000.0).1, None);
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
