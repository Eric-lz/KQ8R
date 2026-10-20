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
    /// Its list has `Birth` (no random wander points then, rule 10).
    pub has_birth: bool,
    /// The wander points in the order it visits them, once round (rule
    /// 10), made the first time it wanders; the one it heads for now; the
    /// closest it has been to it (rule 11).
    pub route: Option<Vec<[f32; 2]>>,
    pub point: usize,
    pub closest: f32,
    /// `wanderAndStop` ended its points: no more wandering (rule 11).
    pub wander_done: bool,
    /// Game time its next fidget is due (rules 16, 17): 0 at first, so the
    /// first comes as soon as it is active.
    pub idle_due: f64,
    /// The fidget playing now (its name in the player).
    pub fidget: Option<String>,
}

impl Ai {
    pub fn new(settings: AiSettings, home: [f32; 3], eye: f32, has_hide: bool) -> Self {
        Ai { settings, home, eye, has_hide, active: false, sees: false, blocked_by: None, checked: false, unseen: 0, on_way_home: false, has_birth: false, route: None, point: 0, closest: f32::MAX, wander_done: false, idle_due: 0.0, fidget: None }
    }

    /// Rule 10: the points once round. `loop`: in order; `patrol`: there
    /// and back (`A B C D` → `A B C D C B A`); `random` or no points (and
    /// no `Birth`): its own position and three random points at 2–3 × its
    /// alarm radius, each direction at least 0.79 rad on from the last.
    pub fn make_route(&self, at: [f32; 3], random: &mut dyn FnMut() -> f32) -> (Vec<[f32; 2]>, &'static str) {
        use kq8_data::characters::Wander;
        let flat = |p: &[f32; 3]| [p[0], p[1]];
        match &self.settings.wander {
            Wander::Loop(p) if !p.is_empty() => (p.iter().map(flat).collect(), "loop"),
            Wander::Patrol(p) if !p.is_empty() => {
                let mut r: Vec<[f32; 2]> = p.iter().map(flat).collect();
                r.extend(p[..p.len() - 1].iter().rev().map(flat));
                (r, "patrol (there and back)")
            }
            _ if self.has_birth => (vec![flat(&at)], "no points and a Birth movement: none (stays)"),
            _ => {
                let mut r = vec![flat(&at)];
                let mut angle = random() * std::f32::consts::TAU;
                for k in 0..3 {
                    if k > 0 {
                        // **Ours**: at least 0.79 on, the rest of the
                        // circle short of 0.79 back, evenly.
                        angle += RANDOM_POINT_TURN + random() * (std::f32::consts::TAU - 2.0 * RANDOM_POINT_TURN);
                    }
                    let d = self.settings.alarm_radius * (2.0 + random());
                    r.push([at[0] + d * angle.sin(), at[1] - d * angle.cos()]);
                }
                (r, "random (own position + 3)")
            }
        }
    }
}

/// The idle list's movements are in the character's player under this
/// prefix (`idle:attack`), after its own (rules 15, 18; M5 5a-6).
pub const IDLE_PREFIX: &str = "idle:";

/// Rule 11: a wander point counts as reached within this (x, y).
pub const POINT_REACHED: f32 = 250.0;
/// Rule 11: it re-aims only when its distance² is more than its closest²
/// plus this².
pub const REAIM_DRIFT: f32 = 12500.0;
/// Rule 10: random wander points' directions are at least this far apart.
pub const RANDOM_POINT_TURN: f32 = 0.79;

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
            self.idle(i, now);
            self.wander(i, now);
        }
    }

    /// Rules 15–19: an active character with an idle list, when its idle
    /// time has come: a walking one goes back to Wait; in Wait it plays a
    /// fidget picked at random from the list (an ordinary request). When
    /// the fidget ends (any movement that ends while it plays counts), the
    /// next is due 2 + round(random × 10) s later; Wait brings back `Stop`.
    fn idle(&mut self, i: usize, now: f64) {
        use crate::character::State;
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_ref() else { return };
        let fidgets: Vec<String> = a.player.movements.iter().filter(|m| m.name.starts_with(IDLE_PREFIX)).map(|m| m.name.clone()).collect();
        if !ai.active || fidgets.is_empty() || ai.on_way_home {
            return;
        }
        let mut line = None;
        if let Some(f) = ai.fidget.clone() {
            let ended = !a.player.movement_name().eq_ignore_ascii_case(&f) || a.player.has_ended();
            if ended {
                let extra = (c.random() * 10.0).round();
                let ai = c.ai.as_mut().unwrap();
                ai.fidget = None;
                ai.idle_due = now + 2.0 + f64::from(extra);
                line = Some(format!("fidget '{}' ended: the next is due in {} s, at {:.2} (rule 17)", &f[IDLE_PREFIX.len()..], 2.0 + extra, ai.idle_due));
            }
        } else if now + 1e-6 >= ai.idle_due {
            match c.state {
                State::Move => {
                    let w = c.set_state(&mut a.player, "Wait", &[], now);
                    line = Some(format!("idle time: stops walking for a fidget (rule 16): {w}"));
                }
                State::Wait => {
                    let k = ((c.random() * fidgets.len() as f32) as usize).min(fidgets.len() - 1);
                    let f = fidgets[k].clone();
                    let how = match a.player.request(&f) {
                        crate::named_anim::Request::AtOnce(_) => "starts at once".to_string(),
                        _ => format!("waits for '{}'", a.player.movement_name()),
                    };
                    line = Some(format!("idle time: fidget '{}' (1 of {}, at random) {how} (rules 16, 18)", &f[IDLE_PREFIX.len()..], fidgets.len()));
                    c.ai.as_mut().unwrap().fidget = Some(f);
                }
                _ => {}
            }
        }
        if let Some(l) = line {
            self.character_note(&name, l);
        }
    }

    /// Rules 9–14: an active wanderer, in Wait (not on its way home, not
    /// done), starts walking (`walk`) in whatever way it faces; in Move it
    /// follows its points: within 250 → the next one, turning toward it;
    /// otherwise it re-aims only when it has drifted well off (rule 11);
    /// after the stuck escape it takes the next point (rule 7).
    fn wander(&mut self, i: usize, now: f64) {
        use crate::character::State;
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_ref() else { return };
        if !ai.active || !ai.settings.wanders || ai.wander_done || ai.on_way_home {
            return;
        }
        let mut lines = Vec::new();
        let pos = a.player.base.position;
        if ai.route.is_none() {
            let ai = c.ai.take().unwrap();
            let (route, kind) = ai.make_route(pos, &mut || c.random());
            lines.push(format!(
                "wander points (rule 10): {kind}: {}",
                route.iter().map(|p| format!("({:.0}, {:.0})", p[0], p[1])).collect::<Vec<_>>().join(" ")
            ));
            c.ai = Some(Ai { route: Some(route), point: 0, closest: f32::MAX, ..ai });
        }
        // Rule 12: not while a fidget plays or is due.
        let has_idle = a.player.movements.iter().any(|m| m.name.starts_with(IDLE_PREFIX));
        let ai = c.ai.as_ref().unwrap();
        let idle_waits = has_idle && (ai.fidget.is_some() || now + 1e-6 >= ai.idle_due);
        match c.state {
            State::Wait if idle_waits => {}
            State::Wait => {
                let line = c.enter_move(&mut a.player, now, false, "wandering, rules 9, 12");
                lines.push(line);
            }
            State::Move => {
                let stuck = std::mem::take(&mut c.walk.stuck_now);
                let ai = c.ai.as_mut().unwrap();
                let route = ai.route.as_ref().unwrap();
                let target = route[ai.point % route.len()];
                let d = ((pos[0] - target[0]).powi(2) + (pos[1] - target[1]).powi(2)).sqrt();
                if d < POINT_REACHED || stuck {
                    let n = route.len();
                    let why = if stuck { "stuck: the next point (rule 7)".to_string() } else { format!("point {} reached ({d:.0} < {POINT_REACHED})", ai.point % n + 1) };
                    ai.point += 1;
                    if ai.point % n == 0 && ai.settings.wander_and_stop && !stuck {
                        ai.wander_done = true;
                        let w = c.set_state(&mut a.player, "Wait", &[], now);
                        lines.push(format!("{why}: the last; wanderAndStop: no more wandering ({w}); its cue is not traced (rule 11)"));
                    } else {
                        let next = route[ai.point % n];
                        ai.closest = ((pos[0] - next[0]).powi(2) + (pos[1] - next[1]).powi(2)).sqrt();
                        c.walk.steer = Some(next);
                        lines.push(format!("{why}; heads for point {} ({:.0}, {:.0}), {:.0} away (rule 11)", ai.point % n + 1, next[0], next[1], ai.closest));
                    }
                } else {
                    if d * d > ai.closest * ai.closest + REAIM_DRIFT * REAIM_DRIFT {
                        c.walk.steer = Some(target);
                        lines.push(format!("drifted off: {d:.0} from point {}, closest was {:.0}: re-aims (rule 11)", ai.point % route.len() + 1, ai.closest));
                        ai.closest = d;
                    }
                    ai.closest = ai.closest.min(d);
                }
            }
            _ => {}
        }
        for l in lines {
            self.character_note(&name, l);
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

impl WorldState {
    /// `KQMonster::setHome <name> x y z` (rule 13): its home point (z −1 =
    /// the floor, kept as written and placed when used); it never moves
    /// the character. Returns the result for `$Console::Return`.
    pub(crate) fn set_home(&mut self, words: &[String], at: &str) -> String {
        let nums: Vec<f32> = words.iter().skip(2).filter_map(|w| w.parse().ok()).collect();
        let (Some(name), [x, y, z, ..]) = (words.get(1), nums.as_slice()) else {
            self.note(format!("{at}: usage: KQMonster::setHome <name> x y z; nothing"));
            return "1".into();
        };
        let Some(i) = self.character_index(name) else {
            self.note(format!("{at}: KQMonster::setHome {name}: no such character; nothing"));
            return "1".into();
        };
        let a = &mut self.anims[i];
        let who = a.player.name.clone();
        let line = match a.character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
            Some(ai) => {
                ai.home = [*x, *y, *z];
                format!("{at}: KQMonster::setHome: home ({x}, {y}, {z}); not moved (rule 13)")
            }
            None => format!("{at}: KQMonster::setHome: no movement rules; nothing"),
        };
        self.character_note(&who, line);
        "0".into()
    }
}
