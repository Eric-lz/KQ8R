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
/// Rule 37: the Sage Gnome's `earnTeleport <file>` marks Barren.
pub const BARREN: usize = 4;
/// Rule 38: the unlit "Go" button's help (`500.MSG` 0 0 151).
pub const GO_HELP: u32 = 151;

/// An 8-bit picture: palette indices, top row first.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct IndexPic {
    pub width: usize,
    pub height: usize,
    pub pixels: Vec<u8>,
}

/// Reads a picture file (an icon `dv_tree2.bmp`, another world's map
/// picture) for [`MapState`]; the caller knows where the files are.
pub type IconReader = std::sync::Arc<dyn Fn(&str) -> Result<IndexPic, String> + Send + Sync>;

/// An RGB palette of 256 entries (rule 34: `GAME/Resource/<name>.ppl`).
pub type Colours = [[u8; 3]; 256];

/// Rule 34: the palette files by map index.
pub const PALETTES: [&str; MAP_COUNT] = [
    "daventry.ppl",
    "newdead.ppl",
    "swamp.ppl",
    "gnome.ppl",
    "barren.ppl",
    "dwarf.ppl",
    "ice.ppl",
    "temple01.ppl",
    "temple02.ppl",
    "temple03.ppl",
    "temple04.ppl",
    "icelord.ppl",
    "icelord.ppl",
    "icelord.ppl",
    "castle.ppl",
    "exit.ppl",
];

/// "Do not reproduce" 6: the maps no script loads; never shown, their
/// indexes kept.
pub const UNUSED_MAPS: [&str; 4] = ["dwrfmap.kq", "ice1map.kq", "ice2map.kq", "ice3map.kq"];

/// Rule 34: each entry of `from` replaced by the entry of `to` with the
/// smallest squared RGB difference (any of the 256; the first on a tie).
pub fn match_palette(from: &Colours, to: &Colours) -> [u8; 256] {
    let mut out = [0u8; 256];
    for (k, c) in from.iter().enumerate() {
        let d = |t: &[u8; 3]| (0..3).map(|i| (c[i] as i32 - t[i] as i32).pow(2)).sum::<i32>();
        out[k] = (0..256).min_by_key(|&i| d(&to[i])).unwrap_or(0) as u8;
    }
    out
}

/// Rule 19: an icon on a picture, top-left at (x, y), index 0 see-through.
fn stamp_on(picture: &mut [u8], pic: &IndexPic, x0: i32, y0: i32) {
    for y in 0..pic.height {
        for x in 0..pic.width {
            let v = pic.pixels[y * pic.width + x];
            let (mx, my) = (x0 + x as i32, y0 + y as i32);
            if v == 0 || mx < 0 || my < 0 || mx >= MAP_SIZE as i32 || my >= MAP_SIZE as i32 {
                continue;
            }
            picture[my as usize * MAP_SIZE + mx as usize] = v;
        }
    }
}

/// Another world's map in the window (rule 34).
#[derive(Debug, Clone)]
pub struct OtherMap {
    pub index: usize,
    /// Its file (`davMap.kq`) and where its keys came from (kept or the
    /// shipped file).
    pub file: String,
    pub source: String,
    pub keys: MapKeys,
    /// Its picture in the current world's colours.
    pub picture: Vec<u8>,
}

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
/// Rule 5: the map area, 296 × 296 layout pixels.
pub const AREA_SIZE: f64 = 296.0;
/// Rule 24: sideways mouse movement that doubles the size.
pub const ZOOM_DOUBLING: f64 = 64.0;
/// Rule 24: the shown width's limits, × 0.25 and × 8 of the picture.
pub const ZOOM_MIN: f64 = MAP_SIZE as f64 / 4.0;
pub const ZOOM_MAX: f64 = MAP_SIZE as f64 * 8.0;
/// Rule 25, STAND-IN (open question 2, DESIGN 6d): the arrow's top-left
/// corner is kept at least this far inside the visible map area on all
/// four sides.
pub const FOLLOW_MARGIN: f64 = 20.0;
/// Rule 25: the step is (distance past the margin) ÷ this + 1 pixels.
pub const FOLLOW_DIVISOR: f64 = 10.0;
/// Rule 25: steps at most per draw.
pub const FOLLOW_STEPS: usize = 6;

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

    /// Rule 23: the picture moved by (`dx`, `dy`) layout pixels, kept so
    /// that the area's centre stays on it (its left edge not right of the
    /// centre, its right edge not left of it; the same up and down).
    pub fn pan(&mut self, dx: f64, dy: f64) {
        let dx = dx.min(-self.left).max(-self.right);
        let dy = dy.min(-self.top).max(-self.bottom);
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
    }

    /// Rule 24: a sideways movement of `dx` layout pixels multiplies the
    /// size by 2^(dx ÷ 64) about the area's centre, the shown width kept
    /// between 115 and 3,680.
    pub fn zoom(&mut self, dx: f64) {
        let w = self.width();
        if w <= 0.0 {
            return;
        }
        let wanted = (w * (dx / ZOOM_DOUBLING).exp2()).clamp(ZOOM_MIN, ZOOM_MAX);
        let f = wanted / w;
        self.left *= f;
        self.right *= f;
        self.top *= f;
        self.bottom *= f;
    }

    /// Rule 25 (stand-in margins): one follow step for the arrow at map
    /// pixel `p`; `false` when its top-left corner is already inside.
    pub fn follow_once(&mut self, p: (i32, i32)) -> bool {
        let (x, y) = self.arrow_at(p);
        let half = AREA_SIZE / 2.0;
        let (lo, hi) = (FOLLOW_MARGIN, AREA_SIZE - FOLLOW_MARGIN);
        let step = |v: f64| {
            let v = v + half;
            if v < lo {
                ((lo - v) / FOLLOW_DIVISOR).trunc() + 1.0
            } else if v > hi {
                -(((v - hi) / FOLLOW_DIVISOR).trunc() + 1.0)
            } else {
                0.0
            }
        };
        let (dx, dy) = (step(x), step(y));
        if dx == 0.0 && dy == 0.0 {
            return false;
        }
        self.left += dx;
        self.right += dx;
        self.top += dy;
        self.bottom += dy;
        true
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
    /// Rule 25: Connor's pixel changed since the view last had the arrow
    /// inside its margins.
    pub follow: bool,
    /// Rule 22: the big map's view (its changes are stored on Connor by
    /// the caller); rule 27: the mini-map's own copy. Both taken from the
    /// view stored on Connor when the map is set up ([`MapState::set_views`]).
    pub view: View,
    pub mini_view: View,
    /// Rule 34: another world's map shown now (`None`: this world's).
    pub other: Option<OtherMap>,
    /// Rule 36: Connor is on the teleporter pad (not saved).
    pub on_pad: bool,
    /// Rule 38: the "Go" button is lit.
    pub go_lit: bool,
    /// Rule 34: the palettes by map index (set by the caller).
    pub palettes: Vec<Option<Colours>>,
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
            follow: false,
            view: START_VIEW,
            mini_view: START_VIEW,
            other: None,
            on_pad: false,
            go_lit: false,
            palettes: Vec::new(),
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
    /// the saved icons name the full picture): another world's when it is
    /// shown (rule 34).
    pub fn shown_name(&self) -> &str {
        self.other.as_ref().map_or(&self.keys.full_map, |o| &o.keys.full_map)
    }

    /// The map index shown now (rule 32).
    pub fn shown_index(&self) -> Option<usize> {
        self.other.as_ref().map(|o| o.index).or(self.index)
    }

    /// The picture the big map shows now.
    pub fn shown_picture(&self) -> &[u8] {
        self.other.as_ref().map_or(&self.picture, |o| &o.picture)
    }

    /// Rule 34: another world's map shown: its own picture (with its fog if
    /// its `useFog` is on, else its full picture; its own icons), this
    /// world's icons kept for that picture, then its colours matched to
    /// this world's palette.
    pub fn show_other(&mut self, index: usize, file: &str, source: &str, keys: MapKeys) {
        let mut read = |name: &str| -> Option<Vec<u8>> {
            match self.reader.as_ref().map(|r| r(name)) {
                Some(Ok(p)) => map_sized(Some(p)),
                Some(Err(e)) => {
                    self.log.push(format!("map picture {name}: {e}; drawn black"));
                    None
                }
                None => None,
            }
        };
        let full = read(&keys.full_map);
        let blank = if keys.use_fog { read(&keys.blank_map) } else { None };
        let mut picture = match (&full, keys.use_fog) {
            (Some(f), false) => f.clone(),
            _ => {
                let (fog, _) = fog_from_runs(&keys.runs);
                let mut p = blank.unwrap_or_else(|| vec![0; MAP_SIZE * MAP_SIZE]);
                if let Some(f) = &full {
                    for (k, out) in fog.iter().enumerate() {
                        if *out {
                            p[k] = f[k];
                        }
                    }
                }
                p
            }
        };
        let own: Vec<MapIcon> = keys.icons.iter().filter(|i| i.picture.eq_ignore_ascii_case(&keys.full_map)).cloned().collect();
        let here: Vec<MapIcon> = self.icons.iter().filter(|i| i.picture.eq_ignore_ascii_case(&keys.full_map)).cloned().collect();
        for i in own.iter().chain(&here) {
            if let Some(pic) = self.icon_pic(&i.file) {
                stamp_on(&mut picture, &pic, i.x, i.y);
            }
        }
        let pal = |k: Option<usize>| k.and_then(|k| self.palettes.get(k).copied().flatten());
        let colours = match (pal(Some(index)), pal(self.index)) {
            (Some(from), Some(to)) => {
                let table = match_palette(&from, &to);
                picture.iter_mut().for_each(|v| *v = table[*v as usize]);
                let changed = (0..256).filter(|&k| table[k] as usize != k).count();
                format!("colours matched to this world's palette ({changed} of 256 entries moved)")
            }
            _ => "a palette is missing: colours as they are".to_string(),
        };
        self.log.push(format!(
            "shows map index {index} ({file}, {source}): {}, its {} icons and {} of this world's for {}; {colours} (rule 34)",
            if keys.use_fog { "its fog" } else { "its full picture" },
            own.len(),
            here.len(),
            keys.full_map
        ));
        self.other = Some(OtherMap { index, file: file.to_string(), source: source.to_string(), keys, picture });
        self.version += 1;
    }

    /// This world's map shown again (rules 9, 32, 34).
    pub fn show_own(&mut self) {
        if self.other.take().is_some() {
            self.version += 1;
        }
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
        stamp_on(&mut self.picture, &pic, icon.x, icon.y);
    }

    /// Rule 9: the icons of this world's picture drawn again.
    pub fn draw_icons(&mut self) {
        let shown = self.keys.full_map.clone();
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
            // Rule 9: another world's map shown snaps back.
            if let Some(o) = self.other.as_ref().map(|o| o.index) {
                self.show_own();
                self.log.push(format!("Connor's map pixel changed while map index {o} was shown: back to this world's map (rules 9, 34)"));
            }
            step.revealed = self.reveal(p.0, p.1);
            self.draw_icons();
            self.follow = true;
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

    /// Rule 25, at each draw of the big map: while marked, up to 6 steps
    /// toward bringing the arrow inside; inside clears the mark. Returns
    /// the steps made.
    pub fn follow_step(&mut self) -> usize {
        let Some(p) = self.last_pixel.filter(|_| self.follow) else { return 0 };
        let mut steps = 0;
        while steps < FOLLOW_STEPS {
            if !self.view.follow_once(p) {
                self.follow = false;
                break;
            }
            steps += 1;
        }
        steps
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
        let own = icon.picture.eq_ignore_ascii_case(&self.keys.full_map);
        self.icons.push(icon.clone());
        if own {
            self.stamp(&icon);
            self.version += 1;
        } else if shown_now {
            if let (Some(pic), Some(o)) = (self.icon_pic(&icon.file), self.other.as_mut()) {
                // Drawn in the shown map's colours like the rest of it.
                stamp_on(&mut o.picture, &pic, icon.x, icon.y);
                self.version += 1;
            }
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
        if let Some(o) = self.other.take() {
            // Another world's map shown: made again without the icon.
            self.show_other(o.index, &o.file, &o.source, o.keys);
            return Some(icon);
        }
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

    /// Rule 32: the page arrows (`dir` −1 previous, +1 next) step through
    /// the 16 map indexes from the one shown, wrapping, to the first marked
    /// visited on Connor (this world's counts); none in 16 steps: no
    /// change. Another world's map comes from [`WorldState::other_maps`].
    pub fn map_page(&mut self, dir: i32, how: &str) {
        let Some(map) = self.map.as_mut() else {
            self.map_log.push(format!("{how}: no map; nothing"));
            return;
        };
        let Some(from) = map.shown_index() else {
            self.map_log.push(format!("{how}: this map has no index; nothing (rule 3)"));
            return;
        };
        let marks = &self.map_marks;
        let target = (1..=MAP_COUNT as i32).map(|k| (from as i32 + dir * k).rem_euclid(MAP_COUNT as i32) as usize).find(|i| marks.visited[*i]);
        let line = match target {
            None => format!("{how}: page {dir:+} from index {from}: no visited map in 16 steps; nothing (rule 32)"),
            Some(i) if Some(i) == map.index => {
                map.show_own();
                format!("{how}: page {dir:+} from index {from}: index {i}, this world's map (rule 32)")
            }
            Some(i) => match self.other_maps.get(&i) {
                Some((file, keys, source)) => {
                    map.show_other(i, file, source, keys.clone());
                    format!("{how}: page {dir:+} from index {from}: index {i} ({file}, name #{}; rule 32)", keys.map_name.unwrap_or(0))
                }
                None => format!("{how}: page {dir:+} from index {from}: index {i} is visited but no map for it is known; nothing"),
            },
        };
        let mut lines = std::mem::take(&mut map.log);
        lines.push(line);
        self.map_log.extend(lines);
        self.map_go_refresh(how);
    }

    /// Rule 38: the "Go" button lit only when another world's map is shown,
    /// Connor is on the pad and that world's teleporter is earned (its map's
    /// `earnedTeleport`, or for Barren the Sage Gnome's mark). Worked out
    /// again whenever the shown map or the pad changes ("Do not reproduce"
    /// 1: also on a snap-back).
    pub fn map_go_refresh(&mut self, why: &str) {
        let sage = self.map_marks.sage_gnome;
        let Some(map) = self.map.as_mut() else { return };
        let lit = map.on_pad && map.other.as_ref().is_some_and(|o| o.keys.earned_teleport || (o.index == BARREN && sage));
        if lit != map.go_lit {
            map.go_lit = lit;
            let shown = map.shown_index();
            self.map_log.push(format!("{why}: the Go button {} (map index {shown:?} shown, on the pad {}; rule 38)", if lit { "lights" } else { "goes out" }, map.on_pad));
        }
    }

    /// `onTeleport`, `offTeleport` (rule 36), `earnTeleport [file]` (rule
    /// 37), `teleportersEarned` (rule 37: the count in `$Console::Return`).
    pub(crate) fn teleport_command(&mut self, words: &[String], at: &str) -> String {
        let all = words.join(" ");
        let marks = &mut self.map_marks;
        let (line, result) = match words[0].as_str() {
            "teleportersEarned" => {
                let n = marks.teleport_earned.iter().filter(|v| **v).count();
                (format!("{at}: {all}: {n} worlds with the teleporter earned (rule 37)"), n.to_string())
            }
            "earnTeleport" if words.len() > 1 => {
                // Rule 37: the argument's text is not looked at.
                marks.sage_gnome = true;
                marks.visited[BARREN] = true;
                marks.teleport_earned[BARREN] = true;
                (format!("{at}: {all}: Barren (index {BARREN}) marked visited and its teleporter earned, SageGnomeEarnedTeleport set (rule 37)"), "0".into())
            }
            "earnTeleport" => match self.map.as_mut() {
                Some(map) => {
                    map.earned_teleport = true;
                    let mark = map.index.filter(|i| *i < TELEPORT_MARKS);
                    if let Some(i) = mark {
                        marks.teleport_earned[i] = true;
                    }
                    (format!("{at}: {all}: {} earnedTeleport=1{} (rule 37)", map.file, mark.map_or(" (index 8 or more: no mark on Connor)".to_string(), |i| format!(", map index {i} marked on Connor"))), "0".into())
                }
                None => (format!("{at}: {all}: KQMap not loaded; nothing"), "1".into()),
            },
            on_off => match self.map.as_mut() {
                Some(map) => {
                    map.on_pad = on_off == "onTeleport";
                    (format!("{at}: {all}: Connor {} the pad (rule 36)", if map.on_pad { "is on" } else { "leaves" }), "0".into())
                }
                None => (format!("{at}: {all}: KQMap not loaded; nothing"), "1".into()),
            },
        };
        self.map_note(line);
        self.map_go_refresh(at);
        result
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
        self.map_go_refresh(&format!("update {updates}"));
    }
}

impl crate::active::Active {
    /// Rules 38, 39: a click on the "Go" button. Unlit: the teleporting
    /// help (popup 151; help popups are not built: logged). Lit: the
    /// current world's `TeleportScript` runs with `$1` = the shown world's
    /// `TeleportPortalKQ`.
    pub fn map_go(&mut self, how: &str) {
        let w = &mut self.world;
        let Some(map) = w.map.as_ref() else { return };
        if !map.go_lit {
            w.map_log.push(format!("{how}: the Go button is not lit: help popup {GO_HELP} (\"TELEPORTING: ...\"; rule 38; help popups are not built: logged only)"));
            return;
        }
        let script = map.keys.teleport_script.clone();
        let portal = map.other.as_ref().and_then(|o| o.keys.teleport_portal.clone());
        match (script, portal) {
            (Some(script), Some(portal)) => {
                w.map_log.push(format!("{how}: the Go button: runs {script} with $1 = {portal} (rule 39)"));
                let found = self.scripts.call(&script, &[portal], &mut self.world);
                if !found {
                    self.world.map_log.push(format!("{how}: {script} not found"));
                }
            }
            (s, p) => w.map_log.push(format!("{how}: the Go button: no TeleportScript ({s:?}) or no TeleportPortalKQ ({p:?}); nothing")),
        }
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

    fn pics_pair() -> (Option<IndexPic>, Option<IndexPic>) {
        pics()
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
    fn pan_keeps_the_centre_on_the_picture() {
        let mut v = START_VIEW;
        v.pan(10.0, -5.0);
        assert_eq!(v.as_array(), [-142.83, -157.11, 155.44, 141.16]);
        // Far right: the left edge stops at the centre.
        v.pan(1000.0, 0.0);
        assert_eq!((v.left, v.right), (0.0, 298.27));
        v.pan(-5000.0, 5000.0);
        assert!((v.right - 0.0).abs() < 1e-9 && (v.top - 0.0).abs() < 1e-9);
    }

    #[test]
    fn zoom_doubles_per_64_and_stops_at_the_limits() {
        let mut v = View { left: -100.0, top: -50.0, right: 100.0, bottom: 150.0 };
        v.zoom(64.0);
        assert_eq!(v.as_array(), [-200.0, -100.0, 200.0, 300.0], "about the area's centre");
        v.zoom(-128.0);
        assert!((v.width() - 115.0).abs() < 1e-9, "x 0.25 of 460 at most out");
        v.zoom(10000.0);
        assert!((v.width() - 3680.0).abs() < 1e-9);
    }

    #[test]
    fn follow_brings_the_arrow_inside_the_margins() {
        // A view at x 1 (460 layout px for 460 map pixels): pixel (0, 230)
        // puts the arrow's top-left at the picture's left - 5.
        let mut m = {
            let (b, f) = pics();
            MapState::new("davmap.kq", keys(), b, f, None, None)
        };
        m.view = View { left: 0.0, top: -230.0, right: 460.0, bottom: 230.0 };
        m.last_pixel = Some((0, 230));
        assert_eq!(m.follow_step(), 0, "not marked: stays put");
        m.follow = true;
        // Top-left x = -5 from the centre = 143 in the area: inside.
        assert_eq!(m.follow_step(), 0);
        assert!(!m.follow);
        // Far right: x = 455 - 5 + 148 = 598 > 276: 33 px per step.
        m.last_pixel = Some((455, 230));
        m.follow = true;
        assert_eq!(m.follow_step(), 6, "at most 6 steps a draw");
        assert!(m.follow);
        let x = |m: &MapState| m.view.arrow_at(m.last_pixel.unwrap()).0 + 148.0;
        // Steps of 33, 29, 27, 24, 21, 19 (each from the new distance).
        assert!((x(&m) - 445.0).abs() < 1e-9, "{}", x(&m));
        while m.follow_step() > 0 {}
        assert!(x(&m) <= 276.0 && x(&m) > 266.0 && !m.follow);
    }

    #[test]
    fn palettes_match_to_the_nearest_colour() {
        let mut from = [[0u8; 3]; 256];
        let mut to = [[0u8; 3]; 256];
        for k in 0..256 {
            to[k] = [k as u8, 0, 0];
        }
        from[1] = [10, 0, 0];
        from[2] = [255, 255, 255];
        from[3] = [100, 3, 0];
        let t = match_palette(&from, &to);
        assert_eq!((t[0], t[1], t[2], t[3]), (0, 10, 255, 100));
        // A tie: the first.
        let mut to2 = [[50u8, 50, 50]; 256];
        to2[7] = [0, 0, 0];
        to2[9] = [0, 0, 0];
        assert_eq!(match_palette(&from, &to2)[0], 7);
    }

    #[test]
    fn paging_to_visited_maps_and_back() {
        use crate::active::Active;
        let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        let pics = |v: u8| IndexPic { width: MAP_SIZE, height: MAP_SIZE, pixels: vec![v; MAP_SIZE * MAP_SIZE] };
        let reader: IconReader = std::sync::Arc::new(move |name: &str| Ok(if name.contains('0') { pics(5) } else { pics(6) }));
        let (b, f) = pics_pair();
        let mut m = MapState::new("Davmap.kq", keys(), b, f, None, Some(reader));
        let mut pal_a = [[0u8; 3]; 256];
        let mut pal_b = [[0u8; 3]; 256];
        pal_a[6] = [200, 0, 0];
        pal_b[9] = [199, 0, 0];
        pal_a[5] = [0, 0, 90];
        pal_b[4] = [0, 0, 90];
        m.palettes = vec![None; MAP_COUNT];
        m.palettes[0] = Some(pal_b);
        m.palettes[3] = Some(pal_a);
        a.world.map = Some(m);
        a.world.map.as_mut().unwrap().set_active(true, "t");
        a.world.map_marks.visited[0] = true;
        a.world.map_page(1, "t");
        assert!(a.world.map.as_ref().unwrap().other.is_none(), "only this world visited: stays");
        // Gnome (index 3) kept, revealed in its first row only.
        let gnome = MapKeys { name: "gnomMap".into(), map_name: Some(41), full_map: "gnomap.pbm".into(), blank_map: "gnomap0.pbm".into(), runs: vec![0, 460], ..keys() };
        a.world.other_maps.insert(3, ("gnomMap.kq".into(), gnome, "kept".into()));
        a.world.map_marks.visited[3] = true;
        a.world.map.as_mut().unwrap().add_icon("x.bmp", 0, 100, Some("gnomap.pbm"));
        a.world.map_page(-1, "t");
        let m = a.world.map.as_ref().unwrap();
        assert_eq!(m.shown_index(), Some(3), "-1 from 0 wraps to 15 … 3");
        // Row 0 revealed: full (6 → 9 by colour); the rest blank (5 → 4);
        // this world's icon for gnomap.pbm drawn (6 → 9).
        let p = m.shown_picture();
        assert_eq!((p[0], p[MAP_SIZE], p[100 * MAP_SIZE]), (9, 4, 9));
        a.world.map_page(1, "t");
        assert_eq!(a.world.map.as_ref().unwrap().shown_index(), Some(0), "+1 from 3 wraps round to 0");
        a.world.map_page(1, "t");
        assert_eq!(a.world.map.as_ref().unwrap().shown_index(), Some(3));
        // Connor's pixel changes: snaps back.
        a.world.map_step([200000.0, 250000.0, 0.0], 0.0);
        assert_eq!(a.world.map.as_ref().unwrap().shown_index(), Some(0));
        assert!(a.world.map_log.iter().any(|l| l.contains("back to this world's map")), "{:?}", a.world.map_log);
    }

    #[test]
    fn teleport_marks_and_the_go_button() {
        use crate::active::Active;
        let sources = vec![("davtele.cs".to_string(), b"set Ran $1\nKQMonster::setState Connor Portal Fred\n".to_vec())];
        let mut a = Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], sources);
        let (b, f) = pics();
        a.world.map = Some(MapState::new("Davmap.kq", MapKeys { teleport_script: Some("Davtele.cs".into()), ..keys() }, b, f, None, None));
        assert_eq!(a.test_command("teleportersEarned", "t").as_deref(), Some("0"));
        a.test_command("earnTeleport", "t");
        assert!(a.world.map.as_ref().unwrap().earned_teleport && a.world.map_marks.teleport_earned[0]);
        a.test_command("earnTeleport barrMap.kq", "t");
        assert!(a.world.map_marks.sage_gnome && a.world.map_marks.visited[BARREN]);
        assert_eq!(a.test_command("teleportersEarned", "t").as_deref(), Some("2"));
        // Barren shown (the Sage's mark is enough), not on the pad: unlit.
        a.world.map_marks.visited[0] = true;
        let barren = MapKeys { name: "barrMap".into(), map_name: Some(42), teleport_portal: Some("bartele.kq".into()), ..keys() };
        a.world.other_maps.insert(BARREN, ("barrMap.kq".into(), barren, "shipped".into()));
        a.world.map_page(1, "t");
        assert!(!a.world.map.as_ref().unwrap().go_lit);
        a.map_go("t");
        assert!(a.world.map_log.iter().any(|l| l.contains("help popup 151")));
        a.test_command("onTeleport", "t");
        assert!(a.world.map.as_ref().unwrap().go_lit, "on the pad: lit");
        a.map_go("t");
        assert_eq!(a.scripts.vars.get("Ran"), Some("bartele.kq"), "$1 = the shown world's portal");
        // M6 6e-6: the Portal state is asked for (no portal file loaded here).
        assert!(a.world.map_log.iter().any(|l| l.contains("setState Connor Portal")), "{:?}", a.world.map_log);
        // Back to this world's map: the button goes out (Do not reproduce 1).
        a.world.map_step([200000.0, 250000.0, 0.0], 0.0);
        a.world.map.as_mut().unwrap().set_active(true, "t");
        a.world.map_step([200000.0, 250000.0, 0.0], 0.0);
        assert_eq!(a.world.map.as_ref().unwrap().shown_index(), Some(0));
        assert!(!a.world.map.as_ref().unwrap().go_lit);
        a.test_command("offTeleport", "t");
        assert!(!a.world.map.as_ref().unwrap().on_pad);
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
        let reader: IconReader = std::sync::Arc::new(move |_| Ok(icon.clone()));
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
