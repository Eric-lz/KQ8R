//! Point lights on building surfaces (M1 step 7d, docs/DESIGN.md; rules:
//! docs/behaviour/object-lighting.md rule 16, rewritten 2026-10-16 for
//! Q78). See docs/provenance/README.md.
//!
//! Everything here works in the building's own axes: the caller turns the
//! light's position into them first. Per surface and light, in this order:
//!
//! - **In front**: facing · unit(P0 − light) below −0.1, P0 being the
//!   surface light map's first corner; facing = the plane normal when the
//!   surface's byte-0 bit 0x80 is set, else the reversed normal.
//! - **Reach**: the nearest of the light map rectangle's four corners and
//!   its centre, closer than Begin + End.
//! - **Line test** (done by the caller, against this building's own BSP):
//!   from the light to the point 64 units from that nearest spot, towards
//!   the light.
//! - **Adding**: every light texel gets colour × falloff (End a fade
//!   length: `lightmap::fade_falloff`), stored back in 4 bits as on the
//!   ground (`lightmap::add_light`). Texel positions are the true ones,
//!   where rule 13 lays the texels on the surface, not the original's
//!   U ÷ width spacing (the note's "Do not reproduce"): one step apart is
//!   U ÷ (width − 1) and V ÷ (height − 1). **Measured** (`kq8
//!   lighting-verify`, 164,685 of 164,838 surfaces): the stored P0 lies 4
//!   texture texels (a quarter of a light texel) further along x and y than
//!   rule 13's texel (0, 0), so texel (i, j) is at P0 + U × (i − ¼) ÷
//!   (width − 1) + V × (j − ¼) ÷ (height − 1) (QUESTIONS.md Q79). P0 itself
//!   is used as stored for "in front" and reach.
//! - A surface that passes every test has its haze multiplied by 0.04, once
//!   per such light (`HAZE_FACTOR`).

use crate::dil::LightRect;
use crate::lightmap::{add_light, fade_falloff, Channels};

/// "In front" needs facing · direction below this (rule 16).
pub const IN_FRONT_LIMIT: f32 = -0.1;
/// The line test ends this far from the nearest spot, towards the light.
pub const LINE_BACK_OFF: f32 = 64.0;
/// A lit surface's haze is multiplied by this, once per light (rule 16).
pub const HAZE_FACTOR: f32 = 0.04;
/// How far the stored P0 lies past rule 13's first texel, in light texels
/// along U and V (4 of 16 texture texels; measured, Q79).
pub const P0_SHIFT: f32 = 0.25;

/// A surface's front direction from its plane normal and byte-0 bit 0x80
/// (rule 16; `buildings.md` as corrected 2026-10-16).
pub fn facing(normal: [f32; 3], flag_0x80: bool) -> [f32; 3] {
    if flag_0x80 { normal } else { normal.map(|v| -v) }
}

/// What the light does to one surface, before the line test.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Test {
    /// Not in front (or the light sits exactly on P0: our choice).
    Behind,
    /// In front, but the nearest of the five points is this far away (not
    /// below Begin + End).
    OutOfReach(f32),
    /// Passes both: the nearest point, its distance, and where the line test
    /// ends.
    Reached { spot: [f32; 3], distance: f32, line_end: [f32; 3] },
}

/// The four corners of the light map's rectangle and its centre.
pub fn reach_points(r: &LightRect) -> [[f32; 3]; 5] {
    let a = r.p0;
    let b = add(r.p0, r.u);
    let c = add(r.p0, r.v);
    let d = add(b, r.v);
    let centre = std::array::from_fn(|k| (a[k] + b[k] + c[k] + d[k]) / 4.0);
    [a, b, c, d, centre]
}

/// The "in front" and reach tests of rule 16, in that order.
pub fn test_surface(rect: &LightRect, facing: [f32; 3], light: [f32; 3], begin: f32, end: f32) -> Test {
    let to_p0 = sub(rect.p0, light);
    let len = length(to_p0);
    if len <= 0.0 || dot(facing, to_p0) / len >= IN_FRONT_LIMIT {
        return Test::Behind;
    }
    let (spot, distance) = reach_points(rect)
        .into_iter()
        .map(|p| (p, length(sub(p, light))))
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .expect("five points");
    if distance >= begin + end {
        return Test::OutOfReach(distance);
    }
    Test::Reached { spot, distance, line_end: line_end(spot, light) }
}

/// The point `LINE_BACK_OFF` units from `spot` towards the light (the
/// light itself if they coincide).
pub fn line_end(spot: [f32; 3], light: [f32; 3]) -> [f32; 3] {
    let d = sub(light, spot);
    let len = length(d);
    if len <= 0.0 {
        return light;
    }
    add(spot, scale(d, LINE_BACK_OFF / len))
}

/// Where light texel (i, j) sits, as rule 13 lays it on the surface: the
/// true spacing U ÷ (width − 1) and V ÷ (height − 1), starting `P0_SHIFT`
/// steps before P0 (measured, Q79). A map one texel wide or high has its
/// single column or row at P0 (its U or V is 0, so there is no step to
/// shift by).
pub fn texel_position(rect: &LightRect, width: usize, height: usize, i: usize, j: usize) -> [f32; 3] {
    let fu = if width > 1 { (i as f32 - P0_SHIFT) / (width - 1) as f32 } else { 0.0 };
    let fv = if height > 1 { (j as f32 - P0_SHIFT) / (height - 1) as f32 } else { 0.0 };
    std::array::from_fn(|k| rect.p0[k] + rect.u[k] * fu + rect.v[k] * fv)
}

/// Adds one light to a surface's texels (`width` × `height`, row by row,
/// as in the `.dil`), on the values as they are now (a second light adds
/// to the first one's rounded result). Returns the largest change of one
/// colour channel, in 0-15 steps.
#[allow(clippy::too_many_arguments)]
pub fn paint_surface(texels: &mut [u16], rect: &LightRect, width: usize, height: usize, light: [f32; 3], colour: [f32; 3], begin: f32, end: f32) -> u8 {
    let mut largest = 0;
    for j in 0..height {
        for i in 0..width {
            let Some(t) = texels.get_mut(j * width + i) else { return largest };
            let d = length(sub(texel_position(rect, width, height, i, j), light));
            let before = Channels::of(*t);
            *t = add_light(*t, colour, fade_falloff(d, begin, end));
            let after = Channels::of(*t);
            largest = largest.max(after.red.saturating_sub(before.red)).max(after.green.saturating_sub(before.green)).max(after.blue.saturating_sub(before.blue));
        }
    }
    largest
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn add(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn scale(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A floor 3 x 2 light texels (Daventry spacing, 128 units) at z = 0,
    /// facing up.
    fn floor() -> LightRect {
        LightRect { p0: [0.0, 0.0, 0.0], u: [256.0, 0.0, 0.0], v: [0.0, 128.0, 0.0] }
    }

    #[test]
    fn facing_follows_bit_0x80() {
        assert_eq!(facing([0.0, 0.0, 1.0], true), [0.0, 0.0, 1.0]);
        assert_eq!(facing([0.0, 0.0, 1.0], false), [0.0, 0.0, -1.0]);
    }

    #[test]
    fn in_front_is_measured_to_p0() {
        let up = [0.0, 0.0, 1.0];
        // Above P0: direction light -> P0 is straight down, facing · d = -1.
        assert!(matches!(test_surface(&floor(), up, [0.0, 0.0, 500.0], 700.0, 850.0), Test::Reached { .. }));
        // Below the floor: behind.
        assert_eq!(test_surface(&floor(), up, [0.0, 0.0, -500.0], 700.0, 850.0), Test::Behind);
        // Almost level with the floor (about 3 degrees above it, measured to
        // P0): facing · d = -0.05, not below -0.1, so not in front.
        assert_eq!(test_surface(&floor(), up, [-1000.0, 0.0, 50.0], 5000.0, 5000.0), Test::Behind);
        // About 8 degrees: in front.
        assert!(matches!(test_surface(&floor(), up, [-1000.0, 0.0, 150.0], 5000.0, 5000.0), Test::Reached { .. }));
        // On P0 itself: our choice, not in front.
        assert_eq!(test_surface(&floor(), up, [0.0, 0.0, 0.0], 700.0, 850.0), Test::Behind);
    }

    #[test]
    fn reach_uses_the_four_corners_and_the_centre() {
        let up = [0.0, 0.0, 1.0];
        // A fireplace light (Begin 700, End 850): reach 1,550.
        let far_corner = [256.0, 128.0, 0.0];
        let light = [256.0 + 1000.0, 128.0, 1000.0];
        let Test::Reached { spot, distance, line_end } = test_surface(&floor(), up, light, 700.0, 850.0) else { panic!("in reach") };
        assert_eq!(spot, far_corner);
        assert!((distance - 1414.2135).abs() < 0.01);
        // The line ends 64 units from that corner, towards the light.
        assert!((length(sub(line_end, far_corner)) - 64.0).abs() < 0.01);
        assert!((length(sub(light, line_end)) - (distance - 64.0)).abs() < 0.01);
        // Just beyond 1,550 (1,552) from the nearest corner: out of reach.
        let l = [256.0, 128.0 + 1500.0, 400.0];
        assert!(matches!(test_surface(&floor(), up, l, 700.0, 850.0), Test::OutOfReach(_)));
        // The middle of a long edge is not tried: a long wall with the light
        // 1,000 off the middle of its edge but 2,000+ from the corners and
        // the centre fails.
        let long = LightRect { p0: [0.0, 0.0, 0.0], u: [4096.0, 0.0, 0.0], v: [0.0, 4096.0, 0.0] };
        assert!(matches!(test_surface(&long, up, [2048.0, -900.0, 400.0], 700.0, 850.0), Test::OutOfReach(_)));
    }

    #[test]
    fn texels_sit_at_the_true_spacing() {
        let r = floor();
        // 3 x 2 texels: U / 2 and V / 1 apart (not U / 3, V / 2), starting a
        // quarter step (32 units here) before P0 (Q79).
        assert_eq!(texel_position(&r, 3, 2, 0, 0), [-32.0, -32.0, 0.0]);
        assert_eq!(texel_position(&r, 3, 2, 1, 0), [96.0, -32.0, 0.0]);
        assert_eq!(texel_position(&r, 3, 2, 2, 1), [224.0, 96.0, 0.0]);
        // One texel wide: the column at P0's x.
        assert_eq!(texel_position(&r, 1, 2, 0, 1), [0.0, 96.0, 0.0]);
    }

    #[test]
    fn painting_adds_and_rounds_like_the_ground() {
        let r = floor();
        // Stored grey 4 everywhere (brightness bits 0x5000 kept).
        let mut t = vec![0x5444u16; 6];
        // A light 500 above texel (0, 0): Begin 700 -> falloff 1 there.
        // Colour 0.2: 4 / 15 + 0.2 = 0.4667, x 15.9 = 7.42 -> 7.
        let change = paint_surface(&mut t, &r, 3, 2, [0.0, 0.0, 500.0], [0.2, 0.2, 0.0], 700.0, 850.0);
        assert_eq!(t[0], 0x5774);
        assert_eq!(change, 3);
        // Texel (2, 1) is about 550 away: also falloff 1.
        assert_eq!(t[5], 0x5774);
        // A second light adds to the rounded result: 7 / 15 + 0.2 -> 10.6 -> 10.
        paint_surface(&mut t, &r, 3, 2, [0.0, 0.0, 500.0], [0.2, 0.0, 0.0], 700.0, 850.0);
        assert_eq!(t[0], 0x5A74);
        // Far away (falloff 0): 4 / 15 x 15.9 = 4.24 -> 4, no change.
        let mut far = vec![0x0444u16; 6];
        assert_eq!(paint_surface(&mut far, &r, 3, 2, [0.0, 0.0, 9000.0], [1.0, 1.0, 1.0], 700.0, 850.0), 0);
        assert_eq!(far, vec![0x0444; 6]);
    }
}
