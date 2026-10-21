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
//! The item in hand (M4 4c-4; `inventory.md` rules 7, 9, 12): a left
//! click on a quest item, the coins, the rocks or the rope makes it the
//! hand (its type's `cursor` is the pointer, and it leaves the quest
//! strip); the same item again puts it back. The scroll can't be taken:
//! a remark, at most every 3 s and not while Connor talks. A type with
//! cursor `none` does nothing. A world click with the hand: accepted →
//! the hand is emptied, except coins while `goldCoinPersists 1`; refused or
//! out of reach → it stays.
//!
//! STAND-INS: "applied" is kept in the item's `userId` (rule 13) as 0 for
//! Connor (the original keeps an object id there); an old piece taken off
//! by a `give` (no place where the new one lay) drops at Connor's feet
//! ("Edge cases" infers the new object's own position). The potions'
//! timers: `crate::potions` (M5 5g-2).

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
    added_extras(w, &o.kind, &t.category);
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
        // The user's check in the original (5d fix 10): a hand weapon taken
        // while one is ready is readied in its place.
        if t.category == "ShortWeapon" && w.weapon_mode == crate::weapons::WeaponMode::Hand {
            let was = w.weapon_ready.replace(o.kind.clone());
            w.weapon_swapped = true;
            w.item_note(format!("{}: a hand weapon was ready ({was:?}): {} is ready now (the user's check)", o.name, o.kind));
        }
        if t.category == "LongWeapon" {
            // ranged-weapons.md rules 17, 21 (M5 5f-4).
            let line = w.take_ranged_item(&o.kind);
            w.item_note(format!("{}: {line}", o.name));
            w.fight_note("Connor", format!("{} goes into his inventory: {line}", o.name));
            // "Edge cases": picked up with the ranged mode ready: the new
            // list at once, no swap.
            if w.weapon_mode == crate::weapons::WeaponMode::Ranged && !o.kind.eq_ignore_ascii_case(crate::weapons::NO_RANGED) {
                let was = w.weapon_ready.replace(o.kind.clone());
                w.weapon_swapped = true;
                w.fight_note("Connor", format!("the ranged mode was ready ({was:?}): {} is ready now, its list at once, no swap (ranged-weapons.md, Edge cases)", o.kind));
            }
        }
        if o.kind.eq_ignore_ascii_case("INVITEM_TempleArmor") {
            w.item_note("INVITEM_TempleArmor: Connor's animation set would change (contpsd2.anm, conicbw2.anm); not done");
        }
        return;
    }
    w.item_note(line + " (inventory.md rule 8)");
}

/// `potions.md` rules 40–41 (M5 5g-8): what Connor does when any item is
/// added, every time: built-in experience for some quest items (0.3 s
/// later or at once), and the help pop-ups (logged only: not studied).
fn added_extras(w: &mut WorldState, kind: &str, category: &str) {
    if let Some(&(_, exp, later)) = crate::potions::PICKUP_EXPERIENCE.iter().find(|(k, ..)| k.eq_ignore_ascii_case(kind)) {
        if later {
            let due = w.now() + crate::potions::PICKUP_DELAY;
            w.pickup_experience.push((due, exp, kind.to_string()));
            w.item_note(format!("{kind} added: +{exp} experience due 0.3 s later, at game time {due:.3} (potions.md rules 40-41)"));
        } else {
            let line = connor_add(w, Stats([0, 0, 0, 0, exp]), &format!("{kind} added: +{exp} experience at once (potions.md rule 41)"));
            w.item_note(line);
        }
    }
    let help = match category {
        "HealthItem" => Some(("0x83", 3.0, "the first health item or potion")),
        "QuestItem" => Some(("0x84", 0.0, "the first quest item")),
        _ => None,
    };
    if let Some((id, after, what)) = help.filter(|(id, ..)| !w.helps_asked.contains(id)) {
        w.helps_asked.insert(id);
        w.item_note(format!("{what} ({kind}): help pop-up {id} asked for{} (potions.md rule 41; pop-ups not studied: logged only)", if after > 0.0 { format!(" {after} s later") } else { String::new() }));
    }
    if kind.eq_ignore_ascii_case("INVITEM_RopeAndHook") {
        w.item_note("INVITEM_RopeAndHook added: help pop-up 0x80 asked for 4.5 s later (potions.md rule 41; logged only)");
    }
}

/// Rule 14: applies item `name` to Connor. Returns the log line.
pub fn apply(w: &mut WorldState, name: &str, why: &str) -> String {
    let Some(i) = w.items.index(name) else { return format!("apply {name}: no such item") };
    let o = w.items.objects[i].clone();
    let Some(t) = w.items.kind(&o.kind).cloned() else { return format!("apply {name}: unknown type {}", o.kind) };
    if o.third != -1 {
        return format!("apply {name} ({why}): already applied (userId {}); nothing (inventory.md rule 14)", o.third);
    }
    // potions.md rule 13: a potion's effect switches on first.
    let (added, effect) = if t.duration > 0.0 { crate::potions::effect_on(w, crate::potions::Effect::of(&o.kind)) } else { (0, String::new()) };
    if stacks(&t.category) {
        w.items.objects[i].quantity -= 1;
    }
    w.items.objects[i].third = CONNOR_USER_ID;
    let stats = connor_add(w, Stats(t.stats), &format!("its numbers {:?} added", t.stats));
    let effect = if effect.is_empty() { effect } else { format!("{effect}; ") };
    let mut line = format!("apply {name} ({}, {why}): {effect}x{} -> x{}; {stats}", o.kind, o.quantity, w.items.objects[i].quantity);
    if t.duration > 0.0 {
        // potions.md rule 33: whole seconds, counted from this update.
        let mut r = crate::potions::Running::new(name, &o.kind, t.duration.round() as i64, w.updates);
        r.added = added;
        line.push_str(&format!("; Dur {} s: the timer starts at {} s left (game time {:.3}), the next second off at update {} (potions.md rule 33)", t.duration, r.left, w.now(), r.next_tick));
        // Rule 35: its icon in the first free place.
        r.place = crate::potions::take_place(&mut w.status_icons, crate::potions::StatusIcon::Potion(name.to_string()));
        line.push_str(&match r.place {
            Some(k) => format!("; icon {} in IDCTL_TIMER_{}, 100 % (potions.md rule 35)", t.extra.as_deref().unwrap_or("none"), k + 1),
            None => "; all 7 icon places taken: no icon (potions.md rule 35)".into(),
        });
        w.potions.push(r);
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
    let running = w.potions.iter().position(|r| r.item == name).map(|k| w.potions.remove(k));
    let icon = crate::potions::free_place(&mut w.status_icons, &crate::potions::StatusIcon::Potion(name.to_string()));
    // potions.md rule 13: at the end the stats event, then the effect off.
    let back = Stats(t.stats.map(|v| -v));
    let stats = connor_add(w, back, &format!("{name} un-applied ({why}): its numbers taken back"));
    let effect = running.map(|r| crate::potions::effect_off(w, &r)).filter(|e| !e.is_empty()).map_or(String::new(), |e| format!("; {e}"));
    let icon = icon.map_or(String::new(), |k| format!("; icon place IDCTL_TIMER_{} freed (potions.md rule 35)", k + 1));
    format!("; {stats}{effect}{icon}{} (inventory.md rule 15)", remove_if_empty(w, name))
}

/// The potions' timers (`potions.md` rule 33): one second off each
/// running potion per second of game time; un-applies those over.
pub fn timers(w: &mut WorldState) {
    // potions.md rules 40-41: experience due for items added.
    let t = w.now();
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut w.pickup_experience).into_iter().partition(|(at, ..)| *at <= t + 1e-9);
    w.pickup_experience = later;
    for (_, exp, kind) in due {
        let line = connor_add(w, Stats([0, 0, 0, 0, exp]), &format!("{kind} added 0.3 s ago: +{exp} experience (potions.md rules 40-41)"));
        w.item_note(line);
    }
    let now = w.updates;
    let mut over = Vec::new();
    let mut lines = Vec::new();
    for r in &mut w.potions {
        let (seconds, ended) = r.tick(now);
        for left in seconds {
            lines.push(format!("{} ({}): {left} s left of {}, {} % (potions.md rule 33)", r.item, r.kind, r.duration, left * 100 / r.duration.max(1)));
        }
        if ended {
            over.push(r.item.clone());
        }
    }
    for line in lines {
        w.item_note(line);
    }
    for name in over {
        let line = unapply(w, &name, "its time is up");
        w.item_note(format!("{name}: its time is up at game time {:.3}{line}", w.now()));
    }
}

/// The scroll that can't be taken, and its remark (rule 9 writes
/// `0.MSG` (0, 92, 14, 0); read as file 0, noun 92, verb 14, case 0,
/// sequence 1: the reading that names a line in `0.MSG`, Q110).
const SCROLL: &str = "INVITEM_Scroll";
const SCROLL_REMARK: kq8_data::msg::Tuple = kq8_data::msg::Tuple { file: 0, noun: 92, verb: 14, case: 0, seq: 1 };
/// The rope away from a rope place (rule 9 writes (0, 0, 58, 37); read the
/// same way, Q110).
const ROPE_REMARK: kq8_data::msg::Tuple = kq8_data::msg::Tuple { file: 0, noun: 0, verb: 58, case: 37, seq: 1 };

/// Rule 9: at most one scroll remark every 3 s.
const SCROLL_GAP: f64 = 3.0;

/// What a click on a bar slot did.
#[derive(Debug, Clone, PartialEq)]
pub enum BarResult {
    Ignored,
    /// Now the item in hand (4c-4).
    Hold,
    /// The item in hand went back (4c-4).
    PutBack,
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
    let slot = SLOT_TYPES.iter().position(|s| s.eq_ignore_ascii_case(&o.kind)).map_or(String::new(), |k| format!("slot {}, ", k + 1));
    let head = format!("bar click {} on {name} ({slot}{}, {}, x{}; {how})", if left { "left" } else { "right" }, o.kind, t.category, o.quantity);
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
        w.item_note(format!("{head}: refused (handsOff): nothing, no sound, no remark (inventory.md rule 9, potions.md rule 2)"));
        return BarResult::Ignored;
    }
    if w.connor_dying {
        w.item_note(format!("{head}: refused (Connor is dying or dead): nothing, no sound, no remark (potions.md rule 2)"));
        return BarResult::Ignored;
    }
    match t.category.as_str() {
        "QuestItem" | "Money" | "Rock" | "Rope" => hold(w, name, &t, &head),
        "MaskPiece" => {
            w.item_note(format!("{head}: the mask button opens the main menu (interface.md rule 41); no menu yet"));
            BarResult::Menu
        }
        _ => {
            let was = o.third != -1;
            if was && t.category == "HealthItem" {
                // potions.md rule 7: the same potion while it runs: no
                // sound, no count change, the timer untouched.
                w.item_note(format!("{head}: already running: nothing (no sound, count stays x{}, timer untouched; potions.md rule 7)", o.quantity));
                return BarResult::Nothing;
            }
            let line = apply(w, name, "slot used");
            // potions.md rule 4: the use sound only when it takes effect.
            let sound = match (&t.use_sound, was) {
                (Some(s), false) => {
                    w.sounds.push(crate::sound::Start::plain_one_shot(s, format!("use of {}", o.kind)));
                    format!("; use sound {s} (plain, not placed)")
                }
                (None, false) => "; no use sound".into(),
                (_, true) => "; already applied: no use sound".into(),
            };
            // ranged-weapons.md rule 1 (M5 5f-3): a weapon slot asks for
            // its mode column's mode; the viewer takes it (rule 5).
            let ready = match t.category.as_str() {
                "ShortWeapon" | "LongWeapon" => {
                    w.ask_weapon(crate::weapons::Ask::Slot { item: o.kind.clone(), column: t.mode.clone() }, how);
                    format!("; asks for its mode ({})", t.mode)
                }
                _ => String::new(),
            };
            w.item_note(format!("{head}: {line}{sound}{ready} (inventory.md rule 9)"));
            BarResult::Applied
        }
    }
}

/// `inventory.md` rule 9, `fighting.md` rule 39 (M5 5c-5): clicking the
/// hand-weapon slot readies that weapon (`INVITEM_Fists` too: Connor
/// readies his fists); **stand-in**: clicking it again while it is ready
/// puts it away (how a weapon is put away was not traced). Returns the log
/// text.
pub fn ready_hand_weapon(w: &mut WorldState, kind: &str) -> String {
    if w.weapon_ready.as_deref().is_some_and(|r| r.eq_ignore_ascii_case(kind)) {
        w.set_weapon(crate::weapons::WeaponMode::None, None);
        return format!("puts {kind} away (stand-in: a second click on a ready weapon)");
    }
    w.set_weapon(crate::weapons::WeaponMode::Hand, Some(kind.to_string()));
    format!("readies {kind}: a left click in the world now swings (fighting.md rule 39; the input side is a stand-in)")
}

/// The hand weapon Connor carries (the first carried `ShortWeapon`; the
/// placeholder `INVITEM_Fists` when he has no other), its type name.
pub fn hand_weapon(w: &WorldState) -> Option<String> {
    w.items.objects.iter().filter(|o| o.carried_by(crate::talk::CONNOR)).find(|o| w.items.types.find(&o.kind).is_some_and(|t| t.category == "ShortWeapon")).map(|o| o.kind.clone())
}

/// Rule 9: a left click on a quest item, coins, rocks or the rope.
fn hold(w: &mut WorldState, name: &str, t: &kq8_data::items::ItemType, head: &str) -> BarResult {
    if w.hand.as_deref().is_some_and(|h| h.eq_ignore_ascii_case(name)) {
        w.hand = None;
        w.item_note(format!("{head}: it is the item in hand: put back, the play pointer again (inventory.md rule 9)"));
        return BarResult::PutBack;
    }
    if t.name.eq_ignore_ascii_case(SCROLL) {
        let now = w.now();
        let recent = w.scroll_remark_at.is_some_and(|t| now - t < SCROLL_GAP);
        let line = if recent {
            "the scroll can't be taken; its remark was less than 3 s ago: nothing".to_string()
        } else if w.talk.is_talking(crate::talk::CONNOR) {
            "the scroll can't be taken; Connor is talking: no remark".to_string()
        } else {
            w.scroll_remark_at = Some(now);
            format!("the scroll can't be taken: {}", w.connor_remark(SCROLL_REMARK, "scroll"))
        };
        w.item_note(format!("{head}: {line} (inventory.md rule 9)"));
        return BarResult::Nothing;
    }
    let Some(cursor) = t.cursor.clone() else {
        w.item_note(format!("{head}: its type has cursor none: can't be held; nothing (inventory.md rule 9)"));
        return BarResult::Nothing;
    };
    let mut extra = String::new();
    if t.category == "Rope" {
        // Rule 9: there are no rope places yet (M6). STAND-IN (Q110): held
        // and the remark.
        extra = format!("; not at a rope place (none yet): {}", w.connor_remark(ROPE_REMARK, "can't use the rope here"));
    } else if t.category == "Rock" {
        extra = "; none of the three states that refuse it exists yet".into();
    }
    let before = w.hand.replace(name.to_string());
    let back = before.map_or(String::new(), |b| format!("; {b} goes back to its slot"));
    w.item_note(format!("{head}: now the item in hand, the pointer {cursor}{back}{extra} (inventory.md rule 9)"));
    BarResult::Hold
}

/// Rule 12: after a world click made while holding `held`: `accepted`
/// (the object's script got `ConnorAction <item> <type>`) empties the
/// hand, except coins while `goldCoinPersists 1`; otherwise it stays.
pub fn after_world_click(w: &mut WorldState, held: &str, accepted: bool) {
    let kind = w.items.get(held).map(|o| o.kind.clone()).unwrap_or_default();
    let money = w.items.kind(&kind).is_some_and(|t| t.category == "Money");
    let line = if !accepted {
        "not accepted or out of reach: it stays in hand".to_string()
    } else if money && w.gold_persists {
        "accepted; coins while goldCoinPersists 1: they stay in hand".to_string()
    } else {
        w.hand = None;
        let gone = if w.items.get(held).is_some_and(|o| o.carried_by(crate::talk::CONNOR)) { "still in the inventory" } else { "taken by the script" };
        format!("accepted: the hand is emptied, the play pointer back; the item is {gone}")
    };
    w.item_note(format!("world click with {held} ({kind}) in hand: {line} (inventory.md rule 12)"));
}

/// The item in hand, if it still exists and is carried (a script may
/// delete or take it; then the hand is emptied).
pub fn hand(w: &mut WorldState) -> Option<(String, String)> {
    let name = w.hand.clone()?;
    match w.items.get(&name) {
        Some(o) if o.carried_by(crate::talk::CONNOR) => Some((o.name.clone(), o.kind.clone())),
        _ => {
            w.hand = None;
            w.item_note(format!("the item in hand {name} is no longer carried: the hand is emptied"));
            None
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
        let rows: [Row; 11] = [
            ("INVITEM_Clarity", [0; 5], 1, "HealthItem", 60, "clarity.aud"),
            ("INVITEM_Invisible", [0; 5], 1, "HealthItem", 15, "invisibl.aud"),
            ("INVITEM_Invulnerable", [0; 5], 1, "HealthItem", 20, "invuln.aud"),
            ("INVITEM_Mushroom", [5, 0, 0, 0, 0], 1, "HealthItem", -1, "healing.aud"),
            ("INVITEM_Strength", [0; 5], 1, "HealthItem", 20, "none"),
            ("INVITEM_Fists", [0; 5], 0, "ShortWeapon", 0, "none"),
            ("INVITEM_Dagger", [0, 5, 0, 0, 0], 0, "ShortWeapon", 0, "unsheath.aud"),
            ("INVITEM_LeatherGloves", [0, 0, 0, 1, 0], 0, "Gloves", 0, "none"),
            ("INVITEM_FullArmor", [0, 0, 0, 20, 0], 0, "FullArmor", 0, "none"),
            ("INVITEM_SilverCoins", [0; 5], 1, "Money", 0, "none"),
            ("INVITEM_Candle", [0, 0, 0, 0, 25], 0, "QuestItem", 60, "none"),
        ];
        let mut text = format!("[InventoryItemTypeList]\nnumItems={}\n", rows.len());
        for (n, (name, s, comb, cat, dur, snd)) in rows.iter().enumerate() {
            let cursor = if matches!(*cat, "QuestItem" | "Money") { "IDBMA_CURS" } else { "none" };
            text.push_str(&format!(
                "item{n}=\tKQInventoryItemType\t{name}\t1\t{}\t{}\t{}\t{}\t{}\t{comb}\t{cat}\tIDBMP_X\t1 0 0 1\tNoModeChange\t{dur}\t1\tnone\tnone\t{snd}\tx.kqs\t{cursor}\t1\t256\n",
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
        a.world.updates = 20 * 15 - 1;
        timers(&mut a.world);
        assert_eq!((stats(&a)[1], a.world.potions[0].left), (15, 1), "1 s left at 19 s");
        a.world.updates = 20 * 15;
        timers(&mut a.world);
        assert_eq!((stats(&a)[1], a.world.items.get(&p).map(|o| o.third)), (10, Some(-1)));
        assert!(a.world.potions.is_empty());
    }

    #[test]
    fn the_last_dose_runs_at_count_0_and_dying_refuses() {
        let mut a = active();
        let p = carry(&mut a, "Strength", 1);
        assert_eq!(bar_click(&mut a.world, &p, true, false, "key 6"), BarResult::Applied);
        assert_eq!(a.world.sounds.len(), 0, "the test type's use sound is none");
        // potions.md rule 8: kept at count 0 while it runs, the slot empty.
        let o = a.world.items.get(&p).unwrap();
        assert_eq!((o.quantity, o.carried_by("Connor")), (0, true));
        assert_eq!(slot_item(&a.world, 5), None);
        // A new one picked up meanwhile joins it; clicking it is the
        // running potion again: nothing (rule 7).
        carry(&mut a, "Strength", 1);
        assert_eq!(a.world.items.get(&p).map(|o| o.quantity), Some(1));
        assert_eq!(bar_click(&mut a.world, &p, true, false, "key 6"), BarResult::Nothing);
        assert_eq!(a.world.items.get(&p).map(|o| o.quantity), Some(1));
        // Rule 2: refused while dying.
        let m = carry(&mut a, "Mushroom", 1);
        a.world.connor_dying = true;
        assert_eq!(bar_click(&mut a.world, &m, true, false, "key 1"), BarResult::Ignored);
        assert_eq!(a.world.items.get(&m).map(|o| o.quantity), Some(1));
        assert!(a.world.sounds.is_empty());
    }

    #[test]
    fn the_shield_raises_protection_and_takes_off_only_what_it_added() {
        let mut a = active();
        a.world.connor_stats.as_mut().unwrap().stats.0[3] = 40;
        let v = carry(&mut a, "Invulnerable", 1);
        bar_click(&mut a.world, &v, true, false, "key 5");
        assert_eq!(stats(&a)[3], 60);
        // The last temple: a hit uses 40 + 20 / 2 = 50 (potions.md rule 19).
        a.world.connor_in_temple4 = true;
        a.world.hit_connor("goblin", 5, crate::fight::HitKind::Normal);
        assert!(a.world.fight_log.iter().any(|l| l.contains("protection 60 used as 50")), "{:?}", a.world.fight_log);
        // Armour put on meanwhile (+1) is kept at the end: 61 - 20 = 41.
        a.world.connor_stats.as_mut().unwrap().stats.0[3] += 1;
        a.world.updates = 20 * 15;
        timers(&mut a.world);
        assert_eq!(stats(&a)[3], 41);
        assert!(a.world.status_icons.iter().all(Option::is_none));
    }

    #[test]
    fn invisible_reports_him_100000_below() {
        let mut a = active();
        let p = carry(&mut a, "Invisible", 1);
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0]));
        bar_click(&mut a.world, &p, true, false, "key 8");
        assert!(a.world.connor_invisible());
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0 - 100_000.0]));
        assert_eq!(a.world.connor_at, Some([1.0, 2.0, 3.0]), "his true place is kept");
        a.world.updates = 15 * 15;
        timers(&mut a.world);
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0]));
        // M5 5g-9: a cutscene does the same, but checkInvisible stays 0.
        a.world.connor_hands_off = true;
        assert_eq!(a.world.connor_reported(), Some([1.0, 2.0, 3.0 - 100_000.0]));
        assert!(!a.world.connor_invisible());
    }

    #[test]
    fn clarity_tells_the_reveal_objects_at_start_and_end() {
        let mut a = active();
        a.world.reveal_objects = vec!["keepwall".into(), "wizchest1".into()];
        let p = carry(&mut a, "Clarity", 1);
        bar_click(&mut a.world, &p, true, false, "key 7");
        assert_eq!(std::mem::take(&mut a.world.reveal_calls), vec![("keepwall".to_string(), true), ("wizchest1".to_string(), true)]);
        a.world.updates = 60 * 15;
        timers(&mut a.world);
        assert_eq!(a.world.reveal_calls, vec![("keepwall".to_string(), false), ("wizchest1".to_string(), false)]);
    }

    #[test]
    fn the_candle_gives_25_experience_0_3_s_after_it_is_added() {
        let mut a = active();
        carry(&mut a, "Candle", 1);
        assert_eq!(stats(&a)[4], 0);
        a.world.updates = 4;
        timers(&mut a.world);
        assert_eq!(stats(&a)[4], 0, "0.27 s: not yet");
        a.world.updates = 5;
        timers(&mut a.world);
        // +25 at level 1 (threshold 10): level 2, 15 left.
        let level = |a: &Active| a.world.connor_stats.unwrap().level;
        assert_eq!((level(&a), stats(&a)[4]), (2, 15));
        // Never taken back (rule 40): its Dur is not used.
        a.world.updates = 2000;
        timers(&mut a.world);
        assert_eq!((level(&a), stats(&a)[4]), (2, 15));
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
        assert_eq!(a.world.hand.as_deref(), Some(c.as_str()));
        // The coins: they become the hand, the candle goes back.
        assert_eq!(bar_click(&mut a.world, &coins, true, false, "click"), BarResult::Hold);
        assert_eq!(a.world.hand.as_deref(), Some(coins.as_str()));
        // The same again: put back.
        assert_eq!(bar_click(&mut a.world, &coins, true, false, "click"), BarResult::PutBack);
        assert_eq!(a.world.hand, None);
    }

    #[test]
    fn the_hand_empties_when_accepted_except_coins_that_persist() {
        let mut a = active();
        let c = carry(&mut a, "Candle", 1);
        let coins = carry(&mut a, "SilverCoins", 10);
        bar_click(&mut a.world, &c, true, false, "click");
        after_world_click(&mut a.world, &c, false);
        assert_eq!(a.world.hand.as_deref(), Some(c.as_str()), "refused: it stays");
        after_world_click(&mut a.world, &c, true);
        assert_eq!(a.world.hand, None, "accepted: emptied");
        assert!(a.world.items.get(&c).unwrap().carried_by("Connor"), "the item itself is not removed by this");
        bar_click(&mut a.world, &coins, true, false, "click");
        a.world.gold_persists = true;
        after_world_click(&mut a.world, &coins, true);
        assert_eq!(a.world.hand.as_deref(), Some(coins.as_str()), "goldCoinPersists 1: the coins stay");
        // A script deleted the held item: the hand is emptied.
        a.world.items.objects.iter_mut().find(|o| o.name == coins).unwrap().place = Place::Gone;
        assert_eq!(hand(&mut a.world), None);
        assert_eq!(a.world.hand, None);
    }
}
