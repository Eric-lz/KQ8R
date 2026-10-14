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
//! Our stand-ins (not in the note): the name of a new item object is
//! `<type without INVITEM_>.<n>`, n counting from 1 in the order items are
//! made (Q60); an item is clicked by the box of its type's shape in
//! `BOUNDS.KQ` (shape name lower-cased; our reading of `chests.md`'s
//! crystal check), which the caller supplies.

use std::collections::BTreeMap;

use kq8_data::items::{ItemType, ItemTypes, PREFIX};

use crate::collide::Placement;

/// Rule 4.
pub const ITEM_TYPE: u32 = 0x220;
pub const ITEM_MASK: u32 = 0x809;

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

/// Every item object of the world and the type list.
#[derive(Debug, Clone, Default)]
pub struct Items {
    pub types: ItemTypes,
    /// Each shape's click box (own space) by lower-case shape name.
    boxes: BTreeMap<String, ([f32; 3], [f32; 3])>,
    /// Every item object made so far, also carried and gone ones.
    pub objects: Vec<ItemObject>,
    /// Item objects made so far (for the names).
    made: usize,
}

impl Items {
    /// `boxes`: (shape name, min, max) from `BOUNDS.KQ`.
    pub fn new(types: ItemTypes, boxes: impl IntoIterator<Item = (String, [f32; 3], [f32; 3])>) -> Self {
        let boxes = boxes.into_iter().map(|(s, a, b)| (s.to_ascii_lowercase(), (a, b))).collect();
        Items { types, boxes, objects: Vec::new(), made: 0 }
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
        self.made += 1;
        // Q60 stand-in.
        let short = kind.strip_prefix(PREFIX).unwrap_or(&kind);
        let name = format!("{short}.{}", self.made);
        self.objects.push(ItemObject { name, kind, quantity, third: -1, place, hidden: false, script: None, do_not_point_at: true });
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
        assert_eq!((c.name.as_str(), c.kind.as_str(), c.quantity, c.third), ("Crystal.1", "INVITEM_Crystal", 3, -1));
        assert!(c.do_not_point_at && !c.hidden && c.script.is_none());
        let s = it.make("INVITEM_SilverCoins", 200, Place::Carried("Connor".into())).unwrap();
        assert_eq!(s.name, "SilverCoins.2");
        assert!(it.make("ladybell", 1, at).is_err());
        assert_eq!(it.objects.len(), 2);
    }

    #[test]
    fn find_looks_in_the_world_or_an_inventory() {
        let mut it = items();
        it.make("Crystal", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap();
        it.make("SilverCoins", 6, Place::Carried("Connor".into())).unwrap();
        it.make("SilverCoins", 4, Place::Carried("connor".into())).unwrap();
        assert_eq!(it.find("WORLD", "crystal").map(|o| o.name.as_str()), Some("Crystal.1"));
        assert!(it.find("Connor", "crystal").is_none());
        assert_eq!(it.find("Connor", "silverCoins").map(|o| o.name.as_str()), Some("SilverCoins.2"));
        assert!(it.find("world", "silverCoins").is_none());
        assert_eq!(it.count("Connor", "INVITEM_SilverCoins"), 10);
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
