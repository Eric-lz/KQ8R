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
//! - A world with `floorBMPName` also has a floor grid (the ground under
//!   water) built the same way; without one, floor questions get the
//!   drawn grid (water.md "Two ground grids").
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
    /// `floorBMPName`: the floor grid's image, the ground under water
    /// (water.md).
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

    /// From values listed bottom row first (`values[j * width + i]`).
    pub fn from_rows(width: usize, height: usize, values: Vec<u8>) -> Result<Self, String> {
        if values.len() != width * height {
            return Err(format!("{} values for a {width}x{height} grid", values.len()));
        }
        Ok(Grid { width, height, values })
    }

    pub fn get(&self, i: usize, j: usize) -> Option<u8> {
        (i < self.width && j < self.height).then(|| self.values[j * self.width + i])
    }

    /// Changes one value; false (nothing changed) outside the grid.
    pub fn set(&mut self, i: usize, j: usize, v: u8) -> bool {
        let inside = i < self.width && j < self.height;
        if inside {
            self.values[j * self.width + i] = v;
        }
        inside
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

/// Which of the two height grids (water.md "Which grid answers").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    /// The ground under water (`floorBMPName`); the surface where a world
    /// has no floor image.
    Floor,
    /// The drawn ground (`heightBMP`): the water's surface where there is
    /// water.
    Surface,
}

#[derive(Debug, Clone)]
pub struct Terrain {
    pub settings: TerrainSettings,
    /// The surface grid (`heightBMP`), the one that is drawn.
    pub heights: Grid,
    /// The floor grid (`floorBMPName`), if the world has one.
    pub floor: Option<Grid>,
    /// Ground flags per square (`envInfo.kq` by the square's material,
    /// water.md), row by row from the bottom; empty if not read.
    pub square_flags: Vec<u32>,
}

impl Terrain {
    pub fn new(settings: TerrainSettings, heights: Grid) -> Result<Self, String> {
        if heights.width < 2 || heights.height < 2 {
            return Err(format!("height grid {}x{} is too small", heights.width, heights.height));
        }
        Ok(Terrain { settings, heights, floor: None, square_flags: Vec::new() })
    }

    /// Adds the floor grid; it must be the same size as the surface grid.
    pub fn with_floor(mut self, floor: Grid) -> Result<Self, String> {
        if (floor.width, floor.height) != (self.heights.width, self.heights.height) {
            return Err(format!(
                "floor grid {}x{} differs from the height grid {}x{}",
                floor.width, floor.height, self.heights.width, self.heights.height
            ));
        }
        self.floor = Some(floor);
        Ok(self)
    }

    /// Adds the ground flags: `materials` (`terrm.dib`, one value per
    /// square) looked up in `flags` (per material number; 0 past its end).
    pub fn with_ground_flags(mut self, materials: &Grid, flags: &[u32]) -> Result<Self, String> {
        let (nx, ny) = self.squares();
        if (materials.width, materials.height) != (nx, ny) {
            return Err(format!("material image is {}x{}, ground has {nx}x{ny} squares", materials.width, materials.height));
        }
        self.square_flags = materials.values().iter().map(|&m| flags.get(m as usize).copied().unwrap_or(0)).collect();
        Ok(self)
    }

    /// Ground flags of the square (x, y) is in; 0 outside the grid or
    /// without flags.
    pub fn flags_at(&self, x: f32, y: f32) -> u32 {
        let (nx, _) = self.squares();
        self.square_of(x, y).and_then(|(i, j)| self.square_flags.get(j * nx + i).copied()).unwrap_or(0)
    }

    /// The square (i, j) that (x, y) is in, `None` outside the grid. A point
    /// on the far edge belongs to the last square.
    fn square_of(&self, x: f32, y: f32) -> Option<(usize, usize)> {
        let size = self.settings.square_size();
        let (fx, fy) = (x / size, y / size);
        let (nx, ny) = self.squares();
        if !(fx >= 0.0 && fy >= 0.0) || fx > nx as f32 || fy > ny as f32 {
            return None;
        }
        Some(((fx.floor() as usize).min(nx - 1), (fy.floor() as usize).min(ny - 1)))
    }

    fn grid(&self, level: Level) -> &Grid {
        match level {
            Level::Floor => self.floor.as_ref().unwrap_or(&self.heights),
            Level::Surface => &self.heights,
        }
    }

    /// Number of squares along x and y.
    pub fn squares(&self) -> (usize, usize) {
        (self.heights.width - 1, self.heights.height - 1)
    }

    /// Height of grid point (i, j) in game units (the surface).
    pub fn point_height(&self, i: usize, j: usize) -> Option<f32> {
        self.point_height_on(Level::Surface, i, j)
    }

    /// Height of grid point (i, j) on one of the two grids.
    pub fn point_height_on(&self, level: Level, i: usize, j: usize) -> Option<f32> {
        self.grid(level).get(i, j).map(|v| v as f32 * self.settings.height_scale)
    }

    /// True if square (i, j)'s diagonal runs from its (0, 0) corner to
    /// its (1, 1) corner; false if from (1, 0) to (0, 1).
    pub fn diagonal_is_rising(i: usize, j: usize) -> bool {
        (i + j).is_multiple_of(2)
    }

    /// The drawn ground (surface) under (x, y), or `None` outside the
    /// grid. Holes are not known yet (terrain.md open question 3), so
    /// there are none.
    pub fn height_at(&self, x: f32, y: f32) -> Option<Ground> {
        self.ground_at(Level::Surface, x, y)
    }

    /// The floor under (x, y): what Connor stands on (water.md).
    pub fn floor_at(&self, x: f32, y: f32) -> Option<Ground> {
        self.ground_at(Level::Floor, x, y)
    }

    /// Ground under (x, y) on one of the two grids.
    pub fn ground_at(&self, level: Level, x: f32, y: f32) -> Option<Ground> {
        let size = self.settings.square_size();
        let (i, j) = self.square_of(x, y)?;
        let (u, v) = (x / size - i as f32, y / size - j as f32);
        let h = |di, dj| self.point_height_on(level, i + di, j + dj).unwrap();
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

    /// The ground as triangles, two per square, using the same diagonals
    /// as `height_at`. Corners are listed **clockwise seen from above**,
    /// the game's front-face rule (models-and-textures.md, Q2), so a
    /// renderer can treat ground and models the same way.
    ///
    /// `materials` gives one material number per square (`terrm.dib`);
    /// `orientation[m]` is the direction-table code (0..7) of material m,
    /// 0 if the table is shorter.
    pub fn ground_triangles(&self, materials: &Grid, orientation: &[u32]) -> Result<Vec<GroundTriangle>, String> {
        let (nx, ny) = self.squares();
        if (materials.width, materials.height) != (nx, ny) {
            return Err(format!(
                "material image is {}x{}, ground has {nx}x{ny} squares",
                materials.width, materials.height
            ));
        }
        let mut out = Vec::with_capacity(nx * ny * 2);
        for j in 0..ny {
            for i in 0..nx {
                let material = materials.get(i, j).unwrap();
                let code = orientation.get(material as usize).copied().unwrap_or(0);
                out.extend(self.square_triangles(i, j, material, code));
            }
        }
        Ok(out)
    }

    /// The two triangles of square (i, j) showing `material` with
    /// orientation code `code` (also used when a ground animator changes a
    /// square's material). Empty outside the grid.
    pub fn square_triangles(&self, i: usize, j: usize, material: u8, code: u32) -> Vec<GroundTriangle> {
        let (nx, ny) = self.squares();
        if i >= nx || j >= ny {
            return Vec::new();
        }
        let size = self.settings.square_size();
        // Corners as (east, north) offsets within the square.
        let tris: [[(usize, usize); 3]; 2] = if Self::diagonal_is_rising(i, j) {
            [[(0, 0), (1, 1), (1, 0)], [(0, 0), (0, 1), (1, 1)]]
        } else {
            [[(0, 0), (0, 1), (1, 0)], [(1, 0), (0, 1), (1, 1)]]
        };
        tris.iter()
            .map(|corners| {
                let positions = corners.map(|(di, dj)| {
                    [(i + di) as f32 * size, (j + dj) as f32 * size, self.point_height(i + di, j + dj).unwrap()]
                });
                // u runs west -> east, v runs north -> south.
                let uvs = corners.map(|(di, dj)| texture_coord(code, di as f32, 1.0 - dj as f32));
                GroundTriangle { positions, uvs, material, square: (i, j) }
            })
            .collect()
    }
}

/// One ground triangle in game units, corners clockwise seen from above.
#[derive(Debug, Clone, Copy)]
pub struct GroundTriangle {
    pub positions: [[f32; 3]; 3],
    /// Texture coordinates: x = image column (0 left .. 1 right),
    /// y = image row (0 top .. 1 bottom).
    pub uvs: [[f32; 2]; 3],
    pub material: u8,
    /// Grid square (column, row from the bottom).
    pub square: (usize, usize),
}

/// Where in the texture the point (u, v) of a square looks, for
/// orientation code `code` (terrain.md "Ground textures", rule 5).
/// u = 0 at the square's west edge, 1 at its east edge; v = 0 at its north
/// edge, 1 at its south edge. Unknown codes are treated as 0.
pub fn texture_coord(code: u32, u: f32, v: f32) -> [f32; 2] {
    match code {
        1 => [1.0 - u, v],
        2 => [u, 1.0 - v],
        3 => [1.0 - u, 1.0 - v],
        4 => [v, 1.0 - u],
        5 => [1.0 - v, 1.0 - u],
        6 => [v, u],
        7 => [1.0 - v, u],
        _ => [u, v],
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

    /// 5 x 4 grid points (4 x 3 squares) with uneven heights.
    fn bumpy() -> (Terrain, Grid) {
        let mut t = terrain([[0, 0], [0, 0]], 3.0);
        let values: Vec<u8> = (0..20u32).map(|k| ((k * 37 + 11) % 23) as u8).collect();
        t.heights = Grid { width: 5, height: 4, values };
        let materials = Grid { width: 4, height: 3, values: (0..12).collect() };
        (t, materials)
    }

    #[test]
    fn floor_grid_answers_floor_questions_and_falls_back_to_the_surface() {
        let t = terrain([[24, 24], [24, 24]], 255.0);
        // No floor image: the floor is the surface.
        assert_eq!(t.floor_at(1.0, 1.0).unwrap().height, 6120.0);
        let floor = Grid { width: 2, height: 2, values: vec![21, 21, 21, 21] };
        let t = t.with_floor(floor).unwrap();
        assert_eq!(t.floor_at(1.0, 1.0).unwrap().height, 5355.0);
        assert_eq!(t.height_at(1.0, 1.0).unwrap().height, 6120.0);
        assert_eq!(t.ground_at(Level::Surface, 1.0, 1.0).unwrap().height, 6120.0);
        let wrong = Grid { width: 3, height: 2, values: vec![0; 6] };
        assert!(terrain([[0, 0], [0, 0]], 1.0).with_floor(wrong).is_err());
    }

    #[test]
    fn ground_flags_follow_the_square_material() {
        let (t, materials) = bumpy();
        let flags = [0, crate::envinfo::WATER];
        let t = t.with_ground_flags(&materials, &flags).unwrap();
        let (nx, ny) = t.squares();
        for j in 0..ny {
            for i in 0..nx {
                let m = materials.get(i, j).unwrap() as usize;
                let want = flags.get(m).copied().unwrap_or(0);
                assert_eq!(t.flags_at(i as f32 * 4.0 + 1.0, j as f32 * 4.0 + 1.0), want, "square {i},{j}");
            }
        }
        assert_eq!(t.flags_at(-1.0, 0.0), 0);
    }

    #[test]
    fn ground_triangles_lie_on_the_height_lookup() {
        let (t, m) = bumpy();
        let tris = t.ground_triangles(&m, &[]).unwrap();
        assert_eq!(tris.len(), 4 * 3 * 2);
        for tri in &tris {
            let c: [f32; 3] = std::array::from_fn(|k| tri.positions.iter().map(|p| p[k]).sum::<f32>() / 3.0);
            let h = t.height_at(c[0], c[1]).unwrap().height;
            assert!((h - c[2]).abs() < 1e-3, "square {:?}: centre {c:?} lookup {h}", tri.square);
            assert_eq!(tri.material as usize, tri.square.1 * 4 + tri.square.0);
        }
    }

    #[test]
    fn ground_triangles_are_clockwise_from_above() {
        let (t, m) = bumpy();
        for tri in t.ground_triangles(&m, &[]).unwrap() {
            let [a, b, c] = tri.positions;
            // z of the cross product of (b - a) and (c - a): negative = clockwise seen from above
            let cross_z = (b[0] - a[0]) * (c[1] - a[1]) - (b[1] - a[1]) * (c[0] - a[0]);
            assert!(cross_z < 0.0, "square {:?} is not clockwise", tri.square);
        }
    }

    #[test]
    fn orientation_codes_are_eight_different_turns_and_mirrors() {
        // Where the square's north-west and north-east corners land in the texture.
        let maps: Vec<_> = (0..8).map(|c| (texture_coord(c, 0.0, 0.0), texture_coord(c, 1.0, 0.0))).collect();
        for a in 0..8 {
            for b in a + 1..8 {
                assert_ne!(maps[a], maps[b], "codes {a} and {b} are the same");
            }
        }
        // Code 0: as stored, image top at the north edge.
        assert_eq!(texture_coord(0, 0.25, 0.0), [0.25, 0.0]);
        // Code 4 (quarter turn clockwise): the image's bottom row ends up
        // along the square's west edge.
        assert_eq!(texture_coord(4, 0.0, 0.5), [0.5, 1.0]);
    }

    #[test]
    fn ground_texture_coords_follow_the_code() {
        let (t, m) = bumpy();
        let orientation: Vec<u32> = (0..12).map(|k| k % 8).collect();
        for tri in t.ground_triangles(&m, &orientation).unwrap() {
            let code = orientation[tri.material as usize];
            for (p, uv) in tri.positions.iter().zip(tri.uvs) {
                let (i, j) = tri.square;
                let u = p[0] / 4.0 - i as f32;
                let v = 1.0 - (p[1] / 4.0 - j as f32);
                assert_eq!(uv, texture_coord(code, u, v));
            }
        }
    }
}
