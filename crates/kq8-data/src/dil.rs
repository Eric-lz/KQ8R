//! Building light maps: `.dil` files, class `ITRLighting`. One small
//! light map per surface of the building's `.dig`.
//!
//! Layout and rules from the behaviour note
//! `docs/behaviour/object-lighting.md` (session A), section "`.dil`
//! layout" and rules 12-16; rule numbers below are that note's. Checked
//! with `kq8 lighting-verify`; see docs/provenance/README.md.
//!
//! After the PERS header and version (read by `pers::read_block`): build
//! id (u32, equals the `.dig`'s), light scale shift (u32), **one more u32
//! the note doesn't list** (0 in every file; measured, QUESTIONS.md Q67),
//! 5 counts (u32), then tables A-E with records of 16, 8, 24, 48 and 1
//! bytes, then one u32 (0). Tables A-C are empty in every file and are
//! skipped. Table D has one record per `.dig` surface, same order. Table E
//! holds the texels: 16-bit values, 4 bits each brightness, red, green,
//! blue (bits 12-15, 8-11, 4-7, 0-3), as the ground maps.
//!
//! Measured on all 235 files (Q67): in 5 files (`daventry/zedrock1`,
//! `zedrock2`, `deadcity/platform`, `swamp/ramp`, `temple4/level4`, the
//! ones the note says miss the size rule) table D records are **12 bytes**,
//! not 48: only the fields we use, without the floats. Nothing in the
//! header says which; the record size is worked out from the block size
//! (48 or 12, anything else is refused). 4 files have bytes after their
//! PERS block (as some `.dig` files); they are ignored.
//!
//! Also measured (Q67): ~100 surfaces of Castle D's `castle.dil` have the
//! byte offset `0xFFFFFFFF` (the only file that does). They are read as
//! "no light map" (`SurfaceLight::has_map` false); how such a wall looks
//! is not known.

use crate::building::Surface;
use crate::pers::{read_block, ReadError, Reader};

pub const CLASS: &str = "ITRLighting";

const RECORD_SIZES: [usize; 5] = [16, 8, 24, 48, 1];

/// A table D byte offset meaning "no light map" (Castle D, Q67).
pub const NO_MAP: u32 = u32::MAX;

/// Table D record sizes found in the data (Q67).
const SURFACE_RECORD_SIZES: [usize; 2] = [48, 12];

/// Where one surface's light map lies (a table D record; only the fields
/// the note says we need).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceLight {
    /// Byte offset of the texels in table E.
    pub offset: u32,
    /// Light map size in texels.
    pub width: u8,
    pub height: u8,
    /// Light map offset in texture texels (0..15), x and y.
    pub map_offset: [u8; 2],
}

impl SurfaceLight {
    /// False when the byte offset is `NO_MAP` (Q67).
    pub fn has_map(&self) -> bool {
        self.offset != NO_MAP
    }
}

#[derive(Debug, Clone)]
pub struct BuildingLight {
    pub version: i32,
    /// Equals the `.dig`'s build id (checked on all pairs by the note).
    pub build_id: u32,
    /// One light texel per 2^shift x 2^shift texture texels (4 in every
    /// file: 16 x 16).
    pub scale_shift: u32,
    /// The u32 after the shift that the note doesn't list (0 in every
    /// file, Q67).
    pub unlisted: u32,
    /// Counts of tables A-E, as stored.
    pub counts: [usize; 5],
    /// Bytes per table D record: 48, or 12 in 5 files (Q67).
    pub surface_record_size: usize,
    /// Table D: one per surface.
    pub surfaces: Vec<SurfaceLight>,
    /// Table E, raw bytes.
    texels: Vec<u8>,
    /// The u32 after table E (0 in the data).
    pub end_value: u32,
    /// Bytes after the PERS block (3 files have some).
    pub trailing_bytes: usize,
}

impl BuildingLight {
    pub fn parse(bytes: &[u8]) -> Result<Self, ReadError> {
        let block = read_block(bytes, 0)?;
        if block.class != CLASS {
            return Err(ReadError { offset: 0, message: format!("expected class {CLASS}, found '{}'", block.class) });
        }
        let mut r = Reader::new(bytes, block.data_offset);
        let build_id = r.u32()?;
        let scale_shift = r.u32()?;
        if scale_shift > 8 {
            return Err(r.error(format!("light scale shift {scale_shift} out of range")));
        }
        let unlisted = r.u32()?;
        let mut counts = [0usize; 5];
        for (i, c) in counts.iter_mut().enumerate() {
            *c = r.count(&format!("table {} count", (b'A' + i as u8) as char))?;
        }
        // Tables A-C: animated lights and light states, empty in every file.
        for k in 0..3 {
            r.bytes(counts[k] * RECORD_SIZES[k])?;
        }
        let data_end = block.data_offset + block.data.len();
        // Table D's record size from what is left: D x size + E + 4 (Q67).
        let left = data_end.saturating_sub(r.pos + counts[4] + 4);
        let surface_record_size = if counts[3] == 0 {
            RECORD_SIZES[3]
        } else {
            *SURFACE_RECORD_SIZES.iter().find(|&&n| n * counts[3] == left).ok_or_else(|| {
                r.error(format!("{left} bytes left for {} surface records: not 48 or 12 bytes each", counts[3]))
            })?
        };
        let surfaces = (0..counts[3])
            .map(|_| {
                let b = r.bytes(surface_record_size)?;
                Ok(SurfaceLight {
                    offset: u32::from_le_bytes(b[0..4].try_into().unwrap()),
                    width: b[8],
                    height: b[9],
                    map_offset: [b[10], b[11]],
                })
            })
            .collect::<Result<Vec<_>, ReadError>>()?;
        let texels = r.bytes(counts[4])?.to_vec();
        let end_value = r.u32()?;
        if r.pos != data_end {
            return Err(r.error(format!("light data ends at 0x{:X} but its block ends at 0x{data_end:X}", r.pos)));
        }
        for (i, s) in surfaces.iter().enumerate().filter(|(_, s)| s.has_map()) {
            let need = s.offset as usize + 2 * s.width as usize * s.height as usize;
            if need > texels.len() {
                return Err(ReadError {
                    offset: block.data_offset,
                    message: format!(
                        "surface {i}: light map {}x{} at byte {} runs past table E ({} bytes)",
                        s.width,
                        s.height,
                        s.offset,
                        texels.len()
                    ),
                });
            }
        }
        Ok(BuildingLight {
            version: block.version,
            build_id,
            scale_shift,
            unlisted,
            counts,
            surface_record_size,
            surfaces,
            texels,
            end_value,
            trailing_bytes: bytes.len() - block.end,
        })
    }

    /// Table E as stored.
    pub fn texel_bytes(&self) -> &[u8] {
        &self.texels
    }

    /// All of table E as 16-bit values (for histograms).
    pub fn values(&self) -> impl Iterator<Item = u16> + '_ {
        self.texels.as_chunks::<2>().0.iter().map(|&c| u16::from_le_bytes(c))
    }

    /// Texel (i, j) of surface `surface`'s light map, i along the width,
    /// row by row. `None` outside the map or when the surface has none.
    pub fn texel(&self, surface: usize, i: usize, j: usize) -> Option<u16> {
        let s = self.surfaces.get(surface)?;
        if !s.has_map() || i >= s.width as usize || j >= s.height as usize {
            return None;
        }
        let at = s.offset as usize + 2 * (j * s.width as usize + i);
        Some(u16::from_le_bytes([self.texels[at], self.texels[at + 1]]))
    }

    /// Texture texels per light texel (16 in every file).
    pub fn texels_per_light(&self) -> u32 {
        1 << self.scale_shift
    }

    /// The size check of rule 13: width = ceil((offset_x + size_x + 1) / 16) + 1,
    /// height likewise, where size is the `.dig` surface's texture size
    /// minus 1 and 16 the texels per light texel.
    pub fn fits(&self, surface: usize, dig: &Surface) -> bool {
        let Some(s) = self.surfaces.get(surface) else { return false };
        let step = self.texels_per_light() as usize;
        let want = |k: usize| (s.map_offset[k] as usize + dig.texture_size[k] as usize + 1).div_ceil(step) + 1;
        s.width as usize == want(0) && s.height as usize == want(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pers::tests::block;

    /// A `.dil` with two surfaces: 3 x 2 texels at byte 0, 2 x 2 at byte 12.
    fn sample() -> Vec<u8> {
        sample_with(48)
    }

    /// `sample()` with table D records of `record` bytes (48 or 12, Q67).
    fn sample_with(record: usize) -> Vec<u8> {
        let mut d = Vec::new();
        d.extend(77u32.to_le_bytes()); // build id
        d.extend(4u32.to_le_bytes()); // shift
        d.extend(0u32.to_le_bytes()); // the field the note doesn't list
        let texels: Vec<u16> = (0..10).map(|k| 0x1000 * (k % 16) + 0x0111 * k).collect();
        for c in [0u32, 0, 0, 2, 20] {
            d.extend(c.to_le_bytes());
        }
        for (offset, w, h, ox, oy) in [(0u32, 3u8, 2u8, 5u8, 0u8), (12, 2, 2, 0, 15)] {
            let mut rec = vec![0xEEu8; record];
            rec[0..4].copy_from_slice(&offset.to_le_bytes());
            rec[4..8].copy_from_slice(&0u32.to_le_bytes());
            rec[8..12].copy_from_slice(&[w, h, ox, oy]);
            d.extend(rec);
        }
        texels.iter().for_each(|t| d.extend(t.to_le_bytes()));
        d.extend(0u32.to_le_bytes());
        block(CLASS, 4, &d)
    }

    fn dig_surface(size: [u8; 2]) -> Surface {
        Surface {
            flags: 0,
            material: 0,
            texture_size: size,
            texture_offset: [0, 0],
            plane: 0,
            first_corner: 0,
            corner_count: 3,
            light_first: 0,
            light_count: 0,
        }
    }

    #[test]
    fn layout_and_texels() {
        let l = BuildingLight::parse(&sample()).unwrap();
        assert_eq!((l.version, l.build_id, l.scale_shift, l.unlisted, l.counts), (4, 77, 4, 0, [0, 0, 0, 2, 20]));
        assert_eq!((l.surface_record_size, l.trailing_bytes), (48, 0));
        assert_eq!(l.surfaces[1], SurfaceLight { offset: 12, width: 2, height: 2, map_offset: [0, 15] });
        assert_eq!(l.values().count(), 10);
        // Surface 0, row 1, column 2 = value 5; surface 1 starts at value 6.
        assert_eq!(l.texel(0, 2, 1), Some(0x5000 + 0x0555));
        assert_eq!(l.texel(1, 0, 0), Some(0x6000 + 0x0666));
        assert_eq!(l.texel(1, 1, 1), Some(0x9000 + 0x0999));
        assert_eq!(l.texel(0, 3, 0), None);
        assert_eq!(l.texel(2, 0, 0), None);
        assert!(l.surfaces[0].has_map());
        assert_eq!(l.end_value, 0);
    }

    #[test]
    fn size_rule() {
        let l = BuildingLight::parse(&sample()).unwrap();
        // Surface 0: offset_x 5; width 3 = ceil((5 + s + 1) / 16) + 1 for
        // s + 1 from 12 to 27; offset_y 0, height 2 for s + 1 up to 16.
        assert!(l.fits(0, &dig_surface([26, 15])));
        assert!(l.fits(0, &dig_surface([11, 0])));
        assert!(!l.fits(0, &dig_surface([27, 15])));
        assert!(!l.fits(0, &dig_surface([10, 15])));
        assert!(!l.fits(0, &dig_surface([26, 16])));
        // Surface 1: offset_y 15, so a 1-texel-high texture already needs 2.
        assert!(l.fits(1, &dig_surface([15, 0])));
        assert!(!l.fits(1, &dig_surface([15, 1])));
        assert!(!l.fits(5, &dig_surface([0, 0])));
    }

    #[test]
    fn short_surface_records() {
        // 12-byte records (5 files in the data, Q67): same fields, same texels.
        let l = BuildingLight::parse(&sample_with(12)).unwrap();
        assert_eq!(l.surface_record_size, 12);
        assert_eq!(l.surfaces[1], SurfaceLight { offset: 12, width: 2, height: 2, map_offset: [0, 15] });
        assert_eq!(l.texel(1, 1, 1), Some(0x9000 + 0x0999));
        // Any other size is refused.
        assert!(BuildingLight::parse(&sample_with(20)).is_err());
        // Bytes after the block are counted, not read.
        let mut b = sample();
        b.extend([1, 2, 3]);
        assert_eq!(BuildingLight::parse(&b).unwrap().trailing_bytes, 3);
    }

    #[test]
    fn bad_files_are_refused() {
        let good = sample();
        // Cut short.
        assert!(BuildingLight::parse(&good[..good.len() - 10]).is_err());
        // A light map running past table E.
        let mut bad = good.clone();
        let rec1 = good.len() - 4 - 20 - 48; // second table D record
        bad[rec1] = 18;
        assert!(BuildingLight::parse(&bad).is_err());
        // ... unless it is the "no light map" offset (Q67).
        bad[rec1..rec1 + 4].copy_from_slice(&NO_MAP.to_le_bytes());
        let l = BuildingLight::parse(&bad).unwrap();
        assert!(!l.surfaces[1].has_map());
        assert_eq!(l.texel(1, 0, 0), None);
        // Another class.
        assert!(BuildingLight::parse(&block("ITRGeometry", 4, &[0; 40])).is_err());
    }
}
