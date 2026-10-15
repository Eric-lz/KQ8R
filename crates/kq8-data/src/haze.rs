//! Haze (distance fog) on models.
//!
//! Rules from the behaviour note `docs/behaviour/object-lighting.md` rules
//! 19-23 (session A); rule numbers below are that note's. The viewer's
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
    /// `visibleDistance`. STAND-IN (Q69, the user's check in Dead City):
    /// no object is drawn farther than this from the camera.
    pub visible: f32,
    /// `resetHazeColor`, 0..1 per channel (the file's 0..255 / 255; the
    /// 0-255 scale is the note's assumption).
    pub colour: [f32; 3],
}

impl Haze {
    /// end = 0.85 x visibleDistance, unless that is not beyond
    /// hazeDistance, then visibleDistance (rule 19).
    pub fn new(haze_distance: f32, visible_distance: f32, colour: [f32; 3]) -> Self {
        let near_end = 0.85 * visible_distance;
        let end = if near_end > haze_distance { near_end } else { visible_distance };
        Haze { start: haze_distance, end, colour, visible: visible_distance }
    }

    /// STAND-IN (Q69): an object farther than `visibleDistance` from the
    /// camera is not drawn, whatever its haze. The user saw Dead City's
    /// torch flames (see-through, material `0x10000`, so by rules 20-21
    /// never left out) vanish at about 11,000 in the original; Dead City's
    /// `visibleDistance` is 12,000.
    pub fn beyond_view(&self, distance: f32) -> bool {
        distance > self.visible
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

/// `resetHazeColor R G B` from a console script such as `world.cs`
/// (object-lighting.md rule 18). The last such line wins; `None` if there
/// is none. Lines starting with `#` and text after `//` are comments.
pub fn reset_haze_color(script: &[u8]) -> Option<[f32; 3]> {
    let text: String = script.iter().map(|&b| b as char).collect();
    let mut found = None;
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        if line.starts_with('#') {
            continue;
        }
        let mut words = line.split_whitespace();
        if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("resetHazeColor")) {
            continue;
        }
        let v: Vec<f32> = words.take(3).filter_map(|w| w.parse::<f32>().ok()).collect();
        if let [r, g, b] = v[..] {
            found = Some([r, g, b].map(|c| (c / 255.0).clamp(0.0, 1.0)));
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
        let dead = Haze::new(9_000.0, 12_000.0, [0.0; 3]);
        assert!(!dead.beyond_view(12_000.0) && dead.beyond_view(12_001.0));
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
        let c = reset_haze_color(script).unwrap();
        assert!((c[0] - 4.0 / 255.0).abs() < 1e-6 && (c[2] - 12.0 / 255.0).abs() < 1e-6);
        assert_eq!(reset_haze_color(b"// resetHazeColor 1 2 3\n"), None);
        assert_eq!(reset_haze_color(b"resetHazeColor 1 2\n"), None);
        assert_eq!(reset_haze_color(b"resetHazeColor 255 0 0\nresetHazeColor 0 255 0\n"), Some([0.0, 1.0, 0.0]));
    }
}
