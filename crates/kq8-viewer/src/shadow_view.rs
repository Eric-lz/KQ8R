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

use bevy::prelude::*;
use bevy::camera::visibility::RenderLayers;
use kq8_core::shadow;

use crate::draw_distance::{DistanceTest, ObjectActivity};
use crate::log::Log;
use crate::object_light::ObjectLight;

/// The world's shadow mask (`characters.md` rule 36).
#[derive(Resource)]
pub struct ShadowWorld(pub u32);

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

/// Rule 37: what a caster's shadow lies on: the terrain's triangles
/// (within 1,500 of the ground) and those of the `buildings` (active ones)
/// that `shadow::building_receivers` picks. Also the count of terrain
/// triangles and the buildings' pick, for the logs.
pub fn receivers(ground: &kq8_data::Terrain, position: shadow::V3, corners: &[shadow::V3], dir: shadow::V3, buildings: &[(u32, &shadow::ReceivingBuilding)]) -> (Vec<[shadow::V3; 3]>, usize, shadow::BuildingPick) {
    if corners.is_empty() {
        return (Vec::new(), 0, shadow::BuildingPick::default());
    }
    let mut all = shadow::terrain_receivers(ground, position, corners, dir);
    let terrain = all.len();
    let mut pick = shadow::building_receivers(buildings.iter().map(|(_, b)| *b), corners, dir);
    all.append(&mut pick.triangles);
    (all, terrain, pick)
}

/// The shadows' look: unlit, multiply blending at the strength 0.5 (each
/// layer halves what is under it), no haze (rule 37).
pub fn shadow_material(materials: &mut Assets<StandardMaterial>) -> Handle<StandardMaterial> {
    let v = shadow::STRENGTH;
    materials.add(StandardMaterial {
        base_color: Color::linear_rgb(v, v, v),
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
pub fn frame_triangles(frame: Entity, children: &Query<&Children>, parts: &Query<(&GlobalTransform, &Mesh3d)>, meshes: &Assets<Mesh>) -> Vec<[shadow::V3; 3]> {
    let mut model = Vec::new();
    for part in children.get(frame).into_iter().flat_map(|c| c.iter()) {
        mesh_triangles(part, parts, meshes, &mut model);
    }
    model
}

/// One mesh entity's triangles in the game's space and corner order.
fn mesh_triangles(part: Entity, parts: &Query<(&GlobalTransform, &Mesh3d)>, meshes: &Assets<Mesh>, model: &mut Vec<[shadow::V3; 3]>) {
    let Ok((at, mesh)) = parts.get(part) else { return };
    let Some(mesh) = meshes.get(&mesh.0) else { return };
    let Some(bevy::mesh::VertexAttributeValues::Float32x3(points)) = mesh.attribute(Mesh::ATTRIBUTE_POSITION) else { return };
    // Bevy (x, y, z) = game (x, z, −y) / 100, so back: (x, −z, y) × 100.
    let game: Vec<shadow::V3> = points.iter().map(|&q| at.transform_point(Vec3::from(q))).map(|v| [v.x * 100.0, -v.z * 100.0, v.y * 100.0]).collect();
    let corners: Vec<usize> = match mesh.indices() {
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
#[allow(clippy::too_many_arguments)]
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
    (buildings, activity): (Option<Res<BuildingShadows>>, Option<Res<ObjectActivity>>),
) {
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
                mesh_triangles(m, &parts, &meshes, &mut model);
            }
            let position = [t.x * 100.0, -t.z * 100.0, t.y * 100.0];
            let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
            let (receivers, _, pick) = receivers(&ground.0, position, &corners, dir, &active);
            let cast = shadow::cast(&model, dir, &receivers);
            entry.pieces = cast.len();
            entry.on_buildings = pick.surfaces;
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
    }
    log.spent += started.elapsed();
    if log.frames % 600 == 1 {
        let line = format!(
            "frame {}: {shown} plain objects' shadows shown, {pieces} pieces; {} of them reach building surfaces ({} surfaces; {} buildings active); worked out {} times so far; {:.3} ms a frame on average in this system",
            log.frames,
            on_buildings.0,
            on_buildings.1,
            active.len(),
            log.worked_out,
            log.spent.as_secs_f64() * 1000.0 / log.frames as f64
        );
        if let Some(l) = log.log.as_mut() {
            l.detail(line);
        }
    }
}
