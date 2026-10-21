//! Connor's weapon modes (M5 5f-3): what is ready, and the requests that
//! change it.
//!
//! Rules from `docs/behaviour/ranged-weapons.md` (session A, 2026-10-20;
//! checked by the user in the original):
//!
//! - Words: the **weapon mode** is hand (0), ranged (1) or none (4), as
//!   `KQConner::setDoMode` numbers them; Connor starts with none.
//! - Rule 1: three things ask for a mode: a click on a weapon slot (the
//!   item's mode column: `SwordMode` hand, `BowMode` ranged, `HandMode`
//!   none, `NoModeChange` nothing), Delete (`Draw`), Alt (`Away`); all
//!   ignored while `handsOff`. Scripts ask too (rule 9).
//! - Rule 2, Delete: hand ready → ranged, only with a real ranged item;
//!   ranged ready → hand; nothing ready → the one put away last (hand at
//!   the start).
//! - Rule 3, Alt: whatever is ready → none (remembering whether it was the
//!   ranged one); nothing ready: nothing.
//! - Rule 4: the mode already ready asked again becomes none. A request
//!   for ranged needs a ranged item; `NoRanged` counting is **not
//!   reproduced** ("Do not reproduce": a request for ranged needs a real
//!   ranged item, else the hand weapon is asked for).
//! - Rule 5: a request is taken only while Connor stands or moves (Wait,
//!   Move); otherwise it is dropped. The viewer knows his state and
//!   decides that ([`WorldState::take_weapon_request`]'s `free`).
//! - Rule 9: `KQConner::setDoMode <0|1|4>`: if the mode differs, he is put
//!   into standing and the same request is made (the full swap plays).

use crate::active::WorldState;

/// What Connor has ready (rule 1's words).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum WeaponMode {
    Hand,
    Ranged,
    #[default]
    None,
}

impl WeaponMode {
    /// `KQConner::setDoMode` / `getDoMode`'s number.
    pub fn number(self) -> i64 {
        match self {
            WeaponMode::Hand => 0,
            WeaponMode::Ranged => 1,
            WeaponMode::None => 4,
        }
    }

    pub fn from_number(n: i64) -> Option<Self> {
        match n {
            0 => Some(WeaponMode::Hand),
            1 => Some(WeaponMode::Ranged),
            4 => Some(WeaponMode::None),
            _ => None,
        }
    }

    /// An item's mode column (`Invinfo.itl`; rule 1): `None` for
    /// `NoModeChange` and anything else.
    pub fn from_column(column: &str) -> Option<Self> {
        match column.trim() {
            c if c.eq_ignore_ascii_case("SwordMode") => Some(WeaponMode::Hand),
            c if c.eq_ignore_ascii_case("BowMode") => Some(WeaponMode::Ranged),
            c if c.eq_ignore_ascii_case("HandMode") => Some(WeaponMode::None),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            WeaponMode::Hand => "hand",
            WeaponMode::Ranged => "ranged",
            WeaponMode::None => "none",
        }
    }
}

/// What asked for a new mode (rule 1).
#[derive(Debug, Clone, PartialEq)]
pub enum Ask {
    /// A left click on a weapon slot: the item's mode column.
    Slot { item: String, column: String },
    /// Delete (`Connor.sae`: `Draw`).
    Draw,
    /// Alt (`Away`).
    Away,
    /// `KQConner::setDoMode <n>` (rule 9).
    Script(WeaponMode),
}

/// A request waiting for the viewer, which knows Connor's state (rule 5).
#[derive(Debug, Clone, PartialEq)]
pub struct Request {
    pub ask: Ask,
    /// For the log: the key, slot or script line.
    pub how: String,
}

/// What the rules make of a request, before the state test of rule 5.
#[derive(Debug, Clone, PartialEq)]
pub enum Decision {
    /// Asks for this mode (different from the one ready).
    Asks(WeaponMode, String),
    /// Nothing happens, and why.
    Nothing(String),
}

/// Rules 2–4 (and the Do instead of rule 4): what `ask` asks for, with
/// `ready` now, a real ranged item carried or not, and whether the last
/// weapon put away was the ranged one.
pub fn decide(ask: &Ask, ready: WeaponMode, real_ranged: bool, last_away_ranged: bool) -> Decision {
    use WeaponMode::*;
    let wanted = match ask {
        Ask::Slot { column, .. } => match WeaponMode::from_column(column) {
            Some(m) => (m, format!("the slot's mode {column}")),
            Option::None => return Decision::Nothing(format!("the slot's mode {column} asks nothing (rule 1)")),
        },
        Ask::Draw => match ready {
            Hand if real_ranged => (Ranged, "Delete with the hand weapon ready: the ranged one (rule 2)".into()),
            Hand => return Decision::Nothing("Delete with the hand weapon ready and no real ranged weapon: nothing (rule 2)".into()),
            Ranged => (Hand, "Delete with the ranged weapon ready: the hand one (rule 2)".into()),
            None if last_away_ranged => (Ranged, "Delete with nothing ready: the ranged one, put away last (rule 2)".into()),
            None => (Hand, "Delete with nothing ready: the hand one (put away last, or the start) (rule 2)".into()),
        },
        Ask::Away => match ready {
            None => return Decision::Nothing("Alt with nothing ready: nothing (rule 3)".into()),
            _ => (None, "Alt: put away (rule 3)".into()),
        },
        Ask::Script(m) if *m == ready => return Decision::Nothing(format!("setDoMode {}: already {} (rule 9)", m.number(), m.name())),
        Ask::Script(m) => (*m, format!("setDoMode {} (rule 9)", m.number())),
    };
    let (mut mode, mut why) = wanted;
    // Rule 4: the mode already ready becomes none.
    if mode == ready && mode != None {
        why = format!("{why}; already ready: put away (rule 4)");
        mode = None;
    }
    // Rule 4 with the Do instead: ranged needs a real ranged item.
    if mode == Ranged && !real_ranged {
        why = format!("{why}; no real ranged weapon: the hand one instead (rule 4, Do instead)");
        mode = if ready == Hand { return Decision::Nothing(format!("{why}; the hand one is ready: nothing")) } else { Hand };
    }
    if mode == ready {
        return Decision::Nothing(format!("{why}: nothing to change"));
    }
    Decision::Asks(mode, why)
}

/// Rule 15: the talk pick's types (bits 0x40, 0x20, 0x08).
pub const TALK_PICK: u32 = 0x40 | 0x20 | 0x08;
/// Rule 16: the aim pick's types (bits 0x40, 0x08).
pub const AIM_PICK: u32 = 0x40 | 0x08;
/// Rule 32: the swing pick's types (bits 0x40, 0x20).
pub const SWING_PICK: u32 = 0x40 | 0x20;
/// Rule 15: a character whose type has this bit gets the plain click.
pub const PLAIN_CLICK_BIT: u32 = 0x8000;
/// Rule 15: an object of this name gets the plain click.
pub const ICEWATER: &str = "icewater";
/// Rule 18: the shot is never steeper than 1 rad from the vertical.
pub const STEEPEST: f32 = 1.0;
/// Rule 20: more than this off his facing, the shot waits for the turn.
pub const SHOT_WAIT_ANGLE: f32 = std::f32::consts::FRAC_PI_4;
/// Rule 20: a waiting shot is asked when the turn has this many updates
/// or fewer to go.
pub const SHOT_TURN_UPDATES: f32 = 3.0;

/// What the click line met first among the things a mask covers (rules
/// 15, 16, 32).
#[derive(Debug, Clone, PartialEq)]
pub struct MaskHit {
    pub name: String,
    /// A character (its index in `anims`).
    pub character: Option<usize>,
    pub building: bool,
    pub collision_type: u32,
    /// Where the line meets it.
    pub point: [f32; 3],
}

/// Rule 18: the aim point moved out so that the line from his feet + the
/// offset's height to it is at most 1 rad from the vertical: with *h* the
/// height difference and *d* the level distance, *d* < tan(1) × |*h*| →
/// *d* = tan(1) × |*h*|, same direction and height. Returns the point and
/// whether it moved.
pub fn steepness_limit(feet: [f32; 3], offset_z: f32, aim: [f32; 3]) -> ([f32; 3], bool) {
    let h = aim[2] - (feet[2] + offset_z);
    let (dx, dy) = (aim[0] - feet[0], aim[1] - feet[1]);
    let d = (dx * dx + dy * dy).sqrt();
    let least = STEEPEST.tan() * h.abs();
    if d >= least {
        return (aim, false);
    }
    // Straight above or below: out along his facing is not known here;
    // the caller's direction is kept (none: +x, ours).
    let (ux, uy) = if d > 1e-3 { (dx / d, dy / d) } else { (1.0, 0.0) };
    ([feet[0] + ux * least, feet[1] + uy * least, aim[2]], true)
}

/// What changed when a request was taken, for the viewer's swap (rule 6).
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub from: WeaponMode,
    pub to: WeaponMode,
    /// The item ready before and after (`None` with nothing ready).
    pub from_item: Option<String>,
    pub to_item: Option<String>,
    /// A script asked: he is put into standing first (rule 9).
    pub script: bool,
}

/// `INVITEM_Fists`, his hand weapon when he carries no other.
pub const FISTS: &str = "INVITEM_Fists";
/// `INVITEM_NoRanged`, the ranged slot's placeholder: no real weapon.
pub const NO_RANGED: &str = "INVITEM_NoRanged";

impl WorldState {
    /// The ranged slot item (rules 1, 21: the one `LongWeapon` he carries),
    /// its type name; `None` if he carries none or only `NoRanged`.
    pub fn ranged_weapon(&self) -> Option<String> {
        self.items
            .objects
            .iter()
            .filter(|o| o.carried_by(crate::talk::CONNOR))
            .find(|o| !o.kind.eq_ignore_ascii_case(NO_RANGED) && self.items.types.find(&o.kind).is_some_and(|t| t.category == "LongWeapon"))
            .map(|o| o.kind.clone())
    }

    /// Rules 1, 9: a request waits for the viewer.
    pub fn ask_weapon(&mut self, ask: Ask, how: &str) {
        self.weapon_requests.push(Request { ask, how: how.to_string() });
    }

    /// Rules 2–5, 9: the viewer takes a waiting request. `free`: Connor
    /// stands or moves and no swap plays (`Err`: his state, for the log);
    /// `hands_off`: requests are ignored (rule 1). A script's request
    /// needs no state (rule 9: it puts him into standing). Returns the
    /// change, if any; every request is logged in `fight.log`.
    pub fn take_weapon_request(&mut self, r: &Request, free: Result<(), String>, hands_off: bool) -> Option<Change> {
        let script = matches!(r.ask, Ask::Script(_));
        if hands_off && !script {
            self.fight_note("Connor", format!("weapon request ({}): ignored, handsOff (rule 1)", r.how));
            return None;
        }
        let ready = self.weapon_mode;
        let real_ranged = self.ranged_weapon().is_some();
        let (to, why) = match decide(&r.ask, ready, real_ranged, self.last_away_ranged) {
            Decision::Nothing(why) => {
                self.fight_note("Connor", format!("weapon request ({}): {why}", r.how));
                return None;
            }
            Decision::Asks(m, why) => (m, why),
        };
        if let (Err(state), false) = (&free, script) {
            self.fight_note("Connor", format!("weapon request ({}): {why}: asks {}; dropped, he is not standing or moving ({state}) (rule 5)", r.how, to.name()));
            return None;
        }
        let from_item = self.weapon_ready.clone();
        let to_item = match to {
            WeaponMode::Hand => Some(crate::connor_inventory::hand_weapon(self).unwrap_or_else(|| FISTS.to_string())),
            WeaponMode::Ranged => self.ranged_weapon(),
            WeaponMode::None => None,
        };
        if to == WeaponMode::None {
            self.last_away_ranged = ready == WeaponMode::Ranged;
        }
        self.weapon_mode = to;
        self.weapon_ready = to_item.clone();
        self.fight_note("Connor", format!("weapon request ({}): {why}: taken, {} ({from_item:?}) -> {} ({to_item:?}) (rules 4, 5)", r.how, ready.name(), to.name()));
        Some(Change { from: ready, to, from_item, to_item, script })
    }

    /// `Connor.kq`'s fight settings (M5 5c-4); his projectile and shoot
    /// offset start as its `projectileFile` and `shootOffsetX/Y/Z`, then
    /// a ranged item he already carries replaces them (rule 21).
    pub fn set_connor_fight(&mut self, f: Option<crate::fight::ConnorFight>) {
        self.connor_projectile = f.as_ref().and_then(|f| f.start_projectile.clone());
        self.connor_shoot_offset = f.as_ref().map_or([0.0; 3], |f| f.start_offset);
        self.connor_fight = f;
        if let Some(kind) = self.ranged_weapon() {
            let line = self.take_ranged_item(&kind);
            self.fight_note("Connor", format!("carried at the start: {line}"));
        }
    }

    /// Rules 17, 21 (M5 5f-4): a `LongWeapon` goes into his inventory: its
    /// projectile file (the item list's last column; `NoRanged` has none:
    /// no bolt) becomes his, and its line of `Connor.kq`'s `shootOffsets`
    /// (any capitals; `NoRanged` looks for `INVITEM_All`, which has none)
    /// his shoot offset; no line: the offset stays. Returns the log text.
    pub fn take_ranged_item(&mut self, kind: &str) -> String {
        let real = !kind.eq_ignore_ascii_case(NO_RANGED);
        self.connor_projectile = if real { self.items.kind(kind).and_then(|t| t.extra.clone()) } else { None };
        let name = if real { kind } else { "INVITEM_All" };
        let found = self.connor_fight.as_ref().and_then(|f| f.shoot_offsets.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, o)| *o));
        if let Some(o) = found {
            self.connor_shoot_offset = o;
        }
        let o = self.connor_shoot_offset;
        format!(
            "{kind}: shoots {:?}; shoot offset ({:.0}, {:.0}, {:.0}){} (ranged-weapons.md rules 17, 21)",
            self.connor_projectile,
            o[0],
            o[1],
            o[2],
            if found.is_some() { format!(" from its shootOffsets line ({name})") } else { format!(" kept (no shootOffsets line for {name})") }
        )
    }

    /// Rules 15, 16, 32: the first thing the click line from `origin`
    /// along `dir` (length 1) meets among those whose collision type
    /// shares a bit with `mask`: characters (their `BOUNDS.KQ` boxes),
    /// objects (`objects`: every object's box; switched-off ones left out)
    /// and buildings (`buildings`, in the world's order: their type is
    /// `self.buildings`').
    pub fn mask_pick(&self, origin: [f32; 3], dir: [f32; 3], mask: u32, objects: &[crate::collide::BoxCollider], buildings: &[crate::collide::BuildingCollider]) -> Option<MaskHit> {
        let mut best: Option<(f32, MaskHit)> = None;
        let mut keep = |t: f32, h: MaskHit| {
            if best.as_ref().is_none_or(|(b, _)| t < *b) {
                best = Some((t, h));
            }
        };
        let at = |t: f32| -> [f32; 3] { std::array::from_fn(|k| origin[k] + dir[k] * t) };
        for (j, a) in self.anims.iter().enumerate() {
            let Some(c) = a.character.as_deref() else { continue };
            if a.gone || c.unborn || c.collision_type & mask == 0 || self.is_switched_off(&a.player.name) {
                continue;
            }
            let Some((min, max)) = c.bounds else { continue };
            if let Some(t) = crate::click::ray_box(origin, dir, &a.player.placement(), min, max) {
                keep(t, MaskHit { name: a.player.name.clone(), character: Some(j), building: false, collision_type: c.collision_type, point: at(t) });
            }
        }
        for o in objects.iter().filter(|o| o.kind.collision_type & mask != 0 && !self.is_switched_off(&o.name)) {
            if self.character_index(&o.name).is_some() {
                continue;
            }
            if let Some(t) = crate::click::ray_box(origin, dir, &o.placement, o.min, o.max) {
                keep(t, MaskHit { name: o.name.clone(), character: None, building: false, collision_type: o.kind.collision_type, point: at(t) });
            }
        }
        let far = at(crate::active::CLICK_LINE_LENGTH);
        for (b, info) in buildings.iter().zip(&self.buildings) {
            if info.collision_type & mask == 0 || self.is_switched_off(&info.name) {
                continue;
            }
            if let Some(p) = b.line_hit(origin, far) {
                let t = (0..3).map(|k| (p[k] - origin[k]).powi(2)).sum::<f32>().sqrt();
                keep(t, MaskHit { name: b.name.clone(), character: None, building: true, collision_type: info.collision_type, point: p });
            }
        }
        best.map(|(_, h)| h)
    }

    /// Rules 22–24 (M5 5f-5): Connor's bolt leaves at his `do` cue. `feet`,
    /// `rotation`: his position and his full turn (lean included); `part`:
    /// the bolt file's model part in the frame playing (found: it is also
    /// kept as his shoot offset), else his shoot offset; `aim`: the aim
    /// point (rules 16, 18); his ranged damage now.
    pub fn connor_shoot(&mut self, feet: [f32; 3], rotation: [[f32; 3]; 3], part: Option<([f32; 3], String)>, aim: [f32; 3], movement: &str) {
        let Some(file) = self.connor_projectile.clone() else {
            self.fight_note("Connor", format!("do ({movement}): he has no bolt file (NoRanged): no shot (rule 21)"));
            return;
        };
        let Some(p) = self.projectiles.get(&file.to_ascii_lowercase()).cloned() else {
            self.fight_note("Connor", format!("do ({movement}): projectile file {file} not read: no shot"));
            return;
        };
        let (offset, how) = match part {
            Some((o, how)) => {
                self.connor_shoot_offset = o;
                (o, format!("{how} (kept as his shoot offset)"))
            }
            None => (self.connor_shoot_offset, "no such part in the frame: his shoot offset".to_string()),
        };
        let r = rotation;
        let from: [f32; 3] = std::array::from_fn(|k| feet[k] + offset[0] * r[0][k] + offset[1] * r[1][k] + offset[2] * r[2][k]);
        let damage = self.connor_stats.map_or(0, |c| c.stats.0[2]);
        let why = format!("do ({movement}): start offset ({:.0}, {:.0}, {:.0}): {how} (rule 24); aim point ({:.0}, {:.0}, {:.0}), no lead (rule 25); ranged damage {damage} (rule 22)", offset[0], offset[1], offset[2], aim[0], aim[1], aim[2]);
        self.launch_bolt(crate::talk::CONNOR, &file, &p, from, aim, damage, &why);
    }

    /// A weapon readied or put away outside a request (the `--swing` test,
    /// a conversation's put-away): the mode follows the item.
    pub fn set_weapon(&mut self, mode: WeaponMode, item: Option<String>) {
        if mode == WeaponMode::None && self.weapon_mode != WeaponMode::None {
            self.last_away_ranged = self.weapon_mode == WeaponMode::Ranged;
        }
        self.weapon_mode = mode;
        self.weapon_ready = item;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use WeaponMode::*;

    fn asks(d: Decision) -> Option<WeaponMode> {
        match d {
            Decision::Asks(m, _) => Some(m),
            Decision::Nothing(_) => Option::None,
        }
    }

    #[test]
    fn delete_swaps_hand_and_ranged_and_draws_the_last_put_away() {
        // Rule 2.
        assert_eq!(asks(decide(&Ask::Draw, Hand, true, false)), Some(Ranged));
        assert_eq!(asks(decide(&Ask::Draw, Hand, false, false)), Option::None, "no real ranged weapon: nothing");
        assert_eq!(asks(decide(&Ask::Draw, Ranged, true, false)), Some(Hand));
        assert_eq!(asks(decide(&Ask::Draw, None, true, true)), Some(Ranged));
        assert_eq!(asks(decide(&Ask::Draw, None, true, false)), Some(Hand), "the start: his fists");
        // The Do instead: put away a bow, then lost it: the hand weapon.
        assert_eq!(asks(decide(&Ask::Draw, None, false, true)), Some(Hand));
    }

    #[test]
    fn alt_puts_away_and_slots_toggle() {
        // Rules 3, 4.
        assert_eq!(asks(decide(&Ask::Away, Ranged, true, false)), Some(None));
        assert_eq!(asks(decide(&Ask::Away, None, true, false)), Option::None);
        let slot = |c: &str| Ask::Slot { item: "x".into(), column: c.into() };
        assert_eq!(asks(decide(&slot("SwordMode"), None, false, false)), Some(Hand));
        assert_eq!(asks(decide(&slot("SwordMode"), Hand, false, false)), Some(None), "the ready slot again: put away");
        assert_eq!(asks(decide(&slot("BowMode"), Hand, true, false)), Some(Ranged));
        assert_eq!(asks(decide(&slot("BowMode"), Ranged, true, false)), Some(None));
        assert_eq!(asks(decide(&slot("BowMode"), None, false, false)), Some(Hand), "NoRanged's slot: Do instead");
        assert_eq!(asks(decide(&slot("NoModeChange"), None, true, false)), Option::None);
        assert_eq!(asks(decide(&slot("HandMode"), Hand, true, false)), Some(None));
    }

    #[test]
    fn the_aim_is_never_steeper_than_one_radian_from_the_vertical() {
        // Rule 18: a click on the ground at his feet: out to 1.557 × h.
        let feet = [0.0, 0.0, 0.0];
        let (p, moved) = steepness_limit(feet, 1065.0, [0.0, -100.0, 0.0]);
        assert!(moved);
        assert!((p[1] + 1.5574 * 1065.0).abs() < 1.0 && p[0].abs() < 1e-3 && p[2] == 0.0, "{p:?}");
        // The bolt then goes down at about 33° (90° − 57.3°).
        let deg = (1065.0f32 / (1.5574 * 1065.0)).atan().to_degrees();
        assert!((deg - 32.7).abs() < 0.1, "{deg}");
        // Far enough: unchanged.
        assert_eq!(steepness_limit(feet, 1065.0, [0.0, -5000.0, 0.0]), ([0.0, -5000.0, 0.0], false));
    }

    #[test]
    fn scripts_ask_only_for_another_mode() {
        // Rule 9.
        assert_eq!(asks(decide(&Ask::Script(Hand), None, false, false)), Some(Hand));
        assert_eq!(asks(decide(&Ask::Script(None), Hand, false, false)), Some(None));
        assert_eq!(asks(decide(&Ask::Script(Hand), Hand, false, false)), Option::None);
        assert_eq!((WeaponMode::from_number(1), Ranged.number(), None.number()), (Some(Ranged), 1, 4));
    }
}
