//! One update of Connor: his walker (`connor.rs`, the animation and the
//! step it asks for) and his mover (`connor_mover.rs`, what happens to that step)
//! together, with what passes between them: falls, landings, head-on hits
//! and jumps.
//!
//! Rules: `connor-physics.md` "Falling off an edge", "Walls", "Jumping";
//! the user's checks of jumping in the original (docs/DESIGN.md M2 2e).

use crate::connor_auto::{AutoEnd, AutoKind, Before};
use crate::connor::{Intent, JumpKind, Mode, MovementLoop, Step, Walker};
use crate::connor_mover::{FallEvent, HitRecord, Mover, MoverWorld, PushResult, Report, State, Wish, JUMP_ABOVE_TERRAIN, WALKABLE_NORMAL_Z};

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
    /// A scripted turn or walk ended this update (its cue, if any, is
    /// for the caller to send).
    pub auto_end: Option<AutoEnd>,
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
            mover.position[2] - crate::connor_mover::JUMP_DEATH_OFFSET
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
    // anim-scripts.md rules 24-25: a scripted turn or walk turns him, or
    // gives the walker its walk instead of the keys.
    let mut intent = intent;
    let mut auto_end = None;
    let dt = (1.0 / crate::UPDATES_PER_SECOND) as f32;
    if let Some(a) = walker.auto.as_mut() {
        let mut done = None;
        match a.before(mover.position, &mut walker.heading, intent.turn, dt) {
            Before::Keys | Before::Turned { .. } => {}
            Before::Cancelled => {
                notes.push("scripted turn cancelled by a turn key: its cue never comes (connor-movement.md \"Turning\" rule 2)".into());
                done = Some(None);
            }
            Before::TurnDone => match a.kind {
                AutoKind::Point => {
                    notes.push(format!("scripted turn done: heading {:.3}", walker.heading));
                    done = Some(Some(AutoEnd { cue: a.cue.take(), why: "turned to face the spot".into() }));
                }
                AutoKind::Walk { run } => {
                    if let crate::connor_auto::AutoPhase::Walking { distance, .. } = a.phase {
                        notes.push(format!("scripted turn done: heading {:.3}; {} straight ahead for {distance:.0}", walker.heading, if run { "runs" } else { "walks" }));
                    }
                    intent = Intent { forward: 1, turn: 0, run };
                }
            },
            Before::Walk(i) => intent = i,
        }
        if let Some(end) = done {
            walker.auto = None;
            auto_end = end;
        }
    }
    // The choice this update uses the water test from the end of the last
    // move (water.md "Wading animations" rule 3).
    walker.submerged = mover.submerged;
    let mut step = None;
    let report = if mover.state == State::Dead {
        // The mover stands still; the death animation plays on.
        step = Some(walker.update(intent, loops));
        mover.update([0.0; 3], world)
    } else {
        mover.update_with(world, |_, fall| {
            // Fall replaces the walk at once, so its first update already
            // uses the hold animation's step. A walk-off falls as `Stand`.
            // A crash too (the user's check in the original, 2026-10-17:
            // after a standing jump into a tree the stumble is cut short
            // when he starts falling; Q82 b).
            if fall == Some(FallEvent::Started) {
                walker.kind = crate::connor::JumpKind::Stand;
                walker.set_mode(Mode::Fall);
            }
            let s = walker.update(intent, loops);
            // Head-on acts while jumping too, steep surfaces counting, but
            // never during `backJump` ("Walls").
            let jumping = walker.mode == Mode::Jump && walker.kind != JumpKind::Back;
            let wish = Wish { step: s.wanted, moving: walker.moving() || jumping, steep_head_on: jumping, push: walker.push_ability() };
            step = Some(s);
            wish
        })
    };
    // The water test after the move (water.md "The water test").
    let [x, y, z] = mover.position;
    let water = world.water(x, y);
    let submerged = crate::water::submerged(water, z);
    if submerged != mover.submerged {
        let surface = water.map_or("none".into(), |(s, _)| format!("{s:.0}"));
        notes.push(format!(
            "submerged {}: surface {surface}, feet {z:.0}, ground flags 0x{:X}",
            if submerged { "on" } else { "off" },
            water.map_or(0, |(_, f)| f)
        ));
    }
    mover.submerged = submerged;
    match report.fall {
        Some(FallEvent::Landed(_)) => walker.land(),
        Some(FallEvent::Died(_)) => walker.set_mode(Mode::Dying),
        _ => {}
    }
    // pushing.md rules 2.5, 2.6, 3: what his touches on pushables do to
    // his state.
    for t in &report.pushes {
        notes.push(t.describe());
    }
    let pushed = report.pushes.iter().any(|t| t.result == PushResult::Push);
    let state_note = if pushed {
        walker.push_touch()
    } else if report.pushes.iter().any(|t| t.result == PushResult::WallBackToMove) {
        walker.push_back_to_move()
    } else if report.pushes.iter().any(|t| t.result == PushResult::WallStand) {
        Some(walker.stand())
    } else {
        None
    };
    notes.extend(state_note.map(str::to_string));
    // A steep hit during a jump (connor-physics.md "Jumping" rule 5): a
    // crash ends it; a glancing hit (any `backJump` hit, or not head-on)
    // doesn't: gravity stays off, the animation plays on (Q84 a).
    let mut wall_stop = None;
    let steep_hit = report.hits.iter().find(|h| h.hit.normal[2] < WALKABLE_NORMAL_Z);
    if let (Mode::Jump, Some(_), Some(hit)) = (walker.mode, report.jump_hit, steep_hit) {
        let jump = walker.kind.jump();
        let (crash, to) = jump_hit_outcome(walker.kind, hit);
        if crash {
            mover.end_jump_on_hit();
            walker.set_mode(Mode::Crash);
            wall_stop = Some((jump, "crash"));
            notes.push(format!("{jump} hit {} (steep, {} pass): the jump ends, {to}", hit.hit.what, hit.pass));
        } else if !mover.jump_glanced {
            mover.jump_glanced = true;
            notes.push(format!("{jump} hit {} (steep, {} pass): {to}", hit.hit.what, hit.pass));
        }
    } else if report.head_on && walker.auto.as_ref().is_some_and(|a| a.walk_done(mover.position).is_none() && matches!(a.phase, crate::connor_auto::AutoPhase::Walking { .. })) {
        // A scripted walk ran head-on into something: the cue comes early
        // (rule 24) instead of the crash or stop.
        walker.stop_at_once();
        let cue = walker.auto.take().and_then(|a| a.cue);
        notes.push("scripted walk ran head-on into something: it ends, cue now (stand-in Q94: stands at once, no crash, no wall block)".into());
        auto_end = Some(AutoEnd { cue, why: "head-on".into() });
    } else if report.head_on {
        // Head-on into a wall: walking or stepping back stops at once;
        // running crashes from the next update.
        wall_stop = walker.head_on_at_wall();
    }
    // A scripted walk has gone its distance: Wait, and the cue.
    if let Some((gone, distance)) = walker.auto.as_ref().and_then(|a| a.walk_done(mover.position)) {
        let cue = walker.auto.take().and_then(|a| a.cue);
        notes.push(format!("scripted walk done: {gone:.0} gone of {distance:.0}: Wait"));
        auto_end = Some(AutoEnd { cue, why: format!("walked {gone:.0}") });
    }
    Tick { step, report, wall_stop, notes, auto_end }
}

/// What a steep hit during a jump does (connor-physics.md "Jumping" rule
/// 5 and "Walls", corrected 2026-10-17 by A's answer to Q82): whether he
/// crashes, and the end of the log line. Only a crash ends the jump.
/// - `backJump`: never crashes; a glancing hit, the flip plays on with
///   gravity off (user's check, Q41 b; Q84 a).
/// - A down-pass hit during `jump` or `runJump`: crash.
/// - An up, across or slide hit: crash only if the head-on test said
///   head-on. During a jump every steep surface is head-on (the mover's
///   `steep_head_on`), and both `jump` and `runJump` are in the crash list.
/// - Otherwise (not head-on: against a `KQMonster` or with the Connor flag,
///   neither of which our code has yet): glancing; the jump does not end,
///   gravity stays off, the animation plays on, then Fall (Q84 a).
pub fn jump_hit_outcome(kind: JumpKind, hit: &HitRecord) -> (bool, &'static str) {
    match (kind, hit.pass, hit.result) {
        (JumpKind::Back, _, _) => (false, "glancing: gravity stays off, backJump plays on, then Fall"),
        (_, "down", _) => (true, "crash (landed on something steep)"),
        (_, _, "head-on") => (true, "crash (head-on; jump and runJump are in the crash list)"),
        _ => (false, "not head-on: glancing, gravity stays off, the jump plays on, then Fall (Q84 a)"),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::connor::{MoveKey, Transition};
    use crate::connor_mover::GroundLookup;

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

    // Running and backward jumps (2e-2), with `Conner.anm`'s frame counts
    // and runJumpHold's transitions; arcs made up (flat, so he lands in the
    // first Fall update).

    fn jumps_list() -> Vec<(&'static str, MovementLoop)> {
        let mut list = list();
        let t = |to, to_phase, via| Transition { from: -1.0, to, to_phase, via };
        list.extend([
            ("run", looping(15, -2350.2, Vec::new())),
            ("back", MovementLoop { speed: -1.0, ..looping(20, -1427.1, Vec::new()) }),
            ("runJump", one_shot(vec![[0.0, -317.8, 0.0]; 20])),
            ("runJumpHold", looping(3, 0.0, vec![t("run", 0.8, "jumprun"), t("stop", 0.0, "jumpstop")])),
            ("runJumpDie", one_shot(vec![[0.0; 3]; 10])),
            ("jumprun", one_shot(vec![[0.0, -276.0, 0.0]; 3])),
            ("jumpstop", one_shot(vec![[0.0, -124.0, 0.0]; 15])),
            ("backJump", one_shot(vec![[0.0, 142.6, 0.0]; 18])),
            ("backJumpHold", looping(4, 0.0, Vec::new())),
            ("backJumpEnd", one_shot(vec![[0.0; 3]; 11])),
            ("backJumpDie", one_shot(vec![[0.0; 3]; 10])),
        ]);
        list
    }

    /// Jumps from `from` (held by `keys`), then plays on with `after` held
    /// once the jump is in the air; returns the movements played.
    fn jump_from(from: &'static str, keys: Intent, after: Intent, updates: usize) -> (Vec<&'static str>, Mover, Walker) {
        let list = jumps_list();
        let loops = lookup(&list);
        let ground = Plane(0.0);
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        w.movement = from;
        w.choice = from;
        let mut played = Vec::new();
        let t = tick(&mut m, &mut w, keys, true, &loops, &ground);
        played.extend(t.step.map(|s| s.played));
        for _ in 0..updates {
            let t = tick(&mut m, &mut w, after, false, &loops, &ground);
            played.extend(t.step.map(|s| s.played));
        }
        (played, m, w)
    }

    fn runs(played: &[&'static str]) -> Vec<(&'static str, usize)> {
        let mut out: Vec<(&'static str, usize)> = Vec::new();
        for p in played {
            match out.last_mut() {
                Some((name, n)) if name == p => *n += 1,
                _ => out.push((p, 1)),
            }
        }
        out
    }

    const RUN: Intent = Intent { forward: 1, turn: 0, run: true };

    #[test]
    fn running_jump_lands_into_jumprun_with_run_held() {
        let (played, m, w) = jump_from("run", RUN, RUN, 26);
        assert_eq!(
            runs(&played),
            [("runJump", 20), ("runJumpHold", 2), ("jumprun", 3), ("run", 2)],
            "the hold: the Fall update and the landing's request update"
        );
        // runJump, jumprun, then 2 updates of run (2,350.2 per 14/15 s).
        let run_two = 2.0 * 2350.2 / 14.0;
        assert!((m.position[1] + 6356.0 + 3.0 * 276.0 + run_two).abs() < 1.0, "{:?}", m.position);
        assert_eq!((w.mode, w.movement), (Mode::Free, "run"));
    }

    #[test]
    fn running_jump_lands_into_jumpstop_with_forward_let_go_and_turns_there() {
        let turn = Intent { forward: 0, turn: 1, run: true };
        let (played, _, w) = jump_from("run", RUN, turn, 23);
        assert_eq!(runs(&played)[..3], [("runJump", 20), ("runJumpHold", 2), ("jumpstop", 2)]);
        // No turning until the landing; then he turns during jumpstop
        // (the user's check in the original).
        // (the update after landing, then 2 of jumpstop).
        let turned = w.heading;
        assert!((turned - 3.0 * crate::connor::TURN_RATE / 15.0).abs() < 1e-4, "{turned}");
        let (played, _, w) = jump_from("run", RUN, turn, 40);
        assert_eq!(runs(&played)[2..], [("jumpstop", 15), ("left", 4)]);
        assert_eq!(w.movement, "left");
    }

    #[test]
    fn running_jump_with_run_switched_off_walks_at_once() {
        let walk = Intent { forward: 1, turn: 0, run: false };
        let (played, _, _) = jump_from("run", RUN, walk, 25);
        // No transition to walk: at once, in the update after landing.
        assert_eq!(runs(&played), [("runJump", 20), ("runJumpHold", 1), ("walk", 5)]);
    }

    #[test]
    fn back_jump_then_its_own_hold_and_landing() {
        let back = Intent { forward: -1, turn: 0, run: false };
        let (played, m, w) = jump_from("back", back, STAND, 32);
        assert_eq!(runs(&played)[..3], [("backJump", 18), ("backJumpHold", 1), ("backJumpEnd", 11)]);
        assert!((m.position[1] - 18.0 * 142.6).abs() < 1.0, "backwards: {:?}", m.position);
        assert_eq!(w.kind, JumpKind::Back);
    }

    // Hitting something during a jump (2e-3).

    /// Flat ground at 0 and a wall across y = `at`, facing toward y = 0:
    /// a move crossing it stops 225 short (ball radius 200 + the 25
    /// pull-back), as a building wall does.
    struct Wall {
        at: f32,
    }

    impl MoverWorld for Wall {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((0.0, [0.0, 0.0, 1.0]))
        }

        fn sweep(&self, _: &crate::collide::BallStack, from: [f32; 3], to: [f32; 3]) -> Option<crate::connor_mover::Hit> {
            let side = self.at.signum();
            let touch_y = self.at - side * 200.0;
            if (to[1] - touch_y) * side <= 0.0 || from[1] == to[1] {
                return None;
            }
            let f = (touch_y - from[1]) / (to[1] - from[1]);
            let touch: [f32; 3] = std::array::from_fn(|i| from[i] + (to[i] - from[i]) * f);
            let len = (0..3).map(|i| (to[i] - from[i]).powi(2)).sum::<f32>().sqrt() * f;
            let back = if len > 25.0 { 25.0 / len } else { 1.0 };
            let position: [f32; 3] = std::array::from_fn(|i| touch[i] - (touch[i] - from[i]) * back);
            Some(crate::connor_mover::Hit { position, touch, normal: [0.0, -side, 0.0], what: "wall".into() })
        }
    }

    /// Jumps from `from` toward a wall at `at` (or a steep hill); returns
    /// the tick where the jump ended and the walker and mover after it.
    fn jump_into(from: &'static str, keys: Intent, world: &dyn MoverWorld) -> (Tick, Mover, Walker) {
        let list = jumps_list();
        let loops = lookup(&list);
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        w.movement = from;
        w.choice = from;
        let mut t = tick(&mut m, &mut w, keys, true, &loops, world);
        for _ in 0..30 {
            if w.mode != Mode::Jump {
                return (t, m, w);
            }
            t = tick(&mut m, &mut w, keys, false, &loops, world);
        }
        panic!("the jump never ended: {:?}", t.report);
    }

    // Wading (water.md, M2 2h-2).

    /// Flat floor at 0; for y between `from` and `to` (he walks toward −y)
    /// a water square with the surface 1,020 above the floor, as in
    /// Daventry's river; dry ground (surface = floor) elsewhere.
    struct River {
        from: f32,
        to: f32,
    }

    impl GroundLookup for River {
        fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
            Some((0.0, [0.0, 0.0, 1.0]))
        }

        fn water(&self, _: f32, y: f32) -> Option<(f32, u32)> {
            let wet = y <= self.from && y >= self.to;
            Some(if wet { (1020.0, kq8_data::envinfo::WATER) } else { (0.0, 0x1) })
        }
    }

    fn wading_list() -> Vec<(&'static str, MovementLoop)> {
        let mut list = list();
        let at = |from, to, to_phase, via| Transition { from, to, to_phase, via };
        for (name, l) in list.iter_mut() {
            if *name == "walk" {
                l.transitions.push(at(0.2, "waterwalk", 0.0, "walkwaterwalk"));
            }
        }
        list.extend([
            ("waterwalk", looping(24, -900.0, vec![at(0.958333, "walk", 0.75, "waterwalkwalk")])),
            ("walkwaterwalk", one_shot(vec![[0.0, -40.0, 0.0]; 6])),
            ("waterwalkwalk", one_shot(vec![[0.0, -40.0, 0.0]; 6])),
            ("run", looping(15, -2350.2, vec![at(-1.0, "waterrun", 0.0, "runwaterrun")])),
            ("waterrun", looping(19, -1800.0, Vec::new())),
            ("runwaterrun", one_shot(vec![[0.0, -100.0, 0.0]; 5])),
            ("crash", one_shot(vec![[0.0, 63.0, 0.0]; 7])),
        ]);
        list
    }

    #[test]
    fn walking_through_deep_water_wades_and_comes_out_walking() {
        let list = wading_list();
        let loops = lookup(&list);
        let river = River { from: -1500.0, to: -6000.0 };
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        let walk = Intent { forward: 1, turn: 0, run: false };
        let mut played = Vec::new();
        let mut first_wet = None;
        for i in 0..200 {
            let t = tick(&mut m, &mut w, walk, false, &loops, &river);
            if m.submerged && first_wet.is_none() {
                first_wet = Some(i);
                assert!(t.notes.iter().any(|n| n.starts_with("submerged on: surface 1020, feet 0")), "{:?}", t.notes);
            }
            played.push(t.step.unwrap().played);
        }
        // He stays on the floor: no floating.
        assert_eq!(m.position[2], 0.0);
        let first_wet = first_wet.expect("never submerged");
        // The update after the test turned on asks for waterwalk.
        let order: Vec<_> = runs(&played).into_iter().map(|(n, _)| n).collect();
        assert_eq!(
            order,
            ["stop", "stopwalk", "walk", "walkwaterwalk", "waterwalk", "waterwalkwalk", "walk"],
            "{played:?}"
        );
        assert!(played[..=first_wet].iter().all(|p| !p.starts_with("water") && *p != "walkwaterwalk"));
    }

    #[test]
    fn running_head_on_in_water_crashes() {
        for from in ["waterrun", "runwaterrun"] {
            let mut w = Walker::new(0.0);
            w.movement = from;
            w.choice = "waterrun";
            assert_eq!(w.head_on_at_wall(), Some((from, "crash")));
        }
    }

    /// Walks head-on into a wall until forward is blocked, then holds
    /// forward and right for 45 updates, the turn key pressed once
    /// (`repeat` false) or seen pressed again every update, as keyboard
    /// auto-repeat does (true). Right, because `stopwalk` steps to his
    /// left: turning right brings that step back to head-on. Returns his
    /// heading at the end and the walker.
    fn turn_at_wall(repeat: bool) -> (f32, Walker) {
        let list = list();
        let loops = lookup(&list);
        let wall = Wall { at: -600.0 };
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        let walk = Intent { forward: 1, turn: 0, run: false };
        for _ in 0..30 {
            tick(&mut m, &mut w, walk, false, &loops, &wall);
        }
        assert_eq!((w.movement, w.blocked_name()), ("stop", "fwd"));
        let turning = Intent { forward: 1, turn: -1, run: false };
        for i in 0..45 {
            if i == 0 || repeat {
                w.key_event(MoveKey::Turn, true);
            }
            tick(&mut m, &mut w, turning, false, &loops, &wall);
        }
        (w.heading, w)
    }

    #[test]
    fn at_a_wall_a_held_turn_key_turns_him_only_while_it_repeats() {
        // Pressed once: one update's turn, then the next head-on hit
        // blocks forward again and he stands (user, 2026-10-12: wrong).
        let (once, w) = turn_at_wall(false);
        assert!(once.abs() < 0.4, "turned {once}");
        assert_eq!((w.movement, w.blocked_name()), ("stop", "fwd"));
        // Repeating: he keeps turning past the head-on angle (0.35 rad)
        // and glances off along the wall (user, 2026-10-12: original).
        let (held, _) = turn_at_wall(true);
        assert!(held.abs() > 1.0, "turned {held}");
    }

    #[test]
    fn a_standing_jump_into_a_wall_crashes_head_on() {
        // Rule 5 as corrected by Q82: `jump` is in the crash list.
        let (t, m, w) = jump_into("stop", STAND, &Wall { at: -600.0 });
        assert!(t.notes.iter().any(|n| n.contains("jump hit wall (steep, across pass): the jump ends, crash (head-on")), "{:?}", t.notes);
        assert_eq!(t.report.hits.last().map(|h| h.result), Some("head-on"));
        assert_eq!((w.mode, m.state, m.use_gravity, m.step_up), (Mode::Crash, State::Normal, true, 325.0));
        assert!((m.position[1] + 375.0).abs() < 0.5, "stopped 225 from the wall: {:?}", m.position);
    }

    #[test]
    fn a_running_jump_into_a_wall_crashes() {
        let (t, m, w) = jump_into("run", RUN, &Wall { at: -2000.0 });
        assert!(t.notes.iter().any(|n| n.contains("runJump hit wall (steep, across pass): the jump ends, crash (head-on")), "{:?}", t.notes);
        assert_eq!((w.mode, m.state, m.use_gravity), (Mode::Crash, State::Normal, true));
    }

    #[test]
    fn a_back_flip_slides_along_the_wall_and_plays_on() {
        let list = jumps_list();
        let loops = lookup(&list);
        let back = Intent { forward: -1, turn: 0, run: false };
        let wall = Wall { at: 1000.0 };
        let mut m = Mover::new([0.0; 3], true);
        let mut w = Walker::new(0.0);
        w.movement = "back";
        w.choice = "back";
        let mut notes = Vec::new();
        let mut jump_updates = 0;
        let mut hit_results = Vec::new();
        for n in 0..25 {
            let t = tick(&mut m, &mut w, back, n == 0, &loops, &wall);
            if t.notes.iter().any(|n| n.contains("backJump hit wall")) {
                // Q84 a: the jump doesn't end; gravity stays off.
                assert!(!m.use_gravity && m.step_up == 0.0, "{:?}", t.notes);
            }
            notes.extend(t.notes);
            hit_results.extend(t.report.hits.iter().map(|h| h.result));
            if t.step.as_ref().is_some_and(|s| s.played == "backJump") {
                jump_updates += 1;
            }
        }
        let hits: Vec<_> = notes.iter().filter(|n| n.contains("backJump hit wall")).collect();
        assert_eq!(hits.len(), 1, "noted once: {notes:?}");
        assert!(hits[0].contains("backJump plays on"), "{hits:?}");
        assert_eq!(jump_updates, 18, "the whole animation");
        // Never head-on during backJump: it slides.
        assert!(!hit_results.contains(&"head-on"), "{hit_results:?}");
        assert!(m.position[1] <= 1000.0 - 225.0 + 0.5, "{:?}", m.position);
        assert_ne!(w.mode, Mode::Crash);
    }

    #[test]
    fn landing_on_a_steep_hill_during_a_jump_crashes() {
        // Rising toward −y at 63° (normal z 0.447): the jump's arc goes
        // into it, and the down pass finds the hillside above him.
        let (t, m, w) = jump_into("stop", STAND, &Plane(-2.0));
        assert!(t.notes.iter().any(|n| n.contains("jump hit terrain (steep, down pass): the jump ends, crash (landed")), "{:?}", t.notes);
        assert_eq!((w.mode, m.state), (Mode::Crash, State::Normal));
    }

    #[test]
    fn only_head_on_or_down_pass_jump_hits_crash() {
        let record = |pass, result| HitRecord {
            pass,
            hit: crate::connor_mover::Hit { position: [0.0; 3], touch: [0.0; 3], normal: [0.0, 1.0, 0.0], what: "x".into() },
            angle: None,
            result,
        };
        for kind in [JumpKind::Stand, JumpKind::Run] {
            for (pass, result, crash) in [
                ("down", "glance", true),
                ("up", "head-on", true),
                ("across", "head-on", true),
                ("slide", "head-on", true),
                // Not head-on (a `KQMonster` or the Connor flag): no crash.
                ("across", "glance", false),
                ("up", "stop", false),
            ] {
                assert_eq!(jump_hit_outcome(kind, &record(pass, result)).0, crash, "{kind:?} {pass} {result}");
            }
        }
        for pass in ["down", "up", "across"] {
            assert!(!jump_hit_outcome(JumpKind::Back, &record(pass, "head-on")).0);
        }
    }
}
