//! The journey (M6 6e-3): what goes with Connor from one world to the
//! next, taken from the world he leaves and given to the one he arrives
//! in.
//!
//! Rules from `docs/behaviour/world-change.md` (A, 2026-10-21; the
//! user's check in the original 2026-10-22):
//!
//! - Rule 17: Connor travels whole: his statistics, health, level and
//!   experience (with damage and protection, all in his stats); his
//!   inventory, each item with its own state (counts, applied or not, the
//!   item in hand, the ready weapon and the weapon mode); his flags
//!   (`setConnorFlag`); the magic map's per-map marks and the view stored
//!   on him (`magic-map.md` rules 22, 33, 37). The camera, the map and the
//!   music are the new world's own.
//! - Rule 18: the counter for `NoName<n>` names and the combat level go
//!   along too (`GameState.kq`).
//! - Rule 19 (the user's check): running potions keep their remaining
//!   time.
//! - Rule 13: script variables are per world by name; those named after
//!   the world left (or `KQWorld`) stay with that world, the others stay
//!   in the game's memory, so they go along here.
//!
//! Ours (marked STAND-IN where the note does not say): the potions'
//! timers count updates of the world they run in, so they are moved onto
//! the new world's clock with the same time left; the rope's settings
//! belong to rope places, so they start empty in the new world (rule 19:
//! not traced where the original keeps them; the rope can't be in use
//! during a world change); other worlds' maps (`other_maps`) go along
//! until kept worlds exist (6e-8). The test switches (god mode, AI off)
//! go along.

use std::collections::BTreeMap;

use kq8_data::world_script::Vars;

use crate::active::WorldState;
use crate::item::ItemObject;
use crate::potions::{Running, StatusIcon};
use crate::talk::CONNOR;

/// Everything that goes with Connor (rules 17–19).
#[derive(Debug, Clone, PartialEq)]
pub struct Journey {
    /// The world left.
    pub from: String,
    pub stats: Option<crate::stats::ConnorStats>,
    /// His item objects, as they were (place `Carried("Connor")`).
    pub items: Vec<ItemObject>,
    pub hand: Option<String>,
    pub weapon_ready: Option<String>,
    pub weapon_mode: crate::weapons::WeaponMode,
    pub last_away_ranged: bool,
    pub projectile: Option<String>,
    pub shoot_offset: [f32; 3],
    /// Running potions (their `next_tick` 0), with the updates left until
    /// their next second (the old world's update number means nothing in
    /// the next).
    pub potions: Vec<(Running, u64)>,
    /// Experience due for items added: seconds from now, amount, type.
    pub pickup_experience: Vec<(f64, i64, String)>,
    pub flags: BTreeMap<String, bool>,
    pub in_temple4: bool,
    pub map_marks: crate::magic_map::ConnorMapMarks,
    pub map_view: crate::magic_map::View,
    pub other_maps: BTreeMap<usize, (String, kq8_data::magic_map::MapKeys, String)>,
    pub combat_level: f32,
    /// The next `NoName<n>` number (rule 18).
    pub next_no_name: usize,
    /// The one-off tips already given (game-wide).
    pub helps_asked: std::collections::BTreeSet<&'static str>,
    pub connor_tips: [bool; 2],
    pub kill_tip: bool,
    pub god_mode: bool,
    pub ai_off: bool,
    /// The game-wide script variables (rule 13): name, value.
    pub vars: Vec<(String, String)>,
}

/// A variable that stays with world `world` (rule 13): its name starts
/// with the world's name or with `KQWorld` (not `KQWorld::SaveGameName`).
pub fn belongs_to_world(name: &str, world: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    if lower == "kqworld::savegamename" {
        return false;
    }
    lower.starts_with("kqworld") || (!world.is_empty() && lower.starts_with(&world.to_ascii_lowercase()))
}

impl Journey {
    /// Takes the journey from the world Connor leaves (`from`).
    pub fn take(w: &WorldState, vars: &Vars, from: &str) -> Journey {
        let now = w.updates;
        Journey {
            from: from.to_string(),
            stats: w.connor_stats,
            items: w.items.objects.iter().filter(|o| o.carried_by(CONNOR)).cloned().collect(),
            hand: w.hand.clone(),
            weapon_ready: w.weapon_ready.clone(),
            weapon_mode: w.weapon_mode,
            last_away_ranged: w.last_away_ranged,
            projectile: w.connor_projectile.clone(),
            shoot_offset: w.connor_shoot_offset,
            potions: w.potions.iter().map(|r| (Running { next_tick: 0, ..r.clone() }, r.next_tick.saturating_sub(now))).collect(),
            pickup_experience: w.pickup_experience.iter().map(|(t, n, k)| (t - w.now(), *n, k.clone())).collect(),
            flags: w.connor_flags.clone(),
            in_temple4: w.connor_in_temple4,
            map_marks: w.map_marks.clone(),
            map_view: w.map_view,
            other_maps: w.other_maps.clone(),
            combat_level: w.combat_level,
            next_no_name: w.items.no_name_counter(),
            helps_asked: w.helps_asked.clone(),
            connor_tips: w.connor_tips,
            kill_tip: w.kill_tip,
            god_mode: w.god_mode,
            ai_off: w.ai_off,
            vars: vars.iter().filter(|(n, _)| !belongs_to_world(n, from)).map(|(n, v)| (n.to_string(), v.to_string())).collect(),
        }
    }

    /// Gives the journey to the world Connor arrives in: his own start
    /// items there are replaced by the carried ones. Variables the new
    /// world's start already set are kept (it runs after them in the
    /// original). Returns lines for the log.
    pub fn give(&self, w: &mut WorldState, vars: &mut Vars) -> Vec<String> {
        let mut lines = Vec::new();
        // His start items in this world (`Connor.kq`'s inventory) go.
        let dropped: Vec<String> = w.items.objects.iter().filter(|o| o.carried_by(CONNOR)).map(|o| o.name.clone()).collect();
        w.items.objects.retain(|o| !o.carried_by(CONNOR));
        let mut clashes = Vec::new();
        for item in &self.items {
            if w.items.index(&item.name).is_some() {
                clashes.push(item.name.clone());
            }
            w.items.objects.push(item.clone());
        }
        w.items.start_no_names_at(self.next_no_name);
        lines.push(format!("journey: {} items carried in (this world's start items {dropped:?} replaced); NoName counter at {}", self.items.len(), w.items.no_name_counter()));
        if !clashes.is_empty() {
            lines.push(format!("journey: WARNING: carried items with a name this world already has: {clashes:?}"));
        }
        w.connor_stats = self.stats;
        // The bottom bar takes the numbers after a stats event
        // (`interface.md` rule 24).
        w.connor_stats_events += 1;
        w.hand = self.hand.clone();
        w.weapon_ready = self.weapon_ready.clone();
        w.weapon_mode = self.weapon_mode;
        w.last_away_ranged = self.last_away_ranged;
        if self.weapon_ready.is_some() {
            // Its stance at once, nothing played (as a swap, 5d fix 11).
            w.weapon_swapped = true;
        }
        w.connor_projectile = self.projectile.clone();
        w.connor_shoot_offset = self.shoot_offset;
        // Potions on this world's clock, the same time left (rule 19).
        w.potions = self.potions.iter().map(|(r, until)| Running { next_tick: w.updates + until, ..r.clone() }).collect();
        for r in &w.potions {
            if let Some(k) = r.place {
                w.status_icons[k] = Some(StatusIcon::Potion(r.item.clone()));
            }
        }
        let now = w.now();
        w.pickup_experience = self.pickup_experience.iter().map(|(t, n, k)| (now + t, *n, k.clone())).collect();
        w.connor_flags = self.flags.clone();
        w.connor_in_temple4 = self.in_temple4;
        w.map_marks = self.map_marks.clone();
        w.map_view = self.map_view;
        w.other_maps = self.other_maps.clone();
        w.combat_level = self.combat_level;
        w.helps_asked = self.helps_asked.clone();
        w.connor_tips = self.connor_tips;
        w.kill_tip = self.kill_tip;
        w.god_mode = self.god_mode;
        w.ai_off = self.ai_off;
        let mut set = 0;
        for (n, v) in &self.vars {
            if vars.get(n).is_none() {
                vars.set(n, v.clone());
                set += 1;
            }
        }
        lines.push(format!("journey: {set} of {} game-wide variables set (the others this world's start set itself)", self.vars.len()));
        lines
    }

    /// The journey's contents, for `world.log`.
    pub fn describe(&self) -> Vec<String> {
        let mut out = vec![format!("journey from {}:", self.from)];
        match self.stats {
            Some(c) => out.push(format!("  stats: level {}, health {}, hand {}, ranged {}, protection {}, experience {}", c.level, c.stats.0[0], c.stats.0[1], c.stats.0[2], c.stats.0[3], c.stats.0[4])),
            None => out.push("  stats: none".into()),
        }
        for o in &self.items {
            let applied = if o.third == -1 { "" } else { ", applied" };
            out.push(format!("  item {} ({}) x{}{applied}", o.name, o.kind, o.quantity));
        }
        out.push(format!("  in hand {:?}; ready weapon {:?} ({:?}); projectile {:?}", self.hand, self.weapon_ready, self.weapon_mode, self.projectile));
        for (r, until) in &self.potions {
            out.push(format!("  potion {} ({}) {} of {} s left, next second in {until} updates, icon place {:?}", r.item, r.kind, r.left, r.duration, r.place));
        }
        out.push(format!("  flags {:?}; InTemple4 {}; map marks {:?}; other maps {:?}", self.flags, self.in_temple4, self.map_marks, self.other_maps.keys().collect::<Vec<_>>()));
        out.push(format!("  combat level x{}; next NoName {}; god {}, AI off {}; {} game-wide variables", self.combat_level, self.next_no_name, self.god_mode, self.ai_off, self.vars.len()));
        out
    }
}

/// Connor's things as the journey sees them, for comparing two worlds
/// (`world.log`): the same before leaving and after arriving.
pub fn connor_summary(w: &WorldState) -> String {
    let mut items: Vec<String> = w.items.objects.iter().filter(|o| o.carried_by(CONNOR)).map(|o| format!("{}:{}x{}{}", o.name, o.kind, o.quantity, if o.third == -1 { "" } else { "*" })).collect();
    items.sort();
    let potions: Vec<String> = w.potions.iter().map(|r| format!("{}:{}s", r.item, r.left)).collect();
    format!(
        "stats {:?}; items [{}]; hand {:?}; ready {:?}; potions [{}]; flags {:?}",
        w.connor_stats.map(|c| (c.level, c.stats.0)),
        items.join(", "),
        w.hand,
        w.weapon_ready,
        potions.join(", "),
        w.connor_flags
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::active::Active;
    use crate::item::Place;
    use crate::stats::{ConnorStats, Stats};

    fn place_of(w: &WorldState, name: &str) -> Option<Place> {
        w.items.get(name).map(|o| o.place.clone())
    }

    /// Two item types in `Invinfo.itl`'s layout.
    fn items() -> crate::item::Items {
        let mut text = String::from("[InventoryItemTypeList]
numItems=2
");
        for (n, (name, cat, dur)) in [("INVITEM_Mushroom", "HealthItem", -1), ("INVITEM_Crystal", "QuestItem", 0)].iter().enumerate() {
            text.push_str(&format!("item{n}=	KQInventoryItemType	{name}	1	0	0	0	0	0	1	{cat}	IDBMP_X	1 0 0 1	NoModeChange	{dur}	1	none	none	none	x.kqs	none	1	256
"));
        }
        crate::item::Items::new(kq8_data::items::ItemTypes::parse(text.as_bytes()).0, [])
    }

    fn world() -> Active {
        let mut a = Active::new(Vars::default(), Vec::new(), Vec::new(), Vec::<String>::new(), Vec::<(String, Vec<u8>)>::new());
        a.world.items = items();
        a
    }

    #[test]
    fn world_variables_stay_with_their_world() {
        assert!(belongs_to_world("Daventry::girlhut::FirstTime", "Daventry"));
        assert!(belongs_to_world("KQWorld::MaxMonsters", "Swamp"));
        assert!(!belongs_to_world("KQWorld::SaveGameName", "Daventry"));
        assert!(!belongs_to_world("Swamp::Thing", "Daventry"));
        assert!(!belongs_to_world("KQGame::runOptimal", "Daventry"));
    }

    #[test]
    fn taken_and_given_back_unchanged() {
        let mut a = world();
        a.world.connor_stats = Some(ConnorStats { stats: Stats([80, 60, 19, 11, 1234]), level: 8 });
        a.world.items.make("Mushroom", 3, Place::Carried("Connor".into())).unwrap();
        a.world.items.make("Crystal", 1, Place::World { position: [1.0, 2.0, 3.0], angles: [0.0; 3] }).unwrap();
        a.world.hand = Some("NoName0".into());
        a.world.connor_flags.insert("BeenToCastle".into(), true);
        a.world.combat_level = 1.5;
        a.world.updates = 100;
        a.world.potions.push(Running { next_tick: 110, left: 42, place: Some(2), ..Running::new("NoName0", "INVITEM_Mushroom", 60, 95) });
        a.scripts.vars.set("Daventry::girlhut::FirstTime", "0".into());
        a.scripts.vars.set("KQGame::Something", "7".into());
        let j = Journey::take(&a.world, &a.scripts.vars, "Daventry");
        assert_eq!(j.items.len(), 1, "only Connor's items");
        assert_eq!(j.potions[0].1, 10, "10 updates to the next second");
        assert_eq!(j.vars, vec![("KQGame::Something".to_string(), "7".to_string())], "Daventry's own variables stay");

        // A new world: its own start item for Connor, a clock at 3.
        let mut b = world();
        b.world.items.make("Crystal", 1, Place::Carried("Connor".into())).unwrap();
        b.world.updates = 3;
        b.scripts.vars.set("KQGame::Something", "9".into());
        let lines = j.give(&mut b.world, &mut b.scripts.vars);
        assert!(lines[0].contains("1 items carried in"), "{lines:?}");
        assert_eq!(b.world.potions[0].next_tick, 13, "the same 10 updates on the new clock");
        assert_eq!(b.world.status_icons[2], Some(StatusIcon::Potion("NoName0".into())));
        assert_eq!(b.scripts.vars.get("KQGame::Something"), Some("9"), "the new world's own value wins");
        assert_eq!(place_of(&b.world, "NoName0"), Some(Place::Carried("Connor".into())));
        assert!(b.world.items.no_name_counter() >= 2, "no new name repeats a carried one");
        // Taken again from the new world: the same journey (apart from
        // where it comes from and its variables).
        let again = Journey::take(&b.world, &b.scripts.vars, "Swamp");
        assert_eq!((again.stats, &again.items, &again.hand, &again.flags, again.combat_level), (j.stats, &j.items, &j.hand, &j.flags, j.combat_level));
        assert_eq!(again.potions, j.potions);
        assert_eq!(connor_summary(&b.world), connor_summary(&a.world));
    }
}
