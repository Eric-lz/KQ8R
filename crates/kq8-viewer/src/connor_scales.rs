//! Connor's `Scales` state: Dead City's Scales of Justice
//! (`docs/behaviour/tile-puzzle.md` rules 23-26a; DESIGN U-8b). Built
//! like the `Tile` state: run by [`ConnorSim::scales_step`] before his
//! update; the steps onto and off a dish are his `conscale.anm`
//! movements as one-item Special lists, whose own steps carry him.
//!
//! Not built (Q198 a): the camera's spots and views of rules 25-26; the
//! camera keeps following him.

use kq8_core::connor::{Intent, Mode, SpecialStep};
use kq8_core::connor_auto::{AutoKind, AutoMove};
use kq8_core::weapons::WeaponMode;

use super::ConnorSim;

/// Rule 23: his movement list gains this file.
pub const LIST: &str = "conscale.anm";
/// Its movements (data).
pub const MOVEMENTS: [&str; 4] = ["onscale1", "onscale2", "offscale1", "offscale2"];
/// Rules 23, 25: the start spot (heading 1.57 on entering).
pub const START: [f32; 3] = [217_450.0, 49_140.0, 2_900.0];
/// Rule 25: west of this x he is on the floor, else on a dish.
pub const DISH_X: f32 = 219_000.0;
/// Rule 25: the three headings on the floor: step away, the two dishes.
pub const AWAY: f32 = -1.57;
pub const DISHES: [f32; 2] = [2.207, 0.933];
/// Rule 25: stepping away, he walks to this point.
pub const WAY_OUT: [f32; 2] = [209_500.0, 49_065.0];
/// Rule 26: off a dish: with y above this he turns to `OFF_NORTH`, below
/// it to `OFF_SOUTH`; exactly on it nothing happens.
pub const DISH_Y: f32 = 50_000.0;
pub const OFF_NORTH: f32 = -0.933;
pub const OFF_SOUTH: f32 = -2.207;
/// Rule 26a: after `onscale2` he is turned 180° and raised 200.
pub const ON_RAISE: f32 = 200.0;

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    /// The keys (rule 24).
    Keys,
    /// `onscale1` (gravity off) then `onscale2` (gravity on), rule 26a.
    On1,
    On2,
    /// `offscale1` (gravity off) then `offscale2` (gravity on).
    Off1,
    Off2,
}

/// The state while Connor is in it.
#[derive(Debug, Clone)]
pub struct ScalesRun {
    stage: Stage,
    /// The forward key last update: a step starts when it is pressed
    /// (ours, Q198 b).
    remembered: i8,
}

/// The nearest of `choices` to `heading` (angles wrapped).
fn nearest(heading: f32, choices: &[f32]) -> f32 {
    let off = |a: f32| {
        let d = (a - heading).rem_euclid(std::f32::consts::TAU);
        d.min(std::f32::consts::TAU - d)
    };
    choices.iter().copied().min_by(|a, b| off(*a).total_cmp(&off(*b))).unwrap_or(heading)
}

impl ConnorSim {
    /// Rule 23: into the `Scales` state (a drawn weapon is put away by the
    /// caller, as for `Tile`).
    pub fn enter_scales(&mut self, at: &str) -> String {
        self.walker.auto = None;
        self.intent.forward = 0;
        let start = if self.set_mode != WeaponMode::None {
            let l = self.ready_set(None, WeaponMode::None, None, false, false, true);
            format!("a weapon was out: away, empty hands ({l})")
        } else {
            self.walker.stand_now();
            "stop".to_string()
        };
        let missing: Vec<&str> = MOVEMENTS.iter().copied().filter(|m| self.scales_movement(m).is_none()).collect();
        self.scales = Some(ScalesRun { stage: Stage::Keys, remembered: 0 });
        let p = self.mover.position;
        let line = format!(
            "enters the Scales state at ({:.0}, {:.0}, {:.0}) heading {:.3}; {start}{} (tile-puzzle.md rule 23)",
            p[0],
            p[1],
            p[2],
            self.walker.heading,
            if missing.is_empty() { String::new() } else { format!("; not loaded: {missing:?}") }
        );
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// One of `conscale.anm`'s movements, as loaded.
    fn scales_movement(&self, name: &str) -> Option<&'static str> {
        let wanted = kq8_core::connor::special_movement_name(LIST, name);
        self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
    }

    /// Plays `name` as a one-item Special list; the log text.
    fn scales_play(&mut self, name: &str) -> String {
        match self.scales_movement(name) {
            Some(m) => {
                self.walker.start_special(vec![SpecialStep::Play(m, name.to_string())]);
                format!("'{name}' plays")
            }
            None => format!("'{name}' is not loaded: nothing plays"),
        }
    }

    /// Out of the state (rule 25: stepping away).
    fn leave_scales(&mut self) {
        if self.scales.take().is_some_and(|s| matches!(s.stage, Stage::On1 | Stage::Off1)) {
            self.mover.end_tile_jump();
        }
        self.walker.auto = None;
        self.walker.stand_now();
    }

    /// Before his update (rules 24-26a). `intent` is changed for his update
    /// (no walking on the scales: only turning on the spot). Returns lines
    /// for `connor.log`.
    pub fn scales_step(&mut self, intent: &mut Intent) -> Vec<String> {
        let mut lines = Vec::new();
        let Some(stage) = self.scales.as_ref().map(|s| s.stage) else { return lines };
        let special_over = self.walker.mode == Mode::Free;
        match stage {
            Stage::Keys => {
                // Rule 24: turn keys turn on the spot; back with a turn key
                // turns too; forward takes one step.
                let f = std::mem::replace(&mut intent.forward, 0);
                let s = self.scales.as_mut().unwrap();
                let pressed = f > 0 && s.remembered <= 0;
                s.remembered = f;
                if !(pressed && self.walker.mode == Mode::Free && self.walker.auto.is_none()) {
                    return lines;
                }
                intent.turn = 0;
                let [x, y, _] = self.mover.position;
                let heading = self.walker.heading;
                if x < DISH_X {
                    let to = nearest(heading, &[AWAY, DISHES[0], DISHES[1]]);
                    if to == AWAY {
                        // Rule 25: the only way out: the start spot facing
                        // away, then a walk (ours: the state is left first,
                        // Q198 c).
                        self.put(START, AWAY);
                        self.leave_scales();
                        self.walker.auto = Some(AutoMove::new(AutoKind::Walk { run: false }, WAY_OUT, None));
                        lines.push(format!("Scales: forward on the floor, heading {heading:.3} -> {AWAY}: steps away: put on the start spot, leaves the state, walks to ({:.0}, {:.0}) (rule 25; the camera's spot: not built, Q198)", WAY_OUT[0], WAY_OUT[1]));
                    } else {
                        // Rules 25, 26a: onto a dish.
                        self.put(START, to);
                        self.mover.start_tile_jump();
                        let play = self.scales_play("onscale1");
                        self.scales.as_mut().unwrap().stage = Stage::On1;
                        lines.push(format!("Scales: forward on the floor, heading {heading:.3} -> {to}: onto a dish: put on the start spot, gravity and keep-on-ground off; {play} (rules 25, 26a; the camera's spot: not built, Q198)"));
                    }
                } else {
                    // Rule 26: off a dish.
                    let to = if y > DISH_Y {
                        OFF_NORTH
                    } else if y < DISH_Y {
                        OFF_SOUTH
                    } else {
                        lines.push("Scales: forward on a dish at y 50,000 exactly: nothing (rule 26)".to_string());
                        return lines;
                    };
                    let p = self.mover.position;
                    self.put(p, to);
                    self.mover.start_tile_jump();
                    let play = self.scales_play("offscale1");
                    self.scales.as_mut().unwrap().stage = Stage::Off1;
                    lines.push(format!("Scales: forward on a dish (y {y:.0}): turns to {to} (at once: ours, Q198 d), gravity and keep-on-ground off; {play} (rules 26, 26a)"));
                }
            }
            Stage::On1 | Stage::Off1 => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    // Rule 26a: both back on, then the second movement with
                    // ordinary gravity.
                    self.mover.end_tile_jump();
                    let (next, stage) = if stage == Stage::On1 { ("onscale2", Stage::On2) } else { ("offscale2", Stage::Off2) };
                    let play = self.scales_play(next);
                    self.scales.as_mut().unwrap().stage = stage;
                    let p = self.mover.position;
                    lines.push(format!("Scales: {} ended at ({:.0}, {:.0}, {:.0}): gravity and keep-on-ground on; {play} (rule 26a)", if stage == Stage::On2 { "onscale1" } else { "offscale1" }, p[0], p[1], p[2]));
                }
            }
            Stage::On2 | Stage::Off2 => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    self.walker.stand_now();
                    let p = self.mover.position;
                    let what = if stage == Stage::On2 {
                        // Rule 26a: turned 180° and raised 200.
                        let heading = self.walker.heading + std::f32::consts::PI;
                        self.put([p[0], p[1], p[2] + ON_RAISE], heading);
                        format!("onscale2 ended at ({:.0}, {:.0}, {:.0}): turned 180° to {heading:.3} and raised {ON_RAISE} (rule 26a)", p[0], p[1], p[2])
                    } else {
                        format!("offscale2 ended at ({:.0}, {:.0}, {:.0}) (rule 26)", p[0], p[1], p[2])
                    };
                    self.scales.as_mut().unwrap().stage = Stage::Keys;
                    lines.push(format!("Scales: {what}; keys"));
                }
            }
        }
        lines
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_floor_heading_is_the_nearest_of_three() {
        assert_eq!(nearest(1.4, &[AWAY, DISHES[0], DISHES[1]]), 0.933);
        // The entry heading is exactly half-way: the first wins (ours, Q198 g).
        assert_eq!(nearest(1.57, &[AWAY, DISHES[0], DISHES[1]]), 2.207);
        assert_eq!(nearest(2.0, &[AWAY, DISHES[0], DISHES[1]]), 2.207);
        assert_eq!(nearest(-1.0, &[AWAY, DISHES[0], DISHES[1]]), AWAY);
        // Wrapped: 4.0 is -2.28, nearer -1.57 than 2.207.
        assert_eq!(nearest(4.0, &[AWAY, DISHES[0], DISHES[1]]), AWAY);
    }
}
