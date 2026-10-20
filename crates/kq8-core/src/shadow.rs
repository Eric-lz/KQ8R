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
//! - **Buildings** (rule 37 (2), M4 4d-7b-3; detailed by A in Q135, M4
//!   4d-7b-5) receive it: those near or far (the caller passes only
//!   those) whose ball comes within reach of the caster's model ball;
//!   their surfaces that reach into a search box in the caster's own
//!   space ([`Caster`], [`search_box`], [`building_receivers`]).
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
/// The search box on buildings is at most this tall and this long on
/// each side (rule 37 (2) (b), (c)).
pub const BUILDING_BOX: f32 = 1800.0;
/// The reach size s = this × the caster's model ball radius (rule 37 (2)).
pub const REACH_SIZE: f32 = 0.287;
/// The band runs from s above the ball's centre to this many s below it
/// (rule 37 (2) (b)).
pub const BAND_BELOW_SIZES: f32 = 6.0;

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
    /// The reach size s.
    pub s: f32,
    /// The search box in the caster's own space; `None`: no search (no
    /// downward sun).
    pub search: Option<(V3, V3)>,
    /// The buildings within reach, by name.
    pub in_reach: Vec<String>,
    /// Surfaces chosen (within reach, reaching into the box).
    pub surfaces: usize,
    pub triangles: Vec<[V3; 3]>,
}

/// A caster as rule 37 (2) sees it: its own space (`origin` = its
/// position, `axes` = its own x, y, z in the world, length 1, turned with
/// its heading) and the ball stored in its shown model's `.kqs` file
/// (`radius`, `centre` in its own space).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Caster {
    pub origin: V3,
    pub axes: [V3; 3],
    pub centre: V3,
    pub radius: f32,
}

impl Caster {
    /// A point of its own space in the world.
    pub fn to_world(&self, p: V3) -> V3 {
        (0..3).fold(self.origin, |w, k| add(w, scale(self.axes[k], p[k])))
    }

    /// A world point in its own space.
    pub fn to_own(&self, p: V3) -> V3 {
        let d = sub(p, self.origin);
        self.axes.map(|a| dot(d, a))
    }

    /// A world direction in its own space.
    fn dir_to_own(&self, d: V3) -> V3 {
        self.axes.map(|a| dot(d, a))
    }

    /// OUR FALLBACK for a caster without a stored ball (not in the note):
    /// world axes, origin at the middle of the bottom of the box around
    /// `points`, the ball around that box (middle, half diagonal).
    pub fn around(points: &[V3]) -> Option<Caster> {
        let (lo, hi) = bounds(points)?;
        let d = sub(hi, lo);
        Some(Caster {
            origin: [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0, lo[2]],
            axes: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            centre: [0.0, 0.0, d[2] / 2.0],
            radius: dot(d, d).sqrt() / 2.0,
        })
    }
}

/// A span `lo..hi` longer than `max` cut to `max` around its middle.
fn at_most(lo: f32, hi: f32, max: f32) -> (f32, f32) {
    if hi - lo > max {
        let m = (lo + hi) / 2.0;
        (m - max / 2.0, m + max / 2.0)
    } else {
        (lo, hi)
    }
}

/// Rule 37 (2) (b), (c): the search box in the caster's own space, as
/// (low corner, high corner), with s = [`REACH_SIZE`] × its ball radius:
/// height from 6 s below to s above the ball's centre (at most 1,800, cut
/// around its middle); across, the rectangle around a 2 s square on the
/// ball's centre at the band's top and its copy moved along the sun down
/// to height 0. `None` when the sun has no downward part (no search).
///
/// **Not reproduced (the user's decision, 2026-10-21):** the original also
/// cuts each side of that rectangle to 1,800. Along the sun that stops it
/// short of the shadow's tip (Connor standing in Daventry: the rectangle
/// ends about 1,750 from him, his shadow about 1,970), so a wall in
/// between got no shadow while the shadow on the ground slid under it.
/// The user does not want the original's cut shadows.
pub fn search_box(c: &Caster, dir: V3) -> Option<(V3, V3)> {
    let d = c.dir_to_own(dir);
    if d[2] >= 0.0 {
        return None;
    }
    let s = REACH_SIZE * c.radius;
    let (bottom, top) = at_most(c.centre[2] - BAND_BELOW_SIZES * s, c.centre[2] + s, BUILDING_BOX);
    // Our reading: "the band's top" is the top after the 1,800 cut.
    let t = top / -d[2];
    let copy = [c.centre[0] + d[0] * t, c.centre[1] + d[1] * t];
    let (mut lo, mut hi) = ([0.0; 3], [0.0; 3]);
    for k in 0..2 {
        lo[k] = c.centre[k].min(copy[k]) - s;
        hi[k] = c.centre[k].max(copy[k]) + s;
    }
    lo[2] = bottom;
    hi[2] = top;
    Some((lo, hi))
}

/// Any part of world triangle `t` inside the caster's own-space box
/// `lo..hi`.
fn reaches_into(c: &Caster, t: &[V3; 3], lo: V3, hi: V3) -> bool {
    let mut poly: Vec<V3> = t.iter().map(|&p| c.to_own(p)).collect();
    for k in 0..3 {
        let mut n = [0.0; 3];
        n[k] = 1.0;
        poly = clip_front(&poly, &Plane { n, d: lo[k] });
        n[k] = -1.0;
        poly = clip_front(&poly, &Plane { n, d: -hi[k] });
        if poly.is_empty() {
            return false;
        }
    }
    !poly.is_empty()
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

/// Rule 37 (2): the building surfaces that receive the shadow of `caster`,
/// from `buildings` (the caller passes only the near and far ones). A
/// building is searched when its ball centre is within its radius + s +
/// [`BUILDING_REACH`] of the caster's ball centre in the world; of it,
/// each surface any part of which is inside the [`search_box`] is taken
/// whole (the box picks surfaces, it does not cut the shadow).
pub fn building_receivers<'a>(buildings: impl IntoIterator<Item = &'a ReceivingBuilding>, caster: &Caster, dir: V3) -> BuildingPick {
    let s = REACH_SIZE * caster.radius;
    let search = search_box(caster, dir);
    let mut pick = BuildingPick { s, search, ..Default::default() };
    let Some((lo, hi)) = search else { return pick };
    let centre = caster.to_world(caster.centre);
    // The box's corners in the world, for a quick first test per surface.
    let corners: Vec<V3> = (0..8).map(|i| caster.to_world([if i & 1 == 0 { lo[0] } else { hi[0] }, if i & 2 == 0 { lo[1] } else { hi[1] }, if i & 4 == 0 { lo[2] } else { hi[2] }])).collect();
    let (wlo, whi) = bounds(&corners).expect("8 corners");
    for b in buildings {
        let d = sub(b.centre, centre);
        if dot(d, d).sqrt() > b.radius + s + BUILDING_REACH {
            continue;
        }
        pick.in_reach.push(b.name.clone());
        for surface in &b.surfaces {
            if !(0..3).all(|k| surface.hi[k] >= wlo[k] && surface.lo[k] <= whi[k]) {
                continue;
            }
            if surface.triangles.iter().any(|t| reaches_into(caster, t, lo, hi)) {
                pick.surfaces += 1;
                pick.triangles.extend_from_slice(&surface.triangles);
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

    const WORLD_AXES: [V3; 3] = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];

    /// Connor standing at the origin, facing along the world's axes:
    /// `C_stop.kqs` ball radius 879, centre 827 up (rule 37 (2)).
    fn connor() -> Caster {
        Caster { origin: [0.0; 3], axes: WORLD_AXES, centre: [0.0, 0.0, 827.0], radius: 879.0 }
    }

    fn near(a: f32, b: f32) -> bool {
        (a - b).abs() < 0.5
    }

    #[test]
    fn a_building_is_searched_within_its_radius_plus_s_plus_500() {
        let dir = [0.0, 0.0, -1.0];
        let s = REACH_SIZE * 879.0;
        assert!(near(s, 252.3), "{s}");
        // Measured from the ball's centre (827 up), not the feet.
        let reach = 1000.0 + s + BUILDING_REACH;
        let at = |x: f32| building([x, 0.0, 827.0], 1000.0, vec![tile(0.0, 0.0, 0.0, 400.0)]);
        let p = building_receivers([&at(reach - 1.0)], &connor(), dir);
        assert_eq!((p.in_reach.len(), p.surfaces, p.triangles.len()), (1, 1, 2));
        assert_eq!(building_receivers([&at(reach + 1.0)], &connor(), dir).in_reach.len(), 0);
    }

    #[test]
    fn the_band_runs_from_6_s_below_to_s_above_the_balls_centre() {
        let dir = [0.0, 0.0, -1.0];
        // Connor standing: 1,079 above his feet down to 687 below them.
        let (lo, hi) = search_box(&connor(), dir).unwrap();
        assert!(near(hi[2], 1079.3) && near(lo[2], -686.6), "{lo:?} {hi:?}");
        let n = |z: f32| building_receivers([&building([0.0; 3], 5000.0, vec![tile(0.0, 0.0, z, 100.0)])], &connor(), dir).surfaces;
        assert_eq!((n(1078.0), n(1081.0), n(-685.0), n(-688.0)), (1, 0, 1, 0));
    }

    #[test]
    fn a_band_over_1800_is_cut_around_its_middle() {
        // Connor running: C_run.kqs 1,134 / 799: about 915 below his feet
        // to 885 above.
        let run = Caster { centre: [0.0, 0.0, 799.0], radius: 1134.0, ..connor() };
        let (lo, hi) = search_box(&run, [0.0, 0.0, -1.0]).unwrap();
        assert!(near(hi[2] - lo[2], 1800.0) && near(hi[2], 885.4) && near(lo[2], -914.6), "{lo:?} {hi:?}");
    }

    #[test]
    fn the_area_spans_the_square_and_its_copy_along_the_sun_uncut() {
        // Daventry's sun, Connor facing along it: the copy moves 1,079 /
        // 0.66 = 1,635; 505 across by 2,139 (the original cuts that to
        // 1,800 along the sun: not reproduced, the user's decision).
        let dir = direction(Some([0.0, 1.0, -0.66]));
        let (lo, hi) = search_box(&connor(), dir).unwrap();
        let s = REACH_SIZE * 879.0;
        assert!(near(lo[0], -s) && near(hi[0], s), "{lo:?} {hi:?}");
        let (y0, y1) = (-s, 1079.3 / 0.66 + s);
        assert!((y1 - y0 - 2139.0).abs() < 1.0, "{y0} {y1}");
        assert!(near(lo[1], y0) && near(hi[1], y1), "{lo:?} {hi:?}");
        let m = (y0 + y1) / 2.0;
        // Whole surfaces: a big tile reaching in is taken whole.
        let p = building_receivers([&building([0.0; 3], 5000.0, vec![tile(0.0, hi[1] + 1900.0, 0.0, 4000.0)])], &connor(), dir);
        assert_eq!((p.surfaces, p.triangles.len()), (1, 2));
        let n = |x: f32, y: f32| building_receivers([&building([0.0; 3], 5000.0, vec![tile(x, y, 0.0, 10.0)])], &connor(), dir).surfaces;
        assert_eq!((n(0.0, hi[1] - 6.0), n(0.0, hi[1] + 6.0), n(s - 6.0, m), n(s + 6.0, m)), (1, 0, 1, 0));
    }

    #[test]
    fn the_box_turns_with_the_casters_heading() {
        // Sun straight down: a 2 s square around the ball's centre. Turned
        // 45 degrees, a tile near the world-axes square's corner is out.
        let dir = [0.0, 0.0, -1.0];
        let s = REACH_SIZE * 879.0;
        let r = std::f32::consts::FRAC_1_SQRT_2;
        let turned = Caster { axes: [[r, r, 0.0], [-r, r, 0.0], [0.0, 0.0, 1.0]], ..connor() };
        let n = |c: &Caster, x: f32, y: f32| building_receivers([&building([0.0; 3], 5000.0, vec![tile(x, y, 0.0, 10.0)])], c, dir).surfaces;
        let corner = 0.9 * s;
        assert_eq!((n(&connor(), corner, corner), n(&turned, corner, corner)), (1, 0));
        // Along its own turned axis it reaches s.
        assert_eq!((n(&turned, 0.9 * s * r, 0.9 * s * r), n(&turned, 1.1 * s * r, 1.1 * s * r)), (1, 0));
    }

    #[test]
    fn no_downward_sun_means_no_search() {
        let p = building_receivers([&building([0.0; 3], 5000.0, vec![tile(0.0, 0.0, 0.0, 1000.0)])], &connor(), [1.0, 0.0, 0.0]);
        assert_eq!((p.search, p.in_reach.len(), p.surfaces), (None, 0, 0));
    }

    #[test]
    fn the_chosen_floor_receives_the_shadow() {
        let dir = [0.0, 0.0, -1.0];
        let floor_at_500 = building([0.0; 3], 5000.0, vec![tile(0.0, 0.0, 500.0, 2000.0)]);
        // Connor standing on it; a sun-facing triangle 500 above it.
        let up = [[0.0, 0.0, 1000.0], [0.0, 100.0, 1000.0], [100.0, 0.0, 1000.0]];
        let on_floor = Caster { origin: [0.0, 0.0, 500.0], ..connor() };
        let p = building_receivers([&floor_at_500], &on_floor, dir);
        let s = cast(&[up], dir, &p.triangles);
        assert!(!s.is_empty() && s.iter().flatten().all(|q| (q[2] - 500.0 - LIFT).abs() < 1e-3), "{s:?}");
    }

    #[test]
    fn the_fallback_caster_is_the_box_around_the_points() {
        let c = Caster::around(&[[-200.0, -200.0, 100.0], [200.0, 200.0, 1400.0]]).unwrap();
        assert_eq!((c.origin, c.centre[2]), ([0.0, 0.0, 100.0], 650.0));
        assert!(Caster::around(&[]).is_none());
        // Own space and world agree.
        let w = connor().to_world([1.0, 2.0, 3.0]);
        assert_eq!(connor().to_own(w), [1.0, 2.0, 3.0]);
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
