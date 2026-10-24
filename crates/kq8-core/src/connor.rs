//! Connor's ground movement: which movement plays, turning, and the step
//! his animation asks the mover for.
//!
//! Rules from the behaviour note `docs/behaviour/connor-movement.md`
//! ("Movement intent", "Which movement plays", "Forward speed comes from
//! the animation", "Turning"). M2 step 2b: a loop's step spread
//! evenly over the loop (Q24). Banking (M2 step 2i, "Turning" rule 3):
//! `Walker::bank`.
//!
//! Start and stop transitions (M2 step 2g, "Starting and stopping", rules
//! 1–6): the movement asked for, the transition chosen by its from point,
//! played to its end with its own frame-by-frame steps, then its `to`
//! movement at its `to` phase.
//!
//! The falling animations (M2 step 2c/2e, `connor-physics.md` "Falling off
//! an edge"): `jumphold` while falling, then `jumpend` (safe landing) or
//! `jumpdie` (death), each played once, with the keys ignored meanwhile.
//!
//! Scripted animations, the Special state (M3 step 3i-7,
//! `anim-scripts.md` rule 16): `KQMonster::setState Connor special
//! <list>`; the items one at a time ([`special_steps`]), each movement
//! played to its end before the next, then standing.

use std::collections::BTreeSet;
use std::sync::Mutex;

use crate::UPDATES_PER_SECOND;

/// A movement name that lives as long as the program: Connor's movements
/// are `&'static str`, and the Special state's lists add names read from
/// the game's files (a few dozen per world).
pub fn intern(name: &str) -> &'static str {
    static NAMES: Mutex<BTreeSet<&'static str>> = Mutex::new(BTreeSet::new());
    let mut names = NAMES.lock().unwrap_or_else(|e| e.into_inner());
    if let Some(n) = names.get(name) {
        return n;
    }
    let n: &'static str = Box::leak(name.to_string().into_boxed_str());
    names.insert(n);
    n
}

/// The name a movement of a Special list is loaded under: `<list>/<name>`
/// (list file name in small letters), so it can't clash with his own.
pub fn special_movement_name(list: &str, name: &str) -> String {
    format!("{}/{name}", list.to_ascii_lowercase())
}

/// The name scripts know a movement by (`push` for `bricks.anm/push`).
pub fn plain_movement_name(name: &str) -> &str {
    name.rsplit('/').next().unwrap_or(name)
}

/// One item of a Special list, worked out (`anim-scripts.md` rule 16).
#[derive(Debug, Clone, PartialEq)]
pub enum SpecialStep {
    /// A movement to play to its end: the loaded name, and the word as
    /// written.
    Play(&'static str, String),
    /// `KillConnor`: his Die state.
    Kill,
    /// Something only logged (`AnimList=`, `AnimListOn`/`Off`, `Inv`, a
    /// movement that isn't loaded).
    Note(String),
}

/// Works out a Special list (`AnimList=bricks.anm,AnimListOn,push`) left
/// to right: `AnimList=<file>` loads that list, `AnimListOn` makes names
/// come from it, `AnimListOff` from his own again, `Inv <item>` (two
/// items) is logged only (not traced), `KillConnor` his Die state, any
/// other word a movement. `resolve(list, word)` finds the loaded
/// movement (`list` is `None` for his own), ignoring capitals.
pub fn special_steps(items: &str, resolve: &dyn Fn(Option<&str>, &str) -> Option<&'static str>) -> Vec<SpecialStep> {
    let mut out = Vec::new();
    let (mut loaded, mut on) = (None::<String>, false);
    let mut words = items.split(',').map(str::trim).filter(|w| !w.is_empty());
    while let Some(w) = words.next() {
        if let Some(file) = w.strip_prefix("AnimList=").or_else(|| w.strip_prefix("animlist=")) {
            loaded = Some(file.to_string());
            out.push(SpecialStep::Note(format!("AnimList={file}: list loaded")));
        } else if w.eq_ignore_ascii_case("AnimListOn") {
            on = true;
            out.push(SpecialStep::Note(format!("AnimListOn: names from {}", loaded.as_deref().unwrap_or("(no list loaded)"))));
        } else if w.eq_ignore_ascii_case("AnimListOff") {
            on = false;
            out.push(SpecialStep::Note("AnimListOff: names from his own list".into()));
        } else if w.eq_ignore_ascii_case("Inv") {
            let item = words.next().unwrap_or("");
            out.push(SpecialStep::Note(format!("Inv {item}: holds that item (not traced, not done)")));
        } else if w.eq_ignore_ascii_case("KillConnor") {
            out.push(SpecialStep::Kill);
        } else {
            let list = if on { loaded.as_deref() } else { None };
            match resolve(list, w) {
                Some(name) => out.push(SpecialStep::Play(name, w.to_string())),
                None => out.push(SpecialStep::Note(format!("movement {w} not loaded from {}: skipped (stand-in)", list.unwrap_or("his own list")))),
            }
        }
    }
    out
}

/// Turning speed while a turn key is held, radians per second.
pub const TURN_RATE: f32 = 1.75;

/// Banking ("Turning" rule 3): the lean takes another step only while it
/// is still short of this on the turn's side (checked before the step),
/// so it ends one step past it, at 0.18 rad.
pub const BANK_LIMIT: f32 = 0.15;
/// How fast the lean changes, radians per second: the original's 0.03 per
/// update at 15 updates per second, made per second as the note's "Do not
/// reproduce" asks.
pub const BANK_RATE: f32 = 0.45;
/// Returning: within this of upright, the lean becomes exactly upright.
pub const BANK_SNAP: f32 = 0.029;

/// What the player wants: forward/back and turn are −1, 0 or +1.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Intent {
    pub forward: i8,
    pub turn: i8,
    pub run: bool,
}

/// The movement name for an intent (the note's table; no water yet).
pub fn choose_movement(intent: Intent, submerged: bool) -> &'static str {
    match (intent.forward, intent.turn) {
        // In water (water.md "Wading animations"): the wading ones.
        (1, _) if intent.run && submerged => "waterrun",
        (1, _) if submerged => "waterwalk",
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
    /// The `loop=` start position, 0..1 (`anim-scripts.md` rule 5).
    pub start: f32,
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
    /// Its `cueN=<phase> <action>` lines, in file order (M3 3h-4).
    pub cues: Vec<(f32, String)>,
}

/// Just above 1: the end of the range checked up to a forward loop's wrap
/// (`objects.md` "Loops" rule 4), so a cue at 1.0 fires once per cycle.
pub const WRAP_END: f32 = 1.0001;

impl MovementLoop {
    /// The cues passed when the phase went from `from` to `to` in one
    /// update (`doors.md` rule 9: a cue at c fires when c is in [lower,
    /// higher)). A forward loop that wrapped checks [from, 1.0001) and
    /// [0, to) (`objects.md` "Loops" rule 4); a backward loop that wrapped
    /// [0, from) and [to, 1.0001) (our mirror of it, Q81 c).
    pub fn cues_between(&self, from: f32, to: f32) -> Vec<&str> {
        let ranges: [(f32, f32); 2] = if self.repeat && self.speed >= 0.0 && to < from {
            [(from, WRAP_END), (0.0, to)]
        } else if self.repeat && self.speed < 0.0 && to > from {
            [(0.0, from), (to, WRAP_END)]
        } else {
            [(from.min(to), from.max(to)), (0.0, 0.0)]
        };
        self.cues
            .iter()
            .filter(|(c, _)| ranges.iter().any(|&(lo, hi)| *c >= lo && *c < hi))
            .map(|(_, a)| a.as_str())
            .collect()
    }

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

    /// The walker's phase a movement starts at: its `loop=` start; a
    /// play-once movement played backwards runs our phase 0 → 1 turned
    /// round ([`Self::frame_at`]), so there it is 1 − start.
    pub fn start_phase(&self) -> f32 {
        if self.repeat {
            self.start.rem_euclid(1.0)
        } else if self.speed < 0.0 {
            1.0 - self.start.clamp(0.0, 1.0)
        } else {
            self.start.clamp(0.0, 1.0)
        }
    }

    /// The frame shown at `phase` (0..1 of one play): floor(phase ×
    /// frames + 0.001), cut down, as for every animated object
    /// (`objects.md` "Loops" rule 6, Q153). A one-shot with speed −1 (`undraw`,
    /// `c_shth`) shows its frames last to first (`anim-scripts.md` rule 5;
    /// M5 5c fix): our phase runs 0..1 and is turned round here (it starts
    /// at [`Self::start_phase`]).
    pub fn frame_at(&self, phase: f32) -> usize {
        let p = if self.repeat {
            phase.rem_euclid(1.0)
        } else if self.speed < 0.0 {
            1.0 - phase.clamp(0.0, 1.0)
        } else {
            phase
        };
        kq8_data::shape::frame_at_phase(p, self.frames, self.repeat)
    }
}

/// The three jumps (connor-physics.md "Jumping"), each with its own hold,
/// landing and death animations. A walk-off falls as `Stand`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum JumpKind {
    #[default]
    Stand,
    Run,
    Back,
}

impl JumpKind {
    /// Rule 2: `runJump` from `run`, `backJump` from `back`, otherwise `jump`
    /// (the movement the keys ask for, Q112).
    pub fn for_movement(movement: &str) -> JumpKind {
        match movement {
            "run" => JumpKind::Run,
            "back" => JumpKind::Back,
            _ => JumpKind::Stand,
        }
    }

    pub fn jump(&self) -> &'static str {
        match self {
            JumpKind::Stand => "jump",
            JumpKind::Run => "runJump",
            JumpKind::Back => "backJump",
        }
    }

    /// Played while falling (repeats).
    pub fn hold(&self) -> &'static str {
        match self {
            JumpKind::Stand => "jumphold",
            JumpKind::Run => "runJumpHold",
            JumpKind::Back => "backJumpHold",
        }
    }

    /// Played once on a safe landing; `runJump` has none.
    pub fn end(&self) -> Option<&'static str> {
        match self {
            JumpKind::Stand => Some("jumpend"),
            JumpKind::Run => None,
            JumpKind::Back => Some("backJumpEnd"),
        }
    }

    pub fn die(&self) -> &'static str {
        match self {
            JumpKind::Stand => "jumpdie",
            JumpKind::Run => "runJumpDie",
            JumpKind::Back => "backJumpDie",
        }
    }
}

/// The movements a jump can start from (the user's check in the
/// original: standing, full walking or full running, never during a start
/// or stop animation; `back` from the note's rule 2; turning on the spot:
/// Q41 a). Wading too: the user's check in the original (2026-10-12),
/// walking or running in water gives the standing jump.
pub const JUMP_FROM: [&str; 8] = ["stop", "walk", "run", "back", "left", "right", "waterwalk", "waterrun"];

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
    /// Jumping: the jump animation once, keys ignored; when it ends the
    /// game starts Fall (`connor_update.rs`).
    Jump,
    /// A script's list of movements (`anim-scripts.md` rule 16), keys
    /// ignored; when the list is used up, back to the keys.
    Special,
    /// Pushing (`pushing.md` rules 6–8): `push` plays instead of the keys'
    /// choice; turning as when walking (Q87 d stand-in).
    Push,
    /// His talk state in a conversation (`conversation.md` rules 6–15, M4
    /// 4e-3): the approach movement in [`Walker::talk_play`] plays (forward
    /// and back do nothing then; the turn keys turn him, rule 13, Q125),
    /// else the keys choose as in Wait and Move (rule 13). No Jump, no
    /// Push (rule 13, Q125 b, c).
    Talk,
    /// The rope state (`ropeandhook.md` rules 13–26; M6 6a-3): the rope
    /// sequence's movement plays (set by the caller), keys ignored, no
    /// turning; while hanging ([`Walker::rope_hang`]) the climb is moved
    /// by forward and back only (rule 22). Fall, landing, push and wall
    /// events don't change it (rule 28).
    Rope,
    /// The hammer-throw state (`azriel-hammer.md` rule 3; M6 6b-3): the
    /// throw's movement plays (`attack`, `wait` repeating, `catch`; set by
    /// the caller), keys ignored, no key turning (the automatic turn still
    /// turns him, rule 2). Fall, landing, push and wall events don't change
    /// it: it can't be left before `catch` has ended.
    Throw,
}

/// Updates without a pushing touch before the Push state ends
/// (`pushing.md` rule 7).
pub const PUSH_UPDATES: u32 = 4;

/// Hand mode with empty hands (`pushing.md`: sword 0, bow 1, empty 4).
pub const EMPTY_HANDS: i64 = 4;
/// Hand mode with a hand weapon ready (the fists too, `inventory.md` rule 9).
pub const SWORD_MODE: i64 = 0;

impl Mode {
    pub fn name(&self) -> &'static str {
        match self {
            Mode::Free => "free",
            Mode::Fall => "fall",
            Mode::Land => "land",
            Mode::Dying => "dying",
            Mode::Dead => "dead",
            Mode::Crash => "crash",
            Mode::Jump => "jump",
            Mode::Special => "special",
            Mode::Push => "push",
            Mode::Talk => "talk",
            Mode::Rope => "rope",
            Mode::Throw => "throw",
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
    /// The mover's `submerged` from the end of the last update's move
    /// (water.md "Wading animations" rule 3).
    pub submerged: bool,
    /// A chosen transition waiting for its from point (rule 3).
    pub waiting: Option<Transition>,
    /// The transition playing now (rules 5 and 6).
    pub playing: Option<Transition>,
    /// The keys' last choice: a request is made only when it changes.
    pub(crate) choice: &'static str,
    /// The last jump's kind, for the hold, landing and death animations
    /// (`Stand` after a walk-off).
    pub kind: JumpKind,
    /// The jump going on is finished (`KQConner::endJump`, or a state
    /// request with `handsOff` on; `anim-scripts.md` rule 28): the Jump
    /// state now lets another state replace it.
    pub jump_finished: bool,
    /// The lean into a turn while running, radians about his forward axis;
    /// positive = leaning to his left (turning left). Drawing only.
    pub bank: f32,
    /// The "lean allowed" switch: set in the keys' updates (on when they
    /// pick `run` or `waterrun`), off while jumping, falling and landing
    /// (the user's check, Q59), kept as it is in the other modes.
    pub lean_allowed: bool,
    /// The Special state's list and the item playing now (rule 16).
    pub special: Vec<SpecialStep>,
    pub special_at: usize,
    /// Updates since the last pushing touch (`pushing.md` rule 7).
    pub push_counter: u32,
    /// Which weapon he holds: 4 = empty hands, the only one so far
    /// (`KQConner::setDoMode` is a stub).
    pub hand_mode: i64,
    /// A scripted turn or walk (`walkTo`, `runTo`, `pointAndCue`;
    /// `anim-scripts.md` rules 24–25), run by `connor_update::tick`.
    pub auto: Option<crate::connor_auto::AutoMove>,
    /// The automatic turn turned him this update (set by
    /// `connor_update::tick`): the turn direction it gives counts as a turn
    /// key for his movement, but his own key turning stays off.
    pub turn_by_script: bool,
    /// A turn key went **down** since the last update (set by the caller,
    /// cleared by `connor_update::tick`): the only thing that cancels an
    /// automatic turn (`connor-movement.md` "Turning" rule 2, Q104 a).
    pub turn_press: bool,
    /// In the talk state: the approach movement playing (`stopwalk`,
    /// `walkstopright`; `conversation.md` rule 10), `None` when the keys
    /// choose (rule 13).
    pub talk_play: Option<&'static str>,
    /// Killed by a hit (`fighting.md` rule 29; M5 5c-4): the death
    /// movement the Dying and Dead modes show instead of the fall's.
    pub death: Option<&'static str>,
    /// In the Rope mode: he hangs on the rope and `movement` is the climb
    /// (`ropeandhook.md` rules 19, 21, 22).
    pub rope_hang: bool,
    /// The climb's direction this update: +1 up (forwards), −1 down, 0
    /// frozen (rule 22).
    pub climb_dir: i8,
    /// A movement was started from phase 0 since the last update: the
    /// next update puts it at its `loop=` start first (`anim-scripts.md`
    /// rule 5, Q138 e). Not for transitions (`connor-movement.md` rule 5:
    /// from phase 0).
    pub fresh: bool,
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
    /// A play-once movement reached its end in this update (for
    /// `getEndLoop` listeners, `clicking.md` rule 21).
    pub ended: Option<&'static str>,
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
            kind: JumpKind::Stand,
            jump_finished: false,
            submerged: false,
            bank: 0.0,
            lean_allowed: false,
            special: Vec::new(),
            special_at: 0,
            push_counter: 0,
            hand_mode: EMPTY_HANDS,
            auto: None,
            turn_by_script: false,
            turn_press: false,
            talk_play: None,
            death: None,
            rope_hang: false,
            climb_dir: 0,
            fresh: false,
        }
    }

    /// The rope state starts or its next movement plays (`ropeandhook.md`
    /// rules 13–17): `movement` from its start, keys ignored.
    pub fn rope_play(&mut self, movement: &'static str) {
        self.clear_transitions();
        self.mode = Mode::Rope;
        self.movement = movement;
        self.phase = 0.0;
        self.fresh = true;
        self.rope_hang = false;
        self.climb_dir = 0;
    }

    /// Rules 19, 21: he hangs on the rope, the climb `movement` at `phase`
    /// (0 = the foot of the rope, 1 = the top), frozen until a key moves it.
    pub fn rope_hang_at(&mut self, movement: &'static str, phase: f32) {
        self.rope_play(movement);
        self.phase = phase;
        self.rope_hang = true;
    }

    /// `azriel-hammer.md` rule 3: the hammer-throw state starts or its next
    /// movement plays: `movement` from its start, keys ignored.
    pub fn throw_play(&mut self, movement: &'static str) {
        self.clear_transitions();
        self.mode = Mode::Throw;
        self.movement = movement;
        self.phase = 0.0;
        self.fresh = true;
    }

    /// Rule 3: `catch` ended: back to standing (the keys choose again).
    pub fn throw_leave(&mut self) {
        self.set_mode(Mode::Free);
    }

    /// Rule 26: the rope state ends; standing (`Wait`).
    pub fn rope_leave(&mut self) {
        self.rope_hang = false;
        self.climb_dir = 0;
        self.set_mode(Mode::Free);
    }

    /// The keys choose his movement: Wait, Move, Push, or the talk state
    /// outside its approach movements.
    pub fn keys_choose(&self) -> bool {
        matches!(self.mode, Mode::Free | Mode::Push) || (self.mode == Mode::Talk && self.talk_play.is_none())
    }

    /// He goes into his talk state (`conversation.md` rule 2): asked for
    /// `Stop` (the keys ask again from here, so with no key held a walk
    /// stops the ordinary way). Returns a line for the log.
    pub fn enter_talk(&mut self) -> String {
        let from = self.mode.name();
        self.mode = Mode::Talk;
        self.talk_play = None;
        self.wanted = None;
        self.waiting = None;
        // No choice yet: the keys' choice (`stop` with no key) is a new
        // request, made the ordinary way.
        self.choice = "";
        format!("talk state (from {from}): asks for Stop the ordinary way; {} playing", self.movement)
    }

    /// A lip-sync gesture code for Connor (`conversation.md` rule 18, M4
    /// 4e-7): codes 8–15 play `gesturea`…`gestureh`, 16 plays `stop`, at
    /// once from phase 0, only when he stands still with the keys choosing
    /// (Wait, or the talk state outside its approach), no start or stop
    /// movement playing, no weapon drawn and not in first person (never, for
    /// now). `name`: the movement, as loaded (`None`: not in his list).
    /// Returns what happened, for the log.
    pub fn gesture(&mut self, code: u16, name: Option<&'static str>) -> String {
        let wanted = match code {
            8..=15 => kq8_data::characters::GESTURES[(code - 8) as usize],
            16 => "stop",
            _ => return format!("code {code}: not a gesture code; nothing"),
        };
        let Some(name) = name else {
            return format!("code {code}: '{wanted}' is not in his movement list; nothing");
        };
        let why = if !self.keys_choose() || self.mode == Mode::Push {
            Some(format!("in {} (not standing in Wait or the talk state)", self.mode.name()))
        } else if self.moving() {
            Some(format!("he is moving ({})", self.choice))
        } else if let Some(t) = self.playing {
            Some(format!("{} is playing", t.via))
        } else if self.hand_mode != EMPTY_HANDS {
            Some("a weapon is drawn".to_string())
        } else {
            None
        };
        if let Some(why) = why {
            return format!("code {code}: '{wanted}' not played: {why} (rule 18)");
        }
        let from = self.movement;
        self.movement = name;
        self.phase = 0.0;
        self.fresh = true;
        self.wanted = None;
        self.waiting = None;
        format!("code {code}: plays '{name}' from phase 0 (was '{from}'; rule 18)")
    }

    /// He leaves his talk state (`conversation.md` rule 15): Move if he is
    /// moving (the movement goes on), else `Stop` and Wait. Returns the
    /// state he goes to.
    pub fn leave_talk(&mut self) -> &'static str {
        if self.mode != Mode::Talk {
            return "not in the talk state";
        }
        self.talk_play = None;
        if self.moving() {
            self.mode = Mode::Free;
            "Move"
        } else {
            self.set_mode(Mode::Free);
            "Wait (Stop)"
        }
    }

    /// What he brings to a push now (`pushing.md` rule 2): his state takes
    /// Push (Wait, Move or Push), already pushing, empty hands, his chosen
    /// movement `walk`, `run` or `push`, his facing.
    pub fn push_ability(&self) -> crate::connor_mover::PushAbility {
        let (s, c) = self.heading.sin_cos();
        crate::connor_mover::PushAbility {
            accepts: matches!(self.mode, Mode::Free | Mode::Push),
            pushing: self.mode == Mode::Push,
            empty_hands: self.hand_mode == EMPTY_HANDS,
            movement_ok: self.mode == Mode::Push || matches!(self.choice, "walk" | "run"),
            // The model faces its −Y, turned by the heading.
            facing: [s, -c],
        }
    }

    /// A pushing touch (`pushing.md` rule 3): the Push state (`push` is
    /// asked for in the next update), the counter back to 0.
    pub fn push_touch(&mut self) -> Option<&'static str> {
        self.push_counter = 0;
        if self.mode == Mode::Push {
            return None;
        }
        self.mode = Mode::Push;
        Some("enters the Push state")
    }

    /// A scripted turn or walk starts (`anim-scripts.md` rules 24–25). A
    /// walk puts him in Wait first (the keys' control, standing); a turn
    /// (`pointAndCue`) only turns him. Returns a line for the log.
    pub fn start_auto(&mut self, a: crate::connor_auto::AutoMove) -> String {
        use crate::connor_auto::{AutoKind, AutoPhase};
        let walk = matches!(a.kind, AutoKind::Walk { .. });
        let mut line = format!("scripted {:?} to ({:.0}, {:.0})", a.kind, a.target[0], a.target[1]);
        // There is only one automatic turn (clicking.md rule 32,
        // anim-scripts.md "Edge cases"): a turn without a cue (click,
        // speak, chest) asked for while a turn is running only replaces
        // its target; a pointAndCue keeps its cue for the new end.
        if let Some(old) = self.auto.as_mut().filter(|o| o.kind == AutoKind::Point && o.phase == AutoPhase::Turning && a.kind == AutoKind::Point && a.cue.is_none()) {
            old.target = a.target;
            let cue = if old.cue.is_some() { "; its cue is kept" } else { "" };
            return format!("{line}: one automatic turn: the running turn (update {}) gets this target{cue}", old.turn_updates);
        }
        if let Some(old) = self.auto.take() {
            line += &format!(" (replaces the unfinished {:?}; its cue never comes)", old.kind);
        }
        if walk && self.mode != Mode::Free {
            line += &format!(" (mode {} -> free: Wait)", self.mode.name());
            self.set_mode(Mode::Free);
        }
        self.auto = Some(a);
        line
    }

    /// A scripted walk ran head-on into something (rule 24, Q94 a): he is
    /// put at the contact (the mover's head-on stop), the cue comes early,
    /// and he goes to Wait (`stop`), with no crash and no wall block.
    pub fn stop_at_once(&mut self) {
        self.clear_transitions();
        self.movement = "stop";
        self.phase = 0.0;
        self.fresh = true;
    }

    /// Rule 2.5 failed while pushing: back to Move (the keys decide).
    pub fn push_back_to_move(&mut self) -> Option<&'static str> {
        (self.mode == Mode::Push).then(|| {
            self.mode = Mode::Free;
            "leaves Push for Move (facing off by more than 60 degrees)"
        })
    }

    /// Straight to standing, its `stop` movement from the start: no stop
    /// transition, nothing waiting, no Special list (M6 6e fix, the user's
    /// check 2026-10-22: after No at a walking exit he must not finish a
    /// walk's stop and step back into the trap).
    pub fn stand_now(&mut self) {
        self.mode = Mode::Free;
        self.movement = "stop";
        self.phase = 0.0;
        self.fresh = true;
        self.wanted = None;
        self.waiting = None;
        self.playing = None;
        self.choice = "stop";
        self.special.clear();
        self.special_at = 0;
        self.blocked_forward = false;
        self.blocked_back = false;
        self.bank = 0.0;
    }

    /// Rule 2.6 failed: the standing state (Wait). Only the state: the
    /// animation goes on (a stop transition already asked for still
    /// plays); the keys ask again next update.
    pub fn stand(&mut self) -> &'static str {
        self.mode = Mode::Free;
        self.choice = "stop";
        "standing state (Wait): his movement can't push"
    }

    /// Rule 3 (traced, Q56), every update whatever the mode:
    /// - building: turning, the switch on and forward held: one step of
    ///   `BANK_RATE` × `dt` (0.03 at 15 per second) towards the turn's
    ///   side, if the lean is still short of `BANK_LIMIT` on that side
    ///   before the step (so it ends at 0.18; the same on both sides, also
    ///   after swinging over: "Do not reproduce");
    /// - returning: not turning, or the switch off: within `BANK_SNAP` of
    ///   upright → upright, otherwise one step towards upright;
    /// - turning with the switch on but forward not held: unchanged.
    fn update_bank(&mut self, intent: Intent, dt: f32) {
        let step = BANK_RATE * dt;
        if intent.turn != 0 && self.lean_allowed {
            if intent.forward == 1 {
                let side = f32::from(intent.turn.signum());
                // "Short of 0.15": a hair of slack, as the original's 0.03
                // steps land a hair under 0.15 (rule 3).
                if self.bank * side < BANK_LIMIT + 1e-4 {
                    self.bank += step * side;
                }
            }
        } else if self.bank.abs() < BANK_SNAP {
            self.bank = 0.0;
        } else {
            self.bank -= step * self.bank.signum();
        }
    }

    /// The movement the mode plays (`None`: the keys decide).
    fn mode_movement(&self) -> Option<&'static str> {
        match self.mode {
            // Push: the keys' path, with `push` as the choice (rule 6).
            Mode::Free | Mode::Push => None,
            Mode::Talk => self.talk_play,
            Mode::Rope | Mode::Throw => Some(self.movement),
            Mode::Fall => Some(self.kind.hold()),
            // runJump has no landing animation; it never gets Land.
            Mode::Land => Some(self.kind.end().unwrap_or("jumpend")),
            Mode::Dying | Mode::Dead => Some(self.death.unwrap_or(self.kind.die())),
            Mode::Crash => Some("crash"),
            Mode::Jump => Some(self.kind.jump()),
            Mode::Special => match self.special.get(self.special_at) {
                Some(SpecialStep::Play(name, _)) => Some(name),
                _ => Some(self.movement),
            },
        }
    }

    /// `KQMonster::setState Connor special <list>` (rule 16): the Special
    /// state; the first movement starts at once (it plays from the next
    /// update). Returns the log lines for the items handled before it.
    pub fn start_special(&mut self, steps: Vec<SpecialStep>) -> Vec<String> {
        self.clear_transitions();
        self.mode = Mode::Special;
        self.special = steps;
        self.special_at = 0;
        let mut notes = Vec::new();
        self.special_items(&mut notes);
        notes
    }

    /// Handles items from `special_at` on until one is a movement (it
    /// starts from phase 0) or the list is used up (back to standing).
    fn special_items(&mut self, notes: &mut Vec<String>) {
        while let Some(step) = self.special.get(self.special_at).cloned() {
            match step {
                SpecialStep::Play(name, written) => {
                    notes.push(format!("special: plays {written} ({name})"));
                    self.movement = name;
                    self.phase = 0.0;
                    self.fresh = true;
                    return;
                }
                SpecialStep::Kill => {
                    // His Die state; our Dead mode holds the last frame of
                    // the fall-death animation (stand-in: what he shows
                    // while dead is M5).
                    notes.push("special: KillConnor: his Die state".into());
                    self.special.clear();
                    self.mode = Mode::Dead;
                    return;
                }
                SpecialStep::Note(n) => notes.push(format!("special: {n}")),
            }
            self.special_at += 1;
        }
        notes.push("special: list used up: back to standing".into());
        self.special.clear();
        self.set_mode(Mode::Free);
    }

    /// Which jump the Jump key would start now, or why none: only in the
    /// keys' control, with no start or stop animation playing, from
    /// `JUMP_FROM` (the ground conditions are the mover's, `connor_update.rs`).
    /// The kind comes from the movement the held keys ask for (`intent`),
    /// not the one playing and not a wall block (connor-physics.md
    /// "Jumping" rule 2, Q112): back held → `backJump` (also at a wall and
    /// in water), forward with running on → `runJump` (in water `jump`),
    /// otherwise `jump`.
    pub fn jump_kind(&self, intent: Intent) -> Result<JumpKind, String> {
        if self.mode != Mode::Free {
            return Err(format!("not in the keys' control (mode {})", self.mode.name()));
        }
        if let Some(t) = self.playing {
            return Err(format!("{} is playing", t.via));
        }
        if !JUMP_FROM.contains(&self.movement) {
            return Err(format!("not from {}", self.movement));
        }
        Ok(JumpKind::for_movement(choose_movement(intent, self.submerged)))
    }

    /// A safe landing (connor-physics.md "Jumping" rule 4): the kind's
    /// landing animation, or for `runJump`, which has none, the keys'
    /// control at once while `runJumpHold` plays on. The keys then always
    /// ask for a movement from there, so `Conner.anm`'s transitions give
    /// `jumprun` (run held), `jumpstop` (forward let go) or a direct switch
    /// (measured in the original: `jumprun` 3 updates then `run`;
    /// `jumpstop` 13 updates then standing).
    pub fn land(&mut self) {
        if self.kind.end().is_some() {
            self.set_mode(Mode::Land);
            return;
        }
        self.clear_transitions();
        self.mode = Mode::Free;
        // No choice yet: whatever the keys choose is a new request.
        self.choice = "";
    }

    /// A standing jump hit something steep in the up or across pass: he
    /// stands (`stop`), as for a walking head-on stop, which sets the
    /// forward wall block (connor-physics.md "Jumping" rule 5, "Walk held
    /// against a wall").
    pub fn stop_after_jump(&mut self) {
        self.set_mode(Mode::Free);
        self.blocked_forward = true;
    }

    /// A jump starts: its animation plays once from the next `update`,
    /// keys ignored. Any waiting transition is dropped.
    pub fn start_jump(&mut self, kind: JumpKind) {
        self.clear_transitions();
        self.kind = kind;
        self.jump_finished = false;
        self.mode = Mode::Jump;
        self.movement = kind.jump();
        self.phase = 0.0;
        self.fresh = true;
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
            self.fresh = true;
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
            self.fresh = true;
            self.wanted = None;
        } else {
            notes.push(format!("{} can't be cut short: {wanted} kept until it ends", self.movement));
        }
    }

    /// A script's `KQAnimPlayEvent <movement>` to Connor (`anim-scripts.md`
    /// rules 4-5: the same request as for any animated object, Connor
    /// included): asked for as the keys ask (rule 4 here: transition, cut
    /// short, else kept for the end), but it starts afresh even when it is
    /// the movement already playing. The keys' choice is left as it was,
    /// so they don't ask again until it changes (rule 1): with `handsOff`
    /// a play-once movement holds its last frame. Only in the states where
    /// the keys choose; elsewhere not done (not described). Returns the
    /// log notes.
    pub fn script_request<'a>(&mut self, wanted: &'static str, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>) -> Vec<String> {
        let mut notes = Vec::new();
        if !self.keys_choose() {
            notes.push(format!("in {}: not done (a request outside his keys' states is not described)", self.mode.name()));
            return notes;
        }
        if wanted == self.movement && self.playing.is_none() && self.waiting.is_none() {
            if loops(self.movement).is_none_or(|l| l.cut_short || self.phase >= 1.0) {
                notes.push(format!("{wanted} again: afresh from phase 0 (rule 5)"));
                self.phase = 0.0;
                self.fresh = true;
                self.wanted = None;
            } else {
                notes.push(format!("{wanted} again, can't be cut short: kept until it ends"));
                self.wanted = Some(wanted);
            }
            return notes;
        }
        self.ask(wanted, loops, &mut notes);
        notes
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
    /// (walking, running, stepping back, turning on the spot). It follows
    /// the keys' choice, not the movement playing: while a walk or run
    /// plays on to its stop point, and while the stop animation plays, he
    /// is already in Wait (connor-movement.md open question 2, measured),
    /// so a wall there is never head-on.
    pub fn moving(&self) -> bool {
        match self.mode {
            Mode::Free => self.choice != "stop",
            // In the talk state: the keys chose a movement since he entered
            // it (rule 13).
            Mode::Talk => self.talk_play.is_none() && !matches!(self.choice, "stop" | ""),
            _ => false,
        }
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
        if matches!(from, "run" | "stoprun" | "waterrun" | "runwaterrun") {
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
        self.fresh = true;
        Some((from, "stop"))
    }

    /// `ropeandhook.md` rules 22–23 (M6 6a-4): one update of the climb.
    /// Forward plays it forwards (up), back backwards (down), any other key
    /// or none leaves it frozen (the user's check: only forward and back).
    /// His position never changes. It ends when it reaches the end it is
    /// playing toward (`step.ended`, `climb_dir` tells which).
    fn climb_update<'a>(&mut self, intent: Intent, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>, step: &mut Step) {
        self.climb_dir = intent.forward.signum();
        let time = loops(self.movement).map_or(0.0, |l| l.play_time() / l.speed.abs().max(1e-6));
        if self.climb_dir != 0 && time > 0.0 {
            let share = (1.0 / UPDATES_PER_SECOND) as f32 / time;
            self.phase = (self.phase + share * f32::from(self.climb_dir)).clamp(0.0, 1.0);
            if (self.climb_dir > 0 && self.phase > 1.0 - 1e-4) || (self.climb_dir < 0 && self.phase < 1e-4) {
                self.phase = if self.climb_dir > 0 { 1.0 } else { 0.0 };
                step.ended = Some(self.movement);
            }
        }
        step.played_to = self.phase;
    }

    /// One update (1/15 s). `loops` gives a movement's loop by name.
    pub fn update<'a>(&mut self, intent: Intent, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>) -> Step {
        let dt = (1.0 / UPDATES_PER_SECOND) as f32;
        let mut mode_changed = None;
        let mut notes = Vec::new();
        let start = self.movement;
        // pushing.md rule 7: the counter, every update in Push.
        if self.mode == Mode::Push {
            let key = intent.forward != 0 || intent.turn != 0;
            if self.push_counter < PUSH_UPDATES && key {
                self.push_counter += 1;
            } else {
                let to = if key { "Move" } else { "Wait" };
                notes.push(format!("push: {} updates without a pushing touch{}: {to}", self.push_counter, if key { "" } else { " or no key" }));
                self.mode = Mode::Free;
                mode_changed = Some((Mode::Push, Mode::Free));
            }
        }
        // Rule 16: the Special movement playing has ended: the next item.
        if self.mode == Mode::Special && self.phase >= 1.0 {
            self.special_at += 1;
            self.special_items(&mut notes);
            if self.mode != Mode::Special {
                mode_changed = Some((Mode::Special, self.mode));
            }
        }
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
        if let Some(m) = self.mode_movement() {
            if m != self.movement {
                self.movement = m;
                self.phase = 0.0;
                self.fresh = true;
            }
        } else {
            // While a wall block is on he stands, whatever the keys ask.
            // Pushing: `push` instead of the keys' choice (rule 6).
            let mut choice = if self.mode == Mode::Push {
                "push"
            } else if self.blocked() && !self.auto.as_ref().is_some_and(|a| matches!(a.phase, crate::connor_auto::AutoPhase::Walking { .. })) {
                // A scripted walk after its turn ignores the blocks (rule
                // 24, Q94 b: Move's own updates don't look at them).
                "stop"
            } else {
                choose_movement(intent, self.submerged)
            };
            // Just landed from a running jump (`choice` is ""): a turn key
            // alone counts as standing, so `jumpstop` plays (he turns
            // during it) and the turn on the spot is asked for once it has
            // started. The user's check in the original; not traced (Q41 d).
            if self.choice.is_empty() && matches!(choice, "left" | "right") {
                choice = "stop";
            }
            // Rule 3: the lean switch, set only in the keys' updates.
            self.lean_allowed = matches!(choice, "run" | "waterrun");
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
            ended: None,
        };
        let finish = |mut step: Step, walker: &Walker, notes: Vec<String>| {
            step.changed = (walker.movement != start).then_some((start, walker.movement));
            step.notes = notes;
            step
        };
        // The user's check in the original (2026-10-13): he doesn't stay
        // leaned through a jump, turn key held or not. The note says the
        // switch is kept in jumps (rule 3); asked as Q59. Stand-in: off
        // while jumping, falling and landing, so he straightens up.
        if matches!(self.mode, Mode::Jump | Mode::Fall | Mode::Land) {
            self.lean_allowed = false;
        }
        self.update_bank(intent, dt);
        if self.mode == Mode::Dead {
            self.phase = 1.0;
            return finish(step, self, notes);
        }
        // ropeandhook.md rule 23: hanging, his position doesn't change;
        // the climb is moved by the keys (rule 22, `climb_update`).
        if self.mode == Mode::Rope && self.rope_hang {
            self.climb_update(intent, loops, &mut step);
            return finish(step, self, notes);
        }
        // The keys turn him only when they choose the movement; that
        // includes the start and stop transitions. Also while `crash`
        // plays (connor-physics.md "Jumping" rule 5, "Walls"; Q84 b,
        // confirmed by the user): at the normal rate, from the run crash
        // and the jump crashes alike; forward, back and run wait for its
        // end. Never from a jump's start to its landing's end (Q41).
        // The talk state turns him whatever plays, its approach movements
        // too (conversation.md rule 13, Q125 a).
        let turns = ((self.keys_choose() && !self.blocked()) || matches!(self.mode, Mode::Crash | Mode::Talk)) && !self.turn_by_script;
        if turns {
            self.heading += f32::from(intent.turn) * TURN_RATE * dt;
        }
        let Some(l) = loops(played) else {
            step.missing = true;
            return finish(step, self, notes);
        };
        // anim-scripts.md rule 5 (Q138 e): a movement starts at its `loop=`
        // start (`undraw` at frame 7 of 8, played backwards).
        if std::mem::take(&mut self.fresh) && self.phase == 0.0 && l.start_phase() != 0.0 {
            self.phase = l.start_phase();
            step.played_from = self.phase;
            step.played_to = self.phase;
            notes.push(format!("{played} starts at its loop= start {:.4} (our phase {:.4}; anim-scripts.md rule 5)", l.start, self.phase));
        }
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
            if from < 1.0 && self.phase >= 1.0 {
                step.ended = Some(played);
            }
            let own = l.one_shot_step(from, self.phase).map(|v| v * l.speed.signum());
            if self.phase >= 1.0 && self.keys_choose() {
                match self.playing {
                    Some(t) => self.end_transition(t, loops, &mut notes),
                    // Rule 6, last case: a one-shot ending with no
                    // transition in progress.
                    None => {
                        if let Some(w) = self.wanted.take() {
                            notes.push(format!("{played} ended: {w} from phase 0"));
                            self.movement = w;
                            self.phase = 0.0;
                            self.fresh = true;
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

    #[test]
    fn his_frames_are_cut_down_as_every_object_s() {
        // objects.md "Loops" rule 6 (Q153): floor(p × N + 0.001).
        let once = MovementLoop { duration: 0.5, speed: 1.0, repeat: false, frames: 8, ..Default::default() };
        assert_eq!((once.frame_at(0.0), once.frame_at(0.5), once.frame_at(0.99), once.frame_at(1.0)), (0, 4, 7, 7));
        // The old round(p × 7) gave frame 1 here.
        assert_eq!(once.frame_at(0.12), 0, "0.96 cut down");
        let back = MovementLoop { speed: -1.0, ..once.clone() };
        assert_eq!((back.frame_at(0.0), back.frame_at(1.0)), (7, 0));
        let walk = MovementLoop { duration: 1.2667, speed: 1.0, repeat: true, frames: 20, ..Default::default() };
        assert_eq!((walk.frame_at(0.999), walk.frame_at(1.0), walk.frame_at(0.04999)), (19, 0, 1));
    }

    fn with_cues(repeat: bool, speed: f32, cues: &[f32]) -> MovementLoop {
        MovementLoop { repeat, speed, cues: cues.iter().map(|&c| (c, format!("at {c}"))).collect(), ..Default::default() }
    }

    #[test]
    fn cue_ranges_follow_rule_9_and_the_wrap() {
        // Walk: footsteps at 0.1 and 0.6 (Conner.anm).
        let walk = with_cues(true, 1.0, &[0.6, 0.1]);
        assert_eq!(walk.cues_between(0.05, 0.15), ["at 0.1"]);
        assert!(walk.cues_between(0.1001, 0.5).is_empty());
        assert_eq!(walk.cues_between(0.1, 0.2), ["at 0.1"], "the lower end counts");
        assert!(walk.cues_between(0.0, 0.1).is_empty(), "the higher end doesn't");
        assert_eq!(walk.cues_between(0.55, 0.15), ["at 0.6", "at 0.1"], "wrap: both");
        // A cue at 1.0 fires at a forward wrap, one at 0.0 after it.
        let ends = with_cues(true, 1.0, &[1.0, 0.0]);
        assert_eq!(ends.cues_between(0.95, 0.02), ["at 1", "at 0"]);
        // A play-once movement ending at exactly 1.0 excludes it (rule 4).
        let once = with_cues(false, 1.0, &[0.0, 1.0]);
        assert_eq!(once.cues_between(0.0, 0.1), ["at 0"]);
        assert!(once.cues_between(0.9, 1.0).is_empty());
        // Back (c_walk at speed -1): phase falls; wrap mirrors.
        let back = with_cues(true, -1.0, &[0.6, 0.1]);
        assert_eq!(back.cues_between(0.65, 0.55), ["at 0.6"]);
        assert_eq!(back.cues_between(0.12, 0.08), ["at 0.1"]);
        assert_eq!(back.cues_between(0.03, 0.95), Vec::<&str>::new());
        // A (large) backward wrap from 0.15 to 0.58 passes both.
        assert_eq!(back.cues_between(0.15, 0.58), ["at 0.6", "at 0.1"]);
    }

    /// A lookup over a list of named loops, as the viewer gives it.
    fn lookup<'a>(list: &'a [(&'static str, MovementLoop)]) -> impl Fn(&str) -> Option<&'a MovementLoop> + 'a {
        move |m| list.iter().find(|(n, _)| *n == m).map(|(_, l)| l)
    }

    fn intent(forward: i8, turn: i8, run: bool) -> Intent {
        Intent { forward, turn, run }
    }

    #[test]
    fn the_throw_state_plays_its_movements_and_ignores_the_keys() {
        // azriel-hammer.md rule 3 (M6 6b-3): `attack` once (19 frames),
        // `wait` repeating; held keys neither move nor turn him.
        let list = [
            ("attack", MovementLoop { duration: 19.0 / 15.0, speed: 1.0, frames: 19, ..Default::default() }),
            ("wait", MovementLoop { duration: 29.0 / 15.0, speed: 1.0, frames: 29, repeat: true, ..Default::default() }),
        ];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.throw_play("attack");
        assert_eq!(w.mode, Mode::Throw);
        let mut ended = None;
        for n in 1..=19 {
            let s = w.update(intent(1, 1, true), &loops);
            assert_eq!((s.played, w.heading), ("attack", 0.0), "update {n}");
            if s.ended.is_some() {
                ended = Some(n);
            }
        }
        assert_eq!(ended, Some(19), "attack ends after its 19 frames");
        // Held at its end until the caller moves on.
        w.update(intent(0, 0, false), &loops);
        assert_eq!((w.movement, w.phase, w.mode), ("attack", 1.0, Mode::Throw));
        w.throw_play("wait");
        for _ in 0..40 {
            let s = w.update(intent(1, 0, false), &loops);
            assert!(s.ended.is_none() && s.played == "wait");
        }
        w.throw_leave();
        assert_eq!((w.mode, w.movement), (Mode::Free, "stop"));
    }

    #[test]
    fn a_movement_starts_at_its_loop_start() {
        // anim-scripts.md rule 5 (Q138 e): confists.anm `undraw` (cf_draw
        // 0.888889 −1.0, 8 frames) starts at frame 7 and plays backwards to
        // frame 0 in 0.889 of its length: here 8/15 s, so from our phase
        // 0.111 at 0.125 an update it ends in the 8th (from 0: also 8).
        let list = [("undraw", MovementLoop { duration: 8.0 / 15.0, speed: -1.0, start: 0.888889, frames: 8, ..Default::default() })];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.start_special(vec![SpecialStep::Play("undraw", "undraw".into())]);
        let l = loops("undraw").unwrap();
        assert_eq!(l.frame_at(l.start_phase()), 7);
        let mut ended = None;
        for n in 1..=10 {
            let s = w.update(intent(0, 0, false), &loops);
            if n == 1 {
                assert!((s.played_from - (1.0 - 0.888889)).abs() < 1e-5, "{}", s.played_from);
            }
            if s.ended.is_some() && ended.is_none() {
                ended = Some(n);
            }
        }
        assert_eq!(ended, Some(8), "0.111 + 8 × 0.125 ≥ 1");
    }

    #[test]
    fn the_climb_moves_only_with_forward_and_back() {
        // ropeandhook.md rules 22-23: 78 frames in 5.2 s (78 updates).
        let list = [("climb", MovementLoop { duration: 5.2, speed: 1.0, frames: 78, ..Default::default() })];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.rope_hang_at("climb", 0.0);
        let s = w.update(intent(0, 1, false), &loops);
        assert_eq!((w.phase, w.climb_dir, w.heading), (0.0, 0, 0.0), "a turn key: frozen, no turn");
        assert!(s.ended.is_none());
        w.update(intent(1, 0, false), &loops);
        assert!((w.phase - 1.0 / 78.0).abs() < 1e-5 && w.climb_dir == 1, "forward: up, {}", w.phase);
        let s = w.update(intent(-1, 0, false), &loops);
        assert_eq!(w.phase, 0.0);
        assert_eq!(s.ended, Some("climb"), "back to the foot: the climb ends there");
        assert_eq!(w.mode, Mode::Rope);
        w.rope_hang_at("climb", 1.0);
        assert_eq!(w.update(intent(1, 0, false), &loops).ended, Some("climb"), "forward at the top: ends at once");
    }

    #[test]
    fn gestures_play_only_while_he_stands() {
        // conversation.md rule 18.
        let mut w = Walker::new(0.0);
        assert!(w.gesture(10, Some("gesturec")).contains("plays 'gesturec'"));
        assert_eq!((w.movement, w.phase), ("gesturec", 0.0));
        assert!(w.gesture(16, Some("stop")).contains("plays 'stop'"));
        assert!(w.gesture(9, None).contains("not in his movement list"));
        assert!(w.gesture(7, Some("gesturea")).contains("not a gesture code"));
        // In the talk state, standing: yes; during the approach: no.
        w.enter_talk();
        w.choice = "stop";
        assert!(w.gesture(8, Some("gesturea")).contains("plays"));
        w.talk_play = Some("stopwalk");
        assert!(w.gesture(8, Some("gesturea")).contains("not played"));
        // Q125 a: during the approach a turn key turns him, forward does
        // nothing.
        let list = [("stopwalk", MovementLoop { duration: 1.0, speed: 1.0, frames: 15, ..Default::default() })];
        let (heading, movement) = (w.heading, w.movement);
        w.movement = "stopwalk";
        w.update(intent(1, 1, false), &lookup(&list));
        assert!(w.heading > heading, "turned left: {}", w.heading);
        assert_eq!((w.movement, w.talk_play), ("stopwalk", Some("stopwalk")), "was {movement}");
        // Walking: no.
        let mut w = Walker::new(0.0);
        w.choice = "walk";
        assert!(w.gesture(8, Some("gesturea")).contains("moving"));
        let mut w = Walker::new(0.0);
        w.mode = Mode::Special;
        assert!(w.gesture(8, Some("gesturea")).contains("not played"));
    }

    #[test]
    fn a_second_turn_request_only_replaces_the_target() {
        // clicking.md rule 32: one automatic turn; a pointAndCue keeps its
        // cue (anim-scripts.md "Edge cases").
        use crate::connor_auto::{AutoKind, AutoMove, Cue};
        let mut w = Walker::new(0.0);
        let cue = Cue { who: "x".into(), c: "1".into(), d: "0".into() };
        w.start_auto(AutoMove::new(AutoKind::Point, [10.0, 0.0], Some(cue.clone())));
        w.auto.as_mut().unwrap().turn_updates = 3;
        let line = w.start_auto(AutoMove::new(AutoKind::Point, [0.0, 10.0], None));
        assert!(line.contains("one automatic turn") && line.contains("cue is kept"), "{line}");
        let a = w.auto.as_ref().unwrap();
        assert_eq!((a.target, a.turn_updates, a.cue.as_ref()), ([0.0, 10.0], 3, Some(&cue)));
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
            w.choice = movement;
            w.phase = 0.5;
            assert_eq!(w.head_on_at_wall(), change, "{movement}");
            assert_eq!(w.mode, mode, "{movement}");
        }
        let mut w = Walker::new(0.0);
        w.movement = "walk";
        w.choice = "walk";
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
        // Straight back from his facing, which the held turn key keeps
        // turning at 1.75 rad/s while `crash` plays (Q84 b).
        let heading = w.heading;
        let mut back = 0.0;
        for n in 1..=7 {
            let s = w.update(held, &loops);
            assert_eq!((w.movement, w.mode), ("crash", Mode::Crash), "update {n}");
            assert!((w.heading - heading - n as f32 * TURN_RATE / 15.0).abs() < 1e-5, "update {n}: heading {}", w.heading);
            let behind = [-w.heading.sin(), w.heading.cos()];
            let along = s.wanted[0] * behind[0] + s.wanted[1] * behind[1];
            assert!((along - 63.07).abs() < 0.01 && s.wanted[0].hypot(s.wanted[1]) - along < 1e-3, "update {n}: {s:?}");
            assert_eq!(crash().frame_at(w.phase), n, "update {n}: frame");
            back += along;
        }
        assert!((back - 441.5).abs() < 0.01, "{back}");
        // Forward and run waited; then running again.
        let behind = [-w.heading.sin(), w.heading.cos()];
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
            // Q112 (the user's check, traced): backed against the wall,
            // back held, Jump gives the back flip; forward: the standing
            // jump (running off).
            assert_eq!(w.jump_kind(intent(forward, 0, false)), Ok(if forward > 0 { JumpKind::Stand } else { JumpKind::Back }));
            // Forward held at the wall with running on: `runJump` while
            // standing (inferred in the note).
            if forward > 0 {
                assert_eq!(w.jump_kind(intent(1, 0, true)), Ok(JumpKind::Run));
            }
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
            // Back let go: standing still, the standing jump again.
            assert_eq!(w.jump_kind(intent(0, 0, false)), Ok(JumpKind::Stand));
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
        // Wading jumps are standing jumps (user's check in the original),
        // back in water the back flip (Q112: no water form of back).
        for (m, run) in [("waterwalk", false), ("waterrun", true)] {
            let w = Walker { movement: m, submerged: true, ..Walker::new(0.0) };
            assert_eq!(w.jump_kind(intent(1, 0, run)), Ok(JumpKind::Stand), "{m}");
            assert_eq!(w.jump_kind(intent(-1, 0, run)), Ok(JumpKind::Back), "{m}");
        }
        // Q112: the held keys, not the movement playing: back held while a
        // walk still plays gives the back flip.
        let w = Walker { movement: "walk", ..Walker::new(0.0) };
        assert_eq!(w.jump_kind(intent(-1, 0, false)), Ok(JumpKind::Back));
        assert_eq!(choose_movement(intent(1, 0, true), false), "run");
        assert_eq!(choose_movement(intent(1, 1, false), false), "walk");
        assert_eq!(choose_movement(intent(-1, -1, true), false), "back");
        assert_eq!(choose_movement(intent(0, 1, true), false), "left");
        assert_eq!(choose_movement(intent(0, -1, false), false), "right");
        assert_eq!(choose_movement(intent(0, 0, true), false), "stop");
        // In water only forward changes (water.md "Wading animations").
        assert_eq!(choose_movement(intent(1, 0, true), true), "waterrun");
        assert_eq!(choose_movement(intent(1, 1, false), true), "waterwalk");
        assert_eq!(choose_movement(intent(-1, 0, true), true), "back");
        assert_eq!(choose_movement(intent(0, 1, false), true), "left");
        assert_eq!(choose_movement(intent(0, 0, false), true), "stop");
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
    fn a_script_request_plays_once_and_holds_while_the_keys_dont_change() {
        // anim-scripts.md rules 4-5, connor-movement.md rules 1, 4: the
        // `portal` of `sdwtrap1.cs`, with no key held (handsOff).
        let mut list = stop_list();
        list.push(("portal", once()));
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(0, 0, false), &loops);
        assert_eq!(w.movement, "stop");
        w.script_request("portal", &loops);
        assert_eq!((w.movement, w.phase), ("portal", 0.0), "stop can be cut short: at once");
        let mut ended = Vec::new();
        for _ in 0..20 {
            ended.extend(w.update(intent(0, 0, false), &loops).ended);
        }
        assert_eq!((w.movement, w.phase, ended), ("portal", 1.0, vec!["portal"]), "ends once, holds its last frame");
        // A new request for it starts it afresh (rule 5).
        w.script_request("portal", &loops);
        assert_eq!(w.phase, 0.0);
        // Outside the keys' states: not done.
        w.set_mode(Mode::Fall);
        assert!(w.script_request("stop", &loops)[0].contains("not done"));
        // A key pressed: the keys ask again.
        w.set_mode(Mode::Free);
        w.update(intent(1, 0, false), &loops);
        assert_eq!(w.movement, "walk");
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
            start: 0.0,
            repeat: false,
            frames: steps.len() + 1,
            frame_steps,
            cut_short: false,
            transitions: Vec::new(),
            cues: Vec::new(),
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
    fn a_backwards_one_shot_shows_its_last_frame_first() {
        let l = MovementLoop { speed: -1.0, repeat: false, frames: 9, ..Default::default() };
        assert_eq!((l.frame_at(0.0), l.frame_at(0.5), l.frame_at(1.0)), (8, 4, 0));
        let f = MovementLoop { speed: 1.0, ..l };
        assert_eq!((f.frame_at(0.0), f.frame_at(1.0)), (0, 8));
    }

    #[test]
    fn one_shot_steps_frame_by_frame() {
        let a = transition_anim(&[(1.0, 0.0), (0.0, 10.0)]);
        // Half of frame 1 and a quarter of frame 2.
        let s = a.one_shot_step(0.25, 0.625);
        assert!((s[0] - 0.5).abs() < 1e-6 && (s[1] + 2.5).abs() < 1e-6, "{s:?}");
    }

    // The stop transitions (M2 2g-2), with `Conner.anm`'s from points and
    // frame counts. The stop animations' steps don't matter here (zero).

    /// `start_list` plus walk's and run's stop transitions and the six stop
    /// animations.
    fn full_list() -> Vec<(&'static str, MovementLoop)> {
        let to_stop = |from, via| Transition { from, to: "stop", to_phase: 0.0, via };
        let mut list = start_list();
        for (name, l) in list.iter_mut() {
            match *name {
                "walk" => {
                    l.transitions = vec![to_stop(0.9, "stopwalkstop"), to_stop(0.95, "walkstopright"), to_stop(0.45, "walkstopleft")]
                }
                "run" => {
                    l.transitions =
                        vec![to_stop(0.066667, "runstopright"), to_stop(0.866667, "stoprunstop"), to_stop(0.6, "runstopleft")]
                }
                _ => {}
            }
        }
        for (name, frames) in [
            ("walkstopleft", 6),
            ("stopwalkstop", 9),
            ("walkstopright", 7),
            ("runstopright", 12),
            ("runstopleft", 16),
            ("stoprunstop", 10),
        ] {
            list.push((name, transition_anim(&vec![(0.0, 0.0); frames - 1])));
        }
        list
    }

    /// Walks or runs from standing until the loop's phase is in [release,
    /// before), lets go, and returns the next 30 updates.
    fn stop_from(run: bool, release: f32, before: f32) -> (Vec<Step>, Walker) {
        let list = full_list();
        let loops = lookup(&list);
        let main = if run { "run" } else { "walk" };
        let mut w = Walker::new(0.0);
        for _ in 0..200 {
            w.update(intent(1, 0, run), &loops);
            if w.movement == main && w.phase >= release && w.phase < before {
                let steps = (0..30).map(|_| w.update(intent(0, 0, run), &loops)).collect();
                return (steps, w);
            }
        }
        panic!("{main} never reached phase {release}");
    }

    /// The movements played, as (name, updates) runs.
    fn runs(steps: &[Step]) -> Vec<(&'static str, usize)> {
        let mut out: Vec<(&'static str, usize)> = Vec::new();
        for s in steps {
            match out.last_mut() {
                Some((name, n)) if *name == s.played => *n += 1,
                _ => out.push((s.played, 1)),
            }
        }
        out
    }

    #[test]
    fn the_stop_is_picked_by_the_next_from_point_after_letting_go() {
        for (run, release, before, from, via, frames) in [
            (false, 0.3, 0.45, 0.45, "walkstopleft", 6),
            (false, 0.76, 0.9, 0.9, "stopwalkstop", 9),
            (false, 0.9, 0.95, 0.95, "walkstopright", 7),
            (true, 0.5, 0.6, 0.6, "runstopleft", 16),
            (true, 0.7, 0.866, 0.866667, "stoprunstop", 10),
            (true, 0.9, 1.0, 0.066667, "runstopright", 12),
        ] {
            let (steps, w) = stop_from(run, release, before);
            let r = runs(&steps);
            let main = if run { "run" } else { "walk" };
            assert_eq!((r[0].0, r[1], r[2].0), (main, (via, frames - 1), "stop"), "release at {release}: {r:?}");
            // The loop played exactly up to the from point.
            let last = &steps[r[0].1 - 1];
            assert!((last.played_to - from).abs() < 1e-5, "release at {release}: {last:?}");
            assert_eq!((w.movement, w.wanted, w.waiting, w.playing), ("stop", None, None, None));
        }
    }

    #[test]
    fn letting_go_during_stopwalk_walks_to_0_9_then_stopwalkstop() {
        let list = full_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &loops);
        w.update(intent(1, 0, false), &loops);
        let steps: Vec<_> = (0..12).map(|_| w.update(intent(0, 0, false), &loops)).collect();
        assert_eq!(runs(&steps), [("stopwalk", 6), ("walk", 1), ("stopwalkstop", 5)]);
        let walk = &steps[6];
        assert_eq!(walk.played_from, 0.85);
        assert!((walk.played_to - 0.9).abs() < 1e-5 && walk.own_step[1] < 0.0, "{walk:?}");
    }

    #[test]
    fn letting_go_during_stoprun_gives_one_update_of_run_then_stoprunstop() {
        let list = full_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, true), &loops);
        w.update(intent(1, 0, true), &loops);
        let steps: Vec<_> = (0..20).map(|_| w.update(intent(0, 0, true), &loops)).collect();
        assert_eq!(runs(&steps), [("stoprun", 9), ("run", 1), ("stoprunstop", 9), ("stop", 1)]);
        let run = &steps[9];
        assert_eq!((run.played_from, run.played_to, run.own_step), (0.866667, 0.866667, [0.0; 3]));
    }

    #[test]
    fn pressing_forward_again_while_a_stop_waits_walks_on() {
        let list = full_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        for _ in 0..12 {
            w.update(intent(1, 0, false), &loops);
        }
        // Walk at about 0.1: let go (walkstopleft waits for 0.45), then
        // press forward again (Q40 b, our reading).
        w.update(intent(0, 0, false), &loops);
        assert_eq!(w.waiting.map(|t| t.via), Some("walkstopleft"));
        let steps: Vec<_> = (0..30).map(|_| w.update(intent(1, 0, false), &loops)).collect();
        assert_eq!(runs(&steps), [("walk", 30)]);
        assert_eq!((w.wanted, w.waiting), (None, None));
    }

    #[test]
    fn forward_pressed_during_a_stop_animation_waits_for_its_end_then_starts_again() {
        let (_, mut w) = stop_from(false, 0.3, 0.45);
        // stop_from already played the stop to its end; walk and let go again.
        let list = full_list();
        let loops = lookup(&list);
        let mut steps = Vec::new();
        for _ in 0..40 {
            let s = w.update(intent(1, 0, false), &loops);
            let walking = s.played == "walk";
            steps.push(s);
            if walking {
                break;
            }
        }
        assert_eq!(runs(&steps), [("stop", 1), ("stopwalk", 7), ("walk", 1)]);
        let (_, mut w) = {
            let list = full_list();
            let loops = lookup(&list);
            let mut w = Walker::new(0.0);
            let mut played = Vec::new();
            for _ in 0..200 {
                w.update(intent(1, 0, false), &loops);
                if w.movement == "walk" && w.phase > 0.3 && w.phase < 0.35 {
                    break;
                }
            }
            while w.movement != "walkstopleft" {
                played.push(w.update(intent(0, 0, false), &loops));
            }
            (played, w)
        };
        let steps: Vec<_> = (0..14).map(|_| w.update(intent(1, 0, false), &loops)).collect();
        assert_eq!(runs(&steps), [("walkstopleft", 5), ("stop", 1), ("stopwalk", 7), ("walk", 1)]);
    }

    #[test]
    fn no_head_on_while_stopping() {
        let list = full_list();
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        for _ in 0..12 {
            w.update(intent(1, 0, true), &loops);
        }
        assert!(w.moving());
        // Let go: run plays on to its stop point, already in Wait.
        w.update(intent(0, 0, true), &loops);
        assert_eq!(w.movement, "run");
        assert!(!w.moving());
        assert_eq!(w.head_on_at_wall(), None);
        assert_eq!((w.mode, w.blocked_name()), (Mode::Free, "none"));
    }

    /// Runs `n` updates with `held`; returns the lean after each.
    fn leans<'a>(w: &mut Walker, held: Intent, n: usize, loops: &dyn Fn(&str) -> Option<&'a MovementLoop>) -> Vec<f32> {
        (0..n).map(|_| { w.update(held, loops); w.bank }).collect()
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-5
    }

    #[test]
    fn running_and_turning_leans_in_0_03_steps_up_to_0_18() {
        let list = [("run", run())];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        // Turning left: leans to his left (positive), 0.03 per update, a
        // step taken while still short of 0.15, so it ends at 0.18.
        let l = leans(&mut w, intent(1, 1, true), 9, &loops);
        for (k, b) in l.iter().enumerate() {
            assert!(near(*b, (0.03 * (k + 1) as f32).min(0.18)), "update {}: {b}", k + 1);
        }
        // The other way: straight through upright to −0.18 (also on this
        // side, "Do not reproduce"), 12 updates.
        let l = leans(&mut w, intent(1, -1, true), 13, &loops);
        assert!(near(l[5], 0.0) && near(l[6], -0.03), "{l:?}");
        assert!(near(l[11], -0.18) && near(l[12], -0.18), "{l:?}");
        // Turn key let go, still running: back upright in 6 updates.
        let l = leans(&mut w, intent(1, 0, true), 6, &loops);
        assert!(near(l[4], -0.03) && l[5] == 0.0, "{l:?}");
    }

    #[test]
    fn returning_snaps_upright_within_0_029_before_stepping() {
        let list = [("run", run())];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        // 0.04: one step to 0.01, then the snap in the next update.
        w.bank = 0.04;
        let l = leans(&mut w, intent(1, 0, true), 2, &loops);
        assert!(near(l[0], 0.01) && l[1] == 0.0, "{l:?}");
        // Building, small values are kept (the snap is only for returning).
        w.bank = -0.01;
        assert!(near(leans(&mut w, intent(1, 1, true), 1, &loops)[0], 0.02));
    }

    #[test]
    fn walking_and_turning_on_the_spot_never_lean() {
        let list = [("walk", run()), ("waterwalk", run()), ("left", run())];
        let loops = lookup(&list);
        for (held, submerged) in [(intent(1, 1, false), false), (intent(1, 1, false), true), (intent(0, 1, true), false)] {
            let mut w = Walker::new(0.0);
            w.submerged = submerged;
            let l = leans(&mut w, held, 10, &loops);
            assert!(l.iter().all(|b| *b == 0.0), "{held:?} submerged {submerged}: {l:?}");
        }
    }

    #[test]
    fn running_through_water_leans_too() {
        let list = [("waterrun", run())];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.submerged = true;
        let l = leans(&mut w, intent(1, -1, true), 7, &loops);
        assert_eq!(w.movement, "waterrun");
        assert!(near(l[0], -0.03) && near(l[6], -0.18), "{l:?}");
    }

    #[test]
    fn a_jump_straightens_him_up_even_with_the_turn_key_held() {
        let list = [("run", run())];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        leans(&mut w, intent(1, 1, true), 6, &loops);
        assert!(near(w.bank, 0.18) && w.lean_allowed);
        w.start_jump(JumpKind::Run);
        // Forward and turn still held: back to upright in 0.03 steps (the
        // user's check in the original, Q59).
        let l = leans(&mut w, intent(1, 1, true), 6, &loops);
        assert!(near(l[0], 0.15) && near(l[4], 0.03) && l[5] == 0.0, "{l:?}");
        assert!(!w.lean_allowed && w.mode == Mode::Jump);
        // Falling after it: still upright.
        w.set_mode(Mode::Fall);
        assert_eq!(leans(&mut w, intent(1, 1, true), 3, &loops), [0.0; 3]);
    }

    #[test]
    fn the_running_start_leans_and_the_stops_dont() {
        let mut start = run();
        start.repeat = false;
        let mut r = run();
        r.transitions = vec![Transition { from: -1.0, to: "stop", to_phase: 0.0, via: "stoprunstop" }];
        let mut s = run();
        s.transitions = vec![Transition { from: -1.0, to: "run", to_phase: 0.0, via: "stoprun" }];
        let list = [("stop", s), ("stoprun", start.clone()), ("run", r), ("stoprunstop", start)];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        // The keys pick `run` at once, so the start (`stoprun`) leans.
        let l = leans(&mut w, intent(1, 1, true), 2, &loops);
        assert_eq!(w.movement, "stoprun");
        assert!(near(l[1], 0.06), "{l:?}");
        // Forward let go, still turning: the pick is `left`, switch off,
        // back towards upright although `run` or its stop still plays.
        let l = leans(&mut w, intent(0, 1, true), 2, &loops);
        assert!(near(l[0], 0.03) && l[1] == 0.0, "{l:?}");
    }

    #[test]
    fn a_special_list_is_worked_out_left_to_right() {
        let resolve = |list: Option<&str>, w: &str| -> Option<&'static str> {
            match (list, w.to_ascii_lowercase().as_str()) {
                (Some(l), "push") if l.eq_ignore_ascii_case("bricks.anm") => Some("bricks.anm/push"),
                (None, "kneel") => Some("kneel"),
                _ => None,
            }
        };
        let steps = special_steps("AnimList=bricks.anm,AnimListOn,PUSH,AnimListOff,kneel,Inv,SmCrossBow,push,KillConnor", &resolve);
        let plays: Vec<&str> = steps.iter().filter_map(|s| if let SpecialStep::Play(n, _) = s { Some(*n) } else { None }).collect();
        assert_eq!(plays, ["bricks.anm/push", "kneel"]);
        assert!(steps.iter().any(|s| matches!(s, SpecialStep::Note(n) if n.starts_with("Inv SmCrossBow"))));
        // `push` after AnimListOff is looked up in his own list: not there.
        assert!(steps.iter().any(|s| matches!(s, SpecialStep::Note(n) if n.starts_with("movement push not loaded from his own list"))));
        assert_eq!(steps.last(), Some(&SpecialStep::Kill));
        assert_eq!(plain_movement_name("bricks.anm/push"), "push");
        assert!(std::ptr::eq(intern("bricks.anm/push"), intern("bricks.anm/push")));
    }

    #[test]
    fn special_plays_each_movement_to_its_end_then_stands() {
        let list = [("stop", MovementLoop { step_sum: [0.0; 3], ..walk(1.0) }), ("walk", walk(1.0)), ("a", once()), ("b", once())];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &loops);
        let notes = w.start_special(vec![SpecialStep::Play("a", "a".into()), SpecialStep::Note("x".into()), SpecialStep::Play("b", "b".into())]);
        assert_eq!((w.mode, w.movement, w.phase), (Mode::Special, "a", 0.0), "{notes:?}");
        let mut ended = Vec::new();
        let heading = w.heading;
        for _ in 0..40 {
            let s = w.update(intent(1, 1, true), &loops);
            ended.extend(s.ended);
            if w.mode == Mode::Free {
                break;
            }
            // Keys ignored (no turning) while the list plays.
            assert_eq!(w.heading, heading);
        }
        assert_eq!(ended, ["a", "b"]);
        // Back to the keys (forward and run held: they ask for `run`).
        assert_eq!((w.mode, w.movement), (Mode::Free, "run"));
    }

    #[test]
    fn the_push_state_asks_for_push_and_ends_after_4_updates_without_a_touch() {
        let push = MovementLoop { duration: 29.0 / 15.0, step_sum: [0.0, -1150.0, 0.0], speed: 1.0, repeat: true, frames: 30, cut_short: true, ..Default::default() };
        let list = [("stop", MovementLoop { step_sum: [0.0; 3], ..walk(1.0) }), ("walk", walk(1.0)), ("push", push)];
        let loops = lookup(&list);
        let mut w = Walker::new(0.0);
        w.update(intent(1, 0, false), &loops);
        assert!(w.push_ability().movement_ok && w.push_ability().accepts);
        assert_eq!(w.push_touch(), Some("enters the Push state"));
        w.update(intent(1, 0, false), &loops);
        assert_eq!((w.mode, w.movement), (Mode::Push, "push"));
        // Touched every update: stays.
        for _ in 0..10 {
            w.push_touch();
            w.update(intent(1, 0, false), &loops);
            assert_eq!(w.mode, Mode::Push);
        }
        // No touch: 4 more updates of Push, then Move (key held).
        let mut n = 0;
        while w.mode == Mode::Push {
            w.update(intent(1, 0, false), &loops);
            n += 1;
        }
        // (The update right after the last touch already counted one.)
        assert_eq!((n, w.movement), (4, "walk"));
        // Let go while pushing: Wait at once.
        w.push_touch();
        w.update(intent(1, 0, false), &loops);
        w.update(intent(0, 0, false), &loops);
        assert_eq!((w.mode, w.movement), (Mode::Free, "stop"));
        // Backing up isn't a pushing movement.
        w.update(intent(-1, 0, false), &loops);
        assert!(!w.push_ability().movement_ok);
    }
}
