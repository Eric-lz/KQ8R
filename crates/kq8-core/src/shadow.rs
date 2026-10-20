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
//! - **Buildings** (rule 37 (2), M4 4d-7b-3) receive it: those near or far
//!   (the caller passes only those) whose ball comes within reach of the
//!   caster (the sum of their sizes plus 500); their surfaces near the
//!   caster's height, in an area at most 1,800 across centred on the
//!   shadow ([`building_receivers`]). The sizes, the band's numbers and
//!   the area's shape are not in the note: **stand-ins**, question Q135.
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
/// A building receives when its ball comes within the sum of the two
/// sizes plus this (rule 37 (2)).
pub const BUILDING_REACH: f32 = 500.0;
/// The area on buildings is at most this across (rule 37 (2)).
pub const BUILDING_AREA: f32 = 1800.0;
/// STAND-IN (Q135 b): "a little above the character's top".
pub const BAND_ABOVE: f32 = 100.0;
/// STAND-IN (Q135 b): "several times its size below": this many times
/// the caster's height below its feet.
pub const BAND_BELOW_HEIGHTS: f32 = 2.0;

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

/// One drawn surface of a building, in the world: its triangles (game
/// corner order) and the box around them.
#[derive(Debug, Clone)]
pub struct ReceivingSurface {
    pub triangles: Vec<[V3; 3]>,
    pub lo: V3,
    pub hi: V3,
}

impl ReceivingSurface {
    /// `None` without triangles.
    pub fn new(triangles: Vec<[V3; 3]>) -> Option<Self> {
        let (lo, hi) = bounds(triangles.iter().flatten())?;
        Some(ReceivingSurface { triangles, lo, hi })
    }
}

/// A building that can receive shadows: its ball (world centre, radius)
/// and its drawn surfaces in the world (buildings don't move).
#[derive(Debug, Clone)]
pub struct ReceivingBuilding {
    pub name: String,
    pub centre: V3,
    pub radius: f32,
    pub surfaces: Vec<ReceivingSurface>,
}

/// What [`building_receivers`] chose, with its counts for the logs.
#[derive(Debug, Clone, Default)]
pub struct BuildingPick {
    /// The buildings within reach, by name.
    pub in_reach: Vec<String>,
    /// Surfaces chosen (within reach, in the band and the area).
    pub surfaces: usize,
    pub triangles: Vec<[V3; 3]>,
}

/// The box around `points`; `None` if there are none.
fn bounds<'a>(points: impl IntoIterator<Item = &'a V3>) -> Option<(V3, V3)> {
    let mut it = points.into_iter().peekable();
    it.peek()?;
    let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
    for p in it {
        for k in 0..3 {
            lo[k] = lo[k].min(p[k]);
            hi[k] = hi[k].max(p[k]);
        }
    }
    Some((lo, hi))
}

/// Rule 37 (2): the building surfaces that receive the shadow of a caster
/// whose shown model has the points `corners` (world), from `buildings`
/// (the caller passes only the near and far ones). **Stand-ins** for what
/// the note leaves open (Q135), all from the box around `corners`:
///
/// - **reach**: distance between the building's ball centre and the box's
///   middle <= building radius + half the box's diagonal +
///   [`BUILDING_REACH`];
/// - **band**: a surface counts when any part of it lies between
///   [`BAND_BELOW_HEIGHTS`] x the box's height below its bottom and
///   [`BAND_ABOVE`] above its top;
/// - **area**: a square [`BUILDING_AREA`] across on x and y, around the
///   box's middle moved along `dir` down to the box's bottom (the middle
///   of the shadow on a floor at the feet); a surface counts when its x, y
///   extent reaches into it, and receives whole (no cut).
pub fn building_receivers<'a>(buildings: impl IntoIterator<Item = &'a ReceivingBuilding>, corners: &[V3], dir: V3) -> BuildingPick {
    let mut pick = BuildingPick::default();
    let Some((lo, hi)) = bounds(corners) else { return pick };
    let middle = scale(add(lo, hi), 0.5);
    let size = dot(sub(hi, lo), sub(hi, lo)).sqrt() / 2.0;
    let height = hi[2] - lo[2];
    let (band_lo, band_hi) = (lo[2] - BAND_BELOW_HEIGHTS * height, hi[2] + BAND_ABOVE);
    let centre = if dir[2] < 0.0 { add(middle, scale(dir, (middle[2] - lo[2]) / -dir[2])) } else { middle };
    let half = BUILDING_AREA / 2.0;
    for b in buildings {
        let d = sub(b.centre, middle);
        if dot(d, d).sqrt() > b.radius + size + BUILDING_REACH {
            continue;
        }
        pick.in_reach.push(b.name.clone());
        for s in &b.surfaces {
            let in_band = s.hi[2] >= band_lo && s.lo[2] <= band_hi;
            let in_area = (0..2).all(|k| s.hi[k] >= centre[k] - half && s.lo[k] <= centre[k] + half);
            if in_band && in_area {
                pick.surfaces += 1;
                pick.triangles.extend_from_slice(&s.triangles);
            }
        }
    }
    pick
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

    /// A square floor surface `w` across centred on (x, y) at height z,
    /// clockwise seen from above.
    fn tile(x: f32, y: f32, z: f32, w: f32) -> ReceivingSurface {
        let h = w / 2.0;
        let (a, b, c, d) = ([x - h, y - h, z], [x - h, y + h, z], [x + h, y + h, z], [x + h, y - h, z]);
        ReceivingSurface::new(vec![[a, b, d], [d, b, c]]).unwrap()
    }

    fn building(centre: V3, radius: f32, surfaces: Vec<ReceivingSurface>) -> ReceivingBuilding {
        ReceivingBuilding { name: "hut".into(), centre, radius, surfaces }
    }

    /// A caster standing at the origin: 400 wide, 1,300 high (Connor's box).
    fn caster() -> Vec<V3> {
        vec![[-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]]
    }

    #[test]
    fn a_building_receives_only_within_the_sum_of_sizes_plus_500() {
        let dir = [0.0, 0.0, -1.0];
        // Caster's ball: middle (0, 0, 650), radius ~743.
        let size = (400.0f32 * 400.0 * 2.0 + 1300.0 * 1300.0).sqrt() / 2.0;
        let reach = 1000.0 + size + BUILDING_REACH;
        let near = building([reach - 1.0, 0.0, 650.0], 1000.0, vec![tile(0.0, 0.0, 0.0, 1000.0)]);
        let far = building([reach + 1.0, 0.0, 650.0], 1000.0, vec![tile(0.0, 0.0, 0.0, 1000.0)]);
        let p = building_receivers([&near], &caster(), dir);
        assert_eq!((p.in_reach.len(), p.surfaces, p.triangles.len()), (1, 1, 2));
        let p = building_receivers([&far], &caster(), dir);
        assert_eq!((p.in_reach.len(), p.surfaces), (0, 0));
    }

    #[test]
    fn the_band_runs_from_2_heights_below_the_feet_to_100_above_the_top() {
        let dir = [0.0, 0.0, -1.0];
        let at = |z: f32| building([0.0; 3], 5000.0, vec![tile(0.0, 0.0, z, 1000.0)]);
        let n = |z: f32| building_receivers([&at(z)], &caster(), dir).surfaces;
        // Feet at 0, top at 1,300: band -2,600 .. 1,400.
        assert_eq!((n(0.0), n(-2599.0), n(-2601.0), n(1399.0), n(1401.0)), (1, 1, 0, 1, 0));
    }

    #[test]
    fn the_area_is_1800_across_around_the_shadows_middle_on_the_floor() {
        // Daventry's sun: the shadow's middle on the floor lies 650 / 0.66
        // = 985 toward +y from the feet.
        let dir = direction(Some([0.0, 1.0, -0.66]));
        let n = |x: f32, y: f32| building_receivers([&building([0.0; 3], 5000.0, vec![tile(x, y, 0.0, 100.0)])], &caster(), dir).surfaces;
        let mid = 650.0 / 0.66;
        // Tiles 100 wide: their near edge 50 from their centre.
        assert_eq!((n(0.0, mid + 949.0), n(0.0, mid + 951.0)), (1, 0));
        assert_eq!((n(0.0, mid - 949.0), n(0.0, mid - 951.0)), (1, 0));
        assert_eq!((n(949.0, mid), n(951.0, mid)), (1, 0));
        // A big tile reaching into the area is taken whole (no cut).
        let p = building_receivers([&building([0.0; 3], 5000.0, vec![tile(0.0, mid + 1900.0, 0.0, 4000.0)])], &caster(), dir);
        assert_eq!((p.surfaces, p.triangles.len()), (1, 2));
    }

    #[test]
    fn the_chosen_floor_receives_the_shadow() {
        let dir = [0.0, 0.0, -1.0];
        let floor_at_500 = building([0.0; 3], 5000.0, vec![tile(0.0, 0.0, 500.0, 2000.0)]);
        // The caster stands on it; a sun-facing triangle 500 above it.
        let up = [[0.0, 0.0, 1000.0], [0.0, 100.0, 1000.0], [100.0, 0.0, 1000.0]];
        let corners = vec![[-200.0, -200.0, 500.0], [200.0, 200.0, 1800.0]];
        let p = building_receivers([&floor_at_500], &corners, dir);
        let s = cast(&[up], dir, &p.triangles);
        assert!(!s.is_empty() && s.iter().flatten().all(|q| (q[2] - 500.0 - LIFT).abs() < 1e-3), "{s:?}");
        // No points: nothing chosen.
        assert_eq!(building_receivers([&floor_at_500], &[], dir).surfaces, 0);
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
