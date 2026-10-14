//! Light and shadow maps of the outdoor ground (`lightmapName`,
//! `shadowmapName`, `SWlightmapName`, `SWshadowmapName` in `TERRAIN.KQ`).
//!
//! Rules from the behaviour note `docs/behaviour/terrain-lighting.md`
//! (session A); rule numbers below are that note's. See
//! docs/provenance/README.md.
//!
//! - A file is a u32 count, then count u16 values, all little-endian
//!   (rule 5). The count must be side x side, side = 128 x 2^lightScale + 1
//!   (rule 6); otherwise the file is not used.
//! - Value j x side + i is point (i, j), at x = i x spacing, y = j x spacing,
//!   spacing = 2^groundScale / 2^lightScale (rules 6, 7).
//! - A value is four 4-bit numbers: brightness (bits 12-15), red (8-11),
//!   green (4-7), blue (0-3) (rule 9).
//! - No file: every channel 15, full brightness (rule 4).

use crate::TerrainSettings;

/// Ground squares along each side of every world (rule 6).
pub const SQUARES_PER_SIDE: usize = 128;

/// The value of a point at full brightness: every channel 15 (rule 4).
pub const FULL: u16 = 0xFFFF;

/// Map points per side for a `lightScale` (rule 6).
pub fn side_for(light_scale: u32) -> usize {
    SQUARES_PER_SIDE * (1usize << light_scale) + 1
}

/// Which map of a pair is shown (rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `lightmapName` / `SWlightmapName`: `useDynamicLighting 0`.
    Light,
    /// `shadowmapName` / `SWshadowmapName`: `useDynamicLighting 1`.
    Shadow,
}

impl Kind {
    /// Rule 2: dynamic lighting on shows the shadow map, off the light map.
    pub fn for_dynamic_lighting(on: bool) -> Self {
        if on {
            Kind::Shadow
        } else {
            Kind::Light
        }
    }
}

/// The file name `TERRAIN.KQ` gives for one map; `None` when the name is
/// empty (rule 4: full brightness). `hardware` picks the
/// `lightmapName`/`shadowmapName` pair, otherwise the `SW` pair (rule 1).
pub fn file_name(settings: &TerrainSettings, hardware: bool, kind: Kind) -> Option<&str> {
    let name = match (hardware, kind) {
        (true, Kind::Light) => &settings.lightmap_name,
        (true, Kind::Shadow) => &settings.shadowmap_name,
        (false, Kind::Light) => &settings.sw_lightmap_name,
        (false, Kind::Shadow) => &settings.sw_shadowmap_name,
    };
    name.as_deref().filter(|n| !n.is_empty())
}

/// `useDynamicLighting <0|1>` from a console script such as `Options.cs`
/// (rule 2). The last such line wins; `None` if there is none (the game
/// then keeps its starting value 0).
pub fn use_dynamic_lighting(script: &[u8]) -> Option<bool> {
    let text: String = script.iter().map(|&b| b as char).collect();
    let mut found = None;
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let mut words = line.split_whitespace();
        if words.next().is_some_and(|w| w.eq_ignore_ascii_case("useDynamicLighting")) {
            if let Some(v) = words.next().and_then(|v| v.parse::<f64>().ok()) {
                found = Some(v != 0.0);
            }
        }
    }
    found
}

/// The four 4-bit numbers of a map value (rule 9).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Channels {
    pub brightness: u8,
    pub red: u8,
    pub green: u8,
    pub blue: u8,
}

impl Channels {
    pub fn of(value: u16) -> Self {
        Channels {
            brightness: (value >> 12) as u8 & 15,
            red: (value >> 8) as u8 & 15,
            green: (value >> 4) as u8 & 15,
            blue: value as u8 & 15,
        }
    }

    /// Red, green and blue equal.
    pub fn is_grey(&self) -> bool {
        self.red == self.green && self.green == self.blue
    }
}

/// One light or shadow map, ready for lookups in world coordinates.
#[derive(Debug, Clone)]
pub struct LightMap {
    /// Points per side (rule 6).
    pub side: usize,
    /// World units between neighbouring points (rule 6).
    pub spacing: f32,
    /// Row by row along +x, rows along +y from y = 0 (rule 7).
    values: Vec<u16>,
}

impl LightMap {
    /// A map at full brightness everywhere (rule 4), for worlds with no
    /// file or a file that cannot be used.
    pub fn full(settings: &TerrainSettings) -> Self {
        let side = side_for(settings.light_scale);
        LightMap { side, spacing: spacing(settings), values: vec![FULL; side * side] }
    }

    /// Reads a map file for this terrain (rules 5-7). A count that is not
    /// side x side, or a file shorter than its count, is an error; the
    /// caller then uses `full` (rule 4, edge cases).
    pub fn parse(bytes: &[u8], settings: &TerrainSettings) -> Result<Self, String> {
        let side = side_for(settings.light_scale);
        let head: [u8; 4] = bytes.get(..4).and_then(|b| b.try_into().ok()).ok_or("file shorter than its 4-byte count")?;
        let count = u32::from_le_bytes(head) as usize;
        if count != side * side {
            return Err(format!(
                "count {count} is not side x side = {} (side {side} for lightScale {})",
                side * side,
                settings.light_scale
            ));
        }
        let body = &bytes[4..];
        if body.len() < count * 2 {
            return Err(format!("count {count} needs {} bytes of values, the file has {}", count * 2, body.len()));
        }
        let values = body[..count * 2].as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c)).collect();
        Ok(LightMap { side, spacing: spacing(settings), values })
    }

    /// The value of point (i, j), i along +x, j along +y.
    pub fn get(&self, i: usize, j: usize) -> Option<u16> {
        (i < self.side && j < self.side).then(|| self.values[j * self.side + i])
    }

    pub fn values(&self) -> &[u16] {
        &self.values
    }

    /// Red, green and blue at world point (x, y), each 0..15, blended
    /// bilinearly from the four surrounding points (rule 11).
    ///
    /// STAND-IN: the note assumes bilinear filtering; the exact filtering
    /// of the original renderer was not traced (open question 2).
    /// Outside the map the nearest edge point is used.
    pub fn colour_at(&self, x: f32, y: f32) -> [f32; 3] {
        let last = (self.side - 1) as f32;
        let fx = (x / self.spacing).clamp(0.0, last);
        let fy = (y / self.spacing).clamp(0.0, last);
        let (i0, j0) = (fx.floor() as usize, fy.floor() as usize);
        let (i1, j1) = ((i0 + 1).min(self.side - 1), (j0 + 1).min(self.side - 1));
        let (tx, ty) = (fx - i0 as f32, fy - j0 as f32);
        let rgb = |i, j| {
            let c = Channels::of(self.values[j * self.side + i]);
            [c.red as f32, c.green as f32, c.blue as f32]
        };
        let (a, b, c, d) = (rgb(i0, j0), rgb(i1, j0), rgb(i0, j1), rgb(i1, j1));
        std::array::from_fn(|k| {
            let bottom = a[k] + (b[k] - a[k]) * tx;
            let top = c[k] + (d[k] - c[k]) * tx;
            bottom + (top - bottom) * ty
        })
    }

    /// How many points have each value 0..15 in each channel:
    /// `[brightness, red, green, blue][value]`.
    pub fn histograms(&self) -> [[usize; 16]; 4] {
        let mut h = [[0usize; 16]; 4];
        for &v in &self.values {
            let c = Channels::of(v);
            for (k, n) in [c.brightness, c.red, c.green, c.blue].into_iter().enumerate() {
                h[k][n as usize] += 1;
            }
        }
        h
    }
}

/// World units between map points (rule 6).
fn spacing(settings: &TerrainSettings) -> f32 {
    settings.square_size() / (1u32 << settings.light_scale) as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings(light_scale: u32, ground_scale: u32) -> TerrainSettings {
        TerrainSettings {
            ground_scale,
            height_scale: 1.0,
            height_image: String::new(),
            floor_image: None,
            material_image: String::new(),
            material_list: String::new(),
            material_direction: None,
            light_scale,
            lightmap_name: Some("l.bin".into()),
            shadowmap_name: Some("s.bin".into()),
            sw_lightmap_name: Some(String::new()),
            sw_shadowmap_name: None,
            all: Vec::new(),
        }
    }

    fn file(values: &[u16], count: u32) -> Vec<u8> {
        let mut b = count.to_le_bytes().to_vec();
        values.iter().for_each(|v| b.extend(v.to_le_bytes()));
        b
    }

    #[test]
    fn side_and_spacing() {
        assert_eq!(side_for(0), 129);
        assert_eq!(side_for(1), 257);
        assert_eq!(side_for(2), 513);
        // Daventry: 513 x 513 points, 1,024 apart; file 4 + 2 x 263,169 bytes.
        let m = LightMap::full(&settings(2, 12));
        assert_eq!((m.side, m.spacing), (513, 1024.0));
        assert_eq!(4 + 2 * m.values().len(), 526_342);
        // Dead City: groundScale 11, half the spacing.
        assert_eq!(LightMap::full(&settings(2, 11)).spacing, 512.0);
        assert_eq!(LightMap::full(&settings(0, 12)).spacing, 4096.0);
    }

    #[test]
    fn order_and_channels() {
        let s = settings(0, 12);
        let n = 129 * 129;
        let values: Vec<u16> = (0..n as u32).map(|k| (k % 65_536) as u16).collect();
        let m = LightMap::parse(&file(&values, n as u32), &s).unwrap();
        // Point (i, j) is value j x side + i.
        assert_eq!(m.get(5, 0), Some(5));
        assert_eq!(m.get(0, 1), Some(129));
        assert_eq!(m.get(3, 2), Some(2 * 129 + 3));
        assert_eq!(m.get(129, 0), None);
        let c = Channels::of(0x7468);
        assert_eq!((c.brightness, c.red, c.green, c.blue), (7, 4, 6, 8));
        assert!(!c.is_grey());
        assert!(Channels::of(0x6777).is_grey());
    }

    #[test]
    fn wrong_count_or_short_file_is_refused() {
        let s = settings(0, 12);
        assert!(LightMap::parse(&file(&[1, 2, 3], 3), &s).is_err());
        // Right count, values missing.
        assert!(LightMap::parse(&file(&[1, 2, 3], 129 * 129), &s).is_err());
        assert!(LightMap::parse(&[1, 0], &s).is_err());
    }

    #[test]
    fn bilinear_blend() {
        let s = settings(0, 12);
        let mut values = vec![FULL; 129 * 129];
        // Grey 4 at (0, 0), 8 at (1, 0), 12 at (0, 1), 0 at (1, 1).
        values[0] = 0x4444;
        values[1] = 0x8888;
        values[129] = 0xCCCC;
        values[130] = 0x0000;
        let m = LightMap::parse(&file(&values, 129 * 129), &s).unwrap();
        assert_eq!(m.colour_at(0.0, 0.0), [4.0; 3]);
        assert_eq!(m.colour_at(4096.0, 0.0), [8.0; 3]);
        // Halfway along x on the bottom edge.
        assert_eq!(m.colour_at(2048.0, 0.0), [6.0; 3]);
        // Middle of the cell: average of the four.
        assert_eq!(m.colour_at(2048.0, 2048.0), [6.0; 3]);
        // A quarter along x, three quarters along y:
        // bottom 4 + 4 x 0.25 = 5, top 12 - 12 x 0.25 = 9, 5 + 4 x 0.75 = 8.
        assert_eq!(m.colour_at(1024.0, 3072.0), [8.0; 3]);
        // Outside: nearest edge point.
        assert_eq!(m.colour_at(-500.0, -500.0), [4.0; 3]);
        assert_eq!(m.colour_at(1e9, 1e9), [15.0; 3]);
    }

    #[test]
    fn which_file() {
        let s = settings(2, 12);
        assert_eq!(Kind::for_dynamic_lighting(true), Kind::Shadow);
        assert_eq!(Kind::for_dynamic_lighting(false), Kind::Light);
        assert_eq!(file_name(&s, true, Kind::Shadow), Some("s.bin"));
        assert_eq!(file_name(&s, true, Kind::Light), Some("l.bin"));
        // Empty or missing name: no file (full brightness).
        assert_eq!(file_name(&s, false, Kind::Light), None);
        assert_eq!(file_name(&s, false, Kind::Shadow), None);
    }

    #[test]
    fn options_line() {
        assert_eq!(use_dynamic_lighting(b"setBrightness 50\r\nuseDynamicLighting 1\r\n"), Some(true));
        assert_eq!(use_dynamic_lighting(b"useDynamicLighting 1\nuseDynamicLighting 0\n"), Some(false));
        assert_eq!(use_dynamic_lighting(b"// useDynamicLighting 1\n"), None);
        assert_eq!(use_dynamic_lighting(b"setBrightness 50\n"), None);
    }
}
