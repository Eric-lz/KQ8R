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
//! - The shield (rules 17–20): protection → p + ⌊p/2⌋, at least 50, at
//!   most 100. **Do not reproduce 1**: at the end we take off exactly what
//!   it added (the original puts back the value from before, losing
//!   armour changes made meanwhile); cut to 0–100. In the last temple
//!   (`setConnorFlag InTemple4 1`) each hit uses old + ⌊(raised − old)/2⌋,
//!   at most 85; the number shown stays. The orbit `orbit1.kq` is drawn
//!   by the viewer from [`orbit_place`].
//!
//! - Strength (rules 21–25): the hand number + ⌊hand/2⌋, that half
//!   remembered and taken off at the end; ranged unchanged. The look
//!   (iron skin, face) is the viewer's, from [`list_bits`].
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

/// Rule 17: the shield's lowest and highest protection.
pub const SHIELD_MIN: i64 = 50;
pub const SHIELD_MAX: i64 = 100;
/// Rule 19: the most a hit's protection can be in the last temple.
pub const TEMPLE4_CAP: i64 = 85;
/// Rule 20: the orbit's centre is Connor's `body` part lowered by this
/// (then raised by the file's `orbitZoffset`).
pub const ORBIT_DROP: f32 = 820.0;

/// Rule 17: protection `p` under the shield.
pub fn shield_raise(p: i64) -> i64 {
    (p + p.div_euclid(2)).clamp(SHIELD_MIN, SHIELD_MAX)
}

/// Rule 19: a hit's protection in the last temple, from the protection
/// now (`raised`) and what the shield added.
pub fn temple4_protection(raised: i64, added: i64) -> i64 {
    let old = raised - added;
    (old + added.div_euclid(2)).min(TEMPLE4_CAP)
}

/// Rule 20: the orbit's place at `angle` (radians about the vertical)
/// around Connor's `body` part at `body`: `radius` out in the ground
/// plane, at `body` − 820 + `z_offset` height. Angle 0 is +x (our
/// reading: `orbitStartAngZ` 0).
pub fn orbit_place(body: [f32; 3], radius: f32, z_offset: f32, angle: f32) -> [f32; 3] {
    [body[0] + radius * angle.cos(), body[1] + radius * angle.sin(), body[2] - ORBIT_DROP + z_offset]
}

/// Rule 13: the effect switches on at a potion's start. Returns what it
/// added to remember (the shield's protection) and the log text.
pub fn effect_on(w: &mut crate::active::WorldState, effect: Effect) -> (i64, String) {
    let Some(c) = w.connor_stats.as_mut() else { return (0, "Connor's stats unknown: no effect".into()) };
    match effect {
        Effect::Shield => {
            let p = c.stats.0[3];
            let raised = shield_raise(p);
            c.stats.0[3] = raised;
            (raised - p, format!("the shield: protection {p} -> {raised} (p + p/2, 50 to 100; potions.md rule 17)"))
        }
        Effect::Strength => {
            let h = c.stats.0[1];
            let half = h.div_euclid(2);
            c.stats.0[1] = h + half;
            (half, format!("strength: hand number {h} -> {} (+ half, remembered; ranged unchanged; potions.md rule 21)", c.stats.0[1]))
        }
        _ => (0, String::new()),
    }
}

/// Rule 36: the effect switches off at the end of `r`.
pub fn effect_off(w: &mut crate::active::WorldState, r: &Running) -> String {
    let Some(c) = w.connor_stats.as_mut() else { return String::new() };
    match r.effect {
        Effect::Shield => {
            let p = c.stats.0[3];
            c.stats.0[3] = (p - r.added).clamp(0, crate::stats::FOURTH_CAP);
            format!("the shield ends: protection {p} -> {} (the {} it added taken off: potions.md 'Do not reproduce' 1)", c.stats.0[3], r.added)
        }
        Effect::Strength => {
            let h = c.stats.0[1];
            c.stats.0[1] = h - r.added;
            format!("strength ends: hand number {h} -> {} (the remembered {} taken off; potions.md rule 22)", c.stats.0[1], r.added)
        }
        _ => String::new(),
    }
}

/// The bits of a `KQSkinList` (`kq8_data::characters::SkinList`) for
/// Connor now: item N's bit (2^N) is set while he has an item of that
/// type applied (worn, or a potion running; `potions.md` rules 25, 30).
pub fn list_bits(w: &crate::active::WorldState, items: &[String]) -> u32 {
    items
        .iter()
        .enumerate()
        .filter(|(_, kind)| w.items.objects.iter().any(|o| o.carried_by(crate::talk::CONNOR) && o.third != -1 && o.kind.eq_ignore_ascii_case(kind)))
        .fold(0, |bits, (n, _)| bits | (1 << n))
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
    /// What the effect added and takes off at the end (the shield's
    /// protection).
    pub added: i64,
}

impl Running {
    pub fn new(item: &str, kind: &str, duration: i64, now_update: u64) -> Running {
        Running { item: item.to_string(), kind: kind.to_string(), effect: Effect::of(kind), duration, left: duration, next_tick: now_update + crate::UPDATES_PER_SECOND as u64, place: None, added: 0 }
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
    fn the_shield_numbers() {
        // Rule 17's examples.
        let got: Vec<i64> = [0, 10, 34, 40, 66, 80].iter().map(|&p| shield_raise(p)).collect();
        assert_eq!(got, vec![50, 50, 51, 60, 99, 100]);
        // Rule 19: 40 -> 60 (added 20): a hit uses 40 + 10 = 50; 80 -> 100:
        // 80 + 10 = 90, cut to 85.
        assert_eq!(temple4_protection(60, 20), 50);
        assert_eq!(temple4_protection(100, 20), 85);
        let o = orbit_place([100.0, 200.0, 1000.0], 500.0, 500.0, 0.0);
        assert_eq!(o, [600.0, 200.0, 680.0]);
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
        assert_eq!(red_numbers(std::slice::from_ref(&s)), [false, true, false]);
        assert_eq!(red_numbers(&[s, v]), [true, true, false]);
    }
}
