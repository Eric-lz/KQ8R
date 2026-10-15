//! Animated objects playing their movement in the ground view (M3 step 3c,
//! docs/DESIGN.md; rules: docs/behaviour/objects.md "Animated objects at
//! load", the phase rule of docs/behaviour/doors.md).
//!
//! - The loops live in `kq8_core::anim` and move at 15 updates per second.
//! - Each object is a body entity at its placement with one child per
//!   frame; only the shown frame is visible (as doors). If the loop has
//!   whole-object steps (frame transforms), the body is moved by the shown
//!   frame's chained steps, as doors are turned. The original adds the
//!   steps up and the data's cue puts the object back at each wrap; for
//!   Gnome's rolling rocks both give the same picture, so this stays
//!   (objects.md Q51 rules 1–7, checked by the user). Such an object's
//!   collider is moved to the same pose every update (Q51 rule 5, M3
//!   step 3r-4).
//! - Hidden and unused objects keep their H-key visibility; see-through
//!   objects keep their opacity.
//!
//! Log: `kq8-artifacts/logs/anim.log`.

use bevy::prelude::*;
use kq8_data::shape::Transform as GameTransform;
use kq8_core::anim::{pose_at_frame, LoopPlayer};
use kq8_core::emitter::Rng;

use crate::load::{AnimatedAssets, Animation};
use crate::log::Log;
use crate::terrain_view::{bevy_transform_from, ObjectColliders, ShownWithH};
use crate::model_mesh;
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};

/// Seed for the `flags` bit 8 random starts and speeds.
const SEED: u64 = 0x0A11_3C00_5EED_0001;

/// Objects whose phase and frame are logged every update (for a while).
const TRACKED: [&str; 4] = ["wellwatr", "windprop", "millwheel", "Davtorch.01"];
const TRACK_UPDATES: u64 = 120;

struct Playing {
    name: String,
    player: LoopPlayer,
    /// The object's placement (rotation, position).
    rotation: [[f32; 3]; 3],
    position: [f32; 3],
    /// Each frame's whole-object step; empty when no frame has one.
    steps: Vec<GameTransform>,
    body: Entity,
    frames: Vec<Entity>,
    shown: Option<usize>,
    tracked: bool,
    /// Its entry in `ObjectColliders` (only for objects with steps):
    /// `None` = not looked up yet, `Some(None)` = it has none.
    collider: Option<Option<usize>>,
}

#[derive(Resource)]
pub struct AnimSim {
    playing: Vec<Playing>,
    updates: u64,
    log: Log,
}

/// A step that does nothing (no move, no turn), within rounding.
fn is_still(t: &GameTransform) -> bool {
    let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    t.p.iter().all(|v| v.abs() < 0.001) && (0..3).all(|i| (0..3).all(|j| (t.m[i][j] - id[i][j]).abs() < 1e-5))
}

/// The pose `pose` (own space) followed by the placement: m = pose.m × R,
/// p = pose.p × R + position (row vectors).
fn combine(pose: &GameTransform, r: &[[f32; 3]; 3], position: [f32; 3]) -> ([[f32; 3]; 3], [f32; 3]) {
    let m = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| pose.m[i][k] * r[k][j]).sum()));
    let p = std::array::from_fn(|j| (0..3).map(|k| pose.p[k] * r[k][j]).sum::<f32>() + position[j]);
    (m, p)
}

impl AnimSim {
    /// `deleteObject` (3f): its collider is no longer moved.
    pub fn forget_collider(&mut self, name: &str) {
        for p in self.playing.iter_mut().filter(|p| p.name.eq_ignore_ascii_case(name)) {
            p.collider = Some(None);
        }
    }
}

/// Builds the playing objects. `placed` is `Objects::placed`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_animated(
    commands: &mut Commands,
    animated: Vec<AnimatedAssets>,
    animations: Vec<Animation>,
    placed: &[(kq8_data::objects::Placement, usize, bool)],
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let mut log = match Log::create("anim") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("animated objects: cannot create anim.log: {e}; not started"));
            return;
        }
    };
    // Per animation: its frames as (mesh, material index) lists, the
    // materials by index, its length and steps.
    struct Built {
        frames: Vec<Vec<(Handle<Mesh>, i32)>>,
        materials: Vec<(i32, Handle<ObjectMaterial>)>,
        duration: f32,
        steps: Vec<GameTransform>,
        moves: bool,
    }
    let built: Vec<Built> = animations
        .into_iter()
        .map(|a| {
            let shading: std::collections::BTreeMap<i32, crate::load::Shading> = a.looks.iter().map(|(m, _, s)| (*m, *s)).collect();
            let materials = a.looks.into_iter().map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)))).collect();
            let frames = a
                .frames
                .into_iter()
                .map(|frame| {
                    frame
                        .into_iter()
                        .map(|(m, triangles)| {
                            let l = shading.get(&m).map_or(kq8_data::Lighting::None, |s| s.lighting);
                            (meshes.add(model_mesh(&triangles, l, [0.0; 3])), m)
                        })
                        .collect()
                })
                .collect();
            let moves = a.frame_transforms.iter().skip(1).any(|t| !is_still(t));
            Built { frames, materials, duration: a.duration, steps: a.frame_transforms, moves }
        })
        .collect();
    let mut rng = Rng::new(SEED);
    let mut playing = Vec::new();
    let mut moving = Vec::new();
    let mut random = 0;
    for a in animated {
        let (p, _, unused) = &placed[a.placed];
        random += usize::from(a.random);
        let b = &built[a.animation];
        let opacity = p.opacity();
        let player = LoopPlayer::new(a.start, a.speed, a.repeat, b.frames.len(), b.duration, if a.random { Some(&mut rng) } else { None });
        let rotation = kq8_data::objects::rotation(p.angles);
        let mut body = commands.spawn((bevy_transform_from(rotation, p.position), Visibility::default(), crate::terrain_view::ObjectName(p.name.clone())));
        if *unused || !p.drawn() {
            body.insert((Visibility::Hidden, ShownWithH));
        }
        let body = body.id();
        let mut frames = Vec::with_capacity(b.frames.len());
        for frame in &b.frames {
            let parent = commands.spawn((Transform::default(), Visibility::Hidden)).id();
            commands.entity(body).add_child(parent);
            for (mesh, m) in frame {
                let Some((_, material)) = b.materials.iter().find(|(i, _)| i == m) else { continue };
                let material = crate::object_light::for_object(materials, material, opacity, p.haze_solid, p.haze_multiplier);
                let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material))).id();
                commands.entity(parent).add_child(part);
            }
            frames.push(parent);
        }
        // Whole-object steps: how far the chained pose goes over the loop.
        let steps = if b.moves { b.steps.clone() } else { Vec::new() };
        let mut reach = String::from("none");
        if b.moves {
            let (mut far, mut turn) = (0.0f32, 0.0f32);
            for k in 0..b.steps.len() {
                let pose = pose_at_frame(&b.steps, k);
                far = far.max(pose.p.iter().map(|v| v * v).sum::<f32>().sqrt());
                turn = turn.max(kq8_core::door::turn_degrees(&pose).abs());
            }
            let last = pose_at_frame(&b.steps, b.steps.len().saturating_sub(1));
            reach = format!(
                "up to {far:.1} units and {turn:.1} deg about z; at the last frame ({:.1}, {:.1}, {:.1}), {:.1} deg",
                last.p[0],
                last.p[1],
                last.p[2],
                kq8_core::door::turn_degrees(&last)
            );
            moving.push(p.name.clone());
        }
        log.detail(format!(
            "[{}] {} frames, {:.4} s, start {} speed {} repeat {} random {} -> phase {:.4} speed {:.4}, frame {}; whole-object steps: {reach}{}",
            p.name,
            b.frames.len(),
            b.duration,
            a.start,
            a.speed,
            a.repeat,
            a.random,
            player.phase,
            player.speed,
            player.frame(),
            if *unused || !p.drawn() { "; hidden (H)" } else { "" }
        ));
        let tracked = TRACKED.iter().any(|t| t.eq_ignore_ascii_case(&p.name));
        playing.push(Playing { name: p.name.clone(), player, rotation, position: p.position, steps, body, frames, shown: None, tracked, collider: None });
    }
    main_log.summary(format!(
        "animated objects: {} playing ({} with a random start and speed, flags bit 8), {} different loops; whole-object steps in {} objects {moving:?}; log in {}",
        playing.len(),
        random,
        built.len(),
        moving.len(),
        log.path().display()
    ));
    commands.insert_resource(AnimSim { playing, updates: 0, log });
}

/// One update (15 per second): every loop moves on, and an object whose
/// loop moves it takes its collider along (3r-4).
pub fn anim_step(sim: Option<ResMut<AnimSim>>, colliders: Option<ResMut<ObjectColliders>>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    let mut colliders = colliders;
    for p in &mut sim.playing {
        p.player.advance(1.0 / kq8_core::UPDATES_PER_SECOND as f32);
        if p.tracked && n <= TRACK_UPDATES {
            sim.log.detail(format!("update {n} [{}] phase {:.4} frame {}{}", p.name, p.player.phase, p.player.frame(), if p.player.finished { " finished" } else { "" }));
        }
        if p.steps.is_empty() {
            continue;
        }
        let Some(colliders) = colliders.as_mut() else { continue };
        let slot = *p.collider.get_or_insert_with(|| {
            let found: Vec<usize> = colliders.0.iter().enumerate().filter(|(_, c)| c.name == p.name).map(|(i, _)| i).collect();
            match found[..] {
                [i] => {
                    sim.log.detail(format!("[{}] its collider moves with it (objects.md Q51 rule 5)", p.name));
                    Some(i)
                }
                [] => {
                    sim.log.detail(format!("[{}] has no collider: nothing to move", p.name));
                    None
                }
                _ => {
                    sim.log.detail(format!("[{}] {} colliders by that name: none moved", p.name, found.len()));
                    None
                }
            }
        });
        let Some(i) = slot else { continue };
        let frame = p.player.frame().min(p.frames.len().saturating_sub(1));
        let (rotation, position) = combine(&pose_at_frame(&p.steps, frame), &p.rotation, p.position);
        colliders.0[i].placement = kq8_core::collide::Placement { rotation, position };
        if n % 15 == 0 && n <= 150 {
            sim.log.detail(format!(
                "update {n} [{}] frame {frame}: collider at ({:.0}, {:.0}, {:.0})",
                p.name, position[0], position[1], position[2]
            ));
        }
    }
}

/// Each object shows its current frame (and pose, if its loop moves it).
pub fn anim_draw(sim: Option<ResMut<AnimSim>>, mut transforms: Query<&mut Transform>, mut visibility: Query<&mut Visibility>) {
    let Some(mut sim) = sim else { return };
    for p in &mut sim.playing {
        let frame = p.player.frame().min(p.frames.len().saturating_sub(1));
        if p.shown == Some(frame) {
            continue;
        }
        if let Some(mut v) = p.shown.and_then(|k| visibility.get_mut(p.frames[k]).ok()) {
            *v = Visibility::Hidden;
        }
        if let Some(mut v) = p.frames.get(frame).and_then(|e| visibility.get_mut(*e).ok()) {
            *v = Visibility::Inherited;
        }
        p.shown = Some(frame);
        if !p.steps.is_empty() {
            let pose = pose_at_frame(&p.steps, frame);
            let (m, at) = combine(&pose, &p.rotation, p.position);
            if let Ok(mut t) = transforms.get_mut(p.body) {
                *t = bevy_transform_from(m, at);
            }
        }
    }
}
