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

/// Original texture metadata on a static object part so `KQObject::changeBmp`
/// and `KQSwampTile` Clarity Reveal swaps can clone & restore its material
/// without mutating shared model materials across other copies.
#[derive(Component)]
pub struct StaticTexture {
    pub object_lc: String,
    pub original_tex: String,
    pub original_material: Handle<ObjectMaterial>,
    pub swapped_to: Option<String>,
}

/// Applies `KQObject::changeBmp` / Clarity `KQSwampTile` texture swaps to static
/// object parts (`characters.md` rule 4a; `tile-puzzle.md` rule 38).
pub fn static_bmp_swap_step(
    doors: Option<Res<crate::door_view::DoorSim>>,
    anim: Option<Res<crate::anim_view::AnimSim>>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut query: Query<(&mut StaticTexture, &mut MeshMaterial3d<ObjectMaterial>)>,
) {
    let (Some(doors), Some(anim)) = (doors, anim) else { return };
    for (mut st, mut mat_handle) in &mut query {
        let wanted = doors.active.world.bmp_swaps.get(&st.object_lc).and_then(|pairs| {
            pairs.iter().rev().find_map(|(old, new)| {
                (st.original_tex.eq_ignore_ascii_case(old)
                    && !new.eq_ignore_ascii_case(&st.original_tex)
                    && !new.eq_ignore_ascii_case(old))
                    .then(|| new.to_ascii_lowercase())
            })
        });
        if wanted == st.swapped_to {
            continue;
        }
        match wanted {
            Some(ref new_lc) => {
                if let (Some(img), Some(mut cloned)) = (anim.swap_image(new_lc), materials.get(&st.original_material).cloned()) {
                    cloned.texture = Some(img.clone());
                    mat_handle.0 = materials.add(cloned);
                    st.swapped_to = Some(new_lc.clone());
                }
            }
            None => {
                mat_handle.0 = st.original_material.clone();
                st.swapped_to = None;
            }
        }
    }
}
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

/// The order the object colliders entered the world (indices into
/// `ObjectColliders`), the order Connor's sweep tests them in
/// (`connor-physics.md` "Against a building" step 7, Q88; M3 3l-4): by
/// where each object's name comes in the load order.
#[derive(Resource, Default)]
pub struct ColliderOrder(pub Vec<usize>);

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
        let material = ground_material(group.look, ImageAddressMode::ClampToEdge, &haze, &mut light, &mut images);
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
        // Colours as computed, no tone-mapping curve (M1 step 4b). Dithering
        // only works with tone mapping, so it is switched off too (it did
        // nothing; Bevy warned about it).
        Tonemapping::None,
        bevy::core_pipeline::tonemapping::DebandDither::Disabled,
        // 4c-10: the panels are pixel pictures laid edge to edge: no
        // smoothing of their edges, which let the world show through
        // between them (the user's check).
        bevy::ui_render::UiAntiAlias::Off,
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
    anim_buffers: Res<crate::anim_gpu::AnimBuffers>,
) {
    let Some(mut objects) = loaded.0.take() else { return };
    // Step 4: the world's animations to the graphics card.
    let (gpu, line) = crate::anim_gpu::AnimGpu::install(objects.pack.take().unwrap_or_default(), &anim_buffers, &mut buffers, &mut meshes);
    shared.log.lock().unwrap().summary(line);
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
    let item_looks: Vec<(String, usize, f32, String)> = objects
        .item_shapes
        .iter()
        .map(|(kind, slot)| {
            let t = objects.items.kind(kind).and_then(|t| t.translucency).unwrap_or(256.0);
            let shape = objects.items.kind(kind).map(|t| t.shape.clone()).unwrap_or_default();
            // As objects' `translucency` (objects.md Q14).
            (kind.clone(), *slot, if t >= 256.0 { 1.0 } else { (t / 255.0).clamp(0.0, 1.0) }, shape)
        })
        .collect();
    // Every model's bounding ball, for the draw distance (M1 6h, rule 35).
    let bounds = match objects.game_bounds.take() {
        Some(t) => Some(t),
        None => {
            log.summary("draw distance: GAME/patch/BOUNDS.KQ could not be read: every ball radius 100");
            None
        }
    };
    let corpse_bounds: std::collections::BTreeMap<String, ([f32; 3], [f32; 3])> = objects
        .corpse_shapes
        .keys()
        .filter_map(|k| {
            bounds
                .as_ref()
                .and_then(|t| t.lookup(k))
                .map(|e| (e.min, e.max))
                .or_else(|| objects.shapes.iter().find(|(n, _)| n.eq_ignore_ascii_case(k)).map(|(_, m)| (m.min, m.max)))
                .map(|b| (k.clone(), b))
        })
        .collect();
    commands.insert_resource(crate::shadow_view::ShadowWorld { mask: objects.shadow_world_mask, allow: objects.allow_shadows });
    let balls = crate::draw_distance::ShapeBalls::new(bounds, objects.shapes.iter().map(|(name, m)| (name.as_str(), m.min, m.max)));
    // Animated objects whose movement plays or can be switched (3c,
    // 3i-4): drawn by anim_view, played by the world's rules.
    let animated: std::collections::BTreeSet<usize> = objects.animated.iter().map(|a| a.placed).collect();
    let (anims, own_players) = crate::anim_view::spawn_animated(
        &mut commands,
        std::mem::take(&mut objects.animated),
        std::mem::take(&mut objects.object_animations),
        &objects.placed,
        &objects.templates.iter().map(|t| t.placed).collect(),
        &mut object_materials,
        &light,
        &mut images,
        &balls,
        &std::mem::take(&mut objects.character_lines),
        &mut log,
        &gpu,
        &std::mem::take(&mut objects.swap_looks),
    );
    crate::door_view::spawn_doors(&mut commands, &mut objects, &mut colliders, (held.0.clone(), held.1), (anims, own_players), &mut object_materials, &light, &mut images, &balls, &mut log, &gpu);
    // 3l-4: doors, chests and scripted objects were added after the
    // plain objects; the sweep tests them all in load order.
    let load_index = |name: &str| objects.placed.iter().position(|(p, _, _)| p.name.eq_ignore_ascii_case(name)).unwrap_or(usize::MAX);
    let mut order: Vec<usize> = (0..colliders.len()).collect();
    order.sort_by_key(|&i| (load_index(&colliders[i].name), i));
    let moved = order.iter().enumerate().filter(|(k, i)| k != *i).count();
    log.detail(format!("collision: {} object colliders tested in load order ({moved} not where they are listed)", colliders.len()));
    commands.insert_resource(ColliderOrder(order));
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
        crate::connor_view::spawn_connor(&mut commands, connor, start, &mut object_materials, &light, &mut images, &mut log, &gpu);
    }
    commands.insert_resource(gpu);
    // One (mesh, material) list per model, shared by all its copies.
    let mut built: Vec<Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>> = Vec::new();
    // Original texture name per built part (same shape/part indexing as `built`).
    let mut shape_textures: Vec<Vec<Option<String>>> = Vec::new();
    // Buildings (M1 5c): their walls use the building material instead.
    let mut built_walls: Vec<Vec<(Handle<Mesh>, Handle<crate::building_light::BuildingMaterial>)>> = Vec::new();
    let mut first_corner: Vec<Option<[f32; 3]>> = Vec::new();
    // Each model's smooth corner normals, for the light log (M1 5b).
    let mut smooth: Vec<Vec<Vec3>> = Vec::new();
    // Each building's surface corners (model space), for the haze log (6f).
    let mut wall_surfaces: Vec<Vec<Vec<[f32; 3]>>> = Vec::new();
    // Buildings' light pictures and point-light data (M1 7d, rule 16).
    type WallInfo = (std::sync::Arc<crate::building_light::LightAtlas>, std::sync::Arc<crate::wall_lights::WallData>);
    let mut wall_info: Vec<Option<WallInfo>> = Vec::new();
    // Each model's lights in its shown frame (M1 7c, rule 26b).
    let mut shape_lights: Vec<std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>> = Vec::new();
    // Each building's drawn triangles by surface (own space), for shadows
    // on buildings (M4 4d-7b-3); empty for models.
    let mut shadow_surfaces: Vec<Vec<Vec<[[f32; 3]; 3]>>> = Vec::new();
    // Each model's `.kqs` ball (M4 4d-7b-5b); `None` for buildings.
    let mut model_balls: Vec<Option<crate::shadow_view::ModelBall>> = Vec::new();
    for (_, model) in objects.shapes {
        let mut by_surface = vec![Vec::new(); if model.light_atlas.is_some() { model.surfaces.len() } else { 0 }];
        for t in model.groups.iter().flat_map(|g| &g.triangles) {
            if let Some(s) = by_surface.get_mut(t.node) {
                s.push(t.positions);
            }
        }
        shadow_surfaces.push(by_surface);
        model_balls.push(model.ball.map(crate::shadow_view::ModelBall::of));
        shape_lights.push(std::sync::Arc::new(vec![model.lights.clone()]));
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
                let factors = buffers.add(crate::building_light::factor_buffer(&vec![1.0; model.surfaces.len()]));
                for group in model.groups {
                    let mesh = crate::building_mesh(&group, &corners);
                    let material =
                        crate::building_light::building_material(group.look, light_image.clone(), corner_buffer.clone(), factors.clone(), &light, &mut images);
                    walls.push((meshes.add(mesh), building_materials.add(material)));
                }
            }
            // A model: lit by the world's lights (M1 5b).
            None => {
                let mut tex_names = Vec::new();
                for group in model.groups {
                    let tex_name = match &group.look {
                        crate::load::Look::Texture { name, .. } => Some(name.clone()),
                        _ => None,
                    };
                    let mesh = model_mesh(&group.triangles, group.shading.lighting, [0.0; 3]);
                    let material = object_material(group.look, group.shading, &light, &mut images);
                    parts.push((meshes.add(mesh), object_materials.add(material)));
                    tex_names.push(tex_name);
                }
                shape_textures.push(tex_names);
            }
        }
        if model.light_atlas.is_some() {
            shape_textures.push(Vec::new());
        }
        wall_info.push(model.light_atlas.map(std::sync::Arc::new).zip(model.wall_light));
        built.push(parts);
        built_walls.push(walls);
        wall_surfaces.push(model.surfaces);
    }
    crate::item_view::spawn_items(&mut commands, item_looks, &built, &mut object_materials, &balls);
    // fighting.md rule 32 (M5 5c-7): the corpses' models.
    crate::item_view::spawn_corpses(&mut commands, std::mem::take(&mut objects.corpse_shapes), &built, corpse_bounds);
    // potions.md rule 20 (M5 5g-4): the shield's orbit.
    crate::potion_view::spawn_orbit(&mut commands, std::mem::take(&mut objects.orbit), &built, &mut object_materials);
    // M5 5d-3: the templates' models, for copies loaded while playing.
    crate::copies_view::keep_templates(&mut commands, &objects.templates, &objects.placed, &built);
    // Emitters (M3 step 3b-1): their particles use the object models.
    // Particle catchers (3b-2) look up their emitter and building.
    let emitters = std::mem::take(&mut objects.emitters);
    let catchers = std::mem::take(&mut objects.catchers);
    crate::effects_view::spawn_emitters(&mut commands, emitters, catchers, &buildings, &targets, &built, &mut object_materials, &mut log);
    // Which collider is which building copy (M1 7d: the line test uses its
    // own BSP).
    let collider_keys: Vec<(String, [f32; 3])> = buildings.iter().map(|b| (b.name.clone(), b.placement.position)).collect();
    commands.insert_resource(Buildings(buildings));
    let mut wall_lit = (0, Vec::new());
    let mut shadow_receivers = Vec::new();
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
        let test = (!*unused).then(|| balls.test_at(p, *i));
        let opacity = p.opacity();
        if opacity < 1.0 {
            see_through += 1;
            log.detail(format!("see-through [{}]: translucency={} -> opacity {opacity:.2}", p.name, p.translucency));
        }
        // The lights it carries (M1 7c), on its first part only.
        let mut carried = test.as_ref().and_then(|_| crate::scene_lights::CarriedLights::new(vec![shape_lights[*i].clone()], transform, p.turn_off_dyn_lighting));
        for (part_idx, (mesh, material)) in built[*i].iter().enumerate() {
            // Every material of a see-through object (objects.md Q14); a
            // separate copy, because other copies of the model may be solid.
            let material = crate::object_light::for_object(&mut object_materials, material, opacity, p.haze_solid, p.haze_multiplier);
            let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), transform, ObjectName(p.name.clone())));
            if let Some(orig_tex) = shape_textures[*i].get(part_idx).and_then(|t| t.as_deref()) {
                e.insert(StaticTexture {
                    object_lc: p.name.to_ascii_lowercase(),
                    original_tex: orig_tex.to_string(),
                    original_material: material,
                    swapped_to: None,
                });
            }
            // Unused objects, and hidden ones (objects.md Q18): only with H.
            if *unused || !p.drawn() {
                e.insert((Visibility::Hidden, ShownWithH));
            }
            if let Some(t) = &test {
                e.insert(t.clone());
            }
            if let Some(c) = carried.take() {
                e.insert(c);
            }
            if let Some(b) = model_balls[*i] {
                e.insert(b);
            }
        }
        // A building's draw-distance ball (rule 40, M1 7h).
        if let (false, Some(t)) = (built_walls[*i].is_empty(), &test) {
            log.detail(format!(
                "draw distance: building [{}] ball radius {:.0} (half the .dig box diagonal), centre = box middle {:.0} from its position, turned with it (the original: at its position)",
                p.name,
                t.radius,
                t.centre.length() * UNITS_PER_METRE
            ));
        }
        // Shadows on buildings (M4 4d-7b-3, `characters.md` rule 37 (2)):
        // its drawn surfaces in the world and its draw-distance ball.
        if let (false, Some(t)) = (built_walls[*i].is_empty(), &test) {
            let place = p.transform();
            let surfaces = shadow_surfaces[*i].iter().filter_map(|s| kq8_core::shadow::ReceivingSurface::new(s.iter().map(|tri| tri.map(|c| place.apply(c))).collect())).collect();
            let c = transform.transform_point(t.centre) * UNITS_PER_METRE;
            shadow_receivers.push((t.object, kq8_core::shadow::ReceivingBuilding { name: p.name.clone(), centre: [c.x, -c.z, c.y], radius: t.radius, surfaces }));
        }
        // Point lights (M1 7d): this copy's own light picture and haze
        // factors, so copies of one building are lit separately.
        let own = (!built_walls[*i].is_empty()).then(|| wall_info[*i].clone()).flatten().map(|(atlas, data)| {
            let image = images.add(atlas.image());
            let factors = buffers.add(crate::building_light::factor_buffer(&vec![1.0; data.light.surfaces.len()]));
            (atlas, data, image, factors)
        });
        let mut wall_lights = own.as_ref().map(|(atlas, data, image, factors)| {
            let collider = collider_keys.iter().position(|(n, at)| *n == p.name && *at == p.position);
            if collider.is_some() {
                wall_lit.0 += 1;
            } else if !*unused {
                wall_lit.1.push(p.name.clone());
            }
            crate::wall_lights::WallLights::new(p.name.clone(), data.clone(), atlas.clone(), image.clone(), factors.clone(), collider)
        });
        for (mesh, material) in &built_walls[*i] {
            let material = match (&own, building_materials.get(material).cloned()) {
                (Some((_, _, image, factors)), Some(mut m)) => {
                    m.light = image.clone();
                    m.factors = factors.clone();
                    building_materials.add(m)
                }
                _ => material.clone(),
            };
            let material = crate::building_light::for_building(&mut building_materials, &material, opacity, p.haze_solid);
            let mut e = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material), transform, ObjectName(p.name.clone())));
            if *unused || !p.drawn() {
                e.insert((Visibility::Hidden, ShownWithH));
            }
            if let Some(t) = &test {
                e.insert(t.clone());
            }
            if let Some(w) = wall_lights.take() {
                e.insert(w);
            }
        }
    }
    log.summary(format!(
        "wall point lights (M1 7d): {} building copies can be lit; without a collider (no lights): {:?}",
        wall_lit.0, wall_lit.1
    ));
    let drawn_surfaces: usize = shadow_receivers.iter().map(|(_, b)| b.surfaces.len()).sum();
    let all_surfaces: usize = objects.placed.iter().filter(|(_, i, unused)| !*unused && !built_walls[*i].is_empty()).map(|(_, i, _)| shadow_surfaces[*i].len()).sum();
    log.summary(format!(
        "shadows on buildings (characters.md rule 37 (2), Q135, M4 4d-7b-3 / 5): {} building copies receive, {drawn_surfaces} drawn surfaces ({} not drawn left out); s = {} x the caster's model ball radius; reach = building radius + s + {}; box in the caster's own space: {} s below to s above its ball's centre, the 2 s square and its copy along the sun, sides at most {}; whole surfaces",
        shadow_receivers.len(),
        all_surfaces - drawn_surfaces,
        kq8_core::shadow::REACH_SIZE,
        kq8_core::shadow::BUILDING_REACH,
        kq8_core::shadow::BAND_BELOW_SIZES,
        kq8_core::shadow::BUILDING_BOX
    ));
    commands.insert_resource(crate::shadow_view::BuildingShadows(shadow_receivers));
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
    // KQPointLight objects (M1 7c, rules 24, 25): an entity at the light.
    // Not in the near / far / off test: it is always in the lights group
    // and shines everywhere (rule 37, Q77 b; M1 7g).
    let point_lights = std::mem::take(&mut objects.point_lights);
    log.summary(format!("KQPointLight: {} loaded, always shining (not in the near / far / off test, rule 37)", point_lights.len()));
    for (name, light) in point_lights {
        commands.spawn((Transform::from_translation(Vec3::from(to_bevy(light.position, [0.0; 3]))), crate::scene_lights::PointLightSource { name, light }));
    }
    // Invisible objects (key H): color-coded bounding boxes for triggers and
    // click boxes; 1 m cubes for single points (or zero-extent boxes).
    use crate::load::MarkerKind;
    let point_cube = meshes.add(Cuboid::from_length(100.0 / UNITS_PER_METRE));
    let unit_cube = meshes.add(Cuboid::from_length(1.0));
    let wire_cube = meshes.add(wireframe_cube_mesh());
    let point_green = materials.add(StandardMaterial { base_color: Color::srgb(0.1, 1.0, 0.2), unlit: true, fog_enabled: false, ..default() });
    let point_orange = materials.add(StandardMaterial { base_color: Color::srgb(1.0, 0.55, 0.0), unlit: true, fog_enabled: false, ..default() });
    let box_mats = |materials: &mut Assets<StandardMaterial>, r: f32, g: f32, b: f32, alpha: f32| {
        (
            materials.add(StandardMaterial { base_color: Color::srgb(r, g, b), unlit: true, fog_enabled: false, ..default() }),
            materials.add(StandardMaterial {
                base_color: Color::srgba(r, g, b, alpha),
                alpha_mode: AlphaMode::Blend,
                unlit: true,
                cull_mode: None,
                fog_enabled: false,
                ..default()
            }),
        )
    };
    let damage_mats = box_mats(&mut materials, 1.0, 0.15, 0.15, 0.18);
    let rope_mats = box_mats(&mut materials, 1.0, 0.55, 0.1, 0.18);
    let portal_mats = box_mats(&mut materials, 0.75, 0.3, 1.0, 0.18);
    let walk_mats = box_mats(&mut materials, 0.7, 0.72, 0.78, 0.06);
    let music_mats = box_mats(&mut materials, 0.25, 0.55, 1.0, 0.14);
    let generic_mats = box_mats(&mut materials, 0.1, 0.95, 1.0, 0.14);
    let click_mats = box_mats(&mut materials, 1.0, 0.9, 0.15, 0.18);
    let other_mats = box_mats(&mut materials, 0.1, 1.0, 0.2, 0.15);
    let haze_outer_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.2, 1.0, 0.75), unlit: true, fog_enabled: false, ..default() });
    let haze_inner_mat = materials.add(StandardMaterial { base_color: Color::srgb(0.95, 1.0, 0.25), unlit: true, fog_enabled: false, ..default() });
    let sphere_rings = meshes.add(wireframe_sphere_rings_mesh(64));
    let unit_ring = meshes.add(wireframe_ring_mesh(64));
    let cyl_guides = meshes.add(wireframe_cylinder_guides_mesh());
    let (
        mut n_damage,
        mut n_rope,
        mut n_portal,
        mut n_walk,
        mut n_music,
        mut n_generic,
        mut n_haze,
        mut n_click,
        mut n_other,
        mut n_point,
    ) = (0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize, 0usize);
    for m in &objects.markers {
        if m.kind == MarkerKind::HazeZone {
            n_haze += 1;
            if let Some((transform, _scale)) = m.bounds.and_then(|(min, max)| marker_box_layout(m.position, m.angles, m.rotated, min, max)) {
                commands.spawn((Mesh3d(wire_cube.clone()), MeshMaterial3d(haze_outer_mat.clone()), transform, Visibility::Hidden, ShownWithH));
            }
            let (radius, inner_radius) = m.hazer_radii.unwrap_or((0.0, 0.0));
            let center = Vec3::from(to_bevy(m.position, [0.0; 3]));
            if radius > 1e-3 {
                commands.spawn((
                    Mesh3d(sphere_rings.clone()),
                    MeshMaterial3d(haze_outer_mat.clone()),
                    Transform::from_translation(center).with_scale(Vec3::splat(radius / UNITS_PER_METRE)),
                    Visibility::Hidden,
                    ShownWithH,
                ));
            }
            let inner_r = inner_radius.min(radius);
            if inner_r > 1e-3 {
                commands.spawn((
                    Mesh3d(unit_ring.clone()),
                    MeshMaterial3d(haze_inner_mat.clone()),
                    Transform::from_translation(center).with_scale(Vec3::splat(inner_r / UNITS_PER_METRE)),
                    Visibility::Hidden,
                    ShownWithH,
                ));
                if radius > 1e-3 {
                    commands.spawn((
                        Mesh3d(cyl_guides.clone()),
                        MeshMaterial3d(haze_inner_mat.clone()),
                        Transform::from_translation(center).with_scale(Vec3::new(
                            inner_r / UNITS_PER_METRE,
                            2.0 * radius / UNITS_PER_METRE,
                            inner_r / UNITS_PER_METRE,
                        )),
                        Visibility::Hidden,
                        ShownWithH,
                    ));
                }
            }
            log.detail(format!(
                "marker haze zone [{}] center ({:.0}, {:.0}, {:.0}), cube {:.0}, radius {radius:.0}, innerRadius {inner_radius:.0}",
                m.name,
                m.position[0],
                m.position[1],
                m.position[2],
                2.0 * radius
            ));
        } else if let Some((transform, scale)) = m.bounds.and_then(|(min, max)| marker_box_layout(m.position, m.angles, m.rotated, min, max)) {
            let (wire_mat, fill_mat) = match m.kind {
                MarkerKind::DamageTrap => {
                    n_damage += 1;
                    damage_mats.clone()
                }
                MarkerKind::RopeTrap => {
                    n_rope += 1;
                    rope_mats.clone()
                }
                MarkerKind::PortalTrap => {
                    n_portal += 1;
                    portal_mats.clone()
                }
                MarkerKind::WalkRunTrap => {
                    n_walk += 1;
                    walk_mats.clone()
                }
                MarkerKind::MusicTrap => {
                    n_music += 1;
                    music_mats.clone()
                }
                MarkerKind::GenericTrap => {
                    n_generic += 1;
                    generic_mats.clone()
                }
                MarkerKind::ClickBox => {
                    n_click += 1;
                    click_mats.clone()
                }
                MarkerKind::OtherBox | MarkerKind::Point | MarkerKind::HazeZone => {
                    n_other += 1;
                    other_mats.clone()
                }
            };
            commands.spawn((Mesh3d(wire_cube.clone()), MeshMaterial3d(wire_mat), transform, Visibility::Hidden, ShownWithH));
            if scale.x > 1e-4 && scale.y > 1e-4 && scale.z > 1e-4 {
                commands.spawn((Mesh3d(unit_cube.clone()), MeshMaterial3d(fill_mat), transform, Visibility::Hidden, ShownWithH));
            }
            let center = game_position(transform.translation);
            let ext = [scale.x * UNITS_PER_METRE, scale.z * UNITS_PER_METRE, scale.y * UNITS_PER_METRE];
            log.detail(format!(
                "marker box [{}] ({}, {}) center ({:.0}, {:.0}, {:.0}), size {:.0} x {:.0} x {:.0}",
                m.name,
                m.class,
                m.kind.label(),
                center[0],
                center[1],
                center[2],
                ext[0],
                ext[1],
                ext[2]
            ));
        } else {
            n_point += 1;
            // Cube resting on the point, so ground markers are not half buried.
            let at = Vec3::from(to_bevy(m.position, [0.0; 3])) + Vec3::Y * 50.0 / UNITS_PER_METRE;
            commands.spawn((
                Mesh3d(point_cube.clone()),
                MeshMaterial3d(if m.anm { point_orange.clone() } else { point_green.clone() }),
                Transform::from_translation(at),
                Visibility::Hidden,
                ShownWithH,
            ));
        }
    }
    log.summary(format!(
        "markers (key H): {} boxed ({n_damage} damage, {n_rope} rope, {n_portal} portal, {n_walk} walk/run, {n_music} music, {n_generic} generic, {n_click} click box, {n_other} other), {n_haze} haze zone, {n_point} point cubes",
        n_damage + n_rope + n_portal + n_walk + n_music + n_generic + n_click + n_other
    ));
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

/// Transform and Bevy-space extents `(x_game, z_game, y_game) / UNITS_PER_METRE`
/// for a unit cube `[-0.5, 0.5]^3` matching an unmodeled object's `BBMin`/`BBMax`
/// box in game space. Returns `None` when all three extents are `<= 1e-3` (point).
pub fn marker_box_layout(position: [f32; 3], angles: [f32; 3], rotated: bool, min: [f32; 3], max: [f32; 3]) -> Option<(Transform, Vec3)> {
    let lo: [f32; 3] = std::array::from_fn(|k| min[k].min(max[k]));
    let hi: [f32; 3] = std::array::from_fn(|k| min[k].max(max[k]));
    let ext: [f32; 3] = std::array::from_fn(|k| hi[k] - lo[k]);
    if ext.iter().all(|&e| e <= 1e-3) {
        return None;
    }
    let center_local: [f32; 3] = std::array::from_fn(|k| 0.5 * (lo[k] + hi[k]));
    let r = if rotated {
        kq8_data::objects::rotation(angles)
    } else {
        [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
    };
    let center_world: [f32; 3] = std::array::from_fn(|k| position[k] + center_local[0] * r[0][k] + center_local[1] * r[1][k] + center_local[2] * r[2][k]);
    let base = bevy_transform_from(r, center_world);
    let scale = Vec3::new(ext[0] / UNITS_PER_METRE, ext[2] / UNITS_PER_METRE, ext[1] / UNITS_PER_METRE);
    Some((Transform { scale, ..base }, scale))
}

/// 12-edge `LineList` wireframe of a unit cube `[-0.5, 0.5]^3`.
pub fn wireframe_cube_mesh() -> Mesh {
    const H: f32 = 0.5;
    let corners = [
        [-H, -H, -H],
        [H, -H, -H],
        [H, H, -H],
        [-H, H, -H],
        [-H, -H, H],
        [H, -H, H],
        [H, H, H],
        [-H, H, H],
    ];
    let edges = [
        [0, 1], [1, 2], [2, 3], [3, 0],
        [4, 5], [5, 6], [6, 7], [7, 4],
        [0, 4], [1, 5], [2, 6], [3, 7],
    ];
    let mut positions = Vec::with_capacity(edges.len() * 2);
    for [a, b] in edges {
        positions.push(corners[a]);
        positions.push(corners[b]);
    }
    line_list_mesh(positions)
}

fn line_list_mesh(positions: Vec<[f32; 3]>) -> Mesh {
    let normals = vec![[0.0, 1.0, 0.0]; positions.len()];
    let uvs = vec![[0.0, 0.0]; positions.len()];
    Mesh::new(bevy::mesh::PrimitiveTopology::LineList, bevy::asset::RenderAssetUsages::default())
        .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
        .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
        .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
}

/// Horizontal unit circle (`y = 0`) as a `LineList` wireframe.
pub fn wireframe_ring_mesh(segments: usize) -> Mesh {
    let n = segments.max(8);
    let mut positions = Vec::with_capacity(n * 2);
    for i in 0..n {
        let a0 = (i as f32) * std::f32::consts::TAU / (n as f32);
        let a1 = ((i + 1) as f32) * std::f32::consts::TAU / (n as f32);
        positions.push([a0.cos(), 0.0, a0.sin()]);
        positions.push([a1.cos(), 0.0, a1.sin()]);
    }
    line_list_mesh(positions)
}

/// Three orthogonal unit-radius great circles (XZ, XY, YZ) as a `LineList` wireframe.
pub fn wireframe_sphere_rings_mesh(segments: usize) -> Mesh {
    let n = segments.max(8);
    let mut positions = Vec::with_capacity(n * 6);
    for i in 0..n {
        let a0 = (i as f32) * std::f32::consts::TAU / (n as f32);
        let a1 = ((i + 1) as f32) * std::f32::consts::TAU / (n as f32);
        let (c0, s0, c1, s1) = (a0.cos(), a0.sin(), a1.cos(), a1.sin());
        positions.push([c0, 0.0, s0]);
        positions.push([c1, 0.0, s1]);
        positions.push([c0, s0, 0.0]);
        positions.push([c1, s1, 0.0]);
        positions.push([0.0, c0, s0]);
        positions.push([0.0, c1, s1]);
    }
    line_list_mesh(positions)
}

/// Four vertical guide lines (`y = -0.5..+0.5` at cardinal unit circle points).
pub fn wireframe_cylinder_guides_mesh() -> Mesh {
    let mut positions = Vec::with_capacity(8);
    for [x, z] in [[1.0, 0.0], [-1.0, 0.0], [0.0, 1.0], [0.0, -1.0]] {
        positions.push([x, -0.5, z]);
        positions.push([x, 0.5, z]);
    }
    line_list_mesh(positions)
}

/// H shows or hides everything marked `ShownWithH`.
pub fn toggle_markers(
    keys: Res<ButtonInput<KeyCode>>,
    shared: Res<Shared>,
    mut shown: Local<bool>,
    mut markers: Query<&mut Visibility, With<ShownWithH>>,
    mut label: Query<&mut Text, With<HLabel>>,
    (change, mut seen): (Res<crate::world_build::WorldChange>, Local<u64>),
) {
    // A new world starts with its markers hidden (M6 6e-2).
    if crate::world_build::new_world(&change, &mut seen) {
        *shown = false;
    }
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sorted_corners(pts: &mut [Vec3; 8]) {
        pts.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)).then(a.z.total_cmp(&b.z)));
    }

    #[test]
    fn marker_box_corners_match_game_space_rotated_and_axis_aligned() {
        let cases = [
            // Off-center RiverControl (`rivercn2`)
            ([123_125.0, 195_000.0, 889.0], [0.0, 0.0, 0.0], true, [0.0, 0.0, -200.0], [75_925.0, 8_100.0, 10.0]),
            // Axis-aligned trap ignores non-zero placement angles
            ([70_248.0, 100_809.0, 1_584.0], [0.3, 0.4, 1.2], false, [-23_250.0, -28_686.0, 0.0], [24_016.0, 29_967.0, 3_000.0]),
            // Rotated 90 deg around Z
            ([10_000.0, 20_000.0, 500.0], [0.0, 0.0, std::f32::consts::FRAC_PI_2], true, [-100.0, -200.0, 0.0], [300.0, 600.0, 1_000.0]),
        ];
        for (pos, angles, rotated, min, max) in cases {
            let (tr, scale) = marker_box_layout(pos, angles, rotated, min, max).unwrap();
            assert!(scale.x > 0.0 && scale.y > 0.0 && scale.z > 0.0);
            let mut got = [Vec3::ZERO; 8];
            let mut want = [Vec3::ZERO; 8];
            let r = if rotated {
                kq8_data::objects::rotation(angles)
            } else {
                [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]
            };
            for (idx, bit) in (0..8).enumerate() {
                let sx = if bit & 1 == 0 { -0.5 } else { 0.5 };
                let sy = if bit & 2 == 0 { -0.5 } else { 0.5 };
                let sz = if bit & 4 == 0 { -0.5 } else { 0.5 };
                got[idx] = tr.transform_point(Vec3::new(sx, sy, sz));
                let lx = if bit & 1 == 0 { min[0] } else { max[0] };
                let ly = if bit & 2 == 0 { min[1] } else { max[1] };
                let lz = if bit & 4 == 0 { min[2] } else { max[2] };
                let gx = pos[0] + lx * r[0][0] + ly * r[1][0] + lz * r[2][0];
                let gy = pos[1] + lx * r[0][1] + ly * r[1][1] + lz * r[2][1];
                let gz = pos[2] + lx * r[0][2] + ly * r[1][2] + lz * r[2][2];
                want[idx] = Vec3::from(to_bevy([gx, gy, gz], [0.0; 3]));
            }
            sorted_corners(&mut got);
            sorted_corners(&mut want);
            for (g, w) in got.iter().zip(want.iter()) {
                assert!((*g - *w).length() < 1e-2, "corner mismatch: got {g:?}, want {w:?}");
            }
        }
    }

    #[test]
    fn marker_box_falls_back_on_point_and_allows_flat_box() {
        assert!(marker_box_layout([100.0, 200.0, 300.0], [0.0; 3], false, [0.0; 3], [0.0; 3]).is_none());
        assert!(marker_box_layout([100.0, 200.0, 300.0], [0.0; 3], false, [5.0, 5.0, 5.0], [5.0, 5.0, 5.0]).is_none());

        // Reversed bounds are normalized and flat dz == 0 produces a valid 2D box outline.
        let (tr, scale) = marker_box_layout([0.0; 3], [0.0; 3], false, [500.0, 200.0, 10.0], [-500.0, -200.0, 10.0]).unwrap();
        assert!((scale.x - 10.0).abs() < 1e-5 && scale.y == 0.0 && (scale.z - 4.0).abs() < 1e-5);
        assert!((tr.translation.y - 10.0 / UNITS_PER_METRE).abs() < 1e-5);
    }

    #[test]
    fn wireframe_cube_has_twelve_edges_with_normals_and_uvs() {
        let mesh = wireframe_cube_mesh();
        assert_eq!(mesh.primitive_topology(), bevy::mesh::PrimitiveTopology::LineList);
        assert_eq!(mesh.count_vertices(), 24);
        assert!(mesh.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(mesh.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }

    #[test]
    fn wireframe_hazer_rings_and_guides_match_line_list_contract() {
        let sphere = wireframe_sphere_rings_mesh(64);
        assert_eq!(sphere.primitive_topology(), bevy::mesh::PrimitiveTopology::LineList);
        assert_eq!(sphere.count_vertices(), 384);
        assert!(sphere.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(sphere.attribute(Mesh::ATTRIBUTE_UV_0).is_some());

        let ring = wireframe_ring_mesh(64);
        assert_eq!(ring.primitive_topology(), bevy::mesh::PrimitiveTopology::LineList);
        assert_eq!(ring.count_vertices(), 128);
        assert!(ring.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(ring.attribute(Mesh::ATTRIBUTE_UV_0).is_some());

        let guides = wireframe_cylinder_guides_mesh();
        assert_eq!(guides.primitive_topology(), bevy::mesh::PrimitiveTopology::LineList);
        assert_eq!(guides.count_vertices(), 8);
        assert!(guides.attribute(Mesh::ATTRIBUTE_NORMAL).is_some());
        assert!(guides.attribute(Mesh::ATTRIBUTE_UV_0).is_some());
    }
}
