//! `kq8 item-verify` (M3 step 3e-1): reads the item type list
//! `GAME/Kq/Invinfo.itl` (docs/behaviour/items.md rules 1 and 2) and
//! checks it against the rest of the data:
//!
//! - each type's model is found (a world's archives, or `GAME/Common`,
//!   where the viewer looks for models) and has a box in
//!   `GAME/patch/BOUNDS.KQ` (shape name lower-cased, as for objects;
//!   connor-physics.md Q39);
//! - every item type named by a world's scripts (`KQMonster::inventory`,
//!   `give`, `KQMonster::checkInventory`, `KQMonster::dropItem`,
//!   `KQObject::tossItems`) or object files (`invItemTypeName`,
//!   `acceptsInventory`) is found by the name rule.

use std::collections::{BTreeMap, BTreeSet};
use std::process::ExitCode;

use kq8_data::bounds::BoundingBoxes;
use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::items::ItemTypes;
use kq8_data::{GameDir, VolArchive};

use crate::log::Log;

const LIST: &str = "GAME/Kq/Invinfo.itl";

/// A data file's name for the log and its text.
struct Text {
    label: String,
    name: String,
    text: String,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("item-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("item-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let Some(path) = game.find(LIST) else {
        log.summary(format!("FAIL {LIST} not found"));
        return ExitCode::FAILURE;
    };
    let bytes = match std::fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            log.summary(format!("FAIL {}: {e}", path.display()));
            return ExitCode::FAILURE;
        }
    };
    let (list, problems) = ItemTypes::parse(&bytes);
    for p in &problems {
        log.summary(format!("problem: {p}"));
    }
    let mut classes: BTreeMap<&str, usize> = BTreeMap::new();
    let mut categories: BTreeMap<&str, usize> = BTreeMap::new();
    for t in &list.types {
        *classes.entry(&t.class).or_default() += 1;
        *categories.entry(&t.category).or_default() += 1;
    }
    let mut names: BTreeMap<String, usize> = BTreeMap::new();
    for t in &list.types {
        *names.entry(t.name.to_ascii_lowercase()).or_default() += 1;
    }
    let twice: Vec<&String> = names.iter().filter(|(_, n)| **n > 1).map(|(k, _)| k).collect();
    log.summary(format!(
        "{}: {} types read, numItems={:?}, {} problems; names found more than once (ignoring capitals): {twice:?}",
        path.display(),
        list.types.len(),
        list.declared,
        problems.len()
    ));
    log.summary(format!("classes: {classes:?}"));
    log.summary(format!("categories: {categories:?}"));

    // Every archive once: which world has which file names, and the
    // world's scripts and object files.
    let mut in_world: BTreeMap<String, BTreeSet<&str>> = BTreeMap::new();
    let mut texts: Vec<Text> = Vec::new();
    let mut other_lists = Vec::new();
    for world in WORLDS {
        for volume in VOLUMES {
            let Some(p) = game.volume_path(world, volume) else { continue };
            let vol = match VolArchive::open(&p) {
                Ok(v) => v,
                Err(e) => {
                    log.summary(format!("FAIL {}: {e}", p.display()));
                    continue;
                }
            };
            for e in vol.entries() {
                let lower = e.name.to_ascii_lowercase();
                if lower == "invinfo.itl" {
                    other_lists.push(format!("vol:{world}/{volume}/{}", e.name));
                }
                if lower.ends_with(".cs") || lower.ends_with(".kq") {
                    match vol.unpack(e) {
                        Ok(u) => texts.push(Text {
                            label: format!("vol:{world}/{volume}/{}", e.name),
                            name: lower.clone(),
                            text: u.data.iter().map(|&b| b as char).collect(),
                        }),
                        Err(err) => log.summary(format!("FAIL vol:{world}/{volume}/{}: {err}", e.name)),
                    }
                }
                in_world.entry(lower).or_default().insert(world);
            }
        }
    }
    for dir in ["GAME/Kq", "GAME/patch"] {
        let Some(d) = game.find(dir) else { continue };
        let Ok(entries) = std::fs::read_dir(&d) else { continue };
        for e in entries.flatten() {
            let name = e.file_name().to_string_lossy().to_string();
            let lower = name.to_ascii_lowercase();
            if lower == "invinfo.itl" && dir != "GAME/Kq" {
                other_lists.push(format!("{dir}/{name}"));
            }
            if lower.ends_with(".cs") || lower.ends_with(".kq") {
                if let Ok(b) = std::fs::read(e.path()) {
                    texts.push(Text { label: format!("{dir}/{name}"), name: lower, text: b.iter().map(|&b| b as char).collect() });
                }
            }
        }
    }
    log.summary(format!("other copies of Invinfo.itl: {other_lists:?}"));
    let common: BTreeSet<String> = game
        .find("GAME/Common")
        .and_then(|d| std::fs::read_dir(d).ok())
        .map(|r| r.flatten().map(|e| e.file_name().to_string_lossy().to_ascii_lowercase()).collect())
        .unwrap_or_default();
    let bounds = match game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) {
        Some(Ok(b)) => Some(BoundingBoxes::parse(&b).0),
        _ => {
            log.summary("GAME/patch/BOUNDS.KQ not found or unreadable; boxes not checked");
            None
        }
    };

    // Each type: model, box.
    let (mut no_model, mut no_box) = (Vec::new(), Vec::new());
    log.detail("");
    log.detail("types: key, name, id, category, shape (where found), box (BOUNDS.KQ, lower-cased shape), radius, translucency, land / pickup sound");
    for t in &list.types {
        let lower = t.shape.to_ascii_lowercase();
        let worlds: Vec<&str> = in_world.get(&lower).map(|w| w.iter().copied().collect()).unwrap_or_default();
        let where_ = match (common.contains(&lower), worlds.is_empty()) {
            (true, true) => "GAME/Common".to_string(),
            (true, false) => format!("GAME/Common and {worlds:?}"),
            (false, false) => format!("{worlds:?}"),
            (false, true) => {
                no_model.push(format!("{} ({})", t.name, t.shape));
                "NOT FOUND".to_string()
            }
        };
        let b = bounds.as_ref().and_then(|b| b.lookup(&lower));
        if bounds.is_some() && b.is_none() {
            no_box.push(format!("{} ({})", t.name, t.shape));
        }
        log.detail(format!(
            "{} {} id {:?} {} shape {} in {where_}; box {}; radius {:?} translucency {:?}; sounds {:?} / {:?}",
            t.key,
            t.name,
            t.id,
            t.category,
            t.shape,
            b.map_or("none".to_string(), |e| format!("{:?}..{:?} (item{})", e.min, e.max, e.item)),
            t.radius,
            t.translucency,
            t.land_sound,
            t.pickup_sound
        ));
    }
    log.summary(format!("types whose model is not found: {} {no_model:?}", no_model.len()));
    log.summary(format!("types without a box in BOUNDS.KQ: {} {no_box:?}", no_box.len()));

    // Every type name the data uses.
    let mut used: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut variables = 0usize;
    for f in &texts {
        let script = f.name.ends_with(".cs");
        for (n, raw) in f.text.lines().enumerate() {
            let at = format!("{} line {}", f.label, n + 1);
            if script {
                let line = raw.split('#').next().unwrap_or("");
                let w: Vec<&str> = line.split_whitespace().collect();
                let Some(first) = w.first() else { continue };
                let named: Vec<&str> = match first.to_ascii_lowercase().as_str() {
                    "kqmonster::inventory" | "kqmonster::checkinventory" | "kqmonster::dropitem" => w.get(2).copied().into_iter().collect(),
                    "give" => w.get(1).copied().into_iter().collect(),
                    "kqobject::tossitems" => w.get(3).map(|v| v.split(',').filter(|s| !s.is_empty()).collect()).unwrap_or_default(),
                    _ => Vec::new(),
                };
                for name in named {
                    if name.starts_with('$') {
                        variables += 1;
                    } else {
                        used.entry(name.to_string()).or_default().push(at.clone());
                    }
                }
            } else {
                let line = raw.trim();
                if line.starts_with(';') {
                    continue;
                }
                let Some((k, v)) = line.split_once('=') else { continue };
                let k = k.trim();
                if k.eq_ignore_ascii_case("invItemTypeName") || k.eq_ignore_ascii_case("acceptsInventory") {
                    // `acceptsInventory` can be a comma-separated list (Q62).
                    let v = v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("");
                    for name in v.split(',').filter(|s| !s.is_empty()) {
                        used.entry(name.to_string()).or_default().push(format!("{at} ({k})"));
                    }
                }
            }
        }
    }
    let uses: usize = used.values().map(Vec::len).sum();
    let missing: Vec<(&String, &Vec<String>)> = used.iter().filter(|(n, _)| list.find(n).is_none()).collect();
    log.summary(format!(
        "data: {} scripts and object files read; {uses} uses of {} different type names; {variables} uses of a $variable (not checked); not found: {}",
        texts.len(),
        used.len(),
        missing.len()
    ));
    for (name, places) in &missing {
        // Neither is covered by items.md rule 2 (Q62).
        let note = if name.parse::<i64>().is_ok_and(|n| list.types.iter().any(|t| t.id == Some(n))) {
            " (the id number of a type)"
        } else if name.eq_ignore_ascii_case("INVITEM_All") {
            " (acceptsInventory: any item?)"
        } else {
            ""
        };
        log.summary(format!("  NOT FOUND '{name}'{note} ({} uses), first at {}", places.len(), places[0]));
    }
    log.detail("");
    log.detail("every name used: name -> type found, uses, first place");
    for (name, places) in &used {
        log.detail(format!(
            "{name} -> {}, {} uses, first at {}",
            list.find(name).map_or("NOT FOUND", |t| t.name.as_str()),
            places.len(),
            places[0]
        ));
    }
    ExitCode::SUCCESS
}
