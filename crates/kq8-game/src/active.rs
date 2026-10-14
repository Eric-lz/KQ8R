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
//!
//! Items (M3 step 3e-2, `docs/behaviour/items.md`, `crate::item`): scripts
//! make them (`KQMonster::inventory`, `give`), ask for them
//! (`KQMonster::checkInventory`) and change them (`KQObject::setScript`,
//! `setDontLook`, `setHidden`, `deleteObject`); Connor picks a world item
//! up by clicking it in reach (rules 6-8). Item lines also go to
//! [`WorldState::item_log`] (`items.log`).
//!
//! Chests (M3 step 3e-4, `docs/behaviour/chests.md`, `crate::chest`):
//! clicked like doors (reach, "too far", "doesn't work"), but a click runs
//! the chest's own three steps instead of its script (rules 6-9); its
//! script gets `Init`, `EndLoop <movement>` and `Cue`.
//!
//! Scripted objects (M3 step 3e-5): clickable animated objects with a
//! script that are not doors, chests or items (`smlchst1`, `jar1`, the
//! mushroom baskets): a click runs `ConnorAction` as for doors (`chests.md`
//! rule 11, `doors.md` rules 12-14); `KQAnimPlayEvent` plays a movement
//! (`doors.md` rule 6); `Init` and `EndLoop` as chests get them (our
//! reading of `chests.md` rule 6, asked as Q64).

use std::collections::{BTreeMap, BTreeSet};

use kq8_data::world_script::Vars;

use crate::camera::ViewBlockers;
use crate::chest::{Chest, ChestState};
use crate::click::{distance_to_box, ray_box, CLICKABLE};
use crate::collide::{pair_collides, BoxCollider, Placement, CONNOR_MASK, CONNOR_TYPE};
use crate::door::{turn_degrees, Door};
use crate::events::{EventQueue, Pending};
use crate::item::{Items, Place};
use crate::movement::MovementPlayer;
use crate::script::{looks_like_number, number_value, ScriptHost, Scripts};
use crate::trap::{Trap, Visitor};

/// Event types we know (doors.md "Commands door scripts use"). Any other
/// is an error and is not sent.
pub const EVENT_TYPES: [&str; 4] = ["KQDoorEvent", "KQAnimPlayEvent", "KQCueEvent", "KQStatsChangeRequestEvent"];

/// What a script asked of Connor or the camera, for the caller to apply
/// (M3 step 3d-3). `at` is where in which script.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// `KQConner::setRun 0|1` (connor-movement.md rule 4).
    SetRun { run: bool, at: String },
    /// `KQCamera::setRange <min> <max>` (camera.md "Distance" rule 2),
    /// already checked: min < max.
    SetRange { min: f32, max: f32, at: String },
}

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

/// A chest and what clicking and collision need to know about it.
#[derive(Debug, Clone)]
pub struct ActiveChest {
    pub chest: Chest,
    pub script: Option<String>,
    /// Its `BOUNDS.KQ` box (own space); `None`: never clicked.
    pub bounds: Option<([f32; 3], [f32; 3])>,
    /// 0x8021 unless `colliderType` (chests.md rule 2); scripts change it.
    pub collision_type: u32,
    pub collision_mask: u32,
    /// `hidden=1` / `KQObject::setHidden`: not drawn, can't be clicked.
    pub hidden: bool,
    pub do_not_point_at: bool,
}

impl ActiveChest {
    pub fn collider(&self) -> Option<BoxCollider> {
        self.bounds.map(|(min, max)| BoxCollider { name: self.chest.name.clone(), min, max, placement: self.chest.placement() })
    }

    pub fn collides_with_connor(&self) -> bool {
        self.bounds.is_some() && pair_collides(CONNOR_TYPE, CONNOR_MASK, self.collision_type, self.collision_mask)
    }
}

/// A clickable animated object with a script that is not a door, chest or
/// item (3e-5).
#[derive(Debug, Clone)]
pub struct ActiveObject {
    pub player: MovementPlayer,
    pub script: Option<String>,
    pub bounds: Option<([f32; 3], [f32; 3])>,
    pub collision_type: u32,
    pub collision_mask: u32,
    pub accepts_inventory: Option<String>,
    pub accept_all_items: bool,
    pub too_far_away_action: i64,
    pub do_not_point_at: bool,
    pub hidden: bool,
}

impl ActiveObject {
    pub fn name(&self) -> &str {
        &self.player.name
    }

    pub fn collider(&self) -> Option<BoxCollider> {
        self.bounds.map(|(min, max)| BoxCollider { name: self.player.name.clone(), min, max, placement: self.player.placement() })
    }

    pub fn collides_with_connor(&self) -> bool {
        self.bounds.is_some() && pair_collides(CONNOR_TYPE, CONNOR_MASK, self.collision_type, self.collision_mask)
    }
}

/// What decides a click's script call (`doors.md` rules 12-14).
struct ClickRules<'a> {
    accepts_inventory: &'a Option<String>,
    accept_all_items: bool,
    too_far_away_action: i64,
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
    /// A world item went into Connor's inventory (items.md rule 7).
    PickedUp(String),
    /// A chest's own click steps ran (chests.md rules 7-9).
    Chest { before: ChestState, after: ChestState },
}

/// What the mouse ray picked (M3 step 3e-4b).
#[derive(Debug, Clone, PartialEq)]
pub enum Pick {
    /// The nearest clickable box, `t` units along the ray, in sight.
    Seen { name: String, t: f32 },
    /// The nearest clickable box, but something nearer blocks the line to
    /// it: nothing is clicked.
    Blocked { name: String, t: f32, by: String, at: f32 },
    /// No clickable box on the ray.
    Nothing,
}

/// Ground height at (x, y): the floor (`true`) or the surface (`false`).
pub type Heights = Box<dyn Fn(f32, f32, bool) -> Option<f32> + Send + Sync>;

/// Everything a script can touch, apart from the variables.
pub struct WorldState {
    pub doors: Vec<ActiveDoor>,
    pub chests: Vec<ActiveChest>,
    pub objects: Vec<ActiveObject>,
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
    /// Requests for Connor and the camera, taken by the caller.
    pub requests: Vec<Request>,
    /// Every item object and the item types (3e-2).
    pub items: Items,
    /// Lines about items for `items.log` (also in the other logs).
    pub item_log: Vec<String>,
    /// For z -1 / -2 of a new world item; `None`: kept as written.
    pub heights: Option<Heights>,
    /// Variables a command set, for the script runner.
    vars_out: Vec<(String, String)>,
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

    /// A line about items: `items.log` and the current log.
    fn item_note(&mut self, line: impl AsRef<str>) {
        self.item_log.push(format!("update {}: {}", self.updates, line.as_ref()));
        self.note(line);
    }

    pub fn object_index(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name().eq_ignore_ascii_case(name))
    }

    pub fn chest_index(&self, name: &str) -> Option<usize> {
        self.chests.iter().position(|c| c.chest.name.eq_ignore_ascii_case(name))
    }

    /// Makes a world item at `position` (z -1 / -2: floor / surface,
    /// `objects.md` Q17 rule 2) and returns its name.
    fn make_world_item(&mut self, kind: &str, quantity: i64, mut position: [f32; 3], angles: [f32; 3], at: &str) -> Result<String, String> {
        if position[2] == -1.0 || position[2] == -2.0 {
            let floor = position[2] == -1.0;
            match self.heights.as_ref().and_then(|h| h(position[0], position[1], floor)) {
                Some(z) => position[2] = z,
                None => self.note(format!("{at}: z {} at ({:.0}, {:.0}): no ground height known; kept", position[2], position[0], position[1])),
            }
        }
        let name = self.items.make(kind, quantity, Place::World { position, angles })?.name.clone();
        self.known.insert(name.to_ascii_lowercase());
        Ok(name)
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
            // traps.md rule 16: nothing sent now; they get Off when they leave.
            line.push_str(&format!("; inside now: {} (stay inside, Off when they leave)", inside.join(", ")));
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
        } else if let Some(i) = self.object_index(name) {
            let o = &mut self.objects[i];
            let old = o.collision_type;
            o.collision_type = change(old);
            (old, o.collision_type)
        } else if let Some(i) = self.chest_index(name) {
            let c = &mut self.chests[i];
            let old = c.collision_type;
            c.collision_type = change(old);
            (old, c.collision_type)
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

    /// `KQConner::setRun 0|1`: the run switch (connor-movement.md rule 4;
    /// any number but 0 counts as on: our reading).
    fn set_run(&mut self, words: &[String], at: &str) -> String {
        match words.get(1).filter(|w| looks_like_number(w)) {
            Some(v) => {
                let run = number_value(v) != 0.0;
                self.note(format!("{at}: setRun {v}: Connor's run switch {}", if run { "on" } else { "off" }));
                self.requests.push(Request::SetRun { run, at: at.to_string() });
                "0".into()
            }
            None => {
                self.note(format!("{at}: usage: KQConner::setRun 0|1: '{}'", words.join(" ")));
                "1".into()
            }
        }
    }

    /// `KQCamera::setRange <min> <max>`: refused unless min < max
    /// (camera.md "Distance" rule 2).
    fn set_range(&mut self, words: &[String], at: &str) -> String {
        let (Some(a), Some(b)) = (words.get(1), words.get(2)) else {
            self.note(format!("{at}: usage: KQCamera::setRange <min> <max>: '{}'", words.join(" ")));
            return "1".into();
        };
        if !looks_like_number(a) || !looks_like_number(b) {
            self.note(format!("{at}: setRange {a} {b}: not numbers; refused"));
            return "1".into();
        }
        let (min, max) = (number_value(a) as f32, number_value(b) as f32);
        if min >= max {
            self.note(format!("{at}: setRange {min} {max}: min not below max; refused (camera.md)"));
            return "1".into();
        }
        self.note(format!("{at}: setRange {min} {max}: camera range"));
        self.requests.push(Request::SetRange { min, max, at: at.to_string() });
        "0".into()
    }

    /// `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]`
    /// (items.md rule 9); `give <type> [qty]` is the same for Connor.
    /// Result 0 and `NewInvItem` = the new object's name, or 1 on an error.
    fn make_item(&mut self, words: &[String], at: &str) -> String {
        let m = match crate::item::read_make(words) {
            Ok(m) => m,
            Err(e) => {
                self.note(format!("{at}: {e}; nothing made"));
                return "1".into();
            }
        };
        let made = match m.position.filter(|_| m.in_world()) {
            Some(p) => self.make_world_item(&m.kind, m.quantity, p, m.angles, at),
            None => {
                if !self.known.contains(&m.who.to_ascii_lowercase()) {
                    self.note(format!("{at}: ERROR {}: unknown character '{}'; nothing made", words.join(" "), m.who));
                    return "1".into();
                }
                let made = self.items.make(&m.kind, m.quantity, Place::Carried(m.who.clone())).map(|o| o.name.clone());
                if let Ok(name) = &made {
                    self.known.insert(name.to_ascii_lowercase());
                }
                made
            }
        };
        match made {
            Ok(name) => {
                let line = format!("{at}: made {}; NewInvItem = {name}", self.items.describe(&name));
                self.vars_out.push(("NewInvItem".into(), name));
                self.item_note(line);
                "0".into()
            }
            Err(e) => {
                self.item_note(format!("{at}: ERROR {}: {e}; nothing made", words.join(" ")));
                "1".into()
            }
        }
    }

    /// `KQMonster::checkInventory <who> <type> [variable]` (items.md rule
    /// 10): 1 if found, 0 if not or on any error; the variable gets the
    /// found object's name.
    fn check_inventory(&mut self, words: &[String], at: &str) -> String {
        let (Some(who), Some(kind)) = (words.get(1), words.get(2)) else {
            self.note(format!("{at}: usage: KQMonster::checkInventory <who> <type> [variable]"));
            return "0".into();
        };
        if self.items.kind(kind).is_none() {
            self.item_note(format!("{at}: ERROR checkInventory {who} {kind}: no such item type; 0"));
            return "0".into();
        }
        match self.items.find(who, kind).map(|o| o.name.clone()) {
            Some(name) => {
                let into = words.get(3).map_or(String::new(), |v| format!(" (into {v})"));
                if let Some(var) = words.get(3) {
                    self.vars_out.push((var.clone(), name.clone()));
                }
                self.item_note(format!("{at}: checkInventory {who} {kind}: found {name}{into}: 1"));
                "1".into()
            }
            None => {
                self.item_note(format!("{at}: checkInventory {who} {kind}: none: 0"));
                "0".into()
            }
        }
    }

    /// `KQObject::setScript`, `setDontLook`, `setHidden` and `deleteObject`
    /// on an item object. `setHidden` and `deleteObject` on other objects
    /// stay stubs (as before 3e-2); the other two are not tracked there.
    fn item_change(&mut self, words: &[String], at: &str) -> String {
        let command = words[0].as_str();
        let Some(name) = words.get(1) else {
            self.note(format!("{at}: usage: {command} <object> ..."));
            return "1".into();
        };
        if let (Some(o), "KQObject::setHidden") = (self.object_index(name), command) {
            let hidden = words.get(2).is_some_and(|v| looks_like_number(v) && number_value(v) != 0.0);
            self.objects[o].hidden = hidden;
            self.note(format!("{at}: setHidden {name}: object hidden {hidden}"));
            return "0".into();
        }
        if let (Some(c), "KQObject::setHidden") = (self.chest_index(name), command) {
            // A chest (`wizchest.cs` on `Reveal`): not drawn, not clickable.
            let hidden = words.get(2).is_some_and(|v| looks_like_number(v) && number_value(v) != 0.0);
            self.chests[c].hidden = hidden;
            self.note(format!("{at}: setHidden {name}: chest hidden {hidden}"));
            return "0".into();
        }
        let Some(i) = self.items.index(name) else {
            if command == "deleteObject" || command == "KQObject::setHidden" {
                self.note(format!("{at}: stub (later milestone, nothing done): {}", words.join(" ")));
                return "0".into();
            }
            let why = if self.known.contains(&name.to_ascii_lowercase()) { "object kind not tracked yet" } else { "unknown object" };
            self.note(format!("{at}: {}: {why}; not changed", words.join(" ")));
            return "1".into();
        };
        let value = words.get(2).cloned().unwrap_or_default();
        let flag = looks_like_number(&value).then(|| number_value(&value) != 0.0);
        let o = &mut self.items.objects[i];
        let line = match (command, flag) {
            ("deleteObject", _) => {
                o.place = Place::Gone;
                format!("deleteObject {}: item gone", o.name)
            }
            ("KQObject::setScript", _) if !value.is_empty() => {
                // `none` removes it (chests.md rule 11, `Tavcoin.cs`).
                o.script = (!value.eq_ignore_ascii_case("none")).then(|| value.clone());
                format!("setScript {}: script {:?}", o.name, o.script)
            }
            ("KQObject::setDontLook", Some(on)) => {
                o.do_not_point_at = on;
                format!("setDontLook {}: doNotPointAt {on}", o.name)
            }
            ("KQObject::setHidden", Some(on)) => {
                o.hidden = on;
                format!("setHidden {}: hidden {on}", o.name)
            }
            _ => {
                self.note(format!("{at}: usage: '{}'", words.join(" ")));
                return "1".into();
            }
        };
        self.item_note(format!("{at}: {line}"));
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
            "KQConner::setRun" => Some(self.set_run(words, at)),
            "KQCamera::setRange" => Some(self.set_range(words, at)),
            "KQMonster::inventory" | "give" => Some(self.make_item(words, at)),
            "KQMonster::checkInventory" => Some(self.check_inventory(words, at)),
            "KQObject::setScript" | "KQObject::setDontLook" | "KQObject::setHidden" | "deleteObject" => Some(self.item_change(words, at)),
            _ => None,
        }
    }

    fn log(&mut self, line: String) {
        self.note(line);
    }

    fn take_vars(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.vars_out)
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
                chests: Vec::new(),
                objects: Vec::new(),
                traps: Vec::new(),
                clickables,
                known,
                sources,
                queue: EventQueue::default(),
                updates: 0,
                log: Vec::new(),
                trap_log: Vec::new(),
                in_trap: false,
                requests: Vec::new(),
                items: Items::default(),
                item_log: Vec::new(),
                heights: None,
                vars_out: Vec::new(),
            },
        }
    }

    /// Adds the world's chests (before [`Active::start`]).
    pub fn add_chests(&mut self, chests: impl IntoIterator<Item = ActiveChest>) {
        for c in chests {
            self.world.known.insert(c.chest.name.to_ascii_lowercase());
            self.world.chests.push(c);
        }
    }

    /// Adds the scripted objects (before [`Active::start`]).
    pub fn add_objects(&mut self, objects: impl IntoIterator<Item = ActiveObject>) {
        for o in objects {
            self.world.known.insert(o.name().to_ascii_lowercase());
            self.world.objects.push(o);
        }
    }

    /// Runs object `i`'s script with `args` (`$1` is its name).
    fn call_object(&mut self, i: usize, args: &[&str]) {
        let o = &self.world.objects[i];
        let name = o.name().to_string();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match o.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
            }
            None => self.world.note(format!("object {name} has no script; {} not called", all.join(" "))),
        }
    }

    /// Runs chest `i`'s script with `args` (`$1` is the chest's name).
    fn call_chest(&mut self, i: usize, args: &[&str]) {
        let c = &self.world.chests[i];
        let name = c.chest.name.clone();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match c.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
            }
            None => self.world.note(format!("chest {name} has no script; {} not called", all.join(" "))),
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
        // chests.md rule 6 (after the doors: our order).
        for i in 0..self.world.chests.len() {
            self.call_chest(i, &["Init"]);
        }
        // Scripted objects (Q64 stand-in; after the chests: our order).
        for i in 0..self.world.objects.len() {
            self.call_object(i, &["Init"]);
        }
        // traps.md rule 13 (after the doors, chests, objects: our order).
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
        // Chests (chests.md rule 4): as doors; hidden ones can't be clicked.
        let chests = self.world.chests.iter().filter(|c| c.collision_type & CLICKABLE != 0 && !c.hidden).filter_map(|c| {
            let (min, max) = c.bounds?;
            Some((c.chest.name.clone(), ray_box(origin, dir, &c.chest.placement(), min, max)?))
        });
        let objects = self.world.objects.iter().filter(|o| o.collision_type & CLICKABLE != 0 && !o.hidden).filter_map(|o| {
            let (min, max) = o.bounds?;
            Some((o.name().to_string(), ray_box(origin, dir, &o.player.placement(), min, max)?))
        });
        let doors = doors.chain(chests).chain(objects);
        // World items (items.md rule 6): by their type's box; hidden ones
        // can't be clicked.
        let items = &self.world.items;
        let items = items.objects.iter().filter(|o| !o.hidden).filter_map(|o| {
            let (min, max) = items.click_box(o)?;
            Some((o.name.clone(), ray_box(origin, dir, &o.placement()?, min, max)?))
        });
        doors.chain(others).chain(items).min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The middle of a clickable object's box in the world (for the
    /// viewer's `--aim` test option).
    pub fn target_point(&self, name: &str) -> Option<[f32; 3]> {
        let mid = |p: &Placement, (min, max): ([f32; 3], [f32; 3])| p.to_world(std::array::from_fn(|k| 0.5 * (min[k] + max[k])));
        let w = &self.world;
        if let Some(i) = w.door_index(name) {
            return w.doors[i].bounds.map(|b| mid(&w.doors[i].door.placement(), b));
        }
        if let Some(i) = w.chest_index(name) {
            return w.chests[i].bounds.map(|b| mid(&w.chests[i].chest.placement(), b));
        }
        if let Some(i) = w.object_index(name) {
            return w.objects[i].bounds.map(|b| mid(&w.objects[i].player.placement(), b));
        }
        if let Some(o) = w.items.get(name) {
            return Some(mid(&o.placement()?, w.items.click_box(o)?));
        }
        w.clickables.iter().find(|c| c.name.eq_ignore_ascii_case(name)).map(|c| mid(&c.placement, (c.min, c.max)))
    }

    /// [`Active::pick`], but only if the line from `origin` (the camera)
    /// to the picked box is not blocked by `view` (M3 step 3e-4b). Stand-in
    /// for Q63 (the user's check in the original: no clicks through walls
    /// or the ground; clicks from behind a wall work when the camera sees
    /// the object): the camera's view obstacles (`camera.md` "Obstacles")
    /// block. The line is tested up to 1 unit short of the box, so the
    /// picked object never blocks itself. `dir` has length 1.
    pub fn pick_seen(&self, origin: [f32; 3], dir: [f32; 3], view: &dyn ViewBlockers) -> Pick {
        let Some((name, t)) = self.pick(origin, dir) else { return Pick::Nothing };
        let short = (t - 1.0).max(0.0);
        let to: [f32; 3] = std::array::from_fn(|k| origin[k] + dir[k] * short);
        match view.first_block(origin, to) {
            Some((p, by)) => {
                let at = (0..3).map(|k| (p[k] - origin[k]).powi(2)).sum::<f32>().sqrt();
                Pick::Blocked { name, t, by, at }
            }
            None => Pick::Seen { name, t },
        }
    }

    /// Connor clicks object `name` (rules 11–17). `feet`: his position;
    /// `attack_dist`: his `attackDist` (reach = twice that); `item`: what
    /// he holds.
    pub fn click(&mut self, name: &str, feet: [f32; 3], attack_dist: f32, item: Option<&Item>) -> ClickResult {
        let reach = 2.0 * attack_dist;
        let feet_text = format!("({:.0}, {:.0}, {:.0})", feet[0], feet[1], feet[2]);
        if let Some(i) = self.world.items.index(name) {
            return self.click_item(i, feet, reach, item);
        }
        if let Some(i) = self.world.chest_index(name) {
            return self.click_chest(i, feet, reach, item);
        }
        if let Some(i) = self.world.object_index(name) {
            return self.click_object(i, feet, reach, item);
        }
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
        let rules = ClickRules { accepts_inventory: &d.accepts_inventory, accept_all_items: d.accept_all_items, too_far_away_action: d.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
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

    /// The script arguments of a click (`doors.md` rules 12-14), or what
    /// happened instead (logged).
    fn script_args(&mut self, head: &str, in_reach: bool, item: Option<&Item>, rules: &ClickRules) -> Result<Vec<String>, ClickResult> {
        match (in_reach, item) {
            (true, None) => Ok(vec!["ConnorAction".into(), "do".into(), "do".into()]),
            (true, Some(it)) => {
                let accepted = rules.accept_all_items || rules.accepts_inventory.as_deref() == Some(it.kind.as_str());
                if !accepted {
                    self.world.note(format!(
                        "{head}; holds {} ({}): not accepted (acceptsInventory {:?}): Connor's \"that doesn't work\" remark (line not traced, M4); script not run",
                        it.object, it.kind, rules.accepts_inventory
                    ));
                    return Err(ClickResult::Refused);
                }
                Ok(vec!["ConnorAction".into(), it.object.clone(), it.kind.clone()])
            }
            (false, _) if rules.too_far_away_action == 0 => {
                self.world.note(format!("{head}; tooFarAwayAction 0: Connor's \"too far\" remark (line not traced, M4); nothing sent"));
                Err(ClickResult::TooFar)
            }
            (false, _) => Ok(vec!["ConnorAction".into(), "TooFar".into(), "TooFar".into()]),
        }
    }

    /// Connor clicks scripted object `i` (3e-5): as a door (`doors.md`
    /// rules 11-16, `chests.md` rule 11).
    fn click_object(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let o = self.world.objects[i].clone();
        let name = o.name().to_string();
        if o.collision_type & CLICKABLE == 0 || o.hidden {
            self.world.note(format!("click {name}: not clickable now (type {:#x}, hidden {}); nothing", o.collision_type, o.hidden));
            return ClickResult::Nothing;
        }
        let Some((min, max)) = o.bounds else {
            self.world.note(format!("click {name}: no box in BOUNDS.KQ; can't be clicked"));
            return ClickResult::Nothing;
        };
        let distance = distance_to_box(feet, &o.player.placement(), min, max);
        let in_reach = distance < reach;
        let head = format!(
            "click {name} (scripted object): Connor at ({:.0}, {:.0}, {:.0}), distance to its box {distance:.1}, reach {reach:.0}: {}; movement {}",
            feet[0],
            feet[1],
            feet[2],
            if in_reach { "IN REACH" } else { "OUT OF REACH" },
            o.player.movement_name()
        );
        let rules = ClickRules { accepts_inventory: &o.accepts_inventory, accept_all_items: o.accept_all_items, too_far_away_action: o.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
        };
        self.world.note(format!("{head}; script arguments: {name} {}", args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.call_object(i, &refs);
        if !o.do_not_point_at {
            self.world.note(format!("Connor would turn to face {name} (doors.md rule 16; not done: his states not traced)"));
        }
        let mut full = vec![name];
        full.extend(args);
        ClickResult::Script(full)
    }

    /// Connor clicks item object `i` (items.md rules 6-8).
    fn click_item(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let o = self.world.items.objects[i].clone();
        let (Some(placement), false) = (o.placement(), o.hidden) else {
            self.world.item_note(format!("click {}: not lying in the world (or hidden); nothing", o.name));
            return ClickResult::Nothing;
        };
        let Some((min, max)) = self.world.items.click_box(&o) else {
            self.world.item_note(format!("click {}: its type's shape has no box in BOUNDS.KQ; can't be clicked", o.name));
            return ClickResult::Nothing;
        };
        let distance = distance_to_box(feet, &placement, min, max);
        let head = format!(
            "click {} ({} x{}): Connor at ({:.0}, {:.0}, {:.0}), distance to its box {distance:.1}, reach {reach:.0}",
            o.name, o.kind, o.quantity, feet[0], feet[1], feet[2]
        );
        if distance >= reach {
            self.world.item_note(format!("{head}: OUT OF REACH: Connor's \"too far\" remark (line not traced, M4); nothing"));
            return ClickResult::TooFar;
        }
        if let Some(it) = item {
            self.world.item_note(format!("{head}: holds {} ({}): items accept nothing: Connor's \"that doesn't work\" remark (M4)", it.object, it.kind));
            return ClickResult::Refused;
        }
        let before = self.world.items.count("Connor", &o.kind);
        self.world.items.objects[i].place = Place::Carried("Connor".into());
        let after = self.world.items.count("Connor", &o.kind);
        let sound = self.world.items.kind(&o.kind).and_then(|t| t.pickup_sound.clone());
        self.world.item_note(format!(
            "{head}: IN REACH: picked up; Connor's {} {before} -> {after}; pickup sound {} (no audio yet); gone from the world",
            o.kind,
            sound.as_deref().unwrap_or("none")
        ));
        // Rule 7: then its script, as a normal click.
        if let Some(script) = o.script.clone() {
            let args: Vec<String> = [o.name.as_str(), "ConnorAction", "do", "do"].map(String::from).to_vec();
            self.world.item_note(format!("script {script} {}", args.join(" ")));
            self.scripts.call(&script, &args, &mut self.world);
        }
        ClickResult::PickedUp(o.name)
    }

    /// Connor clicks chest `i` (chests.md rules 4-10).
    fn click_chest(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let c = self.world.chests[i].clone();
        let name = c.chest.name.clone();
        if c.collision_type & CLICKABLE == 0 || c.hidden {
            self.world.note(format!("click {name}: not clickable now (type {:#x}, hidden {}); nothing", c.collision_type, c.hidden));
            return ClickResult::Nothing;
        }
        let Some((min, max)) = c.bounds else {
            self.world.note(format!("click {name}: no box in BOUNDS.KQ; can't be clicked"));
            return ClickResult::Nothing;
        };
        let distance = distance_to_box(feet, &c.chest.placement(), min, max);
        let before = c.chest.state;
        let head = format!(
            "click {name} (chest): Connor at ({:.0}, {:.0}, {:.0}), distance to its box {distance:.1}, reach {reach:.0}",
            feet[0], feet[1], feet[2]
        );
        if distance >= reach {
            self.world.note(format!("{head}: OUT OF REACH: Connor's \"too far\" remark (line not traced, M4); nothing"));
            return ClickResult::TooFar;
        }
        if let Some(it) = item {
            // Rule 5: chests accept no item.
            self.world.note(format!("{head}: holds {} ({}): chests accept nothing: Connor's \"that doesn't work\" remark (M4)", it.object, it.kind));
            return ClickResult::Refused;
        }
        let at = format!("chest {name}");
        match before {
            ChestState::Closed => {
                // Rule 7: open plays; the item appears at invLoc / invDir.
                let s = c.chest.settings.clone();
                let opened = self.world.chests[i].chest.play("open");
                let made = s.item.as_ref().map(|kind| self.world.make_world_item(kind, s.quantity, s.item_position, s.item_angles, &at));
                let (after, what) = match made {
                    Some(Ok(item_name)) => {
                        let line = format!("{at}: made {}", self.world.items.describe(&item_name));
                        self.world.item_note(line);
                        self.world.chests[i].chest.remembered = Some(item_name.clone());
                        (ChestState::Open, format!("item {item_name} put out, remembered"))
                    }
                    Some(Err(e)) => (ChestState::Empty, format!("ERROR item {:?}: {e}; counts as holding nothing (our choice)", s.item)),
                    None => (ChestState::Empty, "holds nothing".to_string()),
                };
                self.world.chests[i].chest.state = after;
                let play = match opened {
                    Ok(()) => "plays 'open'".to_string(),
                    Err(e) => e,
                };
                self.world.note(format!("{head}: IN REACH: 1st click (closed): {play}; {what}; state closed -> {}", after.name()));
            }
            ChestState::Open => {
                // Rule 8: the remembered item goes to Connor, then Take.
                let remembered = self.world.chests[i].chest.remembered.take();
                let what = match remembered.as_deref().and_then(|n| self.world.items.index(n)) {
                    Some(k) => {
                        let o = self.world.items.objects[k].clone();
                        if o.carried_by("Connor") {
                            // The note's stand-in ("Edge cases").
                            format!("item {} already in Connor's inventory: only the state changes", o.name)
                        } else if o.place == Place::Gone {
                            format!("item {} no longer exists", o.name)
                        } else {
                            let count = self.world.items.count("Connor", &o.kind);
                            self.world.items.objects[k].place = Place::Carried("Connor".into());
                            let line = format!(
                                "{at}: 2nd click: {} moved into Connor's inventory ({} {count} -> {}); pickup sound not stated for this case (Q61), none",
                                o.name,
                                o.kind,
                                self.world.items.count("Connor", &o.kind)
                            );
                            self.world.item_note(line);
                            format!("item {} moved into Connor's inventory", o.name)
                        }
                    }
                    None => format!("remembered item {remembered:?} no longer exists"),
                };
                let take = match c.chest.settings.take.filter(|r| r.said()) {
                    Some(r) => format!("; Take remark {:?}{} (line M4)", r.0, self.turn_note(&c)),
                    None => "; no Take remark".into(),
                };
                self.world.chests[i].chest.state = ChestState::Empty;
                self.world.note(format!("{head}: IN REACH: 2nd click (open): {what}{take}; state open -> empty"));
            }
            ChestState::Empty => {
                let empty = match c.chest.settings.empty.filter(|r| r.said()) {
                    Some(r) => format!("Empty remark {:?}{} (line M4)", r.0, self.turn_note(&c)),
                    None => "no Empty remark: nothing".into(),
                };
                self.world.note(format!("{head}: IN REACH: click (empty): {empty}"));
            }
        }
        ClickResult::Chest { before, after: self.world.chests[i].chest.state }
    }

    /// Before a chest's remark Connor turns to face it, unless
    /// `doNotPointAt` (chests.md rules 8-10); not done yet.
    fn turn_note(&self, c: &ActiveChest) -> &'static str {
        if c.do_not_point_at { "" } else { "; Connor would turn to face the chest (not done: his states not traced)" }
    }

    /// Lines for `items.log` since the last call.
    pub fn take_item_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.world.item_log)
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
        for i in 0..self.world.chests.len() {
            let a = self.world.chests[i].chest.advance(dt);
            let c = &self.world.chests[i].chest;
            let name = c.name.clone();
            if a.from != a.to {
                let line = format!("chest {name}: movement {} phase {:.4} -> {:.4} frame {}", c.movement_name(), a.from, a.to, c.frame());
                self.world.note(line);
            }
            for cue in &a.cues {
                self.world.note(format!("chest {name}: cue '{cue}' fires (no audio yet)"));
            }
            if let Some(m) = a.finished {
                // chests.md rule 6: EndLoop with the movement's name.
                self.world.note(format!("chest {name}: movement '{m}' finished; stays on its last frame; calls its script with EndLoop {m}"));
                self.call_chest(i, &["EndLoop", &m]);
            }
        }
        for i in 0..self.world.objects.len() {
            let a = self.world.objects[i].player.advance(dt);
            let p = &self.world.objects[i].player;
            let name = p.name.clone();
            if a.from != a.to {
                let line = format!("object {name}: movement {} phase {:.4} -> {:.4} frame {}", p.movement_name(), a.from, a.to, p.frame());
                self.world.note(line);
            }
            for cue in &a.cues {
                self.world.note(format!("object {name}: cue '{cue}' fires (cue actions not done)"));
            }
            if let Some(m) = a.finished {
                // Q64 stand-in: EndLoop as chests get it (chests.md rule 6).
                self.world.note(format!("object {name}: movement '{m}' finished; stays on its last frame; calls its script with EndLoop {m}"));
                self.call_object(i, &["EndLoop", &m]);
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
        if let Some(i) = self.world.object_index(&e.target) {
            match e.kind.as_str() {
                "KQAnimPlayEvent" => {
                    // doors.md rule 6, as for doors.
                    let line = match e.data.first() {
                        Some(m) => match self.world.objects[i].player.play(m) {
                            Ok(()) => format!("KQAnimPlayEvent {m} to object {}{from}: starts '{m}'", e.target),
                            Err(err) => format!("KQAnimPlayEvent {m} to object {}{from}: {err}; nothing", e.target),
                        },
                        None => format!("KQAnimPlayEvent to object {}{from}: no movement named; nothing", e.target),
                    };
                    self.world.note(line);
                }
                "KQCueEvent" => {
                    let n = |k: usize| e.data.get(k).cloned().unwrap_or_else(|| "0".into());
                    let mut args = vec!["Cue".to_string(), n(0), n(1)];
                    if e.data.get(2).is_some_and(|v| number_value(v) != 0.0) {
                        args.push(e.data[2].clone());
                    }
                    self.world.note(format!("KQCueEvent {} to object {}{from}", e.data.join(" "), e.target));
                    let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                    self.call_object(i, &refs);
                }
                other => self.world.note(format!("event {other} {} to object {}{from}: not handled for scripted objects", e.data.join(" "), e.target)),
            }
            return;
        }
        if let Some(i) = self.world.chest_index(&e.target) {
            if e.kind == "KQCueEvent" {
                let n = |k: usize| e.data.get(k).cloned().unwrap_or_else(|| "0".into());
                let mut args = vec!["Cue".to_string(), n(0), n(1)];
                if e.data.get(2).is_some_and(|v| number_value(v) != 0.0) {
                    args.push(e.data[2].clone());
                }
                self.world.note(format!("KQCueEvent {} to chest {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_chest(i, &refs);
            } else {
                self.world.note(format!("event {} {} to chest {}{from}: not handled for chests (not in chests.md)", e.kind, e.data.join(" "), e.target));
            }
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
        let p = [x, 0.0, 0.0];
        let v = Visitor { name: "Connor", collision_type: CONNOR_TYPE, from: p, position: p, min: [-200.0, -200.0, 0.0], max: [200.0, 200.0, 1300.0], balls: &balls };
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
        // Leaving: he was inside, so he still gets the Off (rule 16) ...
        assert_eq!(connor_at(&mut a, 5000.0), ["clifTrap Off Connor"]);
        assert_eq!(a.scripts.vars.get("T::Out"), Some("Connor"));
        // ... but can't come in again.
        assert!(connor_at(&mut a, 0.0).is_empty());
    }

    /// Made-up scripts in the style of `WorldInv.cs` and `Tavcoin.cs` (not
    /// copies).
    const MAKE: &str = "KQMonster::inventory world Crystal 3 1000 0 500 0 0 1.57\nKQObject::setScript $NewInvItem coin.cs\nset First $NewInvItem\nKQMonster::inventory\tworld\tcrystal\t1\t2000\t0\t-1\t0\t0\t0\ngive silverCoins 5\nset Gave $NewInvItem\n";
    const COIN: &str = "if test $2 == ConnorAction\n KQMonster::speak Connor Connor 1000 15 14 108 1 0\n KQObject::setScript $1 none\n set Coin::Said $1\nendif\n";

    fn with_items(script: &'static str) -> Active {
        let sources = [("s.cs".to_string(), script.as_bytes().to_vec()), ("coin.cs".to_string(), COIN.as_bytes().to_vec())];
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], sources);
        a.world.items = crate::item::tests::items();
        // Floor 700 below the surface everywhere (for z -1 / -2).
        a.world.heights = Some(Box::new(|_, _, floor| Some(if floor { 100.0 } else { 800.0 })));
        a.scripts.call("s.cs", &[], &mut a.world);
        a
    }

    #[test]
    fn scripts_make_items_in_the_world_and_in_inventories() {
        let a = with_items(MAKE);
        let v = |k: &str| a.scripts.vars.get(k).map(str::to_string);
        assert_eq!((v("First"), v("Gave"), v("NewInvItem")), (Some("Crystal.1".into()), Some("SilverCoins.3".into()), Some("SilverCoins.3".into())));
        let it = &a.world.items;
        let c = it.get("crystal.1").unwrap();
        assert_eq!((c.quantity, c.place.clone(), c.script.as_deref()), (3, Place::World { position: [1000.0, 0.0, 500.0], angles: [0.0, 0.0, 1.57] }, Some("coin.cs")));
        // z -1 is the floor height there.
        assert_eq!(it.get("Crystal.2").unwrap().place, Place::World { position: [2000.0, 0.0, 100.0], angles: [0.0; 3] });
        assert_eq!(it.count("Connor", "INVITEM_SilverCoins"), 5);
        assert_eq!(a.scripts.vars.get(crate::script::RETURN), Some("0"));
        assert!(a.world.known.contains("crystal.1"), "new items are known objects");
        assert!(a.world.item_log.iter().any(|l| l.contains("made Crystal.1: INVITEM_Crystal x3 in the world at (1000, 0, 500)")), "{:?}", a.world.item_log);
    }

    #[test]
    fn errors_make_nothing() {
        let a = with_items("KQMonster::inventory world ladybell 1 0 0 0\nset R1 $Console::Return\nKQMonster::inventory world Crystal 1\nset R2 $Console::Return\nKQMonster::inventory nobody Crystal\nset R3 $Console::Return\nKQMonster::inventory world Crystal 1 5 6\nset R4 $Console::Return\n");
        for r in ["R1", "R2", "R3", "R4"] {
            assert_eq!(a.scripts.vars.get(r), Some("1"), "{r}");
        }
        assert!(a.world.items.objects.is_empty());
        assert_eq!(a.scripts.vars.get("NewInvItem"), None);
    }

    #[test]
    fn check_inventory_answers_1_for_found() {
        let a = with_items("give Crystal 2\nif KQMonster::checkInventory Connor crystal Found\n set A no\nelse\n set A yes\nendif\nKQMonster::checkInventory Connor SilverCoins\nset B $Console::Return\nKQMonster::checkInventory world Crystal\nset C $Console::Return\nKQMonster::checkInventory Connor NoSuchThing\nset D $Console::Return\n");
        let v = |k: &str| a.scripts.vars.get(k);
        // Rule 10: 1 = found, so an `if` on it takes the else branch.
        assert_eq!((v("A"), v("Found")), (Some("yes"), Some("Crystal.1")));
        assert_eq!((v("B"), v("C"), v("D")), (Some("0"), Some("0"), Some("0")));
    }

    #[test]
    fn clicking_an_item_in_reach_picks_it_up_and_runs_its_script_once() {
        let mut a = with_items(MAKE);
        // Crystal.1 at (1000, 0, 500), box x -60..70 turned 90 degrees.
        let near = [1000.0, -3000.0, 500.0];
        assert_eq!(a.click("Crystal.1", [1000.0, -9000.0, 500.0], 2350.0, None), ClickResult::TooFar);
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("Crystal.1", near, 2350.0, Some(&bread)), ClickResult::Refused);
        assert!(a.world.items.get("Crystal.1").unwrap().in_world());
        // The pick ray finds it.
        assert_eq!(a.pick([1000.0, -5000.0, 600.0], [0.0, 1.0, 0.0]).map(|(n, _)| n), Some("Crystal.1".to_string()));
        assert_eq!(a.click("Crystal.1", near, 2350.0, None), ClickResult::PickedUp("Crystal.1".into()));
        let c = a.world.items.get("Crystal.1").unwrap();
        assert!(c.carried_by("Connor") && c.script.is_none(), "script removed by itself (setScript none)");
        assert_eq!(a.scripts.vars.get("Coin::Said"), Some("Crystal.1"));
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 3);
        let log = a.take_item_log().join("\n");
        assert!(log.contains("Connor's INVITEM_Crystal 0 -> 3; pickup sound energy.aud"), "{log}");
        assert!(log.contains("script coin.cs Crystal.1 ConnorAction do do"), "{log}");
        // Gone from the world: not picked, a second click does nothing.
        assert_eq!(a.pick([1000.0, -5000.0, 600.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("Crystal.1", near, 2350.0, None), ClickResult::Nothing);
    }

    #[test]
    fn hidden_and_deleted_items_cant_be_clicked() {
        let mut a = with_items("KQMonster::inventory world Crystal 1 0 0 0\nKQObject::setHidden $NewInvItem 1\nKQMonster::inventory world Crystal 1 0 5000 0\ndeleteObject $NewInvItem\nKQObject::setDontLook Crystal.1 0\nKQObject::setHidden smlchst3 1\n");
        let it = &a.world.items;
        assert!(it.get("Crystal.1").unwrap().hidden && !it.get("Crystal.1").unwrap().do_not_point_at);
        assert_eq!(it.get("Crystal.2").unwrap().place, Place::Gone);
        assert_eq!(a.pick([0.0, -5000.0, 100.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("Crystal.1", [0.0, -500.0, 0.0], 2350.0, None), ClickResult::Nothing);
        // setHidden on another object is still a stub.
        assert!(a.take_log().join("\n").contains("stub (later milestone, nothing done): KQObject::setHidden smlchst3 1"));
    }

    /// A made-up chest script in the style of Swamp's `chest.cs` (not a
    /// copy): it records every call.
    const CHEST: &str = "set C::Last $2 $3\nif test $2 == EndLoop\n set C::EndLoop $3\nendif\nif test $2 == ConnorAction\n set C::Clicked 1\nendif\n";

    fn with_chest(item: Option<&str>) -> Active {
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("chest.cs".to_string(), CHEST.as_bytes().to_vec())]);
        a.world.items = crate::item::tests::items();
        a.add_chests([ActiveChest {
            chest: crate::chest::tests::test_chest(item),
            script: Some("chest.cs".into()),
            // chest2.kqs's box (BOUNDS.KQ).
            bounds: Some(([-480.0, -310.0, 0.0], [470.0, 290.0, 2060.0])),
            collision_type: crate::chest::CHEST_TYPE,
            collision_mask: 0,
            hidden: false,
            do_not_point_at: false,
        }]);
        a.start();
        a
    }

    /// Connor 1,000 in front of the chest.
    const BY_CHEST: [f32; 3] = [0.0, -1310.0, 0.0];

    fn chest_state(a: &Active) -> ChestState {
        a.world.chests[0].chest.state
    }

    #[test]
    fn a_chest_opens_then_gives_its_item_then_is_empty() {
        let mut a = with_chest(Some("INVITEM_Crystal"));
        assert_eq!(a.scripts.vars.get("C::Last"), Some("Init"), "Init at the start");
        // 1st click: open plays, the item appears at invLoc.
        let r = a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(r, ClickResult::Chest { before: ChestState::Closed, after: ChestState::Open });
        let crystal = a.world.items.get("Crystal.1").unwrap().clone();
        assert_eq!((crystal.quantity, crystal.place), (3, Place::World { position: [0.0, 0.0, 301.0], angles: [0.0, 0.0, -2.5] }));
        assert_eq!(a.world.chests[0].chest.remembered.as_deref(), Some("Crystal.1"));
        assert_eq!(a.scripts.vars.get("C::Clicked"), None, "rule 6: the script is not run on a click");
        // The crystal lies inside the chest's box: the ray picks the chest.
        assert_eq!(a.pick([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]).map(|(n, _)| n), Some("smlchst3".to_string()));
        // `open` ends after 1 s (started in update 1, moves in 2..16): EndLoop open.
        for _ in 0..15 {
            a.update();
        }
        assert_eq!(a.scripts.vars.get("C::EndLoop"), None);
        a.update();
        assert_eq!(a.scripts.vars.get("C::EndLoop"), Some("open"));
        let log = a.take_log().join("\n");
        assert!(log.contains("cue 'playsound chstopen' fires") && log.contains("script chest.cs smlchst3 EndLoop open"), "{log}");
        // 2nd click: into Connor's inventory.
        let r = a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(r, ClickResult::Chest { before: ChestState::Open, after: ChestState::Empty });
        assert!(a.world.items.get("Crystal.1").unwrap().carried_by("Connor"));
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 3);
        assert_eq!(a.world.chests[0].chest.remembered, None);
        // 3rd click: the Empty remark, nothing else.
        let r = a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(r, ClickResult::Chest { before: ChestState::Empty, after: ChestState::Empty });
        let log = a.take_log().join("\n");
        assert!(log.contains("Empty remark [0, 109, 14, 33, 1, 1]"), "{log}");
        assert_eq!(a.world.items.objects.len(), 1, "no second item");
    }

    #[test]
    fn a_chest_without_an_item_goes_straight_to_empty() {
        let mut a = with_chest(None);
        a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(chest_state(&a), ChestState::Empty);
        assert!(a.world.items.objects.is_empty());
        assert_eq!(a.world.chests[0].chest.movement_name(), "open", "open still plays");
    }

    #[test]
    fn an_item_already_picked_up_only_changes_the_state() {
        let mut a = with_chest(Some("Crystal"));
        a.click("smlchst3", BY_CHEST, 2350.0, None);
        // Picked up directly (possible when an item lies outside the box).
        assert_eq!(a.click("Crystal.1", BY_CHEST, 2350.0, None), ClickResult::PickedUp("Crystal.1".into()));
        a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!((chest_state(&a), a.world.items.count("Connor", "INVITEM_Crystal")), (ChestState::Empty, 3));
        assert!(a.take_log().join("\n").contains("already in Connor's inventory: only the state changes"));
    }

    #[test]
    fn far_away_with_an_item_in_hand_or_unclickable_nothing_happens() {
        let mut a = with_chest(Some("Crystal"));
        assert_eq!(a.click("smlchst3", [0.0, -6000.0, 0.0], 2350.0, None), ClickResult::TooFar);
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("smlchst3", BY_CHEST, 2350.0, Some(&bread)), ClickResult::Refused);
        assert_eq!(chest_state(&a), ChestState::Closed);
        // `wizchest.cs`-style: hidden and not clickable until revealed.
        a.scripts.call("chest.cs", &[], &mut a.world);
        a.world.chests[0].collision_type = 1;
        assert_eq!(a.click("smlchst3", BY_CHEST, 2350.0, None), ClickResult::Nothing);
        assert_eq!(a.pick([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]), None);
        let reveal = "KQObject::setHidden smlchst3 0\nKQObject::collisionType smlchst3 32\n";
        let mut b = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("r.cs".to_string(), reveal.as_bytes().to_vec())]);
        b.add_chests(a.world.chests.drain(..));
        b.world.chests[0].hidden = true;
        b.scripts.call("r.cs", &[], &mut b.world);
        assert_eq!((b.world.chests[0].hidden, b.world.chests[0].collision_type), (false, 0x21));
        assert_eq!(b.click("smlchst3", BY_CHEST, 2350.0, None), ClickResult::Chest { before: ChestState::Closed, after: ChestState::Empty }, "no item types loaded here: holds nothing");
    }

    /// A wall: the plane y = `y` blocks every line crossing it.
    struct Wall {
        y: f32,
    }

    impl ViewBlockers for Wall {
        fn first_block(&self, from: [f32; 3], to: [f32; 3]) -> Option<([f32; 3], String)> {
            let (a, b) = (from[1] - self.y, to[1] - self.y);
            if a * b > 0.0 || a == b {
                return None;
            }
            let t = a / (a - b);
            Some((std::array::from_fn(|k| from[k] + (to[k] - from[k]) * t), "wall".into()))
        }
    }

    #[test]
    fn a_click_needs_the_cameras_line_of_sight() {
        let a = with_chest(Some("Crystal"));
        // The chest's box starts at y -310; the camera looks along +y.
        let (from, dir) = ([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]);
        assert_eq!(a.pick_seen(from, dir, &Wall { y: -3000.0 }), Pick::Blocked { name: "smlchst3".into(), t: 4690.0, by: "wall".into(), at: 2000.0 });
        // A wall behind the camera, or behind the chest: in sight.
        assert_eq!(a.pick_seen(from, dir, &Wall { y: -6000.0 }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        assert_eq!(a.pick_seen(from, dir, &Wall { y: 0.0 }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        // A wall exactly at the box's face is the box itself: in sight.
        assert_eq!(a.pick_seen(from, dir, &Wall { y: -310.0 }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        assert_eq!(a.pick_seen(from, [0.0, -1.0, 0.0], &Wall { y: -3000.0 }), Pick::Nothing);
    }

    /// Made-up scripts in the style of `basket1.cs` and `smlchst1.cs` (not
    /// copies).
    const BASKET: &str = "if test $2 == ConnorAction\n if test $B::full == true\n  set B::full false\n  give crystal 5\n  sendEvent 0 $1 KQAnimPlayEvent open\n endif\nendif\nif test $2 == EndLoop\n set B::ended $3\nendif\nif test $2 == Init\n set B::init $1\nendif\n";
    const SMALL_CHEST: &str = "if test $2 == ConnorAction\n if test $S::full == true\n  set S::full false\n  KQMonster::inventory world SilverCoins 35 300 0 0 0 0 1.57\n  KQObject::setScript $NewInvItem coin.cs\n  sendEvent 0 $1 KQAnimPlayEvent open\n else\n  KQMonster::speak Connor Connor 0 109 14 33 1 1\n endif\nendif\n";

    fn with_object(script: &'static str, var: &str) -> Active {
        let sources = [("o.cs".to_string(), script.as_bytes().to_vec()), ("coin.cs".to_string(), COIN.as_bytes().to_vec())];
        let mut v = Vars::default();
        v.set(var, "true".into());
        let mut a = Active::new(v, vec![], vec![], ["Connor".to_string()], sources);
        a.world.items = crate::item::tests::items();
        let mut player = crate::movement::tests::still_then_open("basket1");
        player.movements[0].name = "stop".into();
        a.add_objects([ActiveObject {
            player,
            script: Some("o.cs".into()),
            bounds: Some(([-200.0, -200.0, 0.0], [200.0, 200.0, 400.0])),
            collision_type: 33,
            collision_mask: 0,
            accepts_inventory: None,
            accept_all_items: false,
            too_far_away_action: 0,
            do_not_point_at: false,
            hidden: false,
        }]);
        a.start();
        a
    }

    #[test]
    fn a_basket_gives_once_and_opens() {
        let mut a = with_object(BASKET, "B::full");
        assert_eq!(a.scripts.vars.get("B::init"), Some("basket1"), "Init at the start (Q64 stand-in)");
        let near = [0.0, -1000.0, 0.0];
        let r = a.click("basket1", near, 2350.0, None);
        assert_eq!(r, ClickResult::Script(["basket1", "ConnorAction", "do", "do"].map(String::from).to_vec()));
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 5);
        // The event is handled in the next update, which starts `open`.
        a.update();
        assert_eq!(a.world.objects[0].player.movement_name(), "open");
        for _ in 0..14 {
            a.update();
        }
        assert_eq!(a.scripts.vars.get("B::ended"), None);
        a.update();
        assert_eq!(a.scripts.vars.get("B::ended"), Some("open"), "EndLoop open (Q64 stand-in)");
        // Second click: empty now, nothing more given.
        a.click("basket1", near, 2350.0, None);
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 5);
        // Too far; an item in hand that it doesn't accept.
        assert_eq!(a.click("basket1", [0.0, -6000.0, 0.0], 2350.0, None), ClickResult::TooFar);
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("basket1", near, 2350.0, Some(&bread)), ClickResult::Refused);
        // The pick ray finds it.
        assert_eq!(a.pick([0.0, -5000.0, 200.0], [0.0, 1.0, 0.0]).map(|(n, _)| n), Some("basket1".to_string()));
    }

    #[test]
    fn a_small_chest_puts_coins_beside_itself_then_says_empty() {
        let mut a = with_object(SMALL_CHEST, "S::full");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let coins = a.world.items.get("SilverCoins.1").unwrap().clone();
        assert_eq!((coins.quantity, coins.place, coins.script.as_deref()), (35, Place::World { position: [300.0, 0.0, 0.0], angles: [0.0, 0.0, 1.57] }, Some("coin.cs")));
        a.update();
        assert_eq!(a.world.objects[0].player.movement_name(), "open");
        a.take_log();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.take_log().join("\n");
        assert!(log.contains("stub (later milestone, nothing done): KQMonster::speak Connor Connor 0 109 14 33 1 1"), "{log}");
        // A movement it doesn't have.
        let mut b = with_object("sendEvent 0 $1 KQAnimPlayEvent fly\n", "x");
        b.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        b.update();
        assert!(b.take_log().join("\n").contains("basket1 has no movement 'fly'; nothing"));
    }

    #[test]
    fn walk_and_run_zones_ask_for_the_run_switch_and_the_camera_range() {
        let mut a = with_trap("if test $2 == On\n KQConner::setRun 0\n KQCamera::setRange 1000 2500\n KQMusic::push minside.aud\nelse if test $2 == Off\n KQCamera::setRange 3000 3000\nendif\nendif\n");
        connor_at(&mut a, 0.0);
        let r = std::mem::take(&mut a.world.requests);
        assert!(matches!(&r[..], [Request::SetRun { run: false, .. }, Request::SetRange { min, max, .. }] if *min == 1000.0 && *max == 2500.0), "{r:?}");
        assert!(a.take_trap_log().join("\n").contains("stub (later milestone, nothing done): KQMusic::push minside.aud"));
        // Refused: min not below max; nothing asked.
        connor_at(&mut a, 5000.0);
        assert!(a.world.requests.is_empty());
        assert_eq!(a.scripts.vars.get(crate::script::RETURN), Some("1"));
    }
}
