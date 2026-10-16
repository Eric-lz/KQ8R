// Building walls lit by their `.dil` light maps (M1 step 5c,
// docs/DESIGN.md; rules: docs/behaviour/object-lighting.md, rule numbers
// are that note's). Used by `building_light.rs`.
//
// Colour = wall texture x light map colour (channel / 15, blended
// bilinearly between light texels by the sampler), no doubling, no sun or
// ambient (rules 13, 14, 16).
//
// Haze per wall surface (M1 step 6f, rule 29): one value for the whole
// surface, from the distance of its nearest corner to the camera, with
// the objects' numbers (rule 19): 0.01 nearer than the start, (d - start)
// / (end - start) after. At 0.99 or more the surface is left out, unless
// the building has b_hazeSolid (then it is drawn at 0.99). Blended on
// screen colours (rule 23).

import bevy_pbr::render::{
    mesh_functions,
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
};

struct Settings {
    // Linear colour (white for textures), alpha = opacity.
    base_colour: vec4<f32>,
    // Haze colour, screen colour 0..1 (rule 23); w unused.
    haze_colour: vec4<f32>,
    // 0 = x1, 1 = x2 (doubled, capped), 2 = off (full brightness).
    mode: u32,
    // 1 if `colour_texture` holds the texture, 0 for a plain colour.
    textured: u32,
    // 1 = haze on (F8); start and end in game units (rules 19, 29).
    haze_on: u32,
    haze_start: f32,
    haze_end: f32,
    // The building's b_hazeSolid (rule 29).
    haze_solid: u32,
    // Game units per Bevy unit.
    game_units: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: Settings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var colour_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var colour_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var light_sampler: sampler;
// Every surface's corners, Bevy model space (`WallCorners`).
@group(constants::MATERIAL_BIND_GROUP) @binding(5) var<storage, read> corners: array<vec4<f32>>;

// The mesh data the wall shader reads (`BuildingMaterial::specialize`).
struct WallIn {
    @builtin(instance_index) instance_index: u32,
    @location(0) position: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) uv_b: vec2<f32>,
    // This corner's surface: its first corner and corner count in `corners`.
    @location(8) surface: vec2<u32>,
};

struct WallVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light_uv: vec2<f32>,
    // The surface's haze (rule 29); above 1 = left out.
    @location(2) @interpolate(flat) haze: f32,
};

@vertex
fn vertex(vertex_in: WallIn) -> WallVertex {
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(vertex_in.position, 1.0));
    // Rule 29: the surface's nearest corner to the camera.
    var nearest = 1.0e30;
    for (var i: u32 = 0u; i < vertex_in.surface.y; i++) {
        let corner = mesh_functions::mesh_position_local_to_world(world_from_local, corners[vertex_in.surface.x + i]);
        nearest = min(nearest, length(corner.xyz - view.world_position));
    }
    let distance = nearest * settings.game_units;
    var haze = 0.01;
    if distance >= settings.haze_start {
        haze = (distance - settings.haze_start) / max(settings.haze_end - settings.haze_start, 0.001);
    }
    if haze >= 0.99 {
        // Left out, unless b_hazeSolid: then drawn at 0.99.
        haze = select(2.0, 0.99, settings.haze_solid == 1u);
    }
    var out: WallVertex;
    out.position = position_world_to_clip(world_position.xyz);
    out.uv = vertex_in.uv;
    out.light_uv = vertex_in.uv_b;
    out.haze = haze;
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

@fragment
fn fragment(in: WallVertex) -> @location(0) vec4<f32> {
    var colour = settings.base_colour;
    if settings.textured == 1u {
        colour = colour * textureSample(colour_texture, colour_sampler, in.uv);
    }
    // The light image holds channel x 17 as plain values: channel / 15.
    var light = textureSample(light_texture, light_sampler, in.light_uv).rgb;
    if settings.mode == 1u {
        light = min(light * 2.0, vec3<f32>(1.0));
    } else if settings.mode == 2u {
        light = vec3<f32>(1.0);
    }
    // STAND-IN (as the ground and the objects): the multiply is meant on
    // the colour as shown on screen.
    var lit = colour.rgb * screen_to_linear(light);
    if settings.haze_on == 1u {
        if in.haze > 1.0 {
            discard; // rule 29: left out
        }
        lit = haze_blend(lit, settings.haze_colour.rgb, in.haze);
    }
    return vec4<f32>(lit, colour.a);
}
