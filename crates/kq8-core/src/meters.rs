//! What the bottom bar shows of Connor's stats (M4 step 4c-2): the health
//! and experience bars, the level, the three slot numbers.
//!
//! Rules from `docs/behaviour/interface.md` (A, 2026-10-18):
//!
//! - A bar shows width × percent / 100 pixels, cut to whole pixels, at
//!   least 1 when the percent isn't 0; the percent is kept in 0–100
//!   (rule 20).
//! - Health percent = health × 100 / (10 × level), whole-number division,
//!   at least 1 while health isn't 0, 0 while Connor is dead (rule 21).
//! - Experience percent = experience × 100 / the level's threshold, sent
//!   only when the experience changed (rule 22).
//! - After every stats change all values are sent at once: armour, hand,
//!   ranged, health %, experience % (if changed), level (rule 24).
//! - The three slot numbers are Connor's stats 4, 2 and 3 (rules 27, 31);
//!   a number that differs from the last one shown flashes its slot (rule
//!   33).

use crate::stats::ConnorStats;

/// Rule 21.
pub fn health_percent(c: &ConnorStats, dead: bool) -> i64 {
    let health = c.stats.0[0];
    if dead {
        return 0;
    }
    let p = health * 100 / c.max_health().max(1);
    if health != 0 && p == 0 {
        1
    } else {
        p
    }
}

/// Rule 22.
pub fn experience_percent(c: &ConnorStats) -> i64 {
    c.stats.0[4] * 100 / ConnorStats::threshold(c.level).max(1)
}

/// Rule 20: the shown width of a bar `full` pixels wide.
pub fn bar_width(full: u32, percent: i64) -> u32 {
    let p = percent.clamp(0, 100);
    let w = (full as i64 * p / 100) as u32;
    if p != 0 {
        w.max(1)
    } else {
        w
    }
}

/// The three slots whose number is shown (rules 27, 33, 36).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Slot {
    Armour,
    Hand,
    Ranged,
}

impl Slot {
    pub const ALL: [Slot; 3] = [Slot::Armour, Slot::Hand, Slot::Ranged];

    /// Its place in Connor's five stats (rule 27).
    pub fn stat(self) -> usize {
        match self {
            Slot::Armour => 3,
            Slot::Hand => 1,
            Slot::Ranged => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Slot::Armour => "armour",
            Slot::Hand => "hand weapon",
            Slot::Ranged => "ranged weapon",
        }
    }
}

/// One sending of the values (rule 24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Sent {
    /// Armour, hand, ranged (in [`Slot::ALL`] order).
    pub numbers: [i64; 3],
    /// The slots whose number changed since the last sending (rule 33).
    pub flashes: Vec<Slot>,
    pub health_percent: i64,
    /// `None`: the experience didn't change, so nothing was sent.
    pub experience_percent: Option<i64>,
    pub level: i64,
}

/// What the bottom bar last showed.
#[derive(Debug, Clone, Default)]
pub struct Meters {
    pub numbers: Option<[i64; 3]>,
    pub health_percent: i64,
    pub experience_percent: i64,
    pub level: i64,
    /// The experience the last percent was worked out from.
    last_experience: Option<i64>,
}

impl Meters {
    /// Rule 24: sends Connor's values; remembers them for drawing.
    pub fn send(&mut self, c: &ConnorStats, dead: bool) -> Sent {
        let numbers = Slot::ALL.map(|s| c.stats.0[s.stat()]);
        let flashes = match self.numbers {
            Some(old) => Slot::ALL.iter().zip(old.iter().zip(numbers)).filter(|(_, (o, n))| **o != *n).map(|(s, _)| *s).collect(),
            None => Vec::new(),
        };
        let health = health_percent(c, dead);
        let experience = (self.last_experience != Some(c.stats.0[4])).then(|| experience_percent(c));
        self.numbers = Some(numbers);
        self.health_percent = health;
        if let Some(e) = experience {
            self.experience_percent = e;
            self.last_experience = Some(c.stats.0[4]);
        }
        self.level = c.level;
        Sent { numbers, flashes, health_percent: health, experience_percent: experience, level: c.level }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::stats::Stats;

    #[test]
    fn percents_and_widths() {
        let start = ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 };
        assert_eq!(health_percent(&start, false), 100);
        assert_eq!(health_percent(&start, true), 0, "dead: 0");
        // 1 health of 200 (level 20) is 0 by division, but not 0: 1.
        let low = ConnorStats { stats: Stats([1, 0, 0, 0, 0]), level: 20 };
        assert_eq!(health_percent(&low, false), 1);
        // interface.md check 2: the bricks at level 1 give level 2,
        // health 20 of 20, experience 40 of 20: 200 %, drawn as 100 %.
        let bricks = ConnorStats { stats: Stats([20, 15, 2, 1, 40]), level: 2 };
        assert_eq!((health_percent(&bricks, false), experience_percent(&bricks)), (100, 200));
        assert_eq!(bar_width(86, 200), 86);
        assert_eq!(bar_width(86, 50), 43);
        assert_eq!(bar_width(86, 1), 1, "86 x 1 / 100 = 0, but the percent isn't 0");
        assert_eq!(bar_width(86, 0), 0);
        assert_eq!(bar_width(86, -5), 0);
    }

    #[test]
    fn every_sending_has_health_experience_only_when_changed_and_flashes_changed_numbers() {
        let mut m = Meters::default();
        let mut c = ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 };
        let first = m.send(&c, false);
        assert_eq!((first.numbers, first.flashes.clone(), first.health_percent, first.experience_percent, first.level), ([0, 10, 0], vec![], 100, Some(0), 1));
        // A fish: health only; the experience isn't sent again.
        c.stats.0[0] = 8;
        let fish = m.send(&c, false);
        assert_eq!((fish.health_percent, fish.experience_percent, fish.flashes.clone()), (80, None, vec![]));
        // The bricks: level 2, all three numbers change and flash.
        c = ConnorStats { stats: Stats([20, 15, 2, 1, 40]), level: 2 };
        let up = m.send(&c, false);
        assert_eq!((up.numbers, up.flashes, up.experience_percent, up.level), ([1, 15, 2], vec![Slot::Armour, Slot::Hand, Slot::Ranged], Some(200), 2));
        assert_eq!((m.health_percent, m.experience_percent, m.level), (100, 200, 2));
    }
}
