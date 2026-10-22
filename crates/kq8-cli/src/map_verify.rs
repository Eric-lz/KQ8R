//! `kq8 map-verify` (M6 6d-1): reads the magic map's files
//! (`docs/behaviour/magic-map.md`) and the maps in the user's saved games.
//!
//! - Every `GAME/Kq/*map.kq` (`KQMap`): its keys, map index and name text
//!   (rule 3), world units per map pixel (rule 4), the fog radius in map
//!   pixels (rule 13: 460 × `fogRadius` ÷ (east − west), whole number;
//!   missing → the world's `hazeDistance`, else 20,000), the world that
//!   loads it (its `newWorld.cs`).
//! - Every `GAME/Kq/*tele.kq` (`KQPortal`): its keys (rules 39–40).
//! - Every `MASK/save/*.vol` (read only): each saved map in text form,
//!   its revealed pixel count, unused places marked revealed, and its run
//!   list written back from the fog (rule 16): equal when no unused place
//!   is marked; otherwise the fog read again from it must be the same
//!   (the unused places are ignored on reading and written as fog).
//!   The current world's map is kept in another (binary) form there,
//!   which no note describes: listed, not read.

use std::collections::BTreeMap;
use std::process::ExitCode;

use kq8_data::game_dir::WORLDS;
use kq8_data::kq_text;
use kq8_data::magic_map::{fog_from_runs, revealed_count, runs_from_fog, MapKeys, PortalKeys, MAP_SIZE, ROW_PLACES};
use kq8_data::pvol::Pvol;
use kq8_data::{GameDir, TerrainSettings};

use crate::log::Log;
use crate::terrain_verify::{load, open_world};

/// Rule 13: no `fogRadius` and no terrain.
const NO_TERRAIN_FOG: f64 = 20000.0;

/// What each world says about the map: the map file its `newWorld.cs`
/// loads and its terrain's `hazeDistance`.
struct WorldMap {
    world: &'static str,
    file: Option<String>,
    haze: Option<f32>,
}

fn world_maps(game: &GameDir, log: &mut Log) -> Vec<WorldMap> {
    let mut out = Vec::new();
    for world in WORLDS {
        let vols = open_world(game, world, log);
        let file = load(game, world, &vols, "newWorld.cs").ok().and_then(|(_, b)| {
            let text: String = b.iter().map(|&c| c as char).collect();
            text.lines().map(str::trim).filter(|l| !l.starts_with('#')).find_map(|l| {
                let mut w = l.split_whitespace();
                (w.next()?.eq_ignore_ascii_case("loadKQ")).then_some(())?;
                w.next().filter(|f| f.to_ascii_lowercase().ends_with("map.kq")).map(str::to_string)
            })
        });
        let haze = load(game, world, &vols, "terrain.kq").ok().and_then(|(_, b)| {
            let file = kq_text::parse(&b);
            file.first().and_then(|s| TerrainSettings::from_section(s).ok()).and_then(|t| t.haze_distance)
        });
        out.push(WorldMap { world, file, haze });
    }
    out
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("map-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("map-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let mut failures = 0usize;
    let worlds = world_maps(game, &mut log);
    let names = message_names(game);
    let Some(kq) = game.find("GAME/Kq") else {
        log.summary("FAIL GAME/Kq not found");
        return ExitCode::FAILURE;
    };
    let mut files: Vec<_> = std::fs::read_dir(&kq).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
    files.sort_by_key(|p| p.file_name().map(|n| n.to_ascii_lowercase()));
    log.summary("== maps (GAME/Kq/*map.kq; magic-map.md rules 3, 4, 13)");
    for path in &files {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".kq") {
            continue;
        }
        let Ok(bytes) = std::fs::read(path) else { continue };
        let sections = kq_text::parse(&bytes);
        let Some(first) = sections.first() else { continue };
        if let Some(r) = MapKeys::from_section(first) {
            let m = match r {
                Ok(m) => m,
                Err(e) => {
                    failures += 1;
                    log.summary(format!("FAIL {name}: {e}"));
                    continue;
                }
            };
            let loader = worlds.iter().find(|w| w.file.as_deref().is_some_and(|f| f.eq_ignore_ascii_case(name)));
            let across = m.east - m.west;
            let down = (m.north - m.south).abs();
            let (radius_units, from) = match (m.fog_radius.filter(|r| *r > 0.0), loader.and_then(|w| w.haze)) {
                (Some(r), _) => (r, "fogRadius".to_string()),
                (None, Some(h)) => (h as f64, "hazeDistance (no fogRadius)".to_string()),
                (None, None) => (NO_TERRAIN_FOG, "20,000 (no fogRadius, no terrain known)".to_string()),
            };
            let radius_px = if across != 0.0 { (MAP_SIZE as f64 * radius_units / across) as i64 } else { 0 };
            let index = m.index().map_or("none".to_string(), |i| i.to_string());
            let text = m.map_name.and_then(|n| names.get(&n)).map_or("(no text)".to_string(), |t| format!("{t:?}"));
            log.summary(format!(
                "{name} [{}]: index {index}, name #{} {text}; box west {} east {} south {} north {}: {:.1} x {:.1} units per map pixel; fog radius {radius_units} ({from}) = {radius_px} map pixels; useFog {}, isActive {}, blank {}, full {}, arrow {}, teleport {:?} / {:?}, {} icons, {} runs; loaded by {}",
                m.name,
                m.map_name.map_or("?".into(), |n| n.to_string()),
                m.west,
                m.east,
                m.south,
                m.north,
                across / MAP_SIZE as f64,
                down / MAP_SIZE as f64,
                u8::from(m.use_fog),
                u8::from(m.is_active),
                m.blank_map,
                m.full_map,
                m.arrow_pba,
                m.teleport_script,
                m.teleport_portal,
                m.icons.len(),
                m.runs.len(),
                loader.map_or("no world (never loaded, rule 3)".to_string(), |w| w.world.to_string()),
            ));
        } else if let Some(p) = PortalKeys::from_section(first) {
            log.summary(format!(
                "{name} [{}] KQPortal: world {:?}, enter {:?}, exit {:?}, Connor to {:?} dir {:?}, camera to {:?} dir {:?}, noLoc {:?}",
                p.name, p.world, p.enter_anim, p.exit_anim, p.dest_connor_loc, p.dest_connor_dir, p.dest_cam_loc, p.dest_cam_dir, p.no_loc
            ));
        }
    }
    log.summary("== saved maps (MASK/save/*.vol, read only; rule 16)");
    let saves = game.find("save").map(|d| {
        let mut v: Vec<_> = std::fs::read_dir(d).map(|r| r.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
        v.retain(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("vol")));
        v.sort_by_key(|p| p.file_name().and_then(|n| n.to_str()).and_then(|n| n.trim_start_matches("save").trim_end_matches(".vol").parse::<u32>().ok()).unwrap_or(u32::MAX));
        v
    });
    let mut maps_read = 0usize;
    for path in saves.unwrap_or_default() {
        let label = path.file_name().and_then(|n| n.to_str()).unwrap_or("?").to_string();
        let archive = match std::fs::read(&path).map_err(|e| e.to_string()).and_then(Pvol::parse) {
            Ok(a) => a,
            Err(e) => {
                failures += 1;
                log.summary(format!("FAIL {label}: {e}"));
                continue;
            }
        };
        let world = archive.find("worldName").and_then(|e| archive.data(e).ok()).map(|b| String::from_utf8_lossy(b).trim_end_matches('\0').to_string());
        log.summary(format!("{label}: {} files, world {}", archive.entries.len(), world.as_deref().unwrap_or("?")));
        for e in &archive.entries {
            if !e.name.to_ascii_lowercase().ends_with("map.kq") {
                continue;
            }
            let Ok(data) = archive.data(e) else { continue };
            if data.starts_with(b"PERS") {
                log.summary(format!("  {}: {} bytes in the binary form (the current world's map): not read (no note describes it)", e.name, data.len()));
                continue;
            }
            let sections = kq_text::parse(data);
            let Some(Some(Ok(m))) = sections.first().map(MapKeys::from_section) else {
                failures += 1;
                log.summary(format!("  FAIL {}: not a readable KQMap", e.name));
                continue;
            };
            maps_read += 1;
            let (fog, check) = fog_from_runs(&m.runs);
            let back = runs_from_fog(&fog);
            let same = back == m.runs;
            let same_fog = fog_from_runs(&back).0 == fog;
            let total = MAP_SIZE * ROW_PLACES;
            if !same_fog || (!same && check.unused_revealed == 0) || check.beyond != 0 {
                failures += 1;
            }
            // Unused places marked: which columns and rows (the note's
            // "Do not reproduce" 2 marks one pixel past the copied ones,
            // which at the right edge is column 460).
            let mut unused: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
            let mut place = 0usize;
            for (k, &run) in m.runs.iter().enumerate() {
                let run = run.max(0) as usize;
                if k % 2 == 1 {
                    for p in place..(place + run).min(total) {
                        if p % ROW_PLACES >= MAP_SIZE {
                            unused.entry(p % ROW_PLACES).or_default().push(p / ROW_PLACES);
                        }
                    }
                }
                place += run;
            }
            log.summary(format!(
                "  {} [{}]: isActive {}, earnedTeleport {}, {} icons, {} runs over {} places (of {total}): {} map pixels revealed ({:.1} %), unused places revealed {}, beyond the last row {}; written back: {}",
                e.name,
                m.name,
                u8::from(m.is_active),
                u8::from(m.earned_teleport),
                m.icons.len(),
                m.runs.len(),
                check.places,
                revealed_count(&fog),
                100.0 * revealed_count(&fog) as f64 / (MAP_SIZE * MAP_SIZE) as f64,
                check.unused_revealed,
                check.beyond,
                match (same, same_fog) {
                    (true, _) => "the same runs".to_string(),
                    (false, true) => format!("{} runs (the unused places written as fog), the same fog", back.len()),
                    (false, false) => format!("DIFFERENT fog ({} runs)", back.len()),
                }
            ));
            for (col, rows) in &unused {
                log.summary(format!("    unused column {col} marked revealed in {} rows, {}-{} (one past the copied pixels at the right edge: magic-map.md \"Do not reproduce\" 2)", rows.len(), rows[0], rows[rows.len() - 1]));
            }
            for i in &m.icons {
                log.detail(format!("    icon {} at ({}, {}) on {}", i.file, i.x, i.y, i.picture));
            }
        }
    }
    log.summary(format!("map-verify: {maps_read} saved maps read; {failures} failures"));
    if failures == 0 {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// `500.MSG` noun 0 verb 0: case → text (rule 8).
fn message_names(game: &GameDir) -> BTreeMap<u32, String> {
    let Some(bytes) = game.find("GAME/ENGLISH/500.MSG").and_then(|p| std::fs::read(p).ok()) else { return BTreeMap::new() };
    let Ok(file) = kq8_data::msg::MessageFile::parse(500, &bytes) else { return BTreeMap::new() };
    let mut out = BTreeMap::new();
    for l in file.lines.iter().filter(|l| l.tuple.noun == 0 && l.tuple.verb == 0) {
        out.entry(l.tuple.case as u32).or_insert_with(|| l.raw.clone());
    }
    out
}
