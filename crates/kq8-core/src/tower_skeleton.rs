//! Dead City's tower skeleton (`KQTowerSkeleton`), batch 2 step T. Rules
//! from `docs/behaviour/fighting.md` section 7a (session A, traced).
//!
//! T-1: its own values, and Cue 201 (its alarm radius becomes
//! `realAlarm`). Its standing, moving and falling rules come in T-2 and
//! T-3.

/// Cue 201 (7a): the alarm radius becomes `realAlarm`.
pub const CUE_REAL_ALARM: &str = "201";
/// `stunPause` and `reactPause` when the file gives none (7a "Keys").
pub const DEFAULT_STUN_PAUSE: f32 = 3.0;
pub const DEFAULT_REACT_PAUSE: f32 = 2.0;
/// `timeToRun` is set this long after it enters the world (kept only).
pub const TIME_TO_RUN_AFTER: f64 = 5.0;

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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn defaults_and_the_data() {
        // skeltowr.kq: realAlarm 5000, stunPause 1.5, reactPause 1.2.
        let t = TowerSkeleton::new(Some(5000.0), Some(1.5), Some(1.2), 0.0);
        assert_eq!((t.real_alarm, t.stun_pause, t.react_pause, t.move_time, t.time_to_run), (5000.0, 1.5, 1.2, 0.0, 5.0));
        let d = TowerSkeleton::new(None, None, None, 2.0);
        assert_eq!((d.real_alarm, d.stun_pause, d.react_pause, d.time_to_run), (0.0, 3.0, 2.0, 7.0));
    }
}
