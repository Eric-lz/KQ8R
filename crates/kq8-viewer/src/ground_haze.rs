//! The ground's own material with its haze (M1 step 6e, docs/DESIGN.md;
//! rules: docs/behaviour/object-lighting.md rules 23, 27, 28, rule numbers
//! are that note's). See docs/provenance/README.md.
//!
//! Replaces Bevy's unlit standard material and distance fog on the ground
//! (surface, floor view, flowing squares): texture x the light map, looked
//! up per pixel in its picture with bilinear filtering (M1 7e-0,
//! `light_view`), then per corner haze from the camera,
//! 0.01 to 0.99 at `visibleDistance`, and nothing drawn beyond
//! `visibleDistance` (`ground_haze.wesl`). F8 switches it with the rest.

use bevy::asset::embedded_asset;
use bevy::image::ImageAddressMode;
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, ShaderType};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;

use crate::load::Look;
use crate::object_light::{texture_image, ObjectLight};
use crate::{Shared, UNITS_PER_METRE};

const SHADER: &str = "embedded://kq8_viewer/ground_haze.wesl";

/// The material's settings, as the shader reads them.
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Settings {
    /// Linear colour (white for textures); alpha = opacity.
    pub base_colour: Vec4,
    pub textured: u32,
    /// Game units per Bevy unit.
    pub game_units: f32,
    /// The light map's mode (F7): 0 x1, 1 x2, 2 off (terrain-lighting.md
    /// rule 10).
    pub light_mode: u32,
    /// 1 ÷ the map's point spacing (world units) and its side in points
    /// (rules 6, 7), to find a world point in the picture.
    pub light_inv_spacing: f32,
    pub light_side: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct GroundMaterial {
    #[uniform(0)]
    pub settings: Settings,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    /// The shown light map as one picture (`light_view::light_image`).
    #[texture(3)]
    #[sampler(4)]
    pub light_map: Handle<Image>,
    /// Shared haze parameters (`HazeBuffer`).
    #[storage(5, read_only)]
    pub haze_params: Handle<ShaderBuffer>,
    pub alpha_mode: AlphaMode,
}

impl Material for GroundMaterial {
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
    embedded_asset!(app, "ground_haze.wesl");
    app.add_plugins(MaterialPlugin::<GroundMaterial>::default());
}

/// A ground material for a texture, colour or missing material (magenta),
/// lit by the ground's light map.
pub fn ground_material(
    look: Look,
    address: ImageAddressMode,
    light: &ObjectLight,
    ground: &mut crate::light_view::GroundLight,
    images: &mut Assets<Image>,
) -> GroundMaterial {
    let light_map = ground.image(images);
    let (light_inv_spacing, light_side) = ground.lookup();
    let (base, texture) = match look {
        Look::Texture { width, height, rgba, .. } => (Vec4::ONE, Some(images.add(texture_image(width, height, rgba, address)))),
        Look::Color([r, g, b]) => (LinearRgba::from(Color::srgb_u8(r, g, b)).to_vec4(), None),
        Look::Missing(_) => (LinearRgba::from(Color::srgb(1.0, 0.0, 1.0)).to_vec4(), None),
    };
    GroundMaterial {
        settings: Settings {
            base_colour: base,
            textured: u32::from(texture.is_some()),
            game_units: UNITS_PER_METRE,
            light_mode: ground.mode.code(),
            light_inv_spacing,
            light_side,
        },
        texture,
        light_map,
        haze_params: light.haze_buffer.clone(),
        alpha_mode: AlphaMode::Opaque,
    }
}

/// A copy of `material` drawn at `opacity` (0..1), for the floor view.
pub fn with_opacity(material: &GroundMaterial, opacity: f32) -> GroundMaterial {
    let mut m = material.clone();
    m.settings.base_colour.w *= opacity;
    m.alpha_mode = AlphaMode::Blend;
    m
}

/// A copy of `material` multiplied by `tint` (the floor view's colours).
pub fn tinted(material: &GroundMaterial, tint: Color) -> GroundMaterial {
    let mut m = material.clone();
    m.settings.base_colour = LinearRgba::from(tint).to_vec4();
    m
}

/// Logs the ground's haze numbers at start (the note's rule 27 at a few
/// distances).
pub fn report_ground_haze(light: Res<ObjectLight>, materials: Res<Assets<GroundMaterial>>, shared: Res<Shared>) {
    let h = &light.haze;
    let at = |d: f32| format!("{d:.0}: {:.3}", h.ground_amount(d));
    shared.log.lock().unwrap().summary(format!(
        "ground haze (rule 27): starts at {:.0}, 0.99 at {:.0} (visibleDistance), nothing drawn beyond it; haze at {}, {}, {}, {}; {} ground materials",
        h.ground_start(),
        h.visible,
        at(h.ground_start()),
        at((h.ground_start() + h.visible) / 2.0),
        at(h.visible),
        at(h.visible + 1_000.0),
        materials.len()
    ));
}
