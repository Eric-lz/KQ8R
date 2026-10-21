//! The four potions while they run (M5 5g; `docs/behaviour/potions.md`,
//! A 2026-10-20, checks 2–6 done by the user in the original).
//!
//! - The timer (rules 33–34): each running potion counts whole seconds,
//!   `Dur` at the start, one less every second of game time (15 updates);
//!   when it would drop below 1 the potion ends, so it lasts exactly `Dur`
//!   seconds. Game time is our updates: they keep running when the window
//!   loses focus; only a main menu would stop them (none yet).
//! - Nothing ends a potion early (rule 15): only its timer.
//!
//! Each running potion is one record ([`Running`]) Connor keeps, holding
//! everything a saved game needs (rule 34: the potions survive a save);
//! there are no saves yet.

/// Which built-in effect an item has, by its type (rule 12).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Effect {
    Clarity,
    Invisible,
    /// `INVITEM_Invulnerable`.
    Shield,
    Strength,
    /// Any other item with a duration: only its numbers (`inventory.md`
    /// rule 14).
    Plain,
}

impl Effect {
    pub fn of(kind: &str) -> Effect {
        match kind.to_ascii_lowercase().as_str() {
            "invitem_clarity" => Effect::Clarity,
            "invitem_invisible" => Effect::Invisible,
            "invitem_invulnerable" => Effect::Shield,
            "invitem_strength" => Effect::Strength,
            _ => Effect::Plain,
        }
    }
}

/// A potion running on Connor.
#[derive(Debug, Clone, PartialEq)]
pub struct Running {
    /// The item object (the stack) and its type.
    pub item: String,
    pub kind: String,
    pub effect: Effect,
    /// `Dur`, whole seconds.
    pub duration: i64,
    /// Whole seconds left.
    pub left: i64,
    /// The update at which the next second is taken off.
    pub next_tick: u64,
}

impl Running {
    pub fn new(item: &str, kind: &str, duration: i64, now_update: u64) -> Running {
        Running { item: item.to_string(), kind: kind.to_string(), effect: Effect::of(kind), duration, left: duration, next_tick: now_update + crate::UPDATES_PER_SECOND as u64 }
    }

    /// The part left in percent (rule 35: whole-number division).
    pub fn percent(&self) -> i64 {
        if self.duration <= 0 {
            return 0;
        }
        self.left * 100 / self.duration
    }

    /// Rule 33 at update `now`: the seconds taken off (each one's seconds
    /// left after it), and whether the potion is over.
    pub fn tick(&mut self, now: u64) -> (Vec<i64>, bool) {
        let mut seconds = Vec::new();
        while now >= self.next_tick {
            self.next_tick += crate::UPDATES_PER_SECOND as u64;
            if self.left - 1 < 1 {
                self.left = 0;
                return (seconds, true);
            }
            self.left -= 1;
            seconds.push(self.left);
        }
        (seconds, false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_potion_lasts_exactly_its_whole_seconds() {
        let mut r = Running::new("p", "INVITEM_Strength", 20, 30);
        assert_eq!(r.effect, Effect::Strength);
        assert_eq!(r.percent(), 100);
        assert_eq!(r.tick(44), (vec![], false));
        assert_eq!(r.tick(45), (vec![19], false));
        assert_eq!(r.percent(), 95);
        let mut ended = None;
        for u in 46..=400 {
            if r.tick(u).1 {
                ended = Some(u);
                break;
            }
        }
        assert_eq!(ended, Some(30 + 20 * 15), "20 s of game time after the start");
        assert_eq!(r.percent(), 0);
    }
}
