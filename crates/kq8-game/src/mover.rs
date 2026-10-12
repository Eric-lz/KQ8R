//! Connor's mover: steps, gravity, slopes, landing and the Fall state.
//!
//! Rules from the behaviour note `docs/behaviour/connor-physics.md`
//! ("One update of the mover", "The terrain in the passes", "Gravity",
//! "Slopes", "Falling off an edge", "Is gravity on at the start?"). M2
//! steps 2a and 2c: only the ground (terrain) is solid; there are no walls,
//! ceilings or objects yet (2d).

/// The ground under a point: height and unit normal (z > 0), or `None`
/// where there is no ground (outside the terrain grid).
pub trait GroundLookup {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])>;
}

impl GroundLookup for kq8_data::Terrain {
    fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])> {
        self.height_at(x, y).map(|g| (g.height, g.normal))
    }
}

/// Step up and step down for Connor (monster/Connor default, traced).
pub const STEP: f32 = 325.0;
/// A surface is walkable if its normal's z is above this (about 36.9°).
pub const WALKABLE_NORMAL_Z: f32 = 0.8;
/// Gravity in units per second squared (with the fall timer's seconds):
/// the drop `GRAVITY × (t² − t_prev²) / 2` is the note's measured
/// `10000 × (t² − t_prev²)`. The traced value was 5,000; the original
/// falls 4 times as far (measured 2026-10-12, Q30).
pub const GRAVITY: f32 = 20000.0;
/// The fall timer grows by this much per update, whatever the real time.
pub const FALL_TIMER_STEP: f32 = 0.06;
/// At most this much drop in one update.
pub const MAX_DROP: f32 = 5000.0;
/// A pull less steep than this below horizontal (sin 30°) gives no drop
/// and puts the fall timer back to 0.
pub const MIN_PULL_DOWN: f32 = 0.5;
/// Updates in the air before the Fall state starts.
pub const AIR_UPDATES_TO_FALL: u32 = 5;
/// The fall timer when the Fall state starts **after a jump** (previous
/// 0.30). Not used yet: jumps come in step 2e. From walking or standing,
/// Fall leaves the timer running on.
pub const FALL_START_TIMER: f32 = 0.36;
/// In Fall, dropping less than this in one update counts as "stuck".
pub const STUCK_DROP: f32 = 50.0;
/// More than this many stuck updates ends the fall like a landing.
pub const STUCK_UPDATES: u32 = 5;
/// Falling further than this kills Connor.
pub const DEATH_DROP: f32 = 3840.0;
/// At most this many push-offs (slides along a surface) in one update.
pub const MAX_PUSH_OFFS: u32 = 3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum State {
    /// On the ground or moving normally (the original's Wait/Move).
    Normal,
    /// The Fall state: `start_z` is the height when it started.
    Fall { start_z: f32, stuck: u32 },
    /// Died from a fall; the mover no longer updates.
    Dead,
}

impl State {
    pub fn name(&self) -> &'static str {
        match self {
            State::Normal => "normal",
            State::Fall { .. } => "fall",
            State::Dead => "dead",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Mover {
    pub position: [f32; 3],
    pub on_ground: bool,
    pub use_gravity: bool,
    pub keep_on_ground: bool,
    /// Seconds of falling so far (reset when the down pass lands him on
    /// walkable ground).
    pub fall_timer: f32,
    /// Updates spent in the air with gravity on.
    pub air_updates: u32,
    /// Direction gravity pulls: straight down, or down a slope.
    pub pull: [f32; 3],
    pub step_up: f32,
    pub step_down: f32,
    pub state: State,
}

/// What the down pass ended on.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub enum DownResult {
    /// Nothing: he is in the air (or the ground check catches him).
    #[default]
    None,
    /// Landed on walkable terrain.
    Walkable,
    /// Hit steep terrain and was pushed off down the slope.
    Steep,
}

impl DownResult {
    pub fn name(&self) -> &'static str {
        match self {
            DownResult::None => "none",
            DownResult::Walkable => "walkable",
            DownResult::Steep => "steep",
        }
    }
}

/// A change of the Fall state, for the animations.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FallEvent {
    Started,
    /// Safe landing after falling this far.
    Landed(f32),
    /// Fatal landing after falling this far.
    Died(f32),
}

/// What one update did, for the log.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub drop: f32,
    /// He began the update on walkable ground (logged only).
    pub began_on_ground: bool,
    /// The up pass hit the terrain: its normal.
    pub up_contact: Option<[f32; 3]>,
    pub down: DownResult,
    /// Normal of the down pass's contact.
    pub down_normal: Option<[f32; 3]>,
    /// How far the push-off moved him along the surface.
    pub push: f32,
    /// What the ground check did: `above`, `walkable`, `steep` or `none`
    /// (no ground, or keepOnGround off).
    pub check: &'static str,
    /// Ground height and normal under the final position, if any.
    pub ground: Option<(f32, [f32; 3])>,
    pub fall: Option<FallEvent>,
    /// Things worth a line in the main log (state changes, landings).
    pub events: Vec<String>,
}

impl Mover {
    /// Connor as created: on the ground at `position`, gravity as given.
    pub fn new(position: [f32; 3], use_gravity: bool) -> Self {
        Mover {
            position,
            on_ground: true,
            use_gravity,
            keep_on_ground: true,
            fall_timer: 0.0,
            air_updates: 0,
            pull: [0.0, 0.0, -1.0],
            step_up: STEP,
            step_down: STEP,
            state: State::Normal,
        }
    }

    /// Test helper: lift Connor by `height` (a dead Connor comes back to
    /// life, so a fall can be tried again). "On ground" is kept: the update
    /// before the lift ended where it ended, so the first update after it
    /// counts in the air only if that one ended off the ground (Q29).
    pub fn lift(&mut self, height: f32) {
        self.position[2] += height;
        self.state = State::Normal;
    }

    /// One update (1/15 s) with a fixed wanted movement (x, y, z change);
    /// zero while standing.
    pub fn update(&mut self, wanted: [f32; 3], ground: &dyn GroundLookup) -> Report {
        self.update_with(ground, |_, _| wanted)
    }

    /// One update (1/15 s). `wanted` gives the movement the animation asks
    /// for. It is called after the start of the update, so it already knows
    /// whether Fall started this update (then the fall animation's step is
    /// used, not the walk's). Not called when Connor is dead.
    pub fn update_with(
        &mut self,
        ground: &dyn GroundLookup,
        wanted: impl FnOnce(&Mover, Option<FallEvent>) -> [f32; 3],
    ) -> Report {
        let mut report = Report { check: "none", ..Default::default() };
        if self.state == State::Dead {
            report.ground = ground.ground(self.position[0], self.position[1]);
            return report;
        }
        let before_z = self.position[2];
        report.began_on_ground = self.on_ground;

        // 1. Start of the update.
        if self.on_ground || !self.use_gravity {
            self.on_ground = false;
            self.pull = [0.0, 0.0, -1.0];
            self.air_updates = 0;
        } else {
            self.air_updates += 1;
            // From 5 on, every update asks for Fall; normal play accepts.
            // Not after a jump (none yet), so gravity is left alone.
            if self.air_updates >= AIR_UPDATES_TO_FALL && self.state == State::Normal {
                self.state = State::Fall { start_z: self.position[2], stuck: 0 };
                report.fall = Some(FallEvent::Started);
                report.events.push(format!(
                    "fall starts at z={:.0} after {} updates in the air (fall timer runs on from {:.2} s)",
                    self.position[2], self.air_updates, self.fall_timer
                ));
            }
        }

        let wanted = wanted(self, report.fall);

        // 2. Up pass: step up plus the wanted height change. The terrain
        //    counts as a hit if it is above where the pass ends; he stops
        //    at it (no ceilings yet).
        self.position[2] += self.step_up + wanted[2];
        if let Some((height, normal)) = ground.ground(self.position[0], self.position[1]) {
            if height > self.position[2] {
                self.position[2] = height;
                report.up_contact = Some(normal);
            }
        }

        // 3. Across pass: the terrain is not tested (Q26); no walls yet.
        self.position[0] += wanted[0];
        self.position[1] += wanted[1];

        // 4. Down pass: this update's gravity drop along the pull, then
        //    step down.
        if self.use_gravity {
            if -self.pull[2] < MIN_PULL_DOWN {
                self.fall_timer = 0.0;
            } else {
                let t_prev = self.fall_timer;
                self.fall_timer += FALL_TIMER_STEP;
                let t = self.fall_timer;
                report.drop = (GRAVITY * (t * t - t_prev * t_prev) / 2.0).min(MAX_DROP);
            }
        }
        // The vertical drop projected onto the pull (Q28 a): d along a
        // straight-down pull, d·sinθ along a slope of angle θ.
        let start = self.position;
        let mut end = start;
        let along = report.drop * -self.pull[2];
        for (e, d) in end.iter_mut().zip(self.pull) {
            *e += d * along;
        }
        end[2] -= self.step_down;
        self.position = end;
        match ground.ground(end[0], end[1]) {
            // The terrain is above where the pass ended: a hit there.
            Some((height, normal)) if height > end[2] => {
                let contact = [end[0], end[1], height];
                report.down_normal = Some(normal);
                self.pull = down_slope(normal);
                if normal[2] > WALKABLE_NORMAL_Z {
                    report.down = DownResult::Walkable;
                    self.position = contact;
                    self.on_ground = true;
                    self.fall_timer = 0.0;
                } else {
                    report.down = DownResult::Steep;
                    if !self.use_gravity {
                        report.events.push("steep ground: gravity switched on".into());
                    }
                    self.use_gravity = true;
                    // Push-off: the rest of the move, projected onto the
                    // surface, done as an across move (terrain ignored).
                    // With no walls it never hits anything, so one try of
                    // the MAX_PUSH_OFFS is enough.
                    let rest: [f32; 3] = std::array::from_fn(|i| end[i] - contact[i]);
                    let along = slide_along(rest, normal);
                    report.push = length(along);
                    self.position = std::array::from_fn(|i| contact[i] + along[i]);
                }
            }
            // Not hit: he is in the air. Going down a steep walkable slope
            // that is often for an update or a few; the gravity brings him
            // back before the air count reaches 5 (measured, Q32).
            _ => {}
        }

        // 5. Ground check (keepOnGround): at or below the ground -> on it.
        //    Only the height changes; the fall timer is left alone.
        let under = ground.ground(self.position[0], self.position[1]);
        report.ground = under;
        if let (true, Some((height, normal))) = (self.keep_on_ground, under) {
            if self.position[2] <= height {
                self.position[2] = height;
                if normal[2] > WALKABLE_NORMAL_Z {
                    report.check = "walkable";
                    self.on_ground = true;
                } else {
                    report.check = "steep";
                    self.on_ground = false;
                    self.pull = down_slope(normal);
                    if !self.use_gravity {
                        report.events.push("steep ground: gravity switched on".into());
                    }
                    self.use_gravity = true;
                }
            } else {
                report.check = "above";
            }
        }

        // The Fall state (connor-physics.md "Falling off an edge").
        if let State::Fall { start_z, stuck } = self.state {
            let z = self.position[2];
            let stuck = if before_z - z < STUCK_DROP { stuck + 1 } else { 0 };
            let ended = if self.on_ground {
                Some("landed")
            } else if stuck > STUCK_UPDATES {
                Some("stuck for more than 5 updates (counts as landed)")
            } else if z > before_z {
                Some("moved up")
            } else {
                None
            };
            match ended {
                Some(why) => {
                    let fallen = start_z - z;
                    if fallen > DEATH_DROP {
                        self.state = State::Dead;
                        report.fall = Some(FallEvent::Died(fallen));
                        report.events.push(format!("fall ends ({why}) after {fallen:.0} units: more than {DEATH_DROP}, Connor dies"));
                    } else {
                        self.state = State::Normal;
                        report.fall = Some(FallEvent::Landed(fallen));
                        report.events.push(format!("fall ends ({why}) after {fallen:.0} units: safe landing"));
                    }
                }
                None => self.state = State::Fall { start_z, stuck },
            }
        }
        report
    }
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// `v` with the part along `normal` removed: what is left of a move after
/// hitting a surface.
fn slide_along(v: [f32; 3], normal: [f32; 3]) -> [f32; 3] {
    let d = dot(v, normal);
    std::array::from_fn(|i| v[i] - d * normal[i])
}

/// The direction straight down, flattened onto a surface with `normal`.
fn down_slope(normal: [f32; 3]) -> [f32; 3] {
    let d = slide_along([0.0, 0.0, -1.0], normal);
    let len = length(d);
    if len < 1e-6 { [0.0, 0.0, -1.0] } else { d.map(|v| v / len) }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Flat ground at a fixed height everywhere.
    struct Flat(f32);
    impl GroundLookup for Flat {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((self.0, [0.0, 0.0, 1.0]))
        }
    }

    /// Flat at height 0 for x < 0; from x = 0 a plane rising (positive
    /// angle) or falling (negative) towards +x at `angle` radians.
    struct Ramp(f32);
    impl GroundLookup for Ramp {
        fn ground(&self, x: f32, _: f32) -> Option<(f32, [f32; 3])> {
            if x < 0.0 {
                Some((0.0, [0.0, 0.0, 1.0]))
            } else {
                let (s, c) = self.0.sin_cos();
                Some((x * self.0.tan(), [-s, 0.0, c]))
            }
        }
    }

    /// Height 0 for x < 0, `-drop` from x = 0 on.
    struct Ledge(f32);
    impl GroundLookup for Ledge {
        fn ground(&self, x: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((if x < 0.0 { 0.0 } else { -self.0 }, [0.0, 0.0, 1.0]))
        }
    }

    /// Updates until the state settles; returns (updates, events).
    fn run(m: &mut Mover, ground: &dyn GroundLookup, max: usize) -> (usize, Vec<String>) {
        let mut events = Vec::new();
        for n in 1..=max {
            let r = m.update([0.0; 3], ground);
            events.extend(r.events);
            if m.on_ground && m.state != State::Normal || m.state == State::Dead || (m.on_ground && n > 1) {
                return (n, events);
            }
        }
        (max, events)
    }

    #[test]
    fn standing_on_flat_ground_stays_put() {
        let mut m = Mover::new([0.0, 0.0, 100.0], true);
        for _ in 0..30 {
            let r = m.update([0.0; 3], &Flat(100.0));
            assert!(r.events.is_empty());
            assert_eq!(r.down, DownResult::Walkable);
            assert_eq!(m.position[2], 100.0);
            assert!(m.on_ground);
            assert_eq!((m.fall_timer, m.air_updates), (0.0, 0));
        }
    }

    #[test]
    fn gravity_drop_follows_the_fall_timer() {
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        m.lift(100_000.0);
        let mut drops = Vec::new();
        for _ in 0..3 {
            drops.push(m.update([0.0; 3], &Flat(0.0)).drop);
        }
        // t = 0.06, 0.12, 0.18: 10000 × (t² − t_prev²).
        let want = [36.0, 108.0, 180.0];
        for (d, w) in drops.iter().zip(want) {
            assert!((d - w).abs() < 0.05, "{drops:?}");
        }
        // Up 325 and down 325 cancel: only gravity moves him.
        assert!((m.position[2] - (100_000.0 - 324.0)).abs() < 0.1);
    }

    #[test]
    fn short_fall_is_safe_long_fall_kills() {
        let mut safe = Mover::new([0.0, 0.0, 0.0], true);
        safe.lift(2000.0);
        let (_, events) = run(&mut safe, &Flat(0.0), 200);
        assert_eq!(safe.state, State::Normal, "{events:?}");
        assert!(events.iter().any(|e| e.contains("safe landing")), "{events:?}");
        assert_eq!(safe.position[2], 0.0);

        let mut dead = Mover::new([0.0, 0.0, 0.0], true);
        dead.lift(5000.0);
        let (_, events) = run(&mut dead, &Flat(0.0), 200);
        assert_eq!(dead.state, State::Dead, "{events:?}");
        assert!(events.iter().any(|e| e.contains("Connor dies")), "{events:?}");
    }

    #[test]
    fn fall_starts_after_5_updates_in_the_air() {
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        m.lift(50_000.0);
        let mut first_fall = None;
        for n in 1..=10 {
            let r = m.update([0.0; 3], &Flat(0.0));
            if r.fall == Some(FallEvent::Started) {
                assert!(first_fall.is_none(), "asked for once, accepted once");
                first_fall = Some(n);
            }
        }
        // Updates 1..5 end in the air; Fall is asked for at the start of
        // update 6 (Q29: after 5 updates in a row ending off the ground).
        assert_eq!(first_fall, Some(6));
    }

    /// Walking off a ledge: the drops carry on through the Fall start
    /// (36 … 324 in the air, then 396, 468 in Fall), with no head start.
    #[test]
    fn walking_off_a_ledge_keeps_the_fall_timer() {
        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        let mut drops = Vec::new();
        let mut fall_at = None;
        for n in 1..=12 {
            let r = m.update_with(&Ledge(50_000.0), |m, _| if m.state == State::Normal { [112.0, 0.0, 0.0] } else { [0.0; 3] });
            if r.fall == Some(FallEvent::Started) {
                fall_at = Some(n);
            }
            if r.drop > 0.0 && !m.on_ground {
                drops.push(r.drop);
            }
        }
        assert!(fall_at.is_some(), "{drops:?}");
        let want = [36.0, 108.0, 180.0, 252.0, 324.0, 396.0, 468.0];
        for (d, w) in drops.iter().zip(want) {
            assert!((d - w).abs() < 0.1, "{drops:?}");
        }
    }

    #[test]
    fn down_slope_points_downhill() {
        // A slope rising towards +x: normal leans to -x.
        let n = [-0.6, 0.0, 0.8];
        let d = down_slope(n);
        assert!(d[0] < 0.0 && d[2] < 0.0, "{d:?}");
        assert!(dot(d, n).abs() < 1e-6, "lies in the surface");
    }

    /// Running (168 per update along +x) at a 45° hill from its foot: each
    /// update follows the note's rule (Q28 a), 168·cos²θ − d·sinθ·cosθ
    /// with d = 36, 108, 180, 252, 324, so he gains 66, then 30, then
    /// slides back. In update 5 the push-off carries him past the foot and
    /// the ground check puts him on the flat: only 4 updates ended in the
    /// air, so no Fall.
    #[test]
    fn running_into_a_steep_hill_slides_back() {
        let hill = Ramp(45f32.to_radians());
        let mut m = Mover::new([0.0, 0.0, 0.0], true);
        let mut xs = vec![m.position[0]];
        let mut events = Vec::new();
        for n in 1..=20 {
            let r = m.update_with(&hill, |m, _| if m.state == State::Normal { [168.0, 0.0, 0.0] } else { [0.0; 3] });
            events.extend(r.events);
            assert_ne!(r.fall, Some(FallEvent::Started), "update {n}: {events:?}");
            if r.down == DownResult::Steep {
                assert!(r.push > 0.0);
                // Only the ground check over the flat puts him on the ground.
                assert_eq!(m.on_ground, r.check == "walkable", "update {n}");
            }
            xs.push(m.position[0]);
            if m.on_ground && m.position[0] < 0.0 {
                break;
            }
        }
        let gains: Vec<f32> = xs.windows(2).map(|w| w[1] - w[0]).collect();
        let want = [66.0, 30.0, -6.0, -42.0];
        assert!(gains.len() >= want.len(), "{gains:?}");
        for (g, w) in gains.iter().zip(want) {
            assert!((g - w).abs() < 0.5, "{gains:?}");
        }
        // Back on the flat, safe, after 5 updates.
        assert_eq!(gains.len(), 5, "{xs:?}");
        assert!(m.position[0] < 0.0 && m.on_ground && m.position[2] == 0.0, "{xs:?}");
        assert_eq!(m.state, State::Normal, "{events:?}");
    }

    /// Walking (75 per update) and running (168) down walkable planes,
    /// even the steepest: off the ground for a few updates at a time, but
    /// never 5 in a row, so never Fall (the user's check in the original;
    /// measured there: off 1–4 updates at a time, Q32).
    #[test]
    fn running_down_walkable_slopes_never_falls() {
        for step in [75.0f32, 168.0] {
            for deg in [5.0f32, 20.0, 30.0, 36.0] {
                let slope = Ramp(-deg.to_radians());
                let mut m = Mover::new([0.0, 0.0, 0.0], true);
                let mut most_air = 0;
                for n in 0..100 {
                    m.update([step, 0.0, 0.0], &slope);
                    most_air = most_air.max(m.air_updates);
                    assert_eq!(m.state, State::Normal, "{step}, {deg}°, update {n}");
                }
                assert!(most_air < AIR_UPDATES_TO_FALL, "{step}, {deg}°: {most_air}");
                assert!(m.position[2] < -1000.0 * (step / 168.0), "{step}, {deg}°: he went down");
            }
        }
    }

    #[test]
    fn small_ledge_is_followed_big_ledge_kills() {
        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        for _ in 0..5 {
            m.update([112.0, 0.0, 0.0], &Ledge(300.0));
        }
        assert!(m.on_ground && m.position[2] == -300.0 && m.state == State::Normal);

        let mut m = Mover::new([-100.0, 0.0, 0.0], true);
        let mut events = Vec::new();
        for _ in 0..100 {
            let r = m.update_with(&Ledge(5000.0), |m, _| if m.state == State::Normal { [112.0, 0.0, 0.0] } else { [0.0; 3] });
            events.extend(r.events);
        }
        assert_eq!(m.state, State::Dead, "{events:?}");
        assert!(events.iter().any(|e| e.contains("fall starts")), "{events:?}");
    }

    #[test]
    fn ground_check_leaves_the_fall_timer_alone() {
        // Gravity off, in the air, ending exactly at ground height: the
        // down pass doesn't hit (not above), the ground check puts him on.
        let mut m = Mover::new([0.0, 0.0, 0.0], false);
        m.on_ground = false;
        m.fall_timer = 0.5;
        let r = m.update([0.0; 3], &Flat(0.0));
        assert_eq!((r.down, r.check), (DownResult::None, "walkable"));
        assert!(m.on_ground);
        assert_eq!(m.fall_timer, 0.5);
    }

    #[test]
    fn up_pass_stops_at_the_terrain() {
        let mut m = Mover::new([0.0, 0.0, -1000.0], true);
        let r = m.update([0.0; 3], &Flat(0.0));
        assert!(r.up_contact.is_some());
        assert_eq!(r.down, DownResult::Walkable);
        assert_eq!(m.position[2], 0.0);
    }
}
