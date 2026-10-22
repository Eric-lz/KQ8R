//! Drawing one panel of the game-play screen into a picture (M4 step
//! 4c-2): its pictures and numbers, placed by its layout block.
//!
//! From `docs/behaviour/interface.md`:
//!
//! - Children's rectangles are relative to their dialog (rule 5).
//! - A text with a rectangle whose right is left of its left is anchored
//!   at (left, top) and aligned by its justification (rule 5); otherwise
//!   it is aligned inside its rectangle. Justification: horizontal 1 left,
//!   2 right, 4 centred; vertical 0x8 centred, 0x10 from the top (rule 6).
//! - A bar is its picture cut to a width (rule 20).
//!
//! Our reading of the data, not in the note: a dialog's children are
//! drawn **last first**, so the first one listed is on top (in
//! `Gplayscr.gui` every panel lists its numbers before its pictures).
//! A picture is see-through where its index is 0 only if its own flags
//! have bit 1 (the community notes `kq8pbmp.txt`: "transparent"; only the
//! stone strip has it among the panels' pictures).
//! STAND-INS (Q108): rectangles are scaled with the screen (× 1.25 at
//! 800 × 600) and rounded; pictures stretched with them (nearest pixel)
//! and tiled from their rectangle's top-left when smaller; fonts at their
//! own size.
//! 4c-9: every edge is rounded where it stands on the screen (the panel's
//! place plus the child's), as the panel's own edges are, so a picture
//! is exactly as many pixels as its place: the meters are 14 layout
//! pixels at 466, 582.5 to 600 on the screen, 17 rows (not 18).

use kq8_data::font::Font;
use kq8_data::palette::Rgb;

/// A picture: size, palette indices, whether index 0 is see-through.
#[derive(Clone, Copy)]
pub struct Pic<'a> {
    pub width: u32,
    pub height: u32,
    pub pixels: &'a [u8],
    pub see_through: bool,
}

impl<'a> Pic<'a> {
    /// From a bitmap (8-bit only).
    pub fn of(b: &'a kq8_data::Bitmap) -> Option<Self> {
        (b.bit_count == 8 && b.width > 0 && b.height > 0).then_some(Pic { width: b.width, height: b.height, pixels: &b.pixels, see_through: b.flags & 1 != 0 })
    }
}

/// One thing to draw, its rectangle in layout pixels relative to the
/// panel.
pub enum Piece<'a> {
    /// A picture filling its rectangle; `cut`: only this many screen
    /// pixels of its width, from the left (a meter, rule 20).
    Picture { rect: [i32; 4], pic: Pic<'a>, cut: Option<u32> },
    /// A picture stretched to its rectangle (nearest pixel), not repeated
    /// (M6 6e fix: the popup's highlighted button, 88 x 25 on a 35-high
    /// button).
    Stretched { rect: [i32; 4], pic: Pic<'a> },
    /// A picture filling its rectangle, only its lowest `rows` screen
    /// rows (a potion's timer icon, the part left: `potions.md` rule 35).
    Lower { rect: [i32; 4], pic: Pic<'a>, rows: u32 },
    /// A text in a font with a justification.
    Text { rect: [i32; 4], font: &'a Font, justification: u32, text: String },
    /// A rectangle of one colour (4c-8: the black behind the meters'
    /// bars, the user's check).
    Fill { rect: [i32; 4], rgb: [u8; 3] },
    /// The magic map (M6 6d-3, `magic-map.md` rules 6, 18): a 460 × 460
    /// picture drawn at `view` (left, top, right, bottom in layout pixels
    /// from the centre of `rect`), nearest pixel, cut to `rect`; the arrow
    /// on top with its top-left at `arrow.1` (from the centre), cut too.
    Map { rect: [i32; 4], pixels: &'a [u8], view: [f64; 4], arrow: Option<(Pic<'a>, [f64; 2])> },
}

/// Layout pixels → screen pixels, rounded.
pub fn scaled(v: i32, scale: f32) -> i32 {
    (v as f32 * scale).round() as i32
}

fn code(c: char) -> u8 {
    u8::try_from(c as u32).unwrap_or(b'?')
}

/// A text's width in a font (glyph width × horizontal scale + spacing,
/// as the caption's, `interface.md` rule 47).
pub fn text_width(font: &Font, text: &str) -> i32 {
    text.chars().map(|c| font.glyph(code(c)).map_or(0.0, |g| g.width as f32 * font.h_scale) + font.char_spacing as f32).sum::<f32>().round() as i32
}

/// Where a text's top-left goes (screen pixels, relative to the panel):
/// rule 5 (anchor) and rule 6 (justification).
pub fn text_origin(rect: [i32; 4], justification: u32, width: i32, height: i32, scale: f32) -> (i32, i32) {
    let [l, t, r, b] = rect.map(|v| scaled(v, scale));
    let anchor = rect[2] < rect[0];
    let h = justification & 0x7;
    let v = (justification >> 3) & 0x7;
    let x = match (anchor, h) {
        (true, 2) => l - width,
        (true, 4) => l - width / 2,
        (true, _) => l,
        (false, 2) => r - width,
        (false, 4) => l + (r - l - width) / 2,
        (false, _) => l,
    };
    let y = match (anchor, v) {
        (true, 1) => t - height / 2,
        (true, _) => t,
        (false, 1) => t + (b - t - height) / 2,
        (false, _) => t,
    };
    (x, y)
}

/// A panel's picture: `size` its layout size; RGBA, see-through where
/// nothing is drawn.
pub struct Canvas {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    /// The panel's top-left on the screen; pieces are drawn at their
    /// screen places less this.
    offset: [i32; 2],
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        Canvas { width, height, rgba: vec![0; (width * height * 4) as usize], offset: [0, 0] }
    }

    fn put(&mut self, x: i32, y: i32, index: u8, see_through: bool, palette: &[Rgb; 256]) {
        let (x, y) = (x - self.offset[0], y - self.offset[1]);
        if (see_through && index == 0) || x < 0 || y < 0 || x >= self.width as i32 || y >= self.height as i32 {
            return;
        }
        let at = ((y as u32 * self.width + x as u32) * 4) as usize;
        let [r, g, b] = palette[index as usize];
        self.rgba[at..at + 4].copy_from_slice(&[r, g, b, 255]);
    }

    /// RGBA at (x, y), for tests and logs.
    pub fn at(&self, x: u32, y: u32) -> [u8; 4] {
        let at = ((y * self.width + x) * 4) as usize;
        self.rgba[at..at + 4].try_into().unwrap()
    }

    /// Draws `pic` filling the layout rectangle `rect` (stretched by
    /// `scale`, tiled), only its first `cut` screen pixels if given.
    pub fn picture(&mut self, rect: [i32; 4], pic: Pic, cut: Option<u32>, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let right = cut.map_or(r, |w| (l + w as i32).min(r));
        for y in t..b {
            for x in l..right {
                let sx = ((x - l) as f32 / scale) as u32 % pic.width;
                let sy = ((y - t) as f32 / scale) as u32 % pic.height;
                self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], pic.see_through, palette);
            }
        }
    }

    /// Draws `pic` stretched over the layout rectangle `rect` (nearest
    /// pixel).
    pub fn picture_stretched(&mut self, rect: [i32; 4], pic: Pic, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let (w, h) = ((r - l).max(1) as f32, (b - t).max(1) as f32);
        for y in t..b {
            for x in l..r {
                let sx = (((x - l) as f32 / w * pic.width as f32) as u32).min(pic.width - 1);
                let sy = (((y - t) as f32 / h * pic.height as f32) as u32).min(pic.height - 1);
                self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], pic.see_through, palette);
            }
        }
    }

    /// Draws `pic` as [`Canvas::picture`], only its lowest `rows` screen
    /// rows (the rows above are left as they are).
    pub fn picture_lower(&mut self, rect: [i32; 4], pic: Pic, rows: u32, scale: f32, palette: &[Rgb; 256]) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        for y in (b - rows as i32).max(t)..b {
            for x in l..r {
                let sx = ((x - l) as f32 / scale) as u32 % pic.width;
                let sy = ((y - t) as f32 / scale) as u32 % pic.height;
                self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], pic.see_through, palette);
            }
        }
    }

    /// Fills the layout rectangle `rect` with one colour.
    pub fn fill(&mut self, rect: [i32; 4], rgb: [u8; 3], scale: f32) {
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let [ox, oy] = self.offset;
        for y in (t - oy).max(0)..(b - oy).min(self.height as i32) {
            for x in (l - ox).max(0)..(r - ox).min(self.width as i32) {
                let at = ((y as u32 * self.width + x as u32) * 4) as usize;
                self.rgba[at..at + 4].copy_from_slice(&[rgb[0], rgb[1], rgb[2], 255]);
            }
        }
    }

    /// Draws the magic map (see [`Piece::Map`]); `rect` in layout pixels
    /// where the panel stands.
    pub fn map(&mut self, rect: [i32; 4], pixels: &[u8], view: [f64; 4], arrow: Option<(Pic, [f64; 2])>, scale: f32, palette: &[Rgb; 256]) {
        let size = kq8_data::magic_map::MAP_SIZE;
        let [l, t, r, b] = rect.map(|v| scaled(v, scale));
        let s = scale as f64;
        let (cx, cy) = ((rect[0] + rect[2]) as f64 / 2.0, (rect[1] + rect[3]) as f64 / 2.0);
        let (w, h) = (view[2] - view[0], view[3] - view[1]);
        if pixels.len() == size * size && w > 0.0 && h > 0.0 {
            for y in t..b {
                let uy = (y as f64 + 0.5) / s - cy;
                let my = ((uy - view[1]) / h * size as f64).floor();
                if my < 0.0 || my >= size as f64 {
                    continue;
                }
                for x in l..r {
                    let ux = (x as f64 + 0.5) / s - cx;
                    let mx = ((ux - view[0]) / w * size as f64).floor();
                    if mx < 0.0 || mx >= size as f64 {
                        continue;
                    }
                    self.put(x, y, pixels[my as usize * size + mx as usize], false, palette);
                }
            }
        }
        if let Some((pic, at)) = arrow {
            let (ax, ay) = (((cx + at[0]) * s).round() as i32, ((cy + at[1]) * s).round() as i32);
            let (aw, ah) = ((pic.width as f64 * s).round() as i32, (pic.height as f64 * s).round() as i32);
            for y in ay.max(t)..(ay + ah).min(b) {
                for x in ax.max(l)..(ax + aw).min(r) {
                    let sx = (((x - ax) as f64 / s) as u32).min(pic.width - 1);
                    let sy = (((y - ay) as f64 / s) as u32).min(pic.height - 1);
                    self.put(x, y, pic.pixels[(sy * pic.width + sx) as usize], true, palette);
                }
            }
        }
    }

    /// Draws `text` in `font`, placed by `rect` and `justification`
    /// (rules 5–6); letters at the font's own size, index 0 see-through.
    pub fn text(&mut self, rect: [i32; 4], font: &Font, justification: u32, text: &str, scale: f32, palette: &[Rgb; 256]) {
        let (mut x, y) = text_origin(rect, justification, text_width(font, text), font.char_height, scale);
        for c in text.chars() {
            let Some(g) = font.glyph(code(c)) else { continue };
            if let Some(px) = font.glyph_pixels(g) {
                for gy in 0..g.height as i32 {
                    for gx in 0..g.width as i32 {
                        self.put(x + gx, y + gy, px[(gy * g.width as i32 + gx) as usize], true, palette);
                    }
                }
            }
            x += (g.width as f32 * font.h_scale + font.char_spacing as f32).round() as i32;
        }
    }
}

/// The screen pixels a panel at layout point `origin`, `size` layout
/// pixels big, covers: its edges rounded where they stand (4c-9).
pub fn screen_size(origin: [i32; 2], size: [i32; 2], scale: f32) -> [u32; 2] {
    [0, 1].map(|k| (scaled(origin[k] + size[k], scale) - scaled(origin[k], scale)).max(1) as u32)
}

/// Draws a panel standing at layout point `origin`, `size` layout pixels
/// big, from its pieces (rectangles relative to the panel), **last
/// first** (the first piece ends on top).
pub fn draw(origin: [i32; 2], size: [i32; 2], scale: f32, pieces: &[Piece], palette: &[Rgb; 256]) -> Canvas {
    let [w, h] = screen_size(origin, size, scale);
    let mut c = Canvas::new(w, h);
    c.offset = origin.map(|v| scaled(v, scale));
    let [ox, oy] = origin;
    let at = |r: &[i32; 4]| [r[0] + ox, r[1] + oy, r[2] + ox, r[3] + oy];
    for piece in pieces.iter().rev() {
        match piece {
            Piece::Picture { rect, pic, cut } => c.picture(at(rect), *pic, *cut, scale, palette),
            Piece::Lower { rect, pic, rows } => c.picture_lower(at(rect), *pic, *rows, scale, palette),
            Piece::Stretched { rect, pic } => c.picture_stretched(at(rect), *pic, scale, palette),
            Piece::Text { rect, font, justification, text } => c.text(at(rect), font, *justification, text, scale, palette),
            Piece::Fill { rect, rgb } => c.fill(at(rect), *rgb, scale),
            Piece::Map { rect, pixels, view, arrow } => c.map(at(rect), pixels, *view, *arrow, scale, palette),
        }
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_anchored_at_a_point_or_aligned_in_their_box() {
        // IDCTL_HEALTHBAR_1TEXT (29, 2)-(28, 1), 0x12: right edge at x 29,
        // top at y 2 (rule 5).
        assert_eq!(text_origin([29, 2, 28, 1], 0x12, 10, 15, 1.0), (19, 2));
        // IDCTL_INVBAR_ARMORTEXT (26, 5)-(56, 19), 0x12: right edge at 56.
        assert_eq!(text_origin([26, 5, 56, 19], 0x12, 10, 15, 1.0), (46, 5));
        // IDCTL_METERS_LEVEL (100, 0)-(156, 14), 0xC: centred across, top.
        assert_eq!(text_origin([100, 0, 156, 14], 0xC, 10, 14, 1.0), (123, 0));
        // 0x9: left, centred down.
        assert_eq!(text_origin([0, 0, 50, 20], 0x9, 10, 10, 1.0), (0, 5));
        // Scaled by 1.25: the anchor moves, the text keeps its size.
        assert_eq!(text_origin([56, 5, 26, 4], 0x12, 10, 15, 1.25), (60, 6));
    }

    #[test]
    fn a_lower_part_keeps_its_own_rows() {
        let pal = [[0u8; 3]; 256].map(|_| [0, 0, 0]);
        let mut pal = pal;
        pal[1] = [10, 0, 0];
        pal[2] = [20, 0, 0];
        let empty = [1u8; 4];
        let full = [2u8; 4];
        // 2 x 2: the lower row from `full`, the upper from `empty`.
        let c = draw([0, 0], [2, 2], 1.0, &[
            Piece::Lower { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &full, see_through: true }, rows: 1 },
            Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &empty, see_through: true }, cut: None },
        ], &pal);
        assert_eq!((c.at(0, 0)[0], c.at(1, 1)[0]), (10, 20));
    }

    #[test]
    fn a_stretched_picture_is_not_repeated() {
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [255, 0, 0];
        pal[2] = [0, 255, 0];
        // 1 x 2: red on top, green below; stretched over 1 x 4.
        let pixels = [1u8, 2];
        let c = draw([0, 0], [1, 4], 1.0, &[Piece::Stretched { rect: [0, 0, 1, 4], pic: Pic { width: 1, height: 2, pixels: &pixels, see_through: false } }], &pal);
        assert_eq!([c.at(0, 0)[0], c.at(0, 1)[0], c.at(0, 2)[1], c.at(0, 3)[1]], [255, 255, 255, 255], "top half red, bottom half green");
    }

    #[test]
    fn pictures_fill_their_box_bars_are_cut_and_the_first_piece_is_on_top() {
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [255, 0, 0];
        pal[2] = [0, 255, 0];
        let red = [1u8; 4];
        let back = [2u8, 2, 0, 2];
        let pieces = [
            // A bar 10 wide, 2x2 picture tiled, cut at 4 pixels.
            Piece::Picture { rect: [0, 0, 10, 2], pic: Pic { width: 2, height: 2, pixels: &red, see_through: false }, cut: Some(4) },
            // Under it: a background whose index 0 is drawn (not see-through).
            Piece::Picture { rect: [0, 0, 10, 4], pic: Pic { width: 2, height: 2, pixels: &back, see_through: false }, cut: None },
        ];
        let c = draw([0, 0], [10, 4], 1.0, &pieces, &pal);
        assert_eq!((c.width, c.height), (10, 4));
        assert_eq!(c.at(3, 1), [255, 0, 0, 255], "the bar on top");
        assert_eq!(c.at(4, 0), [0, 255, 0, 255], "past 4 pixels: the background");
        assert_eq!(c.at(0, 3), [0, 0, 0, 255], "index 0 of an opaque picture is drawn");
        // See-through index 0.
        let st = draw([0, 0], [2, 2], 1.0, &[Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &back, see_through: true }, cut: None }], &pal);
        assert_eq!(st.at(0, 1), [0, 0, 0, 0]);
        // Scaled by 2: a 2x2 picture covers 4x4.
        let big = draw([0, 0], [2, 2], 2.0, &[Piece::Picture { rect: [0, 0, 2, 2], pic: Pic { width: 2, height: 2, pixels: &back, see_through: true }, cut: None }], &pal);
        assert_eq!((big.width, big.at(1, 1), big.at(1, 2)), (4, [0, 255, 0, 255], [0, 0, 0, 0]));
        // 4c-8: a cut bar over black: the empty part is black, not see-through.
        let bar = [
            Piece::Picture { rect: [0, 0, 10, 2], pic: Pic { width: 2, height: 2, pixels: &red, see_through: false }, cut: Some(4) },
            Piece::Fill { rect: [0, 0, 10, 2], rgb: [0, 0, 0] },
        ];
        let c = draw([0, 0], [10, 2], 1.0, &bar, &pal);
        assert_eq!((c.at(3, 0), c.at(9, 1)), ([255, 0, 0, 255], [0, 0, 0, 255]));
        // 4c-9: the meters, 14 layout pixels high at 466, × 1.25: 582.5
        // to 600 on the screen, so 17 rows, and a piece filling them all.
        assert_eq!(screen_size([192, 466], [256, 14], 1.25), [320, 17]);
        let full = draw([192, 466], [2, 14], 1.25, &[Piece::Fill { rect: [0, 0, 2, 14], rgb: [9, 9, 9] }], &pal);
        assert_eq!((full.height, full.at(0, 0), full.at(1, 16)), (17, [9, 9, 9, 255], [9, 9, 9, 255]));
    }

    #[test]
    fn the_map_at_its_view_with_the_arrow_cut_to_the_area() {
        let size = kq8_data::magic_map::MAP_SIZE;
        let mut pal = [[0u8; 3]; 256];
        pal[1] = [10, 0, 0];
        pal[2] = [20, 0, 0];
        pal[3] = [30, 0, 0];
        let mut pixels = vec![1u8; size * size];
        // The top-left 10 x 10 map pixels (one layout pixel at this view).
        for y in 0..10 {
            pixels[y * size..y * size + 10].fill(2);
        }
        // Area 10 x 10 at (0, 0); the view puts the picture's top-left at
        // the area's centre, 460 map pixels over 46 layout pixels.
        let arrow = [3u8; 4];
        let pic = Pic { width: 2, height: 2, pixels: &arrow, see_through: true };
        let c = draw([0, 0], [10, 10], 1.0, &[Piece::Map { rect: [0, 0, 10, 10], pixels: &pixels, view: [0.0, 0.0, 46.0, 46.0], arrow: Some((pic, [3.0, 3.0])) }], &pal);
        assert_eq!(c.at(4, 4), [0, 0, 0, 0], "left of and above the picture: nothing");
        assert_eq!(c.at(5, 5), [20, 0, 0, 255], "the picture's first pixel at the centre");
        assert_eq!(c.at(6, 6), [10, 0, 0, 255]);
        assert_eq!((c.at(8, 8), c.at(9, 9)), ([30, 0, 0, 255], [30, 0, 0, 255]), "the arrow at centre + 3");
        // Cut at the area's edge: the arrow's second column would be x 10.
        let c = draw([0, 0], [11, 10], 1.0, &[Piece::Map { rect: [0, 0, 10, 10], pixels: &pixels, view: [0.0, 0.0, 46.0, 46.0], arrow: Some((pic, [4.0, 0.0])) }], &pal);
        assert_eq!((c.at(9, 5), c.at(10, 5)), ([30, 0, 0, 255], [0, 0, 0, 0]));
    }

    #[test]
    fn a_text_is_drawn_at_its_anchor() {
        let f = crate::caption::tests::font();
        let mut pal = [[0u8; 3]; 256];
        pal[7] = [200, 150, 50];
        let c = draw([0, 0], [40, 4], 1.0, &[Piece::Text { rect: [30, 1, 29, 0], font: &f, justification: 0x12, text: "ab".into() }], &pal);
        // "ab" is 20 wide: x 10..30, y 1..3.
        assert_eq!(c.at(10, 1), [200, 150, 50, 255]);
        assert_eq!(c.at(29, 2), [200, 150, 50, 255]);
        assert_eq!(c.at(9, 1), [0, 0, 0, 0]);
        assert_eq!(c.at(30, 1), [0, 0, 0, 0]);
    }
}
