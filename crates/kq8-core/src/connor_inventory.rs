//! Connor's inventory rules (M4 step 4c-3): an item going in (stacking,
//! swapping equipment), applying and un-applying items, using the bar's
//! slots, the description on a right click.
//!
//! Rules from `docs/behaviour/inventory.md` (A, 2026-10-18; check 4
//! done by the user in the original):
//!
//! - Places (rule 3): a full suit covers body armour, pants, gloves and
//!   boots, not the helmet; the chain-mail suit (`UnderArmor`) pants,
//!   gloves and boots; `GodArmor` has its own place; each of the weapon
//!   and armour categories is one place.
//! - Going in (rule 8): a stacking item (`HealthItem`, `Money`, `Rock`)
//!   whose type has `Comb` 1 joins one Connor already has (the new object
//!   is deleted); everything else is a new entry. Equipment first takes
//!   off what shares a place: placeholders are deleted, other items put
//!   into the world where the new one lay; then the new one is put on
//!   (applied).
//! - Applying (rule 14): only if not applied; a stacking item loses 1;
//!   its five numbers are added to Connor's stats through the stats
//!   event; `Dur` 0 stays applied; `Dur` > 0 runs a timer and is
//!   un-applied at its end; `Dur` −1 is un-applied at once without taking
//!   the numbers back. Un-applying (rule 15): for `Dur` ≥ 0 the numbers
//!   are taken back; a stacking item at 0 leaves the inventory. Taking an
//!   item out un-applies it first (rule 17).
//! - A left click on a slot (rule 9): ignored during handsOff; quest
//!   items, money, rocks and the rope are held (4c-4); the mask piece
//!   opens the main menu; everything else is applied, with its `useSnd`
//!   if it wasn't applied already; a hand weapon is also readied (M5).
//! - A right click describes the item (rule 10): `talking.md` rule 20,
//!   the voice of `0.MSG` (N V C S of the type's message column), played
//!   as a plain sound, a new one stopping the last.
//!
//! STAND-INS: "applied" is kept in the item's `userId` (rule 13) as 0 for
//! Connor (the original keeps an object id there); an old piece taken off
//! by a `give` (no place where the new one lay) drops at Connor's feet
//! ("Edge cases" infers the new object's own position); the potions'
//! timers count game time.

use crate::active::WorldState;
use crate::item::Place;
use crate::stats::Stats;

/// Connor's `userId` while an item is applied to him (stand-in).
pub const CONNOR_USER_ID: i64 = 0;

/// The placeholders that are deleted when taken off (rule 8).
pub const PLACEHOLDERS: [&str; 3] = ["INVITEM_Fists", "INVITEM_NoRanged", "INVITEM_NoArmor"];

/// The eight slots' types, left to right (`interface.md` rule 38).
pub const SLOT_TYPES: [&str; 8] = [
    "INVITEM_Mushroom",
    "INVITEM_Crystal",
    "INVITEM_SacredWater",
    "INVITEM_ElixerOfLife",
    "INVITEM_Invulnerable",
    "INVITEM_Strength",
    "INVITEM_Clarity",
    "INVITEM_Invisible",
];

/// The categories that stack (rule 8).
const STACKING: [&str; 3] = ["HealthItem", "Money", "Rock"];

/// The places a category takes (rule 3); empty for non-equipment.
pub fn places(category: &str) -> &'static [&'static str] {
    match category {
        "ShortWeapon" => &["hand"],
        "LongWeapon" => &["ranged"],
        "Armor" => &["body"],
        "Gloves" => &["gloves"],
        "FootWear" => &["boots"],
        "Helmet" => &["helmet"],
        "Pants" => &["pants"],
        "UnderArmor" => &["pants", "gloves", "boots"],
        "FullArmor" => &["body", "pants", "gloves", "boots"],
        "GodArmor" => &["god"],
        _ => &[],
    }
}

pub fn stacks(category: &str) -> bool {
    STACKING.contains(&category)
}

/// A description voice to play (`talking.md` rule 20): the file, or
/// `None` to only stop the last one.
#[derive(Debug, Clone, PartialEq)]
pub struct ItemVoice {
    pub path: Option<String>,
    pub why: String,
}

fn category(w: &WorldState, kind: &str) -> String {
    w.items.kind(kind).map_or(String::new(), |t| t.category.clone())
}

/// Connor's stats changed by `d` (`interface.md` rule 24: one stats
/// event); logged with before and after.
fn connor_add(w: &mut WorldState, d: Stats, why: &str) -> String {
    let Some(c) = w.connor_stats.as_mut() else { return "Connor's stats unknown".into() };
    let before = *c;
    let level = c.change(d);
    let after = *c;
    w.connor_stats_events += 1;
    format!(
        "{why}: stats {:?} level {} -> {:?} level {}{}",
        before.stats.0,
        before.level,
        after.stats.0,
        after.level,
        level.map_or(String::new(), |l| format!("; {l}"))
    )
}

/// Rule 8: item `name`, now an object, goes into Connor's inventory.
/// `lay_at`: where it lay in the world (for what it pushes out).
pub fn receive(w: &mut WorldState, name: &str, lay_at: Option<[f32; 3]>, why: &str) {
    let Some(i) = w.items.index(name) else { return };
    let o = w.items.objects[i].clone();
    let Some(t) = w.items.kind(&o.kind).cloned() else { return };
    // What he had of this type, the new object left out (a give makes it
    // carried already).
    let before = w.items.count("Connor", &o.kind) - if o.carried_by("Connor") { o.quantity } else { 0 };
    // Stacking: joins the entry he has.
    if stacks(&t.category) && t.combine {
        if let Some(k) = w.items.objects.iter().position(|x| x.name != o.name && x.kind == o.kind && x.carried_by("Connor")) {
            w.items.objects[k].quantity += o.quantity;
            w.items.objects[i].place = Place::Gone;
            let into = w.items.objects[k].name.clone();
            w.item_note(format!("inventory ({why}): {} x{} joins {into}: {} {before} -> {}; {} deleted (inventory.md rule 8)", o.name, o.quantity, o.kind, before + o.quantity, o.name));
            return;
        }
    }
    w.items.objects[i].place = Place::Carried("Connor".into());
    w.items.mark_added(name);
    let mut line = format!("inventory ({why}): {} ({}, {}) x{} is a new entry: {} {before} -> {}", o.name, o.kind, t.category, o.quantity, o.kind, before + o.quantity);
    let mine = places(&t.category);
    if !mine.is_empty() {
        // Rule 8: everything sharing a place comes off.
        let off: Vec<String> = w
            .items
            .objects
            .iter()
            .filter(|x| x.name != o.name && x.carried_by("Connor"))
            .filter(|x| places(&category(w, &x.kind)).iter().any(|p| mine.contains(p)))
            .map(|x| x.name.clone())
            .collect();
        for old in off {
            let kind = w.items.get(&old).map(|x| x.kind.clone()).unwrap_or_default();
            let undone = unapply(w, &old, "taken off");
            let k = w.items.index(&old).expect("listed");
            if PLACEHOLDERS.iter().any(|p| p.eq_ignore_ascii_case(&kind)) {
                w.items.objects[k].place = Place::Gone;
                line.push_str(&format!("; placeholder {old} ({kind}) deleted"));
            } else {
                let (at, whence) = match (lay_at, w.connor_at) {
                    (Some(p), _) => (p, "where the new one lay"),
                    (None, Some(c)) => (c, "at Connor's feet (stand-in: a give has no place)"),
                    (None, None) => ([0.0; 3], "at (0, 0, 0): no place known"),
                };
                w.items.objects[k].place = Place::World { position: at, angles: [0.0; 3] };
                line.push_str(&format!("; {old} ({kind}) taken off and put into the world at ({:.0}, {:.0}, {:.0}) {whence}{undone}", at[0], at[1], at[2]));
            }
        }
        line.push_str(" (inventory.md rule 8)");
        w.item_note(line);
        let on = apply(w, name, "put on");
        w.item_note(on);
        if t.category == "LongWeapon" || t.category == "ShortWeapon" {
            w.item_note(format!("{}: weapon modes and readying are M5; not done", o.name));
        }
        if o.kind.eq_ignore_ascii_case("INVITEM_TempleArmor") {
            w.item_note("INVITEM_TempleArmor: Connor's animation set would change (contpsd2.anm, conicbw2.anm); not done");
        }
        return;
    }
    w.item_note(line + " (inventory.md rule 8)");
}

/// Rule 14: applies item `name` to Connor. Returns the log line.
pub fn apply(w: &mut WorldState, name: &str, why: &str) -> String {
    let Some(i) = w.items.index(name) else { return format!("apply {name}: no such item") };
    let o = w.items.objects[i].clone();
    let Some(t) = w.items.kind(&o.kind).cloned() else { return format!("apply {name}: unknown type {}", o.kind) };
    if o.third != -1 {
        return format!("apply {name} ({why}): already applied (userId {}); nothing (inventory.md rule 14)", o.third);
    }
    if stacks(&t.category) {
        w.items.objects[i].quantity -= 1;
    }
    w.items.objects[i].third = CONNOR_USER_ID;
    let stats = connor_add(w, Stats(t.stats), &format!("its numbers {:?} added", t.stats));
    let mut line = format!("apply {name} ({}, {why}): x{} -> x{}; {stats}", o.kind, o.quantity, w.items.objects[i].quantity);
    if t.duration > 0.0 {
        let due = w.now() + t.duration as f64;
        w.item_timers.push((name.to_string(), due));
        line.push_str(&format!("; Dur {} s: un-applied at t {due:.3}", t.duration));
    } else if t.duration < 0.0 {
        // Dur -1: un-applied at once, the numbers kept.
        w.items.objects[i].third = -1;
        line.push_str("; Dur -1: un-applied at once, the numbers kept");
        line.push_str(&remove_if_empty(w, name));
    } else {
        line.push_str("; Dur 0: stays applied while carried");
    }
    line + " (inventory.md rule 14)"
}

/// A stacking item at 0 leaves the inventory (rule 15).
fn remove_if_empty(w: &mut WorldState, name: &str) -> String {
    let Some(i) = w.items.index(name) else { return String::new() };
    let o = &w.items.objects[i];
    if o.quantity <= 0 && stacks(&category(w, &o.kind)) && o.carried_by("Connor") {
        w.items.objects[i].place = Place::Gone;
        return format!("; count 0: {name} leaves the inventory");
    }
    String::new()
}

/// Rule 15: un-applies item `name`. Returns a note for the log ("" if it
/// wasn't applied).
pub fn unapply(w: &mut WorldState, name: &str, why: &str) -> String {
    let Some(i) = w.items.index(name) else { return String::new() };
    let o = w.items.objects[i].clone();
    if o.third == -1 {
        return String::new();
    }
    let Some(t) = w.items.kind(&o.kind).cloned() else { return String::new() };
    w.items.objects[i].third = -1;
    w.item_timers.retain(|(n, _)| n != name);
    let back = Stats(t.stats.map(|v| -v));
    let stats = connor_add(w, back, &format!("{name} un-applied ({why}): its numbers taken back"));
    format!("; {stats}{} (inventory.md rule 15)", remove_if_empty(w, name))
}

/// The potions' timers (rule 14): un-applies those due by now.
pub fn timers(w: &mut WorldState) {
    let now = w.now();
    let due: Vec<String> = w.item_timers.iter().filter(|(_, t)| *t <= now).map(|(n, _)| n.clone()).collect();
    for name in due {
        let line = unapply(w, &name, "its Dur ran out");
        w.item_note(format!("{name}: timer over{line}"));
    }
}

/// What a click on a bar slot did.
#[derive(Debug, Clone, PartialEq)]
pub enum BarResult {
    Ignored,
    /// Held as the pointer (4c-4).
    Hold,
    /// The main menu (the mask button).
    Menu,
    Applied,
    Described,
    Nothing,
}

/// Rules 9–10: a click on the slot of item `name`; `left` or right;
/// `hands_off`: Connor's handsOff. Logged to `items.log`.
pub fn bar_click(w: &mut WorldState, name: &str, left: bool, hands_off: bool, how: &str) -> BarResult {
    let Some(o) = w.items.get(name).cloned() else {
        w.item_note(format!("bar click on {name} ({how}): no such item; nothing"));
        return BarResult::Nothing;
    };
    let Some(t) = w.items.kind(&o.kind).cloned() else { return BarResult::Nothing };
    let head = format!("bar click {} on {name} ({}, {}, x{}; {how})", if left { "left" } else { "right" }, o.kind, t.category, o.quantity);
    if !left {
        // Rule 10, talking.md rule 20: the description, as a plain voice.
        let words: Vec<u8> = t.message.split_whitespace().filter_map(|v| v.parse().ok()).collect();
        let voice = match words[..] {
            [n, v, c, s] => {
                let tuple = kq8_data::msg::Tuple::new(0, n, v, c, s);
                let folder = w.talk.book.line(tuple).map(|(f, _)| f.to_string());
                let path = folder.and_then(|f| w.talk.voices.as_ref().and_then(|v| v(&f, tuple))).map(|v| v.path);
                w.item_voices.push(ItemVoice { path: path.clone(), why: format!("description of {}", o.kind) });
                format!("0.MSG {tuple}: {}", path.as_deref().unwrap_or("no voice file"))
            }
            _ => format!("message '{}' is not 4 numbers; nothing", t.message),
        };
        w.item_note(format!("{head}: describes it: {voice}; no caption, no ducking (talking.md rule 20)"));
        return BarResult::Described;
    }
    if hands_off {
        w.item_note(format!("{head}: ignored during handsOff (inventory.md rule 9)"));
        return BarResult::Ignored;
    }
    match t.category.as_str() {
        "QuestItem" | "Money" | "Rock" | "Rope" => {
            w.item_note(format!("{head}: would be held as the pointer (inventory.md rule 9; 4c-4)"));
            BarResult::Hold
        }
        "MaskPiece" => {
            w.item_note(format!("{head}: the mask button opens the main menu (interface.md rule 41); no menu yet"));
            BarResult::Menu
        }
        _ => {
            let was = o.third != -1;
            let line = apply(w, name, "slot used");
            let sound = match (&t.use_sound, was) {
                (Some(s), false) => {
                    w.sounds.push(crate::sound::Start::plain_one_shot(s, format!("use of {}", o.kind)));
                    format!("; use sound {s}")
                }
                (None, false) => "; no use sound".into(),
                (_, true) => "; already applied: no use sound".into(),
            };
            let ready = match (t.category.as_str(), t.mode.as_str()) {
                ("ShortWeapon", m) => format!("; readies the weapon ({m}; M5, not done)"),
                ("LongWeapon", "BowMode") => "; readies the bow (BowMode; M5, not done)".into(),
                _ => String::new(),
            };
            w.item_note(format!("{head}: {line}{sound}{ready} (inventory.md rule 9)"));
            BarResult::Applied
        }
    }
}

/// `interface.md` rule 40: the carried item of slot `k` (0–7).
pub fn slot_item(w: &WorldState, k: usize) -> Option<String> {
    let kind = SLOT_TYPES.get(k)?;
    w.items.objects.iter().find(|o| o.carried_by("Connor") && o.kind.eq_ignore_ascii_case(kind) && o.quantity > 0).map(|o| o.name.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active::Active;
    use crate::item::Items;
    use crate::stats::ConnorStats;
    use kq8_data::items::ItemTypes;

    /// Types in `Invinfo.itl`'s layout: name, Hlth SDam LDam Prot Exp,
    /// Comb, category, Dur, use sound.
    /// name, Hlth SDam LDam Prot Exp, Comb, category, Dur, use sound.
    type Row = (&'static str, [i64; 5], u8, &'static str, i32, &'static str);

    fn types() -> ItemTypes {
        let rows: [Row; 8] = [
            ("INVITEM_Mushroom", [5, 0, 0, 0, 0], 1, "HealthItem", -1, "healing.aud"),
            ("INVITEM_Strength", [0, 5, 0, 0, 0], 1, "HealthItem", 20, "none"),
            ("INVITEM_Fists", [0; 5], 0, "ShortWeapon", 0, "none"),
            ("INVITEM_Dagger", [0, 5, 0, 0, 0], 0, "ShortWeapon", 0, "unsheath.aud"),
            ("INVITEM_LeatherGloves", [0, 0, 0, 1, 0], 0, "Gloves", 0, "none"),
            ("INVITEM_FullArmor", [0, 0, 0, 20, 0], 0, "FullArmor", 0, "none"),
            ("INVITEM_SilverCoins", [0; 5], 1, "Money", 0, "none"),
            ("INVITEM_Candle", [0, 0, 0, 0, 25], 0, "QuestItem", 60, "none"),
        ];
        let mut text = format!("[InventoryItemTypeList]\nnumItems={}\n", rows.len());
        for (n, (name, s, comb, cat, dur, snd)) in rows.iter().enumerate() {
            text.push_str(&format!(
                "item{n}=\tKQInventoryItemType\t{name}\t1\t{}\t{}\t{}\t{}\t{}\t{comb}\t{cat}\tIDBMP_X\t1 0 0 1\tNoModeChange\t{dur}\t1\tnone\tnone\t{snd}\tx.kqs\tnone\t1\t256\n",
                s[0], s[1], s[2], s[3], s[4]
            ));
        }
        ItemTypes::parse(text.as_bytes()).0
    }

    fn active() -> Active {
        let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        a.world.items = Items::new(types(), []);
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_at = Some([1.0, 2.0, 3.0]);
        a
    }

    fn carry(a: &mut Active, kind: &str, n: i64) -> String {
        let name = a.world.items.make(kind, n, Place::World { position: [100.0, 200.0, 0.0], angles: [0.0; 3] }).unwrap().name.clone();
        receive(&mut a.world, &name, Some([100.0, 200.0, 0.0]), "test");
        name
    }

    fn stats(a: &Active) -> [i64; 5] {
        a.world.connor_stats.unwrap().stats.0
    }

    #[test]
    fn stacks_join_and_health_items_heal_for_good() {
        let mut a = active();
        let first = carry(&mut a, "Mushroom", 2);
        let second = carry(&mut a, "Mushroom", 3);
        assert_eq!(a.world.items.get(&first).map(|o| o.quantity), Some(5));
        assert_eq!(a.world.items.get(&second).map(|o| o.place.clone()), Some(Place::Gone), "the new object is deleted");
        a.world.connor_stats.as_mut().unwrap().stats.0[0] = 3;
        assert_eq!(slot_item(&a.world, 0).as_deref(), Some(first.as_str()));
        assert_eq!(bar_click(&mut a.world, &first, true, false, "key 1"), BarResult::Applied);
        assert_eq!((stats(&a)[0], a.world.items.get(&first).map(|o| (o.quantity, o.third))), (8, Some((4, -1))), "+5, Dur -1: not applied any more, the heal kept");
        assert_eq!(a.world.sounds.len(), 1, "healing.aud");
        // Capped at 10 x level.
        bar_click(&mut a.world, &first, true, false, "key 1");
        assert_eq!(stats(&a)[0], 10);
        // Ignored during handsOff.
        assert_eq!(bar_click(&mut a.world, &first, true, true, "key 1"), BarResult::Ignored);
        for _ in 0..3 {
            bar_click(&mut a.world, &first, true, false, "key 1");
        }
        assert_eq!(a.world.items.get(&first).map(|o| o.place.clone()), Some(Place::Gone), "count 0: leaves the inventory");
        assert_eq!(slot_item(&a.world, 0), None);
    }

    #[test]
    fn a_potion_runs_out_and_its_numbers_go_back() {
        let mut a = active();
        let p = carry(&mut a, "Strength", 2);
        bar_click(&mut a.world, &p, true, false, "key 6");
        assert_eq!(stats(&a)[1], 15);
        bar_click(&mut a.world, &p, true, false, "key 6");
        assert_eq!((stats(&a)[1], a.world.items.get(&p).map(|o| o.quantity)), (15, Some(1)), "already applied: nothing");
        a.world.updates = 20 * 15;
        timers(&mut a.world);
        assert_eq!((stats(&a)[1], a.world.items.get(&p).map(|o| o.third)), (10, Some(-1)));
        assert!(a.world.item_timers.is_empty());
    }

    #[test]
    fn equipment_pushes_out_what_shares_its_place() {
        let mut a = active();
        let fists = a.world.items.add_named("Fist1", "Fists", 1, -1, Place::Carried("Connor".into())).unwrap().name.clone();
        let dagger = carry(&mut a, "Dagger", 1);
        assert_eq!(a.world.items.get(&fists).map(|o| o.place.clone()), Some(Place::Gone), "a placeholder is deleted");
        assert_eq!(stats(&a)[1], 15, "the dagger is put on: +5");
        let gloves = carry(&mut a, "LeatherGloves", 1);
        assert_eq!(stats(&a)[3], 1);
        // The full suit covers the gloves: they come off and lie where the
        // suit lay; their number is taken back.
        let suit = a.world.items.make("FullArmor", 1, Place::World { position: [7.0, 8.0, 9.0], angles: [0.0; 3] }).unwrap().name.clone();
        receive(&mut a.world, &suit, Some([7.0, 8.0, 9.0]), "picked up");
        assert_eq!(a.world.items.get(&gloves).map(|o| o.place.clone()), Some(Place::World { position: [7.0, 8.0, 9.0], angles: [0.0; 3] }));
        assert_eq!(stats(&a)[3], 20);
        // Gloves again (a give: no place): the suit drops at Connor's feet.
        let g2 = a.world.items.make("LeatherGloves", 1, Place::Carried("Connor".into())).unwrap().name.clone();
        receive(&mut a.world, &g2, None, "give");
        assert_eq!(a.world.items.get(&suit).map(|o| o.place.clone()), Some(Place::World { position: [1.0, 2.0, 3.0], angles: [0.0; 3] }));
        assert_eq!(stats(&a)[3], 1);
        assert!(a.world.items.get(&dagger).unwrap().carried_by("Connor"));
    }

    #[test]
    fn quest_items_money_are_held_the_order_kept() {
        let mut a = active();
        let c = carry(&mut a, "Candle", 1);
        let coins = carry(&mut a, "SilverCoins", 10);
        carry(&mut a, "SilverCoins", 5);
        assert_eq!(a.world.items.get(&coins).map(|o| o.quantity), Some(15));
        assert!(a.world.items.get(&c).unwrap().added < a.world.items.get(&coins).unwrap().added);
        assert_eq!(bar_click(&mut a.world, &c, true, false, "click"), BarResult::Hold);
        assert_eq!(bar_click(&mut a.world, &coins, false, false, "right click"), BarResult::Described);
        assert_eq!(a.world.item_voices.len(), 1);
        assert_eq!(stats(&a)[4], 0, "a bar click never applies a quest item");
    }
}
