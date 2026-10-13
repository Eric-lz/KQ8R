//! The geometry of clicking an object (M3 step 3a-3): which box the mouse
//! ray hits, and how far Connor is from a box.
//!
//! Rules from `docs/behaviour/doors.md` "Clicking a door" (traced):
//! rule 10 (the nearest object under the cursor whose collision type has
//! bit 0x20) and rule 11 (the distance from Connor's feet to the object's
//! box, measured in the object's own axes: on each axis only the part
//! outside the box counts).

use crate::collide::Placement;

/// The "clickable" bit of a collision type (rule 10).
pub const CLICKABLE: u32 = 0x20;

/// The distance from world point `p` to the box `min..max` of an object
/// placed by `placement` (rule 11). 0 inside the box.
pub fn distance_to_box(p: [f32; 3], placement: &Placement, min: [f32; 3], max: [f32; 3]) -> f32 {
    let own = placement.to_own(p);
    let outside: [f32; 3] = std::array::from_fn(|k| (min[k] - own[k]).max(own[k] - max[k]).max(0.0));
    outside.iter().map(|v| v * v).sum::<f32>().sqrt()
}

/// How far along the ray `origin + t × dir` (t ≥ 0) it first meets the
/// box `min..max` of an object placed by `placement`; `None` if it misses.
/// A ray starting inside the box meets it at 0.
pub fn ray_box(origin: [f32; 3], dir: [f32; 3], placement: &Placement, min: [f32; 3], max: [f32; 3]) -> Option<f32> {
    let o = placement.to_own(origin);
    // Directions turn like points, without the position.
    let d: [f32; 3] = std::array::from_fn(|j| (0..3).map(|k| dir[k] * placement.rotation[j][k]).sum());
    let (mut t0, mut t1) = (0.0f32, f32::INFINITY);
    for k in 0..3 {
        if d[k].abs() < 1e-12 {
            if o[k] < min[k] || o[k] > max[k] {
                return None;
            }
            continue;
        }
        let (a, b) = ((min[k] - o[k]) / d[k], (max[k] - o[k]) / d[k]);
        let (near, far) = if a < b { (a, b) } else { (b, a) };
        t0 = t0.max(near);
        t1 = t1.min(far);
        if t0 > t1 {
            return None;
        }
    }
    Some(t0)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `door1.kqs`'s box (BOUNDS.KQ), placed at (1000, 2000, 0) turned 90°.
    fn door() -> (Placement, [f32; 3], [f32; 3]) {
        let placement = Placement { rotation: kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]), position: [1000.0, 2000.0, 0.0] };
        (placement, [-630.0, -190.0, 0.0], [600.0, -60.0, 1730.0])
    }

    #[test]
    fn distance_is_measured_in_the_doors_own_axes() {
        let (pl, min, max) = door();
        // Own (0, -1000, 0) is 810 in front of the y = -190 face; turned
        // 90° it is at world (1000 + 1000, 2000 + 0).
        let p = pl.to_world([0.0, -1000.0, 0.0]);
        assert!((distance_to_box(p, &pl, min, max) - 810.0).abs() < 0.01);
        // A corner: (300, 400) outside on x and y -> 500.
        let p = pl.to_world([900.0, -590.0, 500.0]);
        assert!((distance_to_box(p, &pl, min, max) - 500.0).abs() < 0.01);
        assert_eq!(distance_to_box(pl.to_world([0.0, -100.0, 10.0]), &pl, min, max), 0.0);
        // Reach 4,700: 4,699 out is in reach, 4,701 is not.
        let at = |d: f32| distance_to_box(pl.to_world([0.0, -190.0 - d, 0.0]), &pl, min, max);
        assert!(at(4699.0) < 4700.0 && at(4701.0) >= 4700.0);
    }

    #[test]
    fn the_ray_meets_the_turned_box() {
        let (pl, min, max) = door();
        // From 5,000 in front of the door (own -y side), looking at it.
        let from = pl.to_world([0.0, -5000.0, 1000.0]);
        let to = pl.to_world([0.0, 0.0, 1000.0]);
        let dir: [f32; 3] = std::array::from_fn(|k| (to[k] - from[k]) / 5000.0);
        let t = ray_box(from, dir, &pl, min, max).unwrap();
        assert!((t - 4810.0).abs() < 0.05, "{t}");
        // Looking the other way: no hit.
        assert_eq!(ray_box(from, dir.map(|v| -v), &pl, min, max), None);
        // Passing above the door: no hit.
        let high = pl.to_world([0.0, -5000.0, 3000.0]);
        assert_eq!(ray_box(high, dir, &pl, min, max), None);
    }
}
