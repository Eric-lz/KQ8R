//! The "PERS" block that wraps every saved object (models, meshes,
//! material lists).
//!
//! Layout (libkq8fpc `kq8pers.txt`, facts only; checked against the real
//! files with `kq8 shape-verify`, see docs/provenance/README.md):
//!
//! ```text
//! "PERS"
//! u32   bits 0..30: block size; bit 31: block is padded to 4 (else to 2)
//! block:
//!   i16   class name length
//!   name  ASCII, padded to an even length
//!   class data, starting with an i32 version number
//! ```

use std::fmt;

/// One PERS block.
#[derive(Debug, Clone)]
pub struct PersBlock<'a> {
    /// Class name, e.g. `KQ8::Shape`.
    pub class: String,
    /// Class version (first 4 bytes of the class data).
    pub version: i32,
    /// Class data after the version number.
    pub data: &'a [u8],
    /// Offset of `data` in the buffer that was parsed.
    pub data_offset: usize,
    /// Offset just past the block (where the next thing starts).
    pub end: usize,
}

/// Reads the PERS block that starts at `at`.
pub fn read_block(bytes: &[u8], at: usize) -> Result<PersBlock<'_>, ReadError> {
    let mut r = Reader::new(bytes, at);
    let tag = r.bytes(4)?;
    if tag != b"PERS" {
        return Err(r.error(format!("expected 'PERS', found {:02X?}", tag)));
    }
    let raw = r.u32()?;
    let size = (raw & 0x7FFF_FFFF) as usize;
    let align = if raw & 0x8000_0000 != 0 { 4 } else { 2 };
    let start = r.pos;
    // The padding byte may be missing when the block is the last thing in
    // the data: 61 of the game's 153 `.dig` files have an odd size and end
    // exactly at size + 8 (docs/behaviour/buildings.md).
    let padded = start + size.next_multiple_of(align);
    let end = if padded > bytes.len() && start + size == bytes.len() { start + size } else { padded };
    if end > bytes.len() {
        return Err(r.error(format!(
            "PERS block size {size} runs past end of data ({} bytes left)",
            bytes.len() - start
        )));
    }
    let name_len = r.i16()?;
    if !(1..=64).contains(&name_len) {
        return Err(r.error(format!("class name length {name_len} out of range")));
    }
    let name = r.bytes((name_len as usize).next_multiple_of(2))?;
    let class = String::from_utf8_lossy(&name[..name_len as usize]).into_owned();
    let version = r.i32()?;
    let data_offset = r.pos;
    // Class data runs to the unpadded block size.
    let data_end = start + size;
    if data_offset > data_end {
        return Err(r.error("PERS block smaller than its own header".into()));
    }
    Ok(PersBlock {
        class,
        version,
        data: &bytes[data_offset..data_end],
        data_offset,
        end,
    })
}

#[derive(Debug, Clone)]
pub struct ReadError {
    /// Byte offset in the buffer being read.
    pub offset: usize,
    pub message: String,
}

impl fmt::Display for ReadError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at 0x{:X})", self.message, self.offset)
    }
}

impl std::error::Error for ReadError {}

/// Little-endian reader over a byte slice that reports where it failed.
pub struct Reader<'a> {
    pub bytes: &'a [u8],
    pub pos: usize,
}

impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8], pos: usize) -> Self {
        Reader { bytes, pos }
    }

    pub fn error(&self, message: String) -> ReadError {
        ReadError { offset: self.pos, message }
    }

    pub fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.pos)
    }

    pub fn bytes(&mut self, n: usize) -> Result<&'a [u8], ReadError> {
        let out = self.bytes.get(self.pos..self.pos + n).ok_or_else(|| {
            self.error(format!("need {n} bytes, only {} left", self.remaining()))
        })?;
        self.pos += n;
        Ok(out)
    }

    pub fn array<const N: usize>(&mut self) -> Result<[u8; N], ReadError> {
        Ok(self.bytes(N)?.try_into().unwrap())
    }

    pub fn u8(&mut self) -> Result<u8, ReadError> {
        Ok(self.array::<1>()?[0])
    }

    pub fn u16(&mut self) -> Result<u16, ReadError> {
        Ok(u16::from_le_bytes(self.array()?))
    }

    pub fn i16(&mut self) -> Result<i16, ReadError> {
        Ok(i16::from_le_bytes(self.array()?))
    }

    pub fn u32(&mut self) -> Result<u32, ReadError> {
        Ok(u32::from_le_bytes(self.array()?))
    }

    pub fn i32(&mut self) -> Result<i32, ReadError> {
        Ok(i32::from_le_bytes(self.array()?))
    }

    pub fn f32(&mut self) -> Result<f32, ReadError> {
        Ok(f32::from_le_bytes(self.array()?))
    }

    pub fn vec3(&mut self) -> Result<[f32; 3], ReadError> {
        Ok([self.f32()?, self.f32()?, self.f32()?])
    }

    /// A count field that must be non-negative and not absurdly large.
    pub fn count(&mut self, what: &str) -> Result<usize, ReadError> {
        let at = self.pos;
        let n = self.i32()?;
        if !(0..=1_000_000).contains(&n) {
            return Err(ReadError { offset: at, message: format!("{what} = {n} out of range") });
        }
        Ok(n as usize)
    }

    /// Zero-terminated text in a fixed-size field. Bytes after the zero
    /// are leftovers from the original tools and are ignored.
    pub fn fixed_str(&mut self, n: usize) -> Result<String, ReadError> {
        let raw = self.bytes(n)?;
        let len = raw.iter().position(|&b| b == 0).unwrap_or(n);
        Ok(String::from_utf8_lossy(&raw[..len]).into_owned())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    /// Builds a PERS block around `class` / `version` / `data` (padded to 2).
    pub fn block(class: &str, version: i32, data: &[u8]) -> Vec<u8> {
        let mut body = (class.len() as i16).to_le_bytes().to_vec();
        body.extend_from_slice(class.as_bytes());
        if class.len() % 2 == 1 {
            body.push(0);
        }
        body.extend_from_slice(&version.to_le_bytes());
        body.extend_from_slice(data);
        let size = body.len();
        if size % 2 == 1 {
            body.push(0);
        }
        let mut out = b"PERS".to_vec();
        out.extend_from_slice(&(size as u32).to_le_bytes());
        out.extend(body);
        out
    }

    #[test]
    fn odd_class_name_is_padded() {
        let b = block("TS::CelAnimMesh", 3, &[1, 2, 3, 4]);
        let p = read_block(&b, 0).unwrap();
        assert_eq!(p.class, "TS::CelAnimMesh");
        assert_eq!(p.version, 3);
        assert_eq!(p.data, &[1, 2, 3, 4]);
        assert_eq!(p.end, b.len());
    }

    #[test]
    fn missing_padding_at_the_end_is_accepted() {
        // Odd-sized block with no padding byte after it.
        let mut b = block("ITRGeometry", 4, &[9]);
        b.pop();
        let p = read_block(&b, 0).unwrap();
        assert_eq!((p.data, p.end), (&[9u8][..], b.len()));
    }

    #[test]
    fn rejects_other_tags() {
        assert!(read_block(b"PBMP\0\0\0\0", 0).is_err());
    }
}
