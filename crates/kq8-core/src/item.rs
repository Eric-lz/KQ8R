//! Item objects and inventories (M3 step 3e-2): coins, crystals,
//! mushrooms... lying in the world or carried by someone.
//!
//! Rules from `docs/behaviour/items.md` (traced unless marked):
//!
//! - An item object has a type (a line of `Invinfo.itl`,
//!   `kq8_data::items`), a quantity (one object can be a stack of 200
//!   coins) and a third number, default −1 (rule 3, meaning not traced).
//! - Every item object is clickable (collision type 0x220), has mask 0x809
//!   and `doNotPointAt` on (rule 4). By the pair rule it never collides
//!   with Connor (type 4, mask 0x281B).
//! - A placed item stays where it was put (rule 5).
//! - `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` makes a
//!   new item object: `world` → placed there (z −1 / −2 = floor / surface
//!   height, `objects.md` Q17 rule 2), anyone else → carried by them
//!   (rule 9).
//! - `KQMonster::checkInventory <who> <type>`: is there one (rule 10).
//!
//! - A new item object is named `NoName<n>` from one counter that starts
//!   at 0 (rule 9, Q60). The counter is the game's for every nameless
//!   object; only items are nameless here so far, and there is no travel
//!   between worlds or saving yet, so it lives with the item list (M6:
//!   game-wide and saved). The original names an object the first time
//!   the name is needed; ours are named when made, which gives the same
//!   order because every item's name is used at once (`NewInvItem`).
//!
//! Our stand-in (not in the note): an item is clicked by the box of its type's shape in
//! `BOUNDS.KQ` (shape name lower-cased; our reading of `chests.md`'s
//! crystal check), which the caller supplies.

use std::collections::BTreeMap;

use kq8_data::items::{ItemType, ItemTypes};

use crate::collide::Placement;

/// Rule 4.
pub const ITEM_TYPE: u32 = 0x220;
pub const ITEM_MASK: u32 = 0x809;

/// What an object accepts from Connor's hand (M3 step 3g-3, `clicking.md`
/// rules 13-16, traced): an "accept everything" switch and a list of item
/// types (canonical names from `Invinfo.itl`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Accepts {
    pub all: bool,
    pub types: Vec<String>,
}

impl Accepts {
    /// Reads an `acceptsInventory` / `KQObject::setAccepts` value and
    /// **adds** it (rules 14, 16): split at commas; `all` / `INVITEM_All`
    /// (any capitals) turns the switch on; `none` ends the list; any other
    /// word is a type name (rule 2 lookup). Returns the words that name no
    /// type, which are skipped (the original would crash, "Do not
    /// reproduce").
    ///
    /// The original honours `all` only as the first word and then ignores
    /// the rest; later it would crash. We treat it as "accept everything"
    /// anywhere ("Do not reproduce"); no data has it later.
    pub fn read(&mut self, value: &str, types: &ItemTypes) -> Vec<String> {
        let mut unknown = Vec::new();
        for word in value.split(',').map(str::trim) {
            if word.eq_ignore_ascii_case("all") || word.eq_ignore_ascii_case("INVITEM_All") {
                self.all = true;
            } else if word.eq_ignore_ascii_case("none") {
                break;
            } else if let Some(t) = types.find(word) {
                if !self.types.contains(&t.name) {
                    self.types.push(t.name.clone());
                }
            } else {
                unknown.push(word.to_string());
            }
        }
        unknown
    }

    /// Whether an item of type `kind` is accepted (rule 13): the switch,
    /// or its type in the list.
    pub fn takes(&self, kind: &str, types: &ItemTypes) -> bool {
        self.all || types.find(kind).is_some_and(|t| self.types.contains(&t.name))
    }
}

/// Where an item object is.
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    /// Lying in the world.
    World { position: [f32; 3], angles: [f32; 3] },
    /// In someone's inventory (their name as the script gave it).
    Carried(String),
    /// Deleted (`deleteObject`): nowhere.
    Gone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemObject {
    pub name: String,
    /// The type's `IDName`, e.g. `INVITEM_Crystal`.
    pub kind: String,
    pub quantity: i64,
    /// Rule 3: default −1, meaning not traced (M4).
    pub third: i64,
    pub place: Place,
    /// `KQObject::setHidden`: not drawn, can't be clicked (`objects.md`
    /// "Hidden objects").
    pub hidden: bool,
    /// Runs after a pickup (rule 7); `KQObject::setScript`.
    pub script: Option<String>,
    /// Rule 4: on; `KQObject::setDontLook`.
    pub do_not_point_at: bool,
    /// When it last went into someone's inventory: a number counting up
    /// (0: never); the quest strip lists in this order (M4 4c-3,
    /// `inventory.md` rule 5).
    pub added: u64,
}

impl ItemObject {
    /// Where it lies, if in the world.
    pub fn placement(&self) -> Option<Placement> {
        match &self.place {
            Place::World { position, angles } => Some(Placement { rotation: kq8_data::objects::rotation(*angles), position: *position }),
            _ => None,
        }
    }

    pub fn in_world(&self) -> bool {
        matches!(self.place, Place::World { .. })
    }

    pub fn carried_by(&self, who: &str) -> bool {
        matches!(&self.place, Place::Carried(w) if w.eq_ignore_ascii_case(who))
    }
}

/// A `KQMonster::inventory` or `give` line, read (rule 9).
#[derive(Debug, Clone, PartialEq)]
pub struct Make {
    /// `world` or a character's name.
    pub who: String,
    pub kind: String,
    /// Default 1.
    pub quantity: i64,
    /// As written (−1 / −2 not yet replaced).
    pub position: Option<[f32; 3]>,
    /// Missing ones 0.
    pub angles: [f32; 3],
}

impl Make {
    pub fn in_world(&self) -> bool {
        self.who.eq_ignore_ascii_case("world")
    }
}

/// Reads `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` or
/// `give <type> [qty]` (words expanded, `words[0]` the command). `Err`
/// with a reason for the log. A world item needs x y z (our choice: every
/// such line in the data gives one).
pub fn read_make(words: &[String]) -> Result<Make, String> {
    let line = words.join(" ");
    let (who, rest) = if words[0].eq_ignore_ascii_case("give") {
        ("Connor".to_string(), &words[1..])
    } else {
        match words.get(1) {
            Some(w) => (w.clone(), &words[2..]),
            None => return Err("usage: KQMonster::inventory <who> <type> [quantity] [x y z [rx ry rz]]".into()),
        }
    };
    let Some(kind) = rest.first() else { return Err(format!("usage: {} needs an item type: '{line}'", words[0])) };
    let numbers = &rest[1..];
    if let Some(bad) = numbers.iter().find(|w| !crate::script::looks_like_number(w)) {
        return Err(format!("{line}: '{bad}' is not a number"));
    }
    let n: Vec<f32> = numbers.iter().map(|w| crate::script::number_value(w) as f32).collect();
    let position = match n.len() {
        0 | 1 => None,
        2 | 3 => return Err(format!("{line}: a position needs x y z")),
        _ => Some([n[1], n[2], n[3]]),
    };
    let make = Make {
        who,
        kind: kind.clone(),
        quantity: n.first().map_or(1, |q| *q as i64),
        position,
        angles: [n.get(4), n.get(5), n.get(6)].map(|a| a.copied().unwrap_or(0.0)),
    };
    if make.in_world() && make.position.is_none() {
        return Err(format!("{line}: no position for a world item"));
    }
    Ok(make)
}

/// Every item object of the world and the type list.
#[derive(Debug, Clone, Default)]
pub struct Items {
    pub types: ItemTypes,
    /// Each shape's click box (own space) by lower-case shape name.
    boxes: BTreeMap<String, ([f32; 3], [f32; 3])>,
    /// Every item object made so far, also carried and gone ones.
    pub objects: Vec<ItemObject>,
    /// The next `NoName<n>` number (rule 9).
    next_no_name: usize,
    /// The last [`ItemObject::added`] number given.
    last_added: u64,
}

impl Items {
    /// `boxes`: (shape name, min, max) from `BOUNDS.KQ`.
    pub fn new(types: ItemTypes, boxes: impl IntoIterator<Item = (String, [f32; 3], [f32; 3])>) -> Self {
        let boxes = boxes.into_iter().map(|(s, a, b)| (s.to_ascii_lowercase(), (a, b))).collect();
        Items { types, boxes, objects: Vec::new(), next_no_name: 0, last_added: 0 }
    }

    /// The next order number for an item made carried (0 if not).
    fn next_added(&mut self, place: &Place) -> u64 {
        if matches!(place, Place::Carried(_)) {
            self.last_added += 1;
            self.last_added
        } else {
            0
        }
    }

    /// Item `name` has just gone into an inventory (M4 4c-3).
    pub fn mark_added(&mut self, name: &str) {
        if let Some(i) = self.index(name) {
            self.last_added += 1;
            self.objects[i].added = self.last_added;
        }
    }

    /// The type a script means by `name` (rule 2).
    pub fn kind(&self, name: &str) -> Option<&ItemType> {
        self.types.find(name)
    }

    /// The click box of an item object's type, if its shape has one.
    pub fn click_box(&self, item: &ItemObject) -> Option<([f32; 3], [f32; 3])> {
        let t = self.types.find(&item.kind)?;
        self.boxes.get(&t.shape.to_ascii_lowercase()).copied()
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name.eq_ignore_ascii_case(name))
    }

    pub fn get(&self, name: &str) -> Option<&ItemObject> {
        self.index(name).map(|i| &self.objects[i])
    }

    /// Makes a new item object of type `kind` (any spelling rule 2
    /// allows). `Err` if there is no such type.
    pub fn make(&mut self, kind: &str, quantity: i64, place: Place) -> Result<&ItemObject, String> {
        let t = self.types.find(kind).ok_or_else(|| format!("no item type '{kind}' (looked for {})", kq8_data::items::full_name(kind)))?;
        let kind = t.name.clone();
        let name = format!("NoName{}", self.next_no_name);
        self.next_no_name += 1;
        let added = self.next_added(&place);
        self.objects.push(ItemObject { name, kind, quantity, third: -1, place, hidden: false, script: None, do_not_point_at: true, added });
        Ok(self.objects.last().expect("just pushed"))
    }

    /// An item object with a name of its own (`Connor.kq`
    /// `[ConnerInventory]` sections, M4 4c-2): the `NoName` counter is not
    /// used. `Err` if there is no such type or the name is taken.
    pub fn add_named(&mut self, name: &str, kind: &str, quantity: i64, third: i64, place: Place) -> Result<&ItemObject, String> {
        let t = self.types.find(kind).ok_or_else(|| format!("no item type '{kind}' (looked for {})", kq8_data::items::full_name(kind)))?;
        let kind = t.name.clone();
        if self.index(name).is_some() {
            return Err(format!("an item named '{name}' exists already"));
        }
        let added = self.next_added(&place);
        self.objects.push(ItemObject { name: name.to_string(), kind, quantity, third, place, hidden: false, script: None, do_not_point_at: true, added });
        Ok(self.objects.last().expect("just pushed"))
    }

    /// The first item object of type `kind` that `who` carries, or (for
    /// `world`, any capitals) that lies in the world (rule 10).
    pub fn find(&self, who: &str, kind: &str) -> Option<&ItemObject> {
        let t = self.types.find(kind)?;
        let world = who.eq_ignore_ascii_case("world");
        self.objects
            .iter()
            .filter(|o| o.kind == t.name)
            .find(|o| if world { o.in_world() } else { o.carried_by(who) })
    }

    /// One line about item object `name` for the logs: name, type,
    /// quantity, where it is.
    pub fn describe(&self, name: &str) -> String {
        let Some(o) = self.get(name) else { return format!("{name}: no such item") };
        let where_ = match &o.place {
            Place::World { position: p, angles: a } => {
                format!("in the world at ({:.0}, {:.0}, {:.0}) turned ({}, {}, {})", p[0], p[1], p[2], a[0], a[1], a[2])
            }
            Place::Carried(w) => format!("carried by {w} (now {} of them)", self.count(w, &o.kind)),
            Place::Gone => "gone".into(),
        };
        let extra = match (&o.script, o.hidden) {
            (None, false) => String::new(),
            (s, h) => format!(" [script {s:?}, hidden {h}]"),
        };
        format!("{}: {} x{} {where_}{extra}", o.name, o.kind, o.quantity)
    }

    /// How many of type `kind` `who` carries (quantities added up; for the
    /// log).
    pub fn count(&self, who: &str, kind: &str) -> i64 {
        self.objects.iter().filter(|o| o.kind.eq_ignore_ascii_case(kind) && o.carried_by(who)).map(|o| o.quantity).sum()
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// A two-type list in the file's layout (made-up values).
    pub fn types() -> ItemTypes {
        let list = b"[InventoryItemTypeList]\nnumItems=2\n\
item0=\tKQInventoryItemType\tINVITEM_Crystal\t3020\t50\t0\t0\t0\t0\t1\tHealthItem\tIDBMP_X\t13 0 0 1\tNoModeChange\t-1\t1\tpotnland.aud\tenergy.aud\thealing.aud\tcrystl.KQS\tnone\t250\t200\n\
item1=\tKQInventoryItemType\tINVITEM_SilverCoins\t4030\t0\t0\t0\t0\t0\t1\tMoney\tIDBMP_Y\t47 0 0 1\tNoModeChange\t0\t0\tnone\tcoins.aud\tcoins.aud\tpouch.kqs\tnone\t150\t256\n";
        ItemTypes::parse(list).0
    }

    pub fn items() -> Items {
        Items::new(types(), [("crystl.kqs".to_string(), [-60.0, -80.0, 0.0], [70.0, 80.0, 290.0])])
    }

    #[test]
    fn made_items_get_names_types_and_defaults() {
        let mut it = items();
        let at = Place::World { position: [100.0, 200.0, 300.0], angles: [0.0; 3] };
        let c = it.make("crystal", 3, at.clone()).unwrap().clone();
        assert_eq!((c.name.as_str(), c.kind.as_str(), c.quantity, c.third), ("NoName0", "INVITEM_Crystal", 3, -1));
        assert!(c.do_not_point_at && !c.hidden && c.script.is_none());
        let s = it.make("INVITEM_SilverCoins", 200, Place::Carried("Connor".into())).unwrap();
        assert_eq!(s.name, "NoName1");
        assert!(it.make("ladybell", 1, at).is_err());
        assert_eq!(it.objects.len(), 2);
    }

    #[test]
    fn find_looks_in_the_world_or_an_inventory() {
        let mut it = items();
        it.make("Crystal", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap();
        it.make("SilverCoins", 6, Place::Carried("Connor".into())).unwrap();
        it.make("SilverCoins", 4, Place::Carried("connor".into())).unwrap();
        assert_eq!(it.find("WORLD", "crystal").map(|o| o.name.as_str()), Some("NoName0"));
        assert!(it.find("Connor", "crystal").is_none());
        assert_eq!(it.find("Connor", "silverCoins").map(|o| o.name.as_str()), Some("NoName1"));
        assert!(it.find("world", "silverCoins").is_none());
        assert_eq!(it.count("Connor", "INVITEM_SilverCoins"), 10);
    }

    #[test]
    fn accept_lists_split_at_commas_and_compare_types() {
        let types = types();
        let mut a = Accepts::default();
        assert_eq!(a.read("crystal,INVITEM_SilverCoins", &types), Vec::<String>::new());
        assert_eq!(a.types, ["INVITEM_Crystal", "INVITEM_SilverCoins"]);
        assert!(a.takes("INVITEM_Crystal", &types) && a.takes("silvercoins", &types) && !a.all);
        // `none` ends the list; unknown names are skipped and returned.
        let mut b = Accepts::default();
        assert_eq!(b.read("INVITEM_LadyBell,Crystal,NONE,SilverCoins", &types), ["INVITEM_LadyBell"]);
        assert_eq!(b.types, ["INVITEM_Crystal"]);
        assert!(!b.takes("INVITEM_SilverCoins", &types) && !b.takes("INVITEM_LadyBell", &types));
        // `INVITEM_All` / `all`, any capitals: everything.
        let mut c = Accepts::default();
        c.read("invitem_all", &types);
        assert!(c.all && c.takes("INVITEM_LadyBell", &types));
        // Reading again adds; it never empties or switches off.
        b.read("SilverCoins", &types);
        b.read("none", &types);
        assert_eq!(b.types, ["INVITEM_Crystal", "INVITEM_SilverCoins"]);
    }

    #[test]
    fn the_click_box_comes_from_the_types_shape() {
        let mut it = items();
        let c = it.make("Crystal", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap().clone();
        assert_eq!(it.click_box(&c), Some(([-60.0, -80.0, 0.0], [70.0, 80.0, 290.0])), "crystl.KQS lower-cased");
        let s = it.make("SilverCoins", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap().clone();
        assert_eq!(it.click_box(&s), None);
    }
}
