//! Characters moving on their own (M5 5a): what a character "knows"
//! about Connor and what it decides. The walking itself is
//! `crate::character` (the Move state); this module is the part that
//! decides where to go.
//!
//! Rules from `docs/behaviour/character-movement.md` (session A,
//! 2026-10-19; not yet checked in the original):
//!
//! - Rule 20 (M5 5a-4): every 15th update a character that is updated
//!   (near or far) checks whether it **can see Connor**: a straight line
//!   from the top of its `BOUNDS.KQ` box to 1,250 above Connor's
//!   position, blocked by the drawn ground (the water surface included),
//!   plain objects (type bit 1) and buildings (type 8), not by characters
//!   or Connor; no distance limit; `canAlwaysSeeConner=1` skips the test.
//!   Between checks it keeps the result and counts the updates since it
//!   last saw him. In Wait, a character that is not active and can see
//!   him joins the world's **active list** if there is room
//!   (`KQWorld::MaxMonsters`); one with a team may take the place of one
//!   without; the others try again each update.
//! - Rule 21 (M5 5a-4): going off (neither near nor far), or being pushed
//!   off the list by a team member, takes it off the list. Then, if it was
//!   on its way home or Connor is more than twice its alarm radius away,
//!   it is put back on its home point at once; a list with `hide` (the
//!   zombies) goes back into Birth; a moving one goes to Wait.
//!
//! Our readings and stand-ins (`docs/provenance/`): the 15th updates are
//! the world's (update number a multiple of 15); "twice its alarm radius
//! away" is the plain 3D distance (the note's open question 2); a team
//! member takes the place of the **first** one without a team on the list.

use kq8_data::characters::AiSettings;

use crate::active::WorldState;
use crate::connor_mover::{GroundLookup, MoverWorld, Scene};

/// The sight test runs every this many updates (rule 20).
pub const SIGHT_EVERY: u64 = 15;
/// The test line ends this far above Connor's position (rule 20).
pub const SIGHT_ABOVE_CONNOR: f32 = 1250.0;
/// What blocks the line: plain objects (bit 1) and buildings (bit 8).
pub const SIGHT_MASK: u32 = 0x1 | 0x8;
/// `KQWorld::MaxMonsters` when `world.cs` sets none (**ours**: every
/// world's `world.cs` sets it; 6 is the most common value).
pub const DEFAULT_MAX_MONSTERS: usize = 6;

/// A character's own movement rules and what it knows (M5 5a).
#[derive(Debug, Clone)]
pub struct Ai {
    pub settings: AiSettings,
    /// Its home point (rule 13): where it was placed; `setHome` changes it.
    pub home: [f32; 3],
    /// Its eye: the top of its box, above its position (rule 20).
    pub eye: f32,
    /// Its list has `hide` (the zombies, rule 21).
    pub has_hide: bool,
    /// On the world's active list (rule 20).
    pub active: bool,
    /// The last sight test's result, and what blocked the line.
    pub sees: bool,
    pub blocked_by: Option<String>,
    /// A sight test has run.
    pub checked: bool,
    /// Updates since it last saw Connor (rule 20; used by rule 26).
    pub unseen: u32,
    /// On its way home (rules 21, 28).
    pub on_way_home: bool,
}

impl Ai {
    pub fn new(settings: AiSettings, home: [f32; 3], eye: f32, has_hide: bool) -> Self {
        Ai { settings, home, eye, has_hide, active: false, sees: false, blocked_by: None, checked: false, unseen: 0, on_way_home: false }
    }
}

/// The plain distance between two points.
fn distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt()
}

impl WorldState {
    /// One update of the characters' movement rules (M5 5a), after their
    /// own update and before their movers: losing and joining the active
    /// list, the sight test (rules 20, 21). `ground`, `buildings` and
    /// `objects` are the world for the sight test.
    pub fn characters_ai(&mut self, ground: &dyn GroundLookup, buildings: &[crate::collide::BuildingCollider], objects: &[crate::collide::BoxCollider]) {
        let update = self.updates;
        let now = self.now();
        // Rule 21: off (not updated now) → off the list.
        for i in 0..self.anims.len() {
            let a = &self.anims[i];
            let off = a.gone || !self.characters_in_range.contains(&a.player.name.to_ascii_lowercase());
            if off && a.character.as_deref().and_then(|c| c.ai.as_ref()).is_some_and(|ai| ai.active) {
                self.lose_active(i, if a.gone { "deleted" } else { "went off (neither near nor far)" }, now);
            }
        }
        // Rule 20: what blocks the sight line (characters never do).
        let sight_objects: Vec<crate::collide::BoxCollider> = objects.iter().filter(|o| o.kind.collision_type & SIGHT_MASK != 0).cloned().collect();
        let scene = Scene::new(ground, buildings, &sight_objects);
        let connor = self.connor_at;
        let check = update.is_multiple_of(SIGHT_EVERY);
        for i in 0..self.anims.len() {
            let a = &self.anims[i];
            if a.gone || !self.characters_in_range.contains(&a.player.name.to_ascii_lowercase()) {
                continue;
            }
            let Some(c) = a.character.as_deref() else { continue };
            let Some(ai) = c.ai.as_ref() else { continue };
            let pos = a.player.base.position;
            let name = a.player.name.clone();
            let state = c.state;
            let mut lines = Vec::new();
            if check {
                let (sees, blocked) = match connor {
                    None => (false, Some("no Connor".to_string())),
                    Some(_) if ai.settings.can_always_see => (true, None),
                    Some(at) => {
                        let from = [pos[0], pos[1], pos[2] + ai.eye];
                        let to = [at[0], at[1], at[2] + SIGHT_ABOVE_CONNOR];
                        let b = scene.line_blocked(from, to, SIGHT_MASK, &name);
                        (b.is_none(), b)
                    }
                };
                if !ai.checked || sees != ai.sees {
                    let why = match (&blocked, ai.settings.can_always_see) {
                        (_, true) => "canAlwaysSeeConner".to_string(),
                        (Some(b), _) => format!("blocked by {b}"),
                        (None, _) => "a clear line".to_string(),
                    };
                    lines.push(format!("sight (rule 20): {} Connor ({why}; eye {:.0} above its feet)", if sees { "can see" } else { "can't see" }, ai.eye));
                }
                let c = self.anims[i].character.as_deref_mut().unwrap();
                let ai = c.ai.as_mut().unwrap();
                ai.sees = sees;
                ai.blocked_by = blocked;
                ai.checked = true;
            }
            let a = &mut self.anims[i];
            let c = a.character.as_deref_mut().unwrap();
            let ai = c.ai.as_mut().unwrap();
            ai.unseen = if ai.sees { 0 } else { ai.unseen.saturating_add(1) };
            let wants_in = !ai.active && ai.sees && state == crate::character::State::Wait;
            if check {
                lines.push(status_line(&a.player, c, connor));
            }
            for l in lines {
                self.character_note(&name, l);
            }
            if wants_in {
                self.join_active(i, now);
            }
        }
    }

    /// Rule 20: joins the active list if there is room; a character with a
    /// team may take the place of the first one without a team.
    fn join_active(&mut self, i: usize, now: f64) {
        let name = self.anims[i].player.name.clone();
        let team = self.anims[i].character.as_deref().and_then(|c| c.ai.as_ref()).and_then(|ai| ai.settings.team.clone());
        let max = self.max_monsters;
        if self.active_list.len() >= max {
            let Some(team) = team else { return };
            let untimed = self.active_list.iter().position(|n| {
                self.character_index(n).and_then(|j| self.anims[j].character.as_deref()).and_then(|c| c.ai.as_ref()).is_some_and(|ai| ai.settings.team.is_none())
            });
            let Some(k) = untimed else { return };
            let out = self.active_list[k].clone();
            if let Some(j) = self.character_index(&out) {
                self.lose_active(j, &format!("pushed off the active list by {name} (team {team})"), now);
            }
        }
        self.active_list.push(name.to_ascii_lowercase());
        let ai = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()).unwrap();
        ai.active = true;
        let line = format!("joins the active list (rule 20): {} of {max} ({})", self.active_list.len(), self.active_list.join(", "));
        self.character_note(&name, line);
    }

    /// Rule 21: off the active list (`why`); forgets its target; put back
    /// on its home point at once if it was on its way home or Connor is
    /// more than twice its alarm radius away.
    pub(crate) fn lose_active(&mut self, i: usize, why: &str, now: f64) {
        let connor = self.connor_at;
        let heights = self.heights.as_ref();
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_mut() else { return };
        ai.active = false;
        self.active_list.retain(|n| !n.eq_ignore_ascii_case(&name));
        let far = connor.map(|at| distance(at, a.player.base.position));
        let limit = 2.0 * ai.settings.alarm_radius;
        let home_now = ai.on_way_home || far.is_none_or(|d| d > limit);
        let mut line = format!(
            "leaves the active list (rule 21): {why}; Connor {} away (2 × alarm radius {limit:.0}){}",
            far.map_or("not known".to_string(), |d| format!("{d:.0}")),
            if ai.on_way_home { "; it was on its way home" } else { "" }
        );
        if home_now {
            let mut home = ai.home;
            if home[2] == -1.0 || home[2] == -2.0 {
                if let Some(z) = heights.and_then(|h| h(home[0], home[1], home[2] == -1.0)) {
                    home[2] = z;
                }
            }
            ai.on_way_home = false;
            let has_hide = ai.has_hide;
            a.player.base.position = home;
            line.push_str(&format!("; put back on its home point ({:.0}, {:.0}, {:.0}) at once", home[0], home[1], home[2]));
            if has_hide {
                c.state = crate::character::State::Birth;
                line.push_str("; its list has hide: back into Birth (hiding a drawn character is not built yet)");
            }
        }
        if c.state == crate::character::State::Move {
            let w = c.set_state(&mut a.player, "Wait", &[], now);
            line.push_str(&format!("; moving: Wait ({w})"));
        }
        self.character_note(&name, line);
    }
}

/// The once-a-second line (the note's "How to check", our logs).
fn status_line(player: &crate::named_anim::NamedAnimPlayer, c: &crate::character::Character, connor: Option<[f32; 3]>) -> String {
    let p = player.base.position;
    let ai = c.ai.as_ref().unwrap();
    format!(
        "status: {:?}, {}, at ({:.0}, {:.0}, {:.0}) heading {:.3}, '{}'; Connor {}; {} ({} updates unseen){}",
        c.state,
        if ai.active { "active" } else { "not active" },
        p[0],
        p[1],
        p[2],
        crate::character::heading_of(&player.base.rotation),
        player.movement_name(),
        connor.map_or("not known".to_string(), |at| format!("{:.0} away", distance(at, p))),
        if ai.sees { "sees him".to_string() } else { format!("can't see him ({})", ai.blocked_by.as_deref().unwrap_or("?")) },
        ai.unseen,
        if ai.on_way_home { "; on its way home" } else { "" }
    )
}
