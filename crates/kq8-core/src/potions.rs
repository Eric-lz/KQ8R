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
//! - The timer icon (rule 35): at the start the potion's picture takes
//!   the first free place of the 7 status icons (`IDCTL_TIMER_1` first,
//!   the right-most in the layout); it is freed at the end. The spinning
//!   mask (handsOff) takes a place the same way (STAND-IN, Q143: the note
//!   says it is an icon place, not that it is filled first-free; it fits
//!   the user's check that it sits in the bottom right corner).
//! - The red numbers (rules 37–39): while Strength runs the hand number
//!   is red, while the shield runs the armour number; the ranged one is
//!   never red.
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

/// How many status icon places there are (`IDCTL_TIMER_1`–`_7`).
pub const ICON_PLACES: usize = 7;

/// What a status icon place shows.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StatusIcon {
    /// A running potion, by its item object's name.
    Potion(String),
    /// The spinning mask (handsOff, `interface.md` rule 15).
    Mask,
}

/// Rule 35: `icon` takes the first free place; `None` when all 7 are
/// taken (or it already has one: then that one).
pub fn take_place(places: &mut [Option<StatusIcon>; ICON_PLACES], icon: StatusIcon) -> Option<usize> {
    if let Some(k) = places.iter().position(|p| p.as_ref() == Some(&icon)) {
        return Some(k);
    }
    let k = places.iter().position(Option::is_none)?;
    places[k] = Some(icon);
    Some(k)
}

/// Frees the place of `icon`; returns it.
pub fn free_place(places: &mut [Option<StatusIcon>; ICON_PLACES], icon: &StatusIcon) -> Option<usize> {
    let k = places.iter().position(|p| p.as_ref() == Some(icon))?;
    places[k] = None;
    Some(k)
}

/// Rules 37, 39: which slot numbers are red now (armour, hand, ranged).
pub fn red_numbers(running: &[Running]) -> [bool; 3] {
    let on = |e: Effect| running.iter().any(|r| r.effect == e);
    [on(Effect::Shield), on(Effect::Strength), false]
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
    /// Its status icon place (0 = `IDCTL_TIMER_1`), `None` if all were
    /// taken.
    pub place: Option<usize>,
}

impl Running {
    pub fn new(item: &str, kind: &str, duration: i64, now_update: u64) -> Running {
        Running { item: item.to_string(), kind: kind.to_string(), effect: Effect::of(kind), duration, left: duration, next_tick: now_update + crate::UPDATES_PER_SECOND as u64, place: None }
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

    #[test]
    fn icons_take_the_first_free_place() {
        let mut places: [Option<StatusIcon>; ICON_PLACES] = Default::default();
        assert_eq!(take_place(&mut places, StatusIcon::Potion("a".into())), Some(0));
        assert_eq!(take_place(&mut places, StatusIcon::Mask), Some(1));
        assert_eq!(take_place(&mut places, StatusIcon::Mask), Some(1), "already has one");
        assert_eq!(free_place(&mut places, &StatusIcon::Potion("a".into())), Some(0));
        assert_eq!(take_place(&mut places, StatusIcon::Potion("b".into())), Some(0), "the freed one is first");
        let s = Running::new("s", "INVITEM_Strength", 20, 0);
        let v = Running::new("v", "INVITEM_Invulnerable", 20, 0);
        assert_eq!(red_numbers(&[s.clone()]), [false, true, false]);
        assert_eq!(red_numbers(&[s, v]), [true, true, false]);
    }
}
