//! Connor's mover: steps, gravity, slopes, walls, landing and the Fall
//! state.
//!
//! Rules from the behaviour note `docs/behaviour/connor-physics.md`
//! ("One update of the mover", "The terrain in the passes", "Gravity",
//! "Slopes", "Falling off an edge", "Walls" with "Exact hit handling",
//! "Collision shape", "Is gravity on at the start?"). M2 steps 2a, 2c,
//! 2d-3 and 2d-5: the terrain, buildings and objects are solid.
//!
//! Pushing (M3 step 3j, `pushing.md` rules 1–3): in the across pass a hit
//! on a pushable (type bit 0x100) goes through the push check
//! ([`Mover::push_check`]); it is a wall only when the check fails.

use std::cell::RefCell;

use crate::collide::{BallStack, BoxCollider, BuildingCollider, ColliderKind, ObjectHit, OBSTACLE_BITS, PUSHABLE, SCRIPTED_PUSH};

/// The ground under a point: height and unit normal (z > 0), or `None`
/// where there is no ground (outside the terrain grid). For the terrain
/// this is the **floor** grid, the ground under water (water.md: Connor,
/// with the default `keepOnGround`, takes height, normal and material from
/// the floor; the camera keeps above the floor too).
pub trait GroundLookup {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])>;
    /// The water test's view (water.md): the **surface** height and the
    /// ground flags at (x, y); `None` where there is no water to test.
    fn water(&self, _x: f32, _y: f32) -> Option<(f32, u32)> {
        None
    }
}

impl GroundLookup for kq8_data::Terrain {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])> {
        self.floor_at(x, y).map(|g| (g.height, g.normal))
    }

    fn water(&self, x: f32, y: f32) -> Option<(f32, u32)> {
        self.height_at(x, y).map(|g| (g.height, self.flags_at(x, y)))
    }
}

/// A contact a pass ended on.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    /// Where the mover is put (for buildings: already pulled back 25).
    pub position: [f32; 3],
    /// Where it touched (the terrain has no pull-back: the same).
    pub touch: [f32; 3],
    /// Unit surface direction (for buildings: pointing against the move).
    pub normal: [f32; 3],
    /// `terrain`, or the building's or object's name.
    pub what: String,
}

/// What the mover moves through: the ground, and the solid things a pass
/// can hit.
pub trait MoverWorld {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])>;
    /// The building or object contact nearest `from` when the balls move
    /// from `from` to `to`; the terrain is not part of this.
    fn sweep(&self, _balls: &BallStack, _from: [f32; 3], _to: [f32; 3]) -> Option<Hit> {
        None
    }
    /// Notes gathered by the sweeps since the last call (for the report).
    fn take_notes(&self) -> Vec<String> {
        Vec::new()
    }
    /// Surface height and ground flags for the water test (`GroundLookup::water`).
    fn water(&self, _x: f32, _y: f32) -> Option<(f32, u32)> {
        None
    }
    /// The object `name` if it is a pushable (type bit 0x100) now
    /// (`pushing.md` rule 1).
    fn push_target(&self, _name: &str) -> Option<PushTarget> {
        None
    }
    /// Moves physical pushable `name` by `step` (`pushing.md` rule 4):
    /// its own sweep, then the ground rule. Returns how far it really
    /// moved (zero if it couldn't) and a note for the log.
    fn push_object(&self, _name: &str, _step: [f32; 3]) -> ([f32; 3], String) {
        ([0.0; 3], "objects can't be moved here".into())
    }
    /// A scripted pushable's touch is not a hit (`pushing.md` rule 3):
    /// `name` is passed through for the rest of this update's move.
    fn pass_through(&self, _name: &str) {}
}

/// Two contacts this close count as at the same distance (Q88 stand-in).
pub const TIE: f32 = 0.001;

/// Connor follows a physical pushable by this share of its movement
/// (`pushing.md` rule 4.3).
pub const PUSH_FOLLOW: f32 = 0.95;
/// Smallest step that pushes, on some axis (rule 4).
pub const PUSH_MIN_STEP: f32 = 0.000001;

/// A pushable that was touched (`pushing.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PushTarget {
    pub kind: ColliderKind,
    /// The object's position (its placement point; Q87 e stand-in for
    /// "its centre").
    pub position: [f32; 3],
}

impl PushTarget {
    /// Scripted pushable: bits 0x100 and 0x10000.
    pub fn scripted(&self) -> bool {
        self.kind.collision_type & SCRIPTED_PUSH != 0
    }
}

/// What Connor brings to a push (`pushing.md` rule 2), from the walker.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct PushAbility {
    /// His state would take the Push state (Wait, Move, or Push itself).
    pub accepts: bool,
    /// He is in the Push state already.
    pub pushing: bool,
    /// Empty hands (hand mode 4).
    pub empty_hands: bool,
    /// His chosen movement is `walk`, `run` or `push`.
    pub movement_ok: bool,
    /// The way he faces, horizontal, unit length.
    pub facing: [f32; 2],
}

/// Facing within 60° (`pushing.md` rule 2.5).
pub const PUSH_FACING_COS: f32 = 0.5;

/// One pushing touch's check, for the log and the walker.
#[derive(Debug, Clone, PartialEq)]
pub struct PushTouch {
    pub object: String,
    pub scripted: bool,
    /// Rule 2.2: on the ground (not jumping or falling) or pushing.
    pub on_ground: bool,
    /// Rule 2.3.
    pub empty_hands: bool,
    /// Rule 2.4: the across pass (or its slides).
    pub across: bool,
    /// Rule 2.5: the cosine of his facing against the surface (scripted)
    /// or against the direction to the object (physical).
    pub cos: f32,
    /// Rule 2.6.
    pub movement_ok: bool,
    /// Rule 3: his state takes Push.
    pub accepts: bool,
    /// What came of it.
    pub result: PushResult,
    /// A physical push: the step asked for, how far the object moved,
    /// and why (rule 4).
    pub moved: Option<([f32; 3], [f32; 3], String)>,
}

/// What a touch on a pushable led to.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PushResult {
    /// A check failed: an ordinary wall hit.
    Wall,
    /// Rule 2.5 failed while pushing: a wall hit, and back to Move.
    WallBackToMove,
    /// Rule 2.6 failed: a wall hit, and standing (Wait).
    WallStand,
    /// All checks passed: Push state (rule 3). In step 3j-1 the touch
    /// still counts as a wall hit (the object's part comes in 3j-2/3).
    Push,
}

impl PushTouch {
    pub fn describe(&self) -> String {
        let yes = |b: bool| if b { "yes" } else { "NO" };
        format!(
            "push touch {} ({}): on ground {}, empty hands {}, across pass {}, facing cos {:.3} ({}), movement {}, state takes Push {} -> {:?}",
            self.object,
            if self.scripted { "scripted" } else { "physical" },
            yes(self.on_ground),
            yes(self.empty_hands),
            yes(self.across),
            self.cos,
            yes(self.cos >= PUSH_FACING_COS),
            yes(self.movement_ok),
            yes(self.accepts),
            self.result
        ) + &self.moved.as_ref().map_or(String::new(), |(asked, moved, why)| {
            format!("; asked ({:.1}, {:.1}), moved ({:.1}, {:.1}, {:.1}): {why}", asked[0], asked[1], moved[0], moved[1], moved[2])
        })
    }
}

/// Ground only, nothing else solid (the unit tests' made-up grounds).
impl<T: GroundLookup> MoverWorld for T {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])> {
        GroundLookup::ground(self, x, y)
    }

    fn water(&self, x: f32, y: f32) -> Option<(f32, u32)> {
        GroundLookup::water(self, x, y)
    }
}

/// The ground, the world's buildings and the objects that collide with
/// Connor (already chosen by the type and mask rule).
pub struct Scene<'a> {
    pub ground: &'a dyn GroundLookup,
    pub buildings: &'a [BuildingCollider],
    pub objects: &'a [BoxCollider],
    /// Objects a ball started inside (no usable contact), noted by `sweep`.
    pub notes: RefCell<Vec<String>>,
    /// Physical pushables moved this update (`push_object`): name and
    /// movement, for the caller to apply.
    pub moved: RefCell<Vec<(String, [f32; 3])>>,
    /// Scripted pushables passed through this update (`pass_through`).
    pub passed: RefCell<Vec<String>>,
    /// Scripted pushables whose script didn't take control: touches are
    /// wall hits ("Do not reproduce" of `pushing.md`), set by the caller.
    pub not_pushable: Vec<String>,
}

impl<'a> Scene<'a> {
    pub fn new(ground: &'a dyn GroundLookup, buildings: &'a [BuildingCollider], objects: &'a [BoxCollider]) -> Self {
        Scene { ground, buildings, objects, notes: RefCell::new(Vec::new()), moved: RefCell::new(Vec::new()), passed: RefCell::new(Vec::new()), not_pushable: Vec::new() }
    }
}

impl MoverWorld for Scene<'_> {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])> {
        self.ground.ground(x, y)
    }

    fn water(&self, x: f32, y: f32) -> Option<(f32, u32)> {
        self.ground.water(x, y)
    }

    /// Over every building and object hit, the contact nearest the old
    /// position is kept (connor-physics.md "Against a building", step 7).
    /// A ball that starts inside an object's box gives no usable contact;
    /// what the game does then was not traced, so it counts as no hit (he
    /// can walk out) and is noted.
    fn sweep(&self, balls: &BallStack, from: [f32; 3], to: [f32; 3]) -> Option<Hit> {
        let (lo, hi) = balls.reach(from, to);
        let mut best: Option<Hit> = None;
        // Q88 stand-in: on an exact tie a pushable's contact is taken first.
        let pushable = |name: &str| self.push_target(name).is_some();
        let mut keep = |c: crate::collide::Contact, what: &str| {
            let d = length(sub(c.position, from));
            let better = best.as_ref().is_none_or(|h| {
                let old = length(sub(h.position, from));
                d < old - TIE || ((d - old).abs() <= TIE && pushable(what) && !pushable(&h.what))
            });
            if better {
                best = Some(Hit { position: c.position, touch: c.touch, normal: c.normal, what: what.to_string() });
            }
        };
        for b in self.buildings.iter().filter(|b| b.overlaps(lo, hi)) {
            if let Some(c) = b.sweep(balls, from, to) {
                keep(c, &b.name);
            }
        }
        let passed = self.passed.borrow();
        for o in self.objects.iter().filter(|o| o.overlaps(balls.radius, lo, hi) && !passed.contains(&o.name)) {
            match o.sweep(balls, from, to) {
                Some(ObjectHit::Contact(c)) => keep(c, &o.name),
                Some(ObjectHit::StartedInside { ball }) => self.notes.borrow_mut().push(format!(
                    "ball {ball} started inside {}'s box: no usable contact, not counted as a hit (not traced)",
                    o.name
                )),
                None => {}
            }
        }
        best
    }

    fn take_notes(&self) -> Vec<String> {
        std::mem::take(&mut *self.notes.borrow_mut())
    }

    fn pass_through(&self, name: &str) {
        self.passed.borrow_mut().push(name.to_string());
    }

    fn push_target(&self, name: &str) -> Option<PushTarget> {
        if self.not_pushable.iter().any(|n| n == name) {
            return None;
        }
        let o = self.objects.iter().find(|o| o.name == name && o.kind.collision_type & PUSHABLE != 0)?;
        Some(PushTarget { kind: o.kind, position: o.placement.position })
    }

    /// `pushing.md` rule 4.1: the object's own sweep over the step,
    /// against the buildings and the objects its mask makes obstacles
    /// (Connor is not in this list, so he is ignored as the rule says);
    /// it stops at the first contact (Q87 b stand-in). Then its ground
    /// rule (Q87 c stand-in: exactly on the floor grid); any change of
    /// height puts it back (rule 4.2). Only ball-stack objects: a box
    /// object's own sweep is not described (Q87 a).
    fn push_object(&self, name: &str, step: [f32; 3]) -> ([f32; 3], String) {
        let Some(o) = self.objects.iter().find(|o| o.name == name) else {
            return ([0.0; 3], "no such collider".into());
        };
        if !o.kind.ball_stack {
            return ([0.0; 3], "box-shaped: its own sweep is not described (Q87 a); not moved".into());
        }
        let balls = BallStack::from_box(o.min, o.max);
        let from = o.placement.position;
        let to = [from[0] + step[0], from[1] + step[1], from[2]];
        let (lo, hi) = balls.reach(from, to);
        let mut best: Option<(crate::collide::Contact, String)> = None;
        let mut keep = |c: crate::collide::Contact, what: &str| {
            if best.as_ref().is_none_or(|(b, _)| length(sub(c.position, from)) < length(sub(b.position, from))) {
                best = Some((c, what.to_string()));
            }
        };
        // Buildings are type 0x28 (or 0x88): obstacles if its mask has 0x8.
        if o.kind.collision_mask & 0x8 != 0 {
            for b in self.buildings.iter().filter(|b| b.overlaps(lo, hi)) {
                if let Some(c) = b.sweep(&balls, from, to) {
                    keep(c, &b.name);
                }
            }
        }
        let obstacle = |other: &BoxCollider| o.kind.collision_mask & other.kind.collision_type & OBSTACLE_BITS != 0;
        for other in self.objects.iter().filter(|x| x.name != o.name && obstacle(x) && x.overlaps(balls.radius, lo, hi)) {
            if let Some(ObjectHit::Contact(c)) = other.sweep(&balls, from, to) {
                keep(c, &other.name);
            }
        }
        let (mut end, mut why) = match &best {
            Some((c, what)) => (c.position, format!("stopped by {what}")),
            None => (to, "free".to_string()),
        };
        if o.kind.keep_on_ground {
            if let Some((h, _)) = self.ground.ground(end[0], end[1]) {
                end[2] = h;
            }
        }
        if end[2] != from[2] {
            return ([0.0; 3], format!("{why}; height changed by {:.3}: put back (rule 4.2)", end[2] - from[2]));
        }
        let moved = sub(end, from);
        if moved.iter().all(|v| *v == 0.0) {
            why.push_str("; did not move");
        } else {
            self.moved.borrow_mut().push((o.name.clone(), moved));
        }
        (moved, why)
    }
}

/// What the walker asks of one update.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Wish {
    /// The movement (x, y, z change), world space.
    pub step: [f32; 3],
    /// Connor is in the Move state (walking, running, stepping back,
    /// turning on the spot) or jumping (not `backJump`): only then can a
    /// hit be head-on.
    pub moving: bool,
    /// Jumping (not `backJump`): any surface with normal z < 0.8 also
    /// counts as head-on (connor-physics.md "Walls").
    pub steep_head_on: bool,
    /// What he brings to a push (`pushing.md` rule 2).
    pub push: PushAbility,
}

impl From<[f32; 3]> for Wish {
    fn from(step: [f32; 3]) -> Self {
        Wish { step, moving: true, steep_head_on: false, push: PushAbility::default() }
    }
}

/// Step up and step down for Connor (monster/Connor default, traced).
pub const STEP: f32 = 325.0;
/// A surface is walkable if its normal's z is above this (about 36.9°).
pub const WALKABLE_NORMAL_Z: f32 = 0.8;
/// Gravity in units per second squared (with the fall timer's seconds):
/// the drop `GRAVITY × (t² − t_prev²) / 2` is the note's measured
/// `10000 × (t² − t_prev²)`. The traced value was 5,000; the original
/// falls 4 times as far (measured 2026-10-12, Q30).
pub const GRAVITY: f32 = 20000.0;
/// The fall timer grows by this much per update, whatever the real time.
pub const FALL_TIMER_STEP: f32 = 0.06;
/// At most this much drop in one update.
pub const MAX_DROP: f32 = 5000.0;
/// A pull less steep than this below horizontal (sin 30°) gives no drop
/// and puts the fall timer back to 0.
pub const MIN_PULL_DOWN: f32 = 0.5;
/// Updates in the air before the Fall state starts.
pub const AIR_UPDATES_TO_FALL: u32 = 5;
/// The fall timer when the Fall state starts **after a jump** (previous
/// 0.30, so the first drop is 396). From walking or standing, Fall leaves
/// the timer running on.
pub const FALL_START_TIMER: f32 = 0.36;
/// After a jump the death limit is measured from this far below the
/// height where the jump animation ended (connor-physics.md "Jumping"
/// rule 4): death if he lands more than 3,840 + 1,280 = 5,120 lower.
pub const JUMP_DEATH_OFFSET: f32 = 1280.0;
/// A jump can start this high above the terrain (connor-physics.md
/// "Jumping" rule 1).
pub const JUMP_ABOVE_TERRAIN: f32 = 250.0;
/// In Fall, dropping less than this in one update counts as "stuck".
pub const STUCK_DROP: f32 = 50.0;
/// More than this many stuck updates ends the fall like a landing.
pub const STUCK_UPDATES: u32 = 5;
/// Falling further than this kills Connor.
pub const DEATH_DROP: f32 = 3840.0;
/// On this many hits in one update he stays at the last contact.
pub const MAX_HITS: u32 = 3;
/// A hit is head-on when his horizontal movement is less than this
/// (0.35 rad, about 20°) from the reversed surface direction.
pub const HEAD_ON_ANGLE: f32 = 0.35;
/// No slide when the rest of the move is shorter than this in every axis.
pub const SLIDE_MIN: f32 = 0.000001;
/// Connor's collision box, `GAME/patch/BOUNDS.KQ` entry `Connor` (the
/// viewer reads it from the file and logs whether it differs).
pub const CONNOR_BOX: ([f32; 3], [f32; 3]) = ([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// On the ground or moving normally (the original's Wait/Move).
    Normal,
    /// Jumping: gravity off, steps 0, the arc is the animation's own steps.
    Jump,
    /// The Fall state: `start_z` is the height when it started.
    Fall { start_z: f32, stuck: u32 },
    /// Died from a fall; the mover no longer updates.
    Dead,
}

impl State {
    pub fn name(&self) -> &'static str {
        match self {
            State::Normal => "normal",
            State::Jump => "jump",
            State::Fall { .. } => "fall",
            State::Dead => "dead",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mover {
    pub position: [f32; 3],
    pub on_ground: bool,
    pub use_gravity: bool,
    pub keep_on_ground: bool,
    /// Seconds of falling so far (reset when the down pass lands him on
    /// walkable ground).
    pub fall_timer: f32,
    /// Updates spent in the air with gravity on.
    pub air_updates: u32,
    /// Direction gravity pulls: straight down, or down a slope.
    pub pull: [f32; 3],
    pub step_up: f32,
    pub step_down: f32,
    pub state: State,
    /// This update: steep surfaces count as head-on (jumping).
    steep_head_on: bool,
    /// This update: what he brings to a push, and whether he began it on
    /// the ground (rule 2.2).
    push: PushAbility,
    push_ground: bool,
    /// The balls he collides with (built from his box).
    pub balls: BallStack,
    /// The water test's result after the last move (water.md).
    pub submerged: bool,
}

/// What the down pass ended on.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DownResult {
    /// Nothing: he is in the air (or the ground check catches him).
    #[default]
    None,
    /// Landed on walkable terrain.
    Walkable,
    /// Hit steep terrain and was pushed off down the slope.
    Steep,
}

impl DownResult {
    pub fn name(&self) -> &'static str {
        match self {
            DownResult::None => "none",
            DownResult::Walkable => "walkable",
            DownResult::Steep => "steep",
        }
    }
}

/// A change of the Fall state, for the animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FallEvent {
    Started,
    /// Safe landing after falling this far.
    Landed(f32),
    /// Fatal landing after falling this far.
    Died(f32),
}

/// What one update did, for the log.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub drop: f32,
    /// He began the update on walkable ground (logged only).
    pub began_on_ground: bool,
    /// The up pass hit the terrain: its normal.
    pub up_contact: Option<[f32; 3]>,
    pub down: DownResult,
    /// Normal of the down pass's contact.
    pub down_normal: Option<[f32; 3]>,
    /// How far the push-off moved him along the surface.
    pub push: f32,
    /// What the ground check did: `above`, `walkable`, `steep` or `none`
    /// (no ground, or keepOnGround off).
    pub check: &'static str,
    /// Ground height and normal under the final position, if any.
    pub ground: Option<(f32, [f32; 3])>,
    pub fall: Option<FallEvent>,
    /// Every hit this update, in order.
    pub hits: Vec<HitRecord>,
    /// What the across pass did: `free`, `glance`, `head-on` or `3 hits`.
    pub across: &'static str,
    /// A hit was head-on while he was moving: he stops (the walker plays
    /// `Stop`).
    pub head_on: bool,
    /// While jumping: the pass (`up`, `across`, `slide`, `down`) of the
    /// first hit on a surface with normal z < 0.8, which ends the jump
    /// (connor-physics.md "Jumping" rule 5).
    pub jump_hit: Option<&'static str>,
    /// Things worth a line in the main log (state changes, landings).
    pub events: Vec<String>,
    /// Every touch on a pushable this update (`pushing.md`).
    pub pushes: Vec<PushTouch>,
}

/// One hit, for the log.
#[derive(Debug, Clone, PartialEq)]
pub struct HitRecord {
    /// `up`, `across`, `down` or `slide`.
    pub pass: &'static str,
    pub hit: Hit,
    /// Angle between his horizontal movement and the reversed surface
    /// direction; `None` without horizontal movement.
    pub angle: Option<f32>,
    /// What was done: `stop`, `head-on`, `landing`, `glance`, `3 hits`.
    pub result: &'static str,
}

impl Mover {
    /// Connor as created: on the ground at `position`, gravity as given.
    pub fn new(position: [f32; 3], use_gravity: bool) -> Self {
        Mover {
            position,
            on_ground: true,
            use_gravity,
            keep_on_ground: true,
            fall_timer: 0.0,
            air_updates: 0,
            pull: [0.0, 0.0, -1.0],
            step_up: STEP,
            step_down: STEP,
            state: State::Normal,
            steep_head_on: false,
            push: PushAbility::default(),
            push_ground: false,
            balls: BallStack::from_box(CONNOR_BOX.0, CONNOR_BOX.1),
            submerged: false,
        }
    }

    /// Test helper: lift Connor by `height` (a dead Connor comes back to
    /// life, so a fall can be tried again). "On ground" is kept: the update
    /// before the lift ended where it ended, so the first update after it
    /// counts in the air only if that one ended off the ground (Q29).
    pub fn lift(&mut self, height: f32) {
        self.position[2] += height;
        if self.state == State::Jump {
            // Lifted mid-jump: the jump is over (gravity on, steps back).
            self.use_gravity = true;
            self.step_up = STEP;
            self.step_down = STEP;
        }
        self.state = State::Normal;
    }

    /// A jump starts (connor-physics.md "Jumping" rule 3): gravity off and
    /// step up and down 0, so the jump animation's own steps make the arc.
    pub fn start_jump(&mut self) {
        self.state = State::Jump;
        self.use_gravity = false;
        self.step_up = 0.0;
        self.step_down = 0.0;
    }

    /// A steep hit ended the jump and he crashes or stands (rule 5): gravity
    /// on, steps back to 325, the normal state.
    pub fn end_jump_on_hit(&mut self) {
        self.use_gravity = true;
        self.step_up = STEP;
        self.step_down = STEP;
        self.state = State::Normal;
    }

    /// A steep hit during `backJump` (rule 5: the jump ends, gravity on,
    /// steps 325), but the animation plays on (the user's check in the
    /// original), so the state stays Jump: no Fall request from the air
    /// counter until it ends.
    pub fn back_jump_hit(&mut self) {
        self.use_gravity = true;
        self.step_up = STEP;
        self.step_down = STEP;
    }

    /// The jump animation has ended (rule 4): gravity on, steps back to
    /// 325, and the Fall state, with the death limit measured from 1,280
    /// below here and the fall timer at 0.36 (previous 0.30): the next down
    /// pass drops 396.
    pub fn end_jump(&mut self) {
        self.use_gravity = true;
        self.step_up = STEP;
        self.step_down = STEP;
        self.fall_timer = FALL_START_TIMER - FALL_TIMER_STEP;
        self.state = State::Fall { start_z: self.position[2] - JUMP_DEATH_OFFSET, stuck: 0 };
    }

    /// One update (1/15 s) with a fixed wanted movement (x, y, z change);
    /// zero while standing. Counts as moving (the Move state).
    pub fn update(&mut self, wanted: [f32; 3], world: &dyn MoverWorld) -> Report {
        self.update_with(world, |_, _| wanted)
    }

    /// One update (1/15 s). `wanted` gives the movement the animation asks
    /// for (a plain step counts as moving). It is called after the start of
    /// the update, so it already knows whether Fall started this update
    /// (then the fall animation's step is used, not the walk's). Not called
    /// when Connor is dead.
    pub fn update_with<W: Into<Wish>>(
        &mut self,
        world: &dyn MoverWorld,
        wanted: impl FnOnce(&Mover, Option<FallEvent>) -> W,
    ) -> Report {
        let mut report = Report { check: "none", across: "none", ..Default::default() };
        if self.state == State::Dead {
            report.ground = world.ground(self.position[0], self.position[1]);
            return report;
        }
        let before_z = self.position[2];
        report.began_on_ground = self.on_ground;

        // 1. Start of the update.
        if self.on_ground || !self.use_gravity {
            self.on_ground = false;
            self.pull = [0.0, 0.0, -1.0];
            self.air_updates = 0;
        } else {
            self.air_updates += 1;
            // From 5 on, every update asks for Fall; normal play accepts.
            // Not after a jump (none yet), so gravity is left alone.
            if self.air_updates >= AIR_UPDATES_TO_FALL && self.state == State::Normal {
                self.state = State::Fall { start_z: self.position[2], stuck: 0 };
                report.fall = Some(FallEvent::Started);
                report.events.push(format!(
                    "fall starts at z={:.0} after {} updates in the air (fall timer runs on from {:.2} s)",
                    self.position[2], self.air_updates, self.fall_timer
                ));
            }
        }

        let wish: Wish = wanted(self, report.fall).into();
        let wanted = wish.step;
        // Head-on only acts in the Move state (Fall is not Move).
        // Head-on also acts while jumping (state Jump), where a steep
        // surface counts too.
        let head_on_allowed = wish.moving && matches!(self.state, State::Normal | State::Jump);
        self.steep_head_on = head_on_allowed && wish.steep_head_on && self.state == State::Jump;
        self.push = wish.push;
        // Rule 2.2: not jumping or falling ("on the ground" when the
        // update began), or already pushing.
        self.push_ground = (self.state == State::Normal && report.began_on_ground) || wish.push.pushing;
        let mut hits = 0;

        // 2. Up pass: step up plus the wanted height change. A building
        //    (ceiling) or the terrain above where the pass ends stops it;
        //    the nearer contact wins. Up-pass hits never slide.
        let start = self.position;
        let end = add(start, [0.0, 0.0, self.step_up + wanted[2]]);
        match nearest(start, world.sweep(&self.balls, start, end), terrain_hit(world, end)) {
            Some(h) => {
                hits += 1;
                self.position = h.position;
                report.up_contact = Some(h.normal);
                // A vertical move is never head-on by angle; while jumping a
                // steep surface (a ceiling too) is.
                let head_on = self.steep_head_on && h.normal[2] < WALKABLE_NORMAL_Z;
                report.head_on |= head_on;
                report.hits.push(HitRecord { pass: "up", hit: h, angle: None, result: if head_on { "head-on" } else { "stop" } });
            }
            None => self.position = end,
        }

        // 3. Across pass: buildings only, the terrain is not tested (Q26).
        let start = self.position;
        let target = [start[0] + wanted[0], start[1] + wanted[1], start[2]];
        report.across = self.across(world, start, target, head_on_allowed, "across", &mut hits, &mut report);

        // 4. Down pass: this update's gravity drop along the pull, then
        //    step down.
        if self.use_gravity {
            if -self.pull[2] < MIN_PULL_DOWN {
                self.fall_timer = 0.0;
            } else {
                let t_prev = self.fall_timer;
                self.fall_timer += FALL_TIMER_STEP;
                let t = self.fall_timer;
                report.drop = (GRAVITY * (t * t - t_prev * t_prev) / 2.0).min(MAX_DROP);
            }
        }
        // The vertical drop projected onto the pull (Q28 a): d along a
        // straight-down pull, d·sinθ along a slope of angle θ.
        let start = self.position;
        let mut end = add(start, scale(self.pull, report.drop * -self.pull[2]));
        end[2] -= self.step_down;
        self.position = end;
        // Not hit: he is in the air. Going down a steep walkable slope that
        // is often for an update or a few; the gravity brings him back
        // before the air count reaches 5 (measured, Q32).
        if let Some(h) = nearest(start, world.sweep(&self.balls, start, end), terrain_hit(world, end)) {
            hits += 1;
            let normal = h.normal;
            report.down_normal = Some(normal);
            self.pull = down_slope(normal);
            let angle = head_on_angle(sub(end, start), normal);
            if normal[2] > WALKABLE_NORMAL_Z {
                report.down = DownResult::Walkable;
                self.on_ground = true;
                self.fall_timer = 0.0;
            } else {
                report.down = DownResult::Steep;
                if !self.use_gravity {
                    report.events.push("steep ground: gravity switched on".into());
                }
                self.use_gravity = true;
            }
            let head_on = head_on_allowed && angle.is_some_and(|a| a < HEAD_ON_ANGLE);
            // "Exact hit handling": walkable landings (surface z ≥ 0.8),
            // head-on hits and the last allowed hit stay at the contact;
            // anything else is pushed off along the surface.
            let result = if normal[2] >= WALKABLE_NORMAL_Z {
                "landing"
            } else if head_on {
                "head-on"
            } else if hits >= MAX_HITS {
                "3 hits"
            } else {
                "glance"
            };
            let contact = h.position;
            report.head_on |= head_on;
            report.hits.push(HitRecord { pass: "down", hit: h, angle, result });
            self.position = contact;
            if result == "glance" {
                let target = project_onto(end, contact, normal);
                report.push = length(sub(target, contact));
                self.across(world, contact, target, head_on_allowed, "down slide", &mut hits, &mut report);
            }
        }

        // 5. Ground check (keepOnGround): at or below the ground -> on it.
        //    Only the height changes; the fall timer is left alone. The
        //    terrain only, as the note says.
        let under = world.ground(self.position[0], self.position[1]);
        report.ground = under;
        if let (true, Some((height, normal))) = (self.keep_on_ground, under) {
            if self.position[2] <= height {
                self.position[2] = height;
                if normal[2] > WALKABLE_NORMAL_Z {
                    report.check = "walkable";
                    self.on_ground = true;
                } else {
                    report.check = "steep";
                    self.on_ground = false;
                    self.pull = down_slope(normal);
                    if !self.use_gravity {
                        report.events.push("steep ground: gravity switched on".into());
                    }
                    self.use_gravity = true;
                }
            } else {
                report.check = "above";
            }
        }

        // The Fall state (connor-physics.md "Falling off an edge").
        if let State::Fall { start_z, stuck } = self.state {
            let z = self.position[2];
            let stuck = if before_z - z < STUCK_DROP { stuck + 1 } else { 0 };
            let ended = if self.on_ground {
                Some("landed")
            } else if stuck > STUCK_UPDATES {
                Some("stuck for more than 5 updates (counts as landed)")
            } else if z > before_z {
                Some("moved up")
            } else {
                None
            };
            match ended {
                Some(why) => {
                    let fallen = start_z - z;
                    if fallen > DEATH_DROP {
                        self.state = State::Dead;
                        report.fall = Some(FallEvent::Died(fallen));
                        report.events.push(format!(
                            "fall ends ({why}) {fallen:.0} below the death limit's start z={start_z:.0} (after a jump 1,280 under where it ended): more than {DEATH_DROP}, Connor dies"
                        ));
                    } else {
                        self.state = State::Normal;
                        report.fall = Some(FallEvent::Landed(fallen));
                        report.events.push(format!(
                            "fall ends ({why}) {fallen:.0} below the death limit's start z={start_z:.0} (after a jump 1,280 under where it ended): safe landing"
                        ));
                    }
                }
                None => self.state = State::Fall { start_z, stuck },
            }
        }
        if self.state == State::Jump {
            report.jump_hit = report.hits.iter().find(|h| h.hit.normal[2] < WALKABLE_NORMAL_Z).map(|h| h.pass);
        }
        report.events.extend(world.take_notes());
        report
    }

    /// An across move from `start` to `target` against the buildings (the
    /// terrain is ignored), with "Exact hit handling": a head-on hit (in the
    /// Move state) stops him at the contact; otherwise the rest of the move
    /// is projected onto the surface and tried again from the contact; on
    /// the 3rd hit of the update he stays at that contact. Returns what
    /// happened: `free`, `glance`, `head-on` or `3 hits`.
    #[allow(clippy::too_many_arguments)]
    fn across(
        &mut self,
        world: &dyn MoverWorld,
        mut start: [f32; 3],
        mut target: [f32; 3],
        head_on_allowed: bool,
        mut pass: &'static str,
        hits: &mut u32,
        report: &mut Report,
    ) -> &'static str {
        let mut outcome = "free";
        let pass_start = start;
        // Rule 2.4: the across pass and its slides, not the down pass's.
        let across_pass = pass == "across";
        if pass == "down slide" {
            pass = "slide";
        }
        loop {
            let Some(h) = world.sweep(&self.balls, start, target) else {
                self.position = target;
                return outcome;
            };
            if let Some(t) = world.push_target(&h.what) {
                let mut touch = self.push_check(&h, &t, across_pass);
                if touch.result == PushResult::Push {
                    // Rule 3: not a hit for him.
                    if !t.scripted() {
                        // Rule 4: the object is moved by his horizontal
                        // step of this pass; he follows by 0.95 of what it
                        // really moved, from the start of the pass.
                        let step = [target[0] - pass_start[0], target[1] - pass_start[1], 0.0];
                        let moved = if step.iter().any(|v| v.abs() > PUSH_MIN_STEP) {
                            let (moved, why) = world.push_object(&h.what, step);
                            touch.moved = Some((step, moved, why));
                            moved
                        } else {
                            [0.0; 3]
                        };
                        self.position = add(pass_start, scale(moved, PUSH_FOLLOW));
                    } else {
                        // Rule 3: the object's script gets `Push` (the
                        // caller runs it after this move); he is not
                        // stopped: the rest of the pass goes on through it.
                        world.pass_through(&h.what);
                        report.pushes.push(touch);
                        outcome = "push";
                        continue;
                    }
                    report.pushes.push(touch);
                    return "push";
                }
                report.pushes.push(touch);
            }
            *hits += 1;
            let angle = head_on_angle(sub(target, start), h.normal);
            let head_on = head_on_allowed
                && (angle.is_some_and(|a| a < HEAD_ON_ANGLE) || (self.steep_head_on && h.normal[2] < WALKABLE_NORMAL_Z));
            let (contact, normal) = (h.position, h.normal);
            let result = if head_on {
                "head-on"
            } else if *hits >= MAX_HITS {
                "3 hits"
            } else {
                "glance"
            };
            report.hits.push(HitRecord { pass, hit: h, angle, result });
            self.position = contact;
            if result != "glance" {
                report.head_on |= head_on;
                return result;
            }
            outcome = "glance";
            if sub(target, contact).iter().all(|v| v.abs() < SLIDE_MIN) {
                return outcome;
            }
            target = project_onto(target, contact, normal);
            start = contact;
            pass = "slide";
        }
    }
}

impl Mover {
    /// The push check of `pushing.md` rule 2 for a hit `h` on pushable
    /// `t` (rule 2.1, Connor moving, holds: this is his move).
    fn push_check(&self, h: &Hit, t: &PushTarget, across_pass: bool) -> PushTouch {
        let p = self.push;
        let flat = |v: [f32; 2]| {
            let l = (v[0] * v[0] + v[1] * v[1]).sqrt();
            if l > 0.0 { [v[0] / l, v[1] / l] } else { [0.0, 0.0] }
        };
        // Rule 2.5: into the surface (scripted), or towards the object.
        let towards = if t.scripted() {
            flat([-h.normal[0], -h.normal[1]])
        } else {
            flat([t.position[0] - self.position[0], t.position[1] - self.position[1]])
        };
        let cos = p.facing[0] * towards[0] + p.facing[1] * towards[1];
        let mut touch = PushTouch {
            object: h.what.clone(),
            scripted: t.scripted(),
            on_ground: self.push_ground,
            empty_hands: p.empty_hands,
            across: across_pass,
            cos,
            movement_ok: p.movement_ok,
            accepts: p.accepts,
            result: PushResult::Wall,
            moved: None,
        };
        if !(touch.on_ground && touch.empty_hands && touch.across) {
            return touch;
        }
        if cos < PUSH_FACING_COS {
            touch.result = if p.pushing { PushResult::WallBackToMove } else { PushResult::Wall };
            return touch;
        }
        if !p.movement_ok {
            touch.result = PushResult::WallStand;
            return touch;
        }
        if p.accepts {
            touch.result = PushResult::Push;
        }
        touch
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + b[i])
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    std::array::from_fn(|i| a[i] - b[i])
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    a.map(|v| v * s)
}

/// `target` moved onto the plane through `contact` with unit normal `n`:
/// target − n × ((target − contact) · n) ("Exact hit handling").
fn project_onto(target: [f32; 3], contact: [f32; 3], n: [f32; 3]) -> [f32; 3] {
    sub(target, scale(n, dot(sub(target, contact), n)))
}

/// The angle between the horizontal part of `movement` (made unit length)
/// and the reversed surface direction; `None` without horizontal movement
/// (never head-on).
fn head_on_angle(movement: [f32; 3], normal: [f32; 3]) -> Option<f32> {
    let h = [movement[0], movement[1], 0.0];
    let len = length(h);
    if len == 0.0 {
        return None;
    }
    Some((-dot(h, normal) / len).clamp(-1.0, 1.0).acos())
}

/// The terrain as a hit at the end of an up or down pass: if it is above
/// where the pass ended, the contact is (end x, end y, terrain height),
/// with no pull-back (connor-physics.md "The terrain in the passes").
fn terrain_hit(world: &dyn MoverWorld, end: [f32; 3]) -> Option<Hit> {
    let (height, normal) = world.ground(end[0], end[1]).filter(|(h, _)| *h > end[2])?;
    let p = [end[0], end[1], height];
    Some(Hit { position: p, touch: p, normal, what: "terrain".into() })
}

/// Of a building contact and a terrain contact, the one nearer the start
/// of the pass.
fn nearest(start: [f32; 3], a: Option<Hit>, b: Option<Hit>) -> Option<Hit> {
    match (a, b) {
        (Some(a), Some(b)) => Some(if length(sub(a.position, start)) <= length(sub(b.position, start)) { a } else { b }),
        (a, b) => a.or(b),
    }
}

/// `v` with the part along `normal` removed: what is left of a move after
/// hitting a surface.
fn slide_along(v: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = dot(v, normal);
    std::array::from_fn(|i| v[i] - d * normal[i])
}

/// The direction straight down, flattened onto a surface with `normal`.
fn down_slope(normal: [f32; 3]) -> [f32; 3] {
    let d = slide_along([0.0, 0.0, -1.0], normal);
    let len = length(d);
    if len < 1e-6 { [0.0, 0.0, -1.0] } else { d.map(|v| v / len) }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collide::BoxCollider;

    /// Flat ground at a fixed height everywhere.
    struct Flat(f32);
    impl GroundLookup for Flat {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((self.0, [0.0, 0.0, 1.0]))
        }
    }

    /// Flat at height 0 for x < 0; from x = 0 a plane rising (positive
    /// angle) or falling (negative) towards +x at `angle` radians.
    struct Ramp(f32);
    impl GroundLookup for Ramp {
        fn ground(&self, x: f32, _: f32) -> Option<(f32, [f32; 3])> {
            if x < 0.0 {
                Some((0.0, [0.0, 0.0, 1.0]))
            } else {
                let (s, c) = self.0.sin_cos();
                Some((x * self.0.tan(), [-s, 0.0, c]))
            }
        }
    }

    /// Height 0 for x < 0, `-drop` from x = 0 on.
    struct Ledge(f32);
    impl GroundLookup for Ledge {
        fn ground(&self, x: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((if x < 0.0 { 0.0 } else { -self.0 }, [0.0, 0.0, 1.0]))
        }
    }

    /// Updates until the state settles; returns (updates, events).
    fn run(m: &mut Mover, ground: &dyn MoverWorld, max: usize) -> (usize, Vec<String>) {
        let mut events = Vec::new();
        for n in 1..=max {
            let r = m.update([0.0; 3], ground);
            events.extend(r.events);
            if m.on_ground && m.state != State::Normal || m.state == State::Dead || (m.on_ground && n > 1) {
                return (n, events);
            }
        }
        (max, events)
    }

    #[test]
    fn standing_on_flat_ground_stays_put() {
        let mut m = Mover::new([0.0, 0.0, 100.0], true);
        for _ in 0..30 {
            let r = m.update([0.0; 3], &Flat(100.0));
            assert!(r.events.is_empty());
            assert_eq!(r.down, DownResult::Walkable);
            assert_eq!(m.position[2], 100.0);
            assert!(m.on_ground);
            assert_eq!((m.fall_timer, m.air_updates), (0.0, 0));
        }
    }

    #[test]
    fn gravity_drop_follows_the_fall_timer() {
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        m.lift(100_000.0);
        let mut drops = Vec::new();
        for _ in 0..3 {
            drops.push(m.update([0.0; 3], &Flat(0.0)).drop);
        }
        // t = 0.06, 0.12, 0.18: 10000 × (t² − t_prev²).
        let want = [36.0, 108.0, 180.0];
        for (d, w) in drops.iter().zip(want) {
            assert!((d - w).abs() < 0.05, "{drops:?}");
        }
        // Up 325 and down 325 cancel: only gravity moves him.
        assert!((m.position[2] - (100_000.0 - 324.0)).abs() < 0.1);
    }

    #[test]
    fn short_fall_is_safe_long_fall_kills() {
        let mut safe = Mover::new([0.0, 0.0, 0.0], true);
        safe.lift(2000.0);
        let (_, events) = run(&mut safe, &Flat(0.0), 200);
        assert_eq!(safe.state, State::Normal, "{events:?}");
        assert!(events.iter().any(|e| e.contains("safe landing")), "{events:?}");
        assert_eq!(safe.position[2], 0.0);

        let mut dead = Mover::new([0.0, 0.0, 0.0], true);
        dead.lift(5000.0);
        let (_, events) = run(&mut dead, &Flat(0.0), 200);
        assert_eq!(dead.state, State::Dead, "{events:?}");
        assert!(events.iter().any(|e| e.contains("Connor dies")), "{events:?}");
    }

    #[test]
    fn fall_starts_after_5_updates_in_the_air() {
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        m.lift(50_000.0);
        let mut first_fall = None;
        for n in 1..=10 {
            let r = m.update([0.0; 3], &Flat(0.0));
            if r.fall == Some(FallEvent::Started) {
                assert!(first_fall.is_none(), "asked for once, accepted once");
                first_fall = Some(n);
            }
        }
        // Updates 1..5 end in the air; Fall is asked for at the start of
        // update 6 (Q29: after 5 updates in a row ending off the ground).
        assert_eq!(first_fall, Some(6));
    }

    /// Walking off a ledge: the drops carry on through the Fall start
    /// (36 … 324 in the air, then 396, 468 in Fall), with no head start.
    #[test]
    fn walking_off_a_ledge_keeps_the_fall_timer() {
        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        let mut drops = Vec::new();
        let mut fall_at = None;
        for n in 1..=12 {
            let r = m.update_with(&Ledge(50_000.0), |m, _| if m.state == State::Normal { [112.0, 0.0, 0.0] } else { [0.0; 3] });
            if r.fall == Some(FallEvent::Started) {
                fall_at = Some(n);
            }
            if r.drop > 0.0 && !m.on_ground {
                drops.push(r.drop);
            }
        }
        assert!(fall_at.is_some(), "{drops:?}");
        let want = [36.0, 108.0, 180.0, 252.0, 324.0, 396.0, 468.0];
        for (d, w) in drops.iter().zip(want) {
            assert!((d - w).abs() < 0.1, "{drops:?}");
        }
    }

    #[test]
    fn down_slope_points_downhill() {
        // A slope rising towards +x: normal leans to -x.
        let n = [-0.6, 0.0, 0.8];
        let d = down_slope(n);
        assert!(d[0] < 0.0 && d[2] < 0.0, "{d:?}");
        assert!(dot(d, n).abs() < 1e-6, "lies in the surface");
    }

    /// Running (168 per update along +x) at a 45° hill from its foot: each
    /// update follows the note's rule (Q28 a), 168·cos²θ − d·sinθ·cosθ
    /// with d = 36, 108, 180, 252, 324, so he gains 66, then 30, then
    /// slides back. In update 5 the push-off carries him past the foot and
    /// the ground check puts him on the flat: only 4 updates ended in the
    /// air, so no Fall.
    #[test]
    fn running_into_a_steep_hill_slides_back() {
        let hill = Ramp(45f32.to_radians());
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        let mut xs = vec![m.position[0]];
        let mut events = Vec::new();
        for n in 1..=20 {
            let r = m.update_with(&hill, |m, _| if m.state == State::Normal { [168.0, 0.0, 0.0] } else { [0.0; 3] });
            events.extend(r.events);
            assert_ne!(r.fall, Some(FallEvent::Started), "update {n}: {events:?}");
            if r.down == DownResult::Steep {
                assert!(r.push > 0.0);
                // Only the ground check over the flat puts him on the ground.
                assert_eq!(m.on_ground, r.check == "walkable", "update {n}");
            }
            xs.push(m.position[0]);
            if m.on_ground && m.position[0] < 0.0 {
                break;
            }
        }
        let gains: Vec<f32> = xs.windows(2).map(|w| w[1] - w[0]).collect();
        let want = [66.0, 30.0, -6.0, -42.0];
        assert!(gains.len() >= want.len(), "{gains:?}");
        for (g, w) in gains.iter().zip(want) {
            assert!((g - w).abs() < 0.5, "{gains:?}");
        }
        // Back on the flat, safe, after 5 updates.
        assert_eq!(gains.len(), 5, "{xs:?}");
        assert!(m.position[0] < 0.0 && m.on_ground && m.position[2] == 0.0, "{xs:?}");
        assert_eq!(m.state, State::Normal, "{events:?}");
    }

    /// Walking (75 per update) and running (168) down walkable planes,
    /// even the steepest: off the ground for a few updates at a time, but
    /// never 5 in a row, so never Fall (the user's check in the original;
    /// measured there: off 1–4 updates at a time, Q32).
    #[test]
    fn running_down_walkable_slopes_never_falls() {
        for step in [75.0f32, 168.0] {
            for deg in [5.0f32, 20.0, 30.0, 36.0] {
                let slope = Ramp(-deg.to_radians());
                let mut m = Mover::new([0.0, 0.0, 0.0], true);
                let mut most_air = 0;
                for n in 0..100 {
                    m.update([step, 0.0, 0.0], &slope);
                    most_air = most_air.max(m.air_updates);
                    assert_eq!(m.state, State::Normal, "{step}, {deg}°, update {n}");
                }
                assert!(most_air < AIR_UPDATES_TO_FALL, "{step}, {deg}°: {most_air}");
                assert!(m.position[2] < -1000.0 * (step / 168.0), "{step}, {deg}°: he went down");
            }
        }
    }

    #[test]
    fn small_ledge_is_followed_big_ledge_kills() {
        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        for _ in 0..5 {
            m.update([112.0, 0.0, 0.0], &Ledge(300.0));
        }
        assert!(m.on_ground && m.position[2] == -300.0 && m.state == State::Normal);

        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        let mut events = Vec::new();
        for _ in 0..100 {
            let r = m.update_with(&Ledge(5000.0), |m, _| if m.state == State::Normal { [112.0, 0.0, 0.0] } else { [0.0; 3] });
            events.extend(r.events);
        }
        assert_eq!(m.state, State::Dead, "{events:?}");
        assert!(events.iter().any(|e| e.contains("fall starts")), "{events:?}");
    }

    #[test]
    fn ground_check_leaves_the_fall_timer_alone() {
        // Gravity off, in the air, ending exactly at ground height: the
        // down pass doesn't hit (not above), the ground check puts him on.
        let mut m = Mover::new([0.0, 0.0, 0.0], false);
        m.on_ground = false;
        m.fall_timer = 0.5;
        let r = m.update([0.0; 3], &Flat(0.0));
        assert_eq!((r.down, r.check), (DownResult::None, "walkable"));
        assert!(m.on_ground);
        assert_eq!(m.fall_timer, 0.5);
    }

    #[test]
    fn up_pass_stops_at_the_terrain() {
        let mut m = Mover::new([0.0, 0.0, -1000.0], true);
        let r = m.update([0.0; 3], &Flat(0.0));
        assert!(r.up_contact.is_some());
        assert_eq!(r.down, DownResult::Walkable);
        assert_eq!(m.position[2], 0.0);
    }

    // --- Buildings in the passes (M2 step 2d-3) ---

    use crate::collide::{BuildingCollider, Placement};
    use kq8_data::building::{BspChild::Leaf, BspChild::Node, BspNode};

    fn node(plane: u16, front: kq8_data::building::BspChild, back: kq8_data::building::BspChild) -> BspNode {
        BspNode { plane, front, back }
    }

    /// A made-up building from its tree.
    fn building(nodes: Vec<BspNode>, solid: Vec<bool>, planes: Vec<([f32; 3], f32)>) -> BuildingCollider {
        BuildingCollider::from_parts("test", nodes, solid, planes, ([-1e5; 3], [1e5; 3]), Placement::at([0.0; 3]))
    }

    /// A wall whose face is y = 0, solid for y < 0 (endless).
    fn wall() -> BuildingCollider {
        building(vec![node(0, Leaf(0), Leaf(1))], vec![false, true], vec![([0.0, 1.0, 0.0], 0.0)])
    }

    fn near(a: f32, b: f32, tol: f32) -> bool {
        (a - b).abs() <= tol
    }

    #[test]
    fn walking_head_on_stops_225_from_the_wall() {
        let b = [wall()];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([0.0, 600.0, 0.0], true);
        let mut stopped = None;
        for n in 1..=20 {
            let r = m.update([2.0, -72.0, 0.0], &scene);
            assert!(m.position[1] >= 199.0, "never inside the wall: {:?}", m.position);
            if r.head_on {
                assert_eq!(r.across, "head-on");
                stopped.get_or_insert((n, m.position));
            }
        }
        let (_, p) = stopped.expect("head-on");
        // The ball touches at y 200, pulled back 25 along the move (or he
        // stays where the step started, if that is nearer).
        assert!(p[1] >= 200.0 && p[1] <= 225.0 + 72.0, "{p:?}");
        assert_eq!(m.position[2], 0.0, "still on the ground");
        assert!(m.on_ground);
    }

    /// A made-up object box (like the signpost: −250…−10, 0…250, 0…2,420).
    fn post(position: [f32; 3]) -> BoxCollider {
        BoxCollider {
            name: "post".into(),
            min: [-250.0, 0.0, 0.0],
            max: [-10.0, 250.0, 2420.0],
            placement: crate::collide::Placement::at(position),
            kind: crate::collide::ColliderKind::default(),
        }
    }

    #[test]
    fn walking_head_on_into_an_object_stops_225_from_its_face() {
        let o = [post([0.0; 3])];
        let scene = Scene::new(&Flat(0.0), &[], &o);
        // From +y toward −y at the post's middle (x −130): its face is y 250.
        let mut m = Mover::new([-130.0, 1000.0, 0.0], true);
        let mut stopped = None;
        for _ in 1..=20 {
            let r = m.update([0.0, -72.0, 0.0], &scene);
            assert!(m.position[1] >= 449.0, "never inside: {:?}", m.position);
            if r.head_on {
                assert!(r.hits.iter().any(|h| h.hit.what == "post" && h.pass == "across"), "{:?}", r.hits);
                stopped.get_or_insert(m.position);
            }
        }
        let p = stopped.expect("head-on");
        assert!(p[1] >= 450.0 && p[1] <= 475.0 + 72.0, "{p:?}");
    }

    /// The post as a pushable: physical (0x101) or scripted (0x10121).
    fn pushable(scripted: bool) -> BoxCollider {
        let mut b = post([0.0; 3]);
        b.kind.collision_type = if scripted { 0x10121 } else { 0x101 };
        b
    }

    /// Walking towards −y (facing −y: heading 0) with what he brings.
    fn push_wish(facing: [f32; 2], movement_ok: bool, empty_hands: bool) -> Wish {
        Wish { step: [0.0, -72.0, 0.0], moving: true, steep_head_on: false, push: PushAbility { accepts: true, pushing: false, empty_hands, movement_ok, facing } }
    }

    #[test]
    fn walking_into_a_pushable_passes_the_push_check_and_stays_at_the_pass_start() {
        for scripted in [false, true] {
            let o = [pushable(scripted)];
            let scene = Scene::new(&Flat(0.0), &[], &o);
            let mut m = Mover::new([-130.0, 1000.0, 0.0], true);
            let mut pushed = None;
            for _ in 1..=20 {
                let before = m.position;
                let r = m.update_with(&scene, |_, _| push_wish([0.0, -1.0], true, true));
                if let Some(t) = r.pushes.first() {
                    assert_eq!((t.result, t.scripted), (PushResult::Push, scripted), "{}", t.describe());
                    assert!(t.cos > 0.95, "{}", t.describe());
                    // Not a hit for him: no head-on, no hit recorded.
                    assert!(!r.head_on && r.hits.iter().all(|h| h.hit.what != "post"), "{:?}", r.hits);
                    if scripted {
                        // Rule 3: he goes on through it this update.
                        assert!((m.position[1] - (before[1] - 72.0)).abs() < 1e-3, "full step: {:?}", m.position);
                        assert_eq!(scene.passed.take(), ["post"]);
                    } else {
                        // A box-shaped one doesn't move (Q87 a): he stays.
                        assert!((m.position[1] - before[1]).abs() < 1e-3, "stays at the pass start");
                    }
                    pushed = Some(m.position);
                    break;
                }
            }
            assert!(pushed.is_some(), "scripted {scripted}: no push touch");
        }
    }

    #[test]
    fn the_push_check_fails_facing_away_with_a_weapon_or_backing_up() {
        let o = [pushable(false)];
        let scene = Scene::new(&Flat(0.0), &[], &o);
        let first_touch = |wish: Wish| {
            let mut m = Mover::new([-130.0, 1000.0, 0.0], true);
            (0..20).find_map(|_| m.update_with(&scene, |_, _| wish).pushes.first().cloned()).expect("a touch")
        };
        // Facing 90 degrees off (cos 0): an ordinary wall hit.
        assert_eq!(first_touch(push_wish([1.0, 0.0], true, true)).result, PushResult::Wall);
        // A sword drawn: a wall.
        assert_eq!(first_touch(push_wish([0.0, -1.0], true, false)).result, PushResult::Wall);
        // Backing up or turning: a wall, and he stands (rule 2.6).
        assert_eq!(first_touch(push_wish([0.0, -1.0], false, true)).result, PushResult::WallStand);
        // Facing off while already pushing: back to Move (rule 2.5).
        let mut w = push_wish([1.0, 0.0], true, true);
        w.push.pushing = true;
        assert_eq!(first_touch(w).result, PushResult::WallBackToMove);
    }

    /// A hay bale like `haybail.kq`: a ball stack (box ±500, 0…1,000),
    /// type 0x101, mask 0xB.
    fn bale(position: [f32; 3], keep_on_ground: bool) -> BoxCollider {
        BoxCollider {
            name: "bale".into(),
            min: [-500.0, -500.0, 0.0],
            max: [500.0, 500.0, 1000.0],
            placement: crate::collide::Placement::at(position),
            kind: ColliderKind { collision_type: 0x101, collision_mask: 0xB, ball_stack: true, keep_on_ground },
        }
    }

    /// Pushes for `n` updates towards −y; the object moves between
    /// updates as the viewer moves it. Returns his and its y, and the
    /// touches.
    fn push_for(n: usize, mut objects: Vec<BoxCollider>, ground: &dyn GroundLookup) -> (f32, f32, Vec<PushTouch>) {
        let mut m = Mover::new([0.0, 1000.0, ground.ground(0.0, 1000.0).unwrap().0], true);
        let mut touches = Vec::new();
        for _ in 0..n {
            let scene = Scene::new(ground, &[], &objects);
            let r = m.update_with(&scene, |_, _| push_wish([0.0, -1.0], true, true));
            touches.extend(r.pushes);
            for (name, d) in scene.moved.take() {
                let o = objects.iter_mut().find(|o| o.name == name).unwrap();
                o.placement.position = add(o.placement.position, d);
            }
        }
        (m.position[1], objects[0].placement.position[1], touches)
    }

    #[test]
    fn a_ball_stack_pushable_slides_ahead_and_stops_at_a_wall() {
        // A wall 3,000 away beyond the bale (type 0x41: an obstacle for
        // its mask 0xB).
        let mut wall = post([0.0, -3000.0, 0.0]);
        wall.min = [-2000.0, -250.0, 0.0];
        wall.max = [2000.0, 0.0, 2000.0];
        let (connor, bale_y, touches) = push_for(120, vec![bale([0.0; 3], false), wall], &Flat(0.0));
        let moves: Vec<&([f32; 3], [f32; 3], String)> = touches.iter().filter_map(|t| t.moved.as_ref()).collect();
        assert!(!moves.is_empty());
        // It moved by his step at first, and he followed by 0.95 of it.
        let (asked, moved, _) = moves[0];
        assert!((moved[1] - asked[1]).abs() < 1e-3 && moved[1] < 0.0, "{moves:?}");
        // Stopped by the wall: its ball (radius 500) touches the wall's face.
        assert!(moves.iter().any(|(_, _, why)| why.contains("stopped by post")), "{moves:?}");
        assert!(bale_y > -3000.0 + 500.0 - 30.0 && bale_y < -2400.0, "bale at {bale_y}");
        // He is behind it, never inside it.
        assert!(connor > bale_y + 500.0, "Connor {connor}, bale {bale_y}");
    }

    #[test]
    fn a_box_shaped_pushable_or_one_on_a_slope_stays() {
        // Box-shaped (Q87 a stand-in): not moved.
        let mut b = bale([0.0; 3], false);
        b.kind.ball_stack = false;
        let (_, bale_y, touches) = push_for(30, vec![b], &Flat(0.0));
        assert_eq!(bale_y, 0.0);
        assert!(touches.iter().any(|t| t.moved.as_ref().is_some_and(|m| m.2.contains("Q87 a"))));
        // On a slope with keepOnGround: its height would change, so it is
        // put back (rule 4.2).
        struct Slope;
        impl GroundLookup for Slope {
            fn ground(&self, _: f32, y: f32) -> Option<(f32, [f32; 3])> {
                Some((-0.05 * y, [0.0, 0.05, 0.9987]))
            }
        }
        let (_, bale_y, touches) = push_for(30, vec![bale([0.0; 3], true)], &Slope);
        assert_eq!(bale_y, 0.0);
        assert!(touches.iter().any(|t| t.moved.as_ref().is_some_and(|m| m.2.contains("put back"))));
    }

    #[test]
    fn glancing_off_an_object_slides_along_it() {
        let o = [post([0.0; 3])];
        let scene = Scene::new(&Flat(0.0), &[], &o);
        // 35° off the face's normal: slides along x, keeps clear of y 450
        // while beside the post (its enlarged box: x −450…190).
        let (s, c) = 35f32.to_radians().sin_cos();
        let mut m = Mover::new([-545.0, 800.0, 0.0], true);
        let mut slid = false;
        for _ in 0..40 {
            let r = m.update([400.0 * s, -400.0 * c, 0.0], &scene);
            assert!(!r.head_on);
            slid |= r.across == "glance";
            let beside = m.position[0] > -450.0 && m.position[0] < 190.0;
            assert!(!beside || m.position[1] >= 449.0, "{:?}", m.position);
        }
        assert!(slid);
        assert!(m.position[0] > 210.0, "went past the post: {:?}", m.position);
    }

    #[test]
    fn standing_on_top_of_an_object_stays_25_above_it() {
        // A low crate, 300 high, under him: he lands on it 25 above.
        let o = [BoxCollider { name: "crate".into(), min: [-500.0; 3], max: [500.0, 500.0, 300.0], placement: crate::collide::Placement::at([0.0, 0.0, 0.0]), kind: crate::collide::ColliderKind::default() }];
        let scene = Scene::new(&Flat(0.0), &[], &o);
        let mut m = Mover::new([0.0, 0.0, 800.0], true);
        for _ in 0..30 {
            m.update([0.0; 3], &scene);
        }
        assert!(near(m.position[2], 325.0, 0.01), "{:?}", m.position);
        assert_eq!(m.state, State::Normal);
    }

    #[test]
    fn starting_inside_an_object_is_no_hit_and_is_noted() {
        let o = [post([0.0; 3])];
        let scene = Scene::new(&Flat(0.0), &[], &o);
        let mut m = Mover::new([-130.0, 125.0, 0.0], true);
        let r = m.update([0.0, -72.0, 0.0], &scene);
        assert!(near(m.position[1], 53.0, 0.01), "walks on: {:?}", m.position);
        assert!(r.events.iter().any(|e| e.contains("started inside post")), "{:?}", r.events);
    }

    #[test]
    fn a_crash_pushing_him_into_a_wall_behind_stops_there_without_head_on() {
        // `crash` (not the Move state) pushes him 63 per update for 7
        // updates straight at a wall 300 away: he stops at it, never inside.
        let b = [wall()];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([0.0, 300.0, 0.0], true);
        for n in 1..=7 {
            let r = m.update_with(&scene, |_, _| Wish { step: [0.0, -63.07, 0.0], moving: false, steep_head_on: false, push: PushAbility::default() });
            assert!(!r.head_on, "update {n}: no head-on outside Move");
            assert!(m.position[1] >= 199.0, "update {n}: never inside the wall: {:?}", m.position);
        }
        assert!(m.position[1] <= 225.0 + 0.01, "stopped at the wall: {:?}", m.position);
    }

    #[test]
    fn stepping_backwards_into_a_wall_is_head_on_too() {
        // The mover only sees the movement: backwards is still straight in.
        let b = [wall()];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([0.0, 260.0, 0.0], true);
        let r = m.update([0.0, -60.0, 0.0], &scene);
        assert!(r.head_on, "{:?}", r.hits);
    }

    #[test]
    fn not_moving_is_never_head_on() {
        let b = [wall()];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([0.0, 260.0, 0.0], true);
        let r = m.update_with(&scene, |_, _| Wish { step: [0.0, -60.0, 0.0], moving: false, steep_head_on: false, push: PushAbility::default() });
        assert!(!r.head_on);
        assert_eq!(r.across, "glance");
    }

    /// Measured in the original: glancing, the slide along the wall per
    /// update is the step × sin(angle from the normal): × 0.37 at 21.6°,
    /// × 0.57 at 35°.
    #[test]
    fn glancing_slides_step_times_sine() {
        for (deg, factor) in [(21.6f32, 0.37f32), (35.0, 0.57)] {
            let b = [wall()];
            let scene = Scene::new(&Flat(0.0), &b, &[]);
            let a = deg.to_radians();
            let step = [72.0 * a.sin(), -72.0 * a.cos(), 0.0];
            let mut m = Mover::new([0.0, 400.0, 0.0], true);
            let mut xs = Vec::new();
            for _ in 0..15 {
                let r = m.update(step, &scene);
                assert!(!r.head_on, "{deg}°");
                xs.push(m.position[0]);
            }
            // Once against the wall: the full step along x is kept, which
            // is 72 × sin(angle).
            let last = xs[14] - xs[13];
            assert!(near(last, 72.0 * a.sin(), 0.01), "{deg}°: {last}");
            assert!(near(last / 72.0, factor, 0.01), "{deg}°: {}", last / 72.0);
            assert!(m.position[1] >= 200.0 && m.position[1] <= 225.0, "{deg}°: {:?}", m.position);
        }
    }

    #[test]
    fn into_a_corner_he_stops_outside_both_walls() {
        // Solid where y < 0 or x < 0.
        let corner = building(
            vec![node(0, Node(1), Leaf(0)), node(1, Leaf(1), Leaf(2))],
            vec![true, false, true],
            vec![([0.0, 1.0, 0.0], 0.0), ([1.0, 0.0, 0.0], 0.0)],
        );
        let b = [corner];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([600.0, 500.0, 0.0], true);
        for _ in 0..30 {
            m.update([-100.0, -90.0, 0.0], &scene);
            assert!(m.position[0] >= 199.0 && m.position[1] >= 199.0, "{:?}", m.position);
        }
        assert!(m.position[0] <= 300.0 && m.position[1] <= 300.0, "in the corner: {:?}", m.position);
    }

    #[test]
    fn past_the_end_of_a_wall_he_goes_on_at_full_step() {
        // Solid where y < 0 and x < 0: the wall ends at x = 0.
        let half = building(
            vec![node(0, Leaf(0), Node(1)), node(1, Leaf(1), Leaf(2))],
            vec![false, false, true],
            vec![([0.0, 1.0, 0.0], 0.0), ([1.0, 0.0, 0.0], 0.0)],
        );
        let b = [half];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let step = [50.0, -50.0, 0.0];
        let mut m = Mover::new([-1500.0, 400.0, 0.0], true);
        let mut last = m.position;
        let mut slid = false;
        for _ in 0..80 {
            let r = m.update(step, &scene);
            slid |= r.across == "glance";
            last = std::array::from_fn(|i| m.position[i] - last[i]);
            let moved = last;
            last = m.position;
            if m.position[0] > 400.0 {
                assert!(near(moved[0], 50.0, 0.01) && near(moved[1], -50.0, 0.01), "{moved:?}");
            }
        }
        assert!(slid, "it slid along the wall first");
        assert!(m.position[1] < 0.0, "then went on past its end: {:?}", m.position);
    }

    /// A block whose top is at `height`, solid for x > 0 below it.
    fn step_block(height: f32) -> BuildingCollider {
        building(
            vec![node(0, Node(1), Leaf(0)), node(1, Leaf(1), Leaf(2))],
            vec![false, false, true],
            vec![([1.0, 0.0, 0.0], 0.0), ([0.0, 0.0, 1.0], -height)],
        )
    }

    #[test]
    fn a_300_step_is_climbed_a_400_one_stops_him() {
        let b = [step_block(300.0)];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([-500.0, 0.0, 0.0], true);
        for _ in 0..20 {
            m.update([75.0, 0.0, 0.0], &scene);
        }
        assert!(m.position[0] > 500.0, "{:?}", m.position);
        // On the block: 25 above its top, on the ground.
        assert!(near(m.position[2], 325.0, 0.01) && m.on_ground, "{:?}", m.position);

        let b = [step_block(400.0)];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([-500.0, 0.0, 0.0], true);
        let mut head_on = false;
        for _ in 0..20 {
            head_on |= m.update([75.0, 0.0, 0.0], &scene).head_on;
        }
        assert!(head_on);
        assert!(m.position[0] <= -200.0 && m.position[2] == 0.0, "{:?}", m.position);
    }

    #[test]
    fn a_low_ceiling_cuts_the_up_pass() {
        // Solid above z = 1500.
        let ceiling = building(vec![node(0, Leaf(0), Leaf(1))], vec![false, true], vec![([0.0, 0.0, -1.0], 1500.0)]);
        let b = [ceiling];
        let scene = Scene::new(&Flat(0.0), &b, &[]);
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        let r = m.update([0.0; 3], &scene);
        let up = r.hits.iter().find(|h| h.pass == "up").expect("up-pass hit");
        // Top ball (centre 1,100, radius 200) touches at feet 200, pulled
        // back 25 towards the start.
        assert!(near(up.hit.position[2], 175.0, 0.01), "{up:?}");
        assert_eq!(up.hit.what, "test");
        assert!(m.on_ground && m.position[2] == 0.0, "then down onto the ground: {:?}", m.position);
    }

    #[test]
    fn standing_on_a_building_floor_stays_25_above_it() {
        // Solid below z = 0; the terrain far below.
        let floor = building(vec![node(0, Leaf(0), Leaf(1))], vec![false, true], vec![([0.0, 0.0, 1.0], 0.0)]);
        let b = [floor];
        let scene = Scene::new(&Flat(-5000.0), &b, &[]);
        let mut m = Mover::new([0.0, 0.0, 25.0], true);
        for _ in 0..30 {
            let r = m.update([0.0; 3], &scene);
            assert_eq!(r.down, DownResult::Walkable);
            assert!(near(m.position[2], 25.0, 0.001) && m.on_ground, "{:?}", m.position);
            assert_eq!(m.air_updates, 0);
        }
    }
}
