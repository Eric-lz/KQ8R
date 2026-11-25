//! `kq8 save-verify`: verifies binary `PERS` and embedded `PVOL` saved-game
//! archives (`docs/behaviour/savefile.md` & `submenus.md` rule 10).
//!
//! By default scans every `MASK/save/save*.vol` (read only) and asserts:
//! - Every binary `PERS` chunk parses with 0 leftover bytes (`9,686` chunks
//!   across `save1.vol`..`save24.vol`).
//! - Embedded kept-world `.vol` archives parse as `PVOL` with ASCII `.kq` INI.
//! - Logs observed `KQDoor::state` / `KQChest::chestState` / `KQConner` /
//!   `KQWorldState` values for implementation checks.
//! - Spot-checks `save1.vol` and `save18.vol` against `savefile.md`.
//!
//! With `--file <path>`, verifies a single saved-game `.vol` file (such as a
//! KQ8R save in `kq8-artifacts/user/save/save25.vol`).

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kq8_data::kq_text;
use kq8_data::pvol::Pvol;
use kq8_data::save_pers::{parse_pers, KqStateKind, PersObject};
use kq8_data::GameDir;

use crate::log::Log;

fn ascii_text(b: &[u8]) -> String {
    let end = b.iter().position(|&c| c == 0).unwrap_or(b.len());
    String::from_utf8_lossy(&b[..end]).into_owned()
}

fn slot_number(path: &Path) -> u32 {
    path.file_name()
        .and_then(|n| n.to_str())
        .and_then(|n| {
            let lower = n.to_ascii_lowercase();
            lower
                .strip_prefix("save")
                .and_then(|s| s.strip_suffix(".vol"))
                .and_then(|s| s.parse::<u32>().ok())
        })
        .unwrap_or(u32::MAX)
}

pub fn run(game: &GameDir, single_file: Option<&str>) -> ExitCode {
    let mut log = match Log::create("save-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("save-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));

    let mut files: Vec<PathBuf> = if let Some(path) = single_file {
        vec![PathBuf::from(path)]
    } else {
        let Some(save_dir) = game.find("save") else {
            log.summary("FAIL save/ folder not found");
            return ExitCode::FAILURE;
        };
        let mut v: Vec<PathBuf> = std::fs::read_dir(&save_dir)
            .map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect())
            .unwrap_or_default();
        v.retain(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("vol")));
        v.sort_by_key(|p| slot_number(p));
        v
    };

    if files.is_empty() {
        log.summary("FAIL no save*.vol files found");
        return ExitCode::FAILURE;
    }

    let mut failures = 0usize;
    let mut total_pers = 0usize;
    let mut total_ascii_kq = 0usize;
    let mut total_kept_vols = 0usize;
    let mut class_counts: BTreeMap<String, usize> = BTreeMap::new();
    let mut door_states: BTreeSet<(i32, String)> = BTreeSet::new();
    let mut chest_states: BTreeSet<(i32, String, bool)> = BTreeSet::new();
    let mut event_samples: Vec<(String, String)> = Vec::new();
    let mut connor_samples: Vec<String> = Vec::new();

    for path in files.drain(..) {
        let label = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("?")
            .to_string();
        let bytes = match std::fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                failures += 1;
                log.summary(format!("FAIL {label}: {e}"));
                continue;
            }
        };
        let pvol = match Pvol::parse(bytes) {
            Ok(p) => p,
            Err(e) => {
                failures += 1;
                log.summary(format!("FAIL {label}: PVOL parse: {e}"));
                continue;
            }
        };

        let chunk_str = |name: &str| -> (usize, String) {
            pvol.find(name)
                .and_then(|e| pvol.data(e).ok())
                .map(|b| (b.len(), ascii_text(b)))
                .unwrap_or((0, String::new()))
        };

        let (name_len, save_name) = chunk_str("savedGameName");
        let (_, world_name) = chunk_str("worldName");
        let (_, version) = chunk_str("savedGameVersion");
        let (_, world_vols) = chunk_str("worldVolumes");
        let (_, maps_list) = chunk_str("maps");

        let mut pers_ok = 0usize;
        let mut ascii_kq = 0usize;
        let mut kept_vols = 0usize;
        let mut other = 0usize;

        for e in &pvol.entries {
            let Ok(data) = pvol.data(e) else {
                failures += 1;
                log.summary(format!("FAIL {label}/{}: cannot read entry data", e.name));
                continue;
            };
            let lower = e.name.to_ascii_lowercase();
            if lower.ends_with(".kq") {
                if data.starts_with(b"PERS") || data.starts_with(b"DICT") {
                    match parse_pers(data) {
                        Ok(obj) => {
                            pers_ok += 1;
                            total_pers += 1;
                            *class_counts.entry(obj.class_name().to_string()).or_default() += 1;
                            match &obj {
                                PersObject::KqDoor(d) => {
                                    let anim = d
                                        .anim_object
                                        .curr_anim_name
                                        .clone()
                                        .unwrap_or_default();
                                    door_states.insert((d.state, anim));
                                }
                                PersObject::KqChest(c) => {
                                    let anim = c
                                        .anim_object
                                        .curr_anim_name
                                        .clone()
                                        .unwrap_or_default();
                                    chest_states.insert((
                                        c.chest_state,
                                        anim,
                                        c.inv_item_name.is_some(),
                                    ));
                                }
                                PersObject::KqWorldState(ws) => {
                                    for ev in ws.events.iter().flatten() {
                                        if event_samples.len() < 20 {
                                            event_samples.push((label.clone(), ev.clone()));
                                        }
                                    }
                                }
                                PersObject::KqConner(c) => {
                                    if let PersObject::KqState(st) =
                                        &c.monster.state.as_ref().map(|(_, b)| b.as_ref()).unwrap_or(&PersObject::KqBase(Default::default()))
                                    {
                                        if let KqStateKind::Unknown { class, remaining } = &st.kind {
                                            failures += 1;
                                            log.summary(format!(
                                                "FAIL {label}/{}: unknown Connor state {class} ({} trailing bytes)",
                                                e.name,
                                                remaining.len()
                                            ));
                                        }
                                    }
                                    let pos = c
                                        .monster
                                        .anim_object
                                        .object
                                        .transform()
                                        .map(|t| t.position)
                                        .unwrap_or([0.0; 3]);
                                    connor_samples.push(format!(
                                        "{label}: world={world_name} level={} stats={:?} worldFlags={:#x} doMode={} itemInHand={:?} spells=({},{},{},{}) alt=({},{},{}) pos=({:.0},{:.0},{:.0}) anim={:?}@{:.3}",
                                        c.level,
                                        c.monster.anim_object.object.stats,
                                        c.world_flags,
                                        c.monster.do_mode,
                                        c.item_in_hand,
                                        c.clarity_spell,
                                        c.strength_spell,
                                        c.invulnerability_spell,
                                        c.invisibility_spell,
                                        c.alt_short,
                                        c.alt_long,
                                        c.alt_armor,
                                        pos[0],
                                        pos[1],
                                        pos[2],
                                        c.monster.anim_object.curr_anim_name,
                                        c.monster.anim_object.anim_pos,
                                    ));
                                }
                                PersObject::KqMonster { data: m, .. } => {
                                    if let Some((_, st_obj)) = &m.state {
                                        if let PersObject::KqState(st) = st_obj.as_ref() {
                                            if let KqStateKind::Unknown { class, remaining } = &st.kind {
                                                failures += 1;
                                                log.summary(format!(
                                                    "FAIL {label}/{}: unknown monster state {class} ({} trailing bytes)",
                                                    e.name,
                                                    remaining.len()
                                                ));
                                            }
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                        Err(err) => {
                            failures += 1;
                            log.summary(format!("FAIL {label}/{}: {err}", e.name));
                        }
                    }
                } else {
                    let sections = kq_text::parse(data);
                    if sections.is_empty() {
                        failures += 1;
                        log.summary(format!("FAIL {label}/{}: empty ASCII .kq", e.name));
                    } else {
                        ascii_kq += 1;
                        total_ascii_kq += 1;
                    }
                }
            } else if lower.ends_with(".vol") {
                match Pvol::parse(data.to_vec()) {
                    Ok(inner) => {
                        kept_vols += 1;
                        total_kept_vols += 1;
                        for ke in &inner.entries {
                            if ke.name.to_ascii_lowercase().ends_with(".kq") {
                                if let Ok(kd) = inner.data(ke) {
                                    if kd.starts_with(b"PERS") || kd.starts_with(b"DICT") {
                                        if let Err(err) = parse_pers(kd) {
                                            failures += 1;
                                            log.summary(format!(
                                                "FAIL {label}/{}/{}: {err}",
                                                e.name, ke.name
                                            ));
                                        }
                                    } else if kq_text::parse(kd).is_empty() {
                                        failures += 1;
                                        log.summary(format!(
                                            "FAIL {label}/{}/{}: unreadable ASCII .kq",
                                            e.name, ke.name
                                        ));
                                    }
                                }
                            }
                        }
                    }
                    Err(err) => {
                        failures += 1;
                        log.summary(format!("FAIL {label}/{}: kept PVOL: {err}", e.name));
                    }
                }
            } else {
                other += 1;
            }
        }

        // Spot checks from docs/behaviour/savefile.md ("How to check")
        if single_file.is_none() && label.eq_ignore_ascii_case("save1.vol") {
            let check_pers = |entry_name: &str| -> Option<PersObject> {
                let entry = pvol.find(entry_name)?;
                let data = pvol.data(entry).ok()?;
                parse_pers(data).ok()
            };
            match check_pers("GameState.kq") {
                Some(PersObject::KqGameState(gs))
                    if gs.next_no_name == 439 && (gs.difficulty_factor - 0.66).abs() < 0.01 => {}
                other => {
                    failures += 1;
                    log.summary(format!("FAIL save1.vol/GameState.kq mismatch: {other:?}"));
                }
            }
            match check_pers("worldState.kq") {
                Some(PersObject::KqWorldState(ws)) if ws.events.len() == 2 => {}
                other => {
                    failures += 1;
                    log.summary(format!("FAIL save1.vol/worldState.kq mismatch: {other:?}"));
                }
            }
            match check_pers("Connor.kq") {
                Some(PersObject::KqConner(c))
                    if c.monster.anim_object.object.stats.as_deref() == Some("10 10 0 0 0")
                        && c.monster.anim_object.curr_anim_name.as_deref() == Some("Stop")
                        && (c.monster.anim_object.anim_pos - 0.2667).abs() < 0.01 => {}
                other => {
                    failures += 1;
                    log.summary(format!(
                        "FAIL save1.vol/Connor.kq mismatch: stats={:?}",
                        other.map(|o| o.class_name().to_string())
                    ));
                }
            }
            match check_pers("Condoor.kq") {
                Some(PersObject::KqDoor(d))
                    if d.state == 2
                        && d.anim_object.curr_anim_name.as_deref() == Some("closed") => {}
                other => {
                    failures += 1;
                    log.summary(format!("FAIL save1.vol/Condoor.kq mismatch: {other:?}"));
                }
            }
        }

        if single_file.is_none() && label.eq_ignore_ascii_case("save18.vol") {
            if !world_name.eq_ignore_ascii_case("Swamp.kq") {
                failures += 1;
                log.summary(format!("FAIL save18.vol worldName={world_name}"));
            }
            let swmp_is_pers = pvol
                .find("swmpMap.kq")
                .and_then(|e| pvol.data(e).ok())
                .is_some_and(|d| d.starts_with(b"PERS"));
            let cast_is_ascii = pvol
                .find("castMap.kq")
                .and_then(|e| pvol.data(e).ok())
                .is_some_and(|d| !d.starts_with(b"PERS"));
            if !swmp_is_pers || !cast_is_ascii || kept_vols != 3 {
                failures += 1;
                log.summary(format!(
                    "FAIL save18.vol layout check: swmp_pers={swmp_is_pers} cast_ascii={cast_is_ascii} kept_vols={kept_vols}"
                ));
            }
        }

        log.summary(format!(
            "{label}: \"{save_name}\" ({name_len} B), world={world_name}, ver={version}; {} entries ({pers_ok} PERS, {ascii_kq} ASCII .kq, {kept_vols} kept .vol, {other} other); worldVolumes=[{world_vols}] maps=[{maps_list}]",
            pvol.entries.len()
        ));
    }

    log.summary(format!(
        "save-verify: {total_pers} binary PERS chunks parsed across {} classes, {total_ascii_kq} ASCII .kq chunks, {total_kept_vols} kept .vol archives; {failures} failures",
        class_counts.len()
    ));
    log.detail("== Class histogram ==");
    for (cls, count) in &class_counts {
        log.detail(format!("  {cls}: {count}"));
    }
    log.detail("== Observed KQDoor (state, currAnimName) ==");
    for (st, anim) in &door_states {
        log.detail(format!("  state={st} currAnimName={anim:?}"));
    }
    log.detail("== Observed KQChest (chestState, currAnimName, has_invItemName) ==");
    for (st, anim, has_item) in &chest_states {
        log.detail(format!(
            "  chestState={st} currAnimName={anim:?} has_invItemName={has_item}"
        ));
    }
    log.detail("== Sample KQWorldState events ==");
    for (save, ev) in &event_samples {
        log.detail(format!("  {save}: {ev:?}"));
    }
    log.detail("== KQConner summaries ==");
    for line in &connor_samples {
        log.detail(format!("  {line}"));
    }

    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}
