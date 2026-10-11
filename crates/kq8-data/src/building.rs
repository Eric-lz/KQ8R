//! Buildings: `.dig` files, class `ITRGeometry` (the shape of a
//! `KQInterior` object). Lighting (`.dil`) is not read yet.
//!
//! Layout and drawing rules from the behaviour note
//! `docs/behaviour/buildings.md` (session A, answers Q11), checked with
//! `kq8 building-verify`; see docs/provenance/README.md.
//!
//! After the PERS header and version (read by `pers::read_block`):
//! build id (u32), texture scale (f32), bounding box (6 x f32), 8 counts
//! (i32), then 8 tables in that order with records of 20, 8, 48, 1, 4, 12,
//! 8 and 16 bytes, then one u32 if version > 3 and one more if version 4.
//! Tables used here: 1 surfaces, 5 corners, 6 points, 7 texture
//! coordinates, 8 planes. Tables 2-4 (visibility: BSP nodes, leaves,
//! bits) are skipped.

use crate::pers::{read_block, ReadError, Reader};

pub const CLASS: &str = "ITRGeometry";

const RECORD_SIZES: [usize; 8] = [20, 8, 48, 1, 4, 12, 8, 16];

/// One flat polygon.
#[derive(Debug, Clone, Copy)]
pub struct Surface {
    /// Bit 0x80: the surface faces the opposite way to its plane.
    pub flags: u8,
    /// Index into the building's material list.
    pub material: u8,
    /// Texture size in texels minus 1 (x, y).
    pub texture_size: [u8; 2],
    /// Texture offset in texels (x, y).
    pub texture_offset: [u8; 2],
    pub plane: u16,
    pub first_corner: u32,
    pub corner_count: u8,
    /// Lighting data reference (probably into `.dil`; not used yet).
    pub light_first: u32,
    pub light_count: u8,
}

impl Surface {
    pub fn flipped(&self) -> bool {
        self.flags & 0x80 != 0
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Corner {
    pub point: u16,
    pub texture_coord: u16,
}

#[derive(Debug, Clone, Copy)]
pub struct Plane {
    pub normal: [f32; 3],
    pub d: f32,
}

#[derive(Debug, Clone)]
pub struct Building {
    pub version: i32,
    /// Texels per unit (0.125 = 1 texel per 8 units in Daventry).
    pub texture_scale: f32,
    pub bounds_min: [f32; 3],
    pub bounds_max: [f32; 3],
    /// Counts of all 8 tables, as stored.
    pub counts: [usize; 8],
    pub surfaces: Vec<Surface>,
    pub corners: Vec<Corner>,
    pub points: Vec<[f32; 3]>,
    pub texture_coords: Vec<[f32; 2]>,
    pub planes: Vec<Plane>,
    /// Bytes after the PERS block (the note: 5 of 153 files have some).
    pub trailing_bytes: usize,
}

/// One triangle ready to draw, in the building's own space, corners in
/// the surface's own order. Seen from the visible side that order is
/// **clockwise**, as for models: measured with `kq8 building-verify` (outer
/// walls and roofs) and seen by eye; buildings.md rule 2 says the opposite
/// (QUESTIONS.md Q15).
#[derive(Debug, Clone, Copy)]
pub struct BuildingTriangle {
    pub positions: [[f32; 3]; 3],
    /// In texels; divide by the texture's width/height to get 0..1
    /// coordinates (likely, not traced: buildings.md rule 3).
    pub texels: [[f32; 2]; 3],
    pub material: u8,
    pub surface: usize,
}

impl Building {
    pub fn parse(bytes: &[u8]) -> Result<Self, ReadError> {
        let block = read_block(bytes, 0)?;
        if block.class != CLASS {
            return Err(ReadError { offset: 0, message: format!("expected class {CLASS}, found '{}'", block.class) });
        }
        let version = block.version;
        let mut r = Reader::new(bytes, block.data_offset);
        let _build_id = r.u32()?;
        let texture_scale = r.f32()?;
        let bounds_min = r.vec3()?;
        let bounds_max = r.vec3()?;
        let mut counts = [0usize; 8];
        for (i, c) in counts.iter_mut().enumerate() {
            *c = r.count(&format!("table {} count", i + 1))?;
        }
        let mut surfaces = Vec::with_capacity(counts[0]);
        for _ in 0..counts[0] {
            let b = r.bytes(20)?;
            surfaces.push(Surface {
                flags: b[0],
                material: b[1],
                texture_size: [b[2], b[3]],
                texture_offset: [b[4], b[5]],
                plane: u16::from_le_bytes([b[6], b[7]]),
                first_corner: u32::from_le_bytes(b[8..12].try_into().unwrap()),
                light_first: u32::from_le_bytes(b[12..16].try_into().unwrap()),
                corner_count: b[16],
                light_count: b[17],
            });
        }
        // Tables 2-4: visibility data, not needed to draw.
        for (i, size) in RECORD_SIZES.iter().enumerate().take(4).skip(1) {
            r.bytes(counts[i] * size)?;
        }
        let corners = (0..counts[4])
            .map(|_| Ok(Corner { point: r.u16()?, texture_coord: r.u16()? }))
            .collect::<Result<Vec<_>, ReadError>>()?;
        let points = (0..counts[5]).map(|_| r.vec3()).collect::<Result<Vec<_>, _>>()?;
        let texture_coords = (0..counts[6]).map(|_| Ok([r.f32()?, r.f32()?])).collect::<Result<Vec<_>, ReadError>>()?;
        let planes = (0..counts[7])
            .map(|_| Ok(Plane { normal: r.vec3()?, d: r.f32()? }))
            .collect::<Result<Vec<_>, ReadError>>()?;
        if version > 3 {
            r.u32()?;
        }
        if version == 4 {
            r.u32()?;
        }
        let data_end = block.data_offset + block.data.len();
        if r.pos != data_end {
            return Err(r.error(format!("building data ends at 0x{:X} but its block ends at 0x{data_end:X}", r.pos)));
        }
        Ok(Building {
            version,
            texture_scale,
            bounds_min,
            bounds_max,
            counts,
            surfaces,
            corners,
            points,
            texture_coords,
            planes,
            trailing_bytes: bytes.len() - block.end,
        })
    }

    /// Corner positions and texel positions of surface `i`, in order.
    /// `None` if an index is out of range.
    pub fn surface_corners(&self, i: usize) -> Option<Vec<([f32; 3], [f32; 2])>> {
        let s = self.surfaces.get(i)?;
        let first = s.first_corner as usize;
        (first..first + s.corner_count as usize)
            .map(|c| {
                let corner = self.corners.get(c)?;
                let p = *self.points.get(corner.point as usize)?;
                let uv = *self.texture_coords.get(corner.texture_coord as usize)?;
                // buildings.md rule 3: texel = offset + uv * (size + 1)
                let texel = std::array::from_fn(|k| s.texture_offset[k] as f32 + uv[k] * (s.texture_size[k] as f32 + 1.0));
                Some((p, texel))
            })
            .collect()
    }

    /// Normal of surface `i` by the right-hand rule on its corner order
    /// (points towards the side from which the corners look anticlockwise,
    /// i.e. away from the visible side), not normalised. `None` for bad indices or fewer than 3 corners.
    pub fn corner_order_normal(&self, i: usize) -> Option<[f32; 3]> {
        let c = self.surface_corners(i)?;
        if c.len() < 3 {
            return None;
        }
        // Newell's method: robust for any flat polygon.
        let mut n = [0.0f32; 3];
        for k in 0..c.len() {
            let (a, b) = (c[k].0, c[(k + 1) % c.len()].0);
            n[0] += (a[1] - b[1]) * (a[2] + b[2]);
            n[1] += (a[2] - b[2]) * (a[0] + b[0]);
            n[2] += (a[0] - b[0]) * (a[1] + b[1]);
        }
        Some(n)
    }

    /// All surfaces as triangles (fan from the first corner), corners in
    /// the surface's order (clockwise from the visible side). Surfaces with bad indices are
    /// counted in the second value and left out.
    pub fn triangles(&self) -> (Vec<BuildingTriangle>, usize) {
        let mut out = Vec::new();
        let mut bad = 0;
        for (i, s) in self.surfaces.iter().enumerate() {
            let Some(c) = self.surface_corners(i) else {
                bad += 1;
                continue;
            };
            for k in 1..c.len().saturating_sub(1) {
                let tri = [c[0], c[k], c[k + 1]];
                out.push(BuildingTriangle {
                    positions: tri.map(|t| t.0),
                    texels: tri.map(|t| t.1),
                    material: s.material,
                    surface: i,
                });
            }
        }
        (out, bad)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pers::tests::block;

    /// A building with one square surface (4 corners) in the z = 0 plane,
    /// corners anticlockwise seen from above.
    fn square() -> Vec<u8> {
        let mut d = Vec::new();
        d.extend(7u32.to_le_bytes()); // build id
        d.extend(0.125f32.to_le_bytes());
        for v in [0.0f32, 0.0, 0.0, 8.0, 8.0, 0.0] {
            d.extend(v.to_le_bytes());
        }
        for c in [1i32, 0, 0, 0, 4, 4, 4, 1] {
            d.extend(c.to_le_bytes());
        }
        // surface: flags 0, material 2, size 15x15, offset 1,0, plane 0,
        // first corner 0, light 0, 4 corners
        d.extend([0, 2, 15, 15, 1, 0]);
        d.extend(0u16.to_le_bytes());
        d.extend(0u32.to_le_bytes());
        d.extend(0u32.to_le_bytes());
        d.extend([4, 0, 0, 0]);
        for k in 0u16..4 {
            d.extend(k.to_le_bytes());
            d.extend(k.to_le_bytes());
        }
        for p in [[0.0f32, 0.0, 0.0], [8.0, 0.0, 0.0], [8.0, 8.0, 0.0], [0.0, 8.0, 0.0]] {
            p.iter().for_each(|v| d.extend(v.to_le_bytes()));
        }
        for uv in [[0.0f32, 0.0], [1.0, 0.0], [1.0, 1.0], [0.0, 1.0]] {
            uv.iter().for_each(|v| d.extend(v.to_le_bytes()));
        }
        for v in [0.0f32, 0.0, 1.0, 0.0] {
            d.extend(v.to_le_bytes());
        }
        d.extend(0u32.to_le_bytes());
        d.extend(1u32.to_le_bytes());
        block(CLASS, 4, &d)
    }

    #[test]
    fn parses_and_triangulates() {
        let b = Building::parse(&square()).unwrap();
        assert_eq!(b.counts, [1, 0, 0, 0, 4, 4, 4, 1]);
        assert_eq!(b.texture_scale, 0.125);
        let (tris, bad) = b.triangles();
        assert_eq!((tris.len(), bad), (2, 0));
        // texel = offset + uv * (size + 1): corner 2 -> (1 + 16, 0 + 16)
        assert_eq!(tris[0].texels[2], [17.0, 16.0]);
        assert_eq!(tris[0].material, 2);
        // Anticlockwise from above: corner-order normal points up (+z),
        // the same way as the plane.
        let n = b.corner_order_normal(0).unwrap();
        assert!(n[2] > 0.0 && b.planes[0].normal[2] > 0.0);
    }

    #[test]
    fn bad_corner_index_is_counted_not_fatal() {
        let mut b = Building::parse(&square()).unwrap();
        b.surfaces[0].first_corner = 99;
        let (tris, bad) = b.triangles();
        assert!(tris.is_empty());
        assert_eq!(bad, 1);
    }
}
