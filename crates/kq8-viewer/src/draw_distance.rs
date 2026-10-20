//! Objects near, far or off by their distance from Connor (M1 step 6h,
//! docs/DESIGN.md; rules: docs/behaviour/object-lighting.md rules 34-40,
//! rule numbers are that note's). See docs/provenance/README.md.
//!
//! - Every frame each object with a `DistanceTest` (placed objects,
//!   buildings, doors, chests, scripted and animated objects, items) is
//!   near, far or off by D = distance from Connor to the centre of its
//!   bounding ball, minus the ball's radius: off when D >= visibleDistance,
//!   far when D > tooFar x visibleDistance, else near (rules 35, 36).
//!   Emitters do the same in `effects_view` (radius 250 at their location).
//! - Its group bits are then groupsAlways plus groupsActive3D (near) or
//!   groupsActive3DFar (far), or only groupsAlways (off); it is drawn if
//!   any of 0x10, 0x40, 0x80, 0x100 is set (rule 37).
//! - An object that is not drawn goes on a drawing layer the camera does
//!   not show (Bevy's `RenderLayers`, passed down to its children), so this
//!   does not fight the systems that already show and hide objects.
//! - Ball (rule 35, Q70; M1 step 7a): always a `BOUNDS.KQ` box, never the
//!   model's geometry: the collision box (the shape of the current
//!   movement for animated objects, else `shapeName`, lower-cased), then
//!   the object's own name; no entry: radius 100 at the position. Worked
//!   out once, when the object is spawned. Centre turned with the object
//!   (our choice, the note's "Do not reproduce"). Buildings (rule 40,
//!   Q76; M1 7h): never `BOUNDS.KQ`; radius = half the diagonal of the
//!   `.dig` header box, centre = that box's middle turned with the building
//!   (the note's "Do not reproduce" recommendation; the original centres it
//!   on the building's position).
//! - Items (rule 43, Q71): their type's shape in `BOUNDS.KQ`, else 100;
//!   `tooFar` 0.83, `groupsAlways` 0x2, near 0x448, **far 0** (not drawn
//!   when far).
//! - The collider switch (rule 41, M1 step 7b): when an object's groups
//!   lose bit 0x8 (far or off with the default masks, or hidden: a hidden
//!   object is always off, rule 36) its collider leaves the collision
//!   world: Connor, the camera's view line and clicks pass through it;
//!   when they gain 0x8 it comes back. Only a change of state switches.
//!   The test runs every drawn frame; the switches are applied at the
//!   next game update (`apply_collider_switches`).
//! - `setActiveCamera` and `makeActive` (rule 42) arrive from the scripts
//!   as requests (`ActiveRules`), used from the next frame. With no Connor
//!   (fly camera), the camera is the reference point (viewer only).
//! - F8 (haze off) shows every object (so the whole world can be seen);
//!   the collider switch keeps working (it is a game rule).

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use bevy::app::{HierarchyPropagatePlugin, Propagate, PropagateSet};
use bevy::camera::visibility::{RenderLayers, VisibilitySystems};
use bevy::prelude::*;
use kq8_data::bounds::BoundingBoxes;
use kq8_data::objects::{shape_ball, Activity, Placement, DRAW_BITS, NO_SHAPE_RADIUS};

use crate::connor_view::ConnorBody;
use crate::object_light::ObjectLight;
use crate::{Shared, UNITS_PER_METRE};

/// The layer objects that are not drawn go to; cameras show layer 0 only.
const OFF_LAYER: usize = 31;

/// Gives each object one number, to count objects, not parts.
static NEXT_OBJECT: AtomicU32 = AtomicU32::new(0);

/// One object's distance test, on its root entity (rules 35-37).
#[derive(Component, Clone, Debug)]
pub struct DistanceTest {
    pub object: u32,
    pub name: String,
    /// Ball centre in the entity's own (Bevy model) space; radius in game
    /// units.
    pub centre: Vec3,
    pub radius: f32,
    pub too_far: f32,
    /// groupsAlways, groupsActive3D, groupsActive3DFar.
    pub groups: [u32; 3],
    /// Where the ball came from (for the log).
    pub ball_from: &'static str,
    /// `hidden=` from its file (always off, rule 36); scripted objects,
    /// chests and items are asked of the running game instead.
    pub hidden: bool,
    /// A character (M4 4d-3): core is told each frame whether it is near
    /// or far (`characters.md` rule 11).
    pub character: bool,
}

/// What the scripts asked of the distance test (rule 42).
#[derive(Resource, Default)]
pub struct ActiveRules {
    /// `setActiveCamera 1`: measure from the camera, not Connor.
    pub camera: bool,
    /// `makeActive <name> 1`: always near (lower case).
    pub always_near: HashSet<String>,
}

/// Collider switches seen by the test, for the next game update: name,
/// collider on (bit 0x8), the new state, its ball distance.
#[derive(Resource, Default)]
pub struct ColliderSwitches(pub Vec<(String, bool, Activity, f32)>);

/// Colliders taken out of the collision world, by lower-case name: their
/// slots in `ObjectColliders` with the boxes as they were, and their view
/// boxes, to put back.
#[derive(Resource, Default)]
pub struct SwitchedOut(HashMap<String, Saved>);

/// One switched-out object's boxes: (slot, box) in `ObjectColliders`, and
/// its view boxes.
type Saved = (Vec<(usize, kq8_core::collide::BoxCollider)>, Vec<kq8_core::collide::BoxCollider>);

/// A ball: centre (game units, model space) and radius.
type Ball = ([f32; 3], f32);

/// An item's fixed `tooFar` and masks (`object-lighting.md` rule 43).
pub const ITEM_TOO_FAR: f32 = 0.83;
/// groupsAlways (in the world), groupsActive3D, groupsActive3DFar.
pub const ITEM_GROUPS: [u32; 3] = [0x2, 0x448, 0];

impl DistanceTest {
    /// For `placement`, with `ball` = (centre, radius) in game units,
    /// model space, or `None` (radius 100 at the position).
    pub fn new(placement: &Placement, ball: Option<([f32; 3], f32)>, ball_from: &'static str) -> Self {
        let (centre, radius) = match ball {
            Some((c, r)) => (Vec3::from(crate::to_bevy(c, [0.0; 3])), r),
            None => (Vec3::ZERO, NO_SHAPE_RADIUS),
        };
        DistanceTest {
            object: NEXT_OBJECT.fetch_add(1, Ordering::Relaxed),
            name: placement.name.clone(),
            centre,
            radius,
            too_far: placement.too_far,
            groups: [placement.groups_always, placement.groups_near, placement.groups_far],
            ball_from,
            hidden: placement.hidden,
            character: false,
        }
    }

    /// A copy for another object of the same kind (a new number).
    pub fn another(&self) -> Self {
        DistanceTest { object: NEXT_OBJECT.fetch_add(1, Ordering::Relaxed), ..self.clone() }
    }
}

/// The bounding balls (rule 35): `BOUNDS.KQ` boxes, and buildings' balls
/// from their `.dig` header box (rule 40), by index in `Objects::shapes`.
#[derive(Resource, Default, Clone)]
pub struct ShapeBalls {
    table: Option<BoundingBoxes>,
    /// Each shape's name as first written, and its ball if it is a building
    /// (`.dig`): from the header box (a building `Model`'s min / max).
    shapes: Vec<(String, Option<Ball>)>,
}

impl ShapeBalls {
    /// `table`: `BOUNDS.KQ` (`None` if it could not be read: every object
    /// gets radius 100); `shapes`: name and model box of each shape (for a
    /// building, its `.dig` header box).
    pub fn new<'a>(table: Option<BoundingBoxes>, shapes: impl Iterator<Item = (&'a str, [f32; 3], [f32; 3])>) -> Self {
        let shapes = shapes.map(|(name, min, max)| (name.to_string(), name.to_ascii_lowercase().ends_with(".dig").then(|| shape_ball(min, max)))).collect();
        ShapeBalls { table, shapes }
    }

    /// A character's collision box (`characters.md` rules 8, 9): its
    /// `BOUNDS.KQ` entry by type, else by name (exact).
    pub fn character_box(&self, name: &str, box_type: Option<&str>) -> Option<([f32; 3], [f32; 3])> {
        let table = self.table.as_ref()?;
        box_type.and_then(|t| table.lookup(t)).or_else(|| table.lookup(name)).map(|e| (e.min, e.max))
    }

    /// The ball of a `BOUNDS.KQ` entry for `name` exactly.
    fn bounds(&self, name: &str) -> Option<([f32; 3], f32)> {
        self.table.as_ref()?.lookup(name).map(|e| shape_ball(e.min, e.max))
    }

    /// Rule 35: `shape` (the collision shape, lower-cased), then the own
    /// name; else none (radius 100).
    fn ball(&self, shape: Option<&str>, own: &str) -> (Option<([f32; 3], f32)>, &'static str) {
        if let Some(b) = shape.and_then(|s| self.bounds(&s.to_ascii_lowercase())) {
            return (Some(b), "BOUNDS.KQ by shape");
        }
        match self.bounds(own) {
            Some(b) => (Some(b), "BOUNDS.KQ by own name"),
            None => (None, "no entry: radius 100"),
        }
    }

    /// The test for a placed object drawn with shape `shape_index`
    /// (its collision shape: an animated object's current movement's).
    pub fn test_at(&self, placement: &Placement, shape_index: usize) -> DistanceTest {
        match self.shapes.get(shape_index) {
            Some((_, Some(building))) => DistanceTest::new(placement, Some(*building), "building .dig box (rule 40)"),
            Some((name, None)) => self.test_shape(placement, name),
            None => self.test(placement),
        }
    }

    /// The test for an object whose collision shape is `shape`.
    pub fn test_shape(&self, placement: &Placement, shape: &str) -> DistanceTest {
        let (ball, from) = self.ball(Some(shape), &placement.name);
        DistanceTest::new(placement, ball, from)
    }

    /// The test for a character (characters.md rule 9): its box by
    /// `box_type` (the monster list's type, else `monsterType`), else by
    /// its name, exact.
    pub fn test_character(&self, placement: &Placement, box_type: Option<&str>) -> DistanceTest {
        let mut test = if let Some(b) = box_type.and_then(|t| self.bounds(t)) {
            DistanceTest::new(placement, Some(b), "BOUNDS.KQ by the character's type")
        } else {
            match self.bounds(&placement.name) {
                Some(b) => DistanceTest::new(placement, Some(b), "BOUNDS.KQ by the character's name"),
                None => DistanceTest::new(placement, None, "character without a box: radius 100"),
            }
        };
        test.character = true;
        test
    }

    /// The test for a placed object by its `shapeName`.
    pub fn test(&self, placement: &Placement) -> DistanceTest {
        let (ball, from) = self.ball(placement.shape.as_deref(), &placement.name);
        DistanceTest::new(placement, ball, from)
    }

    /// An item lying in the world (rule 43): its type's shape in
    /// `BOUNDS.KQ` (items have no name of their own there), fixed `tooFar`
    /// and masks.
    pub fn item(&self, kind: &str, shape: &str) -> DistanceTest {
        let ball = self.bounds(&shape.to_ascii_lowercase());
        let from = if ball.is_some() { "BOUNDS.KQ by the item type's shape" } else { "no entry: radius 100" };
        let mut p = Placement::from_section(&kq8_data::kq_text::parse(b"[item]\n")[0]);
        p.name = kind.to_string();
        p.too_far = ITEM_TOO_FAR;
        [p.groups_always, p.groups_near, p.groups_far] = ITEM_GROUPS;
        DistanceTest::new(&p, ball, from)
    }
}

/// The point distances are measured from (rule 34), in game units, for
/// the emitters; `None` while the test is off (F8).
#[derive(Resource, Default)]
pub struct ActivePoint {
    pub at: Option<[f32; 3]>,
    pub limit: f32,
}

/// The characters near or far in the last frame (lower case), for core
/// (M4 4d-3, `characters.md` rule 11).
#[derive(Resource, Default)]
pub struct CharactersInRange(pub std::collections::BTreeSet<String>);

/// What `update_activity` saw, for the log.
#[derive(Resource, Default)]
pub struct ActivityLog {
    last: Option<Instant>,
}

/// Registers the propagation of the drawing layer to children, before
/// Bevy decides what is visible.
pub fn add(app: &mut App) {
    app.add_plugins(HierarchyPropagatePlugin::<RenderLayers>::new(PostUpdate))
        .configure_sets(PostUpdate, PropagateSet::<RenderLayers>::default().before(VisibilitySystems::CheckVisibility))
        .init_resource::<ActivePoint>()
        .init_resource::<ActivityLog>()
        .init_resource::<CharactersInRange>()
        .init_resource::<ActiveRules>()
        .init_resource::<ColliderSwitches>()
        .init_resource::<SwitchedOut>();
}

fn game(p: Vec3) -> [f32; 3] {
    let g = p * UNITS_PER_METRE;
    [g.x, -g.z, g.y]
}

/// Every frame: near / far / off for every object, and the drawing layer
/// of those not drawn (rules 34-37). Logs every change of state, and once
/// a second how many objects are in each state.
#[allow(clippy::type_complexity)]
#[allow(clippy::too_many_arguments)]
pub fn update_activity(
    mut commands: Commands,
    light: Res<ObjectLight>,
    // Connor's body and the cameras are roots too: their own transforms.
    connor: Query<&Transform, With<ConnorBody>>,
    camera: Query<&Transform, With<Camera3d>>,
    // Every tested entity is a root (no parent), so its own transform is
    // its place in the world, already right on the frame it is spawned.
    tests: Query<(Entity, &DistanceTest, &Transform, Has<Propagate<RenderLayers>>)>,
    layered: Query<(), (With<Mesh3d>, With<RenderLayers>)>,
    mut point: ResMut<ActivePoint>,
    mut states: Local<HashMap<u32, Activity>>,
    mut solid: Local<HashMap<u32, bool>>,
    mut log_state: ResMut<ActivityLog>,
    rules: Res<ActiveRules>,
    mut switches: ResMut<ColliderSwitches>,
    sim: Option<Res<crate::door_view::DoorSim>>,
    mut lights_group: ResMut<crate::scene_lights::LightsGroup>,
    shared: Res<Shared>,
    mut characters: ResMut<CharactersInRange>,
) {
    lights_group.0.clear();
    characters.0.clear();
    // Rule 42: Connor unless a script asked for the camera.
    let from_connor = if rules.camera { None } else { connor.iter().next() };
    let Some(reference) = from_connor.or(camera.iter().next()).map(|t| t.translation) else { return };
    let limit = light.haze.visible;
    point.at = light.haze_on.then(|| game(reference));
    point.limit = limit;
    let mut seen = HashSet::new();
    let mut changes = Vec::new();
    let mut counts: BTreeMap<Activity, usize> = BTreeMap::new();
    let mut hidden_objects = 0;
    let mut sources: BTreeMap<&'static str, usize> = BTreeMap::new();
    // (ball distance, position distance, name) of the nearest object that
    // is off and the farthest that is not.
    let mut nearest_off: Option<(f32, f32, String)> = None;
    let mut farthest_on: Option<(f32, f32, String)> = None;
    for (entity, test, transform, has_layer) in &tests {
        let centre = transform.transform_point(test.centre);
        let d = (reference - centre).length() * UNITS_PER_METRE - test.radius;
        // Rule 36: hidden = off; makeActive = near; else by distance.
        let hidden = sim.as_ref().map_or(test.hidden, |s| s.active.world.hidden_now(&test.name, test.hidden));
        let activity = if hidden {
            Activity::Off
        } else if rules.always_near.contains(&test.name.to_ascii_lowercase()) {
            Activity::Near
        } else {
            Activity::of(d, limit, test.too_far)
        };
        if test.character && activity != Activity::Off {
            characters.0.insert(test.name.to_ascii_lowercase());
        }
        let [always, near, far] = test.groups;
        let drawn = !light.haze_on || activity.groups(always, near, far) & DRAW_BITS != 0;
        if !drawn && !has_layer {
            commands.entity(entity).insert(Propagate(RenderLayers::layer(OFF_LAYER)));
        } else if drawn && has_layer {
            commands.entity(entity).remove::<Propagate<RenderLayers>>();
        }
        if !seen.insert(test.object) {
            continue;
        }
        // Rule 41: the collider follows bit 0x8; colliders exist at load,
        // so only a loss or a later gain is a switch.
        let has_collider = activity.groups(always, near, far) & COLLIDER_BIT != 0;
        // Rule 37: its own lights shine while it has bit 0x400 (M1 7c).
        if activity.groups(always, near, far) & crate::scene_lights::LIGHTS_BIT != 0 {
            lights_group.0.insert(test.object);
        }
        let was = solid.insert(test.object, has_collider);
        if was.unwrap_or(true) != has_collider {
            switches.0.push((test.name.clone(), has_collider, activity, d + test.radius));
        }
        *counts.entry(activity).or_default() += 1;
        *sources.entry(test.ball_from).or_default() += 1;
        hidden_objects += usize::from(!drawn);
        let to_position = (reference - transform.translation).length() * UNITS_PER_METRE;
        let entry = Some((d + test.radius, to_position, test.name.clone()));
        if activity == Activity::Off {
            if nearest_off.as_ref().is_none_or(|n| d + test.radius < n.0) {
                nearest_off = entry;
            }
        } else if farthest_on.as_ref().is_none_or(|n| d + test.radius > n.0) {
            farthest_on = entry;
        }
        let before = states.insert(test.object, activity);
        if before.is_none() && activity != Activity::Off && !drawn {
            changes.push(format!(
                "draw distance: [{}] {activity:?} but not drawn: groups always {:#x} near {:#x} far {:#x}",
                test.name, always, near, far
            ));
        }
        if before.is_some_and(|b| b != activity) {
            changes.push(format!(
                "draw distance: [{}] {:?} -> {activity:?} ({}); ball distance {:.0} (radius {:.0}), position {to_position:.0} from {}",
                test.name,
                before.unwrap(),
                if drawn { "drawn" } else { "not drawn" },
                d + test.radius,
                test.radius,
                if from_connor.is_some() { "Connor" } else { "the camera" }
            ));
        }
    }
    let now = Instant::now();
    let second = log_state.last.is_none_or(|t| now.duration_since(t).as_secs_f32() >= 1.0);
    if changes.is_empty() && !second {
        return;
    }
    let mut log = shared.log.lock().unwrap();
    for c in changes {
        log.detail(c);
    }
    if second {
        log_state.last = Some(now);
        let r = game(reference);
        log.detail(format!(
            "draw distance {}: from {} ({:.0}, {:.0}, {:.0}), limit {limit:.0}: of {} objects {} near, {} far, {} off; {hidden_objects} not drawn; {} meshes on the hidden layer; nearest off {}; farthest near/far {}",
            if light.haze_on { "on" } else { "off (F8)" },
            if from_connor.is_some() { "Connor" } else { "the camera" },
            r[0],
            r[1],
            r[2],
            seen.len(),
            counts.get(&Activity::Near).unwrap_or(&0),
            counts.get(&Activity::Far).unwrap_or(&0),
            counts.get(&Activity::Off).unwrap_or(&0),
            layered.iter().count(),
            describe(&nearest_off),
            describe(&farthest_on)
        ));
        log.detail(format!("draw distance: balls by source {sources:?}"));
    }
}

/// The group bit that puts an object's collider in the collision world
/// (rule 37).
pub const COLLIDER_BIT: u32 = 0x8;

/// At each game update, before the doors and Connor: applies the collider
/// switches the test saw (rule 41). Out: the object's boxes in
/// `ObjectColliders` are switched off (size 0, far below) and its view
/// boxes dropped, and the running game no longer clicks it; back: they
/// are put back as they were (doors, chests and moving objects are moved
/// to where they stand by their own update). A deleted object stays out.
pub fn apply_collider_switches(
    mut switches: ResMut<ColliderSwitches>,
    mut out: ResMut<SwitchedOut>,
    sim: Option<ResMut<crate::door_view::DoorSim>>,
    colliders: Option<ResMut<crate::terrain_view::ObjectColliders>>,
    view: Option<ResMut<crate::terrain_view::ViewBlockerSet>>,
    shared: Res<Shared>,
) {
    if switches.0.is_empty() {
        return;
    }
    let (mut sim, mut colliders, mut view) = (sim, colliders, view);
    let updates = sim.as_ref().map_or(0, |s| s.active.world.updates);
    let mut lines = Vec::new();
    for (name, on, activity, distance) in std::mem::take(&mut switches.0) {
        let key = name.to_ascii_lowercase();
        if let Some(s) = sim.as_mut() {
            if on && s.active.world.is_deleted(&name) {
                continue;
            }
            s.active.switch_collider(&name, on);
        }
        let (boxes, views) = if on {
            let (boxes, views) = out.0.remove(&key).unwrap_or_default();
            if let Some(c) = colliders.as_mut() {
                for (i, b) in &boxes {
                    c.0[*i] = b.clone();
                }
            }
            let n = views.len();
            if let Some(v) = view.as_mut() {
                v.boxes.extend(views);
            }
            (boxes.len(), n)
        } else {
            let mut saved = Vec::new();
            if let Some(c) = colliders.as_mut() {
                for (i, b) in c.0.iter_mut().enumerate().filter(|(_, b)| b.name.eq_ignore_ascii_case(&name) && b.min != b.max) {
                    saved.push((i, b.clone()));
                    crate::door_view::switch_off(b);
                }
            }
            let mut views = Vec::new();
            if let Some(v) = view.as_mut() {
                let (gone, kept): (Vec<_>, Vec<_>) = std::mem::take(&mut v.boxes).into_iter().partition(|b| b.name.eq_ignore_ascii_case(&name));
                v.boxes = kept;
                views = gone;
            }
            let counts = (saved.len(), views.len());
            out.0.entry(key).or_default().0.extend(saved);
            out.0.get_mut(&name.to_ascii_lowercase()).expect("just made").1.extend(views);
            counts
        };
        lines.push(format!(
            "update {updates}: collider switch: [{name}] {activity:?} at ball distance {distance:.0}: collider {} ({boxes} box(es) for Connor, {views} view box(es){})",
            if on { "back" } else { "out" },
            if sim.is_some() { ", clicks follow" } else { "" }
        ));
    }
    let mut log = shared.log.lock().unwrap();
    for l in lines {
        log.detail(l);
    }
}

/// "[name] ball centre at D (position at P)" for the log.
fn describe(o: &Option<(f32, f32, String)>) -> String {
    match o {
        Some((ball, position, name)) => format!("[{name}] ball centre {ball:.0}, position {position:.0}"),
        None => "none".into(),
    }
}
