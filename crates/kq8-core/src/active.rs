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
//! (`doors.md` rule 6); `Init` and `EndLoop` (Q64, `clicking.md` rules
//! 19-21).
//!
//! `EndLoop` (M3 step 3g-4, `clicking.md` rule 21): when a play-once
//! movement of a door, chest or scripted object ends, the object cues
//! itself 3333 through the queue (so it arrives the next update) and its
//! script gets `EndLoop <movement>`; doors get it after `Open`/`Close`.
//! `getEndLoop <from> <to> 1` makes `<to>` get `Cue 300 0 <movement>` at
//! the same moment. Cue 500 (projectile) is only logged.
//!
//! Other clickable objects with a script (M3 step 3f: the fish, the
//! swinging signs, still objects): clicked the same way, `Init` at the
//! start; `deleteObject` removes them (and 3e-5 objects) for good and asks
//! the viewer to hide them and drop their collision (Q64 d stand-in).
//!
//! The click line (M3 step 3g-2, `clicking.md` rules 1-12): only shapes
//! with the clickable bit 0x20 are tested, nearest wins; buildings are
//! clickable by default (type 0x28), so a wall in front takes the click
//! and, with no script, swallows it. The ground is never tested.

use std::collections::{BTreeMap, BTreeSet};

use kq8_data::world_script::Vars;

use crate::chest::{Chest, ChestState};
use crate::click::{distance_to_box, ray_box, CLICKABLE};
use crate::collide::{pair_collides, BoxCollider, ColliderKind, Placement, CONNOR_MASK, CONNOR_TYPE};
use crate::door::{turn_degrees, Door};
use crate::events::{EventQueue, Pending};
use crate::item::{Accepts, Items, Place};
use crate::music::MusicRequest;
use crate::named_anim::{NamedAnimPlayer, Request as AnimRequest, Started};
use crate::script::{looks_like_number, number_value, ScriptHost, Scripts};
use crate::sound::Start;
use crate::stats::{ConnorStats, Stats};
use crate::talk::{Finished, Speak, Talk, CONNOR, TOO_FAR, WONT_WORK};
use kq8_data::msg::Tuple;
use crate::trap::{Trap, Visitor};

/// A `KQCueEvent` with this first number is an `EndLoop` call, not a `Cue`
/// (`clicking.md` rule 21).
pub const CUE_END_LOOP: &str = "3333";
/// A `KQCueEvent` with this first number launches the object's projectile
/// (combat, not studied; `clicking.md` rule 21).
pub const CUE_PROJECTILE: &str = "500";
/// What a `getEndLoop` listener is told: `Cue 300 0 <movement>`.
pub const CUE_HEARD_END: &str = "300";

/// Event types we know (doors.md "Commands door scripts use"). Any other
/// is an error and is not sent.
/// Script `KQMusic::push` / `pop` act this much game time later
/// (`sound.md` rule 57f).
pub const MUSIC_DELAY: f64 = 0.2;
/// The event queue's target for them (our name; no object is called this).
const MUSIC_TARGET: &str = "<KQMusic>";

pub const EVENT_TYPES: [&str; 4] = ["KQDoorEvent", "KQAnimPlayEvent", "KQCueEvent", "KQStatsChangeRequestEvent"];

/// `Die`'s `$3` (`anim-scripts.md` rule 20, Q86 e): the kind of the last
/// weapon hit the object took, `Normal` if it was never hit. Nothing is
/// hit by a weapon yet (fighting is M5), so always `Normal`.
pub const DIE_KIND: &str = "Normal";

/// What a script asked of Connor or the camera, for the caller to apply
/// (M3 step 3d-3). `at` is where in which script.
#[derive(Debug, Clone, PartialEq)]
pub enum Request {
    /// `KQConner::setRun 0|1` (connor-movement.md rule 4).
    SetRun { run: bool, at: String },
    /// `KQCamera::setRange <min> <max>` (camera.md "Distance" rule 2),
    /// already checked: min < max.
    SetRange { min: f32, max: f32, at: String },
    /// `deleteObject` on a placed object (3f): hide it, drop its
    /// collision and view box.
    Delete { name: String, at: String },
    /// `setActiveCamera <n>` (`object-lighting.md` rule 42): the draw
    /// distance is measured from the camera (`true`) or Connor.
    ActiveCamera { camera: bool, at: String },
    /// `makeActive <name> <n>` (rule 42): `name` always near (`true`) or
    /// back to the distance test.
    MakeActive { name: String, on: bool, at: String },
    /// `KQMonster::setState Connor special <list>` (`anim-scripts.md`
    /// rule 16): the comma-separated items as written.
    ConnorSpecial { items: String, at: String },
    /// `move <who> <flags> x y z rx ry rz` (`objects.md` "(d)") for
    /// Connor, the camera or a plain placed object: the caller moves it.
    Move { who: String, flags: u32, position: [f32; 3], turn: [f32; 3], at: String },
    /// `KQCamera::follow Connor|none` (`camera.md` rule 1): `true` =
    /// follow Connor, `false` = stay where it is.
    CameraFollow { connor: bool, at: String },
    /// `handsOff 1|0` (Q89 a stand-in): the player's movement keys, Jump
    /// and clicks are ignored while on.
    HandsOff { on: bool, at: String },
    /// `loadKQ <file>` while playing (M3 3k-4, Q90 stand-in): the copy
    /// of `name` prepared at the start comes into the world.
    Load { file: String, name: String, at: String },
    /// `KQConner::walkTo` / `runTo` / `pointAndCue` (`anim-scripts.md`
    /// rules 24–25, M3 3l-2): a scripted walk or turn; a walk also
    /// switches `handsOff` on.
    ConnorAuto { auto: crate::connor_auto::AutoMove, at: String },
    /// `walkExit`: a scripted walk ends now, with its cue (rule 24).
    WalkExit { at: String },
    /// `KQConner::endJump` while Connor is in the Jump state
    /// (`anim-scripts.md` rule 28, M3 3m-1): the jump is finished as a
    /// crash finishes it (gravity on, steps 325, cut short), no state
    /// change.
    EndJump { at: String },
    /// `KQCamera::addCamLoc x y z [newlist]` (`camera.md` "Camera flights
    /// and cuts" rule 1, M3 3m-4): a spot for the camera's list.
    CamSpot { spot: [f32; 3], newlist: bool, at: String },
    /// `KQCamera::flyToCamLoc x y z [seconds [endRoll]]` (rule 4).
    CamFly { aim: [f32; 3], seconds: Option<f32>, end_roll: Option<f32>, at: String },
    /// `KQCamera::cutToCamLoc x y z [roll]` (rule 3).
    CamCut { aim: [f32; 3], roll: f32, at: String },
    /// The turn after a click (`clicking.md` rules 25–33, M4 4a-8): after
    /// a `ConnorAction` call Connor turns to face `target` (the clicked
    /// object's position) with the automatic turn, if his state allows it
    /// (rule 28). Queued after the script's own requests, so the caller
    /// looks at his state after those (rule 27).
    ClickTurn { name: String, target: [f32; 2], at: String },
}

/// A panel command of a script (`interface.md` rule 15, M4 4c-2).
#[derive(Debug, Clone, PartialEq)]
pub enum GuiOrder {
    /// `setLowerGui` / `setUpperGui` / `setMapGui` 0|1.
    Set { group: crate::panels::Group, on: bool, at: String },
    /// `getGuiStatus`.
    Status { at: String },
}

/// A placement moved by `move` (`objects.md` "(d)", Q21): flags 1 and 3
/// add the position along the world axes, 0 and 2 set it; flags 2 and 3
/// add the turn, 0 and 1 set it. Adding turns is done only about Z (all
/// cases in the game); otherwise the turn stays, and the note says so.
pub fn moved_placement(p: Placement, flags: u32, position: [f32; 3], turn: [f32; 3]) -> (Placement, Option<&'static str>) {
    let position = if flags & 1 != 0 { std::array::from_fn(|k| p.position[k] + position[k]) } else { position };
    let (rotation, note) = if flags & 2 == 0 {
        (kq8_data::objects::rotation(turn), None)
    } else if turn == [0.0; 3] {
        (p.rotation, None)
    } else if turn[..2] == [0.0; 2] {
        // Row vectors: the old turn, then a turn about Z.
        let z = kq8_data::objects::rotation([0.0, 0.0, turn[2]]);
        (std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| p.rotation[i][k] * z[k][j]).sum())), None)
    } else {
        (p.rotation, Some("a relative turn not about Z: not applied (not traced)"))
    };
    (Placement { rotation, position }, note)
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
    /// Deleted (`deleteObject`, 3g-5): no collider, not clicked, not drawn.
    pub gone: bool,
}

impl ActiveDoor {
    /// Its collision settings now (3j).
    pub fn kind(&self) -> ColliderKind {
        ColliderKind { collision_type: self.collision_type, collision_mask: self.collision_mask, ..Default::default() }
    }

    /// Its collider where it stands now (the box turns with the door);
    /// `None` once deleted.
    pub fn collider(&self) -> Option<BoxCollider> {
        let (min, max) = self.bounds.filter(|_| !self.gone)?;
        Some(BoxCollider { name: self.door.name.clone(), min, max, placement: self.door.placement(), kind: self.kind() })
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
    /// Deleted (`deleteObject`, 3g-5).
    pub gone: bool,
}

impl ActiveChest {
    /// Its collision settings now (3j).
    pub fn kind(&self) -> ColliderKind {
        ColliderKind { collision_type: self.collision_type, collision_mask: self.collision_mask, ..Default::default() }
    }

    /// `None` once deleted.
    pub fn collider(&self) -> Option<BoxCollider> {
        let (min, max) = self.bounds.filter(|_| !self.gone)?;
        Some(BoxCollider { name: self.chest.name.clone(), min, max, placement: self.chest.placement(), kind: self.kind() })
    }

    pub fn collides_with_connor(&self) -> bool {
        self.bounds.is_some() && pair_collides(CONNOR_TYPE, CONNOR_MASK, self.collision_type, self.collision_mask)
    }
}

/// A clickable animated object with a script that is not a door, chest or
/// item (3e-5).
#[derive(Debug, Clone)]
pub struct ActiveObject {
    pub player: NamedAnimPlayer,
    pub script: Option<String>,
    pub bounds: Option<([f32; 3], [f32; 3])>,
    pub collision_type: u32,
    pub collision_mask: u32,
    pub accepts_inventory: Option<String>,
    pub accept_all_items: bool,
    pub too_far_away_action: i64,
    pub do_not_point_at: bool,
    pub hidden: bool,
    /// Deleted (`deleteObject`, 3f): never comes back.
    pub gone: bool,
    /// `setDeleteWhenDone <name> 1` on a `KQAnimOnce` (`anim-scripts.md`
    /// rules 11.3, 13): deleted when its movement ends.
    pub delete_when_done: bool,
}

impl ActiveObject {
    /// Its collision settings now (3j).
    pub fn kind(&self) -> ColliderKind {
        ColliderKind { collision_type: self.collision_type, collision_mask: self.collision_mask, ..Default::default() }
    }

    pub fn name(&self) -> &str {
        &self.player.name
    }

    pub fn collider(&self) -> Option<BoxCollider> {
        self.bounds.map(|(min, max)| BoxCollider { name: self.player.name.clone(), min, max, placement: self.player.placement(), kind: self.kind() })
    }

    pub fn collides_with_connor(&self) -> bool {
        self.bounds.is_some() && pair_collides(CONNOR_TYPE, CONNOR_MASK, self.collision_type, self.collision_mask)
    }
}

/// Any other animated object (M3 step 3i-4): the ones the viewer's
/// `anim_view` draws (`water`, the waterfall, torches, the windmill...).
/// Its movements follow `anim-scripts.md` rules 1–10 like the others;
/// clicking it is as before (a `Clickable` by the same name, if any).
#[derive(Debug, Clone)]
pub struct AnimObject {
    pub player: NamedAnimPlayer,
    /// `script=`: gets `EndLoop` and `Cue` when no clickable by that name
    /// has it.
    pub script: Option<String>,
    /// Deleted (`deleteObject`).
    pub gone: bool,
    /// A character's states (M4 4d-3, `characters.md`); `None` for other
    /// animated objects.
    pub character: Option<Box<crate::character::Character>>,
    /// `makeTranslucent` (characters.md rule 22): the see-through value a
    /// script set (256 = solid), for the viewer; `None` = as loaded.
    pub translucency: Option<f32>,
}

/// A `KQCueEvent`'s script arguments (`doors.md` rule 23): `Cue`, its first
/// two numbers, and the third only if it is not zero. A word there (the
/// movement name of a `getEndLoop` relay, `clicking.md` rule 21) counts as
/// not zero.
fn cue_args(data: &[String]) -> Vec<String> {
    let n = |k: usize| data.get(k).cloned().unwrap_or_else(|| "0".into());
    let mut args = vec!["Cue".to_string(), n(0), n(1)];
    if let Some(v) = data.get(2).filter(|v| !looks_like_number(v) || number_value(v) != 0.0) {
        args.push(v.clone());
    }
    args
}

/// `KQAnimPlayEvent <movement>` to an object's player (`anim-scripts.md`
/// rule 4), and its log line.
fn anim_request(player: &mut NamedAnimPlayer, data: &[String], what: &str, from: &str) -> String {
    let Some(m) = data.first() else {
        return format!("KQAnimPlayEvent to {what}{from}: no movement named; nothing");
    };
    let before = format!("current '{}' phase {:.4}", player.movement_name(), player.phase);
    let outcome = match player.request(m) {
        AnimRequest::AtOnce(s) => format!("starts at once: {}", s.describe(&player.movements)),
        AnimRequest::Waiting => "waiting (the running movement can't be cut short; starts when it ends)".to_string(),
        AnimRequest::WaitingForever => "waiting forever (rule 6: a stopped movement that can't be cut short never ends)".to_string(),
    };
    format!("KQAnimPlayEvent {m} to {what}{from}: {before}: {outcome}")
}

/// The line for a waiting movement that started in an update (rules 3, 8).
fn started_line(what: &str, s: &Started, movements: &[crate::door::DoorMovement], why: &str) -> String {
    format!("{what}: waiting movement starts ({why}): {}", s.describe(movements))
}

/// A building (`KQInterior`) as a click target (`clicking.md` rules 3, 4,
/// 10, 12). Its solid space is tested by the caller ([`ClickWalls`]).
#[derive(Debug, Clone)]
pub struct ActiveBuilding {
    pub name: String,
    /// `colliderType`, default 0x28 (clickable); 136 = not clickable.
    pub collision_type: u32,
    pub script: Option<String>,
    pub accepts_inventory: Option<String>,
    pub accept_all_items: bool,
    /// Default **1** for buildings (rule 10; other objects 0).
    pub too_far_away_action: i64,
    /// `doNotPointAt=1`: no turn after a click (`clicking.md` rule 36, Q105
    /// c; M4 4c-5).
    pub do_not_point_at: bool,
    /// Its `.dig` header box, in its own axes, and its placement: reach is
    /// measured to this box in the building's own axes (`clicking.md`
    /// rule 9, Q75; M1 7i).
    pub bounds: ([f32; 3], [f32; 3]),
    pub placement: Placement,
}

/// A building's `tooFarAwayAction` when its file gives none (`clicking.md`
/// rule 10).
pub const BUILDING_TOO_FAR_AWAY_ACTION: i64 = 1;

/// How far the click line reaches. `clicking.md` rule 1: a fixed length
/// from the camera's view, which setting is not identified (open question
/// 1); our stand-in: far beyond any world.
pub const CLICK_LINE_LENGTH: f32 = 1_000_000.0;

/// The clickable buildings' solid space, for the click line (the BSP line
/// test the camera uses, `clicking.md` rule 2). The caller tests only the
/// buildings whose type has bit 0x20 ([`Active::clickable_building`]).
pub trait ClickWalls {
    /// The nearest clickable building the line from `from` to `to` enters:
    /// its name and the distance from `from`.
    fn first_hit(&self, from: [f32; 3], to: [f32; 3]) -> Option<(String, f32)>;
}

/// What decides a click's script call (`doors.md` rules 12-14): the
/// object's name (for its accept list, `clicking.md` rules 13-18) and its
/// `tooFarAwayAction`.
struct ClickRules<'a> {
    name: &'a str,
    too_far_away_action: i64,
}

/// An accept list for the log.
fn describe_accepts(a: &Accepts) -> String {
    match (a.all, a.types.is_empty()) {
        (true, _) => "everything".into(),
        (false, true) => "nothing".into(),
        (false, false) => a.types.join(","),
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
    /// `script=` (3f): clicking it runs `ConnorAction`.
    pub script: Option<String>,
    pub accepts_inventory: Option<String>,
    pub accept_all_items: bool,
    pub too_far_away_action: i64,
    pub do_not_point_at: bool,
    /// Deleted (`deleteObject`, 3f).
    pub gone: bool,
}

impl Clickable {
    /// A clickable object with no script and the default click keys.
    pub fn plain(name: &str, class: &str, min: [f32; 3], max: [f32; 3], placement: Placement, collision_type: u32) -> Self {
        Clickable {
            name: name.into(),
            class: class.into(),
            min,
            max,
            placement,
            collision_type,
            script: None,
            accepts_inventory: None,
            accept_all_items: false,
            too_far_away_action: 0,
            do_not_point_at: false,
            gone: false,
        }
    }
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
    /// A picked building with no script: the call is dropped, nothing
    /// happens, Connor doesn't turn (`clicking.md` rule 10).
    Swallowed,
    /// A world item went into Connor's inventory (items.md rule 7).
    PickedUp(String),
    /// A chest's own click steps ran (chests.md rules 7-9).
    Chest { before: ChestState, after: ChestState },
}

/// What the click line picked (M3 step 3g-2, `clicking.md` rules 1-8).
#[derive(Debug, Clone, PartialEq)]
pub enum Pick {
    /// The nearest clickable box, `t` units along the line.
    Seen { name: String, t: f32 },
    /// A clickable building's wall, floor or ceiling, `t` units along the
    /// line, nearer than any clickable box; `behind`: the box it hides.
    Building { name: String, t: f32, behind: Option<(String, f32)> },
    /// Nothing clickable on the line.
    Nothing,
}

/// Ground height at (x, y): the floor (`true`) or the surface (`false`).
pub type Heights = Box<dyn Fn(f32, f32, bool) -> Option<f32> + Send + Sync>;

/// Everything a script can touch, apart from the variables.
pub struct WorldState {
    pub doors: Vec<ActiveDoor>,
    pub chests: Vec<ActiveChest>,
    pub objects: Vec<ActiveObject>,
    /// The other animated objects (3i-4), in the viewer's order.
    pub anims: Vec<AnimObject>,
    pub traps: Vec<Trap>,
    pub clickables: Vec<Clickable>,
    /// The buildings, in the caller's order (3g-2).
    pub buildings: Vec<ActiveBuilding>,
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
    /// Sounds to start (M3 step 3h-2, `sound.md`), taken by the caller.
    pub sounds: Vec<Start>,
    /// Music requests now due (M3 step 3h-5, `sound.md` rule 57f), taken
    /// by the caller.
    pub music: Vec<MusicRequest>,
    /// For z -1 / -2 of a new world item; `None`: kept as written.
    pub heights: Option<Heights>,
    /// Variables a command set, for the script runner.
    vars_out: Vec<(String, String)>,
    /// Each object's accept list by lower-case name, read when it enters
    /// the world (or when first asked) and added to by
    /// `KQObject::setAccepts` (3g-3).
    accepts: BTreeMap<String, Accepts>,
    /// `getEndLoop <from> <to> 1`: lower-case `<from>` → `<to>` (one
    /// listener per object, `clicking.md` rule 21).
    end_loop_listeners: BTreeMap<String, String>,
    /// Every deleted object, lower case (`deleteObject`, 3g-5): events to
    /// it are dropped.
    deleted: BTreeSet<String>,
    /// `KQObject::setHidden` on other placed objects (3k-2), by
    /// lower-case name: overrides their file's `hidden=`.
    plain_hidden: BTreeMap<String, bool>,
    /// Objects prepared for a script's `loadKQ` (3k-4): lower-case file
    /// name → the object's name and script.
    dormant: BTreeMap<String, (String, Option<String>)>,
    /// Names deleted during the event running now: free only once it has
    /// finished (`clicking.md` rule 22, 3l-7).
    deleted_this_event: BTreeSet<String>,
    /// Scripts a world command asked to run at once (`ScriptHost::take_calls`).
    calls_out: Vec<(String, Vec<String>)>,
    /// `setThirdPerson` (`camera.md` "First person" rule 7): the
    /// first-person key is locked, and the variable
    /// `…::SetThirdPerson::WasFirstPerson` (we are always in third person).
    first_person_locked: bool,
    was_first_person: bool,
    /// Connor's jump, set by the caller before scripts run (M3 3m-1,
    /// `anim-scripts.md` rules 28–29): the jump movement (`jump`,
    /// `runJump`, `backJump`) while he is in the Jump state, else `None`.
    pub connor_jump: Option<&'static str>,
    /// Connor's stats and level (`Connor.kq`; 3i-6).
    pub connor_stats: Option<ConnorStats>,
    /// How many stats events Connor has had (M4 4c-2): the bottom bar
    /// sends its values after each (`interface.md` rule 24).
    pub connor_stats_events: u64,
    /// The panel commands of scripts since the caller last looked (M4
    /// 4c-2, `interface.md` rule 15).
    pub gui_orders: Vec<GuiOrder>,
    /// Applied items with a duration and when they run out, game time
    /// (M4 4c-3, `inventory.md` rule 14).
    pub item_timers: Vec<(String, f64)>,
    /// Item descriptions to play (M4 4c-3, `talking.md` rule 20).
    pub item_voices: Vec<crate::connor_inventory::ItemVoice>,
    /// The item Connor holds as the pointer (M4 4c-4, `inventory.md`
    /// rules 9, 12): its object's name.
    pub hand: Option<String>,
    /// `goldCoinPersists 1` (`inventory.md` rule 12).
    pub gold_persists: bool,
    /// When Connor last said the scroll can't be taken (game time; rule 9).
    pub scroll_remark_at: Option<f64>,
    /// Other objects' stats by lower-case name (`stats=`, doors 10,000
    /// health without one; 3i-6).
    pub stats: BTreeMap<String, Stats>,
    /// Objects whose collider is out of the collision world now (group
    /// bit 0x8 lost: far, off or hidden; `object-lighting.md` rule 41),
    /// lower case: clicks pass through them (M1 step 7b).
    switched_off: BTreeSet<String>,
    /// Remarks and speech (M4 4a-2, `talking.md`): every character's
    /// queue of lines.
    pub talk: Talk,
    /// `KQMonster::stateChangeNotify <recipient> <issuer>` (rule 13):
    /// lower-case issuer → its recipients, in the order registered.
    notify: BTreeMap<String, Vec<String>>,
    /// Connor can turn to face a listener now: he is in Wait or Move
    /// (`talking.md` rule 7). Set by the caller before scripts run.
    pub connor_can_turn: bool,
    /// Where Connor stands, set by the caller before scripts run.
    pub connor_at: Option<[f32; 3]>,
    /// The world's `KQMonster` objects (`talking.md` rule 6).
    pub monsters: Vec<Monster>,
    /// Characters near or far now (lower case), set by the caller before
    /// each update (`characters.md` rule 11).
    pub characters_in_range: BTreeSet<String>,
    /// Lines for `characters.log` (M4 4d-3), taken by the caller.
    pub character_log: Vec<String>,
}

/// A placed `KQMonster` (M4 4a-2): monsters don't move or fight yet (M5);
/// for speech only their name, place and script.
#[derive(Debug, Clone, PartialEq)]
pub struct Monster {
    pub name: String,
    pub position: [f32; 3],
    pub script: Option<String>,
}

impl WorldState {
    pub fn now(&self) -> f64 {
        self.updates as f64 / crate::UPDATES_PER_SECOND
    }

    /// A line about a character: the current log and `characters.log`.
    pub(crate) fn character_note(&mut self, who: &str, line: impl AsRef<str>) {
        let line = format!("{who}: {}", line.as_ref());
        self.character_log.push(format!("update {}: {line}", self.updates));
        self.note(line);
    }

    /// The character called `name` (not deleted): its index in `anims`.
    pub fn character_index(&self, name: &str) -> Option<usize> {
        self.anims.iter().position(|a| !a.gone && a.character.is_some() && a.player.name.eq_ignore_ascii_case(name))
    }

    pub(crate) fn note(&mut self, line: impl AsRef<str>) {
        let line = format!("update {}: {}", self.updates, line.as_ref());
        if self.in_trap {
            self.trap_log.push(line);
        } else {
            self.log.push(line);
        }
    }

    /// A line about items: `items.log` and the current log.
    pub(crate) fn item_note(&mut self, line: impl AsRef<str>) {
        self.item_log.push(format!("update {}: {}", self.updates, line.as_ref()));
        self.note(line);
    }

    pub fn anim_index(&self, name: &str) -> Option<usize> {
        self.anims.iter().position(|a| a.player.name.eq_ignore_ascii_case(name))
    }

    pub fn object_index(&self, name: &str) -> Option<usize> {
        self.objects.iter().position(|o| o.name().eq_ignore_ascii_case(name))
    }

    pub fn chest_index(&self, name: &str) -> Option<usize> {
        self.chests.iter().position(|c| c.chest.name.eq_ignore_ascii_case(name))
    }

    /// Makes a world item at `position` (z -1 / -2: floor / surface,
    /// `objects.md` Q17 rule 2) and returns its name.
    /// What Connor's inventory plays when an item of type `kind` goes in
    /// (items.md rule 7, Q61: every add, not placed in the world): a plain
    /// one-shot in group 0 (`sound.md` rule 42).
    fn pickup_sound(&mut self, kind: &str) -> String {
        let sound = self.items.kind(kind).and_then(|t| t.pickup_sound.clone());
        if let Some(name) = &sound {
            self.sounds.push(Start::plain_one_shot(name, format!("pickup of {kind}")));
        }
        format!("pickup sound {}", sound.as_deref().unwrap_or("none"))
    }

    /// A `playsound` cue of an animated object at `position` (`sound.md`
    /// rule 35): a placed one-shot, 5,000 / 25,000, group 0. Returns what
    /// the log line says about it.
    fn cue_sound(&mut self, what: &str, cue: &str, position: [f32; 3]) -> &'static str {
        let mut words = cue.split_whitespace();
        if !words.next().is_some_and(|w| w.eq_ignore_ascii_case("playsound")) {
            return "cue action not done";
        }
        match words.next() {
            Some(name) => {
                self.sounds.push(Start::placed_one_shot(name, position, format!("{what} cue")));
                "sound started"
            }
            None => "playsound without a file name; nothing",
        }
    }

    /// `KQSound::play <file>` (`sound.md` rule 40): a plain one-shot in
    /// group 0 (group 1 only while the game is paused; there is no pause
    /// yet).
    fn play_sound(&mut self, words: &[String], at: &str) -> String {
        match words.get(1) {
            Some(name) => {
                self.sounds.push(Start::plain_one_shot(name, format!("KQSound::play, {at}")));
                self.note(format!("{at}: KQSound::play {name}: plain sound started"));
                "0".into()
            }
            None => {
                self.note(format!("{at}: KQSound::play without a file name; nothing"));
                "1".into()
            }
        }
    }

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
                let sound = if m.who.eq_ignore_ascii_case("Connor") && !m.in_world() { format!("; {}", self.pickup_sound(&m.kind)) } else { String::new() };
                let line = format!("{at}: made {}; NewInvItem = {name}{sound}", self.items.describe(&name));
                self.vars_out.push(("NewInvItem".into(), name.clone()));
                self.item_note(line);
                // M4 4c-3: Connor's inventory rules (inventory.md rule 8).
                if m.who.eq_ignore_ascii_case("Connor") && !m.in_world() {
                    crate::connor_inventory::receive(self, &name, None, &format!("{at}: {}", words[0]));
                }
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
            if command == "KQObject::setHidden" {
                // Any other placed object (3k-2, objects.md "Hidden
                // objects"): the draw-distance test makes it not drawn and
                // not touched (its collider leaves the collision world).
                if !self.known.contains(&name.to_ascii_lowercase()) {
                    self.note(format!("{at}: setHidden: unknown object '{name}'; nothing"));
                    return "1".into();
                }
                let hidden = words.get(2).is_some_and(|v| looks_like_number(v) && number_value(v) != 0.0);
                self.plain_hidden.insert(name.to_ascii_lowercase(), hidden);
                self.note(format!("{at}: setHidden {name}: placed object hidden {hidden} (not drawn, not touched, from the next frame)"));
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

    /// The `acceptsInventory` / `acceptAllItems` values an object was
    /// loaded with (doors, scripted objects, clickables, buildings; chests
    /// and items have none).
    fn accept_keys(&self, name: &str) -> Option<(Option<String>, bool)> {
        let same = |n: &str| n.eq_ignore_ascii_case(name);
        if let Some(d) = self.doors.iter().find(|d| same(&d.door.name)) {
            return Some((d.accepts_inventory.clone(), d.accept_all_items));
        }
        if let Some(o) = self.objects.iter().find(|o| same(o.name())) {
            return Some((o.accepts_inventory.clone(), o.accept_all_items));
        }
        if let Some(c) = self.clickables.iter().find(|c| same(&c.name)) {
            return Some((c.accepts_inventory.clone(), c.accept_all_items));
        }
        self.buildings.iter().find(|b| same(&b.name)).map(|b| (b.accepts_inventory.clone(), b.accept_all_items))
    }

    /// Object `name`'s accept list (`clicking.md` rules 13-15), read from
    /// its keys the first time (normally when it enters the world).
    fn accepts_entry(&mut self, name: &str) -> &mut Accepts {
        let key = name.to_ascii_lowercase();
        if !self.accepts.contains_key(&key) {
            let mut a = Accepts::default();
            if let Some((list, all)) = self.accept_keys(name) {
                a.all = all;
                let unknown = list.as_deref().map_or_else(Vec::new, |v| a.read(v, &self.items.types));
                if !unknown.is_empty() {
                    self.note(format!("{name}: acceptsInventory names no item type {unknown:?}; skipped (the original would crash)"));
                }
            }
            self.accepts.insert(key.clone(), a);
        }
        self.accepts.get_mut(&key).expect("just inserted")
    }

    /// A copy of object `name`'s accept list.
    fn accepts_of(&mut self, name: &str) -> Accepts {
        self.accepts_entry(name).clone()
    }

    /// `KQObject::setAccepts <object> <value>` (`clicking.md` rule 16):
    /// adds to what the object accepts.
    fn set_accepts(&mut self, words: &[String], at: &str) -> String {
        let [_, name, value @ ..] = words else {
            self.note(format!("{at}: usage: KQObject::setAccepts <object> <value>: '{}'", words.join(" ")));
            return "1".into();
        };
        if value.is_empty() || !self.known.contains(&name.to_ascii_lowercase()) {
            self.note(format!("{at}: ERROR KQObject::setAccepts: unknown object or no value: '{}'", words.join(" ")));
            return "1".into();
        }
        let types = self.items.types.clone();
        let entry = self.accepts_entry(name);
        let unknown = entry.read(&value.join(" "), &types);
        let now = describe_accepts(entry);
        let skipped = if unknown.is_empty() { String::new() } else { format!("; skipped unknown {unknown:?}") };
        self.note(format!("{at}: KQObject::setAccepts {name} {}: now accepts {now}{skipped}", value.join(" ")));
        "0".into()
    }

    /// `getEndLoop <from> <to> <1|0>` (`clicking.md` rule 21): with 1, `<to>`
    /// is told whenever one of `<from>`'s play-once movements ends; 0 stops
    /// it. One listener per object; a new one replaces the old (inferred
    /// by A from the usage text).
    fn get_end_loop(&mut self, words: &[String], at: &str) -> String {
        let [_, from, to, on] = words else {
            self.note(format!("{at}: usage: getEndLoop <from> <to> <1|0>: '{}'", words.join(" ")));
            return "1".into();
        };
        for n in [from, to] {
            if !self.known.contains(&n.to_ascii_lowercase()) {
                self.note(format!("{at}: ERROR getEndLoop: unknown object '{n}'"));
                return "1".into();
            }
        }
        let key = from.to_ascii_lowercase();
        if number_value(on) != 0.0 {
            self.note(format!("{at}: getEndLoop: {to} hears when {from}'s play-once movements end"));
            self.end_loop_listeners.insert(key, to.clone());
        } else {
            self.note(format!("{at}: getEndLoop: {from} no longer reports its movements' ends"));
            self.end_loop_listeners.remove(&key);
        }
        "0".into()
    }

    /// `KQCamera::follow <name>` (`camera.md` rule 1): Connor, or `none`
    /// (the camera stays where it is). Following another object is not
    /// done (logged; no Daventry script does it while playing).
    fn camera_follow(&mut self, words: &[String], at: &str) -> String {
        let Some(who) = words.get(1) else {
            self.note(format!("{at}: usage: KQCamera::follow <object|none>"));
            return "1".into();
        };
        let connor = if who.eq_ignore_ascii_case("Connor") {
            true
        } else if who.eq_ignore_ascii_case("none") {
            false
        } else {
            self.note(format!("{at}: KQCamera::follow {who}: following another object is not done; nothing"));
            return "1".into();
        };
        self.note(format!("{at}: KQCamera::follow {who}: {}", if connor { "the camera follows Connor" } else { "the camera stays where it is (camera.md rule 1)" }));
        self.requests.push(Request::CameraFollow { connor, at: at.to_string() });
        "0".into()
    }

    /// `setThirdPerson [1]` (`camera.md` "First person" rule 7), a toggle
    /// cutscenes call in pairs. We have no first person, so we are always
    /// in third person: with the variable at 1 (never, here) it would go
    /// back to first person; otherwise the first-person key's lock flips
    /// (`setThirdPerson 1` also sets the variable to 0). Camera place,
    /// distance and tilt are not touched.
    fn set_third_person(&mut self, words: &[String], at: &str) -> String {
        if self.was_first_person {
            self.was_first_person = false;
            self.first_person_locked = false;
            self.note(format!("{at}: setThirdPerson: WasFirstPerson 1: key unlocked, back to first person (we have no first person; logged only)"));
            return "0".into();
        }
        self.first_person_locked = !self.first_person_locked;
        let one = if words.get(1).is_some_and(|w| w == "1") { " (setThirdPerson 1: variable set to 0)" } else { "" };
        self.note(format!(
            "{at}: setThirdPerson: third person, WasFirstPerson 0: the first-person key is now {}{one} (we have no first person; camera untouched)",
            if self.first_person_locked { "locked" } else { "allowed" }
        ));
        "0".into()
    }

    /// The camera's spot list, flights and cuts (`camera.md` "Camera
    /// flights and cuts" rules 1, 3, 4): requests for the viewer's camera.
    /// `addCamLoc x y z [newlist]`; `flyToCamLoc x y z [seconds [endRoll]]`
    /// (`endRoll` only read with exactly those five numbers);
    /// `cutToCamLoc x y z [roll]`.
    fn cam_loc(&mut self, words: &[String], at: &str) -> String {
        let n = words.len() - 1;
        if n < 3 || !words[1..].iter().all(|w| looks_like_number(w)) {
            self.note(format!("{at}: usage: {} x y z …: '{}'", words[0], words.join(" ")));
            return "1".into();
        }
        let v = |i: usize| words.get(i).map(|w| number_value(w) as f32);
        let p = [v(1).unwrap_or(0.0), v(2).unwrap_or(0.0), v(3).unwrap_or(0.0)];
        let at_s = at.to_string();
        let (line, request) = match words[0].as_str() {
            "KQCamera::addCamLoc" => {
                let newlist = v(4).is_some_and(|x| x != 0.0);
                (format!("spot ({:.0}, {:.0}, {:.0}){}", p[0], p[1], p[2], if newlist { ", a new list" } else { " added to the list" }), Request::CamSpot { spot: p, newlist, at: at_s })
            }
            "KQCamera::flyToCamLoc" => {
                let seconds = v(4);
                let end_roll = if n == 5 { v(5) } else { None };
                (
                    format!("fly to the nearest spot, aiming at ({:.0}, {:.0}, {:.0}), {}, end roll {:.3}", p[0], p[1], p[2], seconds.map_or("no seconds (5,000 per second)".into(), |s| format!("{s} s")), end_roll.unwrap_or(0.0)),
                    Request::CamFly { aim: p, seconds, end_roll, at: at_s },
                )
            }
            _ => {
                let roll = v(4).unwrap_or(0.0);
                (format!("cut to the nearest spot, aiming at ({:.0}, {:.0}, {:.0}), roll {roll:.3}", p[0], p[1], p[2]), Request::CamCut { aim: p, roll, at: at_s })
            }
        };
        self.note(format!("{at}: {}: {line}", words[0]));
        self.requests.push(request);
        "0".into()
    }

    /// `KQConner::JumpDir` (`anim-scripts.md` rule 29): 1 in the Jump
    /// state with `backJump`, 0 with `jump` or `runJump`; outside a jump 0
    /// and the usage text on the console.
    fn jump_dir(&mut self, at: &str) -> String {
        match self.connor_jump {
            Some(jump) => {
                let back = jump.eq_ignore_ascii_case("backJump");
                self.note(format!("{at}: KQConner::JumpDir: {} (Connor is in the Jump state with {jump})", u8::from(back)));
                u8::from(back).to_string()
            }
            None => {
                self.note(format!("{at}: KQConner::JumpDir: 0; console: \"returns 0 if forward, 1 if backward\" (Connor is not in the Jump state)"));
                "0".into()
            }
        }
    }

    /// `KQConner::endJump` (`anim-scripts.md` rule 28): in the Jump state a
    /// request that finishes the jump (gravity on, steps 325, cut short;
    /// no state change), result 1; otherwise the usage text on the console
    /// and 0.
    fn end_jump(&mut self, at: &str) -> String {
        match self.connor_jump {
            Some(jump) => {
                self.note(format!("{at}: KQConner::endJump: Connor's {jump} is finished (gravity on, steps 325, cut short; he stays in the Jump state)"));
                self.requests.push(Request::EndJump { at: at.to_string() });
                "1".into()
            }
            None => {
                self.note(format!("{at}: KQConner::endJump: 0; console: \"use when Connor is in the jump state to get him out of it\" (he is not in it)"));
                "0".into()
            }
        }
    }

    /// `KQConner::walkTo x y [<who> <c> <d>]`, `KQConner::runTo …` and
    /// `pointAndCue x y z [<who> <c> <d>]` (`anim-scripts.md` rules
    /// 24–25): a request for Connor. Without `<who>` no cue is sent
    /// (`walkTo x y`, 6 lines in the data); a missing `<c>` or `<d>` is 0.
    fn connor_auto(&mut self, words: &[String], at: &str) -> String {
        use crate::connor_auto::{AutoKind, AutoMove, Cue};
        let point = words[0] == "pointAndCue";
        let numbers = if point { 3 } else { 2 };
        let ok = words.len() > numbers && words[1..=numbers].iter().all(|w| looks_like_number(w));
        if !ok {
            self.note(format!("{at}: usage: {} x y{} [<who> <cue> <data>]: '{}'", words[0], if point { " z" } else { "" }, words.join(" ")));
            return "1".into();
        }
        let target = [number_value(&words[1]) as f32, number_value(&words[2]) as f32];
        let kind = if point { AutoKind::Point } else { AutoKind::Walk { run: words[0] == "KQConner::runTo" } };
        let rest = &words[numbers + 1..];
        // `walkTo` / `runTo` with an object but not both cue numbers (4 or
        // 5 words): a console line, the walk, no cue (rule 24, Q94 d).
        if !point && (1..3).contains(&rest.len()) {
            self.note(format!("{at}: {}: console: \"must have objName...\" (an object without both cue numbers): the walk happens, no cue", words[0]));
        }
        let cue = rest.first().filter(|_| point || rest.len() >= 3).map(|who| Cue {
            who: who.clone(),
            c: rest.get(1).cloned().unwrap_or_else(|| "0".into()),
            d: rest.get(2).cloned().unwrap_or_else(|| "0".into()),
        });
        let then = cue.as_ref().map_or("no cue (no object named)".to_string(), |c| format!("then KQCueEvent {} {} to {}", c.c, c.d, c.who));
        self.note(format!("{at}: {}: Connor {} ({:.0}, {:.0}), {then}", words[0], if point { "turns to face" } else { "turns, then goes straight ahead to" }, target[0], target[1]));
        self.requests.push(Request::ConnorAuto { auto: AutoMove::new(kind, target, cue), at: at.to_string() });
        "0".into()
    }

    /// `loadKQ <file>` while playing (`objects.md` "Where a world's objects
    /// come from" rule 6, Q90): a file prepared at the start comes into the
    /// world as at the start (a sound starts at once) and its script gets
    /// `Init` at once, during this line. A name still used by a live
    /// object, or deleted during this same event (`clicking.md` rule 22):
    /// the original stops with a fatal error; we log an error and refuse
    /// the load. With a name or a position on the line, or a file not
    /// prepared: not done (logged).
    fn load_kq(&mut self, words: &[String], at: &str) -> String {
        let Some(file) = words.get(1) else {
            self.note(format!("{at}: usage: loadKQ <file> [name [x y z [rx ry rz]]]"));
            return "1".into();
        };
        if words.len() > 2 {
            self.note(format!("{at}: loadKQ {}: a name or position while playing: not done yet", words[1..].join(" ")));
            return "0".into();
        }
        let key = file.rsplit(['/', '\\']).next().unwrap_or(file).to_ascii_lowercase();
        let Some((name, script)) = self.dormant.get(&key).cloned() else {
            self.note(format!("{at}: loadKQ {file}: no copy prepared at the start (already loaded, or not found): not done"));
            return "0".into();
        };
        let lower = name.to_ascii_lowercase();
        let live = self.known.contains(&lower) && !self.deleted.contains(&lower);
        if live || self.deleted_this_event.contains(&lower) {
            let why = if live { "a live object already has that name" } else { "its name was deleted in this same event and is not free yet" };
            self.note(format!("{at}: ERROR loadKQ {file}: {why} ({name}): the original stops with a fatal error; load refused"));
            return "1".into();
        }
        self.dormant.remove(&key);
        self.known.insert(lower);
        self.note(format!("{at}: loadKQ {file}: {name} comes into the world (as at the start){}", if script.is_some() { "; its script gets Init now" } else { "; no script" }));
        self.requests.push(Request::Load { file: key, name: name.clone(), at: at.to_string() });
        if let Some(script) = script {
            self.calls_out.push((script, vec![name, "Init".into()]));
        }
        "0".into()
    }

    /// `goldCoinPersists 0|1` (`inventory.md` rule 12, M4 4c-4): while on,
    /// coins accepted from the hand stay there; 0 also empties the hand at
    /// once.
    fn gold_coin_persists(&mut self, words: &[String], at: &str) -> String {
        let Some(v) = words.get(1).filter(|v| looks_like_number(v)) else {
            self.note(format!("{at}: usage: goldCoinPersists 0|1; nothing"));
            return "1".into();
        };
        self.gold_persists = number_value(v) != 0.0;
        let hand = if self.gold_persists {
            String::new()
        } else {
            match self.hand.take() {
                Some(h) => format!("; the hand ({h}) emptied, the play pointer back"),
                None => "; the hand was empty".into(),
            }
        };
        self.item_note(format!("{at}: goldCoinPersists {}{hand} (inventory.md rule 12)", u8::from(self.gold_persists)));
        "0".into()
    }

    /// `setLowerGui`, `setUpperGui`, `setMapGui` 0|1 and `getGuiStatus`
    /// (`interface.md` rule 15, M4 4c-2): orders for the panels, carried
    /// out by the caller.
    fn gui_command(&mut self, words: &[String], at: &str) -> String {
        use crate::panels::Group;
        let group = match words[0].as_str() {
            "setLowerGui" => Group::Bottom,
            "setUpperGui" => Group::Top,
            "setMapGui" => Group::Map,
            _ => {
                self.note(format!("{at}: getGuiStatus: the panels' groups are written to gui.log"));
                self.gui_orders.push(GuiOrder::Status { at: at.to_string() });
                return "0".into();
            }
        };
        let Some(v) = words.get(1).filter(|v| looks_like_number(v)) else {
            self.note(format!("{at}: usage: {} 0|1; nothing", words[0]));
            return "1".into();
        };
        let on = number_value(v) != 0.0;
        self.note(format!("{at}: {} {}: the {} {} (interface.md rule 15)", words[0], v, group.name(), if on { "slides in" } else { "slides out" }));
        self.gui_orders.push(GuiOrder::Set { group, on, at: at.to_string() });
        "0".into()
    }

    /// `handsOff 1|0`: not in any note (Q89 a). Stand-in: the player's
    /// movement keys, Jump and clicks are ignored while on.
    fn hands_off(&mut self, words: &[String], at: &str) -> String {
        let on = words.get(1).is_some_and(|v| looks_like_number(v) && number_value(v) != 0.0);
        // N, the "number of dropable frames" (rule 21): logged only.
        let frames = words.get(2).filter(|_| on).map_or(String::new(), |n| format!("; at most ({n} + 1) / 15 s of game time per drawn frame (logged only)"));
        self.note(format!(
            "{at}: handsOff {}: the player's keys, mouse look and clicks {} (anim-scripts.md rules 21-22){frames}",
            if on { 1 } else { 0 },
            if on { "ignored" } else { "work again; held movement keys sent again" }
        ));
        self.requests.push(Request::HandsOff { on, at: at.to_string() });
        "0".into()
    }

    /// `move <who> <flags> x y z [rx ry rz]` (`objects.md` "(d)"): set
    /// directly, without collision passes. Doors, chests, scripted and
    /// animated objects are moved here; Connor, the camera and plain
    /// placed objects by the caller (a request). Flags above 3: not moved.
    fn move_command(&mut self, words: &[String], at: &str) -> String {
        let (who, flags, position, turn) = match kq8_data::world_script::parse_move(&words[1..]) {
            Ok(m) => m,
            Err(e) => {
                self.note(format!("{at}: move: {e}: '{}'", words.join(" ")));
                return "1".into();
            }
        };
        if flags > 3 {
            self.note(format!("{at}: move {who}: flags {flags} unknown; not moved"));
            return "1".into();
        }
        let base = if let Some(i) = self.object_index(&who) {
            Some(&mut self.objects[i].player.base)
        } else if let Some(i) = self.anim_index(&who) {
            Some(&mut self.anims[i].player.base)
        } else if let Some(i) = self.chest_index(&who) {
            Some(&mut self.chests[i].chest.player.base)
        } else if let Some(i) = self.door_index(&who) {
            Some(&mut self.doors[i].door.player.base)
        } else {
            None
        };
        if let Some(base) = base {
            let old = base.position;
            let (new, note) = moved_placement(*base, flags, position, turn);
            *base = new;
            self.note(format!(
                "{at}: move {who} {flags}: ({:.0}, {:.0}, {:.0}) -> ({:.0}, {:.0}, {:.0}){}",
                old[0],
                old[1],
                old[2],
                new.position[0],
                new.position[1],
                new.position[2],
                note.map_or(String::new(), |n| format!("; {n}"))
            ));
            return "0".into();
        }
        let known = who.eq_ignore_ascii_case("Connor") || who.eq_ignore_ascii_case("KQCamera") || self.known.contains(&who.to_ascii_lowercase());
        if !known {
            self.note(format!("{at}: move: unknown object '{who}'; not moved"));
            return "1".into();
        }
        self.note(format!("{at}: move {who} {flags} {position:?} {turn:?}: handed to the viewer"));
        self.requests.push(Request::Move { who, flags, position, turn, at: at.to_string() });
        "0".into()
    }

    /// `KQMonster::setState <who> <state> [list]`: Connor's Special state
    /// is handed to the caller (`anim-scripts.md` rule 16); other states
    /// and other characters are later milestones (logged only, `0` as the
    /// stubs).
    fn set_state(&mut self, words: &[String], at: &str) -> String {
        match words {
            [_, who, state, items @ ..] if who.eq_ignore_ascii_case("Connor") && state.eq_ignore_ascii_case("special") => {
                // One word in the data; several are joined as more items.
                let items = items.join(",");
                self.note(format!("{at}: KQMonster::setState Connor special {items}: handed to Connor (rule 16)"));
                self.requests.push(Request::ConnorSpecial { items, at: at.to_string() });
            }
            [_, who, state, rest @ ..] if self.character_index(who).is_some() => {
                // characters.md rule 22.
                let i = self.character_index(who).unwrap();
                let a = &mut self.anims[i];
                let line = a.character.as_mut().unwrap().set_state(&mut a.player, state, rest);
                let name = a.player.name.clone();
                self.character_note(&name, format!("{at}: KQMonster::setState {state} {}: {line}", rest.join(" ")));
            }
            _ => self.note(format!("{at}: stub (later milestone, nothing done): {}", words.join(" "))),
        }
        "0".into()
    }

    /// `makeTranslucent <name> <0–256>` (`characters.md` rule 22): the
    /// see-through value, at once, no fade; 256 = solid (`objects.md`
    /// "See-through objects" rule 2). Built for animated objects and
    /// characters (the only Daventry use is the Wizard).
    fn make_translucent(&mut self, words: &[String], at: &str) -> String {
        let (Some(who), Some(value)) = (words.get(1), words.get(2).filter(|v| looks_like_number(v))) else {
            self.note(format!("{at}: usage: makeTranslucent <name> <0-256>; nothing"));
            return "1".into();
        };
        let n = number_value(value) as f32;
        match self.anims.iter_mut().position(|a| !a.gone && a.player.name.eq_ignore_ascii_case(who)) {
            Some(i) => {
                self.anims[i].translucency = Some(n);
                let name = self.anims[i].player.name.clone();
                let what = if n >= 256.0 { "solid".to_string() } else { format!("opacity {:.3}", (n / 255.0).clamp(0.0, 1.0)) };
                let line = format!("{at}: makeTranslucent {name} {n}: {what}, at once (rule 22)");
                if self.anims[i].character.is_some() {
                    self.character_note(&name, line);
                } else {
                    self.note(line);
                }
                "0".into()
            }
            None => {
                self.note(format!("{at}: makeTranslucent {who} {n}: not an animated object or character of ours: not done"));
                "1".into()
            }
        }
    }

    /// `KQObject::setUseGravity`, `KQMonster::setMonUseGravity`,
    /// `KQMonster::setCanTurn <name> 0|1` (characters.md rules 8, 22, 23):
    /// kept on the character for the steps that use them (4d-4, 4d-6).
    /// `KQMonster::changeAnmFile <name> <file>`: logged only (rule 22:
    /// traced only as far as "taken at the next end of a movement").
    fn character_switch(&mut self, words: &[String], at: &str) -> String {
        let (Some(who), Some(value)) = (words.get(1), words.get(2)) else {
            self.note(format!("{at}: usage: {} <name> <value>; nothing", words[0]));
            return "1".into();
        };
        let Some(i) = self.character_index(who) else {
            self.note(format!("{at}: {} {who}: not a character; nothing", words[0]));
            return "1".into();
        };
        let on = number_value(value) != 0.0;
        let c = self.anims[i].character.as_mut().unwrap();
        let line = match words[0].as_str() {
            "KQMonster::setCanTurn" => {
                c.no_turn = !on;
                format!("setCanTurn {}: noTurn now {} (used from 4d-4)", u8::from(on), u8::from(c.no_turn))
            }
            "KQMonster::changeAnmFile" => format!("changeAnmFile {value}: not done (not traced far, open question 4)"),
            other => {
                c.gravity = on;
                format!("{other} {}: gravity {} (used from 4d-6)", u8::from(on), if on { "on" } else { "off" })
            }
        };
        let name = self.anims[i].player.name.clone();
        self.character_note(&name, format!("{at}: {line}"));
        "0".into()
    }

    /// The current `colliderType` of a door, chest, scripted object or
    /// clickable object.
    pub fn collision_type_of(&self, name: &str) -> Option<u32> {
        if let Some(i) = self.object_index(name) {
            return Some(self.objects[i].collision_type);
        }
        if let Some(i) = self.door_index(name) {
            return Some(self.doors[i].collision_type);
        }
        if let Some(i) = self.chest_index(name) {
            return Some(self.chests[i].collision_type);
        }
        self.clickables.iter().find(|c| c.name.eq_ignore_ascii_case(name)).map(|c| c.collision_type)
    }

    /// The movement player of the door, chest or scripted object `name`
    /// (not deleted).
    fn player_mut(&mut self, name: &str) -> Option<&mut NamedAnimPlayer> {
        if self.is_gone(name) {
            return None;
        }
        if let Some(i) = self.object_index(name) {
            return Some(&mut self.objects[i].player);
        }
        if let Some(i) = self.chest_index(name) {
            return Some(&mut self.chests[i].chest.player);
        }
        if let Some(i) = self.anim_index(name) {
            return Some(&mut self.anims[i].player);
        }
        let i = self.door_index(name)?;
        Some(&mut self.doors[i].door.player)
    }

    /// `pauseAnim <name>` toggles the pause; `setAnimPos <name> <phase>`
    /// sets the phase (`anim-scripts.md` rules 14, 15). Both print
    /// `<name>, <phase>`. Return values and the usage line are ours (the
    /// note gives none): 0 done, 1 failed.
    fn anim_pos(&mut self, words: &[String], at: &str) -> String {
        let set = words[0] == "setAnimPos";
        let (name, phase) = match (set, words) {
            (false, [_, name]) => (name, None),
            (true, [_, name, phase]) => (name, Some(number_value(phase) as f32)),
            _ => {
                let usage = if set { "setAnimPos <object> <phase 0-1>" } else { "pauseAnim <object>" };
                self.note(format!("{at}: usage: {usage}: '{}'", words.join(" ")));
                return "1".into();
            }
        };
        let Some(p) = self.player_mut(name) else {
            // The note's line, with its two spaces.
            self.note(format!("{at}: {}:  could not find anim {name}", words[0]));
            return "1".into();
        };
        let what = match phase {
            Some(v) => {
                p.phase = v;
                "phase set (rule 15)"
            }
            None => {
                p.paused = !p.paused;
                if p.paused { "paused (rule 14)" } else { "resumed (rule 14)" }
            }
        };
        let line = format!("{name}, {}", p.phase);
        self.note(format!("{at}: {}: {line}: {what}", words[0]));
        "0".into()
    }

    /// `setDeleteWhenDone <name> 1|0` (`anim-scripts.md` rule 13): only
    /// for `KQAnimOnce` objects. The usage line's wording is ours (the
    /// note gives none).
    fn set_delete_when_done(&mut self, words: &[String], at: &str) -> String {
        let [_, name, on] = words else {
            self.note(format!("{at}: usage: setDeleteWhenDone <object> <1|0>: '{}'", words.join(" ")));
            return "1".into();
        };
        let Some(i) = self.object_index(name).filter(|&i| self.objects[i].player.once && !self.objects[i].gone) else {
            self.note(format!("{at}: setDeleteWhenDone: could not find object '{name}'"));
            return "1".into();
        };
        let on = number_value(on) != 0.0;
        self.objects[i].delete_when_done = on;
        self.note(format!("{at}: setDeleteWhenDone {name}: {}", if on { "deleted when its movement ends" } else { "stays when its movement ends" }));
        "0".into()
    }

    /// A play-once movement of `name` ended (`clicking.md` rule 21): the
    /// object cues itself (3333 = `EndLoop`, through the queue) if it has a
    /// script, and its `getEndLoop` listener gets `Cue 300 0 <movement>`.
    fn movement_ended(&mut self, name: &str, movement: &str, has_script: bool) {
        self.report_end(name, movement, has_script);
    }

    /// The end of a movement: `EndLoop` to the object itself if `end_loop`
    /// (it has a script and is not a `KQAnimOnce`), and `Cue 300` to its
    /// `getEndLoop` listener.
    fn report_end(&mut self, name: &str, movement: &str, end_loop: bool) {
        let has_script = end_loop;
        let now = self.now();
        let at = format!("{name}: '{movement}' ended");
        if has_script {
            self.queue.send(now, 0.0, name, "KQCueEvent", &[CUE_END_LOOP.into(), "0".into(), movement.into()], &at);
            self.note(format!("{name}: movement '{movement}' ended: cues itself {CUE_END_LOOP} (EndLoop {movement}, through the queue)"));
        }
        if let Some(to) = self.end_loop_listeners.get(&name.to_ascii_lowercase()).cloned() {
            self.queue.send(now, 0.0, &to, "KQCueEvent", &[CUE_HEARD_END.into(), "0".into(), movement.into()], &at);
            self.note(format!("{name}: movement '{movement}' ended: tells {to} (getEndLoop: Cue {CUE_HEARD_END} 0 {movement})"));
        }
    }

    /// `deleteObject <name…>` (`clicking.md` rule 22): each named object
    /// is deleted at once: no longer clicked, collided with or drawn, and
    /// events to it are dropped. `0` when something was deleted; an unknown
    /// name prints an error (and leaves `1` if nothing was deleted). The
    /// original stops after the first name that exists; we delete every
    /// listed one ("Do not reproduce"; no data passes more than one).
    fn delete_objects(&mut self, words: &[String], at: &str) -> String {
        if words.len() < 2 {
            self.note(format!("{at}: usage: deleteObject <object>"));
            return "1".into();
        }
        let mut any = false;
        for name in &words[1..] {
            if self.delete_one(name, at) {
                any = true;
            } else {
                self.note(format!("{at}: deleteObject: Object \"{name}\" doesn't exist"));
            }
        }
        if any { "0" } else { "1" }.into()
    }

    /// Deletes one object; `false` if there is no such object (or it was
    /// already deleted).
    fn delete_one(&mut self, name: &str, at: &str) -> bool {
        let key = name.to_ascii_lowercase();
        if self.deleted.contains(&key) {
            return false;
        }
        let same = |n: &str| n.eq_ignore_ascii_case(name);
        // An animated object stops playing; the rest as for its kind.
        if let Some(i) = self.anim_index(name) {
            self.anims[i].gone = true;
        }
        let what = if let Some(i) = self.items.index(name) {
            if self.items.objects[i].place == Place::Gone {
                return false;
            }
            self.items.objects[i].place = Place::Gone;
            let line = format!("{at}: deleteObject {name}: item gone");
            self.item_note(line);
            self.deleted_this_event.insert(key.clone());
            self.deleted.insert(key);
            return true;
        } else if let Some(i) = self.object_index(name) {
            let o = &mut self.objects[i];
            (o.gone, o.hidden) = (true, true);
            "scripted object"
        } else if let Some(c) = self.clickables.iter_mut().find(|c| same(&c.name)) {
            c.gone = true;
            "clickable object"
        } else if let Some(i) = self.door_index(name) {
            self.doors[i].gone = true;
            "door"
        } else if let Some(i) = self.chest_index(name) {
            let c = &mut self.chests[i];
            (c.gone, c.hidden) = (true, true);
            "chest"
        } else if let Some(i) = self.trap_index(name) {
            self.traps[i].gone = true;
            "trap"
        } else if let Some(b) = self.buildings.iter_mut().find(|b| same(&b.name)) {
            b.collision_type = 0;
            "building (not clicked any more; its solid space for Connor and the camera stays: not built)"
        } else if self.known.contains(&key) {
            "placed object"
        } else {
            return false;
        };
        self.note(format!("{at}: deleteObject {name}: {what} deleted (not drawn, not clickable, no collision; events to it dropped)"));
        self.requests.push(Request::Delete { name: name.to_string(), at: at.to_string() });
        self.deleted_this_event.insert(key.clone());
        self.deleted.insert(key);
        true
    }

    /// Whether `name` was deleted (events to it are dropped).
    fn is_gone(&self, name: &str) -> bool {
        self.deleted.contains(&name.to_ascii_lowercase())
    }

    /// Whether `name` was deleted (`deleteObject`).
    pub fn is_deleted(&self, name: &str) -> bool {
        self.is_gone(name)
    }

    /// Whether `name`'s collider is out of the collision world now
    /// (`object-lighting.md` rule 41).
    pub fn is_switched_off(&self, name: &str) -> bool {
        self.switched_off.contains(&name.to_ascii_lowercase())
    }

    /// Whether placed object `name` is hidden now: a script's `setHidden`
    /// wins over its file's `hidden=` (`file_hidden`); scripted objects,
    /// chests and items as [`Self::is_hidden`].
    pub fn hidden_now(&self, name: &str, file_hidden: bool) -> bool {
        match self.plain_hidden.get(&name.to_ascii_lowercase()) {
            Some(h) => *h,
            None => file_hidden || self.is_hidden(name),
        }
    }

    /// Whether `name` is hidden now (scripted objects, chests, items;
    /// `objects.md` "Hidden objects"). Other objects: `false` (their
    /// hiding is fixed at load and known to the caller).
    pub fn is_hidden(&self, name: &str) -> bool {
        if let Some(i) = self.object_index(name) {
            return self.objects[i].hidden;
        }
        if let Some(i) = self.chest_index(name) {
            return self.chests[i].hidden;
        }
        self.items.get(name).is_some_and(|o| o.hidden)
    }

    /// `setActiveCamera <n>` (rule 42): 0 = Connor, any other number =
    /// the camera; the usage line if the word count is not 2.
    fn set_active_camera(&mut self, words: &[String], at: &str) -> String {
        let [_, n] = words else {
            self.note(format!("{at}: usage: setActiveCamera <0|1>: '{}'", words.join(" ")));
            return "1".into();
        };
        let camera = number_value(n) != 0.0;
        self.note(format!("{at}: setActiveCamera {n}: the draw distance is measured from {} from the next frame", if camera { "the camera" } else { "Connor" }));
        self.requests.push(Request::ActiveCamera { camera, at: at.to_string() });
        "0".into()
    }

    /// `makeActive <name> <n>` (rule 42): n not 0 = always near.
    fn make_active(&mut self, words: &[String], at: &str) -> String {
        let [_, name, n] = words else {
            self.note(format!("{at}: usage: makeActive <object> <0|1>: '{}'", words.join(" ")));
            return "1".into();
        };
        if !self.known.contains(&name.to_ascii_lowercase()) {
            self.note(format!("{at}: ERROR makeActive: unknown object '{name}'"));
            return "1".into();
        }
        let on = number_value(n) != 0.0;
        self.note(format!("{at}: makeActive {name} {n}: {} from the next frame", if on { "always near" } else { "back to the distance test" }));
        self.requests.push(Request::MakeActive { name: name.clone(), on, at: at.to_string() });
        "0".into()
    }

    /// `KQMusic::push|pop|play <file>` (`sound.md` rule 57f): push and pop
    /// go into the event queue and act 0.2 s of game time later, in the
    /// order sent; play acts at once. `create` and `delete` are obsolete.
    fn music_command(&mut self, words: &[String], at: &str) -> String {
        let verb = words[0].trim_start_matches("KQMusic::");
        if verb == "create" || verb == "delete" {
            self.note(format!("{at}: {} is obsolete, use music.kq instead", words[0]));
            return "0".into();
        }
        let Some(name) = words.get(1) else {
            self.note(format!("{at}: Usage: KQMusic::{verb} music_file"));
            return "1".into();
        };
        if verb == "play" {
            self.music.push(MusicRequest::Play(name.clone()));
            self.note(format!("{at}: KQMusic::play {name}: at once (rule 57f)"));
        } else {
            let now = self.now();
            self.queue.send(now, MUSIC_DELAY, MUSIC_TARGET, &words[0], std::slice::from_ref(name), at);
            self.note(format!("{at}: {} {name}: due at {:.4} s (rule 57f: 0.2 later)", words[0], now + MUSIC_DELAY));
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

/// A tuple from five script words (`talking.md` rule 14: the file as a
/// 16-bit number, the others as single bytes); `None` if one is not a
/// number.
fn tuple_of(words: &[String]) -> Option<Tuple> {
    if words.len() != 5 || !words.iter().all(|w| looks_like_number(w)) {
        return None;
    }
    let n = |k: usize| number_value(&words[k]) as i64;
    Some(Tuple::new(n(0) as u16, n(1) as u8, n(2) as u8, n(3) as u8, n(4) as u8))
}

impl WorldState {
    /// The `KQMonster` (or Connor) called `name`, as the world writes it
    /// (`talking.md` rule 6).
    fn monster_name(&self, name: &str) -> Option<String> {
        if name.eq_ignore_ascii_case(CONNOR) {
            return Some(CONNOR.to_string());
        }
        self.monsters.iter().find(|m| m.name.eq_ignore_ascii_case(name)).map(|m| m.name.clone())
    }

    /// Where the object called `name` stands now (its placement's
    /// position; Connor's from the caller), `None` if there is none.
    pub fn object_point(&self, name: &str) -> Option<[f32; 3]> {
        if name.eq_ignore_ascii_case(CONNOR) {
            return self.connor_at;
        }
        if self.is_gone(name) {
            return None;
        }
        if let Some(i) = self.door_index(name) {
            return Some(self.doors[i].door.placement().position);
        }
        if let Some(i) = self.chest_index(name) {
            return Some(self.chests[i].chest.placement().position);
        }
        if let Some(i) = self.object_index(name) {
            return Some(self.objects[i].player.placement().position);
        }
        if let Some(i) = self.anim_index(name) {
            return Some(self.anims[i].player.placement().position);
        }
        if let Some(m) = self.monsters.iter().find(|m| m.name.eq_ignore_ascii_case(name)) {
            return Some(m.position);
        }
        if let Some(o) = self.items.get(name) {
            return o.placement().map(|p| p.position);
        }
        self.clickables.iter().find(|c| c.name.eq_ignore_ascii_case(name)).map(|c| c.placement.position)
    }

    /// `KQMonster::speak <issuer> <to> <file> <noun> <verb> <case> <first>
    /// <last> [nofade]` (`talking.md` rules 6-9).
    fn speak(&mut self, words: &[String], at: &str) -> String {
        let usage = "usage: KQMonster::speak <issuer> <to> <file> <noun> <verb> <case> <first> <last> [nofade]";
        if !(9..=10).contains(&words.len()) {
            self.note(format!("{at}: {usage} (got {} words); nothing", words.len()));
            return "1".into();
        }
        let Some(first) = tuple_of(&words[3..8]) else {
            self.note(format!("{at}: {usage}: a number is not a number: '{}'; nothing", words.join(" ")));
            return "1".into();
        };
        if !looks_like_number(&words[8]) {
            self.note(format!("{at}: {usage}: <last> '{}' is not a number; nothing", words[8]));
            return "1".into();
        }
        // Rule 6: `nofade` only as the 10th word, exactly.
        let nofade = words.get(9).is_some_and(|w| w == "nofade");
        if words.len() == 10 && !nofade {
            self.note(format!("{at}: 10th word '{}' is not 'nofade': ignored", words[9]));
        }
        let Some(issuer) = self.monster_name(&words[1]) else {
            self.note(format!("{at}: console: '{}' is not KQMonster; nothing (rule 6)", words[1]));
            return "1".into();
        };
        // Rule 8: an issuer in a conversation is ignored; there are no
        // conversations yet (not studied).
        let last_word = number_value(&words[8]) as i64 as u8;
        let last = if last_word == 0 {
            match self.talk.book.last_of_run(first) {
                Some(l) => l,
                None => {
                    self.note(format!("{at}: <last> 0 and line {first} is not in its message file: tried as one line (it will be dropped)"));
                    first.seq
                }
            }
        } else {
            last_word
        };
        let s = Speak { issuer: issuer.clone(), first, last, nofade };
        let to = &words[2];
        let head = format!("{at}: KQMonster::speak {issuer} to {to}: {first}…{last}{}", if last_word == 0 { " (<last> 0)" } else { "" });
        let now = self.now();
        let line = if to.eq_ignore_ascii_case(&issuer) {
            format!("{head}: to itself, no turn; {}", self.talk.queue(&s, now))
        } else {
            match self.object_point(to) {
                None => format!("{head}: no object '{to}': no turn; {}", self.talk.queue(&s, now)),
                Some(_) if issuer == CONNOR && !self.connor_can_turn => {
                    format!("{head}: Connor is not in Wait or Move: no turn; {}", self.talk.queue(&s, now))
                }
                Some(p) if issuer == CONNOR => {
                    use crate::connor_auto::{AutoKind, AutoMove};
                    self.requests.push(Request::ConnorAuto { auto: AutoMove::new(AutoKind::Point, [p[0], p[1]], None), at: at.to_string() });
                    format!("{head}: Connor turns to face {to} at ({:.0}, {:.0}) (rule 7); {}", p[0], p[1], self.talk.queue_later(s, now))
                }
                Some(p) => {
                    // characters.md rule 23 (M4 4d-4): the character turns,
                    // unless noTurn.
                    let turned = match self.character_index(&issuer) {
                        Some(i) => self.anims[i].character.as_mut().unwrap().ask_turn([p[0], p[1]]).to_string(),
                        None => "not a drawn character: nothing turns".to_string(),
                    };
                    self.character_log.push(format!("update {}: {issuer}: speak to {to}: {turned}", self.updates));
                    format!(
                        "{head}: {issuer} turns to face {to} at ({:.0}, {:.0}) (rule 7, Q103 d: always, no state check): {turned}; {}",
                        p[0],
                        p[1],
                        self.talk.queue_later(s, now)
                    )
                }
            }
        };
        self.talk_note(line);
        "0".into()
    }

    /// A line for the current log and `talk.log`.
    fn talk_note(&mut self, line: String) {
        self.talk.log.push(format!("t {:8.3}: {line}", self.now()));
        self.note(line);
    }

    /// `KQMonster::clearSpeak <name>` (rule 15).
    fn clear_speak(&mut self, words: &[String], at: &str) -> String {
        let Some(name) = words.get(1) else {
            self.note(format!("{at}: usage: KQMonster::clearSpeak <name>; nothing"));
            return "1".into();
        };
        self.talk.clear(name, self.now());
        self.note(format!("{at}: KQMonster::clearSpeak {name} (talk.log)"));
        "0".into()
    }

    /// `KQMonster::testMessage f n v c [s] f n v c [s]` (rule 14): 0 when
    /// the two tuples are equal, 1 otherwise.
    fn test_message(&mut self, words: &[String], at: &str) -> String {
        let n = match words.len() {
            9 => 4,
            11 => 5,
            k => {
                self.note(format!(
                    "{at}: usage: KQMonster::testMessage f n v c f n v c; usage: KQMonster::testMessage f n v c s f n v c s (got {k} words); 1"
                ));
                return "1".into();
            }
        };
        let pad = |w: &[String]| -> Vec<String> { w.iter().cloned().chain((n == 4).then(|| "0".to_string())).collect() };
        let (a, b) = (tuple_of(&pad(&words[1..1 + n])), tuple_of(&pad(&words[1 + n..])));
        let equal = a.is_some() && a == b;
        let show = |t: Option<Tuple>| {
            t.map_or("(not numbers)".to_string(), |t| if n == 4 { format!("{} {} {} {}", t.file, t.noun, t.verb, t.case) } else { t.to_string() })
        };
        self.talk_note(format!("{at}: testMessage {} vs {}: {}", show(a), show(b), if equal { "equal: 0" } else { "different: 1" }));
        if equal { "0" } else { "1" }.into()
    }

    /// `KQMonster::stateChangeNotify <recipient> <issuer>` and
    /// `clearStateChangeNotify` (rule 13).
    fn state_change_notify(&mut self, words: &[String], at: &str) -> String {
        let [_, recipient, issuer] = words else {
            self.note(format!("{at}: usage: {} <recipient> <issuer>; nothing", words[0]));
            return "1".into();
        };
        let list = self.notify.entry(issuer.to_ascii_lowercase()).or_default();
        let line = if words[0] == "KQMonster::stateChangeNotify" {
            if list.iter().any(|r| r.eq_ignore_ascii_case(recipient)) {
                format!("{recipient} already hears {issuer}")
            } else {
                list.push(recipient.clone());
                format!("{recipient} now hears {issuer}'s TalkComplete (StateChange: monsters, M5)")
            }
        } else {
            let before = list.len();
            list.retain(|r| !r.eq_ignore_ascii_case(recipient));
            if list.len() < before {
                format!("{recipient} no longer hears {issuer}")
            } else {
                format!("{recipient} did not hear {issuer}; nothing")
            }
        };
        self.talk_note(format!("{at}: {}: {line}", words[0]));
        "0".into()
    }

    /// Connor's built-in remarks (`talking.md` rule 19): one line through
    /// his queue, without turning. Returns a line for the log.
    pub fn connor_remark(&mut self, t: Tuple, what: &str) -> String {
        let s = Speak { issuer: CONNOR.to_string(), first: t, last: t.seq, nofade: false };
        let line = self.talk.queue(&s, self.now());
        format!("Connor's \"{what}\" remark {t}: {line}")
    }

    /// A chest's `Take…` / `Empty…` remark (`talking.md` rule 19,
    /// `chests.md` rules 8-10): its six numbers are the tuple and the last
    /// sequence (`End`, 0: the run). Connor turns to the chest first
    /// unless it has `doNotPointAt`, then the 0.5 s wait; the turn and the
    /// wait go together and don't depend on his state (Q103 e: any state,
    /// unlike `speak`); with `doNotPointAt` queued at once.
    fn chest_remark(&mut self, c: &ActiveChest, r: crate::chest::Remark) -> String {
        let [f, n, v, k, q, end] = r.0;
        let first = Tuple::new(f as u16, n as u8, v as u8, k as u8, q as u8);
        let last = if end == 0 { self.talk.book.last_of_run(first).unwrap_or(first.seq) } else { end as u8 };
        let s = Speak { issuer: CONNOR.to_string(), first, last, nofade: false };
        let now = self.now();
        let at = c.chest.placement().position;
        let line = if c.do_not_point_at {
            format!("doNotPointAt: no turn; {}", self.talk.queue(&s, now))
        } else {
            use crate::connor_auto::{AutoKind, AutoMove};
            let why = format!("chest {} remark", c.chest.name);
            self.requests.push(Request::ConnorAuto { auto: AutoMove::new(AutoKind::Point, [at[0], at[1]], None), at: why });
            let state = if self.connor_can_turn { "Wait or Move" } else { "not Wait or Move" };
            format!("Connor ({state}) turns to face the chest at ({:.0}, {:.0}) (any state, Q103 e); {}", at[0], at[1], self.talk.queue_later(s, now))
        };
        format!("{first}…{last}: {line}")
    }

    /// Lines for `talk.log` since the last call.
    pub fn take_talk_log(&mut self) -> Vec<String> {
        std::mem::take(&mut self.talk.log)
    }
}

impl Active {
    /// A test option (`--command`, viewer): one script command, its words
    /// split at spaces (no variables, no quotes), run as if a script did.
    /// Returns its result, or `None` for a command the world doesn't know.
    pub fn test_command(&mut self, line: &str, at: &str) -> Option<String> {
        let words: Vec<String> = line.split_whitespace().map(str::to_string).collect();
        if words.is_empty() {
            return None;
        }
        self.world.note(format!("{at}: {line}"));
        ScriptHost::world_command(&mut self.world, &words, at)
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
            "KQConner::getDoMode" => {
                // pushing.md "Words used here": the hand mode, sword 0, bow
                // 1, empty hands 4 (the game's help text). Connor has no
                // weapons yet and `KQConner::setDoMode` is a stub (M5), so
                // always empty hands.
                let mode = crate::connor::EMPTY_HANDS;
                self.note(format!("{at}: KQConner::getDoMode: {mode} (empty hands; no weapons yet)"));
                Some(mode.to_string())
            }
            "KQConner::JumpDir" => Some(self.jump_dir(at)),
            "KQConner::endJump" => Some(self.end_jump(at)),
            "KQCamera::setRange" => Some(self.set_range(words, at)),
            "KQCamera::addCamLoc" | "KQCamera::flyToCamLoc" | "KQCamera::cutToCamLoc" => Some(self.cam_loc(words, at)),
            "KQMonster::inventory" | "give" => Some(self.make_item(words, at)),
            "KQMonster::checkInventory" => Some(self.check_inventory(words, at)),
            "KQObject::setAccepts" => Some(self.set_accepts(words, at)),
            "getEndLoop" => Some(self.get_end_loop(words, at)),
            "setDeleteWhenDone" => Some(self.set_delete_when_done(words, at)),
            "KQMonster::setState" => Some(self.set_state(words, at)),
            "makeTranslucent" => Some(self.make_translucent(words, at)),
            "KQObject::setUseGravity" | "KQMonster::setMonUseGravity" | "KQMonster::setCanTurn" | "KQMonster::changeAnmFile" => {
                Some(self.character_switch(words, at))
            }
            "move" => Some(self.move_command(words, at)),
            "KQCamera::follow" => Some(self.camera_follow(words, at)),
            "handsOff" => Some(self.hands_off(words, at)),
            "setLowerGui" | "setUpperGui" | "setMapGui" | "getGuiStatus" => Some(self.gui_command(words, at)),
            "goldCoinPersists" => Some(self.gold_coin_persists(words, at)),
            "setThirdPerson" => Some(self.set_third_person(words, at)),
            "KQConner::walkTo" | "KQConner::runTo" | "pointAndCue" => Some(self.connor_auto(words, at)),
            "walkExit" => {
                self.note(format!("{at}: walkExit: a scripted walk ends now, with its cue (anim-scripts.md rule 24)"));
                self.requests.push(Request::WalkExit { at: at.to_string() });
                Some("0".into())
            }
            "loadKQ" => Some(self.load_kq(words, at)),
            "pauseAnim" | "setAnimPos" => Some(self.anim_pos(words, at)),
            "setActiveCamera" => Some(self.set_active_camera(words, at)),
            "makeActive" => Some(self.make_active(words, at)),
            "KQSound::play" => Some(self.play_sound(words, at)),
            "KQMusic::push" | "KQMusic::pop" | "KQMusic::play" | "KQMusic::create" | "KQMusic::delete" => {
                Some(self.music_command(words, at))
            }
            "deleteObject" => Some(self.delete_objects(words, at)),
            "KQMonster::speak" => Some(self.speak(words, at)),
            "KQMonster::clearSpeak" => Some(self.clear_speak(words, at)),
            "KQMonster::testMessage" => Some(self.test_message(words, at)),
            "KQMonster::stateChangeNotify" | "KQMonster::clearStateChangeNotify" => Some(self.state_change_notify(words, at)),
            "KQObject::setScript" | "KQObject::setDontLook" | "KQObject::setHidden" => Some(self.item_change(words, at)),
            _ => None,
        }
    }

    fn log(&mut self, line: String) {
        self.note(line);
    }

    fn take_vars(&mut self) -> Vec<(String, String)> {
        std::mem::take(&mut self.vars_out)
    }

    fn take_calls(&mut self) -> Vec<(String, Vec<String>)> {
        std::mem::take(&mut self.calls_out)
    }

    fn event_done(&mut self) {
        self.deleted_this_event.clear();
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
                anims: Vec::new(),
                traps: Vec::new(),
                clickables,
                buildings: Vec::new(),
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
                sounds: Vec::new(),
                music: Vec::new(),
                heights: None,
                vars_out: Vec::new(),
                accepts: BTreeMap::new(),
                end_loop_listeners: BTreeMap::new(),
                deleted: BTreeSet::new(),
                switched_off: BTreeSet::new(),
                connor_stats: None,
                connor_stats_events: 0,
                gui_orders: Vec::new(),
                item_timers: Vec::new(),
                item_voices: Vec::new(),
                hand: None,
                gold_persists: false,
                scroll_remark_at: None,
                stats: BTreeMap::new(),
                plain_hidden: BTreeMap::new(),
                dormant: BTreeMap::new(),
                deleted_this_event: BTreeSet::new(),
                calls_out: Vec::new(),
                first_person_locked: false,
                was_first_person: false,
                connor_jump: None,
                talk: Talk::default(),
                notify: BTreeMap::new(),
                connor_can_turn: false,
                connor_at: None,
                monsters: Vec::new(),
                characters_in_range: BTreeSet::new(),
                character_log: Vec::new(),
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

    /// Adds the buildings (before [`Active::start`]), in the order the
    /// caller's [`ClickWalls`] uses.
    pub fn add_buildings(&mut self, buildings: impl IntoIterator<Item = ActiveBuilding>) {
        for b in buildings {
            self.world.known.insert(b.name.to_ascii_lowercase());
            self.world.buildings.push(b);
        }
    }

    /// Whether building `i` takes clicks now (bit 0x20, `clicking.md`
    /// rules 3-4; not while its collider is out, `object-lighting.md` rule
    /// 41).
    pub fn clickable_building(&self, i: usize) -> bool {
        self.world.buildings.get(i).is_some_and(|b| b.collision_type & CLICKABLE != 0 && !self.world.is_switched_off(&b.name))
    }

    /// The collider switch (`object-lighting.md` rule 41): `on` = `name`'s
    /// collider is in the collision world. Returns whether it changed.
    pub fn switch_collider(&mut self, name: &str, on: bool) -> bool {
        let key = name.to_ascii_lowercase();
        if on { self.world.switched_off.remove(&key) } else { self.world.switched_off.insert(key) }
    }

    fn building_index(&self, name: &str) -> Option<usize> {
        self.world.buildings.iter().position(|b| b.name.eq_ignore_ascii_case(name))
    }

    /// A scripted pushable was pushed (`pushing.md` rules 3, 10): its
    /// script runs with `$1` = its name, `$2` = `Push`. Returns `false`
    /// when it is still a pushable afterwards (its script didn't take
    /// control, or it has none): the caller makes its later touches wall
    /// hits ("Do not reproduce").
    pub fn push_call(&mut self, name: &str) -> bool {
        self.world.note(format!("{name}: pushed by Connor: its script gets Push (pushing.md rule 3)"));
        if !self.call_named(name, &["Push"]) {
            self.world.note(format!("{name}: no object with a script by that name: Push not called"));
        }
        let still = self.world.collision_type_of(name).is_some_and(|t| t & crate::collide::PUSHABLE != 0);
        if still {
            self.world.note(format!("{name}: still a pushable after its Push call: from now on a wall for Connor (pushing.md \"Do not reproduce\")"));
        }
        !still
    }

    /// One of Connor's play-once movements ended (`plain` name, e.g.
    /// `push`): his `getEndLoop` listener gets `Cue 300 0 <movement>`
    /// (`clicking.md` rule 21, `anim-scripts.md` rule 16). He has no
    /// script, so no `EndLoop`.
    pub fn connor_movement_ended(&mut self, movement: &str) {
        self.world.report_end("Connor", movement, false);
    }

    /// A scripted turn or walk of Connor ended (`anim-scripts.md` rules
    /// 24–25): `KQCueEvent <c> <d>` to the object the script named.
    pub fn connor_auto_cue(&mut self, cue: &crate::connor_auto::Cue, why: &str) {
        let now = self.world.now();
        let at = format!("Connor's scripted movement ({why})");
        self.world.note(format!("{at}: KQCueEvent {} {} to {}", cue.c, cue.d, cue.who));
        self.world.queue.send(now, 0.0, &cue.who, "KQCueEvent", &[cue.c.clone(), cue.d.clone()], &at);
    }

    /// The objects prepared for a script's `loadKQ` (3k-4): file name,
    /// object name and its script. Their names are not known to the
    /// scripts until loaded.
    pub fn set_dormant(&mut self, files: impl IntoIterator<Item = (String, String, Option<String>)>) {
        for (file, name, script) in files {
            self.world.known.remove(&name.to_ascii_lowercase());
            self.world.dormant.insert(file.to_ascii_lowercase(), (name, script));
        }
    }

    /// Connor's starting stats and level (`Connor.kq`).
    pub fn set_connor_stats(&mut self, c: ConnorStats) {
        self.world.connor_stats = Some(c);
    }

    /// An object's starting stats (its `stats=` line).
    pub fn set_stats(&mut self, name: &str, s: Stats) {
        self.world.stats.insert(name.to_ascii_lowercase(), s);
    }

    /// `KQStatsChangeRequestEvent` (`anim-scripts.md` rule 19,
    /// `clicking.md` rule 24): the five numbers are added. An object other
    /// than Connor whose health drops below 0 gets `Die`.
    fn stats_change(&mut self, e: &Pending, from: &str) {
        let (mut d, problems) = Stats::parse(&e.data);
        // Rule 19: the event's own fourth number is cut to at most 100
        // when read (negative values stay).
        d.0[3] = d.0[3].min(crate::stats::FOURTH_CAP);
        let head = format!("KQStatsChangeRequestEvent {} to {}{from}", e.data.join(" "), e.target);
        for p in problems {
            self.world.note(format!("{head}: {p}"));
        }
        if e.target.eq_ignore_ascii_case("Connor") {
            let Some(c) = self.world.connor_stats.as_mut() else {
                self.world.note(format!("{head}: Connor's stats unknown (no Connor.kq); nothing"));
                return;
            };
            let before = *c;
            // Rule 19: "below 0" is checked right after the adding,
            // before the cap and any level-up heal.
            let dies = {
                let mut t = c.stats;
                t.add(d);
                t.dead()
            };
            let level = c.change(d);
            let after = *c;
            self.world.connor_stats_events += 1;
            self.world.note(format!(
                "{head}: Connor {:?} level {} -> {:?} level {} (health / second / third / fourth / experience){}",
                before.stats.0,
                before.level,
                after.stats.0,
                after.level,
                level.map_or(String::new(), |l| format!("; {l}"))
            ));
            if dies {
                self.world.note(format!("{head}: Connor's health is below 0: he would die (his Die state, M5; not done)"));
            }
            return;
        }
        let key = e.target.to_ascii_lowercase();
        let known = self.world.stats.contains_key(&key);
        let s = self.world.stats.entry(key).or_default();
        let before = *s;
        s.add(d);
        let after = *s;
        let start = if known { "" } else { " (no stats= line: started from 0 0 0 0 0, Q86 d stand-in)" };
        self.world.note(format!("{head}: {:?} -> {:?}{start}", before.0, after.0));
        if after.dead() {
            self.world.note(format!("{head}: health below 0: its script gets Die {DIE_KIND} (rules 19-20)"));
            if !self.call_named(&e.target, &["Die", DIE_KIND]) {
                self.world.note(format!("{}: no script by that name; Die not called", e.target));
            }
        }
    }

    /// Adds the other animated objects (3i-4), in the viewer's order.
    pub fn add_anims(&mut self, anims: impl IntoIterator<Item = AnimObject>) {
        for a in anims {
            self.world.known.insert(a.player.name.to_ascii_lowercase());
            self.world.anims.push(a);
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
    fn call_object(&mut self, i: usize, args: &[&str]) -> bool {
        let o = &self.world.objects[i];
        let name = o.name().to_string();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match o.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
                true
            }
            None => {
                self.world.note(format!("object {name} has no script; {} not called", all.join(" ")));
                false
            }
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
        self.traps_check(v, false)
    }

    /// After a script's `move Connor` (traps.md rule 5 "Placed by a
    /// script", Q102 b): every trap he is inside runs its stay test at the
    /// new spot, so the one he was taken out of gets `Off` (or `Cue 302`)
    /// now; no trap is entered (that waits for his next ordinary move).
    pub fn connor_placed(&mut self, v: &Visitor) -> Vec<String> {
        self.traps_check(v, true)
    }

    fn traps_check(&mut self, v: &Visitor, placed: bool) -> Vec<String> {
        let mut calls = Vec::new();
        for i in 0..self.world.traps.len() {
            let shrunk_before = self.world.traps[i].shrunk;
            let c = if placed { self.world.traps[i].stay_check(v) } else { self.world.traps[i].check(v) };
            let Some(args) = c.call else { continue };
            let t = &self.world.traps[i];
            let (name, gone) = (t.settings.name.clone(), t.gone);
            let change = match (shrunk_before, t.shrunk) {
                (true, false) => "; grows to full",
                (false, true) => "; shrinks again",
                _ => "",
            };
            let line = format!(
                "trap {name}: {}{} at ({:.0}, {:.0}, {:.0}): touching {}, box centre inside {}, {} box ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0}){change}; inside now: [{}]",
                args.join(" "),
                if placed { " (stay test right after a script's move, rule 5)" } else { "" },
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

    /// Runs the script of the object called `name` (door, chest, scripted
    /// object, clickable, building or trap) with `args`. `false`: no such
    /// object with a script.
    fn call_named(&mut self, name: &str, args: &[&str]) -> bool {
        let w = &self.world;
        if let Some(i) = w.door_index(name).filter(|&i| w.doors[i].script.is_some()) {
            self.call_door(i, args);
        } else if let Some(i) = w.chest_index(name).filter(|&i| w.chests[i].script.is_some()) {
            self.call_chest(i, args);
        } else if let Some(i) = w.object_index(name).filter(|&i| w.objects[i].script.is_some()) {
            self.call_object(i, args);
        } else if let Some(i) = w.clickables.iter().position(|c| c.script.is_some() && c.name.eq_ignore_ascii_case(name)) {
            self.call_clickable(i, args);
        } else if let Some(i) = w.trap_index(name).filter(|&i| w.traps[i].settings.script.is_some()) {
            self.call_trap(i, args);
        } else if let Some((n, script)) = w.anim_index(name).and_then(|i| Some((w.anims[i].player.name.clone(), w.anims[i].script.clone()?))) {
            let mut all = vec![n];
            all.extend(args.iter().map(|a| a.to_string()));
            self.world.note(format!("script {script} {}", all.join(" ")));
            self.scripts.call(&script, &all, &mut self.world);
        } else if let Some(b) = w.buildings.iter().find(|b| b.name.eq_ignore_ascii_case(name)).and_then(|b| Some((b.name.clone(), b.script.clone()?))) {
            let mut all = vec![b.0];
            all.extend(args.iter().map(|a| a.to_string()));
            self.world.note(format!("script {} {}", b.1, all.join(" ")));
            self.scripts.call(&b.1, &all, &mut self.world);
        } else if let Some((n, script)) = w.monsters.iter().find(|m| m.name.eq_ignore_ascii_case(name)).and_then(|m| Some((m.name.clone(), m.script.clone()?))) {
            // A monster's script (M4 4a-2: only for TalkComplete; monsters
            // don't get Init yet, M5).
            let mut all = vec![n];
            all.extend(args.iter().map(|a| a.to_string()));
            self.world.note(format!("script {script} {}", all.join(" ")));
            self.scripts.call(&script, &all, &mut self.world);
        } else {
            return false;
        }
        true
    }

    /// Runs door `i`'s script with `args` (`$1` is the door's name).
    fn call_door(&mut self, i: usize, args: &[&str]) -> bool {
        let d = &self.world.doors[i];
        let name = d.door.name.clone();
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match d.script.clone() {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
                true
            }
            None => {
                self.world.note(format!("{name} has no script; {} not called", all.join(" ")));
                false
            }
        }
    }

    /// Rule 22: each door's script gets `Init` as it enters the world.
    /// Every object's accept list is read first (`clicking.md` rule 21:
    /// before `Init`).
    pub fn start(&mut self) {
        let w = &self.world;
        let names: Vec<String> = (w.doors.iter().map(|d| d.door.name.clone()))
            .chain(w.objects.iter().map(|o| o.name().to_string()))
            .chain(w.clickables.iter().map(|c| c.name.clone()))
            .chain(w.buildings.iter().map(|b| b.name.clone()))
            .collect();
        for name in names {
            let a = self.world.accepts_entry(&name).clone();
            if a.all || !a.types.is_empty() {
                self.world.note(format!("{name} accepts {}", describe_accepts(&a)));
            }
        }
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
        // Other scripted clickables (3f, Q64 c stand-in).
        for i in 0..self.world.clickables.len() {
            if self.world.clickables[i].script.is_some() {
                self.call_clickable(i, &["Init"]);
            }
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
        let off = |n: &str| self.world.is_switched_off(n);
        let doors = self.world.doors.iter().filter(|d| d.collision_type & CLICKABLE != 0 && !d.gone && !off(&d.door.name)).filter_map(|d| {
            let (min, max) = d.bounds?;
            Some((d.door.name.clone(), ray_box(origin, dir, &d.door.placement(), min, max)?))
        });
        let others = self
            .world
            .clickables
            .iter()
            .filter(|c| c.collision_type & CLICKABLE != 0 && !c.gone && !off(&c.name))
            .filter_map(|c| Some((c.name.clone(), ray_box(origin, dir, &c.placement, c.min, c.max)?)));
        // Chests (chests.md rule 4): as doors; hidden ones can't be clicked.
        let chests = self.world.chests.iter().filter(|c| c.collision_type & CLICKABLE != 0 && !c.hidden && !c.gone && !off(&c.chest.name)).filter_map(|c| {
            let (min, max) = c.bounds?;
            Some((c.chest.name.clone(), ray_box(origin, dir, &c.chest.placement(), min, max)?))
        });
        let objects = self.world.objects.iter().filter(|o| o.collision_type & CLICKABLE != 0 && !o.hidden && !o.gone && !off(o.name())).filter_map(|o| {
            let (min, max) = o.bounds?;
            Some((o.name().to_string(), ray_box(origin, dir, &o.player.placement(), min, max)?))
        });
        let doors = doors.chain(chests).chain(objects);
        // World items (items.md rule 6): by their type's box; hidden ones
        // can't be clicked.
        let items = &self.world.items;
        let items = items.objects.iter().filter(|o| !o.hidden && !off(&o.name)).filter_map(|o| {
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

    /// The click line from `origin` (the camera's eye) along `dir` (length
    /// 1), `clicking.md` rules 1-8: the nearest clickable box
    /// ([`Active::pick`]) or clickable building ([`ClickWalls`]), whichever
    /// comes first. The ground and objects without bit 0x20 are not
    /// tested.
    pub fn pick_line(&self, origin: [f32; 3], dir: [f32; 3], walls: &dyn ClickWalls) -> Pick {
        let boxed = self.pick(origin, dir);
        let length = boxed.as_ref().map_or(CLICK_LINE_LENGTH, |(_, t)| *t);
        let to: [f32; 3] = std::array::from_fn(|k| origin[k] + dir[k] * length);
        match (walls.first_hit(origin, to), boxed) {
            (Some((name, t)), behind) if behind.as_ref().is_none_or(|(_, tb)| t < *tb) => Pick::Building { name, t, behind },
            (_, Some((name, t))) => Pick::Seen { name, t },
            _ => Pick::Nothing,
        }
    }

    /// Connor clicks object `name` (rules 11–17). `feet`: his position;
    /// `attack_dist`: his `attackDist` (reach = twice that); `item`: what
    /// he holds.
    pub fn click(&mut self, name: &str, feet: [f32; 3], attack_dist: f32, item: Option<&Item>) -> ClickResult {
        let reach = 2.0 * attack_dist;
        if self.world.is_switched_off(name) {
            // object-lighting.md rule 41: no collider, nothing to click.
            self.world.note(format!("click {name}: its collider is out (far, off or hidden); nothing"));
            return ClickResult::Nothing;
        }
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
        if let Some(i) = self.building_index(name).filter(|_| self.world.door_index(name).is_none()) {
            return self.click_building(i, feet, reach, item);
        }
        let Some(i) = self.world.door_index(name) else {
            let found = self.world.clickables.iter().position(|c| c.name.eq_ignore_ascii_case(name));
            if let Some(i) = found.filter(|&i| self.world.clickables[i].script.is_some()) {
                return self.click_clickable(i, feet, reach, item);
            }
            match found.map(|i| self.world.clickables[i].clone()) {
                Some(c) if c.gone => self.world.note(format!("click {}: deleted; nothing", c.name)),
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
        if d.collision_type & CLICKABLE == 0 || d.gone {
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
        let rules = ClickRules { name: &d.door.name, too_far_away_action: d.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
        };
        self.world.note(format!("{head}; script arguments: {} {}", d.door.name, args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let called = self.call_door(i, &refs);
        self.click_turn(&d.door.name, &args, called, d.door.placement().position, d.do_not_point_at);
        let mut full = vec![d.door.name.clone()];
        full.extend(args);
        ClickResult::Script(full)
    }

    /// The turn after a click (`clicking.md` rules 25–33, M4 4a-8): after
    /// a `ConnorAction` call (`args` = `ConnorAction`, `$3`, `$4`; `called`:
    /// a script ran) Connor is asked to turn to `position`, the object's
    /// placement point, unless it has `doNotPointAt` (rule 29). His state
    /// is looked at when the request is applied, after the script's own
    /// requests (rules 27–28). Logs the call kind and the outcome.
    fn click_turn(&mut self, name: &str, args: &[String], called: bool, position: [f32; 3], do_not_point_at: bool) {
        let call = match args.get(1).map(String::as_str) {
            Some(k @ ("do" | "TooFar" | "Special")) => k.to_string(),
            Some(object) => format!("item {object} ({})", args.get(2).map_or("?", |s| s.as_str())),
            None => "none".into(),
        };
        let outcome = if !called {
            "no script, no call: no turn (rule 26)".to_string()
        } else if do_not_point_at {
            "doNotPointAt: no turn (rule 29)".to_string()
        } else {
            let at = format!("click {name} ({call})");
            self.world.requests.push(Request::ClickTurn { name: name.to_string(), target: [position[0], position[1]], at });
            format!(
                "turn to {name} at ({:.0}, {:.0}) asked for; Connor's state is looked at after the script's requests (rules 27-28)",
                position[0], position[1]
            )
        };
        self.world.note(format!("click {name}: call {call}: {outcome}"));
    }

    /// The script arguments of a click (`doors.md` rules 12-14), or what
    /// happened instead (logged).
    fn script_args(&mut self, head: &str, in_reach: bool, item: Option<&Item>, rules: &ClickRules) -> Result<Vec<String>, ClickResult> {
        match (in_reach, item) {
            (true, None) => Ok(vec!["ConnorAction".into(), "do".into(), "do".into()]),
            (true, Some(it)) => {
                let accepts = self.world.accepts_of(rules.name);
                if !accepts.takes(&it.kind, &self.world.items.types) {
                    let remark = self.world.connor_remark(WONT_WORK, "This will not work");
                    self.world.note(format!(
                        "{head}; holds {} ({}): not accepted (accepts {}): {remark}; script not run; no call: no turn (clicking.md rule 26)",
                        it.object,
                        it.kind,
                        describe_accepts(&accepts)
                    ));
                    return Err(ClickResult::Refused);
                }
                // Rule 18: $4 is the item's type name as the type list
                // spells it.
                let kind = self.world.items.kind(&it.kind).map_or_else(|| it.kind.clone(), |t| t.name.clone());
                Ok(vec!["ConnorAction".into(), it.object.clone(), kind])
            }
            (false, _) if rules.too_far_away_action == 0 => {
                let remark = self.world.connor_remark(TOO_FAR, "too far");
                self.world.note(format!("{head}; tooFarAwayAction 0: {remark}; nothing sent; no call: no turn (clicking.md rule 26)"));
                Err(ClickResult::TooFar)
            }
            (false, _) => Ok(vec!["ConnorAction".into(), "TooFar".into(), "TooFar".into()]),
        }
    }

    /// Connor clicks building `i` (`clicking.md` rules 9-12): the calls a
    /// door gets; with no script they are dropped (no remark, no turn).
    fn click_building(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let b = self.world.buildings[i].clone();
        if b.collision_type & CLICKABLE == 0 {
            self.world.note(format!("click building {}: not clickable (type {:#x}); nothing", b.name, b.collision_type));
            return ClickResult::Nothing;
        }
        // Rule 9 (Q75): to the .dig box in the building's own axes.
        let distance = distance_to_box(feet, &b.placement, b.bounds.0, b.bounds.1);
        let in_reach = distance < reach;
        let head = format!(
            "click building {}: Connor at ({:.0}, {:.0}, {:.0}), distance to its .dig box (its own axes) {distance:.1}, reach {reach:.0}: {}",
            b.name,
            feet[0],
            feet[1],
            feet[2],
            if in_reach { "IN REACH" } else { "OUT OF REACH" }
        );
        let rules = ClickRules { name: &b.name, too_far_away_action: b.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
        };
        let Some(script) = b.script.clone() else {
            self.world.note(format!("{head}; {} {}: swallowed, no script: nothing happens, no call: no turn (rules 10, 26)", b.name, args.join(" ")));
            return ClickResult::Swallowed;
        };
        let mut all = vec![b.name.clone()];
        all.extend(args);
        self.world.note(format!("{head}; script {script} {}", all.join(" ")));
        self.scripts.call(&script, &all, &mut self.world);
        self.click_turn(&b.name, &all[1..], true, b.placement.position, b.do_not_point_at);
        ClickResult::Script(all)
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
        let rules = ClickRules { name: &name, too_far_away_action: o.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
        };
        self.world.note(format!("{head}; script arguments: {name} {}", args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        let called = self.call_object(i, &refs);
        self.click_turn(&name, &args, called, o.player.placement().position, o.do_not_point_at);
        let mut full = vec![name];
        full.extend(args);
        ClickResult::Script(full)
    }

    /// Connor clicks clickable object `i`, which has a script (3f): as a
    /// door (Q64 a stand-in).
    fn click_clickable(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let c = self.world.clickables[i].clone();
        if c.gone || c.collision_type & CLICKABLE == 0 {
            self.world.note(format!("click {}: deleted or not clickable now (type {:#x}); nothing", c.name, c.collision_type));
            return ClickResult::Nothing;
        }
        let distance = distance_to_box(feet, &c.placement, c.min, c.max);
        let in_reach = distance < reach;
        let head = format!(
            "click {} (class {}, scripted): Connor at ({:.0}, {:.0}, {:.0}), distance to its box {distance:.1}, reach {reach:.0}: {}",
            c.name,
            c.class,
            feet[0],
            feet[1],
            feet[2],
            if in_reach { "IN REACH" } else { "OUT OF REACH" }
        );
        let rules = ClickRules { name: &c.name, too_far_away_action: c.too_far_away_action };
        let args = match self.script_args(&head, in_reach, item, &rules) {
            Ok(a) => a,
            Err(r) => return r,
        };
        self.world.note(format!("{head}; script arguments: {} {}", c.name, args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.call_clickable(i, &refs);
        // Only clickables with a script come here: always a call.
        self.click_turn(&c.name, &args, true, c.placement.position, c.do_not_point_at);
        let mut full = vec![c.name];
        full.extend(args);
        ClickResult::Script(full)
    }

    /// Runs clickable `i`'s script with `args` (`$1` is its name).
    fn call_clickable(&mut self, i: usize, args: &[&str]) {
        let c = &self.world.clickables[i];
        let (name, script) = (c.name.clone(), c.script.clone());
        let mut all = vec![name.clone()];
        all.extend(args.iter().map(|a| a.to_string()));
        match script {
            Some(script) => {
                self.world.note(format!("script {script} {}", all.join(" ")));
                self.scripts.call(&script, &all, &mut self.world);
            }
            None => self.world.note(format!("{name} has no script; {} not called", all.join(" "))),
        }
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
            let remark = self.world.connor_remark(TOO_FAR, "too far");
            self.world.item_note(format!("{head}: OUT OF REACH: {remark}; nothing; no call: no turn (clicking.md rule 26)"));
            return ClickResult::TooFar;
        }
        if let Some(it) = item {
            let remark = self.world.connor_remark(WONT_WORK, "This will not work");
            self.world.item_note(format!("{head}: holds {} ({}): items accept nothing: {remark}; no call: no turn (clicking.md rule 26)", it.object, it.kind));
            return ClickResult::Refused;
        }
        let before = self.world.items.count("Connor", &o.kind);
        crate::connor_inventory::receive(&mut self.world, &o.name, Some(placement.position), "picked up");
        let after = self.world.items.count("Connor", &o.kind);
        let sound = self.world.pickup_sound(&o.kind);
        self.world.item_note(format!("{head}: IN REACH: picked up; Connor's {} {before} -> {after}; {sound}; gone from the world", o.kind));
        // Rule 7: then its script, as a normal click.
        let called = o.script.clone().map(|script| {
            let args: Vec<String> = [o.name.as_str(), "ConnorAction", "do", "do"].map(String::from).to_vec();
            self.world.item_note(format!("script {script} {}", args.join(" ")));
            self.scripts.call(&script, &args, &mut self.world);
        });
        let do_args = ["ConnorAction".to_string(), "do".into(), "do".into()];
        self.click_turn(&o.name, &do_args, called.is_some(), placement.position, o.do_not_point_at);
        ClickResult::PickedUp(o.name)
    }

    /// Connor clicks chest `i` (chests.md rules 4-10).
    fn click_chest(&mut self, i: usize, feet: [f32; 3], reach: f32, item: Option<&Item>) -> ClickResult {
        let c = self.world.chests[i].clone();
        let name = c.chest.name.clone();
        if c.collision_type & CLICKABLE == 0 || c.hidden || c.gone {
            self.world.note(format!("click {name}: not clickable now (type {:#x}, hidden {}, deleted {}); nothing", c.collision_type, c.hidden, c.gone));
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
            let remark = self.world.connor_remark(TOO_FAR, "too far");
            self.world.note(format!("{head}: OUT OF REACH: {remark}; nothing"));
            return ClickResult::TooFar;
        }
        if let Some(it) = item {
            // Rule 5: chests accept no item.
            let remark = self.world.connor_remark(WONT_WORK, "This will not work");
            self.world.note(format!("{head}: holds {} ({}): chests accept nothing: {remark}", it.object, it.kind));
            return ClickResult::Refused;
        }
        let at = format!("chest {name}");
        self.world.note(format!("click {name}: chest: no ConnorAction call: no click turn (clicking.md rule 33)"));
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
                            let lay = o.placement().map(|p| p.position);
                            crate::connor_inventory::receive(&mut self.world, &o.name, lay, "from a chest");
                            let sound = self.world.pickup_sound(&o.kind);
                            let line = format!(
                                "{at}: 2nd click: {} moved into Connor's inventory ({} {count} -> {}); {sound}",
                                o.name,
                                o.kind,
                                self.world.items.count("Connor", &o.kind),
                            );
                            self.world.item_note(line);
                            format!("item {} moved into Connor's inventory", o.name)
                        }
                    }
                    None => format!("remembered item {remembered:?} no longer exists"),
                };
                let take = match c.chest.settings.take.filter(|r| r.said()) {
                    Some(r) => format!("; Take remark {}", self.world.chest_remark(&c, r)),
                    None => "; no Take remark".into(),
                };
                self.world.chests[i].chest.state = ChestState::Empty;
                self.world.note(format!("{head}: IN REACH: 2nd click (open): {what}{take}; state open -> empty"));
            }
            ChestState::Empty => {
                let empty = match c.chest.settings.empty.filter(|r| r.said()) {
                    Some(r) => format!("Empty remark {}", self.world.chest_remark(&c, r)),
                    None => "no Empty remark: nothing".into(),
                };
                self.world.note(format!("{head}: IN REACH: click (empty): {empty}"));
            }
        }
        ClickResult::Chest { before, after: self.world.chests[i].chest.state }
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
        self.talk_step(now);
        // M4 4c-3: potions whose Dur ran out (inventory.md rule 14).
        crate::connor_inventory::timers(&mut self.world);
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
            let at = self.world.doors[i].door.player.base.position;
            for cue in &a.cues {
                let what = self.world.cue_sound(&format!("door {name}"), cue, at);
                self.world.note(format!("door {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(st) = &a.started {
                let line = started_line(&format!("door {name}"), st, &self.world.doors[i].door.player.movements, "rule 8: the movement before it ended");
                self.world.note(line);
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
            // clicking.md rule 21: after Open/Close, EndLoop through the queue.
            if let Some(m) = &a.finished_movement {
                let has_script = self.world.doors[i].script.is_some();
                self.world.movement_ended(&name, m, has_script);
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
            let at = self.world.chests[i].chest.placement().position;
            for cue in &a.cues {
                let what = self.world.cue_sound(&format!("chest {name}"), cue, at);
                self.world.note(format!("chest {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(st) = &a.started {
                let why = if a.finished.is_some() { "rule 8: the movement before it ended" } else { "rule 3: no current movement name" };
                let line = started_line(&format!("chest {name}"), st, &self.world.chests[i].chest.player.movements, why);
                self.world.note(line);
            }
            if let Some(m) = a.finished {
                // chests.md rule 6, clicking.md rule 21: EndLoop through the queue.
                let after = if a.started.is_some() { "the waiting movement plays" } else { "stays on its last frame" };
                self.world.note(format!("chest {name}: movement '{m}' finished; {after}"));
                let has_script = self.world.chests[i].script.is_some();
                self.world.movement_ended(&name, &m, has_script);
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
            let at = self.world.objects[i].player.placement().position;
            for cue in &a.cues {
                let what = self.world.cue_sound(&format!("object {name}"), cue, at);
                self.world.note(format!("object {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(st) = &a.started {
                let why = if a.finished.is_some() { "rule 8: the movement before it ended" } else { "rule 3: no current movement name" };
                let line = started_line(&format!("object {name}"), st, &self.world.objects[i].player.movements, why);
                self.world.note(line);
            }
            if let Some(m) = a.finished {
                if self.world.objects[i].player.once {
                    self.anim_once_ended(i, &m);
                    continue;
                }
                // anim-scripts.md rule 8, clicking.md rule 21: EndLoop
                // through the queue, naming the movement that ended.
                let after = if a.started.is_some() { "the waiting movement plays" } else { "stays on its last frame" };
                self.world.note(format!("object {name}: movement '{m}' finished; {after}"));
                let has_script = self.world.objects[i].script.is_some();
                self.world.movement_ended(&name, &m, has_script);
            }
        }
        // The other animated objects (3i-4): only starts, cues and ends
        // are logged here (their phases go to anim.log).
        let now = self.world.now();
        let update = self.world.updates;
        for i in 0..self.world.anims.len() {
            if self.world.anims[i].gone {
                continue;
            }
            // A character (M4 4d-3): its states drive the player, and only
            // while near or far (characters.md rule 11).
            let in_range = self.world.characters_in_range.contains(&self.world.anims[i].player.name.to_ascii_lowercase());
            let obj = &mut self.world.anims[i];
            let (a, notes) = match obj.character.as_deref_mut() {
                Some(c) => {
                    c.in_range = in_range;
                    let step = c.update(&mut obj.player, now, dt, update);
                    match step.played {
                        Some(a) => (a, step.notes),
                        None => continue,
                    }
                }
                None => (obj.player.advance(dt), Vec::new()),
            };
            let who = self.world.anims[i].player.name.clone();
            for n in notes {
                self.world.character_note(&who, n);
            }
            let p = &self.world.anims[i].player;
            let (name, at) = (p.name.clone(), p.placement().position);
            for cue in &a.cues {
                let what = self.world.cue_sound(&format!("object {name}"), cue, at);
                self.world.note(format!("object {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(st) = &a.started {
                let why = if a.finished.is_some() { "rule 8: the movement before it ended" } else { "rule 3: no current movement name" };
                let line = started_line(&format!("object {name}"), st, &self.world.anims[i].player.movements, why);
                self.world.note(line);
            }
            if let Some(m) = a.finished {
                let after = if a.started.is_some() { "the waiting movement plays" } else { "stays on its last frame" };
                self.world.note(format!("object {name}: movement '{m}' finished; {after}"));
                let w = &self.world;
                let has_script = w.anims[i].script.is_some() || w.clickables.iter().any(|c| c.script.is_some() && c.name.eq_ignore_ascii_case(&name));
                self.world.movement_ended(&name, &m, has_script);
            }
        }
        self.world.updates += 1;
    }

    /// Remarks and speech, once per update after the events (M4 4a-2,
    /// `talking.md` rules 7, 10-11): waits after a turn, lines ending and
    /// starting, mouth changes; then each finished line's `TalkComplete`.
    fn talk_step(&mut self, now: f64) {
        // Rule 10: captions follow the console variable (Options.cs).
        let captions = self.scripts.vars.get("Options::Captions") == Some("true");
        let mut talk = std::mem::take(&mut self.world.talk);
        let w = &self.world;
        let exists = |name: &str| w.known.contains(&name.to_ascii_lowercase()) && !w.is_gone(name);
        let finished = talk.update(now, captions, &exists);
        // Characters (M4 4d-5, characters.md rules 25-28): each mouth code
        // and line end; the last line of a queue sends the speaker to Wait
        // (Q117 stand-in).
        let mut last_speaker: BTreeMap<String, String> = BTreeMap::new();
        for e in &talk.events {
            match e {
                crate::talk::TalkEvent::Mouth { speaker, shape, .. } => {
                    if let Some(i) = self.world.character_index(speaker) {
                        let a = &mut self.world.anims[i];
                        let line = a.character.as_mut().unwrap().mouth_code(&mut a.player, *shape);
                        self.world.character_note(speaker, line);
                    }
                }
                crate::talk::TalkEvent::End { issuer, speaker, .. } => {
                    last_speaker.insert(issuer.to_ascii_lowercase(), speaker.clone());
                    if let Some(i) = self.world.character_index(speaker) {
                        let c = self.world.anims[i].character.as_mut().unwrap();
                        if !c.geometry && c.mouth != 0 {
                            c.mouth = 0;
                            self.world.character_note(speaker, "line ended: mouth shape 0 (rule 25)");
                        }
                    }
                }
                crate::talk::TalkEvent::Empty { issuer } => {
                    let Some(speaker) = last_speaker.get(&issuer.to_ascii_lowercase()).cloned() else { continue };
                    if let Some(i) = self.world.character_index(&speaker) {
                        let a = &mut self.world.anims[i];
                        let line = a.character.as_mut().unwrap().after_speech(&mut a.player);
                        self.world.character_note(&speaker, line);
                    }
                }
                _ => {}
            }
        }
        self.world.talk = talk;
        for f in finished {
            self.talk_complete(&f, now);
        }
    }

    /// Rule 13: after every line, `TalkComplete <file> <noun> <verb>
    /// <case> <sequence>` to the issuer's own script at once (never
    /// Connor's: he has none), and through the event queue to every
    /// object registered with `stateChangeNotify` for the issuer. A
    /// registration of the issuer for itself is skipped: that is the
    /// message coming back from the group ("Do not reproduce": the
    /// original decided this by a reused address and sometimes swallowed
    /// the next line's message; ours never does).
    fn talk_complete(&mut self, f: &Finished, now: f64) {
        let t = f.tuple;
        let data: Vec<String> = [t.file as u32, t.noun as u32, t.verb as u32, t.case as u32, t.seq as u32].iter().map(u32::to_string).collect();
        let mut args = vec!["TalkComplete"];
        args.extend(data.iter().map(String::as_str));
        let issuer = f.issuer.clone();
        let own = if issuer == CONNOR {
            "Connor has no script (rule 13)".to_string()
        } else if self.call_named(&issuer, &args) {
            "its own script ran".to_string()
        } else {
            "no script of its own".to_string()
        };
        let recipients = self.world.notify.get(&issuer.to_ascii_lowercase()).cloned().unwrap_or_default();
        let mut sent = Vec::new();
        for r in recipients {
            if r.eq_ignore_ascii_case(&issuer) {
                sent.push(format!("{r}: skipped (the issuer itself: the message coming back from its group)"));
                continue;
            }
            self.world.queue.send(now, 0.0, &r, "TalkComplete", &data, &format!("line #{} of {issuer}", f.id));
            sent.push(format!("{r}: queued"));
        }
        let line = format!(
            "{issuer}: TalkComplete {} (line #{}): {own}; notify group: {}",
            data.join(" "),
            f.id,
            if sent.is_empty() { "nobody".to_string() } else { sent.join(", ") }
        );
        self.world.talk.log.push(format!("t {now:8.3}: {line}"));
        self.world.note(line);
    }

    /// A `KQAnimOnce`'s movement ended (`anim-scripts.md` rule 11): its
    /// script gets `Die`, its listener `Cue 300 0 <movement>`, then it is
    /// deleted if `setDeleteWhenDone` asked. No `EndLoop`, no waiting
    /// movement.
    fn anim_once_ended(&mut self, i: usize, movement: &str) {
        let name = self.world.objects[i].name().to_string();
        self.world.note(format!("object {name} (KQAnimOnce): movement '{movement}' finished: its script gets Die {DIE_KIND} (rules 11, 20)"));
        self.call_object(i, &["Die", DIE_KIND]);
        self.world.report_end(&name, movement, false);
        if self.world.objects[i].delete_when_done {
            let at = format!("{name}: KQAnimOnce done");
            if self.world.delete_one(&name, &at) {
                self.world.note(format!("object {name} (KQAnimOnce): deleted (setDeleteWhenDone, rule 11.3)"));
            } else {
                self.world.note(format!("object {name} (KQAnimOnce): setDeleteWhenDone, but already deleted (by its script)"));
            }
        }
    }

    fn handle(&mut self, e: Pending) {
        let from = if e.from.is_empty() { String::new() } else { format!(" (sent by {})", e.from) };
        // sound.md rule 57f: a script's push or pop, now due.
        if e.target == MUSIC_TARGET {
            let name = e.data.first().cloned().unwrap_or_default();
            let req = if e.kind == "KQMusic::push" { MusicRequest::Push(name) } else { MusicRequest::Pop(name) };
            self.world.note(format!("{} now due{from}: handed to the music", req.describe()));
            self.world.music.push(req);
            return;
        }
        // talking.md rule 13: a notify group's TalkComplete, a moment after
        // the line ended.
        if e.kind == "TalkComplete" {
            let line = if self.world.is_gone(&e.target) {
                "deleted; dropped".to_string()
            } else {
                let mut args = vec!["TalkComplete"];
                args.extend(e.data.iter().map(String::as_str));
                if self.call_named(&e.target, &args) { "its script ran".to_string() } else { "no object with a script by that name; dropped".to_string() }
            };
            let line = format!("TalkComplete {} to {}{from}: {line}", e.data.join(" "), e.target);
            self.world.talk.log.push(format!("t {:8.3}: {line}", self.world.now()));
            self.world.note(line);
            return;
        }
        // clicking.md rule 21: reserved cue numbers.
        if e.kind == "KQCueEvent" && !self.world.is_gone(&e.target) {
            match e.data.first().map(String::as_str) {
                Some(CUE_END_LOOP) => {
                    let m = e.data.get(2).cloned().unwrap_or_default();
                    self.world.note(format!("KQCueEvent {CUE_END_LOOP} to {}{from}: EndLoop {m}", e.target));
                    if !self.call_named(&e.target, &["EndLoop", &m]) {
                        self.world.note(format!("{}: no object with a script by that name; EndLoop dropped", e.target));
                    }
                    return;
                }
                Some(CUE_PROJECTILE) => {
                    self.world.note(format!("KQCueEvent {CUE_PROJECTILE} to {}{from}: launches its projectile (combat, M5; not done)", e.target));
                    return;
                }
                _ => {}
            }
        }
        if let Some(i) = self.world.trap_index(&e.target) {
            let was = std::mem::replace(&mut self.world.in_trap, true);
            if e.kind == "KQCueEvent" {
                // traps.md rule 14, as doors (rule 23).
                let args = cue_args(&e.data);
                self.world.note(format!("KQCueEvent {} to trap {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_trap(i, &refs);
            } else {
                self.world.note(format!("event {} {} to trap {}{from}: not handled for traps", e.kind, e.data.join(" "), e.target));
            }
            self.world.in_trap = was;
            return;
        }
        if self.world.is_gone(&e.target) {
            self.world.note(format!("event {} {} to {}{from}: deleted; dropped", e.kind, e.data.join(" "), e.target));
            return;
        }
        if e.kind == "KQStatsChangeRequestEvent" {
            self.stats_change(&e, &from);
            return;
        }
        // anim-scripts.md rule 4 for the other animated objects (3i-4).
        if let Some(i) = self.world.anim_index(&e.target).filter(|_| e.kind == "KQAnimPlayEvent") {
            let line = anim_request(&mut self.world.anims[i].player, &e.data, &format!("object {}", e.target), &from);
            self.world.note(line);
            return;
        }
        if let Some(i) = self.world.clickables.iter().position(|c| c.script.is_some() && c.name.eq_ignore_ascii_case(&e.target)) {
            if e.kind == "KQCueEvent" {
                let args = cue_args(&e.data);
                self.world.note(format!("KQCueEvent {} to {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_clickable(i, &refs);
            } else {
                // Their animation belongs to the viewer's anim_view (3f).
                self.world.note(format!("event {} {} to {}{from}: not handled for this object (its animation isn't run here yet)", e.kind, e.data.join(" "), e.target));
            }
            return;
        }
        if let Some(i) = self.world.object_index(&e.target) {
            match e.kind.as_str() {
                "KQAnimPlayEvent" => {
                    let line = anim_request(&mut self.world.objects[i].player, &e.data, &format!("object {}", e.target), &from);
                    self.world.note(line);
                }
                "KQCueEvent" => {
                    let args = cue_args(&e.data);
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
                let args = cue_args(&e.data);
                self.world.note(format!("KQCueEvent {} to chest {}{from}", e.data.join(" "), e.target));
                let refs: Vec<&str> = args.iter().map(String::as_str).collect();
                self.call_chest(i, &refs);
            } else if e.kind == "KQAnimPlayEvent" {
                // anim-scripts.md rule 4: chests are ordinary animated objects.
                let line = anim_request(&mut self.world.chests[i].chest.player, &e.data, &format!("chest {}", e.target), &from);
                self.world.note(line);
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
                // anim-scripts.md rule 4 (doors.md rule 6: the state stays).
                let door = &mut self.world.doors[i].door;
                let what = format!("door {} (state {})", door.name, door.state.name());
                let line = anim_request(&mut door.player, &e.data, &what, &from);
                self.world.note(line);
            }
            "KQCueEvent" => {
                // Rule 23: Cue, $3, $4, and $5 only if non-zero.
                let args = cue_args(&e.data);
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
    const KEY: &str = "if test $2 == ConnorAction\n if test $4 == do\n  echo locked\n else if test $4 == INVITEM_Crystal\n  deleteObject $3\n  sendEvent 0 $1 KQDoorEvent 0\n  KQObject::collisionType $1 -32\n endif\n endif\nendif\n";
    const SWITCH: &str = "if test $3 == do\n if test $Sw::On == true\n  sendEvent 0 $1 KQDoorEvent 1\n  set Sw::On false\n else\n  sendEvent 0 $1 KQDoorEvent 2\n  set Sw::On true\n endif\nendif\n";
    const CUE: &str = "if test $2 == ConnorAction\n sendEvent .07 $1 KQCueEvent 1 0\nelse if test $2 == Cue\n echo cue $3 $4\nendif\nendif\n";

    fn door_box() -> Option<([f32; 3], [f32; 3])> {
        Some(([-630.0, -190.0, 0.0], [600.0, -60.0, 1730.0]))
    }

    pub(super) fn active(script: &'static str, vars: &[(&str, &str)]) -> Active {
        let mut d = ActiveDoor {
            door: test_door(None),
            script: Some("door.cs".into()),
            bounds: door_box(),
            collision_type: 0x5021,
            collision_mask: 0,
            accepts_inventory: Some("crystal".into()),
            accept_all_items: false,
            too_far_away_action: 0,
            do_not_point_at: false,
            gone: false,
        };
        d.door.player.base = Placement::at([0.0; 3]);
        let mut v = Vars::default();
        for (k, val) in vars {
            v.set(k, val.to_string());
        }
        let mut a = Active::new(v, vec![d], vec![], ["Connor".to_string()], [("door.cs".to_string(), script.as_bytes().to_vec())]);
        a.world.items = crate::item::tests::items();
        a
    }

    #[test]
    fn set_accepts_adds_to_what_the_door_takes() {
        let mut a = active(KEY, &[]);
        a.start();
        let coins = Item { object: "coins.1".into(), kind: "INVITEM_SilverCoins".into() };
        assert_eq!(a.click("Condoor", NEAR, 2350.0, Some(&coins)), ClickResult::Refused);
        a.world.sources.insert("s.cs".into(), b"KQObject::setAccepts Condoor silverCoins,LadyBell\nset R $Console::Return\nKQObject::setAccepts nobody all\n".to_vec());
        a.scripts.call("s.cs", &[], &mut a.world);
        assert_eq!(a.scripts.vars.get("R"), Some("0"));
        assert!(matches!(a.click("Condoor", NEAR, 2350.0, Some(&coins)), ClickResult::Script(_)));
        // The crystal from its file is still accepted (setAccepts adds).
        let crystal = Item { object: "c.1".into(), kind: "INVITEM_Crystal".into() };
        assert!(matches!(a.click("Condoor", NEAR, 2350.0, Some(&crystal)), ClickResult::Script(_)));
        let log = a.take_log().join("\n");
        assert!(log.contains("Condoor accepts INVITEM_Crystal"), "{log}");
        assert!(log.contains("now accepts INVITEM_Crystal,INVITEM_SilverCoins; skipped unknown [\"LadyBell\"]"), "{log}");
        assert!(log.contains("ERROR KQObject::setAccepts: unknown object"), "{log}");
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
        assert!(log.contains("cue 'playsound door4.aud' fires (sound started)"), "{log}");
        // sound.md rule 35: a placed one-shot at the door, 5,000 / 25,000.
        let door_at = a.world.doors[0].door.player.base.position;
        assert_eq!(a.world.sounds, [Start::placed_one_shot("door4.aud", door_at, "door Condoor cue".into())]);
        // Second click: script runs, event ignored, door stays open.
        a.click("Condoor", NEAR, 2350.0, None);
        a.update();
        assert_eq!(state(&a), DoorState::Opened);
        assert!(a.take_log().join("\n").contains("open: ignored while opened"));
    }

    /// Made-up door script that writes down every call.
    const LISTEN: &str = "if test $2 == Init\n getEndLoop $1 $1 1\nendif\nif test $2 == ConnorAction\n sendEvent 0 $1 KQDoorEvent 1\nendif\nif test $2 == Open\n set D::Open yes\nendif\nif test $2 == EndLoop\n set D::End $3\nendif\nif test $2 == Cue\n set D::Cue $3 $4 $5\nendif\n";

    #[test]
    fn a_door_gets_end_loop_after_open_and_get_end_loop_relays() {
        let mut a = active(LISTEN, &[]);
        a.start();
        a.click("Condoor", NEAR, 2350.0, None);
        while state(&a) != DoorState::Opened {
            a.update();
        }
        // Open at once; EndLoop and the listener's cue wait in the queue.
        assert_eq!(a.scripts.vars.get("D::Open"), Some("yes"));
        assert_eq!((a.scripts.vars.get("D::End"), a.scripts.vars.get("D::Cue")), (None, None));
        a.update();
        assert_eq!(a.scripts.vars.get("D::End"), Some("open"), "clicking.md rule 21: doors get EndLoop too");
        assert_eq!(a.scripts.vars.get("D::Cue"), Some("300 0 open"), "getEndLoop: Cue 300 0 <movement>");
        // getEndLoop ... 0 stops it; the door's own EndLoop stays.
        a.world.sources.insert("stop.cs".into(), b"getEndLoop Condoor Condoor 0\ngetEndLoop Condoor nobody 1\nset R $Console::Return\nsendEvent 0 Condoor KQCueEvent 500 0\nsendEvent 0 Condoor KQDoorEvent 0\n".to_vec());
        a.scripts.call("stop.cs", &[], &mut a.world);
        assert_eq!(a.scripts.vars.get("R"), Some("1"), "unknown object");
        a.scripts.vars.set("D::End", String::new());
        a.scripts.vars.set("D::Cue", String::new());
        for _ in 0..20 {
            a.update();
        }
        assert_eq!(state(&a), DoorState::Closed);
        assert_eq!(a.scripts.vars.get("D::End"), Some("close"));
        assert_eq!(a.scripts.vars.get("D::Cue"), Some(""), "no relay after getEndLoop 0, and cue 500 is no Cue call");
        let log = a.take_log().join("\n");
        assert!(log.contains("KQCueEvent 500 to Condoor") && log.contains("projectile"), "{log}");
    }

    #[test]
    fn a_switched_off_collider_is_not_clicked_and_scripts_ask_for_the_reference_point() {
        let mut a = active(PLAIN, &[]);
        assert!(a.switch_collider("condoor", false));
        assert!(!a.switch_collider("Condoor", false), "no change");
        assert_eq!(a.pick([0.0, -5000.0, 1000.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("Condoor", NEAR, 2350.0, None), ClickResult::Nothing);
        assert!(a.switch_collider("Condoor", true));
        assert!(a.pick([0.0, -5000.0, 1000.0], [0.0, 1.0, 0.0]).is_some());
        a.world.sources.insert("c.cs".into(), b"setActiveCamera 1\nmakeActive Condoor 1\nmakeActive nobody 1\nset R $Console::Return\nsetActiveCamera\nsetActiveCamera 0\n".to_vec());
        a.scripts.call("c.cs", &[], &mut a.world);
        assert_eq!(a.scripts.vars.get("R"), Some("1"), "unknown object");
        let r: Vec<String> = a.world.requests.iter().map(|r| format!("{r:?}")).collect();
        assert_eq!(r.len(), 3, "{r:?}");
        assert!(matches!(&a.world.requests[0], Request::ActiveCamera { camera: true, .. }));
        assert!(matches!(&a.world.requests[1], Request::MakeActive { name, on: true, .. } if name == "Condoor"));
        assert!(matches!(&a.world.requests[2], Request::ActiveCamera { camera: false, .. }));
        assert!(a.take_log().join("\n").contains("usage: setActiveCamera"));
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
        let key = Item { object: "key.1".into(), kind: "INVITEM_Crystal".into() };
        a.click("Condoor", NEAR, 2350.0, Some(&key));
        a.update();
        assert_eq!(state(&a), DoorState::Opening, "KQDoorEvent 0 toggles a closed door open");
        assert_eq!(a.world.doors[0].collision_type, 0x5001);
        assert_eq!(a.click("Condoor", NEAR, 2350.0, None), ClickResult::Nothing, "-32: no longer clickable");
        assert_eq!(a.pick([0.0, -5000.0, 1000.0], [0.0, 1.0, 0.0]), None);
        let log = a.take_log().join("\n");
        // key.1 is only the test's hand item, not an object: clicking.md rule 22's error line.
        assert!(log.contains("deleteObject: Object \"key.1\" doesn't exist"), "{log}");
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
        assert!(log.contains("KQStatsChangeRequestEvent 0 0 0 0 25 to Connor") && log.contains("Connor's stats unknown"), "{log}");
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
    fn a_script_s_move_out_of_a_trap_gives_off_at_once() {
        let mut a = with_trap("if test $2 == On\n set T::Who $3\nelse if test $2 == Off\n set T::Out $3\nendif\nendif\n");
        assert_eq!(connor_at(&mut a, 0.0), ["clifTrap On Connor"]);
        let balls = crate::collide::BallStack::from_box([-200.0, -200.0, 0.0], [200.0, 200.0, 1300.0]);
        let far = [20_000.0, 0.0, 0.0];
        let v = Visitor { name: "Connor", collision_type: CONNOR_TYPE, from: far, position: far, min: [-200.0, -200.0, 0.0], max: [200.0, 200.0, 1300.0], balls: &balls };
        assert_eq!(a.connor_placed(&v), ["clifTrap Off Connor"]);
        assert_eq!(a.scripts.vars.get("T::Out"), Some("Connor"));
        // Put back inside by a script: not entered until he moves.
        let v = Visitor { from: [0.0; 3], position: [0.0; 3], ..v };
        assert!(a.connor_placed(&v).is_empty());
        assert!(a.take_trap_log().join("\n").contains("Off Connor (stay test right after a script's move, rule 5)"));
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
        assert_eq!((v("First"), v("Gave"), v("NewInvItem")), (Some("NoName0".into()), Some("NoName2".into()), Some("NoName2".into())));
        let it = &a.world.items;
        let c = it.get("noname0").unwrap();
        assert_eq!((c.quantity, c.place.clone(), c.script.as_deref()), (3, Place::World { position: [1000.0, 0.0, 500.0], angles: [0.0, 0.0, 1.57] }, Some("coin.cs")));
        // z -1 is the floor height there.
        assert_eq!(it.get("NoName1").unwrap().place, Place::World { position: [2000.0, 0.0, 100.0], angles: [0.0; 3] });
        assert_eq!(it.count("Connor", "INVITEM_SilverCoins"), 5);
        assert_eq!(a.scripts.vars.get(crate::script::RETURN), Some("0"));
        assert!(a.world.known.contains("noname0"), "new items are known objects");
        assert!(a.world.item_log.iter().any(|l| l.contains("made NoName0: INVITEM_Crystal x3 in the world at (1000, 0, 500)")), "{:?}", a.world.item_log);
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
        assert_eq!((v("A"), v("Found")), (Some("yes"), Some("NoName0")));
        assert_eq!((v("B"), v("C"), v("D")), (Some("0"), Some("0"), Some("0")));
    }

    #[test]
    fn clicking_an_item_in_reach_picks_it_up_and_runs_its_script_once() {
        let mut a = with_items(MAKE);
        // NoName0 at (1000, 0, 500), box x -60..70 turned 90 degrees.
        let near = [1000.0, -3000.0, 500.0];
        assert_eq!(a.click("NoName0", [1000.0, -9000.0, 500.0], 2350.0, None), ClickResult::TooFar);
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("NoName0", near, 2350.0, Some(&bread)), ClickResult::Refused);
        assert!(a.world.items.get("NoName0").unwrap().in_world());
        // The pick ray finds it.
        assert_eq!(a.pick([1000.0, -5000.0, 600.0], [0.0, 1.0, 0.0]).map(|(n, _)| n), Some("NoName0".to_string()));
        assert_eq!(a.click("NoName0", near, 2350.0, None), ClickResult::PickedUp("NoName0".into()));
        let c = a.world.items.get("NoName0").unwrap();
        assert!(c.carried_by("Connor") && c.script.is_none(), "script removed by itself (setScript none)");
        assert_eq!(a.scripts.vars.get("Coin::Said"), Some("NoName0"));
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 3);
        let log = a.take_item_log().join("\n");
        assert!(log.contains("Connor's INVITEM_Crystal 0 -> 3; pickup sound energy.aud"), "{log}");
        assert!(a.world.sounds.iter().any(|s| s.name == "energy.aud" && s.placed.is_none()), "items.md rule 7: a plain sound");
        assert!(log.contains("script coin.cs NoName0 ConnorAction do do"), "{log}");
        // Gone from the world: not picked, a second click does nothing.
        assert_eq!(a.pick([1000.0, -5000.0, 600.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("NoName0", near, 2350.0, None), ClickResult::Nothing);
    }

    #[test]
    fn hidden_and_deleted_items_cant_be_clicked() {
        let mut a = with_items("KQMonster::inventory world Crystal 1 0 0 0\nKQObject::setHidden $NewInvItem 1\nKQMonster::inventory world Crystal 1 0 5000 0\ndeleteObject $NewInvItem\nKQObject::setDontLook NoName0 0\nKQObject::setHidden smlchst3 1\n");
        let it = &a.world.items;
        assert!(it.get("NoName0").unwrap().hidden && !it.get("NoName0").unwrap().do_not_point_at);
        assert_eq!(it.get("NoName1").unwrap().place, Place::Gone);
        assert_eq!(a.pick([0.0, -5000.0, 100.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("NoName0", [0.0, -500.0, 0.0], 2350.0, None), ClickResult::Nothing);
        // setHidden on another placed object (3k-2): here it isn't loaded.
        assert!(a.take_log().join("\n").contains("setHidden: unknown object 'smlchst3'"));
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
            gone: false,
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
        let crystal = a.world.items.get("NoName0").unwrap().clone();
        assert_eq!((crystal.quantity, crystal.place), (3, Place::World { position: [0.0, 0.0, 301.0], angles: [0.0, 0.0, -2.5] }));
        assert_eq!(a.world.chests[0].chest.remembered.as_deref(), Some("NoName0"));
        assert_eq!(a.scripts.vars.get("C::Clicked"), None, "rule 6: the script is not run on a click");
        // The crystal lies inside the chest's box: the ray picks the chest.
        assert_eq!(a.pick([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]).map(|(n, _)| n), Some("smlchst3".to_string()));
        // `open` ends after 1 s (started in update 1, moves in 2..16); its
        // cue 3333 goes through the queue: EndLoop open one update later.
        for _ in 0..16 {
            a.update();
        }
        assert_eq!(a.scripts.vars.get("C::EndLoop"), None);
        a.update();
        assert_eq!(a.scripts.vars.get("C::EndLoop"), Some("open"));
        let log = a.take_log().join("\n");
        assert!(log.contains("cue 'playsound chstopen' fires (sound started)") && log.contains("script chest.cs smlchst3 EndLoop open"), "{log}");
        assert!(a.world.sounds.iter().any(|s| s.name == "chstopen" && s.placed.is_some_and(|p| p.min == 5_000.0 && p.max == 25_000.0)));
        // 2nd click: into Connor's inventory.
        let r = a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(r, ClickResult::Chest { before: ChestState::Open, after: ChestState::Empty });
        assert!(a.world.items.get("NoName0").unwrap().carried_by("Connor"));
        assert_eq!(a.world.items.count("Connor", "INVITEM_Crystal"), 3);
        assert_eq!(a.world.chests[0].chest.remembered, None);
        // 3rd click: the Empty remark, nothing else. Connor is not in Wait
        // or Move here (the test's default): he still turns to the chest
        // and the remark waits 0.5 s (Q103 e).
        a.world.requests.clear();
        assert!(!a.world.connor_can_turn);
        let r = a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert_eq!(r, ClickResult::Chest { before: ChestState::Empty, after: ChestState::Empty });
        assert!(matches!(&a.world.requests[..], [Request::ConnorAuto { auto, .. }] if auto.target == [0.0, 0.0] && auto.cue.is_none()));
        let log = a.take_log().join("\n");
        assert!(log.contains("Empty remark 0 109 14 33 1…1: Connor (not Wait or Move) turns to face the chest"), "{log}");
        assert!(log.contains("waits 0.5 s"), "{log}");
        assert_eq!(a.world.items.objects.len(), 1, "no second item");
        // With doNotPointAt: no turn, at once.
        a.world.requests.clear();
        a.world.chests[0].do_not_point_at = true;
        a.world.talk.clear("Connor", a.world.now());
        a.click("smlchst3", BY_CHEST, 2350.0, None);
        assert!(a.world.requests.is_empty());
        assert!(a.take_log().join("\n").contains("doNotPointAt: no turn"));
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
        assert_eq!(a.click("NoName0", BY_CHEST, 2350.0, None), ClickResult::PickedUp("NoName0".into()));
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

    /// A building wall: the plane y = `y` of building `Pub`, entered by
    /// every line crossing it, if `clickable`.
    struct Wall {
        y: f32,
        clickable: bool,
    }

    impl ClickWalls for Wall {
        fn first_hit(&self, from: [f32; 3], to: [f32; 3]) -> Option<(String, f32)> {
            let (a, b) = (from[1] - self.y, to[1] - self.y);
            if !self.clickable || a * b > 0.0 || a == b {
                return None;
            }
            let len = (0..3).map(|k| (to[k] - from[k]).powi(2)).sum::<f32>().sqrt();
            Some(("Pub".into(), a / (a - b) * len))
        }
    }

    #[test]
    fn a_clickable_wall_in_front_takes_the_click() {
        let a = with_chest(Some("Crystal"));
        // The chest's box starts at y -310; the camera looks along +y.
        let (from, dir) = ([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]);
        let chest = Some(("smlchst3".to_string(), 4690.0));
        assert_eq!(a.pick_line(from, dir, &Wall { y: -3000.0, clickable: true }), Pick::Building { name: "Pub".into(), t: 2000.0, behind: chest });
        // A wall behind the camera or behind the chest: the chest.
        assert_eq!(a.pick_line(from, dir, &Wall { y: -6000.0, clickable: true }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        assert_eq!(a.pick_line(from, dir, &Wall { y: 0.0, clickable: true }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        // A building that isn't clickable (colliderType 136) is not tested.
        assert_eq!(a.pick_line(from, dir, &Wall { y: -3000.0, clickable: false }), Pick::Seen { name: "smlchst3".into(), t: 4690.0 });
        // Looking away: only a wall can be hit.
        match a.pick_line(from, [0.0, -1.0, 0.0], &Wall { y: -6000.0, clickable: true }) {
            Pick::Building { name, t, behind: None } => assert!(name == "Pub" && (t - 1000.0).abs() < 0.01, "{t}"),
            other => panic!("{other:?}"),
        }
        assert_eq!(a.pick_line(from, [0.0, -1.0, 0.0], &Wall { y: -3000.0, clickable: true }), Pick::Nothing);
    }

    fn building(script: Option<&str>, collision_type: u32) -> ActiveBuilding {
        ActiveBuilding {
            name: "Pub".into(),
            collision_type,
            script: script.map(String::from),
            accepts_inventory: None,
            accept_all_items: false,
            too_far_away_action: BUILDING_TOO_FAR_AWAY_ACTION,
            do_not_point_at: false,
            bounds: ([0.0, 0.0, 0.0], [1000.0, 1000.0, 1000.0]),
            placement: Placement::at([0.0; 3]),
        }
    }

    #[test]
    fn reach_to_a_building_is_in_its_own_axes() {
        // The 1000-unit box turned 45 degrees about z: its corner (1000, 0)
        // goes to (707, 707), (0, 1000) to (-707, 707). The point (650, 50)
        // is inside the axis-aligned box around it, but about 424 outside
        // the turned box itself (rule 9).
        let mut a = with_building(None, 0x28);
        let c = std::f32::consts::FRAC_1_SQRT_2;
        a.world.buildings[0].placement = Placement { rotation: [[c, c, 0.0], [-c, c, 0.0], [0.0, 0.0, 1.0]], position: [0.0; 3] };
        let feet = [650.0, 50.0, 10.0];
        let own = a.world.buildings[0].placement.to_own(feet);
        assert!(own[1] < 0.0, "outside the turned box: {own:?}");
        // Reach 400: out; reach 450: in ("that doesn't work" with an item).
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("Pub", feet, 200.0, Some(&bread)), ClickResult::Swallowed);
        assert_eq!(a.click("Pub", feet, 225.0, Some(&bread)), ClickResult::Refused);
        let log = a.take_log().join("
");
        assert!(log.contains("distance to its .dig box (its own axes) 424.3"), "{log}");
    }

    fn with_building(script: Option<&str>, collision_type: u32) -> Active {
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("b.cs".to_string(), b"set B::Got $2 $3\n".to_vec())]);
        a.add_buildings([building(script, collision_type)]);
        a
    }

    #[test]
    fn a_building_without_a_script_swallows_the_click() {
        let mut a = with_building(None, 0x28);
        assert!(a.clickable_building(0));
        // In reach, and out of reach (tooFarAwayAction 1): swallowed.
        assert_eq!(a.click("Pub", [500.0, -100.0, 0.0], 2350.0, None), ClickResult::Swallowed);
        assert_eq!(a.click("Pub", [500.0, -9000.0, 0.0], 2350.0, None), ClickResult::Swallowed);
        // An item in hand, in reach: "that doesn't work" (rule 12).
        let bread = Item { object: "bread.1".into(), kind: "INVITEM_Bread".into() };
        assert_eq!(a.click("Pub", [500.0, -100.0, 0.0], 2350.0, Some(&bread)), ClickResult::Refused);
        let log = a.take_log().join("\n");
        assert!(log.contains("swallowed, no script"), "{log}");
        // With a script it gets the calls as a door (DeadCity's doorways).
        let mut a = with_building(Some("b.cs"), 0x28);
        let call = ["Pub", "ConnorAction", "TooFar", "TooFar"].map(String::from).to_vec();
        assert_eq!(a.click("Pub", [500.0, -9000.0, 0.0], 2350.0, None), ClickResult::Script(call));
        assert_eq!(a.scripts.vars.get("B::Got"), Some("ConnorAction TooFar"));
        assert!(a.world.requests.iter().any(|r| matches!(r, Request::ClickTurn { .. })), "a building with a script turns Connor");
        // clicking.md rule 36 (Q105 c): its doNotPointAt is read.
        let mut a = with_building(Some("b.cs"), 0x28);
        a.world.buildings[0].do_not_point_at = true;
        a.click("Pub", [500.0, -9000.0, 0.0], 2350.0, None);
        assert!(!a.world.requests.iter().any(|r| matches!(r, Request::ClickTurn { .. })));
        assert!(a.take_log().join("\n").contains("doNotPointAt: no turn"));
        // colliderType 136: not clickable.
        let mut a = with_building(None, 0x88);
        assert!(!a.clickable_building(0));
        assert_eq!(a.click("Pub", [500.0, -100.0, 0.0], 2350.0, None), ClickResult::Nothing);
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
        let mut player = crate::named_anim::tests::still_then_open("basket1");
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
            gone: false,
            delete_when_done: false,
        }]);
        a.start();
        a
    }

    #[test]
    fn a_kq_anim_once_reports_die_tells_its_listener_and_is_deleted() {
        // Made-up script: on a click, ask to be deleted when done, have
        // Connor listen, and play `open` (16 frames over 1 s).
        let script = "if test $2 == ConnorAction\n setDeleteWhenDone $1 1\n getEndLoop $1 Connor 1\n sendEvent 0 $1 KQAnimPlayEvent open\nendif\nif test $2 == Die\n set D::died $1 $3\nendif\n";
        let mut a = with_object(script, "x");
        // Not a KQAnimOnce yet: setDeleteWhenDone finds nothing (rule 13).
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(a.take_log().join("\n").contains("setDeleteWhenDone: could not find object 'basket1'"));
        a.world.objects[0].player.once = true;
        a.world.objects[0].player.start("stop");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(a.world.objects[0].delete_when_done);
        let mut log = String::new();
        for _ in 0..20 {
            a.update();
            log.push_str(&a.take_log().join("\n"));
        }
        assert_eq!(a.scripts.vars.get("D::died"), Some("basket1 Normal"), "{log}");
        assert!(log.contains("tells Connor (getEndLoop: Cue 300 0 open)"), "{log}");
        assert!(!log.contains("EndLoop open"), "no EndLoop for a KQAnimOnce: {log}");
        assert!(a.world.is_deleted("basket1") && a.world.objects[0].gone, "{log}");
    }

    #[test]
    fn pause_anim_toggles_and_set_anim_pos_sets_the_phase() {
        let script = "if test $2 == ConnorAction\n sendEvent 0 $1 KQAnimPlayEvent open\n pauseAnim $1\n pauseAnim nosuch\nendif\nif test $2 == Cue\n pauseAnim $1\n setAnimPos $1 0.5\nendif\n";
        let mut a = with_object(script, "x");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.take_log().join("\n");
        assert!(log.contains("pauseAnim: basket1, 0: paused (rule 14)"), "{log}");
        assert!(log.contains("pauseAnim:  could not find anim nosuch"), "{log}");
        for _ in 0..30 {
            a.update();
        }
        let p = &a.world.objects[0].player;
        assert_eq!((p.movement_name(), p.phase, p.paused), ("open", 0.0, true), "paused: the request started it but it doesn't move");
        a.world.queue.send(a.world.now(), 0.0, "basket1", "KQCueEvent", &["1".into()], "test");
        a.update();
        let log = a.take_log().join("\n");
        assert!(log.contains("resumed (rule 14)") && log.contains("setAnimPos: basket1, 0.5"), "{log}");
        a.update();
        assert!(a.world.objects[0].player.phase > 0.5);
    }

    #[test]
    fn scripts_reach_the_other_animated_objects() {
        let script = "if test $2 == ConnorAction\n sendEvent 0 water KQAnimPlayEvent blocked\n pauseAnim water\n pauseAnim water\nendif\n";
        let mut a = with_object(script, "x");
        let mut player = crate::named_anim::tests::still_then_open("water");
        player.movements[1].name = "blocked".into();
        a.add_anims([AnimObject { player, script: None, gone: false, character: None, translucency: None }]);
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        a.update();
        let log = a.take_log().join("\n");
        assert!(log.contains("pauseAnim: water, 0: paused") && log.contains("pauseAnim: water, 0: resumed"), "{log}");
        assert!(log.contains("KQAnimPlayEvent blocked to object water"), "{log}");
        assert_eq!(a.world.anims[0].player.movement_name(), "blocked");
        let mut log = String::new();
        for _ in 0..20 {
            a.update();
            log.push_str(&a.take_log().join("\n"));
        }
        assert!(log.contains("cue 'playsound chstopen' fires") && log.contains("object water: movement 'blocked' finished"), "{log}");
        let mut d = with_object("deleteObject water\n", "x");
        d.add_anims([AnimObject { player: crate::named_anim::tests::still_then_open("water"), script: None, gone: false, character: None, translucency: None }]);
        d.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(d.world.anims[0].gone && d.world.is_deleted("water"));
    }

    #[test]
    fn make_translucent_sets_the_value_on_an_animated_object() {
        let mut a = with_object("makeTranslucent water 150\nmakeTranslucent nobody 150\n", "x");
        a.add_anims([AnimObject { player: crate::named_anim::tests::still_then_open("water"), script: None, gone: false, character: None, translucency: None }]);
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert_eq!(a.world.anims[0].translucency, Some(150.0));
        let log = a.take_log().join("\n");
        assert!(log.contains("opacity 0.588, at once") && log.contains("makeTranslucent nobody 150: not an animated object"), "{log}");
    }

    #[test]
    fn stats_events_add_and_an_object_below_0_gets_die() {
        let script = "if test $2 == ConnorAction\n sendEvent 0 Connor KQStatsChangeRequestEvent 2 0 0 0 50\n sendEvent 0 $1 KQStatsChangeRequestEvent -3 0 0 0 0\nendif\nif test $2 == Die\n set D::died $1 $3\nendif\n";
        let mut a = with_object(script, "x");
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.set_stats("basket1", Stats([2, 0, 0, 0, 0]));
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        a.update();
        let log = a.take_log().join("\n");
        assert_eq!(a.world.connor_stats.map(|c| (c.stats.0, c.level)), Some(([20, 15, 2, 1, 40], 2)), "{log}");
        assert_eq!(a.world.stats.get("basket1"), Some(&Stats([-1, 0, 0, 0, 0])));
        assert_eq!(a.scripts.vars.get("D::died"), Some("basket1 Normal"), "{log}");
    }

    #[test]
    fn connor_stats_events_are_counted_and_panel_commands_handed_over() {
        let mut a = with_items("setLowerGui 0
setUpperGui 1
setMapGui
set R $Console::Return
getGuiStatus
");
        use crate::panels::Group;
        assert_eq!(
            a.world.gui_orders,
            vec![
                GuiOrder::Set { group: Group::Bottom, on: false, at: "s.cs line 1".into() },
                GuiOrder::Set { group: Group::Top, on: true, at: "s.cs line 2".into() },
                GuiOrder::Status { at: "s.cs line 5".into() },
            ]
        );
        assert_eq!(a.scripts.vars.get("R"), Some("1"), "setMapGui without 0|1: refused");
        // interface.md rule 24: every stats event on Connor counts, even
        // all zeros.
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        let e = |d: &[&str]| Pending { due: 0.0, seq: 0, target: "Connor".into(), kind: "KQStatsChangeRequestEvent".into(), data: d.iter().map(|s| s.to_string()).collect(), from: "test".into() };
        a.stats_change(&e(&["0", "0", "0", "0", "0"]), "");
        a.stats_change(&e(&["-2", "0", "0", "0", "0"]), "");
        assert_eq!((a.world.connor_stats_events, a.world.connor_stats.map(|c| c.stats.0[0])), (2, Some(8)));
    }

    #[test]
    fn set_state_connor_special_is_handed_over_and_his_ends_are_relayed() {
        let script = "if test $2 == ConnorAction\n getEndLoop Connor $1 1\n KQMonster::setState Connor special 'AnimList=bricks.anm,AnimListOn,push'\n KQMonster::setState ogre attack\nendif\nif test $2 == Cue\n set C::heard $5\nendif\n";
        let mut a = with_object(script, "x");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert_eq!(a.world.requests.iter().filter(|r| matches!(r, Request::ConnorSpecial { items, .. } if items == "AnimList=bricks.anm,AnimListOn,push")).count(), 1);
        assert!(a.take_log().join("\n").contains("stub (later milestone, nothing done): KQMonster::setState ogre attack"));
        a.connor_movement_ended("push");
        a.update();
        assert_eq!(a.scripts.vars.get("C::heard"), Some("push"));
    }

    #[test]
    fn a_scripted_pushable_gets_push_and_takes_control_or_becomes_a_wall() {
        // Made-up script in the style of the push scripts: removes its
        // push and click bits once.
        let mut a = with_object("if test $2 == Push\n set P::n 1\n KQObject::collisionType $1 -288\nendif\n", "x");
        a.world.objects[0].collision_type = 0x10121;
        assert!(a.push_call("basket1"));
        assert_eq!((a.scripts.vars.get("P::n"), a.world.objects[0].collision_type), (Some("1"), 0x10001));
        // A script that does nothing: still pushable, so a wall from now on.
        let mut b = with_object("echo nothing\n", "x");
        b.world.objects[0].collision_type = 0x10121;
        assert!(!b.push_call("basket1"));
    }

    #[test]
    fn move_sets_or_adds_along_the_world_axes() {
        let p = Placement { rotation: kq8_data::objects::rotation([0.0, 0.0, 1.57]), position: [100.0, 200.0, 300.0] };
        // Flags 3: both relative; a turn of 0 keeps the turn.
        let (q, note) = moved_placement(p, 3, [0.0, -200.0, -300.0], [0.0; 3]);
        assert_eq!((q.position, q.rotation, note), ([100.0, 0.0, 0.0], p.rotation, None));
        // Flags 0: both absolute.
        let (q, _) = moved_placement(p, 0, [1.0, 2.0, 3.0], [0.0, 0.0, 0.5]);
        assert_eq!((q.position, q.rotation), ([1.0, 2.0, 3.0], kq8_data::objects::rotation([0.0, 0.0, 0.5])));
        // Flags 2: relative turn about Z adds.
        let (q, _) = moved_placement(p, 2, [1.0, 2.0, 3.0], [0.0, 0.0, 0.43]);
        let want = kq8_data::objects::rotation([0.0, 0.0, 2.0]);
        assert!((0..3).all(|i| (0..3).all(|j| (q.rotation[i][j] - want[i][j]).abs() < 1e-5)));
        // In a script: a scripted object moves at once, Connor by request.
        let mut a = with_object("move $1 3 0 -200 -300\nmove Connor 0 1 2 3 0 0 1.57\nmove nobody 0 1 2 3\n", "x");
        let before = a.world.objects[0].player.base.position;
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert_eq!(a.world.objects[0].player.base.position, [before[0], before[1] - 200.0, before[2] - 300.0]);
        assert!(a.world.requests.iter().any(|r| matches!(r, Request::Move { who, flags: 0, .. } if who == "Connor")));
        assert!(a.take_log().join("\n").contains("move: unknown object 'nobody'"));
    }

    #[test]
    fn walk_to_point_and_cue_and_walk_exit_become_requests() {
        use crate::connor_auto::{AutoKind, Cue};
        let mut a = with_object(
            "KQConner::walkTo 190800 230100 $1 2 0\nKQConner::runTo 1 2\npointAndCue 188000 230000 9000 $1 3\nwalkExit\nKQConner::walkTo 5\nKQConner::walkTo 1 2 $1 4\n",
            "x",
        );
        // The helper's start ran the script once already (Init).
        a.world.requests.clear();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let autos: Vec<_> = a
            .world
            .requests
            .iter()
            .filter_map(|r| if let Request::ConnorAuto { auto, .. } = r { Some(auto.clone()) } else { None })
            .collect();
        assert_eq!(autos.len(), 4);
        // An object without both cue numbers: a console line, no cue.
        assert_eq!(autos[3].cue, None);
        assert_eq!((autos[0].kind, autos[0].target), (AutoKind::Walk { run: false }, [190800.0, 230100.0]));
        assert_eq!(autos[0].cue, Some(Cue { who: "basket1".into(), c: "2".into(), d: "0".into() }));
        // No object named: no cue.
        assert_eq!((autos[1].kind, autos[1].cue.clone()), (AutoKind::Walk { run: true }, None));
        // pointAndCue: z ignored; a missing data word is 0.
        assert_eq!((autos[2].kind, autos[2].target), (AutoKind::Point, [188000.0, 230000.0]));
        assert_eq!(autos[2].cue.as_ref().map(|c| c.d.as_str()), Some("0"));
        assert!(a.world.requests.iter().any(|r| matches!(r, Request::WalkExit { .. })));
        let log = a.take_log().join("\n");
        assert!(log.contains("usage: KQConner::walkTo x y") && log.contains("must have objName"), "{log}");
    }

    #[test]
    fn set_third_person_flips_the_key_lock_and_hands_off_logs_its_frames() {
        let mut a = with_object("setThirdPerson\nhandsOff 1 2\nsetThirdPerson\nhandsOff 0\n", "x");
        a.take_log();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.take_log().join("\n");
        let locked = log.find("first-person key is now locked").expect(&log);
        let allowed = log.find("first-person key is now allowed").expect(&log);
        assert!(locked < allowed, "{log}");
        assert!(log.contains("at most (2 + 1) / 15 s"), "{log}");
        assert!(log.contains("held movement keys sent again"), "{log}");
    }

    #[test]
    fn get_do_mode_answers_empty_hands() {
        // miltrap1.cs: the "no weapon out" branch, not the bow's.
        let script = "if test $2 == ConnorAction
 KQConner::getDoMode
 set M::mode $Console::Return
endif
";
        let mut a = with_object(script, "x");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert_eq!(a.scripts.vars.get("M::mode"), Some("4"));
    }

    #[test]
    fn jump_dir_and_end_jump_answer_from_connors_jump() {
        // miltrap1.cs's two calls (anim-scripts.md rules 28-29).
        let script = "if test $2 == ConnorAction
 KQConner::JumpDir
 set M::dir $Console::Return
 KQConner::endJump
 set M::end $Console::Return
endif
";
        let run = |jump: Option<&'static str>| {
            let mut a = with_object(script, "x");
            a.world.connor_jump = jump;
            a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
            let ends = a.world.requests.iter().filter(|r| matches!(r, Request::EndJump { .. })).count();
            (a.scripts.vars.get("M::dir").map(str::to_string), a.scripts.vars.get("M::end").map(str::to_string), ends, a.take_log().join("\n"))
        };
        let (dir, end, ends, _) = run(Some("backJump"));
        assert_eq!((dir.as_deref(), end.as_deref(), ends), (Some("1"), Some("1"), 1));
        for jump in ["jump", "runJump"] {
            let (dir, end, ends, _) = run(Some(jump));
            assert_eq!((dir.as_deref(), end.as_deref(), ends), (Some("0"), Some("1"), 1), "{jump}");
        }
        // Outside a jump: 0, a console line each, no request.
        let (dir, end, ends, log) = run(None);
        assert_eq!((dir.as_deref(), end.as_deref(), ends), (Some("0"), Some("0"), 0));
        assert!(log.contains("returns 0 if forward, 1 if backward") && log.contains("use when Connor is in the jump state"), "{log}");
    }

    #[test]
    fn cam_loc_fly_and_cut_become_requests() {
        // wtrmwalk.cs and wheeltp1.cs lines, and the end roll rule.
        let mut a = with_object(
            "KQCamera::addCamLoc 190000 230000 10600 1\nKQCamera::flyToCamLoc 186960 228000 10000 2\nKQCamera::flyToCamLoc 1 2 3 4 0.5\nKQCamera::flyToCamLoc 1 2 3\nKQCamera::cutToCamLoc 1 2 3 0.25\nKQCamera::addCamLoc 5 6\n",
            "x",
        );
        a.world.requests.clear();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let r = &a.world.requests;
        // The click's own turn comes after the script's requests
        // (clicking.md rule 27).
        assert_eq!(r.len(), 6, "{r:?}");
        assert!(matches!(&r[5], Request::ClickTurn { name, .. } if name == "basket1"));
        assert!(matches!(&r[0], Request::CamSpot { spot, newlist: true, .. } if *spot == [190000.0, 230000.0, 10600.0]));
        assert!(matches!(&r[1], Request::CamFly { seconds: Some(s), end_roll: None, .. } if *s == 2.0));
        assert!(matches!(&r[2], Request::CamFly { end_roll: Some(e), .. } if *e == 0.5));
        assert!(matches!(&r[3], Request::CamFly { seconds: None, end_roll: None, .. }));
        assert!(matches!(&r[4], Request::CamCut { roll, .. } if *roll == 0.25));
        assert!(a.take_log().join("\n").contains("usage: KQCamera::addCamLoc x y z"));
    }

    #[test]
    fn set_hidden_on_a_plain_placed_object_overrides_its_file() {
        let mut a = with_object("KQObject::setHidden Table 1\nKQObject::setHidden nosuch 1\n", "x");
        a.world.known.insert("table".into());
        assert!(!a.world.hidden_now("Table", false));
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(a.world.hidden_now("Table", false));
        assert!(a.take_log().join("\n").contains("setHidden: unknown object 'nosuch'"));
        let mut b = with_object("KQObject::setHidden Table 0\n", "x");
        b.world.known.insert("table".into());
        b.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(!b.world.hidden_now("Table", true), "a script's 0 wins over hidden=1");
    }

    #[test]
    fn load_kq_while_playing_brings_a_prepared_copy_in_once() {
        let mut a = with_object("loadKQ waterfa2.kq\nloadKQ waterfa2.kq\nloadKQ other.kq\nloadKQ x.kq name 1 2 3\n", "x");
        a.set_dormant([("WATERFA2.KQ".to_string(), "waterfall2".to_string(), None)]);
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let loads: Vec<&Request> = a.world.requests.iter().filter(|r| matches!(r, Request::Load { .. })).collect();
        assert_eq!(loads.len(), 1);
        assert!(a.world.known.contains("waterfall2"));
        let log = a.take_log().join("\n");
        assert!(log.contains("loadKQ waterfa2.kq: no copy prepared") && log.contains("loadKQ other.kq: no copy prepared") && log.contains("a name or position while playing"), "{log}");
    }

    #[test]
    fn load_kq_while_playing_calls_init_at_once_and_refuses_a_name_in_use() {
        // objects.md "Where a world's objects come from" rule 6 (Q90 b, d).
        let mut a = with_object("loadKQ new.kq\nset T::after $W::init\nloadKQ dup.kq\n", "x");
        a.world.sources.insert("new.cs".into(), b"if test $2 == Init\n set W::init $1\nendif\n".to_vec());
        a.world.sources.insert("again.cs".into(), b"loadKQ dup.kq\n".to_vec());
        a.set_dormant([
            ("new.kq".to_string(), "newthing".to_string(), Some("new.cs".to_string())),
            ("dup.kq".to_string(), "basket1".to_string(), None),
        ]);
        a.world.known.insert("basket1".into());
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        // Init ran during the loadKQ line, before the next line.
        assert_eq!(a.scripts.vars.get("T::after"), Some("newthing"));
        let log = a.take_log().join("\n");
        assert!(log.contains("ERROR loadKQ dup.kq: a live object already has that name (basket1)"), "{log}");
        assert!(!a.world.requests.iter().any(|r| matches!(r, Request::Load { name, .. } if name == "basket1")));
        // Deleted in the same event: still refused; free in the next one.
        let mut b = with_object("if test $2 == ConnorAction\n deleteObject basket1\n loadKQ dup.kq\nendif\n", "x");
        b.world.sources.insert("again.cs".into(), b"loadKQ dup.kq\n".to_vec());
        b.set_dormant([("dup.kq".to_string(), "basket1".to_string(), None)]);
        b.world.known.insert("basket1".into());
        b.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(b.take_log().join("\n").contains("deleted in this same event"));
        b.scripts.call("again.cs", &[], &mut b.world);
        assert!(b.world.requests.iter().any(|r| matches!(r, Request::Load { name, .. } if name == "basket1")));
    }

    /// A made-up script in the style of `fishies.cs` (not a copy).
    const FISH: &str = "if test $2 == Init\n set F::init $1\nendif\nif test $2 == ConnorAction\n if test $F::full == true\n  set F::full false\n  sendEvent 1.0 $1 KQCueEvent 1 0\n  deleteObject $1\n  KQSound::play eat.aud\n  sendEvent 0 Connor KQStatsChangeRequestEvent 2 0 0 0 0\n endif\nendif\n";

    fn with_clickables(script: Option<&str>) -> Active {
        let mut fish = Clickable::plain("fishies.01", "KQAnimObject", [-200.0, -200.0, 0.0], [200.0, 200.0, 300.0], Placement::at([0.0; 3]), 33);
        fish.script = script.map(|_| "fish.cs".to_string());
        let sources = [("fish.cs".to_string(), FISH.as_bytes().to_vec())];
        let mut v = Vars::default();
        v.set("F::full", "true".into());
        let mut a = Active::new(v, vec![], vec![fish], ["Connor".to_string()], sources);
        a.start();
        a
    }

    #[test]
    fn a_scripted_clickable_eats_itself_once() {
        let mut a = with_clickables(Some("fish.cs"));
        assert_eq!(a.scripts.vars.get("F::init"), Some("fishies.01"), "Init (Q64 c stand-in)");
        let near = [0.0, -1000.0, 0.0];
        let r = a.click("fishies.01", near, 2350.0, None);
        assert_eq!(r, ClickResult::Script(["fishies.01", "ConnorAction", "do", "do"].map(String::from).to_vec()));
        assert!(a.world.clickables[0].gone);
        // It still turns Connor: the removal waits for the end of the
        // event (clicking.md rule 22; our reading, Q105).
        assert!(matches!(&a.world.requests[..], [Request::Delete { name, .. }, Request::ClickTurn { name: t, .. }] if name == "fishies.01" && t == "fishies.01"));
        let log = a.take_log().join("\n");
        assert!(log.contains("KQSound::play eat.aud: plain sound started"), "{log}");
        assert!(matches!(&a.world.sounds[..], [s] if s.name == "eat.aud" && s.placed.is_none() && !s.looping && s.group == 0));
        assert!(log.contains("deleteObject fishies.01: clickable object deleted"), "{log}");
        // Gone: not picked, a click does nothing, its cue (1 s later) is dropped.
        assert_eq!(a.pick([0.0, -5000.0, 100.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("fishies.01", near, 2350.0, None), ClickResult::Nothing);
        for _ in 0..20 {
            a.update();
        }
        let log = a.take_log().join("\n");
        assert!(log.contains("KQStatsChangeRequestEvent 2 0 0 0 0 to Connor") && log.contains("Connor's stats unknown"), "{log}");
        assert!(log.contains("event KQCueEvent 1 0 to fishies.01 (sent by fish.cs line 7): deleted; dropped"), "{log}");
    }

    #[test]
    fn a_click_call_asks_for_a_turn_to_the_object_after_the_script() {
        // clicking.md rules 25-29: after the ConnorAction call, a turn to
        // the object's placement point; not with doNotPointAt; not for a
        // stock remark.
        let mut a = with_object("set Clicked $3
", "x");
        a.world.requests.clear();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(matches!(&a.world.requests[..], [Request::ClickTurn { name, target, at }] if name == "basket1" && *target == [0.0, 0.0] && at.contains("(do)")));
        let log = a.take_log().join("
");
        assert!(log.contains("click basket1: call do: turn to basket1 at (0, 0) asked for"), "{log}");
        a.world.requests.clear();
        a.world.objects[0].do_not_point_at = true;
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(a.world.requests.is_empty());
        assert!(a.take_log().join("
").contains("call do: doNotPointAt: no turn (rule 29)"));
        // Out of reach, tooFarAwayAction 0: the stock remark, no call.
        a.world.objects[0].do_not_point_at = false;
        assert_eq!(a.click("basket1", [0.0, -90000.0, 0.0], 2350.0, None), ClickResult::TooFar);
        assert!(a.world.requests.is_empty());
        assert!(a.take_log().join("
").contains("no call: no turn (clicking.md rule 26)"));
        // tooFarAwayAction 1: the TooFar call turns him from any distance.
        a.world.objects[0].too_far_away_action = 1;
        a.click("basket1", [0.0, -90000.0, 0.0], 2350.0, None);
        assert!(matches!(&a.world.requests[..], [Request::ClickTurn { at, .. }] if at.contains("(TooFar)")));
    }

    #[test]
    fn a_picked_up_item_does_not_turn_him_by_default() {
        // clicking.md rule 29: items default to doNotPointAt 1.
        let mut a = with_items(MAKE);
        a.world.requests.clear();
        a.click("NoName0", [1000.0, -3000.0, 500.0], 2350.0, None);
        assert!(!a.world.requests.iter().any(|r| matches!(r, Request::ClickTurn { .. })));
        assert!(a.take_log().join("
").contains("click NoName0: call do: doNotPointAt: no turn (rule 29)"));
    }

    #[test]
    fn a_clickable_without_a_script_is_only_logged() {
        let mut a = with_clickables(None);
        assert_eq!(a.click("fishies.01", [0.0, -1000.0, 0.0], 2350.0, None), ClickResult::NotADoor);
        assert!(a.world.requests.is_empty());
    }

    #[test]
    fn delete_object_on_a_scripted_object_and_on_other_objects() {
        let script = "if test $2 == ConnorAction\n deleteObject $1\n deleteObject keepwall\n set R1 $Console::Return\n deleteObject nothere\n set R2 $Console::Return\nendif\n";
        let mut a = with_object(script, "x");
        a.world.known.insert("keepwall".into());
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        assert!(a.world.objects[0].gone && a.world.objects[0].hidden);
        assert_eq!(a.pick([0.0, -5000.0, 200.0], [0.0, 1.0, 0.0]), None);
        // Any placed object (here `keepwall`) is deleted too: the viewer hides it.
        assert_eq!(a.world.requests.len(), 3, "{}", a.world.log.join("
"));
        assert_eq!((a.scripts.vars.get("R1"), a.scripts.vars.get("R2")), (Some("0"), Some("1")));
        let log = a.take_log().join("\n");
        assert!(log.contains("deleteObject keepwall: placed object deleted") && log.contains("deleteObject: Object \"nothere\" doesn't exist"), "{log}");
    }

    #[test]
    fn delete_object_on_doors_and_chests_and_several_names() {
        let mut a = with_chest(Some("Crystal"));
        let door = active(PLAIN, &[]).world.doors.remove(0);
        a.world.known.insert(door.door.name.to_ascii_lowercase());
        a.world.doors.push(door);
        a.world.sources.insert("d.cs".into(), b"deleteObject Condoor smlchst3\n".to_vec());
        a.scripts.call("d.cs", &[], &mut a.world);
        assert!(a.world.doors[0].gone && a.world.doors[0].collider().is_none());
        assert!(a.world.chests[0].gone && a.world.chests[0].collider().is_none());
        assert_eq!(a.pick([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]), None);
        assert_eq!(a.click("Condoor", NEAR, 2350.0, None), ClickResult::Nothing);
        assert_eq!(a.world.requests.len(), 2);
        // Deleting again: no such object any more.
        a.scripts.call("d.cs", &[], &mut a.world);
        assert_eq!(a.scripts.vars.get(crate::script::RETURN), Some("1"));
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
        for _ in 0..15 {
            a.update();
        }
        assert_eq!(a.scripts.vars.get("B::ended"), None, "the cue 3333 waits in the queue");
        a.update();
        assert_eq!(a.scripts.vars.get("B::ended"), Some("open"), "EndLoop open (clicking.md rule 21)");
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
        let coins = a.world.items.get("NoName0").unwrap().clone();
        assert_eq!((coins.quantity, coins.place, coins.script.as_deref()), (35, Place::World { position: [300.0, 0.0, 0.0], angles: [0.0, 0.0, 1.57] }, Some("coin.cs")));
        a.update();
        assert_eq!(a.world.objects[0].player.movement_name(), "open");
        a.take_log();
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.take_log().join("\n");
        assert!(log.contains("KQMonster::speak Connor to Connor: 0 109 14 33 1…1: to itself, no turn; Connor: queued"), "{log}");
        // A movement it doesn't have: movement 0 under that name
        // (anim-scripts.md rule 5), logged.
        let mut b = with_object("sendEvent 0 $1 KQAnimPlayEvent fly\n", "x");
        b.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        b.update();
        let log = b.take_log().join("\n");
        assert!(log.contains("starts at once: 'fly' (unknown name: movement 0 'stop' plays)"), "{log}");
        assert_eq!((b.world.objects[0].player.current, b.world.objects[0].player.movement_name()), (0, "fly"));
    }

    #[test]
    fn walk_and_run_zones_ask_for_the_run_switch_and_the_camera_range() {
        let mut a = with_trap("if test $2 == On\n KQConner::setRun 0\n KQCamera::setRange 1000 2500\n KQMusic::push minside.aud\nelse if test $2 == Off\n KQCamera::setRange 3000 3000\nendif\nendif\n");
        connor_at(&mut a, 0.0);
        let r = std::mem::take(&mut a.world.requests);
        assert!(matches!(&r[..], [Request::SetRun { run: false, .. }, Request::SetRange { min, max, .. }] if *min == 1000.0 && *max == 2500.0), "{r:?}");
        assert!(a.take_trap_log().join("\n").contains("KQMusic::push minside.aud: due at"));
        // Refused: min not below max; nothing asked.
        connor_at(&mut a, 5000.0);
        assert!(a.world.requests.is_empty());
        assert_eq!(a.scripts.vars.get(crate::script::RETURN), Some("1"));
    }

    #[test]
    fn music_push_and_pop_act_02_s_later_in_order_play_at_once() {
        // sound.md rule 57f, the church trap's pattern (not a copy).
        let mut a = with_trap("if test $2 == On\n KQMusic::push mchurch.aud\n KQMusic::pop mbattle.aud\n KQMusic::play mwizard.aud\nelse if test $2 == Off\n KQMusic::push\n KQMusic::create x\nendif\nendif\n");
        connor_at(&mut a, 0.0);
        assert_eq!(a.world.music, [MusicRequest::Play("mwizard.aud".into())], "play: at once");
        a.world.music.clear();
        // Sent at game time 0; each update handles what is due at its
        // start (0, 1/15, 2/15, 3/15 s).
        for _ in 0..3 {
            a.update();
        }
        assert!(a.world.music.is_empty(), "not before 0.2 s");
        a.update();
        assert_eq!(a.world.music, [MusicRequest::Push("mchurch.aud".into()), MusicRequest::Pop("mbattle.aud".into())], "at 0.2 s, in order");
        connor_at(&mut a, 5000.0);
        let log = a.take_trap_log().join("\n");
        assert!(log.contains("Usage: KQMusic::push music_file"), "{log}");
        assert!(log.contains("KQMusic::create is obsolete, use music.kq instead"), "{log}");
    }
}

#[cfg(test)]
mod talk_tests {
    use super::tests::active;
    use super::*;
    use crate::talk::Voice;
    use kq8_data::msg::{Line, MessageFile, Talkers};

    /// Made-up lines (never the game's): 63/1 and 55/1-2 Connor's.
    fn with_book(a: &mut Active) {
        let line = |noun, seq, raw: &str| Line { tuple: Tuple::new(1000, noun, 0, 0, seq), talker: 4, raw: raw.into() };
        let file = MessageFile { kind: 0x0F, version: 5010, last_id: 0, lines: vec![line(63, 1, "Rip."), line(55, 1, "([#]) A."), line(55, 2, "B.")] };
        a.world.talk.book.add("daventry/English", file, 1000);
        a.world.talk.book.talkers = Talkers::parse(b"(define CONNOR 4)\n");
        a.world.talk.voices = Some(Box::new(|folder, t| Some(Voice { path: format!("{folder}/{}", t.voice_name()), seconds: 0.4, mouths: vec![] })));
        a.scripts.vars.set("Options::Captions", "true".into());
    }

    /// The pattern of a book that waits for Connor's line (not a copy).
    const LISTENER: &str = "if test $2 == Init\n KQMonster::stateChangeNotify $1 Connor\n KQMonster::stateChangeNotify Connor Connor\nelse if test $2 == ConnorAction\n KQMonster::speak Connor Connor 1000 63 0 0 1 1\nelse if test $2 == TalkComplete\n if eval KQMonster::testMessage $3 $4 $5 $6 $7 1000 63 0 0 1\n  set L::heard $3 $4 $5 $6 $7\n endif\nendif\nendif\nendif\n";

    #[test]
    fn talk_complete_reaches_the_notify_group_a_moment_later() {
        let mut a = active(LISTENER, &[]);
        with_book(&mut a);
        a.start();
        let door = a.world.doors[0].door.name.clone();
        a.scripts.call("door.cs", &[door.clone(), "ConnorAction".into()], &mut a.world);
        // 0.4 s = 6 updates: the line ends in update 6, the group hears it
        // in update 7 (queued, not at once).
        for _ in 0..7 {
            a.update();
        }
        assert_eq!(a.scripts.vars.get("L::heard"), None);
        a.update();
        assert_eq!(a.scripts.vars.get("L::heard"), Some("1000 63 0 0 1"));
        let talk = a.world.take_talk_log().join("\n");
        assert!(talk.contains("Connor has no script (rule 13); notify group: ") && talk.contains("Connor: skipped (the issuer itself"), "{talk}");
        assert!(talk.contains(&format!("TalkComplete 1000 63 0 0 1 to {door} (sent by line #0 of Connor): its script ran")), "{talk}");
        // A different line: testMessage says 1, nothing set.
        a.scripts.vars.set("L::heard", "none".into());
        a.scripts.call("door.cs", &[door.clone(), "TalkComplete".into(), "1000".into(), "55".into(), "0".into(), "0".into(), "1".into()], &mut a.world);
        assert_eq!(a.scripts.vars.get("L::heard"), Some("none"));
        // Cleared: no longer hears.
        a.scripts.call("door.cs", &[], &mut a.world);
        let mut w = &mut a.world;
        w.state_change_notify(&["KQMonster::clearStateChangeNotify".into(), door.clone(), "Connor".into()], "t");
        w = &mut a.world;
        assert!(w.notify["connor"].iter().all(|r| !r.eq_ignore_ascii_case(&door)));
    }

    #[test]
    fn connor_turns_to_the_listener_only_in_wait_or_move() {
        let mut a = active("", &[]);
        with_book(&mut a);
        let door = a.world.doors[0].door.name.clone();
        a.world.connor_at = Some([0.0, -3000.0, 0.0]);
        let speak = |to: &str| -> Vec<String> { ["KQMonster::speak", "Connor", to, "1000", "55", "0", "0", "1", "0"].iter().map(|s| s.to_string()).collect() };
        // Not in Wait or Move: queued at once, no turn.
        a.world.speak(&speak(&door), "t");
        assert!(a.world.requests.is_empty());
        a.update();
        assert!(a.world.talk.is_talking("Connor"));
        a.world.talk.clear("Connor", a.world.now());
        // In Wait or Move: the turn, the lines 0.5 s later; <last> 0 = 2.
        a.world.connor_can_turn = true;
        a.world.speak(&speak(&door), "t");
        assert!(matches!(&a.world.requests[..], [Request::ConnorAuto { auto, .. }] if auto.target == [0.0, 0.0] && auto.cue.is_none()));
        let log = a.world.take_talk_log().join("\n");
        assert!(log.contains("1000 55 0 0 1…2 waits 0.5 s"), "{log}");
        // Not a KQMonster: nothing.
        assert_eq!(a.world.speak(&["KQMonster::speak".into(), door.clone(), "Connor".into(), "0".into(), "0".into(), "0".into(), "1".into(), "1".into(), "1".into()], "t"), "1");
    }

    #[test]
    fn test_message_takes_9_or_11_words() {
        let mut a = active("", &[]);
        let w = |s: &str| -> Vec<String> { s.split_whitespace().map(str::to_string).collect() };
        assert_eq!(a.world.test_message(&w("KQMonster::testMessage 1000 63 0 0 1 1000 63 0 0 1"), "t"), "0");
        assert_eq!(a.world.test_message(&w("KQMonster::testMessage 1000 63 0 0 1000 63 0 0"), "t"), "0");
        assert_eq!(a.world.test_message(&w("KQMonster::testMessage 1000 63 0 0 1 1000 63 0 0 2"), "t"), "1");
        // The file as 16 bits, the others as bytes.
        assert_eq!(a.world.test_message(&w("KQMonster::testMessage 66536 319 0 0 1000 63 0 0"), "t"), "0");
        assert_eq!(a.world.test_message(&w("KQMonster::testMessage 1 2 3"), "t"), "1");
    }
}
