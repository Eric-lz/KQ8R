//! Material lists: `TS::MaterialList`, stored alone in `.dml` files
//! (terrain) and at the end of every `.kqs` model.
//!
//! Layout (libkq8fpc `kq8dml.txt`, facts only; corrected and checked
//! against the real files, see docs/provenance/README.md):
//!
//! ```text
//! PERS block, class TS::MaterialList, version 1 or 2
//!   i32 detail levels, i32 material count
//!   per material (count x levels):
//!     u32  flags (kind in bits 0..3, shading in bits 8..11, more unknown)
//!     f32  alpha, i32 palette, [u8; 4] red, green, blue, unused
//!     texture file name, 16 bytes (version 1) or 32 bytes (version 2)
//! ```

use crate::pers::{read_block, ReadError, Reader};

pub const CLASS: &str = "TS::MaterialList";

/// What a material draws.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MaterialKind {
    /// Nothing (seen in terrain lists).
    Null,
    /// A palette colour (documented as unused).
    Palette,
    /// A flat colour from `rgb`.
    Rgb,
    /// A texture from `texture`.
    Texture,
    /// Any other value of the low 4 bits.
    Other(u8),
}

#[derive(Debug, Clone)]
pub struct Material {
    /// The raw flags word. Only some bits are understood; see `kind`.
    pub flags: u32,
    pub alpha: f32,
    pub palette: i32,
    pub rgb: [u8; 3],
    /// Texture file name as stored (e.g. `wood2.BMP`); may be empty.
    pub texture: String,
}

impl Material {
    pub fn kind(&self) -> MaterialKind {
        match self.flags & 0xF {
            0 => MaterialKind::Null,
            1 => MaterialKind::Palette,
            2 => MaterialKind::Rgb,
            3 => MaterialKind::Texture,
            n => MaterialKind::Other(n as u8),
        }
    }

    /// Shading bits (0x000 null, 0x100 none, 0x200 flat, 0x400 smooth).
    pub fn shading(&self) -> u32 {
        self.flags & 0xF00
    }
}

#[derive(Debug, Clone)]
pub struct MaterialList {
    pub version: i32,
    pub detail_levels: usize,
    /// `detail_levels * count` materials.
    pub materials: Vec<Material>,
    /// Offset just past the list's PERS block.
    pub end: usize,
}

impl MaterialList {
    /// Parses a whole `.dml` file.
    pub fn parse(bytes: &[u8]) -> Result<Self, ReadError> {
        Self::read_at(bytes, 0)
    }

    /// Parses the material list PERS block starting at `at`.
    pub fn read_at(bytes: &[u8], at: usize) -> Result<Self, ReadError> {
        let block = read_block(bytes, at)?;
        if block.class != CLASS {
            return Err(ReadError {
                offset: at,
                message: format!("expected class {CLASS}, found '{}'", block.class),
            });
        }
        let name_len = match block.version {
            1 => 16,
            2 => 32,
            v => {
                return Err(ReadError {
                    offset: at,
                    message: format!("{CLASS} version {v} not supported (know 1 and 2)"),
                })
            }
        };
        let mut r = Reader::new(bytes, block.data_offset);
        let detail_levels = r.count("detail levels")?;
        let count = r.count("material count")?;
        let total = detail_levels * count;
        let mut materials = Vec::with_capacity(total);
        for _ in 0..total {
            let flags = r.u32()?;
            let alpha = r.f32()?;
            let palette = r.i32()?;
            let c = r.array::<4>()?;
            let texture = r.fixed_str(name_len)?;
            materials.push(Material { flags, alpha, palette, rgb: [c[0], c[1], c[2]], texture });
        }
        let data_end = block.data_offset + block.data.len();
        if r.pos != data_end {
            return Err(r.error(format!(
                "material list ends at 0x{:X} but its block ends at 0x{data_end:X}",
                r.pos
            )));
        }
        Ok(MaterialList { version: block.version, detail_levels, materials, end: block.end })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pers::tests::block;

    #[test]
    fn version2_list() {
        let mut data = Vec::new();
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(&1i32.to_le_bytes());
        data.extend_from_slice(&0x403u32.to_le_bytes());
        data.extend_from_slice(&[0; 8]);
        data.extend_from_slice(&[0x7F, 0x7F, 0x7F, 0]);
        let mut name = b"wood2.BMP".to_vec();
        name.resize(32, 0);
        data.extend(name);
        let f = block(CLASS, 2, &data);
        let list = MaterialList::parse(&f).unwrap();
        assert_eq!(list.materials.len(), 1);
        let m = &list.materials[0];
        assert_eq!(m.kind(), MaterialKind::Texture);
        assert_eq!(m.shading(), 0x400);
        assert_eq!(m.texture, "wood2.BMP");
        assert_eq!(m.rgb, [0x7F; 3]);
    }
}
