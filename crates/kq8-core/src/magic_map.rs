//! The magic map (M6 6d, `docs/behaviour/magic-map.md`; A, 2026-10-21,
//! "How to check" 1–11 run by the user in the original, all as
//! described): the current world's map, its fog, Connor's arrow and the
//! icons, as plain rules with no drawing.
//!
//! - Rule 2: the map works only while active: the `MagicMap` item
//!   arriving in Connor's inventory switches it on, leaving switches it
//!   off; `KQMap::activate` / `deactivate` too.
//! - Rule 4: world (x, y) → map pixel: across (x − west) ÷ (east − west)
//!   × 460, down (1 − (y − south) ÷ |north − south|) × 460, fractions
//!   dropped towards zero, no clamp; a box with no width or height does
//!   nothing.
//! - Rule 9: each update while active: unless "stop location" is on,
//!   Connor's map pixel; only when it differs from the last one: the
//!   map's index marked visited on Connor, the fog cleared around it,
//!   the icons drawn again; always the arrow frame. Height plays no part
//!   (rule 10); the last pixel starts unset (rule 11).
//! - Rule 12: the reveal disc: rows cy − r + 1 to cy + r; in row cy + d
//!   the half-width w = ⌊√(r² − e²)⌋, e = 1 − d on or above Connor's row,
//!   d below; columns cx − w to cx + w − 1; outside 0–459 skipped. Never
//!   back into the fog.
//! - Rule 13: r = 460 × `fogRadius` ÷ (east − west), whole number;
//!   missing or ≤ 0: the world's `hazeDistance`, else 20,000.
//! - Rule 14: `useFog 0` shows the whole full picture.
//! - Rule 15: `stopLoc` / `holdMapLoc`: position ignored (no reveal, no
//!   arrow move, no visited), the arrow still turns.
//! - Rule 17: arrow frame = (π − heading) ÷ 2π × 16 rounded, wrapped into
//!   0–15 (frame 0 up, clockwise).
//! - Rules 7, 19–21: the map's own picture: blank, then the full picture
//!   where out of the fog, then the icons (index 0 see-through, drawn over
//!   the fog, top-left at their map pixel). `addMapIcon file x y
//!   [picture]` (default: the picture shown now; not twice the same);
//!   `removeMapIcon x y` (blank, then full where revealed, over the icon's
//!   rectangle).
//!
//! Not copied ("Do not reproduce"): a reveal marks exactly the columns it
//! copies (2); `useFog 0` marks the pixels it shows (3).
//!
//! STAND-IN (rule 9's timing, DESIGN 6d): the caller gives Connor's place
//! once per update, after his movement and the scripts' moves, not at
//! every change of his position.

use std::collections::BTreeMap;

use kq8_data::magic_map::{fog_from_runs, runs_from_fog, MapIcon, MapKeys, Revealed, MAP_COUNT, MAP_SIZE};

/// Rule 13: the fog radius with neither `fogRadius` nor a terrain.
pub const NO_TERRAIN_FOG: f64 = 20_000.0;
/// Rule 17: the arrow's frames.
pub const ARROW_FRAMES: i64 = 16;
/// Rule 37: worlds with a "teleporter earned" mark on Connor (map indexes
/// 0–7).
pub const TELEPORT_MARKS: usize = 8;

/// An 8-bit picture: palette indices, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPic {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// Reads an icon file (`dv_tree2.bmp`) for [`MapState`]; the caller
/// knows where the files are.
pub type IconReader = Box<dyn Fn(&str) -> Result<IndexPic, String> + Send + Sync>;

/// Rule 4: world (x, y) → map pixel; `None` when the box has no width or
/// height (the map then does nothing).
pub fn pixel_of(keys: &MapKeys, x: f64, y: f64) -> Option<(i32, i32)> {
    let across = keys.east - keys.west;
    let down = (keys.north - keys.south).abs();
    if across == 0.0 || down == 0.0 {
        return None;
    }
    let u = (x - keys.west) / across;
    let v = 1.0 - (y - keys.south) / down;
    Some(((u * MAP_SIZE as f64) as i32, (v * MAP_SIZE as f64) as i32))
}

/// Rule 13: the fog radius in map pixels, and where its world units came
/// from.
pub fn fog_radius(keys: &MapKeys, haze: Option<f64>) -> (i32, f64, &'static str) {
    let (units, from) = match (keys.fog_radius.filter(|r| *r > 0.0), haze) {
        (Some(r), _) => (r, "fogRadius"),
        (None, Some(h)) => (h, "hazeDistance"),
        (None, None) => (NO_TERRAIN_FOG, "no terrain: 20,000"),
    };
    let across = keys.east - keys.west;
    let r = if across != 0.0 { (MAP_SIZE as f64 * units / across) as i32 } else { 0 };
    (r, units, from)
}

/// Whole-number square root (rounded down).
fn isqrt(n: i64) -> i64 {
    if n <= 0 {
        return 0;
    }
    let mut r = (n as f64).sqrt() as i64;
    while r * r > n {
        r -= 1;
    }
    while (r + 1) * (r + 1) <= n {
        r += 1;
    }
    r
}

/// Rule 12: the disc's rows as (row offset d, half-width w): columns
/// cx − w to cx + w − 1 of row cy + d.
pub fn disc(r: i32) -> Vec<(i32, i32)> {
    let r = r as i64;
    (1 - r..=r)
        .map(|d| {
            let e = if d <= 0 { 1 - d } else { d };
            (d as i32, isqrt(r * r - e * e) as i32)
        })
        .collect()
}

/// Rule 17: the arrow frame for Connor's heading (0 = facing −Y, south;
/// positive = turned anticlockwise).
pub fn arrow_frame(heading: f64) -> usize {
    let f = ((std::f64::consts::PI - heading) / std::f64::consts::TAU * ARROW_FRAMES as f64).round() as i64;
    f.rem_euclid(ARROW_FRAMES) as usize
}

/// Rule 22: where the map picture is drawn in the map area: a rectangle
/// in 640 × 480 layout pixels measured from the area's centre (STAND-IN
/// at 800 × 600, open question 6: scaled with the screen like the
/// panels).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub left: f64,
    pub top: f64,
    pub right: f64,
    pub bottom: f64,
}

/// Rule 22: a new game's view (the whole picture at about × 0.648).
pub const START_VIEW: View = View { left: -152.83, top: -152.11, right: 145.44, bottom: 146.16 };
/// Rule 18: the arrow's size; its centre is on Connor.
pub const ARROW_HALF: f64 = 5.0;

impl View {
    pub fn width(&self) -> f64 {
        self.right - self.left
    }

    pub fn height(&self) -> f64 {
        self.bottom - self.top
    }

    /// Rule 18: the arrow's top-left corner for map pixel `p`, from the
    /// area's centre: the pixel scaled by the shown size, plus the
    /// picture's top-left, minus half the arrow.
    pub fn arrow_at(&self, p: (i32, i32)) -> (f64, f64) {
        (self.left + p.0 as f64 * self.width() / MAP_SIZE as f64 - ARROW_HALF, self.top + p.1 as f64 * self.height() / MAP_SIZE as f64 - ARROW_HALF)
    }

    /// The map pixel under point (`x`, `y`) from the area's centre
    /// (fractions kept).
    pub fn map_point(&self, x: f64, y: f64) -> (f64, f64) {
        ((x - self.left) / self.width() * MAP_SIZE as f64, (y - self.top) / self.height() * MAP_SIZE as f64)
    }

    pub fn as_array(&self) -> [f64; 4] {
        [self.left, self.top, self.right, self.bottom]
    }
}

impl Default for View {
    fn default() -> Self {
        START_VIEW
    }
}

/// What one update did (rule 9), for the log.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Connor's map pixel (`None` with stop location on, or no box).
    pub pixel: Option<(i32, i32)>,
    /// It differs from the last one: the reveal ran.
    pub moved: bool,
    /// Pixels newly out of the fog.
    pub revealed: usize,
    pub frame: usize,
}

/// Connor's per-map marks (rules 33, 37), kept on him across worlds.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnorMapMarks {
    /// "Visited", by map index.
    pub visited: [bool; MAP_COUNT],
    /// "Teleporter earned", map indexes 0–7.
    pub teleport_earned: [bool; TELEPORT_MARKS],
    /// `SageGnomeEarnedTeleport` (rule 37).
    pub sage_gnome: bool,
}

/// The current world's map (rule 3: one at a time).
pub struct MapState {
    /// The file it came from (`Davmap.kq`).
    pub file: String,
    pub keys: MapKeys,
    /// Rule 3: 0–15, `None` if its `mapName` is not one of them.
    pub index: Option<usize>,
    /// Rule 2.
    pub active: bool,
    /// Rule 15: stop location (`holdMapLoc`, `KQMap::stopLoc`).
    pub hold: bool,
    /// Rule 14.
    pub use_fog: bool,
    /// The world's `hazeDistance` (rule 13's fallback).
    haze: Option<f64>,
    /// Rule 13, map pixels.
    pub radius: i32,
    /// Rule 16: out of the fog, per map pixel.
    pub fog: Revealed,
    /// The blank and full pictures (460 × 460; `None`: not readable, the
    /// map's own picture stays black).
    blank: Option<Vec<u8>>,
    full: Option<Vec<u8>>,
    /// Rule 7: the map's own picture, 460 × 460 palette indices.
    pub picture: Vec<u8>,
    /// Rules 20–21: the icons this world's map object keeps.
    pub icons: Vec<MapIcon>,
    icon_pics: BTreeMap<String, Option<IndexPic>>,
    reader: Option<IconReader>,
    /// Rule 11: the last pixel; `None` = unset.
    pub last_pixel: Option<(i32, i32)>,
    /// Rule 17.
    pub frame: usize,
    /// Rule 37 (saved with the map).
    pub earned_teleport: bool,
    /// Counts every change of [`MapState::picture`], for redrawing.
    pub version: u64,
    /// Rule 22: the big map's view (its changes are stored on Connor by
    /// the caller); rule 27: the mini-map's own copy. Both taken from the
    /// view stored on Connor when the map is set up ([`MapState::set_views`]).
    pub view: View,
    pub mini_view: View,
    /// Lines for `map.log`, taken by the caller.
    pub log: Vec<String>,
}

impl std::fmt::Debug for MapState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MapState").field("file", &self.file).field("index", &self.index).field("active", &self.active).field("radius", &self.radius).finish()
    }
}

/// A picture is usable only at the map's size.
fn map_sized(p: Option<IndexPic>) -> Option<Vec<u8>> {
    p.filter(|p| p.width == MAP_SIZE && p.height == MAP_SIZE && p.pixels.len() == MAP_SIZE * MAP_SIZE).map(|p| p.pixels)
}

impl MapState {
    /// Rules 7, 13, 16: a map as loaded: its fog from its runs, its own
    /// picture made, the fog radius worked out.
    pub fn new(file: &str, keys: MapKeys, blank: Option<IndexPic>, full: Option<IndexPic>, haze: Option<f64>, reader: Option<IconReader>) -> Self {
        let (fog, _) = fog_from_runs(&keys.runs);
        let (radius, ..) = fog_radius(&keys, haze);
        let mut m = MapState {
            file: file.to_string(),
            index: keys.index(),
            active: keys.is_active,
            hold: keys.hold_map_loc,
            use_fog: keys.use_fog,
            haze,
            radius,
            fog,
            blank: map_sized(blank),
            full: map_sized(full),
            picture: vec![0; MAP_SIZE * MAP_SIZE],
            icons: keys.icons.clone(),
            icon_pics: BTreeMap::new(),
            reader,
            last_pixel: None,
            frame: 0,
            earned_teleport: keys.earned_teleport,
            version: 0,
            view: START_VIEW,
            mini_view: START_VIEW,
            log: Vec::new(),
            keys,
        };
        m.rebuild();
        m
    }

    fn note(&mut self, line: String) {
        self.log.push(line);
    }

    /// The picture's name icons are kept under when it is shown (rule 20:
    /// the saved icons name the full picture).
    pub fn shown_name(&self) -> &str {
        &self.keys.full_map
    }

    /// Rule 7: blank, then full where out of the fog (rule 14: all of it
    /// without fog), then the icons.
    pub fn rebuild(&mut self) {
        if !self.use_fog {
            // "Do not reproduce" 3: what is shown is marked.
            self.fog.iter_mut().for_each(|v| *v = true);
        }
        self.picture = self.blank.clone().unwrap_or_else(|| vec![0; MAP_SIZE * MAP_SIZE]);
        if let Some(full) = &self.full {
            for (k, out) in self.fog.iter().enumerate() {
                if *out {
                    self.picture[k] = full[k];
                }
            }
        }
        self.draw_icons();
    }

    fn icon_pic(&mut self, file: &str) -> Option<IndexPic> {
        let key = file.to_ascii_lowercase();
        if !self.icon_pics.contains_key(&key) {
            let read = match &self.reader {
                Some(r) => r(file),
                None => Err("no reader".into()),
            };
            let pic = match read {
                Ok(p) => {
                    self.log.push(format!("icon {file}: {}x{} read", p.width, p.height));
                    Some(p)
                }
                Err(e) => {
                    self.log.push(format!("icon {file}: {e}; not drawn"));
                    None
                }
            };
            self.icon_pics.insert(key.clone(), pic);
        }
        self.icon_pics.get(&key).cloned().flatten()
    }

    /// Rule 19: one icon on the map's own picture, index 0 see-through.
    fn stamp(&mut self, icon: &MapIcon) {
        let Some(pic) = self.icon_pic(&icon.file) else { return };
        for y in 0..pic.height {
            for x in 0..pic.width {
                let v = pic.pixels[y * pic.width + x];
                let (mx, my) = (icon.x + x as i32, icon.y + y as i32);
                if v == 0 || mx < 0 || my < 0 || mx >= MAP_SIZE as i32 || my >= MAP_SIZE as i32 {
                    continue;
                }
                self.picture[my as usize * MAP_SIZE + mx as usize] = v;
            }
        }
    }

    /// Rule 9: the icons of the shown picture drawn again.
    pub fn draw_icons(&mut self) {
        let shown = self.shown_name().to_string();
        let icons: Vec<MapIcon> = self.icons.iter().filter(|i| i.picture.eq_ignore_ascii_case(&shown)).cloned().collect();
        for i in &icons {
            self.stamp(i);
        }
        self.version += 1;
    }

    /// Rule 12: the disc around (cx, cy) copied from the full picture and
    /// marked out of the fog; returns the pixels newly out.
    pub fn reveal(&mut self, cx: i32, cy: i32) -> usize {
        let mut newly = 0;
        for (d, w) in disc(self.radius) {
            let y = cy + d;
            if y < 0 || y >= MAP_SIZE as i32 {
                continue;
            }
            for x in (cx - w).max(0)..(cx + w).min(MAP_SIZE as i32) {
                let k = y as usize * MAP_SIZE + x as usize;
                if !self.fog[k] {
                    self.fog[k] = true;
                    newly += 1;
                }
                if let Some(full) = &self.full {
                    self.picture[k] = full[k];
                }
            }
        }
        self.version += 1;
        newly
    }

    /// Rule 9: one update with Connor at (x, y) facing `heading`. `None`
    /// while the map is not active. The caller marks the map visited on
    /// Connor when `moved`.
    pub fn update(&mut self, x: f64, y: f64, heading: f64) -> Option<Step> {
        if !self.active {
            return None;
        }
        self.frame = arrow_frame(heading);
        let pixel = if self.hold { None } else { pixel_of(&self.keys, x, y) };
        let mut step = Step { pixel, moved: false, revealed: 0, frame: self.frame };
        if let Some(p) = pixel.filter(|p| Some(*p) != self.last_pixel) {
            self.last_pixel = Some(p);
            step.moved = true;
            step.revealed = self.reveal(p.0, p.1);
            self.draw_icons();
        }
        Some(step)
    }

    /// Rules 22, 27: the views when the map is set up, from the one
    /// stored on Connor (STAND-IN, open question 3: also after a world
    /// change).
    pub fn set_views(&mut self, stored: View) {
        self.view = stored;
        self.mini_view = stored;
    }

    /// Rule 2.
    pub fn set_active(&mut self, on: bool, why: &str) {
        if self.active != on {
            self.active = on;
            if on {
                // Rule 2: activating works out the fog circle again.
                self.radius = fog_radius(&self.keys, self.haze).0;
            }
            let line = format!("{why}: the map {} {} (rule 2); fog radius {} map pixels", self.file, if on { "is active" } else { "is not active" }, self.radius);
            self.note(line);
        }
    }

    /// `KQMap::setFogRadius n` (rule 13).
    pub fn set_fog_radius(&mut self, units: f64) {
        self.keys.fog_radius = Some(units);
        let (r, units, from) = fog_radius(&self.keys, self.haze);
        self.radius = r;
        self.note(format!("setFogRadius: {units} ({from}) = {r} map pixels (rule 13)"));
    }

    /// `KQMap::useFog 0|1` (rule 14).
    pub fn set_use_fog(&mut self, on: bool) {
        self.use_fog = on;
        if !on {
            self.rebuild();
        }
    }

    /// `addMapIcon file x y [picture]` (rule 20): `true` if added.
    pub fn add_icon(&mut self, file: &str, x: i32, y: i32, picture: Option<&str>) -> bool {
        let picture = picture.unwrap_or(self.shown_name()).to_string();
        let icon = MapIcon { picture, file: file.to_string(), x, y };
        let same = |a: &MapIcon| a.file.eq_ignore_ascii_case(&icon.file) && a.x == icon.x && a.y == icon.y && a.picture.eq_ignore_ascii_case(&icon.picture);
        if self.icons.iter().any(same) {
            return false;
        }
        let shown_now = icon.picture.eq_ignore_ascii_case(self.shown_name());
        self.icons.push(icon.clone());
        if shown_now {
            self.stamp(&icon);
            self.version += 1;
        }
        true
    }

    /// `removeMapIcon x y` (rule 21): the icon at (x, y) on the shown
    /// picture removed and its rectangle painted back; `None` if there is
    /// none, else the icon removed.
    pub fn remove_icon(&mut self, x: i32, y: i32) -> Option<MapIcon> {
        let shown = self.shown_name().to_string();
        let k = self.icons.iter().position(|i| i.x == x && i.y == y && i.picture.eq_ignore_ascii_case(&shown))?;
        let icon = self.icons.remove(k);
        if let Some(pic) = self.icon_pic(&icon.file) {
            for py in y.max(0)..(y + pic.height as i32).min(MAP_SIZE as i32) {
                for px in x.max(0)..(x + pic.width as i32).min(MAP_SIZE as i32) {
                    let k = py as usize * MAP_SIZE + px as usize;
                    let blank = self.blank.as_ref().map_or(0, |b| b[k]);
                    self.picture[k] = match &self.full {
                        Some(full) if self.fog[k] => full[k],
                        _ => blank,
                    };
                }
            }
            self.version += 1;
        }
        Some(icon)
    }

    /// Rule 16: the map's keys as they would be saved now (fog runs,
    /// icons, flags).
    pub fn saved_keys(&self) -> MapKeys {
        MapKeys {
            is_active: self.active,
            hold_map_loc: self.hold,
            use_fog: self.use_fog,
            earned_teleport: self.earned_teleport,
            icons: self.icons.clone(),
            runs: runs_from_fog(&self.fog),
            ..self.keys.clone()
        }
    }
}

/// The category of the item that switches the map on (rule 1).
pub const MAP_CATEGORY: &str = "MagicMap";

impl crate::active::WorldState {
    fn map_note(&mut self, line: String) {
        self.note(&line);
        self.map_log.push(line);
    }

    /// `KQMap::activate`, `deactivate`, `stopLoc 0|1`, `useFog 0|1`,
    /// `setFogRadius n`, `addMapIcon file x y [picture]`, `removeMapIcon
    /// x y` (rules 2, 13–15, 20, 21).
    pub(crate) fn map_command(&mut self, words: &[String], at: &str) -> String {
        use crate::script::{looks_like_number, number_value};
        let all = words.join(" ");
        let Some(map) = self.map.as_mut() else {
            self.map_note(format!("{at}: {all}: KQMap not loaded (rule 3); nothing"));
            return "1".into();
        };
        let num = |k: usize| words.get(k).filter(|v| looks_like_number(v)).map(|v| number_value(v));
        let line = match words[0].as_str() {
            "KQMap::activate" | "KQMap::deactivate" => {
                map.set_active(words[0] == "KQMap::activate", &format!("{at}: {all}"));
                None
            }
            "KQMap::stopLoc" => match num(1) {
                Some(v) => {
                    map.hold = v != 0.0;
                    Some(format!("{at}: {all}: stop location {} (rule 15)", if map.hold { "on: Connor's position ignored" } else { "off" }))
                }
                None => Some(format!("{at}: usage: KQMap::stopLoc 0|1; nothing")),
            },
            "KQMap::useFog" => match num(1) {
                Some(v) => {
                    map.set_use_fog(v != 0.0);
                    Some(format!("{at}: {all}: useFog {} (rule 14{})", u8::from(map.use_fog), if map.use_fog { "" } else { "; the whole map shown and marked, Do not reproduce 3" }))
                }
                None => Some(format!("{at}: usage: KQMap::useFog 0|1; nothing")),
            },
            "KQMap::setFogRadius" => match num(1) {
                Some(v) => {
                    map.set_fog_radius(v);
                    None
                }
                None => Some(format!("{at}: usage: KQMap::setFogRadius n; nothing")),
            },
            "addMapIcon" => match (words.get(1), num(2), num(3)) {
                (Some(file), Some(x), Some(y)) => {
                    let picture = words.get(4).map(String::as_str);
                    let shown = map.shown_name().to_string();
                    let added = map.add_icon(file, x as i32, y as i32, picture);
                    let on = picture.unwrap_or(&shown);
                    Some(format!(
                        "{at}: {all}: {} (rule 20; {} icons)",
                        match (added, on.eq_ignore_ascii_case(&shown)) {
                            (false, _) => format!("{file} at ({x}, {y}) on {on} already there: not added"),
                            (true, true) => format!("{file} at ({x}, {y}) on {on}, the picture shown: drawn"),
                            (true, false) => format!("{file} at ({x}, {y}) kept for {on} (drawn when it is shown)"),
                        },
                        map.icons.len()
                    ))
                }
                _ => Some(format!("{at}: usage: addMapIcon file x y [picture]; nothing")),
            },
            _ => match (num(1), num(2)) {
                (Some(x), Some(y)) => Some(match map.remove_icon(x as i32, y as i32) {
                    Some(i) => format!("{at}: {all}: {} removed from {}, painted back (rule 21)", i.file, i.picture),
                    None => format!("{at}: {all}: no icon at ({x}, {y}) on {}; nothing (rule 21)", map.shown_name()),
                }),
                _ => Some(format!("{at}: usage: removeMapIcon x y; nothing")),
            },
        };
        let mut lines: Vec<String> = std::mem::take(&mut map.log);
        lines.extend(line);
        for l in lines {
            self.map_note(l);
        }
        "0".into()
    }

    /// One update (rule 9): the `MagicMap` item arriving or leaving
    /// switches the map (rule 2); then Connor at `at` facing `heading`.
    pub fn map_step(&mut self, at: [f32; 3], heading: f32) {
        let carried = self.items.objects.iter().any(|o| o.carried_by("Connor") && self.items.kind(&o.kind).is_some_and(|t| t.category == MAP_CATEGORY));
        let before = self.map_item.replace(carried).unwrap_or(false);
        let updates = self.updates;
        let Some(map) = self.map.as_mut() else { return };
        if carried != before {
            map.set_active(carried, &format!("update {updates}: the MagicMap item {}", if carried { "arrives" } else { "leaves" }));
        }
        let last_frame = map.frame;
        let step = map.update(at[0] as f64, at[1] as f64, heading as f64);
        let index = map.index;
        let mut lines: Vec<String> = std::mem::take(&mut map.log);
        if let Some(s) = &step {
            let total = map.fog.iter().filter(|v| **v).count();
            if s.moved || s.frame != last_frame {
                lines.push(format!(
                    "update {updates}: Connor ({:.0}, {:.0}) heading {heading:.3} -> {}; arrow frame {}{}",
                    at[0],
                    at[1],
                    match s.pixel {
                        Some((x, y)) => format!("map pixel ({x}, {y})"),
                        None => "stop location: position ignored".into(),
                    },
                    s.frame,
                    if s.moved { format!("; new pixel: {} revealed (radius {}), {total} out of the fog", s.revealed, map.radius) } else { String::new() }
                ));
            }
            if s.moved {
                if let Some(i) = index.filter(|i| !self.map_marks.visited[*i]) {
                    self.map_marks.visited[i] = true;
                    lines.push(format!("update {updates}: map index {i} marked visited on Connor (rule 33)"));
                }
            }
        }
        self.map_log.extend(lines);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn keys() -> MapKeys {
        MapKeys {
            name: "davMap".into(),
            north: 344068.0,
            south: 155670.0,
            east: 303114.0,
            west: 114699.0,
            use_fog: true,
            fog_radius: Some(10000.0),
            hold_map_loc: false,
            is_active: false,
            blank_map: "davmap0.pbm".into(),
            full_map: "davmap.pbm".into(),
            arrow_pba: "maparrow.pba".into(),
            map_name: Some(38),
            earned_teleport: false,
            teleport_script: None,
            teleport_portal: None,
            icons: Vec::new(),
            runs: Vec::new(),
        }
    }

    fn pics() -> (Option<IndexPic>, Option<IndexPic>) {
        let p = |v: u8| Some(IndexPic { width: MAP_SIZE, height: MAP_SIZE, pixels: vec![v; MAP_SIZE * MAP_SIZE] });
        (p(1), p(2))
    }

    #[test]
    fn world_to_map_pixel_north_up() {
        let k = keys();
        // The box's north-west corner is pixel (0, 0); its centre 230.
        assert_eq!(pixel_of(&k, 114699.0, 344068.0), Some((0, 0)));
        assert_eq!(pixel_of(&k, (114699.0 + 303114.0) / 2.0, (155670.0 + 344068.0) / 2.0), Some((230, 230)));
        // 409.6 units a pixel: one unit less than a whole pixel drops.
        assert_eq!(pixel_of(&k, 114699.0 + 409.6 * 10.0 - 1.0, 344068.0), Some((9, 0)));
        // Outside the box: no clamp; west of it rounds towards zero.
        assert_eq!(pixel_of(&k, 114699.0 - 1000.0, 155670.0 - 1000.0).map(|p| p.0), Some(-2));
        assert_eq!(pixel_of(&k, 114699.0 - 100.0, 344068.0).map(|p| p.0), Some(0));
        assert_eq!(pixel_of(&MapKeys { east: 114699.0, ..k }, 0.0, 0.0), None);
    }

    #[test]
    fn fog_radius_in_map_pixels() {
        assert_eq!(fog_radius(&keys(), None).0, 24);
        // Missing: the world's hazeDistance, else 20,000 (Snow exit 131).
        let snow = MapKeys { west: 80363.0, east: 143372.0, fog_radius: None, ..keys() };
        assert_eq!(fog_radius(&snow, Some(18000.0)).0, 131);
        assert_eq!(fog_radius(&MapKeys { fog_radius: Some(0.0), ..keys() }, None).1, 20000.0);
    }

    #[test]
    fn the_disc_rows() {
        // r = 3: rows -2..3; e = 3, 2, 1, 1, 2, 3; w = 0, 2, 2, 2, 2, 0.
        assert_eq!(disc(3), vec![(-2, 0), (-1, 2), (0, 2), (1, 2), (2, 2), (3, 0)]);
        // The widest rows are r - 1 (e is never 0).
        let d24 = disc(24);
        assert!(d24.iter().all(|(_, w)| *w <= 23));
        assert!(d24.iter().filter(|(d, _)| *d == 0 || *d == 1).all(|(_, w)| *w == 23), "Connor's row and the one below are widest");
        assert_eq!((d24[0], d24[47]), ((-23, 0), (24, 0)), "the top and bottom rows have w 0");
    }

    #[test]
    fn reveal_copies_and_marks_the_same_columns() {
        let (b, f) = pics();
        let mut m = MapState::new("davmap.kq", keys(), b, f, None, None);
        m.radius = 3;
        // Rows 99..=104, half-widths 0, 2, 2, 2, 2, 0 → 4 × 4 = 16 pixels.
        assert_eq!(m.reveal(100, 100), 16);
        assert!(m.fog[100 * MAP_SIZE + 98] && m.fog[100 * MAP_SIZE + 101] && !m.fog[100 * MAP_SIZE + 102], "Do not reproduce 2: the copied columns only");
        assert_eq!((m.picture[100 * MAP_SIZE + 98], m.picture[100 * MAP_SIZE + 102]), (2, 1));
        // Again: nothing new. At the corner: cut at the edges.
        assert_eq!(m.reveal(100, 100), 0);
        // Rows 0-3 (d 0, 1, 2 have w 2; d 3 has 0): columns 0-1 each.
        assert_eq!(m.reveal(0, 0), 6);
    }

    #[test]
    fn the_start_view_and_the_arrow() {
        let v = START_VIEW;
        assert!((v.width() / MAP_SIZE as f64 - 0.648).abs() < 0.001);
        // Pixel (0, 0): the picture's corner less half the arrow.
        assert_eq!(v.arrow_at((0, 0)), (-152.83 - 5.0, -152.11 - 5.0));
        let (x, y) = v.arrow_at((230, 230));
        assert!((x - (-152.83 + 230.0 * v.width() / 460.0 - 5.0)).abs() < 1e-9 && (y + 5.0 - (-152.11 + 0.5 * v.height())).abs() < 1e-9);
        let (mx, my) = v.map_point(-152.83, 146.16);
        assert!((mx - 0.0).abs() < 1e-9 && (my - 460.0).abs() < 1e-9);
    }

    #[test]
    fn arrow_frames() {
        use std::f64::consts::PI;
        assert_eq!(arrow_frame(PI), 0, "facing north: up");
        assert_eq!(arrow_frame(PI / 2.0), 4, "facing east: right");
        assert_eq!(arrow_frame(0.0), 8, "facing south: down");
        assert_eq!(arrow_frame(-PI / 2.0), 12, "facing west: left");
        // Half a frame is 11.25 deg (0.196 rad).
        assert_eq!(arrow_frame(PI + 0.19), 0, "nearest frame");
        assert_eq!(arrow_frame(PI + 0.2), 15);
        assert_eq!(arrow_frame(PI - 0.2), 1);
        assert_eq!(arrow_frame(3.0 * PI), 0);
    }

    #[test]
    fn updates_reveal_only_on_a_new_pixel() {
        let (b, f) = pics();
        let mut m = MapState::new("davmap.kq", keys(), b, f, None, None);
        assert_eq!(m.update(200000.0, 250000.0, 0.0), None, "not active");
        m.set_active(true, "test");
        let s = m.update(200000.0, 250000.0, std::f64::consts::PI).unwrap();
        assert!(s.moved && s.revealed > 1500 && s.frame == 0);
        let s = m.update(200100.0, 250000.0, 0.0).unwrap();
        assert_eq!((s.moved, s.revealed, s.frame), (false, 0, 8), "same pixel: only the arrow turns");
        m.hold = true;
        let s = m.update(220000.0, 250000.0, 0.0).unwrap();
        assert_eq!((s.pixel, s.moved), (None, false), "stop location");
    }

    #[test]
    fn icons_over_the_fog_and_removed() {
        let (b, f) = pics();
        let icon = IndexPic { width: 2, height: 2, pixels: vec![7, 0, 7, 7] };
        let reader: IconReader = Box::new(move |_| Ok(icon.clone()));
        let mut m = MapState::new("davmap.kq", keys(), b, f, None, Some(reader));
        assert!(m.add_icon("dv_telep.bmp", 250, 98, None));
        assert!(!m.add_icon("DV_TELEP.BMP", 250, 98, Some("davmap.pbm")), "not twice");
        assert!(m.add_icon("dv_poisn.bmp", 10, 10, Some("davmap0.pbm")), "kept for another picture");
        let at = |m: &MapState, x: usize, y: usize| m.picture[y * MAP_SIZE + x];
        assert_eq!((at(&m, 250, 98), at(&m, 251, 98), at(&m, 251, 99)), (7, 1, 7), "over the fog; 0 see-through");
        assert_eq!(at(&m, 10, 10), 1, "not the shown picture: not drawn");
        m.reveal(250, 98);
        m.draw_icons();
        assert_eq!(at(&m, 250, 98), 7, "drawn again after a reveal");
        assert!(m.remove_icon(250, 98).is_some());
        assert_eq!((at(&m, 250, 98), at(&m, 251, 99)), (2, 2), "painted back: full where revealed");
        assert!(m.remove_icon(250, 98).is_none());
        assert_eq!(m.saved_keys().icons.len(), 1);
    }

    #[test]
    fn saved_fog_comes_back() {
        let (b, f) = pics();
        let mut m = MapState::new("davmap.kq", keys(), b.clone(), f.clone(), None, None);
        m.reveal(300, 300);
        let saved = m.saved_keys();
        let again = MapState::new("davmap.kq", saved, b, f, None, None);
        assert_eq!(again.fog, m.fog);
        assert_eq!(again.picture, m.picture);
        // useFog 0: all shown and marked (Do not reproduce 3).
        let mut m = again;
        m.set_use_fog(false);
        assert!(m.fog.iter().all(|v| *v) && m.picture.iter().all(|v| *v == 2));
    }

    #[test]
    fn script_commands_and_the_update() {
        use crate::active::Active;
        let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        assert_eq!(a.test_command("KQMap::activate", "t").as_deref(), Some("1"), "no map: KQMap not loaded");
        let (b, f) = pics();
        a.world.map = Some(MapState::new("Davmap.kq", keys(), b, f, None, None));
        a.world.map_step([200000.0, 250000.0, 0.0], 0.0);
        assert_eq!(a.world.map.as_ref().unwrap().last_pixel, None, "not active: nothing");
        assert_eq!(a.test_command("KQMap::activate", "t").as_deref(), Some("0"));
        a.world.map_step([200000.0, 250000.0, 0.0], 0.0);
        let m = a.world.map.as_ref().unwrap();
        assert_eq!((m.last_pixel, m.frame), (Some((208, 229)), 8));
        assert!(a.world.map_marks.visited[0]);
        assert_eq!(a.test_command("KQMap::stopLoc 1", "t").as_deref(), Some("0"));
        a.world.map_step([220000.0, 250000.0, 0.0], 3.1416);
        let m = a.world.map.as_ref().unwrap();
        assert_eq!((m.last_pixel, m.frame), (Some((208, 229)), 0), "stop location: the arrow only turns");
        a.test_command("KQMap::setFogRadius 5000", "t");
        assert_eq!(a.world.map.as_ref().unwrap().radius, 12);
        assert_eq!(a.test_command("removeMapIcon 1 2", "t").as_deref(), Some("0"));
        assert!(a.world.map_log.iter().any(|l| l.contains("no icon at (1, 2)")), "{:?}", a.world.map_log);
    }
}
