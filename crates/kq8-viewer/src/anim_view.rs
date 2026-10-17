//! Animated objects playing their movements in the ground view (M3 steps
//! 3c and 3i-4, docs/DESIGN.md; rules: docs/behaviour/objects.md
//! "Animated objects at load", docs/behaviour/anim-scripts.md).
//!
//! - Since 3i-4 the movements are played by `kq8_core` (`Active`'s
//!   `anims`, the same player as doors and chests), so scripts reach them
//!   (`KQAnimPlayEvent`, `pauseAnim`, `deleteObject`, `EndLoop`). This
//!   file only draws what core says. Unused objects (shown only with H)
//!   are not in the world's scripts: they keep a player of their own here.
//! - Each object is a body entity at its placement with one child per
//!   frame of every movement; only the shown frame is visible (as doors).
//!   If the shown movement has whole-object steps (frame transforms), the
//!   body is moved by the shown frame's chained steps, as doors are
//!   turned. The original adds the steps up and the data's cue puts the
//!   object back at each wrap; for Gnome's rolling rocks both give the
//!   same picture, so this stays (objects.md Q51 rules 1–7, checked by the
//!   user). Such an object's collider is moved to the same pose every
//!   update (Q51 rule 5, M3 step 3r-4).
//! - Hidden and unused objects keep their H-key visibility; see-through
//!   objects keep their opacity.
//!
//! Log: `kq8-artifacts/logs/anim.log`.

use bevy::prelude::*;
use kq8_core::active::AnimObject;
use kq8_core::anim::pose_at_frame;
use kq8_core::collide::Placement;
use kq8_core::door::{Cue, DoorMovement};
use kq8_core::emitter::Rng;
use kq8_core::named_anim::NamedAnimPlayer;
use kq8_data::shape::Transform as GameTransform;

use crate::door_view::DoorSim;
use crate::load::{AnimatedAssets, Animation};
use crate::log::Log;
use crate::model_mesh;
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::terrain_view::{bevy_transform_from, ObjectColliders, ShownWithH};

/// Seed for the `flags` bit 8 random starts and speeds.
const SEED: u64 = 0x0A11_3C00_5EED_0001;

/// Objects whose phase and frame are logged every update (for a while).
const TRACKED: [&str; 4] = ["wellwatr", "windprop", "millwheel", "Davtorch.01"];
const TRACK_UPDATES: u64 = 120;

/// Who plays an object's movements.
enum Source {
    /// `Active`'s `anims[i]` (the world's scripts reach it).
    Core(usize),
    /// Its own player (unused objects, shown only with H).
    Own(Box<NamedAnimPlayer>),
}

/// One movement as drawn: its frame entities and whole-object steps
/// (empty when no frame has one).
struct DrawnMovement {
    frames: Vec<Entity>,
    steps: Vec<GameTransform>,
}

struct Playing {
    name: String,
    source: Source,
    /// `script=` (for a copy brought in by `loadKQ`, M3 3k-4).
    script: Option<String>,
    /// The object's placement (rotation, position).
    rotation: [[f32; 3]; 3],
    position: [f32; 3],
    movements: Vec<DrawnMovement>,
    body: Entity,
    /// (movement, frame) shown.
    shown: Option<(usize, usize)>,
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
    /// A copy prepared for a script's `loadKQ` comes into the world (M3
    /// 3k-4): its player goes to `active`, which plays it from now on.
    /// `false` if no unused copy by that name plays here.
    pub fn bring_in(&mut self, name: &str, active: &mut kq8_core::active::Active) -> bool {
        let Some(p) = self.playing.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name) && matches!(p.source, Source::Own(_))) else {
            return false;
        };
        let index = active.world.anims.len();
        let Source::Own(player) = std::mem::replace(&mut p.source, Source::Core(index)) else { unreachable!() };
        active.add_anims([AnimObject { player: *player, script: p.script.clone(), gone: false }]);
        true
    }

    /// `deleteObject` (3f): its collider is no longer moved.
    pub fn forget_collider(&mut self, name: &str) {
        for p in self.playing.iter_mut().filter(|p| p.name.eq_ignore_ascii_case(name)) {
            p.collider = Some(None);
        }
    }
}

/// The player of `p`: in core, or its own.
fn player<'a>(p: &'a Playing, doors: Option<&'a DoorSim>) -> Option<&'a NamedAnimPlayer> {
    match &p.source {
        Source::Core(i) => doors.and_then(|d| d.active.world.anims.get(*i)).map(|a| &a.player),
        Source::Own(player) => Some(player),
    }
}

/// Builds the objects and returns the ones the world's scripts can reach,
/// in order, for `Active::add_anims` (3i-4). `placed` is
/// `Objects::placed`.
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
    balls: &crate::draw_distance::ShapeBalls,
    main_log: &mut Log,
) -> Vec<AnimObject> {
    let mut log = match Log::create("anim") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("animated objects: cannot create anim.log: {e}; not started"));
            return Vec::new();
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
        /// Each frame's lights (M1 7c, rule 26b).
        lights: std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>,
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
            Built { frames, materials, duration: a.duration, steps: a.frame_transforms, moves, lights: std::sync::Arc::new(a.frame_lights) }
        })
        .collect();
    let mut rng = Rng::new(SEED);
    let mut playing = Vec::new();
    let mut core = Vec::new();
    let mut moving = Vec::new();
    let (mut random, mut switchable) = (0, 0);
    for a in animated {
        let (p, shape_index, unused) = &placed[a.placed];
        random += usize::from(a.random);
        let opacity = p.opacity();
        let rotation = kq8_data::objects::rotation(p.angles);
        // The movements as core plays them (anim-scripts.md rules 1–10).
        let movements: Vec<DoorMovement> = a
            .movements
            .iter()
            .map(|(name, l, i)| DoorMovement {
                name: name.clone(),
                start: l.start,
                speed: l.speed,
                repeat: l.repeat,
                cut_short: l.unknown != 0,
                play_time: built[*i].duration,
                frame_steps: built[*i].steps.clone(),
                cues: l.cues.iter().map(|(phase, action)| Cue { phase: *phase, action: action.clone() }).collect(),
            })
            .collect();
        let names = (a.anim_names.0.as_deref(), a.anim_names.1.as_deref());
        let mut player = NamedAnimPlayer::new(&p.name, movements, a.current, Placement { rotation, position: p.position }).with_names_at_load(names.0, names.1);
        if a.random {
            player = player.with_random_start(&mut rng);
        }
        let mut body = commands.spawn((bevy_transform_from(rotation, p.position), Visibility::default(), crate::terrain_view::ObjectName(p.name.clone())));
        if *unused || !p.drawn() {
            body.insert((Visibility::Hidden, ShownWithH));
        }
        // Draw distance (M1 6h); unused objects are only shown with H.
        if !*unused {
            body.insert(balls.test_at(p, *shape_index));
            // The lights it carries (M1 7c), by the movement and frame shown.
            let lights = a.movements.iter().map(|(_, _, i)| built[*i].lights.clone()).collect();
            if let Some(c) = crate::scene_lights::CarriedLights::new(lights, bevy_transform_from(rotation, p.position), p.turn_off_dyn_lighting) {
                body.insert(c);
            }
        }
        let body = body.id();
        let mut drawn = Vec::with_capacity(a.movements.len());
        for (_, _, i) in &a.movements {
            let b = &built[*i];
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
            drawn.push(DrawnMovement { frames, steps: if b.moves { b.steps.clone() } else { Vec::new() } });
        }
        // Whole-object steps of the movement shown at load: how far the
        // chained pose goes over the loop.
        let b = &built[a.movements[player.current].2];
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
        let (_, l, _) = &a.movements[player.current];
        let more = if a.movements.len() > 1 {
            switchable += 1;
            format!("; movements {:?}", a.movements.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>())
        } else {
            String::new()
        };
        log.detail(format!(
            "[{}] {} frames, {:.4} s, start {} speed {} repeat {} random {} -> phase {:.4} speed {:.4}, frame {}; whole-object steps: {reach}{}{more}",
            p.name,
            b.frames.len(),
            b.duration,
            l.start,
            l.speed,
            l.repeat,
            a.random,
            player.phase,
            l.speed * player.speed_scale,
            player.frame(),
            if *unused || !p.drawn() { "; hidden (H)" } else { "" }
        ));
        let tracked = TRACKED.iter().any(|t| t.eq_ignore_ascii_case(&p.name));
        let source = if *unused {
            Source::Own(Box::new(player))
        } else {
            core.push(AnimObject { player, script: a.script.clone(), gone: false });
            Source::Core(core.len() - 1)
        };
        playing.push(Playing { name: p.name.clone(), source, script: a.script.clone(), rotation, position: p.position, movements: drawn, body, shown: None, tracked, collider: None });
    }
    main_log.summary(format!(
        "animated objects: {} playing ({} run by the world's scripts, {} with more than one movement, {} with a random start and speed, flags bit 8), {} different loops; whole-object steps in {} objects {moving:?}; log in {}",
        playing.len(),
        core.len(),
        switchable,
        random,
        built.len(),
        moving.len(),
        log.path().display()
    ));
    commands.insert_resource(AnimSim { playing, updates: 0, log });
    core
}

/// One update (15 per second), after `door_step` has run core's update:
/// unused objects move their own loops; an object whose shown movement
/// moves it takes its collider along (3r-4).
pub fn anim_step(sim: Option<ResMut<AnimSim>>, colliders: Option<ResMut<ObjectColliders>>, doors: Option<Res<DoorSim>>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    let doors = doors.as_deref();
    let mut colliders = colliders;
    for p in &mut sim.playing {
        if let Source::Own(player) = &mut p.source {
            player.advance(1.0 / kq8_core::UPDATES_PER_SECOND as f32);
        }
        let Some(player) = player(p, doors) else { continue };
        let (movement, frame) = (player.current, player.frame());
        if p.tracked && n <= TRACK_UPDATES {
            sim.log.detail(format!("update {n} [{}] phase {:.4} frame {}{}", p.name, player.phase, frame, if player.has_ended() { " finished" } else { "" }));
        }
        let Some(drawn) = p.movements.get(movement) else { continue };
        if drawn.steps.is_empty() {
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
        // Switched off (out of the collision world, or deleted): stays off.
        if colliders.0[i].min == colliders.0[i].max {
            continue;
        }
        let frame = frame.min(drawn.frames.len().saturating_sub(1));
        let (rotation, position) = combine(&pose_at_frame(&drawn.steps, frame), &p.rotation, p.position);
        colliders.0[i].placement = kq8_core::collide::Placement { rotation, position };
        if n % 15 == 0 && n <= 150 {
            sim.log.detail(format!(
                "update {n} [{}] frame {frame}: collider at ({:.0}, {:.0}, {:.0})",
                p.name, position[0], position[1], position[2]
            ));
        }
    }
}

/// Each object shows its current movement's frame (and pose, if that
/// movement moves it).
pub fn anim_draw(
    sim: Option<ResMut<AnimSim>>,
    doors: Option<Res<DoorSim>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut carried: Query<&mut crate::scene_lights::CarriedLights>,
) {
    let Some(mut sim) = sim else { return };
    let doors = doors.as_deref();
    for p in &mut sim.playing {
        let Some(player) = player(p, doors) else { continue };
        let movement = player.current.min(p.movements.len().saturating_sub(1));
        let Some(drawn) = p.movements.get(movement) else { continue };
        let frame = player.frame().min(drawn.frames.len().saturating_sub(1));
        if p.shown == Some((movement, frame)) {
            continue;
        }
        if let Ok(mut c) = carried.get_mut(p.body) {
            (c.movement, c.frame) = (movement, frame);
        }
        if let Some(mut v) = p.shown.and_then(|(m, k)| p.movements.get(m)?.frames.get(k)).and_then(|e| visibility.get_mut(*e).ok()) {
            *v = Visibility::Hidden;
        }
        if let Some(mut v) = drawn.frames.get(frame).and_then(|e| visibility.get_mut(*e).ok()) {
            *v = Visibility::Inherited;
        }
        p.shown = Some((movement, frame));
        let pose = if drawn.steps.is_empty() { kq8_core::door::identity() } else { pose_at_frame(&drawn.steps, frame) };
        // Only bodies that move or just stopped moving need a new pose.
        if !drawn.steps.is_empty() || p.movements.iter().any(|m| !m.steps.is_empty()) {
            let (m, at) = combine(&pose, &p.rotation, p.position);
            if let Ok(mut t) = transforms.get_mut(p.body) {
                *t = bevy_transform_from(m, at);
            }
        }
    }
}
