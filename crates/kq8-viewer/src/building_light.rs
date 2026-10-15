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

use bevy::asset::{embedded_asset, RenderAssetUsages};
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::pbr::{Material, MaterialPlugin};
use bevy::prelude::*;
use bevy::render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat};
use bevy::shader::ShaderRef;
use kq8_data::building::Surface;
use kq8_data::dil::BuildingLight;
use kq8_data::lightmap::Channels;

use crate::light_view::LightMode;
use crate::load::Look;
use crate::object_light::{texture_image, ObjectLight};
use crate::Shared;

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

/// The material's settings, as the shader reads them.
#[derive(Debug, Clone, Copy, ShaderType)]
pub struct Settings {
    /// Linear colour (white for textures); alpha = opacity.
    pub base_colour: Vec4,
    /// 0 x1, 1 x2, 2 off.
    pub mode: u32,
    pub textured: u32,
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

/// A wall material: its texture or colour (magenta if missing) and the
/// building's light image.
pub fn building_material(look: Look, light_image: Handle<Image>, mode: LightMode, images: &mut Assets<Image>) -> BuildingMaterial {
    let (base, texture) = match look {
        Look::Texture { width, height, rgba, .. } => (Vec4::ONE, Some(images.add(texture_image(width, height, rgba, ImageAddressMode::Repeat)))),
        Look::Color([r, g, b]) => (LinearRgba::from(Color::srgb_u8(r, g, b)).to_vec4(), None),
        Look::Missing(_) => (LinearRgba::from(Color::srgb(1.0, 0.0, 1.0)).to_vec4(), None),
    };
    let textured = u32::from(texture.is_some());
    BuildingMaterial {
        settings: Settings { base_colour: base, mode: mode_code(mode), textured },
        texture,
        light: light_image,
        alpha_mode: AlphaMode::Opaque,
    }
}

/// `material` as it is (opacity 1), or a see-through copy of it.
pub fn see_through(materials: &mut Assets<BuildingMaterial>, material: &Handle<BuildingMaterial>, opacity: f32) -> Handle<BuildingMaterial> {
    match materials.get(material).filter(|_| opacity < 1.0).cloned() {
        Some(mut m) => {
            m.settings.base_colour.w *= opacity;
            m.alpha_mode = AlphaMode::Blend;
            materials.add(m)
        }
        None => material.clone(),
    }
}

/// F7: the building materials follow the objects' mode (which follows the
/// ground's).
pub fn follow_mode(
    light: Res<ObjectLight>,
    mut applied: Local<Option<LightMode>>,
    mut materials: ResMut<Assets<BuildingMaterial>>,
    shared: Res<Shared>,
) {
    if applied.is_none() {
        *applied = Some(light.mode);
    }
    if *applied == Some(light.mode) {
        return;
    }
    *applied = Some(light.mode);
    let code = mode_code(light.mode);
    let mut changed = 0;
    for (_, m) in materials.iter_mut() {
        m.settings.mode = code;
        changed += 1;
    }
    shared.log.lock().unwrap().summary(format!("building light mode {} ({changed} materials)", light.mode.name()));
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
