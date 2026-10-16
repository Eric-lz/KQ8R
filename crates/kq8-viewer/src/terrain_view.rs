//! The world's ground (the default `kq8-viewer` view): (M1 step 2b) and its
//! objects (M1 step 3a) with a flying camera. Rules from
//! docs/behaviour/terrain.md and docs/behaviour/objects.md.

use std::time::Instant;

use bevy::camera::{PerspectiveProjection, Projection};
use bevy::image::ImageAddressMode;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;

use bevy::core_pipeline::tonemapping::Tonemapping;
use kq8_data::terrain::Level;

use crate::light_view::GroundLight;
use crate::load::{Ground, Objects};
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::ground_haze::{ground_material, tinted, with_opacity, GroundMaterial};
use crate::{mesh_from, model_mesh, to_bevy, Shared, UNITS_PER_METRE};

/// Which part of the ground a mesh is, for the U key (the floor grid,
/// water.md): the drawn surface where it equals the floor or lies over a
/// different floor, and the floor itself where it equals the surface or
/// differs from it (tinted).
#[derive(Component)]
pub enum GroundPart {
    SurfaceSame,
    SurfaceOverFloor { opaque: Handle<GroundMaterial>, see_through: Handle<GroundMaterial> },
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
pub struct Buildings(pub Vec<kq8_core::collide::BuildingCollider>);

/// What blocks the camera's view line besides the ground and the doors
/// (M2 step 2f-2): each building's collision type (same order as
/// `Buildings`) and the view-blocking object boxes.
#[derive(Resource, Default)]
pub struct ViewBlockerSet {
    pub building_types: Vec<u32>,
    pub boxes: Vec<kq8_core::collide::BoxCollider>,
}

/// The world's objects that collide with Connor (M2 step 2d-5).
#[derive(Resource, Default)]
pub struct ObjectColliders(pub Vec<kq8_core::collide::BoxCollider>);

/// The placed object an entity draws (M3 step 3f: `deleteObject` hides
/// it by name).
#[derive(Component)]
pub struct ObjectName(pub String);

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

// A Bevy system gets everything it uses as arguments.
#[allow(clippy::too_many_arguments)]
pub fn setup_ground(
    mut commands: Commands,
    mut loaded: ResMut<LoadedGround>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<GroundMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut light: ResMut<GroundLight>,
    haze: Res<ObjectLight>,
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
    // Light map cells per square side (terrain-lighting.md rule 6).
    let cells = 1usize << terrain.settings.light_scale;
    log.summary(format!(
        "  ground drawn as {cells} x {cells} light-map cells per square ({} units each)",
        size / cells as f32
    ));
    for group in ground.groups {
        // Each triangle is drawn as the light map's cells that lie in it
        // (M1 step 4c), on the surface or the floor grid. Clockwise from
        // above (the game's front) -> reversed for Bevy, exactly as for
        // models. Also returns the corners in game units, for the light.
        let mesh = |tris: &[(kq8_data::terrain::GroundTriangle, [f32; 3])], floor: bool| {
            let level = if floor { Level::Floor } else { Level::Surface };
            let mut positions = Vec::with_capacity(tris.len() * 3 * cells * cells);
            let mut uvs = Vec::with_capacity(positions.capacity());
            let mut game = Vec::with_capacity(positions.capacity());
            for (t, _) in tris {
                let code = ground.orientation.get(t.material as usize).copied().unwrap_or(0);
                let (i, j) = t.square;
                for c in terrain.square_cells(level, i, j, t.material, code, cells).iter().filter(|c| c.half == t.half) {
                    for k in [0, 2, 1] {
                        positions.push(to_bevy(c.positions[k], [0.0; 3]));
                        uvs.push(c.uvs[k]);
                        game.push(c.positions[k]);
                    }
                }
            }
            (mesh_from(positions, uvs), game)
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
        let material = ground_material(group.look, ImageAddressMode::ClampToEdge, &haze, &mut images);
        let tinted = |tint: Color| tinted(&material, tint);
        let opaque = materials.add(material.clone());
        let mut over: Vec<_> = lower.iter().chain(&higher).filter(|(t, _)| !ground.animated.contains(&t.square)).cloned().collect();
        over.sort_by_key(|(t, _)| t.square);
        let mut spawn = |tris: &[_], floor: bool, handle: Handle<GroundMaterial>, part: GroundPart, shown: bool| {
            if tris.is_empty() {
                return;
            }
            let v = if shown { Visibility::Inherited } else { Visibility::Hidden };
            let (m, game) = mesh(tris, floor);
            let m = light.light_mesh(m, &game, &mut meshes);
            commands.spawn((Mesh3d(m), MeshMaterial3d(handle), part, v));
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
        // Colours as computed, no tone-mapping curve (M1 step 4b).
        Tonemapping::None,
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
// A Bevy system gets everything it uses as arguments; splitting it up
// would not make it clearer.
#[allow(clippy::too_many_arguments)]
pub fn setup_objects(
    mut commands: Commands,
    mut loaded: ResMut<LoadedObjects>,
    shared: Res<Shared>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut object_materials: ResMut<Assets<ObjectMaterial>>,
    mut building_materials: ResMut<Assets<crate::building_light::BuildingMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<bevy::render::storage::ShaderBuffer>>,
    held: Res<crate::door_view::HeldItem>,
    light: Res<ObjectLight>,
) {
    let Some(mut objects) = loaded.0.take() else { return };
    let mut log = shared.log.lock().unwrap();
    let buildings = std::mem::take(&mut objects.buildings);
    commands.insert_resource(ViewBlockerSet {
        building_types: std::mem::take(&mut objects.building_types),
        boxes: std::mem::take(&mut objects.view_boxes),
    });
    // Doors (M3 step 3a): drawn and moved by door_view; their colliders
    // join the others and are moved every update.
    let mut colliders = std::mem::take(&mut objects.objects);
    // Items (3e-3): each type's model and opacity, before the items go to
    // the doors' `Active`.
    let item_looks: Vec<(String, usize, f32)> = objects
        .item_shapes
        .iter()
        .map(|(kind, slot)| {
            let t = objects.items.kind(kind).and_then(|t| t.translucency).unwrap_or(256.0);
            // As objects' `translucency` (objects.md Q14).
            (kind.clone(), *slot, if t >= 256.0 { 1.0 } else { (t / 255.0).clamp(0.0, 1.0) })
        })
        .collect();
    // Every model's bounding ball, for the draw distance (M1 6h, rule 35).
    let balls = crate::draw_distance::ShapeBalls::new(objects.shapes.iter().map(|(name, m)| (name.as_str(), m.min, m.max)));
    crate::door_view::spawn_doors(&mut commands, &mut objects, &mut colliders, held.0.clone(), &mut meshes, &mut object_materials, &light, &mut images, &balls, &mut log);
    commands.insert_resource(ObjectColliders(colliders));
    // What following emitters can follow (3b-4): Connor, the camera, or a
    // placed object or marker (any case; the last one loaded by that name).
    let connor_at = objects.connor.as_ref().map(|(p, _)| (p.position, p.angles));
    let camera_at = objects.camera_start;
    let fixed: Vec<(String, [f32; 3], [f32; 3])> = objects
        .placed
        .iter()
        .filter(|(_, _, unused)| !unused)
        .map(|(p, _, _)| (p.name.clone(), p.position, p.angles))
        .chain(objects.markers.iter().map(|m| (m.name.clone(), m.position, [0.0; 3])))
        .collect();
    let targets = move |name: &str| -> Option<(crate::effects_view::Target, [f32; 3], [f32; 3])> {
        use crate::effects_view::Target;
        if name.eq_ignore_ascii_case("Connor") {
            return connor_at.map(|(p, a)| (Target::Connor, p, a));
        }
        if name.eq_ignore_ascii_case("KQCamera") {
            return camera_at.map(|p| (Target::Camera, p, [0.0; 3]));
        }
        fixed.iter().rev().find(|(n, _, _)| n.eq_ignore_ascii_case(name)).map(|(_, p, a)| (Target::Fixed(*p, *a), *p, *a))
    };
    if let Some(connor) = objects.connor.take() {
        let start = objects.camera_start;
        crate::connor_view::spawn_connor(&mut commands, connor, start, &mut meshes, &mut object_materials, &light, &mut images, &mut log);
    }
    // One (mesh, material) list per model, shared by all its copies.
    let mut built: Vec<Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>> = Vec::new();
    // Buildings (M1 5c): their walls use the building material instead.
    let mut built_walls: Vec<Vec<(Handle<Mesh>, Handle<crate::building_light::BuildingMaterial>)>> = Vec::new();
    let mut first_corner: Vec<Option<[f32; 3]>> = Vec::new();
    // Each model's smooth corner normals, for the light log (M1 5b).
    let mut smooth: Vec<Vec<Vec3>> = Vec::new();
    // Each building's surface corners (model space), for the haze log (6f).
    let mut wall_surfaces: Vec<Vec<Vec<[f32; 3]>>> = Vec::new();
    for (_, model) in objects.shapes {
        smooth.push(model.groups.iter().flat_map(|g| crate::object_light::smooth_normals(&g.triangles, g.shading.lighting)).collect());
        let mut parts = Vec::new();
        first_corner.push(model.groups.first().and_then(|g| g.triangles.first()).map(|t| t.positions[0]));
        let mut walls = Vec::new();
        match &model.light_atlas {
            // A building: walls × its light maps (M1 5c).
            Some(atlas) => {
                let light_image = images.add(atlas.image());
                let corners = crate::building_light::WallCorners::new(&model.surfaces);
                let corner_buffer = buffers.add(corners.buffer());
                for group in model.groups {
                    let mesh = crate::building_mesh(&group, &corners);
                    let material = crate::building_light::building_material(group.look, light_image.clone(), corner_buffer.clone(), &light, &mut images);
                    walls.push((meshes.add(mesh), building_materials.add(material)));
                }
            }
            // A model: lit by the world's lights (M1 5b).
            None => {
                for group in model.groups {
                    let mesh = model_mesh(&group.triangles, group.shading.lighting, [0.0; 3]);
                    let material = object_material(group.look, group.shading, &light, &mut images);
                    parts.push((meshes.add(mesh), object_materials.add(material)));
                }
            }
        }
        built.push(parts);
        built_walls.push(walls);
        wall_surfaces.push(model.surfaces);
    }
    crate::item_view::spawn_items(&mut commands, item_looks, &built, &mut object_materials, &balls);
    // Emitters (M3 step 3b-1): their particles use the object models.
    // Particle catchers (3b-2) look up their emitter and building.
    let emitters = std::mem::take(&mut objects.emitters);
    let catchers = std::mem::take(&mut objects.catchers);
    crate::effects_view::spawn_emitters(&mut commands, emitters, catchers, &buildings, &targets, &built, &mut object_materials, &mut log);
    commands.insert_resource(Buildings(buildings));
    // Animated objects whose movement plays (3c): drawn by anim_view.
    let animated: std::collections::BTreeSet<usize> = objects.animated.iter().map(|a| a.placed).collect();
    crate::anim_view::spawn_animated(
        &mut commands,
        std::mem::take(&mut objects.animated),
        std::mem::take(&mut objects.object_animations),
        &objects.placed,
        &mut meshes,
        &mut object_materials,
        &light,
        &mut images,
        &balls,
        &mut log,
    );
    let mut worst = 0.0f32;
    let mut see_through = 0;
    for (k, (p, i, unused)) in objects.placed.iter().enumerate() {
        if animated.contains(&k) {
            continue;
        }
        let transform = bevy_transform(p);
        // Self-check: the game's rule (point x rotation + position, then
        // converted) and Bevy's transform must put a corner at the same spot.
        if let Some(corner) = first_corner[*i] {
            let game = Vec3::from(to_bevy(p.transform().apply(corner), [0.0; 3]));
            let bevy = transform.transform_point(Vec3::from(to_bevy(corner, [0.0; 3])));
            worst = worst.max((game - bevy).length() * UNITS_PER_METRE);
        }
        // Draw distance (M1 6h): one test per object, on each of its parts.
        let test = (!*unused).then(|| crate::draw_distance::DistanceTest::new(p, balls.by_index.get(*i).copied()));
        let opacity = p.opacity();
        if opacity < 1.0 {
            see_through += 1;
            log.detail(format!("see-through [{}]: translucency={} -> opacity {opacity:.2}", p.name, p.translucency));
        }
        for (mesh, material) in &built[*i] {
            // Every material of a see-through object (objects.md Q14); a
            // separate copy, because other copies of the model may be solid.
            let material = crate::object_light::for_object(&mut object_materials, material, opacity, p.haze_solid, p.haze_multiplier);
            let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material), transform, ObjectName(p.name.clone())));
            // Unused objects, and hidden ones (objects.md Q18): only with H.
            if *unused || !p.drawn() {
                e.insert((Visibility::Hidden, ShownWithH));
            }
            if let Some(t) = &test {
                e.insert(t.clone());
            }
        }
        for (mesh, material) in &built_walls[*i] {
            let material = crate::building_light::for_building(&mut building_materials, material, opacity, p.haze_solid);
            let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material), transform, ObjectName(p.name.clone())));
            if *unused || !p.drawn() {
                e.insert((Visibility::Hidden, ShownWithH));
            }
            if let Some(t) = &test {
                e.insert(t.clone());
            }
        }
    }
    // The placed objects and the buildings' wall surfaces (world space,
    // game units) for the haze log (M1 steps 6, 6f).
    let drawn = || objects.placed.iter().filter(|(p, _, unused)| !*unused && p.drawn());
    commands.insert_resource(crate::object_light::HazeCheck::new(
        drawn().map(|(p, _, _)| (p.name.clone(), p.position, p.haze_solid)).collect(),
        drawn()
            .filter(|(_, i, _)| !wall_surfaces[*i].is_empty())
            .map(|(p, i, _)| {
                let t = p.transform();
                (p.haze_solid, wall_surfaces[*i].iter().map(|s| s.iter().map(|&c| t.apply(c)).collect()).collect())
            })
            .collect(),
    ));
    // The light of the three drawn objects nearest Connor's start (M1 5b,
    // object-lighting.md "How to check": Daventry within 0.30-0.42).
    if let Some((at, _)) = connor_at {
        let mut near: Vec<(f32, usize)> = objects
            .placed
            .iter()
            .enumerate()
            .filter(|(_, (p, i, unused))| !*unused && p.drawn() && !smooth[*i].is_empty())
            .map(|(k, (p, _, _))| (((p.position[0] - at[0]).powi(2) + (p.position[1] - at[1]).powi(2)).sqrt(), k))
            .collect();
        near.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (distance, k) in near.into_iter().take(3) {
            let (p, i, _) = &objects.placed[k];
            if let Some(stats) = crate::object_light::light_stats(&light, &smooth[*i], bevy_transform(p).rotation) {
                log.summary(format!("object light: {} ({:.0} from Connor's start): {stats}", p.name, distance));
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
    mut parts: Query<(&GroundPart, &mut Visibility, &mut MeshMaterial3d<GroundMaterial>)>,
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

// A Bevy system gets everything it uses as arguments; splitting it up
// would not make it clearer.
#[allow(clippy::too_many_arguments)]
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
