//! Bitmaps: `.pbm` / archive `.bmp` (chunked) and `.dib` (Windows BMP).
//!
//! Sources are listed in docs/provenance/README.md. Layouts checked against
//! the real files; see `kq8 bitmap-verify`.

use crate::chunks::{read_chunks, tag_string, u16_at, u32_at};
use crate::palette::Rgb;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BitmapFormat {
    /// `PBMP` tag followed by `head`, `data`, optional `DETL`.
    Chunked,
    /// Standard Windows `BM` file.
    WindowsBmp,
}

/// `head` flag bits (chunked bitmaps).
pub const FLAG_TRANSPARENT: u32 = 0x01;
pub const FLAG_DITHER: u32 = 0x02;
pub const FLAG_TRANSLUCENT: u32 = 0x04;

#[derive(Debug, Clone)]
pub struct Bitmap {
    pub format: BitmapFormat,
    pub width: u32,
    pub height: u32,
    /// 8 (palette indices) or 24 (true colour).
    pub bit_count: u32,
    /// `head` flags (chunked only, 0 otherwise).
    pub flags: u32,
    /// Number of mipmap levels (smaller copies) stored; 1 = none.
    pub mip_count: u32,
    /// Chunk count from `head` (chunked only).
    pub declared_chunks: Option<u32>,
    /// Tags of all chunks after the `PBMP` marker, in file order.
    pub chunk_tags: Vec<String>,
    /// Chunks other than PBMP/head/data/DETL, as (tag, size).
    pub other_chunks: Vec<(String, usize)>,
    /// Palette stored inside the file (Windows BMP, or a chunked `PPAL`).
    pub embedded_palette: Option<Vec<Rgb>>,
    /// Bytes in the image data beyond what the levels need (row/chunk padding).
    pub data_padding: usize,
    /// Level 0 pixels, top row first, no row padding.
    /// 8-bit: one palette index per pixel. 24-bit: red, green, blue.
    pub pixels: Vec<u8>,
}

impl Bitmap {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.starts_with(b"PBMP") || bytes.starts_with(b"head") {
            Self::parse_chunked(bytes)
        } else if bytes.starts_with(b"BM") {
            Self::parse_windows(bytes)
        } else {
            let mut t = [0u8; 4];
            t.iter_mut().zip(bytes).for_each(|(d, s)| *d = *s);
            Err(format!("unknown bitmap signature '{}'", tag_string(&t)))
        }
    }

    /// `PBMP` is only a marker: its size field does not describe the data,
    /// so chunk reading starts right after its 8 bytes.
    fn parse_chunked(bytes: &[u8]) -> Result<Self, String> {
        let start = if bytes.starts_with(b"PBMP") { 8 } else { 0 };
        let chunks = read_chunks(bytes, start).map_err(|e| e.to_string())?;

        let mut head = None;
        let mut data: Option<&[u8]> = None;
        let mut mip_count = 1;
        let mut other_chunks = Vec::new();
        let mut embedded_palette = None;
        for c in &chunks {
            match &c.tag {
                b"head" => head = Some(c.data),
                b"data" => data = Some(c.data),
                b"DETL" if c.data.len() >= 4 => mip_count = u32_at(c.data, 0),
                b"PPAL" => {
                    // A palette left in by the exporter. Kept for inspection;
                    // the engine skips it.
                    if let Ok(p) = crate::Palette::parse(c.data) {
                        embedded_palette = Some(p.colors.to_vec());
                    }
                    other_chunks.push((c.tag_str(), c.data.len()));
                }
                _ => other_chunks.push((c.tag_str(), c.data.len())),
            }
        }
        let head = head.ok_or("no 'head' chunk")?;
        if head.len() < 20 {
            return Err(format!("'head' chunk is {} bytes, expected 20", head.len()));
        }
        let data = data.ok_or("no 'data' chunk")?;
        let declared_chunks = u32_at(head, 0) & 0x00FF_FFFF;
        let width = u32_at(head, 4);
        let height = u32_at(head, 8);
        let bit_count = u32_at(head, 12);
        let flags = u32_at(head, 16);
        if !(1..=9).contains(&mip_count) {
            return Err(format!("mipmap count {mip_count} outside 1..=9"));
        }

        let (pixels, used) = match (bit_count, mip_count) {
            // Mipmapped: levels stored back to back, rows not padded, each
            // level a quarter of the previous one.
            (8, m) if m > 1 => {
                let level0 = (width * height) as usize;
                let total: usize = (0..m).map(|i| level0 >> (i * 2)).sum();
                if data.len() < total {
                    return Err(format!(
                        "{m} mipmap levels need {total} bytes, data has {}",
                        data.len()
                    ));
                }
                (data[..level0].to_vec(), total)
            }
            (8, _) | (24, _) => {
                let bpp = (bit_count / 8) as usize;
                let row = width as usize * bpp;
                let stride = row.next_multiple_of(4);
                let need = stride * height as usize;
                if data.len() < need {
                    return Err(format!(
                        "{width}x{height}x{bit_count} needs {need} bytes, data has {}",
                        data.len()
                    ));
                }
                let mut out = Vec::with_capacity(row * height as usize);
                for y in 0..height as usize {
                    let line = &data[y * stride..y * stride + row];
                    if bpp == 3 {
                        // stored blue, green, red
                        line.as_chunks::<3>().0.iter().for_each(|p| out.extend([p[2], p[1], p[0]]));
                    } else {
                        out.extend_from_slice(line);
                    }
                }
                (out, need)
            }
            (b, m) => return Err(format!("unsupported bit depth {b} with {m} mipmaps")),
        };

        Ok(Bitmap {
            format: BitmapFormat::Chunked,
            width,
            height,
            bit_count,
            flags,
            mip_count,
            declared_chunks: Some(declared_chunks),
            chunk_tags: chunks.iter().map(|c| c.tag_str()).collect(),
            other_chunks,
            embedded_palette,
            data_padding: data.len() - used,
            pixels,
        })
    }

    /// Uncompressed 8- or 24-bit Windows BMP.
    fn parse_windows(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 54 {
            return Err("Windows BMP shorter than its headers".into());
        }
        let pixel_offset = u32_at(bytes, 10) as usize;
        let info_size = u32_at(bytes, 14) as usize;
        let width = u32_at(bytes, 18) as i32;
        let raw_height = u32_at(bytes, 22) as i32;
        let bit_count = u16_at(bytes, 28) as u32;
        let compression = u32_at(bytes, 30);
        let colors_used = u32_at(bytes, 46) as usize;
        if compression != 0 {
            return Err(format!("compressed Windows BMP (type {compression}) not supported"));
        }
        if width <= 0 || raw_height == 0 {
            return Err(format!("bad Windows BMP size {width}x{raw_height}"));
        }
        let bottom_up = raw_height > 0;
        let (width, height) = (width as u32, raw_height.unsigned_abs());

        let embedded_palette = if bit_count == 8 {
            let n = if colors_used == 0 { 256 } else { colors_used.min(256) };
            let at = 14 + info_size;
            let table = bytes
                .get(at..at + n * 4)
                .ok_or("Windows BMP colour table runs past end of file")?;
            // RGBQUAD: blue, green, red, reserved
            Some(table.as_chunks::<4>().0.iter().map(|q| [q[2], q[1], q[0]]).collect())
        } else {
            None
        };

        let bpp = match bit_count {
            8 => 1,
            24 => 3,
            b => return Err(format!("unsupported Windows BMP bit depth {b}")),
        };
        let row = width as usize * bpp;
        let stride = row.next_multiple_of(4);
        let need = stride * height as usize;
        let data = bytes
            .get(pixel_offset..pixel_offset + need)
            .ok_or(format!("Windows BMP pixel data ({need} bytes at {pixel_offset}) runs past end of file"))?;
        let mut pixels = Vec::with_capacity(row * height as usize);
        for y in 0..height as usize {
            let src_y = if bottom_up { height as usize - 1 - y } else { y };
            let line = &data[src_y * stride..src_y * stride + row];
            if bpp == 3 {
                line.as_chunks::<3>().0.iter().for_each(|p| pixels.extend([p[2], p[1], p[0]]));
            } else {
                pixels.extend_from_slice(line);
            }
        }

        Ok(Bitmap {
            format: BitmapFormat::WindowsBmp,
            width,
            height,
            bit_count,
            flags: 0,
            mip_count: 1,
            declared_chunks: None,
            chunk_tags: Vec::new(),
            other_chunks: Vec::new(),
            embedded_palette,
            data_padding: bytes.len() - pixel_offset - need,
            pixels,
        })
    }

    /// How many pixels use each palette index (8-bit images only).
    pub fn index_histogram(&self) -> Option<[u32; 256]> {
        (self.bit_count == 8).then(|| {
            let mut h = [0u32; 256];
            self.pixels.iter().for_each(|&i| h[i as usize] += 1);
            h
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn chunk(tag: &[u8; 4], data: &[u8]) -> Vec<u8> {
        let mut v = tag.to_vec();
        v.extend_from_slice(&(data.len() as u32).to_le_bytes());
        v.extend_from_slice(data);
        v
    }

    fn head(w: u32, h: u32, bits: u32, flags: u32) -> Vec<u8> {
        [2, w, h, bits, flags].iter().flat_map(|v| v.to_le_bytes()).collect()
    }

    #[test]
    fn chunked_8bit_with_row_padding() {
        // 3x2: rows padded from 3 to 4 bytes
        let mut f = b"PBMP\0\0\0\0".to_vec();
        f.extend(chunk(b"head", &head(3, 2, 8, 0)));
        f.extend(chunk(b"data", &[1, 2, 3, 0, 4, 5, 6, 0]));
        f.extend(chunk(b"DETL", &1u32.to_le_bytes()));
        let b = Bitmap::parse(&f).unwrap();
        assert_eq!((b.width, b.height, b.mip_count), (3, 2, 1));
        assert_eq!(b.pixels, vec![1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn chunked_mipmaps() {
        // 4x4 with 3 levels: 16 + 4 + 1 bytes, padded to 24
        let mut data: Vec<u8> = (0..21).collect();
        data.extend([0, 0, 0]);
        let mut f = b"PBMP\0\0\0\0".to_vec();
        f.extend(chunk(b"head", &head(4, 4, 8, 0)));
        f.extend(chunk(b"data", &data));
        f.extend(chunk(b"DETL", &3u32.to_le_bytes()));
        let b = Bitmap::parse(&f).unwrap();
        assert_eq!(b.pixels, (0..16).collect::<Vec<u8>>());
        assert_eq!(b.data_padding, 3);
    }

    #[test]
    fn chunked_24bit_is_bgr() {
        let mut f = b"PBMP\0\0\0\0".to_vec();
        f.extend(chunk(b"head", &head(1, 1, 24, 0)));
        f.extend(chunk(b"data", &[10, 20, 30, 0]));
        let b = Bitmap::parse(&f).unwrap();
        assert_eq!(b.pixels, vec![30, 20, 10]);
    }

    #[test]
    fn windows_bmp_bottom_up() {
        // 1x2, 8-bit, 2-colour table
        let mut f = b"BM".to_vec();
        f.extend_from_slice(&0u32.to_le_bytes()); // file size (unused)
        f.extend_from_slice(&0u32.to_le_bytes()); // reserved
        f.extend_from_slice(&(14u32 + 40 + 8).to_le_bytes());
        for v in [40u32, 1, 2] {
            f.extend_from_slice(&v.to_le_bytes());
        }
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&8u16.to_le_bytes());
        for v in [0u32, 0, 0, 0, 2, 0] {
            f.extend_from_slice(&v.to_le_bytes());
        }
        f.extend_from_slice(&[0, 0, 255, 0, 255, 0, 0, 0]); // red, blue
        f.extend_from_slice(&[1, 0, 0, 0, 0, 0, 0, 0]); // bottom row first
        let b = Bitmap::parse(&f).unwrap();
        assert_eq!(b.pixels, vec![0, 1]);
        assert_eq!(b.embedded_palette.unwrap(), vec![[255, 0, 0], [0, 0, 255]]);
    }
}
