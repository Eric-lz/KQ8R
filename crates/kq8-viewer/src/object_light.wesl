// Models lit by the world's ambient light and suns (M1 step 5b,
// docs/DESIGN.md; rules: docs/behaviour/object-lighting.md, rule numbers
// are that note's). Used by `object_light.rs`.
//
// Per corner (vertex shader): light = ambient + each sun's colour x
// max(0, -aim . normal), each channel capped to 0..1 (rules 4, 5); the
// normal is turned into world space with the object's own transform
// (rule 5), the aim is as stored, not unit length. The light is blended
// across the triangle (rule 7, smooth). Flat triangles carry their first
// corner's normal at all three corners and use one grey (rule 7). The
// texture is multiplied by the light, no doubling (rule 9).
//
// Haze (M1 step 6, rules 19-23): one amount per object from the camera's
// distance to the object's position (the drawn part's origin), then the
// material's halving, "left out at full haze" unless solid, the object's
// multiplier, and a blend towards the haze colour (solid) or a fade
// (see-through).

import bevy_pbr::render::{
    mesh_functions,
    forward_io::{Vertex, decompress_vertex},
    view_transformations::position_world_to_clip,
    mesh_view_bindings::view,
};

struct Settings {
    // Linear colour (white for textures), alpha = opacity.
    base_colour: vec4<f32>,
    // Sum of the ambient lights (rule 3); w unused.
    ambient: vec4<f32>,
    // Suns: colour (w unused) and aim in world space (w unused).
    sun_colour: array<vec4<f32>, 4>,
    sun_aim: array<vec4<f32>, 4>,
    // Screen colour 0..1 (rule 23); w unused.
    haze_colour: vec4<f32>,
    sun_count: u32,
    // 0 = none (full brightness), 1 = flat, 2 = smooth (rule 7).
    lighting: u32,
    // 0 = x1, 1 = x2 (doubled, capped), 2 = off (full brightness).
    mode: u32,
    // 1 if `colour_texture` holds the texture, 0 for a plain colour.
    textured: u32,
    // 1 = haze on (F8).
    haze_on: u32,
    // Game units (rule 19).
    haze_start: f32,
    haze_end: f32,
    // Material flag 0x10000 (rule 20), the object's b_hazeSolid (rule 21).
    haze_halved: u32,
    haze_solid: u32,
    // The object's hazeMultiplier (rule 22).
    haze_multiplier: f32,
    // Game units per Bevy unit.
    game_units: f32,
};

@group(constants::MATERIAL_BIND_GROUP) @binding(0) var<uniform> settings: Settings;
@group(constants::MATERIAL_BIND_GROUP) @binding(1) var colour_texture: texture_2d<f32>;
@group(constants::MATERIAL_BIND_GROUP) @binding(2) var colour_sampler: sampler;

struct LitVertex {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    // The shown brightness per channel, 0..1 (screen colour, see below).
    @location(1) light: vec3<f32>,
    // The object's haze before the material's halving (rule 19).
    @location(2) @interpolate(flat) haze: f32,
};

// Rules 4, 5: ambient + suns, capped to 0..1 per channel.
fn corner_light(normal: vec3<f32>) -> vec3<f32> {
    var light = settings.ambient.xyz;
    for (var i: u32 = 0u; i < settings.sun_count; i++) {
        let facing = -dot(settings.sun_aim[i].xyz, normal);
        light += settings.sun_colour[i].xyz * max(facing, 0.0);
    }
    return clamp(light, vec3<f32>(0.0), vec3<f32>(1.0));
}

@vertex
fn vertex(vertex_in: Vertex) -> LitVertex {
    let v = decompress_vertex(vertex_in, vertex_in.instance_index);
    let world_from_local = mesh_functions::get_world_from_local(vertex_in.instance_index);
    let world_position = mesh_functions::mesh_position_local_to_world(world_from_local, vec4<f32>(v.position, 1.0));
    // Unit length; a zero normal (degenerate triangles) stays zero and
    // then gets the ambient light only.
    let normal = mesh_functions::mesh_normal_local_to_world(v.normal, vertex_in.instance_index);

    var light = vec3<f32>(1.0);
    if settings.lighting == 1u {
        // Rule 7, flat: one grey from the first corner's light, capped at 1.
        let c = corner_light(normal);
        light = vec3<f32>(min(dot(c, vec3<f32>(0.299, 0.587, 0.114)), 1.0));
    } else if settings.lighting == 2u {
        light = corner_light(normal);
    }
    if settings.mode == 1u {
        light = min(light * 2.0, vec3<f32>(1.0));
    } else if settings.mode == 2u {
        light = vec3<f32>(1.0);
    }

    // Rule 19: distance from the camera to the object's position, in game
    // units; 0.01 closer than the start, capped at 1.
    let distance = length(world_from_local[3].xyz - view.world_position) * settings.game_units;
    var haze = 0.01;
    if distance >= settings.haze_start {
        haze = min((distance - settings.haze_start) / max(settings.haze_end - settings.haze_start, 0.001), 1.0);
    }

    var out: LitVertex;
    out.position = position_world_to_clip(world_position.xyz);
    out.uv = v.uv;
    out.light = light;
    out.haze = haze;
    return out;
}

// The sRGB curve: a screen colour (0..1) to Bevy's linear colour.
fn screen_to_linear(c: vec3<f32>) -> vec3<f32> {
    let low = c / 12.92;
    let high = pow((c + 0.055) / 1.055, vec3<f32>(2.4));
    return select(high, low, c <= vec3<f32>(0.04045));
}

@fragment
fn fragment(in: LitVertex) -> @location(0) vec4<f32> {
    var colour = settings.base_colour;
    if settings.textured == 1u {
        colour = colour * textureSample(colour_texture, colour_sampler, in.uv);
    }
    // STAND-IN (as the ground, M1 4c): the multiply is meant on the colour
    // as shown on screen, so the light goes through the sRGB curve before
    // it multiplies Bevy's linear colour.
    var lit = vec4<f32>(colour.rgb * screen_to_linear(in.light), colour.a);
    if settings.haze_on == 1u {
        var h = in.haze;
        if settings.haze_halved == 1u {
            h = h * 0.5; // rule 20
        }
        if h >= 1.0 && settings.haze_solid == 0u {
            discard; // rule 21
        }
        h = clamp(h * settings.haze_multiplier, 0.0, 1.0); // rule 22
        if settings.base_colour.a < 1.0 {
            // See-through: no haze colour, a fade (rule 23).
            if h >= 0.5 {
                lit.a = lit.a * max(1.2 - h, 0.0);
            }
        } else {
            // STAND-IN (rule 23, "a plain blend is assumed"): blended
            // towards the haze colour as Bevy's fog blends the ground (on
            // linear colours; M1 6b), so objects and ground match.
            lit = vec4<f32>(mix(lit.rgb, screen_to_linear(settings.haze_colour.rgb), h), lit.a);
        }
    }
    return lit;
}

