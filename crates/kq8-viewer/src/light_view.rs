//! Ground lighting in the ground view (M1 step 4c, docs/DESIGN.md; rules:
//! docs/behaviour/terrain-lighting.md, rule numbers are that note's).
//!
//! - The ground's texture is multiplied by the light map's colour, red by
//!   red, green by green, blue by blue (rule 10). The map's points are
//!   mesh corners (the ground is cut into the map's cells), so each corner
//!   carries its point's colour as a vertex colour, which Bevy multiplies
//!   with the texture.
//! - The factor (rule 10, open question 1). x1 (default): channel / 15;
//!   x2: min(1, 2 x channel / 15); off: 1. x1 is the default from the
//!   user's check by eye (2026-10-14: the original is very dark and
//!   muddy, x1 matches better), against the note's suggested x2 and the
//!   data in Q65; a correction request is in Q66. Still a stand-in until
//!   session A confirms it.
//! - STAND-IN (rule 11, open question 2): between corners the colour is
//!   blended across each cell's two triangles, close to the note's
//!   assumed bilinear filtering but not the same.
//! - STAND-IN: the multiply is meant on the colour as shown on screen (a
//!   1998 card has no gamma step), so the factor is turned into Bevy's
//!   linear colour space (the sRGB curve) before it is stored. Products
//!   of sRGB values are not exactly sRGB of products: dark tones come out
//!   a little darker than a plain multiply.
//! - STAND-IN (open question 3): the brightness setting (`setBrightness`)
//!   is ignored.
//!
//! F7 cycles x1 -> x2 -> off; `--light x1|x2|off` sets the start. The mode
//! is shown on screen and logged; every second the light under Connor
//! and under the camera goes to `viewer.log`.

use std::time::Instant;

use bevy::color::Srgba;
use bevy::prelude::*;
use kq8_data::lightmap::LightMap;

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

/// The ground's light map, the mode, and every lit ground mesh with its
/// corners' raw map colours (to recolour on F7).
#[derive(Resource)]
pub struct GroundLight {
    pub map: LightMap,
    pub mode: LightMode,
    meshes: Vec<(Handle<Mesh>, Vec<[f32; 3]>)>,
    last_report: Option<Instant>,
}

impl GroundLight {
    pub fn new(map: LightMap, mode: LightMode) -> Self {
        GroundLight { map, mode, meshes: Vec::new(), last_report: None }
    }

    /// Adds the light map's colour at each corner (`game` = the corners in
    /// game units, in the mesh's order) to `mesh`, and keeps it for F7.
    pub fn light_mesh(&mut self, mut mesh: Mesh, game: &[[f32; 3]], meshes: &mut Assets<Mesh>) -> Handle<Mesh> {
        let raw: Vec<[f32; 3]> = game.iter().map(|p| self.map.colour_at(p[0], p[1])).collect();
        mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colours(&raw, self.mode));
        let handle = meshes.add(mesh);
        self.meshes.push((handle.clone(), raw));
        handle
    }
}

/// Vertex colours for raw map colours: the shown brightness per channel,
/// turned into Bevy's linear colour space (module comment, stand-in).
fn vertex_colours(raw: &[[f32; 3]], mode: LightMode) -> Vec<[f32; 4]> {
    raw.iter()
        .map(|c| {
            let [r, g, b] = c.map(|v| Srgba::gamma_function(mode.shown(v)));
            [r, g, b, 1.0]
        })
        .collect()
}

#[derive(Component)]
pub struct LightLabel;

fn label_text(mode: LightMode) -> String {
    let what = match mode {
        LightMode::X2 => "x2 (light map doubled, capped at 100%)",
        LightMode::X1 => "x1 (light map as stored)",
        LightMode::Off => "off (no light map)",
    };
    format!("Ground light: {what} (F7)")
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
        "ground light: mode {} at start ({} lit ground meshes, {} corners); F7 cycles x1 -> x2 -> off",
        light.mode.name(),
        light.meshes.len(),
        light.meshes.iter().map(|(_, raw)| raw.len()).sum::<usize>()
    ));
}

/// F7: next mode; every lit ground mesh gets its new colours.
pub fn toggle_light(
    keys: Res<ButtonInput<KeyCode>>,
    shared: Res<Shared>,
    mut light: ResMut<GroundLight>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut label: Query<&mut Text, With<LightLabel>>,
) {
    if !keys.just_pressed(KeyCode::F7) {
        return;
    }
    light.mode = light.mode.next();
    let mode = light.mode;
    let mut changed = 0;
    for (handle, raw) in &light.meshes {
        if let Some(mut mesh) = meshes.get_mut(handle) {
            mesh.insert_attribute(Mesh::ATTRIBUTE_COLOR, vertex_colours(raw, mode));
            changed += 1;
        }
    }
    for mut t in &mut label {
        t.0 = label_text(mode);
    }
    shared.log.lock().unwrap().summary(format!("key F7: ground light mode {} ({changed} meshes recoloured)", mode.name()));
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
