//! The magic map in the viewer (M6 6d, docs/DESIGN.md; rules:
//! docs/behaviour/magic-map.md): the world's map file read at start-up
//! with its pictures and icons, handed to the rules
//! (`kq8_core::magic_map`), and fed Connor's place every update.
//! Everything goes to `kq8-artifacts/logs/map.log`.
//!
//! 6d-2: no drawing yet.

use bevy::prelude::*;
use kq8_core::magic_map::{IconReader, IndexPic, MapState};
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
    log: Option<Log>,
}

/// Reads the world's map pictures (rule 7) and makes its state; `None`
/// when the world loads no map.
pub fn load_map(game: &GameDir, world: &str, file: Option<MapFile>, main: &mut Log) -> MapSim {
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
        return MapSim { pending: None, log };
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
    let reader: IconReader = Box::new(move |name: &str| read_picture(&g, name));
    let base = file.label.rsplit('/').next().unwrap_or(&file.label).to_string();
    let map = MapState::new(&base, file.keys, blank, full, file.haze, Some(reader));
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
    MapSim { pending: Some(map), log }
}

/// One game update, after Connor's movement and the scripts' moves (the
/// trap step): the map gets his place (rule 9).
pub fn map_step(map: Option<ResMut<MapSim>>, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>) {
    let (Some(mut map), Some(mut sim)) = (map, sim) else { return };
    let world = &mut sim.active.world;
    if let Some(m) = map.pending.take() {
        world.map = Some(m);
    }
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
