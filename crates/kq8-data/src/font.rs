//! Fonts (`.pft`, M4 step 4b-1): a `PFON` block holding the font's
//! settings, a character map, a glyph table and the glyph pictures as a
//! bitmap array (`PBMA`), then an optional palette.
//!
//! Layout from the community notes `kq8pfon.txt` and `kq8pbma.txt`
//! (libkq8fpc, GPL-3.0: facts about the layout only; code and wording are
//! our own). Checked against every font in the install by `kq8
//! font-verify`. Sources: docs/provenance/README.md.

use crate::chunks::{tag_string, u16_at, u32_at};
use crate::{Bitmap, Palette};

/// `fontFlags` bits (`kq8pfon.txt`).
pub const FONT_PROPORTIONAL: u32 = 0x1;
pub const FONT_MONOSPACED: u32 = 0x2;
pub const FONT_MONOCHROME: u32 = 0x4;

/// One glyph: where its picture is and how it sits on the line.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Glyph {
    /// Index into [`Font::bitmaps`].
    pub bitmap: u8,
    /// Its rectangle inside that bitmap.
    pub left: u8,
    pub top: u8,
    pub width: u8,
    pub height: u8,
    /// Shift from the baseline, top-down.
    pub baseline_shift: i8,
}

/// A parsed `.pft` font.
#[derive(Debug, Clone)]
pub struct Font {
    pub version: u32,
    pub font_flags: u32,
    /// Alignment (bits 0-2 horizontal, 3-5 vertical) and stretch (bit 6).
    pub text_flags: u32,
    pub char_height: i32,
    /// The widest character (used by monospaced fonts).
    pub char_width: i32,
    pub text_color: u32,
    pub back_color: u32,
    /// Top-down, from the top of the character cell.
    pub baseline: i32,
    /// 16.16 fixed point turned into a number (1.0 = unscaled).
    pub h_scale: f32,
    pub v_scale: f32,
    /// Extra space between characters.
    pub char_spacing: i32,
    /// The first character code the map covers.
    pub char_first: i16,
    /// Glyph index per character code from `char_first` on; −1: none.
    pub char_map: Vec<i16>,
    pub glyphs: Vec<Glyph>,
    pub bitmaps: Vec<Bitmap>,
    /// The bitmap count the array's `head` gives.
    pub declared_bitmaps: u32,
    /// Chunks of the array other than `head` and the bitmaps, as (tag, size).
    pub array_chunks: Vec<(String, usize)>,
    pub palette: Option<Palette>,
    /// Bytes after the palette field and before the end of the block.
    pub trailing: usize,
}

fn i32_at(b: &[u8], at: usize) -> i32 {
    u32_at(b, at) as i32
}

fn need(b: &[u8], at: usize, n: usize, what: &str) -> Result<(), String> {
    if at + n > b.len() {
        return Err(format!("{what} at 0x{at:X} needs {n} bytes, {} left", b.len().saturating_sub(at)));
    }
    Ok(())
}

impl Font {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if !bytes.starts_with(b"PFON") {
            return Err("no 'PFON' tag".into());
        }
        need(bytes, 0, 8, "block header")?;
        let word = u32_at(bytes, 4);
        let size = (word & 0x7FFF_FFFF) as usize;
        let align = 2usize << (word >> 31);
        let end = 8 + size;
        let padded = 8 + size.next_multiple_of(align);
        if padded != bytes.len() {
            return Err(format!("block of {size} bytes (aligned to {align}) ends at {padded}, file has {}", bytes.len()));
        }
        let b = &bytes[..end];
        need(b, 8, 4 + 44 + 4, "version and font info")?;
        let version = u32_at(b, 8);
        let i = 12;
        let font_flags = u32_at(b, i);
        let text_flags = u32_at(b, i + 4);
        let glyph_count = i32_at(b, i + 8);
        let char_height = i32_at(b, i + 12);
        let char_width = i32_at(b, i + 16);
        let text_color = u32_at(b, i + 20);
        let back_color = u32_at(b, i + 24);
        let baseline = i32_at(b, i + 28);
        let h_scale = i32_at(b, i + 32) as f32 / 65536.0;
        let v_scale = i32_at(b, i + 36) as f32 / 65536.0;
        let char_spacing = i32_at(b, i + 40);
        let mut at = i + 44;
        let char_count = u16_at(b, at) as i16;
        let char_first = u16_at(b, at + 2) as i16;
        at += 4;
        if char_count < 0 || glyph_count < 0 {
            return Err(format!("negative count: {char_count} characters, {glyph_count} glyphs"));
        }
        need(b, at, 2 * char_count as usize, "character map")?;
        let char_map: Vec<i16> = (0..char_count as usize).map(|k| u16_at(b, at + 2 * k) as i16).collect();
        at += 2 * char_count as usize;
        need(b, at, 8 * glyph_count as usize, "glyph table")?;
        let glyphs: Vec<Glyph> = (0..glyph_count as usize)
            .map(|k| {
                let g = &b[at + 8 * k..at + 8 * k + 8];
                Glyph { bitmap: g[0], left: g[1], top: g[2], width: g[3], height: g[4], baseline_shift: g[5] as i8 }
            })
            .collect();
        at += 8 * glyph_count as usize;
        let (bitmaps, declared_bitmaps, array_chunks, after) = read_bitmap_array(b, at)?;
        at = after;
        need(b, at, 4, "palette flag")?;
        let has_palette = i32_at(b, at);
        at += 4;
        let palette = if has_palette != 0 {
            let p = Palette::parse(&b[at..]).map_err(|e| format!("palette: {e}"))?;
            at = b.len();
            Some(p)
        } else {
            None
        };
        Ok(Font {
            version,
            font_flags,
            text_flags,
            char_height,
            char_width,
            text_color,
            back_color,
            baseline,
            h_scale,
            v_scale,
            char_spacing,
            char_first,
            char_map,
            glyphs,
            bitmaps,
            declared_bitmaps,
            array_chunks,
            palette,
            trailing: b.len() - at,
        })
    }

    /// The glyph for character code `c`, if the font has one.
    pub fn glyph(&self, c: u8) -> Option<&Glyph> {
        let k = c as i32 - self.char_first as i32;
        let g = *self.char_map.get(usize::try_from(k).ok()?)?;
        self.glyphs.get(usize::try_from(g).ok()?)
    }

    /// The glyph's pixels (palette indices, top row first, `width` ×
    /// `height`), cut from its bitmap; `None` if it doesn't fit.
    pub fn glyph_pixels(&self, g: &Glyph) -> Option<Vec<u8>> {
        let bm = self.bitmaps.get(g.bitmap as usize)?;
        let (l, t, w, h) = (g.left as u32, g.top as u32, g.width as u32, g.height as u32);
        if bm.bit_count != 8 || l + w > bm.width || t + h > bm.height {
            return None;
        }
        let mut out = Vec::with_capacity((w * h) as usize);
        for y in t..t + h {
            let row = (y * bm.width + l) as usize;
            out.extend_from_slice(&bm.pixels[row..row + w as usize]);
        }
        Some(out)
    }

    /// Characters (from 32 to 255) that have a glyph.
    pub fn covered(&self) -> usize {
        (32u8..=255).filter(|&c| self.glyph(c).is_some()).count()
    }
}

/// Reads a `PBMA` bitmap array at `at`: its `head` (chunk count, version,
/// bitmap count), any other chunks before the first bitmap (`rmap`,
/// `user`), then one `PBMP` bitmap per entry, each running until the
/// next `PBMP` tag or the end of its known chunks. The `PBMA` and `PBMP`
/// size fields don't describe the data and are skipped (`kq8pbma.txt`).
/// Returns the bitmaps, the declared count, the other chunks and where
/// the array ends.
#[allow(clippy::type_complexity)]
fn read_bitmap_array(b: &[u8], at: usize) -> Result<(Vec<Bitmap>, u32, Vec<(String, usize)>, usize), String> {
    need(b, at, 8, "bitmap array")?;
    if &b[at..at + 4] != b"PBMA" {
        return Err(format!("no 'PBMA' tag at 0x{at:X}"));
    }
    let mut at = at + 8;
    let chunk = |at: usize| -> Result<([u8; 4], usize), String> {
        need(b, at, 8, "chunk header")?;
        let tag: [u8; 4] = b[at..at + 4].try_into().unwrap();
        Ok((tag, u32_at(b, at + 4) as usize))
    };
    let (tag, size) = chunk(at)?;
    if &tag != b"head" || size < 8 {
        return Err(format!("bitmap array: '{}' ({size} bytes) where 'head' was expected", tag_string(&tag)));
    }
    need(b, at + 8, size, "bitmap array head")?;
    let count = u32_at(b, at + 12);
    at += 8 + size;
    let mut others = Vec::new();
    loop {
        let (tag, size) = chunk(at)?;
        if &tag == b"PBMP" {
            break;
        }
        need(b, at + 8, size, "bitmap array chunk")?;
        others.push((tag_string(&tag), size));
        at += 8 + size;
    }
    let mut bitmaps = Vec::with_capacity(count as usize);
    for k in 0..count {
        let start = at;
        let (tag, _) = chunk(at)?;
        if &tag != b"PBMP" {
            return Err(format!("bitmap {k}: '{}' where 'PBMP' was expected at 0x{at:X}", tag_string(&tag)));
        }
        at += 8;
        // The bitmap's own chunks, until the next bitmap or something else.
        while at + 8 <= b.len() {
            let (tag, size) = chunk(at)?;
            if !matches!(&tag, b"head" | b"data" | b"DETL" | b"PPAL") {
                break;
            }
            need(b, at + 8, size, "bitmap chunk")?;
            at += 8 + size;
        }
        bitmaps.push(Bitmap::parse(&b[start..at]).map_err(|e| format!("bitmap {k}: {e}"))?);
    }
    Ok((bitmaps, count, others, at))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A one-glyph font: 'A' (65) is glyph 0, a 3x2 picture inside a 4x2
    /// bitmap, no palette.
    fn tiny() -> Vec<u8> {
        let mut d = Vec::new();
        d.extend(2u32.to_le_bytes()); // version
        for v in [0x41u32, 0, 1, 2, 3, 0xFF, 0, 2, 0x10000, 0x10000, 0] {
            d.extend(v.to_le_bytes());
        }
        d.extend(2i16.to_le_bytes()); // char count
        d.extend(65i16.to_le_bytes()); // first
        d.extend(0i16.to_le_bytes()); // 'A' -> 0
        d.extend((-1i16).to_le_bytes()); // 'B' none
        d.extend([0, 1, 0, 3, 2, 0, 0, 0]); // glyph 0
        d.extend(b"PBMA");
        d.extend(0u32.to_le_bytes());
        d.extend(b"head");
        d.extend(8u32.to_le_bytes());
        d.extend(2u32.to_le_bytes());
        d.extend(1u32.to_le_bytes());
        d.extend(b"PBMP");
        d.extend(0u32.to_le_bytes());
        d.extend(b"head");
        d.extend(20u32.to_le_bytes());
        for v in [2u32, 4, 2, 8, 0] {
            d.extend(v.to_le_bytes());
        }
        d.extend(b"data");
        d.extend(8u32.to_le_bytes());
        d.extend([0, 1, 2, 3, 4, 5, 6, 7]);
        d.extend(0i32.to_le_bytes()); // no palette
        let mut f = b"PFON".to_vec();
        f.extend((d.len() as u32).to_le_bytes());
        f.extend(d);
        f
    }

    #[test]
    fn a_tiny_font_gives_its_settings_map_and_glyph_pixels() {
        let f = Font::parse(&tiny()).unwrap();
        assert_eq!((f.version, f.font_flags, f.char_height, f.baseline, f.h_scale), (2, 0x41, 2, 2, 1.0));
        assert_eq!(f.bitmaps.len(), 1);
        assert_eq!(f.trailing, 0);
        let g = *f.glyph(b'A').unwrap();
        assert_eq!(f.glyph_pixels(&g), Some(vec![1, 2, 3, 5, 6, 7]));
        assert!(f.glyph(b'B').is_none() && f.glyph(b'C').is_none() && f.glyph(b' ').is_none());
        assert_eq!(f.covered(), 1);
    }

    #[test]
    fn a_wrong_block_size_is_refused() {
        let mut f = tiny();
        f.push(0);
        f.push(0);
        assert!(Font::parse(&f).is_err());
    }
}
