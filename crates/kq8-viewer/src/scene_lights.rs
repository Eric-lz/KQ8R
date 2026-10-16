//! Point lights on models (M1 steps 7c-2 and 7c-3, docs/DESIGN.md; rules:
//! docs/behaviour/object-lighting.md rules 3, 24, 25, 26b, 37, rule
//! numbers are that note's). See docs/provenance/README.md.
//!
//! - Once per frame all lights of the scene are gathered into one list
//!   (rule 3): every `KQPointLight` the world's scripts loaded, and every
//!   light carried by a shown model in its current frame (rule 26b), each
//!   only while its owner is in the lights group, group bit 0x400 (rule
//!   37: near or far with the default masks; off and hidden objects are
//!   not). A deleted owner gives no light.
//! - A carried point light's position is the node's position put through
//!   the object's placement only (its place, turn and scale when it was
//!   placed), never the frame's own step; a sun-type node's aim is used as
//!   stored, in world axes (rule 26b).
//! - The list goes to the graphics card in one storage buffer that every
//!   object material shares; `object_light.wesl` adds each light per model
//!   part and corner (rule 24). The buffer keeps its size, so only its
//!   contents are rewritten.
//! - These lights reach models only; walls (rule 16) and the ground (rule
//!   26) are M1 7d / 7e.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use bevy::asset::RenderAssetUsages;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use kq8_data::lights::PointLight;
use kq8_data::shape::ModelLight;

use crate::draw_distance::DistanceTest;
use crate::{Shared, UNITS_PER_METRE};

/// Lights the card takes per frame; more are left out and logged.
pub const MAX_SCENE_LIGHTS: usize = 128;

/// The group bit that puts an object in the lights group (rule 37).
pub const LIGHTS_BIT: u32 = 0x400;

/// The lights a model carries, per movement and frame (model space), and
/// the frame shown now. On the object's root entity, which also has its
/// `DistanceTest`.
#[derive(Component, Clone)]
pub struct CarriedLights {
    pub movements: Vec<Arc<Vec<Vec<ModelLight>>>>,
    pub movement: usize,
    pub frame: usize,
    /// The object's placement as it was placed (rule 26b: placement only).
    pub placement: Transform,
}

impl CarriedLights {
    /// `None` if no frame of any movement carries a light.
    pub fn new(movements: Vec<Arc<Vec<Vec<ModelLight>>>>, placement: Transform) -> Option<Self> {
        movements
            .iter()
            .any(|m| m.iter().any(|f| !f.is_empty()))
            .then_some(CarriedLights { movements, movement: 0, frame: 0, placement })
    }

    fn current(&self) -> &[ModelLight] {
        self.movements.get(self.movement).and_then(|m| m.get(self.frame).or(m.last())).map_or(&[], Vec::as_slice)
    }
}

/// A `KQPointLight` object (rules 24, 25), on an entity of its own at the
/// light's position, with a `DistanceTest`.
#[derive(Component, Clone)]
pub struct PointLightSource {
    pub light: PointLight,
}

/// Which objects are in the lights group this frame (by `DistanceTest`
/// number); filled by `draw_distance::update_activity`.
#[derive(Resource, Default)]
pub struct LightsGroup(pub HashSet<u32>);

/// The shared light list on the card, and what was gathered (for the log).
#[derive(Resource)]
pub struct SceneLights {
    pub buffer: Handle<ShaderBuffer>,
    last_log: Option<Instant>,
}

/// One light as the shader reads it (three `vec4`s): position (Bevy world
/// space; w = 1 point, 0 sun-type: xyz is then its aim, Bevy axes, as
/// stored), colour (w unused), Begin and End in game units.
fn entry(position: Vec3, point: bool, colour: [f32; 3], begin: f32, end: f32) -> [[f32; 4]; 3] {
    [position.extend(if point { 1.0 } else { 0.0 }).to_array(), [colour[0], colour[1], colour[2], 0.0], [begin, end, 0.0, 0.0]]
}

/// The buffer's contents: a header `vec4` (x = how many lights), then
/// `MAX_SCENE_LIGHTS` entries (unused ones zero). Always the same size.
pub fn buffer_data(lights: &[[[f32; 4]; 3]]) -> Vec<[f32; 4]> {
    let n = lights.len().min(MAX_SCENE_LIGHTS);
    let mut data = Vec::with_capacity(1 + 3 * MAX_SCENE_LIGHTS);
    data.push([n as f32, 0.0, 0.0, 0.0]);
    data.extend(lights.iter().take(n).flatten().copied());
    data.resize(1 + 3 * MAX_SCENE_LIGHTS, [0.0; 4]);
    data
}

/// The empty list, for the materials made before the first frame.
pub fn empty_buffer() -> ShaderBuffer {
    ShaderBuffer::new(buffer_data(&[]), RenderAssetUsages::default())
}

impl SceneLights {
    pub fn new(buffer: Handle<ShaderBuffer>) -> Self {
        SceneLights { buffer, last_log: None }
    }
}

/// Rule 24 on models: 1 up to Begin, straight down to 0 at End (End is an
/// end distance here). The shader's `model_falloff`, for the log and tests.
pub fn model_falloff(d: f32, begin: f32, end: f32) -> f32 {
    if d <= begin {
        1.0
    } else if d >= end {
        0.0
    } else {
        1.0 - (d - begin) / (end - begin).max(0.001)
    }
}

/// Game point (Z up) -> Bevy (Y up), in Bevy units.
fn point_to_bevy(p: [f32; 3]) -> Vec3 {
    Vec3::new(p[0], p[2], -p[1]) / UNITS_PER_METRE
}

/// Every frame, after the near / far / off test: gathers the lights
/// (rule 3) into the shared buffer; logs once a second.
#[allow(clippy::too_many_arguments)]
pub fn gather_lights(
    mut scene: ResMut<SceneLights>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    group: Res<LightsGroup>,
    points: Query<(&DistanceTest, &PointLightSource)>,
    carried: Query<(&DistanceTest, &CarriedLights)>,
    connor: Query<&Transform, With<crate::connor_view::ConnorBody>>,
    sim: Option<Res<crate::door_view::DoorSim>>,
    shared: Res<Shared>,
) {
    let deleted = |name: &str| sim.as_ref().is_some_and(|s| s.active.world.is_deleted(name));
    let mut lights = Vec::new();
    // Position, name, colour, Begin, End of each point light, for the log.
    let mut described: Vec<(Vec3, String, [f32; 3], f32, f32)> = Vec::new();
    let (mut from_points, mut from_models, mut sun_type) = (0, 0, 0);
    for (test, source) in &points {
        if !group.0.contains(&test.object) || deleted(&test.name) {
            continue;
        }
        let l = &source.light;
        let at = point_to_bevy(l.position);
        lights.push(entry(at, true, l.colour, l.begin, l.end));
        described.push((at, test.name.clone(), l.colour, l.begin, l.end));
        from_points += 1;
    }
    for (test, c) in &carried {
        if !group.0.contains(&test.object) || deleted(&test.name) {
            continue;
        }
        for l in c.current() {
            if l.point {
                // Rule 26b: the object's placement only.
                let at = c.placement.transform_point(point_to_bevy(l.position));
                lights.push(entry(at, true, l.colour, l.begin, l.end));
                described.push((at, test.name.clone(), l.colour, l.begin, l.end));
            } else {
                // A sun-type node: its aim as stored, world axes (rule 26b).
                let aim = Vec3::new(l.position[0], l.position[2], -l.position[1]);
                lights.push(entry(aim, false, l.colour, l.begin, l.end));
                sun_type += 1;
            }
            from_models += 1;
        }
    }
    let total = lights.len();
    if let Some(mut b) = buffers.get_mut(&scene.buffer) {
        *b = ShaderBuffer::new(buffer_data(&lights), RenderAssetUsages::default());
    }
    let now = Instant::now();
    if scene.last_log.is_some_and(|t| now.duration_since(t).as_secs_f32() < 1.0) {
        return;
    }
    scene.last_log = Some(now);
    // What the nearest point light gives Connor at his feet, facing it
    // squarely (rule 24's falloff x colour; facing 1).
    let nearest = connor.iter().next().and_then(|c| {
        described
            .iter()
            .map(|(at, name, colour, begin, end)| ((*at - c.translation).length() * UNITS_PER_METRE, name, colour, begin, end))
            .min_by(|a, b| a.0.total_cmp(&b.0))
            .map(|(d, name, colour, begin, end)| {
                let f = model_falloff(d, *begin, *end);
                format!(
                    "[{name}] {d:.0} from Connor (Begin {begin:.0}, End {end:.0}): falloff {f:.3}, adds up to ({:.3}, {:.3}, {:.3}) to his light",
                    colour[0] * f,
                    colour[1] * f,
                    colour[2] * f
                )
            })
    });
    shared.log.lock().unwrap().detail(format!(
        "scene lights: {total} ({from_points} KQPointLight, {from_models} carried by models, {sun_type} of them sun-type){}; nearest {}",
        if total > MAX_SCENE_LIGHTS { format!(", only the first {MAX_SCENE_LIGHTS} drawn") } else { String::new() },
        nearest.unwrap_or_else(|| "none".into())
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn falloff_on_models() {
        // PLIGHT01: Begin 1,000, End 4,000 (an end distance on models).
        assert_eq!(model_falloff(500.0, 1000.0, 4000.0), 1.0);
        assert!((model_falloff(2500.0, 1000.0, 4000.0) - 0.5).abs() < 1e-6);
        assert_eq!(model_falloff(4000.0, 1000.0, 4000.0), 0.0);
        assert_eq!(model_falloff(4500.0, 1000.0, 4000.0), 0.0);
    }

    #[test]
    fn the_buffer_keeps_its_size() {
        let one = entry(Vec3::new(1.0, 2.0, 3.0), true, [0.3, 0.7, 0.4], 1000.0, 4000.0);
        let a = buffer_data(&[]);
        let b = buffer_data(&[one]);
        let many = buffer_data(&vec![one; MAX_SCENE_LIGHTS + 5]);
        assert_eq!((a.len(), b.len(), many.len()), (1 + 3 * MAX_SCENE_LIGHTS, 1 + 3 * MAX_SCENE_LIGHTS, 1 + 3 * MAX_SCENE_LIGHTS));
        assert_eq!((a[0][0], b[0][0], many[0][0]), (0.0, 1.0, MAX_SCENE_LIGHTS as f32));
        assert_eq!(b[1], [1.0, 2.0, 3.0, 1.0]);
        assert_eq!(b[2], [0.3, 0.7, 0.4, 0.0]);
        assert_eq!(b[3], [1000.0, 4000.0, 0.0, 0.0]);
    }
}
