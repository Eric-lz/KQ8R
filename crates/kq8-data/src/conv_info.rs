//! A conversation partner's info object (`classType=KQConvInfo`, M4 step
//! 4e-1): where Connor stands to talk, the areas on the ground and the
//! conversation cameras.
//!
//! From the behaviour note `docs/behaviour/conversation.md` (session A)
//! rule 3, and the data (`daventry/kq/wizinfo.kq`, `uglyinfo.kq`):
//!
//! ```text
//! [WizardInfo]
//! classType=KQConvInfo
//! convLocX=192171
//! nBox=3
//! box0=190821 201946 196408 202146 196237 194713 190191 194917
//! nCam=7
//! cam0=196075 200403 8086 -.21 0 2.28
//! ```
//!
//! - The section is called `<partner name>Info`.
//! - `convLocX/Y/Z`: the talking spot (game units).
//! - `box<n>`: 4 corners `x y`, heights ignored. `box0` = the start area,
//!   `box1` = the stay area, `box2`… = special areas.
//! - `cam<n>`: `x y z rx ry rz`, as `move KQCamera 0 …`.
//! - Inside (rule 3): the angles the corners make, seen from the point, add
//!   up to between 6.1 and 6.4 radians; a point exactly on a corner counts
//!   as outside.

use crate::kq_text::Section;

/// The class name in the data.
pub const CLASS: &str = "KQConvInfo";
/// The info object's name is the partner's with this added (rule 3).
pub const SUFFIX: &str = "Info";
/// Rule 3: inside when the angle sum lies in this range (radians).
pub const INSIDE_MIN: f64 = 6.1;
pub const INSIDE_MAX: f64 = 6.4;

/// One area: four corners (x, y).
pub type Area = [[f32; 2]; 4];

/// A camera: position and the three angles (`rx ry rz`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConvCamera {
    pub position: [f32; 3],
    pub turn: [f32; 3],
}

/// An info object as read.
#[derive(Debug, Clone, PartialEq)]
pub struct ConvInfo {
    /// The section's name (`WizardInfo`).
    pub name: String,
    /// The talking spot.
    pub spot: [f32; 3],
    /// `box0`, `box1`, … in order (a missing or bad one ends the list,
    /// logged in `problems`).
    pub areas: Vec<Area>,
    pub cameras: Vec<ConvCamera>,
    /// Keys that could not be read, for the log.
    pub problems: Vec<String>,
}

fn numbers(v: &str) -> Option<Vec<f32>> {
    v.split_whitespace().map(|w| w.parse::<f32>().ok()).collect()
}

impl ConvInfo {
    /// `None` when the section is not a `KQConvInfo`.
    pub fn from_section(s: &Section) -> Option<Self> {
        if !s.get("classType").is_some_and(|c| c.trim().eq_ignore_ascii_case(CLASS)) {
            return None;
        }
        let mut problems = Vec::new();
        let mut number = |k: &str| match s.number(k) {
            Some(Ok(v)) => v as f32,
            Some(Err(e)) => {
                problems.push(format!("{e}; taken as 0"));
                0.0
            }
            None => {
                problems.push(format!("no {k}; taken as 0"));
                0.0
            }
        };
        let spot = [number("convLocX"), number("convLocY"), number("convLocZ")];
        let count = |k: &str, problems: &mut Vec<String>| match s.number(k) {
            Some(Ok(v)) if v >= 0.0 => v as usize,
            Some(_) => {
                problems.push(format!("{k}={} is not a count; taken as 0", s.get(k).unwrap_or("")));
                0
            }
            None => 0,
        };
        let n_box = count("nBox", &mut problems);
        let n_cam = count("nCam", &mut problems);
        let mut areas = Vec::new();
        for k in 0..n_box {
            match s.get(&format!("box{k}")).and_then(numbers) {
                Some(v) if v.len() == 8 => areas.push(std::array::from_fn(|i| [v[2 * i], v[2 * i + 1]])),
                Some(v) => {
                    problems.push(format!("box{k} has {} numbers, not 8; the list ends here", v.len()));
                    break;
                }
                None => {
                    problems.push(format!("box{k} missing or not numbers; the list ends here"));
                    break;
                }
            }
        }
        let mut cameras = Vec::new();
        for k in 0..n_cam {
            match s.get(&format!("cam{k}")).and_then(numbers) {
                Some(v) if v.len() == 6 => cameras.push(ConvCamera { position: [v[0], v[1], v[2]], turn: [v[3], v[4], v[5]] }),
                _ => {
                    problems.push(format!("cam{k} missing or not 6 numbers; the list ends here"));
                    break;
                }
            }
        }
        Some(ConvInfo { name: s.name.clone(), spot, areas, cameras, problems })
    }

    /// The partner this object is for: its name without `Info` (rule 3).
    pub fn partner(&self) -> &str {
        let n = &self.name;
        if n.len() > SUFFIX.len() && n[n.len() - SUFFIX.len()..].eq_ignore_ascii_case(SUFFIX) {
            &n[..n.len() - SUFFIX.len()]
        } else {
            n
        }
    }

    /// Whether (x, y) is inside area `k` (`false` when there is none).
    pub fn in_area(&self, k: usize, p: [f32; 2]) -> bool {
        self.areas.get(k).is_some_and(|a| inside(a, p))
    }

    /// The special areas (`box2` and later) that hold (x, y).
    pub fn special_area(&self, p: [f32; 2]) -> Option<usize> {
        (2..self.areas.len()).find(|&k| self.in_area(k, p))
    }
}

/// The angles the four corners make, seen from `p`, added up (radians).
pub fn angle_sum(area: &Area, p: [f32; 2]) -> f64 {
    let mut sum = 0.0;
    for i in 0..4 {
        let a = area[i];
        let b = area[(i + 1) % 4];
        let (ax, ay) = (f64::from(a[0]) - f64::from(p[0]), f64::from(a[1]) - f64::from(p[1]));
        let (bx, by) = (f64::from(b[0]) - f64::from(p[0]), f64::from(b[1]) - f64::from(p[1]));
        let (la, lb) = ((ax * ax + ay * ay).sqrt(), (bx * bx + by * by).sqrt());
        if la == 0.0 || lb == 0.0 {
            // On a corner: no angle there; the sum then falls short (rule 3:
            // a point exactly on a corner counts as outside).
            return 0.0;
        }
        sum += ((ax * bx + ay * by) / (la * lb)).clamp(-1.0, 1.0).acos();
    }
    sum
}

/// Rule 3's inside test.
pub fn inside(area: &Area, p: [f32; 2]) -> bool {
    let s = angle_sum(area, p);
    (INSIDE_MIN..=INSIDE_MAX).contains(&s)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wizard() -> ConvInfo {
        // Made-up numbers in the file's layout (not the game's).
        let text = b"[BobInfo]\nclassType=KQConvInfo\nconvLocX=100\nconvLocY=200\nconvLocZ=30\nnBox=3\n\
box0=0 0 1000 0 1000 1000 0 1000\nbox1=0 0 500 0 500 500 0 500\nbox2=-10 -10 -1 -10 -1 -1 -10 -1\n\
nCam=2\ncam0=1 2 3 -.2 0 1.5\ncam1=4 5 6 0 0 0\n";
        ConvInfo::from_section(&crate::kq_text::parse(text)[0]).unwrap()
    }

    #[test]
    fn reads_the_spot_areas_and_cameras() {
        let w = wizard();
        assert_eq!(w.partner(), "Bob");
        assert_eq!(w.spot, [100.0, 200.0, 30.0]);
        assert_eq!(w.areas.len(), 3);
        assert_eq!(w.areas[1][2], [500.0, 500.0]);
        assert_eq!(w.cameras[0], ConvCamera { position: [1.0, 2.0, 3.0], turn: [-0.2, 0.0, 1.5] });
        assert!(w.problems.is_empty(), "{:?}", w.problems);
    }

    #[test]
    fn inside_by_the_angle_sum_corners_outside_order_free() {
        let w = wizard();
        assert!(w.in_area(0, [600.0, 600.0]) && !w.in_area(1, [600.0, 600.0]));
        assert!(w.in_area(1, [250.0, 250.0]));
        assert!(!w.in_area(0, [1500.0, 50.0]));
        assert!(!w.in_area(0, [0.0, 0.0]), "a corner counts as outside");
        assert!(w.in_area(0, [500.0, 0.0]), "a point on an edge: the sum is still 2 pi");
        assert_eq!(w.special_area([-5.0, -5.0]), Some(2));
        assert_eq!(w.special_area([5.0, 5.0]), None);
        // Corners the other way round: the same.
        let mut a = w.areas[1];
        a.reverse();
        assert!(inside(&a, [250.0, 250.0]));
        assert!(!w.in_area(7, [0.0, 0.0]), "no such area");
    }

    #[test]
    fn other_classes_and_bad_lists() {
        let s = &crate::kq_text::parse(b"[X]\nclassType=KQEnvInfo\n")[0];
        assert!(ConvInfo::from_section(s).is_none());
        let s = &crate::kq_text::parse(b"[XInfo]\nclassType=KQConvInfo\nnBox=2\nbox0=1 2 3\n")[0];
        let c = ConvInfo::from_section(s).unwrap();
        assert!(c.areas.is_empty() && c.problems.iter().any(|p| p.contains("box0 has 3")));
    }
}
