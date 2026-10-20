//! `kq8 character-verify` (M4 step 4d-1): reads every world's characters
//! (`docs/behaviour/characters.md` rules 1, 3, 4, 6, 9, 25, 27) and checks
//! them against the rest of the data:
//!
//! - every character file (`classType=KQMonster` or `KQGeometryTalker`):
//!   its movement list (`animFile`, from the world's archives, else
//!   `GAME/Kq`), whether it has `stop`, `Birth`, `PreBirth`, gestures and
//!   (for geometry talkers) the mouth movements; the movement shown at
//!   load; its `Stop` model's frame count; its lips and whether the
//!   `Stop` model carries the face texture; its box;
//! - every monster-list line (`KQMonsterManager`): its type in
//!   `GAME/Kq/Monstats.mtl`, the type's file in the world, its box.
//!
//! Boxes (rule 9): `GAME/patch/BOUNDS.KQ` by the type (monster list) or
//! `monsterType`, else by the object's name, exact (capitals count).
//!
//! Log: `kq8-artifacts/logs/character-verify.log`.

use std::collections::BTreeMap;
use std::process::ExitCode;

use kq8_data::bounds::BoundingBoxes;
use kq8_data::characters::{AiSettings, AnimList, CharacterKeys, Class, Lips, MonsterList, MonsterTypes, Wander, GEOMETRY_MOUTH, GESTURES};
use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::{kq_text, GameDir, Shape, VolArchive};

use crate::log::Log;

const TYPES: &str = "GAME/Kq/Monstats.mtl";
const BOUNDS: &str = "GAME/patch/BOUNDS.KQ";

/// A world's archives.
struct World {
    name: String,
    vols: Vec<VolArchive>,
}

impl World {
    /// The first entry called `name` (any case) in any archive.
    fn load(&self, name: &str) -> Option<Vec<u8>> {
        self.vols.iter().find_map(|a| a.find(name).and_then(|e| a.unpack(e).ok()).map(|u| u.data))
    }

    /// Every `.kq` entry: name and bytes.
    fn kq_files(&self) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for a in &self.vols {
            for e in a.entries() {
                if e.name.to_ascii_lowercase().ends_with(".kq") {
                    if let Ok(u) = a.unpack(e) {
                        out.push((e.name.clone(), u.data));
                    }
                }
            }
        }
        out
    }
}

/// Totals over all worlds.
#[derive(Default)]
struct Totals {
    files: BTreeMap<&'static str, usize>,
    lines: usize,
    problems: usize,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("character-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("character-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let read = |rel: &str| game.find(rel).and_then(|p| std::fs::read(p).ok());
    let Some(type_bytes) = read(TYPES) else {
        log.summary(format!("FAIL {TYPES} not found"));
        return ExitCode::FAILURE;
    };
    let (types, problems) = MonsterTypes::parse(&type_bytes);
    for p in &problems {
        log.summary(format!("{TYPES}: problem: {p}"));
    }
    log.summary(format!("{TYPES}: {} types read, numTypes={:?}, {} problems", types.types.len(), types.declared, problems.len()));
    let bounds = match read(BOUNDS) {
        Some(b) => BoundingBoxes::parse(&b).0,
        None => {
            log.summary(format!("FAIL {BOUNDS} not found"));
            return ExitCode::FAILURE;
        }
    };
    let mut totals = Totals { problems: problems.len(), ..Default::default() };
    for name in WORLDS {
        let vols = VOLUMES.iter().filter_map(|v| game.volume_path(name, v)).filter_map(|p| VolArchive::open(&p).ok()).collect();
        let world = World { name: name.to_string(), vols };
        verify_world(game, &world, &types, &bounds, &mut log, &mut totals);
    }
    log.summary(format!(
        "ALL WORLDS: character files {:?}, monster-list lines {}, problems {}",
        totals.files, totals.lines, totals.problems
    ));
    ExitCode::SUCCESS
}

/// The box rule 9 finds: by `first` (type or `monsterType`), else by the
/// object's name; exact.
fn find_box(bounds: &BoundingBoxes, first: Option<&str>, name: &str) -> String {
    let show = |how: &str, key: &str, e: kq8_data::bounds::BoxEntry| {
        format!("box by {how} '{key}': ({:.0}, {:.0}, {:.0}) .. ({:.0}, {:.0}, {:.0})", e.min[0], e.min[1], e.min[2], e.max[0], e.max[1], e.max[2])
    };
    if let Some(t) = first {
        if let Some(e) = bounds.lookup(t) {
            return show("type", t, e);
        }
    }
    match bounds.lookup(name) {
        Some(e) => show("name", name, e),
        None => format!("NO BOX (type {first:?}, name '{name}')"),
    }
}

fn verify_world(game: &GameDir, world: &World, types: &MonsterTypes, bounds: &BoundingBoxes, log: &mut Log, totals: &mut Totals) {
    log.detail(format!("== {} ==", world.name));
    let mut classes: BTreeMap<&'static str, usize> = BTreeMap::new();
    let mut lists = Vec::new();
    let mut problems = 0usize;
    let mut with = BTreeMap::<&str, usize>::new();
    for (file_name, bytes) in world.kq_files() {
        let file = kq_text::parse(&bytes);
        let Some(s) = file.first() else { continue };
        if s.get("classType").is_some_and(|c| c.trim().eq_ignore_ascii_case("KQMonsterManager")) {
            lists.push((file_name.clone(), file.clone()));
            continue;
        }
        let Some(keys) = CharacterKeys::from_section(s) else { continue };
        *classes.entry(keys.class.name()).or_default() += 1;
        let mut line = format!("{file_name} [{}] {}: flags {:#x}", s.name, keys.class.name(), keys.flags);
        if !keys.gravity {
            line += ", gravity off";
        }
        if keys.no_turn {
            line += ", noTurn";
        }
        line += &format!(", wanders {}, shadowMask {}", u8::from(keys.wanders), keys.shadow_mask);
        line += &format!("; movement rules from its keys: {}", AiSettings::from_keys(&keys).describe());
        if let Some((a, b)) = &keys.skin {
            line += &format!(", skin {a} -> {b}");
        }
        log.detail(line);
        // The movement list.
        let Some(anm_name) = &keys.anim_file else {
            log.detail("  PROBLEM: no animFile");
            problems += 1;
            continue;
        };
        let anm_bytes = world.load(anm_name).or_else(|| game.find(&format!("GAME/Kq/{anm_name}")).and_then(|p| std::fs::read(p).ok()));
        let Some(anm_bytes) = anm_bytes else {
            log.detail(format!("  PROBLEM: animFile {anm_name} not found in the world or GAME/Kq"));
            problems += 1;
            continue;
        };
        let list = match AnimList::parse(&anm_bytes) {
            Ok(l) => l,
            Err(e) => {
                log.detail(format!("  PROBLEM: {anm_name}: {e}"));
                problems += 1;
                continue;
            }
        };
        let gestures = GESTURES.iter().filter(|g| list.has(g)).count();
        let mouth = GEOMETRY_MOUTH.iter().filter(|m| list.has(m)).count();
        let birth = list.has("Birth");
        let names: Vec<&str> = list.movements.iter().map(|(n, _)| n.as_str()).collect();
        log.detail(format!(
            "  {anm_name} [{}]: {} movements {names:?}; stop {}, Birth {birth}, PreBirth {}, gestures {gestures}/8, mouth movements {mouth}/8",
            list.name,
            list.movements.len(),
            list.has("stop"),
            list.has("PreBirth")
        ));
        if birth {
            *with.entry("Birth (not drawn until born)").or_default() += 1;
        }
        if gestures > 0 {
            *with.entry("gestures").or_default() += 1;
        }
        if keys.class == Class::GeometryTalker && mouth < 8 {
            log.detail(format!("  PROBLEM: a geometry talker with only {mouth} of the 8 mouth movements"));
            problems += 1;
        }
        for (n, l) in &list.movements {
            for p in l.cue_problems.iter().chain(&l.transition_problems) {
                log.detail(format!("  {n}: problem: {p}"));
            }
            if !l.transitions.is_empty() {
                let t: Vec<String> = l.transitions.iter().map(|t| format!("{} -> {} via {} at {}", t.from, t.to, t.via, t.to_phase)).collect();
                log.detail(format!("  {n}: transitions {}", t.join("; ")));
            }
        }
        let curr = s.get("currAnimName");
        let next = s.get("nextAnimName").map(str::trim).filter(|n| !n.is_empty());
        match list.current(curr) {
            Ok(c) => log.detail(format!(
                "  at load: currAnimName {curr:?} nextAnimName {next:?} -> movement {} '{}' ({}{}) loop={} {} {} {} {} {}",
                c.movement,
                names.get(c.movement).copied().unwrap_or("?"),
                if c.fell_back { "movement 0" } else { "as named" },
                if curr.map(str::trim).unwrap_or("").is_empty() && next.is_some() { "; rule 12.1: the waiting movement starts first, no state" } else { "" },
                c.shape,
                c.sequence,
                c.start,
                c.speed,
                u8::from(c.repeat),
                c.unknown
            )),
            Err(e) => {
                log.detail(format!("  PROBLEM: movement at load: {e}"));
                problems += 1;
            }
        }
        // The Stop model: frames and textures.
        let stop = list.index("stop").map(|i| &list.movements[i].1);
        let mut stop_textures = Vec::new();
        if let Some(st) = stop {
            let model = world.load(&st.shape).or_else(|| game.find(&format!("GAME/Common/{}", st.shape)).and_then(|p| std::fs::read(p).ok()));
            match model.map(|b| Shape::parse(&b)) {
                Some(Ok(shape)) => {
                    let named = shape.loops.iter().find(|l| usize::try_from(l.name).ok().and_then(|n| shape.names.get(n)).is_some_and(|n| n.eq_ignore_ascii_case(&st.sequence)));
                    let frames = named.or(shape.loops.first()).map_or(0, |l| l.sequence_count);
                    log.detail(format!("  stop: {} '{}': {frames} frames{}", st.shape, st.sequence, if named.is_none() { " (no loop by that name: the first)" } else { "" }));
                    stop_textures = shape.materials.as_ref().map(|m| m.materials.iter().map(|m| m.texture.to_ascii_lowercase()).collect()).unwrap_or_default();
                }
                Some(Err(e)) => {
                    log.detail(format!("  PROBLEM: stop model {}: {e}", st.shape));
                    problems += 1;
                }
                None => {
                    log.detail(format!("  PROBLEM: stop model {} not found", st.shape));
                    problems += 1;
                }
            }
        }
        // The mouth.
        match Lips::of(&file, &keys) {
            None => {}
            Some(Ok(l)) => {
                *with.entry("lips").or_default() += 1;
                let carried = stop_textures.contains(&l.face.to_ascii_lowercase());
                let shapes: Vec<&str> = l.shapes.iter().map(|s| s.as_deref().unwrap_or("-")).collect();
                log.detail(format!("  lips [{}]: face {} ({} the stop model), shapes {shapes:?}", l.section, l.face, if carried { "in" } else { "NOT in" }));
            }
            Some(Err(e)) => {
                log.detail(format!("  PROBLEM: lips: {e}"));
                problems += 1;
            }
        }
        log.detail(format!("  {}", find_box(bounds, keys.monster_type.as_deref(), &s.name)));
    }
    // The monster lists.
    let mut lines = 0;
    let mut folded = 0;
    for (file_name, file) in &lists {
        let (list, list_problems) = MonsterList::from_section(&file[0]);
        for p in &list_problems {
            log.detail(format!("{file_name}: PROBLEM: {p}"));
        }
        problems += list_problems.len();
        lines += list.lines.len();
        log.detail(format!("{file_name} [{}]: numMonsters {:?}, {} lines read", list.name, list.declared, list.lines.len()));
        for l in &list.lines {
            let Some((t, case_only)) = types.find(&l.type_name) else {
                log.detail(format!("  {} {}: PROBLEM: type '{}' undefined (KQMonster::add - Monster type undefined: nothing made)", l.key, l.name, l.type_name));
                problems += 1;
                continue;
            };
            folded += usize::from(case_only);
            let found = world.load(&t.file).map(|b| kq_text::parse(&b));
            let character = found.as_ref().and_then(|f| f.first()).and_then(CharacterKeys::from_section);
            let file_note = match (&found, &character) {
                (None, _) => {
                    problems += 1;
                    format!("PROBLEM: file {} not in the world", t.file)
                }
                (Some(_), None) => {
                    problems += 1;
                    format!("PROBLEM: file {} is not a character", t.file)
                }
                (Some(_), Some(k)) => format!("file {} ({})", t.file, k.class.name()),
            };
            let wander = l.movement.or(t.movement.clone());
            let wander = match &wander {
                Wander::None => "never wanders".to_string(),
                Wander::Random => "random".to_string(),
                Wander::Loop(p) => format!("loop of {} points", p.len()),
                Wander::Patrol(p) => format!("patrol of {} points", p.len()),
            };
            log.detail(format!(
                "  {} {}: type {}{}{} -> {file_note}; at ({:.0}, {:.0}, {}) heading {}; {wander}; alarm {} birthDelay {}; {}",
                l.key,
                l.name,
                t.name,
                if case_only { " (matched ignoring capitals)" } else { "" },
                l.skin.as_ref().map_or(String::new(), |(a, b)| format!(" skin {a} -> {b}")),
                l.position[0],
                l.position[1],
                if l.position[2] == -1.0 { "ground".to_string() } else { format!("{:.0}", l.position[2]) },
                l.heading,
                l.alarm_radius.or(t.alarm_radius),
                l.birth_delay.or(t.birth_delay),
                find_box(bounds, Some(&t.name), &l.name)
            ));
            // M5 5a-1 (character-movement.md rules 9, 23, 27): what the
            // movement rules will use.
            if let Some(k) = &character {
                log.detail(format!("    movement rules: {}", AiSettings::from_line(k, l, t).describe()));
            }
        }
    }
    // Rule 20: the active list's size.
    let max = world.load("world.cs").and_then(|b| {
        let text: String = b.iter().map(|&c| c as char).collect();
        text.lines().find_map(|l| {
            let mut w = l.split_whitespace();
            (w.next()?.eq_ignore_ascii_case("set") && w.next()?.eq_ignore_ascii_case("KQWorld::MaxMonsters")).then(|| w.next().map(str::to_string)).flatten()
        })
    });
    if files_or_lists(&classes, &lists) {
        log.summary(format!("{}: KQWorld::MaxMonsters {} (world.cs; character-movement.md rule 20)", world.name, max.as_deref().unwrap_or("not set")));
    }
    let files: usize = classes.values().sum();
    if files + lines > 0 {
        log.summary(format!(
            "{}: character files {classes:?}; with {with:?}; monster lists {}, lines {lines}{}; problems {problems}",
            world.name,
            lists.len(),
            if folded > 0 { format!(" ({folded} types matched only ignoring capitals)") } else { String::new() }
        ));
    }
    for (k, v) in classes {
        *totals.files.entry(k).or_default() += v;
    }
    totals.lines += lines;
    totals.problems += problems;
}

/// The world has characters or monster lists.
fn files_or_lists(classes: &BTreeMap<&'static str, usize>, lists: &[(String, Vec<kq_text::Section>)]) -> bool {
    !classes.is_empty() || !lists.is_empty()
}
