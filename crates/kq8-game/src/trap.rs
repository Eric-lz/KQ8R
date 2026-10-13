//! Traps (`KQTrap`): invisible boxes that run their script when Connor
//! walks into them or out of them (M3 step 3d-1).
//!
//! Rules from `docs/behaviour/traps.md` (traced):
//!
//! - The box is `BBMin`/`BBMax` added to the trap's position, never turned
//!   (rule 1). A trap is never solid (rule 5).
//! - Who counts: the trap's mask (`colliderMask`) has a bit of the
//!   object's type (rule 4, as rule 16 uses it; see the stand-in below).
//! - Entering: the object's collision shape touches the box. The first one
//!   in gives `On <name>`, any later one `Cue 301 <name>` (rule 6).
//! - Staying: the centre of the object's box is in the trap's box (edges
//!   count), or its shape still touches it (rule 7). Leaving: both fail;
//!   the last one out gives `Off <name>`, others `Cue 302 <name>` (rule 8).
//! - Sticky (`trapType` bit 2): the box is shrunk to `innerPercent` (all
//!   six numbers, so towards the trap's position) while nobody is inside,
//!   full size while someone is (rule 10). One-shot (bit 1): gone after
//!   its `On` (rule 11).
//!
//! Stand-ins (docs/DESIGN.md M3 3d-1): "touches" is one of the object's
//! balls (the same balls as for walls, `collide.rs`) reaching the box,
//! tested where the object ends its move, not along the way (traps.md
//! open question 2); a trap without `colliderMask` has mask 0; a trap
//! without `innerPercent` keeps its full box. The object's box centre is
//! not turned with the object (Connor's box is the same all round).
//!
//! Stand-in, asked as Q58: rule 4 says the usual pair rule (either side's
//! mask covers the other's type), but Connor's own mask 0x281B has the
//! trap type 0x10, so then every trap would see him whatever its mask,
//! and rule 16 (scripts remove bit 4 from a trap's mask so Connor no
//! longer counts) could not work. We use only the trap's side.

use crate::collide::BallStack;

/// A trap's own collision type (rule 4).
pub const TRAP_TYPE: u32 = 0x10;
/// `trapType` bits (rules 10 and 11).
pub const ONE_SHOT: u32 = 1;
pub const STICKY: u32 = 2;
/// Cue numbers for a second object coming in or going out (rules 6, 8).
pub const CUE_ENTER: &str = "301";
pub const CUE_LEAVE: &str = "302";

/// A trap as read from its `.kq` section.
#[derive(Debug, Clone, PartialEq)]
pub struct TrapSettings {
    pub name: String,
    /// `loc`, with a `locZ` of −1 already replaced by the floor height.
    pub position: [f32; 3],
    /// `BBMinX/Y/Z`, `BBMaxX/Y/Z` (missing ones 0), in the trap's own space.
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub script: Option<String>,
    pub mask: u32,
    pub trap_type: u32,
    /// `innerPercent` (a share, e.g. 0.9), `None` if not given.
    pub inner_percent: Option<f32>,
}

/// A trap in the world.
#[derive(Debug, Clone, PartialEq)]
pub struct Trap {
    pub settings: TrapSettings,
    /// Who is inside now, in the order they came in.
    pub inside: Vec<String>,
    /// The box is shrunk to `innerPercent` (sticky traps with nobody in).
    pub shrunk: bool,
    /// A one-shot trap that has fired: it no longer exists.
    pub gone: bool,
}

/// An object that has just moved, as a trap sees it.
#[derive(Debug, Clone)]
pub struct Visitor<'a> {
    pub name: &'a str,
    pub collision_type: u32,
    pub position: [f32; 3],
    /// Its `BOUNDS.KQ` box, own space.
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub balls: &'a BallStack,
}

/// What one check found, for the log, and the script call it leads to.
#[derive(Debug, Clone, PartialEq)]
pub struct Check {
    /// The pair rule lets the trap see this object.
    pub counts: bool,
    pub touching: bool,
    pub centre_inside: bool,
    /// The box tested (world space), before any change this check made.
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// The script's arguments after `$1` (the trap's name): `On <who>`,
    /// `Off <who>`, `Cue 301|302 <who>`; `None` = no call.
    pub call: Option<Vec<String>>,
}

impl Trap {
    /// The trap as it enters the world: sticky ones start shrunk (rule 10).
    pub fn new(settings: TrapSettings) -> Self {
        let shrunk = settings.trap_type & STICKY != 0 && settings.inner_percent.is_some();
        Trap { settings, inside: Vec::new(), shrunk, gone: false }
    }

    /// The box in use now, world space.
    pub fn current_box(&self) -> ([f32; 3], [f32; 3]) {
        let s = &self.settings;
        let k = if self.shrunk { s.inner_percent.unwrap_or(1.0) } else { 1.0 };
        let p = s.position;
        (std::array::from_fn(|i| p[i] + s.min[i] * k), std::array::from_fn(|i| p[i] + s.max[i] * k))
    }

    /// The full (never shrunk) box, world space.
    pub fn full_box(&self) -> ([f32; 3], [f32; 3]) {
        let s = &self.settings;
        let p = s.position;
        (std::array::from_fn(|i| p[i] + s.min[i]), std::array::from_fn(|i| p[i] + s.max[i]))
    }

    /// Whether this trap sees an object of that type: its mask has a bit
    /// of the type (stand-in, Q58; the object's own mask is not used).
    pub fn counts(&self, collision_type: u32) -> bool {
        self.settings.mask & collision_type != 0
    }

    /// `v` has moved: does it come in, stay, or leave (rules 6–11)?
    pub fn check(&mut self, v: &Visitor) -> Check {
        let (min, max) = self.current_box();
        let counts = !self.gone && self.counts(v.collision_type);
        let touching = v.balls.heights.iter().any(|h| {
            let c = [v.position[0], v.position[1], v.position[2] + h];
            distance_squared_to_box(c, min, max) <= v.balls.radius * v.balls.radius
        });
        let centre: [f32; 3] = std::array::from_fn(|i| v.position[i] + (v.min[i] + v.max[i]) / 2.0);
        let centre_inside = (0..3).all(|i| centre[i] >= min[i] && centre[i] <= max[i]);
        let mut out = Check { counts, touching, centre_inside, min, max, call: None };
        if !counts {
            return out;
        }
        let was_inside = self.inside.iter().any(|n| n.eq_ignore_ascii_case(v.name));
        let sticky = self.settings.trap_type & STICKY != 0 && self.settings.inner_percent.is_some();
        if !was_inside && touching {
            self.inside.push(v.name.to_string());
            out.call = Some(if self.inside.len() == 1 {
                vec!["On".into(), v.name.into()]
            } else {
                vec!["Cue".into(), CUE_ENTER.into(), v.name.into()]
            });
            if sticky {
                self.shrunk = false;
            }
            if self.settings.trap_type & ONE_SHOT != 0 && self.inside.len() == 1 {
                self.gone = true;
            }
        } else if was_inside && !centre_inside && !touching {
            self.inside.retain(|n| !n.eq_ignore_ascii_case(v.name));
            out.call = Some(if self.inside.is_empty() {
                vec!["Off".into(), v.name.into()]
            } else {
                vec!["Cue".into(), CUE_LEAVE.into(), v.name.into()]
            });
            if sticky && self.inside.is_empty() {
                self.shrunk = true;
            }
        }
        out
    }
}

/// Squared distance from `p` to the box (0 inside).
fn distance_squared_to_box(p: [f32; 3], min: [f32; 3], max: [f32; 3]) -> f32 {
    (0..3).map(|i| (min[i] - p[i]).max(0.0).max(p[i] - max[i])).map(|d| d * d).sum()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::collide::CONNOR_TYPE;

    /// Connor's box and balls (`BOUNDS.KQ` `Connor`: ±200, 0–1,300;
    /// balls of 200 at 200, 600, 1,000 and 1,100).
    const MIN: [f32; 3] = [-200.0, -200.0, 0.0];
    const MAX: [f32; 3] = [200.0, 200.0, 1300.0];

    fn trap(trap_type: u32, inner: Option<f32>) -> Trap {
        Trap::new(TrapSettings {
            name: "walktp1".into(),
            position: [10_000.0, 20_000.0, 500.0],
            min: [-300.0, -300.0, 0.0],
            max: [300.0, 300.0, 1000.0],
            script: Some("walk_la.cs".into()),
            mask: 4,
            trap_type,
            inner_percent: inner,
        })
    }

    /// Connor at x (y and z level with the trap).
    fn at<'a>(name: &'a str, x: f32, balls: &'a BallStack) -> Visitor<'a> {
        Visitor { name, collision_type: CONNOR_TYPE, position: [x, 20_000.0, 500.0], min: MIN, max: MAX, balls }
    }

    fn call(c: &Check) -> Option<String> {
        c.call.as_ref().map(|v| v.join(" "))
    }

    #[test]
    fn on_when_touching_off_only_when_out_of_touch_and_centre_outside() {
        let balls = BallStack::from_box(MIN, MAX);
        let mut t = trap(0, None);
        // Box x 9,700–10,300; his balls (radius 200) touch from x 9,500.
        assert_eq!(call(&t.check(&at("Connor", 9_400.0, &balls))), None);
        let c = t.check(&at("Connor", 9_500.0, &balls));
        assert!(c.touching && !c.centre_inside);
        assert_eq!(call(&c).as_deref(), Some("On Connor"));
        // Touching again while inside: nothing.
        assert_eq!(call(&t.check(&at("Connor", 9_800.0, &balls))), None);
        // Centre in or touching: still inside.
        assert_eq!(call(&t.check(&at("Connor", 10_450.0, &balls))), None);
        // Out of touch (centre outside): Off.
        assert_eq!(call(&t.check(&at("Connor", 10_501.0, &balls))).as_deref(), Some("Off Connor"));
        assert!(t.inside.is_empty());
    }

    #[test]
    fn centre_inside_keeps_him_in_without_touching() {
        // A shape that doesn't reach its box centre: one small ball (r 10)
        // 10 above his feet; his box centre is 650 above them. Trap box z
        // 500–1,500.
        let balls = BallStack { radius: 10.0, heights: vec![10.0] };
        let mut t = trap(0, None);
        let v = |z: f32| Visitor { name: "Connor", collision_type: CONNOR_TYPE, position: [10_000.0, 20_000.0, z], min: MIN, max: MAX, balls: &balls };
        assert_eq!(call(&t.check(&v(500.0))).as_deref(), Some("On Connor"));
        // Feet at 0: the ball (z 10) is far below the box, the centre
        // (z 650) is in it: still inside.
        let c = t.check(&v(0.0));
        assert!(!c.touching && c.centre_inside, "{c:?}");
        assert_eq!(call(&c), None);
        // Feet at −200: centre z 450, under the box: out.
        let c = t.check(&v(-200.0));
        assert!(!c.touching && !c.centre_inside, "{c:?}");
        assert_eq!(call(&c).as_deref(), Some("Off Connor"));
    }

    #[test]
    fn sticky_trap_starts_on_the_inner_box_and_ends_on_the_full_one() {
        let balls = BallStack::from_box(MIN, MAX);
        let mut t = trap(STICKY, Some(0.9));
        assert!(t.shrunk);
        // Shrunk towards its position: x ±270 (not towards the box middle).
        assert_eq!(t.current_box().0, [9_730.0, 19_730.0, 500.0]);
        assert_eq!(t.current_box().1, [10_270.0, 20_270.0, 1_400.0]);
        // Full box would be touched at 9,500, the inner only from 9,530.
        assert_eq!(call(&t.check(&at("Connor", 9_510.0, &balls))), None);
        assert_eq!(call(&t.check(&at("Connor", 9_530.0, &balls))).as_deref(), Some("On Connor"));
        assert!(!t.shrunk);
        // Back out past the inner edge but still touching the full box.
        assert_eq!(call(&t.check(&at("Connor", 9_510.0, &balls))), None);
        assert_eq!(call(&t.check(&at("Connor", 9_499.0, &balls))).as_deref(), Some("Off Connor"));
        assert!(t.shrunk);
    }

    #[test]
    fn a_second_one_in_or_out_gives_cues() {
        let balls = BallStack::from_box(MIN, MAX);
        let mut t = trap(0, None);
        assert_eq!(call(&t.check(&at("Connor", 10_000.0, &balls))).as_deref(), Some("On Connor"));
        assert_eq!(call(&t.check(&at("Other", 10_000.0, &balls))).as_deref(), Some("Cue 301 Other"));
        assert_eq!(call(&t.check(&at("Connor", 12_000.0, &balls))).as_deref(), Some("Cue 302 Connor"));
        assert_eq!(call(&t.check(&at("Other", 12_000.0, &balls))).as_deref(), Some("Off Other"));
    }

    #[test]
    fn the_pair_rule_decides_who_counts() {
        let balls = BallStack::from_box(MIN, MAX);
        let mut t = trap(0, None);
        // A monster's type (0xC021) has no bit of the trap's mask 4.
        let mut v = at("Monster", 10_000.0, &balls);
        v.collision_type = 0xC021;
        let c = t.check(&v);
        assert!(!c.counts && c.touching && c.call.is_none());
        // Mask 0 (or bit 4 removed, rule 16): not even Connor, although
        // his own mask 0x281B has the trap type 0x10 (Q58).
        t.settings.mask = 0;
        let c = t.check(&at("Connor", 10_000.0, &balls));
        assert!(!c.counts && c.call.is_none());
    }

    #[test]
    fn a_one_shot_trap_fires_once() {
        let balls = BallStack::from_box(MIN, MAX);
        let mut t = trap(ONE_SHOT, None);
        assert_eq!(call(&t.check(&at("Connor", 10_000.0, &balls))).as_deref(), Some("On Connor"));
        assert!(t.gone);
        assert_eq!(call(&t.check(&at("Connor", 12_000.0, &balls))), None);
        assert_eq!(call(&t.check(&at("Connor", 10_000.0, &balls))), None);
    }
}
