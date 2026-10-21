//! Item objects and inventories (M3 step 3e-2): coins, crystals,
//! mushrooms... lying in the world or carried by someone.
//!
//! Rules from `docs/behaviour/items.md` (traced unless marked):
//!
//! - An item object has a type (a line of `Invinfo.itl`,
//!   `kq8_data::items`), a quantity (one object can be a stack of 200
//!   coins) and a third number, default −1 (rule 3, meaning not traced).
//! - Every item object is clickable (collision type 0x220), has mask 0x809
//!   and `doNotPointAt` on (rule 4). By the pair rule it never collides
//!   with Connor (type 4, mask 0x281B).
//! - A placed item stays where it was put (rule 5).
//! - `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` makes a
//!   new item object: `world` → placed there (z −1 / −2 = floor / surface
//!   height, `objects.md` Q17 rule 2), anyone else → carried by them
//!   (rule 9).
//! - `KQMonster::checkInventory <who> <type>`: is there one (rule 10).
//!
//! - A new item object is named `NoName<n>` from one counter that starts
//!   at 0 (rule 9, Q60). The counter is the game's for every nameless
//!   object; only items are nameless here so far, and there is no travel
//!   between worlds or saving yet, so it lives with the item list (M6:
//!   game-wide and saved). The original names an object the first time
//!   the name is needed; ours are named when made, which gives the same
//!   order because every item's name is used at once (`NewInvItem`).
//!
//! Our stand-in (not in the note): an item is clicked by the box of its type's shape in
//! `BOUNDS.KQ` (shape name lower-cased; our reading of `chests.md`'s
//! crystal check), which the caller supplies.

use std::collections::BTreeMap;

use kq8_data::items::{ItemType, ItemTypes};

use crate::collide::Placement;

/// Rule 4.
pub const ITEM_TYPE: u32 = 0x220;
pub const ITEM_MASK: u32 = 0x809;

/// What an object accepts from Connor's hand (M3 step 3g-3, `clicking.md`
/// rules 13-16, traced): an "accept everything" switch and a list of item
/// types (canonical names from `Invinfo.itl`).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Accepts {
    pub all: bool,
    pub types: Vec<String>,
}

impl Accepts {
    /// Reads an `acceptsInventory` / `KQObject::setAccepts` value and
    /// **adds** it (rules 14, 16): split at commas; `all` / `INVITEM_All`
    /// (any capitals) turns the switch on; `none` ends the list; any other
    /// word is a type name (rule 2 lookup). Returns the words that name no
    /// type, which are skipped (the original would crash, "Do not
    /// reproduce").
    ///
    /// The original honours `all` only as the first word and then ignores
    /// the rest; later it would crash. We treat it as "accept everything"
    /// anywhere ("Do not reproduce"); no data has it later.
    pub fn read(&mut self, value: &str, types: &ItemTypes) -> Vec<String> {
        let mut unknown = Vec::new();
        for word in value.split(',').map(str::trim) {
            if word.eq_ignore_ascii_case("all") || word.eq_ignore_ascii_case("INVITEM_All") {
                self.all = true;
            } else if word.eq_ignore_ascii_case("none") {
                break;
            } else if let Some(t) = types.find(word) {
                if !self.types.contains(&t.name) {
                    self.types.push(t.name.clone());
                }
            } else {
                unknown.push(word.to_string());
            }
        }
        unknown
    }

    /// Whether an item of type `kind` is accepted (rule 13): the switch,
    /// or its type in the list.
    pub fn takes(&self, kind: &str, types: &ItemTypes) -> bool {
        self.all || types.find(kind).is_some_and(|t| self.types.contains(&t.name))
    }
}

/// Where an item object is.
#[derive(Debug, Clone, PartialEq)]
pub enum Place {
    /// Lying in the world.
    World { position: [f32; 3], angles: [f32; 3] },
    /// In someone's inventory (their name as the script gave it).
    Carried(String),
    /// Deleted (`deleteObject`): nowhere.
    Gone,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ItemObject {
    pub name: String,
    /// The type's `IDName`, e.g. `INVITEM_Crystal`.
    pub kind: String,
    pub quantity: i64,
    /// Rule 3: default −1, meaning not traced (M4).
    pub third: i64,
    pub place: Place,
    /// `KQObject::setHidden`: not drawn, can't be clicked (`objects.md`
    /// "Hidden objects").
    pub hidden: bool,
    /// Runs after a pickup (rule 7); `KQObject::setScript`.
    pub script: Option<String>,
    /// Rule 4: on; `KQObject::setDontLook`.
    pub do_not_point_at: bool,
    /// When it last went into someone's inventory: a number counting up
    /// (0: never); the quest strip lists in this order (M4 4c-3,
    /// `inventory.md` rule 5).
    pub added: u64,
}

impl ItemObject {
    /// Where it lies, if in the world.
    pub fn placement(&self) -> Option<Placement> {
        match &self.place {
            Place::World { position, angles } => Some(Placement { rotation: kq8_data::objects::rotation(*angles), position: *position }),
            _ => None,
        }
    }

    pub fn in_world(&self) -> bool {
        matches!(self.place, Place::World { .. })
    }

    pub fn carried_by(&self, who: &str) -> bool {
        matches!(&self.place, Place::Carried(w) if w.eq_ignore_ascii_case(who))
    }
}

/// A `KQMonster::inventory` or `give` line, read (rule 9).
#[derive(Debug, Clone, PartialEq)]
pub struct Make {
    /// `world` or a character's name.
    pub who: String,
    pub kind: String,
    /// Default 1.
    pub quantity: i64,
    /// As written (−1 / −2 not yet replaced).
    pub position: Option<[f32; 3]>,
    /// Missing ones 0.
    pub angles: [f32; 3],
}

impl Make {
    pub fn in_world(&self) -> bool {
        self.who.eq_ignore_ascii_case("world")
    }
}

/// Reads `KQMonster::inventory <who> <type> [qty] [x y z [rx ry rz]]` or
/// `give <type> [qty]` (words expanded, `words[0]` the command). `Err`
/// with a reason for the log. A world item needs x y z (our choice: every
/// such line in the data gives one).
pub fn read_make(words: &[String]) -> Result<Make, String> {
    let line = words.join(" ");
    let (who, rest) = if words[0].eq_ignore_ascii_case("give") {
        ("Connor".to_string(), &words[1..])
    } else {
        match words.get(1) {
            Some(w) => (w.clone(), &words[2..]),
            None => return Err("usage: KQMonster::inventory <who> <type> [quantity] [x y z [rx ry rz]]".into()),
        }
    };
    let Some(kind) = rest.first() else { return Err(format!("usage: {} needs an item type: '{line}'", words[0])) };
    let numbers = &rest[1..];
    if let Some(bad) = numbers.iter().find(|w| !crate::script::looks_like_number(w)) {
        return Err(format!("{line}: '{bad}' is not a number"));
    }
    let n: Vec<f32> = numbers.iter().map(|w| crate::script::number_value(w) as f32).collect();
    let position = match n.len() {
        0 | 1 => None,
        2 | 3 => return Err(format!("{line}: a position needs x y z")),
        _ => Some([n[1], n[2], n[3]]),
    };
    let make = Make {
        who,
        kind: kind.clone(),
        quantity: n.first().map_or(1, |q| *q as i64),
        position,
        angles: [n.get(4), n.get(5), n.get(6)].map(|a| a.copied().unwrap_or(0.0)),
    };
    if make.in_world() && make.position.is_none() {
        return Err(format!("{line}: no position for a world item"));
    }
    Ok(make)
}

/// Every item object of the world and the type list.
#[derive(Debug, Clone, Default)]
pub struct Items {
    pub types: ItemTypes,
    /// Each shape's click box (own space) by lower-case shape name.
    boxes: BTreeMap<String, ([f32; 3], [f32; 3])>,
    /// Every item object made so far, also carried and gone ones.
    pub objects: Vec<ItemObject>,
    /// The next `NoName<n>` number (rule 9).
    next_no_name: usize,
    /// The last [`ItemObject::added`] number given.
    last_added: u64,
}

impl Items {
    /// `boxes`: (shape name, min, max) from `BOUNDS.KQ`.
    pub fn new(types: ItemTypes, boxes: impl IntoIterator<Item = (String, [f32; 3], [f32; 3])>) -> Self {
        let boxes = boxes.into_iter().map(|(s, a, b)| (s.to_ascii_lowercase(), (a, b))).collect();
        Items { types, boxes, objects: Vec::new(), next_no_name: 0, last_added: 0 }
    }

    /// The next order number for an item made carried (0 if not).
    fn next_added(&mut self, place: &Place) -> u64 {
        if matches!(place, Place::Carried(_)) {
            self.last_added += 1;
            self.last_added
        } else {
            0
        }
    }

    /// Item `name` has just gone into an inventory (M4 4c-3).
    pub fn mark_added(&mut self, name: &str) {
        if let Some(i) = self.index(name) {
            self.last_added += 1;
            self.objects[i].added = self.last_added;
        }
    }

    /// The type a script means by `name` (rule 2).
    pub fn kind(&self, name: &str) -> Option<&ItemType> {
        self.types.find(name)
    }

    /// The click box of an item object's type, if its shape has one.
    pub fn click_box(&self, item: &ItemObject) -> Option<([f32; 3], [f32; 3])> {
        let t = self.types.find(&item.kind)?;
        self.boxes.get(&t.shape.to_ascii_lowercase()).copied()
    }

    pub fn index(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name.eq_ignore_ascii_case(name))
    }

    pub fn get(&self, name: &str) -> Option<&ItemObject> {
        self.index(name).map(|i| &self.objects[i])
    }

    /// The next `NoName<n>` name of the game-wide counter (rule 9; also
    /// `loadKQ … @`, `anim-scripts.md` rule 12, M5 5d-2).
    pub fn next_name(&mut self) -> String {
        let name = format!("NoName{}", self.next_no_name);
        self.next_no_name += 1;
        name
    }

    /// Makes a new item object of type `kind` (any spelling rule 2
    /// allows). `Err` if there is no such type.
    pub fn make(&mut self, kind: &str, quantity: i64, place: Place) -> Result<&ItemObject, String> {
        let t = self.types.find(kind).ok_or_else(|| format!("no item type '{kind}' (looked for {})", kq8_data::items::full_name(kind)))?;
        let kind = t.name.clone();
        let name = format!("NoName{}", self.next_no_name);
        self.next_no_name += 1;
        let added = self.next_added(&place);
        self.objects.push(ItemObject { name, kind, quantity, third: -1, place, hidden: false, script: None, do_not_point_at: true, added });
        Ok(self.objects.last().expect("just pushed"))
    }

    /// An item object with a name of its own (`Connor.kq`
    /// `[ConnerInventory]` sections, M4 4c-2): the `NoName` counter is not
    /// used. `Err` if there is no such type or the name is taken.
    pub fn add_named(&mut self, name: &str, kind: &str, quantity: i64, third: i64, place: Place) -> Result<&ItemObject, String> {
        let t = self.types.find(kind).ok_or_else(|| format!("no item type '{kind}' (looked for {})", kq8_data::items::full_name(kind)))?;
        let kind = t.name.clone();
        if self.index(name).is_some() {
            return Err(format!("an item named '{name}' exists already"));
        }
        let added = self.next_added(&place);
        self.objects.push(ItemObject { name: name.to_string(), kind, quantity, third, place, hidden: false, script: None, do_not_point_at: true, added });
        Ok(self.objects.last().expect("just pushed"))
    }

    /// The first item object of type `kind` that `who` carries, or (for
    /// `world`, any capitals) that lies in the world (rule 10).
    pub fn find(&self, who: &str, kind: &str) -> Option<&ItemObject> {
        let t = self.types.find(kind)?;
        let world = who.eq_ignore_ascii_case("world");
        self.objects
            .iter()
            .filter(|o| o.kind == t.name)
            .find(|o| if world { o.in_world() } else { o.carried_by(who) })
    }

    /// One line about item object `name` for the logs: name, type,
    /// quantity, where it is.
    pub fn describe(&self, name: &str) -> String {
        let Some(o) = self.get(name) else { return format!("{name}: no such item") };
        let where_ = match &o.place {
            Place::World { position: p, angles: a } => {
                format!("in the world at ({:.0}, {:.0}, {:.0}) turned ({}, {}, {})", p[0], p[1], p[2], a[0], a[1], a[2])
            }
            Place::Carried(w) => format!("carried by {w} (now {} of them)", self.count(w, &o.kind)),
            Place::Gone => "gone".into(),
        };
        let extra = match (&o.script, o.hidden) {
            (None, false) => String::new(),
            (s, h) => format!(" [script {s:?}, hidden {h}]"),
        };
        format!("{}: {} x{} {where_}{extra}", o.name, o.kind, o.quantity)
    }

    /// How many of type `kind` `who` carries (quantities added up; for the
    /// log).
    pub fn count(&self, who: &str, kind: &str) -> i64 {
        self.objects.iter().filter(|o| o.kind.eq_ignore_ascii_case(kind) && o.carried_by(who)).map(|o| o.quantity).sum()
    }
}

/// `items.md` rule 12 (`KQObject::tossItems`; `fighting.md` rule 31 for
/// drops, M5 5c-8): a thrown item's launch: a random heading all round,
/// 75°–85° above the horizontal, 5,000–6,500 units per second
/// (`thrown-items.md` rule 3).
pub const TOSS_ELEVATION: (f32, f32) = (75.0, 85.0);
pub const TOSS_SPEED: (f32, f32) = (5000.0, 6500.0);
/// The arc's own gravity, units/s² (`thrown-items.md` rule 5).
pub const TOSS_GRAVITY: f32 = 8085.0;
/// The first step must leave the launch point by this much (rule 6).
pub const FIRST_STEP: f32 = 25.0;
/// The longest flight since the last throw, seconds (rule 16).
pub const LONGEST_FLIGHT: f32 = 10.0;
/// The object type bits that stop a thrown item (`thrown-items.md` rule
/// 11): the default type 1 and 0x100 (the barrels' 0x141).
pub const ITEM_OBSTACLE_BITS: u32 = 0x101;

/// A thrown item in the air: its thrower (`thrown-items.md`; M5 5e-1).
#[derive(Debug, Clone)]
pub struct Flight {
    /// The item object's name.
    pub item: String,
    /// The launch point and speeds (rule 3): x, y per second along the
    /// heading, and the upward speed.
    pub from: [f32; 3],
    pub velocity: [f32; 3],
    /// The flight clock, seconds since the last throw (rule 3).
    pub clock: f32,
    /// Updates flown (for the log).
    pub updates: u32,
    /// Rule 8: the wanted height went below the height it had.
    pub coming_down: bool,
    /// Rule 12.3: the arc has ended; it only settles now (rule 15).
    pub arc_over: bool,
    /// Rule 12.2: it has touched something (the land sound played).
    pub touched: bool,
    /// Rule 16: its position at the start of the last update, in whole
    /// units (cut toward zero: ours, the note says only "whole units").
    held: Option<[i32; 3]>,
    /// Its mover (rule 7): the item's box, type 0x220, mask 0x809, steps
    /// 0, gravity off during the arc.
    pub mover: Box<crate::connor_mover::Mover>,
    /// Its type's shape has a `BOUNDS.KQ` box ("Edge cases": without one
    /// it tests nothing on the way).
    pub shaped: bool,
    /// Its tumble (rule 9): radians **per update** about its own x axis,
    /// the same every update until a new throw.
    pub spin: f32,
    /// Its turn about its own x axis so far (rule 2: 0 at the throw).
    pub turn: f32,
}

/// The tumble's range, radians per update (`thrown-items.md` rule 3).
pub const TUMBLE: (f32, f32) = (-0.5, 0.5);

/// What one update of a flight did, for the log.
#[derive(Debug, Clone, Default)]
pub struct FlightStep {
    /// Where the arc wanted it (`None` once it only settles).
    pub wanted: Option<[f32; 3]>,
    /// Where it is after the move.
    pub position: [f32; 3],
    /// What it touched this update: pass and name (`terrain` for the
    /// floor grid).
    pub touches: Vec<String>,
    /// The first touch of the whole flight (rule 12.2: the land sound).
    pub first_touch: Option<String>,
    /// The arc ended this update, and why (rules 12.3, 14).
    pub arc_ended: Option<String>,
    /// Thrown again this update (rule 12.3): what by, and the new launch
    /// for the log.
    pub thrown_again: Vec<String>,
    /// The flight ended at the start of this update (rule 16): `held` or
    /// `10 s`. Nothing moved then.
    pub ended: Option<&'static str>,
    /// Its angles now (rules 9, 10): the tumble about its own x axis, all
    /// 0 once the flight has ended.
    pub angles: [f32; 3],
}

/// Rule 3: the launch speeds from three random numbers in [0, 1):
/// heading, elevation, speed.
fn launch_velocity(r: [f32; 3]) -> [f32; 3] {
    let heading = r[0] * std::f32::consts::TAU;
    let up = (TOSS_ELEVATION.0 + r[1] * (TOSS_ELEVATION.1 - TOSS_ELEVATION.0)).to_radians();
    let speed = TOSS_SPEED.0 + r[2] * (TOSS_SPEED.1 - TOSS_SPEED.0);
    let flat = speed * up.cos();
    [flat * heading.sin(), -flat * heading.cos(), speed * up.sin()]
}

/// Rule 3: the tumble from a random number in [0, 1).
fn tumble_of(r: f32) -> f32 {
    TUMBLE.0 + r * (TUMBLE.1 - TUMBLE.0)
}

impl Flight {
    /// The launch from three random numbers in [0, 1): heading,
    /// elevation, speed (rule 3). `shape`: the item's `BOUNDS.KQ` box.
    pub fn launch(item: &str, from: [f32; 3], r: [f32; 3], shape: Option<([f32; 3], [f32; 3])>) -> Self {
        let (min, max) = shape.unwrap_or(([-1.0; 3], [1.0; 3]));
        let mut mover = crate::connor_mover::Mover::character(from, false, min, max, (ITEM_TYPE, ITEM_MASK), (0.0, 0.0), -1);
        // "Edge cases": no box, no collision shape: not even the floor grid
        // in the passes (the raise of rule 15 still applies).
        mover.terrain_in_passes = shape.is_some();
        Flight {
            item: item.to_string(),
            velocity: launch_velocity(r),
            from,
            clock: 0.0,
            updates: 0,
            coming_down: false,
            arc_over: false,
            touched: false,
            held: None,
            mover: Box::new(mover),
            shaped: shape.is_some(),
            spin: 0.0,
            turn: 0.0,
        }
    }

    /// The tumble from a random number in [0, 1) (rule 3).
    pub fn with_tumble(mut self, r: f32) -> Self {
        self.spin = tumble_of(r);
        self
    }

    /// Rule 12.3: thrown again from `from` with fresh random launch values
    /// `r` and tumble `spin`: clock back to 0, not coming down, landed
    /// flag cleared, the arc's own gravity again (ordinary gravity off).
    /// The land sound stays played (once per flight).
    pub fn throw_again(&mut self, from: [f32; 3], r: [f32; 3], spin: f32) {
        self.from = from;
        self.velocity = launch_velocity(r);
        self.clock = 0.0;
        self.coming_down = false;
        self.arc_over = false;
        self.mover.use_gravity = false;
        self.mover.fall_timer = 0.0;
        self.spin = tumble_of(spin);
    }

    /// Rule 5: where the arc puts it `t` seconds after the throw.
    pub fn arc_at(&self, t: f32) -> [f32; 3] {
        let [vx, vy, vz] = self.velocity;
        [self.from[0] + vx * t, self.from[1] + vy * t, self.from[2] + vz * t - 0.5 * TOSS_GRAVITY * t * t]
    }

    /// Rule 12.3, "anything else": the arc ends here; it is landed and
    /// coming down, and ordinary gravity is on (rule 15).
    fn end_arc(&mut self) {
        self.arc_over = true;
        self.coming_down = true;
        self.mover.use_gravity = true;
    }

    /// One update of `dt` s (rule 4) from `at` (the item's place now) in
    /// `world`: the end test (rule 16), the arc's wanted position (rules
    /// 5, 6, 8, 14) or where it is (rule 15), the move through its mover
    /// (rule 7) and what its touches do (rule 12).
    ///
    /// `throws_again`: whether a touched thing (by name) has a script or is
    /// animated (rule 12.3); `random`: fractions in [0, 1) for a new throw.
    pub fn update(&mut self, at: [f32; 3], dt: f32, world: &dyn crate::connor_mover::MoverWorld, throws_again: &dyn Fn(&str) -> bool, random: &mut dyn FnMut() -> f32) -> FlightStep {
        let mut step = FlightStep { position: at, ..Default::default() };
        // Rule 16, at the start of the update; rule 10: no turn at all.
        let whole = at.map(|v| v as i32);
        if self.coming_down && self.held == Some(whole) {
            step.ended = Some("held");
            return step;
        }
        if self.clock > LONGEST_FLIGHT {
            step.ended = Some("10 s");
            return step;
        }
        self.held = Some(whole);
        self.updates += 1;
        self.clock += dt;
        self.mover.position = at;
        let wanted = if self.arc_over {
            at
        } else {
            // Rule 6 (never in practice: the first step is 300+).
            let mut w = self.arc_at(self.clock);
            for _ in 0..1000 {
                let d = (0..3).map(|k| (w[k] - self.from[k]).powi(2)).sum::<f32>().sqrt();
                if d >= FIRST_STEP {
                    break;
                }
                self.clock += dt;
                w = self.arc_at(self.clock);
            }
            if w[2] < at[2] {
                self.coming_down = true;
            }
            // Rule 14: the thrower's own floor-grid check.
            if self.coming_down && world.ground(w[0], w[1]).is_some_and(|(g, _)| w[2] < g) {
                self.end_arc();
                step.arc_ended = Some("its wanted height is below the floor grid (rule 14)".into());
            }
            step.wanted = Some(w);
            w
        };
        let report = self.mover.update(std::array::from_fn(|k| wanted[k] - at[k]), world);
        step.position = self.mover.position;
        step.touches = report.hits.iter().map(|h| format!("{} {} ({})", h.pass, h.hit.what, h.result)).collect();
        // Rule 12, at each touch in order.
        for h in &report.hits {
            if !self.touched {
                self.touched = true;
                step.first_touch = Some(h.hit.what.clone());
            }
            if h.hit.what != "terrain" && throws_again(&h.hit.what) {
                let r = [random(), random(), random()];
                let spin = random();
                self.throw_again(h.hit.position, r, spin);
                let v = self.velocity;
                step.arc_ended = None;
                step.thrown_again.push(format!(
                    "{} from ({:.0}, {:.0}, {:.0}): velocity ({:.0}, {:.0}, {:.0})",
                    h.hit.what, h.hit.position[0], h.hit.position[1], h.hit.position[2], v[0], v[1], v[2]
                ));
            } else if !self.arc_over {
                self.end_arc();
                step.arc_ended = Some(format!("touched {} (rule 12)", h.hit.what));
            }
        }
        // Rule 9: every update until the flight ends, also after a touch
        // (with the new tumble after a throw again: ours, the note gives
        // no order within the update).
        self.turn += self.spin;
        step.angles = [self.turn, 0.0, 0.0];
        step
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    #[test]
    fn a_thrown_item_tumbles_about_one_axis_and_ends_with_no_turn() {
        // thrown-items.md rules 2, 3, 9, 10 (M5 5e-3).
        assert_eq!(Flight::launch("x", [0.0; 3], [0.0; 3], None).with_tumble(0.0).spin, -0.5);
        assert_eq!(Flight::launch("x", [0.0; 3], [0.0; 3], None).with_tumble(0.75).spin, 0.25);
        let ground = crate::connor_mover::tests::Flat(0.0);
        let mut f = Flight::launch("x", [0.0; 3], [0.0, 0.0, 0.0], PEBBLE).with_tumble(0.6);
        let steps = fly(&mut f, &ground);
        let (last, flying) = steps.split_last().unwrap();
        for (n, s) in flying.iter().enumerate() {
            // The same step every update, per update, also after the touch.
            assert!((s.angles[0] - 0.1 * (n + 1) as f32).abs() < 1e-4 && s.angles[1] == 0.0 && s.angles[2] == 0.0, "{n}: {:?}", s.angles);
        }
        assert!(flying.iter().position(|s| s.first_touch.is_some()).unwrap() < flying.len() - 1, "turning after the touch too");
        assert_eq!((last.ended, last.angles), (Some("held"), [0.0; 3]));
    }

    const DT: f32 = 1.0 / 15.0;
    const PEBBLE: Option<([f32; 3], [f32; 3])> = Some(([-50.0, -50.0, 0.0], [50.0, 50.0, 100.0]));

    /// Flies until the flight ends (at most 200 updates): every step.
    fn fly(f: &mut Flight, world: &dyn crate::connor_mover::MoverWorld) -> Vec<FlightStep> {
        fly_by(f, world, &|_| false)
    }

    /// As [`fly`], with what throws it again (rule 12.3).
    fn fly_by(f: &mut Flight, world: &dyn crate::connor_mover::MoverWorld, again: &dyn Fn(&str) -> bool) -> Vec<FlightStep> {
        let mut rng = crate::emitter::Rng::new(7);
        let mut at = f.from;
        let mut steps = Vec::new();
        for _ in 0..200 {
            let s = f.update(at, DT, world, again, &mut || rng.fraction());
            at = s.position;
            let done = s.ended.is_some();
            steps.push(s);
            if done {
                break;
            }
        }
        steps
    }

    #[test]
    fn a_thrown_item_follows_the_exact_arc_and_lands_on_the_floor_grid() {
        // thrown-items.md rules 3-5, 12, 16 (M5 5e-1).
        let low = Flight::launch("x", [0.0; 3], [0.0, 0.0, 0.0], PEBBLE);
        let v = low.velocity;
        assert!((v[2] / (v[0] * v[0] + v[1] * v[1]).sqrt() - 75f32.to_radians().tan()).abs() < 1e-3);
        assert!(((v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt() - 5000.0).abs() < 0.5);
        let ground = crate::connor_mover::tests::Flat(100.0);
        let mut f = Flight::launch("x", [0.0, 0.0, 100.0], [0.25, 1.0, 1.0], PEBBLE);
        let steps = fly(&mut f, &ground);
        // On the arc, exactly, until the first touch.
        for (n, s) in steps.iter().take(20).enumerate() {
            let want = f.arc_at((n + 1) as f32 * DT);
            assert!((0..3).all(|k| (s.position[k] - want[k]).abs() < 0.5), "update {}: {:?} vs {want:?}", n + 1, s.position);
        }
        let touch = steps.iter().position(|s| s.first_touch.is_some()).expect("a touch");
        // About 2 x 6,475 / 8,085 = 1.6 s up and down.
        assert!((22..=25).contains(&touch), "{touch}");
        assert_eq!(steps[touch].first_touch.as_deref(), Some("terrain"));
        assert!(steps[touch].arc_ended.is_some());
        assert_eq!(steps.iter().filter(|s| s.first_touch.is_some()).count(), 1, "the land sound once");
        let last = steps.last().unwrap();
        assert_eq!(last.ended, Some("held"));
        assert_eq!(last.position[2], 100.0);
        assert!((900.0..1000.0).contains(&last.position[0]) && last.position[1].abs() < 1.0, "85 degrees, 6,500: 566 per s for 1.67 s; heading 90 degrees is +x: {:?}", last.position);
    }

    #[test]
    fn a_thrown_item_drops_at_the_foot_of_a_wall() {
        // Rules 12, 15 (Q137 c1): it hits the wall, the arc ends, it falls
        // straight down beside it; never through.
        use crate::connor_mover::tests::{building, node, Flat};
        use kq8_data::building::BspChild::Leaf;
        let b = [building(vec![node(0, Leaf(0), Leaf(1))], vec![false, true], vec![([0.0, 1.0, 0.0], 0.0)])];
        let ground = Flat(0.0);
        let scene = crate::connor_mover::Scene::new(&ground, &b, &[]);
        // Heading 0: toward -y, the wall's face is y = 0.
        let mut f = Flight::launch("x", [0.0, 300.0, 0.0], [0.0, 0.0, 0.0], PEBBLE);
        let steps = fly(&mut f, &scene);
        assert!(steps.iter().all(|s| s.position[1] > 0.0), "never through the wall");
        let touch = steps.iter().find(|s| s.first_touch.is_some()).expect("a touch");
        assert_eq!(touch.first_touch.as_deref(), Some("test"));
        assert!(touch.position[2] > 300.0, "hit high up: {:?}", touch.position);
        let last = steps.last().unwrap();
        assert_eq!(last.ended, Some("held"));
        assert_eq!(last.position[2], 0.0, "down at the foot");
        assert!(last.position[1] < 300.0, "beside the wall: {:?}", last.position);
    }

    #[test]
    fn a_thrown_item_stays_on_a_building_floor_above_the_terrain() {
        // Rule 13 (Q137 c2): a building floor at z 500 over terrain at 0.
        use crate::connor_mover::tests::{building, node, Flat};
        use kq8_data::building::BspChild::Leaf;
        let b = [building(vec![node(0, Leaf(0), Leaf(1))], vec![false, true], vec![([0.0, 0.0, 1.0], -500.0)])];
        let ground = Flat(0.0);
        let scene = crate::connor_mover::Scene::new(&ground, &b, &[]);
        let mut f = Flight::launch("x", [0.0, 0.0, 600.0], [0.4, 0.5, 0.5], PEBBLE);
        let steps = fly(&mut f, &scene);
        let last = steps.last().unwrap();
        assert_eq!(last.ended, Some("held"));
        assert!(last.position[2] >= 499.0 && last.position[2] < 600.0, "on the floor: {:?}", last.position);
        assert_eq!(steps.iter().find_map(|s| s.first_touch.clone()).as_deref(), Some("test"));
    }

    #[test]
    fn a_thrown_item_is_thrown_again_off_a_scripted_object() {
        // Rule 12.3 (M5 5e-2): a wide made-up "barrel" top at z 500 under
        // the launch point throws it up again until it comes down off it,
        // on the terrain at 0; a plain object of the same size stops it.
        use crate::collide::{BoxCollider, ColliderKind, Placement};
        let ground = crate::connor_mover::tests::Flat(0.0);
        let kind = ColliderKind { collision_type: 0x141, ..ColliderKind::default() };
        let o = [BoxCollider { name: "barrel".into(), min: [-1500.0, -1500.0, 0.0], max: [1500.0, 1500.0, 500.0], placement: Placement::at([0.0; 3]), kind }];
        let scene = crate::connor_mover::Scene::new(&ground, &[], &o);
        let mut f = Flight::launch("x", [0.0, 0.0, 600.0], [0.1, 1.0, 0.0], PEBBLE);
        let steps = fly_by(&mut f, &scene, &|n| n == "barrel");
        let again = steps.iter().filter(|s| !s.thrown_again.is_empty()).count();
        assert!(again >= 1, "thrown again at least once: {:?}", steps.iter().map(|s| (s.position, s.touches.clone())).collect::<Vec<_>>());
        assert_eq!(steps.iter().filter(|s| s.first_touch.is_some()).count(), 1, "the land sound once per flight");
        let last = steps.last().unwrap();
        assert_eq!((last.ended, last.position[2]), (Some("held"), 0.0), "off the barrel, on the terrain: {:?}", last.position);
        assert!(last.position[0].abs().max(last.position[1].abs()) > 1500.0, "{:?}", last.position);
        let mut f = Flight::launch("x", [0.0, 0.0, 600.0], [0.1, 1.0, 0.0], PEBBLE);
        let steps = fly(&mut f, &scene);
        let last = steps.last().unwrap();
        assert_eq!(last.ended, Some("held"));
        assert!((499.0..600.0).contains(&last.position[2]), "a plain object stops it on its top: {:?}", last.position);
    }

    #[test]
    fn a_thrown_item_without_a_box_still_ends_on_the_floor_grid() {
        // "Edge cases": no shape, nothing tested on the way; rule 14 ends
        // the arc and rule 15's raise puts it on the floor grid.
        let ground = crate::connor_mover::tests::Flat(0.0);
        let mut f = Flight::launch("x", [0.0; 3], [0.0, 0.0, 0.0], None);
        let steps = fly(&mut f, &ground);
        assert!(steps.iter().any(|s| s.arc_ended.as_deref().is_some_and(|w| w.contains("rule 14"))));
        assert!(steps.iter().all(|s| s.first_touch.is_none()), "no touches");
        let last = steps.last().unwrap();
        assert_eq!((last.ended, last.position[2]), (Some("held"), 0.0));
    }

    /// A two-type list in the file's layout (made-up values).
    pub fn types() -> ItemTypes {
        let list = b"[InventoryItemTypeList]\nnumItems=2\n\
item0=\tKQInventoryItemType\tINVITEM_Crystal\t3020\t50\t0\t0\t0\t0\t1\tHealthItem\tIDBMP_X\t13 0 0 1\tNoModeChange\t-1\t1\tpotnland.aud\tenergy.aud\thealing.aud\tcrystl.KQS\tnone\t250\t200\n\
item1=\tKQInventoryItemType\tINVITEM_SilverCoins\t4030\t0\t0\t0\t0\t0\t1\tMoney\tIDBMP_Y\t47 0 0 1\tNoModeChange\t0\t0\tnone\tcoins.aud\tcoins.aud\tpouch.kqs\tnone\t150\t256\n";
        ItemTypes::parse(list).0
    }

    pub fn items() -> Items {
        Items::new(types(), [("crystl.kqs".to_string(), [-60.0, -80.0, 0.0], [70.0, 80.0, 290.0])])
    }

    #[test]
    fn made_items_get_names_types_and_defaults() {
        let mut it = items();
        let at = Place::World { position: [100.0, 200.0, 300.0], angles: [0.0; 3] };
        let c = it.make("crystal", 3, at.clone()).unwrap().clone();
        assert_eq!((c.name.as_str(), c.kind.as_str(), c.quantity, c.third), ("NoName0", "INVITEM_Crystal", 3, -1));
        assert!(c.do_not_point_at && !c.hidden && c.script.is_none());
        let s = it.make("INVITEM_SilverCoins", 200, Place::Carried("Connor".into())).unwrap();
        assert_eq!(s.name, "NoName1");
        assert!(it.make("ladybell", 1, at).is_err());
        assert_eq!(it.objects.len(), 2);
    }

    #[test]
    fn find_looks_in_the_world_or_an_inventory() {
        let mut it = items();
        it.make("Crystal", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap();
        it.make("SilverCoins", 6, Place::Carried("Connor".into())).unwrap();
        it.make("SilverCoins", 4, Place::Carried("connor".into())).unwrap();
        assert_eq!(it.find("WORLD", "crystal").map(|o| o.name.as_str()), Some("NoName0"));
        assert!(it.find("Connor", "crystal").is_none());
        assert_eq!(it.find("Connor", "silverCoins").map(|o| o.name.as_str()), Some("NoName1"));
        assert!(it.find("world", "silverCoins").is_none());
        assert_eq!(it.count("Connor", "INVITEM_SilverCoins"), 10);
    }

    #[test]
    fn accept_lists_split_at_commas_and_compare_types() {
        let types = types();
        let mut a = Accepts::default();
        assert_eq!(a.read("crystal,INVITEM_SilverCoins", &types), Vec::<String>::new());
        assert_eq!(a.types, ["INVITEM_Crystal", "INVITEM_SilverCoins"]);
        assert!(a.takes("INVITEM_Crystal", &types) && a.takes("silvercoins", &types) && !a.all);
        // `none` ends the list; unknown names are skipped and returned.
        let mut b = Accepts::default();
        assert_eq!(b.read("INVITEM_LadyBell,Crystal,NONE,SilverCoins", &types), ["INVITEM_LadyBell"]);
        assert_eq!(b.types, ["INVITEM_Crystal"]);
        assert!(!b.takes("INVITEM_SilverCoins", &types) && !b.takes("INVITEM_LadyBell", &types));
        // `INVITEM_All` / `all`, any capitals: everything.
        let mut c = Accepts::default();
        c.read("invitem_all", &types);
        assert!(c.all && c.takes("INVITEM_LadyBell", &types));
        // Reading again adds; it never empties or switches off.
        b.read("SilverCoins", &types);
        b.read("none", &types);
        assert_eq!(b.types, ["INVITEM_Crystal", "INVITEM_SilverCoins"]);
    }

    #[test]
    fn the_click_box_comes_from_the_types_shape() {
        let mut it = items();
        let c = it.make("Crystal", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap().clone();
        assert_eq!(it.click_box(&c), Some(([-60.0, -80.0, 0.0], [70.0, 80.0, 290.0])), "crystl.KQS lower-cased");
        let s = it.make("SilverCoins", 1, Place::World { position: [0.0; 3], angles: [0.0; 3] }).unwrap().clone();
        assert_eq!(it.click_box(&s), None);
    }
}
