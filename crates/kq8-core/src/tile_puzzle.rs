//! Dead City's death-hall stones: the rules of Connor's `Tile` state that
//! don't need the viewer (`docs/behaviour/tile-puzzle.md` rules 2-15;
//! batch 2, D-3). The viewer's `connor_view` runs the state with them.
//!
//! The right order and where each right stone sends Connor are in the
//! program, not in the data; they are kept here as a table (the note's
//! "Do not reproduce") so they can be logged and checked.

/// Columns and rows of stones (`tile<row><column>`, rule 3).
pub const SIZE: i32 = 8;
/// Stone spacing in x and y, and the rise per row in z (rule 3).
pub const SPACING: f32 = 2_000.0;
pub const RISE: f32 = 500.0;
/// `tile00`'s place (rule 3).
pub const ORIGIN: [f32; 3] = [230_333.0, 124_297.0, 2_871.0];
/// The first spot row: two rows before row 0 (rule 5).
pub const FIRST_ROW: i32 = -2;
/// The last spot row: one after row 7 (rule 5).
pub const LAST_ROW: i32 = SIZE;
/// Where Connor is put after a wrong stone (rule 14).
pub const START: [f32; 3] = [238_340.0, 122_300.0, 3_100.0];
/// The camera point for a wrong stone (rule 14).
pub const START_CAMERA: [f32; 3] = [237_333.0, 118_500.0, 4_500.0];
/// "Put on" a stone: this high above it; a camera point: this high (rule 13).
pub const PUT_ON_LIFT: f32 = 1_000.0;
pub const CAMERA_LIFT: f32 = 2_000.0;
/// Connor's lines after `teleportup` (rule 15): 2000.msg noun 2, verb 0.
pub const LINE_FILE: u16 = 2000;
pub const LINE_NOUN: u8 = 2;
pub const FIRST_TELEPORT_CONDITION: u8 = 3;
pub const SENT_BACK_CONDITION: u8 = 133;

/// A spot of the floor: a stone (rows 0-7) or one of the extra rows
/// (−2, −1, 8; rule 5). Columns 0-7.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub row: i32,
    pub col: i32,
}

impl Cell {
    pub const fn new(row: i32, col: i32) -> Self {
        Cell { row, col }
    }

    /// A real stone (rows 0-7).
    pub fn is_stone(&self) -> bool {
        (0..SIZE).contains(&self.row) && (0..SIZE).contains(&self.col)
    }

    /// Its stone's name, `tile<row><column>` (rule 2).
    pub fn name(&self) -> String {
        format!("tile{}{}", self.row, self.col)
    }
}

impl std::fmt::Display for Cell {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_stone() {
            write!(f, "{}", self.name())
        } else {
            write!(f, "row {} column {}", self.row, self.col)
        }
    }
}

/// Where Connor stands (rule 6).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Where {
    Spot(Cell),
    South,
    North,
    West,
    East,
}

/// Rule 6: the area is x 229,333–245,333, y 119,297–139,297 (rows −2 to
/// 7; the row-8 spot lies outside it, "north"); inside it the column and
/// row are worked out by whole spacings.
pub fn where_is(x: f32, y: f32) -> Where {
    let half = SPACING / 2.0;
    let (x0, y0) = (ORIGIN[0] - half, ORIGIN[1] + FIRST_ROW as f32 * SPACING - half);
    let (x1, y1) = (x0 + SIZE as f32 * SPACING, y0 + (SIZE - FIRST_ROW) as f32 * SPACING);
    if y < y0 {
        Where::South
    } else if y >= y1 {
        Where::North
    } else if x < x0 {
        Where::West
    } else if x >= x1 {
        Where::East
    } else {
        let col = ((x - x0) / SPACING) as i32;
        let row = ((y - y0) / SPACING) as i32 + FIRST_ROW;
        Where::Spot(Cell::new(row, col))
    }
}

/// The 64 stones' places, copied when the state starts (rule 2).
#[derive(Debug, Clone)]
pub struct Stones {
    places: [[Option<[f32; 3]>; SIZE as usize]; SIZE as usize],
}

impl Stones {
    /// Rule 2: each stone looked up by name. **Do not reproduce** the
    /// original's crash on a missing one: it is skipped (its spot then
    /// uses the rule 3 place) and named in the returned list.
    pub fn read(place_of: &dyn Fn(&str) -> Option<[f32; 3]>) -> (Stones, Vec<String>) {
        let mut places = [[None; SIZE as usize]; SIZE as usize];
        let mut missing = Vec::new();
        for (r, row) in places.iter_mut().enumerate() {
            for (c, place) in row.iter_mut().enumerate() {
                let name = Cell::new(r as i32, c as i32).name();
                *place = place_of(&name);
                if place.is_none() {
                    missing.push(name);
                }
            }
        }
        (Stones { places }, missing)
    }

    /// A spot's place: a stone's copied place, else rule 3's grid (the
    /// extra rows take the z of the nearest stone row, rule 5).
    pub fn place(&self, cell: Cell) -> [f32; 3] {
        let col = cell.col.clamp(0, SIZE - 1);
        let near = cell.row.clamp(0, SIZE - 1);
        let grid = |row: i32| [ORIGIN[0] + SPACING * col as f32, ORIGIN[1] + SPACING * row as f32, ORIGIN[2] + RISE * row as f32];
        let stone = self.places[near as usize][col as usize].unwrap_or(grid(near));
        if cell.row == near {
            stone
        } else {
            [stone[0], grid(cell.row)[1], stone[2]]
        }
    }

    /// The stone a jump lands on, if it is a real one (rule 10: only
    /// those are lit).
    pub fn found(&self, cell: Cell) -> bool {
        cell.is_stone() && self.places[cell.row as usize][cell.col as usize].is_some()
    }
}

/// Rule 9: the neighbouring spot whose direction from Connor is closest
/// to his facing (`back` false) or to straight behind him. Columns are
/// clamped to 0-7 and rows to −2..8; his own spot is never chosen, nor,
/// from the start strip (row −1), the two diagonal spots behind it.
pub fn jump_target(from: Cell, heading: f32, back: bool) -> Option<Cell> {
    let aim = if back { heading + std::f32::consts::PI } else { heading };
    let facing = [aim.sin(), -aim.cos()];
    let mut best: Option<(Cell, f32)> = None;
    for dr in -1..=1 {
        for dc in -1..=1 {
            let cell = Cell::new((from.row + dr).clamp(FIRST_ROW, LAST_ROW), (from.col + dc).clamp(0, SIZE - 1));
            if cell == from || (from.row == -1 && cell.row == FIRST_ROW && cell.col != from.col) {
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

/// Rule 11: same row or same column.
pub fn straight(from: Cell, to: Cell) -> bool {
    from.row == to.row || from.col == to.col
}

/// Rule 11: the jump movements.
pub fn jump_movement(from: Cell, to: Cell, back: bool) -> &'static str {
    match (back, straight(from, to)) {
        (false, true) => "tilejump1",
        (false, false) => "tilejump2",
        (true, true) => "tilebackjump",
        (true, false) => "tilebackjump2",
    }
}

/// One right stone (rule 13): the stone, where Connor is put and the
/// camera point: a stone (`Ok`) or, for the last letter, a fixed place.
#[derive(Debug, Clone, Copy)]
pub struct Letter {
    pub letter: char,
    pub stone: Cell,
    pub put_on: Result<Cell, [f32; 3]>,
    pub camera: Result<Cell, [f32; 3]>,
}

/// Rules 4 and 13: D, E, A, T, H by grid cell (`tile07`, `tile63`,
/// `tile21`, `tile56`, `tile10`).
pub const ORDER: [Letter; 5] = [
    Letter { letter: 'D', stone: Cell::new(0, 7), put_on: Ok(Cell::new(7, 3)), camera: Ok(Cell::new(7, 6)) },
    Letter { letter: 'E', stone: Cell::new(6, 3), put_on: Ok(Cell::new(2, 2)), camera: Ok(Cell::new(3, 4)) },
    Letter { letter: 'A', stone: Cell::new(2, 1), put_on: Ok(Cell::new(4, 6)), camera: Ok(Cell::new(4, 3)) },
    Letter { letter: 'T', stone: Cell::new(5, 6), put_on: Ok(Cell::new(1, 1)), camera: Ok(Cell::new(2, 4)) },
    Letter { letter: 'H', stone: Cell::new(1, 0), put_on: Err([237_333.0, 140_000.0, 6_900.0]), camera: Err([237_333.0, 135_000.0, 8_500.0]) },
];

/// Rule 12: what a landing means.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Landing {
    /// The expected stone (rule 13).
    Right,
    /// Row −2, behind the start strip: the inner door gets a click (rule 19).
    Door,
    /// The start strip: nothing.
    StartStrip,
    /// Anything else (rule 14).
    Wrong,
}

pub fn landing(at: Where, expected: usize) -> Landing {
    match at {
        Where::Spot(c) if Some(c) == ORDER.get(expected).map(|l| l.stone) => Landing::Right,
        Where::Spot(c) if c.row == FIRST_ROW => Landing::Door,
        Where::Spot(c) if c.row == -1 => Landing::StartStrip,
        _ => Landing::Wrong,
    }
}

/// Where a right stone or a wrong one sends Connor and the camera
/// (rules 13, 14): (put at, camera point).
pub fn teleport_places(stones: &Stones, landing: Landing, letter: usize) -> ([f32; 3], [f32; 3]) {
    let lift = |c: Cell, by: f32| {
        let p = stones.place(c);
        [p[0], p[1], p[2] + by]
    };
    match (landing, ORDER.get(letter)) {
        (Landing::Right, Some(l)) => (
            l.put_on.map_or_else(|p| p, |c| lift(c, PUT_ON_LIFT)),
            l.camera.map_or_else(|p| p, |c| lift(c, CAMERA_LIFT)),
        ),
        _ => (START, START_CAMERA),
    }
}

/// The lines of rule 15, once each per visit (the state starts fresh).
#[derive(Debug, Clone, Default)]
pub struct Lines {
    pub teleported: bool,
    pub had_right: bool,
    pub sent_back_said: bool,
}

impl Lines {
    /// After a teleport's `teleportup` ends: the condition to say, if any.
    pub fn after_teleport(&mut self, right: bool) -> Option<u8> {
        let first = !std::mem::replace(&mut self.teleported, true);
        let said = if first {
            Some(FIRST_TELEPORT_CONDITION)
        } else if !right && self.had_right && !self.sent_back_said {
            self.sent_back_said = true;
            Some(SENT_BACK_CONDITION)
        } else {
            None
        };
        self.had_right |= right;
        said
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::{FRAC_PI_2, FRAC_PI_4, PI};

    /// Heading 0 faces −Y (south); north (+Y, rows going up) is π.
    const NORTH: f32 = PI;

    #[test]
    fn spots_by_rule_6() {
        // The start: column 4, row −1.
        assert_eq!(where_is(START[0], START[1]), Where::Spot(Cell::new(-1, 4)));
        assert_eq!(where_is(244_333.0, 124_297.0), Where::Spot(Cell::new(0, 7)));
        assert_eq!(where_is(230_333.0, 138_297.0), Where::Spot(Cell::new(7, 0)));
        assert_eq!(where_is(230_333.0, 140_297.0), Where::North, "the row-8 spot");
        assert_eq!(where_is(237_333.0, 119_296.0), Where::South);
        assert_eq!(where_is(237_333.0, 139_297.0), Where::North);
        assert_eq!(where_is(229_332.0, 125_000.0), Where::West);
        assert_eq!(where_is(245_333.0, 125_000.0), Where::East);
    }

    #[test]
    fn the_jump_goes_to_the_neighbour_nearest_his_facing() {
        let start = Cell::new(-1, 4);
        assert_eq!(jump_target(start, NORTH, false), Some(Cell::new(0, 4)));
        // Facing between north and +x (heading π/2 faces +x): the
        // diagonal toward column 7.
        assert_eq!(jump_target(start, NORTH - FRAC_PI_4, false), Some(Cell::new(0, 5)));
        assert_eq!(jump_target(start, FRAC_PI_2, false), Some(Cell::new(-1, 5)));
        // Back from the strip: straight behind only (no back diagonals).
        assert_eq!(jump_target(start, NORTH, true), Some(Cell::new(-2, 4)));
        assert_eq!(jump_target(start, NORTH + FRAC_PI_4, true), Some(Cell::new(-2, 4)));
        // No wrap at the edge; row 8 can be reached.
        assert_eq!(jump_target(Cell::new(3, 7), FRAC_PI_2, false).map(|c| c.col), Some(7));
        assert_eq!(jump_target(Cell::new(7, 2), NORTH, false), Some(Cell::new(8, 2)));
        assert_eq!(jump_movement(start, Cell::new(0, 5), false), "tilejump2");
        assert_eq!(jump_movement(start, Cell::new(-2, 4), true), "tilebackjump");
    }

    #[test]
    fn landings_and_teleports_follow_the_order() {
        let (stones, missing) = Stones::read(&|_| None);
        assert_eq!(missing.len(), 64);
        let d = Where::Spot(Cell::new(0, 7));
        assert_eq!(landing(d, 0), Landing::Right);
        assert_eq!(landing(d, 1), Landing::Wrong);
        assert_eq!(landing(Where::Spot(Cell::new(-2, 4)), 0), Landing::Door);
        assert_eq!(landing(Where::Spot(Cell::new(-1, 6)), 2), Landing::StartStrip);
        assert_eq!(landing(Where::Spot(Cell::new(8, 0)), 0), Landing::Wrong);
        assert_eq!(landing(Where::East, 0), Landing::Wrong);
        // D puts him on tile73 + 1,000, the camera above tile76 + 2,000.
        let (put, cam) = teleport_places(&stones, Landing::Right, 0);
        assert_eq!(put, [236_333.0, 138_297.0, 6_371.0 + 1_000.0]);
        assert_eq!(cam, [242_333.0, 138_297.0, 6_371.0 + 2_000.0]);
        assert_eq!(teleport_places(&stones, Landing::Right, 4).0, [237_333.0, 140_000.0, 6_900.0]);
        assert_eq!(teleport_places(&stones, Landing::Wrong, 3), (START, START_CAMERA));
        // Each "put on" stone is a neighbour of the next letter (rule 13).
        for w in ORDER.windows(2) {
            let (put, next) = (w[0].put_on.unwrap(), w[1].stone);
            assert!((put.row - next.row).abs() <= 1 && (put.col - next.col).abs() <= 1, "{put} next to {next}");
        }
    }

    #[test]
    fn the_stones_read_by_name_and_the_extra_rows_take_the_nearest_z() {
        let (stones, missing) = Stones::read(&|n| (n != "tile33").then_some([1.0, 2.0, 3.0]));
        assert_eq!(missing, vec!["tile33".to_string()]);
        assert_eq!(stones.place(Cell::new(0, 0)), [1.0, 2.0, 3.0]);
        assert_eq!(stones.place(Cell::new(3, 3)), [236_333.0, 130_297.0, 4_371.0]);
        assert_eq!(stones.place(Cell::new(-2, 0)), [1.0, 120_297.0, 3.0]);
        assert!(!stones.found(Cell::new(3, 3)) && stones.found(Cell::new(7, 7)) && !stones.found(Cell::new(8, 7)));
    }

    #[test]
    fn the_lines_once_per_visit() {
        let mut l = Lines::default();
        assert_eq!(l.after_teleport(false), Some(3));
        assert_eq!(l.after_teleport(false), None, "no right stone yet");
        assert_eq!(l.after_teleport(true), None);
        assert_eq!(l.after_teleport(false), Some(133));
        assert_eq!(l.after_teleport(false), None);
    }
}
