//! Palettes: `.ppl` (chunked) and `.pal` (Windows RIFF).
//!
//! Sources are listed in docs/provenance/README.md. Layouts checked against
//! the real files; see `kq8 bitmap-verify`.

use crate::chunks::{read_chunks, tag_string, u16_at, u32_at};

pub type Rgb = [u8; 3];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PaletteFormat {
    /// `PPAL` tag followed by `head`, `data` and lighting tables.
    Chunked,
    /// Standard Windows `RIFF` / `PAL ` file.
    Riff,
}

#[derive(Debug, Clone)]
pub struct Palette {
    pub format: PaletteFormat,
    /// Colours as stored in the file. Missing entries are black.
    pub colors: [Rgb; 256],
    /// How many entries the file really contained.
    pub entry_count: usize,
    /// Chunk count from the `head` chunk (chunked palettes only).
    pub declared_chunks: Option<u32>,
    /// Tags of all chunks after the marker, in file order.
    pub chunk_tags: Vec<String>,
    /// Chunks after `data` (lighting, haze, colour-match tables), as
    /// (tag, size). Not decoded yet.
    pub extra_chunks: Vec<(String, usize)>,
}

/// The 8 colours the engine forces at each end of a chunked palette,
/// i.e. the standard Windows system colours. Source: libkq8fpc notes
/// (documented, not yet checked against Mask.exe; see provenance).
const SYSTEM_FIRST: [Rgb; 8] = [
    [0x00, 0x00, 0x00], [0x80, 0x00, 0x00], [0x00, 0x80, 0x00], [0x80, 0x80, 0x00],
    [0x00, 0x00, 0x80], [0x80, 0x00, 0x80], [0x00, 0x80, 0x80], [0xC0, 0xC0, 0xC0],
];
const SYSTEM_LAST: [Rgb; 8] = [
    [0x80, 0x80, 0x80], [0xFF, 0x00, 0x00], [0x00, 0xFF, 0x00], [0xFF, 0xFF, 0x00],
    [0x00, 0x00, 0xFF], [0xFF, 0x00, 0xFF], [0x00, 0xFF, 0xFF], [0xFF, 0xFF, 0xFF],
];

impl Palette {
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        if bytes.starts_with(b"RIFF") {
            Self::parse_riff(bytes)
        } else if bytes.starts_with(b"PPAL") {
            Self::parse_chunked(bytes)
        } else {
            Err(format!(
                "unknown palette signature '{}'",
                tag_string(&first4(bytes))
            ))
        }
    }

    /// `PPAL` is only a marker: its size field does not describe the data,
    /// so chunk reading starts right after its 8 bytes.
    fn parse_chunked(bytes: &[u8]) -> Result<Self, String> {
        let chunks = read_chunks(bytes, 8).map_err(|e| e.to_string())?;
        let mut pal = Palette {
            format: PaletteFormat::Chunked,
            colors: [[0; 3]; 256],
            entry_count: 0,
            declared_chunks: None,
            chunk_tags: chunks.iter().map(|c| c.tag_str()).collect(),
            extra_chunks: Vec::new(),
        };
        let mut have_data = false;
        for c in &chunks {
            match &c.tag {
                b"head" if c.data.len() >= 4 => {
                    // low 24 bits: chunk count; high 8 bits: version
                    pal.declared_chunks = Some(u32_at(c.data, 0) & 0x00FF_FFFF);
                }
                b"data" if !have_data => {
                    // PALETTEENTRY: red, green, blue, flags
                    pal.entry_count = (c.data.len() / 4).min(256);
                    for i in 0..pal.entry_count {
                        pal.colors[i] = [c.data[i * 4], c.data[i * 4 + 1], c.data[i * 4 + 2]];
                    }
                    have_data = true;
                }
                _ => pal.extra_chunks.push((c.tag_str(), c.data.len())),
            }
        }
        if !have_data {
            return Err("palette has no 'data' chunk".into());
        }
        Ok(pal)
    }

    fn parse_riff(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[8..12] != b"PAL " {
            return Err("RIFF file is not a palette (no 'PAL ' form)".into());
        }
        let chunks = read_chunks(&bytes[..bytes.len().min(8 + u32_at(bytes, 4) as usize)], 12)
            .map_err(|e| e.to_string())?;
        let data = chunks
            .iter()
            .find(|c| &c.tag == b"data")
            .ok_or("RIFF palette has no 'data' chunk")?;
        if data.data.len() < 4 {
            return Err("RIFF palette 'data' chunk too small".into());
        }
        // LOGPALETTE: u16 version (0x300), u16 count, then entries.
        let count = u16_at(data.data, 2) as usize;
        let available = (data.data.len() - 4) / 4;
        if count > available {
            return Err(format!("RIFF palette claims {count} colours, has room for {available}"));
        }
        let mut colors = [[0; 3]; 256];
        for (i, slot) in colors.iter_mut().enumerate().take(count.min(256)) {
            let e = &data.data[4 + i * 4..];
            *slot = [e[0], e[1], e[2]];
        }
        Ok(Palette {
            format: PaletteFormat::Riff,
            colors,
            entry_count: count,
            declared_chunks: None,
            chunk_tags: chunks.iter().map(|c| c.tag_str()).collect(),
            extra_chunks: chunks
                .iter()
                .filter(|c| &c.tag != b"data")
                .map(|c| (c.tag_str(), c.data.len()))
                .collect(),
        })
    }

    /// The colours as the engine shows them: for chunked palettes the
    /// first and last 8 entries are replaced by the Windows system colours.
    pub fn engine_colors(&self) -> [Rgb; 256] {
        let mut c = self.colors;
        if self.format == PaletteFormat::Chunked {
            c[..8].copy_from_slice(&SYSTEM_FIRST);
            c[248..].copy_from_slice(&SYSTEM_LAST);
        }
        c
    }

    /// Indices whose stored colour differs from what `engine_colors` shows.
    pub fn overridden_differences(&self) -> Vec<usize> {
        let shown = self.engine_colors();
        (0..256).filter(|&i| shown[i] != self.colors[i]).collect()
    }
}

fn first4(bytes: &[u8]) -> [u8; 4] {
    let mut t = [0u8; 4];
    for (d, s) in t.iter_mut().zip(bytes) {
        *d = *s;
    }
    t
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

    #[test]
    fn chunked_palette() {
        let mut entries = vec![0u8; 1024];
        entries[100 * 4..100 * 4 + 3].copy_from_slice(&[1, 2, 3]);
        entries[0..3].copy_from_slice(&[9, 9, 9]);
        let mut f = b"PPAL\xff\xff\xff\xff".to_vec();
        f.extend(chunk(b"head", &1u32.to_le_bytes()));
        f.extend(chunk(b"data", &entries));
        f.extend(chunk(b"pcmt", &[0; 32]));
        let p = Palette::parse(&f).unwrap();
        assert_eq!(p.entry_count, 256);
        assert_eq!(p.colors[100], [1, 2, 3]);
        assert_eq!(p.declared_chunks, Some(1));
        assert_eq!(p.extra_chunks, vec![("pcmt".to_string(), 32)]);
        assert_eq!(p.engine_colors()[0], [0, 0, 0]);
        assert_eq!(p.overridden_differences().first(), Some(&0));
    }

    #[test]
    fn riff_palette() {
        let mut data = vec![0x00, 0x03, 2, 0];
        data.extend_from_slice(&[10, 20, 30, 0, 40, 50, 60, 0]);
        let body = chunk(b"data", &data);
        let mut f = b"RIFF".to_vec();
        f.extend_from_slice(&((4 + body.len()) as u32).to_le_bytes());
        f.extend_from_slice(b"PAL ");
        f.extend(body);
        let p = Palette::parse(&f).unwrap();
        assert_eq!(p.format, PaletteFormat::Riff);
        assert_eq!(p.entry_count, 2);
        assert_eq!(p.colors[1], [40, 50, 60]);
        assert_eq!(p.engine_colors()[0], [10, 20, 30]);
    }
}
