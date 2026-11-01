//! Dead City's tower skeleton (`KQTowerSkeleton`), batch 2 step T. Rules
//! from `docs/behaviour/fighting.md` section 7a (session A, traced).
//!
//! T-1: its own values, and Cue 201 (its alarm radius becomes
//! `realAlarm`). T-2: its standing and moving rules
//! (`character_ai.rs`, `tower_rules`) and its flinch's end (rule 23).
//! T-3: its fall off the tower, a short cutscene (7a "Dying"; stages run
//! by `active.rs`, `tower_fall_next`).

/// Cue 201 (7a): the alarm radius becomes `realAlarm`.
pub const CUE_REAL_ALARM: &str = "201";
/// `stunPause` and `reactPause` when the file gives none (7a "Keys").
pub const DEFAULT_STUN_PAUSE: f32 = 3.0;
pub const DEFAULT_REACT_PAUSE: f32 = 2.0;
/// `timeToRun` is set this long after it enters the world (kept only).
pub const TIME_TO_RUN_AFTER: f64 = 5.0;
/// 7a: the ground probes: ahead before and while moving after Connor,
/// behind (the way `reactback` staggers it) during and after `reactback`;
/// each looks for ground within `PROBE_DEPTH` below its feet.
pub const PROBE_AHEAD: f32 = 1000.0;
pub const PROBE_BEHIND: f32 = 400.0;
pub const PROBE_DEPTH: f32 = 1000.0;
/// Ours: the probe line starts this far above its feet, so a floor right
/// at its feet counts (the note gives only the depth).
pub const PROBE_START: f32 = 100.0;
/// 7a: its stagger.
pub const STAGGER: &str = "reactback";
/// 7a "Dying": its cues (the fall's two, the greeting's: T-4).
pub const CUE_FALL: &str = "50";
/// Step 2: the first shot, 2,000 behind it and 5,000 below, looking at the
/// point 0.9 of the way (our reading: from the camera to it); Cue 50 2.18 s
/// later.
pub const SHOT1_BEHIND: f32 = 2000.0;
pub const SHOT1_BELOW: f32 = 5000.0;
pub const SHOT1_LOOK_SHARE: f32 = 0.9;
pub const CUE50_AFTER_TEETER: f64 = 2.18;
/// Step 3: the ground point 2,000 behind it; the camera 3,000 to its side
/// and 2,000 up from it, looking at it; it is moved 600 back, about 5,000
/// above that ground (the note: "not sure").
pub const GROUND_BEHIND: f32 = 2000.0;
pub const SHOT2_SIDE: f32 = 3000.0;
pub const SHOT2_UP: f32 = 2000.0;
pub const DIE2_BACK: f32 = 600.0;
pub const DIE2_HIGH: f32 = 5000.0;
/// Step 4: put on the ground 1,400 behind where it was.
pub const LAND_BACK: f32 = 1400.0;
/// Step 5: Cue 50 3 s after `fallend`.
pub const CUE50_AFTER_FALLEND: f64 = 3.0;
/// The script the dropped item gets at the end.
pub const DROP_SCRIPT: &str = "chainmal.cs";
/// T-4a (7a): Connor's state numbers it reacts to (`characters.md` rule
/// 16): the rope, and a weapon swap.
pub const CONNOR_ROPE_STATE: &str = "4096";
pub const CONNOR_SWAP_STATE: &str = "32";
/// Its lines (2000.msg noun, verb, case): the greeting, the ranged weapon.
pub const GREETING: (u8, u8, u8) = (16, 0, 18);
pub const RANGED_LINE: (u8, u8, u8) = (16, 0, 102);
/// The music the greeting starts.
pub const MUSIC: &str = "mtower.aud";

/// Where its fall is (7a "Dying"): the movement playing, or the wait for
/// the second Cue 50.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FallStage {
    Teeter,
    Die,
    Die2,
    Fallend,
    Waiting,
}

impl FallStage {
    pub fn movement(self) -> Option<&'static str> {
        match self {
            FallStage::Teeter => Some("teeter"),
            FallStage::Die => Some("die"),
            FallStage::Die2 => Some("die2"),
            FallStage::Fallend => Some("fallend"),
            FallStage::Waiting => None,
        }
    }
}

/// Its own axes in the world: (behind = its +Y, side = its +X), level and
/// of unit length (row vectors of its rotation).
pub fn own_axes(r: &[[f32; 3]; 3]) -> ([f32; 3], [f32; 3]) {
    let level = |v: [f32; 3]| {
        let l = (v[0] * v[0] + v[1] * v[1]).sqrt().max(1e-6);
        [v[0] / l, v[1] / l, 0.0]
    };
    (level(r[1]), level(r[0]))
}

/// The `move KQCamera` tilt and heading that look from `from` to `to`
/// (`camera.md`: the view is (−sin h × cos t, cos h × cos t, sin t)).
pub fn look_angles(from: [f32; 3], to: [f32; 3]) -> (f32, f32) {
    let d = [to[0] - from[0], to[1] - from[1], to[2] - from[2]];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt().max(1e-6);
    ((d[2] / len).asin(), (-d[0]).atan2(d[1]))
}

/// The tower skeleton's own values (7a "Keys").
#[derive(Debug, Clone, PartialEq)]
pub struct TowerSkeleton {
    /// `realAlarm`: its alarm radius from Cue 201 on (0 if not given).
    pub real_alarm: f32,
    pub stun_pause: f32,
    pub react_pause: f32,
    /// `moveTime`: a time of day (game seconds); it doesn't start moving
    /// after Connor before it. Starts at 0.
    pub move_time: f64,
    /// `timeToRun`: a time, never used (kept for saved games).
    pub time_to_run: f64,
    /// Rule 23 (T-2): its `reactback` ended; the probe behind it decides
    /// (falls, or back to standing) in the next AI step.
    pub probe_due: bool,
    /// T-3: its fall, once started; the ground height 2,000 behind it
    /// where it began; the Cue 50s seen; the item it dropped.
    pub fall: Option<FallStage>,
    pub fall_ground: f32,
    pub cue50s: u8,
    pub dropped: Option<String>,
    /// T-4a: `bGreetedConnor` (the greeting is said once), and the
    /// greeting's line is being said (controls off until it ends).
    pub greeted: bool,
    pub greeting: bool,
}

impl TowerSkeleton {
    /// As it enters the world at game time `now`.
    pub fn new(real_alarm: Option<f32>, stun_pause: Option<f32>, react_pause: Option<f32>, now: f64) -> Self {
        TowerSkeleton {
            real_alarm: real_alarm.unwrap_or(0.0),
            stun_pause: stun_pause.unwrap_or(DEFAULT_STUN_PAUSE),
            react_pause: react_pause.unwrap_or(DEFAULT_REACT_PAUSE),
            move_time: 0.0,
            time_to_run: now + TIME_TO_RUN_AFTER,
            probe_due: false,
            fall: None,
            fall_ground: 0.0,
            cue50s: 0,
            dropped: None,
            greeted: false,
            greeting: false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn camera_angles_and_own_axes() {
        // Looking along +Y level: heading 0, tilt 0; along -X: heading pi/2.
        let (t, h) = look_angles([0.0; 3], [0.0, 10.0, 0.0]);
        assert!(t.abs() < 1e-6 && h.abs() < 1e-6);
        let (_, h) = look_angles([0.0; 3], [-10.0, 0.0, 0.0]);
        assert!((h - std::f32::consts::FRAC_PI_2).abs() < 1e-5);
        let (t, _) = look_angles([0.0; 3], [0.0, 10.0, 10.0]);
        assert!((t - std::f32::consts::FRAC_PI_4).abs() < 1e-5);
        // Not turned: behind = +Y, side = +X.
        let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
        assert_eq!(own_axes(&id), ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0]));
    }

    #[test]
    fn defaults_and_the_data() {
        // skeltowr.kq: realAlarm 5000, stunPause 1.5, reactPause 1.2.
        let t = TowerSkeleton::new(Some(5000.0), Some(1.5), Some(1.2), 0.0);
        assert_eq!((t.real_alarm, t.stun_pause, t.react_pause, t.move_time, t.time_to_run), (5000.0, 1.5, 1.2, 0.0, 5.0));
        let d = TowerSkeleton::new(None, None, None, 2.0);
        assert_eq!((d.real_alarm, d.stun_pause, d.react_pause, d.time_to_run), (0.0, 3.0, 2.0, 7.0));
    }
}
