//! Connor's ground movement: which movement plays, turning, and the step
//! his animation asks the mover for.
//!
//! Rules from the behaviour note `docs/behaviour/connor-movement.md`
//! ("Movement intent", "Which movement plays", "Forward speed comes from
//! the animation", "Turning"). M2 step 2b: no banking, a loop's step spread
//! evenly over the loop (Q24).
//!
//! Start and stop transitions (M2 step 2g, "Starting and stopping", rules
//! 1–6): the movement asked for, the transition chosen by its from point,
//! played to its end with its own frame-by-frame steps, then its `to`
//! movement at its `to` phase.
//!
//! The falling animations (M2 step 2c/2e, `connor-physics.md` "Falling off
//! an edge"): `jumphold` while falling, then `jumpend` (safe landing) or
//! `jumpdie` (death), each played once, with the keys ignored meanwhile.

use crate::UPDATES_PER_SECOND;

/// Turning speed while a turn key is held, radians per second.
pub const TURN_RATE: f32 = 1.75;

/// What the player wants: forward/back and turn are −1, 0 or +1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Intent {
    pub forward: i8,
    pub turn: i8,
    pub run: bool,
}

/// The movement name for an intent (the note's table; no water yet).
pub fn choose_movement(intent: Intent) -> &'static str {
    match (intent.forward, intent.turn) {
        (1, _) if intent.run => "run",
        (1, _) => "walk",
        (-1, _) => "back",
        (0, 1) => "left",
        (0, -1) => "right",
        _ => "stop",
    }
}

/// A `transitionN` line of `Conner.anm` (connor-movement.md "Starting and
/// stopping"), with the movement names as the walker knows them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transition {
    /// Phase of the current movement where it starts; −1 = at once.
    pub from: f32,
    pub to: &'static str,
    /// Phase `to` starts at when the transition ends.
    pub to_phase: f32,
    /// The transition movement played in between.
    pub via: &'static str,
}

/// One movement's animation, as far as moving is concerned.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct MovementLoop {
    /// Length of the loop in seconds.
    pub duration: f32,
    /// The loop's per-frame steps added up, in model space (Z up; the
    /// model faces −Y, its +X is Connor's left).
    pub step_sum: [f32; 3],
    /// Playback speed from `Conner.anm` (`back`: −1).
    pub speed: f32,
    /// True = repeats; false = plays once, first to last frame
    /// (`objects.md` "Animated objects at load", loop rule 2).
    pub repeat: bool,
    /// Number of frames.
    pub frames: usize,
    /// Each frame's own step (model space): frame k's is the move from
    /// frame k−1 to k, over one frame time. Used for one-shots; empty =
    /// spread `step_sum` evenly.
    pub frame_steps: Vec<[f32; 3]>,
    /// `loop=` 6th field: a new movement may replace it at once.
    pub cut_short: bool,
    /// Its `transitionN` lines, in file order.
    pub transitions: Vec<Transition>,
}

impl MovementLoop {
    /// The step of a one-shot between phases `p0` and `p1` (0 ≤ p0 ≤ p1
    /// ≤ 1): each frame's step over the part of that frame's time the
    /// interval covers (connor-movement.md "Steps in the start
    /// animations"; frame time = play time / (frames − 1)).
    pub fn one_shot_step(&self, p0: f32, p1: f32) -> [f32; 3] {
        let n = self.frame_steps.len();
        if n < 2 {
            return self.step_sum.map(|v| v * (p1 - p0));
        }
        let spans = (n - 1) as f32;
        let (a, b) = (p0 * spans, p1 * spans);
        let mut out = [0.0f32; 3];
        for (k, step) in self.frame_steps.iter().enumerate().skip(1) {
            let covered = (b.min(k as f32) - a.max((k - 1) as f32)).max(0.0);
            for i in 0..3 {
                out[i] += step[i] * covered;
            }
        }
        out
    }

    /// Rule 2: of the transitions to `wanted`, the one whose from point
    /// comes next going forward from `phase` (wrapping on a loop, never
    /// on a one-shot; −1 on a loop is always ≤ 0, so it wins; ties: the
    /// first listed). Returns it and its distance in phase.
    pub fn choose_transition(&self, phase: f32, wanted: &str) -> Option<(Transition, f32)> {
        let mut best: Option<(Transition, f32)> = None;
        for t in self.transitions.iter().filter(|t| t.to == wanted) {
            let mut d = t.from - phase;
            if d < 0.0 {
                if !self.repeat {
                    continue;
                }
                d += 1.0;
            }
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((*t, d));
            }
        }
        best
    }

    /// Units per second along the loop's step (negative when played back).
    pub fn ground_speed(&self) -> f32 {
        let [x, y, _] = self.step_sum;
        if self.duration > 0.0 { (x * x + y * y).sqrt() / self.duration * self.speed } else { 0.0 }
    }

    /// Seconds one play takes: the stored loop length, repeating or not.
    /// It already runs from the first frame to the last (`crash`: 8 frames
    /// in 7/15 s, and measured: its whole step in 7 ticks; DESIGN 2d-4).
    pub fn play_time(&self) -> f32 {
        self.duration
    }

    /// The frame shown at `phase` (0..1 of one play).
    pub fn frame_at(&self, phase: f32) -> usize {
        let n = self.frames.max(1);
        if self.repeat {
            ((phase.rem_euclid(1.0) * n as f32) as usize).min(n - 1)
        } else {
            (phase.clamp(0.0, 1.0) * (n - 1) as f32).round() as usize
        }
    }
}

/// What drives Connor's animation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Mode {
    /// The keys choose the movement.
    Free,
    /// Falling: `jumphold`, repeating.
    Fall,
    /// Safe landing: `jumpend` once, then back to the keys.
    Land,
    /// Fatal landing: `jumpdie` once, then dead.
    Dying,
    /// The Die state: holds the last frame of `jumpdie`.
    Dead,
    /// Ran head-on into a wall: `crash` once (it pushes him back), then
    /// back to the keys.
    Crash,
}

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Free => "free",
            Mode::Fall => "fall",
            Mode::Land => "land",
            Mode::Dying => "dying",
            Mode::Dead => "dead",
            Mode::Crash => "crash",
        }
    }

    /// The movement this mode plays (`None`: the keys decide).
    fn movement(&self) -> Option<&'static str> {
        match self {
            Mode::Free => None,
            Mode::Fall => Some("jumphold"),
            Mode::Land => Some("jumpend"),
            Mode::Dying | Mode::Dead => Some("jumpdie"),
            Mode::Crash => Some("crash"),
        }
    }
}

/// Connor's heading, current movement and place in its loop.
#[derive(Debug, Clone)]
pub struct Walker {
    /// Radians about Z; 0 = facing −Y (the model's own forward).
    pub heading: f32,
    pub movement: &'static str,
    /// Position in the current loop, 0..1.
    pub phase: f32,
    pub mode: Mode,
    /// Wall blocks set by a head-on stop: while either is on he stands
    /// (connor-physics.md "Walk held against a wall", traced, Q38).
    pub blocked_forward: bool,
    pub blocked_back: bool,
    /// The movement asked for and not reached yet (rule 1).
    pub wanted: Option<&'static str>,
    /// A chosen transition waiting for its from point (rule 3).
    pub waiting: Option<Transition>,
    /// The transition playing now (rules 5 and 6).
    pub playing: Option<Transition>,
    /// The keys' last choice: a request is made only when it changes.
    choice: &'static str,
}

/// A movement key, for the key events that clear wall blocks.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MoveKey {
    Forward,
    Back,
    /// Left or right.
    Turn,
}

/// What one update asks for.
#[derive(Debug, Clone, PartialEq)]
pub struct Step {
    /// Movement change for the mover, world space.
    pub wanted: [f32; 3],
    /// Set when the movement changed this update: (from, to).
    pub changed: Option<(&'static str, &'static str)>,
    /// The movement has no loop in the animation list.
    pub missing: bool,
    /// Set when the mode changed this update: (from, to).
    pub mode_changed: Option<(Mode, Mode)>,
    /// The movement that played (moved him) this update, and its phase
    /// before and after.
    pub played: &'static str,
    pub played_from: f32,
    pub played_to: f32,
    /// The step in his own axes, before turning by the heading: x = his
    /// left, −y = forward.
    pub own_step: [f32; 3],
    /// Requests and transitions this update, for the log.
    pub notes: Vec<String>,
}

impl Walker {
    pub fn new(heading: f32) -> Self {
        Walker {
            heading,
            movement: "stop",
            phase: 0.0,
            mode: Mode::Free,
            blocked_forward: false,
            blocked_back: false,
            wanted: None,
            waiting: None,
            playing: None,
            choice: "stop",
        }
    }

    /// Switch mode (from the mover's fall events, or a test lift). The new
    /// mode's animation starts at its beginning in the next `update`. Wall
    /// blocks stay as they are (traced: only key events clear them). Any
    /// start or stop transition is dropped. Back to `Free`, he is in `stop`
    /// and the keys ask again from there (so with run held after a crash,
    /// `stoprun` plays before `run`, as measured, DESIGN 2d-4; after
    /// `jumpend` our choice).
    pub fn set_mode(&mut self, mode: Mode) {
        if mode == self.mode {
            return;
        }
        self.clear_transitions();
        if mode == Mode::Free {
            self.movement = "stop";
            self.phase = 0.0;
        }
        self.mode = mode;
    }

    fn clear_transitions(&mut self) {
        self.wanted = None;
        self.waiting = None;
        self.playing = None;
        self.choice = "stop";
    }

    /// Rules 1, 2 and 4: `wanted` is asked for. A transition to it from
    /// the current movement waits for its from point; with none, the
    /// movement switches at once from phase 0 if the current one can be
    /// cut short (or has ended); otherwise the request is kept for its end.
    fn ask<'a>(&mut self, wanted: &'static str, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>, notes: &mut Vec<String>) {
        self.wanted = Some(wanted);
        if let Some(old) = self.waiting.take() {
            notes.push(format!("waiting transition {} dropped", old.via));
        }
        if let Some(t) = self.playing {
            notes.push(format!("{wanted} kept until {} ends", t.via));
            return;
        }
        if wanted == self.movement {
            // Not traced (Q40 b): the request is already met.
            self.wanted = None;
            return;
        }
        let current = loops(self.movement);
        if let Some((t, d)) = current.and_then(|l| if l.speed < 0.0 { None } else { l.choose_transition(self.phase, wanted) }) {
            notes.push(format!(
                "transition chosen: {} -> {wanted} via {} from phase {} (now {:.4}, distance {:.4}), then {} at {}",
                self.movement, t.via, t.from, self.phase, d.max(0.0), t.to, t.to_phase
            ));
            self.waiting = Some(t);
            return;
        }
        if current.is_none_or(|l| l.cut_short || self.phase >= 1.0) {
            notes.push(format!("no transition {} -> {wanted}: switched at once from phase 0", self.movement));
            self.movement = wanted;
            self.phase = 0.0;
            self.wanted = None;
        } else {
            notes.push(format!("{} can't be cut short: {wanted} kept until it ends", self.movement));
        }
    }

    /// Rule 6: the transition playing has ended. Its `to` starts at its
    /// `to_phase`; if something else is wanted by now, it is asked for
    /// again from there at once.
    fn end_transition<'a>(&mut self, t: Transition, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>, notes: &mut Vec<String>) {
        self.playing = None;
        self.movement = t.to;
        self.phase = t.to_phase;
        notes.push(format!("{} ended: {} from phase {}", t.via, t.to, t.to_phase));
        match self.wanted {
            Some(w) if w != t.to => self.ask(w, loops, notes),
            _ => self.wanted = None,
        }
    }

    /// Either wall block is on.
    pub fn blocked(&self) -> bool {
        self.blocked_forward || self.blocked_back
    }

    /// The wall blocks for the log: `none`, `fwd`, `back` or `both`.
    pub fn blocked_name(&self) -> &'static str {
        match (self.blocked_forward, self.blocked_back) {
            (false, false) => "none",
            (true, false) => "fwd",
            (false, true) => "back",
            (true, true) => "both",
        }
    }

    /// A movement key pressed or released: clears wall blocks as traced.
    /// Forward released → forward; back released → back; forward pressed
    /// → back; back pressed → forward; a turn key pressed → both.
    /// Returns what it cleared (forward, back).
    pub fn key_event(&mut self, key: MoveKey, pressed: bool) -> (bool, bool) {
        let (forward, back) = match (key, pressed) {
            (MoveKey::Forward, false) | (MoveKey::Back, true) => (true, false),
            (MoveKey::Back, false) | (MoveKey::Forward, true) => (false, true),
            (MoveKey::Turn, true) => (true, true),
            (MoveKey::Turn, false) => (false, false),
        };
        let cleared = (forward && self.blocked_forward, back && self.blocked_back);
        self.blocked_forward &= !forward;
        self.blocked_back &= !back;
        cleared
    }

    /// In the Move state: the keys chose a movement other than standing
    /// (walking, running, stepping back, turning on the spot).
    pub fn moving(&self) -> bool {
        self.mode == Mode::Free && self.movement != "stop"
    }

    /// A head-on hit while moving (connor-physics.md "Walls"). Running:
    /// `crash` plays in full from the next update (traced, Q31 e). Walking
    /// or stepping back switches straight to `stop`, with no stop animation
    /// (measured), and sets a wall block: back blocked after `back`,
    /// forward blocked otherwise (traced, Q38). `stoprun` counts as running
    /// and `stopwalk` as walking (connor-movement.md "Starting and
    /// stopping"); either ends the transition. Returns the change, if any.
    pub fn head_on_at_wall(&mut self) -> Option<(&'static str, &'static str)> {
        if !self.moving() {
            return None;
        }
        let from = self.movement;
        if matches!(from, "run" | "stoprun") {
            self.set_mode(Mode::Crash);
            return Some((from, "crash"));
        }
        if from == "back" {
            self.blocked_back = true;
        } else {
            self.blocked_forward = true;
        }
        self.clear_transitions();
        self.movement = "stop";
        self.phase = 0.0;
        Some((from, "stop"))
    }

    /// One update (1/15 s). `loops` gives a movement's loop by name.
    pub fn update<'a>(&mut self, intent: Intent, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>) -> Step {
        let dt = (1.0 / UPDATES_PER_SECOND) as f32;
        let mut mode_changed = None;
        let mut notes = Vec::new();
        let start = self.movement;
        // A mode's one-shot animation that finished last update hands over.
        if self.phase >= 1.0 {
            let next = match self.mode {
                Mode::Land | Mode::Crash => Some(Mode::Free),
                Mode::Dying => Some(Mode::Dead),
                _ => None,
            };
            if let Some(next) = next {
                mode_changed = Some((self.mode, next));
                self.set_mode(next);
            }
        }
        if let Some(m) = self.mode.movement() {
            if m != self.movement {
                self.movement = m;
                self.phase = 0.0;
            }
        } else {
            // While a wall block is on he stands, whatever the keys ask.
            let choice = if self.blocked() { "stop" } else { choose_movement(intent) };
            if choice != self.choice {
                self.choice = choice;
                notes.push(format!("asks for {choice}"));
                self.ask(choice, loops, &mut notes);
            }
        }
        let played = self.movement;
        let played_from = self.phase;
        let mut step = Step {
            wanted: [0.0; 3],
            changed: None,
            missing: false,
            mode_changed,
            played,
            played_from,
            played_to: played_from,
            own_step: [0.0; 3],
            notes: Vec::new(),
        };
        let finish = |mut step: Step, walker: &Walker, notes: Vec<String>| {
            step.changed = (walker.movement != start).then_some((start, walker.movement));
            step.notes = notes;
            step
        };
        if self.mode == Mode::Dead {
            self.phase = 1.0;
            return finish(step, self, notes);
        }
        // The keys turn him only when they choose the movement; that
        // includes the start and stop transitions.
        if self.mode == Mode::Free && !self.blocked() {
            self.heading += f32::from(intent.turn) * TURN_RATE * dt;
        }
        let Some(l) = loops(played) else {
            step.missing = true;
            return finish(step, self, notes);
        };
        let time = l.play_time();
        if time <= 0.0 {
            return finish(step, self, notes);
        }
        // Share of one play this update (negative when played backwards).
        let share = dt * l.speed / time;
        // Rule 3: a chosen transition starts where the movement reaches
        // its from point; this update plays only up to there.
        let reach = self.waiting.filter(|_| l.speed > 0.0).map(|t| {
            if t.from < 0.0 {
                0.0
            } else {
                let d = t.from - self.phase;
                if d < 0.0 && l.repeat { d + 1.0 } else { d.max(0.0) }
            }
        });
        let own = if let Some(d) = reach.filter(|d| *d <= share) {
            let own = if l.repeat { l.step_sum.map(|v| v * d) } else { l.one_shot_step(self.phase, self.phase + d) };
            self.phase = if l.repeat { (self.phase + d).rem_euclid(1.0) } else { self.phase + d };
            step.played_to = self.phase;
            let t = self.waiting.take().expect("reach comes from waiting");
            notes.push(format!("{played} reached phase {:.4}: {} starts", self.phase, t.via));
            self.playing = Some(t);
            self.movement = t.via;
            self.phase = 0.0;
            own
        } else if l.repeat {
            // Loops: the step spread evenly over the loop (Q24).
            self.phase = (self.phase + share).rem_euclid(1.0);
            step.played_to = self.phase;
            l.step_sum.map(|v| v * share)
        } else {
            // One-shots run forward, frame by frame, and stop on the last
            // frame.
            let from = self.phase;
            self.phase = (self.phase + share.abs()).min(1.0);
            if self.phase > 1.0 - 1e-4 {
                self.phase = 1.0;
            }
            step.played_to = self.phase;
            let own = l.one_shot_step(from, self.phase).map(|v| v * l.speed.signum());
            if self.phase >= 1.0 && self.mode == Mode::Free {
                match self.playing {
                    Some(t) => self.end_transition(t, loops, &mut notes),
                    // Rule 6, last case: a one-shot ending with no
                    // transition in progress.
                    None => {
                        if let Some(w) = self.wanted.take() {
                            notes.push(format!("{played} ended: {w} from phase 0"));
                            self.movement = w;
                            self.phase = 0.0;
                        }
                    }
                }
            }
            own
        };
        step.own_step = own;
        let [x, y, z] = own;
        // Turn the model-space step by the heading (positive = left, CCW).
        let (s, c) = self.heading.sin_cos();
        step.wanted = [x * c - y * s, x * s + y * c, z];
        finish(step, self, notes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A lookup over a list of named loops, as the viewer gives it.
    fn lookup<'a>(list: &'a [(&'static str, MovementLoop)]) -> impl Fn(&str) -> Option<&'a MovementLoop> + 'a {
        move |m| list.iter().find(|(n, _)| *n == m).map(|(_, l)| l)
    }

    fn intent(forward: i8, turn: i8, run: bool) -> Intent {
        Intent { forward, turn, run }
    }

    #[test]
    fn head_on_stops_walking_and_stepping_back_and_crashes_running() {
        for (movement, change, mode) in [
            ("walk", Some(("walk", "stop")), Mode::Free),
            ("back", Some(("back", "stop")), Mode::Free),
            ("run", Some(("run", "crash")), Mode::Crash),
            ("stop", None, Mode::Free),
        ] {
            let mut w = Walker::new(0.0);
            w.movement = movement;
            w.phase = 0.5;
            assert_eq!(w.head_on_at_wall(), change, "{movement}");
            assert_eq!(w.mode, mode, "{movement}");
        }
        let mut w = Walker::new(0.0);
        w.movement = "walk";
        w.set_mode(Mode::Fall);
        assert_eq!(w.head_on_at_wall(), None, "not while falling");
        w.movement = "crash";
        w.set_mode(Mode::Crash);
        assert_eq!(w.head_on_at_wall(), None, "not while crashing");
    }

    /// `crash` as in the data: 8 frames in 7/15 s, 441.5 back (+Y in the
    /// model, which faces −Y).
    fn crash() -> MovementLoop {
        MovementLoop { duration: 7.0 / 15.0, step_sum: [0.0, 441.5, 0.0], speed: 1.0, repeat: false, frames: 8, ..Default::default() }
    }

    #[test]
    fn running_head_on_crashes_7_updates_441_5_back_then_runs() {
        let list = [("run", run()), ("crash", crash())];
        let loops = lookup(&list);
        let held = intent(1, 1, true);
        let mut w = Walker::new(0.0);
        w.update(held, &loops);
        assert_eq!(w.head_on_at_wall(), Some(("run", "crash")));
        // Straight back from his facing (he turned a little while running).
        let heading = w.heading;
        let behind = [-heading.sin(), heading.cos()];
        let mut back = 0.0;
        for n in 1..=7 {
            let s = w.update(held, &loops);
            assert_eq!((w.movement, w.mode), ("crash", Mode::Crash), "update {n}");
            let along = s.wanted[0] * behind[0] + s.wanted[1] * behind[1];
            assert!((along - 63.07).abs() < 0.01 && s.wanted[0].hypot(s.wanted[1]) - along < 1e-3, "update {n}: {s:?}");
            assert_eq!(crash().frame_at(w.phase), n, "update {n}: frame");
            back += along;
        }
        assert!((back - 441.5).abs() < 0.01, "{back}");
        // Keys ignored while it played (no turning); then running again.
        assert_eq!(w.heading, heading);
        let s = w.update(held, &loops);
        assert_eq!(s.mode_changed, Some((Mode::Crash, Mode::Free)));
        assert_eq!(w.movement, "run");
        assert!(s.wanted[0] * behind[0] + s.wanted[1] * behind[1] < 0.0, "runs forward: {s:?}");
    }

    fn stop_list() -> Vec<(&'static str, MovementLoop)> {
        vec![("walk", walk(1.0)), ("back", walk(-1.0)), ("stop", MovementLoop { step_sum: [0.0; 3], ..walk(1.0) })]
    }

    #[test]
    fn a_wall_stop_blocks_until_the_key_is_released() {
        for (forward, movement, key) in [(1, "walk", MoveKey::Forward), (-1, "back", MoveKey::Back)] {
            let mut w = Walker::new(0.0);
            w.update(intent(forward, 0, false), &lookup(&stop_list()));
            assert_eq!(w.head_on_at_wall(), Some((movement, "stop")));
            assert_eq!(w.blocked_name(), if forward > 0 { "fwd" } else { "back" });
            // Key held for 3 s: standing, the idle animation playing on.
            let mut last = w.phase;
            for n in 0..45 {
                let s = w.update(intent(forward, 0, false), &lookup(&stop_list()));
                assert_eq!((w.movement, s.wanted), ("stop", [0.0; 3]), "{movement} update {n}");
                assert_ne!(w.phase, last, "{movement} update {n}: stop's frame moves on");
                last = w.phase;
            }
            // Pressing the same key again clears nothing; releasing it does.
            assert_eq!(w.key_event(key, true), (false, false));
            assert_eq!(w.key_event(key, false), (forward > 0, forward < 0));
            w.update(intent(0, 0, false), &lookup(&stop_list()));
            assert_eq!(w.key_event(key, true), (false, false));
            w.update(intent(forward, 0, false), &lookup(&stop_list()));
            assert_eq!(w.movement, movement);
        }
    }

    #[test]
    fn while_blocked_he_neither_runs_nor_turns_on_the_spot() {
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &lookup(&stop_list()));
        w.head_on_at_wall();
        for keys in [intent(1, 0, true), intent(1, 1, false), intent(0, -1, false), intent(-1, 0, false)] {
            w.update(keys, &lookup(&stop_list()));
            assert_eq!((w.movement, w.heading), ("stop", 0.0), "{keys:?}");
        }
        // A mode change (falling, landing) leaves the block on.
        w.set_mode(Mode::Fall);
        w.set_mode(Mode::Free);
        assert!(w.blocked_forward);
    }

    #[test]
    fn which_key_events_clear_which_block() {
        use MoveKey::*;
        let both = || Walker { blocked_forward: true, blocked_back: true, ..Walker::new(0.0) };
        for (key, pressed, cleared) in [
            (Forward, false, (true, false)),
            (Back, false, (false, true)),
            (Forward, true, (false, true)),
            (Back, true, (true, false)),
            (Turn, true, (true, true)),
            (Turn, false, (false, false)),
        ] {
            let mut w = both();
            assert_eq!(w.key_event(key, pressed), cleared, "{key:?} pressed {pressed}");
            assert_eq!((w.blocked_forward, w.blocked_back), (!cleared.0, !cleared.1), "{key:?} pressed {pressed}");
        }
        // Forward held against the wall, a turn key pressed: he walks
        // again, turning as he goes (forward wins over turning).
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &lookup(&stop_list()));
        w.head_on_at_wall();
        w.key_event(Turn, true);
        w.update(intent(1, 1, false), &lookup(&stop_list()));
        assert_eq!(w.movement, "walk");
        assert!(w.heading > 0.0);
    }

    #[test]
    fn movement_table() {
        assert_eq!(choose_movement(intent(1, 0, true)), "run");
        assert_eq!(choose_movement(intent(1, 1, false)), "walk");
        assert_eq!(choose_movement(intent(-1, -1, true)), "back");
        assert_eq!(choose_movement(intent(0, 1, true)), "left");
        assert_eq!(choose_movement(intent(0, -1, false)), "right");
        assert_eq!(choose_movement(intent(0, 0, true)), "stop");
    }

    /// The note's walk: 1,427.1 per 1.2667 s loop, along −Y.
    fn walk(speed: f32) -> MovementLoop {
        MovementLoop { duration: 1.2667, step_sum: [0.0, -1427.1, 0.0], speed, repeat: true, frames: 20, cut_short: true, ..Default::default() }
    }

    /// A one-shot of 11 frames, 10/15 s from the first to the last (as
    /// `jumpend` in the data).
    fn once() -> MovementLoop {
        MovementLoop { duration: 10.0 / 15.0, step_sum: [0.0; 3], speed: 1.0, repeat: false, frames: 11, ..Default::default() }
    }

    #[test]
    fn landing_plays_once_then_the_keys_take_over() {
        let mut w = Walker::new(0.0);
        let list = [("walk", walk(1.0)), ("jumphold", MovementLoop { repeat: true, ..once() }), ("jumpend", once())];
        let loops = lookup(&list);
        w.set_mode(Mode::Fall);
        for _ in 0..3 {
            let s = w.update(intent(1, 1, false), &loops);
            assert_eq!(w.movement, "jumphold");
            assert_eq!(s.wanted, [0.0; 3], "no walking while falling");
        }
        assert_eq!(w.heading, 0.0, "no turning while falling");
        w.set_mode(Mode::Land);
        // 10 updates of jumpend (10/15 s), keys ignored; then walking.
        for n in 1..=10 {
            w.update(intent(1, 0, false), &loops);
            assert_eq!((w.movement, w.mode), ("jumpend", Mode::Land), "update {n}");
        }
        assert_eq!(once().frame_at(w.phase), 10, "last frame shown");
        let s = w.update(intent(1, 0, false), &loops);
        assert_eq!(s.mode_changed, Some((Mode::Land, Mode::Free)));
        assert_eq!(w.movement, "walk");
    }

    #[test]
    fn dying_ends_dead_on_the_last_frame() {
        let mut w = Walker::new(0.0);
        let list = [("jumpdie", once())];
        let loops = lookup(&list);
        w.set_mode(Mode::Dying);
        let mut changes = Vec::new();
        for _ in 0..20 {
            changes.extend(w.update(intent(1, 0, false), &loops).mode_changed);
        }
        assert_eq!(changes, vec![(Mode::Dying, Mode::Dead)]);
        assert_eq!((w.movement, w.mode), ("jumpdie", Mode::Dead));
        assert_eq!(once().frame_at(w.phase), 10);
    }

    #[test]
    fn walking_10_seconds_covers_about_11270() {
        let mut w = Walker::new(0.0);
        let list = [("walk", walk(1.0))];
        let loops = lookup(&list);
        let mut y = 0.0;
        for _ in 0..150 {
            y += w.update(intent(1, 0, false), &loops).wanted[1];
        }
        assert!((y + 11_266.6).abs() < 5.0, "{y}");
        assert!((walk(1.0).ground_speed() - 1126.6).abs() < 1.0);
    }

    #[test]
    fn back_moves_the_other_way() {
        let mut w = Walker::new(0.0);
        let list = [("back", walk(-1.0))];
        let loops = lookup(&list);
        let s = w.update(intent(-1, 0, false), &loops);
        assert!(s.wanted[1] > 0.0, "{s:?}");
        assert_eq!(s.changed, Some(("stop", "back")));
    }

    #[test]
    fn turning_left_2_seconds_is_3_5_radians() {
        let mut w = Walker::new(0.0);
        let loops = lookup(&[]);
        for _ in 0..30 {
            w.update(intent(0, 1, false), &loops);
        }
        assert!((w.heading - 3.5).abs() < 1e-4, "{}", w.heading);
    }

    #[test]
    fn heading_turns_the_step() {
        // Turned left by 90°: facing −Y becomes facing +X.
        let mut w = Walker::new(std::f32::consts::FRAC_PI_2);
        let list = [("walk", walk(1.0))];
        let loops = lookup(&list);
        let s = w.update(intent(1, 0, false), &loops);
        assert!(s.wanted[0] > 0.0 && s.wanted[1].abs() < 1e-3, "{s:?}");
    }

    // The start transitions with the note's data (connor-movement.md
    // "Starting and stopping", "Steps in the start animations"). Model
    // space: +x = his left, −y = forward.

    /// `run` (`c_run.kqs`): 15 frames, 14/15 s, a loop that can be cut short.
    fn run() -> MovementLoop {
        MovementLoop {
            duration: 14.0 / 15.0,
            step_sum: [0.0, -2350.2, 0.0],
            speed: 1.0,
            repeat: true,
            frames: 15,
            cut_short: true,
            ..Default::default()
        }
    }

    /// One-shot transition animation from its per-frame steps (left,
    /// forward); frame 0's step is zero. Plays (frames − 1) / 15 s, `0 0`.
    fn transition_anim(steps: &[(f32, f32)]) -> MovementLoop {
        let mut frame_steps = vec![[0.0f32; 3]];
        frame_steps.extend(steps.iter().map(|&(left, fwd)| [left, -fwd, 0.0]));
        let step_sum = frame_steps.iter().fold([0.0; 3], |a, s| [a[0] + s[0], a[1] + s[1], a[2] + s[2]]);
        MovementLoop {
            duration: steps.len() as f32 / 15.0,
            step_sum,
            speed: 1.0,
            repeat: false,
            frames: steps.len() + 1,
            frame_steps,
            cut_short: false,
            transitions: Vec::new(),
        }
    }

    const STOPRUN: [(f32, f32); 10] = [
        (0.18, 0.19),
        (1.32, 1.28),
        (3.85, 3.15),
        (7.81, 5.73),
        (8.79, 12.77),
        (6.11, 86.57),
        (6.04, 129.17),
        (4.43, 136.26),
        (3.59, 107.78),
        (1.77, 43.80),
    ];

    /// stop, walk, run, stopwalk and stoprun as in `Conner.anm`.
    fn start_list() -> Vec<(&'static str, MovementLoop)> {
        let stop = MovementLoop {
            duration: 29.0 / 15.0,
            step_sum: [0.0; 3],
            speed: 1.0,
            repeat: true,
            frames: 30,
            cut_short: true,
            transitions: vec![
                Transition { from: -1.0, to: "run", to_phase: 0.866667, via: "stoprun" },
                Transition { from: -1.0, to: "walk", to_phase: 0.85, via: "stopwalk" },
            ],
            ..Default::default()
        };
        let walk = MovementLoop { duration: 19.0 / 15.0, ..walk(1.0) };
        vec![
            ("stop", stop),
            ("walk", walk),
            ("run", run()),
            ("stopwalk", transition_anim(&[(7.32, 11.22); 7])),
            ("stoprun", transition_anim(&STOPRUN)),
            ("crash", crash()),
        ]
    }

    /// Plays the start from standing with forward held; returns the updates
    /// of the transition and the walker after it.
    fn start(run: bool) -> (Vec<Step>, Walker) {
        let list = start_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        let first = w.update(intent(1, 0, run), &loops);
        let via = if run { "stoprun" } else { "stopwalk" };
        // The update the key is seen: stop reaches its from point (−1, at
        // once) with no step; the transition plays from the next one.
        assert_eq!((first.played, first.own_step), ("stop", [0.0; 3]));
        assert_eq!((w.movement, w.phase), (via, 0.0));
        let mut steps = Vec::new();
        while steps.len() < 20 {
            let s = w.update(intent(1, 0, run), &loops);
            assert_eq!(s.played, via, "update {}", steps.len() + 2);
            let ended = w.movement != via;
            steps.push(s);
            if ended {
                break;
            }
        }
        (steps, w)
    }

    fn totals(steps: &[Step]) -> (f32, f32) {
        steps.iter().fold((0.0, 0.0), |(l, f), s| (l + s.own_step[0], f - s.own_step[1]))
    }

    #[test]
    fn stopwalk_7_updates_then_walk_from_0_85() {
        let (steps, mut w) = start(false);
        assert_eq!(steps.len(), 7);
        for s in &steps {
            assert!((s.own_step[0] - 7.32).abs() < 1e-3 && (s.own_step[1] + 11.22).abs() < 1e-3, "{s:?}");
        }
        let (left, forward) = totals(&steps);
        assert!((left - 51.24).abs() < 0.01 && (forward - 78.54).abs() < 0.01, "{left} {forward}");
        assert_eq!((w.movement, w.phase, w.playing, w.wanted), ("walk", 0.85, None, None));
        let list = start_list();
        let s = w.update(intent(1, 0, false), &lookup(&list));
        assert_eq!((s.played, s.played_from), ("walk", 0.85));
    }

    #[test]
    fn stoprun_10_updates_with_its_uneven_steps_then_run_from_0_867() {
        let (steps, mut w) = start(true);
        assert_eq!(steps.len(), 10);
        for (s, (left, fwd)) in steps.iter().zip(STOPRUN) {
            assert!((s.own_step[0] - left).abs() < 1e-3 && (s.own_step[1] + fwd).abs() < 1e-3, "{s:?}");
        }
        let (left, forward) = totals(&steps);
        // The note's total says 536.7 forward, but its own per-frame steps
        // and `c_stprn.kqs` both add up to 526.7 (Q40 c).
        assert!((left - 43.89).abs() < 0.01 && (forward - 526.7).abs() < 0.01, "{left} {forward}");
        assert_eq!((w.movement, w.phase), ("run", 0.866667));
        let list = start_list();
        let s = w.update(intent(1, 0, true), &lookup(&list));
        assert_eq!((s.played, s.played_from), ("run", 0.866667));
    }

    #[test]
    fn start_steps_go_to_his_left() {
        // Facing −Y (heading 0) his left is +X; facing +X (heading π/2) it
        // is +Y.
        let list = start_list();
        let loops = lookup(&list);
        for (heading, left) in [(0.0, [1.0, 0.0]), (std::f32::consts::FRAC_PI_2, [0.0, 1.0])] {
            let mut w = Walker::new(heading);
            w.update(intent(1, 0, false), &loops);
            let s = w.update(intent(1, 0, false), &loops);
            let along = s.wanted[0] * left[0] + s.wanted[1] * left[1];
            assert!((along - 7.32).abs() < 1e-3, "heading {heading}: {s:?}");
        }
    }

    #[test]
    fn letting_go_during_stopwalk_plays_it_to_its_end_then_stands() {
        let list = start_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &loops);
        w.update(intent(1, 0, false), &loops);
        w.update(intent(1, 0, false), &loops);
        let played: Vec<_> = (0..7).map(|_| w.update(intent(0, 0, false), &loops).played).collect();
        // 2 updates of stopwalk before letting go, 5 after; no stop
        // animation is loaded (2g-1): walk at 0.85, then stop at once.
        let expected: Vec<_> = ["stopwalk"; 5].into_iter().chain(["stop"; 2]).collect();
        assert_eq!(played, expected);
        assert_eq!((w.movement, w.wanted, w.playing), ("stop", None, None));
    }

    #[test]
    fn run_switched_on_during_stopwalk_runs_from_0_when_it_ends() {
        let list = start_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &loops);
        w.update(intent(1, 0, false), &loops);
        let played: Vec<_> = (0..7).map(|_| w.update(intent(1, 0, true), &loops).played).collect();
        assert_eq!(played[..6], ["stopwalk"; 6]);
        assert_eq!((played[6], w.movement), ("run", "run"));
        assert!(w.phase > 0.0 && w.phase < 0.1, "run from phase 0: {}", w.phase);
    }

    #[test]
    fn head_on_during_stopwalk_stops_and_during_stoprun_crashes() {
        let list = start_list();
        let loops = lookup(&list);
        for (run, change, blocked) in [(false, ("stopwalk", "stop"), "fwd"), (true, ("stoprun", "crash"), "none")] {
            let mut w = Walker::new(0.0);
            w.update(intent(1, 0, run), &loops);
            w.update(intent(1, 0, run), &loops);
            assert!(w.moving());
            assert_eq!(w.head_on_at_wall(), Some(change));
            assert_eq!((w.blocked_name(), w.playing, w.waiting, w.wanted), (blocked, None, None, None));
        }
    }

    #[test]
    fn he_turns_during_a_start() {
        let list = start_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 1, false), &loops);
        let before = w.heading;
        let s = w.update(intent(1, 1, false), &loops);
        assert_eq!(s.played, "stopwalk");
        assert!((w.heading - before - TURN_RATE / 15.0).abs() < 1e-5);
    }

    #[test]
    fn after_a_crash_with_run_held_stoprun_plays_before_run() {
        let list = start_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.movement = "run";
        w.choice = "run";
        assert_eq!(w.head_on_at_wall(), Some(("run", "crash")));
        let played: Vec<_> = (0..20).map(|_| w.update(intent(1, 0, true), &loops).played).collect();
        let expected: Vec<_> = ["crash"; 7].into_iter().chain(["stop"]).chain(["stoprun"; 10]).chain(["run"; 2]).collect();
        assert_eq!(played, expected);
    }

    #[test]
    fn choosing_the_next_from_point_ahead() {
        // run's transitions in `Conner.anm` (waterrun's left out), plus a
        // tie at 0.6.
        let t = |from, via| Transition { from, to: "stop", to_phase: 0.0, via };
        let running = MovementLoop {
            transitions: vec![t(0.066667, "runstopright"), t(0.866667, "stoprunstop"), t(0.6, "runstopleft"), t(0.6, "tie")],
            ..run()
        };
        let via = |phase| running.choose_transition(phase, "stop").map(|(t, d)| (t.via, (d * 1e4).round() / 1e4));
        assert_eq!(via(0.5), Some(("runstopleft", 0.1)), "next ahead; the tie goes to the first listed");
        assert_eq!(via(0.7), Some(("stoprunstop", 0.1667)));
        assert_eq!(via(0.866667), Some(("stoprunstop", 0.0)), "distance 0");
        assert_eq!(via(0.9), Some(("runstopright", 0.1667)), "after the wrap");
        assert_eq!(running.choose_transition(0.5, "walk"), None);
        // −1 on a loop always wins; on a one-shot a point behind is never
        // reached.
        let at_once = MovementLoop { transitions: vec![t(0.6, "later"), t(-1.0, "now")], ..run() };
        assert_eq!(at_once.choose_transition(0.3, "stop").map(|(t, _)| t.via), Some("now"));
        let once = MovementLoop { repeat: false, ..running };
        assert_eq!(once.choose_transition(0.95, "stop"), None);
        assert_eq!(once.choose_transition(0.5, "stop").map(|(t, _)| t.via), Some("runstopleft"));
    }

    #[test]
    fn one_shot_steps_frame_by_frame() {
        let a = transition_anim(&[(1.0, 0.0), (0.0, 10.0)]);
        // Half of frame 1 and a quarter of frame 2.
        let s = a.one_shot_step(0.25, 0.625);
        assert!((s[0] - 0.5).abs() < 1e-6 && (s[1] + 2.5).abs() < 1e-6, "{s:?}");
    }
}
