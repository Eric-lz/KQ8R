// The ground (surface, floor view, flowing squares) with its own haze
// (M1 step 6e, docs/DESIGN.md; rules: docs/behaviour/object-lighting.md,
// rule numbers are that note's). Used by `ground_haze.rs`.
//
// Colour = base colour x texture x the light map (M1 7e-0): the shown map
// is one picture, one texel per map point, looked up per pixel from the
// world position with bilinear filtering (terrain-lighting.md rules 7,
// 10, 11; object-lighting.md rule 13), then the F7 mode.
//
// Haze per corner (rule 27): 0.01 nearer than the start, rising in a
// straight line to 0.99 at visibleDistance, 0.99 beyond; the card blends
// it across the triangle. Plain blend on screen colours (rule 23). The
// ground ends at visibleDistance (rule 28): the corners' distances are
// blended across the triangle too and every point past visibleDistance is
// left out, so a triangle that crosses it is cut there. This is the
// original's cut (rule 28, traced 2026-10-17): along each edge at
// t = (visibleDistance - d1) / (d2 - d1), everything else blended with t;
// inside the triangle the cut is the straight line between those points.

import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, decompress_vertex},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
};

struct Settings {
    // Linear colour (white for textures), alpha = opacity.
    base_colour: vec4<f32>,
    // Haze colour, screen colour 0..1 (rule 23); w unused.
    haze_colour: vec4<f32>,
    // 1 if `colour_texture` holds the texture, 0 for a plain colour.
    textured: u32,
    // 1 = haze on (F8).
    haze_on: u32,
    // Game units: where the ground's haze starts, and visibleDistance.
    haze_start: f32,
    visible_distance: f32,
    // Game units per Bevy unit.
    game_units: f32,
    // F7: 0 = x1, 1 = x2 (doubled, capped), 2 = off (full brightness).
    light_mode: u32,
    // 1 / the map's point spacing (world units), its side in points.
    light_inv_spacing: f32,
    light_side: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: Settings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var colour_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var colour_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var light_sampler: sampler;

struct GroundVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // The world position in game units, x and y (for the light map).
    @location(1) ground_xy: vec2<f32>,
    // This corner's haze (rule 27) and distance in game units (rule 28).
    @location(2) haze: f32,
    @location(3) distance: f32,
};

@vertex
fn vertex(vertex_in: Vertex) -> GroundVertex {
    let v = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
    let distance = length(world_position.xyz - view.world_position) * settings.game_units;
    // Rule 27: 0.01 + (d - start) / (1.02 x (visible - start)), capped 0.99.
    var haze = 0.01;
    if distance >= settings.haze_start {
        let span = 1.02 * max(settings.visible_distance - settings.haze_start, 0.001);
        haze = min(0.01 + (distance - settings.haze_start) / span, 0.99);
    }
    var out: GroundVertex;
    out.position = position_world_to_clip(world_position.xyz);
    out.uv = v.uv;
    // Bevy (x, y, z) = game (x, z, -y) / 100.
    out.ground_xy = vec2<f32>(world_position.x, -world_position.z) * settings.game_units;
    out.haze = haze;
    out.distance = distance;
    return out;
}

// The sRGB curve: a screen colour (0..1) to Bevy's linear colour.
fn screen_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

// And back: Bevy's linear colour to a screen colour (0..1).
fn linear_to_screen(c: vec3<f32>) -> vec3<f32> {
    let low = c * 12.92;
    let high = 1.055 * pow(c, vec3<f32>(1.0 / 2.4)) - 0.055;
    return select(high, low, c <= vec3<f32>(0.0031308));
}

// Rule 23: the card blends the final colour towards the haze colour by h,
// on screen colours (gamma 1.0). `lit` is linear, `haze` a screen colour.
fn haze_blend(lit: vec3<f32>, haze: vec3<f32>, h: f32) -> vec3<f32> {
    return screen_to_linear(mix(linear_to_screen(lit), haze, h));
}

// The light map at a world point (game x, y): channel / 15 per colour,
// bilinear between the map's points (texel (i, j) = point (i, j)).
fn ground_light(xy: vec2<f32>) -> vec3<f32> {
    let uv = (xy * settings.light_inv_spacing + vec2<f32>(0.5)) / settings.light_side;
    let raw = textureSample(light_texture, light_sampler, uv).rgb;
    if settings.light_mode == 1u {
        return min(raw * 2.0, vec3<f32>(1.0));
    } else if settings.light_mode == 2u {
        return vec3<f32>(1.0);
    }
    return raw;
}

@fragment
fn fragment(in: GroundVertex) -> @location(0) vec4<f32> {
    // STAND-IN (M1 4c): the multiply is meant on the colour as shown on
    // screen, so the light goes through the sRGB curve first.
    var colour = settings.base_colour * vec4<f32>(screen_to_linear(ground_light(in.ground_xy)), 1.0);
    if settings.textured == 1u {
        colour = colour * textureSample(colour_texture, colour_sampler, in.uv);
    }
    if settings.haze_on == 1u {
        if in.distance > settings.visible_distance {
            discard; // rule 28
        }
        colour = vec4<f32>(haze_blend(colour.rgb, settings.haze_colour.rgb, in.haze), colour.a);
    }
    return colour;
}
