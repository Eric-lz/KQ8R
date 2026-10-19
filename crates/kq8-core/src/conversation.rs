//! Conversations, the command and Connor's talk state (M4 step 4e-3).
//!
//! Rules from the behaviour note `docs/behaviour/conversation.md` (session
//! A); rule numbers below are that note's. See docs/provenance/README.md.
//!
//! - `KQConner::conversation <partner> <file> <noun> <verb> <case> <first>
//!   <last> <enemy>` (rules 1–2): the partner's `speak` queue is emptied,
//!   its conversation queue made (with its goodbye and welcome-back lines),
//!   the lines prepared, the partner put into its Talk state; Connor is
//!   "sent a start message", handled after the scripts of the update
//!   ([`Active::conversation_tick`]).
//! - Connor's talk state (rules 6–11): his `speak` queue emptied, the
//!   bottom bar put away; the case from where he stands: in the stay area
//!   (`box1`) and under 125 from the spot "there", in the stay area
//!   "close", in the start area (`box0`) "walk". "There": the partner is
//!   told to start. "Close": the same, and he is put on the spot (heading
//!   kept). "Walk": he turns toward the spot and plays `stopwalk`; at its
//!   end he is put 440 short of the spot on the line from where he stood,
//!   at the spot's height, facing it, and plays `walkstopright` (the
//!   partner is told to start then); at its end he turns to the partner.
//! - The stay test (rule 13): from then on, every update, outside the stay
//!   area ends the conversation ("walked away"; the goodbye, 4e-6).
//! - The end (rules 15–16): Move if he is moving, else `Stop` and Wait; the
//!   camera put at `cam0` looking at the partner, then following Connor;
//!   `TalkExit` to the partner's script (unless a goodbye plays: then at
//!   its end); the partner to Wait; the bottom bar back.
//!
//! "Do not reproduce": no info object or no case → treated as "there" (no
//! cameras, no stay test without an info object); walking away on the last
//! line → `TalkExit` sent once anyway. STAND-INS (Q125): the keys are
//! ignored during the approach movements; no Jump or Push in the talk
//! state (`connor.rs`). Weapons are not built: he never holds one drawn,
//! so `away` / `out` (rule 7) never play (logged). First person is not
//! built: he is always in third person (logged).

use kq8_data::conv_info::ConvInfo;

use crate::active::{Active, GuiOrder, Request, WorldState};
use crate::connor::{Walker, EMPTY_HANDS};
use crate::connor_auto::{heading_towards, AutoKind, AutoMove};
use crate::talk::{Goodbye, Speak, CONNOR};

/// Rule 6: case "there" under this distance (3D) from the talking spot.
pub const THERE_DISTANCE: f32 = 125.0;
/// Rule 10: case "walk" puts him this far short of the spot.
pub const WALK_IN_SHORT: f32 = 440.0;
/// The approach movements (rule 10).
pub const STOP_WALK: &str = "stopwalk";
pub const WALK_STOP_RIGHT: &str = "walkstopright";
/// Our guard: an approach movement that has not ended after this many
/// updates (not loaded, or held) is given up and the next step taken.
pub const APPROACH_GUARD: u32 = 150;

/// Rule 6: how Connor gets to his talking spot.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Case {
    There,
    Close,
    Walk,
}

/// Where Connor's talk state is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage {
    /// The command was given; he has not entered his talk state yet.
    Asked,
    /// Case "walk": `stopwalk` plays while he turns toward the spot.
    StopWalk,
    /// Case "walk": put 440 short; `walkstopright` plays.
    WalkStopRight,
    /// At his spot (or where he stands): the stay test runs (rule 13).
    Talking,
}

/// One conversation, Connor's side.
#[derive(Debug, Clone)]
pub struct Conversation {
    /// The partner's name as the world writes it.
    pub partner: String,
    /// The partner's position when the command was given (rule 2).
    pub partner_at: [f32; 3],
    /// Its info object (`None`: Do not reproduce, treated as "there").
    pub info: Option<ConvInfo>,
    /// The lines `<first>`…`<last>`, queued when Connor is ready.
    pub lines: Speak,
    /// `<enemy>` not 0: the weapon stays drawn (rule 7; not in the data).
    pub keep_weapon: bool,
    pub case: Option<Case>,
    pub stage: Stage,
    /// Where he stood when he entered (case "walk", rule 10).
    pub walk_from: [f32; 3],
    /// Updates spent in the approach movement now playing (our guard).
    pub approach_updates: u32,
    /// Where the command was given (for the logs).
    pub at: String,
}

impl Conversation {
    /// Connor is in his talk state.
    pub fn entered(&self) -> bool {
        self.stage != Stage::Asked
    }

    /// Connor has reached his spot (rule 14): case "there" or "close", or
    /// `walkstopright` has ended.
    pub fn reached(&self) -> bool {
        self.stage == Stage::Talking
    }
}

/// Rule 6: the case from where Connor stands, and a line for the log.
pub fn case_for(info: &ConvInfo, at: [f32; 3]) -> (Option<Case>, String) {
    let p = [at[0], at[1]];
    let (start, stay) = (info.in_area(0, p), info.in_area(1, p));
    let d = (0..3).map(|k| (at[k] - info.spot[k]).powi(2)).sum::<f32>().sqrt();
    let case = if stay && d < THERE_DISTANCE {
        Some(Case::There)
    } else if stay {
        Some(Case::Close)
    } else if start {
        Some(Case::Walk)
    } else {
        None
    };
    let line = format!(
        "Connor at ({:.0}, {:.0}, {:.0}): start area {}, stay area {}, {d:.0} from the spot ({:.0}, {:.0}, {:.0}) (3D): case {}",
        at[0],
        at[1],
        at[2],
        if start { "in" } else { "out" },
        if stay { "in" } else { "out" },
        info.spot[0],
        info.spot[1],
        info.spot[2],
        match case {
            Some(Case::There) => "\"there\" (under 125)",
            Some(Case::Close) => "\"close\"",
            Some(Case::Walk) => "\"walk\"",
            None => "none",
        }
    );
    (case, line)
}

/// Rule 10: the point 440 short of `spot` on the line from `from`, at the
/// spot's height, and the heading facing the spot from there. `from` on
/// the spot (flat): the point is the spot itself (our choice).
pub fn walk_in_point(from: [f32; 3], spot: [f32; 3]) -> ([f32; 3], f32) {
    let (dx, dy) = (spot[0] - from[0], spot[1] - from[1]);
    let len = (dx * dx + dy * dy).sqrt();
    if len < 1e-3 {
        return (spot, heading_towards(from, [spot[0], spot[1]]));
    }
    let p = [spot[0] - dx / len * WALK_IN_SHORT, spot[1] - dy / len * WALK_IN_SHORT, spot[2]];
    (p, heading_towards(p, [spot[0], spot[1]]))
}

/// The point a conversation camera looks at: the partner's position plus
/// the following camera's 1,400 (Q126 stand-in).
pub fn partner_aim(partner_at: [f32; 3]) -> [f32; 3] {
    crate::camera::Camera::aim_for(partner_at)
}

impl WorldState {
    /// Connor is in his talk state now.
    pub fn connor_in_talk(&self) -> bool {
        self.conversation.as_ref().is_some_and(Conversation::entered)
    }

    /// `talking.md` rule 8: a `speak` of Connor or of the partner is
    /// ignored while either is in its talk state.
    pub(crate) fn speak_ignored(&self, issuer: &str) -> bool {
        let Some(c) = &self.conversation else { return false };
        let partner_talks = self.character_index(&c.partner).is_some_and(|i| self.anims[i].character.as_ref().is_some_and(|ch| ch.state == crate::character::State::Talk));
        let either = c.entered() || partner_talks;
        either && (issuer.eq_ignore_ascii_case(CONNOR) || issuer.eq_ignore_ascii_case(&c.partner))
    }

    /// A line for `conversation.log` (and the current log).
    pub(crate) fn conversation_note(&mut self, line: impl AsRef<str>) {
        let line = line.as_ref().to_string();
        self.conversation_log.push(format!("update {}: {line}", self.updates));
        self.talk.log.push(format!("t {:8.3}: conversation: {line}", self.now()));
        self.note(format!("conversation: {line}"));
    }

    /// `KQConner::conversation <partner> <file> <noun> <verb> <case>
    /// <first> <last> <enemy>` (rules 1–2).
    pub(crate) fn conversation_command(&mut self, words: &[String], at: &str) -> String {
        let usage = "usage: KQConner::conversation <monster> <file> <noun> <verb> <case> <first> <last> <enemy>";
        if words.len() < 9 {
            self.note(format!("{at}: {usage} (got {} words); nothing (rule 1)", words.len()));
            return "1".into();
        }
        let Some(partner) = self.monster_name(&words[1]).filter(|n| !n.eq_ignore_ascii_case(CONNOR)) else {
            self.note(format!("{at}: console: '{}' is not KQMonster; nothing (rule 1)", words[1]));
            return "1".into();
        };
        let Some(first) = crate::active::tuple_of(&words[2..7]) else {
            self.note(format!("{at}: {usage}: a number is not a number: '{}'; nothing", words.join(" ")));
            return "1".into();
        };
        let last_word = crate::script::number_value(&words[7]) as i64 as u8;
        let last = if last_word == 0 { self.talk.book.last_of_run(first).unwrap_or(first.seq) } else { last_word };
        let keep_weapon = crate::script::number_value(&words[8]) != 0.0;
        let now = self.now();
        let mut notes = vec![format!("{at}: KQConner::conversation {partner} {first}…{last}{}{}", if last_word == 0 { " (<last> 0)" } else { "" }, if keep_weapon { ", <enemy> not 0: the weapon stays drawn" } else { "" })];
        // Rule 2: the partner's speak queue emptied, no TalkComplete.
        notes.push(format!("partner's speak queue: {}", self.talk.clear(&partner, now)));
        let (goodbye, welcome_back) = self.monsters.iter().find(|m| m.name.eq_ignore_ascii_case(&partner)).map_or((None, None), |m| (m.goodbye, m.welcome_back));
        notes.push(self.talk.conv_queue(&partner, goodbye, welcome_back, now));
        // Rule 2: Stop and the Talk state.
        match self.character_index(&partner) {
            Some(i) => {
                let a = &mut self.anims[i];
                let line = a.character.as_mut().unwrap().enter_talk(&mut a.player);
                self.character_note(&partner, &line);
                notes.push(format!("partner: {line}"));
            }
            None => notes.push("the partner is not a drawn character: no Talk state".into()),
        }
        let info = self.conv_infos.iter().find(|i| i.partner().eq_ignore_ascii_case(&partner)).cloned();
        if info.is_none() {
            notes.push(format!("no {partner}Info object (rule 4: the original would crash; Do not reproduce: treated as case \"there\", no cameras)"));
        }
        let partner_at = self.object_point(&partner).unwrap_or([0.0; 3]);
        let lines = Speak { issuer: partner.clone(), first, last, nofade: false };
        let c = Conversation { partner: partner.clone(), partner_at, info, lines, keep_weapon, case: None, stage: Stage::Asked, walk_from: [0.0; 3], approach_updates: 0, at: at.to_string() };
        match &self.conversation {
            Some(old) if old.entered() => {
                // "Edge cases": Connor's state can't be entered again; the
                // partner is not told to start (not traced further).
                notes.push(format!("Connor is still in his talk state with {}: it can't be entered again; no case, the partner is not told to start (Edge cases)", old.partner));
            }
            Some(old) => {
                notes.push(format!("replaces the conversation with {} not entered yet", old.partner));
                self.conversation = Some(c);
            }
            None => {
                notes.push(format!("Connor is sent the start message (handled after the scripts); the partner at ({:.0}, {:.0}, {:.0})", partner_at[0], partner_at[1], partner_at[2]));
                self.conversation = Some(c);
            }
        }
        let line = notes.join("; ");
        self.conversation_note(&line);
        "0".into()
    }
}

impl Active {
    /// Connor's side of a conversation, once per update after his move
    /// (the caller: the viewer, before it applies the requests). `may_enter`:
    /// his state lets him into the talk state now (an unfinished jump
    /// doesn't, `anim-scripts.md` rule 28: tried again next update).
    /// Returns nothing; lines go to `conversation_log`.
    pub fn conversation_tick(&mut self, walker: &mut Walker, position: [f32; 3], may_enter: Result<(), String>) {
        if let Some(why) = self.world.connor_leave.take() {
            let to = walker.leave_talk();
            self.world.conversation_note(format!("Connor leaves his talk state ({why}): {to} (rule 15)"));
        }
        let Some(stage) = self.world.conversation.as_ref().map(|c| c.stage) else { return };
        match stage {
            Stage::Asked => match may_enter {
                Ok(()) => self.conversation_enter(walker, position),
                Err(why) => self.world.conversation_note(format!("Connor can't go into his talk state now ({why}); tried again next update")),
            },
            Stage::StopWalk | Stage::WalkStopRight => self.approach(walker, position, stage),
            Stage::Talking => self.stay_test(walker, position),
        }
    }

    /// Rule 2's start message and rule 6: Connor goes into his talk state.
    fn conversation_enter(&mut self, walker: &mut Walker, position: [f32; 3]) {
        let now = self.world.now();
        let mut notes = Vec::new();
        notes.push(format!("Connor's speak queue: {}", self.world.talk.clear(CONNOR, now)));
        notes.push(walker.enter_talk());
        // interface.md rule 16: the bottom bar's panels away (not during
        // handsOff: the panels decide).
        self.world.gui_orders.push(GuiOrder::Conversation { start: true, at: "conversation starts".into() });
        notes.push("the bottom bar is put away (unless handsOff; remembered for the end)".into());
        notes.push("first person is not built: he is in third person; switching views is blocked until the end (rule 6, nothing to block)".into());
        let w = &mut self.world;
        let c = w.conversation.as_mut().expect("asked");
        let (case, line) = match &c.info {
            Some(info) => case_for(info, position),
            None => (Some(Case::There), "no info object: case \"there\" (Do not reproduce)".to_string()),
        };
        notes.push(line);
        let case = case.unwrap_or_else(|| {
            notes.push("in neither area: no case (rule 11: the original stays stuck); Do not reproduce: treated as \"there\"".into());
            Case::There
        });
        c.case = Some(case);
        c.walk_from = position;
        // Rule 7: weapons are not built; he never holds one drawn.
        if walker.hand_mode == EMPTY_HANDS || c.keep_weapon {
            notes.push(format!("weapon: {} (rule 7: no away/out)", if c.keep_weapon { "<enemy> not 0, kept" } else { "none drawn (empty hands)" }));
        }
        match case {
            Case::There => {
                c.stage = Stage::Talking;
                notes.push("case \"there\": Connor stays where he is; the stay test runs (rules 8, 13)".into());
            }
            Case::Close => {
                c.stage = Stage::Talking;
                let spot = c.info.as_ref().map_or(position, |i| i.spot);
                let at = format!("conversation with {} (case \"close\")", c.partner);
                w.requests.push(Request::Move { who: CONNOR.into(), flags: 2, position: spot, turn: [0.0; 3], at });
                notes.push(format!("case \"close\": Connor put on the spot ({:.0}, {:.0}, {:.0}), heading kept (move Connor 2, rule 9); the stay test runs", spot[0], spot[1], spot[2]));
            }
            Case::Walk => {
                c.stage = Stage::StopWalk;
                c.approach_updates = 0;
                let spot = c.info.as_ref().map_or(position, |i| i.spot);
                let at = format!("conversation with {} (case \"walk\")", c.partner);
                w.requests.push(Request::ConnorAuto { auto: AutoMove::new(AutoKind::Point, [spot[0], spot[1]], None), at });
                walker.talk_play = Some(STOP_WALK);
                notes.push(format!("case \"walk\": turns toward the spot ({:.0}, {:.0}) with the automatic turn and plays {STOP_WALK} (rule 10; keys ignored meanwhile, Q125 stand-in)", spot[0], spot[1]));
            }
        }
        let tell = case != Case::Walk;
        self.world.conversation_note(format!("Connor enters his talk state: {}", notes.join("; ")));
        if tell {
            self.tell_partner_start();
        }
    }

    /// Rules 8–10: the partner is told to start: its `speak` queue emptied,
    /// the conversation's lines queued (rule 21).
    fn tell_partner_start(&mut self) {
        let now = self.world.now();
        let Some(c) = self.world.conversation.as_ref() else { return };
        let (partner, lines) = (c.partner.clone(), c.lines.clone());
        let cleared = self.world.talk.clear(&partner, now);
        let queued = self.world.talk.conv_start(&lines, now);
        self.world.conversation_note(format!("the partner {partner} is told to start: speak queue: {cleared}; {queued}"));
    }

    /// Rule 10: the approach movements.
    fn approach(&mut self, walker: &mut Walker, position: [f32; 3], stage: Stage) {
        let movement = if stage == Stage::StopWalk { STOP_WALK } else { WALK_STOP_RIGHT };
        let c = self.world.conversation.as_mut().expect("in a stage");
        c.approach_updates += 1;
        let ended = walker.movement == movement && walker.phase >= 1.0;
        let guard = c.approach_updates > APPROACH_GUARD;
        if !ended && !guard {
            return;
        }
        let why = if ended { format!("{movement} ended") } else { format!("{movement} did not end in {APPROACH_GUARD} updates (our guard; movement now {})", walker.movement) };
        let spot = c.info.as_ref().map_or(position, |i| i.spot);
        match stage {
            Stage::StopWalk => {
                let from = c.walk_from;
                let (p, heading) = walk_in_point(from, spot);
                c.stage = Stage::WalkStopRight;
                c.approach_updates = 0;
                let at = format!("conversation with {} (walk-in)", c.partner);
                self.world.requests.push(Request::Move { who: CONNOR.into(), flags: 0, position: p, turn: [0.0, 0.0, heading], at });
                // The heading is set: the turn toward the spot is over.
                if walker.auto.as_ref().is_some_and(|a| a.kind == AutoKind::Point) {
                    walker.auto = None;
                }
                walker.talk_play = Some(WALK_STOP_RIGHT);
                self.world.conversation_note(format!(
                    "{why} at ({:.0}, {:.0}, {:.0}): put {WALK_IN_SHORT} short of the spot at ({:.0}, {:.0}, {:.0}) on the line from ({:.0}, {:.0}), heading {heading:.3} (facing the spot); plays {WALK_STOP_RIGHT} (rule 10)",
                    position[0], position[1], position[2], p[0], p[1], p[2], from[0], from[1]
                ));
                self.tell_partner_start();
            }
            _ => {
                c.stage = Stage::Talking;
                let to = c.partner_at;
                let at = format!("conversation with {} (at the spot)", c.partner);
                walker.talk_play = None;
                self.world.requests.push(Request::ConnorAuto { auto: AutoMove::new(AutoKind::Point, [to[0], to[1]], None), at });
                let d = ((position[0] - spot[0]).powi(2) + (position[1] - spot[1]).powi(2)).sqrt();
                self.world.conversation_note(format!(
                    "{why} at ({:.0}, {:.0}, {:.0}), {d:.0} from the spot (flat): turns to face the partner at ({:.0}, {:.0}) with the automatic turn; the keys choose again; the stay test runs (rules 10, 13)",
                    position[0], position[1], position[2], to[0], to[1]
                ));
            }
        }
    }

    /// Rule 13: every update, outside the stay area ends the conversation.
    fn stay_test(&mut self, walker: &mut Walker, position: [f32; 3]) {
        let c = self.world.conversation.as_ref().expect("talking");
        let Some(info) = &c.info else { return };
        if info.in_area(1, [position[0], position[1]]) {
            return;
        }
        let partner = c.partner.clone();
        let now = self.world.now();
        let captions = self.scripts.vars.get("Options::Captions") == Some("true");
        let mut talk = std::mem::take(&mut self.world.talk);
        let w = &self.world;
        let exists = |name: &str| w.known.contains(&name.to_ascii_lowercase()) && !w.is_gone(name);
        let goodbye = talk.conv_goodbye(&partner, now, captions, &exists);
        self.world.talk = talk;
        let (talk_exit, what) = match goodbye {
            Goodbye::Started => (false, "the goodbye plays; TalkExit when it ends (rule 22)".to_string()),
            // Do not reproduce: the original sends no TalkExit here.
            Goodbye::None(why) => (true, format!("no goodbye ({why}); TalkExit now (Do not reproduce: the original sends none)")),
        };
        self.world.conversation_note(format!("stay test: Connor at ({:.0}, {:.0}) is outside the stay area: walked away (rule 13); {what}", position[0], position[1]));
        self.conversation_leave("walked out of the stay area", talk_exit);
        if let Some(why) = self.world.connor_leave.take() {
            let to = walker.leave_talk();
            self.world.conversation_note(format!("Connor leaves his talk state ({why}): {to} (rule 15)"));
        }
    }

    /// Rule 15: the conversation ends. Connor's own state changes at the
    /// next [`Active::conversation_tick`] (this update, before his next).
    pub(crate) fn conversation_leave(&mut self, why: &str, talk_exit: bool) {
        let Some(c) = self.world.conversation.take() else { return };
        if !c.entered() {
            self.world.conversation = Some(c);
            return;
        }
        let mut notes = Vec::new();
        // The camera: at cam0 looking at the partner, then following Connor.
        match c.info.as_ref().and_then(|i| i.cameras.first()) {
            Some(cam0) => {
                let aim = partner_aim(c.partner_at);
                let d: [f32; 3] = std::array::from_fn(|k| aim[k] - cam0.position[k]);
                let (tilt, heading) = crate::camera::view_angles(d);
                let at = format!("conversation with {} ends", c.partner);
                self.world.requests.push(Request::Move { who: "KQCamera".into(), flags: 0, position: cam0.position, turn: [tilt, 0.0, heading], at: at.clone() });
                notes.push(format!(
                    "camera to cam0 ({:.0}, {:.0}, {:.0}) looking at ({:.0}, {:.0}, {:.0}) (the partner + 1400, Q126 stand-in), then follows Connor",
                    cam0.position[0], cam0.position[1], cam0.position[2], aim[0], aim[1], aim[2]
                ));
                self.world.requests.push(Request::CameraFollow { connor: true, at });
            }
            None => {
                self.world.requests.push(Request::CameraFollow { connor: true, at: format!("conversation with {} ends", c.partner) });
                notes.push("no cam0: the camera follows Connor".into());
            }
        }
        notes.push("first person not built: nothing to restore".into());
        self.world.connor_leave = Some(why.to_string());
        // The partner goes to Wait (rules 15, 16).
        if let Some(i) = self.world.character_index(&c.partner) {
            let a = &mut self.world.anims[i];
            if let Some(line) = a.character.as_mut().unwrap().leave_talk(&mut a.player) {
                self.world.character_note(&c.partner, &line);
                notes.push(format!("partner: {line}"));
            }
        }
        self.world.gui_orders.push(GuiOrder::Conversation { start: false, at: "conversation ends".into() });
        notes.push("the bottom bar comes back if it was shown at the start".into());
        self.world.conversation_note(format!("the conversation with {} ends ({why}): {}", c.partner, notes.join("; ")));
        if talk_exit {
            self.talk_exit(&c.partner);
        } else {
            self.world.conversation_note(format!("{}: TalkExit waits for the goodbye's end", c.partner));
        }
    }

    /// `TalkExit` to the partner's script: `$1` its name, `$2 TalkExit`.
    pub(crate) fn talk_exit(&mut self, partner: &str) {
        let ran = self.call_named(partner, &["TalkExit"]);
        self.world.conversation_note(format!("{partner}: TalkExit: {}", if ran { "its script ran" } else { "no script" }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn info() -> ConvInfo {
        // The Wizard's file, as Daventry has it (numbers from the data
        // are not written into the repo: made-up ones in the same shape).
        let text = b"[BobInfo]\nclassType=KQConvInfo\nconvLocX=1000\nconvLocY=1000\nconvLocZ=100\nnBox=2\n\
box0=0 0 4000 0 4000 4000 0 4000\nbox1=500 500 1500 500 1500 1500 500 1500\nnCam=1\ncam0=0 0 500 0 0 0\n";
        ConvInfo::from_section(&kq8_data::kq_text::parse(text)[0]).unwrap()
    }

    #[test]
    fn the_case_comes_from_the_areas_and_the_distance() {
        let i = info();
        assert_eq!(case_for(&i, [1050.0, 1000.0, 100.0]).0, Some(Case::There));
        assert_eq!(case_for(&i, [1124.0, 1000.0, 100.0]).0, Some(Case::There));
        assert_eq!(case_for(&i, [1125.0, 1000.0, 100.0]).0, Some(Case::Close), "125 or more: close");
        assert_eq!(case_for(&i, [1000.0, 1000.0, 230.0]).0, Some(Case::Close), "3D distance");
        assert_eq!(case_for(&i, [3000.0, 3000.0, 100.0]).0, Some(Case::Walk));
        assert_eq!(case_for(&i, [5000.0, 5000.0, 100.0]).0, None);
    }

    /// A world with one partner, Bob (no drawn character), its info object
    /// and two made-up lines without voices (1 s each: 15 updates).
    fn world() -> Active {
        use kq8_data::msg::{Line, MessageFile, Tuple};
        let sources = [("bob.cs".to_string(), b"echo $1 $2\n".to_vec())];
        let mut a = Active::new(kq8_data::world_script::Vars::default(), vec![], vec![], ["Connor".to_string(), "Bob".to_string()], sources);
        a.world.monsters.push(crate::active::Monster { name: "Bob".into(), position: [1000.0, 1000.0, 0.0], script: Some("bob.cs".into()), goodbye: None, welcome_back: None });
        a.world.conv_infos.push(info());
        let line = |seq| Line { tuple: Tuple::new(1000, 60, 0, 0, seq), talker: 9, raw: "Hm.".into() };
        let file = MessageFile { kind: 0x0F, version: 5010, last_id: 0, lines: vec![line(1), line(2)] };
        a.world.talk.book.add("x", file, 1000);
        a
    }

    #[test]
    fn a_conversation_enters_plays_and_ends_with_talk_exit_before_the_last_talk_complete() {
        let mut a = world();
        a.test_command("KQConner::conversation Bob 1000 60 0 0 1 0 0", "test");
        assert_eq!(a.world.conversation.as_ref().map(|c| (c.stage, c.lines.last)), Some((Stage::Asked, 2)), "<last> 0: the run");
        let mut walker = Walker::new(0.0);
        a.conversation_tick(&mut walker, [1000.0, 1000.0, 100.0], Ok(()));
        assert_eq!(a.world.conversation.as_ref().map(|c| (c.case, c.stage)), Some((Some(Case::There), Stage::Talking)));
        assert_eq!(walker.mode, crate::connor::Mode::Talk);
        assert!(a.world.speak_ignored("Connor") && a.world.speak_ignored("bob") && !a.world.speak_ignored("Wizard"));
        a.world.take_talk_log();
        for _ in 0..40 {
            a.update();
        }
        let log = a.take_log().join("\n");
        let exit = log.find("script bob.cs Bob TalkExit").expect(&log);
        let last = log.find("Bob: TalkComplete 1000 60 0 0 2").expect(&log);
        assert!(log.contains("Bob: TalkComplete 1000 60 0 0 1") && exit < last, "TalkExit before the last line's TalkComplete: {log}");
        assert!(a.world.conversation.is_none());
        a.conversation_tick(&mut walker, [1000.0, 1000.0, 100.0], Ok(()));
        assert_eq!(walker.mode, crate::connor::Mode::Free, "Wait");
        assert!(a.world.gui_orders.iter().filter(|o| matches!(o, GuiOrder::Conversation { .. })).count() == 2, "the bar away and back");
    }

    #[test]
    fn walking_out_of_the_stay_area_ends_it_and_sends_talk_exit_without_a_goodbye() {
        let mut a = world();
        a.test_command("KQConner::conversation Bob 1000 60 0 0 1 2 0", "test");
        let mut walker = Walker::new(0.0);
        a.conversation_tick(&mut walker, [1000.0, 1000.0, 100.0], Ok(()));
        a.update();
        a.conversation_tick(&mut walker, [1000.0, 1000.0, 100.0], Ok(()));
        assert!(a.world.conversation.is_some(), "still inside");
        a.take_log();
        a.conversation_tick(&mut walker, [2000.0, 1000.0, 100.0], Ok(()));
        assert!(a.world.conversation.is_none() && walker.mode == crate::connor::Mode::Free);
        let log = a.take_log().join("\n");
        assert!(log.contains("no goodbye") && log.contains("script bob.cs Bob TalkExit"), "Do not reproduce: TalkExit anyway: {log}");
    }

    #[test]
    fn a_missing_info_object_and_no_case_are_treated_as_there() {
        let mut a = world();
        a.world.conv_infos.clear();
        a.test_command("KQConner::conversation Bob 1000 60 0 0 1 2 0", "test");
        let mut walker = Walker::new(0.0);
        a.conversation_tick(&mut walker, [9000.0, 9000.0, 0.0], Ok(()));
        assert_eq!(a.world.conversation.as_ref().map(|c| c.case), Some(Some(Case::There)));
        let mut a = world();
        a.test_command("KQConner::conversation Bob 1000 60 0 0 1 2 0", "test");
        a.conversation_tick(&mut walker, [9000.0, 9000.0, 0.0], Err("jumping".into()));
        assert_eq!(a.world.conversation.as_ref().map(|c| c.stage), Some(Stage::Asked), "tried again later");
        let mut w2 = Walker::new(0.0);
        a.conversation_tick(&mut w2, [9000.0, 9000.0, 0.0], Ok(()));
        assert_eq!(a.world.conversation.as_ref().map(|c| c.case), Some(Some(Case::There)), "no case: there");
        // Bad commands.
        let mut a = world();
        a.test_command("KQConner::conversation Bob 1000 60", "test");
        a.test_command("KQConner::conversation Nobody 1000 60 0 0 1 2 0", "test");
        assert!(a.world.conversation.is_none());
        assert!(a.take_log().join("\n").contains("'Nobody' is not KQMonster"));
    }

    #[test]
    fn the_walk_in_puts_him_440_short_facing_the_spot() {
        let (p, h) = walk_in_point([1000.0, 3000.0, 0.0], [1000.0, 1000.0, 100.0]);
        assert_eq!(p, [1000.0, 1440.0, 100.0]);
        // Facing −Y (toward the spot): heading 0.
        assert!(h.abs() < 1e-5, "{h}");
        let (p, _) = walk_in_point([1000.0, 1200.0, 0.0], [1000.0, 1000.0, 100.0]);
        assert_eq!(p, [1000.0, 1440.0, 100.0], "starting nearer: put further back (Edge cases)");
    }
}
