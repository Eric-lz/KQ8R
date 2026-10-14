//! Emitters: a point that keeps a fixed number of small particles flying
//! (falling leaves, drips). Rules from `docs/behaviour/emitters.md`
//! "Emitter rules" (session A, traced); the numbers are read by
//! `kq8_data::effects::EmitterSettings`.
//!
//! Each particle's choices (start offset, velocity, spin, sway speeds) are
//! made once, so it repeats exactly the same path every life (rule 2); only
//! the mix of starting ages makes the stream look random.
//!
//! A particle catcher (`Catcher`) hides one emitter's particles inside a
//! building's box (emitters.md "Particle catchers").
//!
//! Following an object (rule 11): `Emitter::follow_to` before each tick.
//!
//! Not done here: dying (rule 10), the script commands (open question 4). `randomZRot` has no effect in the
//! original and is left out ("Do not reproduce").

use kq8_data::effects::EmitterSettings;

/// "Non-zero" for spin, sway and gravity: absolute value at least this
/// (emitters.md "Numbers and units").
pub const NONZERO: f32 = 0.000001;

/// The sway factor as stored in the game: 6.28, not exactly 2π (rule 6).
#[allow(clippy::approx_constant)]
pub const SWAY_FACTOR: f32 = 6.28;

fn nonzero(v: f32) -> bool {
    v.abs() >= NONZERO
}

/// A small seeded random number generator (SplitMix64). The original uses
/// the C library's generator; matching it is not needed, uniform fractions
/// are enough (emitters.md "Do not reproduce").
#[derive(Debug, Clone)]
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Self {
        Rng(seed)
    }

    /// A new generator seeded from this one (for an emitter's own draws).
    pub fn fork(&mut self) -> Rng {
        Rng::new(u64::from(self.fraction().to_bits()) << 32 | u64::from(self.fraction().to_bits()))
    }

    /// A uniform fraction in [0, 1).
    pub fn fraction(&mut self) -> f32 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^= z >> 31;
        // 24 bits: every value is exact in an f32 and below 1.
        (z >> 40) as f32 / (1u64 << 24) as f32
    }

    /// Uniform between `a` and `b` (either order).
    pub fn between(&mut self, a: f32, b: f32) -> f32 {
        a + self.fraction() * (b - a)
    }
}

/// One particle: its fixed choices and where it is now.
#[derive(Debug, Clone, PartialEq)]
pub struct Particle {
    /// Start offset from the emitter (rule 1), chosen once.
    pub offset: [f32; 3],
    /// Velocity, units per tick, chosen once.
    pub velocity: [f32; 3],
    /// Spin, radians per tick, if the emitter spins its particles.
    pub spin: Option<[f32; 3]>,
    /// Sway speed per axis, radians per tick (0 on axes without sway).
    pub sway_speed: [f32; 3],
    /// Sway angle per axis; starts at 0, never reset (rule 3.6).
    pub sway_angle: [f32; 3],
    /// Ticks into the current life.
    pub age: u32,
    pub position: [f32; 3],
    /// Turn, used like an object's `dirX/Y/Z` (the note's assumption);
    /// never reset.
    pub angles: [f32; 3],
    /// Hidden by `delayStart` until the first restart, or by a catcher
    /// until the next restart (rule 4).
    pub visible: bool,
    /// Restarted in the last tick (drawing doesn't blend across it).
    pub restarted: bool,
}

impl Particle {
    /// On screen: visible and age not 0 (rule 4).
    pub fn drawn(&self) -> bool {
        self.visible && self.age != 0
    }
}

/// An emitter and its particles.
#[derive(Debug, Clone)]
pub struct Emitter {
    pub name: String,
    /// `loc`: where particles start (plus their offset).
    pub loc: [f32; 3],
    pub settings: EmitterSettings,
    pub particles: Vec<Particle>,
    /// Ticks run so far.
    pub ticks: u64,
    /// Following an object (rule 11): its turn now (angles like
    /// `dirX/Y/Z`), and the generator for `followMesh` restarts.
    pub following: Option<Following>,
}

/// What a following emitter knows about its target.
#[derive(Debug, Clone)]
pub struct Following {
    pub angles: [f32; 3],
    pub rng: Rng,
}

/// Where a following emitter is: the target's position + `followOffset`,
/// the offset in the target's own axes unless its x and y are both 0, then
/// in world axes (rule 11).
pub fn follow_loc(position: [f32; 3], angles: [f32; 3], offset: [f32; 3]) -> [f32; 3] {
    let turned = if offset[0] == 0.0 && offset[1] == 0.0 { offset } else { turn(offset, angles) };
    std::array::from_fn(|k| position[k] + turned[k])
}

/// A vector turned like an object with `angles` (row vector × rotation).
fn turn(v: [f32; 3], angles: [f32; 3]) -> [f32; 3] {
    let r = kq8_data::objects::rotation(angles);
    std::array::from_fn(|j| (0..3).map(|k| v[k] * r[k][j]).sum())
}

/// A random velocity within the limits (rule 1).
fn random_velocity(s: &EmitterSettings, rng: &mut Rng) -> [f32; 3] {
    std::array::from_fn(|k| rng.between(s.force_min[k], s.force_max[k]))
}

impl Emitter {
    /// The emitter entering the world: `maxParticles` particles, each with
    /// its random choices, spread along their paths by their starting age
    /// (rule 1).
    pub fn new(name: &str, loc: [f32; 3], settings: EmitterSettings, rng: &mut Rng) -> Self {
        Self::make(name, loc, settings, rng, None)
    }

    /// A following emitter entering the world (rule 11): placed at the
    /// target, its particles' velocities turned with the target's turn.
    pub fn new_following(name: &str, target: [f32; 3], angles: [f32; 3], settings: EmitterSettings, rng: &mut Rng) -> Self {
        let loc = follow_loc(target, angles, settings.follow_offset);
        let own = rng.fork();
        Self::make(name, loc, settings, rng, Some(Following { angles, rng: own }))
    }

    fn make(name: &str, loc: [f32; 3], settings: EmitterSettings, rng: &mut Rng, following: Option<Following>) -> Self {
        let s = &settings;
        let angles = following.as_ref().map(|f| f.angles);
        let spins = s.rot_min.iter().chain(&s.rot_max).any(|v| nonzero(*v));
        let particles = (0..s.max_particles)
            .map(|_| {
                let offset: [f32; 3] = std::array::from_fn(|k| rng.between(-s.radius[k], s.radius[k]));
                let mut velocity = random_velocity(s, rng);
                // Following: the velocity limits are turned with the target.
                if let Some(a) = angles {
                    velocity = turn(velocity, a);
                }
                let spin = spins.then(|| std::array::from_fn(|k| rng.between(s.rot_min[k], s.rot_max[k])));
                // Rule 6: random fraction × P × 6.28 ÷ maxLife radians per tick.
                let sway_speed: [f32; 3] = std::array::from_fn(|k| {
                    let p = s.max_sin_period[k];
                    if nonzero(p) && s.max_life > 0 {
                        rng.fraction() * p * SWAY_FACTOR / s.max_life as f32
                    } else {
                        0.0
                    }
                });
                // A whole number from 0 to maxLife: fraction × maxLife, rounded down.
                let age = (rng.fraction() * s.max_life as f32) as u32;
                // Moved on by age × velocity; no gravity or sway in this jump.
                let position = std::array::from_fn(|k| loc[k] + offset[k] + age as f32 * velocity[k]);
                Particle {
                    offset,
                    velocity,
                    spin,
                    sway_speed,
                    sway_angle: [0.0; 3],
                    age,
                    position,
                    angles: [0.0; 3],
                    visible: !s.delay_start,
                    restarted: false,
                }
            })
            .collect();
        Emitter { name: name.to_string(), loc, settings, particles, ticks: 0, following }
    }

    /// A following emitter moves with its target (call before `tick`):
    /// new `loc` and turn. Particles already flying stay where they are.
    pub fn follow_to(&mut self, target: [f32; 3], angles: [f32; 3]) {
        self.loc = follow_loc(target, angles, self.settings.follow_offset);
        if let Some(f) = &mut self.following {
            f.angles = angles;
        }
    }

    /// One tick (rule 3), particles in list order. No collision test:
    /// particles fly through the ground and walls (rule 5).
    pub fn tick(&mut self) {
        self.ticks += 1;
        let s = &self.settings;
        let gravity = s.gravity.iter().any(|g| nonzero(*g)).then_some(s.gravity);
        // `followMesh`: a restart takes the target's turn and a new velocity.
        let mut mesh = self.following.as_mut().filter(|_| s.follow_mesh);
        for p in &mut self.particles {
            p.restarted = false;
            // 1. age + 1
            p.age += 1;
            // 2. sway angles + sway speeds
            for k in 0..3 {
                p.sway_angle[k] += p.sway_speed[k];
            }
            // 3. position + velocity, or velocity × sin(angle) on a swaying axis
            for k in 0..3 {
                let step = if p.sway_angle[k] != 0.0 { p.velocity[k] * p.sway_angle[k].sin() } else { p.velocity[k] };
                p.position[k] += step;
            }
            // 4. spin: the same small turn every tick
            if let Some(spin) = p.spin {
                for (a, s) in p.angles.iter_mut().zip(spin) {
                    *a += s;
                }
            }
            // 5. gravity: age × gravity (rule 7)
            if let Some(g) = gravity {
                for (x, g) in p.position.iter_mut().zip(g) {
                    *x += p.age as f32 * g;
                }
            }
            // 6. past its life: restart at loc + its own offset, visible again.
            // Turn and sway angle are not reset.
            if p.age > s.max_life {
                p.age = 0;
                p.position = std::array::from_fn(|k| self.loc[k] + p.offset[k]);
                p.visible = true;
                p.restarted = true;
                if let Some(f) = mesh.as_mut() {
                    p.angles = f.angles;
                    p.velocity = turn(random_velocity(s, &mut f.rng), f.angles);
                }
            }
        }
    }
}

/// A particle catcher: hides its emitter's particles that enter a
/// building's box, until their next restart (emitters.md "Particle
/// catchers" rules 3–5).
#[derive(Debug, Clone)]
pub struct Catcher {
    pub name: String,
    /// The building's bounding box in its own space.
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// The building's placement.
    pub placement: crate::collide::Placement,
    /// The box's top z + the building's z.
    pub top: f32,
}

impl Catcher {
    pub fn new(name: &str, min: [f32; 3], max: [f32; 3], placement: crate::collide::Placement) -> Self {
        let top = max[2] + placement.position[2];
        Catcher { name: name.to_string(), min, max, placement, top }
    }

    /// One tick (rule 4): each particle at or below the top is turned into
    /// the building's own axes and hidden if min ≤ value < max on all three
    /// axes. Returns the particles it hid now (they were visible before).
    pub fn catch(&self, emitter: &mut Emitter) -> Vec<usize> {
        let mut hid = Vec::new();
        for (k, p) in emitter.particles.iter_mut().enumerate() {
            if p.position[2] > self.top {
                continue;
            }
            let own = self.placement.to_own(p.position);
            if (0..3).all(|a| self.min[a] <= own[a] && own[a] < self.max[a]) {
                if p.visible {
                    hid.push(k);
                }
                p.visible = false;
            }
        }
        hid
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn settings() -> EmitterSettings {
        EmitterSettings {
            shape: Some("bit.kqs".into()),
            radius: [0.0; 3],
            force_min: [0.0; 3],
            force_max: [0.0; 3],
            gravity: [0.0; 3],
            rot_min: [0.0; 3],
            rot_max: [0.0; 3],
            max_sin_period: [0.0; 3],
            max_life: 10,
            max_particles: 1,
            delay_start: false,
            follow: None,
            follow_mesh: false,
            follow_offset: [0.0; 3],
            problems: Vec::new(),
        }
    }

    /// Ticks until particle 0 restarts.
    fn to_restart(e: &mut Emitter) -> u32 {
        let mut n = 0;
        loop {
            e.tick();
            n += 1;
            if e.particles[0].restarted {
                return n;
            }
            assert!(n < 10_000);
        }
    }

    #[test]
    fn random_fractions_stay_in_range() {
        let mut r = Rng::new(7);
        let v: Vec<f32> = (0..10_000).map(|_| r.fraction()).collect();
        assert!(v.iter().all(|f| (0.0..1.0).contains(f)));
        let mean = v.iter().sum::<f32>() / v.len() as f32;
        assert!((mean - 0.5).abs() < 0.02, "mean {mean}");
    }

    #[test]
    fn choices_lie_within_their_limits() {
        let mut s = settings();
        s.max_particles = 500;
        s.radius = [30.0, 30.0, 0.0];
        s.force_min = [0.0, 10.0, -30.0];
        s.force_max = [200.0, 20.0, -30.0];
        s.rot_min = [-0.1, 0.0, 0.0];
        s.rot_max = [0.1, 0.0, 0.0];
        s.max_life = 150;
        let e = Emitter::new("e", [1000.0, 2000.0, 3000.0], s, &mut Rng::new(1));
        assert_eq!(e.particles.len(), 500);
        for p in &e.particles {
            assert!(p.offset[0].abs() <= 30.0 && p.offset[1].abs() <= 30.0 && p.offset[2] == 0.0);
            assert!((0.0..=200.0).contains(&p.velocity[0]) && (10.0..=20.0).contains(&p.velocity[1]));
            assert_eq!(p.velocity[2], -30.0);
            let spin = p.spin.unwrap();
            assert!(spin[0].abs() <= 0.1 && spin[1] == 0.0 && spin[2] == 0.0);
            assert!(p.age <= 150);
            // Moved on by age × velocity from loc + offset.
            for k in 0..3 {
                let want = [1000.0, 2000.0, 3000.0][k] + p.offset[k] + p.age as f32 * p.velocity[k];
                assert!((p.position[k] - want).abs() < 0.01);
            }
            assert!(p.visible);
        }
    }

    #[test]
    fn no_spin_when_all_rot_limits_are_zero() {
        let e = Emitter::new("e", [0.0; 3], settings(), &mut Rng::new(1));
        assert!(e.particles[0].spin.is_none());
    }

    #[test]
    fn a_life_is_max_life_plus_one_ticks_with_one_hidden() {
        let mut s = settings();
        s.force_min = [1.0, 0.0, 0.0];
        s.force_max = [1.0, 0.0, 0.0];
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(3));
        to_restart(&mut e);
        // From a restart to the next: maxLife + 1 = 11 ticks, drawn in 10.
        let mut drawn = 0;
        for n in 1..=11 {
            e.tick();
            drawn += usize::from(e.particles[0].drawn());
            assert_eq!(e.particles[0].restarted, n == 11, "tick {n}");
        }
        assert_eq!(drawn, 10);
        assert!(!e.particles[0].drawn());
        assert_eq!(e.particles[0].position, [0.0; 3]);
    }

    #[test]
    fn every_life_repeats_the_same_path() {
        let mut s = settings();
        s.max_particles = 3;
        s.max_life = 20;
        s.radius = [50.0; 3];
        s.force_min = [-5.0, 1.0, 10.0];
        s.force_max = [5.0, 3.0, 12.0];
        s.gravity = [0.0, 0.0, -1.0];
        s.max_sin_period = [3.0, 0.0, 0.0];
        let mut e = Emitter::new("e", [100.0, 200.0, 300.0], s, &mut Rng::new(9));
        to_restart(&mut e);
        let life = |e: &mut Emitter| (0..21).map(|_| { e.tick(); e.particles[0].position }).collect::<Vec<_>>();
        let first = life(&mut e);
        let second = life(&mut e);
        // The sway angle is not reset, so x drifts differently each life;
        // y and z (no sway) repeat exactly.
        for (a, b) in first.iter().zip(&second) {
            assert_eq!((a[1], a[2]), (b[1], b[2]));
        }
    }

    #[test]
    fn gravity_adds_up_to_n_n_plus_1_over_2() {
        let mut s = settings();
        s.max_life = 40;
        s.force_min = [0.0, 0.0, 50.0];
        s.force_max = [0.0, 0.0, 50.0];
        s.gravity = [0.0, 0.0, -5.0];
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(5));
        to_restart(&mut e);
        for n in 1..=40u32 {
            e.tick();
            let n_f = n as f32;
            let want = 50.0 * n_f - 5.0 * n_f * (n_f + 1.0) / 2.0;
            assert_eq!(e.particles[0].position[2], want, "tick {n}");
        }
        // stream1-like drop: up about 50 per tick at first, back down
        // below its start before the life ends.
        assert!(e.particles[0].position[2] < 0.0);
    }

    #[test]
    fn swaying_steps_are_velocity_times_sine() {
        let mut s = settings();
        s.max_life = 100;
        s.force_min = [10.0, 10.0, 0.0];
        s.force_max = [10.0, 10.0, 0.0];
        s.max_sin_period = [3.0, 0.0, 0.0];
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(11));
        let speed = e.particles[0].sway_speed[0];
        assert!(speed > 0.0 && speed <= 3.0 * SWAY_FACTOR / 100.0);
        assert_eq!(e.particles[0].sway_speed[1], 0.0);
        for n in 1..=5 {
            let before = e.particles[0].position;
            e.tick();
            if e.particles[0].restarted {
                break;
            }
            let after = e.particles[0].position;
            let angle = speed * n as f32;
            assert!((after[0] - before[0] - 10.0 * angle.sin()).abs() < 1e-3);
            assert!((after[1] - before[1] - 10.0).abs() < 1e-3);
        }
    }

    #[test]
    fn delay_start_hides_until_the_first_restart_and_spins_keep_turning() {
        let mut s = settings();
        s.delay_start = true;
        s.max_particles = 20;
        s.rot_min = [0.0, 0.0, 0.1];
        s.rot_max = [0.0, 0.0, 0.1];
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(2));
        assert!(e.particles.iter().all(|p| !p.visible));
        let mut seen_restart = [false; 20];
        for _ in 0..11 {
            e.tick();
            for (k, p) in e.particles.iter().enumerate() {
                seen_restart[k] |= p.restarted;
                assert_eq!(p.visible, seen_restart[k]);
            }
        }
        // Every particle has restarted once within maxLife + 1 ticks.
        assert!(seen_restart.iter().all(|r| *r));
        // 11 ticks of 0.1: the turn is not reset at restarts.
        assert!((e.particles[0].angles[2] - 1.1).abs() < 1e-5);
    }

    #[test]
    fn a_catcher_hides_particles_inside_the_turned_box_until_their_restart() {
        // A house turned a quarter turn, its box 0..100 x 0..50 x 0..40 in its own space.
        let quarter = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::FRAC_PI_2]);
        let house = crate::collide::Placement { rotation: quarter, position: [1000.0, 0.0, 10.0] };
        let c = Catcher::new("c", [0.0, 0.0, 0.0], [100.0, 50.0, 40.0], house);
        assert_eq!(c.top, 50.0);
        // Falling straight down from above the house: 10 per tick.
        let mut s = settings();
        s.max_life = 20;
        s.force_min = [0.0, 0.0, -10.0];
        s.force_max = [0.0, 0.0, -10.0];
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(1));
        to_restart(&mut e);
        // Own (20, 30) is world (1000 - 30, 20) after the quarter turn.
        e.loc = [970.0, 20.0, 100.0];
        e.particles[0].position = e.loc;
        let mut hidden_at = None;
        for n in 1..=21 {
            e.tick();
            let hid = c.catch(&mut e);
            if !hid.is_empty() {
                hidden_at.get_or_insert((n, e.particles[0].position[2]));
            }
        }
        // Hidden on the first tick it is below the roof (own z 40 = world 50).
        assert_eq!(hidden_at, Some((6, 40.0)));
        // The restart (tick 21) made it visible again.
        assert!(e.particles[0].restarted && e.particles[0].visible);
        // Outside the box: never hidden.
        e.loc = [1100.0, 20.0, 100.0];
        e.particles[0].position = e.loc;
        for _ in 0..21 {
            e.tick();
            assert!(c.catch(&mut e).is_empty());
        }
    }

    #[test]
    fn a_following_emitter_sits_at_its_target_plus_the_offset() {
        let quarter = [0.0, 0.0, std::f32::consts::FRAC_PI_2];
        // x and y of the offset 0: world axes, whatever the turn.
        let up = follow_loc([10.0, 20.0, 30.0], quarter, [0.0, 0.0, 4000.0]);
        assert_eq!(up, [10.0, 20.0, 4030.0]);
        // Otherwise the target's own axes: own +x is world +y after a quarter turn.
        let side = follow_loc([10.0, 20.0, 30.0], quarter, [100.0, 0.0, 0.0]);
        assert!((side[0] - 10.0).abs() < 1e-3 && (side[1] - 120.0).abs() < 1e-3 && side[2] == 30.0);
    }

    #[test]
    fn following_turns_the_velocity_and_restarts_at_the_moved_spot() {
        let mut s = settings();
        s.force_min = [100.0, 0.0, 0.0];
        s.force_max = [100.0, 0.0, 0.0];
        s.follow = Some("Connor".into());
        let quarter = [0.0, 0.0, std::f32::consts::FRAC_PI_2];
        let mut e = Emitter::new_following("e", [0.0; 3], quarter, s, &mut Rng::new(1));
        // Own +x (100) became world +y.
        let v = e.particles[0].velocity;
        assert!(v[0].abs() < 1e-3 && (v[1] - 100.0).abs() < 1e-3);
        // The target moves; the next restart is at the new spot.
        for _ in 0..30 {
            e.follow_to([5000.0, 0.0, 0.0], quarter);
            e.tick();
            if e.particles[0].restarted {
                break;
            }
        }
        assert!(e.particles[0].restarted);
        assert_eq!(e.particles[0].position, [5000.0, 0.0, 0.0]);
        // Without followMesh the velocity is kept (rule 2).
        assert!((e.particles[0].velocity[1] - 100.0).abs() < 1e-3);
    }

    #[test]
    fn follow_mesh_restarts_take_the_targets_turn_and_a_new_velocity() {
        let mut s = settings();
        s.force_min = [100.0, 0.0, 0.0];
        s.force_max = [100.0, 0.0, 0.0];
        s.follow = Some("head".into());
        s.follow_mesh = true;
        let mut e = Emitter::new_following("e", [0.0; 3], [0.0; 3], s, &mut Rng::new(1));
        let half = [0.0, 0.0, std::f32::consts::PI];
        for _ in 0..30 {
            e.follow_to([0.0; 3], half);
            e.tick();
            if e.particles[0].restarted {
                break;
            }
        }
        let p = &e.particles[0];
        assert!(p.restarted);
        assert_eq!(p.angles, half);
        // Turned half round: own +x is now world -x.
        assert!((p.velocity[0] + 100.0).abs() < 1e-3);
    }

    #[test]
    fn zero_particles_do_nothing() {
        let mut s = settings();
        s.max_particles = 0;
        let mut e = Emitter::new("e", [0.0; 3], s, &mut Rng::new(1));
        e.tick();
        assert!(e.particles.is_empty());
    }
}
