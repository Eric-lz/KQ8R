//! The magic map in the viewer (M6 6d, docs/DESIGN.md; rules:
//! docs/behaviour/magic-map.md): the world's map file read at start-up
//! with its pictures and icons, handed to the rules
//! (`kq8_core::magic_map`), and fed Connor's place every update.
//! Everything goes to `kq8-artifacts/logs/map.log`.
//!
//! 6d-6: other worlds' maps: the palettes by map index, the shipped map
//! files (`GAME/Kq`) as a STAND-IN for a world never left (open question
//! 5), and `--kept-maps SAVE` (a test option of ours): the maps an
//! original saved game kept for the worlds Connor left, read only, marked
//! visited.

use bevy::prelude::*;
use kq8_core::magic_map::{Colours, IconReader, IndexPic, MapState, PALETTES, UNUSED_MAPS};
use kq8_data::magic_map::MapKeys;
use kq8_data::{Bitmap, GameDir};

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::load::MapFile;
use crate::log::Log;

/// Where the map pictures and icons are (`magic-map.md` "Data used").
const PICTURE_FOLDER: &str = "GAME/Resource";

/// An 8-bit picture file from `GAME/Resource` (any case).
fn read_picture(game: &GameDir, name: &str) -> Result<IndexPic, String> {
    let rel = format!("{PICTURE_FOLDER}/{name}");
    let path = game.find(&rel).ok_or_else(|| format!("{rel} not found"))?;
    let bytes = std::fs::read(&path).map_err(|e| format!("{rel}: {e}"))?;
    let b = Bitmap::parse(&bytes).map_err(|e| format!("{rel}: {e}"))?;
    if b.bit_count != 8 {
        return Err(format!("{rel}: {} bits per pixel, not 8", b.bit_count));
    }
    let (width, height) = (b.width as usize, b.height as usize);
    Ok(IndexPic { width, height, pixels: b.pixels[..width * height].to_vec() })
}

/// The map made at start-up, waiting for the world's rules to exist; and
/// the log.
#[derive(Resource)]
pub struct MapSim {
    pending: Option<MapState>,
    /// Other worlds' maps by index (file, keys, source) and the indexes to
    /// mark visited (`--kept-maps`), for the rules once they exist.
    others: Vec<(usize, String, MapKeys, String)>,
    visited: Vec<usize>,
    log: Option<Log>,
}

/// Rule 34: the 16 palettes, by map index.
fn read_palettes(game: &GameDir, note: &mut impl FnMut(String) -> String) -> Vec<Option<Colours>> {
    PALETTES
        .iter()
        .map(|name| {
            let rel = format!("{PICTURE_FOLDER}/{name}");
            match game.find(&rel).ok_or("not found".to_string()).and_then(|p| std::fs::read(p).map_err(|e| e.to_string())).and_then(|b| kq8_data::Palette::parse(&b)) {
                Ok(p) => Some(p.colors),
                Err(e) => {
                    note(format!("palette {rel}: {e}; that world's map keeps its colours"));
                    None
                }
            }
        })
        .collect()
}

/// The shipped map files of `GAME/Kq` by index, but the unused ones
/// ("Do not reproduce" 6).
fn shipped_maps(game: &GameDir) -> Vec<(usize, String, MapKeys)> {
    let Some(dir) = game.find("GAME/Kq") else { return Vec::new() };
    let mut out = Vec::new();
    let mut names: Vec<_> = std::fs::read_dir(dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
    names.sort();
    for path in names {
        let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with("map.kq") || UNUSED_MAPS.contains(&lower.as_str()) {
            continue;
        }
        let Ok(bytes) = std::fs::read(&path) else { continue };
        let file = kq8_data::kq_text::parse(&bytes);
        if let Some(Ok(keys)) = file.first().and_then(MapKeys::from_section) {
            if let Some(i) = keys.index() {
                out.push((i, name, keys));
            }
        }
    }
    out
}

/// `--kept-maps SAVE`: the maps an original saved game kept for the worlds
/// Connor left (text form; read only).
fn kept_maps(path: &str) -> Result<Vec<(usize, String, MapKeys)>, String> {
    let archive = kq8_data::pvol::Pvol::parse(std::fs::read(path).map_err(|e| format!("{path}: {e}"))?)?;
    let mut out = Vec::new();
    for e in &archive.entries {
        if !e.name.to_ascii_lowercase().ends_with("map.kq") {
            continue;
        }
        let Ok(data) = archive.data(e) else { continue };
        if data.starts_with(b"PERS") {
            continue;
        }
        let file = kq8_data::kq_text::parse(data);
        if let Some(Ok(keys)) = file.first().and_then(MapKeys::from_section) {
            if let Some(i) = keys.index() {
                out.push((i, e.name.clone(), keys));
            }
        }
    }
    Ok(out)
}

/// Reads the world's map pictures (rule 7) and makes its state; `None`
/// when the world loads no map.
/// `own_kept` (M6 6e-8, `world-change.md` rule 16): the maps kept when
/// Connor left worlds; this world's own one, if there, is used instead of
/// its shipped file (its fog, icons and teleporter as left).
pub fn load_map(game: &GameDir, world: &str, file: Option<MapFile>, kept: Option<&str>, own_kept: Option<&std::collections::BTreeMap<usize, (String, MapKeys, String)>>, main: &mut Log) -> MapSim {
    let mut log = match Log::create("map") {
        Ok(l) => Some(l),
        Err(e) => {
            main.summary(format!("map: cannot create map.log: {e}; the map still works, not logged"));
            None
        }
    };
    let mut note = |line: String| {
        if let Some(l) = &mut log {
            l.detail(&line);
        }
        line
    };
    let Some(file) = file else {
        main.summary(note(format!("map: {world} loads no map file (magic-map.md rule 3)")));
        return MapSim { pending: None, others: Vec::new(), visited: Vec::new(), log };
    };
    let mut pic = |name: &str| match read_picture(game, name) {
        Ok(p) => {
            note(format!("map picture {name}: {}x{}", p.width, p.height));
            Some(p)
        }
        Err(e) => {
            main.summary(note(format!("map picture {name}: {e}; drawn black")));
            None
        }
    };
    let blank = pic(&file.keys.blank_map);
    let full = pic(&file.keys.full_map);
    let g = game.clone();
    let reader: IconReader = std::sync::Arc::new(move |name: &str| read_picture(&g, name));
    let base = file.label.rsplit('/').next().unwrap_or(&file.label).to_string();
    let mut keys = file.keys;
    if let Some((f, k, source)) = keys.index().and_then(|i| own_kept.and_then(|m| m.get(&i))).filter(|(_, _, s)| s.starts_with("kept when Connor left")) {
        main.summary(note(format!("map: {f}: the map {source} (its fog, icons and teleporter as left; world-change.md rule 16)")));
        keys = k.clone();
    }
    let mut map = MapState::new(&base, keys, blank, full, file.haze, Some(reader));
    map.palettes = read_palettes(game, &mut note);
    let (radius, units, from) = kq8_core::magic_map::fog_radius(&map.keys, file.haze);
    main.summary(note(format!(
        "map: {} [{}]: index {:?}, box west {} east {} south {} north {} ({:.1} units per map pixel), fog radius {radius} map pixels ({units} from {from}), {} out of the fog, {} icons, active {} (magic-map.md rules 3, 4, 13, 16)",
        file.label,
        map.keys.name,
        map.index,
        map.keys.west,
        map.keys.east,
        map.keys.south,
        map.keys.north,
        (map.keys.east - map.keys.west) / kq8_data::magic_map::MAP_SIZE as f64,
        map.fog.iter().filter(|v| **v).count(),
        map.icons.len(),
        map.active
    )));
    // Rule 34: other worlds' maps; the shipped files first, then kept
    // ones over them.
    let mut others: Vec<(usize, String, MapKeys, String)> = shipped_maps(game)
        .into_iter()
        .filter(|(i, ..)| Some(*i) != map.index)
        .map(|(i, f, k)| (i, f, k, "the shipped file, no fog revealed (stand-in: open question 5)".to_string()))
        .collect();
    let mut visited = Vec::new();
    if let Some(path) = kept {
        match kept_maps(path) {
            Ok(list) => {
                for (i, f, k) in list.into_iter().filter(|(i, ..)| Some(*i) != map.index) {
                    let revealed = kq8_data::magic_map::revealed_count(&kq8_data::magic_map::fog_from_runs(&k.runs).0);
                    main.summary(note(format!("map: --kept-maps {path}: {f} (index {i}), {revealed} pixels out of the fog, {} icons: kept, marked visited (a test option of ours)", k.icons.len())));
                    others.retain(|(j, ..)| *j != i);
                    others.push((i, f, k, format!("kept in {path}")));
                    visited.push(i);
                }
            }
            Err(e) => {
                main.summary(note(format!("map: --kept-maps {path}: {e}; none")));
            }
        }
    }
    MapSim { pending: Some(map), others, visited, log }
}

/// First in each update: the map made at start-up is handed to the rules
/// (once), before any script or trap runs (the original loads it in
/// `newWorld.cs`; the keep's pad trap says `onTeleport` at update 1).
pub fn map_install(map: Option<ResMut<MapSim>>, sim: Option<ResMut<DoorSim>>) {
    let (Some(mut map), Some(mut sim)) = (map, sim) else { return };
    let world = &mut sim.active.world;
    if let Some(mut m) = map.pending.take() {
        // Rules 22, 27: both views from the one stored on Connor.
        m.set_views(world.map_view);
        world.map_log.push(format!("update {}: the map is set up; views from Connor's {:?} (rules 22, 27)", world.updates, world.map_view.as_array()));
        world.map = Some(m);
        // M6 6e-8: a map kept when Connor left its world (the journey's)
        // stays; the shipped files and `--kept-maps` fill the others.
        for (i, f, k, source) in std::mem::take(&mut map.others) {
            if world.other_maps.get(&i).is_some_and(|(_, _, s)| s.starts_with("kept when Connor left")) {
                continue;
            }
            world.other_maps.insert(i, (f, k, source));
        }
        for i in std::mem::take(&mut map.visited) {
            world.map_marks.visited[i] = true;
        }
    }
}

/// One game update, after Connor's movement and the scripts' moves (the
/// trap step): the map gets his place (rule 9).
pub fn map_step(map: Option<ResMut<MapSim>>, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>) {
    let (Some(mut map), Some(mut sim)) = (map, sim) else { return };
    let world = &mut sim.active.world;
    if let Some(c) = connor {
        world.map_step(c.mover.position, c.walker.heading);
    }
    let lines = std::mem::take(&mut world.map_log);
    if let Some(log) = &mut map.log {
        for l in lines {
            log.detail(l);
        }
    }
}
