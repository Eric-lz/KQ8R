//! Stats and `KQStatsChangeRequestEvent` (M3 step 3i-6).
//!
//! Rules from `docs/behaviour/anim-scripts.md` rule 19 and
//! `docs/behaviour/clicking.md` rule 24 (traced; Connor's levels "for M5,
//! not checked in the game"):
//!
//! - Five whole numbers in the order of an object's `stats=` line: first
//!   health, fourth armour, fifth (for Connor) experience. The event's
//!   five numbers are **added**; the fourth is then capped at 100.
//! - An ordinary object whose health drops below 0 gets `Die` (the caller
//!   runs its script).
//! - Connor: added the same way, then his levels: health capped at
//!   10 × level; when experience passes 10 × 2^(level − 1) (150,000 from
//!   level 15 on) it drops by that amount, the level goes up by 1, health
//!   is set to the new maximum, the second stat +5, the third +2, the
//!   fourth +1 (cap 100).
//!
//! Refined by A's answer to Q86 (rules 19–20): strictly more than the
//! threshold; once per event; health capped at the old level, then a full
//! heal on level up; the "below 0" check comes right after the adding,
//! before any cap; the event's own fourth number is cut to at most 100.

/// The fourth stat's cap.
pub const FOURTH_CAP: i64 = 100;

/// Experience needed from level 15 on.
pub const TOP_THRESHOLD: i64 = 150_000;

/// The meters' level-up sound (`interface.md` rule 26; the user's check,
/// Q149 c).
pub const LEVEL_UP_SOUND: &str = "LEVEL.AUD";

/// The experience sound (`interface.md` rule 26a; the user's check, Q159).
pub const EXPERIENCE_SOUND: &str = "STATS.AUD";

/// Rule 26a: plays when the experience percent is set and is not 0 and at
/// least the last percent shown (`last`, which starts at 0 when the
/// program starts and is never reset). The percent counts as 0-100.
pub fn experience_sound_plays(percent: i64, last: i64) -> bool {
    let p = percent.clamp(0, 100);
    p != 0 && p >= last.clamp(0, 100)
}

/// Health of a door without a `stats=` line (`doors.md` rule 18).
pub const DOOR_HEALTH: i64 = 10_000;

/// Five stats: health first.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Stats(pub [i64; 5]);

impl Stats {
    /// A `stats=` line or an event's numbers: up to five whole numbers,
    /// missing ones 0. `Err` names a word that is not a number (it counts
    /// as 0).
    pub fn parse<S: AsRef<str>>(words: &[S]) -> (Stats, Vec<String>) {
        let mut out = [0i64; 5];
        let mut problems = Vec::new();
        for (slot, w) in out.iter_mut().zip(words) {
            let w = w.as_ref();
            match w.parse::<f64>() {
                Ok(v) => *slot = v as i64,
                Err(_) => problems.push(format!("'{w}' is not a number (taken as 0)")),
            }
        }
        if words.len() > 5 {
            problems.push(format!("{} numbers, only the first 5 used", words.len()));
        }
        (Stats(out), problems)
    }

    /// Adds `d`, then caps the fourth at 100.
    pub fn add(&mut self, d: Stats) {
        for (s, v) in self.0.iter_mut().zip(d.0) {
            *s += v;
        }
        self.0[3] = self.0[3].min(FOURTH_CAP);
    }

    /// Health below 0.
    pub fn dead(&self) -> bool {
        self.0[0] < 0
    }
}

/// Connor's stats and level.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ConnorStats {
    pub stats: Stats,
    pub level: i64,
}

impl ConnorStats {
    /// Experience needed to go up from `level`.
    pub fn threshold(level: i64) -> i64 {
        if level >= 15 {
            TOP_THRESHOLD
        } else {
            10 * (1i64 << (level - 1).clamp(0, 62))
        }
    }

    /// Full health at the current level.
    pub fn max_health(&self) -> i64 {
        10 * self.level
    }

    /// An event's numbers: added, health capped at the old level, then
    /// one level check (rule 19, Q86). Returns a note when he went up a
    /// level.
    pub fn change(&mut self, d: Stats) -> Option<String> {
        self.stats.add(d);
        self.stats.0[0] = self.stats.0[0].min(self.max_health());
        let t = Self::threshold(self.level);
        if self.stats.0[4] <= t {
            return None;
        }
        self.stats.0[4] -= t;
        self.level += 1;
        let s = &mut self.stats.0;
        s[0] = 10 * self.level;
        s[1] += 5;
        s[2] += 2;
        s[3] = (s[3] + 1).min(FOURTH_CAP);
        Some(format!("experience passed {t}: level {} (health {}, second {}, third {}, fourth {}, experience {} left)", self.level, s[0], s[1], s[2], s[3], s[4]))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_are_added_and_the_fourth_capped() {
        let mut s = Stats([5, 0, 0, 90, 0]);
        s.add(Stats([2, 1, 0, 30, 0]));
        assert_eq!(s, Stats([7, 1, 0, 100, 0]));
        s.add(Stats([-8, 0, 0, 0, 0]));
        assert!(s.dead());
        let (p, problems) = Stats::parse(&["2", "0", "x"]);
        assert_eq!(p, Stats([2, 0, 0, 0, 0]));
        assert_eq!(problems.len(), 1);
    }

    #[test]
    fn connor_levels() {
        // Connor.kq: stats=10 10 0 0 0, level=1.
        let mut c = ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 };
        // A fish: health 2, capped at 10 × 1.
        assert_eq!(c.change(Stats([2, 0, 0, 0, 0])), None);
        assert_eq!(c.stats.0[0], 10);
        // Exactly 10 isn't more than 10 (Q86 a).
        assert_eq!(c.change(Stats([0, 0, 0, 0, 10])), None);
        // 10 + 50 = 60 is more than 10: level 2, 50 left, once per event,
        // full heal, second +5, third +2, fourth +1 (Q86 b, c).
        assert!(c.change(Stats([0, 0, 0, 0, 50])).is_some());
        assert_eq!((c.level, c.stats), (2, Stats([20, 15, 2, 1, 50])));
        // The bricks (bricks.cs): +50 at level 1 with 0: level 2, 40 left.
        let mut b = ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 };
        b.change(Stats([0, 0, 0, 0, 50]));
        assert_eq!((b.level, b.stats.0[0], b.stats.0[4]), (2, 20, 40));
        // The 40 left is more than 20, but waits for the next event of any
        // kind, even all zeros.
        assert!(b.change(Stats([0; 5])).is_some());
        assert_eq!((b.level, b.stats.0[4]), (3, 20));
        assert_eq!(ConnorStats::threshold(2), 20);
        assert_eq!(ConnorStats::threshold(14), 81_920);
        assert_eq!(ConnorStats::threshold(15), 150_000);
    }

    #[test]
    fn plays_when_not_0_and_not_lower_than_the_last() {
        // interface.md rule 26a (Q159).
        assert!(!experience_sound_plays(0, 0), "0 % is silent");
        assert!(experience_sound_plays(50, 0));
        assert!(experience_sound_plays(50, 50), "equal counts");
        assert!(!experience_sound_plays(15, 80), "a level-up's drop: only LEVEL.AUD");
        assert!(experience_sound_plays(200, 100), "200 % counts as 100");
    }
}
