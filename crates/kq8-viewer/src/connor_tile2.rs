//! Connor's `Tile2` state: Swamp's gas/stump puzzle tiles (`docs/behaviour/tile-puzzle.md`
//! rules 29–35; `ranged-weapons.md` 8e; Batch 3 B3-3). Rules that need no viewer live in
//! `kq8_core::tile2`.
//!
//! Mirrors `connor_tile.rs` (`TileRun`), without teleports or camera hold, adding `Die` exit while
//! locked (rule 31), plain `jump` off West/East (`100`/`200`), row-3 `fiendoor` Cue 1 / `Rtswdoor`
//! `ConnorAction "do"` exits (rule 34), and built-in rock throwing (`handsOff 1` -> `away` ->
//! `throw` -> `handsOff 0`, staying in `Tile2`, rule 35).

use kq8_core::connor::{Intent, Mode, SpecialStep};
use kq8_core::connor_auto::{AutoKind, AutoMove};
use kq8_core::tile2::{self, Cell2, Landing2, Tile2Grid, Where2};
use kq8_core::weapons::WeaponMode;

use super::{tile::TileCall, ConnorSim};

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage2 {
    Keys,
    Turning { from: Cell2, to: Cell2, back: bool },
    Jumping { from: Option<Cell2>, to: Option<Cell2> },
    RockTurning { aim: [f32; 3] },
    RockAway { aim: [f32; 3] },
    RockThrowing { aim: [f32; 3], launched: bool },
}

/// Connor's `Tile2` runtime state.
#[derive(Debug, Clone)]
pub struct Tile2Run {
    grid: Tile2Grid,
    pub can_exit: bool,
    stage: Stage2,
    lit: Option<Cell2>,
    remembered: i8,
}

impl Tile2Run {
    /// Object name of the lit stone (`tile<col><row>`), if any.
    pub fn lit_name(&self) -> Option<String> {
        self.lit.and_then(|c| c.tile_name())
    }
}

impl ConnorSim {
    /// Rule 31: enter `Tile2`. Builds the 10×7 grid from `place_of`, sets exit switch off,
    /// puts away any drawn weapon (`0/0/37/23` spoken by caller if weapon was out).
    pub fn enter_tile2(&mut self, place_of: &dyn Fn(&str) -> Option<[f32; 3]>, at: &str) -> String {
        let (grid, missing) = Tile2Grid::read(place_of);
        let found = 42 - missing.len();
        let missing = if missing.is_empty() {
            String::new()
        } else {
            format!("; not found, skipped (Do not reproduce: the original crashes): {}", missing.join(" "))
        };
        self.walker.auto = None;
        self.intent.forward = 0;
        let start = if self.set_mode != WeaponMode::None {
            let l = self.ready_set(None, WeaponMode::None, None, false, false, true);
            format!("a weapon was out: away, empty hands ({l})")
        } else {
            self.walker.stand_now();
            "stop".to_string()
        };
        self.tile2 = Some(Tile2Run { grid, can_exit: false, stage: Stage2::Keys, lit: None, remembered: 0 });
        let spot = self.tile2_where();
        let line = format!(
            "enters the Tile2 state: {found} of 42 Swamp tiles{missing}; at {}; exit switch off; {start} (tile-puzzle.md rules 29–31)",
            show2(spot)
        );
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    fn tile2_where(&self) -> Where2 {
        let p = self.mover.position;
        self.tile2.as_ref().map_or(Where2::South, |t| t.grid.where_is(p[0], p[1]))
    }

    /// Out of `Tile2` into `Wait` (called by `tileExit 1` or on `Die`).
    pub fn leave_tile2(&mut self) -> String {
        let was_throwing = self.tile2.as_ref().is_some_and(|t| {
            matches!(t.stage, Stage2::RockTurning { .. } | Stage2::RockAway { .. } | Stage2::RockThrowing { .. })
        });
        if self.tile2.as_ref().is_some_and(|t| matches!(t.stage, Stage2::Jumping { .. })) {
            self.mover.end_tile_jump();
        }
        if was_throwing {
            self.tile_calls.push(TileCall::HandsOff(false));
        }
        self.tile2.take();
        self.walker.auto = None;
        self.walker.stand_now();
        "leaves the Tile2 state: Wait".to_string()
    }

    /// Rule 35 & `ranged-weapons.md` 8e: starts `Tile2`'s built-in rock throw (`handsOff 1` ->
    /// `away` -> `throw` -> `handsOff 0`) without leaving `Tile2`.
    pub fn enter_tile2_rock(&mut self, aim: [f32; 3], at: &str) -> String {
        let Some(t) = self.tile2.as_mut() else {
            return "not in Tile2; ignored".into();
        };
        if t.stage != Stage2::Keys {
            let line = format!("Tile2 rock click at ({:.0}, {:.0}, {:.0}): busy ({:?}); ignored (rule 35)", aim[0], aim[1], aim[2], t.stage);
            self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
            return line;
        }
        t.stage = Stage2::RockTurning { aim };
        t.remembered = 0;
        self.walker.auto = Some(AutoMove::new(AutoKind::Point, [aim[0], aim[1]], None));
        self.intent.forward = 0;
        self.intent.turn = 0;
        self.tile_calls.push(TileCall::HandsOff(true));
        let line = format!("Tile2: built-in rock throw toward ({:.0}, {:.0}, {:.0}): handsOff 1, turns first (tile-puzzle.md rule 35)", aim[0], aim[1], aim[2]);
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// Fires the `do` cue during `Tile2`'s built-in rock throw (`rule 35`).
    pub fn tile2_rock_do(&mut self, played: &str) -> bool {
        let Some(t) = self.tile2.as_mut() else { return false };
        let Stage2::RockThrowing { aim, launched } = &mut t.stage else { return false };
        if *launched || !played.eq_ignore_ascii_case(super::ROCK_THROW) {
            return false;
        }
        *launched = true;
        let aim = *aim;
        self.rock_launches.push((self.mover.position, self.walker.heading, aim));
        true
    }

    /// Before Connor's physics update (`rules 32–35`).
    pub fn tile2_step(&mut self, intent: &mut Intent, jump: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let Some(stage) = self.tile2.as_ref().map(|t| t.stage) else { return lines };
        let spot = self.tile2_where();
        let special_over = self.walker.mode == Mode::Free;
        match stage {
            Stage2::Keys => {
                let f = std::mem::replace(&mut intent.forward, 0);
                let pressed = {
                    let t = self.tile2.as_mut().unwrap();
                    let p = (f != 0 && f != t.remembered) || jump;
                    t.remembered = f;
                    p
                };
                match spot {
                    Where2::West | Where2::East => {
                        self.tile2_light(None, &mut lines);
                        if pressed && special_over && self.walker.auto.is_none() {
                            // Rule 32: on West (100) or East (200), play plain `jump` without turning first.
                            self.mover.start_tile_jump();
                            let play = self.tile2_play("jump");
                            self.tile2.as_mut().unwrap().stage = Stage2::Jumping { from: None, to: None };
                            lines.push(format!("Tile2: outside {}: {play} without turning (rule 32)", show2(spot)));
                        }
                    }
                    Where2::Spot(from) => {
                        let ahead = tile2::jump_target(from, self.walker.heading, false)
                            .filter(|c| self.tile2.as_ref().unwrap().grid.has_tile(*c));
                        self.tile2_light(ahead, &mut lines);
                        if pressed && special_over && self.walker.auto.is_none() {
                            let back = f < 0 && !jump;
                            if jump {
                                lines.push("Tile2: the Jump key: a forward jump (rule 32)".to_string());
                            }
                            if let Some(to) = tile2::jump_target(from, self.walker.heading, back) {
                                if back {
                                    let lit = Some(to).filter(|c| self.tile2.as_ref().unwrap().grid.has_tile(*c));
                                    self.tile2_light(lit, &mut lines);
                                }
                                let p = self.mover.position;
                                let target = self.tile2.as_ref().unwrap().grid.place(to);
                                let face = if back { [2.0 * p[0] - target[0], 2.0 * p[1] - target[1]] } else { [target[0], target[1]] };
                                self.walker.auto = Some(AutoMove::new(AutoKind::Point, face, None));
                                intent.turn = 0;
                                self.tile2.as_mut().unwrap().stage = Stage2::Turning { from, to, back };
                                lines.push(format!(
                                    "Tile2: {} jump from {from} toward {to} (heading {:.3}, {}): turns to ({:.0}, {:.0}) first (rules 32–33)",
                                    if back { "back" } else { "forward" },
                                    self.walker.heading,
                                    if tile2::straight(from, to) { "straight" } else { "diagonal" },
                                    face[0],
                                    face[1]
                                ));
                            }
                        }
                    }
                    Where2::North | Where2::South => {
                        self.tile2_light(None, &mut lines);
                    }
                }
            }
            Stage2::Turning { from, to, back } => {
                intent.forward = 0;
                intent.turn = 0;
                if self.walker.auto.is_none() {
                    let name = tile2::jump_movement(from, to, back);
                    let play = self.tile2_play(name);
                    if play.ends_with("plays") {
                        self.mover.start_tile_jump();
                        self.tile2.as_mut().unwrap().stage = Stage2::Jumping { from: Some(from), to: Some(to) };
                        lines.push(format!("Tile2: turned (heading {:.3}); {play}, gravity & keep-on-ground off (rule 33)", self.walker.heading));
                    } else {
                        self.tile2.as_mut().unwrap().stage = Stage2::Keys;
                        lines.push(format!("Tile2: {play}; no jump"));
                    }
                }
            }
            Stage2::Jumping { from, to } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    self.mover.end_tile_jump();
                    self.walker.stand_now();
                    self.tile2_land(from, to, spot, &mut lines);
                }
            }
            Stage2::RockTurning { aim } => {
                intent.forward = 0;
                intent.turn = 0;
                if self.walker.auto.is_none() && special_over {
                    let play = self.tile2_play("away");
                    if play.ends_with("plays") {
                        self.tile2.as_mut().unwrap().stage = Stage2::RockAway { aim };
                        lines.push(format!("Tile2 rock throw: turned (heading {:.3}); {play} with Prrock.kq (rule 35)", self.walker.heading));
                    } else {
                        self.tile_calls.push(TileCall::HandsOff(false));
                        self.tile2.as_mut().unwrap().stage = Stage2::Keys;
                        lines.push(format!("Tile2 rock throw: {play}; aborted, handsOff 0"));
                    }
                }
            }
            Stage2::RockAway { aim } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    let play = self.tile2_play(super::ROCK_THROW);
                    if play.ends_with("plays") {
                        self.tile2.as_mut().unwrap().stage = Stage2::RockThrowing { aim, launched: false };
                        lines.push(format!("Tile2 rock throw: away ended; {play} (rule 35)"));
                    } else {
                        self.tile_calls.push(TileCall::HandsOff(false));
                        self.tile2.as_mut().unwrap().stage = Stage2::Keys;
                        lines.push(format!("Tile2 rock throw: {play}; aborted, handsOff 0"));
                    }
                }
            }
            Stage2::RockThrowing { launched, .. } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    self.walker.stand_now();
                    self.tile_calls.push(TileCall::HandsOff(false));
                    if let Some(t) = self.tile2.as_mut() {
                        t.stage = Stage2::Keys;
                        t.remembered = 0;
                    }
                    lines.push(format!(
                        "Tile2 rock throw: 'throw' ended{}; handsOff 0, stays in Tile2 (rule 35)",
                        if launched { "" } else { " (its `do` never fired)" }
                    ));
                }
            }
        }
        lines
    }

    /// Follow-on immediately after a `Tile2` jump finishes (`stop` skipped, matching `Tile`).
    pub fn tile2_follow_on(&mut self) -> Vec<String> {
        let ended = self.walker.mode == Mode::Free && self.tile2.as_ref().is_some_and(|t| matches!(t.stage, Stage2::Jumping { .. }));
        if !ended {
            return Vec::new();
        }
        let mut idle = Intent { forward: 0, turn: 0, ..self.intent };
        self.tile2_step(&mut idle, false)
    }

    fn tile2_land(&mut self, from: Option<Cell2>, to: Option<Cell2>, spot: Where2, lines: &mut Vec<String>) {
        let t = self.tile2.as_mut().unwrap();
        t.remembered = 0;
        t.stage = Stage2::Keys;
        let p = self.mover.position;
        let via = match (from, to) {
            (Some(f), Some(t)) => format!("from {f} (aimed at {t})"),
            _ => "plain jump".into(),
        };
        let head = format!("Tile2: landed {via} at ({:.0}, {:.0}, {:.0}) = {}", p[0], p[1], p[2], show2(spot));
        match tile2::landing(spot) {
            Landing2::WestDoor(c) => {
                self.tile_calls.push(TileCall::Line(format!("sendEvent 0 {} KQCueEvent 1 0", tile2::WEST_DOOR)));
                lines.push(format!("{head}: cell {} -> Cue 1 to {} (rule 34)", c.index(), tile2::WEST_DOOR));
            }
            Landing2::EastDoor(c) => {
                self.tile_calls.push(TileCall::Click(tile2::EAST_DOOR));
                lines.push(format!("{head}: cell {} -> ConnorAction do to {} (rule 34)", c.index(), tile2::EAST_DOOR));
            }
            Landing2::Stay => {
                let ahead = match spot {
                    Where2::Spot(c) => tile2::jump_target(c, self.walker.heading, false).filter(|n| t.grid.has_tile(*n)),
                    _ => None,
                };
                self.tile2_light(ahead, lines);
                lines.push(format!("{head}: stays in Tile2 (rule 34)"));
            }
        }
    }

    fn tile2_play(&mut self, name: &str) -> String {
        match self.drawn(name).map(|m| m.name) {
            Some(m) => {
                self.walker.start_special(vec![SpecialStep::Play(m, name.to_string())]);
                format!("{name} plays")
            }
            None => format!("{name} is not loaded; skipped"),
        }
    }

    fn tile2_light(&mut self, cell: Option<Cell2>, lines: &mut Vec<String>) {
        let Some(t) = self.tile2.as_mut() else { return };
        if t.lit != cell {
            lines.push(format!(
                "Tile2: lit stone {} -> {} (rule 33)",
                t.lit.map_or("none".into(), |c| c.to_string()),
                cell.map_or("none".into(), |c| c.to_string())
            ));
            t.lit = cell;
        }
    }
}

fn show2(w: Where2) -> String {
    match w {
        Where2::Spot(c) => c.to_string(),
        Where2::West => "west (100)".into(),
        Where2::East => "east (200)".into(),
        Where2::South => "south (300)".into(),
        Where2::North => "north (400)".into(),
    }
}
