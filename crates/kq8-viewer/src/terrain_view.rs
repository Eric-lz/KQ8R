//! `kq8-viewer --terrain`: the world's ground (M1 step 2b) and its
//! objects (M1 step 3a) with a flying camera. Rules from
//! docs/behaviour/terrain.md and docs/behaviour/objects.md.

use std::time::Instant;

use bevy::camera::{PerspectiveProjection, Projection};
use bevy::image::ImageAddressMode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use crate::load::{Ground, Objects};
use crate::{material_for, mesh_from, to_bevy, with_opacity, Shared, UNITS_PER_METRE};

/// Which part of the ground a mesh is, for the U key (the floor grid,
/// water.md): the drawn surface where it equals the floor or lies over a
/// different floor, and the floor itself where it equals the surface or
/// differs from it (tinted).
#[derive(Component)]
pub enum GroundPart {
    SurfaceSame,
    SurfaceOverFloor { opaque: Handle<StandardMaterial>, see_through: Handle<StandardMaterial> },
    FloorSame,
    FloorDiffers,
}

/// Tint for floor squares lower than the surface (under water).
const FLOOR_LOWER_TINT: Color = Color::srgb(0.45, 0.6, 1.0);
/// Tint for floor squares higher than the surface.
const FLOOR_HIGHER_TINT: Color = Color::srgb(1.0, 0.4, 0.4);
/// Opacity of the surface over a different floor while both are shown.
const SURFACE_SEE_THROUGH: f32 = 0.35;

#[derive(Resource)]
pub struct LoadedGround(pub Option<Ground>);

/// What the U key shows: 0 the surface (normal), 1 both, 2 the floor only.
#[derive(Resource)]
pub struct FloorView(pub u8);

/// Kept after setup so the camera report can ask for the ground height.
#[derive(Resource)]
pub struct GroundLookup(pub kq8_data::Terrain);

/// The world's buildings as colliders, for Connor's mover (M2 step 2d-3).
#[derive(Resource, Default)]
pub struct Buildings(pub Vec<kq8_game::collide::BuildingCollider>);

/// The world's objects that collide with Connor (M2 step 2d-5).
#[derive(Resource, Default)]
pub struct ObjectColliders(pub Vec<kq8_game::collide::BoxCollider>);

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
    let terrain = &ground.terrain;
    let size = terrain.settings.square_size();
    // Floor height at a grid corner given in game units.
    let floor_z = |p: [f32; 3]| {
        let (i, j) = ((p[0] / size).round() as usize, (p[1] / size).round() as usize);
        terrain.point_height_on(kq8_data::terrain::Level::Floor, i, j).unwrap_or(p[2])
    };
    // Per kind: (surface same, surface over a different floor, floor same,
    // floor lower, floor higher) triangle counts, for the log.
    let mut counts = [0usize; 5];
    for group in ground.groups {
        // Clockwise from above (the game's front) -> reversed for Bevy,
        // exactly as for models.
        let mesh = |tris: &[(kq8_data::terrain::GroundTriangle, [f32; 3])], floor: bool| {
            let mut positions = Vec::with_capacity(tris.len() * 3);
            let mut uvs = Vec::with_capacity(tris.len() * 3);
            for (t, fz) in tris {
                for k in [0, 2, 1] {
                    let mut p = t.positions[k];
                    if floor {
                        p[2] = fz[k];
                    }
                    positions.push(to_bevy(p, [0.0; 3]));
                    uvs.push(t.uvs[k]);
                }
            }
            mesh_from(positions, uvs)
        };
        // Sort the triangles by how the floor compares with the surface.
        let (mut same, mut lower, mut higher, mut floor_only) = (Vec::new(), Vec::new(), Vec::new(), Vec::new());
        for t in &group.triangles {
            // Animated squares' surface is drawn by ground_anim_view (3b-3);
            // their floor stays here.
            let animated = ground.animated.contains(&t.square);
            let fz = t.positions.map(floor_z);
            let d: f32 = (0..3).map(|k| fz[k] - t.positions[k][2]).sum();
            let differs = (0..3).any(|k| fz[k] != t.positions[k][2]);
            match (differs, d < 0.0) {
                (false, _) => same.push((*t, fz)),
                (true, true) => lower.push((*t, fz)),
                (true, false) => higher.push((*t, fz)),
            }
            if animated {
                floor_only.push((*t, fz));
            }
        }
        // Surface parts without the animated squares; floor parts with them.
        let surface_same: Vec<_> = same.iter().filter(|(t, _)| !ground.animated.contains(&t.square)).cloned().collect();
        let floor_same_animated = floor_only.len();
        triangles += group.triangles.len();
        // One texture covers exactly one square, so it must not repeat
        // into the neighbour: clamp at the edges.
        let material = material_for(group.look, ImageAddressMode::ClampToEdge, &mut images);
        let tinted = |tint: Color| {
            let mut m = material.clone();
            m.base_color = tint;
            m
        };
        let opaque = materials.add(material.clone());
        let mut over: Vec<_> = lower.iter().chain(&higher).filter(|(t, _)| !ground.animated.contains(&t.square)).cloned().collect();
        over.sort_by_key(|(t, _)| t.square);
        let mut spawn = |tris: &[_], floor: bool, handle: Handle<StandardMaterial>, part: GroundPart, shown: bool| {
            if tris.is_empty() {
                return;
            }
            let v = if shown { Visibility::Inherited } else { Visibility::Hidden };
            commands.spawn((Mesh3d(meshes.add(mesh(tris, floor))), MeshMaterial3d(handle), part, v));
        };
        spawn(&surface_same, false, opaque.clone(), GroundPart::SurfaceSame, true);
        let see_through = materials.add(with_opacity(&material, SURFACE_SEE_THROUGH));
        spawn(&over, false, opaque.clone(), GroundPart::SurfaceOverFloor { opaque: opaque.clone(), see_through }, true);
        spawn(&same, true, opaque.clone(), GroundPart::FloorSame, false);
        spawn(&lower, true, materials.add(tinted(FLOOR_LOWER_TINT)), GroundPart::FloorDiffers, false);
        spawn(&higher, true, materials.add(tinted(FLOOR_HIGHER_TINT)), GroundPart::FloorDiffers, false);
        for (k, n) in [same.len(), over.len(), same.len(), lower.len(), higher.len()].into_iter().enumerate() {
            counts[k] += n;
        }
        log.detail(format!(
            "spawned ground meshes for {} ({} triangles; floor lower under {}, higher under {}; {floor_same_animated} on animated squares, surface drawn by the animators)",
            group.label,
            group.triangles.len(),
            lower.len(),
            higher.len()
        ));
    }
    log.summary(format!(
        "  floor view (U): {} triangles where the floor differs ({} lower, blue; {} higher, red)",
        counts[1], counts[3], counts[4]
    ));

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
        // The whole ground must stay inside the view distance. Field of
        // view 70 degrees vertically (Bevy's default is 45; user's choice).
        // modified manually to what feels more natural, but could be wrong
        Projection::Perspective(PerspectiveProjection { fov: 50f32.to_radians(), far: extent * 2.0, ..default() }),
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
    held: Res<crate::door_view::HeldItem>,
) {
    let Some(mut objects) = loaded.0.take() else { return };
    let mut log = shared.log.lock().unwrap();
    let buildings = std::mem::take(&mut objects.buildings);
    // Doors (M3 step 3a): drawn and moved by door_view; their colliders
    // join the others and are moved every update.
    let mut colliders = std::mem::take(&mut objects.objects);
    crate::door_view::spawn_doors(&mut commands, &mut objects, &mut colliders, held.0.clone(), &mut meshes, &mut materials, &mut images, &mut log);
    commands.insert_resource(ObjectColliders(colliders));
    if let Some(connor) = objects.connor.take() {
        let start = objects.camera_start;
        crate::connor_view::spawn_connor(&mut commands, connor, start, &mut meshes, &mut materials, &mut images, &mut log);
    }
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
            let material = material_for(group.look, ImageAddressMode::Repeat, &mut images);
            parts.push((meshes.add(mesh_from(positions, uvs)), materials.add(material)));
        }
        built.push(parts);
    }
    // Emitters (M3 step 3b-1): their particles use the object models.
    // Particle catchers (3b-2) look up their emitter and building.
    let emitters = std::mem::take(&mut objects.emitters);
    let catchers = std::mem::take(&mut objects.catchers);
    crate::effects_view::spawn_emitters(&mut commands, emitters, catchers, &buildings, &built, &mut materials, &mut log);
    commands.insert_resource(Buildings(buildings));
    let mut worst = 0.0f32;
    let mut see_through = 0;
    for (p, i, unused) in &objects.placed {
        let transform = bevy_transform(p);
        // Self-check: the game's rule (point x rotation + position, then
        // converted) and Bevy's transform must put a corner at the same spot.
        if let Some(corner) = first_corner[*i] {
            let game = Vec3::from(to_bevy(p.transform().apply(corner), [0.0; 3]));
            let bevy = transform.transform_point(Vec3::from(to_bevy(corner, [0.0; 3])));
            worst = worst.max((game - bevy).length() * UNITS_PER_METRE);
        }
        let opacity = p.opacity();
        if opacity < 1.0 {
            see_through += 1;
            log.detail(format!("see-through [{}]: translucency={} -> opacity {opacity:.2}", p.name, p.translucency));
        }
        for (mesh, material) in &built[*i] {
            let material = if opacity < 1.0 {
                // Every material of a see-through object (objects.md Q14);
                // a separate copy, because other copies of the model may
                // be solid.
                let original = materials.get(material).cloned().unwrap_or_default();
                materials.add(with_opacity(&original, opacity))
            } else {
                material.clone()
            };
            let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material), transform));
            // Unused objects, and hidden ones (objects.md Q18): only with H.
            if *unused || !p.drawn() {
                e.insert((Visibility::Hidden, ShownWithH));
            }
        }
    }
    // Invisible objects: a 1 m cube each, hidden until H is pressed.
    // Green = no model in the game; orange = a character we cannot draw yet.
    let cube = meshes.add(Cuboid::from_length(100.0 / UNITS_PER_METRE));
    let green = materials.add(StandardMaterial { base_color: Color::srgb(0.1, 1.0, 0.2), unlit: true, ..default() });
    let orange = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.55, 0.0), unlit: true, ..default() });
    for m in &objects.markers {
        // Cube resting on the point, so ground markers are not half buried.
        let at = Vec3::from(to_bevy(m.position, [0.0; 3])) + Vec3::Y * 50.0 / UNITS_PER_METRE;
        commands.spawn((
            Mesh3d(cube.clone()),
            MeshMaterial3d(if m.anm { orange.clone() } else { green.clone() }),
            Transform::from_translation(at),
            Visibility::Hidden,
            ShownWithH,
        ));
    }
    log.summary(format!(
        "spawned objects: {} copies ({} of them unused or hidden, shown with H) of {} models ({see_through} see-through); largest difference between the game's placement rule and Bevy's transform: {worst:.3} game units",
        objects.placed.len(),
        objects.placed.iter().filter(|x| x.2 || !x.0.drawn()).count(),
        built.len()
    ));
}

/// Hidden at start, shown by H: cubes for objects with nothing to draw,
/// and unused objects (no start-up script loads them).
#[derive(Component)]
pub struct ShownWithH;

/// The small on-screen text that says whether H is on.
#[derive(Component)]
pub struct HLabel;

/// The on-screen text that says what U shows.
#[derive(Component)]
pub struct FloorLabel;

fn floor_label_text(mode: u8) -> String {
    let what = match mode {
        1 => "BOTH (surface see-through over a different floor; floor blue = lower, red = higher)",
        2 => "FLOOR ONLY (blue = lower than the surface, red = higher)",
        _ => "surface (normal)",
    };
    format!("Ground: {what} (U)")
}

fn h_label_text(shown: bool) -> String {
    format!("Hidden objects: {} (H)", if shown { "SHOWN" } else { "hidden" })
}

pub fn setup_h_label(mut commands: Commands) {
    commands.spawn((
        Text::new(h_label_text(false)),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(8.0), left: Val::Px(10.0), ..default() },
        HLabel,
    ));
    commands.spawn((
        Text::new(floor_label_text(0)),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(30.0), left: Val::Px(10.0), ..default() },
        FloorLabel,
    ));
    commands.spawn((
        Text::new(""),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, bottom: Val::Px(8.0), left: Val::Px(10.0), ..default() },
        CameraLabel,
    ));
}

/// The on-screen camera position (bottom left).
#[derive(Component)]
pub struct CameraLabel;

/// Every frame: camera position in game units (same as the log's
/// `camera game=(...)` lines) and the ground height below it.
pub fn update_camera_label(
    lookup: Option<Res<GroundLookup>>,
    camera: Single<&Transform, With<Camera3d>>,
    mut label: Single<&mut Text, With<CameraLabel>>,
) {
    let game = game_position(camera.translation);
    let below = lookup.and_then(|l| l.0.height_at(game[0], game[1]));
    label.0 = format!(
        "Camera X {:.0}  Y {:.0}  Z {:.0}   ground {}",
        game[0],
        game[1],
        game[2],
        below.map_or("none".into(), |g| format!("{:.0}", g.height))
    );
}

/// Bevy position -> game units: Bevy (x, y, z) -> game (x, -z, y), the
/// inverse of `to_bevy`.
fn game_position(bevy: Vec3) -> [f32; 3] {
    let p = bevy * UNITS_PER_METRE;
    [p.x, -p.z, p.y]
}

/// H shows or hides everything marked `ShownWithH`.
pub fn toggle_markers(
    keys: Res<ButtonInput<KeyCode>>,
    shared: Res<Shared>,
    mut shown: Local<bool>,
    mut markers: Query<&mut Visibility, With<ShownWithH>>,
    mut label: Query<&mut Text, With<HLabel>>,
) {
    if !keys.just_pressed(KeyCode::KeyH) {
        return;
    }
    *shown = !*shown;
    for mut v in &mut markers {
        *v = if *shown { Visibility::Inherited } else { Visibility::Hidden };
    }
    for mut t in &mut label {
        t.0 = h_label_text(*shown);
    }
    shared.log.lock().unwrap().detail(format!(
        "key H: invisible objects and unused objects {} ({} entities)",
        if *shown { "shown" } else { "hidden" },
        markers.iter().count()
    ));
}

/// U cycles the ground view (a viewer helper for water.md's floor grid):
/// normal → both (the surface see-through where a different floor lies
/// under or over it, that floor tinted) → floor only → normal.
pub fn toggle_floor(
    keys: Res<ButtonInput<KeyCode>>,
    shared: Res<Shared>,
    mut view: ResMut<FloorView>,
    mut parts: Query<(&GroundPart, &mut Visibility, &mut MeshMaterial3d<StandardMaterial>)>,
    mut label: Query<&mut Text, With<FloorLabel>>,
) {
    if !keys.just_pressed(KeyCode::KeyU) {
        return;
    }
    view.0 = (view.0 + 1) % 3;
    let mode = &view.0;
    let show = |on: bool| if on { Visibility::Inherited } else { Visibility::Hidden };
    for (part, mut v, mut m) in &mut parts {
        match part {
            GroundPart::SurfaceSame => *v = show(*mode != 2),
            GroundPart::SurfaceOverFloor { opaque, see_through } => {
                *v = show(*mode != 2);
                m.0 = if *mode == 1 { see_through.clone() } else { opaque.clone() };
            }
            GroundPart::FloorSame => *v = show(*mode == 2),
            GroundPart::FloorDiffers => *v = show(*mode != 0),
        }
    }
    for mut t in &mut label {
        t.0 = floor_label_text(*mode);
    }
    let name = ["surface (normal)", "both", "floor only"][*mode as usize];
    shared.log.lock().unwrap().summary(format!("key U: ground view {name} ({} ground meshes)", parts.iter().count()));
}

/// A placement as a Bevy transform. The game moves a point as
/// `point × R + position` (row vectors, Z up); in Bevy's Y-up space the
/// same turn is `C · Rᵀ · Cᵀ`, where C is the axis change of `to_bevy`.
pub fn bevy_transform(p: &kq8_data::objects::Placement) -> Transform {
    bevy_transform_from(p.transform().m, p.position)
}

/// The same for a rotation matrix R (row vectors) and a position, e.g. a
/// door turned by its animation.
pub fn bevy_transform_from(r: [[f32; 3]; 3], position: [f32; 3]) -> Transform {
    // Columns of a glam matrix built from R's rows = Rᵀ.
    let r_t = Mat3::from_cols_array_2d(&r);
    // to_bevy: (x, y, z) -> (x, z, -y), as columns.
    let c = Mat3::from_cols(Vec3::X, Vec3::NEG_Z, Vec3::Y);
    let rotation = Quat::from_mat3(&(c * r_t * c.transpose()));
    Transform { translation: Vec3::from(to_bevy(position, [0.0; 3])), rotation, scale: Vec3::ONE }
}

pub fn fly_input(
    mut fly: ResMut<Fly>,
    buttons: Res<ButtonInput<MouseButton>>,
    keys: Res<ButtonInput<KeyCode>>,
    motion: Res<AccumulatedMouseMotion>,
    time: Res<Time>,
    mut camera: Single<&mut Transform, With<Camera3d>>,
    mut commands: Commands,
    choice: Res<crate::game_camera::CameraChoice>,
) {
    if keys.just_pressed(KeyCode::Escape) {
        commands.write_message(AppExit::Success);
    }
    // The game camera (M2 step 2f) drives the view; F switches back here.
    if choice.game {
        return;
    }
    // Right mouse to look, as the original's camera button.
    if buttons.pressed(MouseButton::Right) {
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
    let game = game_position(camera.translation);
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
