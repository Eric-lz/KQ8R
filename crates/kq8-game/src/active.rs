//! The world's active objects while the game plays (M3 step 3a): doors,
//! the event queue, the scripts they run, and Connor's clicks.
//!
//! Rules from `docs/behaviour/doors.md`; how one update is ordered is our
//! stand-in (docs/DESIGN.md M3 3a-2, Q45 d):
//!
//! 1. clicks (handled by [`Active::click`] before [`Active::update`]);
//! 2. events due by now, also those just sent with delay 0;
//! 3. each moving door moves (one started in this update waits for the
//!    next), its cues fire, and a finished `open` / `close` calls the
//!    script with `Open` / `Close` (events sent there wait for the next
//!    update).
//!
//! Every line worth checking goes to [`WorldState::log`], which the caller
//! writes to `doors.log`.
//!
//! Traps (M3 step 3d-2, `docs/behaviour/traps.md`) share the variables,
//! scripts and event queue. Each trap's script gets `Init` at the start
//! (rule 13); after Connor's update, if he moved, every trap checks him
//! ([`Active::connor_moved`], `crate::trap`) and runs its script with `On`,
//! `Off` or `Cue` (rules 6-8); `KQCueEvent`s sent to a trap run it with
//! `Cue` (rule 14); `KQObject::collision` changes a trap's mask (rule 16).
//! Lines from trap calls go to [`WorldState::trap_log`] (`traps.log`).

use std::collections::{BTreeMap, BTreeSet};

use kq8_data::world_script::Vars;

use crate::click::{distance_to_box, ray_box, CLICKABLE};
use crate::collide::{pair_collides, BoxCollider, Placement, CONNOR_MASK, CONNOR_TYPE};
use crate::door::{turn_degrees, Door};
use crate::events::{EventQueue, Pending};
use crate::script::{looks_like_number, number_value, ScriptHost, Scripts};
use crate::trap::{Trap, Visitor};

/// Event types we know (doors.md "Commands door scripts use"). Any other
/// is an error and is not sent.
pub const EVENT_TYPES: [&str; 4] = ["KQDoorEvent", "KQAnimPlayEvent", "KQCueEvent", "KQStatsChangeRequestEvent"];

/// A door and what clicking and collision need to know about it.
#[derive(Debug, Clone)]
pub struct ActiveDoor {
    pub door: Door,
    /// `script=`, if any.
    pub script: Option<String>,
    /// Its `BOUNDS.KQ` box (own space); `None` = no box (never clicked,
    /// never collides).
    pub bounds: Option<([f32; 3], [f32; 3])>,
    /// `colliderType` now (scripts change it).
    pub collision_type: u32,
    pub collision_mask: u32,
    /// `acceptsInventory=<item type>`.
    pub accepts_inventory: Option<String>,
    /// `acceptAllItems=1`.
    pub accept_all_items: bool,
    /// `tooFarAwayAction` (default 0).
    pub too_far_away_action: i64,
    /// `doNotPointAt` set.
    pub do_not_point_at: bool,
}

impl ActiveDoor {
    /// Its collider where it stands now (the box turns with the door).
    pub fn collider(&self) -> Option<BoxCollider> {
        self.bounds.map(|(min, max)| BoxCollider { name: self.door.name.clone(), min, max, placement: self.door.placement() })
    }

    /// Collides with Connor by the type and mask rule.
    pub fn collides_with_connor(&self) -> bool {
        self.bounds.is_some() && pair_collides(CONNOR_TYPE, CONNOR_MASK, self.collision_type, self.collision_mask)
    }
}

/// Any other object that can be clicked: never moves in this step.
#[derive(Debug, Clone)]
pub struct Clickable {
    pub name: String,
    pub class: String,
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub placement: Placement,
    pub collision_type: u32,
}

/// An inventory item in Connor's hand (only a test option sets one; no
/// inventory yet).
#[derive(Debug, Clone, PartialEq)]
pub struct Item {
    /// The item object's name (`$3`).
    pub object: String,
    /// Its type name, e.g. `INVITEM_KeepKey` (`$4`).
    pub kind: String,
}

/// What a click led to.
#[derive(Debug, Clone, PartialEq)]
pub enum ClickResult {
    /// No such object, or it is not clickable.
    Nothing,
    /// A clickable object that is not a door: logged only (doors only in
    /// this step).
    NotADoor,
    /// The door's script ran with these arguments.
    Script(Vec<String>),
    /// Out of reach, `tooFarAwayAction` 0: Connor's remark, nothing sent.
    TooFar,
    /// The item is not accepted: Connor's remark, no script.
    Refused,
}

/// Everything a script can touch, apart from the variables.
pub struct WorldState {
    pub doors: Vec<ActiveDoor>,
    pub traps: Vec<Trap>,
    pub clickables: Vec<Clickable>,
    /// Every loaded object's name, lower case (for "unknown object").
    pub known: BTreeSet<String>,
    /// The world's scripts by lower-case file name.
    sources: BTreeMap<String, Vec<u8>>,
    pub queue: EventQueue,
    /// Updates done; game time = updates / 15.
    pub updates: u64,
    /// Lines for `doors.log`, taken by the caller.
    pub log: Vec<String>,
    /// Lines for `traps.log`: everything while a trap's call runs.
    pub trap_log: Vec<String>,
    /// A trap's call is running: lines go to `trap_log`.
    in_trap: bool,
}

impl WorldState {
    pub fn now(&self) -> f64 {
        self.updates as f64 / crate::UPDATES_PER_SECOND
    }

    fn note(&mut self, line: impl AsRef<str>) {
        let line = format!("update {}: {}", self.updates, line.as_ref());
        if self.in_trap {
            self.trap_log.push(line);
        } else {
            self.log.push(line);
        }
    }

    pub fn door_index(&self, name: &str) -> Option<usize> {
        self.doors.iter().position(|d| d.door.name.eq_ignore_ascii_case(name))
    }

    pub fn trap_index(&self, name: &str) -> Option<usize> {
        self.traps.iter().position(|t| t.settings.name.eq_ignore_ascii_case(name))
    }

    /// `KQObject::collision <name> <+-bits>`: adds or removes bits of an
    /// object's collision mask (traps.md rule 16; traps and doors).
    fn collision_mask(&mut self, words: &[String], at: &str) -> String {
        let (Some(name), Some(bits)) = (words.get(1), words.get(2)) else {
            self.note(format!("{at}: usage: KQObject::collision <name> <bits>"));
            return "1".into();
        };
        let Ok(bits) = bits.parse::<i64>() else {
            self.note(format!("{at}: collision {name} {bits}: not a whole number"));
            return "1".into();
        };
        let change = |m: u32| if bits >= 0 { m | bits as u32 } else { m & !(bits.unsigned_abs() as u32) };
        let (old, new, inside) = if let Some(i) = self.trap_index(name) {
            let t = &mut self.traps[i];
            let old = t.settings.mask;
            t.settings.mask = change(old);
            (old, t.settings.mask, t.inside.clone())
        } else if let Some(i) = self.door_index(name) {
            let d = &mut self.doors[i];
            let old = d.collision_mask;
            d.collision_mask = change(old);
            (old, d.collision_mask, Vec::new())
        } else {
            let why = if self.known.contains(&name.to_ascii_lowercase()) { "object kind not tracked yet" } else { "unknown object" };
            self.note(format!("{at}: collision {name} {bits}: {why}; not changed"));
            return "1".into();
        };
        let mut line = format!("{at}: collision {name} {bits:+}: mask {old:#x} -> {new:#x}");
        if !inside.is_empty() {
            // traps.md open question 1; stand-in: no Off.
            line.push_str(&format!("; inside now: {} (stay listed, no Off sent: stand-in)", inside.join(", ")));
        }
        self.note(line);
        "0".into()
    }

    /// `KQObject::collisionType <name> <±bits>`.
    fn collision_type(&mut self, words: &[String], at: &str) -> String {
        let (Some(name), Some(bits)) = (words.get(1), words.get(2)) else {
            self.note(format!("{at}: usage: KQObject::collisionType <name> <bits>"));
            return "1".into();
        };
        let Ok(bits) = bits.parse::<i64>() else {
            self.note(format!("{at}: collisionType {name} {bits}: not a whole number"));
            return "1".into();
        };
        if bits == 0 {
            self.note(format!("{at}: collisionType {name} 0: refused"));
            return "1".into();
        }
        let change = |t: u32| if bits > 0 { t | bits as u32 } else { t & !(bits.unsigned_abs() as u32) };
        let (old, new) = if let Some(i) = self.door_index(name) {
            let d = &mut self.doors[i];
            let old = d.collision_type;
            d.collision_type = change(old);
            (old, d.collision_type)
        } else if let Some(c) = self.clickables.iter_mut().find(|c| c.name.eq_ignore_ascii_case(name)) {
            let old = c.collision_type;
            c.collision_type = change(old);
            (old, c.collision_type)
        } else {
            let why = if self.known.contains(&name.to_ascii_lowercase()) { "object kind not tracked yet" } else { "unknown object" };
            self.note(format!("{at}: collisionType {name} {bits}: {why}; not changed"));
            return "1".into();
        };
        self.note(format!(
            "{at}: collisionType {name} {bits:+}: {old:#x} -> {new:#x} (clickable {} -> {})",
            old & CLICKABLE != 0,
            new & CLICKABLE != 0
        ));
        if (old ^ new) & CONNOR_MASK != 0 {
            // No Daventry door script does this; the collider list is not
            // rebuilt in this step.
            self.note(format!("{at}: NOTE bits Connor collides with changed on {name}; not applied to his collision (not done)"));
        }
        "0".into()
    }

    /// `sendEvent <delay> <object> <type> [data…]`.
    fn send_event(&mut self, words: &[String], at: &str) -> String {
        if words.len() < 4 {
            self.note(format!("{at}: usage: sendEvent <delay> <object> <EventType> [data]: '{}'", words.join(" ")));
            return "1".into();
        }
        let (delay, target, kind) = (&words[1], &words[2], &words[3]);
        if !looks_like_number(delay) {
            self.note(format!("{at}: sendEvent: delay '{delay}' is not a number; not sent"));
            return "1".into();
        }
        if !self.known.contains(&target.to_ascii_lowercase()) {
            self.note(format!("{at}: ERROR sendEvent to unknown object '{target}'; not sent"));
            return "1".into();
        }
        if !EVENT_TYPES.contains(&kind.as_str()) {
            self.note(format!("{at}: ERROR sendEvent: event type '{kind}' unknown to us; not sent"));
            return "1".into();
        }
        let delay = number_value(delay);
        let now = self.now();
        self.queue.send(now, delay, target, kind, &words[4..], at);
        self.note(format!("{at}: sendEvent {delay} {target} {kind} {} (due at {:.4} s)", words[4..].join(" "), now + delay.max(0.0)));
        "0".into()
    }
}

impl ScriptHost for WorldState {
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let base = name.rsplit(['/', '\\']).next().unwrap_or(name).to_ascii_lowercase();
        self.sources.get(&base).cloned()
    }

    fn world_command(&mut self, words: &[String], at: &str) -> Option<String> {
        match words[0].as_str() {
            "sendEvent" => Some(self.send_event(words, at)),
            "KQObject::collisionType" => Some(self.collision_type(words, at)),
            "KQObject::collision" => Some(self.collision_mask(words, at)),
            _ => None,
        }
    }

    fn log(&mut self, line: String) {
        self.note(line);
    }
}

/// The scripts and the world they act on.
pub struct Active {
    pub scripts: Scripts,
    pub world: WorldState,
}

impl Active {
    /// `vars`: the variables the start-up scripts left; `sources`: every
    /// `.cs` of the world (name, bytes); `known`: every loaded object's
    /// name.
    pub fn new(
        vars: Vars,
        doors: Vec<ActiveDoor>,
        clickables: Vec<Clickable>,
        known: impl IntoIterator<Item = String>,
        sources: impl IntoIterator<Item = (String, Vec<u8>)>,
    ) -> Self {
        let mut known: BTreeSet<String> = known.into_iter().map(|n| n.to_ascii_lowercase()).collect();
        known.extend(doors.iter().map(|d| d.door.name.to_ascii_lowercase()));
        known.extend(clickables.iter().map(|c| c.name.to_ascii_lowercase()));
        let sources = sources.into_iter().map(|(n, b)| (n.to_ascii_lowercase(), b)).collect();
        Active {
            scripts: Scripts::new(vars),
            world: WorldState {
                doors,
                traps: Vec::new(),
                clickables,
                known,
                sources,
                queue: EventQueue::default(),
                updates: 0,
                log: Vec::new(),
                trap_log: Vec::new(),
                in_trap: false,
            },
        }
    }

    /// Adds the world's traps (before [`Active::start`]).
    pub fn add_traps(&mut self, traps: impl IntoIterator<Item = Trap>) {
        for t in traps {
            self.world.known.insert(t.settings.name.to_ascii_lowercase());
            self.world.traps.push(t);
        }
    }

    /// Runs trap `i`'s script with `args` (`$1` is the trap's name); its
    /// lines go to the trap log.
    fn call_trap(&mut self, i: usize, args: &[&str]) {
        let was = std::mem::replace(&mut self.world.in_trap, true);
        let t = &self.world.traps[i];
        let name = t.settings.name.clone();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match t.settings.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
            }
            None => self.world.note(format!("trap {name} has no script; {} not called", all.join(" "))),
        }
        self.world.in_trap = was;
    }

    /// After an object's move (Connor's, in this step): every trap checks
    /// it and runs its script for `On`, `Off` or `Cue` (traps.md rules
    /// 6-9). Returns the calls made, as `<trap> <args>`.
    pub fn connor_moved(&mut self, v: &Visitor) -> Vec<String> {
        let mut calls = Vec::new();
        for i in 0..self.world.traps.len() {
            let shrunk_before = self.world.traps[i].shrunk;
            let c = self.world.traps[i].check(v);
            let Some(args) = c.call else { continue };
            let t = &self.world.traps[i];
            let (name, gone) = (t.settings.name.clone(), t.gone);
            let change = match (shrunk_before, t.shrunk) {
                (true, false) => "; grows to full",
                (false, true) => "; shrinks again",
                _ => "",
            };
            let line = format!(
                "trap {name}: {} at ({:.0}, {:.0}, {:.0}): touching {}, box centre inside {}, {} box ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0}){change}; inside now: [{}]",
                args.join(" "),
                v.position[0],
                v.position[1],
                v.position[2],
                c.touching,
                c.centre_inside,
                if shrunk_before { "inner (shrunk)" } else { "full" },
                c.min[0],
                c.min[1],
                c.min[2],
                c.max[0],
                c.max[1],
                c.max[2],
                t.inside.join(", ")
            );
            let was = std::mem::replace(&mut self.world.in_trap, true);
            self.world.note(line);
            if gone {
                self.world.note(format!("trap {name}: one-shot, deleted after its On (rule 11)"));
            }
            self.world.in_trap = was;
            calls.push(format!("{name} {}", args.join(" ")));
            let refs: Vec<&str> = args.iter().map(String::as_str).collect();
            self.call_trap(i, &refs);
        }
        calls
    }

    /// Runs door `i`'s script with `args` (`$1` is the door's name).
    fn call_door(&mut self, i: usize, args: &[&str]) {
        let d = &self.world.doors[i];
        let name = d.door.name.clone();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match d.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
            }
            None => self.world.note(format!("{name} has no script; {} not called", all.join(" "))),
        }
    }

    /// Rule 22: each door's script gets `Init` as it enters the world.
    pub fn start(&mut self) {
        for i in 0..self.world.doors.len() {
            self.call_door(i, &["Init"]);
        }
        // traps.md rule 13 (after the doors: our order).
        for i in 0..self.world.traps.len() {
            self.call_trap(i, &["Init"]);
        }
    }

    /// The nearest clickable box the ray `origin + t × dir` meets (rule
    /// 10): doors where they stand now, and the other clickable objects.
    /// Only those boxes are tested (stand-in, Q45 b).
    pub fn pick(&self, origin: [f32; 3], dir: [f32; 3]) -> Option<(String, f32)> {
        let doors = self.world.doors.iter().filter(|d| d.collision_type & CLICKABLE != 0).filter_map(|d| {
            let (min, max) = d.bounds?;
            Some((d.door.name.clone(), ray_box(origin, dir, &d.door.placement(), min, max)?))
        });
        let others = self
            .world
            .clickables
            .iter()
            .filter(|c| c.collision_type & CLICKABLE != 0)
            .filter_map(|c| Some((c.name.clone(), ray_box(origin, dir, &c.placement, c.min, c.max)?)));
        doors.chain(others).min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// Connor clicks object `name` (rules 11–17). `feet`: his position;
    /// `attack_dist`: his `attackDist` (reach = twice that); `item`: what
    /// he holds.
    pub fn click(&mut self, name: &str, feet: [f32; 3], attack_dist: f32, item: Option<&Item>) -> ClickResult {
        let reach = 2.0 * attack_dist;
        let feet_text = format!("({:.0}, {:.0}, {:.0})", feet[0], feet[1], feet[2]);
        let Some(i) = self.world.door_index(name) else {
            let found = self.world.clickables.iter().find(|c| c.name.eq_ignore_ascii_case(name)).cloned();
            match found {
                Some(c) if c.collision_type & CLICKABLE != 0 => {
                    let d = distance_to_box(feet, &c.placement, c.min, c.max);
                    self.world.note(format!(
                        "click {} (class {}): Connor at {feet_text}, distance to its box {d:.1}, reach {reach:.0}: {}; not a door, nothing done (only doors in this step)",
                        c.name,
                        c.class,
                        if d < reach { "in reach" } else { "out of reach" }
                    ));
                    return ClickResult::NotADoor;
                }
                Some(c) => self.world.note(format!("click {}: not clickable (collision type {:#x}); nothing", c.name, c.collision_type)),
                None => self.world.note(format!("click {name}: no clickable object by that name; nothing")),
            }
            return ClickResult::Nothing;
        };
        let d = self.world.doors[i].clone();
        if d.collision_type & CLICKABLE == 0 {
            self.world.note(format!("click {}: not clickable now (collision type {:#x}); nothing", d.door.name, d.collision_type));
            return ClickResult::Nothing;
        }
        let Some((min, max)) = d.bounds else {
            self.world.note(format!("click {}: no box in BOUNDS.KQ; can't be clicked", d.door.name));
            return ClickResult::Nothing;
        };
        let distance = distance_to_box(feet, &d.door.placement(), min, max);
        let in_reach = distance < reach;
        let state = d.door.state.name();
        let head = format!(
            "click {}: Connor at {feet_text}, distance to its box {distance:.1}, reach {reach:.0} (2 x attackDist): {}; state {state}",
            d.door.name,
            if in_reach { "IN REACH" } else { "OUT OF REACH" }
        );
        let args: Vec<String> = match (in_reach, item) {
            (true, None) => vec!["ConnorAction".into(), "do".into(), "do".into()],
            (true, Some(it)) => {
                let accepted = d.accept_all_items || d.accepts_inventory.as_deref() == Some(it.kind.as_str());
                if !accepted {
                    self.world.note(format!(
                        "{head}; holds {} ({}): not accepted (acceptsInventory {:?}): Connor's \"that doesn't work\" remark (line not traced, M4); script not run",
                        it.object, it.kind, d.accepts_inventory
                    ));
                    return ClickResult::Refused;
                }
                vec!["ConnorAction".into(), it.object.clone(), it.kind.clone()]
            }
            (false, _) if d.too_far_away_action == 0 => {
                self.world.note(format!("{head}; tooFarAwayAction 0: Connor's \"too far\" remark (line not traced, M4); nothing sent"));
                return ClickResult::TooFar;
            }
            (false, _) => vec!["ConnorAction".into(), "TooFar".into(), "TooFar".into()],
        };
        self.world.note(format!("{head}; script arguments: {} {}", d.door.name, args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.call_door(i, &refs);
        if !d.do_not_point_at {
            self.world.note(format!("Connor would turn to face {} (rule 16; not done: his states not traced)", d.door.name));
        }
        let mut full = vec![d.door.name.clone()];
        full.extend(args);
        ClickResult::Script(full)
    }

    /// One update (15 per second): events due, then the doors move.
    pub fn update(&mut self) {
        let now = self.world.now();
        let mut handled = 0;
        while let Some(e) = self.world.queue.take_due(now) {
            self.handle(e);
            handled += 1;
            if handled > 1000 {
                self.world.note("more than 1000 events in one update; the rest wait (a guard of ours)");
                break;
            }
        }
        let dt = (1.0 / crate::UPDATES_PER_SECOND) as f32;
        for i in 0..self.world.doors.len() {
            let a = self.world.doors[i].door.advance(dt);
            let d = &self.world.doors[i].door;
            let name = d.name.clone();
            if a.from != a.to {
                let line = format!(
                    "door {name}: movement {} phase {:.4} -> {:.4} frame {} turn {:.2} deg state {}",
                    a.finished_movement.as_deref().unwrap_or(d.movement_name()),
                    a.from,
                    a.to,
                    d.movement().map_or(0, |m| m.frame_at(a.to)),
                    turn_degrees(&d.movement().map_or_else(crate::door::identity, |m| m.turn_at_frame(m.frame_at(a.to)))),
                    d.state.name()
                );
                self.world.note(line);
            }
            for cue in &a.cues {
                let what = if cue.split_whitespace().next().is_some_and(|w| w.eq_ignore_ascii_case("playsound")) {
                    "no audio yet"
                } else {
                    "cue action not done"
                };
                self.world.note(format!("door {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(p) = &a.problem {
                self.world.note(format!("door {name}: {p}"));
            }
            if let Some(m) = &a.finished_movement {
                let d = &self.world.doors[i].door;
                self.world.note(format!(
                    "door {name}: movement '{m}' finished; state {}, shows '{}' (frame {}, turn {:.2} deg){}",
                    d.state.name(),
                    d.movement_name(),
                    d.frame(),
                    turn_degrees(&d.turn()),
                    a.finished.map_or(String::new(), |c| format!("; calls its script with {c}"))
                ));
            }
            if let Some(call) = a.finished {
                self.call_door(i, &[call]);
            }
        }
        self.world.updates += 1;
    }

    fn handle(&mut self, e: Pending) {
        let from = if e.from.is_empty() { String::new() } else { format!(" (sent by {})", e.from) };
        if let Some(i) = self.world.trap_index(&e.target) {
            let was = std::mem::replace(&mut self.world.in_trap, true);
            if e.kind == "KQCueEvent" {
                // traps.md rule 14, as doors (rule 23).
                let n = |k: usize| e.data.get(k).cloned().unwrap_or_else(|| "0".into());
                let mut args = vec!["Cue".to_string(), n(0), n(1)];
                if e.data.get(2).is_some_and(|v| number_value(v) != 0.0) {
                    args.push(e.data[2].clone());
                }
                self.world.note(format!("KQCueEvent {} to trap {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_trap(i, &refs);
            } else {
                self.world.note(format!("event {} {} to trap {}{from}: not handled for traps", e.kind, e.data.join(" "), e.target));
            }
            self.world.in_trap = was;
            return;
        }
        let Some(i) = self.world.door_index(&e.target) else {
            let why = if e.kind == "KQStatsChangeRequestEvent" { "stats are M5" } else { "object kind not implemented yet" };
            self.world.note(format!("event {} {} to {}{from}: not handled ({why})", e.kind, e.data.join(" "), e.target));
            return;
        };
        match e.kind.as_str() {
            "KQDoorEvent" => {
                let value = match e.data.first() {
                    None => None,
                    Some(v) => match v.parse::<i64>() {
                        Ok(n) => Some(n),
                        Err(_) => {
                            self.world.note(format!("KQDoorEvent value '{v}' is not a whole number; counts as no number (toggle)"));
                            None
                        }
                    },
                };
                let door = &mut self.world.doors[i].door;
                let (before, note) = door.door_event(value);
                let (after, name) = (door.state, door.name.clone());
                self.world.note(format!(
                    "KQDoorEvent {} to {name}{from}: state {} -> {}: {note}",
                    value.map_or("(no number)".to_string(), |v| v.to_string()),
                    before.name(),
                    after.name()
                ));
            }
            "KQAnimPlayEvent" => {
                let door = &mut self.world.doors[i].door;
                let name = door.name.clone();
                let line = match e.data.first() {
                    Some(m) => match door.play(m) {
                        Ok(()) => format!("KQAnimPlayEvent {m} to {name}{from}: starts '{m}', state stays {}", door.state.name()),
                        Err(err) => format!("KQAnimPlayEvent {m} to {name}{from}: {err}; nothing"),
                    },
                    None => format!("KQAnimPlayEvent to {name}{from}: no movement named; nothing"),
                };
                self.world.note(line);
            }
            "KQCueEvent" => {
                // Rule 23: Cue, $3, $4, and $5 only if non-zero.
                let n = |k: usize| e.data.get(k).cloned().unwrap_or_else(|| "0".into());
                let mut args = vec!["Cue".to_string(), n(0), n(1)];
                if e.data.get(2).is_some_and(|v| number_value(v) != 0.0) {
                    args.push(e.data[2].clone());
                }
                self.world.note(format!("KQCueEvent {} to {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_door(i, &refs);
            }
            other => self.world.note(format!("event {other} {} to {}{from}: not handled for doors", e.data.join(" "), e.target)),
        }
    }

    /// Lines for the log since the last call.
    pub fn take_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.world.log)
    }

    /// Lines for the trap log since the last call.
    pub fn take_trap_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.world.trap_log)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::door::{tests::test_door, DoorState};

    /// Made-up scripts in the style of the door scripts (not copies).
    const PLAIN: &str = "if test $2 == ConnorAction\n\tsendEvent 0 $1 KQDoorEvent 1\nendif\n";
    const FRONT: &str = "if test $2 == ConnorAction\n  if test $Front::Entered == 0\n    KQMonster::speak Connor $1 1000 11 14 10 1 0\n    set Front::Entered 1\n  endif\n  sendEvent 0 $1 KQDoorEvent 1\nelse if test $2 == Open\n  echo opened $1\nendif\nendif\n";
    const KEY: &str = "if test $2 == ConnorAction\n if test $4 == do\n  echo locked\n else if test $4 == INVITEM_Key\n  deleteObject $3\n  sendEvent 0 $1 KQDoorEvent 0\n  KQObject::collisionType $1 -32\n endif\n endif\nendif\n";
    const SWITCH: &str = "if test $3 == do\n if test $Sw::On == true\n  sendEvent 0 $1 KQDoorEvent 1\n  set Sw::On false\n else\n  sendEvent 0 $1 KQDoorEvent 2\n  set Sw::On true\n endif\nendif\n";
    const CUE: &str = "if test $2 == ConnorAction\n sendEvent .07 $1 KQCueEvent 1 0\nelse if test $2 == Cue\n echo cue $3 $4\nendif\nendif\n";

    fn door_box() -> Option<([f32; 3], [f32; 3])> {
        Some(([-630.0, -190.0, 0.0], [600.0, -60.0, 1730.0]))
    }

    fn active(script: &'static str, vars: &[(&str, &str)]) -> Active {
        let mut d = ActiveDoor {
            door: test_door(None),
            script: Some("door.cs".into()),
            bounds: door_box(),
            collision_type: 0x5021,
            collision_mask: 0,
            accepts_inventory: Some("INVITEM_Key".into()),
            accept_all_items: false,
            too_far_away_action: 0,
            do_not_point_at: false,
        };
        d.door.base = Placement::at([0.0; 3]);
        let mut v = Vars::default();
        for (k, val) in vars {
            v.set(k, val.to_string());
        }
        Active::new(v, vec![d], vec![], ["Connor".to_string()], [("door.cs".to_string(), script.as_bytes().to_vec())])
    }

    /// Connor 1,000 in front of the door (in reach).
    const NEAR: [f32; 3] = [0.0, -1190.0, 0.0];
    const FAR: [f32; 3] = [0.0, -6000.0, 0.0];

    fn state(a: &Active) -> DoorState {
        a.world.doors[0].door.state
    }

    #[test]
    fn click_in_reach_opens_for_15_updates_then_open_and_a_second_click_keeps_it_open() {
        let mut a = active(FRONT, &[("Front::Entered", "0")]);
        let r = a.click("condoor", NEAR, 2350.0, None);
        assert_eq!(r, ClickResult::Script(vec!["Condoor".into(), "ConnorAction".into(), "do".into(), "do".into()]));
        assert_eq!(a.scripts.vars.get("Front::Entered"), Some("1"));
        // The event (delay 0) is handled in the same update.
        a.update();
        assert_eq!(state(&a), DoorState::Opening);
        let mut opening = 1;
        while state(&a) == DoorState::Opening {
            a.update();
            opening += 1;
        }
        assert_eq!((opening, state(&a)), (16, DoorState::Opened), "started in update 1, moved in 2..16");
        let log = a.take_log().join("\n");
        assert!(log.contains("KQDoorEvent 1 to Condoor"), "{log}");
        assert!(log.contains("script door.cs Condoor Open") && log.contains("echo opened Condoor"), "{log}");
        assert!(log.contains("cue 'playsound door4.aud' fires (no audio yet)"), "{log}");
        // Second click: script runs, event ignored, door stays open.
        a.click("Condoor", NEAR, 2350.0, None);
        a.update();
        assert_eq!(state(&a), DoorState::Opened);
        assert!(a.take_log().join("\n").contains("open: ignored while opened"));
    }

    #[test]
    fn too_far_sends_nothing() {
        let mut a = active(PLAIN, &[]);
        assert_eq!(a.click("Condoor", FAR, 2350.0, None), ClickResult::TooFar);
        a.update();
        assert_eq!(state(&a), DoorState::Closed);
        assert!(a.world.queue.is_empty());
        // With tooFarAwayAction set the script gets TooFar TooFar.
        a.world.doors[0].too_far_away_action = 1;
        let r = a.click("Condoor", FAR, 2350.0, None);
        assert_eq!(r, ClickResult::Script(vec!["Condoor".into(), "ConnorAction".into(), "TooFar".into(), "TooFar".into()]));
    }

    #[test]
    fn an_accepted_item_unlocks_and_makes_the_door_unclickable() {
        let mut a = active(KEY, &[]);
        let wrong = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("Condoor", NEAR, 2350.0, Some(&wrong)), ClickResult::Refused);
        let key = Item { object: "key.1".into(), kind: "INVITEM_Key".into() };
        a.click("Condoor", NEAR, 2350.0, Some(&key));
        a.update();
        assert_eq!(state(&a), DoorState::Opening, "KQDoorEvent 0 toggles a closed door open");
        assert_eq!(a.world.doors[0].collision_type, 0x5001);
        assert_eq!(a.click("Condoor", NEAR, 2350.0, None), ClickResult::Nothing, "-32: no longer clickable");
        assert_eq!(a.pick([0.0, -5000.0, 1000.0], [0.0, 1.0, 0.0]), None);
        let log = a.take_log().join("\n");
        assert!(log.contains("stub (later milestone, nothing done): deleteObject key.1"), "{log}");
    }

    #[test]
    fn a_switch_clicked_while_moving_gets_out_of_step() {
        // The edge case of doors.md: the variable flips on every click.
        let mut a = active(SWITCH, &[("Sw::On", "true")]);
        a.click("Condoor", NEAR, 2350.0, None);
        a.update();
        a.update();
        // Second click while it opens: the script asks to close (2), which
        // a moving door ignores, but the variable flips back to true.
        a.click("Condoor", NEAR, 2350.0, None);
        a.update();
        assert_eq!((state(&a), a.scripts.vars.get("Sw::On")), (DoorState::Opening, Some("true")));
        for _ in 0..20 {
            a.update();
        }
        // Opened, while the variable says "on" (as before the first click).
        assert_eq!((state(&a), a.scripts.vars.get("Sw::On")), (DoorState::Opened, Some("true")));
    }

    #[test]
    fn a_cue_event_reaches_the_script_two_updates_later() {
        let mut a = active(CUE, &[]);
        a.click("Condoor", NEAR, 2350.0, None);
        a.update();
        a.update();
        assert!(!a.take_log().join("\n").contains("echo cue"));
        a.update();
        let log = a.take_log().join("\n");
        assert!(log.contains("script door.cs Condoor Cue 1 0") && log.contains("echo cue 1 0"), "{log}");
    }

    #[test]
    fn unknown_objects_and_event_types_are_not_sent() {
        let mut a = active("sendEvent 0 nobody KQDoorEvent 1\nsendEvent 0 $1 KQFlyEvent\nsendEvent 0 $1\nsendEvent 0 Connor KQStatsChangeRequestEvent 0 0 0 0 25\n", &[]);
        a.click("Condoor", NEAR, 2350.0, None);
        assert_eq!(a.world.queue.len(), 1);
        a.update();
        let log = a.take_log().join("\n");
        assert!(log.contains("unknown object 'nobody'") && log.contains("'KQFlyEvent' unknown") && log.contains("usage: sendEvent"), "{log}");
        assert!(log.contains("KQStatsChangeRequestEvent 0 0 0 0 25 to Connor") && log.contains("stats are M5"), "{log}");
    }

    #[test]
    fn the_open_door_box_stands_out_of_the_doorway() {
        let mut a = active(PLAIN, &[]);
        let closed = a.world.doors[0].collider().unwrap();
        a.click("Condoor", NEAR, 2350.0, None);
        for _ in 0..20 {
            a.update();
        }
        let open = a.world.doors[0].collider().unwrap();
        // The middle of the doorway (own (0, -125, 900) when closed) is
        // inside the closed box and outside the open one.
        let middle = closed.placement.to_world([0.0, -125.0, 900.0]);
        let inside = |c: &BoxCollider| distance_to_box(middle, &c.placement, c.min, c.max) == 0.0;
        assert!(inside(&closed) && !inside(&open));
        // The pick ray finds the door when closed.
        assert_eq!(
            Active::pick(&active(PLAIN, &[]), [0.0, -5000.0, 900.0], [0.0, 1.0, 0.0]).map(|(n, t)| (n, t.round())),
            Some(("Condoor".to_string(), 4810.0))
        );
    }

    /// A made-up trap script in the style of `clifTrap.cs` (not a copy):
    /// on its first On it cues itself and switches itself off for Connor.
    const TRAP: &str = "if test $2 == Init\n set T::Init 1\nelse if test $2 == On\n set T::Who $3\n sendEvent 0 $1 KQCueEvent 10 0\n KQObject::collision $1 -4\nelse if test $2 == Off\n set T::Out $3\nelse if test $2 == Cue\n echo cue $3\nendif\nendif\nendif\nendif\n";

    fn with_trap(script: &'static str) -> Active {
        use crate::trap::TrapSettings;
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("trap.cs".to_string(), script.as_bytes().to_vec())]);
        a.add_traps([Trap::new(TrapSettings {
            name: "clifTrap".into(),
            position: [0.0; 3],
            min: [-500.0, -500.0, 0.0],
            max: [500.0, 500.0, 1000.0],
            script: Some("trap.cs".into()),
            mask: 4,
            trap_type: 0,
            inner_percent: None,
        })]);
        a.start();
        a
    }

    fn connor_at(a: &mut Active, x: f32) -> Vec<String> {
        let balls = crate::collide::BallStack::from_box([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]);
        let v = Visitor { name: "Connor", collision_type: CONNOR_TYPE, position: [x, 0.0, 0.0], min: [-200.0, -200.0, 0.0], max: [200.0, 200.0, 1300.0], balls: &balls };
        a.connor_moved(&v)
    }

    #[test]
    fn a_trap_gets_init_on_and_off_with_its_name_and_connors() {
        let mut a = with_trap("if test $2 == Init\n set T::Init $1\nelse if test $2 == On\n set T::Who $3\nelse if test $2 == Off\n set T::Out $3\nendif\nendif\nendif\n");
        assert_eq!(a.scripts.vars.get("T::Init"), Some("clifTrap"));
        assert!(connor_at(&mut a, -2000.0).is_empty());
        assert_eq!(connor_at(&mut a, -600.0), ["clifTrap On Connor"]);
        assert_eq!(a.scripts.vars.get("T::Who"), Some("Connor"));
        assert!(connor_at(&mut a, 0.0).is_empty());
        assert_eq!(connor_at(&mut a, 800.0), ["clifTrap Off Connor"]);
        assert_eq!(a.scripts.vars.get("T::Out"), Some("Connor"));
        let log = a.take_trap_log().join("\n");
        assert!(log.contains("script trap.cs clifTrap Init") && log.contains("trap clifTrap: On Connor at (-600, 0, 0)"), "{log}");
        assert!(a.take_log().is_empty(), "trap lines stay out of doors.log");
    }

    #[test]
    fn a_trap_cues_itself_and_switches_itself_off_for_connor() {
        let mut a = with_trap(TRAP);
        assert_eq!(connor_at(&mut a, 0.0), ["clifTrap On Connor"]);
        assert_eq!(a.world.traps[0].settings.mask, 0, "bit 4 removed (rule 16)");
        // The cue (delay 0) is handled in the next update's events.
        a.update();
        let log = a.take_trap_log().join("\n");
        assert!(log.contains("KQCueEvent 10 0 to trap clifTrap") && log.contains("echo cue 10"), "{log}");
        assert!(log.contains("mask 0x4 -> 0x0; inside now: Connor"), "{log}");
        // Leaving: Connor no longer counts, so no Off (stand-in).
        assert!(connor_at(&mut a, 5000.0).is_empty());
        assert_eq!(a.scripts.vars.get("T::Out"), None);
    }
}
