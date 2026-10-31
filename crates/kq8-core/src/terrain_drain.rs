//! `matchHeights minX minY maxX maxY frames` (batch 2 D-8b): the surface
//! height points (`terrh.dib`, the liquid's top) of a rectangle are moved
//! to the floor points (`floorBMPName`) in `frames` even steps, one per
//! update, so a liquid drains to its bed. Rules from
//! `docs/behaviour/terrain.md` "`matchHeights`" (session A, traced).
//!
//! The grids hold whole pixel values (height = value × `heightBMPScale`),
//! so a step in between is rounded to the nearest value; the last step is
//! exactly the floor. Only `frames` 1 is used in the data (Dead City
//! `scale.cs`), where that makes no difference.
//!
//! Not done: pausing the animated height cycle while a drain runs (not
//! built; no Dead City data) and remembering drains for saved games.

use kq8_data::terrain::Grid;

/// Rule: up to this many run at once.
pub const MAX_RUNNING: usize = 32;

/// One running `matchHeights`.
#[derive(Debug, Clone)]
pub struct Drain {
    /// Corner columns `i0..i1` and rows `j0..j1` (the max not included).
    pub columns: (usize, usize),
    pub rows: (usize, usize),
    pub frames: u32,
    /// Steps done so far.
    pub done: u32,
    /// Per corner (row by row): (i, j, start value, floor value).
    corners: Vec<(usize, usize, u8, u8)>,
}

impl Drain {
    /// The corners of the rectangle (world units, read as whole numbers;
    /// corner = coordinate ÷ square size, the max one not included),
    /// clipped to the grid. `Err` with the reason when nothing can run.
    pub fn new(words: [i64; 5], square_size: i64, surface: &Grid, floor: &Grid) -> Result<Self, String> {
        let [min_x, min_y, max_x, max_y, frames] = words;
        if frames < 1 {
            // Not in the note and not in the data: ours, nothing changes.
            return Err(format!("frames {frames} is not 1 or more; nothing"));
        }
        if square_size < 1 {
            return Err(format!("square size {square_size}; nothing"));
        }
        let corner = |v: i64, n: usize| v.div_euclid(square_size).clamp(0, n as i64) as usize;
        let columns = (corner(min_x, surface.width), corner(max_x, surface.width));
        let rows = (corner(min_y, surface.height), corner(max_y, surface.height));
        let mut corners = Vec::new();
        for j in rows.0..rows.1 {
            for i in columns.0..columns.1 {
                let (Some(s), Some(f)) = (surface.get(i, j), floor.get(i, j)) else { continue };
                corners.push((i, j, s, f));
            }
        }
        Ok(Drain { columns, rows, frames: frames as u32, done: 0, corners })
    }

    /// How many corners it covers, and how many of them differ from the
    /// floor at the start.
    pub fn counts(&self) -> (usize, usize) {
        (self.corners.len(), self.corners.iter().filter(|c| c.2 != c.3).count())
    }

    pub fn finished(&self) -> bool {
        self.done >= self.frames
    }

    /// The next step: writes its values into `surface` and returns the
    /// corners whose value changed.
    pub fn step(&mut self, surface: &mut Grid) -> Vec<(usize, usize)> {
        if self.finished() {
            return Vec::new();
        }
        self.done += 1;
        let k = self.done as f32 / self.frames as f32;
        let mut changed = Vec::new();
        for &(i, j, start, floor) in &self.corners {
            let v = if self.done == self.frames { floor } else { (start as f32 + (floor as f32 - start as f32) * k).round() as u8 };
            if surface.get(i, j) != Some(v) {
                surface.set(i, j, v);
                changed.push((i, j));
            }
        }
        changed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 6×5 corners: surface 100 everywhere, floor 40.
    fn grids() -> (Grid, Grid) {
        (Grid::from_rows(6, 5, vec![100; 30]).unwrap(), Grid::from_rows(6, 5, vec![40; 30]).unwrap())
    }

    #[test]
    fn the_rectangle_leaves_out_the_max_corner_and_one_frame_reaches_the_floor() {
        let (mut s, f) = grids();
        // Square size 2048: x 2100..6200 -> columns 1, 2 (6200 / 2048 = 3, not included).
        let mut d = Drain::new([2100, 0, 6200, 4096, 1], 2048, &s, &f).unwrap();
        assert_eq!((d.columns, d.rows, d.counts()), ((1, 3), (0, 2), (4, 4)));
        let changed = d.step(&mut s);
        assert_eq!(changed, vec![(1, 0), (2, 0), (1, 1), (2, 1)]);
        assert!(d.finished());
        assert_eq!((s.get(1, 0), s.get(2, 1), s.get(3, 0), s.get(1, 2)), (Some(40), Some(40), Some(100), Some(100)));
        assert!(d.step(&mut s).is_empty());
    }

    #[test]
    fn several_frames_are_even_steps_ending_on_the_floor() {
        let (mut s, f) = grids();
        let mut d = Drain::new([0, 0, 2048, 2048, 4], 2048, &s, &f).unwrap();
        let seen: Vec<u8> = (0..4).map(|_| {
            d.step(&mut s);
            s.get(0, 0).unwrap()
        }).collect();
        // 100 -> 40 in 4 steps of 15.
        assert_eq!(seen, vec![85, 70, 55, 40]);
        assert!(d.finished());
    }

    #[test]
    fn outside_the_grid_is_clipped_and_bad_frames_do_nothing() {
        let (s, f) = grids();
        let d = Drain::new([-5000, 6000, 99999, 99999, 1], 2048, &s, &f).unwrap();
        assert_eq!((d.columns, d.rows), ((0, 6), (2, 5)));
        assert!(Drain::new([0, 0, 2048, 2048, 0], 2048, &s, &f).is_err());
    }
}
