//! Shadows drawn (M4 steps 4d-7b-2 and 4d-7b-4): the meshes, their
//! material, and plain objects' shadows. Characters' and Connor's are cast
//! in `anim_view::shadow_draw`; the geometry is `kq8_core::shadow`.
//!
//! Rules from `docs/behaviour/characters.md` rules 36 and 37 ("Plain
//! objects", Q123 d): a plain object casts when (the world's mask) AND (its
//! `shadowMask`, default 8) is not 0 and it is solid, only while it is
//! drawn (near or far, not hidden); the same direction, 20 lift and 0.5 as
//! characters. Buildings only receive. `allowShadows` is taken as on (as
//! for characters; Daventry's `world.cs` says 1).
//!
//! Ours: a plain object's shadow is worked out again only when what is
//! shown of it (its parts and its place) changes, so a still object costs
//! nothing after its first frame. The terrain search uses its model's
//! points (as for characters, 4d-7b-2).
//!
//! Buildings receive (M4 4d-7b-3, rule 37 (2)): [`BuildingShadows`] holds
//! each building copy's drawn surfaces; [`receivers`] adds those of the
//! near and far ones that `kq8_core::shadow::building_receivers` picks to
//! the terrain's, for characters, Connor and plain objects alike.

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use kq8_core::shadow;

use crate::draw_distance::{DistanceTest, ObjectActivity};
use crate::log::Log;
use crate::object_light::ObjectLight;

/// The world's shadow mask (`characters.md` rule 36).
#[derive(Resource)]
pub struct ShadowWorld(pub u32);

/// The ball stored in a model's `.kqs` file (`characters.md` rule 37 (2),
/// Q135; M4 4d-7b-5b): radius and centre in the model's own space, game
/// units. On each shown frame entity (and each still object's part),
/// whose place is the model's own space in the world.
#[derive(Component, Clone, Copy, Debug)]
pub struct ModelBall {
    pub radius: f32,
    pub centre: [f32; 3],
}

impl ModelBall {
    pub fn of((radius, centre): (f32, [f32; 3])) -> Self {
        ModelBall { radius, centre }
    }
}

/// Rule 37 (2): the caster of a shown model whose own space is placed at
/// `at`: origin and axes (turned with its heading) in the game's world.
pub fn caster(at: &GlobalTransform, ball: &ModelBall) -> shadow::Caster {
    // Bevy (x, y, z) = game (x, z, −y) / 100.
    let o = at.translation() * 100.0;
    let axis = |e: [f32; 3]| {
        let v = at.affine().transform_vector3(Vec3::new(e[0], e[2], -e[1])).normalize_or_zero();
        [v.x, -v.z, v.y]
    };
    shadow::Caster { origin: [o.x, -o.z, o.y], axes: [axis([1.0, 0.0, 0.0]), axis([0.0, 1.0, 0.0]), axis([0.0, 0.0, 1.0])], centre: ball.centre, radius: ball.radius }
}

/// Every building copy that can receive shadows (M4 4d-7b-3): its draw
/// distance number (`DistanceTest::object`) and its drawn surfaces.
#[derive(Resource, Default)]
pub struct BuildingShadows(pub Vec<(u32, shadow::ReceivingBuilding)>);

impl BuildingShadows {
    /// Those near or far in the last draw-distance test (rule 37 (2):
    /// "active"); with no test yet, none.
    pub fn active<'a>(&'a self, states: Option<&ObjectActivity>) -> Vec<(u32, &'a shadow::ReceivingBuilding)> {
        let Some(states) = states else { return Vec::new() };
        self.0.iter().filter(|(id, _)| states.0.get(id).is_some_and(|a| *a != kq8_data::objects::Activity::Off)).map(|(id, b)| (*id, b)).collect()
    }
}

/// What a caster's shadow lies on (rule 37), for the logs too.
pub struct Receivers {
    pub triangles: Vec<[shadow::V3; 3]>,
    /// How many of `triangles` (the first ones) are the terrain's.
    pub terrain: usize,
    /// The buildings' pick (its triangles moved into `triangles`).
    pub pick: shadow::BuildingPick,
    /// No stored ball: the box around its points stood in (ours).
    pub fallback: bool,
}

/// Rule 37: the terrain's triangles (within 1,500 of the ground) and those
/// of the `buildings` (active ones) that `shadow::building_receivers`
/// picks for `caster` (its model's stored ball; without one, our fallback:
/// the box around `corners`, the shown model's points).
pub fn receivers(ground: &kq8_data::Terrain, position: shadow::V3, corners: &[shadow::V3], dir: shadow::V3, caster: Option<shadow::Caster>, buildings: &[(u32, &shadow::ReceivingBuilding)]) -> Receivers {
    let mut out = Receivers { triangles: Vec::new(), terrain: 0, pick: shadow::BuildingPick::default(), fallback: caster.is_none() };
    if corners.is_empty() {
        return out;
    }
    out.triangles = shadow::terrain_receivers(ground, position, corners, dir);
    out.terrain = out.triangles.len();
    let Some(caster) = caster.or_else(|| shadow::Caster::around(corners)) else { return out };
    out.pick = shadow::building_receivers(buildings.iter().map(|(_, b)| *b), &caster, dir);
    out.triangles.append(&mut out.pick.triangles);
    out
}

/// The stored ball of what is shown under `root` (itself first, then its
/// shown children), as a caster.
fn shown_caster(root: Entity, children: &Query<&Children>, visibility: &Query<&Visibility>, balls: &Query<(&GlobalTransform, &ModelBall)>) -> Option<shadow::Caster> {
    if visibility.get(root).is_ok_and(|v| *v == Visibility::Hidden) {
        return None;
    }
    if let Ok((at, ball)) = balls.get(root) {
        return Some(caster(at, ball));
    }
    children.get(root).into_iter().flat_map(|c| c.iter()).find_map(|c| shown_caster(c, children, visibility, balls))
}

/// The shadows' look: unlit, multiply blending at the strength 0.5 (each
/// layer halves what is under it), no haze (rule 37).
pub fn shadow_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    let v = shadow::STRENGTH;
    materials.add(StandardMaterial {
        // Half of the brightness as shown (sRGB), as an 8-bit screen
        // halves it; Bevy blends in linear light, where 0.5 would only
        // darken to about 73 % (the user's check 2026-10-21: too light).
        base_color: Color::srgb(v, v, v),
        alpha_mode: AlphaMode::Multiply,
        unlit: true,
        cull_mode: None,
        fog_enabled: false,
        ..default()
    })
}

/// The triangles of a shown model frame (`frame`'s children), in the
/// game's space and corner order (Bevy's run anticlockwise, the game's
/// clockwise: (a, c, b)).
pub fn frame_triangles(frame: Entity, children: &Query<&Children>, parts: &Query<(&GlobalTransform, &Mesh3d)>, meshes: &Assets<Mesh>, sources: &FrameSources) -> Vec<[shadow::V3; 3]> {
    let mut model = Vec::new();
    for part in children.get(frame).into_iter().flat_map(|c| c.iter()) {
        mesh_triangles(part, parts, meshes, sources, &mut model);
    }
    model
}

/// Animation frames' meshes kept only on the graphics card ("Animation
/// memory" step 1, DESIGN): where each came from, so a shadow can build
/// its points again from the parsed model: the same `Shape::geometry` the
/// loader cut the frame from, the same corners in the same order. A few
/// thousand recent frames' points are kept.
#[derive(Resource, Default)]
pub struct FrameSources {
    /// Per mesh: its model, sequence and material.
    sources: HashMap<AssetId<Mesh>, FrameSource>,
    cache: Mutex<HashMap<AssetId<Mesh>, Points>>,
    /// Frames' points built, and the time spent, for the log.
    pub built: AtomicU64,
    pub build_ns: AtomicU64,
}

type FrameSource = (crate::load::FrameShape, usize, i32);
/// A frame's points in Bevy's space.
type Points = Arc<Vec<[f32; 3]>>;

/// Recent frames' points kept (each a few hundred triangles).
const KEPT_FRAMES: usize = 4096;

impl FrameSources {
    /// Adds an animation frame's mesh: with its source, kept only on the
    /// graphics card; without (not cut from a model), as before.
    pub fn add(&mut self, meshes: &mut Assets<Mesh>, mut mesh: Mesh, source: Option<&crate::load::FrameShape>, frame: usize, material: i32) -> Handle<Mesh> {
        let Some(source) = source else { return meshes.add(mesh) };
        mesh.asset_usage = bevy::asset::RenderAssetUsages::RENDER_WORLD;
        let handle = meshes.add(mesh);
        self.sources.insert(handle.id(), (source.clone(), source.first + frame, material));
        handle
    }

    /// Meshes recorded (for the log).
    pub fn len(&self) -> usize {
        self.sources.len()
    }

    /// The points of a recorded mesh, in Bevy's space and the mesh's
    /// corner order (as `main.rs` `model_mesh` put them).
    fn points(&self, id: AssetId<Mesh>) -> Option<Points> {
        let (source, sequence, material) = self.sources.get(&id)?;
        let mut cache = self.cache.lock().unwrap();
        if let Some(p) = cache.get(&id) {
            return Some(p.clone());
        }
        let started = std::time::Instant::now();
        let geometry = source.shape.geometry(*sequence);
        let points: Vec<[f32; 3]> = geometry.triangles.iter().filter(|t| t.material == *material).flat_map(|t| [0, 2, 1].map(|k| crate::to_bevy(t.positions[k], [0.0; 3]))).collect();
        self.built.fetch_add(1, Ordering::Relaxed);
        self.build_ns.fetch_add(started.elapsed().as_nanos() as u64, Ordering::Relaxed);
        if cache.len() >= KEPT_FRAMES {
            cache.clear();
        }
        let points = Arc::new(points);
        cache.insert(id, points.clone());
        Some(points)
    }

    /// "N frames' points built for shadows, M ms in all".
    pub fn line(&self) -> String {
        let (n, ns) = (self.built.load(Ordering::Relaxed), self.build_ns.load(Ordering::Relaxed));
        format!("{n} frames' points built for shadows from the models, {:.1} ms in all ({:.3} ms each)", ns as f64 / 1e6, if n == 0 { 0.0 } else { ns as f64 / 1e6 / n as f64 })
    }
}

/// One mesh entity's triangles in the game's space and corner order.
fn mesh_triangles(part: Entity, parts: &Query<(&GlobalTransform, &Mesh3d)>, meshes: &Assets<Mesh>, sources: &FrameSources, model: &mut Vec<[shadow::V3; 3]>) {
    let Ok((at, handle)) = parts.get(part) else { return };
    let Some(mesh) = meshes.get(&handle.0) else { return };
    // Step 1: an animation frame's points from its model; others' from
    // the mesh.
    let built;
    let points: &[[f32; 3]] = if mesh.asset_usage.contains(bevy::asset::RenderAssetUsages::MAIN_WORLD) {
        let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
        points
    } else {
        let Some(p) = sources.points(handle.id()) else { return };
        built = p;
        &built
    };
    // Bevy (x, y, z) = game (x, z, −y) / 100, so back: (x, −z, y) × 100.
    let game: Vec<shadow::V3> = points.iter().map(|&q| at.transform_point(Vec3::from(q))).map(|v| [v.x * 100.0, -v.z * 100.0, v.y * 100.0]).collect();
    let corners: Vec<usize> = match mesh.try_indices_option().ok().flatten() {
        Some(ix) => ix.iter().collect(),
        None => (0..game.len()).collect(),
    };
    model.extend(corners.as_chunks::<3>().0.iter().map(|t| [game[t[0]], game[t[2]], game[t[1]]]));
}

/// The mesh entities shown under `root` (not under a hidden entity).
fn shown_meshes(root: Entity, children: &Query<&Children>, visibility: &Query<&Visibility>, parts: &Query<(&GlobalTransform, &Mesh3d)>, out: &mut Vec<Entity>) {
    if visibility.get(root).is_ok_and(|v| *v == Visibility::Hidden) {
        return;
    }
    if parts.contains(root) {
        out.push(root);
    }
    for c in children.get(root).into_iter().flat_map(|c| c.iter()) {
        shown_meshes(c, children, visibility, parts, out);
    }
}

/// A shadow's entity: its mesh, and whether it is shown.
pub struct ShadowSlot {
    entity: Entity,
    mesh: Handle<Mesh>,
    shown: bool,
}

/// Puts `cast` into the shadow `slot` (made on first use). With nothing to
/// draw the entity is hidden and its mesh left as it was: an empty mesh
/// makes Bevy warn every frame.
pub fn show_shadow(cast: &[[shadow::V3; 3]], slot: &mut Option<ShadowSlot>, commands: &mut Commands, meshes: &mut Assets<Mesh>, material: &Handle<StandardMaterial>) {
    if cast.is_empty() {
        if let Some(s) = slot.as_mut().filter(|s| s.shown) {
            commands.entity(s.entity).insert(Visibility::Hidden);
            s.shown = false;
        }
        return;
    }
    // Bevy's space and corner order.
    let positions: Vec<[f32; 3]> = cast.iter().flat_map(|t| [t[0], t[2], t[1]]).map(|q| [q[0] / 100.0, q[2] / 100.0, -q[1] / 100.0]).collect();
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let mut mesh = Mesh::new(bevy::mesh::PrimitiveTopology::TriangleList, bevy::asset::RenderAssetUsages::default());
    mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
    mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
    match slot {
        Some(s) => {
            if let Some(mut m) = meshes.get_mut(&s.mesh) {
                *m = mesh;
            }
            if !s.shown {
                commands.entity(s.entity).insert(Visibility::Inherited);
                s.shown = true;
            }
        }
        None => {
            let h = meshes.add(mesh);
            let entity = commands.spawn((Mesh3d(h.clone()), MeshMaterial3d(material.clone()), Transform::default(), Visibility::Inherited, bevy::light::NotShadowCaster)).id();
            *slot = Some(ShadowSlot { entity, mesh: h, shown: true });
        }
    }
}

/// One plain object's shadow: what it was worked out from, and its slot.
#[derive(Default)]
pub struct ObjectShadow {
    /// Its shown meshes, its place, and the active buildings' numbers.
    key: Option<(Vec<Entity>, [u32; 7], Vec<u32>)>,
    slot: Option<ShadowSlot>,
    pieces: usize,
    /// Building surfaces it lies on (4d-7b-3).
    on_buildings: usize,
    /// Worked out with our fallback box (no stored ball).
    fallback: bool,
}

/// `shadows.log` and the counts for it.
#[derive(Default)]
pub struct ShadowLog {
    log: Option<Log>,
    frames: u64,
    worked_out: usize,
    /// Time spent in this system, for the log.
    spent: std::time::Duration,
}

/// Rule 37 "Plain objects" (M4 4d-7b-4): every drawn object whose test
/// says it casts (not a character, not a building) gets its shown parts
/// thrown along the sun onto the terrain; worked out again only when its
/// shown parts or its place change. Every 10 s a count in `shadows.log`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn object_shadow_draw(
    mut commands: Commands,
    world: Option<Res<ShadowWorld>>,
    light: Option<Res<ObjectLight>>,
    ground: Option<Res<crate::terrain_view::GroundLookup>>,
    roots: Query<(Entity, &DistanceTest, &GlobalTransform, Has<bevy::app::Propagate<RenderLayers>>)>,
    children: Query<&Children>,
    parts: Query<(&GlobalTransform, &Mesh3d)>,
    visibility: Query<&Visibility>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut shadows: Local<HashMap<Entity, ObjectShadow>>,
    mut log: Local<ShadowLog>,
    (buildings, activity, balls, change, mut seen, sources): (
        Option<Res<BuildingShadows>>,
        Option<Res<ObjectActivity>>,
        Query<(&GlobalTransform, &ModelBall)>,
        Res<crate::world_build::WorldChange>,
        Local<u64>,
        Option<Res<FrameSources>>,
    ),
) {
    let no_sources = FrameSources::default();
    let sources = sources.as_deref().unwrap_or(&no_sources);
    // A new world (M6 6e-2): the old objects' shadows are dropped.
    if crate::world_build::new_world(&change, &mut seen) {
        shadows.clear();
        *log = ShadowLog::default();
    }
    let (Some(world), Some(ground)) = (world, ground) else { return };
    let active = buildings.as_deref().map(|b| b.active(activity.as_deref())).unwrap_or_default();
    let active_ids: Vec<u32> = active.iter().map(|(id, _)| *id).collect();
    let started = std::time::Instant::now();
    let dir = shadow::direction(light.as_ref().and_then(|l| l.lights.suns.first()).map(|s| s.aim));
    let material = material.get_or_insert_with(|| shadow_material(&mut materials)).clone();
    if log.log.is_none() && log.frames == 0 {
        log.log = Log::create("shadows").ok();
        let casting = roots.iter().filter(|(_, t, _, _)| !t.character && kq8_data::characters::casts_shadow(world.0, t.shadow_mask, true, 256.0)).count();
        let line = format!(
            "plain objects' shadows (characters.md rule 37, M4 4d-7b-4): world mask {}, {casting} of {} objects cast (shadowMask, default 8; not buildings, see-through objects or characters); sun direction ({:.3}, {:.3}, {:.3})",
            world.0,
            roots.iter().count(),
            dir[0],
            dir[1],
            dir[2]
        );
        let mut names: Vec<&str> = roots.iter().filter(|(_, t, _, _)| !t.character && kq8_data::characters::casts_shadow(world.0, t.shadow_mask, true, 256.0)).map(|(_, t, _, _)| t.name.as_str()).collect();
        names.sort_unstable();
        names.dedup();
        if let Some(l) = log.log.as_mut() {
            l.summary(line);
            l.detail(format!("casting: {}", names.join(", ")));
        }
    }
    log.frames += 1;
    let mut shown = 0;
    let mut pieces = 0;
    let mut on_buildings = (0, 0);
    let mut fallbacks = 0;
    for (root, test, at, off) in &roots {
        if test.character || !kq8_data::characters::casts_shadow(world.0, test.shadow_mask, true, 256.0) {
            continue;
        }
        let entry = shadows.entry(root).or_default();
        let mut meshes_shown = Vec::new();
        let drawn = !off && visibility.get(root).is_ok_and(|v| *v != Visibility::Hidden);
        if drawn {
            shown_meshes(root, &children, &visibility, &parts, &mut meshes_shown);
        }
        let (r, t) = (at.rotation(), at.translation());
        let place = [r.x, r.y, r.z, r.w, t.x, t.y, t.z].map(f32::to_bits);
        let key = Some((meshes_shown, place, active_ids.clone()));
        if entry.key != key {
            let mut model = Vec::new();
            for &m in key.as_ref().map_or(&[][..], |k| &k.0[..]) {
                mesh_triangles(m, &parts, &meshes, sources, &mut model);
            }
            let position = [t.x * 100.0, -t.z * 100.0, t.y * 100.0];
            let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
            let caster = if model.is_empty() { None } else { shown_caster(root, &children, &visibility, &balls) };
            let on = receivers(&ground.0, position, &corners, dir, caster, &active);
            let cast = shadow::cast(&model, dir, &on.triangles);
            entry.pieces = cast.len();
            entry.on_buildings = on.pick.surfaces;
            entry.fallback = on.fallback && !model.is_empty();
            show_shadow(&cast, &mut entry.slot, &mut commands, &mut meshes, &material);
            entry.key = key;
            log.worked_out += 1;
        }
        if entry.pieces > 0 {
            shown += 1;
            pieces += entry.pieces;
        }
        if entry.on_buildings > 0 {
            on_buildings.0 += 1;
            on_buildings.1 += entry.on_buildings;
        }
        fallbacks += usize::from(entry.fallback);
    }
    log.spent += started.elapsed();
    if log.frames % 600 == 1 {
        let line = format!(
            "frame {}: {shown} plain objects' shadows shown, {pieces} pieces; {} of them reach building surfaces ({} surfaces; {} buildings active); {} without a stored model ball (fallback box); worked out {} times so far; {:.3} ms a frame on average in this system; {}",
            log.frames,
            on_buildings.0,
            on_buildings.1,
            active.len(),
            fallbacks,
            log.worked_out,
            log.spent.as_secs_f64() * 1000.0 / log.frames as f64,
            sources.line()
        );
        if let Some(l) = log.log.as_mut() {
            l.detail(line);
        }
    }
}
