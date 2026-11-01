//! Connor's `ThrowRock` state (`docs/behaviour/ranged-weapons.md` 8a-8b;
//! DESIGN U-9c). Run by [`ConnorSim::rock_step`] before his update, like
//! the Tile state: he turns toward the aim (an automatic turn), puts a
//! drawn weapon away, plays `throw` (`Conner.anm`, `c_rktoss`); its `do`
//! cue launches the rock (taken by `door_view`, which flies it in the
//! core: `WorldState::launch_rock`); then the old weapon is readied again
//! and he stands.

use kq8_core::connor::{Intent, Mode, SpecialStep};
use kq8_core::connor_auto::{AutoKind, AutoMove};
use kq8_core::weapons::WeaponMode;

use super::ConnorSim;

/// 8b: his throw movement.
pub const THROW: &str = "throw";

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    /// The turn toward the aim (8a step 3).
    Turning,
    /// `away` playing (a weapon was out).
    Away,
    /// `throw` playing.
    Throwing,
}

/// The state while Connor is in it.
#[derive(Debug, Clone)]
pub struct RockRun {
    stage: Stage,
    pub aim: [f32; 3],
    /// The weapon mode before (8b: it comes back at the end).
    pub had: Option<WeaponMode>,
    /// The `do` cue fired (one rock per throw).
    pub launched: bool,
}

impl ConnorSim {
    /// 8b: into `ThrowRock`, toward `aim`. `had`: the weapon mode the core
    /// put away for it (`None`: empty hands).
    pub fn enter_rock(&mut self, aim: [f32; 3], had: Option<WeaponMode>, at: &str) -> String {
        self.walker.auto = Some(AutoMove::new(AutoKind::Point, [aim[0], aim[1]], None));
        self.intent.forward = 0;
        self.rock = Some(RockRun { stage: Stage::Turning, aim, had, launched: false });
        let line = format!(
            "enters ThrowRock toward ({:.0}, {:.0}, {:.0}): turns to it first{} (ranged-weapons.md 8b)",
            aim[0],
            aim[1],
            aim[2],
            had.map_or(String::new(), |m| format!("; a weapon was out ({}): away", m.name()))
        );
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// Before his update (8b). Returns lines for `connor.log`.
    pub fn rock_step(&mut self, intent: &mut Intent) -> Vec<String> {
        let mut lines = Vec::new();
        let Some(stage) = self.rock.as_ref().map(|r| r.stage) else { return lines };
        intent.forward = 0;
        intent.turn = 0;
        let free = self.walker.mode == Mode::Free;
        match stage {
            Stage::Turning if self.walker.auto.is_none() && free => {
                if self.set_mode != WeaponMode::None {
                    let l = self.ready_set(None, WeaponMode::None, None, false, false, true);
                    self.rock.as_mut().unwrap().stage = Stage::Away;
                    lines.push(format!("ThrowRock: turned (heading {:.3}); the weapon goes away first: {l} (8b)", self.walker.heading));
                } else {
                    let heading = self.walker.heading;
                    let play = self.rock_play();
                    lines.push(format!("ThrowRock: turned (heading {heading:.3}); {play} (8b)"));
                }
            }
            Stage::Away if free && !self.readying => {
                lines.push(format!("ThrowRock: empty hands; {} (8b)", self.rock_play()));
            }
            Stage::Throwing if free => {
                // 8b: done: the old mode back (the caller asks for it), Stop,
                // Wait.
                let r = self.rock.take().unwrap();
                self.walker.stand_now();
                self.rock_rearm = r.had;
                lines.push(format!(
                    "ThrowRock: 'throw' ended{}; {}; Wait (8b)",
                    if r.launched { "" } else { " (its `do` never fired: no rock)" },
                    r.had.map_or("empty hands before: done".to_string(), |m| format!("the old mode ({}) is asked back", m.name()))
                ));
            }
            _ => {}
        }
        lines
    }

    /// Plays `throw` as a one-item Special list; the log text.
    fn rock_play(&mut self) -> String {
        match self.drawn(THROW).map(|m| m.name) {
            Some(m) => {
                self.walker.start_special(vec![SpecialStep::Play(m, THROW.to_string())]);
                self.rock.as_mut().unwrap().stage = Stage::Throwing;
                "'throw' plays".to_string()
            }
            None => {
                self.rock = None;
                self.walker.stand_now();
                "'throw' is not loaded: no throw, Wait".to_string()
            }
        }
    }

    /// The `do` cue of `throw` (8b): the rock leaves now, once. Returns
    /// whether it did.
    pub fn rock_do(&mut self, played: &str) -> bool {
        let Some(r) = self.rock.as_mut().filter(|r| r.stage == Stage::Throwing && !r.launched && played.eq_ignore_ascii_case(THROW)) else { return false };
        r.launched = true;
        let aim = r.aim;
        self.rock_launches.push((self.mover.position, self.walker.heading, aim));
        true
    }
}
