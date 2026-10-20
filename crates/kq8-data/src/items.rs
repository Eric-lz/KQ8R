//! The game's list of inventory item types, `GAME/Kq/Invinfo.itl` (M3
//! step 3e-1).
//!
//! Rules from the behaviour note `docs/behaviour/items.md` "Item types
//! (data)" (rules 1 and 2), checked against the file:
//!
//! - Section `[InventoryItemTypeList]`, `numItems=<n>`, then one
//!   `itemN=` line per type. The value's fields are separated by tabs, in
//!   the order of the file's commented `;itemX=` header: class, `IDName`,
//!   id number, health, short and long damage, protection, experience,
//!   combat flag, category, inventory picture, message (4 numbers in one
//!   field), mode, duration, display flag, land sound, pickup sound, use
//!   sound, shape, cursor, radius, translucency, extra data.
//! - A type name in scripts and data may leave out the `INVITEM_` prefix:
//!   if it doesn't start with exactly `INVITEM_` (capitals counting), that
//!   is put in front; then it is compared with `IDName` ignoring capitals.
//!
//! M3 kept the name, id, category, sounds, shape, radius and translucency;
//! M4 4c-2 adds what the inventory and its panels use
//! (`docs/behaviour/inventory.md` rule 1): the five stat changes, `Comb`,
//! the slot picture, the description, the weapon mode, `Dur`, the use
//! sound and the cursor.

use crate::kq_text;

/// The prefix every `IDName` has (rule 2).
pub const PREFIX: &str = "INVITEM_";

/// Field positions in an `itemN=` value (rule 1).
const CLASS: usize = 0;
const ID_NAME: usize = 1;
const ID: usize = 2;
const STATS: usize = 3;
const COMBINE: usize = 8;
const CATEGORY: usize = 9;
const PICTURE: usize = 10;
const MESSAGE: usize = 11;
const MODE: usize = 12;
const DURATION: usize = 13;
const LAND_SOUND: usize = 15;
const PICKUP_SOUND: usize = 16;
const USE_SOUND: usize = 17;
const SHAPE: usize = 18;
const CURSOR: usize = 19;
const RADIUS: usize = 20;
const TRANSLUCENCY: usize = 21;
const EXTRA: usize = 22;

#[derive(Debug, Clone, PartialEq)]
pub struct ItemType {
    /// The key it was read from, e.g. `item43`.
    pub key: String,
    /// `KQInventoryItemType` in every line of the data.
    pub class: String,
    /// `INVITEM_Crystal`.
    pub name: String,
    /// The id number (`3020`), if it reads as one.
    pub id: Option<i64>,
    /// `ShortWeapon`, `HealthItem`, `Money`, `QuestItem`, ...
    pub category: String,
    /// Played when a thrown item lands (rule 12); `None` for `none`.
    pub land_sound: Option<String>,
    /// Played when Connor picks it up (rule 7); `None` for `none`.
    pub pickup_sound: Option<String>,
    /// The model drawn in the world (`crystl.KQS`).
    pub shape: String,
    pub radius: Option<f32>,
    /// 256 = solid (as objects' `translucency`).
    pub translucency: Option<f32>,
    /// `Hlth SDam LDam Prot Exp`: what it adds to the stats when applied
    /// (`inventory.md` rule 1); words that aren't numbers count 0.
    pub stats: [i64; 5],
    /// `Comb` 1: several can share one stack.
    pub combine: bool,
    /// `GUIBitmap`: the slot picture's tag name, e.g. `IDBMP_2SORD1`.
    pub picture: String,
    /// `Message`: the description line's four numbers as written.
    pub message: String,
    /// `Mode`: `SwordMode`, `BowMode`, `NoModeChange`.
    pub mode: String,
    /// `Dur`: seconds the effect lasts; 0 = while carried; −1 = once.
    pub duration: f32,
    /// `useSnd`; `None` for `none`.
    pub use_sound: Option<String>,
    /// `cursor`: the pointer's tag name while held; `None` for `none`.
    pub cursor: Option<String>,
    /// The last column (`extraData`): for a `LongWeapon` the projectile
    /// file it shoots (`Prscbolt.kq`; `fighting.md` rule 36, M5 5c-1).
    pub extra: Option<String>,
}

#[derive(Debug, Clone, Default)]
pub struct ItemTypes {
    /// In file order.
    pub types: Vec<ItemType>,
    /// `numItems` as written, if it reads as a number.
    pub declared: Option<usize>,
}

/// `name` with the `INVITEM_` prefix put in front unless it already
/// starts with exactly that (rule 2): `silverCoins` → `INVITEM_silverCoins`,
/// `invitem_X` → `INVITEM_invitem_X`.
pub fn full_name(name: &str) -> String {
    if name.starts_with(PREFIX) { name.to_string() } else { format!("{PREFIX}{name}") }
}

/// A sound field: `none` (any capitals) or empty means no sound.
fn sound(v: &str) -> Option<String> {
    (!v.is_empty() && !v.eq_ignore_ascii_case("none")).then(|| v.to_string())
}

impl ItemTypes {
    /// Reads the list. Lines that can't be read are returned as problems
    /// (one text each) and left out.
    pub fn parse(bytes: &[u8]) -> (Self, Vec<String>) {
        let mut problems = Vec::new();
        let sections = kq_text::parse(bytes);
        let Some(section) = sections.iter().find(|s| s.name.eq_ignore_ascii_case("InventoryItemTypeList")) else {
            problems.push("no [InventoryItemTypeList] section".to_string());
            return (Self::default(), problems);
        };
        let declared = match section.get("numItems").map(|v| v.trim().parse::<usize>()) {
            Some(Ok(n)) => Some(n),
            Some(Err(_)) => {
                problems.push(format!("numItems={} is not a number", section.get("numItems").unwrap_or("")));
                None
            }
            None => None,
        };
        let mut types = Vec::new();
        for (key, value) in &section.entries {
            let is_item = match (key.get(..4), key.get(4..)) {
                (Some(head), Some(n)) => head.eq_ignore_ascii_case("item") && !n.is_empty() && n.bytes().all(|b| b.is_ascii_digit()),
                _ => false,
            };
            if !is_item {
                continue;
            }
            let fields: Vec<&str> = value.split('\t').map(str::trim).collect();
            let field = |i: usize| fields.get(i).copied().unwrap_or("");
            if field(ID_NAME).is_empty() || field(SHAPE).is_empty() {
                problems.push(format!("{key}: {} fields, no name or no shape; left out", fields.len()));
                continue;
            }
            types.push(ItemType {
                key: key.clone(),
                class: field(CLASS).to_string(),
                name: field(ID_NAME).to_string(),
                id: field(ID).parse().ok(),
                category: field(CATEGORY).to_string(),
                land_sound: sound(field(LAND_SOUND)),
                pickup_sound: sound(field(PICKUP_SOUND)),
                shape: field(SHAPE).to_string(),
                radius: field(RADIUS).parse().ok(),
                translucency: field(TRANSLUCENCY).parse().ok(),
                stats: std::array::from_fn(|k| field(STATS + k).parse::<f64>().map_or(0, |v| v as i64)),
                combine: field(COMBINE).parse::<f64>().is_ok_and(|v| v != 0.0),
                picture: field(PICTURE).to_string(),
                message: field(MESSAGE).to_string(),
                mode: field(MODE).to_string(),
                duration: field(DURATION).parse().unwrap_or(0.0),
                use_sound: sound(field(USE_SOUND)),
                cursor: sound(field(CURSOR)),
                extra: sound(field(EXTRA)),
            });
        }
        (ItemTypes { types, declared }, problems)
    }

    /// The type a script or data file means by `name` (rule 2).
    pub fn find(&self, name: &str) -> Option<&ItemType> {
        let full = full_name(name);
        self.types.iter().find(|t| t.name.eq_ignore_ascii_case(&full))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Two lines in the file's layout (made-up values where they don't
    /// matter): a leading tab after `=`, CRLF, trailing tabs.
    const LIST: &[u8] = b"[InventoryItemTypeList]\t\t\r\nclassType=KQInventoryItemTypeList\t\t\r\n\r\n; comment\t\t\r\nnumItems=\t2\t\t\r\n;itemX=\tclassType\tIDName\r\n\
item0=\tKQInventoryItemType\tINVITEM_Crystal\t3020\t50\t0\t0\t0\t0\t1\tHealthItem\tIDBMP_X\t13 0 0 1\tNoModeChange\t-1\t1\tpotnland.aud\tenergy.aud\thealing.aud\tcrystl.KQS\tnone\t250\t200\t\r\n\
item1=\tKQInventoryItemType\tINVITEM_SilverCoins\t4030\t0\t0\t0\t0\t0\t1\tMoney\tIDBMP_Y\t47 0 0 1\tNoModeChange\t0\t0\tnone\tcoins.aud\tcoins.aud\tpouch.kqs\tIDBMA_GOLDCURS\t150\t256\r\n";

    #[test]
    fn reads_the_fields_by_position() {
        let (list, problems) = ItemTypes::parse(LIST);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(list.declared, Some(2));
        assert_eq!(list.types.len(), 2);
        let c = &list.types[0];
        assert_eq!((c.key.as_str(), c.class.as_str(), c.name.as_str(), c.id), ("item0", "KQInventoryItemType", "INVITEM_Crystal", Some(3020)));
        assert_eq!(c.category, "HealthItem");
        assert_eq!((c.land_sound.as_deref(), c.pickup_sound.as_deref()), (Some("potnland.aud"), Some("energy.aud")));
        assert_eq!((c.shape.as_str(), c.radius, c.translucency), ("crystl.KQS", Some(250.0), Some(200.0)));
        assert_eq!((c.stats, c.combine, c.picture.as_str(), c.message.as_str()), ([50, 0, 0, 0, 0], true, "IDBMP_X", "13 0 0 1"));
        assert_eq!((c.mode.as_str(), c.duration, c.use_sound.as_deref(), c.cursor.as_deref()), ("NoModeChange", -1.0, Some("healing.aud"), None));
        // `none` is no sound; a line without the last (extra data) field.
        let s = &list.types[1];
        assert_eq!((s.land_sound.as_deref(), s.pickup_sound.as_deref()), (None, Some("coins.aud")));
        assert_eq!((s.shape.as_str(), s.translucency), ("pouch.kqs", Some(256.0)));
        assert_eq!(s.cursor.as_deref(), Some("IDBMA_GOLDCURS"));
        assert_eq!((c.extra.as_deref(), s.extra.as_deref()), (None, None));
    }

    #[test]
    fn the_prefix_rule() {
        let (list, _) = ItemTypes::parse(LIST);
        let name = |n: &str| list.find(n).map(|t| t.name.as_str());
        assert_eq!(name("silverCoins"), Some("INVITEM_SilverCoins"));
        assert_eq!(name("INVITEM_Crystal"), Some("INVITEM_Crystal"));
        assert_eq!(name("crystal"), Some("INVITEM_Crystal"));
        assert_eq!(name("INVITEM_crystal"), Some("INVITEM_Crystal"));
        // Not exactly `INVITEM_`: a second prefix goes in front.
        assert_eq!(full_name("invitem_Crystal"), "INVITEM_invitem_Crystal");
        assert_eq!(name("invitem_Crystal"), None);
        assert_eq!(name("ladybell"), None);
    }

    #[test]
    fn a_line_without_a_shape_is_a_problem() {
        let (list, problems) = ItemTypes::parse(b"[InventoryItemTypeList]\nnumItems=1\nitem0=\tKQInventoryItemType\tINVITEM_X\t1\n");
        assert!(list.types.is_empty());
        assert_eq!(problems.len(), 1, "{problems:?}");
    }
}
