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
//!    script with `Open` / `Close`;
//! 4. the events sent meanwhile with delay 0 (M4 4e-10, `anim-scripts.md`
//!    rule 8, Q119: handled before the frame is drawn).
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
//! itself 3333 through the queue (handled at the end of the same update,
//! Q119) and its
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
    /// Connor straight to standing (M6 6e: after No to the Portal state's
    /// question).
    ConnorStandNow { at: String },
    /// A hit took health from Connor and he lives (`fighting.md` rule 29;
    /// M5 5c-4): he flinches if `roll_over` (*R* > 0.9 *D*), his movement
    /// can be cut short and his last flinch ended over 2 s ago, else his
    /// ouch sound plays (at most once a second). The caller decides: it
    /// has his movement.
    ConnorHurt { roll_over: bool, at: String },
    /// Connor's health fell below 0 (rules 18, 29): his dying state, by
    /// the hit's kind.
    ConnorDies { kind: crate::fight::HitKind, at: String },
    /// `move <who> <flags> x y z rx ry rz` (`objects.md` "(d)") for
    /// Connor, the camera or a plain placed object: the caller moves it.
    Move { who: String, flags: u32, position: [f32; 3], turn: [f32; 3], at: String },
    /// `KQCamera::follow Connor|none` (`camera.md` rule 1): `true` =
    /// follow Connor, `false` = stay where it is.
    CameraFollow { connor: bool, at: String },
    /// The player swings and zooms the camera around `point` instead of
    /// Connor (`conversation.md` rule 14, the right button; M4 4e-5);
    /// `None`: around Connor again. `KQCamera::follow` ends it too.
    CameraAround { point: Option<[f32; 3]>, at: String },
    /// `ropeandhook.md` rule 9 (M6 6a-2): the camera goes to `spot` and
    /// looks at `aim` (no roll), as a `move KQCamera` does while it
    /// follows; Connor's first move after is not followed (Q147 c).
    RopeCamera { spot: [f32; 3], aim: [f32; 3], at: String },
    /// `handsOff 1|0` (Q89 a stand-in): the player's movement keys, Jump
    /// and clicks are ignored while on.
    HandsOff { on: bool, at: String },
    /// `loadKQ <file>` while playing (M3 3k-4, Q90 stand-in): the copy
    /// of `name` prepared at the start comes into the world.
    Load { file: String, name: String, at: String },
    /// `loadKQ` while playing made a new copy of a template (M5 5d-2):
    /// the viewer draws it there.
    Spawn { file: String, name: String, placement: Placement, at: String },
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
    /// A conversation starts (`true`) or ends (`interface.md` rule 16,
    /// `conversation.md` rules 2, 15; M4 4e-3).
    Conversation { start: bool, at: String },
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
        AnimRequest::Transition(t) => format!("transition '{}' chosen: waits for phase {} of '{}', then '{}' at {} (rule 4.2)", t.via, t.from, player.movement_name(), t.to, t.to_phase),
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

/// A file a script may load while playing (M5 5d-2; `objects.md` "Where
/// a world's objects come from" rules 2, 6): what the viewer read from it
/// at the start, to make new copies of.
#[derive(Debug, Clone)]
pub struct Template {
    /// Its section name (kept with `same` or no name word).
    pub name: String,
    pub script: Option<String>,
    /// The file's own place (used when the line gives none).
    pub placement: Placement,
    pub kind: TemplateKind,
}

#[derive(Debug, Clone)]
pub enum TemplateKind {
    /// An animated object (`KQAnimOnce`, `KQAnimObject`): its player.
    Anim(Box<NamedAnimPlayer>),
    /// A still model: its `BOUNDS.KQ` box, if any, and collision type.
    Still { bounds: Option<([f32; 3], [f32; 3])>, collision_type: u32 },
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
    /// The files scripts may load while playing, by lower-case base name
    /// (M5 5d-2).
    templates: BTreeMap<String, Template>,
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
    /// Applied items with a duration: the running potions and their
    /// whole-second timers (M5 5g-2, `potions.md` rules 33–34).
    pub potions: Vec<crate::potions::Running>,
    /// Experience due for an item added (`potions.md` rule 41; M5 5g-8):
    /// game time, amount, the item type.
    pub pickup_experience: Vec<(f64, i64, String)>,
    /// The help pop-ups of rule 41 already asked for (logged only).
    pub helps_asked: BTreeSet<&'static str>,
    /// The objects whose `groupsAlways` has bit 2048 (`potions.md` rule
    /// 31), set by the loader (M5 5g-7).
    pub reveal_objects: Vec<String>,
    /// `Reveal True` / `False` script calls to make, taken by
    /// `Active::update` (M5 5g-7).
    pub reveal_calls: Vec<(String, bool)>,
    /// `KQObject::dither <name> 1` (lower case), for the viewer (M5 5g-7;
    /// STAND-IN, Q144).
    pub dithered: BTreeSet<String>,
    /// Connor's handsOff is on (a cutscene), set by the caller before each
    /// update (M5 5g-9, `potions.md` rule 26).
    pub connor_hands_off: bool,
    /// The reason last logged by [`WorldState::note_connor_report`].
    connor_hidden_logged: Option<&'static str>,
    /// `setConnorFlag InTemple4 1` (M5 5g-4, `potions.md` rule 19).
    pub connor_in_temple4: bool,
    /// Every `setConnorFlag` flag (M6 6e-3, `world-change.md` rule 17:
    /// they go with Connor from world to world).
    pub connor_flags: BTreeMap<String, bool>,
    /// Connor's rope settings and status (M6 6a-1, `ropeandhook.md`
    /// rules 1–6).
    pub rope: crate::rope::Rope,
    /// Connor hangs on the rope (rule 7: no hook icon), set by the caller
    /// (M6 6a-3).
    pub connor_on_rope: bool,
    /// Connor is in the rope state, from `away` to `out` (rules 8, 27–30),
    /// set by the caller (M6 6a-3).
    pub connor_roping: bool,
    /// The rope's own first-person lock (rules 9, 12, 13, 26; M6 6a-2):
    /// kept apart from `setThirdPerson`'s. We have no first person: only
    /// logged.
    pub rope_first_person_lock: bool,
    /// Lines for `rope.log` (M6 6a), taken by the caller.
    pub rope_log: Vec<String>,
    /// The current world's magic map (M6 6d, `magic-map.md` rule 3), set
    /// by the caller once loaded.
    pub map: Option<crate::magic_map::MapState>,
    /// Connor's per-map marks (rules 33, 37).
    pub map_marks: crate::magic_map::ConnorMapMarks,
    /// The map's view stored on Connor (rule 22), carried into the next
    /// world.
    pub map_view: crate::magic_map::View,
    /// Other worlds' maps by map index (rule 34): file, keys, where they
    /// came from (kept when Connor left the world, or the shipped file).
    pub other_maps: BTreeMap<usize, (String, kq8_data::magic_map::MapKeys, String)>,
    /// Whether Connor carried the `MagicMap` item at the last update
    /// (rule 2: arriving and leaving switch the map).
    pub map_item: Option<bool>,
    /// Lines for `map.log` (M6 6d), taken by the caller.
    pub map_log: Vec<String>,
    /// The world's portals by lower-case object name (M6 6e-6): their
    /// files' keys, set by the loader and by a `loadKQ` of a portal file.
    pub portals: BTreeMap<String, kq8_data::magic_map::PortalKeys>,
    /// Connor's Portal state (`world-change.md` rules 2-5), if he is in it.
    pub connor_portal: Option<crate::portal::PortalState>,
    /// Portal files a script may `loadKQ` while playing (M6 6e-6c).
    portal_files: BTreeMap<String, kq8_data::magic_map::PortalKeys>,
    /// The status icon places (M5 5g-3, `potions.md` rule 35).
    pub status_icons: [Option<crate::potions::StatusIcon>; crate::potions::ICON_PLACES],
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
    /// Characters prepared for a script's `loadKQ`: they join `monsters`
    /// when loaded (2026-10-22, the Castle's Lucreto).
    pub dormant_monsters: Vec<Monster>,
    /// Characters near or far now (lower case), set by the caller before
    /// each update (`characters.md` rule 11).
    pub characters_in_range: BTreeSet<String>,
    /// Characters whose collider is on now (lower case): near, not far,
    /// off or hidden (`characters.md` rule 33), set by the caller before
    /// each update (M4 4d-12-2).
    pub characters_solid: BTreeSet<String>,
    /// The characters solid for Connor where they stand now, rebuilt by
    /// [`WorldState::characters_gravity`] (M4 4d-12-2).
    pub character_colliders: Vec<crate::collide::CylinderCollider>,
    /// Lines for `characters.log` (M4 4d-3), taken by the caller.
    pub character_log: Vec<String>,
    /// Lines for `fight.log` (M5 5c), taken by the caller.
    pub fight_log: Vec<String>,
    /// The active characters, lower case, in the order they joined
    /// (`character-movement.md` rule 20; M5 5a-4).
    pub active_list: Vec<String>,
    /// `KQWorld::MaxMonsters` (rule 20), set by the caller.
    pub max_monsters: usize,
    /// Connor's forward or back key is held now (`character-movement.md`
    /// rule 25, Q133 b; a scripted walk sends no key), set by the caller.
    pub connor_move_key: bool,
    /// Connor's back key is held now (`fighting.md` rule 13: the reach is
    /// 1.25 × then; M5 5c-3), set by the caller.
    pub connor_back_key: bool,
    /// What `Connor.kq` gives the fight (M5 5c-4), set by the caller.
    pub connor_fight: Option<crate::fight::ConnorFight>,
    /// Connor is in his dying state: hits are ignored (`fighting.md` rule
    /// 29).
    pub connor_dying: bool,
    /// The one-off tips of rule 29 were sent (logged only).
    pub connor_tips: [bool; 2],
    /// The fight's random draws (the rolls, M5 5c).
    pub fight_rng: crate::emitter::Rng,
    /// The weapon Connor has ready (its type name; `inventory.md` rule 9,
    /// `fighting.md` rule 39; M5 5c-5), `None` = none drawn; since M5 5f-3
    /// the hand or the ranged one, as `weapon_mode` says.
    pub weapon_ready: Option<String>,
    /// What is ready: hand, ranged or none (`ranged-weapons.md` rule 1;
    /// M5 5f-3).
    pub weapon_mode: crate::weapons::WeaponMode,
    /// The last weapon put away was the ranged one (rule 2's Delete).
    pub last_away_ranged: bool,
    /// Requests for a new mode, taken by the viewer, which knows his
    /// state (rules 1, 5, 9).
    pub weapon_requests: Vec<crate::weapons::Request>,
    /// The bolt file Connor shoots and his shoot offset (own space): the
    /// start values of `Connor.kq`, then the last ranged item's
    /// (`ranged-weapons.md` rules 17, 21; M5 5f-4), set by
    /// [`WorldState::set_connor_fight`].
    pub connor_projectile: Option<String>,
    pub connor_shoot_offset: [f32; 3],
    /// The ready weapon was swapped by picking up another (5d fix 11):
    /// Connor takes its stance at once, nothing plays. Taken by the viewer.
    pub weapon_swapped: bool,
    /// The combat level's factor (`fighting.md` rule 3): 1.0 by default.
    pub combat_level: f32,
    /// Test switch (ours, not in the original): no character is alarmed,
    /// so none chases or attacks Connor; the alarmed ones give up.
    pub ai_off: bool,
    /// Test switch (ours, not in the original): god mode. Hits on Connor
    /// do nothing (no health lost, no flinch, no death); the viewer also
    /// switches off his fall death (`Mover::no_fall_death`).
    pub god_mode: bool,
    /// Bodies left by the dead (`fighting.md` rule 32; M5 5c-7).
    pub corpses: Vec<crate::fight::Corpse>,
    /// Scripts of the dead to call with `Die <kind>` (rule 32.3), taken
    /// by `Active::update`.
    pub die_calls: Vec<(String, &'static str)>,
    /// Scripts to call with `ProjImmune`: a bolt hit them and their type
    /// has bit 0x4000 (`ranged-weapons.md` rule 30; M5 5f-1).
    pub immune_calls: Vec<String>,
    /// Scripts to call with `ConnorAction`: Azriel's Hammer touched the
    /// Dead City lever (`azriel-hammer.md` rule 8; M6 6b-2).
    pub hammer_calls: Vec<String>,
    /// Azriel's Hammer came back (rule 9): the thrower and a log text,
    /// taken by the viewer (Connor plays `catch`).
    pub hammers_back: Vec<(String, String)>,
    /// The first-kill tip was sent (rule 33; logged only).
    pub kill_tip: bool,
    /// Thrown items in the air (`items.md` rule 12; M5 5c-8).
    pub flights: Vec<crate::item::Flight>,
    /// Projectile files by lower-case name (`fighting.md` rule 36; M5
    /// 5c-9), set by the caller.
    pub projectiles: std::collections::BTreeMap<String, kq8_data::characters::ProjectileKeys>,
    /// Bolts in the air (M5 5c-9), and how many were shot (their names).
    pub bolts: Vec<crate::fight::Bolt>,
    pub bolts_shot: u64,
    /// Where Connor was at the last update (his speed for the aim, rule
    /// 36).
    pub connor_was: Option<[f32; 3]>,
    /// The conversation info objects (M4 4e-3, `conversation.md` rule 3).
    pub conv_infos: Vec<kq8_data::conv_info::ConvInfo>,
    /// The conversation going on or asked for (`crate::conversation`).
    pub conversation: Option<crate::conversation::Conversation>,
    /// Connor is to leave his talk state (why), at the next
    /// `Active::conversation_tick` (rule 15).
    pub connor_leave: Option<String>,
    /// Lines for `conversation.log` (M4 4e-3), taken by the caller.
    pub conversation_log: Vec<String>,
    /// Connor's lip-sync codes 8–16 of this update (gestures,
    /// `conversation.md` rule 18; M4 4e-7), taken by the caller.
    pub connor_gestures: Vec<u16>,
}

/// A placed `KQMonster` (M4 4a-2): monsters don't move or fight yet (M5);
/// for speech only their name, place and script.
#[derive(Debug, Clone, PartialEq)]
pub struct Monster {
    pub name: String,
    pub position: [f32; 3],
    pub script: Option<String>,
    /// Its goodbye and welcome-back lines (`conversation.md` rule 20).
    pub goodbye: Option<Tuple>,
    pub welcome_back: Option<Tuple>,
}

impl WorldState {
    /// The objects that have left this world (M6 6e-8, `world-change.md`
    /// rule 12: deleted objects are not in the kept world): deleted ones,
    /// items carried by Connor or used up, characters dead or gone. Lower
    /// case. On a return they are not loaded again.
    pub fn gone_names(&self) -> Vec<String> {
        let mut out: BTreeSet<String> = self.deleted.clone();
        for o in &self.items.objects {
            if o.carried_by(crate::talk::CONNOR) || matches!(o.place, crate::item::Place::Gone) {
                out.insert(o.name.to_ascii_lowercase());
            }
        }
        for a in &self.anims {
            let dead = a.character.as_deref().and_then(|c| c.fight.as_ref()).is_some_and(|f| f.dying.is_some());
            if a.gone || dead {
                out.insert(a.player.name.to_ascii_lowercase());
            }
        }
        out.into_iter().collect()
    }

    pub fn now(&self) -> f64 {
        self.updates as f64 / crate::UPDATES_PER_SECOND
    }

    /// `potions.md` rule 26 (M5 5g-6, 5g-9): where the characters are
    /// told Connor is: 100,000 below his place while Invisible runs, he is
    /// in the rope state (`ropeandhook.md` rule 29, M6 6a-6) or a cutscene
    /// has handsOff on.
    pub fn connor_reported(&self) -> Option<[f32; 3]> {
        let p = self.connor_at?;
        Some(if self.connor_hidden_why().is_some() { [p[0], p[1], p[2] - crate::potions::HIDDEN_DROP] } else { p })
    }

    /// Why the characters are told Connor is far below (rule 26), if they
    /// are.
    pub fn connor_hidden_why(&self) -> Option<&'static str> {
        if self.connor_invisible() {
            Some("the Invisible potion")
        } else if self.connor_roping {
            // ropeandhook.md rule 29 (M6 6a-6).
            Some("the rope state")
        } else if self.connor_hands_off {
            Some("a cutscene (handsOff)")
        } else {
            None
        }
    }

    /// Rule 26: logs each change of what the characters are told
    /// (`characters.log`), once per update.
    pub(crate) fn note_connor_report(&mut self) {
        let why = self.connor_hidden_why();
        if why == self.connor_hidden_logged {
            return;
        }
        self.connor_hidden_logged = why;
        let line = match why {
            Some(w) => format!("characters are told Connor is 100,000 below where he is: {w} (potions.md rule 26)"),
            None => "characters are told where Connor really is again (potions.md rule 26)".to_string(),
        };
        self.character_log.push(format!("update {}: Connor: {line}", self.updates));
        self.note(line);
    }

    /// The Invisible potion runs (rule 28: `KQConner::checkInvisible`).
    pub fn connor_invisible(&self) -> bool {
        self.potions.iter().any(|r| r.effect == crate::potions::Effect::Invisible)
    }

    /// A line about the fight (M5 5c): `fight.log`, `characters.log` and
    /// the current log.
    pub fn fight_note(&mut self, who: &str, line: impl AsRef<str>) {
        self.fight_log.push(format!("update {}: {who}: {}", self.updates, line.as_ref()));
        self.character_note(who, line);
    }

    /// A line about a character: the current log and `characters.log`.
    pub(crate) fn character_note(&mut self, who: &str, line: impl AsRef<str>) {
        let line = format!("{who}: {}", line.as_ref());
        self.character_log.push(format!("update {}: {line}", self.updates));
        self.note(line);
    }

    /// `characters.md` rule 12 (M4 4d-9): the start-up runs before a
    /// character's first movement request (`why`), whatever its range.
    pub(crate) fn start_up_character(&mut self, i: usize, why: &str) {
        let now = self.now();
        let a = &mut self.anims[i];
        let Some(c) = a.character.as_mut() else { return };
        if let Some(line) = c.start_up(&mut a.player, now, why) {
            let name = a.player.name.clone();
            self.character_note(&name, line);
        }
    }

    /// Rule 12.2: each character's queued Wait, if due (after each pass
    /// over the event queue; whatever its range).
    fn characters_due(&mut self, now: f64) {
        for i in 0..self.anims.len() {
            if self.anims[i].gone {
                continue;
            }
            let a = &mut self.anims[i];
            let Some(c) = a.character.as_mut() else { continue };
            if let Some(line) = c.wait_due(&mut a.player, now) {
                let name = a.player.name.clone();
                self.character_note(&name, line);
            }
        }
    }

    /// `characters.md` rule 8 (M4 4d-6b-2): every character's mover, once
    /// per update after the characters' own update.
    ///
    /// M5 5a-3 (`character-movement.md` rules 6, 33–35, Q128): each
    /// character's move sees the terrain, the buildings (mask bit 0x8),
    /// and as obstacles the plain objects, the other characters (their
    /// cylinders where they stand now, while solid) and Connor whose type
    /// its mask covers in the bits of its own touch table; `colliderMask=0`
    /// sees nothing but the ground check.
    pub fn characters_gravity(&mut self, ground: &dyn crate::connor_mover::GroundLookup, buildings: &[crate::collide::BuildingCollider], objects: &[crate::collide::BoxCollider]) {
        use crate::collide::CylinderCollider;
        let now = self.now();
        // The objects each mask meets (most characters share 0x281F).
        let mut by_mask: BTreeMap<u32, Vec<crate::collide::BoxCollider>> = BTreeMap::new();
        for i in 0..self.anims.len() {
            if self.anims[i].gone {
                continue;
            }
            let Some(c) = self.anims[i].character.as_deref() else { continue };
            let (mask, hits_buildings) = (c.collision_mask, c.hits_buildings);
            let obstacle = |t: u32| mask & t & crate::character::OWN_TOUCH_OBSTACLES != 0;
            let objs = by_mask.entry(mask).or_insert_with(|| objects.iter().filter(|o| obstacle(o.kind.collision_type)).cloned().collect());
            // The others where they stand now (those already moved in this
            // pass, at their new place).
            let mut others: Vec<CylinderCollider> = self
                .anims
                .iter()
                .enumerate()
                .filter(|(j, a)| *j != i && !a.gone && self.characters_solid.contains(&a.player.name.to_ascii_lowercase()))
                .filter_map(|(_, a)| {
                    let o = a.character.as_deref()?;
                    let (min, max) = o.bounds?;
                    (!o.unborn && obstacle(o.collision_type)).then(|| CylinderCollider::from_box(&a.player.name, a.player.base.position, min, max, o.collision_type, o.collision_mask))
                })
                .collect();
            // Q132 (c) stand-in: Connor as the same cylinder, from his box.
            if let (true, Some(at)) = (obstacle(crate::collide::CONNOR_TYPE), self.connor_at) {
                let (min, max) = crate::connor_mover::CONNOR_BOX;
                others.push(CylinderCollider::from_box("Connor", at, min, max, crate::collide::CONNOR_TYPE, CONNOR_MASK));
            }
            let mut world = crate::connor_mover::Scene::new(ground, if hits_buildings { buildings } else { &[] }, if mask == 0 { &[] } else { objs });
            if mask != 0 {
                world.characters = &others;
            }
            let a = &mut self.anims[i];
            let Some(c) = a.character.as_deref_mut() else { continue };
            let notes = c.gravity_step(&mut a.player, &world, now);
            let name = a.player.name.clone();
            let at = a.player.placement();
            if let Some(k) = self.clickables.iter_mut().find(|k| k.name.eq_ignore_ascii_case(&name)) {
                k.placement = at;
            }
            for n in notes {
                self.character_note(&name, n);
            }
        }
        self.update_character_colliders();
    }

    /// M4 4d-12-2: the characters solid for Connor, as cylinders where they
    /// stand now (`connor-collision.md` rules 2, 4, 18; `characters.md`
    /// rules 9, 16, 33): not deleted, with a box, not in Birth, its
    /// collider on, and Connor's mask covering its type (so he is told and
    /// it is a wall for him). Logs each change to solid / not solid.
    pub fn update_character_colliders(&mut self) {
        let mut now = Vec::new();
        let mut why_not = Vec::new();
        for a in &self.anims {
            let Some(c) = a.character.as_deref() else { continue };
            let name = &a.player.name;
            let why = if a.gone {
                Some("deleted")
            } else if c.unborn {
                // characters.md rule 16; a sinking zombie stays solid
                // until its `hide` ends (character-movement.md rule 30).
                Some("underground (rule 16)")
            } else if !self.characters_solid.contains(&name.to_ascii_lowercase()) {
                Some("its collider is off (far, off or hidden, rule 33)")
            } else if CONNOR_MASK & !crate::trap::TRAP_TYPE & c.collision_type == 0 {
                // Rule 4: he ignores type 0x10 (traps).
                Some("Connor's mask does not cover its type (traps' 0x10 ignored)")
            } else {
                None
            };
            match (why, c.bounds) {
                (None, Some((min, max))) => now.push(crate::collide::CylinderCollider::from_box(name, a.player.base.position, min, max, c.collision_type, c.collision_mask)),
                (None, None) => why_not.push((name.clone(), "no box (rule 9)")),
                (Some(w), _) => why_not.push((name.clone(), w)),
            }
        }
        let was: Vec<String> = self.character_colliders.iter().map(|c| c.name.clone()).collect();
        for c in now.iter().filter(|c| !was.contains(&c.name)) {
            let p = c.position;
            let line = format!(
                "solid for Connor: cylinder radius {:.0} (+ his ball radius), height {:.0} from ({:.0}, {:.0}, {:.0}); type {:#x} mask {:#x}{} (connor-collision.md rule 18)",
                c.radius,
                c.height,
                p[0],
                p[1],
                p[2],
                c.collision_type,
                c.collision_mask,
                if c.collision_type & crate::collide::CREATURE_BITS != 0 { "; a creature: starting inside is no hit" } else { "; not a creature: starting inside is a hit" }
            );
            self.character_note(&c.name.clone(), line);
        }
        for (name, why) in why_not.into_iter().filter(|(n, _)| was.contains(n)) {
            self.character_note(&name, format!("not solid for Connor: {why}"));
        }
        self.character_colliders = now;
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
    /// A line about the rope (M6 6a): `rope.log` and the current log.
    pub fn rope_note(&mut self, line: impl AsRef<str>) {
        self.rope_log.push(format!("update {}: {}", self.updates, line.as_ref()));
        self.note(line);
    }

    /// Connor carries an item of category `Rope` (`ropeandhook.md` rule
    /// 4): its object's name.
    pub fn connor_rope(&self) -> Option<String> {
        self.items.objects.iter().filter(|o| o.carried_by(CONNOR)).find(|o| self.items.kind(&o.kind).is_some_and(|t| t.category == crate::rope::ROPE_CATEGORY)).map(|o| o.name.clone())
    }

    /// The six `KQConner::setRope*` commands (M6 6a-1, rules 2–6).
    fn rope_command(&mut self, words: &[String], at: &str) -> Option<String> {
        let carries = self.connor_rope().is_some();
        let now = self.now();
        let d = self.rope.command(words, carries, now)?;
        let mut line = format!("{at}: {}", d.line);
        if d.put_back {
            // Rule 6: a held rope goes back.
            let held = self.hand.as_ref().filter(|h| self.items.get(h).and_then(|o| self.items.kind(&o.kind)).is_some_and(|t| t.category == crate::rope::ROPE_CATEGORY)).cloned();
            if let Some(h) = held {
                self.hand = None;
                line += &format!("; {h} was in hand: put back, the play pointer again (rule 6)");
                self.item_note(format!("{h}: put back by setRope None (ropeandhook.md rule 6)"));
            }
        }
        self.rope_note(line);
        self.rope_icon_step();
        Some(d.result.to_string())
    }

    /// Rule 7: the hook icon for the status now (and rule 4: none without
    /// the rope; none while on the rope), each change logged.
    pub fn rope_icon_step(&mut self) {
        let wanted = self.rope.icon(self.connor_rope().is_some(), self.connor_on_rope);
        for line in crate::rope::sync_icon(&mut self.status_icons, wanted) {
            self.rope_note(format!("hook icon: {line}"));
        }
    }

    pub(crate) fn item_note(&mut self, line: impl AsRef<str>) {
        self.item_log.push(format!("update {}: {}", self.updates, line.as_ref()));
        self.note(line);
    }

    /// DESIGN L-3: Connor went up to `level`: the level-up sound. Returns
    /// what the log line says about it.
    pub(crate) fn level_up_sound(&mut self, level: i64) -> String {
        self.sounds.push(Start::plain_one_shot(crate::stats::LEVEL_UP_SOUND, format!("level up to {level}")));
        format!("; sound {} (stand-in Q149 c)", crate::stats::LEVEL_UP_SOUND)
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
        let first = words.next().unwrap_or("");
        // anim-scripts.md rule 7 (M5 5c-2): done by the player itself.
        if first.eq_ignore_ascii_case("interruptable") {
            return if words.next().and_then(|n| n.parse::<f64>().ok()).is_some_and(|n| n != 0.0) { "can be cut short from here, anim-scripts.md rule 7" } else { "can't be cut short from here, anim-scripts.md rule 7" };
        }
        if !first.eq_ignore_ascii_case("playsound") {
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

    /// `KQConner::setLevel <n>` (every world's `conStart.cs`). STAND-IN
    /// (Q149 a): only the level number is set; the stats event that
    /// follows it in every script brings health and the rest. The meters
    /// are sent again (ours).
    fn set_level(&mut self, words: &[String], at: &str) -> String {
        let level = words.get(1).filter(|w| looks_like_number(w)).map(|v| number_value(v) as i64).filter(|l| *l >= 1);
        match (level, self.connor_stats.as_mut()) {
            (Some(l), Some(c)) => {
                let (was, stats) = (c.level, c.stats.0);
                c.level = l;
                self.connor_stats_events += 1;
                self.note(format!("{at}: setLevel {l}: Connor's level {was} -> {l}; stats unchanged {stats:?} (Q149 a stand-in)"));
                "0".into()
            }
            (Some(_), None) => {
                self.note(format!("{at}: setLevel: Connor's stats unknown (no Connor.kq); nothing"));
                "1".into()
            }
            (None, _) => {
                self.note(format!("{at}: usage: KQConner::setLevel <level from 1>: '{}'", words.join(" ")));
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
        let key = file.rsplit(['/', '\\']).next().unwrap_or(file).to_ascii_lowercase();
        // M5 5d-2: a name or a place on the line, or no prepared copy: a
        // new copy of its template.
        if words.len() > 2 || !self.dormant.contains_key(&key) {
            if let Some(t) = self.templates.get(&key).cloned() {
                return self.load_copy(&key, t, words, at);
            }
        }
        // M6 6e-6c: a portal file (the teleporters' `GAME/Kq/*tele.kq`).
        if let Some(keys) = self.portal_files.get(&key).cloned() {
            return self.load_portal(&key, keys, words, at);
        }
        if words.len() > 2 {
            self.note(format!("{at}: loadKQ {}: a name or position while playing, and no template of {file}: not done", words[1..].join(" ")));
            return "0".into();
        }
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
        // A trap: its box is in place from now on (it was off since the start).
        let trap = match self.trap_index(&name) {
            Some(i) => {
                self.traps[i].gone = false;
                "; its trap box is in place now (traps.md rules 1, 2)"
            }
            None => "",
        };
        self.note(format!("{at}: loadKQ {file}: {name} comes into the world (as at the start){}{trap}", if script.is_some() { "; its script gets Init now" } else { "; no script" }));
        // A character brought in by a script can speak, as the ones placed
        // at the start (`talking.md` rule 6; found 2026-10-22: Castle
        // Daventry's `mirror.cs` loads `lucreto.kq`, then has him speak).
        if let Some(k) = self.dormant_monsters.iter().position(|m| m.name.eq_ignore_ascii_case(&name)) {
            let m = self.dormant_monsters.remove(k);
            self.note(format!("{at}: loadKQ {file}: {} is a character: it can speak now (talking.md rule 6)", m.name));
            self.monsters.push(m);
        }
        self.requests.push(Request::Load { file: key, name: name.clone(), at: at.to_string() });
        if let Some(script) = script {
            self.calls_out.push((script, vec![name, "Init".into()]));
        }
        "0".into()
    }

    /// `loadKQ <portal file> [name ...]` while playing (M6 6e-6c,
    /// `magic-map.md` rule 39: the teleport scripts load the shown world's
    /// `TeleportPortalKQ` at their pad, then start the Portal state with
    /// `$ObjectName`): the portal is known by its new name (`@`: the next
    /// `NoName<n>`); `$ObjectName` is set. It is not drawn (its model is a
    /// marker, `clicker.kqs`) and has no collider (`colliderType=0`).
    fn load_portal(&mut self, key: &str, mut keys: kq8_data::magic_map::PortalKeys, words: &[String], at: &str) -> String {
        let name = match words.get(2).map(String::as_str) {
            None | Some("same") => keys.name.clone(),
            Some("@") => self.items.next_name(),
            Some(n) => n.to_string(),
        };
        let lower = name.to_ascii_lowercase();
        if self.known.contains(&lower) && !self.deleted.contains(&lower) {
            self.note(format!("{at}: ERROR loadKQ {key}: a live object already has that name ({name}): load refused"));
            return "1".into();
        }
        self.deleted.remove(&lower);
        self.known.insert(lower.clone());
        keys.name = name.clone();
        self.note(format!("{at}: loadKQ {key}: the portal {name} to {:?} (magic-map.md rule 39); $ObjectName = {name}", keys.world));
        self.portals.insert(lower, keys);
        self.vars_out.push(("ObjectName".into(), name));
        "0".into()
    }

    /// The portal files a script may load while playing, by lower-case
    /// file name (M6 6e-6c), from the viewer at the start.
    pub fn set_portal_files(&mut self, files: impl IntoIterator<Item = (String, kq8_data::magic_map::PortalKeys)>) {
        self.portal_files.extend(files.into_iter().map(|(f, k)| (f.to_ascii_lowercase(), k)));
    }

    /// The template files (M5 5d-2), from the viewer at the start.
    pub fn set_templates(&mut self, t: impl IntoIterator<Item = (String, Template)>) {
        self.templates.extend(t.into_iter().map(|(f, t)| (f.to_ascii_lowercase(), t)));
    }

    /// `loadKQ <file> [name [x y z [rx ry rz]]]` making a new copy of a
    /// template (M5 5d-2; `objects.md` rules 2, 6; `anim-scripts.md` rule
    /// 12): the name (`same` or none: the file's; `@`: `NoName<n>` now; else
    /// that name), the place (x y z and the angles, 0 0 0 if missing,
    /// replace the file's; z −1 / −2: the floor / surface height), refused
    /// for a name in use as today; `$ObjectName` set; the object is in the
    /// world at once (core side), its script gets `Init` now, and the
    /// viewer is asked to draw it.
    fn load_copy(&mut self, key: &str, t: Template, words: &[String], at: &str) -> String {
        let name = match words.get(2).map(String::as_str) {
            None | Some("same") => t.name.clone(),
            Some("@") => self.items.next_name(),
            Some(n) => n.to_string(),
        };
        let lower = name.to_ascii_lowercase();
        let live = self.known.contains(&lower) && !self.deleted.contains(&lower);
        if live || self.deleted_this_event.contains(&lower) {
            let why = if live { "a live object already has that name" } else { "its name was deleted in this same event and is not free yet" };
            self.note(format!("{at}: ERROR loadKQ {key}: {why} ({name}): the original stops with a fatal error; load refused"));
            return "1".into();
        }
        let num = |k: usize| words.get(k).filter(|w| looks_like_number(w)).map(|w| number_value(w) as f32);
        let placement = match (num(3), num(4), num(5)) {
            (Some(x), Some(y), Some(mut z)) => {
                if z == -1.0 || z == -2.0 {
                    match self.heights.as_ref().and_then(|h| h(x, y, z == -1.0)) {
                        Some(g) => z = g,
                        None => self.note(format!("{at}: loadKQ {key}: z {z} at ({x:.0}, {y:.0}): no ground height known; kept")),
                    }
                }
                let angles = [num(6).unwrap_or(0.0), num(7).unwrap_or(0.0), num(8).unwrap_or(0.0)];
                Placement { rotation: kq8_data::objects::rotation(angles), position: [x, y, z] }
            }
            _ => t.placement,
        };
        self.deleted.remove(&lower);
        self.known.insert(lower);
        let what = match &t.kind {
            TemplateKind::Anim(p) => {
                let mut player = (**p).clone();
                player.name = name.clone();
                player.base = placement;
                self.anims.push(AnimObject { player, script: t.script.clone(), gone: false, character: None, translucency: None });
                "animated"
            }
            TemplateKind::Still { bounds: Some((min, max)), collision_type } => {
                let mut c = Clickable::plain(&name, "KQObject", *min, *max, placement, *collision_type);
                c.script = t.script.clone();
                self.clickables.push(c);
                "still, with its box"
            }
            TemplateKind::Still { bounds: None, .. } => "still, no box",
        };
        self.vars_out.push(("ObjectName".into(), name.clone()));
        let p = placement.position;
        self.note(format!(
            "{at}: loadKQ {key}: a new copy {name} ({what}) at ({:.0}, {:.0}, {:.0}); $ObjectName = {name}{} (objects.md rules 2, 6)",
            p[0],
            p[1],
            p[2],
            if t.script.is_some() { "; its script gets Init now" } else { "; no script" }
        ));
        self.requests.push(Request::Spawn { file: key.to_string(), name: name.clone(), placement, at: at.to_string() });
        if let Some(script) = t.script {
            self.calls_out.push((script, vec![name, "Init".into()]));
        }
        "0".into()
    }

    /// `goldCoinPersists 0|1` (`inventory.md` rule 12, M4 4c-4): while on,
    /// coins accepted from the hand stay there; 0 also empties the hand at
    /// once.
    /// `KQObject::dither <name> 0|1` (`potions.md` rule 31: the keep wall
    /// "becomes see-through"): kept for the viewer. STAND-IN (Q144): how a
    /// dithered object is drawn is not described; the viewer draws it half
    /// see-through.
    fn dither(&mut self, words: &[String], at: &str) -> String {
        let (Some(name), Some(v)) = (words.get(1), words.get(2).filter(|v| looks_like_number(v))) else {
            self.note(format!("{at}: usage: KQObject::dither <name> 0|1; nothing"));
            return "1".into();
        };
        let on = number_value(v) != 0.0;
        if on {
            self.dithered.insert(name.to_ascii_lowercase());
        } else {
            self.dithered.remove(&name.to_ascii_lowercase());
        }
        self.item_note(format!("{at}: KQObject::dither {name} {}: drawn {} (potions.md rule 31; how it looks: stand-in, Q144)", u8::from(on), if on { "dithered (half see-through, stand-in)" } else { "as before" }));
        "0".into()
    }

    /// `setConnorFlag <flag> 0|1`: every flag is kept on Connor
    /// (`world-change.md` rule 17); only `InTemple4` changes a rule
    /// (`potions.md` rule 19, `temple4/kq/archon.cs`).
    fn set_connor_flag(&mut self, words: &[String], at: &str) -> String {
        let (Some(flag), Some(v)) = (words.get(1), words.get(2).filter(|v| looks_like_number(v))) else {
            self.note(format!("{at}: usage: setConnorFlag <flag> 0|1; nothing"));
            return "1".into();
        };
        self.connor_flags.insert(flag.clone(), number_value(v) != 0.0);
        if !flag.eq_ignore_ascii_case("InTemple4") {
            self.note(format!("{at}: setConnorFlag {flag} {v}: kept on Connor (world-change.md rule 17)"));
            return "0".into();
        }
        self.connor_in_temple4 = number_value(v) != 0.0;
        self.item_note(format!("{at}: setConnorFlag InTemple4 {}: the shield gives half its raise per hit, at most 85 (potions.md rule 19)", u8::from(self.connor_in_temple4)));
        "0".into()
    }

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

    /// `KQObject::getLoc <name>` / `getLoc <name>` while playing
    /// (`objects.md` "Script variables, getLoc and move" rule 1): its
    /// current position and turn into `$KQObject::loc` (`x y z dirX dirY
    /// dirZ`) and `$KQObject::locOnly` (`x y z`). Objects, animated objects
    /// and characters, chests and doors; Connor by his position (his turn:
    /// 0 0 0, **ours**). Unknown: 1, the variables unchanged.
    fn get_loc(&mut self, words: &[String], at: &str) -> String {
        let Some(who) = words.get(1) else {
            self.note(format!("{at}: getLoc without a name; nothing"));
            return "1".into();
        };
        let Some(b) = self.place_of(who) else {
            self.note(format!("{at}: getLoc {who}: no such object; variables unchanged"));
            return "1".into();
        };
        let a = angles_of(&b.rotation);
        let f = |v: &[f32]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ");
        let only = f(&b.position);
        self.vars_out.push(("KQObject::loc".into(), format!("{only} {}", f(&a))));
        self.vars_out.push(("KQObject::locOnly".into(), only.clone()));
        self.note(format!("{at}: getLoc {who}: {only} (turn {:.3} {:.3} {:.3})", a[0], a[1], a[2]));
        "0".into()
    }

    /// `checkDistance` (`anim-scripts.md` rule 31): `<obj1> <obj2> <max>`,
    /// `<obj1> <x> <y> <z> <max>` or `<x> <y> <z> <x2> <y2> <z2> <max>`.
    /// Straight 3D distance between the positions (Connor: his feet; an
    /// object: where it stands now); 1 when at most `max`, else 0. An
    /// unknown name or a wrong number of words: an error line and 0.
    fn check_distance(&mut self, words: &[String], at: &str) -> String {
        let point = |a: &Self, w: &[String]| -> Result<[f32; 3], String> {
            match w {
                [name] => a.place_of(name).map(|b| b.position).ok_or_else(|| format!("can't find '{name}'")),
                [x, y, z] => Ok([x, y, z].map(|v| number_value(v) as f32)),
                _ => unreachable!(),
            }
        };
        let args = &words[1..];
        let (one, two, max) = match args.len() {
            3 => (&args[0..1], &args[1..2], &args[2]),
            5 => (&args[0..1], &args[1..4], &args[4]),
            7 => (&args[0..3], &args[3..6], &args[6]),
            _ => {
                self.note(format!("{at}: ERROR usage: checkDistance <obj1> <obj2> <maxDist> | <obj1> <x> <y> <z> <maxDist> | <x> <y> <z> <x2> <y2> <z2> <maxDist>: '{}'; 0 (anim-scripts.md rule 31)", words.join(" ")));
                return "0".into();
            }
        };
        let (a, b) = match (point(self, one), point(self, two)) {
            (Ok(a), Ok(b)) => (a, b),
            (Err(e), _) | (_, Err(e)) => {
                self.note(format!("{at}: ERROR checkDistance: {e}; 0 (anim-scripts.md rule 31)"));
                return "0".into();
            }
        };
        let max = number_value(max) as f32;
        let d = ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2) + (a[2] - b[2]).powi(2)).sqrt();
        let within = d <= max;
        self.note(format!(
            "{at}: checkDistance {} ({:.0}, {:.0}, {:.0}) to {} ({:.0}, {:.0}, {:.0}): {d:.0} {} {max:.0}: {} (anim-scripts.md rule 31)",
            one.join(" "), a[0], a[1], a[2], two.join(" "), b[0], b[1], b[2],
            if within { "<=" } else { ">" }, u8::from(within)
        ));
        u8::from(within).to_string()
    }

    /// `KQObject::tossItems <object> <offsetZ> <type1,type2,...>`
    /// (`items.md` rule 12; M5 5d-5): one item (quantity 1) per listed type
    /// (`INVITEM_` optional), thrown from the object's place raised by
    /// `offsetZ`, each with its own flight and tumble (`thrown-items.md`). An
    /// object that doesn't exist: an error line, nothing made.
    fn toss_items(&mut self, words: &[String], at: &str) -> String {
        let (Some(who), Some(dz), Some(list)) = (words.get(1), words.get(2), words.get(3)) else {
            self.note(format!("{at}: usage: KQObject::tossItems <object> <offsetZ> <type1,type2,...>: '{}'", words.join(" ")));
            return "1".into();
        };
        let Some(b) = self.place_of(who) else {
            self.note(format!("{at}: ERROR tossItems: no object '{who}'; nothing made (items.md rule 12)"));
            return "1".into();
        };
        let from = [b.position[0], b.position[1], b.position[2] + number_value(dz) as f32];
        let lines: Vec<String> = list.split(',').map(str::trim).filter(|t| !t.is_empty()).map(|t| self.toss(t, 1, from)).collect();
        self.item_note(format!("{at}: tossItems {who} {dz}: {} (items.md rule 12)", lines.join("; ")));
        "0".into()
    }

    /// Where an object stands, for `getLoc` and `tossItems`.
    fn place_of(&self, who: &str) -> Option<Placement> {
        if let Some(i) = self.object_index(who) {
            Some(self.objects[i].player.base)
        } else if let Some(i) = self.anim_index(who) {
            Some(self.anims[i].player.base)
        } else if let Some(i) = self.chest_index(who) {
            Some(self.chests[i].chest.player.base)
        } else if let Some(i) = self.door_index(who) {
            Some(self.doors[i].door.player.base)
        } else if who.eq_ignore_ascii_case("Connor") {
            self.connor_at.map(|p| Placement { rotation: crate::door::identity().m, position: p })
        } else {
            // A box object (a barrel; M5 5c fix 5).
            self.clickables.iter().find(|c| !c.gone && c.name.eq_ignore_ascii_case(who)).map(|c| c.placement)
        }
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
            [_, who, state, rest @ ..] if who.eq_ignore_ascii_case("Connor") && state.eq_ignore_ascii_case("Portal") => {
                // world-change.md rules 1-2, magic-map.md rule 40 (M6 6e-6).
                return self.start_portal(rest, at);
            }
            [_, who, state, rest @ ..] if state.eq_ignore_ascii_case("Die") && self.character_index(who).is_some_and(|i| self.anims[i].character.as_deref().is_some_and(|c| c.fight.is_some())) => {
                // fighting.md rule 30 (M5 5c-7): `Die now` skips the movement.
                let i = self.character_index(who).unwrap();
                let now_flag = rest.iter().any(|w| w.eq_ignore_ascii_case("now"));
                self.fight_note(who, format!("{at}: KQMonster::setState {who} Die {}", rest.join(" ")));
                let kind = self.anims[i].character.as_deref().and_then(|c| c.fight.as_ref()).and_then(|f| f.last_kind).unwrap_or(crate::fight::HitKind::Normal);
                self.character_dies(i, kind, now_flag);
            }
            [_, who, state, rest @ ..] if self.character_index(who).is_some() => {
                // characters.md rule 22.
                let i = self.character_index(who).unwrap();
                let now = self.now();
                let a = &mut self.anims[i];
                let line = a.character.as_mut().unwrap().set_state(&mut a.player, state, rest, now);
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
        let on = number_value(on) != 0.0;
        // M5 5d-1: a KQAnimOnce among the other animated objects too.
        if let Some(j) = self.anim_index(name).filter(|&j| self.anims[j].player.once && !self.anims[j].gone) {
            self.anims[j].player.delete_when_done = on;
            self.note(format!("{at}: setDeleteWhenDone {name}: {}", if on { "deleted when its movement ends" } else { "stays when its movement ends" }));
            return "0".into();
        }
        let Some(i) = self.object_index(name).filter(|&i| self.objects[i].player.once && !self.objects[i].gone) else {
            self.note(format!("{at}: setDeleteWhenDone: could not find object '{name}'"));
            return "1".into();
        };
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
    pub(crate) fn is_gone(&self, name: &str) -> bool {
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
        if let Some(h) = self.plain_hidden.get(&name.to_ascii_lowercase()) {
            return *h;
        }
        // M5 5g-7: a tracked object starts from its file's `hidden=`, so
        // its own state wins (`wizchest1` shown by `setHidden 0`).
        if self.object_index(name).is_some() || self.chest_index(name).is_some() || self.items.get(name).is_some() {
            return self.is_hidden(name);
        }
        file_hidden
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
pub(crate) fn tuple_of(words: &[String]) -> Option<Tuple> {
    if words.len() != 5 || !words.iter().all(|w| looks_like_number(w)) {
        return None;
    }
    let n = |k: usize| number_value(&words[k]) as i64;
    Some(Tuple::new(n(0) as u16, n(1) as u8, n(2) as u8, n(3) as u8, n(4) as u8))
}

impl WorldState {
    /// The `KQMonster` (or Connor) called `name`, as the world writes it
    /// (`talking.md` rule 6).
    pub(crate) fn monster_name(&self, name: &str) -> Option<String> {
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
        // Rule 8: Connor or the partner while either is in its talk state.
        if self.speak_ignored(&issuer) {
            self.talk_note(format!("{at}: KQMonster::speak {issuer}: ignored: {issuer} or the conversation's partner is in a talk state (talking.md rule 8)"));
            return "0".into();
        }
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
        let now = self.now();
        self.talk.clear(name, now);
        self.note(format!("{at}: KQMonster::clearSpeak {name} (talk.log)"));
        // conversation.md rule 23: a partner's conversation queue too; the
        // partner to Wait, Connor sent an empty "line finished" (his mouth
        // to rest, his notify group TalkComplete 0 0 0 0 0); his talk state
        // goes on.
        if let Some(line) = self.talk.clear_conv(name, now) {
            let mut notes = vec![line];
            if let Some(i) = self.character_index(name) {
                let a = &mut self.anims[i];
                let who = a.player.name.clone();
                let line = a.character.as_mut().unwrap().set_state(&mut a.player, "Wait", &[], now);
                self.character_note(&who, format!("clearSpeak on a partner: {line} (conversation.md rule 23)"));
                notes.push(format!("{who} to Wait"));
            }
            self.talk.events.push(crate::talk::TalkEvent::Mouth { id: 0, speaker: CONNOR.into(), shape: 0 });
            let data: Vec<String> = vec!["0".into(); 5];
            let recipients = self.notify.get(&CONNOR.to_ascii_lowercase()).cloned().unwrap_or_default();
            for r in &recipients {
                self.queue.send(now, 0.0, r, "TalkComplete", &data, &format!("clearSpeak {name}"));
            }
            notes.push(format!(
                "Connor: an empty line finished: mouth at rest; his notify group gets TalkComplete 0 0 0 0 0: {}",
                if recipients.is_empty() { "nobody".to_string() } else { recipients.join(", ") }
            ));
            self.conversation_note(format!("{at}: KQMonster::clearSpeak {name} on a partner (rule 23): {}; Connor's talk state goes on", notes.join("; ")));
        }
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
    /// Runs `script` with `args` (`$1`, `$2`, ...) at once, as a script
    /// line naming it would (M6 6e-4b: a portal's `exitScript` with its
    /// name and `Exit`, `world-change.md` rule 20.4). False if the world
    /// has no such script.
    pub fn run_script(&mut self, script: &str, args: &[String]) -> bool {
        let file = if script.contains('.') { script.to_string() } else { format!("{script}.cs") };
        if ScriptHost::read(&self.world, &file).is_none() {
            self.world.note(format!("script {file} {}: no such script in this world; nothing", args.join(" ")));
            return false;
        }
        self.world.note(format!("script {file} {}", args.join(" ")));
        self.scripts.call(&file, args, &mut self.world);
        true
    }

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
            "KQConner::setLevel" => Some(self.set_level(words, at)),
            c if c.starts_with("KQConner::setRope") => self.rope_command(words, at),
            "KQConner::getDoMode" => {
                // pushing.md "Words used here": the hand mode, sword 0, bow
                // 1, empty hands 4 (the game's help text); ranged-weapons.md
                // rule 1 (M5 5f-3).
                let mode = self.weapon_mode.number();
                self.note(format!("{at}: KQConner::getDoMode: {mode} ({:?} ready)", self.weapon_ready));
                Some(mode.to_string())
            }
            "KQConner::setDoMode" => {
                // ranged-weapons.md rule 9 (M5 5f-3): the request waits for
                // the viewer, which puts him into standing and plays the swap.
                // The optional movement word is not used (no Daventry script
                // gives one).
                let Some(mode) = words.get(1).and_then(|w| w.parse::<i64>().ok()).and_then(crate::weapons::WeaponMode::from_number) else {
                    self.note(format!("{at}: usage: KQConner::setDoMode 0|1|4 [movement]; nothing"));
                    return Some("1".into());
                };
                let extra = words.get(2).map_or(String::new(), |m| format!(" (movement {m} not used)"));
                self.note(format!("{at}: KQConner::setDoMode {}: asks for {}{extra} (ranged-weapons.md rule 9)", mode.number(), mode.name()));
                self.ask_weapon(crate::weapons::Ask::Script(mode), &format!("{at}: setDoMode {}", mode.number()));
                Some("0".into())
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
            "setCombatLevel" => Some(self.set_combat_level(words, at)),
            "KQMonster::setHome" => Some(self.set_home(words, at)),
            "KQObject::getLoc" | "getLoc" | "getloc" => Some(self.get_loc(words, at)),
            "KQObject::tossItems" => Some(self.toss_items(words, at)),
            "checkDistance" => Some(self.check_distance(words, at)),
            "KQMonster::goHome" | "KQMonster::setAlarmRadius" | "KQMonster::setChaseRadius" | "KQMonster::setHomeRadius" => Some(self.monster_ai_command(words, at)),
            "makeTranslucent" => Some(self.make_translucent(words, at)),
            "KQObject::setUseGravity" | "KQMonster::setMonUseGravity" | "KQMonster::setCanTurn" | "KQMonster::changeAnmFile" => {
                Some(self.character_switch(words, at))
            }
            "move" => Some(self.move_command(words, at)),
            "KQCamera::follow" => Some(self.camera_follow(words, at)),
            "handsOff" => Some(self.hands_off(words, at)),
            "setLowerGui" | "setUpperGui" | "setMapGui" | "getGuiStatus" => Some(self.gui_command(words, at)),
            "KQMap::activate" | "KQMap::deactivate" | "KQMap::stopLoc" | "KQMap::useFog" | "KQMap::setFogRadius" | "addMapIcon" | "removeMapIcon" => Some(self.map_command(words, at)),
            "onTeleport" | "offTeleport" | "earnTeleport" | "teleportersEarned" => Some(self.teleport_command(words, at)),
            "goldCoinPersists" => Some(self.gold_coin_persists(words, at)),
            "setConnorFlag" => Some(self.set_connor_flag(words, at)),
            "KQObject::dither" => Some(self.dither(words, at)),
            "KQConner::checkInvisible" => {
                let v = u8::from(self.connor_invisible());
                self.note(format!("{at}: KQConner::checkInvisible: {v} (potions.md rule 28)"));
                Some(v.to_string())
            }
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
            "KQConner::conversation" => Some(self.conversation_command(words, at)),
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
                potions: Vec::new(),
                status_icons: Default::default(),
                rope: Default::default(),
                connor_on_rope: false,
                connor_roping: false,
                rope_first_person_lock: false,
                rope_log: Vec::new(),
                map: None,
                map_marks: Default::default(),
                map_view: Default::default(),
                other_maps: BTreeMap::new(),
                map_item: None,
                map_log: Vec::new(),
                portals: BTreeMap::new(),
                connor_portal: None,
                portal_files: BTreeMap::new(),
                connor_in_temple4: false,
                connor_flags: BTreeMap::new(),
                connor_hands_off: false,
                connor_hidden_logged: None,
                reveal_objects: Vec::new(),
                pickup_experience: Vec::new(),
                helps_asked: BTreeSet::new(),
                reveal_calls: Vec::new(),
                dithered: BTreeSet::new(),
                item_voices: Vec::new(),
                hand: None,
                gold_persists: false,
                scroll_remark_at: None,
                stats: BTreeMap::new(),
                plain_hidden: BTreeMap::new(),
                dormant: BTreeMap::new(),
                deleted_this_event: BTreeSet::new(),
                calls_out: Vec::new(),
                templates: BTreeMap::new(),
                first_person_locked: false,
                was_first_person: false,
                connor_jump: None,
                talk: Talk::default(),
                notify: BTreeMap::new(),
                connor_can_turn: false,
                connor_at: None,
                monsters: Vec::new(),
                dormant_monsters: Vec::new(),
                characters_in_range: BTreeSet::new(),
                characters_solid: BTreeSet::new(),
                character_colliders: Vec::new(),
                character_log: Vec::new(),
                fight_log: Vec::new(),
                active_list: Vec::new(),
                max_monsters: crate::character_ai::DEFAULT_MAX_MONSTERS,
                connor_move_key: false,
                connor_back_key: false,
                connor_fight: None,
                connor_dying: false,
                connor_tips: [false; 2],
                fight_rng: crate::emitter::Rng::new(0x5EED_F1647),
                weapon_ready: None,
                weapon_mode: crate::weapons::WeaponMode::None,
                last_away_ranged: false,
                weapon_requests: Vec::new(),
                connor_projectile: None,
                connor_shoot_offset: [0.0; 3],
                weapon_swapped: false,
                combat_level: 1.0,
                ai_off: false,
                god_mode: false,
                corpses: Vec::new(),
                die_calls: Vec::new(),
                immune_calls: Vec::new(),
                hammer_calls: Vec::new(),
                hammers_back: Vec::new(),
                kill_tip: false,
                flights: Vec::new(),
                projectiles: Default::default(),
                bolts: Vec::new(),
                bolts_shot: 0,
                connor_was: None,
                conv_infos: Vec::new(),
                conversation: None,
                connor_leave: None,
                conversation_log: Vec::new(),
                connor_gestures: Vec::new(),
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
            let sound = if level.is_some() { self.world.level_up_sound(after.level) } else { String::new() };
            self.world.note(format!(
                "{head}: Connor {:?} level {} -> {:?} level {} (health / second / third / fourth / experience){}{sound}",
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
            // fighting.md rule 2 (M5 5c-1): what it brings to the fight.
            let mut a = a;
            // fighting.md rule 34 (M5 5c-9): holding a LongWeapon, doMode 1.
            let ranged = a.character.as_deref_mut().is_some_and(|c| {
                let r = c.fight.as_ref().is_some_and(|f| f.ranged());
                if let (true, Some(ai)) = (r, c.ai.as_mut()) {
                    ai.settings.do_mode = 1;
                }
                r
            });
            // fighting.md rule 3 (M5 5c-10): the combat level on entry.
            let factor = self.world.combat_level;
            let scaled = a.character.as_deref_mut().and_then(|c| c.fight.as_mut()).filter(|f| f.combat_scaled).map(|f| f.apply_combat_level(factor));
            if let Some(f) = a.character.as_deref().and_then(|c| c.fight.as_ref()) {
                let mut line = format!("fights (rule 2): {}; from {}", f.settings.describe(), f.settings.from);
                line.push_str(&match scaled {
                    Some(s) => format!("; {s}"),
                    None => "; loaded while applyCombatLevel was off: not scaled (rule 3)".to_string(),
                });
                if ranged {
                    line.push_str("; ranged: doMode 1 (rule 34)");
                }
                let name = a.player.name.clone();
                self.world.anims.push(a);
                self.world.fight_note(&name, line);
                continue;
            }
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
    pub(crate) fn call_named(&mut self, name: &str, args: &[&str]) -> bool {
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
            // A trap a script loads later gets its Init then (loadKQ).
            if self.world.traps[i].gone {
                continue;
            }
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

    /// An attack click's target (M5 5c fix 4, **stand-in**, Q139): the
    /// nearest character that fights (not dying) whose box (`BOUNDS.KQ`,
    /// as it stands) the line meets, whatever its collision type: the
    /// creatures' type 0x42 has no clickable bit (`characters.md` rule 8,
    /// `clicking.md` rule 1), yet `fighting.md` rule 39 says the clicked
    /// character becomes his target. Fix 5: also an object with a script
    /// and a box (the barrels, type 0x141), which a hit can kill
    /// (`doors.md` rules 19-20). Name and distance along the line.
    pub fn pick_attack(&self, origin: [f32; 3], dir: [f32; 3]) -> Option<(String, f32)> {
        let off = |n: &str| self.world.is_switched_off(n);
        let objects = self
            .world
            .clickables
            .iter()
            .filter(|c| c.script.is_some() && !c.gone && !off(&c.name))
            .filter_map(|c| Some((c.name.clone(), ray_box(origin, dir, &c.placement, c.min, c.max)?)));
        let animated = self.world.objects.iter().filter(|o| o.script.is_some() && !o.hidden && !o.gone && !off(o.name())).filter_map(|o| {
            let (min, max) = o.bounds?;
            Some((o.name().to_string(), ray_box(origin, dir, &o.player.placement(), min, max)?))
        });
        self.world
            .anims
            .iter()
            .filter(|a| !a.gone && !self.world.is_switched_off(&a.player.name))
            .filter_map(|a| {
                let c = a.character.as_deref()?;
                let f = c.fight.as_ref()?;
                if f.dying.is_some() {
                    return None;
                }
                let (min, max) = c.bounds?;
                Some((a.player.name.clone(), ray_box(origin, dir, &a.player.placement(), min, max)?))
            })
            .chain(objects)
            .chain(animated)
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }

    /// The middle of a clickable object's box in the world (for the
    /// viewer's `--aim` test option).
    pub fn target_point(&self, name: &str) -> Option<[f32; 3]> {
        let mid = |p: &Placement, (min, max): ([f32; 3], [f32; 3])| p.to_world(std::array::from_fn(|k| 0.5 * (min[k] + max[k])));
        let w = &self.world;
        if let Some(i) = w.door_index(name) {
            return w.doors[i].bounds.map(|b| mid(&w.doors[i].door.placement(), b));
        }
        // A character's box (M5 5c fix 4: `--aim` at a creature).
        if let Some(j) = w.character_index(name) {
            return w.anims[j].character.as_ref()?.bounds.map(|b| mid(&w.anims[j].player.placement(), b));
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
        // conversation.md rule 19 (M4 4e-4): in the talk state a world click
        // does nothing, except one on the partner with an item held.
        if let Some(c) = self.world.conversation.as_ref().filter(|c| c.entered()) {
            let partner = c.partner.clone();
            match item {
                Some(it) if name.eq_ignore_ascii_case(&partner) => {
                    let kind = self.world.items.kind(&it.kind).map_or_else(|| it.kind.clone(), |t| t.name.clone());
                    self.world.conversation_note(format!("click {name} with {} ({kind}) during the talk state: InvConv {kind} to its script, Connor leaves (with TalkExit), then the click goes on (rule 19)", it.object));
                    self.call_named(&partner, &["InvConv", &kind]);
                    self.conversation_leave("an item clicked on the partner", true);
                }
                _ => {
                    self.world.conversation_note(format!("click {name}{}: Connor is in his talk state: nothing (rule 19)", item.map_or(String::new(), |it| format!(" with {}", it.object))));
                    return ClickResult::Nothing;
                }
            }
        }
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
            // conversation.md rule 5: a character with an info object.
            if let Some(i) = found.filter(|&i| self.world.clickables[i].script.is_some()) {
                if let Some(info) = self.world.conv_infos.iter().find(|f| f.partner().eq_ignore_ascii_case(name)).cloned() {
                    return self.click_partner(i, &info, feet, item);
                }
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

    /// Connor clicks conversation partner `i` (`conversation.md` rule 5,
    /// M4 4e-4): no reach test; where he stands decides. In a special area
    /// (`box2`…): `ConnorAction Special Special`; else in the start area
    /// (`box0`): in reach whatever the distance (`do do`, or the held item
    /// as usual); else `TooFar TooFar` (`tooFarAwayAction` not looked at).
    /// The turn afterwards as for any click.
    fn click_partner(&mut self, i: usize, info: &kq8_data::conv_info::ConvInfo, feet: [f32; 3], item: Option<&Item>) -> ClickResult {
        let c = self.world.clickables[i].clone();
        if c.gone || c.collision_type & CLICKABLE == 0 {
            self.world.note(format!("click {}: deleted or not clickable now (type {:#x}); nothing", c.name, c.collision_type));
            return ClickResult::Nothing;
        }
        let p = [feet[0], feet[1]];
        let special = info.special_area(p);
        let start = info.in_area(0, p);
        let head = format!(
            "click {} (conversation partner, {}): Connor at ({:.0}, {:.0}, {:.0}): {}",
            c.name,
            info.name,
            feet[0],
            feet[1],
            feet[2],
            match (special, start) {
                (Some(k), _) => format!("in special area box{k}: Special (rule 5.1)"),
                (None, true) => "in the start area box0: in reach whatever the distance (rule 5.2)".to_string(),
                (None, false) => "in no area: TooFar (rule 5.3; tooFarAwayAction not looked at)".to_string(),
            }
        );
        let args = match (special, start) {
            (Some(_), _) => vec!["ConnorAction".to_string(), "Special".into(), "Special".into()],
            (None, true) => {
                let rules = ClickRules { name: &c.name, too_far_away_action: c.too_far_away_action };
                match self.script_args(&head, true, item, &rules) {
                    Ok(a) => a,
                    Err(r) => return r,
                }
            }
            (None, false) => vec!["ConnorAction".to_string(), "TooFar".into(), "TooFar".into()],
        };
        self.world.note(format!("{head}; script arguments: {} {}", c.name, args.join(" ")));
        self.world.conversation_note(format!("{head}: {} {}", c.name, args.join(" ")));
        let refs: Vec<&str> = args.iter().map(String::as_str).collect();
        self.call_clickable(i, &refs);
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

    /// The events due by `now`, in time order, until none is left: what
    /// they send with delay 0 is handled in the same pass (`anim-scripts.md`
    /// rule 8, Q119).
    fn handle_due(&mut self, now: f64) -> usize {
        let mut handled = 0;
        while let Some(e) = self.world.queue.take_due(now) {
            self.handle(e);
            handled += 1;
            if handled > 1000 {
                self.world.note("more than 1000 events in one update; the rest wait (a guard of ours)");
                break;
            }
        }
        handled
    }

    /// One update (15 per second): events due, then the doors move; then
    /// the events sent meanwhile with delay 0 (the ends of movements,
    /// `EndLoop`, `Cue 300`, scripts' `sendEvent 0`), before the frame is
    /// drawn (`anim-scripts.md` rule 8, Q119; M4 4e-10).
    pub fn update(&mut self) {
        let now = self.world.now();
        self.handle_due(now);
        self.world.characters_due(now);
        self.talk_step(now);
        // M5 5g-2: the potions' whole seconds (potions.md rule 33).
        crate::connor_inventory::timers(&mut self.world);
        // M5 5g-9: what the characters are told (potions.md rule 26).
        self.world.note_connor_report();
        // M6 6a-1: the hook icon (ropeandhook.md rule 7).
        self.world.rope_icon_step();
        // M5 5g-7: Clarity's Reveal calls (potions.md rule 31).
        for (name, on) in std::mem::take(&mut self.world.reveal_calls) {
            let word = if on { "True" } else { "False" };
            let line = if self.call_named(&name, &["Reveal", word]) { format!("{name}: its script gets Reveal {word}") } else { format!("{name}: no script: Reveal {word} not called") };
            self.world.item_note(format!("clarity: {line} (potions.md rule 31)"));
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
            let connor_at = self.world.connor_at.map(|p| [p[0], p[1]]);
            let obj = &mut self.world.anims[i];
            let (a, notes) = match obj.character.as_deref_mut() {
                Some(c) => {
                    c.in_range = in_range;
                    c.connor_at = connor_at;
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
                // M5 5c: the character's own fight lines go to fight.log too.
                match n.strip_prefix("fight: ") {
                    Some(f) => self.world.fight_note(&who, f),
                    None => self.world.character_note(&who, n),
                }
            }
            // A character's click box is where the character is (after a
            // script's `move` or a turn, M4 4d-4).
            if self.world.anims[i].character.is_some() {
                let now_at = self.world.anims[i].player.placement();
                if let Some(c) = self.world.clickables.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&who)) {
                    c.placement = now_at;
                }
            }
            let p = &self.world.anims[i].player;
            let (name, at) = (p.name.clone(), p.placement().position);
            for cue in &a.cues {
                // fighting.md rule 13 (M5 5c): a fighter's `Do` cue.
                if cue.trim().eq_ignore_ascii_case("do") && self.world.anims[i].character.as_deref().is_some_and(|c| c.fight.is_some()) {
                    self.world.character_do(i);
                    continue;
                }
                let what = self.world.cue_sound(&format!("object {name}"), cue, at);
                self.world.note(format!("object {name}: cue '{cue}' fires ({what})"));
            }
            if let Some(st) = &a.started {
                let why = if a.finished.is_some() { "rule 8: the movement before it ended" } else { "rule 3: no current movement name" };
                let line = started_line(&format!("object {name}"), st, &self.world.anims[i].player.movements, why);
                self.world.note(line);
            }
            if let Some(m) = a.finished {
                // anim-scripts.md rule 11 (M5 5d-1): a KQAnimOnce's end.
                if self.world.anims[i].player.once {
                    self.world.note(format!("object {name} (KQAnimOnce): movement '{m}' finished: its script gets Die {DIE_KIND} (rules 11, 20)"));
                    if !self.call_named(&name, &["Die", DIE_KIND]) {
                        self.world.note(format!("object {name} (KQAnimOnce): no script; Die not called"));
                    }
                    self.world.report_end(&name, &m, false);
                    if self.world.anims[i].player.delete_when_done {
                        let at = format!("{name}: KQAnimOnce done");
                        if self.world.delete_one(&name, &at) {
                            self.world.note(format!("object {name} (KQAnimOnce): deleted (setDeleteWhenDone, rule 11.3)"));
                        }
                    }
                    continue;
                }
                let after = if a.started.is_some() { "the waiting movement plays" } else { "stays on its last frame" };
                self.world.note(format!("object {name}: movement '{m}' finished; {after}"));
                let w = &self.world;
                let has_script = w.anims[i].script.is_some() || w.clickables.iter().any(|c| c.script.is_some() && c.name.eq_ignore_ascii_case(&name));
                self.world.movement_ended(&name, &m, has_script);
                // fighting.md rule 32 (M5 5c-7): its die movement ended.
                let dying = self.world.anims[i].character.as_deref().and_then(|c| c.fight.as_ref()).and_then(|f| f.dying.clone());
                if dying.flatten().is_some_and(|d| d.eq_ignore_ascii_case(&m)) {
                    self.world.death_ended(i, now);
                }
            }
        }
        // Rule 32.3: the dead's scripts get `Die <kind>`.
        for (name, kind) in std::mem::take(&mut self.world.die_calls) {
            if !self.call_named(&name, &["Die", kind]) {
                self.world.fight_note(&name, format!("no script: Die {kind} not called (rule 32)"));
            }
        }
        for name in std::mem::take(&mut self.world.immune_calls) {
            if !self.call_named(&name, &["ProjImmune"]) {
                self.world.fight_note(&name, "no script: ProjImmune not called (ranged-weapons.md rule 30)");
            }
        }
        // azriel-hammer.md rule 8 (M6 6b-2): the hammer on the lever.
        for name in std::mem::take(&mut self.world.hammer_calls) {
            if !self.call_named(&name, &["ConnorAction"]) {
                self.world.fight_note(&name, "no script: ConnorAction from the hammer not called (azriel-hammer.md rule 8)");
            }
        }
        // Thrown items fly after the characters' movers (M5 5e-1:
        // `World::flights_step`, called with the buildings and objects).
        // Q119: nothing sent with delay 0 waits for the next update.
        let late = self.handle_due(now);
        if late > 0 {
            self.world.note(format!("{late} event(s) sent with delay 0 during this update handled before drawing (anim-scripts.md rule 8, Q119)"));
        }
        self.world.characters_due(now);
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
        let fresh = talk.events.len();
        let finished = talk.update(now, captions, &exists);
        // Characters (M4 4d-5, characters.md rules 25-28): each mouth code
        // and line end.
        for e in &talk.events {
            match e {
                // conversation.md rule 18: Connor's gesture codes, for his
                // walker (the caller's).
                crate::talk::TalkEvent::Mouth { speaker, shape: shape @ 8..=16, .. } if speaker.eq_ignore_ascii_case(CONNOR) => {
                    self.world.connor_gestures.push(*shape);
                }
                crate::talk::TalkEvent::Mouth { speaker, shape, .. } => {
                    if let Some(i) = self.world.character_index(speaker) {
                        self.world.start_up_character(i, &format!("lip-sync code {shape}"));
                        let a = &mut self.world.anims[i];
                        let line = a.character.as_mut().unwrap().mouth_code(&mut a.player, *shape);
                        self.world.character_note(speaker, line);
                    }
                }
                crate::talk::TalkEvent::End { speaker, .. } => {
                    if let Some(i) = self.world.character_index(speaker) {
                        let c = self.world.anims[i].character.as_mut().unwrap();
                        if !c.geometry && c.mouth != 0 {
                            c.mouth = 0;
                            self.world.character_note(speaker, "line ended: mouth shape 0 (rule 25)");
                        }
                    }
                }
                _ => {}
            }
        }
        let goodbyes = std::mem::take(&mut talk.goodbyes_ended);
        // conversation.md rule 14: a conversation line with a camera number.
        let cuts: Vec<(String, u32, u64)> = talk.events[fresh..]
            .iter()
            .filter_map(|e| match e {
                crate::talk::TalkEvent::Start { issuer, camera: Some(n), conv: true, id, .. } => Some((issuer.clone(), *n, *id)),
                _ => None,
            })
            .collect();
        self.world.talk = talk;
        for (partner, n, id) in cuts {
            self.conversation_camera(&partner, n, id);
        }
        for f in finished {
            // conversation.md rule 15: the conversation's last line ends
            // Connor's talk state, TalkExit before its TalkComplete.
            let ends = f.conv && f.conv_end && self.world.conversation.as_ref().is_some_and(|c| c.entered() && c.partner.eq_ignore_ascii_case(&f.issuer));
            if ends {
                self.conversation_leave("the conversation's last line ended", true);
            }
            // talking.md rule 13a (Q117): the line marked last of a speak in
            // a character's queue: Stop and Wait, before its TalkComplete.
            if f.last {
                if let Some(i) = self.world.character_index(&f.issuer) {
                    self.world.start_up_character(i, "the end of its speak");
                    let a = &mut self.world.anims[i];
                    let line = a.character.as_mut().unwrap().after_speech(&mut a.player);
                    let name = a.player.name.clone();
                    self.world.character_note(&name, format!("line #{}: {line}", f.id));
                }
            }
            self.talk_complete(&f, now);
        }
        // Rule 22: a goodbye ended: TalkExit (no TalkComplete).
        for partner in goodbyes {
            self.talk_exit(&partner);
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
            self.world.start_up_character(i, &format!("KQAnimPlayEvent {}", e.data.join(" ")));
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

/// The `dirX dirY dirZ` angles of a rotation, the reverse of
/// `kq8_data::objects::rotation` (y, then x, then z, row vectors): the
/// second row is (−cos x sin z, cos x cos z, sin x).
pub fn angles_of(r: &[[f32; 3]; 3]) -> [f32; 3] {
    let x = r[1][2].clamp(-1.0, 1.0).asin();
    let z = (-r[1][0]).atan2(r[1][1]);
    let y = (-r[0][2]).atan2(r[2][2]);
    [x, y, z]
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
        // Open at once; EndLoop and the listener's cue go through the queue,
        // handled in the same update before drawing (anim-scripts.md rule 8,
        // Q119).
        assert_eq!(a.scripts.vars.get("D::Open"), Some("yes"));
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
    fn a_trap_loaded_while_playing_works_only_from_its_load_kq() {
        // objects.md rule 6, traps.md rules 1, 2: `ax.cs` loads
        // `henchtp.kq` when Connor takes the axe; its box counts from then.
        use crate::trap::TrapSettings;
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("trap.cs".to_string(), TRAP.as_bytes().to_vec())]);
        let settings = TrapSettings { name: "Henchtp".into(), position: [0.0; 3], min: [-500.0, -500.0, 0.0], max: [500.0, 500.0, 500.0], script: Some("trap.cs".into()), mask: 4, trap_type: 0, inner_percent: None };
        a.add_traps([Trap { gone: true, ..Trap::new(settings) }]);
        a.set_dormant([("henchtp.kq".to_string(), "Henchtp".to_string(), Some("trap.cs".to_string()))]);
        a.start();
        assert_eq!(a.scripts.vars.get("T::Init"), None, "no Init before its loadKQ");
        assert!(connor_at(&mut a, 0.0).is_empty(), "switched off: Connor walks through");
        connor_at(&mut a, 5000.0);
        assert_eq!(a.test_command("loadKQ henchtp.kq", "t").as_deref(), Some("0"));
        assert!(!a.world.traps[0].gone, "its box is in place");
        assert!(a.take_log().join("
").contains("its trap box is in place now"));
        assert_eq!(connor_at(&mut a, 0.0), vec!["Henchtp On Connor".to_string()]);
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
        // cue 3333 goes through the queue, handled in that same update
        // (Q119).
        for _ in 0..15 {
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
    fn a_kq_anim_once_among_the_other_animated_objects_reports_die_too() {
        // M5 5d-1 (the barrels' burst): rule 11 for `world.anims`.
        let script = "if test $2 == Init\n setDeleteWhenDone $1 1\n sendEvent 0 $1 KQAnimPlayEvent open\nendif\nif test $2 == Die\n set D::died $1 $3\nendif\n";
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("o.cs".to_string(), script.as_bytes().to_vec())]);
        let mut player = crate::named_anim::tests::still_then_open("burst");
        player.once = true;
        a.add_anims([AnimObject { player, script: Some("o.cs".into()), gone: false, character: None, translucency: None }]);
        a.call_named("burst", &["Init"]);
        let mut log = String::new();
        for _ in 0..25 {
            a.update();
            log.push_str(&a.take_log().join("\n"));
        }
        assert_eq!(a.scripts.vars.get("D::died"), Some("burst Normal"), "{log}");
        assert!(log.contains("(KQAnimOnce): deleted (setDeleteWhenDone"), "{log}");
        assert!(!log.contains("EndLoop open"), "no EndLoop for a KQAnimOnce: {log}");
    }

    #[test]
    fn load_kq_while_playing_makes_new_copies_by_the_name_and_place_rules() {
        // M5 5d-2: objects.md rules 2, 6; anim-scripts.md rule 12.
        let script = "if test $2 == ConnorAction\n loadKQ burst.kq @ 100 200 300 0 0 1.57\n set T::n $ObjectName\n loadKQ burst.kq @ 400 200 300\n set T::m $ObjectName\n loadKQ wreck.kq same\n loadKQ wreck.kq same\n loadKQ wreck.kq other\nendif\n";
        let mut a = with_object(script, "x");
        let player = crate::named_anim::tests::still_then_open("barrelsm");
        let at = Placement::at([0.0; 3]);
        a.world.set_templates([
            ("BURST.KQ".to_string(), Template { name: "barrelsm".into(), script: None, placement: at, kind: TemplateKind::Anim(Box::new(player)) }),
            ("wreck.kq".to_string(), Template { name: "barelded".into(), script: None, placement: Placement::at([7.0, 8.0, 9.0]), kind: TemplateKind::Still { bounds: Some(([-1.0; 3], [1.0; 3])), collision_type: 0x41 } }),
        ]);
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.take_log().join("\n");
        let (n, m) = (a.scripts.vars.get("T::n").map(str::to_string), a.scripts.vars.get("T::m").map(str::to_string));
        assert_eq!((n.as_deref(), m.as_deref()), (Some("NoName0"), Some("NoName1")), "@: the counter, at the line: {log}");
        let burst = a.world.anims.iter().find(|o| o.player.name == "NoName0").expect("in the world at once");
        assert_eq!(burst.player.base.position, [100.0, 200.0, 300.0]);
        assert!((burst.player.base.rotation[0][0] - 1.57f32.cos()).abs() < 1e-4, "the angles given");
        let second = a.world.anims.iter().find(|o| o.player.name == "NoName1").unwrap();
        assert_eq!(second.player.base.rotation, kq8_data::objects::rotation([0.0; 3]), "no angles: 0 0 0");
        let wreck = a.world.clickables.iter().find(|c| c.name == "barelded").expect("same: the file's name");
        assert_eq!(wreck.placement.position, [7.0, 8.0, 9.0], "no place: the file's");
        assert!(log.contains("ERROR loadKQ wreck.kq: a live object already has that name (barelded)"), "{log}");
        assert!(a.world.clickables.iter().any(|c| c.name == "other"));
        let spawns = a.world.requests.iter().filter(|r| matches!(r, Request::Spawn { .. })).count();
        assert_eq!(spawns, 4);
    }

    #[test]
    fn toss_items_throws_one_of_each_type_from_the_object_raised() {
        // M5 5d-5: items.md rule 12.
        let script = "if test $2 == ConnorAction\n KQObject::tossItems $1 100 Crystal,INVITEM_Crystal\n KQObject::tossItems nosuch 0 Crystal\nendif\n";
        let mut a = with_object(script, "x");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let log = a.world.item_log.join("\n");
        assert_eq!(a.world.flights.len(), 2, "{log}");
        assert!(a.world.flights.iter().all(|f| f.from == [0.0, 0.0, 100.0]), "raised by offsetZ: {log}");
        assert!(a.take_log().join("\n").contains("ERROR tossItems: no object 'nosuch'; nothing made"));
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
    fn a_character_s_click_box_follows_it() {
        let mut a = with_object("x\n", "x");
        let mut player = crate::named_anim::tests::still_then_open("wiz");
        player.movements[0].name = "stop".into();
        let character = crate::character::Character::new(false, 0x20, 0.0, false, false, true, 1);
        a.add_anims([AnimObject { player, script: None, gone: false, character: Some(Box::new(character)), translucency: None }]);
        let at = Placement { rotation: crate::door::identity().m, position: [0.0; 3] };
        a.world.clickables.push(Clickable::plain("wiz", "KQMonster", [-100.0; 3], [100.0; 3], at, 33));
        a.world.characters_in_range.insert("wiz".into());
        assert_eq!(a.test_command("move wiz 0 5000 0 0 0 0 0", "t").as_deref(), Some("0"));
        a.update();
        let c = a.world.clickables.iter().find(|c| c.name == "wiz").unwrap();
        assert_eq!(c.placement.position, [5000.0, 0.0, 0.0]);
    }

    struct FlatGround;
    impl crate::connor_mover::GroundLookup for FlatGround {
        fn ground(&self, _x: f32, _y: f32) -> Option<(f32, [f32; 3])> {
            Some((100.0, [0.0, 0.0, 1.0]))
        }
    }

    /// M5 5a-3: a goblin walking −Y at 917 for 25 updates from (0, 0,
    /// 100) (it moves 24 × 917 / 15 = 1,467; not long enough for a stuck
    /// escape); where does it end?
    fn walk_into(setup: impl FnOnce(&mut Active, &mut Vec<crate::collide::BoxCollider>)) -> ([f32; 3], String) {
        let mut a = with_object("x\n", "x");
        let (mut c, mut p) = crate::character::tests::walker(917.0, 3068.0);
        p.name = "gob".into();
        c.collision_type = 0x42;
        c.bounds = Some(([-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0]));
        a.add_anims([AnimObject { player: p, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
        a.world.characters_in_range.insert("gob".into());
        a.world.characters_solid.insert("gob".into());
        let mut objects = Vec::new();
        setup(&mut a, &mut objects);
        assert_eq!(a.test_command("KQMonster::setState gob Move", "t").as_deref(), Some("0"));
        for _ in 0..25 {
            a.update();
            a.world.characters_gravity(&FlatGround, &[], &objects);
        }
        let log = a.world.character_log.join("\n");
        (a.world.anims[0].player.base.position, log)
    }

    #[test]
    fn a_character_s_own_move_meets_objects_characters_and_connor() {
        // character-movement.md rule 33 (Q128): obstacles in its own move.
        let (free, _) = walk_into(|_, _| {});
        assert!((free[1] + 1467.0).abs() < 1.0, "nothing in the way: {free:?}");
        // A plain object (type 0x41) across its way at y −2000: it stops
        // with its ball a radius (800) short of the box, less 25.
        let (at, log) = walk_into(|_, o| {
            let mut b = crate::collide::BoxCollider { name: "cart".into(), min: [-3000.0, -100.0, 0.0], max: [3000.0, 100.0, 1000.0], placement: Placement { rotation: crate::door::identity().m, position: [0.0, -2000.0, 100.0] }, kind: Default::default() };
            b.kind.collision_type = 0x41;
            o.push(b);
        });
        assert!(at[1] > -1100.0 && at[1] < -1000.0, "{at:?}");
        assert!(log.contains("own move: across pass hit cart"), "{log}");
        // A trap (0x10): nothing.
        let (at, _) = walk_into(|_, o| {
            let mut b = crate::collide::BoxCollider { name: "trap".into(), min: [-3000.0, -100.0, 0.0], max: [3000.0, 100.0, 1000.0], placement: Placement { rotation: crate::door::identity().m, position: [0.0, -2000.0, 100.0] }, kind: Default::default() };
            b.kind.collision_type = 0x10;
            o.push(b);
        });
        assert!((at[1] + 1467.0).abs() < 1.0, "{at:?}");
        // Connor standing in its way (stand-in cylinder, radius 200): it
        // stops 800 + 200 from his position, less 25.
        let (at, log) = walk_into(|a, _| a.world.connor_at = Some([0.0, -2300.0, 100.0]));
        assert!(at[1] > -1300.0 && at[1] < -1250.0, "{at:?} {log}");
        assert!(log.contains("hit Connor"), "{log}");
        // Another goblin, standing (solid): 800 + 800.
        let (at, log) = walk_into(|a, _| {
            let mut player = crate::named_anim::tests::still_then_open("gob2");
            player.movements[0].name = "stop".into();
            player.base.position = [0.0, -2800.0, 100.0];
            let mut c = crate::character::Character::new(false, 0x20, 0.0, false, false, false, 2);
            c.bounds = Some(([-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0]));
            a.add_anims([AnimObject { player, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
            a.world.characters_solid.insert("gob2".into());
        });
        assert!(at[1] > -1200.0 && at[1] < -1150.0, "{at:?} {log}");
        assert!(log.contains("hit gob2"), "{log}");
        // colliderMask 0 (the Wizard, the Knight): through everything.
        let (at, _) = walk_into(|a, _| {
            a.world.connor_at = Some([0.0, -2300.0, 100.0]);
            a.world.anims[0].character.as_mut().unwrap().collision_mask = 0;
        });
        assert!((at[1] + 1467.0).abs() < 1.0, "{at:?}");
    }

    /// M5 5a-4: standing characters with their movement rules, in Wait.
    fn watchers(names: &[(&str, Option<&str>, f32)]) -> Active {
        let mut a = with_object("x\n", "x");
        for (k, (name, team, alarm)) in names.iter().enumerate() {
            let mut player = crate::named_anim::tests::still_then_open(name);
            player.movements[0].name = "stop".into();
            player.base.position = [k as f32 * 1000.0, 0.0, 100.0];
            let mut c = crate::character::Character::new(false, 0x20, 0.0, false, false, false, k as u64);
            c.bounds = Some(([-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0]));
            let kq = kq8_data::kq_text::parse(b"[Gob]\nclassType=KQMonster\nanimFile=g.anm\n");
            let mut s = kq8_data::characters::AiSettings::from_keys(&kq8_data::characters::CharacterKeys::from_section(&kq[0]).unwrap());
            s.team = team.map(str::to_string);
            s.alarm_radius = *alarm;
            c.ai = Some(crate::character_ai::Ai::new(s, player.base.position, 1300.0, false));
            a.add_anims([AnimObject { player, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
            a.world.characters_in_range.insert(name.to_ascii_lowercase());
        }
        a.world.connor_at = Some([0.0, -5000.0, 100.0]);
        a
    }

    /// Updates until update `to`, with the movement rules.
    fn watch(a: &mut Active, objects: &[crate::collide::BoxCollider], to: u64) {
        while a.world.updates < to {
            a.update();
            a.world.characters_ai(&FlatGround, &[], objects);
            a.world.characters_gravity(&FlatGround, &[], objects);
        }
    }

    #[test]
    fn characters_see_connor_and_join_the_active_list() {
        // Rule 20: the sight test every 15th update, from the box top; in
        // Wait, seen → active (at most MaxMonsters).
        let mut a = watchers(&[("g1", None, 0.0), ("g2", None, 0.0), ("g3", Some("windguys"), 0.0)]);
        a.world.max_monsters = 2;
        watch(&mut a, &[], 14);
        assert!(!a.world.character_log.join("\n").contains("sight"), "the first test is at update 15");
        assert!(a.world.active_list.is_empty());
        watch(&mut a, &[], 16);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("update 15: g1: sight (rule 20): can see Connor (a clear line; eye 1300"), "{log}");
        assert!(log.contains("g1: status:") && log.contains("Connor 5000 away"), "{log}");
        // g1 and g2 fill the two places; g3 (a team) takes g1's.
        assert_eq!(a.world.active_list, ["g2", "g3"], "{}", a.world.character_log.join("\n"));
        let log = a.world.character_log.join("\n");
        assert!(log.contains("g1: leaves the active list (rule 21): pushed off the active list by g3 (team windguys)"), "{log}");
        // Alarm radius 0: Connor is always farther than twice it: home.
        assert!(log.contains("put back on its home point (0, 0, 100) at once"), "{log}");
    }

    #[test]
    fn a_plain_object_blocks_the_sight_line_a_character_does_not() {
        let mut a = watchers(&[("g1", None, 0.0)]);
        // A wall of a box between them (type 0x41).
        let mut wall = crate::collide::BoxCollider { name: "cart".into(), min: [-3000.0, -100.0, 0.0], max: [3000.0, 100.0, 3000.0], placement: Placement { rotation: crate::door::identity().m, position: [0.0, -2500.0, 100.0] }, kind: Default::default() };
        wall.kind.collision_type = 0x41;
        watch(&mut a, std::slice::from_ref(&wall), 20);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("g1: sight (rule 20): can't see Connor (blocked by object cart"), "{log}");
        assert!(a.world.active_list.is_empty());
        // A character's type (0x42) is not in the line's mask: no block.
        wall.kind.collision_type = 0x42;
        watch(&mut a, std::slice::from_ref(&wall), 31);
        assert_eq!(a.world.active_list, ["g1"], "{}", a.world.character_log.join("\n"));
        // Going off takes it off the list; Connor near (alarm 3,000:
        // within 6,000): it stays where it is.
        a.world.anims[0].character.as_mut().unwrap().ai.as_mut().unwrap().settings.alarm_radius = 3000.0;
        a.world.anims[0].player.base.position = [0.0, -1000.0, 100.0];
        a.world.characters_in_range.clear();
        watch(&mut a, &[], 32);
        assert!(a.world.active_list.is_empty());
        assert_eq!(a.world.anims[0].player.base.position, [0.0, -1000.0, 100.0]);
        assert!(a.world.character_log.join("\n").contains("went off (neither near nor far); Connor 4000 away (2 × alarm radius 6000)"));
    }

    /// M5 5a-5: a goblin walker (917 / 3,068) at (0, 0, 100) with the
    /// movement field `wander`, seeing Connor.
    fn wanderer(wander: kq8_data::characters::Wander) -> Active {
        let mut a = with_object("x\n", "x");
        let (mut c, mut p) = crate::character::tests::walker(917.0, 3068.0);
        p.name = "gob".into();
        c.bounds = Some(([-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0]));
        let kq = kq8_data::kq_text::parse(b"[Gob]\nclassType=KQMonster\nanimFile=g.anm\n");
        let mut s = kq8_data::characters::AiSettings::from_keys(&kq8_data::characters::CharacterKeys::from_section(&kq[0]).unwrap());
        s.wanders = wander.wanders();
        s.wander = wander;
        s.alarm_radius = 1000.0;
        c.ai = Some(crate::character_ai::Ai::new(s, p.base.position, 1300.0, false));
        a.add_anims([AnimObject { player: p, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
        a.world.characters_in_range.insert("gob".into());
        a.world.connor_at = Some([20_000.0, 0.0, 100.0]);
        a
    }

    #[test]
    fn an_active_wanderer_follows_its_points() {
        // Rules 9–11: active → Move; the first point (its own position) is
        // reached at once, it turns to the second and walks there, then
        // back to the first.
        use kq8_data::characters::Wander;
        let mut a = wanderer(Wander::Loop(vec![[0.0, 0.0, -1.0], [0.0, -5000.0, -1.0]]));
        watch(&mut a, &[], 14);
        assert_eq!(a.world.anims[0].character.as_ref().unwrap().state, crate::character::State::Wait, "not active before the first sight test");
        watch(&mut a, &[], 18);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("wander points (rule 10): loop: (0, 0) (0, -5000)"), "{log}");
        assert!(log.contains("state Move (from Wait; wandering"), "{log}");
        assert!(log.contains("point 1 reached (0 < 250); heads for point 2 (0, -5000), 5000 away"), "{log}");
        // 5,000 − 250 at 917 a second: about 5.2 s.
        watch(&mut a, &[], 18 + 15 * 6);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("point 2 reached") && log.contains("heads for point 1 (0, 0)"), "{log}");
        assert!(!log.contains("is stuck"), "{log}");
        // It turns round (3.5 rad/s) and walks back.
        watch(&mut a, &[], 18 + 15 * 8);
        let y = a.world.anims[0].player.base.position[1];
        assert!(y > -4600.0, "on its way back: {y}");
    }

    #[test]
    fn an_active_character_fidgets_then_walks_on() {
        // Rules 15–18: the first fidget as soon as it is active, then Wait's
        // Stop and walking; the next due 2–12 s after the fidget ended; a
        // walking wanderer stops for it.
        use kq8_data::characters::Wander;
        let mut a = wanderer(Wander::Loop(vec![[0.0, 0.0, -1.0], [0.0, -50_000.0, -1.0]]));
        a.world.anims[0].player.movements.push(crate::named_anim::tests::movement("idle:react", 0.0, 1.0, false, false, vec![]));
        watch(&mut a, &[], 17);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("idle time: fidget 'react' (1 of 1, at random) starts at once"), "{log}");
        assert!(!log.contains("state Move"), "no walking during the fidget: {log}");
        // The fidget lasts 1 s; then Stop, and walking on.
        watch(&mut a, &[], 40);
        let log = a.world.character_log.join("\n");
        let due_line = log.lines().find(|l| l.contains("fidget 'react' ended")).expect("ended");
        assert!(log.contains("'idle:react' ended in Wait: asks for Stop again"), "{log}");
        assert!(log.contains("state Move (from Wait; wandering"), "{log}");
        let secs: f64 = due_line.split("due in ").nth(1).unwrap().split(' ').next().unwrap().parse().unwrap();
        assert!((2.0..=12.0).contains(&secs), "{due_line}");
        // When it is due again it stops walking and fidgets.
        watch(&mut a, &[], 40 + ((secs + 1.0) * 15.0) as u64);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("idle time: stops walking for a fidget"), "{log}");
        assert_eq!(log.matches("idle time: fidget 'react'").count(), 2, "{log}");
    }

    #[test]
    fn an_alarmed_character_chases_stops_at_reach_gives_up_and_goes_home() {
        // Rules 23–28 (the fight itself is the next study).
        use kq8_data::characters::Wander;
        let mut a = wanderer(Wander::None);
        {
            let ai = a.world.anims[0].character.as_mut().unwrap().ai.as_mut().unwrap();
            (ai.settings.alarm_radius, ai.settings.attack_dist, ai.settings.home_radius, ai.settings.chase_radius) = (4000.0, 1500.0, 3000.0, 6000.0);
        }
        a.world.connor_at = Some([0.0, -3500.0, 100.0]);
        watch(&mut a, &[], 16);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("alarmed: sees Connor 3500 away, within its alarm radius 4000 (rule 23); reacts in"), "{log}");
        // Within 1 s it reacts: after him (walking or running).
        watch(&mut a, &[], 32);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("state Move (from Wait; after Connor"), "{log}");
        // It stops within its attack distance, facing him.
        watch(&mut a, &[], 90);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("within its attack distance 1500, and he has no forward / back key: stops, facing him") && log.contains("its attack is not due yet"), "{log}");
        let y = a.world.anims[0].player.base.position[1];
        assert!(y < -1900.0 && y > -2100.0, "1,500 short of him, give or take one update: {y}");
        // Connor runs far off (beyond twice the alarm radius): after the
        // chase timer it gives up and goes home, where it waits.
        a.world.connor_at = Some([0.0, -20_000.0, 100.0]);
        watch(&mut a, &[], 120);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("gives up: Connor") && log.contains("beyond twice its alarm radius (rule 27); on its way home"), "{log}");
        watch(&mut a, &[], 300);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("home reached (0, 0)"), "{log}");
        assert_eq!(a.world.anims[0].character.as_ref().unwrap().state, crate::character::State::Wait);
    }

    #[test]
    fn a_zombie_rises_chases_and_sinks_back() {
        // character-movement.md rules 28, 30 (M5 5a-8).
        let mut a = with_object("x\n", "x");
        let (_, mut p) = crate::character::tests::walker(511.0, 511.0);
        p.name = "zom".into();
        p.movements.push(crate::named_anim::tests::movement("birth", 0.0, 1.0, false, false, vec![]));
        p.movements.push(crate::named_anim::tests::movement("hide", 0.0, 1.0, false, false, vec![]));
        let mut c = crate::character::Character::new(false, 0x10, 1.0, true, false, true, 3);
        c.in_range = true;
        c.unborn = true;
        c.bounds = Some(([-350.0, -350.0, 0.0], [350.0, 350.0, 1750.0]));
        c.mover = Some(Box::new(crate::connor_mover::Mover::character(p.base.position, true, [-350.0, -350.0, 0.0], [350.0, 350.0, 1750.0], (0x42, 0x281F), (325.0, 325.0), -1)));
        let kq = kq8_data::kq_text::parse(b"[Zombie]\nclassType=KQMonster\nanimFile=zombie.anm\nwanders=0\n");
        let mut s = kq8_data::characters::AiSettings::from_keys(&kq8_data::characters::CharacterKeys::from_section(&kq[0]).unwrap());
        (s.alarm_radius, s.attack_dist, s.home_radius, s.chase_radius) = (4000.0, 1100.0, 8000.0, 12000.0);
        let mut ai = crate::character_ai::Ai::new(s, p.base.position, 1750.0, true);
        ai.has_birth = true;
        c.ai = Some(ai);
        a.add_anims([AnimObject { player: p, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
        a.world.characters_in_range.insert("zom".into());
        a.world.connor_at = Some([0.0, -3000.0, 100.0]);
        watch(&mut a, &[], 16);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("alarmed: seen from underground within its alarm radius (rule 30)") && log.contains("rises in 1 s"), "{log}");
        assert!(a.world.anims[0].character.as_ref().unwrap().unborn, "still underground");
        watch(&mut a, &[], 32);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("rises (rule 30): Connor 3000 away") && log.contains("'birth' starts at once"), "{log}");
        let c = a.world.anims[0].character.as_ref().unwrap();
        assert!(!c.unborn && c.state != crate::character::State::Birth);
        // After 'birth' it comes after him.
        watch(&mut a, &[], 60);
        assert!(a.world.character_log.join("\n").contains("state Move (from Wait; after Connor"));
        // Connor far away: it gives up, sinks back ('hide'), underground again.
        a.world.connor_at = Some([0.0, -50_000.0, 100.0]);
        watch(&mut a, &[], 140);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("its list has hide: Birth, 'hide'") && log.contains("'hide' ended in Birth: underground again"), "{log}");
        assert!(a.world.anims[0].character.as_ref().unwrap().unborn);
        // When `hide` ended it left the list and is back on its home point.
        assert!(log.contains("underground (rule 30): its 'hide' ended; off the active list") && log.contains("on its home point (0, 0, 100)"), "{log}");
        assert!(a.world.active_list.is_empty());
        assert_eq!(a.world.anims[0].player.base.position, [0.0, 0.0, 100.0]);
        // Connor comes back within its alarm radius: it rises again by the
        // rising test (rule 30), from home.
        a.world.connor_at = Some([0.0, -3000.0, 100.0]);
        watch(&mut a, &[], 200);
        let log = a.world.character_log.join("\n");
        assert_eq!(log.matches("alarmed: seen from underground").count(), 2, "{log}");
        assert_eq!(log.matches("rises (rule 30)").count(), 2, "{log}");
        assert!(!a.world.anims[0].character.as_ref().unwrap().unborn);
    }

    #[test]
    fn within_reach_it_follows_a_connor_who_holds_a_move_key() {
        // Rule 25 (Q133 b, the user's check): it stops only while he has no
        // forward / back key.
        use kq8_data::characters::Wander;
        let mut a = wanderer(Wander::None);
        {
            let ai = a.world.anims[0].character.as_mut().unwrap().ai.as_mut().unwrap();
            (ai.settings.alarm_radius, ai.settings.attack_dist, ai.settings.home_radius, ai.settings.chase_radius) = (4000.0, 1500.0, 3000.0, 6000.0);
        }
        a.world.connor_at = Some([0.0, -3500.0, 100.0]);
        a.world.connor_move_key = true;
        // (He doesn't really move here: held long enough, it would walk
        // into him, be stuck and run off, rule 7.)
        watch(&mut a, &[], 70);
        let log = a.world.character_log.join("
");
        assert!(log.contains("within reach but moving (forward / back key): it follows him at reach"), "{log}");
        assert!(!log.contains("stops, facing him"), "{log}");
        // He lets go of the key: it stops.
        a.world.connor_move_key = false;
        watch(&mut a, &[], 73);
        let log = a.world.character_log.join("\n");
        assert!(log.contains("he has no forward / back key: stops, facing him"), "{log}");
    }

    /// goblin22 for the fight (M5 5c): the wanderer with `attack` (1 s,
    /// cut short until 0.1875, `Do` at 0.5), alarm 4,000, reach 1,500,
    /// stats 15 / 2 / 2 / 0 / 3, attack delay 1 s.
    fn fighter() -> Active {
        use crate::door::Cue;
        let mut a = wanderer(kq8_data::characters::Wander::None);
        let cue = |phase: f32, action: &str| Cue { phase, action: action.into() };
        let o = &mut a.world.anims[0];
        o.player.movements.push(crate::named_anim::tests::movement("attack", 0.0, 1.0, false, false, vec![cue(0.0, "interruptable 1"), cue(0.1875, "interruptable 0"), cue(0.5, "do")]));
        let c = o.character.as_mut().unwrap();
        c.fight = Some(crate::fight::Fight::new(crate::fight::tests::settings([15, 2, 2, 0, 3], 1.0)));
        let ai = c.ai.as_mut().unwrap();
        (ai.settings.alarm_radius, ai.settings.attack_dist, ai.settings.home_radius, ai.settings.chase_radius) = (4000.0, 1500.0, 3000.0, 6000.0);
        a
    }

    #[test]
    fn a_character_attacks_at_once_then_every_swing_plus_its_delay() {
        // fighting.md rules 6–10 and the Do instead of rule 11b (M5 5c-3).
        let mut a = fighter();
        a.world.connor_at = Some([0.0, -3500.0, 100.0]);
        watch(&mut a, &[], 200);
        let log = a.world.fight_log.join("
");
        let starts: Vec<u64> = a.world.fight_log.iter().filter(|l| l.contains(": attack (rule 7,")).map(|l| l["update ".len()..].split(':').next().unwrap().parse().unwrap()).collect();
        assert!(log.contains("attack (rule 7, moving)") && log.contains("attack time 0.00 -> never"), "the first comes at once, from Move: {log}");
        assert!(log.contains("attack (rule 7, standing)"), "then standing (no key held): {log}");
        assert!(log.contains("Do (Attack at phase 0.5"), "{log}");
        assert!(log.contains("'Attack' ended: next attack due at") && log.contains("now + attackDelay 1 s"), "{log}");
        assert!(starts.len() >= 3, "{log}");
        // Between two swings: its length (1 s) + the delay (1 s), give or
        // take the update it starts in.
        for w in starts.windows(2).skip(1) {
            assert!((29..=32).contains(&(w[1] - w[0])), "{starts:?}
{log}");
        }
        // Rule 11b's Do instead: no stop while the swing from Move plays.
        let first = a.world.character_log.iter().position(|l| l.contains("attack (rule 7, moving)")).unwrap();
        let ended = a.world.character_log.iter().position(|l| l.contains("'Attack' ended")).unwrap();
        assert!(!a.world.character_log[first..ended].iter().any(|l| l.contains("stops, facing him")), "{log}");
    }

    #[test]
    fn its_swings_hurt_connor_until_he_dies() {
        // fighting.md rules 13, 16, 18, 29 (M5 5c-4): a zombie's 15.
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().fight.as_mut().unwrap().stats.0[1] = 15;
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_at = Some([0.0, -3500.0, 100.0]);
        watch(&mut a, &[], 600);
        let log = a.world.fight_log.join("
");
        assert!(log.contains("Do (Attack at phase 0.5333): hits Connor: distance") && log.contains("hand damage 15, Normal"), "{log}");
        assert!(log.contains("Connor: hit by gob, Normal: D 15, R "), "{log}");
        assert!(log.contains("first hit by a character: a one-off tip"), "{log}");
        assert!(log.contains("is below 0: his dying state, kind Normal"), "{log}");
        assert!(a.world.connor_stats.unwrap().stats.0[0] < 0);
        assert!(a.world.connor_dying);
        // While dying, hits are ignored (the requests are taken by the
        // viewer, so here they pile up: one ConnorDies only).
        assert_eq!(a.world.requests.iter().filter(|r| matches!(r, Request::ConnorDies { .. })).count(), 1);
        assert!(log.contains("ignored, he is dying (rule 29)"), "{log}");
    }

    #[test]
    fn a_swing_misses_when_connor_is_behind_it() {
        // Rule 13: the front half. Connor stands right behind it, within
        // reach; it can't turn (noTurn), so its swings miss.
        let mut a = fighter();
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.anims[0].character.as_mut().unwrap().no_turn = true;
        a.world.connor_at = Some([0.0, 1000.0, 100.0]);
        watch(&mut a, &[], 120);
        let log = a.world.fight_log.join("
");
        assert!(log.contains("gob missing Connor, too far away: miss, not in its front half: distance 1000, reach 1500"), "{log}");
        assert!(!log.contains("hits Connor"), "{log}");
        assert_eq!(a.world.connor_stats.unwrap().stats.0[0], 10);
    }

    #[test]
    fn connor_s_do_hits_his_target_misses_or_hits_dead_air() {
        // fighting.md rules 39-41, 16 (M5 5c-5).
        let mut a = fighter();
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_fight = Some(crate::fight::ConnorFight { ouch_sound: "conrow3.aud".into(), min_damage_factor: 0.75, max_damage_factor: 4.0, short_lists: vec![], long_lists: vec![], shoot_offsets: vec![], start_projectile: None, start_offset: [0.0; 3] });
        // The goblin at (0, 0, 100); Connor 2,000 south of it, facing it
        // (heading pi faces +y).
        let from = [0.0, -2000.0, 100.0];
        a.world.connor_do(from, std::f32::consts::PI, 2350.0, Some("gob"), false, "attack");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hits gob: distance 2000, reach 2350") && log.contains("hand damage 10 = 10, Normal"), "{log}");
        assert!(log.contains("gob: hit by Connor, Normal: D 10, R "), "{log}");
        let h = a.world.anims[0].character.as_ref().unwrap().fight.as_ref().unwrap().stats.0[0];
        assert!((3..=7).contains(&h), "15 - (8 to 12): {h}");
        // Facing away: a miss; no target: dead air.
        a.world.connor_do(from, 0.0, 2350.0, Some("gob"), false, "attack");
        a.world.connor_do(from, std::f32::consts::PI, 2350.0, None, false, "attack");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("Connor missing gob, too far away: miss, not in its front half"), "{log}");
        assert!(log.contains("no target: he hits dead air"), "{log}");
        // The big swing: x 0.75 to 4, cut.
        a.world.connor_do(from, std::f32::consts::PI, 2350.0, Some("gob"), true, "attack2");
        assert!(a.world.fight_log.join("\n").contains("(the big swing, 0.75 to 4, cut)"));
    }

    /// The goblin with a `react` (0.6 s, one-shot) added, health 500 so it
    /// never dies, Connor 1,400 south of it standing.
    fn hit_goblin() -> Active {
        let mut a = fighter();
        let o = &mut a.world.anims[0];
        let mut react = crate::named_anim::tests::movement("react", 0.0, 1.0, false, false, vec![]);
        react.play_time = 0.6;
        o.player.movements.push(react);
        o.character.as_mut().unwrap().fight.as_mut().unwrap().stats.0[0] = 500;
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_at = Some([0.0, -1400.0, 100.0]);
        a
    }

    #[test]
    fn a_hit_character_flinches_or_says_ouch_and_runs_after_connor() {
        // fighting.md rules 22-24, 26 (M5 5c-6).
        let mut a = hit_goblin();
        watch(&mut a, &[], 40);
        let from = [0.0, -1400.0, 100.0];
        let mut flinches = 0;
        let mut ouches = 0;
        for k in 0..40 {
            watch(&mut a, &[], 40 + k * 5);
            let last = a.world.fight_log.len();
            a.world.connor_do(from, std::f32::consts::PI, 2350.0, Some("gob"), false, "attack");
            let new = a.world.fight_log[last..].join("\n");
            flinches += new.matches("flinches: 'React'").count();
            ouches += new.matches("ouch sound sword.aud").count();
        }
        let log = a.world.fight_log.join("\n");
        assert!(flinches > 0 && ouches > 0, "{log}");
        assert!(log.contains("chase timer 5 s, hit count 1 (rule 22)"), "{log}");
        assert!(log.contains("the alarm with run (rule 26): gob (hit count 2, chase timer 10 s"), "{log}");
        assert!(log.contains("its last flinch ended") || log.contains("can't be cut short"), "a flinch refused at some point: {log}");
        assert!(log.contains("its flinch 'React' ended: no flinch for 2 s"), "{log}");
        // Never two flinches starting within 2 s of a flinch's end: with
        // hits every 5 updates (1/3 s) and a 0.6 s flinch, at most one in
        // about every 2.6 s.
        let starts: Vec<u64> = a.world.fight_log.iter().filter(|l| l.contains("flinches: 'React'")).map(|l| l["update ".len()..].split(':').next().unwrap().parse().unwrap()).collect();
        for w in starts.windows(2) {
            assert!(w[1] - w[0] >= 39, "{starts:?}");
        }
        assert!(a.world.anims[0].character.as_ref().unwrap().ai.as_ref().unwrap().hit_count >= 2, "the alarm with run: hit count 2 at least");
    }

    #[test]
    fn hit_unseen_with_no_team_it_runs_with_no_target() {
        // fighting.md rule 28 (M5 5c-6): it could not see him.
        let mut a = hit_goblin();
        a.world.connor_at = Some([0.0, -3000.0, 100.0]);
        {
            let ai = a.world.anims[0].character.as_mut().unwrap().ai.as_mut().unwrap();
            ai.settings.alarm_radius = 1500.0;
        }
        watch(&mut a, &[], 20);
        assert!(a.world.anims[0].character.as_ref().unwrap().ai.as_ref().unwrap().active, "active by its first sight test");
        a.world.anims[0].character.as_mut().unwrap().ai.as_mut().unwrap().sees = false;
        a.world.hit_character(0, "Connor", 10, crate::fight::HitKind::Normal);
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hit while not alarmed, no team: runs with no target, fighting.md rule 28"), "{log}");
        assert!(a.world.anims[0].character.as_ref().unwrap().ai.as_ref().unwrap().hunting);
        // It heads for the point 0.52 rad anticlockwise off the line to
        // him; within its alarm radius it is alarmed with no delay.
        watch(&mut a, &[], 120);
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("alarmed while hunting: no reaction delay (fighting.md rule 28)"), "{}", a.world.character_log.join("
"));
    }

    #[test]
    fn a_character_dies_then_gives_experience_a_corpse_and_goes() {
        // fighting.md rules 30, 32, 33 (M5 5c-7).
        let mut a = fighter();
        let o = &mut a.world.anims[0];
        o.player.movements.push(crate::named_anim::tests::movement("die", 0.0, 1.0, false, false, vec![]));
        let f = o.character.as_mut().unwrap().fight.as_mut().unwrap();
        f.settings.dead_dts = Some("g2dead.kqs".into());
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.connor_at = Some([0.0, -1400.0, 100.0]);
        watch(&mut a, &[], 40);
        a.world.hit_character(0, "Connor", 100, crate::fight::HitKind::Normal);
        let c = a.world.anims[0].character.as_ref().unwrap();
        assert_eq!((c.state, c.collision_type, c.collision_mask, c.gravity), (crate::character::State::Die, 0, 0, false));
        assert_eq!(c.fight.as_ref().unwrap().stats.0[0], 0);
        assert!(!a.world.active_list.iter().any(|n| n == "gob"));
        // It never changes state again; hits only take health.
        assert!(a.test_command("KQMonster::setState gob Wait", "t").is_some());
        assert_eq!(a.world.anims[0].character.as_ref().unwrap().state, crate::character::State::Die);
        a.world.hit_character(0, "Connor", 10, crate::fight::HitKind::Normal);
        assert!(a.world.fight_log.join("\n").contains("it is dying: nothing else (rule 21)"));
        // Experience only when the death movement ends (1 s).
        assert_eq!(a.world.connor_stats.unwrap().stats.0[4], 0);
        watch(&mut a, &[], 60);
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("its death ended (rule 32): Connor gets its experience 3"), "{log}");
        assert_eq!(a.world.connor_stats.unwrap().stats.0[4], 3);
        assert_eq!(a.world.corpses.len(), 1);
        assert_eq!((a.world.corpses[0].shape.as_str(), a.world.corpses[0].position), ("g2dead.kqs", a.world.anims[0].player.base.position));
        assert!(a.world.anims[0].gone);
        assert!(log.contains("first kill (type goblin2)"), "{log}");
        assert!(log.contains("no script: Die Normal not called"), "{log}");
    }

    #[test]
    fn the_console_kill_dies_as_from_a_hit_once() {
        // The viewer console's `kill` (ours): rule 30's death, no alarm.
        let mut a = fighter();
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        assert_eq!(a.world.console_kill("GOB"), Ok("gob".to_string()));
        let c = a.world.anims[0].character.as_ref().unwrap();
        assert_eq!(c.state, crate::character::State::Die);
        assert_eq!(c.fight.as_ref().unwrap().last_kind, Some(crate::fight::HitKind::Normal));
        assert!(a.world.console_kill("gob").unwrap_err().contains("already dying"));
        assert!(a.world.console_kill("nobody").unwrap_err().contains("no character"));
    }

    #[test]
    fn die_now_skips_the_movement_and_fire_leaves_no_corpse() {
        // Rules 30, 32: `setState <name> Die now`; a Fire kill burns away.
        let mut a = fighter();
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        a.world.anims[0].character.as_mut().unwrap().fight.as_mut().unwrap().last_kind = Some(crate::fight::HitKind::Fire);
        a.test_command("KQMonster::setState gob Die now", "t");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("Die now: no movement") && log.contains("no corpse (Fire: burnt away)"), "{log}");
        assert!(a.world.anims[0].gone && a.world.corpses.is_empty());
    }

    #[test]
    fn the_hand_weapon_slot_readies_and_puts_away() {
        // inventory.md rule 9, fighting.md rule 39 (M5 5c-5; putting it
        // away with a second click is our stand-in).
        let mut a = with_object("x\n", "x");
        assert_eq!(crate::connor_inventory::ready_hand_weapon(&mut a.world, "INVITEM_Fists").split(':').next(), Some("readies INVITEM_Fists"));
        assert_eq!(a.world.weapon_ready.as_deref(), Some("INVITEM_Fists"));
        assert!(crate::connor_inventory::ready_hand_weapon(&mut a.world, "INVITEM_Fists").starts_with("puts INVITEM_Fists away"));
        assert_eq!(a.world.weapon_ready, None);
    }

    #[test]
    fn weapon_requests_are_taken_standing_or_moving_and_scripts_ask_too() {
        // ranged-weapons.md rules 1-5, 9 (M5 5f-3).
        use crate::weapons::{Ask, Request, WeaponMode};
        let mut a = with_object("x\n", "x");
        let r = |ask: Ask| Request { ask, how: "t".into() };
        // Delete at the start: his fists (no hand weapon carried here).
        let ch = a.world.take_weapon_request(&r(Ask::Draw), Ok(()), false).unwrap();
        assert_eq!((ch.from, ch.to, ch.to_item.as_deref()), (WeaponMode::None, WeaponMode::Hand, Some("INVITEM_Fists")));
        assert_eq!((a.world.weapon_mode, a.world.weapon_ready.as_deref()), (WeaponMode::Hand, Some("INVITEM_Fists")));
        // Delete again: no real ranged weapon: nothing.
        assert!(a.world.take_weapon_request(&r(Ask::Draw), Ok(()), false).is_none());
        // Alt while jumping: dropped, not kept (rule 5).
        assert!(a.world.take_weapon_request(&r(Ask::Away), Err("mode Jump".into()), false).is_none());
        assert_eq!(a.world.weapon_mode, WeaponMode::Hand);
        assert!(a.world.fight_log.join("\n").contains("dropped, he is not standing or moving (mode Jump) (rule 5)"));
        // handsOff: ignored.
        assert!(a.world.take_weapon_request(&r(Ask::Away), Ok(()), true).is_none());
        // A script: setDoMode 4 asks, whatever his state.
        assert_eq!(a.test_command("KQConner::setDoMode 4", "t").as_deref(), Some("0"));
        let req = a.world.weapon_requests.remove(0);
        let ch = a.world.take_weapon_request(&req, Err("mode Special".into()), false).unwrap();
        assert!(ch.script && ch.to == WeaponMode::None && a.world.weapon_ready.is_none());
        assert_eq!(a.test_command("KQConner::getDoMode", "t").as_deref(), Some("4"));
    }

    #[test]
    fn go_home_and_the_radius_commands() {
        // Rules 23, 27, 29.
        let mut a = wanderer(kq8_data::characters::Wander::None);
        assert_eq!(a.test_command("KQMonster::setAlarmRadius gob 4500", "t").as_deref(), Some("0"));
        assert_eq!(a.test_command("KQMonster::setChaseRadius gob 100", "t").as_deref(), Some("0"));
        assert_eq!(a.test_command("KQMonster::setHomeRadius gob 6000", "t").as_deref(), Some("0"));
        let s = a.world.anims[0].character.as_ref().unwrap().ai.as_ref().unwrap().settings.clone();
        assert_eq!((s.alarm_radius, s.chase_radius, s.home_radius), (4500.0, 100.0, 6000.0));
        // goHome: never alarmed until goHome 0.
        a.world.connor_at = Some([0.0, -2000.0, 100.0]);
        assert_eq!(a.test_command("KQMonster::goHome gob", "t").as_deref(), Some("0"));
        watch(&mut a, &[], 40);
        assert!(!a.world.character_log.join("\n").contains("alarmed:"));
        assert_eq!(a.test_command("KQMonster::goHome gob 0", "t").as_deref(), Some("0"));
        watch(&mut a, &[], 46);
        assert!(a.world.character_log.join("\n").contains("alarmed: sees Connor 2000 away"), "{}", a.world.character_log.join("\n"));
    }

    #[test]
    fn patrol_and_random_points() {
        use kq8_data::characters::Wander;
        let kq = kq8_data::kq_text::parse(b"[Gob]\nclassType=KQMonster\nanimFile=g.anm\n");
        let mut s = kq8_data::characters::AiSettings::from_keys(&kq8_data::characters::CharacterKeys::from_section(&kq[0]).unwrap());
        s.wander = Wander::Patrol(vec![[1.0, 0.0, 0.0], [2.0, 0.0, 0.0], [3.0, 0.0, 0.0], [4.0, 0.0, 0.0]]);
        let mut ai = crate::character_ai::Ai::new(s, [0.0; 3], 1300.0, false);
        let (r, _) = ai.make_route([0.0; 3], &mut || 0.5);
        assert_eq!(r.iter().map(|p| p[0]).collect::<Vec<_>>(), [1.0, 2.0, 3.0, 4.0, 3.0, 2.0, 1.0]);
        ai.settings.wander = Wander::Random;
        ai.settings.alarm_radius = 1000.0;
        let mut k = 0.0;
        let (r, _) = ai.make_route([100.0, 200.0, 0.0], &mut || {
            k += 0.25;
            k % 1.0
        });
        assert_eq!(r.len(), 4);
        assert_eq!(r[0], [100.0, 200.0]);
        for p in &r[1..] {
            let d = ((p[0] - 100.0).powi(2) + (p[1] - 200.0).powi(2)).sqrt();
            assert!((2000.0..=3000.0).contains(&d), "{d}");
        }
        ai.has_birth = true;
        assert_eq!(ai.make_route([5.0, 6.0, 0.0], &mut || 0.5).0, [[5.0, 6.0]]);
    }

    #[test]
    fn get_loc_while_playing_feeds_set_home() {
        // objects.md "Script variables, getLoc and move" rule 1: zomtrap.cs
        // runs `getloc zograv3` then `KQMonster::setHome zograv3
        // $KQObject::locOnly`.
        let mut a = wanderer(kq8_data::characters::Wander::None);
        a.world.anims[0].player.base.rotation = kq8_data::objects::rotation([0.1, -0.2, 1.3]);
        assert_eq!(a.test_command("KQObject::getLoc gob", "t").as_deref(), Some("0"));
        let vars = std::mem::take(&mut a.world.vars_out);
        assert_eq!(vars[1], ("KQObject::locOnly".to_string(), "0 0 100".to_string()));
        let loc: Vec<f32> = vars[0].1.split(' ').map(|w| w.parse().unwrap()).collect();
        for (got, want) in loc[3..].iter().zip([0.1, -0.2, 1.3]) {
            assert!((got - want).abs() < 1e-5, "{loc:?}");
        }
        assert_eq!(a.test_command("getLoc nobody", "t").as_deref(), Some("1"));
    }

    #[test]
    fn check_distance_is_straight_3d_and_inclusive() {
        // anim-scripts.md rule 31; gob stands at (0, 0, 100).
        let mut a = wanderer(kq8_data::characters::Wander::None);
        a.world.connor_at = Some([300.0, 0.0, 500.0]);
        // 3D distance 500 (300 sideways, 400 up): equal counts as within.
        assert_eq!(a.test_command("checkDistance gob Connor 500", "t").as_deref(), Some("1"));
        assert_eq!(a.test_command("checkDistance Connor gob 499", "t").as_deref(), Some("0"));
        assert_eq!(a.test_command("checkDistance gob 0 0 5100 5000", "t").as_deref(), Some("1"));
        assert_eq!(a.test_command("checkDistance 0 0 0 0 0 5001 5000", "t").as_deref(), Some("0"));
        // Errors leave 0.
        assert_eq!(a.test_command("checkDistance nobody Connor 99999", "t").as_deref(), Some("0"));
        assert_eq!(a.test_command("checkDistance gob 99999", "t").as_deref(), Some("0"));
    }

    #[test]
    fn set_home_never_moves_the_character() {
        let mut a = wanderer(kq8_data::characters::Wander::None);
        assert_eq!(a.test_command("KQMonster::setHome gob 10 20 -1", "t").as_deref(), Some("0"));
        let c = a.world.anims[0].character.as_ref().unwrap();
        assert_eq!(c.ai.as_ref().unwrap().home, [10.0, 20.0, -1.0]);
        assert_eq!(a.world.anims[0].player.base.position, [0.0, 0.0, 100.0]);
    }

    #[test]
    fn characters_are_solid_for_connor_only_with_their_collider_on() {
        let mut a = with_object("x
", "x");
        let mut add = |name: &str, kind: u32, bounds: bool| {
            let mut player = crate::named_anim::tests::still_then_open(name);
            player.movements[0].name = "stop".into();
            let mut c = crate::character::Character::new(false, 0x20, 0.0, false, false, false, 1);
            c.collision_type = kind;
            c.bounds = bounds.then_some(([-350.0, -800.0, 0.0], [350.0, 800.0, 1300.0]));
            a.add_anims([AnimObject { player, script: None, gone: false, character: Some(Box::new(c)), translucency: None }]);
        };
        add("gob", 0x42, true);
        add("wiz", 0xC021, true);
        add("ghost", 0x10, true);
        add("nobox", 0x42, true);
        a.world.anims[3].character.as_mut().unwrap().bounds = None;
        for n in ["gob", "wiz", "ghost", "nobox"] {
            a.world.characters_solid.insert(n.into());
        }
        a.world.characters_in_range.insert("gob".into());
        assert_eq!(a.test_command("move gob 0 5000 0 0 0 0 0", "t").as_deref(), Some("0"));
        a.update();
        a.world.update_character_colliders();
        let names: Vec<&str> = a.world.character_colliders.iter().map(|c| c.name.as_str()).collect();
        // Only 0x10 (traps, which he ignores); no box: never solid.
        assert_eq!(names, ["gob", "wiz"]);
        let gob = &a.world.character_colliders[0];
        assert_eq!((gob.position, gob.radius, gob.height), ([5000.0, 0.0, 0.0], 800.0, 1600.0));
        // Far (collider off) or deleted: no longer solid, logged.
        a.world.characters_solid.remove("gob");
        a.world.anims[1].gone = true;
        a.world.update_character_colliders();
        assert!(a.world.character_colliders.is_empty());
        let log = a.world.character_log.join("
");
        assert!(log.contains("gob: solid for Connor: cylinder radius 800") && log.contains("gob: not solid for Connor: its collider is off"), "{log}");
        assert!(log.contains("wiz: not solid for Connor: deleted") && log.contains("not a creature"), "{log}");
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
        // DESIGN L-3: the level-up sound, once.
        assert_eq!(a.world.sounds.iter().filter(|s| s.name == crate::stats::LEVEL_UP_SOUND).count(), 1, "{log}");
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

    /// `prscbolt.kq` with `scbolt.kqs`'s `BOUNDS.KQ` box (data).
    fn scbolt() -> kq8_data::characters::ProjectileKeys {
        let mut p = kq8_data::characters::ProjectileKeys::from_section(&kq8_data::kq_text::parse(b"[Arrow]\nclassType=KQProjectile\nshapeName=scbolt.kqs\nvelocity=8000\nmaxDistance=18000\nmeshName=arrow\n")[0]).unwrap();
        p.bounds = Some(([-50.0, 0.0, 0.0], [50.0, 470.0, 50.0]));
        p
    }

    /// One bolt update, with the flat ground at 100.
    fn fly(a: &mut Active) {
        a.world.bolts_step(&FlatGround, &[], &[]);
    }

    /// Daventry's `prhammer.kq` with `hammer.kqs`'s `BOUNDS.KQ` box (data).
    fn prhammer() -> kq8_data::characters::ProjectileKeys {
        let mut p = kq8_data::characters::ProjectileKeys::from_section(&kq8_data::kq_text::parse(b"[Hammer]\nclassType=KQHammer\nshapeName=hammer.kqs\nvelocity=24000\nmaxDistance=15000\nmeshName=hammer\nmodifier=30\nminDamage=0\nmaxDamage=300\n")[0]).unwrap();
        p.bounds = Some(([-90.0, 370.0, 0.0], [80.0, 470.0, 70.0]));
        p
    }

    #[test]
    fn the_hammer_s_damage_is_capped_then_raised() {
        // azriel-hammer.md rule 8 and "Do not reproduce" (the order kept).
        assert_eq!(crate::fight::hammer_damage(30, 10, 0, 300), 40);
        assert_eq!(crate::fight::hammer_damage(30, 400, 0, 300), 300);
        assert_eq!(crate::fight::hammer_damage(0, 10, 0, 0), 0, "a world without the keys: 0");
        assert_eq!(crate::fight::hammer_damage(30, 10, 50, 20), 50, "a minimum above the maximum wins");
    }

    #[test]
    fn the_hammer_hits_one_character_flies_back_and_is_caught() {
        // azriel-hammer.md rules 7-9 (M6 6b-2): Connor 5,000 south of gob,
        // a second character behind gob on the same line.
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().collision_type = 0x42;
        a.world.characters_solid.insert("gob".into());
        let mut second = a.world.anims[0].clone();
        second.player.name = "gob2".into();
        second.player.base.position = [0.0, 3000.0, 100.0];
        a.world.anims.push(second);
        a.world.characters_solid.insert("gob2".into());
        a.world.connor_at = Some([0.0, -5000.0, 100.0]);
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, -4400.0, 1300.0], [0.0, 5000.0, 1300.0], 10, "test");
        assert_eq!(a.world.bolts[0].name, "hammer1");
        for _ in 0..12 {
            fly(&mut a);
        }
        assert!(a.world.bolts.is_empty(), "back in his hands");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("touches gob at") && log.contains("a hit of 40 (modifier 30 + ranged damage 10 = 40, capped at 300, at least 0: 40), Projectile"), "{log}");
        assert!(!log.contains("touches gob2"), "one thing only: {log}");
        // Out: 1,600 per update; the touch in update 3 (cylinder at y −1,035
        // minus the ball): put back at its place after update 2.
        assert!(log.contains("hammer1: turns back (its touch) at (") && log.contains(", -1200, 1300), its last place before; 3200 flown out, range back 1600 (3200 - one step 1600)"), "{log}");
        assert!(log.contains("hammer1 update 4 (back): touches Connor at") || log.contains("hammer1: the way back ends (more than its range 1600)"), "{log}");
        assert_eq!(a.world.hammers_back.len(), 1);
        assert_eq!(a.world.hammers_back[0].0, "Connor");
        assert_eq!(a.world.anims[1].character.as_ref().unwrap().fight.as_ref().unwrap().stats.0[0], 15, "gob2 untouched");
    }

    #[test]
    fn the_hammer_turns_back_at_its_range_and_at_the_ground() {
        // Rules 7, 9: nothing in the way: out past 15,000, back 15,000 − 1,600.
        let mut a = fighter();
        a.world.connor_at = Some([0.0, 1_000_000.0, 100.0]);
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, 0.0, 1300.0], [0.0, -5000.0, 1300.0], 10, "test");
        let mut updates = 0;
        while !a.world.bolts.is_empty() && updates < 40 {
            fly(&mut a);
            updates += 1;
        }
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hammer1: turns back (more than its range 15000) at (0, -14400, 1300), its last place before; 14400 flown out, range back 12800"), "{log}");
        assert!(log.contains("hammer1: the way back ends (more than its range 12800): removed; Connor is told it came back, 19 updates after the throw"), "{log}");
        // 12,800 back is reached at update 18; it ends on passing it.
        assert_eq!(updates, 19, "never reaches him: caught anyway");
        // At the ground near him: back at once.
        a.world.hammers_back.clear();
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, 0.0, 600.0], [0.0, -700.0, 0.0], 10, "test");
        fly(&mut a);
        fly(&mut a);
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hammer2: turns back (at or below the floor grid (100)) at (0, 0, 600)"), "{log}");
        assert_eq!(a.world.hammers_back.len(), 1, "{log}");
        // Connor on its line: ignored on the way out, he stops it on the way back.
        a.world.hammers_back.clear();
        a.world.connor_at = Some([0.0, -8000.0, 100.0]);
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, 0.0, 1300.0], [0.0, -5000.0, 1300.0], 10, "test");
        for _ in 0..20 {
            fly(&mut a);
        }
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hammer3 update 5 (out): at (") && !log.contains("hammer3 update 5 (out): touches"), "{log}");
        assert!(log.contains("(back): touches Connor at") && log.contains("hammer3: the way back ends (touching Connor)"), "{log}");
        assert_eq!(a.world.hammers_back.len(), 1);
    }

    #[test]
    fn the_hammer_on_the_lever_runs_its_script_and_spares_immune_ones() {
        // Rule 8: `tmblrcyl` (type 0x461) runs lever1's script; an object
        // with bit 0x4000 takes 0 and gets no ProjImmune.
        let mut a = fighter();
        a.world.connor_at = None;
        let obj = |name: &str, t: u32| crate::collide::BoxCollider {
            name: name.into(),
            min: [-300.0, -300.0, 0.0],
            max: [300.0, 300.0, 2000.0],
            placement: Placement::at([0.0, -4000.0, 100.0]),
            kind: crate::collide::ColliderKind { collision_type: t, ..Default::default() },
        };
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, 0.0, 1300.0], [0.0, -5000.0, 1300.0], 10, "test");
        for _ in 0..3 {
            a.world.bolts_step(&FlatGround, &[], &[obj("tmblrcyl", 0x461)]);
        }
        assert_eq!(a.world.hammer_calls, vec!["lever1".to_string()]);
        assert!(a.world.fight_log.join("\n").contains("object tmblrcyl at") && a.world.fight_log.join("\n").contains("no damage; lever1's script runs with lever1 ConnorAction"));
        a.world.hammer_calls.clear();
        a.world.bolts.clear();
        a.world.launch_bolt("Connor", "prhammer.kq", &prhammer(), [0.0, 0.0, 1300.0], [0.0, -5000.0, 1300.0], 10, "test");
        for _ in 0..3 {
            a.world.bolts_step(&FlatGround, &[], &[obj("statue", 0x4001)]);
        }
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("object statue at") && log.contains("a hit of 0, no ProjImmune"), "{log}");
        assert!(a.world.immune_calls.is_empty() && a.world.hammer_calls.is_empty());
    }

    #[test]
    fn a_bolt_flies_straight_and_hits_the_first_character() {
        // ranged-weapons.md rules 26-30 (M5 5f-1).
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().collision_type = 0x42;
        a.world.characters_solid.insert("gob".into());
        a.world.connor_at = Some([0.0, 50_000.0, 100.0]);
        // From 5,000 south of gob (at 0, 0, 100) at 1,000 up, aimed level at it.
        a.world.launch_bolt("shooter", "prscbolt.kq", &scbolt(), [0.0, -5000.0, 1000.0], [0.0, 0.0, 1000.0], 5, "test");
        let b = a.world.bolts[0].clone();
        assert!((b.heading - std::f32::consts::PI).abs() < 1e-4 && b.pitch.abs() < 1e-6, "{b:?}");
        let balls = b.balls.clone().unwrap();
        assert_eq!((balls.radius, balls.heights.clone()), (235.0, vec![235.0]), "one ball of 235 (Q140 a)");
        fly(&mut a);
        let y = a.world.bolts[0].position[1];
        assert!((y + 5000.0 - 8000.0 / 15.0).abs() < 0.1, "533 per update: {y}");
        for _ in 0..10 {
            fly(&mut a);
        }
        assert!(a.world.bolts.is_empty());
        let log = a.world.fight_log.join("\n");
        // Gob's cylinder: radius 800 (+235), so the ball touches it at y −1,035.
        assert!(log.contains("bolt1 update 8: touches gob at (") && log.contains(", -1035, 1000), 3965 travelled") && log.contains("a hit of 5, Projectile"), "{log}");
        assert!(log.contains("gob: hit by shooter, Projectile"), "{log}");
        // Pitched: aimed 1,000 down over 5,000, asin(1000 / 5099).
        a.world.launch_bolt("shooter", "prscbolt.kq", &scbolt(), [0.0, -5000.0, 2000.0], [0.0, 0.0, 1000.0], 5, "test");
        let p = a.world.bolts[0].pitch.to_degrees();
        assert!((p + 11.31).abs() < 0.01, "{p}");
    }

    #[test]
    fn a_bolt_ends_at_the_floor_grid_or_its_range_and_spares_its_shooter() {
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().collision_type = 0x42;
        a.world.characters_solid.insert("gob".into());
        a.world.connor_at = None;
        // Shot by gob itself from inside its own cylinder: not hit (rule 29);
        // level at 1,000 up, away from anything: gone after 34 updates (18,133 > 18,000).
        a.world.launch_bolt("gob", "prscbolt.kq", &scbolt(), [0.0, 0.0, 1000.0], [0.0, -5000.0, 1000.0], 5, "test");
        for _ in 0..33 {
            fly(&mut a);
        }
        assert_eq!(a.world.bolts.len(), 1);
        fly(&mut a);
        assert!(a.world.bolts.is_empty());
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("bolt1 update 34: at (0, -18133, 1000), 18133 travelled; more than its range 18000: removed"), "{log}");
        // Aimed down at the ground: removed below the floor grid (100).
        a.world.launch_bolt("shooter", "prscbolt.kq", &scbolt(), [0.0, -20_000.0, 600.0], [0.0, -19_000.0, 0.0], 5, "test");
        fly(&mut a);
        fly(&mut a);
        assert!(a.world.bolts.is_empty());
        assert!(a.world.fight_log.join("\n").contains("at or below the floor grid (100): removed, no damage"));
    }

    #[test]
    fn a_character_s_bolt_starts_at_its_arrow_part_found_once() {
        // ranged-weapons.md rule 24 (M5 5f-2): spbattck's `arrow` at its
        // `do` frame, about (−55, −1,367, 954); gob faces −y (heading 0).
        let mut a = fighter();
        {
            let o = &mut a.world.anims[0];
            let attack = o.player.movements.iter_mut().find(|m| m.name == "attack").unwrap();
            attack.frame_parts = (0..16).map(|k| if k == 8 { vec![("arrow".to_string(), [-55.0, -1367.0, 954.0])] } else { vec![] }).collect();
            o.player.start("attack");
            // One update past its `do` cue (0.5, frame 8): the frame at the
            // cue counts (Q140 c), not frame 9 shown now.
            o.player.phase = 0.6;
            let c = o.character.as_mut().unwrap();
            let f = c.fight.as_mut().unwrap();
            f.settings.ranged_weapon = Some("INVITEM_SmCrossbow".into());
            f.settings.projectile = Some("Prscbolt.kq".into());
            c.ai.as_mut().unwrap().alarmed = true;
        }
        a.world.projectiles.insert("prscbolt.kq".into(), scbolt());
        a.world.connor_at = Some([0.0, -8000.0, 100.0]);
        a.world.character_do(0);
        let b = a.world.bolts[0].clone();
        assert_eq!(b.position.map(f32::round), [-55.0, -1367.0, 1054.0], "gob at (0, 0, 100) + the part");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("start offset (-55, -1367, 954): its part 'arrow' in frame 8 of 'attack' (kept from now on)"), "{log}");
        // Kept: a later shot from another frame starts at the same offset.
        a.world.anims[0].player.phase = 0.0;
        a.world.character_do(0);
        assert!(a.world.fight_log.join("\n").contains("start offset (-55, -1367, 954): kept from an earlier shot"));
    }

    #[test]
    fn connor_s_bolt_starts_at_the_crossbow_part_and_flies_to_the_aim_point() {
        // ranged-weapons.md rules 17, 21-25 (M5 5f-4, 5f-5).
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().collision_type = 0x42;
        a.world.characters_solid.insert("gob".into());
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 5, 0, 0]), level: 1 });
        a.world.projectiles.insert("prscbolt.kq".into(), scbolt());
        a.world.connor_projectile = Some("Prscbolt.kq".into());
        a.world.connor_shoot_offset = [-315.0, -327.0, 1027.0];
        // Connor 5,000 south of gob, facing it (heading pi: +y).
        let r = kq8_data::objects::rotation([0.0, 0.0, std::f32::consts::PI]);
        a.world.connor_shoot([0.0, -5000.0, 100.0], r, Some(([-231.0, -832.0, 1067.0], "test part".into())), [0.0, 0.0, 1167.0], "attack");
        let b = a.world.bolts[0].clone();
        assert_eq!(b.position.map(f32::round), [231.0, -4168.0, 1167.0], "832 ahead, 231 to his right, 1,067 up");
        assert_eq!((b.damage, b.shooter.as_str()), (5, "Connor"));
        assert_eq!(a.world.connor_shoot_offset, [-231.0, -832.0, 1067.0], "kept as his shoot offset");
        for _ in 0..10 {
            fly(&mut a);
        }
        assert!(a.world.fight_log.join("\n").contains("gob: hit by Connor, Projectile"));
        // No part in the frame: his shoot offset; NoRanged: no bolt.
        a.world.connor_shoot([0.0, -5000.0, 100.0], r, None, [0.0, 0.0, 1167.0], "attack");
        assert!(a.world.fight_log.join("\n").contains("start offset (-231, -832, 1067): no such part in the frame: his shoot offset"));
        a.world.connor_projectile = None;
        a.world.connor_shoot([0.0, -5000.0, 100.0], r, None, [0.0, 0.0, 1167.0], "attack");
        assert!(a.world.fight_log.join("\n").contains("he has no bolt file (NoRanged): no shot"));
    }

    #[test]
    fn a_bolt_on_an_immune_character_does_no_damage_and_tells_its_script() {
        // Rule 30: type bit 0x4000 (the Wizard and the Knight, 0xC021).
        let mut a = fighter();
        a.world.anims[0].character.as_mut().unwrap().collision_type = 0xC021;
        a.world.characters_solid.insert("gob".into());
        a.world.connor_at = None;
        a.world.launch_bolt("shooter", "prscbolt.kq", &scbolt(), [0.0, -3000.0, 1000.0], [0.0, 0.0, 1000.0], 5, "test");
        for _ in 0..6 {
            fly(&mut a);
        }
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("has the immune bit 0x4000: a hit of 0, its script gets ProjImmune"), "{log}");
        assert_eq!(a.world.anims[0].character.as_ref().unwrap().fight.as_ref().unwrap().stats.0[0], 15);
        assert_eq!(a.world.immune_calls, vec!["gob".to_string()]);
    }

    #[test]
    fn connor_s_hit_on_a_scripted_object_kills_it_below_0() {
        // M5 5c fix 5: doors.md rules 19-20, anim-scripts.md rule 19 (all 0
        // without a stats= line); a barrel's type 0x141 (not clickable).
        let script = "if test $2 == Die\n set B::died $1 $3\nendif\n";
        let mut barrel = Clickable::plain("barrel.07", "KQObject", [-300.0, -310.0, 0.0], [300.0, 310.0, 680.0], Placement::at([0.0, 0.0, 100.0]), 0x141);
        barrel.script = Some("barelup.cs".into());
        let mut a = Active::new(Vars::default(), vec![], vec![barrel], ["Connor".to_string()], [("barelup.cs".to_string(), script.as_bytes().to_vec())]);
        a.start();
        a.set_connor_stats(ConnorStats { stats: Stats([10, 10, 0, 0, 0]), level: 1 });
        assert_eq!(a.pick_attack([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]).map(|(n, _)| n).as_deref(), Some("barrel.07"));
        assert!(a.pick([0.0, -5000.0, 400.0], [0.0, 1.0, 0.0]).is_none(), "the plain click doesn't pick it");
        a.world.connor_do([0.0, -1500.0, 100.0], std::f32::consts::PI, 2350.0, Some("barrel.07"), false, "attack");
        let log = a.world.fight_log.join("\n");
        assert!(log.contains("hits object barrel.07") && log.contains("health 0 -> -") && log.contains("Die Normal"), "{log}");
        a.update();
        assert_eq!(a.scripts.vars.get("B::died"), Some("barrel.07 Normal"));
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
    fn what_an_end_loop_script_sends_with_delay_0_happens_before_drawing() {
        // anim-scripts.md rule 8 (Q119): the Uglybeast's `pivot` → `stand`.
        let script = "if test $2 == ConnorAction\n sendEvent 0 $1 KQAnimPlayEvent open\nelse if test $2 == EndLoop\n sendEvent 0 $1 KQAnimPlayEvent stop\nendif\nendif\n";
        let mut a = with_object(script, "x");
        a.click("basket1", [0.0, -1000.0, 0.0], 2350.0, None);
        let mut n = 0;
        while a.world.objects[0].player.movement_name() != "open" && n < 5 {
            a.update();
            n += 1;
        }
        // Until `open` has played: then, in the update it ends, `stop` is
        // already the movement (never a frame of `open`'s end left over).
        while a.world.objects[0].player.movement_name() == "open" && n < 40 {
            a.update();
            n += 1;
        }
        assert_eq!(a.world.objects[0].player.movement_name(), "stop");
        let log = a.take_log().join("\n");
        assert!(log.contains("handled before drawing"), "{log}");
        let ended = log.lines().find(|l| l.contains("movement 'open' finished")).map(|l| l.split(':').next().unwrap().to_string());
        let stopped = log.lines().find(|l| l.contains("KQAnimPlayEvent stop to")).map(|l| l.split(':').next().unwrap().to_string());
        assert_eq!(ended, stopped, "the same update: {log}");
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
        assert_eq!(a.scripts.vars.get("B::ended"), None, "open still playing");
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
        assert!(log.contains("KQMonster::speak Connor to Connor: 0 109 14 33 1…1: to itself, no turn; ConnorTalk: queued"), "{log}");
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
        // 0.4 s = 6 updates: the line ends in update 6; the group hears it
        // through the queue, in that same update (Q119: delay 0 doesn't wait).
        for _ in 0..6 {
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
