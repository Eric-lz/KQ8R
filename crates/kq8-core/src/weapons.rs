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
    fn scripts_ask_only_for_another_mode() {
        // Rule 9.
        assert_eq!(asks(decide(&Ask::Script(Hand), None, false, false)), Some(Hand));
        assert_eq!(asks(decide(&Ask::Script(None), Hand, false, false)), Some(None));
        assert_eq!(asks(decide(&Ask::Script(Hand), Hand, false, false)), Option::None);
        assert_eq!((WeaponMode::from_number(1), Ranged.number(), None.number()), (Some(Ranged), 1, 4));
    }
}
