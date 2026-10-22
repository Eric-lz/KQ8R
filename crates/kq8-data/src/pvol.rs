//! Saved games (`MASK/save/save<n>.vol`): a `PVOL` archive (M6 6d-1, to
//! check the magic map's saved fog, `magic-map.md` rule 16). Read only.
//!
//! Layout as the Tribes community describes `PVOL` (docs/RESEARCH.md,
//! "Engine lineage") and as KQ8's saves show it (checked on all of the
//! user's saves by `kq8 map-verify`):
//!
//! ```text
//! "PVOL", u32 offset of the name table
//! files, each "VBLK", u32 (bits 0..23 size, top byte 0x80), data, padded to 4
//! "vols", u32 size, file names each ending with a zero byte
//! (padding to an even offset: save2's table ends at an odd one)
//! "voli", u32 size, 17-byte entries:
//!   u32 (not used here), u32 name offset in the table,
//!   u32 offset of the file's "VBLK", u32 size, u8 compression (0 = none)
//! ```

/// One file in the archive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PvolEntry {
    pub name: String,
    /// Offset of its `VBLK` header.
    pub at: usize,
    pub size: usize,
    pub compression: u8,
}

/// A saved game's archive.
#[derive(Debug, Clone)]
pub struct Pvol {
    pub bytes: Vec<u8>,
    pub entries: Vec<PvolEntry>,
}

fn u32_at(b: &[u8], at: usize) -> Result<u32, String> {
    b.get(at..at + 4).map(|s| u32::from_le_bytes(s.try_into().unwrap())).ok_or_else(|| format!("short file at {at}"))
}

impl Pvol {
    pub fn parse(bytes: Vec<u8>) -> Result<Self, String> {
        if !bytes.starts_with(b"PVOL") {
            return Err("not a PVOL archive".into());
        }
        let names_at = u32_at(&bytes, 4)? as usize;
        if bytes.get(names_at..names_at + 4) != Some(b"vols") {
            return Err(format!("no 'vols' table at {names_at}"));
        }
        let names_size = u32_at(&bytes, names_at + 4)? as usize;
        let names = bytes.get(names_at + 8..names_at + 8 + names_size).ok_or("name table runs past the end")?;
        let names_end = names_at + 8 + names_size;
        let index_at = (names_end..names_end + 4).find(|&at| bytes.get(at..at + 4) == Some(b"voli")).ok_or_else(|| format!("no 'voli' index after the name table ({names_end})"))?;
        let index_size = u32_at(&bytes, index_at + 4)? as usize;
        let index = bytes.get(index_at + 8..index_at + 8 + index_size).ok_or("index runs past the end")?;
        let mut entries = Vec::new();
        for e in index.as_chunks::<17>().0 {
            let name_at = u32_at(e, 4)? as usize;
            let end = names.get(name_at..).and_then(|n| n.iter().position(|c| *c == 0)).ok_or_else(|| format!("bad name offset {name_at}"))?;
            let name: String = names[name_at..name_at + end].iter().map(|&c| c as char).collect();
            let at = u32_at(e, 8)? as usize;
            let size = u32_at(e, 12)? as usize;
            if bytes.get(at..at + 4) != Some(b"VBLK") {
                return Err(format!("{name}: no VBLK at {at}"));
            }
            entries.push(PvolEntry { name, at, size, compression: e[16] });
        }
        Ok(Pvol { bytes, entries })
    }

    /// A file's bytes (uncompressed files only).
    pub fn data(&self, e: &PvolEntry) -> Result<&[u8], String> {
        if e.compression != 0 {
            return Err(format!("{}: compression {} not read", e.name, e.compression));
        }
        self.bytes.get(e.at + 8..e.at + 8 + e.size).ok_or_else(|| format!("{}: runs past the end", e.name))
    }

    /// The first file called `name` (any case).
    pub fn find(&self, name: &str) -> Option<&PvolEntry> {
        self.entries.iter().find(|e| e.name.eq_ignore_ascii_case(name))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_small_archive() {
        let mut b = b"PVOL\0\0\0\0".to_vec();
        b.extend_from_slice(b"VBLK\x05\0\0\x80hello\0\0\0");
        let names_at = b.len() as u32;
        b[4..8].copy_from_slice(&names_at.to_le_bytes());
        b.extend_from_slice(b"vols\x06\0\0\0a.txt\0");
        b.extend_from_slice(&[0, 0]);
        b.extend_from_slice(b"voli\x11\0\0\0");
        b.extend_from_slice(&[0, 0, 0, 0]);
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&8u32.to_le_bytes());
        b.extend_from_slice(&5u32.to_le_bytes());
        b.push(0);
        let v = Pvol::parse(b).unwrap();
        assert_eq!(v.entries, vec![PvolEntry { name: "a.txt".into(), at: 8, size: 5, compression: 0 }]);
        assert_eq!(v.data(v.find("A.TXT").unwrap()).unwrap(), b"hello");
        assert!(Pvol::parse(b"RESLIST\0".to_vec()).is_err());
    }
}
