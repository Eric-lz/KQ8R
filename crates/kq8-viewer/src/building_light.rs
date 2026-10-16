//! Buildings lit by their `.dil` light maps (M1 step 5c, docs/DESIGN.md;
//! rules: docs/behaviour/object-lighting.md, rule numbers are that
//! note's). See docs/provenance/README.md.
//!
//! - Each wall's colour = its texture x its light map, channel by channel
//!   (channel / 15), no doubling (rules 13, 14). No sun or ambient on walls
//!   (rule 16, STAND-IN: the wall drawing code was not traced).
//! - Every surface's light map is packed into one light image per building
//!   ("atlas"); each wall corner gets a second set of picture coordinates
//!   into it, and the graphics card blends between light texels (bilinear,
//!   rule 13, the note's assumption).
//! - Light texel (i, j) sits at texture position (16 i - offset_x,
//!   16 j - offset_y), measured from the surface's own texture origin
//!   (rule 13).
//! - Texel 0,0 of the image is white (all channels 15): surfaces without a
//!   map, and buildings with no usable `.dil`, are drawn full bright
//!   (note: edge cases; STAND-IN for a surface with no map other than
//!   Castle D's material-255 ones, which are not drawn at all: Q67 c).
//! - `0x1000` building materials use the light map too (open question 7,
//!   STAND-IN).
//! - F7: x1 (as the note), x2 (doubled, capped), off (full bright).
//! - Haze per wall surface (M1 step 6f, rule 29): from the distance of
//!   the surface's nearest corner to the camera, with the objects' numbers
//!   (end = 0.85 x visibleDistance), 0.01 nearer than `hazeDistance`;
//!   at 0.99 or more the surface is left out, unless the building has
//!   `b_hazeSolid` (then drawn at 0.99). Plain screen-colour blend
//!   (rule 23). The building's surface corners are one list on the
//!   graphics card (`WallCorners`); each wall corner carries its surface's
//!   place in it (`ATTRIBUTE_WALL_SURFACE`). F8 switches it.
//! - Not done: point lights on walls and their haze x 0.04 (rule 16).

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::mesh::{MeshVertexAttribute, MeshVertexBufferLayoutRef};
use bevy::pbr::{Material, MaterialPipeline, MaterialPipelineKey, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{
    AsBindGroup, Extent3d, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError, TextureDimension, TextureFormat, VertexFormat,
};
use bevy::render::storage::ShaderBuffer;
use bevy::shader::ShaderRef;
use kq8_data::building::Surface;
use kq8_data::dil::BuildingLight;
use kq8_data::lightmap::Channels;

use crate::light_view::LightMode;
use crate::load::Look;
use crate::object_light::{texture_image, ObjectLight};
use crate::{Shared, UNITS_PER_METRE};

const SHADER: &str = "embedded://kq8_viewer/building_light.wesl";

/// One building's light maps packed into one picture.
#[derive(Debug, Clone)]
pub struct LightAtlas {
    pub width: u32,
    pub height: u32,
    /// Red, green, blue = channel x 17 (0..255, not sRGB), alpha 255.
    pub rgba: Vec<u8>,
    /// Where each surface's map starts in the picture (x, y); `None` for a
    /// surface with no map (it uses the white texel 0,0).
    pub places: Vec<Option<[u32; 2]>>,
    /// Texture texels per light texel (16).
    pub texels_per_light: f32,
}

impl LightAtlas {
    /// A picture of only the white texel: a building with no usable
    /// `.dil`, drawn full bright.
    pub fn white(surfaces: usize) -> Self {
        LightAtlas { width: 1, height: 1, rgba: vec![255; 4], places: vec![None; surfaces], texels_per_light: 16.0 }
    }

    /// Packs every surface's map (rows of maps, tallest first) after the
    /// white texel at 0,0.
    pub fn build(light: &BuildingLight) -> Self {
        let sizes: Vec<Option<[u32; 2]>> =
            light.surfaces.iter().map(|s| (s.has_map() && s.width > 0 && s.height > 0).then_some([s.width as u32, s.height as u32])).collect();
        let area: u32 = sizes.iter().flatten().map(|[w, h]| w * h).sum::<u32>() + 1;
        let widest = sizes.iter().flatten().map(|s| s[0]).max().unwrap_or(1);
        let width = (((area as f32).sqrt() * 1.15) as u32).max(widest).max(16).next_power_of_two();
        // Tallest first; the white texel is a 1 x 1 item placed first.
        let mut order: Vec<usize> = (0..sizes.len()).filter(|&i| sizes[i].is_some()).collect();
        order.sort_by_key(|&i| std::cmp::Reverse(sizes[i].unwrap()[1]));
        let mut places = vec![None; sizes.len()];
        let (mut x, mut y, mut row) = (1u32, 0u32, 1u32);
        for i in order {
            let [w, h] = sizes[i].unwrap();
            if x + w > width {
                (x, y, row) = (0, y + row, 0);
            }
            places[i] = Some([x, y]);
            x += w;
            row = row.max(h);
        }
        let height = y + row;
        let mut rgba = vec![0u8; (width * height * 4) as usize];
        rgba[..4].copy_from_slice(&[255; 4]);
        for (i, place) in places.iter().enumerate() {
            let (Some([px, py]), Some([w, h])) = (place, sizes[i]) else { continue };
            for j in 0..h {
                for k in 0..w {
                    let c = Channels::of(light.texel(i, k as usize, j as usize).unwrap_or(0));
                    let at = (((py + j) * width + px + k) * 4) as usize;
                    rgba[at..at + 4].copy_from_slice(&[c.red * 17, c.green * 17, c.blue * 17, 255]);
                }
            }
        }
        LightAtlas { width, height, rgba, places, texels_per_light: light.texels_per_light() as f32 }
    }

    /// Picture coordinates of the white texel's centre.
    pub fn white_uv(&self) -> [f32; 2] {
        [0.5 / self.width as f32, 0.5 / self.height as f32]
    }

    /// Picture coordinates for a corner of surface `index` at texel
    /// position `texel` (as `Building::surface_corners` gives it: the
    /// surface's texture offset included). Rule 13: light texel i sits at
    /// texture position 16 i - offset_x, the position measured without the
    /// texture offset. Clamped to the map, so a surface whose map is too
    /// small reads its edge.
    pub fn light_uv(&self, light: &BuildingLight, index: usize, dig: &Surface, texel: [f32; 2]) -> [f32; 2] {
        let (Some(Some([px, py])), Some(s)) = (self.places.get(index), light.surfaces.get(index)) else { return self.white_uv() };
        let size = [s.width as f32, s.height as f32];
        let place = [*px as f32, *py as f32];
        let picture = [self.width as f32, self.height as f32];
        std::array::from_fn(|k| {
            let p = texel[k] - dig.texture_offset[k] as f32;
            let i = ((p + s.map_offset[k] as f32) / self.texels_per_light).clamp(0.0, size[k] - 1.0);
            (place[k] + i + 0.5) / picture[k]
        })
    }

    /// The picture for the graphics card: plain values, blended
    /// bilinearly, clamped at the edges, no mipmaps.
    pub fn image(&self) -> Image {
        let mut image = Image::new(
            Extent3d { width: self.width, height: self.height, depth_or_array_layers: 1 },
            TextureDimension::D2,
            self.rgba.clone(),
            TextureFormat::Rgba8Unorm,
            RenderAssetUsages::default(),
        );
        image.sampler = ImageSampler::Descriptor(ImageSamplerDescriptor {
            address_mode_u: ImageAddressMode::ClampToEdge,
            address_mode_v: ImageAddressMode::ClampToEdge,
            ..ImageSamplerDescriptor::linear()
        });
        image
    }
}

/// Per wall corner: its surface's first corner and corner count in the
/// building's `WallCorners` list.
pub const ATTRIBUTE_WALL_SURFACE: MeshVertexAttribute = MeshVertexAttribute::new("Wall_Surface", 0x4B51_3806, VertexFormat::Uint32x2);

/// One building's surface corners for the graphics card: every surface's
/// corners one after another (Bevy model space, w = 1), and per surface
/// where its corners start and how many there are.
#[derive(Debug, Clone, Default)]
pub struct WallCorners {
    pub corners: Vec<[f32; 4]>,
    pub ranges: Vec<[u32; 2]>,
}

impl WallCorners {
    /// From the model's surfaces (game units, model space).
    pub fn new(surfaces: &[Vec<[f32; 3]>]) -> Self {
        let mut w = WallCorners::default();
        for s in surfaces {
            w.ranges.push([w.corners.len() as u32, s.len() as u32]);
            w.corners.extend(s.iter().map(|&p| {
                let [x, y, z] = crate::to_bevy(p, [0.0; 3]);
                [x, y, z, 1.0]
            }));
        }
        w
    }

    /// The list as a buffer for the card (never empty: a buffer of size 0
    /// can't be bound).
    pub fn buffer(&self) -> ShaderBuffer {
        let mut data: Vec<f32> = self.corners.iter().flatten().copied().collect();
        if data.is_empty() {
            data.extend([0.0; 4]);
        }
        ShaderBuffer::new(data, RenderAssetUsages::default())
    }

    /// `ATTRIBUTE_WALL_SURFACE` for a mesh of these triangles (each
    /// triangle's `node` is its surface).
    pub fn attribute(&self, triangles: &[kq8_data::shape::Triangle]) -> Vec<[u32; 2]> {
        triangles.iter().flat_map(|t| [self.ranges.get(t.node).copied().unwrap_or([0, 0]); 3]).collect()
    }
}

/// The material's settings, as the shader reads them.
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Settings {
    /// Linear colour (white for textures); alpha = opacity.
    pub base_colour: Vec4,
    /// Haze colour, screen colour 0..1 (rule 23); w unused.
    pub haze_colour: Vec4,
    /// 0 x1, 1 x2, 2 off.
    pub mode: u32,
    pub textured: u32,
    /// 1 = haze on (F8).
    pub haze_on: u32,
    /// Haze start and end in game units (rules 19, 29).
    pub haze_start: f32,
    pub haze_end: f32,
    /// The building's `b_hazeSolid` (rule 29).
    pub haze_solid: u32,
    /// Game units per Bevy unit.
    pub game_units: f32,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub struct BuildingMaterial {
    #[uniform(0)]
    pub settings: Settings,
    #[texture(1)]
    #[sampler(2)]
    pub texture: Option<Handle<Image>>,
    #[texture(3)]
    #[sampler(4)]
    pub light: Handle<Image>,
    /// The building's `WallCorners` list.
    #[storage(5, read_only)]
    pub corners: Handle<ShaderBuffer>,
    pub alpha_mode: AlphaMode,
}

impl Material for BuildingMaterial {
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

    /// The wall shader reads position, both picture coordinates and the
    /// surface's place in the corner list.
    fn specialize(
        _pipeline: &MaterialPipeline,
        descriptor: &mut RenderPipelineDescriptor,
        layout: &MeshVertexBufferLayoutRef,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        let vertex_layout = layout.0.get_layout(&[
            Mesh::ATTRIBUTE_POSITION.at_shader_location(0),
            Mesh::ATTRIBUTE_UV_0.at_shader_location(2),
            Mesh::ATTRIBUTE_UV_1.at_shader_location(3),
            ATTRIBUTE_WALL_SURFACE.at_shader_location(8),
        ])?;
        descriptor.vertex.buffers = vec![vertex_layout];
        Ok(())
    }
}

/// Registers the shader and the material.
pub fn add(app: &mut App) {
    embedded_asset!(app, "building_light.wesl");
    app.add_plugins(MaterialPlugin::<BuildingMaterial>::default());
}

fn mode_code(mode: LightMode) -> u32 {
    match mode {
        LightMode::X1 => 0,
        LightMode::X2 => 1,
        LightMode::Off => 2,
    }
}

/// A wall material: its texture or colour (magenta if missing), the
/// building's light image and its corner list.
pub fn building_material(
    look: Look,
    light_image: Handle<Image>,
    corners: Handle<ShaderBuffer>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
) -> BuildingMaterial {
    let (base, texture) = match look {
        Look::Texture { width, height, rgba, .. } => (Vec4::ONE, Some(images.add(texture_image(width, height, rgba, ImageAddressMode::Repeat)))),
        Look::Color([r, g, b]) => (LinearRgba::from(Color::srgb_u8(r, g, b)).to_vec4(), None),
        Look::Missing(_) => (LinearRgba::from(Color::srgb(1.0, 0.0, 1.0)).to_vec4(), None),
    };
    let textured = u32::from(texture.is_some());
    BuildingMaterial {
        settings: Settings {
            base_colour: base,
            haze_colour: Vec3::from(light.haze.colour).extend(0.0),
            mode: mode_code(light.mode),
            textured,
            haze_on: u32::from(light.haze_on),
            haze_start: light.haze.start,
            haze_end: light.haze.end,
            haze_solid: 0,
            game_units: UNITS_PER_METRE,
        },
        texture,
        light: light_image,
        corners,
        alpha_mode: AlphaMode::Opaque,
    }
}

/// `material` as it is, or a copy for one building drawn differently:
/// see-through (opacity below 1) or `b_hazeSolid` (rule 29).
pub fn for_building(materials: &mut Assets<BuildingMaterial>, material: &Handle<BuildingMaterial>, opacity: f32, haze_solid: bool) -> Handle<BuildingMaterial> {
    if opacity >= 1.0 && !haze_solid {
        return material.clone();
    }
    let Some(mut m) = materials.get(material).cloned() else { return material.clone() };
    if opacity < 1.0 {
        m.settings.base_colour.w *= opacity;
        m.alpha_mode = AlphaMode::Blend;
    }
    m.settings.haze_solid = u32::from(haze_solid);
    materials.add(m)
}

/// A wall surface at this haze or more is left out unless `b_hazeSolid`
/// (rule 29).
pub const WALL_MAX_HAZE: f32 = 0.99;

/// The walls' haze on the CPU (rule 29), for the log and tests: one
/// surface's haze from its nearest corner's distance to the camera (game
/// units); `None` if it is left out.
pub fn surface_haze(haze: &kq8_data::haze::Haze, corners: &[[f32; 3]], camera: [f32; 3], solid: bool) -> Option<f32> {
    let d = corners
        .iter()
        .map(|p| ((p[0] - camera[0]).powi(2) + (p[1] - camera[1]).powi(2) + (p[2] - camera[2]).powi(2)).sqrt())
        .fold(f32::MAX, f32::min);
    let h = haze.amount(d);
    match (h >= WALL_MAX_HAZE, solid) {
        (true, false) => None,
        (true, true) => Some(WALL_MAX_HAZE),
        (false, _) => Some(h),
    }
}

/// F7 / F8: the building materials follow the objects' light mode (which
/// follows the ground's) and haze switch.
pub fn follow_mode(
    light: Res<ObjectLight>,
    mut applied: Local<Option<(LightMode, bool)>>,
    mut materials: ResMut<Assets<BuildingMaterial>>,
    shared: Res<Shared>,
) {
    let now = (light.mode, light.haze_on);
    if applied.is_none() {
        *applied = Some(now);
    }
    if *applied == Some(now) {
        return;
    }
    *applied = Some(now);
    let (code, haze) = (mode_code(light.mode), u32::from(light.haze_on));
    let mut changed = 0;
    for (_, m) in materials.iter_mut() {
        m.settings.mode = code;
        m.settings.haze_on = haze;
        changed += 1;
    }
    shared.log.lock().unwrap().summary(format!(
        "building materials: light mode {}, haze {} ({changed} materials)",
        light.mode.name(),
        if light.haze_on { "on" } else { "off" }
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A `.dil` (as bytes) with the given maps: (width, height, offset x,
    /// offset y); texel values count up from 1.
    fn dil(maps: &[(u8, u8, u8, u8)]) -> BuildingLight {
        let total: usize = maps.iter().map(|m| m.0 as usize * m.1 as usize).sum();
        let mut d = Vec::new();
        for v in [5u32, 4, 0, 0, 0, 0, maps.len() as u32, 2 * total as u32] {
            d.extend(v.to_le_bytes());
        }
        let mut offset = 0u32;
        for &(w, h, ox, oy) in maps {
            let mut rec = [0u8; 48];
            rec[0..4].copy_from_slice(&offset.to_le_bytes());
            rec[8..12].copy_from_slice(&[w, h, ox, oy]);
            d.extend(rec);
            offset += 2 * w as u32 * h as u32;
        }
        for k in 0..total as u16 {
            d.extend((0x0100 * ((k + 1) % 16)).to_le_bytes());
        }
        d.extend(0u32.to_le_bytes());
        let mut body = (11i16).to_le_bytes().to_vec();
        body.extend(b"ITRLighting\0");
        body.extend(4i32.to_le_bytes());
        body.extend(d);
        let mut out = b"PERS".to_vec();
        out.extend((body.len() as u32).to_le_bytes());
        out.extend(body);
        BuildingLight::parse(&out).unwrap()
    }

    fn surface(size: [u8; 2], offset: [u8; 2]) -> Surface {
        Surface { flags: 0, material: 0, texture_size: size, texture_offset: offset, plane: 0, first_corner: 0, corner_count: 3, light_first: 0, light_count: 0 }
    }

    #[test]
    fn packing_keeps_maps_apart() {
        let light = dil(&[(3, 2, 0, 0), (18, 17, 1, 1), (2, 5, 0, 0), (17, 3, 0, 0)]);
        let a = LightAtlas::build(&light);
        assert_eq!(&a.rgba[..4], &[255; 4]);
        let rects: Vec<[u32; 4]> = a
            .places
            .iter()
            .zip(&light.surfaces)
            .map(|(p, s)| {
                let [x, y] = p.unwrap();
                [x, y, x + s.width as u32, y + s.height as u32]
            })
            .collect();
        for (i, r) in rects.iter().enumerate() {
            assert!(r[2] <= a.width && r[3] <= a.height, "map {i} outside: {r:?} in {}x{}", a.width, a.height);
            // Not over the white texel.
            assert!(!(r[0] == 0 && r[1] == 0), "map {i} covers 0,0");
            for (k, o) in rects.iter().enumerate().skip(i + 1) {
                let apart = r[2] <= o[0] || o[2] <= r[0] || r[3] <= o[1] || o[3] <= r[1];
                assert!(apart, "maps {i} and {k} overlap");
            }
        }
        // Surface 0's texel (2, 1) = value 6 -> red 6 x 17.
        let [x, y] = a.places[0].unwrap();
        let at = (((y + 1) * a.width + x + 2) * 4) as usize;
        assert_eq!(a.rgba[at], 6 * 17);
    }

    #[test]
    fn wall_haze() {
        // Daventry: start 18,000, end 22,950; the nearest corner counts.
        let h = kq8_data::haze::Haze::new(18_000.0, 27_000.0, [0.0; 3]);
        let wall = [[20_475.0, 0.0, 0.0], [30_000.0, 0.0, 0.0]];
        assert!((surface_haze(&h, &wall, [0.0; 3], false).unwrap() - 0.5).abs() < 1e-5);
        assert_eq!(surface_haze(&h, &[[100.0, 0.0, 0.0]], [0.0; 3], false), Some(0.01));
        // 0.99 or more: left out, unless solid (then 0.99).
        let far = [[18_000.0 + 0.99 * 4_950.0 + 1.0, 0.0, 0.0]];
        assert_eq!(surface_haze(&h, &far, [0.0; 3], false), None);
        assert_eq!(surface_haze(&h, &far, [0.0; 3], true), Some(0.99));
        assert!(surface_haze(&h, &[[22_850.0, 0.0, 0.0]], [0.0; 3], false).is_some());
        // The corner list: ranges in order, Bevy space.
        let w = WallCorners::new(&[vec![[100.0, 200.0, 300.0]; 3], vec![[0.0; 3]; 4]]);
        assert_eq!(w.ranges, vec![[0, 3], [3, 4]]);
        let b = crate::to_bevy([100.0, 200.0, 300.0], [0.0; 3]);
        assert_eq!(w.corners[0], [b[0], b[1], b[2], 1.0]);
    }

    #[test]
    fn corner_coordinates() {
        // One map 3 x 2, offset 5, 0; texture offset 2, 1 on the surface.
        let light = dil(&[(3, 2, 5, 0)]);
        let a = LightAtlas::build(&light);
        let [px, py] = a.places[0].unwrap();
        let (w, h) = (a.width as f32, a.height as f32);
        let s = surface([26, 15], [2, 1]);
        // Texture position 11 (texel 13 with offset 2) = 16 x 1 - 5: light
        // texel 1 exactly, i.e. its centre.
        let uv = a.light_uv(&light, 0, &s, [13.0, 1.0]);
        assert!((uv[0] - (px as f32 + 1.5) / w).abs() < 1e-6);
        assert!((uv[1] - (py as f32 + 0.5) / h).abs() < 1e-6);
        // The surface's far edge (position 27, 16) stays inside the map:
        // (27 + 5) / 16 = 2 = the last column; 16 / 16 = 1 = the last row.
        let far = a.light_uv(&light, 0, &s, [29.0, 17.0]);
        assert!((far[0] - (px as f32 + 2.5) / w).abs() < 1e-6);
        assert!((far[1] - (py as f32 + 1.5) / h).abs() < 1e-6);
        // Beyond the map: clamped to its edge.
        let past = a.light_uv(&light, 0, &s, [200.0, 200.0]);
        assert_eq!(past, far);
        // No such surface: the white texel.
        assert_eq!(a.light_uv(&light, 9, &s, [0.0, 0.0]), a.white_uv());
    }
}
