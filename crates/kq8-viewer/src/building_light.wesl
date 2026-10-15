// Building walls lit by their `.dil` light maps (M1 step 5c,
// docs/DESIGN.md; rules: docs/behaviour/object-lighting.md, rule numbers
// are that note's). Used by `building_light.rs`.
//
// Colour = wall texture x light map colour (channel / 15, blended
// bilinearly between light texels by the sampler), no doubling, no sun or
// ambient (rules 13, 14, 16).
//
// Fog (M1 step 6b, STAND-IN from the user's check, Q68): each point fades
// into the haze colour by its own distance from the camera, linear from
// the haze start to its end, blended as Bevy's fog does (on linear
// colours), so walls match the ground.

import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, decompress_vertex},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
};

struct Settings {
    // Linear colour (white for textures), alpha = opacity.
    base_colour: vec4<f32>,
    // Haze colour, linear; w unused.
    haze_colour: vec4<f32>,
    // 0 = x1, 1 = x2 (doubled, capped), 2 = off (full brightness).
    mode: u32,
    // 1 if `colour_texture` holds the texture, 0 for a plain colour.
    textured: u32,
    // 1 = fog on (F8); start and end in game units.
    haze_on: u32,
    haze_start: f32,
    haze_end: f32,
    // Game units per Bevy unit.
    game_units: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: Settings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var colour_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var colour_sampler: sampler;
@group(constants::MATERIAL_BIND_GROUP) @binding(3) var light_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(4) var light_sampler: sampler;

struct WallVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) light_uv: vec2<f32>,
    @location(2) world_position: vec3<f32>,
};

@vertex
fn vertex(vertex_in: Vertex) -> WallVertex {
    let v = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
    var out: WallVertex;
    out.position = position_world_to_clip(world_position.xyz);
    out.uv = v.uv;
    out.light_uv = v.uv_b;
    out.world_position = world_position.xyz;
    return out;
}

// The sRGB curve: a screen colour (0..1) to Bevy's linear colour.
fn screen_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
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
        let distance = length(in.world_position - view.world_position) * settings.game_units;
        let fog = clamp((distance - settings.haze_start) / max(settings.haze_end - settings.haze_start, 0.001), 0.0, 1.0);
        lit = mix(lit, settings.haze_colour.rgb, fog);
    }
    return vec4<f32>(lit, colour.a);
}
