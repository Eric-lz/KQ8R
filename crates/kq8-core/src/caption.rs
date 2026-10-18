//! The caption as a picture (M4 step 4b-3): a line's text broken into
//! lines in the game's own font and drawn over the stone strip, on the
//! game's 640×480 screen.
//!
//! From the layout `IDDLG_CAPTION` in `Gplayscr.gui` (16 pixels per line,
//! text 2 pixels down, the font and strip picture it names) and the user's
//! check in the original (2026-10-18): at the bottom of the screen, text
//! centred, the strip growing upwards one line at a time, ending about two
//! letters beyond the text on each side; the colours look right (the
//! world's palette, index 0 see-through); the line breaks mostly match.
//! STAND-INS until the interface is studied (Q106, Q107): the wrap width
//! [`WRAP_WIDTH`], greedy word wrapping, the strip's side margin
//! [`STRIP_MARGIN`] around the widest line, tiled from its top-left.

use kq8_data::font::Font;
use kq8_data::palette::Rgb;

/// The game's screen.
pub const SCREEN: [u32; 2] = [640, 480];

/// STAND-IN (Q107 a): the widest a caption line may be, in pixels. The
/// user's check puts it between 365 and 418 ("… could be something" fits,
/// "… something useful" doesn't).
pub const WRAP_WIDTH: u32 = 400;

/// STAND-IN (Q107 a): the strip reaches this far beyond the widest line
/// on each side. The user saw "at most around 2 letters" in the original;
/// a lower-case letter is about 8 pixels in `console.pft`.
pub const STRIP_MARGIN: u32 = 16;

/// Width of `text` in `font` (characters without a glyph count 0).
pub fn text_width(font: &Font, text: &str) -> u32 {
    text.chars().map(|c| glyph_advance(font, c)).sum()
}

fn code(c: char) -> u8 {
    u8::try_from(c as u32).unwrap_or(b'?')
}

fn glyph_advance(font: &Font, c: char) -> u32 {
    font.glyph(code(c)).map_or(0, |g| (g.width as i32 + font.char_spacing).max(0) as u32)
}

/// Breaks `text` into lines no wider than `width`: words (split at
/// spaces) are added while they fit; a single word wider than `width`
/// gets a line of its own. Leading and trailing spaces are dropped.
pub fn wrap(font: &Font, text: &str, width: u32) -> Vec<String> {
    let mut lines = Vec::new();
    let mut line = String::new();
    for word in text.split_whitespace() {
        let candidate = if line.is_empty() { word.to_string() } else { format!("{line} {word}") };
        if !line.is_empty() && text_width(font, &candidate) > width {
            lines.push(std::mem::replace(&mut line, word.to_string()));
        } else {
            line = candidate;
        }
    }
    if !line.is_empty() {
        lines.push(line);
    }
    lines
}

/// What the caption box needs from the layout.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Box {
    /// Height of one line of the strip (the layout box's height).
    pub line_height: u32,
    /// Text top below the top of its line (the layout's caption y).
    pub text_y: u32,
}

/// The strip picture: width, height, palette indices (0 see-through).
pub struct Strip<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
}

/// Where the strip starts and ends on the screen (x, from..to): the
/// widest line plus [`STRIP_MARGIN`] on each side, centred.
pub fn strip_span(font: &Font, lines: &[String]) -> (u32, u32) {
    let widest = lines.iter().map(|l| text_width(font, l)).max().unwrap_or(0);
    let width = (widest + 2 * STRIP_MARGIN).min(SCREEN[0]);
    let x0 = (SCREEN[0] - width) / 2;
    (x0, x0 + width)
}

/// The caption's picture: `SCREEN[0]` wide, one `line_height` per line,
/// RGBA (see-through where neither the strip nor a letter has ink).
pub fn picture(font: &Font, strip: Option<&Strip>, palette: &[Rgb; 256], lines: &[String], b: Box) -> (u32, u32, Vec<u8>) {
    let (w, h) = (SCREEN[0], b.line_height * lines.len() as u32);
    let mut rgba = vec![0u8; (w * h * 4) as usize];
    let mut put = |x: u32, y: u32, index: u8| {
        if index != 0 && x < w && y < h {
            let at = ((y * w + x) * 4) as usize;
            let [r, g, bl] = palette[index as usize];
            rgba[at..at + 4].copy_from_slice(&[r, g, bl, 255]);
        }
    };
    if let Some(s) = strip.filter(|s| s.width > 0 && s.height > 0) {
        // Centred, as wide as the widest line plus the margin each side.
        let (x0, x1) = strip_span(font, lines);
        for y in 0..h {
            for x in x0..x1 {
                put(x, y, s.pixels[((y % s.height) * s.width + (x - x0) % s.width) as usize]);
            }
        }
    }
    for (k, line) in lines.iter().enumerate() {
        let top = k as u32 * b.line_height + b.text_y;
        let mut x = (w.saturating_sub(text_width(font, line))) / 2;
        for c in line.chars() {
            let Some(g) = font.glyph(code(c)) else { continue };
            if let Some(px) = font.glyph_pixels(g) {
                for gy in 0..g.height as u32 {
                    for gx in 0..g.width as u32 {
                        put(x + gx, top + gy, px[(gy * g.width as u32 + gx) as usize]);
                    }
                }
            }
            x += glyph_advance(font, c);
        }
    }
    (w, h, rgba)
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::font::Glyph;

    /// A font where every letter is 10 wide and the space 5 wide (one
    /// 10x2 picture of index 7).
    fn font() -> Font {
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
    fn words_are_added_while_they_fit() {
        let f = font();
        assert_eq!(text_width(&f, "ab cd"), 45);
        // "aaa bbb" = 65 fits 70; "aaa bbb ccc" = 105 doesn't.
        assert_eq!(wrap(&f, " aaa bbb ccc ", 70), vec!["aaa bbb", "ccc"]);
        assert_eq!(wrap(&f, "aaaaaaaaaa b", 50), vec!["aaaaaaaaaa", "b"], "a long word gets its own line");
        assert!(wrap(&f, "   ", 70).is_empty());
    }

    #[test]
    fn the_picture_has_one_strip_line_per_text_line_and_centred_text() {
        let f = font();
        let mut pal = [[0u8; 3]; 256];
        pal[7] = [200, 150, 50];
        pal[3] = [80, 80, 80];
        let strip = [0u8, 3, 3, 0];
        let lines = vec!["ab".to_string(), "c".to_string()];
        let (w, h, rgba) = picture(&f, Some(&Strip { width: 2, height: 2, pixels: &strip }), &pal, &lines, Box { line_height: 16, text_y: 2 });
        assert_eq!((w, h), (640, 32));
        let at = |x: u32, y: u32| &rgba[((y * w + x) * 4) as usize..((y * w + x) * 4 + 4) as usize];
        // The strip: "ab" (20) + 16 each side = 52 wide, x 294..346.
        assert_eq!(strip_span(&f, &lines), (294, 346));
        assert_eq!(at(1, 0), [0, 0, 0, 0], "outside the strip: see-through");
        assert_eq!(at(294, 0), [0, 0, 0, 0], "strip index 0: see-through");
        assert_eq!(at(295, 0), [80, 80, 80, 255]);
        assert_eq!(at(346, 1), [0, 0, 0, 0], "past its right end");
        // "ab" is 20 wide, centred: x 310..330, rows 2..4.
        assert_eq!(at(310, 2), [200, 150, 50, 255]);
        assert_eq!(at(329, 3), [200, 150, 50, 255]);
        // "c" on the second line: x 315..325, rows 18..20.
        assert_eq!(at(315, 18), [200, 150, 50, 255]);
    }
}
