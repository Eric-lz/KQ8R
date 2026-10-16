//! Models lit by the world's ambient light and suns (M1 step 5b,
//! docs/DESIGN.md; rules: docs/behaviour/object-lighting.md, rule numbers
//! are that note's). See docs/provenance/README.md.
//!
//! Every model (placed objects, animated objects, doors and chests, items,
//! particles, Connor, the `--model` view) is drawn with `ObjectMaterial`.
//! The light is worked out on the graphics card (`object_light.wesl`), per
//! corner, from the corner's normal turned into world space with the
//! object's own rotation (rules 4, 5, 7, 9). This way copies of one mesh
//! that face different ways, and objects that turn, are all lit right.
//!
//! - The lights are the files the world's `swtohwlt.cs` loads (rule 1),
//!   read by `load::load_object_lights`.
//! - Lighting kind per material (rule 7): smooth, flat (all three corners
//!   carry the first corner's normal, see `model_mesh`), none. A material
//!   whose shading bits the note doesn't cover: none (STAND-IN; no model
//!   material in the data has them).
//! - The per-object "unlit" switch is treated as always off (rule 10).
//! - STAND-IN (as the ground, M1 4c): the multiply is on the colour as shown
//!   on screen; the shader turns the light into Bevy's linear colour space.
//! - F7 (`light_view`) changes the mode of every object material too:
//!   x1 (the note), x2 (doubled, capped; for comparison), off (full bright).
//! - Haze (M1 step 6, rules 19-23): per object, from the camera's distance
//!   to the object's position, in the shader; F8 switches it on and off.
//!   Objects with their own `b_hazeSolid` / `hazeMultiplier` get their own
//!   material copy (`for_object`).
//! - Particles (M1 6g, rule 30): one haze per emitter, from the emitter's
//!   location (`haze_from`, set by `effects_view`).
//! - Objects far from Connor are switched off by `draw_distance` (M1 6h,
//!   rules 34-40), not here.
//! - Fog over the ground and the background (M1 step 6b, STAND-IN from the
//!   user's check, Q68): Bevy's distance fog on every 3D camera (linear,
//!   haze start to end, haze colour), and the clear colour = the haze
//!   colour; `sync_fog`. F8 switches it with the rest.
//! - Not done: point lights (rule 24), the brightness setting (rule 17;
//!   1.0 at the shipped value).

use bevy::asset::embedded_asset;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use bevy::shader::ShaderRef;
use std::time::Instant;

use kq8_data::haze::{Haze, Hazed};
use kq8_data::lights::Lights;
use kq8_data::Lighting;

use crate::light_view::{GroundLight, LightMode};
use crate::load::{Look, Shading};
use crate::{Shared, UNITS_PER_METRE};

const SHADER: &str = "embedded://kq8_viewer/object_light.wesl";

/// Suns the shader takes; more are left out and logged (the data has one
/// per world).
pub const MAX_SUNS: usize = 4;

/// The material's settings, as the shader reads them (`Settings` in
/// `object_light.wesl`).
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Settings {
    /// Linear colour (white for textures); alpha = opacity.
    pub base_colour: Vec4,
    pub ambient: Vec4,
    pub sun_colour: [Vec4; MAX_SUNS],
    /// In Bevy's world space, not unit length (rule 5).
    pub sun_aim: [Vec4; MAX_SUNS],
    /// Haze colour, screen colour 0..1 (rule 23); w unused.
    pub haze_colour: Vec4,
    pub sun_count: u32,
    /// 0 none, 1 flat, 2 smooth.
    pub lighting: u32,
    /// 0 x1, 1 x2, 2 off.
    pub mode: u32,
    pub textured: u32,
    /// 1 = haze on (F8).
    pub haze_on: u32,
    /// Game units: haze start and end (rule 19).
    pub haze_start: f32,
    pub haze_end: f32,
    /// Material flag 0x10000 (rule 20).
    pub haze_halved: u32,
    /// The object's `b_hazeSolid` (rule 21).
    pub haze_solid: u32,
    /// The object's `hazeMultiplier` (rule 22).
    pub haze_multiplier: f32,
    /// Game units per Bevy unit (distances in the shader are in Bevy's).
    pub game_units: f32,
    /// Particles (rule 30): xyz = their emitter's location in Bevy's world
    /// space, w = 1: the haze is measured from there instead of from the
    /// drawn part's own position (w = 0).
    pub haze_from: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct ObjectMaterial {
    #[uniform(0)]
    pub settings: Settings,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    pub alpha_mode: AlphaMode,
}

impl Material for ObjectMaterial {
    fn vertex_shader() -> ShaderRef {
        SHADER.into()
    }

    fn fragment_shader() -> ShaderRef {
        SHADER.into()
    }

    fn alpha_mode(&self) -> AlphaMode {
        self.alpha_mode
    }

    fn enable_prepass() -> bool {
        false
    }

    fn enable_shadows() -> bool {
        false
    }
}

/// Registers the shader and the material.
pub fn add(app: &mut App) {
    embedded_asset!(app, "object_light.wesl");
    app.add_plugins(MaterialPlugin::<ObjectMaterial>::default());
}

fn mode_code(mode: LightMode) -> u32 {
    match mode {
        LightMode::X1 => 0,
        LightMode::X2 => 1,
        LightMode::Off => 2,
    }
}

fn lighting_code(lighting: Lighting) -> u32 {
    match lighting {
        Lighting::None => 0,
        Lighting::Flat => 1,
        Lighting::Smooth => 2,
    }
}

/// Game direction (Z up) -> Bevy direction (Y up), length kept.
fn direction_to_bevy(v: [f32; 3]) -> Vec3 {
    Vec3::new(v[0], v[2], -v[1])
}

/// The world's lights, haze, and the current modes.
#[derive(Resource)]
pub struct ObjectLight {
    pub lights: Lights,
    pub mode: LightMode,
    pub haze: Haze,
    /// F8 / `--haze`.
    pub haze_on: bool,
}

impl ObjectLight {
    pub fn new(lights: Lights, mode: LightMode, haze: Haze, haze_on: bool) -> Self {
        ObjectLight { lights, mode, haze, haze_on }
    }

    fn settings(&self, base_colour: Vec4, shading: Shading, textured: bool) -> Settings {
        let mut sun_colour = [Vec4::ZERO; MAX_SUNS];
        let mut sun_aim = [Vec4::ZERO; MAX_SUNS];
        for (k, sun) in self.lights.suns.iter().take(MAX_SUNS).enumerate() {
            sun_colour[k] = Vec3::from(sun.colour).extend(0.0);
            sun_aim[k] = direction_to_bevy(sun.aim).extend(0.0);
        }
        Settings {
            base_colour,
            ambient: Vec3::from(self.lights.ambient).extend(0.0),
            sun_colour,
            sun_aim,
            haze_colour: Vec3::from(self.haze.colour).extend(0.0),
            sun_count: self.lights.suns.len().min(MAX_SUNS) as u32,
            lighting: lighting_code(shading.lighting),
            mode: mode_code(self.mode),
            textured: u32::from(textured),
            haze_on: u32::from(self.haze_on),
            haze_start: self.haze.start,
            haze_end: self.haze.end,
            haze_halved: u32::from(shading.haze_halved),
            haze_solid: 0,
            haze_multiplier: 1.0,
            game_units: UNITS_PER_METRE,
            haze_from: Vec4::ZERO,
        }
    }

    /// The light the shader gives a corner with this **unit** normal in
    /// Bevy's world space, before the mode (for the log and tests; same
    /// rules as the shader).
    pub fn corner_light(&self, bevy_normal: Vec3) -> [f32; 3] {
        // Back to game space: Bevy (x, y, z) = game (x, z, -y).
        self.lights.corner_light([bevy_normal.x, -bevy_normal.z, bevy_normal.y])
    }
}

/// For the log ("How to check"): minimum, mean and maximum light (red
/// channel, mode left out) over smooth corners whose normals (Bevy model
/// space) are turned by the object's `rotation`; the shader's rules,
/// worked out on the CPU.
pub fn light_stats(light: &ObjectLight, normals: &[Vec3], rotation: Quat) -> Option<String> {
    let values: Vec<f32> = normals
        .iter()
        .filter(|n| n.length_squared() > 0.0)
        .map(|n| light.corner_light((rotation * *n).normalize())[0])
        .collect();
    let (min, max) = values.iter().fold((f32::MAX, f32::MIN), |(a, b), &v| (a.min(v), b.max(v)));
    (!values.is_empty()).then(|| {
        format!("min {min:.3} mean {:.3} max {max:.3} over {} smooth corners", values.iter().sum::<f32>() / values.len() as f32, values.len())
    })
}

/// Bevy-space normals of a group's corners when it is lit smooth (else
/// none), for `light_stats`.
pub fn smooth_normals(triangles: &[kq8_data::shape::Triangle], lighting: Lighting) -> Vec<Vec3> {
    if lighting != Lighting::Smooth {
        return Vec::new();
    }
    triangles.iter().flat_map(|t| t.normals).map(direction_to_bevy).collect()
}

/// An image for a model texture; `address` repeats or clamps it.
pub fn texture_image(width: u32, height: u32, rgba: Vec<u8>, address: ImageAddressMode) -> Image {
    let mut image = Image::new(
        Extent3d { width, height, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
        address_mode_u: address,
        address_mode_v: address,
        ..ImageSamplerDescriptor::linear()
    });
    image
}

/// A model material for a texture, colour or missing material (magenta,
/// never lit, so it stays visible).
pub fn object_material(look: Look, shading: Shading, light: &ObjectLight, images: &mut Assets<Image>) -> ObjectMaterial {
    let (base, texture, shading) = match look {
        Look::Texture { width, height, rgba, .. } => {
            let image = images.add(texture_image(width, height, rgba, ImageAddressMode::Repeat));
            (Vec4::ONE, Some(image), shading)
        }
        Look::Color([r, g, b]) => (LinearRgba::from(Color::srgb_u8(r, g, b)).to_vec4(), None, shading),
        Look::Missing(_) => (LinearRgba::from(Color::srgb(1.0, 0.0, 1.0)).to_vec4(), None, Shading { lighting: Lighting::None, ..shading }),
    };
    let textured = texture.is_some();
    ObjectMaterial { settings: light.settings(base, shading, textured), texture, alpha_mode: AlphaMode::Opaque }
}

/// A copy of `material` drawn at `opacity` (0..1): an object's
/// `translucency` (objects.md "See-through objects", Q14).
pub fn with_opacity(material: &ObjectMaterial, opacity: f32) -> ObjectMaterial {
    let mut m = material.clone();
    m.settings.base_colour.w *= opacity;
    m.alpha_mode = AlphaMode::Blend;
    m
}

/// `material` as it is (opacity 1), or a see-through copy of it.
pub fn see_through(materials: &mut Assets<ObjectMaterial>, material: &Handle<ObjectMaterial>, opacity: f32) -> Handle<ObjectMaterial> {
    for_object(materials, material, opacity, false, 1.0)
}

/// `material` as it is, or a copy for one object that is drawn
/// differently: see-through (`translucency`, objects.md Q14), `b_hazeSolid`
/// (rule 21) or a `hazeMultiplier` other than 1 (rule 22).
pub fn for_object(
    materials: &mut Assets<ObjectMaterial>,
    material: &Handle<ObjectMaterial>,
    opacity: f32,
    haze_solid: bool,
    haze_multiplier: f32,
) -> Handle<ObjectMaterial> {
    if opacity >= 1.0 && !haze_solid && haze_multiplier == 1.0 {
        return material.clone();
    }
    let Some(mut m) = materials.get(material).cloned() else { return material.clone() };
    if opacity < 1.0 {
        m = with_opacity(&m, opacity);
    }
    m.settings.haze_solid = u32::from(haze_solid);
    m.settings.haze_multiplier = haze_multiplier;
    materials.add(m)
}

/// Logs the lights at start.
pub fn report_lights(light: Res<ObjectLight>, materials: Res<Assets<ObjectMaterial>>, shared: Res<Shared>) {
    let mut log = shared.log.lock().unwrap();
    let l = &light.lights;
    log.summary(format!(
        "object light: ambient ({:.3}, {:.3}, {:.3}) from {} light(s); {} sun(s){}; mode {} at start",
        l.ambient[0],
        l.ambient[1],
        l.ambient[2],
        l.ambient_count,
        l.suns.len(),
        if l.suns.len() > MAX_SUNS { format!(" (only the first {MAX_SUNS} drawn)") } else { String::new() },
        light.mode.name()
    ));
    for s in &l.suns {
        log.summary(format!(
            "  sun colour ({:.3}, {:.3}, {:.3}) aim ({}, {}, {})",
            s.colour[0], s.colour[1], s.colour[2], s.aim[0], s.aim[1], s.aim[2]
        ));
    }
    let mut kinds = [0usize; 3];
    for (_, m) in materials.iter() {
        kinds[m.settings.lighting as usize] += 1;
    }
    log.summary(format!(
        "object materials: {} smooth, {} flat, {} none (incl. missing ones)",
        kinds[2], kinds[1], kinds[0]
    ));
    let h = &light.haze;
    log.summary(format!(
        "haze: {} at start (F8); starts at {:.0}, full at {:.0} game units from the camera; objects off at {:.0} (visibleDistance) from Connor; colour ({:.0}, {:.0}, {:.0}) of 255; {} materials halve it, {} solid, {} with a multiplier",
        if light.haze_on { "on" } else { "off" },
        h.start,
        h.end,
        h.visible,
        h.colour[0] * 255.0,
        h.colour[1] * 255.0,
        h.colour[2] * 255.0,
        materials.iter().filter(|(_, m)| m.settings.haze_halved == 1).count(),
        materials.iter().filter(|(_, m)| m.settings.haze_solid == 1).count(),
        materials.iter().filter(|(_, m)| m.settings.haze_multiplier != 1.0).count()
    ));
}

#[derive(Component)]
pub struct HazeLabel;

fn haze_label_text(on: bool) -> String {
    format!("Haze: {} (F8)", if on { "on" } else { "off" })
}

/// The on-screen line under the F7 line.
pub fn setup_haze_label(mut commands: Commands, light: Res<ObjectLight>) {
    commands.spawn((
        Text::new(haze_label_text(light.haze_on)),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(74.0), left: Val::Px(10.0), ..default() },
        HazeLabel,
    ));
}

/// F8: haze on / off in every object material.
pub fn toggle_haze(
    keys: Res<ButtonInput<KeyCode>>,
    mut light: ResMut<ObjectLight>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut label: Query<&mut Text, With<HazeLabel>>,
    shared: Res<Shared>,
) {
    if !keys.just_pressed(KeyCode::F8) {
        return;
    }
    light.haze_on = !light.haze_on;
    let on = u32::from(light.haze_on);
    let mut changed = 0;
    for (_, m) in materials.iter_mut() {
        m.settings.haze_on = on;
        changed += 1;
    }
    for mut t in &mut label {
        t.0 = haze_label_text(light.haze_on);
    }
    shared.log.lock().unwrap().summary(format!("key F8: haze {} ({changed} materials)", if light.haze_on { "on" } else { "off" }));
}

/// The placed objects, for the haze log: name, position (game units),
/// `b_hazeSolid`; and the buildings: `b_hazeSolid` and each wall
/// surface's corners (world space, game units).
#[derive(Resource, Default)]
pub struct HazeCheck {
    objects: Vec<(String, [f32; 3], bool)>,
    walls: Vec<(bool, Vec<Vec<[f32; 3]>>)>,
    last: Option<Instant>,
}

impl HazeCheck {
    pub fn new(objects: Vec<(String, [f32; 3], bool)>, walls: Vec<(bool, Vec<Vec<[f32; 3]>>)>) -> Self {
        HazeCheck { objects, walls, last: None }
    }
}

/// The background is the world's `newSky` palette colour, with or without
/// the haze (rule 31, M1 6d). Bevy's distance fog is no longer used: the
/// ground has its own haze (`ground_haze`, M1 6e).
pub fn sync_background(light: Res<ObjectLight>, mut clear: ResMut<ClearColor>, shared: Res<Shared>) {
    let [r, g, b] = light.haze.background;
    let want_clear = Color::srgb(r, g, b);
    if clear.0 != want_clear {
        clear.0 = want_clear;
        shared.log.lock().unwrap().summary(format!("background colour ({:.0}, {:.0}, {:.0}) of 255", r * 255.0, g * 255.0, b * 255.0));
    }
}

/// Every second (`viewer.log`, detail): how many placed objects are at
/// full haze from the camera (left out unless `b_hazeSolid`), and the
/// nearest such distance (the note's "How to check": 22,950 in Daventry).
/// Material halving (0x10000) is not counted here. Also how many wall
/// surfaces are left out by the haze (rule 29, M1 6f).
pub fn report_haze(
    mut check: ResMut<HazeCheck>,
    light: Res<ObjectLight>,
    camera: Query<&Transform, With<Camera3d>>,
    shared: Res<Shared>,
) {
    let now = Instant::now();
    if check.last.is_some_and(|t| now.duration_since(t).as_secs_f32() < 1.0) {
        return;
    }
    check.last = Some(now);
    let Some(cam) = camera.iter().next() else { return };
    let c = cam.translation * UNITS_PER_METRE;
    let cam_game = [c.x, -c.z, c.y];
    let (mut gone, mut nearest_gone, mut fading) = (0, f32::MAX, 0);
    for (_, p, solid) in &check.objects {
        let d = ((p[0] - cam_game[0]).powi(2) + (p[1] - cam_game[1]).powi(2) + (p[2] - cam_game[2]).powi(2)).sqrt();
        let hazed = kq8_data::haze::hazed(light.haze.amount(d), false, *solid, 1.0, false);
        match hazed {
            Hazed::Hidden => {
                gone += 1;
                nearest_gone = nearest_gone.min(d);
            }
            Hazed::Blend(h) if h > 0.01 => fading += 1,
            _ => {}
        }
    }
    let (mut surfaces, mut walls_gone, mut walls_solid) = (0, 0, 0);
    for (solid, walls) in &check.walls {
        for corners in walls {
            surfaces += 1;
            match crate::building_light::surface_haze(&light.haze, corners, cam_game, *solid) {
                None => walls_gone += 1,
                Some(h) if h >= crate::building_light::WALL_MAX_HAZE => walls_solid += 1,
                Some(_) => {}
            }
        }
    }
    shared.log.lock().unwrap().detail(format!(
        "haze {}: camera ({:.0}, {:.0}, {:.0}); of {} placed objects {fading} fading, {gone} at full haze (left out){}; of {surfaces} wall surfaces {walls_gone} left out, {walls_solid} kept at 0.99 (b_hazeSolid)",
        if light.haze_on { "on" } else { "off" },
        cam_game[0],
        cam_game[1],
        cam_game[2],
        check.objects.len(),
        if gone > 0 { format!(", nearest at {nearest_gone:.0}") } else { String::new() }
    ));
}

/// F7 changes the ground's mode (`light_view`); every object material
/// follows it.
pub fn follow_ground_mode(
    ground: Option<Res<GroundLight>>,
    mut light: ResMut<ObjectLight>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    shared: Res<Shared>,
) {
    let Some(ground) = ground else { return };
    if ground.mode == light.mode {
        return;
    }
    light.mode = ground.mode;
    let code = mode_code(light.mode);
    let mut changed = 0;
    for (_, m) in materials.iter_mut() {
        m.settings.mode = code;
        changed += 1;
    }
    shared.log.lock().unwrap().summary(format!("object light mode {} ({changed} materials)", light.mode.name()));
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::kq_text;

    #[test]
    fn suns_in_bevy_space() {
        let mut lights = Lights::default();
        lights.add_sections(
            "sun",
            &kq_text::parse(
                b"[s]\nclassType=KQLight\ntsLightType=1\ntsLightAim.x=0\ntsLightAim.y=1\ntsLightAim.z=-0.66\n\
                  tsLightIntensity.red=0.1\ntsLightIntensity.green=0.1\ntsLightIntensity.blue=0.1\n",
            ),
        );
        lights.add_sections("amb", &kq_text::parse(b"[a]\nclassType=KQAmbientLight\nintensity.red=0.3\nintensity.green=0.3\nintensity.blue=0.3\n"));
        let light = ObjectLight::new(lights, LightMode::X1, Haze::new(18_000.0, 27_000.0, [0.0; 3]), true);
        let s = light.settings(Vec4::ONE, Shading { lighting: Lighting::Smooth, haze_halved: true }, true);
        // Game aim (0, 1, -0.66) -> Bevy (0, -0.66, -1), length kept.
        assert_eq!(s.sun_aim[0], Vec4::new(0.0, -0.66, -1.0, 0.0));
        assert_eq!((s.sun_count, s.lighting, s.mode, s.textured), (1, 2, 0, 1));
        assert_eq!((s.haze_on, s.haze_start, s.haze_end, s.haze_halved, s.haze_solid), (1, 18_000.0, 22_950.0, 1, 0));
        // Facing up in Bevy (+Y) is facing up in the game: 0.366.
        assert!((light.corner_light(Vec3::Y)[0] - 0.366).abs() < 1e-5);
        // Bevy's -Z is the game's +Y: facing away from the sun's y part.
        let away = Vec3::new(0.0, -0.66, -1.0).normalize();
        assert!((light.corner_light(away)[0] - 0.3).abs() < 1e-5);
        assert!((light.corner_light(-away)[0] - (0.3 + 0.1 * (1.0f32 + 0.66 * 0.66).sqrt())).abs() < 1e-5);
    }
}
