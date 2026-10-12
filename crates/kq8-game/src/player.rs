//! One update of Connor: his walker (`connor.rs`, the animation and the
//! step it asks for) and his mover (`mover.rs`, what happens to that step)
//! together, with what passes between them: falls, landings, head-on hits
//! and jumps.
//!
//! Rules: `connor-physics.md` "Falling off an edge", "Walls", "Jumping";
//! the user's checks of jumping in the original (docs/DESIGN.md M2 2e).

use crate::connor::{Intent, Mode, MovementLoop, Step, Walker};
use crate::mover::{FallEvent, Mover, MoverWorld, Report, State, Wish, JUMP_ABOVE_TERRAIN, WALKABLE_NORMAL_Z};

/// What one update did.
#[derive(Debug, Clone)]
pub struct Tick {
    /// The walker's update (none while dead and the mover stands still).
    pub step: Option<Step>,
    pub report: Report,
    /// A head-on hit changed the movement: (from, to).
    pub wall_stop: Option<(&'static str, &'static str)>,
    /// Jump lines for the log: asked for, started, refused (why), ended.
    pub notes: Vec<String>,
}

/// Whether a jump can start now, and which (connor-physics.md "Jumping"
/// rules 1 and 2, with the user's checks): the walker's side (`jump_kind`),
/// the jump animation loaded, then the ground: on the ground, or within 250
/// above the terrain (its normal then counts), and that ground walkable or
/// steep with its normal not pointing toward his back.
pub fn can_jump<'a>(
    mover: &Mover,
    walker: &Walker,
    world: &dyn MoverWorld,
    loops: &dyn Fn(&str) -> Option<&'a MovementLoop>,
) -> Result<crate::connor::JumpKind, String> {
    if mover.state != State::Normal {
        return Err(format!("mover state {}", mover.state.name()));
    }
    let kind = walker.jump_kind()?;
    if loops(kind.jump()).is_none() {
        return Err(format!("{} is not loaded yet", kind.jump()));
    }
    if mover.on_ground {
        return Ok(kind);
    }
    let [x, y, z] = mover.position;
    let Some((height, normal)) = world.ground(x, y) else {
        return Err("not on the ground and no terrain under him".into());
    };
    let above = z - height;
    if above > JUMP_ABOVE_TERRAIN {
        return Err(format!("{above:.0} above the terrain (more than {JUMP_ABOVE_TERRAIN})"));
    }
    if normal[2] > WALKABLE_NORMAL_Z {
        return Ok(kind);
    }
    // Facing (the model's −Y turned by the heading): (sin h, −cos h).
    let (s, c) = walker.heading.sin_cos();
    if normal[0] * s - normal[1] * c < 0.0 {
        return Err(format!("steep ground (normal z {:.3}) and facing uphill", normal[2]));
    }
    Ok(kind)
}

/// One update (1/15 s). `jump` = the Jump key was pressed since the last
/// update.
pub fn tick<'a>(
    mover: &mut Mover,
    walker: &mut Walker,
    intent: Intent,
    jump: bool,
    loops: &dyn Fn(&str) -> Option<&'a MovementLoop>,
    world: &dyn MoverWorld,
) -> Tick {
    let mut notes = Vec::new();
    // The jump animation ended last update: gravity on, Fall (rule 4).
    if walker.mode == Mode::Jump && walker.phase >= 1.0 {
        mover.end_jump();
        walker.set_mode(Mode::Fall);
        notes.push(format!(
            "{} ended at z={:.0}: gravity on, steps 325, Fall (death limit from z={:.0}, fall timer 0.36 after 0.30)",
            walker.kind.jump(),
            mover.position[2],
            mover.position[2] - crate::mover::JUMP_DEATH_OFFSET
        ));
    }
    if jump {
        match can_jump(mover, walker, world, loops) {
            Ok(kind) => {
                notes.push(format!(
                    "jump: {} from {} at ({:.0}, {:.0}, {:.0})",
                    kind.jump(),
                    walker.movement,
                    mover.position[0],
                    mover.position[1],
                    mover.position[2]
                ));
                mover.start_jump();
                walker.start_jump(kind);
            }
            Err(why) => notes.push(format!("jump refused: {why}")),
        }
    }
    let mut step = None;
    let report = if mover.state == State::Dead {
        // The mover stands still; the death animation plays on.
        step = Some(walker.update(intent, loops));
        mover.update([0.0; 3], world)
    } else {
        mover.update_with(world, |_, fall| {
            // Fall replaces the walk at once, so its first update already
            // uses the hold animation's step. A walk-off falls as `Stand`.
            if fall == Some(FallEvent::Started) {
                walker.kind = crate::connor::JumpKind::Stand;
                walker.set_mode(Mode::Fall);
            }
            let s = walker.update(intent, loops);
            let wish = Wish { step: s.wanted, moving: walker.moving() };
            step = Some(s);
            wish
        })
    };
    match report.fall {
        Some(FallEvent::Landed(_)) => walker.set_mode(Mode::Land),
        Some(FallEvent::Died(_)) => walker.set_mode(Mode::Dying),
        _ => {}
    }
    // Head-on into a wall: walking or stepping back stops at once; running
    // crashes from the next update.
    let wall_stop = if report.head_on { walker.head_on_at_wall() } else { None };
    Tick { step, report, wall_stop, notes }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connor::{JumpKind, Transition};
    use crate::mover::GroundLookup;

    /// Flat ground at height 0, or a plane rising along +y by `slope`.
    struct Plane(f32);

    impl GroundLookup for Plane {
        fn ground(&self, _: f32, y: f32) -> Option<(f32, [f32; 3])> {
            let n = [0.0, -self.0, 1.0];
            let len = (n[1] * n[1] + 1.0f32).sqrt();
            Some((y * self.0, n.map(|v| v / len)))
        }
    }

    /// A one-shot from per-frame (x, y, z) steps; frame 0's is zero.
    fn one_shot(steps: Vec<[f32; 3]>) -> MovementLoop {
        let mut frame_steps = vec![[0.0f32; 3]];
        frame_steps.extend(steps);
        let step_sum = frame_steps.iter().fold([0.0; 3], |a, s| [a[0] + s[0], a[1] + s[1], a[2] + s[2]]);
        MovementLoop {
            duration: (frame_steps.len() - 1) as f32 / 15.0,
            step_sum,
            speed: 1.0,
            repeat: false,
            frames: frame_steps.len(),
            frame_steps,
            ..Default::default()
        }
    }

    fn looping(frames: usize, step_y: f32, transitions: Vec<Transition>) -> MovementLoop {
        MovementLoop {
            duration: (frames - 1) as f32 / 15.0,
            step_sum: [0.0, step_y, 0.0],
            speed: 1.0,
            repeat: true,
            frames,
            cut_short: true,
            transitions,
            ..Default::default()
        }
    }

    /// A made-up standing jump with the note's numbers: 21 frames
    /// (1.333 s), the first 10 still, then 10 steps that add up to 1,423
    /// forward (−y) and +873 up, rising to 1,697 at the top.
    fn jump_arc() -> Vec<[f32; 3]> {
        let mut steps = vec![[0.0; 3]; 10];
        let ups = [400.0, 400.0, 400.0, 300.0, 197.0, -100.0, -150.0, -150.0, -200.0, -224.0];
        steps.extend(ups.iter().map(|&z| [0.0, -142.3, z]));
        steps
    }

    fn list() -> Vec<(&'static str, MovementLoop)> {
        let stopwalk = Transition { from: -1.0, to: "walk", to_phase: 0.85, via: "stopwalk" };
        vec![
            ("stop", looping(30, 0.0, vec![stopwalk])),
            ("walk", looping(20, -1427.1, Vec::new())),
            ("stopwalk", one_shot(vec![[7.32, -11.22, 0.0]; 7])),
            ("jump", one_shot(jump_arc())),
            ("jumphold", MovementLoop { repeat: true, ..one_shot(vec![[0.0; 3]; 2]) }),
            ("jumpend", one_shot(vec![[0.0; 3]; 10])),
            ("jumpdie", one_shot(vec![[0.0; 3]; 10])),
        ]
    }

    fn lookup<'a>(list: &'a [(&'static str, MovementLoop)]) -> impl Fn(&str) -> Option<&'a MovementLoop> + 'a {
        move |m| list.iter().find(|(n, _)| *n == m).map(|(_, l)| l)
    }

    const STAND: Intent = Intent { forward: 0, turn: 0, run: false };

    #[test]
    fn standing_jump_arc_then_fall_from_0_36_and_jumpend() {
        let list = list();
        let loops = lookup(&list);
        let ground = Plane(0.0);
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        tick(&mut m, &mut w, STAND, false, &loops, &ground);
        let t = tick(&mut m, &mut w, STAND, true, &loops, &ground);
        assert!(t.notes[0].starts_with("jump: jump from stop"), "{:?}", t.notes);
        let mut top: f32 = 0.0;
        // 20 updates of the jump (this one included), keys ignored.
        let mut jumping = 1;
        loop {
            top = top.max(m.position[2]);
            let t = tick(&mut m, &mut w, Intent { forward: 1, turn: 1, run: false }, false, &loops, &ground);
            if w.mode != Mode::Jump {
                assert!(t.notes[0].contains("ended at z=873"), "{:?}", t.notes);
                // The first Fall drop: 0.30 → 0.36 s.
                assert!((t.report.drop - 396.0).abs() < 0.5, "{:?}", t.report);
                break;
            }
            jumping += 1;
            assert_eq!(m.state, State::Jump);
            assert_eq!(w.heading, 0.0, "no turning while jumping");
        }
        assert_eq!(jumping, 20);
        assert!((top - 1697.0).abs() < 0.5, "{top}");
        assert!((m.position[1] + 1423.0).abs() < 0.5, "{:?}", m.position);
        // Falls 396, then lands (873 + 325 − 396 − 325 − 468 < 0), jumpend.
        let mut modes = vec![w.mode];
        for _ in 0..12 {
            tick(&mut m, &mut w, STAND, false, &loops, &ground);
            modes.push(w.mode);
        }
        assert_eq!(modes[..3], [Mode::Fall, Mode::Fall, Mode::Land], "{modes:?}");
        assert!(modes[3..].iter().all(|m| *m == Mode::Land || *m == Mode::Free), "{modes:?}");
        assert_eq!((m.state, m.position[2], m.use_gravity, m.step_up), (State::Normal, 0.0, true, 325.0));
    }

    #[test]
    fn a_jump_off_a_high_ledge_kills_only_past_5120_below_its_end() {
        // He lands depth + 873 below where the jump ended: 4,873 is safe
        // (a walk-off that far would kill him), 5,373 is not.
        for (depth, dies) in [(4000.0, false), (4500.0, true)] {
            let list = list();
            let loops = lookup(&list);
            let mut m = Mover::new([0.0; 3], true);
            let mut w = Walker::new(0.0);
            tick(&mut m, &mut w, STAND, false, &loops, &Plane(0.0));
            tick(&mut m, &mut w, STAND, true, &loops, &Plane(0.0));
            // The ground falls away under him while he jumps.
            let low = |h: f32| move |_: f32, _: f32| Some((h, [0.0, 0.0, 1.0]));
            struct F<G>(G);
            impl<G: Fn(f32, f32) -> Option<(f32, [f32; 3])>> GroundLookup for F<G> {
                fn ground(&self, x: f32, y: f32) -> Option<(f32, [f32; 3])> {
                    (self.0)(x, y)
                }
            }
            let pit = F(low(-depth));
            let mut died = None;
            for _ in 0..60 {
                let t = tick(&mut m, &mut w, STAND, false, &loops, &pit);
                match t.report.fall {
                    Some(FallEvent::Died(_)) => died = Some(true),
                    Some(FallEvent::Landed(_)) => died = Some(false),
                    _ => {}
                }
            }
            assert_eq!(died, Some(dies), "depth {depth}");
        }
    }

    #[test]
    fn jumps_are_refused_while_falling_starting_too_high_or_facing_uphill() {
        let list = list();
        let loops = lookup(&list);
        let flat = Plane(0.0);
        // During stopwalk (user's check).
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        tick(&mut m, &mut w, Intent { forward: 1, turn: 0, run: false }, false, &loops, &flat);
        tick(&mut m, &mut w, Intent { forward: 1, turn: 0, run: false }, false, &loops, &flat);
        let t = tick(&mut m, &mut w, Intent { forward: 1, turn: 0, run: false }, true, &loops, &flat);
        assert_eq!(t.notes, ["jump refused: stopwalk is playing"]);
        // Falling.
        let mut w = Walker::new(0.0);
        w.set_mode(Mode::Fall);
        assert!(can_jump(&m, &w, &flat, &loops).is_err());
        // 300 above the ground, not on it: refused; 200 above: a jump.
        let w = Walker::new(0.0);
        for (z, ok) in [(300.0, false), (200.0, true)] {
            let mut m = Mover::new([0.0, 0.0, z], true);
            m.on_ground = false;
            assert_eq!(can_jump(&m, &w, &flat, &loops).is_ok(), ok, "{z}");
        }
        // Steep ground (45°, rising along +y): facing +y (heading π) is
        // facing uphill; facing −y (heading 0) is not.
        let hill = Plane(1.0);
        for (heading, ok) in [(std::f32::consts::PI, false), (0.0, true)] {
            let mut m = Mover::new([0.0, 0.0, 100.0], true);
            m.on_ground = false;
            let w = Walker::new(heading);
            assert_eq!(can_jump(&m, &w, &hill, &loops).is_ok(), ok, "heading {heading}");
        }
        // From running: runJump isn't loaded here.
        let mut w = Walker::new(0.0);
        w.movement = "run";
        assert_eq!(can_jump(&Mover::new([0.0; 3], true), &w, &flat, &loops), Err("runJump is not loaded yet".into()));
        assert_eq!(JumpKind::for_movement("back"), JumpKind::Back);
    }
}
