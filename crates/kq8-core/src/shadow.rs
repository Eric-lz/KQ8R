//! Shadows, the geometry (M4 step 4d-7b-1): where a character's shadow
//! falls, as triangles in game space. Drawing them is the viewer's.
//!
//! Rules from `docs/behaviour/characters.md` rules 36 and 37 (Q123,
//! traced 2026-10-19; the user's check of the look):
//!
//! - **Direction:** the first sun's aim made length 1; with no sun
//!   (1, 1, −1) made length 1 ([`direction`]).
//! - **The copy:** each model triangle facing the sun is projected along
//!   the sun onto the plane of each receiving triangle and cut to it; each
//!   piece is kept on its own, so overlapping parts darken twice
//!   ([`cast`]).
//! - **Lifted 20:** the receiving plane moves 20 toward its front first.
//! - **Above the plane only:** only the part of a model triangle above the
//!   receiving plane is projected (a wading figure: the part above the
//!   water; rule 37's suggestion).
//! - **The terrain** receives it only within 1,500 of the ground under the
//!   character; the triangles searched are those of the drawn ground (the
//!   surface grid) under its box moved along the sun
//!   ([`terrain_receivers`]).
//!
//! Our readings (not in the note): "facing the sun" is the model
//! triangle's front (clockwise corners seen from the front, Q2) turned
//! toward the sun; a receiving triangle whose front faces away from the
//! sun gets nothing (the projection would run back toward the sun).

pub type V3 = [f32; 3];

/// The receiving plane is moved this far toward its front (rule 37).
pub const LIFT: f32 = 20.0;
/// The terrain receives a shadow only when the character is within this
/// height of the ground under it (rule 37: a cut, not a fade).
pub const TERRAIN_REACH: f32 = 1500.0;
/// The strength passed to the renderer (rule 37): each layer halves.
pub const STRENGTH: f32 = 0.5;

fn sub(a: V3, b: V3) -> V3 {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}
fn add(a: V3, b: V3) -> V3 {
    [a[0] + b[0], a[1] + b[1], a[2] + b[2]]
}
fn scale(a: V3, s: f32) -> V3 {
    [a[0] * s, a[1] * s, a[2] * s]
}
fn dot(a: V3, b: V3) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}
fn cross(a: V3, b: V3) -> V3 {
    [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
}
fn unit(a: V3) -> Option<V3> {
    let l = dot(a, a).sqrt();
    (l > 1e-9).then(|| scale(a, 1.0 / l))
}

/// Rule 37: the sun's aim made length 1, else (1, 1, −1) made length 1.
pub fn direction(sun_aim: Option<V3>) -> V3 {
    sun_aim.and_then(unit).unwrap_or_else(|| unit([1.0, 1.0, -1.0]).unwrap())
}

/// A triangle's front direction (not length 1): its corners run clockwise
/// seen from the front (`models-and-textures.md`, Q2).
pub fn front(t: &[V3; 3]) -> V3 {
    cross(sub(t[2], t[0]), sub(t[1], t[0]))
}

/// The model triangle's front is turned toward the sun.
pub fn faces_sun(t: &[V3; 3], dir: V3) -> bool {
    dot(front(t), dir) < 0.0
}

/// A plane `n · p = d` with `n` length 1; its front is the `n` side.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Plane {
    pub n: V3,
    pub d: f32,
}

impl Plane {
    /// The plane of a triangle, front as the triangle's; `None` if it has
    /// no area.
    pub fn of(t: &[V3; 3]) -> Option<Plane> {
        let n = unit(front(t))?;
        Some(Plane { n, d: dot(n, t[0]) })
    }

    pub fn lifted(self, by: f32) -> Plane {
        Plane { n: self.n, d: self.d + by }
    }

    fn side(&self, p: V3) -> f32 {
        dot(self.n, p) - self.d
    }
}

/// The part of polygon `poly` on the front of `plane` (Sutherland–Hodgman).
pub fn clip_front(poly: &[V3], plane: &Plane) -> Vec<V3> {
    let mut out = Vec::with_capacity(poly.len() + 2);
    for (k, &a) in poly.iter().enumerate() {
        let b = poly[(k + 1) % poly.len()];
        let (sa, sb) = (plane.side(a), plane.side(b));
        if sa >= 0.0 {
            out.push(a);
        }
        if (sa >= 0.0) != (sb >= 0.0) {
            let t = sa / (sa - sb);
            out.push(add(a, scale(sub(b, a), t)));
        }
    }
    out
}

/// `p` moved along `dir` onto `plane`; `None` when `dir` runs along it.
pub fn project(p: V3, dir: V3, plane: &Plane) -> Option<V3> {
    let along = dot(plane.n, dir);
    if along.abs() < 1e-9 {
        return None;
    }
    Some(add(p, scale(dir, -plane.side(p) / along)))
}

/// The part of polygon `poly` (lying in the triangle's plane) inside
/// triangle `t`.
pub fn clip_to_triangle(poly: &[V3], t: &[V3; 3], n: V3) -> Vec<V3> {
    let mut out = poly.to_vec();
    for k in 0..3 {
        if out.is_empty() {
            break;
        }
        let (a, b, c) = (t[k], t[(k + 1) % 3], t[(k + 2) % 3]);
        let mut inward = cross(n, sub(b, a));
        if dot(inward, sub(c, a)) < 0.0 {
            inward = scale(inward, -1.0);
        }
        let Some(inward) = unit(inward) else { return Vec::new() };
        out = clip_front(&out, &Plane { n: inward, d: dot(inward, a) });
    }
    out
}

/// Rule 37: the shadow of `model` (world triangles) along `dir` on each
/// of `receivers` (world triangles): one polygon piece per sun-facing
/// model triangle and receiver it falls on, as triangles (a fan), each
/// kept on its own.
pub fn cast(model: &[[V3; 3]], dir: V3, receivers: &[[V3; 3]]) -> Vec<[V3; 3]> {
    let mut out = Vec::new();
    let planes: Vec<([V3; 3], Plane)> = receivers.iter().filter_map(|r| Plane::of(r).map(|p| (*r, p))).filter(|(_, p)| dot(p.n, dir) < 0.0).collect();
    for m in model.iter().filter(|m| faces_sun(m, dir)) {
        for (r, plane) in &planes {
            let above = clip_front(m, plane);
            if above.len() < 3 {
                continue;
            }
            let on = plane.lifted(LIFT);
            let Some(flat) = above.iter().map(|&p| project(p, dir, &on)).collect::<Option<Vec<V3>>>() else { continue };
            let lifted = r.map(|p| add(p, scale(plane.n, LIFT)));
            let piece = clip_to_triangle(&flat, &lifted, plane.n);
            for k in 1..piece.len().saturating_sub(1) {
                out.push([piece[0], piece[k], piece[k + 1]]);
            }
        }
    }
    out
}

/// The terrain's receiving triangles (rule 37 (1)): none when the
/// character stands more than [`TERRAIN_REACH`] above or below the ground
/// at its position; else the drawn ground's triangles under its box
/// corners (`corners`, world) moved along `dir` down to that ground
/// height, one square of margin.
pub fn terrain_receivers(terrain: &kq8_data::Terrain, position: V3, corners: &[V3], dir: V3) -> Vec<[V3; 3]> {
    let Some(g) = terrain.height_at(position[0], position[1]) else { return Vec::new() };
    if (position[2] - g.height).abs() > TERRAIN_REACH || dir[2] >= 0.0 {
        return Vec::new();
    }
    let (mut lo, mut hi) = ([f32::MAX; 2], [f32::MIN; 2]);
    for &c in corners {
        let t = (c[2] - g.height) / -dir[2];
        let p = [c[0] + dir[0] * t, c[1] + dir[1] * t];
        for k in 0..2 {
            lo[k] = lo[k].min(p[k]).min(c[k]);
            hi[k] = hi[k].max(p[k]).max(c[k]);
        }
    }
    let size = terrain.settings.square_size();
    let (nx, ny) = terrain.squares();
    let cell = |v: f32, n: usize| ((v / size).floor() as i64).clamp(0, n as i64 - 1) as usize;
    let (i0, i1) = (cell(lo[0] - size, nx), cell(hi[0] + size, nx));
    let (j0, j1) = (cell(lo[1] - size, ny), cell(hi[1] + size, ny));
    let mut out = Vec::new();
    for j in j0..=j1 {
        for i in i0..=i1 {
            out.extend(terrain.square_triangles(i, j, 0, 0).into_iter().map(|t| t.positions));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A flat floor at z 0: two triangles, clockwise seen from above.
    fn floor() -> Vec<[V3; 3]> {
        vec![[[-5000.0, -5000.0, 0.0], [-5000.0, 5000.0, 0.0], [5000.0, -5000.0, 0.0]], [[5000.0, -5000.0, 0.0], [-5000.0, 5000.0, 0.0], [5000.0, 5000.0, 0.0]]]
    }

    fn area(t: &[V3; 3]) -> f32 {
        dot(front(t), front(t)).sqrt() / 2.0
    }

    #[test]
    fn the_direction_is_the_suns_made_length_1_else_1_1_minus_1() {
        let d = direction(Some([0.0, 1.0, -0.66]));
        assert!((dot(d, d) - 1.0).abs() < 1e-6 && d[0] == 0.0 && d[1] > 0.0);
        let n = direction(None);
        assert!((n[0] - n[1]).abs() < 1e-6 && (n[0] + n[2]).abs() < 1e-6);
    }

    #[test]
    fn the_floor_is_clockwise_seen_from_above() {
        assert!(Plane::of(&floor()[0]).unwrap().n[2] > 0.99);
    }

    #[test]
    fn a_triangle_straight_under_the_sun_lands_lifted_20_with_its_own_area() {
        // A flat sun-facing triangle 1,000 up, sun straight down.
        let dir = [0.0, 0.0, -1.0];
        let up = [[0.0, 0.0, 1000.0], [0.0, 100.0, 1000.0], [100.0, 0.0, 1000.0]];
        let down_facing = [up[0], up[2], up[1]];
        // Clockwise from above: its front points up, toward the sun.
        assert!(faces_sun(&up, dir) && !faces_sun(&down_facing, dir));
        let s = cast(&[up], dir, &floor());
        assert!(!s.is_empty());
        assert!(s.iter().all(|t| t.iter().all(|p| (p[2] - LIFT).abs() < 1e-3)), "{s:?}");
        let total: f32 = s.iter().map(area).sum();
        assert!((total - 5000.0).abs() < 1.0, "{total}");
        assert!(cast(&[down_facing], dir, &floor()).is_empty());
    }

    #[test]
    fn the_sun_at_an_angle_moves_it_sideways_by_the_height() {
        // Daventry's sun (0, 1, -0.66): 1,000 up lands 1,000/0.66 toward +y.
        let dir = direction(Some([0.0, 1.0, -0.66]));
        let t = [[0.0, 0.0, 1000.0], [0.0, 100.0, 1000.0], [100.0, 0.0, 1000.0]];
        let s = cast(&[t], dir, &floor());
        let y_min = s.iter().flat_map(|t| t.iter().map(|p| p[1])).fold(f32::MAX, f32::min);
        assert!((y_min - (1000.0 - LIFT) / 0.66).abs() < 1.0, "{y_min}");
    }

    #[test]
    fn only_the_part_above_the_plane_casts_and_overlaps_stay_two_layers() {
        let dir = [0.0, 0.0, -1.0];
        // Half under the floor: only the half above casts.
        let tilted = [[0.0, 0.0, -100.0], [0.0, 100.0, 100.0], [100.0, 0.0, 100.0]];
        let below = [[0.0, 0.0, -100.0], [0.0, 100.0, -100.0], [100.0, 0.0, -100.0]];
        assert!(cast(&[below], dir, &floor()).is_empty());
        let s = cast(&[tilted], dir, &floor());
        let total: f32 = s.iter().map(area).sum();
        assert!(total > 100.0 && total < 5000.0, "{total}");
        // Two triangles over the same spot: two pieces, not merged.
        let a = [[0.0, 0.0, 500.0], [0.0, 100.0, 500.0], [100.0, 0.0, 500.0]];
        let b = [[0.0, 0.0, 900.0], [0.0, 100.0, 900.0], [100.0, 0.0, 900.0]];
        assert_eq!(cast(&[a, b], dir, &floor()).len(), 2 * cast(&[a], dir, &floor()).len());
    }

    #[test]
    fn a_shadow_is_cut_to_the_receiving_triangle() {
        let dir = [0.0, 0.0, -1.0];
        // A small receiver under part of a big triangle.
        let small = [[-20.0, -20.0, 0.0], [-20.0, -10.0, 0.0], [-10.0, -20.0, 0.0]];
        let big = [[-500.0, -500.0, 1000.0], [-500.0, 500.0, 1000.0], [500.0, -500.0, 1000.0]];
        let s = cast(&[big], dir, &[small]);
        let total: f32 = s.iter().map(area).sum();
        assert!((total - 50.0).abs() < 0.01, "{total}");
    }
}
