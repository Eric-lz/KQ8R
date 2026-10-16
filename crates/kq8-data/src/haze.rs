//! Haze (distance fog) on models, the ground and walls, and the
//! background colour.
//!
//! Rules from the behaviour note `docs/behaviour/object-lighting.md` rules
//! 19-23, 27-33 (session A); rule numbers below are that note's. The viewer's
//! shader (`object_light.wesl`) does the same per object on the graphics
//! card; this copy is for tests and the log. See docs/provenance/README.md.

/// Used when there is no terrain (rule 19).
pub const NO_TERRAIN_HAZE_DISTANCE: f32 = 20_000.0;
pub const NO_TERRAIN_VISIBLE_DISTANCE: f32 = 25_500.0;

/// Below this the haze is fixed at 0.01 (rule 19).
pub const NEAR_HAZE: f32 = 0.01;

/// A world's haze.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Haze {
    /// `hazeDistance`: haze starts here.
    pub start: f32,
    /// Haze is 1 here (rule 19).
    pub end: f32,
    /// `visibleDistance`: the ground's end (rules 27, 28) and the draw
    /// distance from Connor (rule 36).
    pub visible: f32,
    /// Screen colour 0..1 per channel: the palette colour nearest to
    /// `resetHazeColor` (rule 23, `palette_haze_colour`).
    pub colour: [f32; 3],
    /// Screen colour 0..1 behind everything: palette entry `newSky n`
    /// (rule 31). Black until set.
    pub background: [f32; 3],
}

impl Haze {
    /// end = 0.85 x visibleDistance, unless that is not beyond
    /// hazeDistance, then visibleDistance (rule 19).
    pub fn new(haze_distance: f32, visible_distance: f32, colour: [f32; 3]) -> Self {
        let near_end = 0.85 * visible_distance;
        let end = if near_end > haze_distance { near_end } else { visible_distance };
        Haze { start: haze_distance, end, colour, visible: visible_distance, background: [0.0; 3] }
    }

    /// The haze of an object at `distance` from the camera, before the
    /// material and object adjustments: 0.01 closer than the start, capped
    /// at 1 (rule 19).
    pub fn amount(&self, distance: f32) -> f32 {
        if distance < self.start {
            return NEAR_HAZE;
        }
        let span = self.end - self.start;
        if span <= 0.0 {
            return 1.0;
        }
        ((distance - self.start) / span).min(1.0)
    }
}

/// The ground's haze never reaches 1 (rule 27).
pub const GROUND_MAX_HAZE: f32 = 0.99;

impl Haze {
    /// Where the ground's haze starts (rule 27): `hazeDistance`, unless
    /// `visibleDistance` is not beyond it, then 0.6 x `visibleDistance`.
    pub fn ground_start(&self) -> f32 {
        if self.visible > self.start { self.start } else { 0.6 * self.visible }
    }

    /// The haze of one ground corner at `distance` from the camera
    /// (rule 27): 0.01 nearer than the start, then 0.01 + (d - start) /
    /// (1.02 x (visible - start)), which is 0.99 at `visibleDistance`;
    /// 0.99 beyond. The ground uses `visibleDistance` itself as its end,
    /// not the objects' 0.85 x.
    pub fn ground_amount(&self, distance: f32) -> f32 {
        let start = self.ground_start();
        if distance < start {
            return NEAR_HAZE;
        }
        let span = 1.02 * (self.visible - start);
        if span <= 0.0 {
            return GROUND_MAX_HAZE;
        }
        (NEAR_HAZE + (distance - start) / span).min(GROUND_MAX_HAZE)
    }

    /// The ground ends at `visibleDistance` from the camera (rule 28).
    pub fn ground_drawn(&self, distance: f32) -> bool {
        distance <= self.visible
    }
}

/// How one material of one object is drawn at a given haze (rules 20-23).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Hazed {
    /// Not drawn (rule 21).
    Hidden,
    /// A solid object blended towards the haze colour by this amount 0..1.
    Blend(f32),
    /// A see-through object: its opacity multiplied by this (no haze
    /// colour).
    Fade(f32),
}

/// Rules 20-23 for one material of one object: `halved` = material flag
/// `0x10000`, `solid` = the object's `b_hazeSolid`, `multiplier` = its
/// `hazeMultiplier`, `see_through` = `translucency` below 256.
pub fn hazed(amount: f32, halved: bool, solid: bool, multiplier: f32, see_through: bool) -> Hazed {
    let mut h = amount;
    if halved {
        h *= 0.5;
    }
    if h >= 1.0 && !solid {
        return Hazed::Hidden;
    }
    // STAND-IN: capped at 1 after the multiplier (a blend can't go past
    // the haze colour); the note doesn't say.
    let h = (h * multiplier).clamp(0.0, 1.0);
    if see_through {
        Hazed::Fade(if h < 0.5 { 1.0 } else { (1.2 - h).max(0.0) })
    } else {
        Hazed::Blend(h)
    }
}

/// The script's lines without comments: lines starting with `#` and text
/// after `//` are left out.
fn script_lines(script: &[u8]) -> Vec<String> {
    let text: String = script.iter().map(|&b| b as char).collect();
    text.lines()
        .map(|l| l.split("//").next().unwrap_or("").trim().to_string())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .collect()
}

/// `resetHazeColor R G B` from a console script such as `world.cs`
/// (object-lighting.md rules 18, 23): the values as written, on the 0-255
/// scale. The last such line wins; `None` if there is none.
pub fn reset_haze_color(script: &[u8]) -> Option<[f32; 3]> {
    let mut found = None;
    for line in script_lines(script) {
        let mut words = line.split_whitespace();
        if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("resetHazeColor")) {
            continue;
        }
        let v: Vec<f32> = words.take(3).filter_map(|w| w.parse::<f32>().ok()).collect();
        if let [r, g, b] = v[..] {
            found = Some([r, g, b].map(|c| c.clamp(0.0, 255.0)));
        }
    }
    found
}

/// Palette entries the haze colour may be snapped to (rule 23). STAND-IN
/// (the note's open question 8): every entry from 10 to 244 counts as
/// usable.
pub const HAZE_ENTRIES: std::ops::RangeInclusive<usize> = 10..=244;

/// The palette entry the game fogs with (rule 23): the one among
/// `HAZE_ENTRIES` nearest to `want` (0-255 per channel), by the smallest
/// sum of squared differences; the first one on a tie (STAND-IN, the note
/// doesn't say).
pub fn palette_haze_colour(colours: &[[u8; 3]; 256], want: [f32; 3]) -> usize {
    let mut best = (*HAZE_ENTRIES.start(), f32::MAX);
    for i in HAZE_ENTRIES {
        let d: f32 = (0..3).map(|k| (colours[i][k] as f32 - want[k]).powi(2)).sum();
        if d < best.1 {
            best = (i, d);
        }
    }
    best.0
}

/// The palette entry `newSky` fills the background with (rule 31):
/// `newSky <n>`, or `newSky $<variable>` with the variable's value from a
/// `set <variable> <n>` line (e.g. `set KQWorld::background 17`). The last
/// `newSky` line wins; `None` if there is none or it can't be read.
pub fn new_sky(script: &[u8]) -> Option<usize> {
    let lines = script_lines(script);
    let variable = |name: &str| {
        lines.iter().find_map(|l| {
            let mut w = l.split_whitespace();
            (w.next()?.eq_ignore_ascii_case("set") && w.next()?.eq_ignore_ascii_case(name)).then(|| w.next()).flatten()
        })
    };
    let mut found = None;
    for line in &lines {
        let mut words = line.split_whitespace();
        if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("newSky")) {
            continue;
        }
        let Some(arg) = words.next() else { continue };
        let value = match arg.strip_prefix('$') {
            Some(name) => variable(name),
            None => Some(arg),
        };
        if let Some(n) = value.and_then(|v| v.parse::<usize>().ok()).filter(|&n| n < 256) {
            found = Some(n);
        }
    }
    found
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn end_rule() {
        // Daventry: 18,000 and 27,000 -> end 22,950; Dead City 9,000 and
        // 12,000 -> 10,200 (the note).
        assert_eq!(Haze::new(18_000.0, 27_000.0, [0.0; 3]).end, 22_950.0);
        assert_eq!(Haze::new(9_000.0, 12_000.0, [0.0; 3]).end, 10_200.0);
        // 0.85 x visible not beyond the start: visible itself.
        assert_eq!(Haze::new(20_000.0, 22_000.0, [0.0; 3]).end, 22_000.0);
        let none = Haze::new(NO_TERRAIN_HAZE_DISTANCE, NO_TERRAIN_VISIBLE_DISTANCE, [0.0; 3]);
        assert!((none.end - 21_675.0).abs() < 0.01);
    }

    #[test]
    fn amount() {
        let h = Haze::new(18_000.0, 27_000.0, [0.0; 3]);
        assert_eq!(h.amount(500.0), 0.01);
        assert_eq!(h.amount(17_999.0), 0.01);
        assert_eq!(h.amount(18_000.0), 0.0);
        assert!((h.amount(20_475.0) - 0.5).abs() < 1e-6);
        assert_eq!(h.amount(22_950.0), 1.0);
        assert_eq!(h.amount(40_000.0), 1.0);
    }

    #[test]
    fn ground() {
        // Daventry: 0.01 up to 18,000, 0.99 at 27,000 (not 22,950), never 1.
        let h = Haze::new(18_000.0, 27_000.0, [0.0; 3]);
        assert_eq!(h.ground_amount(17_999.0), 0.01);
        assert!((h.ground_amount(18_000.0) - 0.01).abs() < 1e-6);
        // 0.01 + 1 / 1.02 = 0.9904 at visibleDistance, capped to 0.99.
        assert_eq!(h.ground_amount(27_000.0), 0.99);
        assert!((h.ground_amount(26_900.0) - (0.01 + 8_900.0 / (1.02 * 9_000.0))).abs() < 1e-6);
        assert!((h.ground_amount(22_500.0) - (0.01 + 0.5 / 1.02)).abs() < 1e-6);
        assert_eq!(h.ground_amount(40_000.0), 0.99);
        assert!(h.ground_drawn(27_000.0) && !h.ground_drawn(27_001.0));
        // Dead City: 0.99 at 12,000.
        let d = Haze::new(9_000.0, 12_000.0, [0.0; 3]);
        assert!((d.ground_amount(10_500.0) - (0.01 + 0.5 / 1.02)).abs() < 1e-6);
        // visibleDistance not beyond hazeDistance: start at 0.6 x visible.
        let odd = Haze::new(20_000.0, 10_000.0, [0.0; 3]);
        assert_eq!(odd.ground_start(), 6_000.0);
        assert_eq!(odd.ground_amount(5_000.0), 0.01);
        assert!((odd.ground_amount(8_000.0) - (0.01 + 2_000.0 / (1.02 * 4_000.0))).abs() < 1e-6);
    }

    #[test]
    fn material_and_object_rules() {
        // At the end: gone, unless solid; a halved material stays at 0.5.
        assert_eq!(hazed(1.0, false, false, 1.0, false), Hazed::Hidden);
        assert_eq!(hazed(1.0, false, true, 1.0, false), Hazed::Blend(1.0));
        assert_eq!(hazed(1.0, true, false, 1.0, false), Hazed::Blend(0.5));
        // The multiplier comes after the test, and the blend stays <= 1.
        assert_eq!(hazed(0.4, false, false, 2.0, false), Hazed::Blend(0.8));
        assert_eq!(hazed(0.8, false, false, 2.0, false), Hazed::Blend(1.0));
        // See-through: no colour, fade 1 below 0.5, then 1.2 - haze.
        assert_eq!(hazed(0.3, false, false, 1.0, true), Hazed::Fade(1.0));
        match hazed(0.7, false, false, 1.0, true) {
            Hazed::Fade(f) => assert!((f - 0.5).abs() < 1e-6),
            other => panic!("{other:?}"),
        }
        assert_eq!(hazed(1.0, false, false, 1.0, true), Hazed::Hidden);
    }

    #[test]
    fn haze_colour_line() {
        let script = b"# the loadPalette and resetHazeColor calls must appear together\nloadPalette daventry.ppl\nresetHazeColor 4 8 12\n";
        assert_eq!(reset_haze_color(script), Some([4.0, 8.0, 12.0]));
        assert_eq!(reset_haze_color(b"// resetHazeColor 1 2 3\n"), None);
        assert_eq!(reset_haze_color(b"resetHazeColor 1 2\n"), None);
        assert_eq!(reset_haze_color(b"resetHazeColor 255 0 0\nresetHazeColor 0 255 0\n"), Some([0.0, 255.0, 0.0]));
    }

    #[test]
    fn nearest_palette_colour() {
        let mut colours = [[200u8, 200, 200]; 256];
        // Entries outside 10..=244 are never picked, even if exact.
        colours[3] = [4, 8, 12];
        colours[250] = [4, 8, 12];
        colours[40] = [0, 10, 10];
        colours[41] = [10, 10, 10];
        colours[42] = [0, 10, 10];
        // (4, 8, 12): entry 40 is 16 + 4 + 4 = 24 away, 41 is 36 + 4 + 4 =
        // 44; 42 ties with 40 and loses.
        assert_eq!(palette_haze_colour(&colours, [4.0, 8.0, 12.0]), 40);
        // Nothing near: the nearest anyway; ties go to the first.
        assert_eq!(palette_haze_colour(&[[0, 0, 0]; 256], [255.0; 3]), 10);
    }

    #[test]
    fn sky_line() {
        let script = b"\tset KQWorld::background 17\n\t# The background:\n\tnewSky $KQWorld::background\n";
        assert_eq!(new_sky(script), Some(17));
        assert_eq!(new_sky(b"newSky 229\n"), Some(229));
        assert_eq!(new_sky(b"# newSky 5\n"), None);
        assert_eq!(new_sky(b"newSky $nothing\n"), None);
    }
}
