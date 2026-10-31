//! Connor's `Tile` state: Dead City's death-hall stones
//! (`docs/behaviour/tile-puzzle.md` rules 7-20; batch 2, D-4). The rules
//! that need no viewer are in `kq8_core::tile_puzzle`.
//!
//! Run by [`ConnorSim::tile_step`] before his update, like the Choice
//! state. A jump is our scripted turn to the target (`pointAndCue`'s
//! turn) and then the jump movement as a one-item Special list, whose
//! own steps carry him; a teleport is `teleportdown`, a place, the
//! camera's flight (5,000 units a second, rule 13, Q178) and `teleportup`. What the state asks of the scripts
//! (lines, the camera, a variable, the door click) goes out as
//! [`TileCall`]s, run by `door_view::trap_step`.

use kq8_core::connor::{Intent, Mode, SpecialStep};
use kq8_core::connor_auto::{AutoKind, AutoMove};
use kq8_core::tile_puzzle::{self as tp, Cell, Landing, Lines, Stones, Where};
use kq8_core::weapons::WeaponMode;

use super::ConnorSim;

/// The door the start strip's back row clicks (rule 12).
pub const DOOR: &str = "ltddoor4";
/// What the state asks of the world.
#[derive(Debug, Clone, PartialEq)]
pub enum TileCall {
    /// A script line, run as a command.
    Line(String),
    /// A click on this object, as if the player clicked it (rule 12).
    Click(&'static str),
    /// A script variable set (rule 13, H).
    Set(&'static str, &'static str),
    /// The camera's hold switch (rule 7a, I-4).
    Hold(bool),
    /// Follow Connor again when the camera's flight ends, the hold switch
    /// then this (rule 13, I-4).
    FollowAfterFlight(bool),
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Stage {
    /// The keys: turning on the spot, a jump on a press (rule 8).
    Keys,
    /// The turn to the target before the jump (rule 11).
    Turning { from: Cell, to: Cell, back: bool },
    /// The jump movement playing.
    Jumping { from: Cell, to: Cell },
    /// `teleportdown` playing; then Connor goes to `put` (rules 13, 14).
    Down { right: bool, put: [f32; 3], camera: [f32; 3] },
    /// `teleportup` playing.
    Up { right: bool },
}

/// The state while Connor is in it.
#[derive(Debug, Clone)]
pub struct TileRun {
    stones: Stones,
    /// The letter expected next (index into [`tp::ORDER`]).
    expected: usize,
    /// The exit switch (rules 20, 21).
    pub can_exit: bool,
    lines: Lines,
    stage: Stage,
    /// The stone lit now (rule 10; drawn in D-5).
    pub lit: Option<Cell>,
    /// The remembered wanted movement (rule 8): a jump starts when it
    /// changes; cleared at each landing.
    remembered: i8,
}

impl ConnorSim {
    /// Rule 7: into the `Tile` state. `place_of` finds a stone by name.
    pub fn enter_tile(&mut self, place_of: &dyn Fn(&str) -> Option<[f32; 3]>, at: &str) -> String {
        let (stones, missing) = Stones::read(place_of);
        let found = 64 - missing.len();
        let missing = if missing.is_empty() { String::new() } else { format!("; not found, skipped (Do not reproduce: the original crashes): {}", missing.join(" ")) };
        self.walker.auto = None;
        self.intent.forward = 0;
        let start = if self.set_mode != WeaponMode::None {
            let l = self.ready_set(None, WeaponMode::None, None, false, false, true);
            format!("a weapon was out: away, empty hands ({l})")
        } else {
            self.walker.stand_now();
            "stop".to_string()
        };
        self.tile = Some(TileRun { stones, expected: 0, can_exit: false, lines: Lines::default(), stage: Stage::Keys, lit: None, remembered: 0 });
        self.tile_calls.push(TileCall::Hold(true));
        let spot = self.tile_where();
        let line = format!("enters the Tile state: {found} of 64 stones{missing}; at {}; expects D (tile07); exit switch off; {start} (tile-puzzle.md rules 2, 7)", show(spot));
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    fn tile_where(&self) -> Where {
        tp::where_is(self.mover.position[0], self.mover.position[1])
    }

    /// `tileExit 0|1` (rule 21): the exit switch; with 1 he goes to Wait
    /// at once. Nothing outside the state.
    pub fn tile_exit(&mut self, can_exit: bool, at: &str) -> String {
        let line = match self.tile.as_mut() {
            None => format!("tileExit {}: not in a tile state; nothing (rule 21)", u8::from(can_exit)),
            Some(t) => {
                t.can_exit = can_exit;
                if can_exit {
                    let wait = self.leave_tile();
                    format!("tileExit 1: the exit switch on; {wait} (rule 21)")
                } else {
                    "tileExit 0: the exit switch off (rule 21)".to_string()
                }
            }
        };
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// Out of the state into Wait.
    fn leave_tile(&mut self) -> String {
        if self.tile.as_ref().is_some_and(|t| matches!(t.stage, Stage::Jumping { .. })) {
            self.mover.end_tile_jump();
        }
        self.tile_calls.push(TileCall::Hold(false));
        let lit = self.tile.take().and_then(|t| t.lit);
        if lit.is_some() {
            self.tile_lit_changed = Some(None);
        }
        self.walker.auto = None;
        self.walker.stand_now();
        "leaves the Tile state: Wait".to_string()
    }

    /// Rule 20: while the exit switch is off every other state is refused.
    pub fn tile_holds(&self) -> bool {
        self.tile.as_ref().is_some_and(|t| !t.can_exit)
    }

    /// Before his update (rules 8-15): the keys, the jump, the landing,
    /// the teleports. `intent` is changed for his update (no walking on
    /// the stones); `jump`: the Jump key was pressed (rule 8, Q181: a
    /// forward jump; ignored while a jump, landing or teleport is under
    /// way, "Do not reproduce"). Returns lines for `connor.log`.
    pub fn tile_step(&mut self, intent: &mut Intent, jump: bool) -> Vec<String> {
        let mut lines = Vec::new();
        let Some(stage) = self.tile.as_ref().map(|t| t.stage) else { return lines };
        let spot = self.tile_where();
        let special_over = self.walker.mode == Mode::Free;
        match stage {
            Stage::Keys => {
                // Rule 8: south of the area he walks and runs as usual.
                if spot == Where::South {
                    self.tile.as_mut().unwrap().remembered = intent.forward;
                    self.tile_light(None, &mut lines);
                    return lines;
                }
                let f = std::mem::replace(&mut intent.forward, 0);
                let t = self.tile.as_mut().unwrap();
                let pressed = (f != 0 && f != t.remembered) || jump;
                t.remembered = f;
                let Where::Spot(from) = spot else {
                    self.tile_light(None, &mut lines);
                    return lines;
                };
                let ahead = tp::jump_target(from, self.walker.heading, false);
                self.tile_light(ahead.filter(|c| self.tile.as_ref().unwrap().stones.found(*c)), &mut lines);
                if pressed && self.walker.mode == Mode::Free && self.walker.auto.is_none() {
                    let back = f < 0 && !jump;
                    if jump {
                        lines.push("Tile: the Jump key: a forward jump (rule 8)".to_string());
                    }
                    if let Some(to) = tp::jump_target(from, self.walker.heading, back) {
                        if back {
                            self.tile_light(Some(to).filter(|c| self.tile.as_ref().unwrap().stones.found(*c)), &mut lines);
                        }
                        // Rule 11: face the target, or its mirror image
                        // through him for a back jump.
                        let p = self.mover.position;
                        let target = self.tile.as_ref().unwrap().stones.place(to);
                        let face = if back { [2.0 * p[0] - target[0], 2.0 * p[1] - target[1]] } else { [target[0], target[1]] };
                        self.walker.auto = Some(AutoMove::new(AutoKind::Point, face, None));
                        intent.turn = 0;
                        self.tile.as_mut().unwrap().stage = Stage::Turning { from, to, back };
                        lines.push(format!(
                            "Tile: {} jump from {from} toward {to} (heading {:.3}, {}): turns to ({:.0}, {:.0}) first (rules 9, 11)",
                            if back { "back" } else { "forward" },
                            self.walker.heading,
                            if tp::straight(from, to) { "straight" } else { "diagonal" },
                            face[0],
                            face[1]
                        ));
                    }
                }
            }
            Stage::Turning { from, to, back } => {
                intent.forward = 0;
                intent.turn = 0;
                if self.walker.auto.is_none() {
                    let name = tp::jump_movement(from, to, back);
                    match self.drawn(name).map(|m| m.name) {
                        Some(m) => {
                            self.walker.start_special(vec![SpecialStep::Play(m, name.to_string())]);
                            self.mover.start_tile_jump();
                            self.tile.as_mut().unwrap().stage = Stage::Jumping { from, to };
                            lines.push(format!("Tile: turned (heading {:.3}); {name} plays, gravity and keep-on-ground off while it plays, steps stay 325 (rule 11, Q177)", self.walker.heading));
                        }
                        None => {
                            self.tile.as_mut().unwrap().stage = Stage::Keys;
                            lines.push(format!("Tile: {name} is not loaded; no jump"));
                        }
                    }
                }
            }
            Stage::Jumping { from, to } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    self.mover.end_tile_jump();
                    self.walker.stand_now();
                    self.tile_land(from, to, spot, &mut lines);
                }
            }
            Stage::Down { right, put, camera } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    // Rules 13, 14: put there keeping his heading; the
                    // camera flies to the point, looking at him; held keys
                    // cleared; `teleportup`.
                    let heading = self.walker.heading;
                    self.put(put, heading);
                    self.intent.forward = 0;
                    self.intent.turn = 0;
                    self.tile.as_mut().unwrap().remembered = 0;
                    // Rules 7a, 13: the hold switch off for the flight;
                    // when it ends the camera follows him, held again
                    // (not after H).
                    let t = self.tile.as_ref().unwrap();
                    let solved = right && t.expected >= tp::ORDER.len();
                    self.tile_calls.push(TileCall::Hold(false));
                    self.tile_calls.push(TileCall::Line(format!("KQCamera::addCamLoc {:.0} {:.0} {:.0} 1", camera[0], camera[1], camera[2])));
                    // Rule 13 (Q178): a `flyToCamLoc` with no time:
                    // distance ÷ 5,000 a second, rolling to 0.
                    self.tile_calls.push(TileCall::Line(format!("KQCamera::flyToCamLoc {:.0} {:.0} {:.0}", put[0], put[1], put[2])));
                    self.tile_calls.push(TileCall::FollowAfterFlight(!solved));
                    let up = self.tile_play("teleportup");
                    self.tile.as_mut().unwrap().stage = Stage::Up { right };
                    lines.push(format!(
                        "Tile: teleportdown ended: put at ({:.0}, {:.0}, {:.0}), heading {heading:.3}; the camera flies to ({:.0}, {:.0}, {:.0}) at 5,000 a second (rule 13); {up}",
                        put[0], put[1], put[2], camera[0], camera[1], camera[2]
                    ));
                }
            }
            Stage::Up { right } => {
                intent.forward = 0;
                intent.turn = 0;
                if special_over {
                    self.walker.stand_now();
                    let t = self.tile.as_mut().unwrap();
                    let said = t.lines.after_teleport(right);
                    let solved = right && t.expected >= tp::ORDER.len();
                    if let Some(c) = said {
                        self.tile_calls.push(TileCall::Line(format!("KQMonster::speak Connor Connor {} {} 0 {c} 1 1", tp::LINE_FILE, tp::LINE_NOUN)));
                    }
                    // Rule 13: the keys work again now, even if the
                    // camera is still flying (it follows him when the
                    // flight ends).
                    let said = said.map_or("no line".to_string(), |c| format!("says {}/{}/0/{c} (rule 15)", tp::LINE_FILE, tp::LINE_NOUN));
                    if solved {
                        // Rule 13, H: Wait after `teleportup` (the exit
                        // switch and Death's trap were set at the landing).
                        let wait = self.leave_tile();
                        lines.push(format!("Tile: teleportup ended: {said}; SOLVED: {wait} (rule 13)"));
                    } else {
                        self.tile.as_mut().unwrap().stage = Stage::Keys;
                        lines.push(format!("Tile: teleportup ended: {said}; keys (the camera follows when its flight ends, rule 13)"));
                    }
                }
            }
        }
        lines
    }

    /// Rule 12: what the landing means.
    fn tile_land(&mut self, from: Cell, to: Cell, spot: Where, lines: &mut Vec<String>) {
        let t = self.tile.as_mut().unwrap();
        t.remembered = 0;
        t.stage = Stage::Keys;
        let expected = t.expected;
        let landing = tp::landing(spot, expected);
        let p = self.mover.position;
        let head = format!(
            "Tile: landed from {from} (aimed at {to}) at ({:.0}, {:.0}, {:.0}) = {}; expected {} {}",
            p[0],
            p[1],
            p[2],
            show(spot),
            tp::ORDER[expected].letter,
            tp::ORDER[expected].stone
        );
        match landing {
            Landing::Right | Landing::Wrong => {
                let right = landing == Landing::Right;
                let (put, camera) = tp::teleport_places(&t.stones, landing, expected);
                t.expected = if right { expected + 1 } else { 0 };
                // Rule 13, H: at the landing, before `teleportdown`, the
                // exit is allowed and Death's trap armed, so its `On` when
                // he is put inside it finds the variable true (F-2).
                let solved = right && t.expected >= tp::ORDER.len();
                if solved {
                    t.can_exit = true;
                    self.tile_calls.push(TileCall::Set("DeadCity::Azriel::TrapOn", "true"));
                    lines.push("Tile: H: exit switch on, DeadCity::Azriel::TrapOn true (rule 13)".to_string());
                }
                if !right {
                    self.tile_light(None, lines);
                }
                self.tile_calls.push(TileCall::Line("setThirdPerson".into()));
                let down = self.tile_play("teleportdown");
                self.tile.as_mut().unwrap().stage = Stage::Down { right, put, camera };
                lines.push(format!(
                    "{head}: {} (rule {}); third person; {down}",
                    if right { "RIGHT" } else { "WRONG: back to the start, expects D again, no damage" },
                    if right { 13 } else { 14 }
                ));
            }
            Landing::Door => {
                self.tile_calls.push(TileCall::Click(DOOR));
                lines.push(format!("{head}: the row behind the start strip: {DOOR} gets a click (rules 12, 19)"));
            }
            Landing::StartStrip => lines.push(format!("{head}: the start strip: nothing (rule 12)")),
        }
    }

    /// One movement of his own as a one-item Special list.
    fn tile_play(&mut self, name: &str) -> String {
        match self.drawn(name).map(|m| m.name) {
            Some(m) => {
                self.walker.start_special(vec![SpecialStep::Play(m, name.to_string())]);
                format!("{name} plays")
            }
            None => format!("{name} is not loaded; skipped"),
        }
    }

    /// Rule 10: the lit stone; a change is logged and left for the
    /// drawing (D-5).
    fn tile_light(&mut self, cell: Option<Cell>, lines: &mut Vec<String>) {
        let Some(t) = self.tile.as_mut() else { return };
        if t.lit != cell {
            lines.push(format!("Tile: lit stone {} -> {} (rule 10)", t.lit.map_or("none".into(), |c| c.to_string()), cell.map_or("none".into(), |c| c.to_string())));
            t.lit = cell;
            self.tile_lit_changed = Some(cell);
        }
    }
}

fn show(w: Where) -> String {
    match w {
        Where::Spot(c) => c.to_string(),
        other => format!("{other:?}").to_lowercase(),
    }
}

/// The lit stone's parts drawn with their own material copies (rule 10,
/// D-5): the part entity, its own material and the copy put on.
#[derive(Default)]
pub struct Unlit {
    name: Option<String>,
    parts: std::collections::BTreeMap<bevy::prelude::Entity, (bevy::prelude::Handle<crate::object_light::ObjectMaterial>, bevy::prelude::Handle<crate::object_light::ObjectMaterial>)>,
    made: std::collections::BTreeMap<bevy::prelude::AssetId<crate::object_light::ObjectMaterial>, bevy::prelude::Handle<crate::object_light::ObjectMaterial>>,
    /// Copies put on again after the drawing put the own material back,
    /// since the last log line (counted, not logged one by one).
    again: u64,
}

/// Every drawn frame, after the drawing (as `dither_step`): the stone the
/// Tile state lit is drawn with its smooth materials at full brightness
/// (`object-lighting.md` rule 10: the per-object "unlit" switch); the
/// one before gets its own materials back. A part whose drawing put its
/// own material back (a new frame) gets the copy again.
#[allow(clippy::type_complexity)]
pub fn unlit_step(
    connor: Option<bevy::prelude::Res<ConnorSim>>,
    mut parts: bevy::prelude::Query<(bevy::prelude::Entity, Option<&crate::terrain_view::ObjectName>, Option<&bevy::prelude::ChildOf>, &mut bevy::prelude::MeshMaterial3d<crate::object_light::ObjectMaterial>)>,
    names: bevy::prelude::Query<(Option<&crate::terrain_view::ObjectName>, Option<&bevy::prelude::ChildOf>)>,
    mut materials: bevy::prelude::ResMut<bevy::prelude::Assets<crate::object_light::ObjectMaterial>>,
    mut state: bevy::prelude::Local<Unlit>,
    shared: bevy::prelude::Res<crate::Shared>,
) {
    use bevy::prelude::{ChildOf, DetectChanges};
    let wanted = connor.as_ref().and_then(|c| c.tile.as_ref()).and_then(|t| t.lit).map(|c| c.name());
    if wanted.is_none() && state.parts.is_empty() {
        return;
    }
    let all = wanted != state.name;
    let (mut on, mut off, mut again) = (0, 0, 0);
    let mut kinds = Vec::new();
    let Unlit { parts: done, made, .. } = &mut *state;
    for (e, own_name, parent, mut m) in &mut parts {
        if !all && !m.is_changed() {
            continue;
        }
        let named = |e: bevy::prelude::Entity| names.get(e).ok().and_then(|(o, _)| o.map(|o| &o.0));
        let up = |e: bevy::prelude::Entity| names.get(e).ok().and_then(|(_, c)| c.map(ChildOf::parent));
        let parent = parent.map(ChildOf::parent);
        let Some(name) = own_name.map(|o| &o.0).or_else(|| parent.and_then(named)).or_else(|| parent.and_then(up).and_then(named)) else { continue };
        let lit = wanted.as_ref().is_some_and(|w| w.eq_ignore_ascii_case(name));
        let redrawn = done.get(&e).is_some_and(|(_, put)| *put != m.0);
        if redrawn {
            done.remove(&e);
        }
        if done.contains_key(&e) == lit {
            continue;
        }
        if let Some((own, _)) = done.remove(&e) {
            m.0 = own;
            off += 1;
        }
        if lit {
            let own = m.0.clone();
            let copy = made
                .entry(own.id())
                .or_insert_with(|| {
                    let mut c = materials.get(&own).cloned().expect("a drawn part's material");
                    kinds.push(c.settings.lighting);
                    // Rule 10: smooth materials at full brightness.
                    if c.settings.lighting == 2 {
                        c.settings.lighting = 0;
                    }
                    materials.add(c)
                })
                .clone();
            // D-6: a changeBmp on the stone (the Reveal potion) changes its
            // own material's picture; the bright copy follows it.
            let own_texture = materials.get(&own).and_then(|o| o.texture.clone());
            if materials.get(&copy).is_some_and(|c| c.texture != own_texture) {
                if let Some(mut c) = materials.get_mut(&copy) {
                    c.texture = own_texture;
                }
            }
            m.0 = copy.clone();
            done.insert(e, (own, copy));
            if redrawn {
                again += 1;
            } else {
                on += 1;
            }
        }
    }
    state.again += again;
    if on + off > 0 || all {
        let again = std::mem::take(&mut state.again);
        shared.log.lock().unwrap().detail(format!(
            "unlit: {:?} -> {wanted:?}: {on} part(s) at full brightness (new copies of lighting kinds {kinds:?}; 2 = smooth -> none), {off} back to their own; {again} put on again after a new frame since the last line (object-lighting.md rule 10, tile-puzzle.md rule 10)",
            state.name
        ));
        state.name = wanted;
    }
}
