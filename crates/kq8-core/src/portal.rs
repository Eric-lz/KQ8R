//! Connor's Portal state (M6 6e-6): the question before a world change and
//! its answers.
//!
//! Rules from `docs/behaviour/world-change.md` (A, 2026-10-21; the user's
//! check in the original 2026-10-22) and `magic-map.md` rule 40:
//!
//! - Rule 1: a script starts it: `KQMonster::setState Connor Portal
//!   <portal> [<trap>]`.
//! - Rule 2: Connor stops where he is and plays `stop`; the portal and trap
//!   names are remembered; the Yes/No popup opens unless one is already
//!   open. Nothing else happens until it is answered.
//! - Rule 3, No: the portal's own script gets `Decline`; if the portal's
//!   `noLoc` is not (−1, −1, −1) Connor is moved there; he plays `stop`
//!   and goes back to Wait.
//! - Rule 4, Yes: the portal's script gets `Accept`, `handsOff 1`, first
//!   person off, the target cleared, the time step limited; with an
//!   `enterAnim` Connor plays it (from `enterAnmFile` when named, else
//!   his current list) and the hand-over waits for its end; without one
//!   it comes at once.
//! - Rule 26: the popup: text 114, Yes (29) left, No (14) right; the game
//!   is paused while it is open, the sounds go on.
//!
//! Ours: Connor's `stop` (rules 2, 3) is not played as a scripted
//! movement: the game is paused while the question is open, and after No
//! he is simply back in Wait (the user's walking test: a `stop` played in
//! the Special state never ends). The portal's keys come from its file (placed objects, or a
//! `GAME/Kq` portal file loaded while playing); first person and the time
//! step are logged only (we have neither).

use kq8_data::magic_map::PortalKeys;

/// Where the Portal state is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Phase {
    /// The popup is open (rule 2).
    Asking,
    /// Yes: the `enterAnim` plays (rule 4).
    Entering,
    /// Ready for the hand-over (rule 5): the caller changes the world.
    Leaving,
}

/// Connor's Portal state.
#[derive(Debug, Clone, PartialEq)]
pub struct PortalState {
    pub portal: String,
    pub trap: Option<String>,
    pub keys: PortalKeys,
    pub phase: Phase,
}

/// The popup's text (rule 26): `text` cut at spaces (and line breaks) into
/// lines of at most 38 characters, at most 7 lines.
pub fn popup_lines(text: &str) -> Vec<String> {
    const WIDTH: usize = 38;
    const LINES: usize = 7;
    let mut out = Vec::new();
    for para in text.split(['\n', '\r']).filter(|p| !p.trim().is_empty()) {
        let mut line = String::new();
        for word in para.split_whitespace() {
            if !line.is_empty() && line.chars().count() + 1 + word.chars().count() > WIDTH {
                out.push(std::mem::take(&mut line));
            }
            if !line.is_empty() {
                line.push(' ');
            }
            line.push_str(word);
        }
        if !line.is_empty() {
            out.push(line);
        }
    }
    out.truncate(LINES);
    out
}

impl crate::active::WorldState {
    /// `KQMonster::setState Connor Portal <portal> [<trap>]` (rules 1–2).
    pub(crate) fn start_portal(&mut self, words: &[String], at: &str) -> String {
        let Some(portal) = words.first() else {
            self.note(format!("{at}: KQMonster::setState Connor Portal without a portal; nothing"));
            return "1".into();
        };
        let trap = words.get(1).cloned();
        let Some(keys) = self.portals.get(&portal.to_ascii_lowercase()).cloned() else {
            let line = format!("{at}: KQMonster::setState Connor Portal {portal}: no portal by that name in this world; nothing");
            self.note(&line);
            self.map_log.push(line);
            return "1".into();
        };
        if self.connor_portal.is_some() {
            // Rule 2: a popup already open is not opened again.
            self.note(format!("{at}: KQMonster::setState Connor Portal {portal}: already in the Portal state; nothing (rule 2)"));
            return "0".into();
        }
        let line = format!(
            "{at}: KQMonster::setState Connor Portal {portal} {}: Connor stands still (the game pauses; his stop is not played as a movement); the question opens: text 114, Yes / No; the game pauses (world-change.md rules 2, 26); portal to {:?}",
            trap.as_deref().unwrap_or(""),
            keys.world
        );
        self.note(&line);
        self.map_log.push(line);
        // Ours: he stands still because the game pauses; his `stop` is not
        // played as a Special movement (it repeats, so the Special state
        // would never end and he could not walk after No).
        self.connor_portal = Some(PortalState { portal: portal.clone(), trap, keys, phase: Phase::Asking });
        "0".into()
    }

    /// The popup is open (rule 26: the game pauses).
    pub fn portal_asking(&self) -> bool {
        self.connor_portal.as_ref().is_some_and(|p| p.phase == Phase::Asking)
    }
}

impl crate::active::Active {
    /// The answer to the popup (rules 3, 4). Returns lines for the logs.
    pub fn portal_answer(&mut self, yes: bool, how: &str) -> Vec<String> {
        let Some(state) = self.world.connor_portal.clone().filter(|p| p.phase == Phase::Asking) else {
            return vec![format!("portal answer ({how}): no question open; nothing")];
        };
        let mut lines = Vec::new();
        let portal = state.portal.clone();
        if !yes {
            // Rule 3.
            let declined = self.call_named(&portal, &["Decline"]);
            lines.push(format!("No ({how}): {portal} gets Decline ({})", if declined { "its script ran" } else { "no script" }));
            let [x, y, z] = state.keys.no_loc.map(|v| v as f32);
            if [x, y, z] != [-1.0, -1.0, -1.0] {
                // Flags 2: the place set, the turn kept (+0).
                let r = self.test_command(&format!("move Connor 2 {x} {y} {z} 0 0 0"), "portal No");
                lines.push(format!("No: Connor moved to noLoc ({x}, {y}, {z}) (rule 3): {r:?}"));
            } else {
                lines.push("No: noLoc (-1, -1, -1): Connor stays".into());
            }
            self.world.connor_portal = None;
            lines.push("No: Connor stands and waits (back to Wait); the game goes on".into());
        } else {
            // Rule 4.
            let accepted = self.call_named(&portal, &["Accept"]);
            lines.push(format!("Yes ({how}): {portal} gets Accept ({})", if accepted { "its script ran" } else { "no script" }));
            let r = self.test_command("handsOff 1", "portal Yes");
            lines.push(format!("Yes: handsOff 1: {r:?}; first person off, the time step 2 frame periods (logged only: we have neither)"));
            let next = match &state.keys.enter_anim {
                Some(anim) => {
                    let own = state.keys.enter_anm_file.as_deref().is_none_or(|l| l.eq_ignore_ascii_case("conner.anm"));
                    let items = if own { anim.clone() } else { format!("AnimList={},AnimListOn,{anim}", state.keys.enter_anm_file.clone().unwrap_or_default()) };
                    self.world.requests.push(crate::active::Request::ConnorSpecial { items: items.clone(), at: "portal Yes".into() });
                    lines.push(format!("Yes: enterAnim: special {items}; the hand-over waits for its end"));
                    Phase::Entering
                }
                None => {
                    lines.push("Yes: no enterAnim: the hand-over comes at once".into());
                    Phase::Leaving
                }
            };
            if let Some(p) = self.world.connor_portal.as_mut() {
                p.phase = next;
            }
        }
        for l in &lines {
            self.world.note(format!("portal: {l}"));
            self.world.map_log.push(format!("portal: {l}"));
        }
        lines
    }

    /// The `enterAnim` ended (the caller saw his Special state end): the
    /// hand-over is due (rule 5).
    pub fn portal_entered(&mut self) {
        if let Some(p) = self.world.connor_portal.as_mut().filter(|p| p.phase == Phase::Entering) {
            p.phase = Phase::Leaving;
            let line = format!("portal: {}: the enterAnim ended; the hand-over is due (world-change.md rule 5)", p.portal);
            self.world.note(&line);
            self.world.map_log.push(line);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_question_is_cut_into_38_character_lines() {
        let lines = popup_lines("You are about to load a new region. Are you sure you want to do that?");
        assert_eq!(lines, ["You are about to load a new region.", "Are you sure you want to do that?"]);
        assert!(lines.iter().all(|l| l.chars().count() <= 38));
        let long = "word ".repeat(100);
        assert_eq!(popup_lines(&long).len(), 7, "at most 7 lines");
    }

    fn keys(enter: Option<&str>, no_loc: [f64; 3]) -> PortalKeys {
        PortalKeys {
            name: "portal1".into(),
            world: Some("Castled.kq".into()),
            enter_anim: enter.map(str::to_string),
            exit_anim: Some("out".into()),
            enter_anm_file: None,
            exit_anm_file: None,
            exit_script: None,
            dest_connor_loc: [9000.0, 111805.0, 150.0],
            dest_connor_dir: [0.0; 3],
            dest_cam_loc: [0.0; 3],
            dest_cam_dir: [0.0; 3],
            no_loc,
        }
    }

    fn active(k: PortalKeys) -> crate::active::Active {
        let mut a = crate::active::Active::new(Default::default(), vec![], vec![], ["Connor".to_string()], Vec::<(String, Vec<u8>)>::new());
        a.world.portals.insert("portal1".into(), k);
        a
    }

    #[test]
    fn no_moves_connor_to_no_loc_and_ends_the_state() {
        let mut a = active(keys(None, [158510.0, 307740.0, 9925.0]));
        assert_eq!(a.test_command("KQMonster::setState Connor Portal portal1 portaltp", "test").as_deref(), Some("0"));
        assert!(a.world.portal_asking());
        assert_eq!(a.world.connor_portal.as_ref().and_then(|p| p.trap.clone()).as_deref(), Some("portaltp"));
        a.world.requests.clear();
        a.portal_answer(false, "test");
        assert!(a.world.connor_portal.is_none());
        assert!(a.world.requests.iter().any(|r| matches!(r, crate::active::Request::Move { position, .. } if *position == [158510.0, 307740.0, 9925.0])), "{:?}", a.world.requests);
    }

    #[test]
    fn yes_with_and_without_an_enter_movement() {
        let mut a = active(keys(Some("teleportdown"), [-1.0; 3]));
        a.test_command("KQMonster::setState Connor Portal portal1", "test");
        a.portal_answer(true, "test");
        assert_eq!(a.world.connor_portal.as_ref().map(|p| p.phase), Some(Phase::Entering));
        a.portal_entered();
        assert_eq!(a.world.connor_portal.as_ref().map(|p| p.phase), Some(Phase::Leaving));
        let mut b = active(keys(None, [-1.0; 3]));
        b.test_command("KQMonster::setState Connor Portal portal1", "test");
        b.portal_answer(true, "test");
        assert_eq!(b.world.connor_portal.as_ref().map(|p| p.phase), Some(Phase::Leaving), "no enterAnim: at once");
        assert_eq!(b.test_command("KQMonster::setState Connor Portal nowhere", "test").as_deref(), Some("1"), "an unknown portal");
    }
}
