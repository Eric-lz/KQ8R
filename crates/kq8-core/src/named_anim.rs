//! Plays an object's named animations when asked (M3 steps 3e-4, 3e-5,
//! 3i-1): `open`, `closed`, `stop`... (the game's files call them
//! "movements"; this is not about Connor moving). Which one is shown, its
//! phase, the movement waiting to start, one update's step, the frame
//! shown and where the object stands.
//!
//! Used by doors (`crate::door`, with their four states on top), chests
//! and scripted animated objects. (Animations that simply loop from the
//! start are `crate::anim`.)
//!
//! Rules from `docs/behaviour/anim-scripts.md` (traced unless marked):
//!
//! - The current movement's name (as asked, even when unknown), a waiting
//!   movement (at most one; a newer request replaces it) and the running
//!   animation (rule 1).
//! - At load (rule 3, corrected 2026-10-19, M4 4d-8): animated objects
//!   and characters start with both names `Stop`, replaced by
//!   `currAnimName` / `nextAnimName` ([`NamedAnimPlayer::with_stop_names`]);
//!   so without `currAnimName` they start on `stop` (with the bit-8
//!   random part), and the waiting `Stop` stays until a request that
//!   starts at once clears it or a play-once ends (rule 8.1).
//! - Doors and chests until Q127 (the older reading): no `currAnimName`:
//!   movement 0 shows but the name stays empty, and on the first update an
//!   empty name makes the waiting movement (`nextAnimName`) start at once,
//!   plainly, without the `flags` bit 8 random part; the phase moves on in
//!   that same update (rule 3, refined Q85).
//! - `KQAnimPlayEvent` (rule 4): the movement waits; it starts at once if
//!   the running animation can be cut short (6th field of its `loop=`
//!   line) or has ended (phase ≥ 0.9999 forwards, ≤ 0 backwards), else
//!   when it ends. A stopped (`speed` 0) movement that can't be cut short
//!   never ends, so a request then waits forever (rule 6).
//! - Transitions (rule 4.2, `connor-movement.md` rules 2–6, `characters.md`
//!   rule 20; M4 4d-11; no placed object has any, so in practice only
//!   characters): a request first looks for a transition from the current
//!   movement to the asked one, the one whose from point comes next (−1:
//!   at once; on a play-once a point already passed is never reached); it
//!   then waits for that point (the update is cut there), the transition
//!   movement plays from 0 at speed 1, and at its end (an ordinary end,
//!   reported as `finished`) its target starts at its phase, and the
//!   wanted movement, if another, is asked for again from there. A newer
//!   request chooses again (none found: the waiting transition is dropped).
//! - Starting (rule 5): looked up ignoring capitals, first match; an
//!   unknown name starts movement 0 under the asked name; the movement's
//!   own `loop=` start phase and speed; afresh even if already playing.
//! - When a play-once movement ends (rule 8): the waiting movement starts
//!   first, else the last frame holds; then the caller sends `EndLoop` /
//!   `Cue 300` naming the movement that ended. A repeating movement never
//!   ends (rule 10).
//!
//! As before (`doors.md` rules 6 and 9, Q45 d): a movement started in an
//! update first moves in the next; the object stands where the shown
//! frame's steps put it.

use kq8_data::shape::Transform;

use crate::collide::Placement;
use crate::door::{chain, identity, DoorMovement};

/// "Has ended", forwards: phase at least this (`anim-scripts.md`).
pub const ENDED_FORWARD: f32 = 0.9999;

/// The name both movement slots start with (`anim-scripts.md` rule 3,
/// corrected 2026-10-19).
pub const STOP_NAME: &str = "Stop";

/// What one update did.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Played {
    pub from: f32,
    pub to: f32,
    pub cues: Vec<String>,
    /// The play-once movement that reached its end in this update (its
    /// name as asked).
    pub finished: Option<String>,
    /// The waiting movement that started in this update (rule 3 or 8),
    /// with what it started as; at the end of a transition, its target.
    pub started: Option<Started>,
    /// A transition movement that started in this update (its from point
    /// was reached).
    pub transition: Option<Started>,
}

/// What a request did (`anim-scripts.md` rule 4), for the log.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// Started at once (the running one could be cut short, or had ended).
    AtOnce(Started),
    /// Waits for the running one to end.
    Waiting,
    /// Waits for a stopped movement that can't be cut short: never starts
    /// (rule 6).
    WaitingForever,
    /// A transition was chosen: it waits for the from point (−1: the next
    /// update), then the transition movement plays (`connor-movement.md`
    /// rules 2–5).
    Transition(kq8_data::objects::TransitionLine),
}

/// A movement just started (rule 5), for the log.
#[derive(Debug, Clone, PartialEq)]
pub struct Started {
    /// The name as asked.
    pub name: String,
    /// The movement that plays (movement 0 for an unknown name).
    pub index: usize,
    /// The name was not in the list (movement 0 plays).
    pub unknown: bool,
    pub start: f32,
    pub speed: f32,
}

impl Started {
    pub fn describe(&self, movements: &[DoorMovement]) -> String {
        let shown = movements.get(self.index).map_or("none", |m| m.name.as_str());
        let unknown = if self.unknown { format!(" (unknown name: movement 0 '{shown}' plays)") } else { String::new() };
        let frame = movements.get(self.index).map_or(0, |m| m.frame_at(self.start));
        format!("'{}'{unknown} at phase {:.4} (frame {frame}) speed {}", self.name, self.start, self.speed)
    }
}

#[derive(Debug, Clone)]
pub struct NamedAnimPlayer {
    /// The object's name (for messages).
    pub name: String,
    pub movements: Vec<DoorMovement>,
    /// Index into `movements` of the one shown.
    pub current: usize,
    /// The current movement's name as asked (rule 5); empty when loaded
    /// without `currAnimName` (rule 3).
    pub current_name: String,
    /// The waiting movement (rule 1), at most one.
    pub waiting: Option<String>,
    pub phase: f32,
    /// The current movement still moves.
    pub playing: bool,
    /// The running animation can be cut short (6th field of its `loop=`).
    pub cut_short: bool,
    /// Started in this update: first moves in the next.
    pub started_this_update: bool,
    /// The object's place in the world.
    pub base: Placement,
    /// A `KQAnimOnce`: at the end of a movement no waiting movement starts
    /// (rule 11); the caller reports the end as `Die`.
    pub once: bool,
    /// `setDeleteWhenDone` on a `KQAnimOnce` of the other animated objects
    /// (rule 13; M5 5d-1): deleted when its movement ends.
    pub delete_when_done: bool,
    /// `pauseAnim` (rule 14): the phase doesn't move, no cues fire, nothing
    /// ends, no waiting movement starts.
    pub paused: bool,
    /// `flags` bit 8 (`anim-scripts.md` rule 3, Q85): a random speed
    /// 0.9–1.1 that **replaces** the current movement's `loop=` speed;
    /// `None` for a movement started later (rule 5: "its own `loop=`
    /// line").
    pub random_speed: Option<f32>,
    /// A chosen transition waiting for its from point (`connor-movement.md`
    /// rule 3).
    pub chosen: Option<kq8_data::objects::TransitionLine>,
    /// The transition playing now (rules 5 and 6).
    pub in_transition: Option<kq8_data::objects::TransitionLine>,
    /// A character (`character-movement.md` rule 1; M5 5a-2): a repeating
    /// movement's steps move its place (`base`, through its mover), so the
    /// drawing does not add their shift (as Connor's loops); a turn in the
    /// frames is still drawn.
    pub loops_move_base: bool,
}

impl NamedAnimPlayer {
    /// Showing movement `current` at its start, named as in the list.
    pub fn new(name: &str, movements: Vec<DoorMovement>, current: usize, base: Placement) -> Self {
        let current = current.min(movements.len().saturating_sub(1));
        let m = movements.get(current);
        let (phase, playing, cut_short) = m.map_or((0.0, false, false), |m| (m.start, m.speed != 0.0, m.cut_short));
        let current_name = m.map_or(String::new(), |m| m.name.clone());
        NamedAnimPlayer {
            name: name.to_string(),
            movements,
            current,
            current_name,
            waiting: None,
            phase,
            playing,
            cut_short,
            started_this_update: false,
            base,
            once: false,
            delete_when_done: false,
            paused: false,
            random_speed: None,
            chosen: None,
            in_transition: None,
            loops_move_base: false,
        }
    }

    /// Rule 3: the names the file gives. `curr` = `currAnimName` (empty or
    /// missing: the name stays empty, movement 0 shows); `next` =
    /// `nextAnimName` fills the waiting slot.
    pub fn with_names_at_load(mut self, curr: Option<&str>, next: Option<&str>) -> Self {
        let curr = curr.map(str::trim).unwrap_or("");
        if curr.is_empty() {
            self.current_name.clear();
        } else if !self.movements.iter().any(|m| m.name.eq_ignore_ascii_case(curr)) {
            // Rule 5: an unknown name plays movement 0 under that name.
            self.current_name = curr.to_string();
        }
        self.waiting = next.map(str::trim).filter(|n| !n.is_empty()).map(str::to_string);
        self
    }

    /// Rule 3 as corrected 2026-10-19 (Q121), for animated objects and
    /// characters (M4 4d-8): both names start as `Stop`; `currAnimName`
    /// and `nextAnimName` replace them (a missing or empty key leaves
    /// `Stop`). The current movement is the one its name finds (capitals
    /// ignored), else movement 0 under that name (rule 5), shown at its
    /// own `loop=` start. So the current name is never empty. (Doors and
    /// chests keep [`NamedAnimPlayer::with_names_at_load`] until Q127.)
    pub fn with_stop_names(mut self, curr: Option<&str>, next: Option<&str>) -> Self {
        let given = |v: Option<&str>| v.map(str::trim).filter(|v| !v.is_empty()).unwrap_or(STOP_NAME).to_string();
        let (curr, next) = (given(curr), given(next));
        let found = self.movements.iter().position(|m| m.name.eq_ignore_ascii_case(&curr));
        self.current = found.unwrap_or(0).min(self.movements.len().saturating_sub(1));
        let (phase, playing, cut_short) = self.movement().map_or((0.0, false, false), |m| (m.start, m.speed != 0.0, m.cut_short));
        (self.phase, self.playing, self.cut_short) = (phase, playing, cut_short);
        self.current_name = curr;
        self.waiting = Some(next);
        self
    }

    /// `flags` bit 8 (`anim-scripts.md` rule 3, `objects.md` rule 4, Q85),
    /// given to the current movement as it starts: start frame =
    /// round(random × (frames − 1)), phase = that frame ÷ frames; speed =
    /// a random 0.9–1.1 that replaces the `loop=` speed (so it always
    /// plays forward). Two numbers from `rng`, frame first. An object with
    /// no `currAnimName` whose waiting movement starts on the first update
    /// loses it (that start is plain, [`NamedAnimPlayer::advance`]).
    pub fn with_random_start(mut self, rng: &mut crate::emitter::Rng) -> Self {
        self.randomize(rng);
        self
    }

    /// The same random start frame and speed for the movement playing now
    /// (a character's `Stop`, `characters.md` rules 13 and 18). Returns
    /// the frame and speed, for the log.
    pub fn randomize(&mut self, rng: &mut crate::emitter::Rng) -> (usize, f32) {
        let (r, speed) = (rng.fraction(), rng.between(0.9, 1.1));
        let frames = self.movement().map_or(1, |m| m.frames());
        let frame = (r * (frames - 1) as f32).round();
        self.phase = frame / frames as f32;
        self.random_speed = Some(speed);
        self.playing = true;
        (frame as usize, speed)
    }

    /// The running animation's speed now.
    pub fn speed(&self) -> f32 {
        self.random_speed.unwrap_or_else(|| self.movement().map_or(0.0, |m| m.speed))
    }

    pub fn movement(&self) -> Option<&DoorMovement> {
        self.movements.get(self.current)
    }

    /// The current movement's name as asked (rule 5).
    pub fn movement_name(&self) -> &str {
        if self.current_name.is_empty() {
            self.movement().map_or("none", |m| m.name.as_str())
        } else {
            &self.current_name
        }
    }

    /// The running play-once movement has reached its end (phase ≥ 0.9999
    /// forwards, ≤ 0 backwards). A repeating or stopped one never has.
    pub fn has_ended(&self) -> bool {
        let Some(m) = self.movement() else { return true };
        if m.repeat {
            return false;
        }
        (m.speed > 0.0 && self.phase >= ENDED_FORWARD) || (m.speed < 0.0 && self.phase <= 0.0)
    }

    /// Rule 5: starts movement `name` at once with its own `loop=` line,
    /// afresh even if it is the one playing. An unknown name starts
    /// movement 0 under the asked name.
    pub fn start(&mut self, name: &str) -> Started {
        let found = self.movements.iter().position(|m| m.name.eq_ignore_ascii_case(name));
        let i = found.unwrap_or(0);
        self.current = i.min(self.movements.len().saturating_sub(1));
        self.current_name = name.to_string();
        let (start, speed, cut_short) = self.movement().map_or((0.0, 0.0, false), |m| (m.start, m.speed, m.cut_short));
        self.phase = start;
        self.playing = speed != 0.0;
        self.cut_short = cut_short;
        self.random_speed = None;
        self.started_this_update = true;
        Started { name: name.to_string(), index: self.current, unknown: found.is_none(), start, speed }
    }

    /// Starts movement `name` at once if it is in the list; `Err` if not
    /// (the doors' and chests' own movements: `open`, `opened`, ...).
    pub fn play(&mut self, name: &str) -> Result<(), String> {
        if !self.movements.iter().any(|m| m.name.eq_ignore_ascii_case(name)) {
            return Err(format!("{} has no movement '{name}'", self.name));
        }
        self.start(name);
        Ok(())
    }

    /// `KQAnimPlayEvent <name>` (rule 4): it becomes the waiting movement;
    /// a transition to it is looked for first (rule 4.2); else it starts
    /// at once if the running one can be cut short or has ended.
    pub fn request(&mut self, name: &str) -> Request {
        self.waiting = Some(name.to_string());
        // connor-movement.md rule 6: during a transition the wanted one is
        // only kept; it is asked for again when the transition ends.
        if self.in_transition.is_some() {
            return Request::Waiting;
        }
        // Rule 2 (and Q40 b: a newer request chooses again; none: dropped).
        self.chosen = self.choose_transition(name);
        if let Some(t) = &self.chosen {
            return Request::Transition(t.clone());
        }
        if self.cut_short || self.has_ended() {
            self.waiting = None;
            return Request::AtOnce(self.start(name));
        }
        let stopped = self.movement().is_none_or(|m| m.speed == 0.0);
        if stopped {
            Request::WaitingForever
        } else {
            Request::Waiting
        }
    }

    /// `connor-movement.md` rule 2: of the current movement's transitions
    /// to `wanted` (capitals ignored), the one whose from point comes next
    /// in its playing direction; −1 = at once (always first); a point
    /// already passed counts only on a repeating movement (after the
    /// wrap); a tie goes to the first listed. A stopped movement has none.
    pub fn choose_transition(&self, wanted: &str) -> Option<kq8_data::objects::TransitionLine> {
        let m = self.movement()?;
        let speed = self.speed();
        if speed == 0.0 {
            return None;
        }
        let mut best: Option<(f32, &kq8_data::objects::TransitionLine)> = None;
        for t in m.transitions.iter().filter(|t| t.to.eq_ignore_ascii_case(wanted)) {
            let d = if t.from < 0.0 {
                -1.0
            } else {
                let ahead = if speed > 0.0 { t.from - self.phase } else { self.phase - t.from };
                match (ahead >= 0.0, m.repeat) {
                    (true, _) => ahead,
                    (false, true) => ahead + 1.0,
                    (false, false) => continue,
                }
            };
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, t));
            }
        }
        best.map(|(_, t)| t.clone())
    }

    /// How far (in phase) the chosen transition's from point is ahead in
    /// this update of `travel` phase, if it is reached: `Some(0)` for −1.
    fn reaches_from_point(&self, t: &kq8_data::objects::TransitionLine, travel: f32) -> Option<f32> {
        if t.from < 0.0 {
            return Some(0.0);
        }
        let m = self.movement()?;
        let ahead = if travel > 0.0 { t.from - self.phase } else { self.phase - t.from };
        let ahead = if ahead < 0.0 && m.repeat { ahead + 1.0 } else { ahead };
        (ahead >= 0.0 && ahead <= travel.abs()).then_some(ahead)
    }

    /// `anim-scripts.md` rule 7 (M5 5c-2): a cue `interruptable <n>` the
    /// phase passed sets "can be cut short" to n (the last one passed
    /// wins). The cue stays in the list, for the log.
    fn interruptable_cues(&mut self, cues: &[String]) {
        for c in cues {
            let mut w = c.split_whitespace();
            if w.next().is_some_and(|k| k.eq_ignore_ascii_case("interruptable")) {
                self.cut_short = w.next().and_then(|n| n.parse::<f64>().ok()).is_some_and(|n| n != 0.0);
            }
        }
    }

    /// One update of `dt` seconds.
    pub fn advance(&mut self, dt: f32) -> Played {
        let mut out = Played { from: self.phase, to: self.phase, ..Default::default() };
        if self.paused {
            return out;
        }
        // Rule 3 (refined Q85): an empty current name lets the waiting
        // movement start, plainly (no random part), and its phase moves
        // on in this same update.
        if self.current_name.is_empty() {
            if let Some(w) = self.waiting.take() {
                out.started = Some(self.start(&w));
                self.started_this_update = false;
                out.from = self.phase;
                out.to = self.phase;
            }
        }
        if std::mem::take(&mut self.started_this_update) || !self.playing {
            return out;
        }
        let Some(m) = self.movements.get(self.current) else { return out };
        let speed = self.random_speed.unwrap_or(m.speed);
        // connor-movement.md rule 3: a chosen transition's from point
        // reached in this update cuts it there (−1: the whole update
        // plays); the transition is set up at the end and first moves in
        // the next update.
        if let Some(t) = self.chosen.clone() {
            let travel = if m.play_time > 0.0 { speed * dt / m.play_time } else { 0.0 };
            if let Some(ahead) = self.reaches_from_point(&t, travel) {
                let cut_dt = if t.from < 0.0 { dt } else { ahead * m.play_time / speed.abs() };
                let step = m.step_with(self.phase, cut_dt, speed);
                let at = if t.from < 0.0 { step.to } else { t.from };
                out.to = at;
                out.cues = step.cues;
                self.chosen = None;
                // Rule 5: the transition movement from phase 0 at speed 1, once.
                let mut st = self.start(&t.via);
                // As Connor's walker: it first moves in the next update.
                self.started_this_update = false;
                self.phase = 0.0;
                self.random_speed = Some(1.0);
                st.start = 0.0;
                st.speed = 1.0;
                out.transition = Some(st);
                self.in_transition = Some(t);
                return out;
            }
        }
        let step = m.step_with(self.phase, dt, speed);
        self.phase = step.to;
        out.to = step.to;
        out.cues = step.cues;
        self.interruptable_cues(&out.cues);
        if step.done && self.in_transition.is_some() {
            // Rule 6: an ordinary end of the transition movement; its
            // target starts at its phase; another wanted one is asked for
            // again from there.
            let t = self.in_transition.take().unwrap();
            self.playing = false;
            out.finished = Some(self.movement_name().to_string());
            let mut st = self.start(&t.to);
            self.started_this_update = false;
            self.phase = t.to_phase;
            st.start = t.to_phase;
            out.started = Some(st);
            match self.waiting.take() {
                Some(w) if !w.eq_ignore_ascii_case(&t.to) => {
                    self.request(&w);
                }
                _ => {}
            }
            return out;
        }
        if step.done {
            self.playing = false;
            out.finished = Some(self.movement_name().to_string());
            // Rule 8: the waiting movement starts first; it moves from the
            // next update. Not for a `KQAnimOnce` (rule 11).
            if let Some(w) = self.waiting.take().filter(|_| !self.once) {
                out.started = Some(self.start(&w));
            }
        }
        out
    }

    pub fn frame(&self) -> usize {
        self.movement().map_or(0, |m| m.frame_at(self.phase))
    }

    /// The turn the shown frame's steps give it (own space).
    pub fn turn(&self) -> Transform {
        let mut t = self.movement().map_or_else(identity, |m| m.turn_at_frame(self.frame()));
        if self.loops_move_base && self.movement().is_some_and(|m| m.repeat) {
            // Its steps moved the place instead; a turn in the frames stays.
            t.p = [0.0; 3];
        }
        t
    }

    /// Where it stands now: the steps of the shown frame, then its place.
    pub fn placement(&self) -> Placement {
        let t = self.turn();
        let base = Transform { flags: 0, m: self.base.rotation, p: self.base.position };
        let w = chain(&base, &t);
        Placement { rotation: w.m, position: w.p }
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;
    use crate::door::Cue;

    const DT: f32 = 1.0 / 15.0;

    /// A movement of 16 frames over 1 s (made-up numbers in the style of
    /// the chests and baskets).
    pub fn movement(name: &str, start: f32, speed: f32, repeat: bool, cut_short: bool, cues: Vec<Cue>) -> DoorMovement {
        DoorMovement { name: name.into(), start, speed, repeat, cut_short, play_time: 1.0, frame_steps: vec![identity(); 16], frame_parts: Vec::new(), cues, transitions: Vec::new() }
    }

    /// `stop` still at 0, then `open` 0 -> 1 over 1 s with a cue at 0, 16
    /// frames. Both `0 1` (play once, can be cut short), as most of
    /// Daventry's object movements.
    pub fn still_then_open(name: &str) -> NamedAnimPlayer {
        NamedAnimPlayer::new(
            name,
            vec![
                movement("closed", 0.0, 0.0, false, true, vec![]),
                movement("open", 0.0, 1.0, false, true, vec![Cue { phase: 0.0, action: "playsound chstopen".into() }]),
            ],
            0,
            Placement::at([0.0; 3]),
        )
    }

    /// Updates until something finishes: (updates, the advance).
    fn until_finished(p: &mut NamedAnimPlayer) -> (usize, Played) {
        for n in 1..100 {
            let a = p.advance(DT);
            if a.finished.is_some() {
                return (n, a);
            }
        }
        panic!("nothing finished");
    }

    #[test]
    fn open_plays_once_and_stays_on_its_last_frame() {
        let mut p = still_then_open("smlchst3");
        assert_eq!((p.movement_name(), p.playing), ("closed", false));
        assert!(p.play("OPEN").is_ok());
        assert!(p.play("opened").is_err());
        let first = p.advance(DT);
        assert_eq!(first.to, 0.0, "started in this update: waits");
        assert_eq!(p.advance(DT).cues, ["playsound chstopen"]);
        let (n, a) = until_finished(&mut p);
        assert_eq!((a.finished.as_deref(), n + 2), (Some("OPEN"), 16));
        assert_eq!((p.movement_name(), p.phase, p.frame()), ("OPEN", 1.0, 15), "the last frame stays");
        assert_eq!(p.advance(DT).finished, None);
    }

    /// `stop` repeating and cut short, `swing` play once and not cut short,
    /// `down` a stopped final pose (`0 0`, like `bricks.kq`), `spin`
    /// repeating and not cut short.
    fn object() -> NamedAnimPlayer {
        NamedAnimPlayer::new(
            "thing",
            vec![
                movement("stop", 0.0, 0.0, true, true, vec![]),
                movement("swing", 0.0, 1.0, false, false, vec![]),
                movement("down", 1.0, 0.0, false, false, vec![]),
                movement("spin", 0.0, 1.0, true, false, vec![]),
            ],
            0,
            Placement::at([0.0; 3]),
        )
    }

    #[test]
    fn the_interruptable_cue_switches_cut_short() {
        // anim-scripts.md rule 7 (M5 5c-2): goblin2.anm's attack, 1 s:
        // `interruptable 1` at 0, `interruptable 0` at 0.1875.
        let cue = |phase: f32, action: &str| Cue { phase, action: action.into() };
        let attack = movement("attack", 0.0, 1.0, false, false, vec![cue(0.0, "interruptable 1"), cue(0.1875, "interruptable 0"), cue(0.5, "do")]);
        let mut p = NamedAnimPlayer::new("goblin", vec![movement("stop", 0.0, 1.0, true, true, vec![]), attack], 0, Placement::at([0.0; 3]));
        p.start("attack");
        assert!(!p.cut_short, "its loop= line says it can't be cut short");
        p.advance(DT);
        assert!(!p.cut_short, "started this update: it first moves in the next");
        p.advance(DT);
        assert!(p.cut_short, "the cue at 0 fires in its first moving update");
        p.advance(DT);
        assert!(p.cut_short);
        let cues = p.advance(DT).cues;
        assert_eq!(cues, ["interruptable 0"], "phase 0.133 -> 0.2 passes 0.1875");
        assert!(!p.cut_short);
        assert!(matches!(p.request("stop"), Request::Waiting), "after the cue a request waits for the end");
    }

    #[test]
    fn a_cut_short_movement_is_replaced_at_once() {
        let mut p = object();
        let r = p.request("Down");
        let Request::AtOnce(s) = r else { panic!("{r:?}") };
        assert_eq!((s.index, s.start, s.speed, s.unknown), (2, 1.0, 0.0, false));
        assert_eq!((p.movement_name(), p.waiting.as_deref(), p.phase), ("Down", None, 1.0));
    }

    #[test]
    fn a_play_once_that_cant_be_cut_short_finishes_then_the_waiting_one_starts() {
        let mut p = object();
        p.request("swing");
        p.advance(DT);
        p.advance(DT);
        assert_eq!(p.request("spin"), Request::Waiting);
        // A newer request replaces the older one (rule 4.1).
        assert_eq!(p.request("stop"), Request::Waiting);
        let (_, a) = until_finished(&mut p);
        // The ended one is named, and the waiting one already started.
        assert_eq!(a.finished.as_deref(), Some("swing"));
        assert_eq!(a.started.map(|s| s.name), Some("stop".to_string()));
        assert_eq!((p.movement_name(), p.waiting.as_deref()), ("stop", None));
    }

    #[test]
    fn a_play_once_that_has_ended_is_replaced_at_once() {
        let mut p = object();
        p.request("swing");
        until_finished(&mut p);
        assert!(p.has_ended());
        assert!(matches!(p.request("spin"), Request::AtOnce(_)));
    }

    #[test]
    fn a_stopped_pose_that_cant_be_cut_short_waits_forever() {
        let mut p = object();
        p.request("down");
        assert_eq!(p.request("stop"), Request::WaitingForever);
        for _ in 0..100 {
            assert_eq!(p.advance(DT), Played { from: 1.0, to: 1.0, ..Default::default() });
        }
        assert_eq!(p.movement_name(), "down");
    }

    #[test]
    fn a_repeating_movement_that_cant_be_cut_short_never_lets_a_request_start() {
        let mut p = object();
        p.request("spin");
        assert_eq!(p.request("stop"), Request::Waiting);
        for _ in 0..100 {
            assert_eq!(p.advance(DT).finished, None);
        }
        assert_eq!(p.movement_name(), "spin");
    }

    #[test]
    fn an_unknown_name_plays_movement_0_under_that_name() {
        let mut p = object();
        p.request("spin");
        p.cut_short = true;
        let Request::AtOnce(s) = p.request("nosuch") else { panic!() };
        assert!(s.unknown);
        assert_eq!((p.current, p.movement_name()), (0, "nosuch"));
    }

    #[test]
    fn the_playing_movement_restarts_from_its_start() {
        let mut p = object();
        p.request("swing");
        for _ in 0..5 {
            p.advance(DT);
        }
        p.cut_short = true;
        p.request("swing");
        assert_eq!(p.phase, 0.0);
    }

    #[test]
    fn a_kq_anim_once_keeps_its_last_frame_and_never_starts_the_waiting_one() {
        let mut p = object();
        p.once = true;
        p.request("swing");
        p.advance(DT);
        assert_eq!(p.request("stop"), Request::Waiting);
        let (_, a) = until_finished(&mut p);
        assert_eq!((a.finished.as_deref(), a.started), (Some("swing"), None));
        assert_eq!((p.movement_name(), p.phase), ("swing", 1.0));
    }

    #[test]
    fn a_paused_animation_doesnt_move_fire_cues_or_end() {
        let mut p = still_then_open("jar");
        p.request("open");
        p.advance(DT);
        p.paused = true;
        for _ in 0..40 {
            assert_eq!(p.advance(DT), Played { from: 0.0, to: 0.0, ..Default::default() });
        }
        p.paused = false;
        assert_eq!(p.advance(DT).cues, ["playsound chstopen"], "the cue at 0 fires once it moves again");
    }

    #[test]
    fn without_curr_anim_name_the_random_start_is_lost_q85() {
        // Rule 3 (Q85): the six flags=8 objects with only nextAnimName=stop
        // start plainly: phase 0, speed exactly 1, all in step.
        let mut rng = crate::emitter::Rng::new(4);
        let mut p = object().with_names_at_load(None, Some("spin")).with_random_start(&mut rng);
        let a = p.advance(DT);
        assert_eq!(a.started.as_ref().map(|s| (s.index, s.start)), Some((3, 0.0)));
        assert_eq!(p.random_speed, None);
        // The phase moved on in that same update, at speed 1.
        assert!((p.phase - DT).abs() < 1e-6, "{}", p.phase);
    }

    #[test]
    fn with_curr_anim_name_the_random_frame_and_speed_replace_the_loop_line() {
        // A still loop (speed 0 in its line) still plays forward with bit 8.
        let mut rng = crate::emitter::Rng::new(7);
        let mut p = object().with_names_at_load(Some("stop"), None).with_random_start(&mut rng);
        let speed = p.random_speed.expect("random speed");
        assert!((0.9..=1.1).contains(&speed));
        // A whole frame: phase × 16 frames is a whole number.
        let frames = p.phase * 16.0;
        assert!((frames - frames.round()).abs() < 1e-4 && p.phase < 1.0, "{}", p.phase);
        let before = p.phase;
        p.advance(DT);
        assert!(((p.phase - before).rem_euclid(1.0) - DT * speed).abs() < 1e-5, "moved in the first update");
        // A movement started later runs at its own speed.
        p.cut_short = true;
        p.request("spin");
        assert_eq!(p.random_speed, None);
    }

    /// The Girl's `girl.anm`: `stop` (loop, cut short), `cower` (`0 0`,
    /// `transition0=0.882353 stop 0.000000 cowerstop`), `cowerstop` (`0 0`).
    fn girl() -> NamedAnimPlayer {
        let mut cower = movement("cower", 0.0, 1.0, false, false, vec![]);
        cower.transitions.push(kq8_data::objects::TransitionLine { from: 0.882353, to: "stop".into(), to_phase: 0.0, via: "cowerstop".into() });
        let ms = vec![movement("stop", 0.0, 1.0, true, true, vec![]), cower, movement("cowerstop", 0.0, 1.0, false, false, vec![])];
        NamedAnimPlayer::new("Girl", ms, 0, Placement::at([0.0; 3])).with_stop_names(None, None)
    }

    #[test]
    fn a_transition_waits_for_its_from_point_then_its_target_starts() {
        // characters.md rule 20 (Q124 a), connor-movement.md rules 2-6.
        let mut p = girl();
        assert!(matches!(p.request("cower"), Request::AtOnce(_)));
        for _ in 0..5 {
            p.advance(DT);
        }
        let Request::Transition(t) = p.request("STOP") else { panic!() };
        assert_eq!(t.via, "cowerstop");
        // cower goes on until 0.882353: the update that reaches it is cut there.
        let mut n = 0;
        let a = loop {
            n += 1;
            let a = p.advance(DT);
            if a.transition.is_some() {
                break a;
            }
            assert!(n < 30);
        };
        assert_eq!((a.to, p.movement_name(), p.phase), (0.882353, "cowerstop", 0.0));
        // cowerstop then plays its 1 s (from the next update) and ends as an
        // ordinary end; stop starts at its to-phase.
        let (m, a) = until_finished(&mut p);
        assert_eq!((m, a.finished.as_deref()), (15, Some("cowerstop")));
        assert_eq!(a.started.as_ref().map(|s| (s.name.as_str(), s.start)), Some(("stop", 0.0)));
        assert_eq!((p.movement_name(), p.waiting.as_deref(), p.in_transition.is_none()), ("stop", None, true));
    }

    #[test]
    fn a_from_point_already_passed_on_a_play_once_is_never_reached() {
        let mut p = girl();
        p.request("cower");
        // Past 0.88 (14 of 15 updates): no transition, so stop waits for the end.
        for _ in 0..15 {
            p.advance(DT);
        }
        assert_eq!(p.request("stop"), Request::Waiting);
        let (_, a) = until_finished(&mut p);
        assert_eq!((a.finished.as_deref(), a.started.map(|s| s.name)), (Some("cower"), Some("stop".to_string())));
        // A newer request with no transition drops a chosen one (Q40 b).
        let mut q = girl();
        q.request("cower");
        assert!(matches!(q.request("stop"), Request::Transition(_)));
        q.request("cowerstop");
        assert_eq!(q.chosen, None);
    }

    #[test]
    fn the_names_start_as_stop_q121() {
        // movement 0 'closed', 1 'open', and a looping 'stop' third.
        let mut ms = still_then_open("x").movements;
        ms.push(movement("Stop", 0.25, 1.0, true, true, vec![]));
        let p = NamedAnimPlayer::new("torch", ms.clone(), 0, Placement::at([0.0; 3])).with_stop_names(None, None);
        assert_eq!((p.current, p.movement_name(), p.waiting.as_deref(), p.phase, p.playing), (2, "Stop", Some("Stop"), 0.25, true));
        // The bit-8 random part now applies to it (the torches and fish).
        let mut rng = crate::emitter::Rng::new(3);
        let r = p.clone().with_random_start(&mut rng);
        assert!(r.random_speed.is_some());
        // The keys replace them; an empty one counts as missing.
        let q = NamedAnimPlayer::new("x", ms.clone(), 2, Placement::at([0.0; 3])).with_stop_names(Some("open"), Some(" "));
        assert_eq!((q.current, q.movement_name(), q.waiting.as_deref()), (1, "open", Some("Stop")));
        // No 'stop' in the list: movement 0 under the name Stop.
        let z = NamedAnimPlayer::new("x", still_then_open("x").movements, 1, Placement::at([0.0; 3])).with_stop_names(None, None);
        assert_eq!((z.current, z.movement_name()), (0, "Stop"));
        // A first movement played once ends: the waiting Stop starts (rule
        // 8.1; Daventry's tombdoor plays its stop twice).
        let mut t = NamedAnimPlayer::new("tombdoor", ms, 0, Placement::at([0.0; 3])).with_stop_names(Some("open"), None);
        let (_, a) = until_finished(&mut t);
        assert_eq!(a.started.map(|s| (s.name, s.index)), Some(("Stop".to_string(), 2)));
        // A request that starts at once clears the waiting slot.
        let mut c = NamedAnimPlayer::new("x", still_then_open("x").movements, 0, Placement::at([0.0; 3])).with_stop_names(None, None);
        assert!(matches!(c.request("open"), Request::AtOnce(_)));
        assert_eq!(c.waiting, None);
    }

    #[test]
    fn no_curr_anim_name_lets_next_anim_name_start_on_the_first_update() {
        let p = object().with_names_at_load(None, Some("spin"));
        assert_eq!((p.current, p.current_name.as_str()), (0, ""));
        let mut p = p;
        let a = p.advance(DT);
        assert_eq!(a.started.map(|s| s.index), Some(3));
        // It moves in that same update (refined Q85).
        assert_eq!(p.movement_name(), "spin");
        assert!((p.phase - DT).abs() < 1e-6, "{}", p.phase);
        // With a currAnimName the waiting one waits as usual.
        let mut q = object().with_names_at_load(Some("stop"), Some("spin"));
        assert_eq!(q.advance(DT).started, None);
        assert_eq!(q.movement_name(), "stop");
    }
}
