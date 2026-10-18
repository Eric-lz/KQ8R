//! Connor's turn keys as events (M4 4a-7; `connor-movement.md` "Turning"
//! rule 2, corrected 2026-10-18 by Q104 a and checked by the user in the
//! original).
//!
//! The turn keys reach Connor as events, not as a state read every update:
//! a key going **down** sets the turn direction (+1 left, −1 right) and is
//! the only thing that cancels an automatic turn; a key coming **up** sets
//! it to 0, except that letting go of one key while the other is still
//! held sends the held one again as a fresh down (which cancels too). When
//! an automatic turn ends the direction is set to 0, and nothing sends the
//! held keys again, so a key held through the turn does not take over.

/// The keys held (oldest first) and the turn direction Connor has.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct TurnKeys {
    /// Physically held turn keys (+1 left, −1 right), oldest first.
    held: Vec<i8>,
    /// The turn direction the key events gave him (+1, −1 or 0).
    pub turn: i8,
}

/// What a key event did, for the caller's log and the automatic turn.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum KeyEvent {
    /// A down reached Connor: direction set; cancels an automatic turn.
    Down(i8),
    /// A down swallowed by `handsOff` (`anim-scripts.md` rule 21).
    Swallowed(i8),
    /// The key came up: direction 0 (an up never cancels).
    Up,
    /// The key came up while the other one is held: the held one is sent
    /// again as a down (cancels; `Swallowed` instead during `handsOff`).
    UpOtherSentAgain(i8),
}

impl KeyEvent {
    /// A down that reached Connor (cancels an automatic turn).
    pub fn is_down(self) -> bool {
        matches!(self, KeyEvent::Down(_) | KeyEvent::UpOtherSentAgain(_))
    }
}

fn side(dir: i8) -> &'static str {
    if dir > 0 { "left" } else { "right" }
}

impl std::fmt::Display for KeyEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            KeyEvent::Down(d) => write!(f, "{} down", side(d)),
            KeyEvent::Swallowed(d) => write!(f, "{} down: swallowed (handsOff)", side(d)),
            KeyEvent::Up => write!(f, "up: turn direction 0"),
            KeyEvent::UpOtherSentAgain(d) => write!(f, "up; {} still held: sent again as down", side(d)),
        }
    }
}

impl TurnKeys {
    /// Turn key `dir` went down. `hands_off`: the press is swallowed (the
    /// key still counts as held, for its release).
    pub fn press(&mut self, dir: i8, hands_off: bool) -> KeyEvent {
        self.held.retain(|&d| d != dir);
        self.held.push(dir);
        if hands_off {
            return KeyEvent::Swallowed(dir);
        }
        self.turn = dir;
        KeyEvent::Down(dir)
    }

    /// Turn key `dir` came up (releases count during `handsOff` too).
    pub fn release(&mut self, dir: i8, hands_off: bool) -> KeyEvent {
        self.held.retain(|&d| d != dir);
        self.turn = 0;
        match self.held.last() {
            Some(&other) if hands_off => KeyEvent::Swallowed(other),
            Some(&other) => {
                self.turn = other;
                KeyEvent::UpOtherSentAgain(other)
            }
            None => KeyEvent::Up,
        }
    }

    /// The automatic turn ended normally: direction 0; held keys are not
    /// sent again.
    pub fn auto_turn_ended(&mut self) {
        self.turn = 0;
    }

    /// The turn key held last, if any (sent again when `handsOff` goes
    /// off, `anim-scripts.md` rule 22).
    pub fn held_last(&self) -> Option<i8> {
        self.held.last().copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_press_sets_the_direction_and_a_release_clears_it() {
        let mut k = TurnKeys::default();
        assert_eq!(k.press(1, false), KeyEvent::Down(1));
        assert_eq!(k.turn, 1);
        assert_eq!(k.release(1, false), KeyEvent::Up);
        assert_eq!(k.turn, 0);
        assert!(!KeyEvent::Up.is_down());
    }

    #[test]
    fn after_an_automatic_turn_a_held_key_does_not_take_over() {
        let mut k = TurnKeys::default();
        k.press(1, false);
        k.auto_turn_ended();
        assert_eq!(k.turn, 0);
        assert_eq!(k.held_last(), Some(1));
        // Only letting go and pressing again turns him.
        assert_eq!(k.release(1, false), KeyEvent::Up);
        assert!(k.press(1, false).is_down());
        assert_eq!(k.turn, 1);
    }

    #[test]
    fn letting_go_of_one_key_sends_the_other_again_as_a_down() {
        let mut k = TurnKeys::default();
        k.press(1, false);
        k.press(-1, false);
        assert_eq!(k.turn, -1);
        let e = k.release(-1, false);
        assert_eq!(e, KeyEvent::UpOtherSentAgain(1));
        assert!(e.is_down());
        assert_eq!(k.turn, 1);
    }

    #[test]
    fn hands_off_swallows_presses_but_not_releases() {
        let mut k = TurnKeys::default();
        assert_eq!(k.press(1, true), KeyEvent::Swallowed(1));
        assert_eq!(k.turn, 0);
        k.press(-1, false);
        assert_eq!(k.release(-1, true), KeyEvent::Swallowed(1));
        assert_eq!(k.turn, 0);
        assert!(!KeyEvent::Swallowed(1).is_down());
    }
}
