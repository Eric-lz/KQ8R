//! Outdoor ground (`KQTerrain`): settings from `TERRAIN.KQ`, the height
//! and material images, the material direction table, and the height
//! lookup.
//!
//! Rules from the behaviour note `docs/behaviour/terrain.md` (session A);
//! file layouts observed in the data. See docs/provenance/README.md.
//!
//! - The height image has one grid point per pixel; 129 x 129 points make
//!   128 x 128 squares. A square is 2^groundScale units wide.
//! - Pixel column i is at x = i * square; pixel row j, counted from the
//!   bottom row of the image, is at y = j * square. Grid corner at (0, 0).
//! - height = pixel value * heightBMPScale.
//! - Each square is two flat triangles; the diagonal alternates like a
//!   chessboard (see `Terrain::height_at`).

use crate::kq_text::Section;
use crate::Bitmap;

/// Settings from `TERRAIN.KQ`, with the defaults the behaviour note gives
/// for missing ones.
#[derive(Debug, Clone)]
pub struct TerrainSettings {
    pub ground_scale: u32,
    pub height_scale: f32,
    pub height_image: String,
    /// `floorBMPName`; purpose unknown (terrain.md open question 2).
    pub floor_image: Option<String>,
    pub material_image: String,
    pub material_list: String,
    /// `materialDirection`, e.g. `terrain/daventry.bin`.
    pub material_direction: Option<String>,
    /// Every entry as written, for logging.
    pub all: Vec<(String, String)>,
}

impl TerrainSettings {
    pub fn from_section(s: &Section) -> Result<Self, String> {
        let text = |k: &str, default: &str| s.get(k).filter(|v| !v.is_empty()).unwrap_or(default).to_string();
        let ground_scale = match s.number("groundScale") {
            None => 12.0,
            Some(r) => r?,
        };
        if !(0.0..=20.0).contains(&ground_scale) || ground_scale.fract() != 0.0 {
            return Err(format!("groundScale={ground_scale} is not a whole number 0..20"));
        }
        let height_scale = match s.number("heightBMPScale") {
            None => 60.0,
            Some(r) => r?,
        };
        Ok(TerrainSettings {
            ground_scale: ground_scale as u32,
            height_scale: height_scale as f32,
            height_image: text("heightBMP", "terrh.dib"),
            floor_image: s.get("floorBMPName").filter(|v| !v.is_empty()).map(str::to_string),
            material_image: text("materialBMP", "terrm.dib"),
            material_list: text("materialListFile", "terrain.dml"),
            material_direction: s.get("materialDirection").filter(|v| !v.is_empty()).map(str::to_string),
            all: s.entries.clone(),
        })
    }

    /// Width of one grid square in game units.
    pub fn square_size(&self) -> f32 {
        (1u32 << self.ground_scale) as f32
    }
}

/// An 8-bit image as a grid indexed from the bottom row up:
/// `get(i, j)` = column i, row j counted from the bottom of the image.
#[derive(Debug, Clone)]
pub struct Grid {
    pub width: usize,
    pub height: usize,
    /// Row 0 = bottom row of the image.
    values: Vec<u8>,
}

impl Grid {
    /// From a parsed 8-bit bitmap (whose pixels are stored top row first).
    pub fn from_bitmap(b: &Bitmap) -> Result<Self, String> {
        if b.bit_count != 8 {
            return Err(format!("expected an 8-bit image, got {}-bit", b.bit_count));
        }
        let (w, h) = (b.width as usize, b.height as usize);
        let values = (0..h).rev().flat_map(|row| b.pixels[row * w..(row + 1) * w].iter().copied()).collect();
        Ok(Grid { width: w, height: h, values })
    }

    pub fn get(&self, i: usize, j: usize) -> Option<u8> {
        (i < self.width && j < self.height).then(|| self.values[j * self.width + i])
    }

    pub fn values(&self) -> &[u8] {
        &self.values
    }
}

/// One record of the material direction table (`terrain/<world>.bin`).
/// Layout observed: u32 count, then per record u32 index, 16-byte texture
/// name, u32 (always 0 in Daventry), u32 (0..7, probably the texture's
/// direction; meaning open, QUESTIONS.md Q8).
#[derive(Debug, Clone)]
pub struct DirectionRecord {
    pub index: u32,
    pub texture: String,
    pub unknown: u32,
    pub direction: u32,
}

pub fn parse_directions(bytes: &[u8]) -> Result<Vec<DirectionRecord>, String> {
    let mut r = crate::pers::Reader::new(bytes, 0);
    let count = r.count("direction record count").map_err(|e| e.to_string())?;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let rec = (|| -> Result<_, crate::pers::ReadError> {
            Ok(DirectionRecord { index: r.u32()?, texture: r.fixed_str(16)?, unknown: r.u32()?, direction: r.u32()? })
        })()
        .map_err(|e| e.to_string())?;
        out.push(rec);
    }
    if r.remaining() != 0 {
        return Err(format!("{} bytes left after {count} direction records", r.remaining()));
    }
    Ok(out)
}

/// Result of a height query.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ground {
    pub height: f32,
    /// Unit normal of the triangle the point is on (z > 0).
    pub normal: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Terrain {
    pub settings: TerrainSettings,
    pub heights: Grid,
}

impl Terrain {
    pub fn new(settings: TerrainSettings, heights: Grid) -> Result<Self, String> {
        if heights.width < 2 || heights.height < 2 {
            return Err(format!("height grid {}x{} is too small", heights.width, heights.height));
        }
        Ok(Terrain { settings, heights })
    }

    /// Number of squares along x and y.
    pub fn squares(&self) -> (usize, usize) {
        (self.heights.width - 1, self.heights.height - 1)
    }

    /// Height of grid point (i, j) in game units.
    pub fn point_height(&self, i: usize, j: usize) -> Option<f32> {
        self.heights.get(i, j).map(|v| v as f32 * self.settings.height_scale)
    }

    /// True if square (i, j)'s diagonal runs from its (0, 0) corner to
    /// its (1, 1) corner; false if from (1, 0) to (0, 1).
    pub fn diagonal_is_rising(i: usize, j: usize) -> bool {
        (i + j).is_multiple_of(2)
    }

    /// Ground under (x, y), or `None` outside the grid. Holes are not
    /// known yet (terrain.md open question 3), so there are none.
    pub fn height_at(&self, x: f32, y: f32) -> Option<Ground> {
        let size = self.settings.square_size();
        let (fx, fy) = (x / size, y / size);
        if !(fx >= 0.0 && fy >= 0.0) {
            return None;
        }
        let (nx, ny) = self.squares();
        // A point on the far edge belongs to the last square.
        let i = (fx.floor() as usize).min(nx - 1);
        let j = (fy.floor() as usize).min(ny - 1);
        if fx > nx as f32 || fy > ny as f32 {
            return None;
        }
        let (u, v) = (fx - i as f32, fy - j as f32);
        let h = |di, dj| self.point_height(i + di, j + dj).unwrap();
        let (h00, h10, h01, h11) = (h(0, 0), h(1, 0), h(0, 1), h(1, 1));
        // Height on the triangle's plane as h = base + du * u + dv * v.
        let (base, du, dv) = if Self::diagonal_is_rising(i, j) {
            if u >= v {
                // corners (0,0) (1,0) (1,1)
                (h00, h10 - h00, h11 - h10)
            } else {
                // corners (0,0) (1,1) (0,1)
                (h00, h11 - h01, h01 - h00)
            }
        } else if u + v <= 1.0 {
            // corners (0,0) (1,0) (0,1)
            (h00, h10 - h00, h01 - h00)
        } else {
            // corners (1,0) (1,1) (0,1): plane through them
            (h10 + h01 - h11, h11 - h01, h11 - h10)
        };
        let height = base + du * u + dv * v;
        let (sx, sy) = (du / size, dv / size);
        let len = (sx * sx + sy * sy + 1.0).sqrt();
        Some(Ground { height, normal: [-sx / len, -sy / len, 1.0 / len] })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain(points: [[u8; 2]; 2], scale: f32) -> Terrain {
        let settings = TerrainSettings {
            ground_scale: 2,
            height_scale: scale,
            height_image: String::new(),
            floor_image: None,
            material_image: String::new(),
            material_list: String::new(),
            material_direction: None,
            all: Vec::new(),
        };
        // points[j][i], j = 0 is the bottom row
        let heights = Grid { width: 2, height: 2, values: vec![points[0][0], points[0][1], points[1][0], points[1][1]] };
        Terrain::new(settings, heights).unwrap()
    }

    #[test]
    fn bitmap_rows_are_flipped_to_bottom_up() {
        let b = Bitmap {
            format: crate::BitmapFormat::WindowsBmp,
            width: 1,
            height: 2,
            bit_count: 8,
            flags: 0,
            mip_count: 1,
            declared_chunks: None,
            chunk_tags: vec![],
            other_chunks: vec![],
            embedded_palette: None,
            data_padding: 0,
            pixels: vec![7, 9], // top row first
        };
        let g = Grid::from_bitmap(&b).unwrap();
        assert_eq!(g.get(0, 0), Some(9));
        assert_eq!(g.get(0, 1), Some(7));
    }

    #[test]
    fn corners_and_scale() {
        // square size 4; heights 1, 2 (bottom row), 3, 4 (top row), x10
        let t = terrain([[1, 2], [3, 4]], 10.0);
        assert_eq!(t.height_at(0.0, 0.0).unwrap().height, 10.0);
        assert_eq!(t.height_at(4.0, 0.0).unwrap().height, 20.0);
        assert_eq!(t.height_at(0.0, 4.0).unwrap().height, 30.0);
        assert_eq!(t.height_at(4.0, 4.0).unwrap().height, 40.0);
        assert!(t.height_at(-0.1, 1.0).is_none());
        assert!(t.height_at(1.0, 4.1).is_none());
    }

    #[test]
    fn rising_diagonal_on_even_square() {
        // Square (0,0) is even: diagonal (0,0)-(1,1). Only corner (1,0)
        // is raised; the point (1,3) is on the upper-left triangle
        // (0,0),(1,1),(0,1), which does not include (1,0), so it is flat.
        let t = terrain([[0, 8], [0, 0]], 1.0);
        assert_eq!(t.height_at(1.0, 3.0).unwrap().height, 0.0);
        // and on the other triangle it rises toward (1,0)
        assert_eq!(t.height_at(3.0, 1.0).unwrap().height, 4.0);
        let n = t.height_at(3.0, 1.0).unwrap().normal;
        assert!(n[2] > 0.0 && n[0] < 0.0);
    }

    #[test]
    fn falling_diagonal_on_odd_square_matches_plane() {
        // Use a 3x2 grid so square (1,0) is odd: diagonal (1,0)-(0,1) of
        // that square. Raise only its (1,1) corner (grid point (2,1)).
        let mut t = terrain([[0, 0], [0, 0]], 1.0);
        t.heights = Grid { width: 3, height: 2, values: vec![0, 0, 0, 0, 0, 8] };
        // Point in square (1,0) near its (0,0) corner: triangle without
        // the raised corner, so flat.
        assert_eq!(t.height_at(4.5, 0.5).unwrap().height, 0.0);
        // Near its (1,1) corner: plane through (1,0)=0, (1,1)=8, (0,1)=0.
        // At local (0.75, 0.75): 0 + 8*(0.75+0.75-1) = 4.
        assert_eq!(t.height_at(7.0, 3.0).unwrap().height, 4.0);
    }
}
