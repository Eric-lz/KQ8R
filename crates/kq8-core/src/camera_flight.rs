//! Camera flights and cuts (M3 step 3m-4): `KQCamera::addCamLoc`,
//! `flyToCamLoc` and `cutToCamLoc`.
//!
//! Rules from `camera.md` "Camera flights and cuts" rules 1–7, "Edge
//! cases" and "Do not reproduce":
//! - one list of spots; fly and cut use the spot nearest the camera;
//! - a flight is a straight line at an even speed over N = whole(seconds ×
//!   15) + 1 updates (no seconds: distance ÷ 5,000 per second), the roll
//!   changing evenly to `endRoll`; on update N the camera is pulled onto
//!   the range around the aim point (the jump at the end), then the
//!   height rule with the aim point counting as not moved (Q97): lifted
//!   off the ground or snapped within 100, never eased;
//! - while it aims, the camera looks at the point 1,400 above the aim
//!   point: tilt −0.2 plus the angle up to it, set each update; heading
//!   locked within 0.02, otherwise a third of the rest per update, with
//!   one snap angle (0.063) on both sides.
//!
//! Angles are the camera's own (`camera::view_direction`): tilt positive
//! looks up, heading 0 looks along +Y, anticlockwise.

use crate::camera::{height_rule, height_share, view_angles, wrap, HeightStep, AIM_HEIGHT, DEFAULT_TILT, FOLLOW_TURN_SNAP, UPDATE_SECONDS};

/// Without seconds, a flight goes this fast (units per second).
pub const FLIGHT_SPEED: f32 = 5000.0;
/// The camera looks this far above the aim point (rule 5; `camera.md`
/// "What the camera looks at" rule 2): the follow camera's aim height.
pub const LOOK_ABOVE: f32 = AIM_HEIGHT;
/// Within this of the aim (radians) the heading locks on (rule 5).
pub const LOCK_ON: f32 = 0.02;
/// The default tilt in the camera's angles: 0.2 rad down (rule 5), the
/// follow camera's default.
pub const AIM_TILT: f32 = DEFAULT_TILT;
/// Updates per second the flight's length is counted in (rule 4).
const UPDATES: f32 = 15.0;

/// The camera's list of spots (rule 1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Spots(pub Vec<[f32; 3]>);

impl Spots {
    /// `addCamLoc x y z [newlist]`: `newlist` empties the list first.
    pub fn add(&mut self, spot: [f32; 3], newlist: bool) {
        if newlist {
            self.0.clear();
        }
        self.0.push(spot);
    }

    /// The spot nearest `from` (3D); on a tie the one added first (rule 2).
    pub fn nearest(&self, from: [f32; 3]) -> Option<[f32; 3]> {
        let mut best: Option<([f32; 3], f32)> = None;
        for &s in &self.0 {
            let d = distance(s, from);
            if best.is_none_or(|(_, b)| d < b) {
                best = Some((s, d));
            }
        }
        best.map(|(s, _)| s)
    }
}

fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    (0..3).map(|k| (a[k] - b[k]).powi(2)).sum::<f32>().sqrt()
}

/// How many updates a flight takes (rule 4): N = whole(duration × 15) +
/// 1; the duration is `seconds` (negative counts as 0), or if that is 0
/// or missing, `distance` ÷ 5,000 units per second.
pub fn flight_updates(seconds: Option<f32>, distance: f32) -> u32 {
    let s = seconds.unwrap_or(0.0).max(0.0);
    let duration = if s > 0.0 { s } else { distance / FLIGHT_SPEED };
    (duration * UPDATES).floor() as u32 + 1
}

/// The point the camera looks at: 1,400 above the aim point.
pub fn looked_at(aim: [f32; 3]) -> [f32; 3] {
    [aim[0], aim[1], aim[2] + LOOK_ABOVE]
}

/// Where the camera looks while it aims (rule 5).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub tilt: f32,
    pub heading: f32,
    pub roll: f32,
    /// The heading has locked onto the aim (within 0.02).
    pub locked: bool,
}

impl View {
    /// One update of aiming from `position` at `aim` (rule 5): the tilt
    /// set exactly (−0.2 + the angle up to the point 1,400 above it); the
    /// heading locked (exact), or a third of the rest, or snapped. Returns
    /// a word for the log.
    pub fn aim(&mut self, position: [f32; 3], aim: [f32; 3]) -> &'static str {
        let p = looked_at(aim);
        let (up, wanted) = view_angles(std::array::from_fn(|k| p[k] - position[k]));
        self.tilt = AIM_TILT + up;
        let off = wrap(self.heading - wanted);
        if self.locked || off.abs() <= LOCK_ON {
            self.locked = true;
            self.heading = wanted;
            return "locked";
        }
        // One snap angle on both sides ("Do not reproduce").
        if off.abs() > FOLLOW_TURN_SNAP {
            self.heading = wanted + off * (2.0 / 3.0);
            "turning (a third of the rest)"
        } else {
            self.heading = wanted;
            self.locked = true;
            "snapped"
        }
    }
}

/// "Places" on, around the aim point (rule 3, rule 4's last update):
/// farther than the maximum from the point 1,400 above it → pulled in to
/// max − 1 along the line to it; nearer than the minimum → pushed out to
/// min + 1. Returns the new position and the distance before, if it moved.
pub fn onto_range(position: [f32; 3], aim: [f32; 3], range: (f32, f32)) -> ([f32; 3], Option<f32>) {
    let p = looked_at(aim);
    let d = distance(position, p);
    let to = if d > range.1 {
        range.1 - 1.0
    } else if d < range.0 {
        range.0 + 1.0
    } else {
        return (position, None);
    };
    let dir: [f32; 3] = if d < 1e-3 { [0.0, 1.0, 0.0] } else { std::array::from_fn(|k| (position[k] - p[k]) / d) };
    (std::array::from_fn(|k| p[k] + dir[k] * to), Some(d))
}

/// A flight in progress (rule 4).
#[derive(Debug, Clone, PartialEq)]
pub struct Flight {
    pub spot: [f32; 3],
    pub aim: [f32; 3],
    /// The same step every update.
    pub step: [f32; 3],
    pub roll_step: f32,
    /// Updates in all, and done so far.
    pub n: u32,
    pub k: u32,
}

/// What one update of a flight did, for the log.
#[derive(Debug, Clone, PartialEq)]
pub struct FlightStep {
    pub k: u32,
    pub n: u32,
    /// The view's aiming this update, if "aims" is on.
    pub aimed: Option<&'static str>,
    /// The last update: pulled onto the range from this distance.
    pub pulled_from: Option<f32>,
    /// The last update: what the height rule did, and the floor-grid
    /// ground under the camera there.
    pub height: Option<(HeightStep, Option<f32>)>,
}

/// The height rule after a flight's last pull-in or a cut (Q97, `camera.md`
/// "Camera flights and cuts" rules 3–4): a flight or cut sets the camera's
/// present and previous record of the aim point to the aim point, so it
/// counts as not moved: lifted off the ground at once, or snapped within
/// 100 of the wanted height (the higher of the ground + 1,400 and the aim
/// point + 1,400), otherwise kept. `start` is where the update began.
pub fn settle_height(start: [f32; 3], now: [f32; 3], aim: [f32; 3], range: (f32, f32), ground: &dyn Fn(f32, f32) -> Option<f32>, surface_line: &dyn Fn([f32; 3], [f32; 3]) -> Option<[f32; 3]>) -> ([f32; 3], HeightStep, Option<f32>) {
    let (p, h) = height_rule(start, now, aim[2], range.1, 0.0, height_share(UPDATE_SECONDS), false, false, ground, surface_line);
    (p, h, ground(p[0], p[1]))
}

impl Flight {
    /// `flyToCamLoc` from `position` (roll `roll`) to `spot`, aiming at
    /// `aim` (rule 4); `end_roll` is 0 unless given.
    pub fn new(position: [f32; 3], roll: f32, spot: [f32; 3], aim: [f32; 3], seconds: Option<f32>, end_roll: Option<f32>) -> Flight {
        let n = flight_updates(seconds, distance(position, spot));
        let step = std::array::from_fn(|k| (spot[k] - position[k]) / n as f32);
        let roll_step = (end_roll.unwrap_or(0.0) - roll) / n as f32;
        Flight { spot, aim, step, roll_step, n, k: 0 }
    }

    pub fn done(&self) -> bool {
        self.k >= self.n
    }

    /// One update: the step and the roll step, then (if `aims`) the aim.
    /// On the last update the camera is exactly on the spot, "places"
    /// comes back on, it is pulled onto `range` around the aim point and
    /// the height rule runs as "not moved" (`settle_height`); `ground` is
    /// the floor grid's height under a point, `surface_line` the line test
    /// against the drawn surface (the deep lift).
    pub fn update(&mut self, position: &mut [f32; 3], view: &mut View, aims: bool, range: (f32, f32), ground: &dyn Fn(f32, f32) -> Option<f32>, surface_line: &dyn Fn([f32; 3], [f32; 3]) -> Option<[f32; 3]>) -> FlightStep {
        self.k += 1;
        let last = self.k >= self.n;
        let mut pulled_from = None;
        let mut height = None;
        if last {
            let start = *position;
            let (p, from) = onto_range(self.spot, self.aim, range);
            pulled_from = from;
            let (p, h, g) = settle_height(start, p, self.aim, range, ground, surface_line);
            *position = p;
            height = Some((h, g));
        } else {
            *position = std::array::from_fn(|k| position[k] + self.step[k]);
        }
        view.roll += self.roll_step;
        let aimed = aims.then(|| view.aim(*position, self.aim));
        FlightStep { k: self.k, n: self.n, aimed, pulled_from, height }
    }
}

/// What a cut did, for the log.
#[derive(Debug, Clone, PartialEq)]
pub struct Cut {
    pub position: [f32; 3],
    /// Pulled onto the range from this distance.
    pub pulled_from: Option<f32>,
    /// With "places": what the height rule did, and the ground there.
    pub height: Option<(HeightStep, Option<f32>)>,
}

/// `cutToCamLoc` (rule 3): on the spot at once, the heading toward (x, y)
/// at once, the roll set; then one follow update: if `places`, onto the
/// range around the aim point and the height rule as "not moved"
/// (`settle_height`, Q97); if `aims`, aimed.
#[allow(clippy::too_many_arguments)]
pub fn cut(spot: [f32; 3], aim: [f32; 3], roll: f32, view: &mut View, places: bool, aims: bool, range: (f32, f32), ground: &dyn Fn(f32, f32) -> Option<f32>, surface_line: &dyn Fn([f32; 3], [f32; 3]) -> Option<[f32; 3]>) -> Cut {
    view.heading = view_angles([aim[0] - spot[0], aim[1] - spot[1], 0.0]).1;
    view.roll = roll;
    view.locked = false;
    let (position, pulled_from, height) = if places {
        let (p, from) = onto_range(spot, aim, range);
        let (p, h, g) = settle_height(spot, p, aim, range, ground, surface_line);
        (p, from, Some((h, g)))
    } else {
        (spot, None, None)
    };
    if aims {
        view.aim(position, aim);
    }
    Cut { position, pulled_from, height }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_list_and_the_nearest_spot() {
        let mut s = Spots::default();
        s.add([0.0, 0.0, 0.0], true);
        s.add([100.0, 0.0, 0.0], false);
        assert_eq!(s.nearest([80.0, 0.0, 0.0]), Some([100.0, 0.0, 0.0]));
        // A tie: the one added first.
        assert_eq!(s.nearest([50.0, 0.0, 0.0]), Some([0.0, 0.0, 0.0]));
        s.add([7.0, 7.0, 7.0], true);
        assert_eq!(s.0, [[7.0, 7.0, 7.0]]);
        assert_eq!(Spots::default().nearest([0.0; 3]), None);
    }

    #[test]
    fn flight_lengths() {
        // Rule 4: 2 s → 31, 3 s → 46; no seconds: 5,000 per second.
        assert_eq!(flight_updates(Some(2.0), 1.0), 31);
        assert_eq!(flight_updates(Some(3.0), 1.0), 46);
        assert_eq!(flight_updates(None, 10_000.0), 31);
        assert_eq!(flight_updates(Some(-1.0), 0.0), 1);
    }

    #[test]
    fn the_watermill_flight() {
        // camera.md rule 7: spot (190000, 230000, 10600), aim (186960,
        // 228000, 10000), 2 s, range 1000–2500.
        let spot = [190000.0, 230000.0, 10600.0];
        let aim = [186960.0, 228000.0, 10000.0];
        let start = [192000.0, 229000.0, 10000.0];
        let mut f = Flight::new(start, 0.0, spot, aim, Some(2.0), None);
        assert_eq!(f.n, 31);
        let mut p = start;
        let mut v = View { tilt: AIM_TILT, heading: 1.0, roll: 0.0, locked: false };
        let mut last = None;
        for k in 1..=31 {
            let before = p;
            let s = f.update(&mut p, &mut v, true, (1000.0, 2500.0), &|_, _| Some(7650.0), &|_, _| None);
            if k < 31 {
                let d: Vec<f32> = (0..3).map(|i| p[i] - before[i] - f.step[i]).collect();
                assert!(d.iter().all(|x| x.abs() < 0.05), "update {k}: the same step");
            }
            last = Some(s);
        }
        let s = last.unwrap();
        assert!(f.done() && s.k == 31);
        assert!((s.pulled_from.unwrap() - 3725.8).abs() < 0.1, "{s:?}");
        for (got, want) in p.iter().zip([188999.0, 229341.5, 10863.4]) {
            assert!((got - want).abs() < 0.2, "{p:?}");
        }
        // Locked on, the same view direction as from the spot.
        assert!(v.locked);
        assert!((v.heading - 2.153).abs() < 0.001 && (v.tilt - 0.016).abs() < 0.001, "{v:?}");
        // Q97: not moved, 536.6 under the wanted 11,400: kept.
        let (h, g) = s.height.unwrap();
        assert_eq!((h.then, h.wanted, g), (crate::camera::HeightThen::Kept, 11400.0, Some(7650.0)));
    }

    #[test]
    fn after_a_flight_or_cut_the_height_is_lifted_or_snapped_never_eased() {
        let aim = [0.0, 0.0, 0.0];
        // Under the ground + 1,400: lifted at once.
        let (p, h, _) = settle_height([0.0, 3000.0, 500.0], [0.0, 2000.0, 500.0], aim, (1000.0, 5000.0), &|_, _| Some(0.0), &|_, _| None);
        assert_eq!((p[2], h.lift), (1400.0, crate::camera::Lift::ToGround));
        // Within 100 of the wanted height: snapped.
        let (p, h, _) = settle_height([0.0; 3], [0.0, 2000.0, 1450.0], aim, (1000.0, 5000.0), &|_, _| Some(0.0), &|_, _| None);
        assert_eq!((p[2], h.then), (1400.0, crate::camera::HeightThen::Snapped));
        // Far above: kept, not eased.
        let (p, h, _) = settle_height([0.0; 3], [0.0, 2000.0, 3000.0], aim, (1000.0, 5000.0), &|_, _| Some(0.0), &|_, _| None);
        assert_eq!((p[2], h.then), (3000.0, crate::camera::HeightThen::Kept));
    }

    #[test]
    fn aiming_turns_a_third_then_snaps_and_locks() {
        let aim = [0.0, 10_000.0, -1400.0];
        // Looking along +Y is heading 0; start 0.3 off.
        let mut v = View { tilt: 0.0, heading: 0.3, roll: 0.0, locked: false };
        assert_eq!(v.aim([0.0; 3], aim), "turning (a third of the rest)");
        assert!((v.heading - 0.2).abs() < 1e-5);
        assert!((v.tilt - AIM_TILT).abs() < 1e-5, "level point: tilt is the default");
        let mut words = Vec::new();
        while !v.locked {
            words.push(v.aim([0.0; 3], aim));
        }
        assert_eq!(words.last(), Some(&"snapped"));
        assert_eq!(v.heading, 0.0);
    }

    #[test]
    fn the_roll_goes_evenly_to_the_end_roll_or_to_0() {
        let mut f = Flight::new([0.0; 3], 0.3, [0.0, 1000.0, 0.0], [0.0, 5000.0, 0.0], Some(1.0), None);
        let mut p = [0.0; 3];
        let mut v = View { tilt: 0.0, heading: 0.0, roll: 0.3, locked: false };
        while !f.done() {
            f.update(&mut p, &mut v, false, (1.0, 1e9), &|_, _| None, &|_, _| None);
        }
        assert!(v.roll.abs() < 1e-5, "levelled: {}", v.roll);
        // "aims" off: heading and tilt kept.
        assert_eq!((v.heading, v.tilt), (0.0, 0.0));
        assert_eq!(p, [0.0, 1000.0, 0.0]);
    }

    #[test]
    fn a_cut_goes_to_the_spot_and_aims_at_once() {
        let mut v = View { tilt: 0.5, heading: 3.0, roll: 0.0, locked: false };
        let c = cut([0.0; 3], [0.0, 2000.0, -1400.0], 0.1, &mut v, true, true, (1000.0, 5000.0), &|_, _| None, &|_, _| None);
        assert_eq!((c.position, c.pulled_from), ([0.0; 3], None));
        assert!(v.locked && v.heading.abs() < 1e-6 && (v.tilt - AIM_TILT).abs() < 1e-5 && v.roll == 0.1, "{v:?}");
        // Both switches off: on the spot, heading toward (x, y), tilt kept.
        let mut v = View { tilt: 0.5, heading: 3.0, roll: 0.0, locked: false };
        let c = cut([0.0; 3], [1000.0, 0.0, 0.0], 0.0, &mut v, false, false, (1000.0, 5000.0), &|_, _| Some(0.0), &|_, _| None);
        assert_eq!((c.position, c.height), ([0.0; 3], None));
        assert!((v.heading + std::f32::consts::FRAC_PI_2).abs() < 1e-5 && v.tilt == 0.5, "{v:?}");
    }
}
