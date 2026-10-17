//! Point lights on building surfaces (M1 step 7d, docs/DESIGN.md; rules:
//! docs/behaviour/object-lighting.md rule 16, rewritten 2026-10-16 for
//! Q78; the tests themselves are `kq8_data::wall_light`). See
//! docs/provenance/README.md.
//!
//! - Every surface of a building counts: walls, floors, ceilings, roofs.
//! - Which lights: the same list as the ground (`scene_lights::GroundLights`:
//!   every `KQPointLight` and every point light carried by a model in the
//!   lights group, except those of an object with `turnOffDynLighting=1`);
//!   only with dynamic lighting on.
//! - Which surfaces: every surface of every building copy drawn this frame
//!   (not off by distance, not hidden or unused). The original lights only
//!   the surfaces in view; one out of view can't be seen, so this gives the
//!   same picture (our simplification).
//! - Each frame the surfaces lit last frame get their `.dil` texels back,
//!   then each light is tested per surface: in front, reach, then a line
//!   against **this building's own BSP** (its collider); one that passes
//!   paints the surface's texels and multiplies its haze by 0.04.
//! - A quick skip (our method): a light farther than Begin + End + 256
//!   from the building's `.dig` box can't reach any of its surfaces (every
//!   light-map corner lies within a light texel of the box), so its
//!   surfaces are counted as "skipped" without testing.
//! - Each building copy has its own light picture and haze-factor list.
//! - Logs once a second, per light: surfaces tested, skipped, failed "in
//!   front", failed reach, blocked by the line test, lit, and the largest
//!   change of one texel channel.

use std::collections::{BTreeMap, HashMap};
use std::sync::Arc;
use std::time::Instant;

use bevy::app::Propagate;
use bevy::camera::visibility::RenderLayers;
use bevy::prelude::*;
use bevy::render::storage::ShaderBuffer;
use kq8_data::building::Building;
use kq8_data::dil::BuildingLight;
use kq8_data::wall_light::{facing, paint_surface, test_surface, Test, HAZE_FACTOR};

use crate::building_light::{factor_buffer, LightAtlas};
use crate::Shared;

/// Added to Begin + End for the quick skip (a light texel and more).
const SKIP_MARGIN: f32 = 256.0;

/// What a building's surfaces need for point lights: its `.dil` (stored
/// texels and light-map rectangles), each surface's front direction, its
/// `.dig` box.
pub struct WallData {
    pub light: BuildingLight,
    /// Rule 16: the plane normal if bit 0x80, else reversed (building space).
    pub facing: Vec<[f32; 3]>,
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Surfaces that can be lit (a map and a rectangle).
    pub lightable: Vec<usize>,
}

impl WallData {
    /// Also returns how many surfaces' bit-0x80 facing agrees with the
    /// corner order's front (for the log; expected all).
    pub fn new(light: BuildingLight, b: &Building) -> (Self, usize) {
        let mut agree = 0;
        let facing: Vec<[f32; 3]> = b
            .surfaces
            .iter()
            .enumerate()
            .map(|(i, s)| {
                let n = b.planes.get(s.plane as usize).map_or([0.0; 3], |p| p.normal);
                let f = facing(n, s.flipped());
                // The corner order's normal points away from the front.
                if b.corner_order_normal(i).is_some_and(|c| c[0] * f[0] + c[1] * f[1] + c[2] * f[2] < 0.0) {
                    agree += 1;
                }
                f
            })
            .collect();
        let lightable = (0..light.surfaces.len().min(facing.len()))
            .filter(|&i| light.surfaces[i].has_map() && light.surfaces[i].rect.is_some() && light.surfaces[i].width > 0 && light.surfaces[i].height > 0)
            .collect();
        (WallData { light, facing, min: b.bounds_min, max: b.bounds_max, lightable }, agree)
    }

    /// Surface `s`'s texels as stored, row by row.
    fn stored(&self, s: usize) -> Vec<u16> {
        let l = &self.light.surfaces[s];
        let (w, h) = (l.width as usize, l.height as usize);
        (0..h).flat_map(|j| (0..w).map(move |i| (i, j))).map(|(i, j)| self.light.texel(s, i, j).unwrap_or(0)).collect()
    }
}

/// One building copy's point-light state, on its first wall entity.
#[derive(Component)]
pub struct WallLights {
    pub name: String,
    pub data: Arc<WallData>,
    pub atlas: Arc<LightAtlas>,
    /// This copy's own light picture and haze factors.
    pub image: Handle<Image>,
    pub factors: Handle<ShaderBuffer>,
    /// Its collider in `terrain_view::Buildings` (the BSP for the line
    /// test, and its placement); `None`: no lights (logged at spawn).
    pub collider: Option<usize>,
    /// Surfaces lit last frame (put back first).
    lit: Vec<usize>,
}

impl WallLights {
    pub fn new(name: String, data: Arc<WallData>, atlas: Arc<LightAtlas>, image: Handle<Image>, factors: Handle<ShaderBuffer>, collider: Option<usize>) -> Self {
        WallLights { name, data, atlas, image, factors, collider, lit: Vec::new() }
    }
}

/// One light's counts over all buildings this frame (for the log).
#[derive(Default, Clone)]
struct Counts {
    tested: usize,
    skipped: usize,
    behind: usize,
    out_of_reach: usize,
    blocked: usize,
    lit: usize,
    largest: u8,
    /// Lit surfaces per building.
    buildings: BTreeMap<String, usize>,
}

/// Writes surface `s`'s texels into the building's light picture.
fn write_texels(data: &mut [u8], atlas: &LightAtlas, wall: &WallData, s: usize, texels: &[u16]) {
    let (Some(Some([px, py])), Some(l)) = (atlas.places.get(s), wall.light.surfaces.get(s)) else { return };
    let w = l.width as usize;
    for (k, &v) in texels.iter().enumerate() {
        let (i, j) = (k % w, k / w);
        let at = (((*py as usize + j) * atlas.width as usize) + *px as usize + i) * 4;
        if let Some(t) = data.get_mut(at..at + 4) {
            t.copy_from_slice(&crate::light_view::texel(v));
        }
    }
}

/// Distance from `p` to the box `lo..hi` (0 inside).
fn box_distance(p: [f32; 3], lo: [f32; 3], hi: [f32; 3]) -> f32 {
    (0..3).map(|k| (lo[k] - p[k]).max(0.0).max(p[k] - hi[k]).powi(2)).sum::<f32>().sqrt()
}

/// Every frame, after the lights are gathered: rule 16 on every drawn
/// building copy. Logs once a second.
#[allow(clippy::too_many_arguments)]
#[allow(clippy::type_complexity)]
pub fn light_walls(
    mut walls: Query<(&mut WallLights, Option<&Visibility>, Has<Propagate<RenderLayers>>)>,
    lights: Res<crate::scene_lights::GroundLights>,
    ground: Option<Res<crate::light_view::GroundLight>>,
    buildings: Option<Res<crate::terrain_view::Buildings>>,
    mut images: ResMut<Assets<Image>>,
    mut buffers: ResMut<Assets<ShaderBuffer>>,
    mut last_log: Local<Option<Instant>>,
    shared: Res<Shared>,
) {
    let dynamic = ground.as_ref().is_some_and(|g| g.dynamic);
    let mut counts = vec![Counts::default(); lights.0.len()];
    let (mut drawn_buildings, mut put_back) = (0, 0);
    for (mut w, visibility, off) in &mut walls {
        let drawn = !off && visibility != Some(&Visibility::Hidden);
        let collider = w.collider.and_then(|i| buildings.as_ref().and_then(|b| b.0.get(i)));
        // This frame's lit surfaces and their texels.
        let mut painted: HashMap<usize, Vec<u16>> = HashMap::new();
        let mut factors: HashMap<usize, f32> = HashMap::new();
        if let (true, true, Some(c)) = (drawn, dynamic, collider) {
            drawn_buildings += 1;
            let data = w.data.clone();
            for (n, (_, at, colour, begin, end)) in lights.0.iter().enumerate() {
                let count = &mut counts[n];
                count.tested += data.lightable.len();
                let own = c.placement.to_own(*at);
                if box_distance(own, data.min, data.max) >= begin + end + SKIP_MARGIN {
                    count.skipped += data.lightable.len();
                    continue;
                }
                for &s in &data.lightable {
                    let l = &data.light.surfaces[s];
                    let rect = l.rect.expect("lightable surfaces have one");
                    match test_surface(&rect, data.facing[s], own, *begin, *end) {
                        Test::Behind => count.behind += 1,
                        Test::OutOfReach(_) => count.out_of_reach += 1,
                        Test::Reached { line_end, .. } => {
                            if c.line_hit(*at, c.placement.to_world(line_end)).is_some() {
                                count.blocked += 1;
                                continue;
                            }
                            count.lit += 1;
                            *count.buildings.entry(w.name.clone()).or_default() += 1;
                            let texels = painted.entry(s).or_insert_with(|| data.stored(s));
                            let change = paint_surface(texels, &rect, l.width as usize, l.height as usize, own, *colour, *begin, *end);
                            count.largest = count.largest.max(change);
                            *factors.entry(s).or_insert(1.0) *= HAZE_FACTOR;
                        }
                    }
                }
            }
        }
        let before = std::mem::take(&mut w.lit);
        if before.is_empty() && painted.is_empty() {
            continue;
        }
        put_back += before.len();
        // Put last frame's surfaces back, then write this frame's.
        if let Some(mut image) = images.get_mut(&w.image) {
            if let Some(data) = image.data.as_mut() {
                for &s in before.iter().filter(|s| !painted.contains_key(s)) {
                    write_texels(data, &w.atlas, &w.data, s, &w.data.stored(s));
                }
                for (&s, texels) in &painted {
                    write_texels(data, &w.atlas, &w.data, s, texels);
                }
            }
        }
        let mut all = vec![1.0f32; w.data.light.surfaces.len()];
        for (&s, &f) in &factors {
            all[s] = f;
        }
        if let Some(mut b) = buffers.get_mut(&w.factors) {
            *b = factor_buffer(&all);
        }
        w.lit = painted.into_keys().collect();
    }
    let now = Instant::now();
    if last_log.is_some_and(|t| now.duration_since(t).as_secs_f32() < 1.0) {
        return;
    }
    *last_log = Some(now);
    let lit: usize = counts.iter().map(|c| c.lit).sum();
    let mut lines = vec![format!(
        "wall point lights{}: {} light(s), {drawn_buildings} drawn building(s) tested, {lit} surface(s) lit, {put_back} put back from last frame",
        if dynamic { "" } else { " (dynamic lighting off: none)" },
        lights.0.len()
    )];
    for ((name, at, _, begin, end), c) in lights.0.iter().zip(&counts) {
        lines.push(format!(
            "  [{name}] at ({:.0}, {:.0}, {:.0}) Begin {begin:.0} End {end:.0}: tested {}, skipped {} (building out of reach), not in front {}, out of reach {}, blocked {}, lit {}, largest texel change {}{}",
            at[0],
            at[1],
            at[2],
            c.tested,
            c.skipped,
            c.behind,
            c.out_of_reach,
            c.blocked,
            c.lit,
            c.largest,
            if c.buildings.is_empty() { String::new() } else { format!(" ({:?})", c.buildings) }
        ));
    }
    let mut log = shared.log.lock().unwrap();
    for l in lines {
        log.detail(l);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn box_distance_is_zero_inside() {
        assert_eq!(box_distance([1.0, 1.0, 1.0], [0.0; 3], [2.0; 3]), 0.0);
        assert_eq!(box_distance([5.0, 1.0, 1.0], [0.0; 3], [2.0; 3]), 3.0);
        assert_eq!(box_distance([5.0, 6.0, 1.0], [0.0; 3], [2.0; 3]), 5.0);
    }
}
