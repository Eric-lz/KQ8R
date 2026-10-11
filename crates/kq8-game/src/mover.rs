//! Connor's mover: steps, gravity, landing and the Fall state.
//!
//! Rules from the behaviour note `docs/behaviour/connor-physics.md`
//! ("One update of the mover", "Gravity", "Falling off an edge", "Is
//! gravity on at the start?"). M2 step 2a: only the ground (terrain) is
//! solid; there are no walls, ceilings or objects yet, and no wanted
//! movement apart from what the caller passes.

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
/// Gravity in units per second squared.
pub const GRAVITY: f32 = 5000.0;
/// The fall timer grows by this much per update, whatever the real time.
pub const FALL_TIMER_STEP: f32 = 0.06;
/// At most this much drop in one update.
pub const MAX_DROP: f32 = 5000.0;
/// Updates in the air before the Fall state starts.
pub const AIR_UPDATES_TO_FALL: u32 = 5;
/// The fall timer when the Fall state starts (as if already falling).
pub const FALL_START_TIMER: f32 = 0.36;
/// In Fall, dropping less than this in one update counts as "stuck".
pub const STUCK_DROP: f32 = 50.0;
/// More than this many stuck updates ends the fall like a landing.
pub const STUCK_UPDATES: u32 = 5;
/// Falling further than this kills Connor.
pub const DEATH_DROP: f32 = 3840.0;

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
    /// Seconds of falling so far (reset on walkable ground).
    pub fall_timer: f32,
    /// Updates spent in the air with gravity on.
    pub air_updates: u32,
    /// Direction gravity pulls: straight down, or down a steep slope.
    pub pull: [f32; 3],
    pub step_up: f32,
    pub step_down: f32,
    pub state: State,
}

/// What one update did, for the log.
#[derive(Debug, Clone, Default)]
pub struct Report {
    pub drop: f32,
    /// Ground height and normal under the final position, if any.
    pub ground: Option<(f32, [f32; 3])>,
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

    /// Test helper: lift Connor by `height`, in the air (a dead Connor
    /// comes back to life, so a fall can be tried again).
    pub fn lift(&mut self, height: f32) {
        self.position[2] += height;
        self.on_ground = false;
        self.state = State::Normal;
    }

    /// One update (1/15 s). `wanted` is the movement the animation asks
    /// for (x, y, z change); zero while standing.
    pub fn update(&mut self, wanted: [f32; 3], ground: &dyn GroundLookup) -> Report {
        let mut report = Report::default();
        if self.state == State::Dead {
            report.ground = ground.ground(self.position[0], self.position[1]);
            return report;
        }
        let before_z = self.position[2];

        // 1. Start of the update.
        if self.on_ground || !self.use_gravity {
            self.on_ground = false;
            self.pull = [0.0, 0.0, -1.0];
            self.air_updates = 0;
        } else {
            self.air_updates += 1;
            if self.air_updates == AIR_UPDATES_TO_FALL && self.state == State::Normal {
                self.state = State::Fall { start_z: self.position[2], stuck: 0 };
                self.fall_timer = FALL_START_TIMER;
                report.events.push(format!("fall starts at z={:.0} (fall timer set to {FALL_START_TIMER} s)", self.position[2]));
            }
        }

        // 2. Up pass: step up plus the wanted height change (no ceilings yet).
        self.position[2] += self.step_up + wanted[2];
        // 3. Across pass (no walls yet).
        self.position[0] += wanted[0];
        self.position[1] += wanted[1];
        // 4. Down pass: this update's gravity drop along the pull, then
        //    step down (nothing solid but the ground yet).
        if self.use_gravity {
            let t_prev = self.fall_timer;
            self.fall_timer += FALL_TIMER_STEP;
            let t = self.fall_timer;
            report.drop = (GRAVITY * (t * t - t_prev * t_prev) / 2.0).min(MAX_DROP);
            for (p, d) in self.position.iter_mut().zip(self.pull) {
                *p += d * report.drop;
            }
        }
        self.position[2] -= self.step_down;

        // 5. Ground check (keepOnGround): at or below the ground -> on it.
        let under = ground.ground(self.position[0], self.position[1]);
        report.ground = under;
        if let (true, Some((height, normal))) = (self.keep_on_ground, under) {
            if self.position[2] <= height {
                self.position[2] = height;
                if normal[2] > WALKABLE_NORMAL_Z {
                    self.on_ground = true;
                    self.fall_timer = 0.0;
                } else {
                    // Steep: pulled down the slope, gravity on. (Sliding
                    // itself is M2 step 2c.)
                    self.on_ground = false;
                    self.pull = down_slope(normal);
                    if !self.use_gravity {
                        report.events.push("steep ground: gravity switched on".into());
                    }
                    self.use_gravity = true;
                    report.events.push(format!("on steep ground (normal z {:.3})", normal[2]));
                }
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
                        report.events.push(format!("fall ends ({why}) after {fallen:.0} units: more than {DEATH_DROP}, Connor dies"));
                    } else {
                        self.state = State::Normal;
                        report.events.push(format!("fall ends ({why}) after {fallen:.0} units: safe landing"));
                    }
                }
                None => self.state = State::Fall { start_z, stuck },
            }
        }
        report
    }
}

/// The direction straight down, flattened onto a surface with `normal`.
fn down_slope(normal: [f32; 3]) -> [f32; 3] {
    let [nx, ny, nz] = normal;
    let d = [nz * nx, nz * ny, nz * nz - 1.0];
    let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
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
        // t = 0.06, 0.12, 0.18: 5000 × (t² − t_prev²) / 2.
        let want = [9.0, 27.0, 45.0];
        for (d, w) in drops.iter().zip(want) {
            assert!((d - w).abs() < 0.01, "{drops:?}");
        }
        // Up 325 and down 325 cancel: only gravity moves him.
        assert!((m.position[2] - (100_000.0 - 81.0)).abs() < 0.1);
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
            m.update([0.0; 3], &Flat(0.0));
            if first_fall.is_none() && matches!(m.state, State::Fall { .. }) {
                first_fall = Some(n);
            }
        }
        // Lifted, so already off the ground: updates 1..5 count 1..5.
        assert_eq!(first_fall, Some(5));
    }

    #[test]
    fn down_slope_points_downhill() {
        // A slope rising towards +x: normal leans to -x.
        let n = [-0.6, 0.0, 0.8];
        let d = down_slope(n);
        assert!(d[0] < 0.0 && d[2] < 0.0, "{d:?}");
        assert!((d[0] * n[0] + d[1] * n[1] + d[2] * n[2]).abs() < 1e-6, "lies in the surface");
    }
}
