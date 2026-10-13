//! Emitters in the ground view (M3 step 3b, docs/DESIGN.md; rules:
//! docs/behaviour/emitters.md).
//!
//! - The particles live in `kq8_game::emitter` and move at 15 updates per
//!   second, after the doors and Connor.
//! - Particle catchers run after the emitters in the same update and hide
//!   their emitter's particles inside a building's box (stand-in for the
//!   order, Q48).
//! - Each particle is drawn as its emitter's `shape` (frame 0) at its
//!   position and turn, only while it is on screen (visible and age not 0),
//!   blended between updates except right after a restart.
//!
//! Everything is logged to `kq8-artifacts/logs/effects.log`: each emitter's
//! settings and, for `stream1` and `leaf1` (the note's "How to check"),
//! particle 0's choices, then per update its age, position and visibility,
//! and at each restart how its life compares with its first; each
//! catcher's box and every particle it hides.

use bevy::prelude::*;
use kq8_data::effects::CatcherSettings;
use kq8_game::collide::BuildingCollider;
use kq8_game::emitter::{Catcher, Emitter, Rng};

use crate::connor_view::ConnorSim;
use crate::game_camera::GameCam;
use crate::load::EmitterAssets;
use crate::log::Log;
use crate::terrain_view::bevy_transform_from;
use crate::{with_opacity, Shared};

/// Seed of the particles' random choices: the same every run, so two runs
/// give the same log.
const SEED: u64 = 0x4B51_3820_E417_7E25;

/// Emitters whose particle 0 is followed in the log (emitters.md "How to
/// check"); if a world has none of them, its first emitter.
const TRACKED: [&str; 2] = ["stream1", "leaf1"];

/// Per-update lines for a followed particle stop after this many updates
/// (the restart summaries go on).
const TRACK_UPDATES: u64 = 450;

/// A followed particle: its path in its first full life (by age) and the
/// numbers of the life now running.
struct Tracker {
    emitter: usize,
    first_life: Option<Vec<[f32; 3]>>,
    this_life: Vec<[f32; 3]>,
    lives: u32,
    life_ticks: u32,
    hidden_ticks: u32,
}

/// What a following emitter follows (emitters.md rule 11).
#[derive(Debug, Clone)]
pub enum Target {
    Connor,
    /// The game camera: position only (its turn is not described).
    Camera,
    /// A placed object that doesn't move: (position, angles).
    Fixed([f32; 3], [f32; 3]),
}

/// Finds a following emitter's target by name: what it is, its position
/// and turn now.
pub type TargetLookup<'a> = &'a dyn Fn(&str) -> Option<(Target, [f32; 3], [f32; 3])>;

struct DrawnEmitter {
    emitter: Emitter,
    target: Option<Target>,
    /// One entity per particle (its model's meshes are children).
    entities: Vec<Entity>,
    /// Positions before the last update, for blending.
    previous: Vec<[f32; 3]>,
}

/// A catcher with the index of its emitter, and how many particles it has
/// hidden so far.
struct ActiveCatcher {
    catcher: Catcher,
    emitter: usize,
    hidden: u64,
}

#[derive(Resource)]
pub struct EffectsSim {
    emitters: Vec<DrawnEmitter>,
    catchers: Vec<ActiveCatcher>,
    trackers: Vec<Tracker>,
    updates: u64,
    log: Log,
}

/// Builds the emitters and their particles' entities. `built` is each
/// object model's (mesh, material) list, indexed like `Objects::shapes`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_emitters(
    commands: &mut Commands,
    emitters: Vec<EmitterAssets>,
    catchers: Vec<(String, CatcherSettings)>,
    buildings: &[BuildingCollider],
    targets: TargetLookup,
    built: &[Vec<(Handle<Mesh>, Handle<StandardMaterial>)>],
    materials: &mut Assets<StandardMaterial>,
    main_log: &mut Log,
) {
    let mut log = match Log::create("effects") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("effects: cannot create effects.log: {e}; emitters not started"));
            return;
        }
    };
    let mut rng = Rng::new(SEED);
    let mut drawn = Vec::new();
    let (mut particles, mut not_started) = (0, Vec::new());
    for a in emitters {
        let s = &a.settings;
        let at = |g: Option<f32>| g.map_or("none".to_string(), |g| format!("{g:.0}"));
        let below = a.ground.iter().flatten().any(|g| a.position[2] < *g);
        let ground = format!("floor {} surface {}{}", at(a.ground[0]), at(a.ground[1]), if below { ", BELOW THE GROUND" } else { "" });
        let moved = if a.written != a.position { " CHANGED BY THE PLACEMENT RULES" } else { "" };
        let line = format!(
            "emitter [{}] shape {:?} at ({:.0}, {:.0}, {:.0}) (written ({:.0}, {:.0}, {:.0}){moved}; ground there: {}); radius {:?} force {:?}..{:?} gravity {:?} rot {:?}..{:?} maxSinPeriod {:?} maxLife {} maxParticles {} delayStart {} translucency {}{}",
            a.name,
            s.shape,
            a.position[0],
            a.position[1],
            a.position[2],
            a.written[0],
            a.written[1],
            a.written[2],
            ground,
            s.radius,
            s.force_min,
            s.force_max,
            s.gravity,
            s.rot_min,
            s.rot_max,
            s.max_sin_period,
            s.max_life,
            s.max_particles,
            s.delay_start,
            a.translucency,
            s.follow.as_ref().map_or(String::new(), |f| format!(" follow {f} followMesh {} followOffset {:?}", s.follow_mesh, s.follow_offset))
        );
        main_log.detail(&line);
        log.detail(line);
        let Some(shape) = a.shape else {
            not_started.push(format!("{} (no particle model)", a.name));
            continue;
        };
        // Following (rule 11): Connor, the camera or a placed object. The
        // note doesn't say what happens without a target: not started.
        let (emitter, target) = match &s.follow {
            None => (Emitter::new(&a.name, a.position, a.settings.clone(), &mut rng), None),
            Some(f) => match targets(f) {
                Some((target, at, angles)) => {
                    let e = Emitter::new_following(&a.name, at, angles, a.settings.clone(), &mut rng);
                    let line = format!(
                        "emitter [{}] follows {f} ({target:?}) from ({:.0}, {:.0}, {:.0}) turn {:?}: loc ({:.0}, {:.0}, {:.0}){}",
                        a.name,
                        at[0],
                        at[1],
                        at[2],
                        angles,
                        e.loc[0],
                        e.loc[1],
                        e.loc[2],
                        if s.follow_mesh { "; followMesh: restarts take the target's turn and a new velocity (its mesh name is not used, Q52)" } else { "" }
                    );
                    main_log.summary(format!("effects: {line}"));
                    log.detail(line);
                    (e, Some(target))
                }
                None => {
                    not_started.push(format!("{} (follows {f}: not in this world)", a.name));
                    continue;
                }
            },
        };
        // The emitter's translucency on every particle (likely, not traced:
        // emitters.md rule 9, open question 2).
        let opacity = if a.translucency >= 256.0 { 1.0 } else { (a.translucency / 255.0).clamp(0.0, 1.0) };
        let parts: Vec<(Handle<Mesh>, Handle<StandardMaterial>)> = built[shape]
            .iter()
            .map(|(mesh, material)| {
                let material = if opacity < 1.0 {
                    let original = materials.get(material).cloned().unwrap_or_default();
                    materials.add(with_opacity(&original, opacity))
                } else {
                    material.clone()
                };
                (mesh.clone(), material)
            })
            .collect();
        let mut entities = Vec::with_capacity(emitter.particles.len());
        for p in &emitter.particles {
            let transform = bevy_transform_from(kq8_data::objects::rotation(p.angles), p.position);
            let visibility = if p.drawn() { Visibility::Inherited } else { Visibility::Hidden };
            let parent = commands.spawn((transform, visibility)).id();
            for (mesh, material) in &parts {
                let child = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                commands.entity(parent).add_child(child);
            }
            entities.push(parent);
        }
        particles += emitter.particles.len();
        let previous = emitter.particles.iter().map(|p| p.position).collect();
        drawn.push(DrawnEmitter { emitter, target, entities, previous });
    }
    // Particle catchers: names looked up once, after the start-up scripts
    // (rule 2); any case. Nothing found: the catcher does nothing.
    let mut active_catchers = Vec::new();
    for (name, c) in catchers {
        let emitter = c.emitter.as_ref().and_then(|n| drawn.iter().position(|d| d.emitter.name.eq_ignore_ascii_case(n)));
        let building = c.interior.as_ref().and_then(|n| buildings.iter().find(|b| b.name.eq_ignore_ascii_case(n)));
        let line = match (emitter, building) {
            (Some(e), Some(b)) => {
                let catcher = Catcher::new(&name, b.bounds.0, b.bounds.1, b.placement);
                let line = format!(
                    "catcher [{name}]: emitter {} in building {} at ({:.0}, {:.0}, {:.0}); box (own space) {:?}..{:?}; top z {:.0}",
                    drawn[e].emitter.name,
                    b.name,
                    b.placement.position[0],
                    b.placement.position[1],
                    b.placement.position[2],
                    b.bounds.0,
                    b.bounds.1,
                    catcher.top
                );
                active_catchers.push(ActiveCatcher { catcher, emitter: e, hidden: 0 });
                line
            }
            _ => format!(
                "catcher [{name}]: does nothing: emitter {:?} {}, building {:?} {}",
                c.emitter,
                if emitter.is_some() { "found" } else { "not running" },
                c.interior,
                if building.is_some() { "found" } else { "not found" }
            ),
        };
        main_log.summary(format!("effects: {line}"));
        log.detail(line);
    }
    // The followed particles.
    let mut tracked: Vec<usize> =
        drawn.iter().enumerate().filter(|(_, d)| TRACKED.iter().any(|t| d.emitter.name.eq_ignore_ascii_case(t))).map(|(k, _)| k).collect();
    if tracked.is_empty() && !drawn.is_empty() {
        tracked.push(0);
    }
    let mut trackers = Vec::new();
    for k in tracked {
        let e = &drawn[k].emitter;
        match e.particles.first() {
            Some(p) => {
                log.detail(format!(
                    "follow [{}] particle 0: offset {:?} velocity {:?} spin {:?} sway speed {:?} starting age {} position {:?} visible {}",
                    e.name, p.offset, p.velocity, p.spin, p.sway_speed, p.age, p.position, p.visible
                ));
                trackers.push(Tracker { emitter: k, first_life: None, this_life: Vec::new(), lives: 0, life_ticks: 0, hidden_ticks: 0 });
            }
            None => log.detail(format!("follow [{}]: no particles", e.name)),
        }
    }
    main_log.summary(format!(
        "effects: {} emitters running, {particles} particles; not started: {not_started:?}; following particle 0 of {:?}; log in {}",
        drawn.len(),
        trackers.iter().map(|t| drawn[t.emitter].emitter.name.clone()).collect::<Vec<_>>(),
        log.path().display()
    ));
    commands.insert_resource(EffectsSim { emitters: drawn, catchers: active_catchers, trackers, updates: 0, log });
}

/// One update of every emitter (15 per second), particles in list order.
pub fn effects_step(sim: Option<ResMut<EffectsSim>>, shared: Res<Shared>, connor: Option<Res<ConnorSim>>, camera: Option<Res<GameCam>>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    for d in &mut sim.emitters {
        // Following emitters move with their target before the particles do.
        let at = match &d.target {
            None => None,
            Some(Target::Connor) => connor.as_ref().map(|c| (c.mover.position, [0.0, 0.0, c.walker.heading])),
            Some(Target::Camera) => camera.as_ref().map(|c| (c.position(), [0.0; 3])),
            Some(Target::Fixed(p, a)) => Some((*p, *a)),
        };
        if let Some((position, angles)) = at {
            d.emitter.follow_to(position, angles);
            if n % 15 == 0 && !matches!(d.target, Some(Target::Fixed(..))) {
                sim.log.detail(format!(
                    "update {n} [{}] follows ({:.0}, {:.0}, {:.0}) heading {:.3}: loc ({:.0}, {:.0}, {:.0})",
                    d.emitter.name, position[0], position[1], position[2], angles[2], d.emitter.loc[0], d.emitter.loc[1], d.emitter.loc[2]
                ));
            }
        }
        d.previous.clear();
        d.previous.extend(d.emitter.particles.iter().map(|p| p.position));
        d.emitter.tick();
    }
    // Catchers after the emitters, in the same update (stand-in, Q48).
    for c in &mut sim.catchers {
        let e = &mut sim.emitters[c.emitter].emitter;
        for k in c.catcher.catch(e) {
            c.hidden += 1;
            let p = &e.particles[k];
            sim.log.detail(format!(
                "update {n} catcher [{}] hides [{}] particle {k} at ({:.0}, {:.0}, {:.0}), age {} ({} hidden so far)",
                c.catcher.name, e.name, p.position[0], p.position[1], p.position[2], p.age, c.hidden
            ));
        }
    }
    for t in &mut sim.trackers {
        let e = &sim.emitters[t.emitter].emitter;
        let p = &e.particles[0];
        if n <= TRACK_UPDATES {
            sim.log.detail(format!(
                "update {n} [{}] particle 0: age {} position ({:.2}, {:.2}, {:.2}) angles ({:.3}, {:.3}, {:.3}) visible {} drawn {}{}",
                e.name,
                p.age,
                p.position[0],
                p.position[1],
                p.position[2],
                p.angles[0],
                p.angles[1],
                p.angles[2],
                p.visible,
                p.drawn(),
                if p.restarted { " RESTART" } else { "" }
            ));
        }
        t.life_ticks += 1;
        t.hidden_ticks += u32::from(!p.drawn());
        if p.restarted {
            // The life that just ended: complete only if it began with a restart.
            let line = if t.lives == 0 {
                format!("update {n} [{}] particle 0: first restart (its starting life was partial, {} updates)", e.name, t.life_ticks)
            } else {
                let path = std::mem::take(&mut t.this_life);
                let compare = match &t.first_life {
                    None => {
                        t.first_life = Some(path);
                        "first full life recorded".to_string()
                    }
                    Some(first) => {
                        let mut worst = [0.0f32; 3];
                        for (a, b) in first.iter().zip(&path) {
                            for k in 0..3 {
                                worst[k] = worst[k].max((a[k] - b[k]).abs());
                            }
                        }
                        format!("largest difference to the first full life by axis ({:.2}, {:.2}, {:.2})", worst[0], worst[1], worst[2])
                    }
                };
                format!(
                    "update {n} [{}] particle 0: life {} ended: {} updates, {} of them not drawn; {compare}",
                    e.name, t.lives, t.life_ticks, t.hidden_ticks
                )
            };
            shared.log.lock().unwrap().detail(format!("effects: {line}"));
            sim.log.detail(line);
            t.lives += 1;
            t.life_ticks = 0;
            t.hidden_ticks = 0;
            t.this_life.clear();
        } else if t.lives > 0 {
            t.this_life.push(p.position);
        }
    }
}

/// Each particle where it is, blended between the last two updates (not
/// across a restart), shown only while on screen.
pub fn effects_draw(
    sim: Option<Res<EffectsSim>>,
    fixed: Res<Time<Fixed>>,
    mut query: Query<(&mut Transform, &mut Visibility)>,
) {
    let Some(sim) = sim else { return };
    let a = fixed.overstep_fraction();
    for d in &sim.emitters {
        for ((p, e), before) in d.emitter.particles.iter().zip(&d.entities).zip(&d.previous) {
            let Ok((mut t, mut v)) = query.get_mut(*e) else { continue };
            let shown = if p.drawn() { Visibility::Inherited } else { Visibility::Hidden };
            if *v != shown {
                *v = shown;
            }
            if !p.drawn() {
                continue;
            }
            let at: [f32; 3] =
                if p.restarted { p.position } else { std::array::from_fn(|k| before[k] + (p.position[k] - before[k]) * a) };
            *t = bevy_transform_from(kq8_data::objects::rotation(p.angles), at);
        }
    }
}
