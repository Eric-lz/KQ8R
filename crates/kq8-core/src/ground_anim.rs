//! Ground animators: every `delay` seconds, set a group of ground squares
//! to the next material of a list, which makes water look like it flows.
//! Rules from `docs/behaviour/emitters.md` "Ground animators" (session A,
//! traced; square choice checked against the data). The keys are read by
//! `kq8_data::effects::AnimatorSettings`.
//!
//! Clock (rule 3, Q49, traced): the animator does not run on the 15-per-
//! second update. It is checked against continuous game time, and after
//! each change the next one is due `delay` after **the time the change
//! happened** (not after the previous due time), so each gap is `delay`
//! plus however late that check came.
//!
//! Not done: the envInfo flags of a changed square (open question 3; no
//! difference in Daventry), `frameCount` (saved games; no file sets it),
//! starting later and `terAnimSwitch` (script commands no script uses).

use kq8_data::effects::AnimatorSettings;
use kq8_data::terrain::Grid;

/// Allowance when comparing game time with a due time, so that e.g.
/// 4.5 / 15 counts as reaching 0.3.
const TIME_EPSILON: f64 = 1e-9;

/// The squares an animator works on (rule 1), chosen once from the
/// ground's **current** materials, plus notes for the log.
pub fn choose_squares(settings: &AnimatorSettings, materials: &Grid) -> (Vec<(usize, usize)>, Vec<String>) {
    let mut notes = Vec::new();
    let inside = |(i, j): (i64, i64)| {
        (i >= 0 && j >= 0 && (i as usize) < materials.width && (j as usize) < materials.height).then_some((i as usize, j as usize))
    };
    if let Some(blocks) = &settings.grid_blocks {
        let mut squares = Vec::new();
        for &b in blocks {
            match inside(b) {
                Some(sq) if !squares.contains(&sq) => squares.push(sq),
                Some(sq) => notes.push(format!("square {sq:?} listed twice")),
                None => notes.push(format!("square {b:?} is outside the {}x{} grid; left out", materials.width, materials.height)),
            }
        }
        return (squares, notes);
    }
    let mut squares = Vec::new();
    for j in 0..materials.height {
        for i in 0..materials.width {
            let m = materials.get(i, j).unwrap() as i64;
            if settings.orig_textures.contains(&m) && !settings.exclude.contains(&(i as i64, j as i64)) {
                squares.push((i, j));
            }
        }
    }
    (squares, notes)
}

/// One running animator.
#[derive(Debug, Clone)]
pub struct GroundAnimator {
    pub name: String,
    pub squares: Vec<(usize, usize)>,
    /// The materials shown one after another.
    pub cycle: Vec<u8>,
    pub delay: f64,
    pub repeat: bool,
    /// Counting down to changes.
    pub running: bool,
    /// Game time (s) of the next change: the time of the last change +
    /// `delay` (the start + `delay` before the first).
    pub due: f64,
    /// The cycle entry the next change shows.
    pub next_entry: usize,
}

/// One change: an animator set all its squares to `material` (cycle entry
/// `entry`) at game time `time` (when it was checked, at or after the due
/// time).
#[derive(Debug, Clone, PartialEq)]
pub struct Change {
    pub entry: usize,
    pub material: u8,
    pub time: f64,
}

impl GroundAnimator {
    /// Started at game time `now` when `startNow` is set (rule 3), with a
    /// usable cycle and a positive delay; otherwise it never changes
    /// anything (the reason is returned for the log). Cycle values that are
    /// not material numbers (0–255) are left out, with a note.
    pub fn new(name: &str, settings: &AnimatorSettings, squares: Vec<(usize, usize)>, now: f64) -> (Self, Vec<String>) {
        let mut notes = Vec::new();
        let cycle: Vec<u8> = settings
            .cycle
            .iter()
            .filter_map(|&m| {
                let ok = u8::try_from(m).ok();
                if ok.is_none() {
                    notes.push(format!("cycle value {m} is not a material number; left out"));
                }
                ok
            })
            .collect();
        let why_not = if !settings.start_now {
            Some("startNow is not set: not started (nothing in the scripts starts it)")
        } else if cycle.is_empty() {
            Some("no materials in its cycle: nothing to show")
        } else if settings.delay <= 0.0 {
            Some("delay is not above 0: not started")
        } else {
            None
        };
        notes.extend(why_not.map(str::to_string));
        if settings.frame_count != 0 {
            notes.push(format!("frameCount={} (saved game) not applied", settings.frame_count));
        }
        let a = GroundAnimator {
            name: name.to_string(),
            squares,
            cycle,
            delay: settings.delay,
            repeat: settings.repeat,
            running: why_not.is_none(),
            due: now + settings.delay,
            next_entry: 0,
        };
        (a, notes)
    }

    /// The change due by game time `now`, if any (rules 2–4), applied to
    /// `materials` (every chosen square gets the same material). At most
    /// one per call: the next is due `delay` after `now`.
    pub fn update(&mut self, now: f64, materials: &mut Grid) -> Option<Change> {
        if !(self.running && now + TIME_EPSILON >= self.due) {
            return None;
        }
        let material = self.cycle[self.next_entry];
        for &(i, j) in &self.squares {
            materials.set(i, j, material);
        }
        let change = Change { entry: self.next_entry, material, time: now };
        self.due = now + self.delay;
        self.next_entry += 1;
        if self.next_entry == self.cycle.len() {
            if self.repeat {
                self.next_entry = 0;
            } else {
                // Played once: the squares keep the last material.
                self.running = false;
            }
        }
        Some(change)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A 4×3 grid: material = 10 + column, except square (2, 1) = 20.
    fn grid() -> Grid {
        let mut v = Vec::new();
        for j in 0..3 {
            for i in 0..4 {
                v.push(if (i, j) == (2, 1) { 20 } else { 10 + i as u8 });
            }
        }
        Grid::from_rows(4, 3, v).unwrap()
    }

    fn settings() -> AnimatorSettings {
        AnimatorSettings {
            orig_textures: vec![11, 12],
            exclude: vec![(1, 0)],
            grid_blocks: None,
            cycle: vec![12, 30, 31, 30],
            cycle2: Vec::new(),
            delay: 0.3,
            repeat: true,
            start_now: true,
            frame_count: 0,
            problems: Vec::new(),
        }
    }

    #[test]
    fn squares_by_material_minus_excluded() {
        let (sq, notes) = choose_squares(&settings(), &grid());
        // Columns 1 and 2 have 11 and 12, except (2, 1) = 20 and the excluded (1, 0).
        assert_eq!(sq, vec![(2, 0), (1, 1), (1, 2), (2, 2)]);
        assert!(notes.is_empty());
    }

    #[test]
    fn grid_blocks_ignore_materials_and_excludes() {
        let mut s = settings();
        s.grid_blocks = Some(vec![(0, 0), (3, 2), (9, 9), (0, 0)]);
        let (sq, notes) = choose_squares(&s, &grid());
        assert_eq!(sq, vec![(0, 0), (3, 2)]);
        assert_eq!(notes.len(), 2);
    }

    #[test]
    fn each_change_is_delay_after_the_previous_change() {
        let mut g = grid();
        let (sq, _) = choose_squares(&settings(), &g);
        let (mut a, notes) = GroundAnimator::new("a", &settings(), sq, 0.0);
        assert!(notes.is_empty() && a.running);
        // Checked at uneven frame times (s), as drawn frames come.
        let frames = [0.1, 0.29, 0.31, 0.5, 0.6, 0.62, 0.9, 0.93, 1.3, 1.55];
        let seen: Vec<(f64, usize, u8)> =
            frames.iter().filter_map(|&t| a.update(t, &mut g)).map(|c| (c.time, c.entry, c.material)).collect();
        // First change at the first check at or after 0.3 (0.31); then each
        // one 0.3 after the previous **change**: due 0.61 -> 0.62, due 0.92
        // -> 0.93, due 1.23 -> 1.3 (one change only, though the check is late).
        // With "previous due + 0.3" they would have come at 0.6, 0.9, 1.3, 1.3.
        assert_eq!(seen, vec![(0.31, 0, 12), (0.62, 1, 30), (0.93, 2, 31), (1.3, 3, 30)]);
        assert!((a.due - 1.6).abs() < 1e-9);
        // Every chosen square shows the same material, the others are untouched.
        assert_eq!((g.get(2, 0), g.get(1, 1), g.get(1, 0), g.get(2, 1)), (Some(30), Some(30), Some(11), Some(20)));
    }

    #[test]
    fn played_once_keeps_the_last_material() {
        let mut s = settings();
        s.repeat = false;
        let mut g = grid();
        let (sq, _) = choose_squares(&s, &g);
        let (mut a, _) = GroundAnimator::new("a", &s, sq, 0.0);
        let changes = (1..=600).filter(|&n| a.update(n as f64 / 60.0, &mut g).is_some()).count();
        assert_eq!(changes, 4);
        assert!(!a.running);
        assert_eq!(g.get(2, 0), Some(30));
    }

    #[test]
    fn not_started_without_start_now() {
        let mut s = settings();
        s.start_now = false;
        let mut g = grid();
        let (mut a, notes) = GroundAnimator::new("a", &s, vec![(0, 0)], 0.0);
        assert_eq!(notes.len(), 1);
        assert!(a.update(100.0, &mut g).is_none());
        assert_eq!(g.get(0, 0), Some(10));
    }
}
