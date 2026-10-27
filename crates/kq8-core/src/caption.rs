//! The caption as a picture: a line's text broken into rows in the game's
//! own font and drawn over the stone strip (M4 steps 4b-3 and 4c-1).
//!
//! Rules from `docs/behaviour/interface.md` rules 46–52 (A, 2026-10-18;
//! answers Q107 a), checked by the user in the original:
//!
//! - The caption box is the layout's `IDDLG_CAPTION`: font `console.pft`,
//!   background picture `stontran.pbm`, text 2 pixels down (rule 46).
//! - Rows are cut greedily at the **box's width**: widths are summed
//!   letter by letter (glyph width × the font's horizontal scale, plus the
//!   font's spacing after each letter that fits); a letter that would pass
//!   the width ends the row at the last space before it (that space is
//!   dropped; spaces that fit at a row's end are kept) (rule 47). A row
//!   with no space to break at is cut at the box's edge ("Edge cases",
//!   the note's "do instead").
//! - Height: one row per font height; the strip's top = its bottom − rows
//!   × font height − 4, so it grows upwards (rule 48). Each row is centred
//!   in the box's width, in its own slot, from the top down (rule 49).
//! - The stone covers the **whole box**, not the text (rule 50; the user's
//!   check: a short line's strip is as wide as a long line's).
//! - The box's bottom edge = the top of the highest bottom panel; its
//!   centre is half-way between the map window's right edge and the
//!   screen's right edge (rule 51).
//!
//! STAND-INS (Q108): the screen is 800 × 600 when `Options.cs` says
//! `set800mode 1` and the 640 × 480 layout is scaled with it (rule 47,
//! inferred); pictures are stretched with the layout (nearest pixel) and
//! tiled when smaller than their box; fonts are drawn at their own size.
//! The stone stays tiled: B-7 stretched it to the box by `interface.md`
//! rule 20a (for the caption only A's inference) and the user saw it
//! broken and checkered; the tiled stone matched the original (Q168).

use kq8_data::font::Font;
use kq8_data::palette::Rgb;

/// The layout's screen (every `.gui` rectangle is on it).
pub const LAYOUT: [u32; 2] = [640, 480];

/// Rule 48: the strip is this much taller than its rows.
pub const STRIP_PAD: u32 = 4;

/// The game's screen: 800 × 600 with `set800mode 1`, else the layout's.
pub fn screen(set800mode: bool) -> [u32; 2] {
    if set800mode {
        [800, 600]
    } else {
        LAYOUT
    }
}

/// How much bigger the screen is than the layout (1.25 at 800 × 600).
pub fn scale(screen: [u32; 2]) -> f32 {
    screen[0] as f32 / LAYOUT[0] as f32
}

fn code(c: char) -> u8 {
    u8::try_from(c as u32).unwrap_or(b'?')
}

/// A letter's own width (glyph width × horizontal scale; 0 without a
/// glyph), before the spacing that follows it.
fn glyph_width(font: &Font, c: char) -> f32 {
    font.glyph(code(c)).map_or(0.0, |g| g.width as f32 * font.h_scale)
}

/// Width of a row: every letter's width plus the spacing after it (rule
/// 47).
pub fn text_width(font: &Font, text: &str) -> f32 {
    text.chars().map(|c| glyph_width(font, c) + font.char_spacing as f32).sum()
}

/// Rule 47: breaks `text` into rows no wider than `width`.
pub fn wrap(font: &Font, text: &str, width: f32) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut rows = Vec::new();
    let (mut start, mut i, mut used, mut last_space) = (0usize, 0usize, 0f32, None::<usize>);
    while i < chars.len() {
        let w = glyph_width(font, chars[i]);
        if used + w > width && i > start {
            match last_space {
                // The row ends at the last space before this letter; the
                // space is dropped and the next row starts after it.
                Some(s) => {
                    rows.push(chars[start..s].iter().collect());
                    start = s + 1;
                }
                // No space: cut at the box's edge (our choice).
                None => {
                    rows.push(chars[start..i].iter().collect());
                    start = i;
                }
            }
            (i, used, last_space) = (start, 0.0, None);
            continue;
        }
        if chars[i] == ' ' {
            last_space = Some(i);
        }
        used += w + font.char_spacing as f32;
        i += 1;
    }
    if start < chars.len() {
        rows.push(chars[start..].iter().collect());
    }
    rows
}

/// What the caption box needs, in screen pixels.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box {
    /// The box's width: the layout's width × the screen's scale (400 at
    /// 800 × 600).
    pub width: u32,
    /// One row: the font's height (rule 48).
    pub row_height: u32,
    /// Text top below the top of its row slot (the layout's caption y).
    pub text_y: u32,
    /// Layout pixels per screen pixel for the strip picture.
    pub scale: f32,
}

impl Box {
    /// The strip's height for `rows` rows (rule 48).
    pub fn height(&self, rows: usize) -> u32 {
        rows as u32 * self.row_height + STRIP_PAD
    }
}

/// Where the box goes (rule 51), in screen pixels: (left, top).
/// `bottom`: the top of the highest bottom panel (the screen's height
/// with the bottom bar away); `map_right`: the map window's right edge
/// as it stands (−50/640 of the screen's width when it is away).
pub fn place(screen_width: f32, b: &Box, rows: usize, bottom: f32, map_right: f32) -> (f32, f32) {
    let centre = (map_right + screen_width) / 2.0;
    (centre - b.width as f32 / 2.0, bottom - b.height(rows) as f32)
}

/// The map window's right edge when it is away (`interface.md` rule 9:
/// 50/640 of the screen beyond the left edge).
pub fn map_away_right(screen_width: f32) -> f32 {
    -50.0 / 640.0 * screen_width
}

/// A picture: width, height, palette indices (0 see-through).
pub struct Strip<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

impl Strip<'_> {
    /// The picture's index at pixel (x, y) of a box it fills, tiled from
    /// the top-left (rule 20a), each of its pixels `step` pixels square.
    pub fn at(&self, x: u32, y: u32, step: u32) -> u8 {
        let step = step.max(1);
        let sx = (x / step) % self.width;
        let sy = (y / step) % self.height;
        self.pixels[(sy * self.width + sx) as usize]
    }
}

/// How many whole output pixels one stone pixel covers: the true scale
/// (`b.scale` × `out`) rounded, at least 1. **Ours** (2026-10-27, the
/// user's report of a checkered stone): `stontran.pbm` is a one-pixel
/// checkerboard of see-through pixels (2,048 of 4,096), which only reads
/// as an even half-see-through stone when every stone pixel is the same
/// whole number of screen pixels; stretched by an uneven amount (× 1.25,
/// then by the window) it turns into stripes and blocks.
pub fn stone_step(scale: f32, out: f32) -> u32 {
    ((scale * out).round() as u32).max(1)
}

/// The caption's picture: RGBA, see-through where neither the strip nor
/// a letter has ink. `out`: output pixels per screen pixel (the window's
/// pixels per pixel of the game's 800 × 600 screen); the picture is the
/// box's size × `out`. The letters are drawn at the box's size and
/// enlarged by `out` (nearest pixel); the stone is tiled in whole pixels
/// ([`stone_step`]).
pub fn picture(font: &Font, strip: Option<&Strip>, palette: &[Rgb; 256], rows: &[String], b: Box, out: f32) -> (u32, u32, Vec<u8>) {
    let (w, h) = (b.width, b.height(rows.len()));
    // The letters, at the box's size (palette indices, 0 = no ink).
    let mut ink = vec![0u8; (w * h) as usize];
    let mut put = |x: u32, y: u32, index: u8| {
        if index != 0 && x < w && y < h {
            ink[(y * w + x) as usize] = index;
        }
    };
    for (k, row) in rows.iter().enumerate() {
        let top = k as u32 * b.row_height + b.text_y;
        let mut x = ((w as f32 - text_width(font, row)) / 2.0).max(0.0);
        for c in row.chars() {
            if let Some(g) = font.glyph(code(c)) {
                if let Some(px) = font.glyph_pixels(g) {
                    for gy in 0..g.height as u32 {
                        for gx in 0..g.width as u32 {
                            put(x as u32 + gx, top + gy, px[(gy * g.width as u32 + gx) as usize]);
                        }
                    }
                }
            }
            x += glyph_width(font, c) + font.char_spacing as f32;
        }
    }
    let out = if out > 0.0 { out } else { 1.0 };
    let (ow, oh) = (((w as f32 * out).round() as u32).max(1), ((h as f32 * out).round() as u32).max(1));
    let step = stone_step(b.scale, out);
    let strip = strip.filter(|s| s.width > 0 && s.height > 0);
    let mut rgba = vec![0u8; (ow * oh * 4) as usize];
    for y in 0..oh {
        for x in 0..ow {
            let (bx, by) = (((x as f32 / out) as u32).min(w - 1), ((y as f32 / out) as u32).min(h - 1));
            let index = match ink[(by * w + bx) as usize] {
                0 => strip.map_or(0, |s| s.at(x, y, step)),
                i => i,
            };
            if index != 0 {
                let at = ((y * ow + x) * 4) as usize;
                let [r, g, bl] = palette[index as usize];
                rgba[at..at + 4].copy_from_slice(&[r, g, bl, 255]);
            }
        }
    }
    (ow, oh, rgba)
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use kq8_data::font::Glyph;

    /// A font where every letter is 10 wide and the space 5 wide (one
    /// 10x2 picture of index 7), 2 high.
    pub fn font() -> Font {
        let mut map = vec![-1i16; 224];
        for c in b'a'..=b'z' {
            map[(c - 32) as usize] = 1;
        }
        map[0] = 0;
        let bitmap = kq8_data::Bitmap::parse(&{
            let mut b = b"PBMP".to_vec();
            b.extend(0u32.to_le_bytes());
            b.extend(b"head");
            b.extend(20u32.to_le_bytes());
            for v in [2u32, 12, 2, 8, 0] {
                b.extend(v.to_le_bytes());
            }
            b.extend(b"data");
            b.extend(24u32.to_le_bytes());
            b.extend([7u8; 24]);
            b
        })
        .unwrap();
        Font {
            version: 2,
            font_flags: 0x41,
            text_flags: 0,
            char_height: 2,
            char_width: 10,
            text_color: 0xFF,
            back_color: 0,
            baseline: 2,
            h_scale: 1.0,
            v_scale: 1.0,
            char_spacing: 0,
            char_first: 32,
            char_map: map,
            glyphs: vec![
                Glyph { bitmap: 0, left: 0, top: 0, width: 5, height: 2, baseline_shift: 0 },
                Glyph { bitmap: 0, left: 0, top: 0, width: 10, height: 2, baseline_shift: 0 },
            ],
            bitmaps: vec![bitmap],
            declared_bitmaps: 1,
            array_chunks: vec![],
            palette: None,
            trailing: 0,
        }
    }

    #[test]
    fn rows_break_at_the_last_space_before_the_letter_that_does_not_fit() {
        let f = font();
        assert_eq!(text_width(&f, "ab cd"), 45.0);
        // "aaa bbb" = 65; the space after it still fits (70) and is kept;
        // "c" would pass 70, so the row ends at that last space (dropped).
        assert_eq!(wrap(&f, "aaa bbb ccc", 70.0), vec!["aaa bbb", "ccc"]);
        // A space that fits at the end of a row is kept.
        assert_eq!(wrap(&f, "aaa bbb  ccc", 75.0), vec!["aaa bbb ", "ccc"]);
        // Leading spaces count like any letter.
        assert_eq!(wrap(&f, " aa", 70.0), vec![" aa"]);
        // No space to break at: cut at the box's edge.
        assert_eq!(wrap(&f, "aaaaaaa b", 50.0), vec!["aaaaa", "aa b"]);
        assert!(wrap(&f, "", 70.0).is_empty());
    }

    #[test]
    fn the_box_sits_on_the_bottom_panel_left_of_the_middle_with_the_map_away() {
        let b = Box { width: 400, row_height: 15, text_y: 2, scale: 1.25 };
        assert_eq!(b.height(2), 34, "2 x 15 + 4");
        // 800 x 600, map away (right edge -62.5): centre 368.75.
        let (left, top) = place(800.0, &b, 2, 600.0, map_away_right(800.0));
        assert_eq!((left, top), (168.75, 566.0));
        // Map open (right edge 400 of 800): centre 600.
        assert_eq!(place(800.0, &b, 1, 520.0, 400.0), (400.0, 501.0));
    }

    #[test]
    fn the_stone_covers_the_whole_box_and_rows_are_centred() {
        let f = font();
        let mut pal = [[0u8; 3]; 256];
        pal[7] = [200, 150, 50];
        pal[3] = [80, 80, 80];
        let strip = [0u8, 3, 3, 3];
        let rows = vec!["ab".to_string(), "c".to_string()];
        let b = Box { width: 100, row_height: 15, text_y: 2, scale: 1.0 };
        let (w, h, rgba) = picture(&f, Some(&Strip { width: 2, height: 2, pixels: &strip }), &pal, &rows, b, 1.0);
        assert_eq!((w, h), (100, 34));
        let at = |x: u32, y: u32| &rgba[((y * w + x) * 4) as usize..((y * w + x) * 4 + 4) as usize];
        assert_eq!(at(0, 0), [0, 0, 0, 0], "strip index 0: see-through");
        assert_eq!(at(1, 0), [80, 80, 80, 255]);
        assert_eq!(at(99, 33), [80, 80, 80, 255], "the stone reaches the box's far corner");
        // "ab" is 20 wide, centred: x 40..60, rows 2..4.
        assert_eq!(at(40, 2), [200, 150, 50, 255]);
        assert_eq!(at(59, 3), [200, 150, 50, 255]);
        // "c" on the second row: x 45..55, its slot starts at 15: y 17..19.
        assert_eq!(at(45, 17), [200, 150, 50, 255]);
        // Stretched by 2: the 2x2 stone's see-through pixel is 2x2.
        let b2 = Box { scale: 2.0, ..b };
        let (_, _, rgba2) = picture(&f, Some(&Strip { width: 2, height: 2, pixels: &strip }), &pal, &[], b2, 1.0);
        assert_eq!(&rgba2[4..8], [0, 0, 0, 0]);
        assert_eq!(&rgba2[8..12], [80, 80, 80, 255]);
    }

    #[test]
    fn a_checkerboard_stone_stays_even_at_any_window_size() {
        // stontran.pbm is a one-pixel checkerboard (index 0 see-through).
        let f = font();
        let mut pal = [[0u8; 3]; 256];
        pal[3] = [80, 80, 80];
        let stone: Vec<u8> = (0..64u32 * 64).map(|i| if (i % 64 + i / 64) % 2 == 0 { 0 } else { 3 }).collect();
        let s = Strip { width: 64, height: 64, pixels: &stone };
        let b = Box { width: 400, row_height: 15, text_y: 2, scale: 1.25 };
        // 800x600, a 1280x720 window (x 1.2) and a 1920x1080 one (x 1.8).
        for (out, step) in [(1.0, 1), (1.2, 2), (1.8, 2)] {
            assert_eq!(stone_step(b.scale, out), step);
            let (w, h, rgba) = picture(&f, Some(&s), &pal, &[], b, out);
            assert_eq!(w, (400.0 * out).round() as u32);
            let ink = |x: u32, y: u32| rgba[((y * w + x) * 4 + 3) as usize] != 0;
            for y in 0..h.min(20) {
                for x in 0..w.min(200) {
                    assert_eq!(ink(x, y), (x / step + y / step) % 2 == 1, "out {out}: pixel ({x}, {y})");
                }
            }
        }
    }
}
