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
    /// After Connor (rule 23): its target is Connor.
    pub alarmed: bool,
    /// It reacts from this game time on (a random 0–1 s after the alarm);
    /// until then it carries on as before (rule 23).
    pub react_at: f64,
    /// The chase timer runs out at this game time (rules 23, 27).
    pub chase_until: f64,
    /// Where it last saw Connor (rules 23, 25).
    pub last_seen: Option<[f32; 3]>,
    /// `KQMonster::goHome <name>` is on (rule 29).
    pub go_home: bool,
    /// A zombie seen from Birth rises from this game time on (rule 30).
    pub rise_at: Option<f64>,
    /// Took up the chase again on its way home: during its reaction delay
    /// it keeps heading home (rule 28).
    pub retaking: bool,
    /// Within reach of a Connor who is moving: it follows him (rule 25;
    /// for the log, once per change).
    pub following: bool,
    /// Sinking back: its `hide` plays where it gave up (rule 30).
    pub sinking: bool,
    /// Hits counted (`fighting.md` rule 22.4): from the second on it always
    /// runs when it starts moving (rule 3); cleared at home (rule 28).
    pub hit_count: u32,
    /// Hit while not alarmed, with no team (`fighting.md` rule 28): it
    /// runs about with no target, toward a point beside Connor.
    pub hunting: bool,
}

impl Ai {
    pub fn new(settings: AiSettings, home: [f32; 3], eye: f32, has_hide: bool) -> Self {
        Ai { settings, home, eye, has_hide, active: false, sees: false, blocked_by: None, checked: false, unseen: 0, on_way_home: false, has_birth: false, route: None, point: 0, closest: f32::MAX, wander_done: false, idle_due: 0.0, fidget: None, alarmed: false, react_at: 0.0, chase_until: 0.0, last_seen: None, go_home: false, rise_at: None, retaking: false, following: false, sinking: false, hit_count: 0, hunting: false }
    }

    /// Alarmed and past its reaction delay: the chase rules drive it, not
    /// wandering or idling (rules 14, 16, 23).
    pub fn engaged(&self, now: f64) -> bool {
        self.alarmed && now + 1e-6 >= self.react_at
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
            // fighting.md rule 30: the dying have no movement rules.
            if c.state == crate::character::State::Die {
                continue;
            }
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
            // Rule 30: from Birth only when he is within its alarm radius.
            let rises = !ai.active
                && ai.sees
                && state == crate::character::State::Birth
                && ai.settings.alarm_radius > 0.0
                && !ai.go_home
                && connor.is_some_and(|at| distance(at, a.player.base.position) < ai.settings.alarm_radius);
            if check {
                lines.push(status_line(&a.player, c, connor));
            }
            for l in lines {
                self.character_note(&name, l);
            }
            if wants_in {
                self.join_active(i, now);
            }
            self.sunk(i, now, None);
            if rises {
                self.join_active(i, now);
                self.wake(i, now);
            }
            self.rise(i, now);
            self.alarm_rules(i, now);
            self.idle(i, now);
            self.wander(i, now);
        }
        // fighting.md rule 36 (M5 5c-9), ranged-weapons.md rules 28-30 (M5
        // 5f-1): the bolts fly; Connor's place is kept for the next shot's
        // aim.
        self.bolts_step(ground, buildings, objects);
        self.connor_was = self.connor_at;
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
        // Rule 16: not alarmed (also during its reaction delay; wandering
        // carries on then, rule 23).
        if !ai.active || fidgets.is_empty() || ai.on_way_home || ai.hunting || (ai.alarmed && ai.fidget.is_none()) {
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
                // Rule 18: an `attack` or `react` ending sets the chase timer.
                let n = f[IDLE_PREFIX.len()..].to_ascii_lowercase();
                if matches!(n.as_str(), "attack" | "attack2" | "react") {
                    ai.chase_until = now + CHASE_TIME;
                }
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
        if !ai.active || !ai.settings.wanders || ai.wander_done || ai.on_way_home || ai.hunting || ai.engaged(now) {
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
        ai.alarmed = false;
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
                c.unborn = true;
                ai.rise_at = None;
                line.push_str("; its list has hide: back underground (Birth, not drawn, not solid)");
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
                // z −1 = the floor, −2 = the surface (rule 13).
                let mut home = [*x, *y, *z];
                if *z == -1.0 || *z == -2.0 {
                    if let Some(h) = self.heights.as_ref().and_then(|h| h(*x, *y, *z == -1.0)) {
                        home[2] = h;
                    }
                }
                ai.home = home;
                format!("{at}: KQMonster::setHome: home ({x}, {y}, {}); not moved (rule 13)", home[2])
            }
            None => format!("{at}: KQMonster::setHome: no movement rules; nothing"),
        };
        self.character_note(&who, line);
        "0".into()
    }
}

/// Rule 26: it gives up after not seeing Connor for this many updates.
pub const LOSE_SIGHT_UPDATES: u32 = 45;
/// Rule 23: the chase timer (10 s after being hit: the fight study).
pub const CHASE_TIME: f64 = 5.0;
/// Rule 25: out of sight, it aims at the spot where it last saw him only
/// when that is farther than this.
pub const LAST_SEEN_AIM: f32 = 500.0;
/// Rule 28: home is reached within this in x and in y.
pub const HOME_REACHED: f32 = 100.0;

/// The distance in x and y only.
fn flat_distance(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

impl WorldState {
    /// The character's team has a member (other than `i`) that can see
    /// Connor now (rules 24, 26).
    fn team_sees(&self, i: usize, team: Option<&str>) -> bool {
        let Some(team) = team else { return false };
        self.anims.iter().enumerate().any(|(j, a)| {
            j != i
                && !a.gone
                && a.character.as_deref().and_then(|c| c.ai.as_ref()).is_some_and(|ai| ai.sees && ai.settings.team.as_deref().is_some_and(|t| t.eq_ignore_ascii_case(team)))
        })
    }

    /// Rules 23–29 for one character, each update: the alarm, then (once
    /// its reaction delay is over) the chase, losing sight, the leash,
    /// giving up and the way home.
    fn alarm_rules(&mut self, i: usize, now: f64) {
        use crate::character::State;
        let Some(connor) = self.connor_at else { return };
        let a = &self.anims[i];
        let Some(c) = a.character.as_deref() else { return };
        let Some(ai) = c.ai.as_ref() else { return };
        if !ai.active {
            return;
        }
        let (pos, state) = (a.player.base.position, c.state);
        let d = distance(pos, connor);
        let s = ai.settings.clone();
        let (sees, home, go_home) = (ai.sees, ai.home, ai.go_home);
        // Rules 27, 28: the home distance is 3D (Q133).
        let home_d = distance(pos, home);
        // Rule 30 (Q134): nothing underground or while sinking; a zombie
        // rises again only by the rising test, from its home.
        if state == State::Birth {
            return;
        }
        if self.ai_off {
            // The test switch (ours): an alarmed or hunting character
            // gives up; none is alarmed.
            if ai.alarmed || ai.hunting {
                if let Some(ai) = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
                    ai.hunting = false;
                }
                self.give_up(i, "the AI is switched off (test switch, ours)", now);
            } else if ai.on_way_home {
                self.way_home(i, now);
            }
            return;
        }
        if !ai.alarmed && ai.hunting {
            self.hunt(i, d, home_d, now);
            return;
        }
        if !ai.alarmed {
            // Rule 28: only while moving; case 2 needs it to see him too.
            let retake = ai.on_way_home && state == State::Move && !go_home && s.alarm_radius > 0.0 && sees && (d <= s.attack_dist || (home_d <= s.home_radius && d < s.alarm_radius));
            let alarm = !ai.on_way_home && !go_home && s.alarm_radius > 0.0 && sees && d < s.alarm_radius && ai.fidget.is_none() && matches!(state, State::Wait | State::Move);
            if retake {
                self.alarm(i, &format!("on its way home it takes up the chase again (Connor {d:.0} away, home {home_d:.0} away; rule 28: no coin, it keeps '{}')", a.player.movement_name()), false, now);
                if let Some(ai) = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
                    ai.retaking = true;
                }
            } else if alarm {
                self.alarm(i, &format!("sees Connor {d:.0} away, within its alarm radius {} (rule 23)", s.alarm_radius), false, now);
            } else if ai.on_way_home {
                self.way_home(i, now);
            }
            return;
        }
        if now + 1e-6 < ai.react_at {
            // Rule 28: a re-take keeps heading home during its delay.
            if ai.retaking {
                self.way_home(i, now);
            }
            return;
        }
        let team_sees = self.team_sees(i, s.team.as_deref());
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let c = a.character.as_deref_mut().unwrap();
        let ai = c.ai.as_mut().unwrap();
        ai.retaking = false;
        if sees {
            ai.last_seen = Some(connor);
        }
        let mut lines = Vec::new();
        // Rule 26: out of sight for 45 updates.
        let mut give_up = None;
        if !sees && ai.unseen >= LOSE_SIGHT_UPDATES {
            let standing = state == State::Wait;
            let excused = if standing { s.team.is_some() && s.chase_radius > 0.0 } else { team_sees };
            if excused {
                ai.unseen = 0;
                lines.push(format!("not seen for {LOSE_SIGHT_UPDATES} updates, but {}: the count starts again (rule 26)", if standing { "it has a team and a chase radius" } else { "a team member sees him" }));
            } else {
                give_up = Some(format!("has not seen Connor for {LOSE_SIGHT_UPDATES} updates (rule 26)"));
            }
        }
        // Rule 27: the leash, once the chase timer has run out.
        let mut run_home = false;
        if give_up.is_none() && now + 1e-6 >= ai.chase_until {
            // Standing, also with a chase radius of 0 (rule 27).
            let loose = s.team.is_none() || (state == State::Wait && s.chase_radius <= 0.0);
            if loose && d > 2.0 * s.alarm_radius {
                give_up = Some(format!("Connor {d:.0} away, beyond twice its alarm radius (rule 27)"));
            } else if s.chase_radius > 0.0 && home_d > s.chase_radius {
                give_up = Some(format!("{home_d:.0} from home, beyond its chase radius {} (rule 27)", s.chase_radius));
                run_home = true;
            }
        }
        let last_seen = ai.last_seen;
        if let Some(why) = give_up {
            if run_home {
                let r = c.set_moving(&mut a.player, true);
                lines.push(format!("switches to run: {r} (rule 27)"));
            }
            for l in lines {
                self.character_note(&name, l);
            }
            self.give_up(i, &why, now);
            return;
        }
        // Rule 25: what it does.
        let aim = if sees { Some(connor) } else { last_seen.filter(|p| distance(*p, pos) > LAST_SEEN_AIM) };
        let aim2 = aim.map(|p| [p[0], p[1]]);
        let back_key = self.connor_back_key;
        let mut fight_lines = Vec::new();
        match state {
            State::Wait => {
                if let Some(p) = aim2 {
                    if c.turn_to.is_none() && !c.no_turn {
                        c.ask_turn(p);
                    }
                }
                // fighting.md rule 7 (M5 5c-3): standing, it attacks when it
                // can see him, he is closer than its reach (ranged 1.25 ×),
                // the attack is due and its movement can be cut short.
                fight_lines.extend(attack_now(&mut a.player, c, sees, d, connor, back_key, now, false));
                // Rule 25: ranged (doMode 1) start beyond 1.25 ×. During a
                // zombie's `birth` its walk waits for `birth` to end (rule
                // 30), and its stuck check for the walk (rule 7).
                let start = if s.do_mode == 1 { 1.25 * s.attack_dist } else { s.attack_dist };
                if (!sees || d > start) && s.chase_radius > 0.0 {
                    // Rule 3, fighting.md rule 22.4: hit twice, it always runs.
                    let hits = c.ai.as_ref().map_or(0, |ai| ai.hit_count);
                    let run = hits >= 2 || c.random() < 0.5;
                    let why = if hits >= 2 {
                        "after Connor, running (hit twice: always, fighting.md rule 22.4)"
                    } else if run {
                        "after Connor, running (the coin, rule 3)"
                    } else {
                        "after Connor, walking (the coin, rule 3)"
                    };
                    let line = c.enter_move(&mut a.player, now, run, why);
                    c.walk.steer = aim2;
                    lines.push(line);
                }
            }
            State::Move => {
                c.walk.steer = aim2;
                // Rule 25 (Q133 b): a melee character within reach stops only
                // while Connor has no forward / back key; else it follows him
                // at reach. Ranged (doMode 1): stops within 0.75 ×.
                let ranged = s.do_mode == 1;
                let reach = if ranged { 0.75 * s.attack_dist } else { s.attack_dist };
                let moving = self.connor_move_key;
                if c.attack_playing(&a.player) {
                    // fighting.md rule 11b, Do instead (the user's decision):
                    // while its `Attack` plays, no stop rule; it stays in
                    // Move, steering at him.
                } else if let Some(line) = attack_now(&mut a.player, c, sees, d, connor, back_key, now, true) {
                    // fighting.md rule 7 (M5 5c-3): moving, it attacks when
                    // it can see him, he is closer than its reach (1 ×,
                    // ranged too) and the attack is due, whatever he does.
                    c.ai.as_mut().unwrap().following = false;
                    fight_lines.push(line);
                } else {
                    let ai = c.ai.as_mut().unwrap();
                    if sees && d <= reach && (ranged || !moving) {
                        ai.following = false;
                        let w = c.set_state(&mut a.player, "Wait", &[], now);
                        lines.push(format!(
                            "Connor {d:.0} away, within its {} {reach:.0}{}: stops, facing him ({w}); its attack is not due yet (rule 25; fighting.md rule 7)",
                            if ranged { "stop distance" } else { "attack distance" },
                            if ranged { " (doMode 1)" } else { ", and he has no forward / back key" }
                        ));
                        c.ask_turn([connor[0], connor[1]]);
                    } else if sees && d <= reach {
                        if !ai.following {
                            ai.following = true;
                            lines.push(format!("Connor {d:.0} away, within reach but moving (forward / back key): it follows him at reach (rule 25)"));
                        }
                    } else {
                        ai.following = false;
                    }
                }
            }
            _ => {}
        }
        for l in lines {
            self.character_note(&name, l);
        }
        for l in fight_lines {
            self.fight_note(&name, l);
        }
    }

    /// For the fight (M5 5c-6): rule 20's join, rule 23's alarm (with its
    /// spread, rule 24), rule 26's team test.
    pub(crate) fn join_active_pub(&mut self, i: usize, now: f64) {
        self.join_active(i, now);
    }
    pub(crate) fn alarm_pub(&mut self, i: usize, why: &str, now: f64) {
        self.alarm(i, why, false, now);
    }
    pub(crate) fn team_sees_pub(&self, i: usize, team: Option<&str>) -> bool {
        self.team_sees(i, team)
    }

    /// Rule 23: alarmed (`told`: by another, rule 24, no distance test).
    fn alarm(&mut self, i: usize, why: &str, told: bool, now: f64) {
        if self.ai_off {
            let name = self.anims[i].player.name.clone();
            self.character_note(&name, format!("not alarmed ({why}): the AI is switched off (test switch, ours)"));
            return;
        }
        let connor = self.connor_at;
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let delay = f64::from(c.random());
        let Some(ai) = c.ai.as_mut() else { return };
        ai.alarmed = true;
        ai.on_way_home = false;
        ai.last_seen = connor;
        ai.react_at = now + delay;
        ai.chase_until = now + CHASE_TIME;
        let line = format!("alarmed: {why}; reacts in {delay:.2} s (rule 23); chase timer {CHASE_TIME} s; no alarm line in its file (Daventry has none)");
        let team = ai.settings.team.clone();
        self.character_note(&name, line);
        if !told {
            self.spread_alarm(i, &name, team.as_deref(), now);
        }
    }

    /// Rule 24: the active characters with an alarm radius that see Connor
    /// (or whose team has a member that does), and the alarmed one's team
    /// members (made active if needed), are alarmed too. Being told does
    /// not tell others again (**ours**).
    fn spread_alarm(&mut self, from: usize, who: &str, team: Option<&str>, now: f64) {
        for j in 0..self.anims.len() {
            if j == from || self.anims[j].gone {
                continue;
            }
            let Some(ai) = self.anims[j].character.as_deref().and_then(|c| c.ai.as_ref()) else { continue };
            if ai.sinking {
                // Rule 30: told while its `hide` plays: underground at
                // home at once.
                self.sunk(j, now, Some(who));
                continue;
            }
            if ai.alarmed || ai.go_home || ai.settings.alarm_radius <= 0.0 {
                continue;
            }
            let same_team = team.is_some_and(|t| ai.settings.team.as_deref().is_some_and(|u| u.eq_ignore_ascii_case(t)));
            let other_team = ai.settings.team.clone();
            let active = ai.active;
            let sees = active && (ai.sees || self.team_sees(j, other_team.as_deref()));
            if !(sees || same_team) {
                continue;
            }
            if !active {
                // "made active if needed" (rule 24): **ours**, even past
                // MaxMonsters.
                let n = self.anims[j].player.name.clone();
                self.active_list.push(n.to_ascii_lowercase());
                self.anims[j].character.as_deref_mut().and_then(|c| c.ai.as_mut()).unwrap().active = true;
                let line = format!("made active by its team member {who}'s alarm (rule 24): {} on the list", self.active_list.len());
                self.character_note(&n, line);
            }
            let why = if same_team { format!("told by its team member {who} (rule 24)") } else { format!("told by {who}: it can see Connor, or a team member can (rule 24)") };
            self.alarm(j, &why, true, now);
        }
    }

    /// Rule 28: gives up: forgets its target, on its way home; a list with
    /// `hide` goes back into the ground (Birth, `hide`).
    fn give_up(&mut self, i: usize, why: &str, now: f64) {
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_mut() else { return };
        ai.alarmed = false;
        ai.on_way_home = true;
        let mut line = format!("gives up: {why}; on its way home (rule 28)");
        if ai.has_hide {
            // Rule 30: `hide` where it stands, still drawn, solid, active.
            ai.sinking = true;
            c.state = crate::character::State::Birth;
            let h = match a.player.request("hide") {
                crate::named_anim::Request::AtOnce(_) => "starts at once",
                _ => "waits",
            };
            line.push_str(&format!("; its list has hide: Birth, 'hide' {h}, still drawn and solid (rules 28, 30)"));
        }
        self.character_note(&name, line);
        self.way_home(i, now);
    }

    /// Rule 28: on its way home, it steers home each update and keeps
    /// moving; within 100 in x and y → Wait (its hit count cleared: the
    /// fight study).
    fn way_home(&mut self, i: usize, now: f64) {
        use crate::character::State;
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        if c.state == State::Birth {
            return;
        }
        let Some(ai) = c.ai.as_mut() else { return };
        let (pos, home) = (a.player.base.position, ai.home);
        let line = if (pos[0] - home[0]).abs() <= HOME_REACHED && (pos[1] - home[1]).abs() <= HOME_REACHED {
            ai.on_way_home = false;
            // Rule 28: its hit count is cleared (fighting.md rule 22.4).
            ai.hit_count = 0;
            let w = c.set_state(&mut a.player, "Wait", &[], now);
            Some(format!("home reached ({:.0}, {:.0}): {w} (rule 28)", home[0], home[1]))
        } else {
            let line = if c.state == State::Wait && ai.settings.chase_radius <= 0.0 {
                // Rule 28: a guard stays where it is.
                ai.on_way_home = false;
                Some("gave up standing with a chase radius of 0: stays where it is, no longer on its way home (rule 28)".to_string())
            } else if c.state == State::Wait {
                Some(c.enter_move(&mut a.player, now, true, "on its way home, at a run, rule 28"))
            } else {
                None
            };
            c.walk.steer = Some([home[0], home[1]]);
            line
        };
        if let Some(l) = line {
            self.character_note(&name, l);
        }
    }

    /// `KQMonster::goHome <name> [0]`, `setAlarmRadius`, `setChaseRadius`,
    /// `setHomeRadius <name> <value>` (rules 23, 27, 29). Returns the
    /// result for `$Console::Return`.
    pub(crate) fn monster_ai_command(&mut self, words: &[String], at: &str) -> String {
        let cmd = words[0].clone();
        let Some(i) = words.get(1).and_then(|n| self.character_index(n)) else {
            self.note(format!("{at}: {}: no such character; nothing", words.join(" ")));
            return "1".into();
        };
        let value: Option<f32> = words.get(2).and_then(|v| v.parse().ok());
        let now = self.now();
        let name = self.anims[i].player.name.clone();
        let moving = self.anims[i].character.as_deref().is_some_and(|c| c.state == crate::character::State::Move);
        let Some(ai) = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()) else {
            self.note(format!("{at}: {}: no movement rules; nothing", words.join(" ")));
            return "1".into();
        };
        let line = match (cmd.as_str(), value) {
            ("KQMonster::goHome", v) if v.is_none_or(|v| v != 0.0) => {
                ai.go_home = true;
                let was = ai.alarmed;
                self.character_note(&name, format!("{at}: KQMonster::goHome: walks home, not alarmed until goHome 0 (rule 29)"));
                if was || moving {
                    self.give_up(i, "KQMonster::goHome (rule 29)", now);
                } else {
                    let ai = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()).unwrap();
                    ai.on_way_home = true;
                    self.way_home(i, now);
                }
                return "0".into();
            }
            ("KQMonster::goHome", _) => {
                ai.go_home = false;
                "goHome 0: it can be alarmed again (rule 29)".to_string()
            }
            ("KQMonster::setAlarmRadius", Some(v)) => {
                ai.settings.alarm_radius = v;
                format!("alarm radius {v} (rule 23)")
            }
            ("KQMonster::setChaseRadius", Some(v)) => {
                ai.settings.chase_radius = v;
                format!("chase radius {v} (rule 27)")
            }
            ("KQMonster::setHomeRadius", Some(v)) => {
                ai.settings.home_radius = v;
                format!("home radius {v} (rule 27)")
            }
            _ => {
                self.note(format!("{at}: usage: {cmd} <name> <value>; nothing"));
                return "1".into();
            }
        };
        self.character_note(&name, format!("{at}: {cmd}: {line}"));
        "0".into()
    }
}

/// Rule 30: Connor's ball radius, added to the zombie's for "not standing
/// on it" (his `BOUNDS.KQ` box, 200).
pub const CONNOR_BALL: f32 = 200.0;

impl WorldState {
    /// Rule 30: a zombie in Birth that has just become active is alarmed;
    /// it rises `birthDelay` seconds later.
    fn wake(&mut self, i: usize, now: f64) {
        let Some(c) = self.anims[i].character.as_deref() else { return };
        if !c.ai.as_ref().is_some_and(|ai| ai.active) {
            return;
        }
        let delay = f64::from(c.birth_delay.max(0.0));
        self.alarm(i, "seen from underground within its alarm radius (rule 30)", false, now);
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let ai = a.character.as_deref_mut().and_then(|c| c.ai.as_mut()).unwrap();
        ai.rise_at = Some(now + delay);
        self.character_note(&name, format!("rises in {delay} s (birthDelay, rule 30)"));
    }

    /// Rule 30: when its rise is due and Connor is not standing on it
    /// (farther than its ball and his added), it plays `birth`, is drawn
    /// and solid, its gravity on, turned toward him; in Wait.
    fn rise(&mut self, i: usize, now: f64) {
        let connor = self.connor_at;
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_ref() else { return };
        let Some(t) = ai.rise_at else { return };
        // **Stand-in** (Q134): not before a `hide` under way has ended.
        if now + 1e-6 < t || c.state != crate::character::State::Birth || !c.unborn {
            return;
        }
        let pos = a.player.base.position;
        let ball = c.bounds.map_or(0.0, |(min, max)| crate::collide::BallStack::from_box(min, max).radius);
        let d = connor.map_or(f32::MAX, |at| flat_distance(at, pos));
        if d <= ball + CONNOR_BALL {
            return;
        }
        c.ai.as_mut().unwrap().rise_at = None;
        c.unborn = false;
        c.gravity = true;
        let w = c.set_state(&mut a.player, "Wait", &[], now);
        let st = a.player.start("birth");
        if let Some(at) = connor {
            c.ask_turn([at[0], at[1]]);
        }
        let birth = if st.unknown { "no 'birth' in its list (movement 0)" } else { "'birth' starts at once" };
        self.character_note(&name, format!("rises (rule 30): Connor {d:.0} away (more than {:.0}); {w}; {birth}; drawn and solid, gravity on, turns toward him", ball + CONNOR_BALL));
    }
}

impl WorldState {
    /// Rule 30 (Q134): a sinking zombie whose `hide` has ended (or that was
    /// told by `told_by` during it) leaves the active list, is
    /// underground (not drawn, not solid), its gravity off, on its home
    /// point, and no longer on its way home.
    fn sunk(&mut self, i: usize, _now: f64, told_by: Option<&str>) {
        let heights = self.heights.as_ref();
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_mut() else { return };
        if !ai.sinking || (told_by.is_none() && !c.unborn) {
            return;
        }
        ai.sinking = false;
        ai.on_way_home = false;
        ai.alarmed = false;
        ai.active = false;
        let mut home = ai.home;
        if home[2] == -1.0 || home[2] == -2.0 {
            if let Some(z) = heights.and_then(|h| h(home[0], home[1], home[2] == -1.0)) {
                home[2] = z;
            }
        }
        c.unborn = true;
        c.gravity = false;
        a.player.base.position = home;
        self.active_list.retain(|n| !n.eq_ignore_ascii_case(&name));
        let why = told_by.map_or("its 'hide' ended".to_string(), |w| format!("told by {w} while its 'hide' played"));
        let line = format!("underground (rule 30): {why}; off the active list, not drawn, not solid, gravity off, on its home point ({:.0}, {:.0}, {:.0})", home[0], home[1], home[2]);
        self.character_note(&name, line);
    }
}

/// `fighting.md` rule 7 (M5 5c-3): the attack starts now when it can see
/// Connor, he is closer than its reach (a ranged one standing: 1.25 ×),
/// its attack is due and its running movement can be cut short or has
/// ended (otherwise it is tried again next update). `moving`: from Move.
/// Returns the log line when it starts.
#[allow(clippy::too_many_arguments)]
fn attack_now(player: &mut crate::named_anim::NamedAnimPlayer, c: &mut crate::character::Character, sees: bool, d: f32, connor: [f32; 3], back_key: bool, now: f64, moving: bool) -> Option<String> {
    use crate::connor_auto::{heading_towards, wrap};
    let f = c.fight.as_ref()?;
    let reach = c.ai.as_ref()?.settings.attack_dist;
    let limit = if !moving && f.ranged() { crate::fight::RANGED_STANDING * reach } else { reach };
    if !sees || d >= limit || !f.due(now) || !(player.cut_short || player.has_ended()) {
        return None;
    }
    let before = f.attack_time_text();
    let off = wrap(heading_towards(player.base.position, [connor[0], connor[1]]) - crate::character::heading_of(&player.base.rotation));
    let was = player.movement_name().to_string();
    let how = c.start_attack(player, now);
    Some(format!(
        "attack (rule 7, {}): Connor {d:.0} away (closer than {limit:.0}), {:.0} deg off its heading, his back key {}; attack time {before} -> never; '{was}' replaced: {how}; chase timer 5 s (rules 8, 9)",
        if moving { "moving" } else { "standing" },
        off.to_degrees(),
        if back_key { "held" } else { "not held" }
    ))
}

/// `fighting.md` rule 28: the hunt's point is this far (radians,
/// anticlockwise) off the straight line to Connor ("Do not reproduce":
/// the original's point is about twice that, from a plain arctangent).
pub const HUNT_ANGLE: f32 = 0.52;
/// Rule 28: a hunting character gives up beyond this × its alarm radius
/// from Connor (after its chase timer).
pub const HUNT_GIVE_UP: f32 = 4.0;

impl WorldState {
    /// `fighting.md` rule 28 (M5 5c-6): hit while not alarmed, with no
    /// team, it runs with no target. Each update: Connor within its alarm
    /// radius → alarmed at once (no reaction delay); after its chase
    /// timer, beyond 4 × its alarm radius from him it gives up, beyond its
    /// chase radius from home it runs home; otherwise it heads for the
    /// point at his distance, 0.52 rad anticlockwise off the line to him
    /// (the Do instead).
    fn hunt(&mut self, i: usize, d: f32, home_d: f32, now: f64) {
        let Some(connor) = self.connor_at else { return };
        let a = &mut self.anims[i];
        let name = a.player.name.clone();
        let Some(c) = a.character.as_deref_mut() else { return };
        let Some(ai) = c.ai.as_mut() else { return };
        let s = ai.settings.clone();
        if s.alarm_radius > 0.0 && d < s.alarm_radius {
            ai.hunting = false;
            self.alarm(i, &format!("hunting, Connor {d:.0} away, within its alarm radius {} (fighting.md rule 28)", s.alarm_radius), false, now);
            if let Some(ai) = self.anims[i].character.as_deref_mut().and_then(|c| c.ai.as_mut()) {
                ai.react_at = now;
            }
            self.fight_note(&name, "alarmed while hunting: no reaction delay (fighting.md rule 28)");
            return;
        }
        if now + 1e-6 >= ai.chase_until {
            if d > HUNT_GIVE_UP * s.alarm_radius {
                ai.hunting = false;
                self.give_up(i, &format!("hunting: Connor {d:.0} away, beyond {HUNT_GIVE_UP} x its alarm radius (fighting.md rule 28)"), now);
                return;
            }
            if home_d > s.chase_radius {
                ai.hunting = false;
                self.give_up(i, &format!("hunting: {home_d:.0} from home, beyond its chase radius {} (fighting.md rule 28)", s.chase_radius), now);
                return;
            }
        }
        let p = a.player.base.position;
        let (vx, vy) = (connor[0] - p[0], connor[1] - p[1]);
        let (sn, cs) = HUNT_ANGLE.sin_cos();
        c.walk.steer = Some([p[0] + vx * cs - vy * sn, p[1] + vx * sn + vy * cs]);
    }
}
