//! Characters' states (M4 step 4d-3): what a `KQMonster` or
//! `KQGeometryTalker` does on top of the movement player every animated
//! object has (`crate::named_anim`).
//!
//! Rules from `docs/behaviour/characters.md` (session A, 2026-10-19):
//!
//! - Rule 11: a character updates only while near or far (not off, not
//!   hidden); the caller says which ([`Character::in_range`]).
//! - Rule 12: the first update. A waiting first movement with no current
//!   name starts and the character stays in **no state** (12.1, rule 15);
//!   otherwise the current movement plays and the character goes into
//!   Birth (its list has `Birth`) or into Wait `birthDelay` seconds later,
//!   through the event queue (12.2): at the next update at the earliest.
//! - Rule 13: Wait asks for `Stop` the ordinary way; with `flags` bit 0x10
//!   it starts at a random frame and speed 0.9–1.1.
//! - Rule 18: in Wait, a play-once movement that ends (other than `Stop`
//!   or a `gesture…`) is followed by Wait asking for `Stop` again, always
//!   at a random frame and speed. Rule 19: no such return in Talk.
//! - Rule 22: `setState` by name (capitals ignored); `Special <list>` as
//!   Connor's (`anim-scripts.md` rule 16), then Wait.
//!
//! - Rule 23 (4d-4): a turn request (`speak` with a listener) turns the
//!   character toward the point with Connor's automatic turn, 3.5 rad/s
//!   the short way, landing exactly once less than one update's turn is
//!   left; a newer request replaces the target; `noTurn` ignores it. No
//!   turning movement plays: it turns on the spot, its movement going on.
//!   The heading is read from its rotation the way Connor's is (heading
//!   0 faces −Y, `crate::connor_auto::heading_towards`).
//!
//! - Rules 25–28 (4d-5): a lip-sync code to a character. Codes 0–7 set the
//!   mouth shape (the face texture, drawn by the viewer; a
//!   `KQGeometryTalker` instead **starts** `rest f m r w ih ah e` at once);
//!   8–15 ask for `gesturea`…`gestureh`, 16 for `stop` (a geometry
//!   talker: 16 starts `rest`), only when not walking and the movement
//!   playing can be cut short or has ended, and the list has it; above
//!   16 nothing. A line's end puts the mouth shape back to 0.
//!
//! - The Talk state (M4 4e-3, `conversation.md` rules 2, 15–17,
//!   `characters.md` rules 19, 24): entered at a conversation's start, its
//!   current movement marked "can be cut short" and `Stop` asked for; every
//!   4th update it asks to turn toward Connor's current position (unless
//!   `noTurn`); nothing goes back to `Stop` by itself; at the end, Wait.
//!
//! Not here (later steps): gravity (4d-6); noticing Connor, wandering,
//! idling, Move, Die (M5).

use std::collections::VecDeque;

use crate::emitter::Rng;
use crate::named_anim::{NamedAnimPlayer, Played, Request};

/// What the character is doing (characters.md "Words used here").
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum State {
    /// No state: plays only what it is told (rule 15).
    None,
    Birth,
    Wait,
    Move,
    Special,
    Talk,
    Die,
}

/// The state names `setState` knows that only Connor has (rule 22).
const CONNOR_ONLY: [&str; 10] = ["SwapWeapons", "ThrowRock", "Jump", "Push", "Tile", "Tile2", "Tile3", "Scales", "Choice", "Portal"];

/// The movement Wait asks for (rule 13); looked up ignoring capitals.
pub const STOP: &str = "Stop";

/// In Talk, a turn toward Connor is asked for every this many updates
/// (`conversation.md` rule 17).
pub const TALK_TURN_EVERY: u64 = 4;

/// A character's own values and state.
#[derive(Debug, Clone)]
pub struct Character {
    /// `KQGeometryTalker` (rule 27; used from 4d-5).
    pub geometry: bool,
    /// `flags` bit 0x10 (rule 13).
    pub random_stop: bool,
    /// Seconds from the first update to Wait (rule 12).
    pub birth_delay: f32,
    /// Its list has `Birth` (rule 16).
    pub has_birth: bool,
    /// `noTurn` / `setCanTurn` (rule 23; used from 4d-4).
    pub no_turn: bool,
    /// Gravity on (rule 8; used from 4d-6).
    pub gravity: bool,
    pub state: State,
    /// Near or far now (rule 11), set by the caller before each update.
    pub in_range: bool,
    /// The update number of its first update (rule 12), once it happened.
    pub first_update: Option<u64>,
    /// Game time at which Wait begins (rule 12.2).
    wait_at: Option<f64>,
    /// `Stop` gets the random frame and speed when it next starts (it was
    /// asked for while the running movement could not be cut short).
    random_pending: bool,
    /// Special list items not handled yet, and the movement it waits for.
    special: VecDeque<String>,
    special_playing: Option<String>,
    rng: Rng,
    /// The point it is turning to (rule 23), if any.
    pub turn_to: Option<[f32; 2]>,
    /// The mouth shape (rule 25): 0–7, 0 at the start and after a line.
    pub mouth: u16,
    /// Where Connor stands now, set by the caller before each update
    /// (the Talk state's turn, `conversation.md` rule 17).
    pub connor_at: Option<[f32; 2]>,
}

/// What one update did: the player's step (`None` when not updated) and
/// lines for the log.
#[derive(Debug, Default)]
pub struct Step {
    pub played: Option<Played>,
    pub notes: Vec<String>,
}

impl Character {
    /// A character as loaded; `seed` makes its random draws its own.
    pub fn new(geometry: bool, flags: u32, birth_delay: f32, has_birth: bool, no_turn: bool, gravity: bool, seed: u64) -> Self {
        Character {
            geometry,
            random_stop: flags & kq8_data::characters::RANDOM_STOP != 0,
            birth_delay,
            has_birth,
            no_turn,
            gravity,
            state: State::None,
            in_range: false,
            first_update: None,
            wait_at: None,
            random_pending: false,
            special: VecDeque::new(),
            special_playing: None,
            rng: Rng::new(seed),
            turn_to: None,
            mouth: 0,
            connor_at: None,
        }
    }

    /// A seed from the character's name (so two goblins differ, and a run
    /// is the same every time).
    pub fn seed_for(name: &str) -> u64 {
        name.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| (h ^ u64::from(b.to_ascii_lowercase())).wrapping_mul(0x0100_0000_01B3))
    }

    /// One update (rule 11: nothing at all while off or hidden).
    pub fn update(&mut self, player: &mut NamedAnimPlayer, now: f64, dt: f32, update: u64) -> Step {
        let mut step = Step::default();
        if !self.in_range {
            return step;
        }
        let notes = &mut step.notes;
        // Rule 12.2's Wait, through the queue: never in the first update.
        if self.wait_at.is_some_and(|t| now >= t) {
            self.wait_at = None;
            notes.push(format!("Wait begins (rule 12.2, birthDelay {} s)", self.birth_delay));
            self.enter_wait(player, notes);
        }
        if self.first_update.is_none() {
            self.first_update = Some(update);
            self.first(player, now, notes);
        }
        // conversation.md rule 17: in Talk, every 4th update a turn toward
        // Connor's current position (noTurn blocks it, characters.md rule 24).
        if self.state == State::Talk && !self.no_turn && update.is_multiple_of(TALK_TURN_EVERY) {
            if let Some(c) = self.connor_at {
                if self.turn_to.replace(c).is_none() {
                    notes.push(format!("Talk: turns toward Connor at ({:.0}, {:.0}) (every {TALK_TURN_EVERY}th update, rule 17)", c[0], c[1]));
                }
            }
        }
        self.turn_step(player, dt, notes);
        let played = player.advance(dt);
        if let Some(st) = &played.started {
            if self.random_pending && st.name.eq_ignore_ascii_case(STOP) {
                self.random_pending = false;
                let (frame, speed) = player.randomize(&mut self.rng);
                notes.push(format!("'{}' started after the movement before it ended: random frame {frame}, speed {speed:.3} (rule 13)", st.name));
            }
        }
        if let Some(m) = &played.finished {
            self.after_end(player, m, notes);
        }
        step.played = Some(played);
        step
    }

    /// A lip-sync code for this character (rules 25–28). Returns the log
    /// line.
    pub fn mouth_code(&mut self, player: &mut NamedAnimPlayer, code: u16) -> String {
        use kq8_data::characters::{GEOMETRY_MOUTH, GESTURES};
        if self.geometry {
            // Rule 27: starts at once, replacing whatever plays.
            let name = match code {
                0..=7 => GEOMETRY_MOUTH[code as usize],
                16 => GEOMETRY_MOUTH[0],
                // "Do not reproduce": the original starts a leftover name.
                _ => return format!("code {code}: ignored for a geometry talker (Do not reproduce)"),
            };
            let st = player.start(name);
            let unknown = if st.unknown { " (not in its list: movement 0)" } else { "" };
            return format!("code {code}: mouth movement '{name}' starts at once{unknown} (rule 27)");
        }
        let gesture = match code {
            0..=7 => {
                self.mouth = code;
                return format!("code {code}: mouth shape {code} (rule 25)");
            }
            8..=15 => GESTURES[(code - 8) as usize],
            16 => "stop",
            _ => return format!("code {code}: above 16, nothing (conversation.md rule 18)"),
        };
        // Rule 28: not walking (none walk before M5), the movement playing
        // can be cut short or has ended, and the list has the gesture.
        if !player.movements.iter().any(|m| m.name.eq_ignore_ascii_case(gesture)) {
            return format!("code {code}: no '{gesture}' in its list: nothing (rule 28)");
        }
        if !(player.cut_short || player.has_ended()) {
            return format!("code {code}: '{gesture}' not played: '{}' can't be cut short and hasn't ended (rule 28)", player.movement_name());
        }
        match player.request(gesture) {
            Request::AtOnce(_) => format!("code {code}: gesture '{gesture}' starts (rule 28)"),
            _ => format!("code {code}: gesture '{gesture}' waits (rule 28)"),
        }
    }

    /// The end of the last line of its queue (Q117 stand-in): Wait, from
    /// no state or Wait only. Returns the log line.
    pub fn after_speech(&mut self, player: &mut NamedAnimPlayer) -> String {
        match self.state {
            State::None | State::Wait => {
                let mut notes = Vec::new();
                self.enter_wait(player, &mut notes);
                format!("last line of its queue: the speaker goes to Wait (characters.md rules 15, 27; Q117 stand-in): {}", notes.join("; "))
            }
            s => format!("last line of its queue: stays in {s:?} (Q117 stand-in: Wait only from no state or Wait)"),
        }
    }

    /// A conversation starts with this character as the partner
    /// (`conversation.md` rule 2): its current movement is marked "can be
    /// cut short", `Stop` is asked for and it goes into Talk. Returns the
    /// log line.
    pub fn enter_talk(&mut self, player: &mut NamedAnimPlayer) -> String {
        let from = self.state;
        self.state = State::Talk;
        self.wait_at = None;
        self.special.clear();
        self.special_playing = None;
        player.cut_short = true;
        let stop = match player.request(STOP) {
            Request::AtOnce(st) => format!("Stop starts at once{}", if st.unknown { " (no Stop in its list: movement 0)" } else { "" }),
            _ => format!("Stop waits for '{}'", player.movement_name()),
        };
        let turn = if self.no_turn { "; noTurn: it will not turn toward Connor (characters.md rule 24)" } else { "" };
        format!("state Talk (from {from:?}): '{}' marked can be cut short; {stop}{turn}", player.movement_name())
    }

    /// The conversation ended (`conversation.md` rules 15–16): Wait.
    /// Returns the log line (nothing from another state).
    pub fn leave_talk(&mut self, player: &mut NamedAnimPlayer) -> Option<String> {
        if self.state != State::Talk {
            return None;
        }
        self.turn_to = None;
        let mut notes = Vec::new();
        self.enter_wait(player, &mut notes);
        Some(format!("the conversation ended: {}", notes.join("; ")))
    }

    /// A turn request toward `to` (rule 23). Returns what happened, for
    /// the log.
    pub fn ask_turn(&mut self, to: [f32; 2]) -> &'static str {
        if self.no_turn {
            return "blocked: noTurn (rule 23), the request is ignored";
        }
        let replaced = self.turn_to.replace(to).is_some();
        if replaced { "turns toward it (replacing the turn under way)" } else { "turns toward it at 3.5 rad/s (rule 23)" }
    }

    /// One update of the turn (rule 23): the rotation turns about the
    /// world's z by at most 3.5 rad/s × dt.
    fn turn_step(&mut self, player: &mut NamedAnimPlayer, dt: f32, notes: &mut Vec<String>) {
        use crate::connor_auto::{heading_towards, wrap, AUTO_TURN_RATE};
        let Some(to) = self.turn_to else { return };
        let r = player.base.rotation;
        let heading = heading_of(&r);
        let left = wrap(heading_towards(player.base.position, to) - heading);
        let step = AUTO_TURN_RATE * dt;
        let by = if left.abs() < step {
            self.turn_to = None;
            left
        } else {
            step * left.signum()
        };
        player.base.rotation = times(&r, &kq8_data::objects::rotation([0.0, 0.0, by]));
        let done = if self.turn_to.is_none() { "; turn done" } else { "" };
        notes.push(format!("turning to ({:.0}, {:.0}): heading {heading:.3} -> {:.3} (by {by:.3}){done}", to[0], to[1], heading_of(&player.base.rotation)));
    }

    /// Rule 12: the first update, before the player's step.
    fn first(&mut self, player: &mut NamedAnimPlayer, now: f64, notes: &mut Vec<String>) {
        let n = self.first_update.unwrap_or(0);
        if self.state != State::None {
            // Q121 stand-in: a state set before the first update is kept.
            notes.push(format!("first update {n}: already in {:?} (set by a script before): kept, nothing restarted (Q121 stand-in)", self.state));
            return;
        }
        if player.current_name.is_empty() {
            if let Some(w) = &player.waiting {
                // Rule 12.1 (rule 15, "Do not reproduce": kept, logged once).
                notes.push(format!("first update {n}: rule 12.1: no current movement, the waiting '{w}' starts; NO STATE from now on until something sets one (rule 15)"));
                return;
            }
        }
        // Rule 12.2: movement 0 *starts* here when no name was given, so it
        // is now the current movement by name (rule 14: Wait's `Stop` then
        // waits for its end if it can't be cut short).
        if player.current_name.is_empty() {
            player.current_name = player.movement().map_or(String::new(), |m| m.name.clone());
        }
        notes.push(format!(
            "first update {n}: rule 12.2: movement '{}' ({}) at phase {:.4} speed {:.3}",
            player.movement_name(),
            player.movement().map_or("none", |m| m.name.as_str()),
            player.phase,
            player.speed()
        ));
        if self.has_birth {
            self.state = State::Birth;
            notes.push("its list has Birth: Birth state, not drawn until it rises (rule 16; rising is M5)".into());
        } else {
            self.wait_at = Some(now + f64::from(self.birth_delay.max(0.0)));
        }
    }

    /// Wait (rule 13): asks for `Stop`, random by `flags` bit 0x10.
    fn enter_wait(&mut self, player: &mut NamedAnimPlayer, notes: &mut Vec<String>) {
        self.state = State::Wait;
        self.special.clear();
        self.special_playing = None;
        let random = self.random_stop;
        let why = if random { "flags 0x10: random" } else { "no flags 0x10: from its loop= line" };
        notes.push(format!("state Wait: asks for Stop ({why}): {}", self.ask_stop(player, random)));
    }

    /// Asks for `Stop` the ordinary way (`anim-scripts.md` rule 4).
    fn ask_stop(&mut self, player: &mut NamedAnimPlayer, random: bool) -> String {
        match player.request(STOP) {
            Request::AtOnce(st) => {
                let unknown = if st.unknown { " (no Stop in its list: movement 0)" } else { "" };
                if random {
                    let (frame, speed) = player.randomize(&mut self.rng);
                    format!("starts at once{unknown} at random frame {frame}, speed {speed:.3}")
                } else {
                    format!("starts at once{unknown} at phase {:.4} speed {}", st.start, st.speed)
                }
            }
            Request::Waiting => {
                self.random_pending = random;
                format!("waits for '{}' to end", player.movement_name())
            }
            Request::WaitingForever => {
                self.random_pending = random;
                format!("waits for '{}', which is stopped and can't be cut short: never (anim-scripts.md rule 6)", player.movement_name())
            }
        }
    }

    /// After the ordinary end of a play-once movement (rules 18, 22).
    fn after_end(&mut self, player: &mut NamedAnimPlayer, ended: &str, notes: &mut Vec<String>) {
        match self.state {
            State::Wait => {
                let lower = ended.to_ascii_lowercase();
                if lower == "stop" || lower.starts_with("gesture") {
                    notes.push(format!("'{ended}' ended in Wait: no return to Stop for Stop or a gesture (rule 18)"));
                } else {
                    notes.push(format!("'{ended}' ended in Wait: asks for Stop again (rule 18, always random): {}", self.ask_stop(player, true)));
                }
            }
            State::Special if self.special_playing.as_deref().is_some_and(|p| p.eq_ignore_ascii_case(ended)) => {
                self.special_playing = None;
                notes.push(format!("Special: '{ended}' ended; next item"));
                self.special_next(player, notes);
            }
            State::None => notes.push(format!("'{ended}' ended in no state: holds its last frame (rule 15)")),
            s => notes.push(format!("'{ended}' ended in {s:?}: nothing more")),
        }
    }

    /// `KQMonster::setState <name> <state> [words…]` (rule 22). Returns
    /// the log line.
    pub fn set_state(&mut self, player: &mut NamedAnimPlayer, state: &str, words: &[String]) -> String {
        let mut notes = Vec::new();
        if state.eq_ignore_ascii_case("Wait") {
            self.wait_at = None;
            self.enter_wait(player, &mut notes);
        } else if state.eq_ignore_ascii_case("Special") {
            // The words are joined with spaces into one list (rule 22).
            let list = words.join(" ");
            self.state = State::Special;
            self.wait_at = None;
            self.special = list.split(',').map(str::trim).filter(|w| !w.is_empty()).map(str::to_string).collect();
            self.special_playing = None;
            notes.push(format!("state Special, list '{list}' ({} items)", self.special.len()));
            self.special_next(player, &mut notes);
        } else if state.eq_ignore_ascii_case("Unborn") {
            self.state = State::Birth;
            self.wait_at = None;
            notes.push("state Birth (Unborn); hiding a character that was drawn is not done (none in Daventry)".into());
        } else if state.eq_ignore_ascii_case("Move") || state.eq_ignore_ascii_case("Die") {
            notes.push(format!("{state}: walking and dying are M5; state unchanged ({:?})", self.state));
        } else if CONNOR_ONLY.iter().any(|s| s.eq_ignore_ascii_case(state)) {
            // "Do not reproduce": the original leaves it in an empty state.
            notes.push(format!("{state} is a state only Connor has: logged and ignored (Do not reproduce); state stays {:?}", self.state));
        } else {
            notes.push(format!("cannot find state '{state}'; nothing"));
        }
        notes.join("; ")
    }

    /// The next items of a Special list (`anim-scripts.md` rule 16): a
    /// movement starts at once and handling waits for its end; the list
    /// used up → Wait.
    fn special_next(&mut self, player: &mut NamedAnimPlayer, notes: &mut Vec<String>) {
        while let Some(item) = self.special.pop_front() {
            if item.to_ascii_lowercase().starts_with("animlist") {
                notes.push(format!("Special item '{item}': other movement lists for characters are not built (none in Daventry); skipped"));
                continue;
            }
            if item.eq_ignore_ascii_case("Inv") {
                let what = self.special.pop_front().unwrap_or_default();
                notes.push(format!("Special item 'Inv {what}': not traced; skipped"));
                continue;
            }
            if item.eq_ignore_ascii_case("KillConnor") {
                notes.push("Special item 'KillConnor' on a character: skipped (Connor's only)".into());
                continue;
            }
            let st = player.start(&item);
            let unknown = if st.unknown { " (not in its list: movement 0 plays under that name, anim-scripts.md rule 5)" } else { "" };
            notes.push(format!("Special: '{item}' starts at once{unknown}; waits for its end"));
            self.special_playing = Some(item);
            return;
        }
        notes.push("Special list used up".into());
        self.enter_wait(player, notes);
    }
}

/// The heading a rotation faces, as Connor's: where its −Y goes, as
/// `heading_towards` measures it (0 faces −Y, π/2 faces +X).
pub fn heading_of(r: &[[f32; 3]; 3]) -> f32 {
    // Row vectors: (0, −1, 0) × R = −(second row).
    let (x, y) = (-r[1][0], -r[1][1]);
    x.atan2(-y)
}

/// a × b (row-vector rotations: a first, then b).
fn times(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::named_anim::tests::movement;

    /// attack (one-shot, can't be cut short), stop (repeats, can be cut
    /// short), caw (one-shot), gesturea (one-shot).
    fn player(current: usize, curr: Option<&str>, next: Option<&str>) -> NamedAnimPlayer {
        let ms = vec![
            movement("attack", 0.0, 1.0, false, false, vec![]),
            movement("stop", 0.0, 1.0, true, true, vec![]),
            movement("caw", 0.0, 1.0, false, false, vec![]),
            movement("gesturea", 0.0, 1.0, false, true, vec![]),
        ];
        let base = crate::collide::Placement { rotation: crate::door::identity().m, position: [0.0; 3] };
        NamedAnimPlayer::new("raven", ms, current, base).with_names_at_load(curr, next)
    }

    fn character(flags: u32) -> Character {
        let mut c = Character::new(false, flags, 0.0, false, false, true, 7);
        c.in_range = true;
        c
    }

    /// Runs updates until `done` or `limit`.
    fn run(c: &mut Character, p: &mut NamedAnimPlayer, from: u64, limit: u64, log: &mut Vec<String>) -> u64 {
        let mut n = from;
        while n < from + limit {
            let s = c.update(p, n as f64 / 15.0, 1.0 / 15.0, n);
            log.extend(s.notes);
            n += 1;
        }
        n
    }

    #[test]
    fn mouth_codes_shape_gesture_or_nothing() {
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        assert!(c.mouth_code(&mut p, 5).contains("mouth shape 5"));
        assert_eq!(c.mouth, 5);
        // stop can be cut short: gesturea plays.
        let line = c.mouth_code(&mut p, 8);
        assert!(line.contains("starts"), "{line}");
        assert_eq!(p.movement_name(), "gesturea");
        assert!(c.mouth_code(&mut p, 9).contains("no 'gestureb'"));
        assert!(c.mouth_code(&mut p, 17).contains("nothing"));
        // attack can't be cut short and hasn't ended: no gesture.
        p.start("attack");
        assert!(c.mouth_code(&mut p, 16).contains("not played"));
        assert_eq!(c.mouth, 5, "gesture codes leave the mouth alone");
    }

    #[test]
    fn a_geometry_talker_starts_mouth_movements_at_once() {
        let mut c = Character::new(true, 0x20, 0.0, false, true, true, 1);
        let mut p = player(0, None, None);
        // attack can't be cut short, yet the mouth movement replaces it.
        let line = c.mouth_code(&mut p, 2);
        assert!(line.contains("'m' starts at once"), "{line}");
        assert_eq!(p.movement_name(), "m");
        c.mouth_code(&mut p, 16);
        assert_eq!(p.movement_name(), "rest");
        assert!(c.mouth_code(&mut p, 9).contains("ignored"));
        assert_eq!(c.mouth, 0);
    }

    #[test]
    fn heading_of_matches_dir_z() {
        for h in [0.0f32, 0.8, -1.57, 3.0] {
            let r = kq8_data::objects::rotation([0.0, 0.0, h]);
            assert!((crate::connor_auto::wrap(heading_of(&r) - h)).abs() < 1e-5, "{h}");
        }
    }

    #[test]
    fn a_turn_goes_at_3_5_per_second_and_lands_exactly() {
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        // Facing −Y; the point is at +X: a quarter turn (π/2).
        assert!(c.ask_turn([1000.0, 0.0]).contains("3.5"));
        let mut log = Vec::new();
        let mut n = 0;
        while c.turn_to.is_some() && n < 30 {
            let before = heading_of(&p.base.rotation);
            run(&mut c, &mut p, n, 1, &mut log);
            let by = crate::connor_auto::wrap(heading_of(&p.base.rotation) - before);
            assert!(by.abs() <= 3.5 / 15.0 + 1e-5);
            n += 1;
        }
        // π/2 at 3.5/15 per update: 7 updates (6 full steps and the rest).
        assert_eq!(n, 7, "{log:?}");
        assert!((heading_of(&p.base.rotation) - std::f32::consts::FRAC_PI_2).abs() < 1e-4);
        assert!(p.movement_name().eq_ignore_ascii_case("stop"), "no turning movement");
    }

    #[test]
    fn in_talk_it_turns_toward_connor_every_4th_update_and_wait_ends_it() {
        let mut c = character(0x20);
        let mut p = player(0, None, None);
        let line = c.enter_talk(&mut p);
        assert!(line.contains("can be cut short") && line.contains("Stop starts at once"), "attack made cuttable: {line}");
        assert_eq!(c.state, State::Talk);
        c.connor_at = Some([1000.0, 0.0]);
        let mut log = Vec::new();
        run(&mut c, &mut p, 1, 3, &mut log);
        assert!(c.turn_to.is_none(), "updates 1-3: no turn yet");
        run(&mut c, &mut p, 4, 1, &mut log);
        assert!(log.iter().any(|l| l.contains("every 4th update")), "{log:?}");
        // A no-turn partner never turns.
        let mut n = character(0x20);
        n.no_turn = true;
        n.enter_talk(&mut p);
        n.connor_at = Some([1000.0, 0.0]);
        run(&mut n, &mut p, 4, 1, &mut Vec::new());
        assert!(n.turn_to.is_none());
        assert!(c.leave_talk(&mut p).unwrap().contains("Wait"));
        assert_eq!(c.state, State::Wait);
        assert!(c.leave_talk(&mut p).is_none(), "only from Talk");
    }

    #[test]
    fn no_turn_blocks_turns() {
        let mut c = character(0x20);
        c.no_turn = true;
        assert!(c.ask_turn([1000.0, 0.0]).contains("blocked"));
        assert!(c.turn_to.is_none());
    }

    #[test]
    fn nothing_happens_while_off() {
        let mut c = character(0x10);
        c.in_range = false;
        let mut p = player(0, None, None);
        let s = c.update(&mut p, 0.0, 1.0 / 15.0, 0);
        assert!(s.played.is_none() && c.first_update.is_none() && p.phase == 0.0);
    }

    #[test]
    fn first_update_then_wait_asks_for_stop_when_movement_0_ends() {
        // A raven: movement 0 'attack' can't be cut short.
        let mut c = character(0x10);
        let mut p = player(0, None, None);
        let mut log = Vec::new();
        let s = c.update(&mut p, 0.0, 1.0 / 15.0, 5);
        log.extend(s.notes);
        assert_eq!(c.first_update, Some(5));
        assert_eq!(c.state, State::None, "Wait comes through the queue, not in the first update");
        run(&mut c, &mut p, 6, 1, &mut log);
        assert_eq!(c.state, State::Wait);
        assert_eq!(p.movement_name(), "attack");
        assert_eq!(p.waiting.as_deref(), Some(STOP));
        // The loop is 1 s long in the test movements: 15 updates.
        run(&mut c, &mut p, 7, 20, &mut log);
        assert!(p.movement_name().eq_ignore_ascii_case("stop"), "{log:?}");
        assert!(p.random_speed.is_some(), "Stop got the random speed: {log:?}");
        let joined = log.join("\n");
        assert!(joined.contains("rule 12.2") && joined.contains("waits for 'attack' to end"), "{joined}");
    }

    #[test]
    fn stop_starts_at_once_plain_without_flag_0x10() {
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        let mut log = Vec::new();
        run(&mut c, &mut p, 0, 2, &mut log);
        assert_eq!(c.state, State::Wait);
        assert!(p.random_speed.is_none(), "{log:?}");
        assert!(log.join("\n").contains("starts at once at phase"));
    }

    #[test]
    fn in_wait_a_one_shot_is_followed_by_stop_but_not_a_gesture() {
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 2, &mut log);
        p.request("caw");
        let n = run(&mut c, &mut p, n, 20, &mut log);
        assert!(p.movement_name().eq_ignore_ascii_case("stop"), "{log:?}");
        assert!(p.random_speed.is_some(), "rule 18 is always random");
        p.request("gesturea");
        run(&mut c, &mut p, n, 20, &mut log);
        assert_eq!(p.movement_name(), "gesturea", "a gesture holds its last frame");
        assert!(log.join("\n").contains("no return to Stop for Stop or a gesture"));
    }

    #[test]
    fn a_waiting_first_movement_means_no_state() {
        // The Shadow: no currAnimName, nextAnimName=stop.
        let mut c = character(0x10);
        let mut p = player(0, None, Some("stop"));
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 3, &mut log);
        assert_eq!(c.state, State::None);
        assert!(p.movement_name().eq_ignore_ascii_case("stop"));
        p.request("caw");
        run(&mut c, &mut p, n, 20, &mut log);
        assert_eq!(p.movement_name(), "caw", "no state: holds the last frame: {log:?}");
    }

    #[test]
    fn birth_list_means_birth_state() {
        let mut c = Character::new(false, 0x10, 0.0, true, false, true, 1);
        c.in_range = true;
        let mut p = player(0, None, None);
        let mut log = Vec::new();
        run(&mut c, &mut p, 0, 5, &mut log);
        assert_eq!(c.state, State::Birth);
    }

    #[test]
    fn birth_delay_puts_off_wait() {
        let mut c = Character::new(false, 0x20, 1.0, false, false, true, 1);
        c.in_range = true;
        let mut p = player(1, Some("stop"), None);
        let mut log = Vec::new();
        run(&mut c, &mut p, 0, 14, &mut log);
        assert_eq!(c.state, State::None);
        run(&mut c, &mut p, 14, 2, &mut log);
        assert_eq!(c.state, State::Wait);
    }

    #[test]
    fn special_lists_play_in_turn_then_wait() {
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 2, &mut log);
        let line = c.set_state(&mut p, "special", &["caw,attack".to_string()]);
        assert!(line.contains("'caw' starts at once"), "{line}");
        assert_eq!(c.state, State::Special);
        let n = run(&mut c, &mut p, n, 17, &mut log);
        assert_eq!(p.movement_name(), "attack", "{log:?}");
        run(&mut c, &mut p, n, 17, &mut log);
        assert_eq!(c.state, State::Wait, "{log:?}");
        assert!(p.movement_name().eq_ignore_ascii_case("stop"));
    }

    #[test]
    fn other_states_are_logged() {
        let mut c = character(0x10);
        let mut p = player(1, Some("stop"), None);
        assert!(c.set_state(&mut p, "jump", &[]).contains("only Connor has"));
        assert!(c.set_state(&mut p, "dance", &[]).contains("cannot find state"));
        assert!(c.set_state(&mut p, "die", &[]).contains("M5"));
        assert_eq!(c.state, State::None);
        c.set_state(&mut p, "WAIT", &[]);
        assert_eq!(c.state, State::Wait);
    }

    #[test]
    fn a_state_set_before_the_first_update_is_kept() {
        let mut c = character(0x10);
        let mut p = player(1, Some("stop"), None);
        c.set_state(&mut p, "special", &["caw".to_string()]);
        let mut log = Vec::new();
        run(&mut c, &mut p, 0, 3, &mut log);
        assert_eq!(c.state, State::Special);
        assert_eq!(p.movement_name(), "caw");
        assert!(log.join("\n").contains("Q121"));
    }
}
