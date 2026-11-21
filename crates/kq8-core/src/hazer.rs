//! Proximity haze zones (`classType=KQHazer`, `docs/behaviour/object-lighting.md` rules 23a–23e).
//!
//! An invisible box `[-radius, +radius]` centered on the hazer's position.
//! Entering its 3D sphere of radius `radius` saves the terrain's current
//! `hazeDistance` as both `origHazeEdge` and `hazeEdge`; moving within the cube
//! ramps `hazeDistance` (and `1.5 * hazeDistance` for `visibleDistance`) by 2D
//! horizontal distance between `innerRadius` (`hazeEdge * hazeCenterFactor`)
//! and `radius` (`hazeEdge`); leaving the cube or deleting the hazer restores
//! `origHazeEdge`.

/// `.kq` class name (`object-lighting.md` rule 23a).
pub const CLASS: &str = "KQHazer";
/// Default `colliderMask` when omitted (rule 23a).
pub const DEFAULT_COLLIDER_MASK: u32 = 0x221f;
/// Bit matching Connor (rule 23a).
pub const CONNOR_MASK: u32 = 4;
/// Fallback terrain `hazeDistance` when a world has no terrain (rule 23b).
pub const DEFAULT_HAZE_DISTANCE: f32 = 20_000.0;

/// What happened when checking Connor against one `KQHazer` (`rule 23b–23c`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HazerEvent {
    Entered(f32),
    Updated(f32),
    Exited(f32),
}

/// One placed `KQHazer` proximity haze zone (`object-lighting.md` rules 23a–23d).
#[derive(Debug, Clone, PartialEq)]
pub struct Hazer {
    pub name: String,
    pub position: [f32; 3],
    pub radius: f32,
    pub inner_radius: f32,
    pub haze_center_factor: f32,
    pub collider_mask: u32,
    pub active: bool,
    pub orig_haze_edge: f32,
    pub haze_edge: f32,
    pub gone: bool,
}

impl Hazer {
    /// Creates an inactive hazer with defaults matching rule 23a.
    pub fn new(name: impl Into<String>, position: [f32; 3], radius: f32, inner_radius: f32, haze_center_factor: f32, collider_mask: u32) -> Self {
        Self {
            name: name.into(),
            position,
            radius,
            inner_radius,
            haze_center_factor,
            collider_mask,
            active: false,
            orig_haze_edge: DEFAULT_HAZE_DISTANCE,
            haze_edge: DEFAULT_HAZE_DISTANCE,
            gone: false,
        }
    }

    /// Cube bounds `[-radius, +radius]` on all three axes (rule 23a, 23c).
    pub fn in_cube(&self, p: [f32; 3]) -> bool {
        (p[0] - self.position[0]).abs() <= self.radius
            && (p[1] - self.position[1]).abs() <= self.radius
            && (p[2] - self.position[2]).abs() <= self.radius
    }

    /// 3D Euclidean distance from the hazer's center to `p` (rule 23b).
    pub fn dist_3d(&self, p: [f32; 3]) -> f32 {
        let dx = p[0] - self.position[0];
        let dy = p[1] - self.position[1];
        let dz = p[2] - self.position[2];
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// 2D horizontal (XY) distance from the hazer's center to `p` (rule 23c).
    pub fn dist_2d(&self, p: [f32; 3]) -> f32 {
        let dx = p[0] - self.position[0];
        let dy = p[1] - self.position[1];
        (dx * dx + dy * dy).sqrt()
    }

    /// Target `hazeDistance` while inside the active cube (rule 23c).
    pub fn haze_for(&self, p: [f32; 3]) -> f32 {
        let center = self.haze_edge * self.haze_center_factor;
        let inner = self.inner_radius.min(self.radius);
        let d = self.dist_2d(p);
        if d < inner {
            center
        } else if d <= self.radius {
            let span = self.radius - inner;
            if span.abs() < 1e-6 {
                center
            } else {
                let t = (d - inner) / span;
                center + t * (self.haze_edge - center)
            }
        } else {
            self.haze_edge
        }
    }

    /// Evaluates Connor at `connor` against this hazer (`rules 23b–23c`).
    /// `current_haze` is the terrain's `hazeDistance` (`20,000` if no terrain).
    pub fn step(&mut self, connor: [f32; 3], current_haze: f32) -> Option<HazerEvent> {
        if self.gone || (self.collider_mask & CONNOR_MASK) == 0 {
            return None;
        }
        if !self.active {
            if self.dist_3d(connor) <= self.radius {
                self.active = true;
                self.orig_haze_edge = current_haze;
                self.haze_edge = current_haze;
                Some(HazerEvent::Entered(self.haze_for(connor)))
            } else {
                None
            }
        } else if !self.in_cube(connor) {
            self.active = false;
            Some(HazerEvent::Exited(self.orig_haze_edge))
        } else {
            Some(HazerEvent::Updated(self.haze_for(connor)))
        }
    }

    /// Removes the hazer from the world; returns `origHazeEdge` to restore if it
    /// was active (`rule 23d`).
    pub fn remove(&mut self) -> Option<f32> {
        self.gone = true;
        if self.active {
            self.active = false;
            Some(self.orig_haze_edge)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn litehaze() -> Hazer {
        // Real values from MASK/DeadCity/kq/litehaze.kq.
        Hazer::new("litehaze", [96_069.0, 168_737.0, 13_890.0], 15_000.0, 3_000.0, 2.0, CONNOR_MASK)
    }

    #[test]
    fn cube_corner_does_not_enter_but_stays_at_edge_once_active() {
        let mut h = litehaze();
        // Inside [-15000, +15000]^3 cube, but 3D distance sqrt(3 * 10000^2) = 17320.5 > 15000.
        let corner = [96_069.0 + 10_000.0, 168_737.0 + 10_000.0, 13_890.0 + 10_000.0];
        assert!(h.in_cube(corner));
        assert!(h.step(corner, 9_000.0).is_none());
        assert!(!h.active);

        // Move inside the 3D sphere (d = 3000 <= innerRadius).
        let center_edge = [96_069.0 + 2_000.0, 168_737.0, 13_890.0];
        assert_eq!(h.step(center_edge, 9_000.0), Some(HazerEvent::Entered(18_000.0)));
        assert!(h.active);

        // Move out to the horizontal cube corner (d_2d = sqrt(2 * 14000^2) > 15000, still in cube): stays active at hazeEdge.
        let xy_corner = [96_069.0 + 14_000.0, 168_737.0 + 14_000.0, 13_890.0];
        assert_eq!(h.step(xy_corner, 18_000.0), Some(HazerEvent::Updated(9_000.0)));
        assert!(h.active);
    }

    #[test]
    fn dead_city_litehaze_ramps_and_restores_on_exit_and_remove() {
        let mut h = litehaze();
        let at = |dx: f32| [96_069.0 + dx, 168_737.0, 13_890.0];

        assert_eq!(h.step(at(0.0), 9_000.0), Some(HazerEvent::Entered(18_000.0)));
        assert_eq!(h.step(at(2_999.0), 18_000.0), Some(HazerEvent::Updated(18_000.0)));
        // Midpoint between 3000 (18000) and 15000 (9000) is 9000 -> 13500.
        assert_eq!(h.step(at(9_000.0), 18_000.0), Some(HazerEvent::Updated(13_500.0)));
        assert_eq!(h.step(at(15_000.0), 13_500.0), Some(HazerEvent::Updated(9_000.0)));
        // Exiting cube restores origHazeEdge = 9000.
        assert_eq!(h.step(at(15_001.0), 13_500.0), Some(HazerEvent::Exited(9_000.0)));
        assert!(!h.active);

        // Re-enter and remove while active -> restores origHazeEdge.
        assert_eq!(h.step(at(1_000.0), 9_000.0), Some(HazerEvent::Entered(18_000.0)));
        assert_eq!(h.remove(), Some(9_000.0));
        assert!(h.step(at(1_000.0), 9_000.0).is_none());
    }

    #[test]
    fn non_connor_mask_ignores_connor() {
        let mut h = Hazer::new("other", [0.0; 3], 1_000.0, 500.0, 0.5, 0x221b);
        assert!(h.step([0.0; 3], 10_000.0).is_none());
    }
}
