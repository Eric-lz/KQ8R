//! `kq8-viewer --terrain`: the world's ground (M1 step 2b) and its
//! objects (M1 step 3a) with a flying camera. Rules from
//! docs/behaviour/terrain.md and docs/behaviour/objects.md.

use std::time::Instant;

use bevy::camera::{PerspectiveProjection, Projection};
use bevy::image::ImageAddressMode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::load::{Ground, Objects};
use crate::{material_for, mesh_from, to_bevy, Shared, UNITS_PER_METRE};

#[derive(Resource)]
pub struct LoadedGround(pub Option<Ground>);

/// Kept after setup so the camera report can ask for the ground height.
#[derive(Resource)]
pub struct GroundLookup(pub kq8_data::Terrain);

#[derive(Resource)]
pub struct Fly {
    pub yaw: f32,
    pub pitch: f32,
    last_report: Option<Instant>,
}

impl Fly {
    pub fn new() -> Self {
        // Facing north (game +Y), tilted a little down.
        Fly { yaw: 0.0, pitch: -0.25, last_report: None }
    }
}

pub fn setup_ground(
    mut commands: Commands,
    mut loaded: ResMut<LoadedGround>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let ground = loaded.0.take().expect("ground is loaded once");
    let mut log = shared.log.lock().unwrap();
    let mut triangles = 0;
    for group in ground.groups {
        let mut positions = Vec::with_capacity(group.triangles.len() * 3);
        let mut uvs = Vec::with_capacity(group.triangles.len() * 3);
        for t in &group.triangles {
            // Clockwise from above (the game's front) -> reversed for Bevy,
            // exactly as for models.
            for k in [0, 2, 1] {
                positions.push(to_bevy(t.positions[k], [0.0; 3]));
                uvs.push(t.uvs[k]);
            }
        }
        triangles += group.triangles.len();
        // One texture covers exactly one square, so it must not repeat
        // into the neighbour: clamp at the edges.
        let material = material_for(group.look, ImageAddressMode::ClampToEdge, false, &mut images);
        commands.spawn((Mesh3d(meshes.add(mesh_from(positions, uvs))), MeshMaterial3d(materials.add(material))));
        log.detail(format!("spawned ground mesh for {} ({} triangles)", group.label, group.triangles.len()));
    }

    let t = &ground.terrain;
    let (nx, ny) = t.squares();
    let size = t.settings.square_size();
    let (cx, cy) = (nx as f32 * size / 2.0, ny as f32 * size / 2.0);
    let below = t.height_at(cx, cy).map_or(0.0, |g| g.height);
    let start = [cx, cy, below + 4000.0];
    let extent = nx.max(ny) as f32 * size / UNITS_PER_METRE;
    log.summary(format!(
        "spawned ground: {triangles} triangles; camera starts at game ({:.0}, {:.0}, {:.0}), {:.0} units above the ground",
        start[0], start[1], start[2], start[2] - below
    ));
    commands.spawn((
        Camera3d::default(),
        // The whole ground must stay inside the view distance.
        Projection::Perspective(PerspectiveProjection { far: extent * 2.0, ..default() }),
        Transform::from_translation(Vec3::from(to_bevy(start, [0.0; 3]))),
    ));
    commands.insert_resource(GroundLookup(ground.terrain));
}

#[derive(Resource)]
pub struct LoadedObjects(pub Option<Objects>);

/// Builds each different model once and places every copy
/// (docs/behaviour/objects.md).
pub fn setup_objects(
    mut commands: Commands,
    mut loaded: ResMut<LoadedObjects>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(objects) = loaded.0.take() else { return };
    let mut log = shared.log.lock().unwrap();
    // One (mesh, material) list per model, shared by all its copies.
    let mut built: Vec<Vec<(Handle<Mesh>, Handle<StandardMaterial>)>> = Vec::new();
    let mut first_corner: Vec<Option<[f32; 3]>> = Vec::new();
    for (_, model) in objects.shapes {
        let mut parts = Vec::new();
        first_corner.push(model.groups.first().and_then(|g| g.triangles.first()).map(|t| t.positions[0]));
        for group in model.groups {
            let mut positions = Vec::with_capacity(group.triangles.len() * 3);
            let mut uvs = Vec::with_capacity(group.triangles.len() * 3);
            for t in &group.triangles {
                // Clockwise front (the game) -> counter-clockwise (Bevy).
                for k in [0, 2, 1] {
                    positions.push(to_bevy(t.positions[k], [0.0; 3]));
                    uvs.push(t.uvs[k]);
                }
            }
            let material = material_for(group.look, ImageAddressMode::Repeat, group.see_through, &mut images);
            parts.push((meshes.add(mesh_from(positions, uvs)), materials.add(material)));
        }
        built.push(parts);
    }
    let mut worst = 0.0f32;
    for (p, i) in &objects.placed {
        let transform = bevy_transform(p);
        // Self-check: the game's rule (point x rotation + position, then
        // converted) and Bevy's transform must put a corner at the same spot.
        if let Some(corner) = first_corner[*i] {
            let game = Vec3::from(to_bevy(p.transform().apply(corner), [0.0; 3]));
            let bevy = transform.transform_point(Vec3::from(to_bevy(corner, [0.0; 3])));
            worst = worst.max((game - bevy).length() * UNITS_PER_METRE);
        }
        for (mesh, material) in &built[*i] {
            commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), transform));
        }
    }
    log.summary(format!(
        "spawned objects: {} copies of {} models; largest difference between the game's placement rule and Bevy's transform: {worst:.3} game units",
        objects.placed.len(),
        built.len()
    ));
}

/// A placement as a Bevy transform. The game moves a point as
/// `point × R + position` (row vectors, Z up); in Bevy's Y-up space the
/// same turn is `C · Rᵀ · Cᵀ`, where C is the axis change of `to_bevy`.
fn bevy_transform(p: &kq8_data::objects::Placement) -> Transform {
    // Columns of a glam matrix built from R's rows = Rᵀ.
    let r_t = Mat3::from_cols_array_2d(&p.transform().m);
    // to_bevy: (x, y, z) -> (x, z, -y), as columns.
    let c = Mat3::from_cols(Vec3::X, Vec3::NEG_Z, Vec3::Y);
    let rotation = Quat::from_mat3(&(c * r_t * c.transpose()));
    Transform { translation: Vec3::from(to_bevy(p.position, [0.0; 3])), rotation, scale: Vec3::ONE }
}

pub fn fly_input(
    mut fly: ResMut<Fly>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mut commands: Commands,
) {
    if buttons.pressed(MouseButton::Left) {
        fly.yaw -= motion.delta.x * 0.003;
        fly.pitch = (fly.pitch - motion.delta.y * 0.003).clamp(-1.55, 1.55);
    }
    camera.rotation = Quat::from_euler(EulerRot::YXZ, fly.yaw, fly.pitch, 0.0);
    let forward = camera.rotation * Vec3::NEG_Z;
    let right = camera.rotation * Vec3::X;
    let mut step = Vec3::ZERO;
    for (key, dir) in [
        (KeyCode::KeyW, forward),
        (KeyCode::KeyS, -forward),
        (KeyCode::KeyD, right),
        (KeyCode::KeyA, -right),
        (KeyCode::KeyE, Vec3::Y),
        (KeyCode::KeyQ, Vec3::NEG_Y),
    ] {
        if keys.pressed(key) {
            step += dir;
        }
    }
    // Flying speed. Shift 5x
    let speed = if keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight) { 400.0 } else { 80.0 };
    camera.translation += step.normalize_or_zero() * speed * time.delta_secs();
    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
}

/// Every 2 s: camera position in game units and the ground below it.
pub fn report_camera(
    mut fly: ResMut<Fly>,
    lookup: Option<Res<GroundLookup>>,
    shared: Res<Shared>,
    camera: Single<&Transform, With<Camera3d>>,
) {
    if fly.last_report.is_some_and(|t| t.elapsed().as_secs_f32() < 2.0) {
        return;
    }
    fly.last_report = Some(Instant::now());
    let p = camera.translation * UNITS_PER_METRE;
    // Bevy (x, y, z) -> game (x, -z, y), the inverse of `to_bevy`.
    let game = [p.x, -p.z, p.y];
    let below = lookup.and_then(|l| l.0.height_at(game[0], game[1]));
    shared.log.lock().unwrap().detail(format!(
        "camera game=({:.0}, {:.0}, {:.0}) heading_deg={:.0} pitch_deg={:.0} ground_below={}",
        game[0],
        game[1],
        game[2],
        (-fly.yaw.to_degrees()).rem_euclid(360.0) + 0.0,
        fly.pitch.to_degrees(),
        below.map_or("none (outside the grid)".into(), |g| format!("{:.0} (camera {:.0} above)", g.height, game[2] - g.height))
    ));
}
