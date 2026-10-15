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
//! - Not done: point lights (rule 24), haze (rules 19-23), the brightness
//!   setting (rule 17; 1.0 at the shipped value).

use bevy::asset::embedded_asset;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat};
use bevy::asset::RenderAssetUsages;
use bevy::shader::ShaderRef;
use kq8_data::lights::Lights;
use kq8_data::Lighting;

use crate::light_view::{GroundLight, LightMode};
use crate::load::Look;
use crate::Shared;

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
    pub sun_count: u32,
    /// 0 none, 1 flat, 2 smooth.
    pub lighting: u32,
    /// 0 x1, 1 x2, 2 off.
    pub mode: u32,
    pub textured: u32,
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

/// The world's lights and the current mode.
#[derive(Resource)]
pub struct ObjectLight {
    pub lights: Lights,
    pub mode: LightMode,
}

impl ObjectLight {
    pub fn new(lights: Lights, mode: LightMode) -> Self {
        ObjectLight { lights, mode }
    }

    fn settings(&self, base_colour: Vec4, lighting: Lighting, textured: bool) -> Settings {
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
            sun_count: self.lights.suns.len().min(MAX_SUNS) as u32,
            lighting: lighting_code(lighting),
            mode: mode_code(self.mode),
            textured: u32::from(textured),
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
pub fn object_material(look: Look, lighting: Lighting, light: &ObjectLight, images: &mut Assets<Image>) -> ObjectMaterial {
    let (base, texture, lighting) = match look {
        Look::Texture { width, height, rgba, .. } => {
            let image = images.add(texture_image(width, height, rgba, ImageAddressMode::Repeat));
            (Vec4::ONE, Some(image), lighting)
        }
        Look::Color([r, g, b]) => (LinearRgba::from(Color::srgb_u8(r, g, b)).to_vec4(), None, lighting),
        Look::Missing(_) => (LinearRgba::from(Color::srgb(1.0, 0.0, 1.0)).to_vec4(), None, Lighting::None),
    };
    let textured = texture.is_some();
    ObjectMaterial { settings: light.settings(base, lighting, textured), texture, alpha_mode: AlphaMode::Opaque }
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
    match materials.get(material).filter(|_| opacity < 1.0).cloned() {
        Some(m) => materials.add(with_opacity(&m, opacity)),
        None => material.clone(),
    }
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
        "object materials: {} smooth, {} flat, {} none (incl. buildings until M1 5c, and missing ones)",
        kinds[2], kinds[1], kinds[0]
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
        let light = ObjectLight::new(lights, LightMode::X1);
        let s = light.settings(Vec4::ONE, Lighting::Smooth, true);
        // Game aim (0, 1, -0.66) -> Bevy (0, -0.66, -1), length kept.
        assert_eq!(s.sun_aim[0], Vec4::new(0.0, -0.66, -1.0, 0.0));
        assert_eq!((s.sun_count, s.lighting, s.mode, s.textured), (1, 2, 0, 1));
        // Facing up in Bevy (+Y) is facing up in the game: 0.366.
        assert!((light.corner_light(Vec3::Y)[0] - 0.366).abs() < 1e-5);
        // Bevy's -Z is the game's +Y: facing away from the sun's y part.
        let away = Vec3::new(0.0, -0.66, -1.0).normalize();
        assert!((light.corner_light(away)[0] - 0.3).abs() < 1e-5);
        assert!((light.corner_light(-away)[0] - (0.3 + 0.1 * (1.0f32 + 0.66 * 0.66).sqrt())).abs() < 1e-5);
    }
}
