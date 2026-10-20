//! Characters' states (M4 step 4d-3): what a `KQMonster` or
//! `KQGeometryTalker` does on top of the movement player every animated
//! object has (`crate::named_anim`).
//!
//! Rules from `docs/behaviour/characters.md` (session A, 2026-10-19):
//!
//! - Rule 11: a character updates only while near or far (not off, not
//!   hidden); the caller says which ([`Character::in_range`]).
//! - Rule 12 (corrected 2026-10-19, Q116, Q121; M4 4d-9): the start-up
//!   runs once, at the first update or at the first movement request,
//!   whatever the range ([`Character::start_up`]). The current movement
//!   is the one set at load (both names start as `Stop`, M4 4d-8), so it
//!   is already at its start. Only with no state: Birth (its list has
//!   `Birth`) or Wait `birthDelay` seconds later, in game time through
//!   the queue ([`Character::wait_due`], which the caller runs after each
//!   pass over the event queue, so a delay of 0 is handled before the
//!   first picture, Q119). Mouth shape 0.
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
    /// The start-up (rule 12) has run: at its first update or its first
    /// movement request.
    pub started: bool,
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
    /// Its mover (rule 8, M4 4d-6b), set by the loader; `None` = never
    /// moves (no box).
    pub mover: Option<Box<crate::connor_mover::Mover>>,
    /// `flyingObject=1`: the flying mover (4d-6b-3).
    pub flying: bool,
    /// Its mask has 0x8: buildings stop its mover (the pair rule).
    pub hits_buildings: bool,
    /// Its collision type and mask (rule 8: `colliderType` /
    /// `colliderMask`, default 0x42 / 0x281F), set by the loader.
    pub collision_type: u32,
    pub collision_mask: u32,
    /// Its `BOUNDS.KQ` box (rule 9), set by the loader; `None` = no box
    /// (never solid).
    pub bounds: Option<([f32; 3], [f32; 3])>,
    /// The z it began falling from, while in the air (for the log).
    falling_from: Option<f32>,
    /// Its mover has run once (the first update's log line).
    mover_started: bool,
    /// Walking (M5 5a-2, `character-movement.md` rules 1–8).
    pub walk: Walk,
    /// Its movement rules and what it knows (M5 5a-4), set by the loader;
    /// `None` = none (it only does what it is told).
    pub ai: Option<crate::character_ai::Ai>,
    /// Its part in the fight (M5 5c, `fighting.md`), set by the loader for
    /// a monster-list line; `None` = it never fights.
    pub fight: Option<crate::fight::Fight>,
    /// Underground: not drawn, not solid (`characters.md` rule 16): set by
    /// the loader for a list with `Birth`; cleared when it rises
    /// (`character-movement.md` rule 30), set again when its `hide` ends
    /// after sinking back (rule 28; M5 5a-8).
    pub unborn: bool,
}

/// A character's walking (`character-movement.md` rules 1–8; M5 5a-2).
#[derive(Debug, Clone, Default)]
pub struct Walk {
    /// `run` was asked for rather than `walk` (rule 3).
    pub run: bool,
    /// The point (x, y) it steers at, if any (rule 8); `None` = straight on.
    pub steer: Option<[f32; 2]>,
    /// No steering before this game time (the stuck escape, rule 7).
    pub steer_off_until: f64,
    /// Game time of the next stuck check, and where it was at the last one.
    next_check: f64,
    check_from: [f32; 3],
    /// Stuck events since it started moving: each search starts one
    /// diagonal further round (rule 7).
    stuck_count: usize,
    /// The stuck check is on: its `walk` / `run` really plays (rule 7).
    check_on: bool,
    /// This update's step from the movement playing (world space), taken
    /// by its mover.
    pub step: [f32; 3],
    /// The stuck escape ran in the last mover update (a wanderer takes its
    /// next point then, rule 7; M5 5a-5).
    pub stuck_now: bool,
    /// Once a second while moving: where it was then, and when (the log).
    second_from: Option<([f32; 3], f64)>,
}

/// A character's own touch table (`character-movement.md` rule 33, Q128):
/// another whose type has one of these bits (and which its mask covers)
/// is an obstacle in its own move: pushables 0x100, buildings 0x8, plain
/// objects 0x1, characters 0x2, Connor 0x4, 0x2000. Traps (0x10) are
/// nothing.
pub const OWN_TOUCH_OBSTACLES: u32 = 0x100 | 0x8 | 0x1 | 0x2 | 0x4 | 0x2000;

/// The stuck check (rule 7): first this long after starting to move,
/// then every [`STUCK_EVERY`] s; stuck if it moved less than
/// [`STUCK_DISTANCE`] (3D) since the last check.
pub const STUCK_FIRST: f64 = 1.0;
pub const STUCK_EVERY: f64 = 0.5;
pub const STUCK_DISTANCE: f32 = 200.0;
/// The stuck escape: no steering for this long; the diagonals are tested
/// this far out; the mask of what blocks them (plain objects, characters,
/// buildings; rule 7).
pub const STUCK_NO_STEER: f64 = 3.0;
pub const STUCK_LOOK: f32 = 1250.0;
pub const STUCK_MASK: u32 = 0xB;
/// Rule 7 (Q132 b): the diagonal test line runs level from its feet; we
/// lift it this much, as the note's "Do not reproduce" recommends (the
/// ground blocks a diagonal only where it is higher than its feet).
pub const STUCK_LINE_LIFT: f32 = 1.0;
/// Rule 7 (Q132 a): the world's diagonals in the order tried, as (x, y)
/// steps: north-east, north-west, south-west, south-east.
pub const STUCK_DIAGONALS: [[f32; 2]; 4] = [[1.0, 1.0], [-1.0, 1.0], [-1.0, -1.0], [1.0, -1.0]];
/// The heading of a level direction (heading 0 faces −y, π/2 faces +x).
fn heading_of_direction(x: f32, y: f32) -> f32 {
    x.atan2(-y)
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
            started: false,
            wait_at: None,
            random_pending: false,
            special: VecDeque::new(),
            special_playing: None,
            rng: Rng::new(seed),
            turn_to: None,
            mouth: 0,
            connor_at: None,
            mover: None,
            flying: false,
            hits_buildings: false,
            collision_type: kq8_data::characters::CHARACTER_TYPE,
            collision_mask: kq8_data::characters::CHARACTER_MASK,
            bounds: None,
            falling_from: None,
            mover_started: false,
            walk: Walk::default(),
            ai: None,
            fight: None,
            unborn: false,
        }
    }

    /// A random number 0..1 from its own draws (M5: random wander points,
    /// the fidget choice and times, the reaction delay).
    pub fn random(&mut self) -> f32 {
        self.rng.fraction()
    }

    /// A seed from the character's name (so two goblins differ, and a run
    /// is the same every time).
    pub fn seed_for(name: &str) -> u64 {
        name.bytes().fold(0xCBF2_9CE4_8422_2325u64, |h, b| (h ^ u64::from(b.to_ascii_lowercase())).wrapping_mul(0x0100_0000_01B3))
    }

    /// Rule 8 (M4 4d-6b-2): its mover's update, standing still (walking
    /// is M5), only while near or far and not in Birth (rule 16: gravity
    /// off until born). A script's `move` has set the player's position;
    /// the mover starts from there. Returns log lines: the first update,
    /// leaving the ground, landing.
    pub fn gravity_step(&mut self, player: &mut NamedAnimPlayer, world: &dyn crate::connor_mover::MoverWorld, now: f64) -> Vec<String> {
        let mut notes = Vec::new();
        if !self.in_range || self.state == State::Birth {
            return notes;
        }
        // M5 5a-2 (character-movement.md rule 1): the movement's step.
        let wanted = std::mem::take(&mut self.walk.step);
        self.walk.stuck_now = false;
        let first = !std::mem::replace(&mut self.mover_started, true);
        if self.flying {
            let Some(m) = self.mover.as_deref_mut() else { return notes };
            m.position = player.base.position;
            if first {
                notes.push(format!("mover: first update at z {:.0}: flyingObject=1, the flying mover (no gravity, no steps; down 150 all or nothing, raised up to 300 onto the drawn ground or a building; character-movement.md rule 32)", m.position[2]));
            }
            let before = m.position[2];
            let (r, on) = m.fly_update(wanted, world);
            player.base.position = m.position;
            own_hits(&r, &mut notes);
            let z = m.position[2];
            if (z - before).abs() > 0.5 {
                if self.falling_from.is_none() {
                    self.falling_from = Some(before);
                    notes.push(format!("flying: moves from z {before:.0}"));
                }
            } else if let Some(from) = self.falling_from.take() {
                notes.push(format!("flying: settles at z {z:.0} (moved {:.0}); last update: {} (character-movement.md rule 32)", z - from, on.unwrap_or_default()));
            }
            self.after_move(player, world, now, &mut notes);
            return notes;
        }
        let gravity = self.gravity;
        let hits_buildings = self.hits_buildings;
        let Some(m) = self.mover.as_deref_mut() else {
            if first {
                notes.push("mover: no box, so no mover; it stays where it is".into());
            }
            return notes;
        };
        if first {
            let ground = world.ground(m.position[0], m.position[1]).map_or("none".to_string(), |(h, _)| format!("{h:.0}"));
            notes.push(format!(
                "mover: first update at z {:.0} (floor under it {ground}); gravity {}; steps {:.0}/{:.0}; terrain in the passes {}; buildings {}; keepOnGround {}; plain objects, other characters and Connor are obstacles by its mask {:#x} (character-movement.md rule 33) (rule 8)",
                m.position[2],
                if gravity { "on" } else { "off" },
                m.step_up,
                m.step_down,
                if m.terrain_in_passes { "yes" } else { "no (mask without 0x800)" },
                if hits_buildings { "yes" } else { "no (mask without 0x8)" },
                if !m.keep_on_ground { "off" } else if m.keep_on_surface { "the water surface" } else { "the floor" },
                self.collision_mask
            ));
        }
        m.position = player.base.position;
        if m.use_gravity != gravity {
            notes.push(format!("gravity {} (setUseGravity / setMonUseGravity)", if gravity { "on" } else { "off" }));
            m.use_gravity = gravity;
        }
        let before = m.position[2];
        let r = m.update(wanted, world);
        player.base.position = m.position;
        let z = m.position[2];
        own_hits(&r, &mut notes);
        for e in &r.events {
            notes.push(format!("mover: {e}"));
        }
        if !m.on_ground && z < before - 0.5 && self.falling_from.is_none() {
            self.falling_from = Some(before);
            notes.push(format!("leaves the ground: falling from z {before:.0} (rule 8; no Fall state for characters)"));
        }
        if m.on_ground {
            if let Some(from) = self.falling_from.take() {
                let on = r.hits.iter().rev().find(|h| h.pass == "down").map_or_else(|| format!("the ground check ({})", r.check), |h| h.hit.what.clone());
                notes.push(format!("lands on {on} at z {z:.0} (fell {:.0})", from - z));
            }
        }
        self.after_move(player, world, now, &mut notes);
        notes
    }

    /// After its mover (rule 7): in Move, the once-a-second line and the
    /// stuck check.
    fn after_move(&mut self, player: &mut NamedAnimPlayer, world: &dyn crate::connor_mover::MoverWorld, now: f64, notes: &mut Vec<String>) {
        if self.state != State::Move {
            self.walk.second_from = None;
            return;
        }
        let pos = player.base.position;
        match self.walk.second_from {
            Some((from, t)) if now + 1e-6 >= t + 1.0 => {
                let moved = ((pos[0] - from[0]).powi(2) + (pos[1] - from[1]).powi(2)).sqrt();
                let steering = match self.walk.steer {
                    _ if now < self.walk.steer_off_until => format!("off until {:.2} (stuck escape)", self.walk.steer_off_until),
                    Some(p) => format!("at ({:.0}, {:.0})", p[0], p[1]),
                    None => "none (straight on)".into(),
                };
                notes.push(format!(
                    "moving: at ({:.0}, {:.0}, {:.0}) heading {:.3}, '{}' (speed {:.3}), {moved:.0} units across in the last {:.2} s; steering {steering}",
                    pos[0],
                    pos[1],
                    pos[2],
                    heading_of(&player.base.rotation),
                    player.movement_name(),
                    player.speed(),
                    now - t,
                ));
                self.walk.second_from = Some((pos, now));
            }
            None => self.walk.second_from = Some((pos, now)),
            _ => {}
        }
        // Rule 7 (Q132): the check is on only once its walk or run really
        // plays; the first check is 1 s after that.
        if !self.walk.check_on {
            let wanted = Self::moving_name(player, self.walk.run);
            let plays = player.movement_name().eq_ignore_ascii_case(wanted) || player.movement_name().eq_ignore_ascii_case(Self::moving_name(player, !self.walk.run));
            if !plays {
                return;
            }
            self.walk.check_on = true;
            self.walk.next_check = now + STUCK_FIRST;
            self.walk.check_from = pos;
            notes.push(format!("'{}' plays: the stuck check is on, first at {:.2} (rule 7)", player.movement_name(), self.walk.next_check));
            return;
        }
        if now + 1e-6 < self.walk.next_check {
            return;
        }
        let d: [f32; 3] = std::array::from_fn(|k| pos[k] - self.walk.check_from[k]);
        let moved = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        self.walk.check_from = pos;
        self.walk.next_check = now + STUCK_EVERY;
        if moved < STUCK_DISTANCE {
            self.stuck_escape(player, world, now, moved, notes);
        }
    }

    /// Rule 7, the stuck escape: no steering for 3 s; the first free one of
    /// the four diagonals, from one further round each time; a snap turn
    /// to it; `run`.
    fn stuck_escape(&mut self, player: &mut NamedAnimPlayer, world: &dyn crate::connor_mover::MoverWorld, now: f64, moved: f32, notes: &mut Vec<String>) {
        use crate::connor_auto::wrap;
        let pos = player.base.position;
        let from = [pos[0], pos[1], pos[2] + STUCK_LINE_LIFT];
        let mut tried = Vec::new();
        let mut chosen = None;
        for i in 0..4 {
            let k = (self.walk.stuck_count + i) % 4;
            // Rule 7 (Q132 a): NE, NW, SW, SE, ±1,250 in x and in y.
            let [dx, dy] = STUCK_DIAGONALS[k];
            let h = heading_of_direction(dx, dy);
            let to = [from[0] + STUCK_LOOK * dx, from[1] + STUCK_LOOK * dy, from[2]];
            match world.line_blocked(from, to, STUCK_MASK, &player.name) {
                Some(what) => tried.push(format!("{h:.3} blocked by {what}")),
                None => {
                    tried.push(format!("{h:.3} free"));
                    chosen = Some(h);
                    break;
                }
            }
        }
        self.walk.stuck_count += 1;
        let was = heading_of(&player.base.rotation);
        // Rule 7: all four blocked → it faces north (+y).
        let (to, how) = match chosen {
            Some(h) => (h, "the first free diagonal"),
            None => (heading_of_direction(0.0, 1.0), "all four blocked: faces north (rule 7)"),
        };
        let r = player.base.rotation;
        player.base.rotation = times(&r, &kq8_data::objects::rotation([0.0, 0.0, wrap(to - was)]));
        self.turn_to = None;
        self.walk.steer_off_until = now + STUCK_NO_STEER;
        self.walk.stuck_now = true;
        let run = self.set_moving(player, true);
        notes.push(format!(
            "is stuck (rule 7): moved {moved:.0} (< {STUCK_DISTANCE}) since the last check; diagonals: {}; snap turn {was:.3} -> {:.3} ({how}); no steering until {:.2}; run: {run}",
            tried.join(", "),
            heading_of(&player.base.rotation),
            self.walk.steer_off_until
        ));
    }

    /// Rule 3: the movement for walking or running, with its fallbacks
    /// (no `run`: `walk`; no `walk` but `run`: `run`).
    pub fn moving_name(player: &NamedAnimPlayer, run: bool) -> &'static str {
        let has = |n: &str| player.movements.iter().any(|m| m.name.eq_ignore_ascii_case(n));
        match (run, has("run"), has("walk")) {
            (true, true, _) | (false, true, false) => "run",
            _ => "walk",
        }
    }

    /// Asks for `walk` or `run` (rule 3) the ordinary way. Returns what
    /// happened, for the log.
    pub fn set_moving(&mut self, player: &mut NamedAnimPlayer, run: bool) -> String {
        self.walk.run = run;
        let name = Self::moving_name(player, run);
        if player.movement_name().eq_ignore_ascii_case(name) && player.waiting.is_none() {
            return format!("'{name}' already playing");
        }
        match player.request(name) {
            Request::AtOnce(st) => format!("'{name}' starts at once{}", if st.unknown { " (not in its list: movement 0)" } else { "" }),
            Request::Waiting => format!("'{name}' waits for '{}' to end", player.movement_name()),
            Request::WaitingForever => format!("'{name}' waits for '{}', which never ends", player.movement_name()),
            Request::Transition(t) => format!("transition '{}' toward '{name}'", t.via),
        }
    }

    /// Rule 4: into Move. `Stop` at a random frame and speed first, then at
    /// once `walk` or `run`; the stuck check starts (rule 7). Returns the
    /// log line.
    pub fn enter_move(&mut self, player: &mut NamedAnimPlayer, now: f64, run: bool, why: &str) -> String {
        let cut = self.attack_cut_short(now).map(|l| format!("{l}; ")).unwrap_or_default();
        let from = self.state;
        self.state = State::Move;
        self.wait_at = None;
        self.special.clear();
        self.special_playing = None;
        let mut first = Vec::new();
        self.start_up_in_state(player, &mut first);
        let stop = self.ask_stop(player, true);
        let moving = self.set_moving(player, run);
        // Rule 7 (Q132): the check waits for the walk to play; each start
        // of moving begins the round at north-east again.
        self.walk.check_on = false;
        self.walk.stuck_count = 0;
        self.walk.next_check = f64::MAX;
        self.walk.check_from = player.base.position;
        self.walk.second_from = None;
        let first = first.iter().map(|l| format!("{l}; ")).collect::<String>();
        format!("{cut}{first}state Move (from {from:?}; {why}): Stop {stop}; then {moving} (rule 4)")
    }

    /// Rule 1: this update's step from a repeating movement (its frame
    /// steps added up × the phase moved, spread evenly as Connor's loops),
    /// turned by its rotation, for its mover. Nothing when a movement
    /// started, ended or changed in this update, or for a play-once one.
    fn movement_step(&mut self, player: &NamedAnimPlayer, played: &Played) {
        if played.started.is_some() || played.transition.is_some() || played.finished.is_some() {
            return;
        }
        let Some(m) = player.movement().filter(|m| m.repeat) else { return };
        let mut d = played.to - played.from;
        if player.speed() >= 0.0 && d < 0.0 {
            d += 1.0;
        } else if player.speed() < 0.0 && d > 0.0 {
            d -= 1.0;
        }
        if d == 0.0 {
            return;
        }
        let sum = m.frame_steps.iter().fold([0.0f32; 3], |a, t| [a[0] + t.p[0], a[1] + t.p[1], a[2] + t.p[2]]);
        let r = &player.base.rotation;
        // Row vectors: own step × rotation.
        self.walk.step = std::array::from_fn(|j| (0..3).map(|i| sum[i] * d * r[i][j]).sum());
    }

    /// One update (rule 11: nothing at all while off or hidden).
    pub fn update(&mut self, player: &mut NamedAnimPlayer, now: f64, dt: f32, update: u64) -> Step {
        let mut step = Step::default();
        if !self.in_range {
            return step;
        }
        let notes = &mut step.notes;
        if self.first_update.is_none() {
            self.first_update = Some(update);
            match self.start_up(player, now, &format!("first update {update}")) {
                Some(line) => notes.push(line),
                None => notes.push(format!("first update {update}: the start-up already ran (at its first movement request); nothing more (rule 12)")),
            }
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
        // M5 5a-2, rule 5: in Move, steering aims the turn at its point
        // anew each update (not during the stuck escape's 3 s; noTurn
        // blocks it).
        let steering = self.state == State::Move && !self.no_turn && now >= self.walk.steer_off_until;
        if let Some(p) = self.walk.steer.filter(|_| steering) {
            self.turn_to = Some(p);
        }
        self.turn_step(player, dt, notes, self.state == State::Move);
        let played = player.advance(dt);
        self.movement_step(player, &played);
        if let Some(t) = &played.transition {
            notes.push(format!("transition '{}' starts: the from point was reached at phase {:.4} (rule 20, connor-movement.md rule 3)", t.name, played.to));
        }
        if let (Some(st), Some(f)) = (&played.started, &played.finished) {
            notes.push(format!("'{f}' ended: '{}' starts at phase {:.4}", st.name, st.start));
        }
        if let Some(st) = &played.started {
            if self.random_pending && st.name.eq_ignore_ascii_case(STOP) {
                self.random_pending = false;
                let (frame, speed) = player.randomize(&mut self.rng);
                notes.push(format!("'{}' started after the movement before it ended: random frame {frame}, speed {speed:.3} (rule 13)", st.name));
            }
        }
        if let Some(m) = &played.finished {
            self.after_end(player, m, notes);
            if let Some(l) = self.attack_ended(m, now) {
                notes.push(l);
            }
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
        // Rule 28: not walking, the movement playing can be cut short or
        // has ended, and the list has the gesture.
        if self.state == State::Move {
            return format!("code {code}: '{gesture}' not played: walking (rule 28)");
        }
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

    /// `talking.md` rule 13a (Q117; M4 4d-10): the line marked last of a
    /// `speak` in this character's queue has ended, before its
    /// `TalkComplete`. Already in Wait: nothing (no `Stop`, a held pose
    /// stays). Otherwise `Stop` is asked for and it goes into Wait, which
    /// no state, Talk and Special accept, Special only once its list's
    /// last movement has started (else the list goes on). Walking (Talk to
    /// Move) waits for M5. Returns the log line.
    pub fn after_speech(&mut self, player: &mut NamedAnimPlayer) -> String {
        const WHY: &str = "end of a speak (talking.md rule 13a)";
        match self.state {
            State::Wait => format!("{WHY}: already in Wait: nothing (no Stop request)"),
            State::Special if !self.special.is_empty() => {
                let stop = self.ask_stop(player, false);
                format!("{WHY}: Stop asked for ({stop}); Wait refused: {} item(s) of its Special list still to come, the list goes on", self.special.len())
            }
            State::None | State::Talk | State::Special => {
                let from = self.state;
                self.turn_to = None;
                let mut notes = Vec::new();
                self.enter_wait(player, &mut notes);
                format!("{WHY}: from {from:?} (no walking before M5): {}", notes.join("; "))
            }
            s => {
                let stop = self.ask_stop(player, false);
                format!("{WHY}: Stop asked for ({stop}); state {s:?} kept (the note names only no state, Wait, Talk and Special)")
            }
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
        let mut first = Vec::new();
        self.start_up_in_state(player, &mut first);
        player.cut_short = true;
        let stop = match player.request(STOP) {
            Request::AtOnce(st) => format!("Stop starts at once{}", if st.unknown { " (no Stop in its list: movement 0)" } else { "" }),
            _ => format!("Stop waits for '{}'", player.movement_name()),
        };
        let turn = if self.no_turn { "; noTurn: it will not turn toward Connor (characters.md rule 24)" } else { "" };
        let first = first.iter().map(|l| format!("{l}; ")).collect::<String>();
        format!("{first}state Talk (from {from:?}): '{}' marked can be cut short; {stop}{turn}", player.movement_name())
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
    fn turn_step(&mut self, player: &mut NamedAnimPlayer, dt: f32, notes: &mut Vec<String>, quiet: bool) {
        use crate::connor_auto::{heading_towards, wrap, AUTO_TURN_RATE};
        let Some(to) = self.turn_to else { return };
        let r = player.base.rotation;
        let heading = heading_of(&r);
        let left = wrap(heading_towards(player.base.position, to) - heading);
        let step = AUTO_TURN_RATE * dt;
        let by = if left.abs() < step {
            self.turn_to = None;
            // character-movement.md rule 5: steering aims anew each update
            // until it faces the point, then it walks straight (M5 5a-5).
            if self.state == State::Move {
                self.walk.steer = None;
            }
            left
        } else {
            step * left.signum()
        };
        player.base.rotation = times(&r, &kq8_data::objects::rotation([0.0, 0.0, by]));
        // While walking the turn is aimed anew each update: only real
        // turns are logged.
        if quiet && by.abs() < 0.001 {
            return;
        }
        let done = if self.turn_to.is_none() { "; turn done" } else { "" };
        notes.push(format!("turning to ({:.0}, {:.0}): heading {heading:.3} -> {:.3} (by {by:.3}){done}", to[0], to[1], heading_of(&player.base.rotation)));
    }

    /// Rule 12, the start-up, once: at the first update or before the
    /// first movement request (`why` says which), whatever its range or
    /// `hidden`. The current movement is the one loaded, still at its
    /// start (12.1: "starts afresh"). Only with no state: Birth, or Wait
    /// `birthDelay` s later through the queue (12.2). The mouth goes to 0
    /// (12.3). Returns the log line; `None` if it already ran.
    pub fn start_up(&mut self, player: &mut NamedAnimPlayer, now: f64, why: &str) -> Option<String> {
        if self.started {
            return None;
        }
        self.started = true;
        self.mouth = 0;
        let mut line = format!(
            "start-up ({why}, rule 12): movement '{}' ({}) at phase {:.4} speed {:.3}",
            player.movement_name(),
            player.movement().map_or("none", |m| m.name.as_str()),
            player.phase,
            player.speed()
        );
        if self.state != State::None {
            line.push_str(&format!("; already in {:?} (set by a script before): kept, nothing queued", self.state));
        } else if self.has_birth {
            self.state = State::Birth;
            line.push_str("; its list has Birth: Birth state, not drawn until it rises (rule 16; rising is M5)");
        } else {
            let delay = f64::from(self.birth_delay.max(0.0));
            self.wait_at = Some(now + delay);
            line.push_str(&format!("; Wait queued in {delay} s (birthDelay, game time)"));
        }
        Some(line)
    }

    /// A state's own movement request (Wait's `Stop`, a Special list's
    /// first item) runs the start-up if it never ran: the state is already
    /// set, so nothing is queued (rule 12, `sprdump`).
    fn start_up_in_state(&mut self, player: &mut NamedAnimPlayer, notes: &mut Vec<String>) {
        let why = format!("the {:?} state's movement request", self.state);
        // `now` is not used: with a state set, nothing is queued.
        if let Some(line) = self.start_up(player, 0.0, &why) {
            notes.push(line);
        }
    }

    /// The start-up's Wait, when due (rule 12.2: an event in the queue,
    /// so whatever the range). The caller runs this after each pass over
    /// the event queue. Returns the log line.
    pub fn wait_due(&mut self, player: &mut NamedAnimPlayer, now: f64) -> Option<String> {
        if !self.wait_at.is_some_and(|t| now >= t) {
            return None;
        }
        self.wait_at = None;
        let mut notes = vec![format!("Wait begins (rule 12, birthDelay {} s)", self.birth_delay)];
        notes.extend(self.attack_cut_short(now));
        self.enter_wait(player, &mut notes);
        Some(notes.join("; "))
    }

    /// Wait (rule 13): asks for `Stop`, random by `flags` bit 0x10.
    fn enter_wait(&mut self, player: &mut NamedAnimPlayer, notes: &mut Vec<String>) {
        self.state = State::Wait;
        self.start_up_in_state(player, notes);
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
            // Rule 20: the transition's target starts at its own phase.
            Request::Transition(t) => format!("transition '{}' chosen: waits for phase {} of '{}', then '{}' at {} (rule 20)", t.via, t.from, player.movement_name(), t.to, t.to_phase),
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
            State::Move => {
                // character-movement.md rule 4: a walk or run that ends is
                // asked for again.
                let run = self.walk.run;
                let again = self.set_moving(player, run);
                notes.push(format!("'{ended}' ended in Move: {again} (character-movement.md rule 4)"));
            }
            State::Birth if ended.eq_ignore_ascii_case("hide") => {
                // character-movement.md rule 28 (M5 5a-8): back underground.
                self.unborn = true;
                notes.push("'hide' ended in Birth: underground again, not drawn and not solid (character-movement.md rule 28)".into());
            }
            State::None => notes.push(format!("'{ended}' ended in no state: holds its last frame (rule 15)")),
            s => notes.push(format!("'{ended}' ended in {s:?}: nothing more")),
        }
    }

    /// `KQMonster::setState <name> <state> [words…]` (rule 22). Returns
    /// the log line.
    pub fn set_state(&mut self, player: &mut NamedAnimPlayer, state: &str, words: &[String], now: f64) -> String {
        let mut notes = Vec::new();
        if state.eq_ignore_ascii_case("Wait") {
            notes.extend(self.attack_cut_short(now));
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
            self.start_up_in_state(player, &mut notes);
            self.special_next(player, &mut notes);
        } else if state.eq_ignore_ascii_case("Unborn") {
            self.state = State::Birth;
            self.wait_at = None;
            notes.push("state Birth (Unborn); hiding a character that was drawn is not done (none in Daventry)".into());
        } else if state.eq_ignore_ascii_case("Move") {
            // character-movement.md "Edge cases": not active, it just
            // walks straight on (`walk`, as wandering asks: our reading).
            self.walk.steer = None;
            notes.push(self.enter_move(player, now, false, "setState Move"));
        } else if state.eq_ignore_ascii_case("Die") {
            notes.push(format!("{state}: dying is the fight study; state unchanged ({:?})", self.state));
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

impl Character {
    /// `fighting.md` rule 8 (M5 5c-3): asks for `Attack` the ordinary way
    /// (the caller made sure the running movement can be cut short); the
    /// attack time is "never" and the chase timer 5 s from now (rule 9); a
    /// moving one's stuck check is off until its `walk` / `run` plays
    /// again. Returns what happened, for the log.
    pub fn start_attack(&mut self, player: &mut NamedAnimPlayer, now: f64) -> String {
        let how = match player.request(crate::fight::ATTACK) {
            Request::AtOnce(st) => format!("'{}' starts at once{}", crate::fight::ATTACK, if st.unknown { " (not in its list: movement 0)" } else { "" }),
            _ => format!("'{}' waits for '{}' (should not happen: it can be cut short)", crate::fight::ATTACK, player.movement_name()),
        };
        if let Some(f) = self.fight.as_mut() {
            f.attack_time = crate::fight::NEVER;
        }
        if let Some(ai) = self.ai.as_mut() {
            ai.chase_until = now + crate::fight::ATTACK_CHASE;
        }
        self.walk.check_on = false;
        self.walk.next_check = f64::MAX;
        how
    }

    /// `Attack` is playing (rule 9; for the Do instead of rule 11b).
    pub fn attack_playing(&self, player: &NamedAnimPlayer) -> bool {
        player.movement_name().eq_ignore_ascii_case(crate::fight::ATTACK) && !player.has_ended()
    }

    /// Rule 10: a movement named `Attack` or `Attack2` ended (an idle
    /// fidget too): the next attack is due `attackDelay` from now, the
    /// chase timer 5 s.
    fn attack_ended(&mut self, ended: &str, now: f64) -> Option<String> {
        // fighting.md rule 23 (M5 5c-6): its flinch ended: the 2 s pause
        // starts, chase timer 5 s.
        if let Some(f) = self.fight.as_mut().filter(|f| f.flinching.as_deref().is_some_and(|m| m.eq_ignore_ascii_case(ended))) {
            f.flinching = None;
            f.flinch_end = now;
            if let Some(ai) = self.ai.as_mut() {
                ai.chase_until = now + crate::fight::ATTACK_CHASE;
            }
            return Some(format!("fight: its flinch '{ended}' ended: no flinch for 2 s, chase timer 5 s (fighting.md rule 23)"));
        }
        if !crate::fight::is_attack(ended) {
            return None;
        }
        let f = self.fight.as_mut()?;
        let delay = f64::from(f.settings.attack_delay);
        f.attack_time = now + delay;
        if let Some(ai) = self.ai.as_mut() {
            ai.chase_until = now + crate::fight::ATTACK_CHASE;
        }
        Some(format!("fight: '{ended}' ended: next attack due at {:.2} (now + attackDelay {delay} s), chase timer 5 s (fighting.md rule 10)", f.attack_time))
    }

    /// Rule 11: an attack that never ended (something replaced it) leaves
    /// the attack time at "never"; entering Wait or Move puts it right:
    /// now + `attackDelay`, chase timer 5 s.
    fn attack_cut_short(&mut self, now: f64) -> Option<String> {
        let f = self.fight.as_mut()?;
        if f.attack_time != crate::fight::NEVER {
            return None;
        }
        let delay = f64::from(f.settings.attack_delay);
        f.attack_time = now + delay;
        if let Some(ai) = self.ai.as_mut() {
            ai.chase_until = now + crate::fight::ATTACK_CHASE;
        }
        Some(format!("fight: the attack was cut short: next attack due at {:.2} (now + attackDelay {delay} s), chase timer 5 s (fighting.md rule 11)", f.attack_time))
    }
}

/// The heading a rotation faces, as Connor's: where its −Y goes, as
/// `heading_towards` measures it (0 faces −Y, π/2 faces +X).
pub fn heading_of(r: &[[f32; 3]; 3]) -> f32 {
    // Row vectors: (0, −1, 0) × R = −(second row).
    let (x, y) = (-r[1][0], -r[1][1]);
    x.atan2(-y)
}

/// `character-movement.md` rule 6: the hits of a character's own move
/// (up: stop; across and slides: slide, stays after the 3rd retry), for
/// the log. Down-pass landings are logged as landings.
fn own_hits(r: &crate::connor_mover::Report, notes: &mut Vec<String>) {
    for h in r.hits.iter().filter(|h| h.pass != "down") {
        let (p, n) = (h.hit.position, h.hit.normal);
        notes.push(format!(
            "own move: {} pass hit {} at ({:.0}, {:.0}, {:.0}), surface ({:.2}, {:.2}, {:.2}): {} (rule 6)",
            h.pass, h.hit.what, p[0], p[1], p[2], n[0], n[1], n[2], h.result
        ));
    }
}

/// a × b (row-vector rotations: a first, then b).
fn times(a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

#[cfg(test)]
pub(crate) mod tests {
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
        NamedAnimPlayer::new("raven", ms, current, base).with_stop_names(curr, next)
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
            // The queue's pass at the end of the update (Q119).
            log.extend(c.wait_due(p, n as f64 / 15.0));
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
    fn a_zero_length_mouth_pose_holds_in_wait() {
        // anim-scripts.md rule 10a (Q120): the Uglybeast's mouth poses'
        // models last 0 s: they never end, so Wait brings no Stop back.
        let mut ms = vec![movement("stop", 0.0, 1.0, true, true, vec![]), movement("f", 0.0, 1.0, false, true, vec![])];
        ms[1].play_time = 0.0;
        let base = crate::collide::Placement { rotation: crate::door::identity().m, position: [0.0; 3] };
        let mut p = NamedAnimPlayer::new("ugly", ms, 0, base).with_names_at_load(Some("stop"), None);
        let mut c = Character::new(true, 0x20, 0.0, false, true, true, 1);
        c.in_range = true;
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 3, &mut log);
        assert_eq!(c.state, State::Wait);
        c.mouth_code(&mut p, 1);
        run(&mut c, &mut p, n, 30, &mut log);
        assert_eq!(p.movement_name(), "f", "the pose holds: {log:?}");
        assert!(!log.iter().any(|l| l.contains("'f' ended")));
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
    fn the_start_up_queues_wait_handled_before_the_first_picture() {
        // characters.md rules 12-14: no currAnimName, so it starts on stop
        // (not movement 0 'attack'); birthDelay 0: Wait in the same update,
        // and with flags 0x10 Stop restarts at a random frame and speed.
        let mut c = character(0x10);
        let mut p = player(0, None, None);
        assert!(p.movement_name().eq_ignore_ascii_case("stop"));
        let mut log = Vec::new();
        run(&mut c, &mut p, 5, 1, &mut log);
        assert_eq!((c.first_update, c.state), (Some(5), State::Wait), "{log:?}");
        assert!(p.random_speed.is_some(), "{log:?}");
        let joined = log.join("\n");
        assert!(joined.contains("start-up (first update 5") && joined.contains("Wait queued in 0 s"), "{joined}");
    }

    #[test]
    fn the_start_up_runs_at_the_first_movement_request_whatever_the_range() {
        // sprdump (rule 12): a state set before its first update.
        let mut c = character(0x10);
        c.in_range = false;
        let mut p = player(0, None, None);
        let line = c.set_state(&mut p, "special", &["caw".to_string()], 0.0);
        assert!(line.contains("start-up (the Special state's movement request") && line.contains("kept, nothing queued"), "{line}");
        assert!(c.start_up(&mut p, 0.0, "again").is_none(), "only once");
        assert_eq!(c.wait_due(&mut p, 1.0), None, "nothing queued");
        c.in_range = true;
        let mut log = Vec::new();
        run(&mut c, &mut p, 0, 3, &mut log);
        assert_eq!((c.state, p.movement_name()), (State::Special, "caw"), "{log:?}");
        assert!(log.join("\n").contains("already ran"));
        // A plain request with no state queues Wait.
        let mut d = character(0x10);
        let line = d.start_up(&mut p, 0.0, "KQAnimPlayEvent caw").unwrap();
        assert!(line.contains("Wait queued in 0 s"), "{line}");
        assert!(d.wait_due(&mut p, 0.0).is_some());
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
    fn no_state_until_the_queued_wait() {
        // Rule 15: no state between the start-up and its Wait (here a
        // birthDelay of 2 s); a play-once that ends then holds its last frame.
        let mut c = Character::new(false, 0x20, 2.0, false, false, true, 1);
        c.in_range = true;
        let mut p = player(0, None, Some("stop"));
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 1, &mut log);
        assert_eq!(c.state, State::None);
        p.request("caw");
        run(&mut c, &mut p, n, 20, &mut log);
        assert!(log.iter().any(|l| l.contains("'caw' ended in no state")), "{log:?}");
        assert_eq!((c.state, p.movement_name()), (State::None, "caw"), "holds its last frame: {log:?}");
    }

    #[test]
    fn the_end_of_a_speak_goes_to_wait_but_not_from_wait() {
        let mut p = player(1, Some("stop"), None);
        // No state: Stop and Wait.
        let mut c = character(0x20);
        c.started = true;
        assert!(c.after_speech(&mut p).contains("from None"));
        assert_eq!(c.state, State::Wait);
        // Already in Wait: nothing, a held pose stays.
        p.start("caw");
        assert!(c.after_speech(&mut p).contains("nothing"));
        assert_eq!(p.movement_name(), "caw");
        // Talk accepts it.
        c.enter_talk(&mut p);
        c.after_speech(&mut p);
        assert_eq!(c.state, State::Wait);
        // Special with items to come: Stop asked for, the state stays.
        let mut log = Vec::new();
        c.set_state(&mut p, "special", &["attack,caw".to_string()], 0.0);
        let line = c.after_speech(&mut p);
        assert!(line.contains("refused") && c.state == State::Special, "{line}");
        // ...with its last movement started: Wait.
        run(&mut c, &mut p, 0, 17, &mut log);
        assert_eq!(p.movement_name(), "caw", "{log:?}");
        c.after_speech(&mut p);
        assert_eq!(c.state, State::Wait);
    }

    #[test]
    fn in_wait_the_end_of_a_transition_restarts_stop_at_a_random_frame() {
        // Rule 20 (Q124 a): the Girl's cower -> stop through cowerstop.
        let mut cower = movement("cower", 0.0, 1.0, false, false, vec![]);
        cower.transitions.push(kq8_data::objects::TransitionLine { from: 0.882353, to: "stop".into(), to_phase: 0.0, via: "cowerstop".into() });
        let ms = vec![movement("stop", 0.0, 1.0, true, true, vec![]), cower, movement("cowerstop", 0.0, 1.0, false, false, vec![])];
        let base = crate::collide::Placement { rotation: crate::door::identity().m, position: [0.0; 3] };
        let mut p = NamedAnimPlayer::new("Girl", ms, 0, base).with_stop_names(None, None);
        let mut c = character(0x20);
        let mut log = Vec::new();
        let n = run(&mut c, &mut p, 0, 2, &mut log);
        assert_eq!(c.state, State::Wait);
        p.request("cower");
        let n = run(&mut c, &mut p, n, 3, &mut log);
        assert!(matches!(p.request("stop"), Request::Transition(_)));
        run(&mut c, &mut p, n, 40, &mut log);
        let joined = log.join("
");
        assert!(joined.contains("transition 'cowerstop' starts"), "{joined}");
        assert!(joined.contains("'cowerstop' ended in Wait: asks for Stop again"), "{joined}");
        assert!(p.movement_name().eq_ignore_ascii_case("stop") && p.random_speed.is_some(), "{joined}");
    }

    struct Flat;
    impl crate::connor_mover::MoverWorld for Flat {
        fn ground(&self, _x: f32, _y: f32) -> Option<(f32, [f32; 3])> {
            Some((100.0, [0.0, 0.0, 1.0]))
        }
    }

    #[test]
    fn a_character_in_range_falls_onto_the_ground_and_says_so() {
        // Rule 8 (Q122): only while in range; its player follows the mover.
        let mut c = character(0x20);
        let mut p = player(1, Some("stop"), None);
        p.base.position = [0.0, 0.0, 2000.0];
        c.mover = Some(Box::new(crate::connor_mover::Mover::character(p.base.position, true, [-50.0; 3], [50.0, 50.0, 500.0], (0x42, 0x281F), (325.0, 325.0), -1)));
        c.in_range = false;
        assert!(c.gravity_step(&mut p, &Flat, 0.0).is_empty());
        assert_eq!(p.base.position[2], 2000.0, "not while off");
        c.in_range = true;
        let mut log = Vec::new();
        for _ in 0..60 {
            log.extend(c.gravity_step(&mut p, &Flat, 0.0));
        }
        assert_eq!(p.base.position[2], 100.0, "{log:?}");
        let joined = log.join("
");
        assert!(joined.contains("first update at z 2000") && joined.contains("leaves the ground") && joined.contains("lands on"), "{joined}");
        // Gravity off (setMonUseGravity 0): it stays.
        p.base.position[2] = 900.0;
        c.gravity = false;
        c.gravity_step(&mut p, &Flat, 0.0);
        assert_eq!(p.base.position[2], 900.0);
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
        let line = c.set_state(&mut p, "special", &["caw,attack".to_string()], 0.0);
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
        assert!(c.set_state(&mut p, "jump", &[], 0.0).contains("only Connor has"));
        assert!(c.set_state(&mut p, "dance", &[], 0.0).contains("cannot find state"));
        assert!(c.set_state(&mut p, "die", &[], 0.0).contains("fight study"));
        assert_eq!(c.state, State::None);
        c.set_state(&mut p, "WAIT", &[], 0.0);
        assert_eq!(c.state, State::Wait);
    }

    /// M5 5a-2: stop, a walk of `walk` units per second and a run of
    /// `run` (16 frames over 1 s, each stepping forward, −Y).
    pub(crate) fn walker(walk: f32, run: f32) -> (Character, NamedAnimPlayer) {
        let moving = |name: &str, speed: f32| {
            let mut m = movement(name, 0.0, 1.0, true, true, vec![]);
            m.frame_steps = vec![kq8_data::shape::Transform { flags: 0, m: crate::door::identity().m, p: [0.0, -speed / 16.0, 0.0] }; 16];
            m
        };
        let ms = vec![movement("stop", 0.0, 1.0, true, true, vec![]), moving("walk", walk), moving("run", run)];
        let base = crate::collide::Placement { rotation: crate::door::identity().m, position: [0.0, 0.0, 100.0] };
        let mut p = NamedAnimPlayer::new("goblin", ms, 0, base).with_stop_names(None, None);
        p.loops_move_base = true;
        let mut c = character(0x10);
        c.mover = Some(Box::new(crate::connor_mover::Mover::character(p.base.position, true, [-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0], (0x42, 0x281F), (325.0, 325.0), -1)));
        (c, p)
    }

    /// Updates `n` times from update `from`, each with its mover.
    fn walk_for(c: &mut Character, p: &mut NamedAnimPlayer, world: &dyn crate::connor_mover::MoverWorld, from: u64, n: u64, log: &mut Vec<String>) -> u64 {
        for k in from..from + n {
            let now = k as f64 / 15.0;
            log.extend(c.update(p, now, 1.0 / 15.0, k).notes);
            log.extend(c.gravity_step(p, world, now));
        }
        from + n
    }

    #[test]
    fn walking_moves_by_the_walk_steps_and_the_drawing_does_not_add_them() {
        // Rules 1, 4: Stop at a random frame, then walk at once; its speed
        // is its steps per loop over the loop length.
        let (mut c, mut p) = walker(917.0, 3068.0);
        let mut log = Vec::new();
        let n = walk_for(&mut c, &mut p, &Flat, 0, 2, &mut log);
        let line = c.set_state(&mut p, "Move", &[], n as f64 / 15.0);
        assert!(line.contains("state Move") && line.contains("'walk' starts at once"), "{line}");
        assert_eq!(c.state, State::Move);
        let start = p.base.position;
        // The walk first moves in the next update (started this update).
        let n = walk_for(&mut c, &mut p, &Flat, n, 16, &mut log);
        let gone = start[1] - p.base.position[1];
        assert!((gone - 917.0).abs() < 1.0, "15 moving updates = 1 s at 917: {gone} {log:?}");
        assert!(p.base.position[0].abs() < 0.01 && p.base.position[2] == 100.0);
        // The model is drawn at its place: no frame steps added.
        assert_eq!(p.placement().position, p.base.position);
        // No stuck event: it moved more than 200 since each check.
        assert!(!log.iter().any(|l| l.contains("is stuck")), "{log:?}");
        assert!(log.iter().any(|l| l.contains("moving: at")), "the once-a-second line");
        // Running: 3,068 a second.
        c.set_moving(&mut p, true);
        let from = p.base.position;
        walk_for(&mut c, &mut p, &Flat, n, 16, &mut log);
        let gone = from[1] - p.base.position[1];
        assert!((gone - 3068.0).abs() < 2.0, "{gone}");
    }

    #[test]
    fn steering_turns_at_three_and_a_half_radians_a_second_while_walking() {
        // Rule 5: the automatic turn aimed anew each update; it walks on.
        let (mut c, mut p) = walker(917.0, 3068.0);
        let mut log = Vec::new();
        c.set_state(&mut p, "Move", &[], 0.0);
        c.walk.steer = Some([100_000.0, 0.0]);
        walk_for(&mut c, &mut p, &Flat, 0, 3, &mut log);
        let h = heading_of(&p.base.rotation);
        assert!((h - 3.0 * 3.5 / 15.0).abs() < 0.01, "after 3 updates: {h}");
        walk_for(&mut c, &mut p, &Flat, 3, 12, &mut log);
        // It faced the point once (8 updates in), then walks straight.
        assert!(c.walk.steer.is_none(), "steering ends once it faces the point");
        let h = heading_of(&p.base.rotation);
        assert!((h - std::f32::consts::FRAC_PI_2).abs() < 0.05, "{h}");
        walk_for(&mut c, &mut p, &Flat, 15, 15, &mut log);
        assert_eq!(heading_of(&p.base.rotation), h, "straight on");
        assert!(p.base.position[0] > 0.0 && p.base.position[1] < 0.0, "an arc: {:?}", p.base.position);
        // noTurn: no steering.
        let (mut c, mut p) = walker(917.0, 3068.0);
        c.no_turn = true;
        c.set_state(&mut p, "Move", &[], 0.0);
        c.walk.steer = Some([100_000.0, 0.0]);
        walk_for(&mut c, &mut p, &Flat, 0, 15, &mut log);
        assert_eq!(heading_of(&p.base.rotation), 0.0);
    }

    /// Every line is blocked.
    struct Closed;
    impl crate::connor_mover::MoverWorld for Closed {
        fn ground(&self, _x: f32, _y: f32) -> Option<(f32, [f32; 3])> {
            Some((100.0, [0.0, 0.0, 1.0]))
        }
        fn line_blocked(&self, _from: [f32; 3], _to: [f32; 3], mask: u32, _skip: &str) -> Option<String> {
            assert_eq!(mask, 0xB);
            Some("a wall".into())
        }
    }

    #[test]
    fn the_stuck_escape_snaps_to_a_free_diagonal_and_runs() {
        // Rule 7: a slow walker (60 units a second) is stuck at the check
        // 1 s after it started: no steering for 3 s, a snap turn to the
        // first free diagonal (one further round each time), then run.
        let (mut c, mut p) = walker(60.0, 947.0);
        let mut log = Vec::new();
        c.set_state(&mut p, "Move", &[], 0.0);
        c.walk.steer = Some([0.0, -100_000.0]);
        walk_for(&mut c, &mut p, &Flat, 0, 15, &mut log);
        assert!(!log.iter().any(|l| l.contains("is stuck")), "not before 1 s: {log:?}");
        walk_for(&mut c, &mut p, &Flat, 15, 1, &mut log);
        let stuck = log.iter().find(|l| l.contains("is stuck")).expect("stuck at 1 s");
        // North-east first: heading 3π/4 (+x, +y).
        let ne = 3.0 * std::f32::consts::FRAC_PI_4;
        assert!(stuck.contains("2.356 free") && stuck.contains("'run' starts at once"), "{stuck}");
        assert!((heading_of(&p.base.rotation) - ne).abs() < 0.001);
        assert!(c.walk.run && c.walk.stuck_now);
        // Steering stays off for 3 s: still facing the diagonal.
        walk_for(&mut c, &mut p, &Flat, 17, 30, &mut log);
        assert!((heading_of(&p.base.rotation) - ne).abs() < 0.001);
        // All four blocked: turned round (stand-in).
        let (mut c, mut p) = walker(0.0, 0.0);
        let mut log = Vec::new();
        c.set_state(&mut p, "Move", &[], 0.0);
        walk_for(&mut c, &mut p, &Closed, 0, 16, &mut log);
        let stuck = log.iter().find(|l| l.contains("is stuck")).expect("stuck");
        assert!(stuck.contains("all four blocked: faces north"), "{stuck}");
        assert!((heading_of(&p.base.rotation).abs() - std::f32::consts::PI).abs() < 0.001);
        // The next check is half a second later; the search starts one
        // diagonal further round.
        walk_for(&mut c, &mut p, &Flat, 16, 8, &mut log);
        let second = log.iter().filter(|l| l.contains("is stuck")).nth(1).expect("stuck again at 1.5 s");
        assert!(second.starts_with("is stuck") && second.contains("diagonals: -2.356 free"), "north-west next: {second}");
    }
}
