//! Connor's inventory as the panels show it (M4 step 4c-2 on).
//!
//! Rules from `docs/behaviour/inventory.md` and `interface.md` (A,
//! 2026-10-18):
//!
//! - Everything he carries is one list of item objects; where an item
//!   shows follows from its type's category (`inventory.md` rule 2).
//! - His start (`Connor.kq` `[ConnerInventory]`): the first mask piece
//!   (a quest item), the three placeholders `INVITEM_Fists`,
//!   `INVITEM_NoRanged`, `INVITEM_NoArmor` (empty pictures, no stats) and
//!   the mask button's piece (`inventory.md` rule 4).
//! - The armour, hand-weapon and ranged slots show the picture of the
//!   item worn there (`interface.md` rule 34). Only body armour, under-
//!   armour, full suits and `GodArmor` change the armour picture; with a
//!   chain-mail suit and body armour both worn, the suit's picture unless
//!   the body armour is plate or bronze plate (rule 35).
//!
//! STAND-INS (not traced): with `GodArmor` worn its picture wins; a full
//! suit's picture next; the placeholder `INVITEM_NoArmor` counts as no
//! body armour beside a suit ("Edge cases": its special mark is probably
//! for this).

use crate::item::Items;

/// The categories that change the armour picture (rule 35).
const ARMOUR_PICTURES: [&str; 4] = ["Armor", "UnderArmor", "FullArmor", "GodArmor"];

/// Body armour whose picture wins over a suit's (rule 35).
const HEAVY_BODY: [&str; 2] = ["INVITEM_PlateMail", "INVITEM_BronzePlate"];

/// The placeholder body armour.
pub const NO_ARMOR: &str = "INVITEM_NoArmor";

/// The type names whose pictures the armour, hand-weapon and ranged
/// slots show (`None`: the layout's empty picture), and a note on how the
/// armour one was chosen.
pub fn slot_types(items: &Items, who: &str) -> ([Option<String>; 3], String) {
    let carried: Vec<(&str, &str)> = items
        .objects
        .iter()
        .filter(|o| o.carried_by(who))
        .filter_map(|o| items.kind(&o.kind).map(|t| (t.name.as_str(), t.category.as_str())))
        .collect();
    let first = |cat: &str| carried.iter().find(|(_, c)| c.eq_ignore_ascii_case(cat)).map(|(n, _)| n.to_string());
    let short = first("ShortWeapon");
    let long = first("LongWeapon");
    let worn: Vec<(&str, &str)> = carried.iter().copied().filter(|(_, c)| ARMOUR_PICTURES.iter().any(|a| a.eq_ignore_ascii_case(c))).collect();
    let of = |cat: &str| worn.iter().find(|(_, c)| c.eq_ignore_ascii_case(cat)).map(|(n, _)| n.to_string());
    let body = of("Armor");
    let (armour, why) = if let Some(g) = of("GodArmor") {
        (Some(g), "GodArmor worn: its picture (stand-in)".to_string())
    } else if let Some(f) = of("FullArmor") {
        (Some(f), "a full suit: its picture".to_string())
    } else if let Some(suit) = of("UnderArmor") {
        match body.filter(|b| b != NO_ARMOR) {
            Some(b) if HEAVY_BODY.contains(&b.as_str()) => (Some(b), "suit and plate or bronze plate: the body armour's picture (rule 35)".to_string()),
            Some(_) => (Some(suit), "suit and leather or chain shirt: the suit's picture (rule 35)".to_string()),
            None => (Some(suit), "a suit: its picture".to_string()),
        }
    } else {
        let why = if body.is_some() { "body armour: its picture" } else { "nothing that changes the picture: the empty one" };
        (body, why.to_string())
    };
    ([armour, short, long], why)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::item::Place;
    use kq8_data::items::ItemTypes;

    fn items() -> Items {
        let line = |n: usize, name: &str, cat: &str, pic: &str| format!("item{n}=\tKQInventoryItemType\t{name}\t1\t0\t0\t0\t0\t0\t0\t{cat}\t{pic}\t1 0 0 1\tNoModeChange\t0\t0\tnone\tnone\tnone\tx.kqs\tnone\t1\t256\n");
        let list = [
            ("INVITEM_Fists", "ShortWeapon", "IDBMP_2SORD0"),
            ("INVITEM_NoRanged", "LongWeapon", "IDBMP_2RNG0"),
            ("INVITEM_NoArmor", "Armor", "IDBMP_2ARMR0"),
            ("INVITEM_ChainMailSuit", "UnderArmor", "IDBMP_2ARMR5"),
            ("INVITEM_LeatherArmor", "Armor", "IDBMP_2ARMR1"),
            ("INVITEM_PlateMail", "Armor", "IDBMP_2ARMR3"),
            ("INVITEM_Helmet", "Helmet", "IDBMP_2ARMR6"),
        ];
        let mut text = format!("[InventoryItemTypeList]\nnumItems={}\n", list.len());
        for (n, (name, cat, pic)) in list.iter().enumerate() {
            text.push_str(&line(n, name, cat, pic));
        }
        Items::new(ItemTypes::parse(text.as_bytes()).0, [])
    }

    fn give(it: &mut Items, kind: &str) {
        it.make(kind, 1, Place::Carried("Connor".into())).unwrap();
    }

    #[test]
    fn the_slots_show_what_he_wears() {
        let mut it = items();
        for k in ["Fists", "NoRanged", "NoArmor", "Helmet"] {
            give(&mut it, k);
        }
        let (s, _) = slot_types(&it, "Connor");
        assert_eq!(s, [Some("INVITEM_NoArmor".into()), Some("INVITEM_Fists".into()), Some("INVITEM_NoRanged".into())], "a helmet changes only the number");
        give(&mut it, "ChainMailSuit");
        assert_eq!(slot_types(&it, "Connor").0[0].as_deref(), Some("INVITEM_ChainMailSuit"), "NoArmor beside a suit: the suit");
        give(&mut it, "LeatherArmor");
        assert_eq!(slot_types(&it, "Connor").0[0].as_deref(), Some("INVITEM_ChainMailSuit"), "leather: the suit's picture");
        let mut plate = items();
        give(&mut plate, "PlateMail");
        give(&mut plate, "ChainMailSuit");
        assert_eq!(slot_types(&plate, "Connor").0[0].as_deref(), Some("INVITEM_PlateMail"), "plate: the body armour's picture");
        assert_eq!(slot_types(&plate, "Wizard").0, [None, None, None]);
    }
}
