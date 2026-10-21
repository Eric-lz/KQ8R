//! The console's own words (`docs/DESIGN.md`, "the command console" step
//! 3): lists, Connor's state, and short forms of script commands. Ours,
//! not from the original game. Where a word acts on the game it goes
//! through a rule already built (a script command, a stats event, the
//! F3 / F4 switches, a character's death).

use kq8_core::item::Place;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;

pub const HELP: &str = "\
Console commands (not case-sensitive; Up/Down on the typing line recalls earlier ones)
 Looking around
  list                        what can be listed
  list items [word]           every item type in the game (filter by word)
  list weapons|armor|potions|quest [word]
  list creatures [word]       this world's characters: health, distance
  list doors|chests|objects [word]
  find <word>                 any named thing in this world with that word
  where                       Connor's place and turn (the numbers move takes)
 Connor
  inv                         what he carries
  stats                       health, damage, protection, experience, level
  give <item> [count]         into his inventory, e.g. give TempleSword
  spawn <item> [count]        lay it on the ground in front of him
  heal                        back to full health
  xp <n>                      add experience (he levels up as in the game)
  god                         god mode on/off (F3): no damage, no fall death
  goto <name>                 jump next to a creature, object, door, chest, item
  move Connor <flags> x y z 0 0 t
                              flags 0 set place and turn, 1 add to the place,
                              2 add to the turn, 3 both; t = turn in radians
                              e.g. move Connor 3 0 0 2000 (2,000 higher)
 Creatures
  ai                          creatures' AI off/on (F4)
  kill <name>                 it dies (and drops what it carries)
  kill near [distance]        every creature within distance (default 5000)
 Anything else
  Any script command the game's own scripts use runs as if a script ran it:
    KQSound::play <file.aud>           e.g. KQSound::play eat.aud
    setCombatLevel 1|2|3               easy / normal / hard fights
    handsOff 1 / handsOff 0            take / give back control
    deleteObject <name>                remove a thing from the world
    sendEvent 0 <door> KQDoorEvent 1   open a door (2 closes)
  The result is $Console::Return: 0 usually means done, 1 failed or unknown.
  Everything typed is also in kq8-artifacts/logs/console.log.";

const LISTS: &str = "\
list items | weapons | armor | potions | quest [word]   (item types)
list creatures | doors | chests | objects [word]        (in this world)";

/// The categories each item list shows (`Invinfo.itl`'s `category`
/// column); `None` = all.
fn item_categories(kind: &str) -> Option<Option<&'static [&'static str]>> {
    Some(match kind {
        "items" | "item" => None,
        "weapons" | "weapon" => Some(&["ShortWeapon", "LongWeapon"][..]),
        "armor" | "armour" => Some(&["Armor", "FullArmor", "GodArmor", "Helmet", "Gloves", "FootWear"][..]),
        "potions" | "potion" | "health" => Some(&["HealthItem"][..]),
        "quest" => Some(&["QuestItem", "MaskPiece", "MagicMap"][..]),
        _ => return None,
    })
}

const STAT_NAMES: [&str; 5] = ["health", "hand", "ranged", "protection", "experience"];

/// Handles a console word; `None` = not ours (the line runs as a script
/// command).
pub fn run(line: &str, sim: &mut DoorSim, connor: Option<&ConnorSim>) -> Option<Vec<String>> {
    let words: Vec<&str> = line.split_whitespace().collect();
    let first = words.first()?.to_ascii_lowercase();
    let arg = |k: usize| words.get(k).map(|w| w.to_ascii_lowercase());
    let at = connor.map(|c| (c.mover.position, c.walker.heading));
    Some(match first.as_str() {
        "help" | "?" => vec![HELP.to_string()],
        "where" => vec![match at {
            Some((p, h)) => format!("Connor is at {:.0} {:.0} {:.0}, turn {h:.4} (move Connor 0 {:.0} {:.0} {:.0} 0 0 {h:.4} puts him back here)", p[0], p[1], p[2], p[0], p[1], p[2]),
            None => "no Connor in this view".to_string(),
        }],
        "list" => list(sim, arg(1).as_deref(), arg(2).as_deref(), at.map(|a| a.0)),
        "find" => match arg(1) {
            Some(word) => {
                let mut out = Vec::new();
                for kind in ["creatures", "doors", "chests", "objects", "lying"] {
                    out.extend(things(sim, kind, Some(&word), at.map(|a| a.0)).into_iter().map(|(n, _, d)| format!("  {kind:<9} {n}{d}")));
                }
                if out.is_empty() {
                    out.push(format!("nothing in this world named with '{word}' (item types: list items {word})"));
                }
                out
            }
            None => vec!["usage: find <word>".into()],
        },
        "inv" | "inventory" => inventory(sim),
        "stats" => stats(sim),
        "heal" => {
            let Some(c) = sim.active.world.connor_stats else { return Some(vec!["Connor's stats are unknown (no Connor.kq)".into()]) };
            let missing = c.max_health() - c.stats.0[0];
            if missing <= 0 {
                vec![format!("Connor's health is already full ({})", c.stats.0[0])]
            } else {
                script(sim, &format!("sendEvent 0 Connor KQStatsChangeRequestEvent {missing} 0 0 0 0"))
            }
        }
        "xp" => match words.get(1).and_then(|w| w.parse::<i64>().ok()) {
            Some(n) => script(sim, &format!("sendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 {n}")),
            None => vec!["usage: xp <number>, e.g. xp 100".into()],
        },
        "god" => {
            crate::door_view::flip_god(sim, "console");
            vec![format!("god mode is {}", if sim.active.world.god_mode { "on" } else { "off" })]
        }
        "ai" => {
            crate::door_view::flip_ai(sim, "console");
            vec![format!("the creatures' AI is {}", if sim.active.world.ai_off { "off" } else { "on" })]
        }
        "goto" => match (arg(1), at) {
            (Some(name), Some((p, _))) => goto(sim, &name, p),
            (None, _) => vec!["usage: goto <name> (names: list creatures, list objects, find <word>)".into()],
            (_, None) => vec!["no Connor in this view".into()],
        },
        "spawn" => match (words.get(1), at) {
            (Some(kind), Some((p, h))) => {
                let count = words.get(2).and_then(|w| w.parse::<i64>().ok()).unwrap_or(1).max(1);
                // His forward direction (heading 0 faces -Y).
                let (s, c) = h.sin_cos();
                let x = p[0] + 600.0 * s;
                let y = p[1] - 600.0 * c;
                script(sim, &format!("KQMonster::inventory world {kind} {count} {x:.0} {y:.0} {:.0}", p[2] + 100.0))
            }
            (None, _) => vec!["usage: spawn <item> [count], e.g. spawn Crystal 5".into()],
            (_, None) => vec!["no Connor in this view".into()],
        },
        "kill" => match arg(1).as_deref() {
            Some("near") => {
                let range = words.get(2).and_then(|w| w.parse::<f32>().ok()).unwrap_or(5000.0);
                let Some((p, _)) = at else { return Some(vec!["no Connor in this view".into()]) };
                let near: Vec<String> = sim
                    .active
                    .world
                    .anims
                    .iter()
                    .filter(|a| !a.gone && a.character.as_deref().and_then(|c| c.fight.as_ref()).is_some_and(|f| f.dying.is_none()))
                    .filter(|a| distance(a.player.base.position, p) <= range)
                    .map(|a| a.player.name.clone())
                    .collect();
                if near.is_empty() {
                    vec![format!("no living creature within {range:.0}")]
                } else {
                    near.iter().map(|n| kill(sim, n)).collect()
                }
            }
            Some(name) => vec![kill(sim, name)],
            None => vec!["usage: kill <name> | kill near [distance]".into()],
        },
        _ => return None,
    })
}

/// Runs a script command and returns the log lines it wrote and its
/// result.
pub fn script(sim: &mut DoorSim, line: &str) -> Vec<String> {
    let before = sim.active.world.log.len();
    let result = sim.active.test_command(line, "console");
    let mut out: Vec<String> = sim.active.world.log.get(before..).unwrap_or_default().iter().map(|w| format!("  {w}")).collect();
    let first = line.split_whitespace().next().unwrap_or("");
    out.push(match result {
        Some(r) => format!("  result {r}"),
        None => format!("  unknown command '{first}' ('help' for the list)"),
    });
    out
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

fn kill(sim: &mut DoorSim, name: &str) -> String {
    match sim.active.world.console_kill(name) {
        Ok(n) => format!("{n} dies (its drop and its body as in a fight; fight.log)"),
        Err(e) => e,
    }
}

fn list(sim: &DoorSim, kind: Option<&str>, word: Option<&str>, connor: Option<[f32; 3]>) -> Vec<String> {
    let Some(kind) = kind else { return vec![LISTS.to_string()] };
    if let Some(cats) = item_categories(kind) {
        let mut out = Vec::new();
        for t in &sim.active.world.items.types.types {
            if cats.is_some_and(|c| !c.iter().any(|c| c.eq_ignore_ascii_case(&t.category))) {
                continue;
            }
            let short = t.name.strip_prefix("INVITEM_").unwrap_or(&t.name);
            if word.is_some_and(|w| !short.to_ascii_lowercase().contains(w) && !t.category.to_ascii_lowercase().contains(w)) {
                continue;
            }
            let adds: Vec<String> = t.stats.iter().zip(STAT_NAMES).filter(|(v, _)| **v != 0).map(|(v, n)| format!("{n} {v:+}")).collect();
            let adds = if adds.is_empty() { String::new() } else { format!("  ({})", adds.join(", ")) };
            out.push(format!("  {short:<22} {:<12}{adds}", t.category));
        }
        out.push(format!("{} item types; give <name> or spawn <name> (items of later worlds may have no model here)", out.len()));
        return out;
    }
    let kind = match kind {
        "creature" | "creatures" | "monsters" | "characters" => "creatures",
        "door" | "doors" => "doors",
        "chest" | "chests" => "chests",
        "object" | "objects" => "objects",
        _ => return vec![format!("can't list '{kind}'"), LISTS.to_string()],
    };
    let found = things(sim, kind, word, connor);
    let mut out: Vec<String> = found.iter().map(|(n, extra, d)| format!("  {n:<20}{extra}{d}")).collect();
    out.push(format!("{} {kind}; goto <name> takes you there", found.len()));
    out
}

/// Named things of one kind in this world, nearest first: name, details,
/// distance text.
fn things(sim: &DoorSim, kind: &str, word: Option<&str>, connor: Option<[f32; 3]>) -> Vec<(String, String, String)> {
    let w = &sim.active.world;
    let mut found: Vec<(String, String, [f32; 3])> = Vec::new();
    match kind {
        "creatures" => {
            for a in w.anims.iter().filter(|a| !a.gone) {
                let Some(c) = a.character.as_deref() else { continue };
                let extra = match c.fight.as_ref() {
                    Some(f) if f.dying.is_some() => " dead".to_string(),
                    Some(f) => format!(" health {}", f.stats.0[0]),
                    None => " (doesn't fight)".to_string(),
                };
                found.push((a.player.name.clone(), extra, a.player.base.position));
            }
        }
        "doors" => found.extend(w.doors.iter().map(|d| (d.door.name.clone(), String::new(), d.door.player.base.position))),
        "chests" => found.extend(w.chests.iter().map(|c| (c.chest.name.clone(), String::new(), c.chest.player.base.position))),
        "objects" => {
            found.extend(w.objects.iter().map(|o| (o.name().to_string(), String::new(), o.player.base.position)));
            found.extend(w.anims.iter().filter(|a| !a.gone && a.character.is_none()).map(|a| (a.player.name.clone(), String::new(), a.player.base.position)));
        }
        "lying" => {
            for o in &w.items.objects {
                if let Place::World { position, .. } = o.place {
                    if !o.hidden {
                        found.push((o.name.clone(), format!(" ({})", o.kind.strip_prefix("INVITEM_").unwrap_or(&o.kind)), position));
                    }
                }
            }
        }
        _ => {}
    }
    found.retain(|(n, extra, _)| word.is_none_or(|w| n.to_ascii_lowercase().contains(w) || extra.to_ascii_lowercase().contains(w)));
    let dist = |p: [f32; 3]| connor.map_or(0.0, |c| distance(p, c));
    found.sort_by(|a, b| dist(a.2).total_cmp(&dist(b.2)));
    found.into_iter().map(|(n, e, p)| (n, e, connor.map_or(String::new(), |c| format!("  {:.0} away", distance(p, c))))).collect()
}

/// Where a named thing is: a character or animated object, an object, a
/// door, a chest, an item lying in the world.
fn place_of(sim: &DoorSim, name: &str) -> Option<(String, [f32; 3])> {
    let w = &sim.active.world;
    let same = |n: &str| n.eq_ignore_ascii_case(name);
    w.anims
        .iter()
        .filter(|a| !a.gone)
        .find(|a| same(&a.player.name))
        .map(|a| (a.player.name.clone(), a.player.base.position))
        .or_else(|| w.objects.iter().find(|o| same(o.name())).map(|o| (o.name().to_string(), o.player.base.position)))
        .or_else(|| w.doors.iter().find(|d| same(&d.door.name)).map(|d| (d.door.name.clone(), d.door.player.base.position)))
        .or_else(|| w.chests.iter().find(|c| same(&c.chest.name)).map(|c| (c.chest.name.clone(), c.chest.player.base.position)))
        .or_else(|| {
            w.items.objects.iter().find(|o| same(&o.name)).and_then(|o| match o.place {
                Place::World { position, .. } => Some((o.name.clone(), position)),
                _ => None,
            })
        })
}

/// 1,000 units from it on Connor's side, 500 higher, turned toward it.
fn goto(sim: &mut DoorSim, name: &str, from: [f32; 3]) -> Vec<String> {
    let Some((name, t)) = place_of(sim, name) else {
        return vec![format!("nothing named '{name}' here (find <word> searches)")];
    };
    let (dx, dy) = (from[0] - t[0], from[1] - t[1]);
    let len = (dx * dx + dy * dy).sqrt();
    let (ux, uy) = if len > 1.0 { (dx / len, dy / len) } else { (0.0, 1.0) };
    let (x, y, z) = (t[0] + 1000.0 * ux, t[1] + 1000.0 * uy, t[2] + 500.0);
    // Heading toward the target (heading 0 faces -Y; connor_auto.rs).
    let heading = (t[0] - x).atan2(-(t[1] - y));
    let mut out = vec![format!("to {name} at {:.0} {:.0} {:.0}", t[0], t[1], t[2])];
    out.extend(script(sim, &format!("move Connor 0 {x:.0} {y:.0} {z:.0} 0 0 {heading:.4}")));
    out
}

fn inventory(sim: &DoorSim) -> Vec<String> {
    let mut out: Vec<String> = sim
        .active
        .world
        .items
        .objects
        .iter()
        .filter(|o| matches!(&o.place, Place::Carried(who) if who.eq_ignore_ascii_case("Connor")))
        .map(|o| {
            let short = o.kind.strip_prefix("INVITEM_").unwrap_or(&o.kind);
            let ready = if sim.active.world.weapon_ready.as_deref().is_some_and(|r| r.eq_ignore_ascii_case(&o.name)) { "  (ready)" } else { "" };
            format!("  {short:<22} x{}  [{}]{ready}", o.quantity, o.name)
        })
        .collect();
    if out.is_empty() {
        out.push("Connor carries nothing".into());
    }
    out
}

fn stats(sim: &DoorSim) -> Vec<String> {
    let w = &sim.active.world;
    let mut out = Vec::new();
    match w.connor_stats {
        Some(c) => {
            out.push(format!("level {}, health {} of {}", c.level, c.stats.0[0], c.max_health()));
            out.push(format!("hand {}, ranged {}, protection {}, experience {}", c.stats.0[1], c.stats.0[2], c.stats.0[3], c.stats.0[4]));
        }
        None => out.push("Connor's stats are unknown (no Connor.kq)".into()),
    }
    out.push(format!(
        "weapon ready: {}; god mode {}; creatures' AI {}; combat level {}",
        w.weapon_ready.as_deref().unwrap_or("none"),
        if w.god_mode { "on" } else { "off" },
        if w.ai_off { "off" } else { "on" },
        w.combat_level
    ));
    out
}
