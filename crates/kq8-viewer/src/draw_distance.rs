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
//! - Ball (rule 35): the shape's box (centre turned with the object, as
//!   the note's "Do not reproduce" asks), no shape: radius 100 at the
//!   position. STAND-IN (Q70): objects with a box collider use the same
//!   ball as the others, from the shape's box.
//! - Not done: the collider switch (bit 0x8; a far object is at least
//!   tooFar x visibleDistance from Connor, so he can't reach it anyway),
//!   `makeActive`, `setActiveCamera`. With no Connor (fly camera), the
//!   camera is the reference point (viewer only).
//! - F8 (haze off) switches this off too, so the whole world can be seen.

use std::collections::{BTreeMap, HashMap, HashSet};
use std::sync::atomic::{AtomicU32, Ordering};
use std::time::Instant;

use bevy::app::{HierarchyPropagatePlugin, Propagate, PropagateSet};
use bevy::camera::visibility::{RenderLayers, VisibilitySystems};
use bevy::prelude::*;
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
}

impl DistanceTest {
    /// For `placement`, with `ball` = the shape's (centre, radius) in game
    /// units, model space, or `None` for no shape (radius 100 at the
    /// position).
    pub fn new(placement: &Placement, ball: Option<([f32; 3], f32)>) -> Self {
        let (centre, radius) = match (placement.shape_named, ball) {
            (true, Some((c, r))) => (Vec3::from(crate::to_bevy(c, [0.0; 3])), r),
            _ => (Vec3::ZERO, NO_SHAPE_RADIUS),
        };
        DistanceTest {
            object: NEXT_OBJECT.fetch_add(1, Ordering::Relaxed),
            name: placement.name.clone(),
            centre,
            radius,
            too_far: placement.too_far,
            groups: [placement.groups_always, placement.groups_near, placement.groups_far],
        }
    }

    /// An item lying in the world (`items.md`): its type's model ball and
    /// the default `tooFar` and group masks (STAND-IN: the item types'
    /// `.kq` settings for these are not read).
    pub fn item(kind: &str, ball: Option<([f32; 3], f32)>) -> Self {
        let mut p = Placement::from_section(&kq8_data::kq_text::parse(b"[item]\n")[0]);
        p.name = kind.to_string();
        p.shape_named = ball.is_some();
        DistanceTest::new(&p, ball)
    }

    /// A copy for another object of the same kind (a new number).
    pub fn another(&self) -> Self {
        DistanceTest { object: NEXT_OBJECT.fetch_add(1, Ordering::Relaxed), ..self.clone() }
    }
}

/// Every model's bounding ball (rule 35), by shape name (lower case) and by
/// index in `Objects::shapes`.
#[derive(Resource, Default, Clone)]
pub struct ShapeBalls {
    by_name: HashMap<String, ([f32; 3], f32)>,
    pub by_index: Vec<([f32; 3], f32)>,
}

impl ShapeBalls {
    pub fn new<'a>(shapes: impl Iterator<Item = (&'a str, [f32; 3], [f32; 3])>) -> Self {
        let mut balls = ShapeBalls::default();
        for (name, min, max) in shapes {
            let ball = shape_ball(min, max);
            balls.by_name.entry(name.to_ascii_lowercase()).or_insert(ball);
            balls.by_index.push(ball);
        }
        balls
    }

    /// The test for a placed object, its ball found by its shape name.
    pub fn test(&self, placement: &Placement) -> DistanceTest {
        let ball = placement.shape.as_ref().and_then(|s| self.by_name.get(&s.to_ascii_lowercase()).copied());
        DistanceTest::new(placement, ball)
    }
}

/// The point distances are measured from (rule 34), in game units, for
/// the emitters; `None` while the test is off (F8).
#[derive(Resource, Default)]
pub struct ActivePoint {
    pub at: Option<[f32; 3]>,
    pub limit: f32,
}

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
        .init_resource::<ActivityLog>();
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
    mut log_state: ResMut<ActivityLog>,
    shared: Res<Shared>,
) {
    let from_connor = connor.iter().next();
    let Some(reference) = from_connor.or(camera.iter().next()).map(|t| t.translation) else { return };
    let limit = light.haze.visible;
    point.at = light.haze_on.then(|| game(reference));
    point.limit = limit;
    let mut seen = HashSet::new();
    let mut changes = Vec::new();
    let mut counts: BTreeMap<Activity, usize> = BTreeMap::new();
    let mut hidden_objects = 0;
    // (ball distance, position distance, name) of the nearest object that
    // is off and the farthest that is not.
    let mut nearest_off: Option<(f32, f32, String)> = None;
    let mut farthest_on: Option<(f32, f32, String)> = None;
    for (entity, test, transform, has_layer) in &tests {
        let centre = transform.transform_point(test.centre);
        let d = (reference - centre).length() * UNITS_PER_METRE - test.radius;
        let activity = Activity::of(d, limit, test.too_far);
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
        *counts.entry(activity).or_default() += 1;
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
    }
}

/// "[name] ball centre at D (position at P)" for the log.
fn describe(o: &Option<(f32, f32, String)>) -> String {
    match o {
        Some((ball, position, name)) => format!("[{name}] ball centre {ball:.0}, position {position:.0}"),
        None => "none".into(),
    }
}
