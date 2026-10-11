//! The tagged-chunk layout shared by `.pbm`/`.bmp` bitmaps and `.ppl`
//! palettes: a 4-byte ASCII tag, a little-endian u32 size, then `size`
//! bytes of payload.

use std::fmt;

#[derive(Debug, Clone, Copy)]
pub struct Chunk<'a> {
    pub tag: [u8; 4],
    /// Offset of the payload in the file.
    pub offset: usize,
    pub data: &'a [u8],
}

impl Chunk<'_> {
    pub fn tag_str(&self) -> String {
        tag_string(&self.tag)
    }
}

pub fn tag_string(tag: &[u8; 4]) -> String {
    tag.iter()
        .map(|&b| if b.is_ascii_graphic() || b == b' ' { b as char } else { '?' })
        .collect()
}

#[derive(Debug, Clone)]
pub struct ChunkError {
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ChunkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at 0x{:X})", self.message, self.offset)
    }
}

/// Splits `bytes[start..]` into chunks until the end of the data.
pub fn read_chunks(bytes: &[u8], start: usize) -> Result<Vec<Chunk<'_>>, ChunkError> {
    let mut chunks = Vec::new();
    let mut at = start;
    while at < bytes.len() {
        let header = bytes.get(at..at + 8).ok_or(ChunkError {
            offset: at,
            message: format!("{} stray bytes after last chunk", bytes.len() - at),
        })?;
        let tag: [u8; 4] = header[..4].try_into().unwrap();
        let size = u32_at(header, 4) as usize;
        let offset = at + 8;
        let data = bytes.get(offset..offset + size).ok_or(ChunkError {
            offset: at,
            message: format!(
                "chunk '{}' size {} runs past end of file ({} bytes left)",
                tag_string(&tag),
                size,
                bytes.len() - offset
            ),
        })?;
        chunks.push(Chunk { tag, offset, data });
        at = offset + size;
    }
    Ok(chunks)
}

pub fn u32_at(bytes: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap())
}

pub fn u16_at(bytes: &[u8], at: usize) -> u16 {
    u16::from_le_bytes(bytes[at..at + 2].try_into().unwrap())
}
