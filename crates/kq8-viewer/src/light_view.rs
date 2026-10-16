//! Ground lighting in the ground view (M1 step 4c, docs/DESIGN.md; rules:
//! docs/behaviour/terrain-lighting.md, rule numbers are that note's).
//!
//! - The ground's texture is multiplied by the light map's colour, red by
//!   red, green by green, blue by blue (rule 10). The shown map is one
//!   picture on the graphics card (M1 7e-0): one texel per map point,
//!   looked up per pixel from the world position with bilinear filtering
//!   (rule 11; `object-lighting.md` rule 13, traced 2026-10-17), clamped at
//!   the edges. `ground_haze.wesl` does the lookup.
//! - The factor (rule 10, open question 1). x1 (default): channel / 15;
//!   x2: min(1, 2 x channel / 15); off: 1. x1 is the default from the
//!   user's check by eye (2026-10-14: the original is very dark and
//!   muddy, x1 matches better), against the note's suggested x2 and the
//!   data in Q65; a correction request is in Q66. Still a stand-in until
//!   session A confirms it.
//! - STAND-IN: the multiply is meant on the colour as shown on screen (a
//!   1998 card has no gamma step), so the factor is turned into Bevy's
//!   linear colour space (the sRGB curve) before it is stored. Products
//!   of sRGB values are not exactly sRGB of products: dark tones come out
//!   a little darker than a plain multiply.
//! - STAND-IN (open question 3): the brightness setting (`setBrightness`)
//!   is ignored.
//!
//! F7 cycles x1 -> x2 -> off (a setting of every ground material);
//! `--light x1|x2|off` sets the start. The mode is shown on screen and
//! logged; every second the light under Connor and under the camera goes
//! to `viewer.log`.

use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::image::{ImageAddressMode, ImageSampler, ImageSamplerDescriptor};
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use kq8_data::lightmap::{Channels, LightMap};

use crate::connor_view::ConnorBody;
use crate::{Shared, UNITS_PER_METRE};

/// How the light map's value becomes a brightness (rule 10).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LightMode {
    /// As stored: channel / 15 (default: the user's check).
    X1,
    /// Doubled, capped at 100% (the note's suggested stand-in).
    X2,
    /// No lighting (as before M1 step 4).
    Off,
}

impl LightMode {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_ascii_lowercase().as_str() {
            "x2" | "2" => Ok(LightMode::X2),
            "x1" | "1" => Ok(LightMode::X1),
            "off" | "0" => Ok(LightMode::Off),
            _ => Err(format!("--light needs x1, x2 or off, not '{s}'")),
        }
    }

    fn next(self) -> Self {
        match self {
            LightMode::X1 => LightMode::X2,
            LightMode::X2 => LightMode::Off,
            LightMode::Off => LightMode::X1,
        }
    }

    /// The ground shader's code for the mode.
    pub fn code(self) -> u32 {
        match self {
            LightMode::X1 => 0,
            LightMode::X2 => 1,
            LightMode::Off => 2,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            LightMode::X2 => "x2",
            LightMode::X1 => "x1",
            LightMode::Off => "off",
        }
    }

    /// Brightness shown for a channel value 0..15 (blended values may lie
    /// between whole numbers), 0..1 on screen.
    pub fn shown(self, channel: f32) -> f32 {
        match self {
            LightMode::X2 => (2.0 * channel / 15.0).min(1.0),
            LightMode::X1 => (channel / 15.0).min(1.0),
            LightMode::Off => 1.0,
        }
    }
}

/// The ground's light map, the mode, its picture on the graphics card,
/// and how many ground meshes are lit by it (for the log).
#[derive(Resource)]
pub struct GroundLight {
    pub map: LightMap,
    pub mode: LightMode,
    image: Option<Handle<Image>>,
    meshes: usize,
    corners: usize,
    last_report: Option<Instant>,
}

impl GroundLight {
    pub fn new(map: LightMap, mode: LightMode) -> Self {
        GroundLight { map, mode, image: None, meshes: 0, corners: 0, last_report: None }
    }

    /// The map's picture (made the first time it is asked for).
    pub fn image(&mut self, images: &mut Assets<Image>) -> Handle<Image> {
        if self.image.is_none() {
            self.image = Some(images.add(light_image(&self.map)));
        }
        self.image.clone().expect("just made")
    }

    /// Adds a ground mesh that the map lights (`game` = its corners in game
    /// units); the light itself comes from the picture in the shader.
    pub fn light_mesh(&mut self, mesh: Mesh, game: &[[f32; 3]], meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        self.meshes += 1;
        self.corners += game.len();
        meshes.add(mesh)
    }

    /// What the shader needs to find a world point in the picture: 1 ÷
    /// spacing (world units -> map points) and the side in points.
    pub fn lookup(&self) -> (f32, f32) {
        (1.0 / self.map.spacing, self.map.side as f32)
    }
}

/// The texel of a map value: red, green, blue = channel x 17, so the card
/// reads channel / 15 (rule 10); alpha 255. The brightness bits are not
/// used (rule 10).
pub fn texel(value: u16) -> [u8; 4] {
    let c = Channels::of(value);
    [c.red * 17, c.green * 17, c.blue * 17, 255]
}

/// The whole map as a picture: texel (i, j) = map point (i, j) (rule 7),
/// not sRGB (the values are brightness, not colours), bilinear, clamped
/// at the edges (outside: the nearest edge point).
pub fn light_image(map: &LightMap) -> Image {
    let data: Vec<u8> = map.values().iter().flat_map(|&v| texel(v)).collect();
    let side = map.side as u32;
    let mut image = Image::new(
        Extent3d { width: side, height: side, depth_or_array_layers: 1 },
        TextureDimension::D2,
        data,
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

/// Where the shader samples world point (x, y) (`ground_light` in
/// `ground_haze.wesl`, the same formula; here for the unit test): the texel centre of map
/// point (i, j) is at ((i + 0.5) ÷ side, (j + 0.5) ÷ side).
#[cfg(test)]
pub fn lookup_uv(x: f32, y: f32, inv_spacing: f32, side: f32) -> [f32; 2] {
    [(x * inv_spacing + 0.5) / side, (y * inv_spacing + 0.5) / side]
}

#[derive(Component)]
pub struct LightLabel;

fn label_text(mode: LightMode) -> String {
    let what = match mode {
        LightMode::X2 => "x2 (doubled, capped at 100%)",
        LightMode::X1 => "x1 (as the game)",
        LightMode::Off => "off (full bright)",
    };
    format!("Light, ground and objects: {what} (F7)")
}

pub fn setup_light_label(mut commands: Commands, light: Res<GroundLight>, shared: Res<Shared>) {
    commands.spawn((
        Text::new(label_text(light.mode)),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(Color::WHITE),
        Node { position_type: PositionType::Absolute, top: Val::Px(52.0), left: Val::Px(10.0), ..default() },
        LightLabel,
    ));
    shared.log.lock().unwrap().summary(format!(
        "ground light: mode {} at start ({} lit ground meshes, {} corners); light map {} x {} points as one bilinear picture (M1 7e-0); F7 cycles x1 -> x2 -> off",
        light.mode.name(),
        light.meshes,
        light.corners,
        light.map.side,
        light.map.side
    ));
}

/// F7: next mode, in every ground material.
pub fn toggle_light(
    keys: Res<ButtonInput<KeyCode>>,
    shared: Res<Shared>,
    mut light: ResMut<GroundLight>,
    mut materials: ResMut<Assets<crate::ground_haze::GroundMaterial>>,
    mut label: Query<&mut Text, With<LightLabel>>,
) {
    if !keys.just_pressed(KeyCode::F7) {
        return;
    }
    light.mode = light.mode.next();
    let mode = light.mode;
    let mut changed = 0;
    for (_, m) in materials.iter_mut() {
        m.settings.light_mode = mode.code();
        changed += 1;
    }
    for mut t in &mut label {
        t.0 = label_text(mode);
    }
    shared.log.lock().unwrap().summary(format!("key F7: ground light mode {} ({changed} ground materials)", mode.name()));
}

/// Every second: the light under Connor and under the camera.
pub fn report_light(
    shared: Res<Shared>,
    mut light: ResMut<GroundLight>,
    connor: Query<&Transform, With<ConnorBody>>,
    camera: Query<&Transform, With<Camera3d>>,
) {
    let now = Instant::now();
    if light.last_report.is_some_and(|t| now.duration_since(t).as_secs_f32() < 1.0) {
        return;
    }
    light.last_report = Some(now);
    let describe = |who: &str, t: &Transform| {
        let p = t.translation * UNITS_PER_METRE;
        let (x, y) = (p.x, -p.z);
        let c = light.map.colour_at(x, y);
        let shown = c.map(|v| light.mode.shown(v) * 100.0);
        format!(
            "{who} ({x:.0}, {y:.0}) raw r,g,b {:.2},{:.2},{:.2} shown {:.0}%,{:.0}%,{:.0}%",
            c[0], c[1], c[2], shown[0], shown[1], shown[2]
        )
    };
    let mut parts = Vec::new();
    if let Some(t) = connor.iter().next() {
        parts.push(describe("Connor", t));
    }
    if let Some(t) = camera.iter().next() {
        parts.push(describe("camera", t));
    }
    shared.log.lock().unwrap().detail(format!("ground light {}: {}", light.mode.name(), parts.join("; ")));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_picture_holds_the_map_and_the_lookup_hits_its_points() {
        // A 3 x 3 map, spacing 1,024: values grey 7 / 4 and one colour.
        let mut map = LightMap::with_size(3, 1024.0);
        map.set(1, 0, 0x7777);
        map.set(0, 1, 0x4444);
        map.set(1, 1, 0x0A50);
        let image = light_image(&map);
        let side = map.side;
        let at = |i: usize, j: usize| image.data.as_ref().unwrap()[4 * (j * side + i)..4 * (j * side + i) + 4].to_vec();
        assert_eq!(at(1, 0), vec![119, 119, 119, 255], "7 x 17");
        assert_eq!(at(0, 1), vec![68, 68, 68, 255]);
        assert_eq!(at(1, 1), vec![170, 85, 0, 255], "red 10, green 5, blue 0");
        assert_eq!(at(0, 0), vec![255, 255, 255, 255], "full 15");
        // Map point (i, j) sits at the centre of texel (i, j).
        let (inv, n) = (1.0 / map.spacing, side as f32);
        let uv = lookup_uv(map.spacing, 0.0, inv, n);
        assert!((uv[0] * n - 1.5).abs() < 1e-4 && (uv[1] * n - 0.5).abs() < 1e-4);
        // Half way between points: the texel edge, where bilinear gives the
        // average, as `colour_at` does.
        let uv = lookup_uv(0.5 * map.spacing, 0.0, inv, n);
        assert!((uv[0] * n - 1.0).abs() < 1e-4);
        assert_eq!(map.colour_at(0.5 * map.spacing, 0.0), [11.0, 11.0, 11.0]);
    }

    #[test]
    fn modes() {
        // Daventry: sunlit 7, shadow 4 (terrain-lighting.md rule 10).
        assert!((LightMode::X2.shown(7.0) - 14.0 / 15.0).abs() < 1e-6);
        assert!((LightMode::X1.shown(7.0) - 7.0 / 15.0).abs() < 1e-6);
        assert!((LightMode::X2.shown(4.0) - 8.0 / 15.0).abs() < 1e-6);
        // Values over 7 doubled are capped at 100% (edge cases).
        assert_eq!(LightMode::X2.shown(8.0), 1.0);
        assert_eq!(LightMode::X2.shown(15.0), 1.0);
        assert_eq!(LightMode::Off.shown(4.0), 1.0);
        assert_eq!(LightMode::parse("X1"), Ok(LightMode::X1));
        assert!(LightMode::parse("x3").is_err());
        assert_eq!(LightMode::X1.next(), LightMode::X2);
        assert_eq!(LightMode::X2.next().next().next(), LightMode::X2);
    }
}
