//! Collision tests: Connor's ball stack against buildings (through their
//! BSP tree) and against objects' boxes. M2 step 2d-2; used by the mover
//! for buildings since 2d-3 and for objects since 2d-5.
//!
//! Rules from `docs/behaviour/connor-physics.md` "Collision shape"
//! (traced, Q31 and Q33): the ball stack built from the box, the building
//! test steps 1–7, the object test and the 25-unit pull-back. Data:
//! `GAME/patch/BOUNDS.KQ` (boxes) and the `.dig` BSP tables.
//!
//! Words: a *BSP tree* splits space by planes; each end (*leaf*) is solid
//! or open. A *sweep* moves a shape from its old to its new position and
//! finds the first touch on the way.

use kq8_data::building::{BspChild, BspNode, Building, BSP_ROOT};

/// Every building or object contact is moved this far back towards the
/// old position (to the old position itself if it is closer than this).
pub const PULL_BACK: f32 = 25.0;
/// A building test is skipped for a move shorter than this.
pub const BUILDING_MIN_MOVE: f32 = 0.1;
/// An object test is skipped for a move shorter than this in every axis.
pub const OBJECT_MIN_MOVE: f32 = 0.001;
/// At a BSP node a ball is "touching" the plane unless its centre is more
/// than r + this away from it.
pub const TOUCH_MARGIN: f64 = 0.00001;
/// Connor's collision type and mask (connor-physics.md "Collision shape",
/// read from the running game).
pub const CONNOR_TYPE: u32 = 4;
pub const CONNOR_MASK: u32 = 0x281B;
/// Type and mask of an object that sets neither `colliderType` nor
/// `colliderMask` (connor-physics.md "Collision shape", traced).
pub const DEFAULT_TYPE: u32 = 0x41;
pub const DEFAULT_MASK: u32 = 0;

/// Whether two objects are tested against each other: the first one's
/// mask shares a bit with the second one's type, or the other way round
/// (connor-physics.md "Which pairs collide", traced).
pub fn pair_collides(type_a: u32, mask_a: u32, type_b: u32, mask_b: u32) -> bool {
    mask_a & type_b != 0 || mask_b & type_a != 0
}

/// The balls an object collides with: all of radius `radius`, centred
/// straight above its position at `heights`.
#[derive(Debug, Clone, PartialEq)]
pub struct BallStack {
    pub radius: f32,
    /// Lowest first.
    pub heights: Vec<f32>,
}

impl BallStack {
    /// From a collision box in the object's own space: r = half the larger
    /// of the x and y sizes; n = height / 2r rounded up; ball k (k < n − 1)
    /// at min z + r + 2r·k, the last at max z − r (a single ball at
    /// min z + r).
    pub fn from_box(min: [f32; 3], max: [f32; 3]) -> Self {
        let radius = (max[0] - min[0]).max(max[1] - min[1]) / 2.0;
        let height = max[2] - min[2];
        let n = if radius > 0.0 { ((height / (2.0 * radius)).ceil() as usize).max(1) } else { 1 };
        let mut heights: Vec<f32> = (0..n - 1).map(|k| min[2] + radius + 2.0 * radius * k as f32).collect();
        heights.push(if n == 1 { min[2] + radius } else { max[2] - radius });
        BallStack { radius, heights }
    }

    /// The world box the balls can touch while the mover goes from `from`
    /// to `to`: for the rough check. (The game grows the move's box by
    /// √3 × the largest half-size of the object's box, about 1,126 for
    /// Connor; measured from where is not traced, so this covers the balls'
    /// full reach instead, which can't miss anything.)
    pub fn reach(&self, from: [f32; 3], to: [f32; 3]) -> ([f32; 3], [f32; 3]) {
        let r = self.radius;
        let (low, high) = (self.heights[0] - r, self.heights[self.heights.len() - 1] + r);
        let lo = [from[0].min(to[0]) - r, from[1].min(to[1]) - r, from[2].min(to[2]) + low];
        let hi = [from[0].max(to[0]) + r, from[1].max(to[1]) + r, from[2].max(to[2]) + high];
        (lo, hi)
    }
}

/// Where an object stands: `world = own × rotation + position` (row
/// vectors, the same as drawing; `kq8_data::objects::rotation`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placement {
    pub rotation: [[f32; 3]; 3],
    pub position: [f32; 3],
}

impl Placement {
    pub fn at(position: [f32; 3]) -> Self {
        Placement { rotation: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], position }
    }

    /// A world point in the object's own space (the placement undone).
    pub fn to_own(&self, p: [f32; 3]) -> [f32; 3] {
        let d = sub(p, self.position);
        std::array::from_fn(|j| dot(d, self.rotation[j]))
    }

    pub fn to_world(&self, p: [f32; 3]) -> [f32; 3] {
        add(self.direction_to_world(p), self.position)
    }

    pub fn direction_to_world(&self, v: [f32; 3]) -> [f32; 3] {
        std::array::from_fn(|k| (0..3).map(|j| v[j] * self.rotation[j][k]).sum())
    }
}

/// A building or object contact, in world space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Contact {
    /// Where the mover is put: the touch position pulled back 25 towards
    /// the old position.
    pub position: [f32; 3],
    /// The mover's position when the ball touched (before the pull-back).
    pub touch: [f32; 3],
    /// Unit surface direction, pointing against the move.
    pub normal: [f32; 3],
    /// Which ball touched (index into `BallStack::heights`).
    pub ball: usize,
}

/// Moves `touch` back towards `from` by `PULL_BACK`; `from` itself if it
/// is within that distance.
pub fn pull_back(touch: [f32; 3], from: [f32; 3]) -> [f32; 3] {
    let d = sub(touch, from);
    let len = length(d);
    if len <= PULL_BACK {
        from
    } else {
        sub(touch, scale(d, PULL_BACK / len))
    }
}

/// A building (`KQInterior`) as a collider: its BSP tree and placement.
#[derive(Debug, Clone)]
pub struct BuildingCollider {
    pub name: String,
    nodes: Vec<BspNode>,
    solid: Vec<bool>,
    planes: Vec<([f32; 3], f32)>,
    pub placement: Placement,
    /// The building's own bounding box (from its `.dig`), in its own space.
    pub bounds: ([f32; 3], [f32; 3]),
    /// The building's bounding box turned into the world (an axis-aligned
    /// box around its 8 corners), for the rough check.
    pub world_min: [f32; 3],
    pub world_max: [f32; 3],
}

/// A hit inside the BSP trace, in the building's own space.
#[derive(Debug, Clone, Copy)]
struct BspHit {
    /// Ball centre where the traced piece entered the solid leaf.
    point: [f32; 3],
    /// The crossed plane's normal, pointing against the move.
    normal: [f32; 3],
}

#[derive(Clone, Copy, PartialEq)]
enum Side {
    Front,
    Behind,
    Touching,
}

impl BuildingCollider {
    pub fn new(name: &str, b: &Building, placement: Placement) -> Self {
        let solid = (0..b.leaf_flags.len()).map(|i| b.leaf_solid(i).unwrap_or(false)).collect();
        let planes = b.planes.iter().map(|p| (p.normal, p.d)).collect();
        Self::from_parts(name, b.nodes.clone(), solid, planes, (b.bounds_min, b.bounds_max), placement)
    }

    /// From the tree's parts (also used by the tests).
    pub fn from_parts(
        name: &str,
        nodes: Vec<BspNode>,
        solid: Vec<bool>,
        planes: Vec<([f32; 3], f32)>,
        bounds: ([f32; 3], [f32; 3]),
        placement: Placement,
    ) -> Self {
        let (world_min, world_max) = world_box(bounds.0, bounds.1, &placement);
        BuildingCollider { name: name.to_string(), nodes, solid, planes, placement, bounds, world_min, world_max }
    }

    /// The rough check: does the box `lo..hi` (world space) overlap the
    /// building's box? (Measured on all 153 buildings: the BSP is open
    /// everywhere just outside the building's box, so this only saves time
    /// and never changes a result, as long as `lo..hi` covers everything
    /// the balls can touch: `BallStack::reach`.)
    pub fn overlaps(&self, lo: [f32; 3], hi: [f32; 3]) -> bool {
        (0..3).all(|k| lo[k] <= self.world_max[k] && hi[k] >= self.world_min[k])
    }

    /// Sweeps the mover's balls from `from` to `to` (mover positions, world
    /// space) through the BSP; the earliest touch of any ball, pulled back.
    pub fn sweep(&self, stack: &BallStack, from: [f32; 3], to: [f32; 3]) -> Option<Contact> {
        if length(sub(to, from)) < BUILDING_MIN_MOVE || self.nodes.is_empty() {
            return None;
        }
        let mut best: Option<(f32, usize, BspHit)> = None;
        for (ball, &h) in stack.heights.iter().enumerate() {
            let s = self.placement.to_own(add(from, [0.0, 0.0, h]));
            let e = self.placement.to_own(add(to, [0.0, 0.0, h]));
            let mv = sub(e, s);
            if let Some(hit) = self.trace(BspChild::Node(BSP_ROOT), s, e, stack.radius, None, mv) {
                let dist = length(sub(hit.point, s));
                if best.is_none_or(|(d, _, _)| dist < d) {
                    best = Some((dist, ball, hit));
                }
            }
        }
        let (_, ball, hit) = best?;
        let touch = sub(self.placement.to_world(hit.point), [0.0, 0.0, stack.heights[ball]]);
        let normal = normalise(self.placement.direction_to_world(hit.normal));
        Some(Contact { position: pull_back(touch, from), touch, normal, ball })
    }

    /// Where a straight line from `from` to `to` (world space) first enters
    /// solid: the same BSP walk as `sweep` with a line of no width
    /// (camera.md "Obstacles": buildings block the view by their BSP).
    /// Starting inside solid is not a hit.
    pub fn line_hit(&self, from: [f32; 3], to: [f32; 3]) -> Option<[f32; 3]> {
        if self.nodes.is_empty() || length(sub(to, from)) < BUILDING_MIN_MOVE {
            return None;
        }
        let (s, e) = (self.placement.to_own(from), self.placement.to_own(to));
        let hit = self.trace(BspChild::Node(BSP_ROOT), s, e, 0.0, None, sub(e, s))?;
        Some(self.placement.to_world(hit.point))
    }

    /// Whether a point (world space) is in a solid leaf. For checks and
    /// logs only; the passes use `sweep`.
    pub fn solid_at(&self, p: [f32; 3]) -> bool {
        if self.nodes.is_empty() {
            return false;
        }
        let p = self.placement.to_own(p);
        let mut child = BspChild::Node(BSP_ROOT);
        loop {
            match child {
                BspChild::Leaf(l) => return self.solid.get(l).copied().unwrap_or(false),
                BspChild::Node(i) => {
                    let n = self.nodes[i];
                    let (normal, d) = self.planes[n.plane as usize];
                    child = if dot(normal, p) + d > 0.0 { n.front } else { n.back };
                }
            }
        }
    }

    /// The traced BSP walk (connor-physics.md "Against a building", steps
    /// 4 and 5) for the piece of a ball's path from `s` to `e`. `crossed`
    /// is the last plane crossed on the way here; `mv` the whole move, for
    /// pointing the hit's normal against it.
    fn trace(&self, child: BspChild, s: [f32; 3], e: [f32; 3], r: f32, crossed: Option<[f32; 3]>, mv: [f32; 3]) -> Option<BspHit> {
        let i = match child {
            BspChild::Leaf(l) => {
                // Entering a solid leaf across a plane is a hit where this
                // piece entered it; starting inside solid is not.
                if !self.solid.get(l).copied().unwrap_or(false) {
                    return None;
                }
                let n = crossed?;
                let normal = if dot(n, mv) > 0.0 { scale(n, -1.0) } else { n };
                return Some(BspHit { point: s, normal });
            }
            BspChild::Node(i) => i,
        };
        let node = self.nodes[i];
        let (normal, d) = self.planes[node.plane as usize];
        let (ds, de) = (dot(normal, s) + d, dot(normal, e) + d);
        // In f64: in f32, r + 0.00001 rounds back to r.
        let limit = r as f64 + TOUCH_MARGIN;
        let side = |v: f32| {
            let v = v as f64;
            if v > limit {
                Side::Front
            } else if v < -limit {
                Side::Behind
            } else {
                Side::Touching
            }
        };
        match (side(ds), side(de)) {
            (Side::Front, Side::Front) | (Side::Touching, Side::Front) => self.trace(node.front, s, e, r, crossed, mv),
            (Side::Behind, Side::Behind) | (Side::Touching, Side::Behind) => self.trace(node.back, s, e, r, crossed, mv),
            (Side::Touching, Side::Touching) => {
                let a = self.trace(node.front, s, e, r, crossed, mv);
                let b = self.trace(node.back, s, e, r, crossed, mv);
                match (a, b) {
                    (Some(a), Some(b)) => Some(if length(sub(a.point, s)) <= length(sub(b.point, s)) { a } else { b }),
                    (a, b) => a.or(b),
                }
            }
            (Side::Front, _) => {
                // Split where the ball first touches the plane moved by r
                // towards the front.
                let m = lerp(s, e, ((ds - r) / (ds - de)).clamp(0.0, 1.0));
                self.trace(node.front, s, m, r, crossed, mv).or_else(|| self.trace(node.back, m, e, r, Some(normal), mv))
            }
            (Side::Behind, _) => {
                let m = lerp(s, e, ((ds + r) / (ds - de)).clamp(0.0, 1.0));
                self.trace(node.back, s, m, r, crossed, mv).or_else(|| self.trace(node.front, m, e, r, Some(normal), mv))
            }
        }
    }
}

/// An object's collision box (from `BOUNDS.KQ`) and placement. The box
/// turns with the object.
#[derive(Debug, Clone)]
pub struct BoxCollider {
    pub name: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub placement: Placement,
    /// What kind of thing it is for collisions (M3 step 3j).
    pub kind: ColliderKind,
}

/// An object's collision settings (`pushing.md`, `connor-collision.md`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ColliderKind {
    /// `colliderType` now (scripts change it).
    pub collision_type: u32,
    /// `colliderMask`.
    pub collision_mask: u32,
    /// `collisionShape=1`: a ball stack (`connor-collision.md` "Ball stack
    /// against ball stack").
    pub ball_stack: bool,
    /// `keepOnGround` puts it on its ground grid (for a pushed object's
    /// move, `pushing.md` rule 4.1; Q87 c stand-in: exactly on the floor
    /// grid).
    pub keep_on_ground: bool,
}

impl Default for ColliderKind {
    fn default() -> Self {
        ColliderKind { collision_type: DEFAULT_TYPE, collision_mask: DEFAULT_MASK, ball_stack: false, keep_on_ground: false }
    }
}

/// Bits of another's type that make it an obstacle for a plain object's
/// own move, when its mask covers them (`connor-collision.md` rule 3; 0x1
/// and 0x2000 as world touches, which are obstacles without
/// `worldCollisionAction`, rule 12). Connor's bit 4 is a Connor touch.
pub const OBSTACLE_BITS: u32 = 0x100 | 0x8 | 0x1 | 0x2 | 0x2000;

/// Pushable (`pushing.md`).
pub const PUSHABLE: u32 = 0x100;
/// Pushable by its script (`pushing.md`).
pub const SCRIPTED_PUSH: u32 = 0x10000;

/// What an object test found.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ObjectHit {
    Contact(Contact),
    /// A ball started inside the enlarged box: a hit with no usable
    /// contact (what the game does next was not traced).
    StartedInside { ball: usize },
}

impl BoxCollider {
    /// The rough check, only to save time: does the box `lo..hi` (world
    /// space: `BallStack::reach`) overlap this box enlarged by `r` on every
    /// face and turned into the world (an axis-aligned box round its 8
    /// corners)? Generous for turned objects, so it never skips a hit.
    pub fn overlaps(&self, r: f32, lo: [f32; 3], hi: [f32; 3]) -> bool {
        let (wmin, wmax) = world_box(self.min.map(|v| v - r), self.max.map(|v| v + r), &self.placement);
        (0..3).all(|k| lo[k] <= wmax[k] && hi[k] >= wmin[k])
    }

    /// Where a straight line from `from` to `to` (world space) first enters
    /// the box (camera.md "Obstacles"); starting inside is not a hit.
    pub fn line_hit(&self, from: [f32; 3], to: [f32; 3]) -> Option<[f32; 3]> {
        let (lo, hi) = (self.min, self.max);
        if (0..3).any(|k| lo[k] > hi[k]) {
            return None;
        }
        let (s, e) = (self.placement.to_own(from), self.placement.to_own(to));
        if (0..3).all(|k| s[k] > lo[k] && s[k] < hi[k]) {
            return None;
        }
        let (t, _) = slab_entry(s, sub(e, s), lo, hi)?;
        Some(lerp(from, to, t))
    }

    /// Sweeps the mover's balls against the box enlarged by the radius on
    /// every face (square corners), in the object's own space. An exactly
    /// vertical move tests only the top ball (going up or not moving) or
    /// the bottom ball (going down). The earliest hit counts.
    pub fn sweep(&self, stack: &BallStack, from: [f32; 3], to: [f32; 3]) -> Option<ObjectHit> {
        let mv = sub(to, from);
        if mv.iter().all(|v| v.abs() < OBJECT_MIN_MOVE) {
            return None;
        }
        let r = stack.radius;
        let (lo, hi) = (self.min.map(|v| v - r), self.max.map(|v| v + r));
        if (0..3).any(|k| lo[k] > hi[k]) {
            // An inside-out box: the table lookup fixes these on loading
            // (Q34 c), so only a made-up box gets here.
            return None;
        }
        let last = stack.heights.len() - 1;
        let balls: Vec<usize> = if mv[0] == 0.0 && mv[1] == 0.0 {
            vec![if mv[2] >= 0.0 { last } else { 0 }]
        } else {
            (0..=last).collect()
        };
        let mut best: Option<(f32, usize, [f32; 3])> = None;
        for ball in balls {
            let h = [0.0, 0.0, stack.heights[ball]];
            let s = self.placement.to_own(add(from, h));
            let e = self.placement.to_own(add(to, h));
            if (0..3).all(|k| s[k] > lo[k] && s[k] < hi[k]) {
                return Some(ObjectHit::StartedInside { ball });
            }
            if let Some((t, normal)) = slab_entry(s, sub(e, s), lo, hi) {
                if best.is_none_or(|(bt, _, _)| t < bt) {
                    best = Some((t, ball, normal));
                }
            }
        }
        let (t, ball, normal) = best?;
        let touch = add(from, scale(mv, t));
        let normal = normalise(self.placement.direction_to_world(normal));
        Some(ObjectHit::Contact(Contact { position: pull_back(touch, from), touch, normal, ball }))
    }
}

/// A ball-stack object as the other side of a ball-stack-against-ball-stack
/// test (`connor-collision.md` rules 18–19; M4 4d-12-1): an upright
/// cylinder standing on its position. It does not turn with the object.
#[derive(Debug, Clone, PartialEq)]
pub struct CylinderCollider {
    pub name: String,
    /// Its position (the cylinder's foot, on the axis).
    pub position: [f32; 3],
    pub collision_type: u32,
    pub collision_mask: u32,
    /// Before the mover's ball radius is added.
    pub radius: f32,
    pub height: f32,
}

/// A cylinder test is skipped for a move shorter than this in every axis
/// (`connor-collision.md` rule 18).
pub const CYLINDER_MIN_MOVE: f32 = 0.000001;
/// Type bits of creatures (2) and Connor (4): a ball that starts inside
/// their cylinder is no hit (rule 19).
pub const CREATURE_BITS: u32 = 0x2 | 0x4;

/// What a cylinder test found.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct CylinderSweep {
    pub contact: Option<Contact>,
    /// Balls that started inside a creature's cylinder: not hits (rule 19).
    pub inside: Vec<usize>,
}

impl CylinderCollider {
    /// From the other object's `BOUNDS.KQ` box (own space). **Stand-in**
    /// (rule 18: "not pinned down: likely its ball radius and its box
    /// height"): the radius of its ball stack (half the larger of the x
    /// and y sizes) and the box's height.
    pub fn from_box(name: &str, position: [f32; 3], min: [f32; 3], max: [f32; 3], collision_type: u32, collision_mask: u32) -> Self {
        CylinderCollider {
            name: name.to_string(),
            position,
            collision_type,
            collision_mask,
            radius: BallStack::from_box(min, max).radius,
            height: max[2] - min[2],
        }
    }

    /// The rough check, only to save time: does the box `lo..hi`
    /// (`BallStack::reach`) overlap the cylinder's box widened by `r`?
    pub fn overlaps(&self, r: f32, lo: [f32; 3], hi: [f32; 3]) -> bool {
        let p = self.position;
        let reach = self.radius + r;
        let (cmin, cmax) = ([p[0] - reach, p[1] - reach, p[2] - r], [p[0] + reach, p[1] + reach, p[2] + self.height + r]);
        (0..3).all(|k| lo[k] <= cmax[k] && hi[k] >= cmin[k])
    }

    /// Rules 18–19: each of the mover's balls (its centre's path from
    /// `from` to `to` plus its height) against the cylinder widened by the
    /// ball's radius sideways and at both ends; the earliest hit counts.
    /// A side hit's surface direction is level, out from the axis through
    /// the hit point; a top or bottom hit's is straight up or down. A ball
    /// that starts inside: no hit for a creature or Connor (type bit 2 or
    /// 4), otherwise a hit at the start with the surface direction against
    /// the movement. The contact is pulled back 25 (`PULL_BACK`).
    pub fn sweep(&self, stack: &BallStack, from: [f32; 3], to: [f32; 3]) -> CylinderSweep {
        let mut out = CylinderSweep::default();
        let mv = sub(to, from);
        if mv.iter().all(|v| v.abs() < CYLINDER_MIN_MOVE) {
            return out;
        }
        let r = stack.radius;
        let reach = self.radius + r;
        let (low, high) = (self.position[2] - r, self.position[2] + self.height + r);
        let (cx, cy) = (self.position[0], self.position[1]);
        let mut best: Option<(f32, usize, [f32; 3])> = None;
        let mut keep = |t: f32, ball: usize, normal: [f32; 3]| {
            if best.is_none_or(|(bt, _, _)| t < bt) {
                best = Some((t, ball, normal));
            }
        };
        for (ball, h) in stack.heights.iter().enumerate() {
            let s = [from[0] - cx, from[1] - cy, from[2] + h];
            let inside_xy = |p: [f32; 3]| p[0] * p[0] + p[1] * p[1] < reach * reach;
            if inside_xy(s) && s[2] > low && s[2] < high {
                if self.collision_type & CREATURE_BITS != 0 {
                    out.inside.push(ball);
                } else {
                    keep(0.0, ball, normalise(scale(mv, -1.0)));
                }
                continue;
            }
            // The side: where the level distance from the axis first
            // becomes `reach`, with the centre between the ends then.
            let (a, b, c) = (mv[0] * mv[0] + mv[1] * mv[1], 2.0 * (s[0] * mv[0] + s[1] * mv[1]), s[0] * s[0] + s[1] * s[1] - reach * reach);
            if a > 0.0 && c >= 0.0 {
                let disc = b * b - 4.0 * a * c;
                if disc >= 0.0 {
                    let t = (-b - disc.sqrt()) / (2.0 * a);
                    let z = s[2] + mv[2] * t;
                    if (0.0..=1.0).contains(&t) && z >= low && z <= high {
                        let (x, y) = (s[0] + mv[0] * t, s[1] + mv[1] * t);
                        keep(t, ball, normalise([x, y, 0.0]));
                    }
                }
            }
            // The ends: crossing the bottom going up or the top going down,
            // within the radius there.
            let ends = [(mv[2] > 0.0 && s[2] <= low, low, -1.0), (mv[2] < 0.0 && s[2] >= high, high, 1.0)];
            for (crosses, z, up) in ends {
                if !crosses {
                    continue;
                }
                let t = (z - s[2]) / mv[2];
                let p = [s[0] + mv[0] * t, s[1] + mv[1] * t, z];
                if (0.0..=1.0).contains(&t) && p[0] * p[0] + p[1] * p[1] <= reach * reach {
                    keep(t, ball, [0.0, 0.0, up]);
                }
            }
        }
        if let Some((t, ball, normal)) = best {
            let touch = add(from, scale(mv, t));
            out.contact = Some(Contact { position: pull_back(touch, from), touch, normal, ball });
        }
        out
    }
}

/// The axis-aligned world box round the 8 corners of the own-space box
/// `lo..hi` placed by `placement`.
fn world_box(lo: [f32; 3], hi: [f32; 3], placement: &Placement) -> ([f32; 3], [f32; 3]) {
    let mut world_min = [f32::INFINITY; 3];
    let mut world_max = [f32::NEG_INFINITY; 3];
    for c in 0..8 {
        let corner = [if c & 1 == 0 { lo[0] } else { hi[0] }, if c & 2 == 0 { lo[1] } else { hi[1] }, if c & 4 == 0 { lo[2] } else { hi[2] }];
        let w = placement.to_world(corner);
        for k in 0..3 {
            world_min[k] = world_min[k].min(w[k]);
            world_max[k] = world_max[k].max(w[k]);
        }
    }
    (world_min, world_max)
}

/// Where a point moving from `s` by `d` (fraction 0..1) first enters the
/// box `lo..hi`, and the entered face's normal (pointing against the
/// move). `None` if it never enters within the move.
fn slab_entry(s: [f32; 3], d: [f32; 3], lo: [f32; 3], hi: [f32; 3]) -> Option<(f32, [f32; 3])> {
    let (mut t_in, mut t_out) = (f32::NEG_INFINITY, f32::INFINITY);
    let mut axis = None;
    for k in 0..3 {
        if d[k] == 0.0 {
            if s[k] < lo[k] || s[k] > hi[k] {
                return None;
            }
            continue;
        }
        let (a, b) = ((lo[k] - s[k]) / d[k], (hi[k] - s[k]) / d[k]);
        let (near, far) = if a < b { (a, b) } else { (b, a) };
        if near > t_in {
            t_in = near;
            axis = Some(k);
        }
        t_out = t_out.min(far);
    }
    let k = axis?;
    if t_in > t_out || !(0.0..=1.0).contains(&t_in) {
        return None;
    }
    let mut normal = [0.0; 3];
    normal[k] = if d[k] > 0.0 { -1.0 } else { 1.0 };
    Some((t_in, normal))
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
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

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

fn normalise(a: [f32; 3]) -> [f32; 3] {
    let l = length(a);
    if l > 0.0 { scale(a, 1.0 / l) } else { a }
}

fn lerp(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    std::array::from_fn(|i| a[i] + (b[i] - a[i]) * t)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn connor() -> BallStack {
        BallStack::from_box([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0])
    }

    fn near(a: [f32; 3], b: [f32; 3]) -> bool {
        (0..3).all(|k| (a[k] - b[k]).abs() < 0.01)
    }

    fn node(plane: u16, front: BspChild, back: BspChild) -> BspNode {
        BspNode { plane, front, back }
    }

    use BspChild::{Leaf, Node};

    /// A wall 100 thick between y = −100 and y = 0, endless in x and z.
    /// Node 0: plane y = 0 (normal +y): front open (leaf 0), back node 1.
    /// Node 1: plane −y − 100 = 0 (normal −y): front (y < −100) open
    /// (leaf 1), back solid (leaf 2).
    fn wall() -> BuildingCollider {
        BuildingCollider::from_parts(
            "wall",
            vec![node(0, Leaf(0), Node(1)), node(1, Leaf(1), Leaf(2))],
            vec![false, false, true],
            vec![([0.0, 1.0, 0.0], 0.0), ([0.0, -1.0, 0.0], -100.0)],
            ([-5000.0, -100.0, 0.0], [5000.0, 0.0, 3000.0]),
            Placement::at([0.0; 3]),
        )
    }

    /// Solid below z = 0 (floor) or above z = `top` (ceiling).
    fn floor_and_ceiling(top: f32) -> BuildingCollider {
        BuildingCollider::from_parts(
            "room",
            // Node 0: z = 0, normal +z: front node 1, back solid (leaf 0).
            // Node 1: −z + top = 0, normal −z: front open (leaf 1), back
            // solid (leaf 2).
            vec![node(0, Node(1), Leaf(0)), node(1, Leaf(1), Leaf(2))],
            vec![true, false, true],
            vec![([0.0, 0.0, 1.0], 0.0), ([0.0, 0.0, -1.0], top)],
            ([-5000.0, -5000.0, -100.0], [5000.0, 5000.0, top + 100.0]),
            Placement::at([0.0; 3]),
        )
    }

    #[test]
    fn a_line_stops_at_the_wall_face_and_not_from_inside() {
        let hit = wall().line_hit([0.0, 1000.0, 500.0], [0.0, -1000.0, 500.0]).expect("blocked");
        assert!(near(hit, [0.0, 0.0, 500.0]), "{hit:?}");
        let hit = wall().line_hit([0.0, -1000.0, 500.0], [0.0, 1000.0, 500.0]).expect("blocked");
        assert!(near(hit, [0.0, -100.0, 500.0]), "{hit:?}");
        // Not reaching it, or starting inside the solid: no hit.
        assert!(wall().line_hit([0.0, 1000.0, 500.0], [0.0, 10.0, 500.0]).is_none());
        assert!(wall().line_hit([0.0, -50.0, 500.0], [0.0, 1000.0, 500.0]).is_none());
    }

    #[test]
    fn a_line_meets_a_turned_box() {
        let mut b = BoxCollider { name: "b".into(), min: [-100.0, -50.0, 0.0], max: [100.0, 50.0, 300.0], placement: Placement::at([1000.0, 0.0, 0.0]), kind: ColliderKind::default() };
        let hit = b.line_hit([0.0, 0.0, 100.0], [2000.0, 0.0, 100.0]).expect("blocked");
        assert!(near(hit, [900.0, 0.0, 100.0]), "{hit:?}");
        // Turned a quarter: its long side now runs along y.
        b.placement.rotation = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]);
        let hit = b.line_hit([0.0, 0.0, 100.0], [2000.0, 0.0, 100.0]).expect("blocked");
        assert!(near(hit, [950.0, 0.0, 100.0]), "{hit:?}");
        assert!(b.line_hit([0.0, 0.0, 400.0], [2000.0, 0.0, 400.0]).is_none());
    }

    #[test]
    fn connor_has_four_balls() {
        assert_eq!(connor(), BallStack { radius: 200.0, heights: vec![200.0, 600.0, 1000.0, 1100.0] });
        // The last ball sits at max z − r, overlapping the one below.
        assert_eq!(BallStack::from_box([-200.0, -100.0, 0.0], [200.0, 100.0, 500.0]).heights, [200.0, 300.0]);
        // One ball when the box is no taller than 2r.
        assert_eq!(BallStack::from_box([-200.0, -200.0, 0.0], [200.0, 200.0, 400.0]).heights, [200.0]);
    }

    #[test]
    fn pull_back_moves_25_towards_the_start_or_stays_at_it() {
        assert!(near(pull_back([0.0, 200.0, 0.0], [0.0, 1000.0, 0.0]), [0.0, 225.0, 0.0]));
        assert_eq!(pull_back([0.0, 990.0, 0.0], [0.0, 1000.0, 0.0]), [0.0, 1000.0, 0.0]);
    }

    #[test]
    fn wall_stops_his_centre_225_from_either_face() {
        let c = wall().sweep(&connor(), [0.0, 1000.0, 0.0], [0.0, 100.0, 0.0]).expect("hit from +y");
        assert!(near(c.touch, [0.0, 200.0, 0.0]), "{c:?}");
        assert!(near(c.position, [0.0, 225.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [0.0, 1.0, 0.0]), "{c:?}");
        let c = wall().sweep(&connor(), [0.0, -1000.0, 0.0], [0.0, -150.0, 0.0]).expect("hit from −y");
        assert!(near(c.position, [0.0, -325.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [0.0, -1.0, 0.0]), "{c:?}");
    }

    #[test]
    fn moving_along_the_wall_or_away_from_it_does_not_hit() {
        assert_eq!(wall().sweep(&connor(), [0.0, 500.0, 0.0], [1000.0, 500.0, 0.0]), None);
        // Starting just touching and moving away.
        assert_eq!(wall().sweep(&connor(), [0.0, 200.0, 0.0], [0.0, 600.0, 0.0]), None);
        // Too short a move is not tested at all.
        assert_eq!(wall().sweep(&connor(), [0.0, 250.0, 0.0], [0.0, 249.95, 0.0]), None);
    }

    #[test]
    fn a_move_ending_exactly_touching_counts_as_a_hit() {
        // The rest of the piece after the split is traced in the back
        // child with the plane crossed, so it reaches the solid leaf.
        let c = wall().sweep(&connor(), [0.0, 1000.0, 0.0], [0.0, 200.0, 0.0]).expect("touching");
        assert!(near(c.position, [0.0, 225.0, 0.0]), "{c:?}");
    }

    #[test]
    fn starting_inside_the_wall_is_no_hit() {
        assert_eq!(wall().sweep(&connor(), [0.0, -50.0, 0.0], [0.0, -80.0, 0.0]), None);
    }

    #[test]
    fn floor_25_below_his_feet_ceiling_25_above_his_head() {
        let room = floor_and_ceiling(2000.0);
        let c = room.sweep(&connor(), [0.0, 0.0, 1000.0], [0.0, 0.0, -500.0]).expect("floor");
        assert_eq!(c.ball, 0, "the bottom ball touches first");
        assert!(near(c.touch, [0.0, 0.0, 0.0]) && near(c.position, [0.0, 0.0, 25.0]), "{c:?}");
        assert!(near(c.normal, [0.0, 0.0, 1.0]));
        let c = room.sweep(&connor(), [0.0, 0.0, 100.0], [0.0, 0.0, 1000.0]).expect("ceiling");
        assert_eq!(c.ball, 3, "the top ball touches first");
        // Top ball centre 1,100 above his feet, radius 200: touches at feet
        // 700, pulled back to 675, so his top (1,300) is 25 below.
        assert!(near(c.position, [0.0, 0.0, 675.0]), "{c:?}");
        assert!(near(c.normal, [0.0, 0.0, -1.0]));
    }

    #[test]
    fn a_low_lintel_stops_him_by_his_upper_balls() {
        // Solid only where −100 < y < 0 and z > 950 (a beam over a door).
        // Node 2: plane z = 950 (normal +z): front (above) solid.
        let lintel = BuildingCollider::from_parts(
            "lintel",
            vec![node(0, Leaf(0), Node(1)), node(1, Leaf(1), Node(2)), node(2, Leaf(2), Leaf(3))],
            vec![false, false, true, false],
            vec![([0.0, 1.0, 0.0], 0.0), ([0.0, -1.0, 0.0], -100.0), ([0.0, 0.0, 1.0], -950.0)],
            ([-5000.0, -100.0, 950.0], [5000.0, 0.0, 3000.0]),
            Placement::at([0.0; 3]),
        );
        let c = lintel.sweep(&connor(), [0.0, 1000.0, 0.0], [0.0, -1000.0, 0.0]).expect("upper balls hit");
        assert!(c.ball >= 2, "{c:?}");
        assert!(near(c.position, [0.0, 225.0, 0.0]), "{c:?}");
        // Higher than his top ball can reach (1,100 + 200): he walks under.
        let high = BuildingCollider::from_parts(
            "lintel",
            vec![node(0, Leaf(0), Node(1)), node(1, Leaf(1), Node(2)), node(2, Leaf(2), Leaf(3))],
            vec![false, false, true, false],
            vec![([0.0, 1.0, 0.0], 0.0), ([0.0, -1.0, 0.0], -100.0), ([0.0, 0.0, 1.0], -1301.0)],
            ([-5000.0, -100.0, 1301.0], [5000.0, 0.0, 3000.0]),
            Placement::at([0.0; 3]),
        );
        assert_eq!(high.sweep(&connor(), [0.0, 1000.0, 0.0], [0.0, -1000.0, 0.0]), None);
    }

    #[test]
    fn a_placed_building_is_tested_in_its_own_space() {
        // The wall turned 90° about z (dirZ = π/2) and moved: its own +y
        // face now looks along world −x.
        let rot = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]);
        let w = BuildingCollider::from_parts(
            "turned",
            vec![node(0, Leaf(0), Node(1)), node(1, Leaf(1), Leaf(2))],
            vec![false, false, true],
            vec![([0.0, 1.0, 0.0], 0.0), ([0.0, -1.0, 0.0], -100.0)],
            ([-5000.0, -100.0, 0.0], [5000.0, 0.0, 3000.0]),
            Placement { rotation: rot, position: [10_000.0, 20_000.0, 500.0] },
        );
        // Own +y becomes world −x (row vectors, +x turns towards +y).
        assert!(near(w.placement.direction_to_world([0.0, 1.0, 0.0]), [-1.0, 0.0, 0.0]));
        let c = w.sweep(&connor(), [9000.0, 20_000.0, 500.0], [9900.0, 20_000.0, 500.0]).expect("hit");
        assert!(near(c.position, [10_000.0 - 225.0, 20_000.0, 500.0]), "{c:?}");
        assert!(near(c.normal, [-1.0, 0.0, 0.0]), "{c:?}");
        let (lo, hi) = connor().reach([9000.0, 20_000.0, 500.0], [9900.0, 20_000.0, 500.0]);
        assert!(w.overlaps(lo, hi));
        let (lo, hi) = connor().reach([0.0, 0.0, 0.0], [10.0, 0.0, 0.0]);
        assert!(!w.overlaps(lo, hi));
    }

    fn crate_box(min: [f32; 3], max: [f32; 3], placement: Placement) -> BoxCollider {
        BoxCollider { name: "box".into(), min, max, placement, kind: ColliderKind::default() }
    }

    #[test]
    fn object_box_is_enlarged_with_square_corners() {
        let b = crate_box([-100.0, -100.0, 0.0], [100.0, 100.0, 500.0], Placement::at([0.0; 3]));
        let Some(ObjectHit::Contact(c)) = b.sweep(&connor(), [-1000.0, 0.0, 0.0], [0.0, 0.0, 0.0]) else { panic!() };
        assert!(near(c.touch, [-300.0, 0.0, 0.0]) && near(c.position, [-325.0, 0.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [-1.0, 0.0, 0.0]));
        // Passing the corner at y = −280: a rounded shape would miss (the
        // ball centre stays 269 from the corner), the square one hits.
        assert!(matches!(b.sweep(&connor(), [-1000.0, -280.0, 0.0], [0.0, -280.0, 0.0]), Some(ObjectHit::Contact(_))));
        assert_eq!(b.sweep(&connor(), [-1000.0, -301.0, 0.0], [1000.0, -301.0, 0.0]), None);
    }

    #[test]
    fn object_box_turns_with_the_object() {
        let rot = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]);
        // Long in own y; turned 90°, long in world x.
        let b = crate_box([-100.0, -500.0, 0.0], [100.0, 500.0, 500.0], Placement { rotation: rot, position: [0.0; 3] });
        let Some(ObjectHit::Contact(c)) = b.sweep(&connor(), [400.0, -2000.0, 0.0], [400.0, 0.0, 0.0]) else { panic!() };
        assert!(near(c.position, [400.0, -325.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [0.0, -1.0, 0.0]), "{c:?}");
    }

    #[test]
    fn vertical_moves_test_one_ball_only() {
        // A box at his middle (z 500..600), around him.
        let b = crate_box([-100.0, -100.0, 500.0], [100.0, 100.0, 600.0], Placement::at([0.0; 3]));
        // Straight down: only the bottom ball, which starts below the box.
        assert_eq!(b.sweep(&connor(), [0.0, 0.0, 0.0], [0.0, 0.0, -1000.0]), None);
        // Any sideways part: all balls; the one at 600 starts inside.
        assert_eq!(b.sweep(&connor(), [0.0, 0.0, 0.0], [1.0, 0.0, -1000.0]), Some(ObjectHit::StartedInside { ball: 1 }));
        // Straight up into a box overhead: the top ball.
        let above = crate_box([-100.0, -100.0, 2000.0], [100.0, 100.0, 2500.0], Placement::at([0.0; 3]));
        let Some(ObjectHit::Contact(c)) = above.sweep(&connor(), [0.0, 0.0, 0.0], [0.0, 0.0, 1000.0]) else { panic!() };
        assert_eq!(c.ball, 3);
        assert!(near(c.position, [0.0, 0.0, 675.0]), "{c:?}");
        // Tiny moves are not tested.
        assert_eq!(above.sweep(&connor(), [0.0, 0.0, 699.9995], [0.0, 0.0, 700.0]), None);
    }

    #[test]
    fn an_inside_out_box_given_directly_never_hits() {
        // The table lookup fixes these (Q34 c); a made-up one is refused.
        let b = crate_box([10.0, -240.0, 20.0], [-1520.0, 10.0, 3020.0], Placement::at([0.0; 3]));
        assert_eq!(b.sweep(&connor(), [-3000.0, 0.0, 0.0], [3000.0, 0.0, 0.0]), None);
    }

    #[test]
    fn which_pairs_collide_with_connor() {
        let with_connor = |t, m| pair_collides(CONNOR_TYPE, CONNOR_MASK, t, m);
        assert!(with_connor(DEFAULT_TYPE, DEFAULT_MASK), "the default type 0x41 shares bit 1 with his mask");
        assert!(!with_connor(0, 0));
        assert!(with_connor(0, 4), "its mask covers his type");
        assert!(!with_connor(0x40, 0), "type 0x40 isn't in his mask 0x281B");
        assert!(with_connor(0x800, 0));
    }

    #[test]
    fn rough_check_never_skips_a_turned_box_corner() {
        // Turned 45°: the enlarged box's corner reaches 0.41 r further
        // along x than the plain box's world box plus r.
        let rot = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_4]);
        let b = crate_box([-100.0, -100.0, 0.0], [100.0, 100.0, 500.0], Placement { rotation: rot, position: [0.0; 3] });
        let stack = connor();
        for x in (300..=420).step_by(5) {
            let (from, to) = ([x as f32, -2000.0, 0.0], [x as f32, 2000.0, 0.0]);
            if b.sweep(&stack, from, to).is_some() {
                let (lo, hi) = stack.reach(from, to);
                assert!(b.overlaps(stack.radius, lo, hi), "x {x}: hit but skipped");
            }
        }
        let (lo, hi) = stack.reach([2000.0, 0.0, 0.0], [3000.0, 0.0, 0.0]);
        assert!(!b.overlaps(stack.radius, lo, hi), "far away: skipped");
    }

    /// A goblin-like character: box ±350 × ±800 × 0–1,300 (ball radius
    /// 800), standing at the origin; `kind` its collision type.
    fn goblin(kind: u32) -> CylinderCollider {
        CylinderCollider::from_box("goblin", [0.0; 3], [-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0], kind, 0x281F)
    }

    #[test]
    fn the_cylinder_is_the_ball_radius_and_the_box_height() {
        let g = goblin(0x42);
        assert_eq!((g.radius, g.height), (800.0, 1300.0));
    }

    #[test]
    fn a_side_hit_stops_his_centre_1000_out_with_a_level_direction() {
        // Walking along −x into it: widened radius 800 + 200.
        let s = goblin(0x42).sweep(&connor(), [2000.0, 0.0, 0.0], [500.0, 0.0, 0.0]);
        let c = s.contact.expect("hit");
        assert!(near(c.touch, [1000.0, 0.0, 0.0]), "{c:?}");
        assert!(near(c.position, [1025.0, 0.0, 0.0]), "pulled back 25: {c:?}");
        assert!(near(c.normal, [1.0, 0.0, 0.0]), "{c:?}");
        // At an angle: the direction is out from the axis through the hit
        // point, level.
        let c = goblin(0x42).sweep(&connor(), [2000.0, 600.0, 0.0], [0.0, 600.0, 0.0]).contact.expect("hit");
        assert!(near(c.touch, [800.0, 600.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [0.8, 0.6, 0.0]), "{c:?}");
        // Passing beside it, or stopping short: no hit.
        assert_eq!(goblin(0x42).sweep(&connor(), [2000.0, 1001.0, 0.0], [-2000.0, 1001.0, 0.0]).contact, None);
        assert_eq!(goblin(0x42).sweep(&connor(), [2000.0, 0.0, 0.0], [1001.0, 0.0, 0.0]).contact, None);
    }

    #[test]
    fn top_and_bottom_hits_point_straight_up_and_down() {
        // Coming down on its top: his bottom ball (centre 200 above his
        // feet) meets 1,300 + 200, so his feet at 1,300.
        let c = goblin(0x42).sweep(&connor(), [100.0, 0.0, 2000.0], [100.0, 0.0, 1000.0]).contact.expect("top");
        assert!(near(c.touch, [100.0, 0.0, 1300.0]), "{c:?}");
        assert!(near(c.normal, [0.0, 0.0, 1.0]), "{c:?}");
        // Going up into it from below: his top ball (centre 1,100 above
        // his feet) meets −200, so his feet at −1,300.
        let c = goblin(0x42).sweep(&connor(), [100.0, 0.0, -2000.0], [100.0, 0.0, -1000.0]).contact.expect("bottom");
        assert!(near(c.touch, [100.0, 0.0, -1300.0]), "{c:?}");
        assert!(near(c.normal, [0.0, 0.0, -1.0]), "{c:?}");
    }

    #[test]
    fn the_nearest_of_his_balls_counts() {
        // A short thing (box 0–300 high, radius 100) on a step: only his
        // lowest ball reaches it as he walks; and one high up only his
        // top ball.
        let low = CylinderCollider::from_box("low", [0.0; 3], [-100.0; 3], [100.0, 100.0, 200.0], 0x42, 0);
        let c = low.sweep(&connor(), [1000.0, 0.0, 0.0], [0.0, 0.0, 0.0]).contact.expect("hit");
        assert_eq!(c.ball, 0, "{c:?}");
        assert!(near(c.touch, [300.0, 0.0, 0.0]), "{c:?}");
        let high = CylinderCollider { position: [0.0, 0.0, 1250.0], ..low.clone() };
        let c = high.sweep(&connor(), [1000.0, 0.0, 0.0], [0.0, 0.0, 0.0]).contact.expect("hit");
        assert_eq!(c.ball, 3, "{c:?}");
        // Coming in at a slant, the ball that reaches it first wins.
        let c = goblin(0x42).sweep(&connor(), [1500.0, 0.0, 1500.0], [500.0, 0.0, 500.0]).contact.expect("hit");
        assert_eq!(c.ball, 0, "{c:?}");
    }

    #[test]
    fn starting_inside_a_creature_is_no_hit_inside_anything_else_is() {
        // A creature (bit 2) or Connor (bit 4): he can walk out, or on.
        for kind in [0x42, 0x4, 0xC022] {
            let s = goblin(kind).sweep(&connor(), [500.0, 0.0, 0.0], [600.0, 0.0, 0.0]);
            assert_eq!(s.contact, None, "type {kind:#x}");
            assert_eq!(s.inside, vec![0, 1, 2, 3], "type {kind:#x}");
        }
        // The Wizard's type 0xC021 has neither: a hit where he stands,
        // the surface direction against his movement.
        let c = goblin(0xC021).sweep(&connor(), [500.0, 0.0, 0.0], [600.0, 0.0, 0.0]).contact.expect("hit");
        assert!(near(c.touch, [500.0, 0.0, 0.0]) && near(c.position, [500.0, 0.0, 0.0]), "{c:?}");
        assert!(near(c.normal, [-1.0, 0.0, 0.0]), "{c:?}");
    }

    #[test]
    fn tiny_moves_are_not_tested() {
        let s = goblin(0xC021).sweep(&connor(), [500.0, 0.0, 0.0], [500.0000001, 0.0, 0.0]);
        assert_eq!(s, CylinderSweep::default());
    }

    #[test]
    fn the_rough_check_covers_the_widened_cylinder() {
        let g = goblin(0x42);
        let stack = connor();
        let (lo, hi) = stack.reach([2000.0, 0.0, 0.0], [1100.0, 0.0, 0.0]);
        assert!(g.overlaps(stack.radius, lo, hi));
        let (lo, hi) = stack.reach([2000.0, 0.0, 0.0], [1300.0, 0.0, 0.0]);
        assert!(!g.overlaps(stack.radius, lo, hi));
        let (lo, hi) = stack.reach([0.0, 0.0, 1600.0], [0.0, 0.0, 1700.0]);
        assert!(!g.overlaps(stack.radius, lo, hi), "above it");
    }
}
