//! Swamp's gas/stump puzzle tiles: the rules of Connor's `Tile2` state that
//! don't need the viewer (`docs/behaviour/tile-puzzle.md` rules 29–35; Batch 3 B3-3).
//!
//! 10 columns × 7 rows (`0..70`): columns `2..=7` hold `tile00..tile56` (`tile<col><row>`);
//! columns `0, 1, 8, 9` are empty except on row `3` (the West `fiendoor` and East `Rtswdoor`
//! threshold spots at `x = 46,000 / 48,000` and `62,000 / 64,000`).

/// Grid width (`cols 0..10`) and height (`rows 0..7`).
pub const COLS: i32 = 10;
pub const ROWS: i32 = 7;

/// Spacing between adjacent puzzle tiles (`2,000` game units).
pub const SPACING: f32 = 2_000.0;
/// Half-tile margin used for spot bounds (`rule 30`).
pub const MARGIN: f32 = 1_000.0;
/// Fallback origin if `tile00`..`tile56` are missing (`col = 2, row = 0`).
pub const ORIGIN: [f32; 3] = [50_000.0, 92_600.0, 15_045.0];

/// `tile-puzzle.md` rule 34: doors triggered on landing in row 3's outer cells.
pub const WEST_DOOR: &str = "fiendoor";
pub const EAST_DOOR: &str = "Rtswdoor";

/// A cell on the `10 × 7` Swamp tile grid (`index = row * 10 + col`).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell2 {
    pub row: i32,
    pub col: i32,
}

impl Cell2 {
    pub const fn new(row: i32, col: i32) -> Self {
        Self { row, col }
    }

    pub const fn index(self) -> usize {
        (self.row * COLS + self.col) as usize
    }

    /// True if this cell holds one of the 42 `tile<col><row>` objects (`cols 2..=7`).
    pub fn is_tile(self) -> bool {
        (0..ROWS).contains(&self.row) && (2..=7).contains(&self.col)
    }

    /// Object name (`tile<col-2><row>`) when `is_tile()`.
    pub fn tile_name(self) -> Option<String> {
        self.is_tile().then(|| format!("tile{}{}", self.col - 2, self.row))
    }

    /// Valid walkable/jumpable spot on the 10 × 7 grid (`tile-puzzle.md` rules 29, 32):
    /// all cells in `cols 2..=7`, plus row `3` on `cols 0, 1, 8, 9`.
    pub fn is_valid_spot(self) -> bool {
        (0..ROWS).contains(&self.row) && ((2..=7).contains(&self.col) || (matches!(self.col, 0 | 1 | 8 | 9) && self.row == 3))
    }
}

impl std::fmt::Display for Cell2 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.tile_name() {
            Some(name) => write!(f, "{name} (cell {})", self.index()),
            None => write!(f, "row {} col {} (cell {})", self.row, self.col, self.index()),
        }
    }
}

/// Where Connor stands relative to the `Tile2` grid (`tile-puzzle.md` rule 30).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where2 {
    Spot(Cell2),
    /// `x < minX` (`100`).
    West,
    /// `x > maxX` (`200`).
    East,
    /// `y < minY` (`300`).
    South,
    /// `y > maxY` (`400`).
    North,
}

/// What landing on `spot` triggers (`tile-puzzle.md` rule 34).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing2 {
    /// Cells `30` or `31`: queue `Cue 1` to `fiendoor`.
    WestDoor(Cell2),
    /// Cells `38` or `39`: queue `ConnorAction "do"` to `Rtswdoor`.
    EastDoor(Cell2),
    /// Any other spot or outside area: stay standing in `Tile2`.
    Stay,
}

/// All 70 grid slots for Swamp `Tile2` (`tile-puzzle.md` rule 29).
#[derive(Debug, Clone)]
pub struct Tile2Grid {
    spots: [Option<[f32; 3]>; (COLS * ROWS) as usize],
    min_x: f32,
    max_x: f32,
    min_y: f32,
    max_y: f32,
}

impl Tile2Grid {
    /// Builds the `10 × 7` grid from `place_of`, recording any missing `tile<col><row>` names
    /// ("Do not reproduce" rule 29: skip missing tiles and log instead of crashing).
    pub fn read(place_of: &dyn Fn(&str) -> Option<[f32; 3]>) -> (Self, Vec<String>) {
        let mut spots = [None; (COLS * ROWS) as usize];
        let mut missing = Vec::new();
        for row in 0..ROWS {
            for tile_col in 0..6 {
                let c = Cell2::new(row, tile_col + 2);
                let name = c.tile_name().unwrap();
                match place_of(&name) {
                    Some(p) => spots[c.index()] = Some(p),
                    None => missing.push(name),
                }
            }
        }
        // Fallback helper if `tile03` / `tile53` / `tile00` / `tile56` aren't loaded (unit tests).
        let fallback = |col: i32, row: i32| [ORIGIN[0] + (col - 2) as f32 * SPACING, ORIGIN[1] + row as f32 * SPACING, ORIGIN[2]];
        let west = spots[Cell2::new(3, 2).index()].unwrap_or_else(|| fallback(2, 3));
        let east = spots[Cell2::new(3, 7).index()].unwrap_or_else(|| fallback(7, 3));
        spots[Cell2::new(3, 1).index()] = Some([west[0] - SPACING, west[1], west[2]]);
        spots[Cell2::new(3, 0).index()] = Some([west[0] - 2.0 * SPACING, west[1], west[2]]);
        spots[Cell2::new(3, 8).index()] = Some([east[0] + SPACING, east[1], east[2]]);
        spots[Cell2::new(3, 9).index()] = Some([east[0] + 2.0 * SPACING, east[1], east[2]]);

        let col0_r3 = spots[Cell2::new(3, 0).index()].unwrap();
        let col9_r3 = spots[Cell2::new(3, 9).index()].unwrap();
        let t00 = spots[Cell2::new(0, 2).index()].unwrap_or_else(|| fallback(2, 0));
        let t56 = spots[Cell2::new(6, 7).index()].unwrap_or_else(|| fallback(7, 6));
        let grid = Self {
            spots,
            min_x: col0_r3[0] - MARGIN,
            max_x: col9_r3[0] + MARGIN,
            min_y: t00[1] - MARGIN,
            max_y: t56[1] + MARGIN,
        };
        (grid, missing)
    }

    /// Position for valid cell `c`.
    pub fn place(&self, c: Cell2) -> [f32; 3] {
        self.spots[c.index()].unwrap_or([ORIGIN[0] + (c.col - 2) as f32 * SPACING, ORIGIN[1] + c.row as f32 * SPACING, ORIGIN[2]])
    }

    /// True if `c` is a real placed `tile<col><row>` in columns `2..=7` (used for lighting).
    pub fn has_tile(&self, c: Cell2) -> bool {
        c.is_tile() && self.spots[c.index()].is_some()
    }

    /// Rule 30: which spot or outside side `(x, y)` lies in.
    pub fn where_is(&self, x: f32, y: f32) -> Where2 {
        if y < self.min_y {
            Where2::South
        } else if y > self.max_y {
            Where2::North
        } else if x < self.min_x {
            Where2::West
        } else if x > self.max_x {
            Where2::East
        } else {
            let cell_w = (self.max_x - self.min_x) * 0.1;
            let cell_h = (self.max_y - self.min_y) / (ROWS as f32);
            let col = (((x - self.min_x) / cell_w).trunc() as i32).clamp(0, COLS - 1);
            let row = (((y - self.min_y) / cell_h).trunc() as i32).clamp(0, ROWS - 1);
            Where2::Spot(Cell2::new(row, col))
        }
    }
}

/// Rule 32: chooses the adjacent valid `Cell2` closest to Connor's facing (`back` = opposite),
/// applying the four edge blocks so empty corners (`cols 0, 1, 8, 9` with `row != 3`) are never entered.
pub fn jump_target(from: Cell2, heading: f32, back: bool) -> Option<Cell2> {
    let aim = if back { heading + std::f32::consts::PI } else { heading };
    let facing = [aim.sin(), -aim.cos()];
    let mut best: Option<(Cell2, f32)> = None;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let col = (from.col + dc).clamp(0, COLS - 1);
            let row = (from.row + dr).clamp(0, ROWS - 1);
            let cell = Cell2::new(row, col);
            if cell == from || !cell.is_valid_spot() {
                continue;
            }
            // Enforce rule 32's four edge blocks explicitly too:
            if (from.col == 2 && col == 1 && row != 3)
                || (from.col == 7 && col == 8 && row != 3)
                || (from.col == 1 && from.row == 3 && row != 3 && col != 2)
                || (from.col == 8 && from.row == 3 && row != 3 && col != 7)
            {
                continue;
            }
            let (dx, dy) = ((cell.col - from.col) as f32, (cell.row - from.row) as f32);
            let cos = (dx * facing[0] + dy * facing[1]) / (dx * dx + dy * dy).sqrt();
            if best.is_none_or(|(_, b)| cos > b) {
                best = Some((cell, cos));
            }
        }
    }
    best.map(|(c, _)| c)
}

/// Straight (cardinal) vs diagonal neighbour jump.
pub fn straight(from: Cell2, to: Cell2) -> bool {
    from.row == to.row || from.col == to.col
}

/// Rule 33: `tilejump1`, `tilejump2`, `tilebackjump`, `tilebackjump2`.
pub fn jump_movement(from: Cell2, to: Cell2, back: bool) -> &'static str {
    match (back, straight(from, to)) {
        (false, true) => "tilejump1",
        (false, false) => "tilejump2",
        (true, true) => "tilebackjump",
        (true, false) => "tilebackjump2",
    }
}

/// Rule 34: landing on cells `30/31` (`fiendoor Cue 1`) or `38/39` (`Rtswdoor ConnorAction "do"`).
pub fn landing(spot: Where2) -> Landing2 {
    match spot {
        Where2::Spot(c) if matches!(c.index(), 30 | 31) => Landing2::WestDoor(c),
        Where2::Spot(c) if matches!(c.index(), 38 | 39) => Landing2::EastDoor(c),
        _ => Landing2::Stay,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn shipped_grid() -> Tile2Grid {
        let (grid, missing) = Tile2Grid::read(&|name| {
            let bytes = name.as_bytes();
            let col = (bytes[4] - b'0') as i32;
            let row = (bytes[5] - b'0') as i32;
            Some([ORIGIN[0] + col as f32 * SPACING, ORIGIN[1] + row as f32 * SPACING, ORIGIN[2]])
        });
        assert!(missing.is_empty());
        grid
    }

    #[test]
    fn grid_places_42_tiles_and_outer_row3_door_spots() {
        let g = shipped_grid();
        assert_eq!(g.place(Cell2::new(3, 0)), [46_000.0, 98_600.0, 15_045.0]);
        assert_eq!(g.place(Cell2::new(3, 1)), [48_000.0, 98_600.0, 15_045.0]);
        assert_eq!(g.place(Cell2::new(3, 2)), [50_000.0, 98_600.0, 15_045.0]);
        assert_eq!(g.place(Cell2::new(3, 8)), [62_000.0, 98_600.0, 15_045.0]);
        assert_eq!(g.place(Cell2::new(3, 9)), [64_000.0, 98_600.0, 15_045.0]);
        assert_eq!(Cell2::new(0, 2).tile_name().as_deref(), Some("tile00"));
        assert_eq!(Cell2::new(6, 7).tile_name().as_deref(), Some("tile56"));
        assert_eq!(Cell2::new(3, 1).tile_name(), None);
        assert_eq!(Cell2::new(3, 0).index(), 30);
        assert_eq!(Cell2::new(3, 1).index(), 31);
        assert_eq!(Cell2::new(3, 8).index(), 38);
        assert_eq!(Cell2::new(3, 9).index(), 39);
    }

    #[test]
    fn spot_bounds_and_outside_directions_follow_rule_30() {
        let g = shipped_grid();
        assert_eq!(g.where_is(44_999.0, 98_600.0), Where2::West);
        assert_eq!(g.where_is(65_001.0, 98_600.0), Where2::East);
        assert_eq!(g.where_is(50_000.0, 91_599.0), Where2::South);
        assert_eq!(g.where_is(50_000.0, 105_601.0), Where2::North);
        assert_eq!(g.where_is(46_000.0, 98_600.0), Where2::Spot(Cell2::new(3, 0)));
        assert_eq!(g.where_is(50_000.0, 92_600.0), Where2::Spot(Cell2::new(0, 2)));
        assert_eq!(g.where_is(64_000.0, 98_600.0), Where2::Spot(Cell2::new(3, 9)));
        assert!(matches!(landing(Where2::Spot(Cell2::new(3, 1))), Landing2::WestDoor(_)));
        assert!(matches!(landing(Where2::Spot(Cell2::new(3, 8))), Landing2::EastDoor(_)));
        assert_eq!(landing(Where2::Spot(Cell2::new(3, 2))), Landing2::Stay);
    }

    #[test]
    fn four_edge_blocks_forbid_empty_corners() {
        let pi = std::f32::consts::PI;
        // 1. From col 2 row 0 facing west (-x: heading -pi/2) cannot enter (col 1, row 0).
        let t = jump_target(Cell2::new(0, 2), -pi / 2.0, false).unwrap();
        assert!(!(t.col == 1 && t.row != 3), "must not jump to empty corner {t}");
        // 2. From col 7 row 6 facing east (+x: heading +pi/2) cannot enter (col 8, row 6).
        let t = jump_target(Cell2::new(6, 7), pi / 2.0, false).unwrap();
        assert!(!(t.col == 8 && t.row != 3), "must not jump to empty corner {t}");
        // 3. From col 1 row 3 facing north (+y: heading pi) can only leave eastward onto (col 2, row 4).
        let t = jump_target(Cell2::new(3, 1), pi, false).unwrap();
        assert_eq!(t, Cell2::new(4, 2));
        // 4. From col 8 row 3 facing south (-y: heading 0) can only leave westward onto (col 7, row 2).
        let t = jump_target(Cell2::new(3, 8), 0.0, false).unwrap();
        assert_eq!(t, Cell2::new(2, 7));
    }
}
