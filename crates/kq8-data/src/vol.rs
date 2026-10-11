//! VOL archives (`data/<World>/*.vol`).
//!
//! Layout (learned from Nico Bendlin's MIT-licensed `kqmoevol`, see
//! CREDITS.md, and checked against the real files):
//!
//! ```text
//! "RESLIST\0"                       8-byte signature
//! repeat until end of file:
//!   name: [u8; 20]                  ASCII, zero-terminated
//!   next: u32 little-endian         file offset of the next entry
//!   data: PKWARE DCL imploded bytes, up to `next`
//! ```

use std::fmt;
use std::fs;
use std::io;
use std::path::Path;

const SIGNATURE: &[u8; 8] = b"RESLIST\0";
const NAME_SIZE: usize = 20;
const ENTRY_HEADER_SIZE: usize = NAME_SIZE + 4;

/// One file stored in a VOL archive.
#[derive(Debug, Clone)]
pub struct VolEntry {
    /// File name exactly as stored (case preserved).
    pub name: String,
    /// Offset of this entry's header in the archive.
    pub header_offset: usize,
    /// Offset where the compressed data starts.
    pub data_offset: usize,
    /// Offset where the next entry starts (end of this entry's data).
    pub data_end: usize,
}

impl VolEntry {
    pub fn compressed_size(&self) -> usize {
        self.data_end - self.data_offset
    }
}

/// Result of decompressing one entry.
#[derive(Debug)]
pub struct Unpacked {
    pub data: Vec<u8>,
    /// Compressed bytes the decompressor actually read. Normally equals
    /// `VolEntry::compressed_size`; a difference means padding or damage.
    pub compressed_used: usize,
}

#[derive(Debug)]
pub enum VolError {
    Io(io::Error),
    BadSignature,
    /// Entry header runs past the end of the file.
    Truncated { offset: usize },
    BadName { offset: usize },
    /// `next` does not point forward to a valid place in the file.
    BadNextOffset { offset: usize, next: usize },
    Decompress { name: String, source: explode::Error },
}

impl fmt::Display for VolError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            VolError::Io(e) => write!(f, "I/O error: {e}"),
            VolError::BadSignature => write!(f, "not a VOL archive (no RESLIST signature)"),
            VolError::Truncated { offset } => {
                write!(f, "entry header at 0x{offset:08X} runs past end of file")
            }
            VolError::BadName { offset } => write!(f, "invalid entry name at 0x{offset:08X}"),
            VolError::BadNextOffset { offset, next } => {
                write!(f, "entry at 0x{offset:08X} has invalid next offset 0x{next:08X}")
            }
            VolError::Decompress { name, source } => {
                write!(f, "cannot decompress {name}: {source}")
            }
        }
    }
}

impl std::error::Error for VolError {}

impl From<io::Error> for VolError {
    fn from(e: io::Error) -> Self {
        VolError::Io(e)
    }
}

/// A VOL archive held in memory. The archive is small (largest is ~14 MB),
/// so it is read whole; entries are decompressed on request.
pub struct VolArchive {
    bytes: Vec<u8>,
    entries: Vec<VolEntry>,
}

impl VolArchive {
    pub fn open(path: &Path) -> Result<Self, VolError> {
        Self::parse(fs::read(path)?)
    }

    /// Reads the entry list. Does not decompress anything.
    pub fn parse(bytes: Vec<u8>) -> Result<Self, VolError> {
        if !bytes.starts_with(SIGNATURE) {
            return Err(VolError::BadSignature);
        }
        let mut entries = Vec::new();
        let mut offset = SIGNATURE.len();
        while offset < bytes.len() {
            let header = bytes
                .get(offset..offset + ENTRY_HEADER_SIZE)
                .ok_or(VolError::Truncated { offset })?;
            let name = parse_name(&header[..NAME_SIZE]).ok_or(VolError::BadName { offset })?;
            let next = u32::from_le_bytes(header[NAME_SIZE..].try_into().unwrap()) as usize;
            let data_offset = offset + ENTRY_HEADER_SIZE;
            if next <= data_offset || next > bytes.len() {
                return Err(VolError::BadNextOffset { offset, next });
            }
            entries.push(VolEntry {
                name,
                header_offset: offset,
                data_offset,
                data_end: next,
            });
            offset = next;
        }
        Ok(VolArchive { bytes, entries })
    }

    pub fn entries(&self) -> &[VolEntry] {
        &self.entries
    }

    /// Size of the archive file in bytes.
    pub fn len(&self) -> usize {
        self.bytes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Finds an entry by name, ignoring case (as Windows did).
    pub fn find(&self, name: &str) -> Option<&VolEntry> {
        self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }

    /// Decompresses one entry into memory.
    pub fn unpack(&self, entry: &VolEntry) -> Result<Unpacked, VolError> {
        let compressed = &self.bytes[entry.data_offset..entry.data_end];
        explode_counted(compressed).map_err(|source| VolError::Decompress {
            name: entry.name.clone(),
            source,
        })
    }
}

/// Names are zero-terminated printable ASCII. One real name contains a
/// space ("Ogre01 .BMP" in Barren/8bit.vol), so spaces are allowed.
fn parse_name(raw: &[u8]) -> Option<String> {
    let len = raw.iter().position(|&b| b == 0)?;
    let name = &raw[..len];
    if name.is_empty() || !name.iter().all(|&b| (0x20..0x7F).contains(&b)) {
        return None;
    }
    Some(String::from_utf8_lossy(name).into_owned())
}

/// Same as `explode::explode`, but also reports how many input bytes were
/// read before the end-of-stream marker.
fn explode_counted(input: &[u8]) -> Result<Unpacked, explode::Error> {
    let mut state = explode::Explode::new();
    let mut window = [0u8; 4096];
    let mut buf = state.with_buffer(&mut window);
    let mut out = Vec::new();
    let mut i = 0;
    while i < input.len() {
        match buf.feed(input[i]) {
            Ok(()) => {
                out.extend_from_slice(buf.get());
                if buf.done() {
                    return Ok(Unpacked {
                        data: out,
                        compressed_used: i + 1,
                    });
                }
                buf.reset();
            }
            // The decompressor wants the next input byte.
            Err(explode::Error::IncompleteInput) => i += 1,
            Err(e) => return Err(e),
        }
    }
    Err(explode::Error::IncompleteInput)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tiny hand-built archive with one entry holding the `explode` crate's
    // own documented example stream ("AIAIAIAIAIAIA").
    fn sample() -> Vec<u8> {
        let payload = [0x00, 0x04, 0x82, 0x24, 0x25, 0x8f, 0x80, 0x7f];
        let mut v = SIGNATURE.to_vec();
        let mut name = [0u8; NAME_SIZE];
        name[..5].copy_from_slice(b"A.TXT");
        v.extend_from_slice(&name);
        let next = (v.len() + 4 + payload.len()) as u32;
        v.extend_from_slice(&next.to_le_bytes());
        v.extend_from_slice(&payload);
        v
    }

    #[test]
    fn parses_and_unpacks() {
        let vol = VolArchive::parse(sample()).unwrap();
        assert_eq!(vol.entries().len(), 1);
        let e = vol.find("a.txt").unwrap();
        let u = vol.unpack(e).unwrap();
        assert_eq!(u.data, b"AIAIAIAIAIAIA");
        assert_eq!(u.compressed_used, e.compressed_size());
    }

    #[test]
    fn rejects_bad_signature() {
        let mut v = sample();
        v[0] = b'X';
        assert!(matches!(VolArchive::parse(v), Err(VolError::BadSignature)));
    }

    #[test]
    fn rejects_next_past_end() {
        let mut v = sample();
        let at = SIGNATURE.len() + NAME_SIZE;
        v[at..at + 4].copy_from_slice(&0xFFFF_u32.to_le_bytes());
        assert!(matches!(
            VolArchive::parse(v),
            Err(VolError::BadNextOffset { .. })
        ));
    }
}
