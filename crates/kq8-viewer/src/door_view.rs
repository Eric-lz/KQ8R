//! Doors in the ground view (M3 step 3a, docs/DESIGN.md; rules:
//! docs/behaviour/doors.md).
//!
//! - The doors, their scripts and the event queue live in
//!   `kq8_core::active` and run at 15 updates per second, before Connor's
//!   update, so a door's box is where it is drawn when he moves.
//! - A left click sends a ray from the camera through the mouse and picks
//!   the nearest clickable box (rule 10); the click is handled at the
//!   next update with Connor's position then (reach, rule 11).
//! - Each door is drawn turned by its animation (rule 24), showing the
//!   frame of its current movement.
//!
//! Test options: `--click NAME,N` clicks object NAME before update N + 1;
//! `--item TYPE` gives Connor an item of that type, called `TestItem`, and
//! puts it in his hand (M4 4c-4). Everything is logged to
//! `kq8-artifacts/logs/doors.log`.
//!
//! Traps (M3 step 3d-2, docs/behaviour/traps.md) live in the same
//! `Active` (shared variables, scripts and events). [`trap_step`] runs
//! after Connor's update: if he moved, every trap checks him. Their lines
//! go to `kq8-artifacts/logs/traps.log`.
//!
//! Items (M3 step 3e-3, docs/behaviour/items.md): the item objects the
//! start-up scripts made join the same `Active`, which also handles
//! clicking them (`item_view.rs` draws them). Their lines go to
//! `kq8-artifacts/logs/items.log`.
//!
//! Chests (M3 step 3e-4, docs/behaviour/chests.md) are loaded and drawn
//! like doors (their `open` movement, frame by frame) and live in the
//! same `Active`; a click runs the chest's own steps.

use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use kq8_core::active::{Active, ActiveChest, ActiveDoor, ActiveObject, Item, Pick, Request};
use kq8_core::connor::Mode;
use kq8_core::named_anim::NamedAnimPlayer;
use kq8_core::chest::Chest;
use kq8_core::trap::{Trap, Visitor};
use kq8_core::collide::{BoxCollider, Placement};
use kq8_core::door::{turn_degrees, Cue, Door, DoorMovement};

use crate::connor_view::ConnorSim;
use crate::game_camera::GameCam;
use crate::load::Objects;
use crate::log::Log;
use crate::terrain_view::{bevy_transform_from, Buildings, ObjectColliders, ViewBlockerSet};
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::{Shared, UNITS_PER_METRE};

/// Connor's `attackDist` when `Connor.kq` gives none: the built-in
/// default for characters (doors.md rule 11).
const DEFAULT_ATTACK_DIST: f32 = 1250.0;

/// One door's entities: the body (turned) and its drawn parts.
struct DrawnDoor {
    body: Entity,
    /// Per movement, its number of frames.
    frames: Vec<usize>,
    /// Its drawing (Animation memory step 4).
    slots: Option<crate::anim_gpu::Slots>,
}

/// One door animation built: its ball, its number of frames, its
/// materials by index and its model (step 4).
type BuiltDoorAnim = (crate::shadow_view::ModelBall, usize, Vec<(i32, Handle<ObjectMaterial>)>, Option<crate::load::FrameShape>);

#[derive(Resource)]
pub struct DoorSim {
    pub active: Active,
    /// Copies a script loaded while playing, to draw (M5 5d-3): file,
    /// name, place.
    pub spawns: Vec<(String, String, kq8_core::collide::Placement)>,
    drawn: Vec<DrawnDoor>,
    /// Each door's entry in `ObjectColliders`, if it collides with Connor.
    collider: Vec<Option<usize>>,
    /// The same for the chests (3e-4).
    chest_drawn: Vec<DrawnDoor>,
    chest_collider: Vec<Option<usize>>,
    /// The same for the scripted objects (3e-5).
    object_drawn: Vec<DrawnDoor>,
    object_collider: Vec<Option<usize>>,
    attack_dist: f32,
    /// Objects clicked since the last update, with how (mouse, test).
    pending: Vec<(String, String)>,
    /// Clicks in the world since the last update, picked or not, for
    /// `talking.md` rule 16 (M4 4a-5).
    world_clicks: Vec<String>,
    /// Clicks that swing (a hand weapon ready, M5 5c-5).
    swing_clicks: Vec<SwingClick>,
    shot_clicks: Vec<ShotClick>,
    /// Connor's state at each click whose turn is waiting (M4 4c-5,
    /// `clicking.md` rule 35): whether it allows the turn, its name.
    click_states: std::collections::VecDeque<(bool, &'static str)>,
    pub log: Log,
    /// `traps.log`; `None` if it could not be made.
    trap_log: Option<Log>,
    /// Where the traps last checked Connor.
    connor_checked_at: Option<[f32; 3]>,
    /// `items.log`; `None` if it could not be made.
    item_log: Option<Log>,
    /// `rope.log` (M6 6a).
    rope_log: Option<Log>,
    /// World clicks made with the rope in hand (`ropeandhook.md` rule 10;
    /// M6 6a-3), and the climb asked for (taken by `rope_view`).
    rope_clicks: Vec<String>,
    pub rope_asked: Option<String>,
    /// `talk.log` (M4 4a-2); `None` if it could not be made.
    talk_log: Option<Log>,
    /// `conversation.log` (M4 4e-3); `None` if it could not be made.
    conversation_log: Option<Log>,
}

impl DoorSim {
    /// Writes the item lines gathered since the last call; pickups and
    /// items made also to the main log. Returns the lines, so the door
    /// lines (which hold a copy of each) don't put them on the terminal
    /// a second time.
    /// The rope state put Connor somewhere at once (`ropeandhook.md` rules
    /// 15, 16, 18, 23; M6 6a-3; Q147 e, measured by A): as a script's
    /// `move` (traps.md rule 5): every trap he is in runs its stay test
    /// now, so the end he left gets `Off` at once; no trap is entered now,
    /// the other end's `On` comes at his next move (the sweep starts from
    /// here). Returns the trap calls.
    pub fn connor_placed_by_rope(&mut self, name: &str, position: [f32; 3], balls: &kq8_core::collide::BallStack, shared: &Shared) -> Vec<String> {
        let (min, max) = kq8_core::connor_mover::CONNOR_BOX;
        let v = Visitor { name, collision_type: kq8_core::collide::CONNOR_TYPE, from: position, position, min, max, balls };
        let calls = self.active.connor_placed(&v);
        self.connor_checked_at = Some(position);
        self.flush_traps(shared);
        calls
    }

    /// Writes the rope lines gathered since the last call (M6 6a) to
    /// `rope.log`; status changes also to the main log.
    pub fn flush_rope(&mut self, shared: &Shared) {
        let lines = std::mem::take(&mut self.active.world.rope_log);
        let mut main = shared.log.lock().unwrap();
        for line in lines {
            if line.contains("status ") || line.contains(" state: ") {
                main.summary(format!("rope: {line}"));
            }
            if let Some(log) = &mut self.rope_log {
                log.detail(line);
            }
        }
    }

    fn flush_items(&mut self, shared: &Shared) -> Vec<String> {
        let lines = self.active.take_item_log();
        if lines.is_empty() {
            return lines;
        }
        let mut main = shared.log.lock().unwrap();
        for line in &lines {
            if line.contains("picked up") || line.contains(": made ") {
                main.summary(format!("items: {line}"));
            }
            if let Some(log) = &mut self.item_log {
                log.detail(line);
            }
        }
        lines
    }

    /// Writes the talk lines gathered since the last call (M4 4a-2);
    /// lines starting or finishing also to the main log.
    pub fn flush_talk(&mut self, shared: &Shared) {
        let lines = self.active.world.take_talk_log();
        // The update these lines belong to (the counter has moved on).
        let updates = self.active.world.updates.saturating_sub(1);
        let Some(log) = &mut self.talk_log else { return };
        for line in lines {
            if line.contains(" starts: ") || line.contains(": TalkComplete ") {
                shared.log.lock().unwrap().summary(format!("talk: update {updates} {line}"));
            }
            log.detail(format!("update {updates} {line}"));
        }
    }

    /// Writes the conversation lines gathered since the last call (M4
    /// 4e-3), also to the main log.
    pub fn flush_conversation(&mut self, shared: &Shared) {
        let lines = std::mem::take(&mut self.active.world.conversation_log);
        let Some(log) = &mut self.conversation_log else { return };
        for line in lines {
            shared.log.lock().unwrap().summary(format!("conversation: {line}"));
            log.detail(line);
        }
    }

    /// Writes the trap lines gathered since the last call; `On`/`Off` also
    /// to the main log.
    fn flush_traps(&mut self, shared: &Shared) {
        let lines = self.active.take_trap_log();
        let Some(log) = &mut self.trap_log else { return };
        if lines.is_empty() {
            return;
        }
        let mut main = shared.log.lock().unwrap();
        for line in lines {
            if line.contains(": trap ") && (line.contains(": On ") || line.contains(": Off ") || line.contains(": Cue ")) {
                main.summary(format!("traps: {line}"));
            }
            log.detail(line);
        }
    }
}

/// `--click NAME,N` (each may be given more than once).
#[derive(Resource)]
pub struct ClickAt(pub Vec<(String, u64)>);

/// Objects a script's `loadKQ` brought in this update (M3 3k-4), for
/// [`apply_loads`].
#[derive(Resource, Default)]
pub struct LoadQueue(pub Vec<String>);

/// Brings in the objects a script loaded (M3 3k-4, Q90 stand-in): their
/// drawn parts are shown (no longer only with H), an animated one is
/// played by the world's rules, a sound object starts. Runs after
/// `trap_step`.
pub fn apply_loads(
    mut commands: Commands,
    mut queue: ResMut<LoadQueue>,
    sim: Option<ResMut<DoorSim>>,
    anim: Option<ResMut<crate::anim_view::AnimSim>>,
    sound: Option<ResMut<crate::sound_view::SoundSim>>,
    mut parts: Query<(Entity, &crate::terrain_view::ObjectName, &mut Visibility)>,
    shared: Res<Shared>,
) {
    if queue.0.is_empty() {
        return;
    }
    let (Some(mut sim), mut anim, mut sound) = (sim, anim, sound) else { return };
    for name in std::mem::take(&mut queue.0) {
        let mut shown = 0;
        for (e, _, mut v) in parts.iter_mut().filter(|(_, n, _)| n.0.eq_ignore_ascii_case(&name)) {
            *v = Visibility::Inherited;
            commands.entity(e).remove::<crate::terrain_view::ShownWithH>();
            shown += 1;
        }
        let animated = anim.as_mut().is_some_and(|a| a.bring_in(&name, &mut sim.active));
        // Events sent to it while it came in (the Wizard's `Map1`).
        sim.active.loaded(&name);
        // A character brought in gets its draw-distance test now (M4
        // 4d-3), so it updates while near or far (characters.md rule 11).
        if let Some((body, test)) = anim.as_mut().and_then(|a| a.take_test(&name)) {
            commands.entity(body).insert(test);
        }
        let sounding = sound.as_mut().is_some_and(|s| s.wake(&name));
        let line = format!(
            "update {}: loadKQ: {name} came in: {shown} drawn part(s) shown{}{} (its click box, if it has one, is on from now on; a solid box for Connor: not done)",
            sim.active.world.updates,
            if animated { ", animation played by the world's rules" } else { "" },
            if sounding { ", sound object on" } else { "" }
        );
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
}

/// `--special ITEMS,N` (M3 3i-7 test).
#[derive(Resource)]
pub struct SpecialAt(pub Option<(String, u64)>);

/// `--command "WORDS",N` (M4 4d-5 test).
#[derive(Resource, Default)]
pub struct CommandAt(pub Vec<(String, u64)>);

/// `--right N` (M4 4e-5 test): the right button counts as held from
/// update N on.
#[derive(Resource, Default)]
pub struct RightAt(pub Option<u64>);

/// `--aim NAME,N` (each may be given more than once).
#[derive(Resource)]
pub struct AimAt(pub Vec<(String, u64)>);

/// `--swing NAME,N[,big]` (M5 5c-5 test; each may be given more than once).
#[derive(Resource)]
pub struct SwingAt(pub Vec<(String, u64, bool)>);

/// A click in the world with a hand weapon ready (`fighting.md` rule 39;
/// M5 5c-5): Connor swings toward it.
/// A shot click with the ranged weapon ready (`ranged-weapons.md` rules
/// 13–16; M5 5f-5): the aim point before the steepness limit, what the
/// click line met.
pub struct ShotClick {
    aim: [f32; 3],
    what: String,
    how: String,
}

pub struct SwingClick {
    /// What the click line met (`ranged-weapons.md` rule 32: types with
    /// bit 0x40 or 0x20, not a building) and its position.
    picked: Option<(String, [f32; 3])>,
    /// Where he turns (x, y).
    point: [f32; 2],
    /// Shift held: the big swing.
    big: bool,
    how: String,
}

/// `--item TYPE`; `--combat-level 1|2|3` (M5 5c-10: **stand-in** for the
/// menu that sets it, `fighting.md` open question 3).
#[derive(Resource)]
pub struct HeldItem(pub Option<String>, pub Option<i64>);

/// The AI test switch (ours, not in the original): `--no-ai` switches
/// it on at the start (taken once the world is loaded), F4 flips it while
/// playing. With it on no character is alarmed, so none chases or
/// attacks Connor.
#[derive(Resource)]
pub struct AiOff(pub bool);

/// The on-screen line of the AI test switch.
#[derive(Component)]
pub struct AiLabel;

fn ai_label(off: bool) -> (String, Color) {
    if off {
        ("AI: off, creatures ignore Connor (F4)".into(), Color::srgb(1.0, 0.8, 0.2))
    } else {
        ("AI: on (F4)".into(), Color::WHITE)
    }
}

/// The on-screen line under the F8 line.
pub fn setup_ai_label(mut commands: Commands, start: Res<AiOff>) {
    let (text, color) = ai_label(start.0);
    commands.spawn((
        Text::new(text),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(color),
        Node { position_type: PositionType::Absolute, top: Val::Px(96.0), left: Val::Px(10.0), ..default() },
        AiLabel,
    ));
}

/// `--no-ai` at the start, then F4; the on-screen line follows core's
/// switch (also `--key f4,N`).
pub fn toggle_ai(keys: Res<ButtonInput<KeyCode>>, mut start: ResMut<AiOff>, sim: Option<ResMut<DoorSim>>, mut label: Query<(&mut Text, &mut TextColor), With<AiLabel>>) {
    let Some(mut sim) = sim else { return };
    if std::mem::take(&mut start.0) {
        flip_ai(&mut sim, "--no-ai");
    }
    if keys.just_pressed(KeyCode::F4) {
        flip_ai(&mut sim, "key F4");
    }
    let (text, color) = ai_label(sim.active.world.ai_off);
    for (mut t, mut c) in &mut label {
        if t.0 != text {
            t.0 = text.clone();
            c.0 = color;
        }
    }
}

/// God mode, the test switch (ours, not in the original): `--god` at the
/// start, F3 while playing. Hits on Connor do nothing and a long fall
/// doesn't kill him.
#[derive(Resource)]
pub struct GodStart(pub bool);

/// The on-screen line of god mode.
#[derive(Component)]
pub struct GodLabel;

fn god_label(on: bool) -> (String, Color) {
    if on {
        ("God mode: on, Connor can't be hurt (F3)".into(), Color::srgb(1.0, 0.8, 0.2))
    } else {
        ("God mode: off (F3)".into(), Color::WHITE)
    }
}

/// The on-screen line under the AI line.
pub fn setup_god_label(mut commands: Commands, start: Res<GodStart>) {
    let (text, color) = god_label(start.0);
    commands.spawn((
        Text::new(text),
        TextFont { font_size: FontSize::Px(16.0), ..default() },
        TextColor(color),
        Node { position_type: PositionType::Absolute, top: Val::Px(118.0), left: Val::Px(10.0), ..default() },
        GodLabel,
    ));
}

/// `--god` at the start, then F3; Connor's mover and the on-screen line
/// follow core's switch every frame (also `--key f3,N`).
pub fn toggle_god(
    keys: Res<ButtonInput<KeyCode>>,
    mut start: ResMut<GodStart>,
    sim: Option<ResMut<DoorSim>>,
    connor: Option<ResMut<crate::connor_view::ConnorSim>>,
    mut label: Query<(&mut Text, &mut TextColor), With<GodLabel>>,
) {
    let Some(mut sim) = sim else { return };
    if std::mem::take(&mut start.0) {
        flip_god(&mut sim, "--god");
    }
    if keys.just_pressed(KeyCode::F3) {
        flip_god(&mut sim, "key F3");
    }
    let on = sim.active.world.god_mode;
    if let Some(mut c) = connor {
        if c.mover.no_fall_death != on {
            c.mover.no_fall_death = on;
        }
    }
    let (text, color) = god_label(on);
    for (mut t, mut c) in &mut label {
        if t.0 != text {
            t.0 = text.clone();
            c.0 = color;
        }
    }
}

/// Flips god mode (F3, `--god`, `--key f3,N`).
pub fn flip_god(sim: &mut DoorSim, how: &str) {
    let w = &mut sim.active.world;
    w.god_mode = !w.god_mode;
    let state = if w.god_mode { "on: hits on Connor do nothing, a long fall doesn't kill him" } else { "off" };
    let line = format!("update {}: {how}: god mode is {state} (test switch, ours)", w.updates);
    sim.log.summary(line);
}

/// Flips the AI test switch (F4, `--no-ai`, `--key f4,N`).
pub fn flip_ai(sim: &mut DoorSim, how: &str) {
    let w = &mut sim.active.world;
    w.ai_off = !w.ai_off;
    let state = if w.ai_off { "off: no character is alarmed, the alarmed ones give up" } else { "on again" };
    let line = format!("update {}: {how}: the AI is {state} (test switch, ours)", w.updates);
    sim.log.summary(line);
}

/// A body entity at `placement` with, per movement (index into `built`),
/// one hidden entity per frame. `haze`: the object's `b_hazeSolid` and
/// `hazeMultiplier` (object-lighting.md rules 21, 22). `test`: its draw
/// distance (M1 6h, rules 34-37).
#[allow(clippy::too_many_arguments)]
fn spawn_body(
    commands: &mut Commands,
    placement: &Placement,
    animations: &[usize],
    built: &[BuiltDoorAnim],
    materials: &mut Assets<ObjectMaterial>,
    haze: (bool, f32),
    test: crate::draw_distance::DistanceTest,
    lights: &[std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>],
    dyn_off: bool,
    gpu: &crate::anim_gpu::AnimGpu,
    name: &str,
) -> DrawnDoor {
    let transform = bevy_transform_from(placement.rotation, placement.position);
    // Its name for `dither` only (Q144; not `ObjectName`, which the plain
    // objects' `move` and hiding use).
    let body = commands.spawn((transform, Visibility::default(), test, crate::potion_view::DitherName(name.to_string()))).id();
    // Step 4c: drawn the new way too.
    let own: Vec<((usize, i32), Handle<ObjectMaterial>)> = animations
        .iter()
        .flat_map(|a| built[*a].2.iter().map(move |(m, h)| ((*a, *m), h.clone())))
        .map(|(k, h)| (k, crate::object_light::for_object(materials, &h, 1.0, haze.0, haze.1)))
        .collect();
    let movements = animations.iter().map(|a| (*a, built[*a].3.clone(), built[*a].0)).collect();
    let slots = crate::anim_gpu::Slots::spawn(commands, body, gpu, crate::anim_gpu::List::Doors, movements, own, materials, &mut Default::default());
    // The lights it carries (M1 7c), by movement and frame.
    if let Some(c) = crate::scene_lights::CarriedLights::new(animations.iter().map(|a| lights[*a].clone()).collect(), transform, dyn_off) {
        commands.entity(body).insert(c);
    }
    let frames = animations.iter().map(|a| built[*a].1).collect();
    DrawnDoor { body, frames, slots }
}

/// Builds the doors from the loaded objects: their entities, their game
/// state, and their colliders (added to `colliders`).
#[allow(clippy::too_many_arguments)]
pub fn spawn_doors(
    commands: &mut Commands,
    objects: &mut Objects,
    colliders: &mut Vec<BoxCollider>,
    (item, combat): (Option<String>, Option<i64>),
    (anims, own_players): (Vec<kq8_core::active::AnimObject>, Vec<kq8_core::named_anim::NamedAnimPlayer>),
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    balls: &crate::draw_distance::ShapeBalls,
    main_log: &mut Log,
    gpu: &crate::anim_gpu::AnimGpu,
) {
    let log = match Log::create("doors") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("doors: cannot create doors.log: {e}; doors not started"));
            return;
        }
    };
    // Each animation's length and frame steps (the door's turn, rule 24).
    let timing: Vec<(f32, Vec<kq8_data::shape::Transform>)> =
        objects.door_animations.iter().map(|a| (a.duration, a.frame_transforms.clone())).collect();
    // Each animation's lights per frame (M1 7c, rule 26b).
    let anim_lights: Vec<std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>> =
        objects.door_animations.iter().map(|a| std::sync::Arc::new(a.frame_lights.clone())).collect();
    // Each animation once: per frame, (mesh, material) per material group.
    // With its `.kqs` ball (shadows on buildings, M4 4d-7b-5b).
    let built: Vec<BuiltDoorAnim> = std::mem::take(&mut objects.door_animations)
        .into_iter()
        .map(|anim| {
            let ball = crate::shadow_view::ModelBall::of(anim.ball);
            let source = anim.source.clone();
            let looks: Vec<(i32, Handle<ObjectMaterial>, crate::load::Shading)> =
                anim.looks.into_iter().map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)), l)).collect();
            let by_index: Vec<(i32, Handle<ObjectMaterial>)> = looks.iter().map(|(m, h, _)| (*m, h.clone())).collect();
            (ball, anim.frame_count, by_index, source)
        })
        .collect();
    let attack_dist = objects.attack_dist.unwrap_or_else(|| {
        main_log.summary(format!("doors: Connor.kq gives no attackDist; using the default {DEFAULT_ATTACK_DIST}"));
        DEFAULT_ATTACK_DIST
    });
    let mut active_doors = Vec::new();
    let mut drawn = Vec::new();
    let mut collider = Vec::new();
    let (mut active_chests, mut chest_drawn, mut chest_collider) = (Vec::new(), Vec::new(), Vec::new());
    let (mut active_objects, mut object_drawn, mut object_collider) = (Vec::new(), Vec::new(), Vec::new());
    // objects.md "Dithered objects" rule 8 (Q144): chests and animated
    // objects apply their `dithered=` at every movement start, so they are
    // drawn so from the start (other classes: Q165).
    let file_dithered: Vec<(String, u8)> = objects
        .placed
        .iter()
        .map(|(p, ..)| p)
        .chain(objects.doors.iter().map(|d| &d.placement))
        .filter(|p| p.dithered != 0 && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQChest") || c.eq_ignore_ascii_case("KQAnimObject")))
        .map(|p| (p.name.clone(), p.dithered))
        .collect();
    let sources = std::mem::take(&mut objects.scripts);
    for d in std::mem::take(&mut objects.doors) {
        let p = &d.placement;
        let base = Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position };
        let movements: Vec<DoorMovement> = d
            .movements
            .iter()
            .map(|(name, l, a)| DoorMovement {
                name: name.clone(),
                start: l.start,
                speed: l.speed,
                repeat: l.repeat,
                // anim-scripts.md rule 4: the 6th field.
                cut_short: l.unknown != 0,
                play_time: timing[*a].0,
                frame_steps: timing[*a].1.clone(),
                // Doors never use the frame rule's wrap step (characters only).
                first_in_file: true,
                frame_parts: Vec::new(),
                cues: l.cues.iter().map(|(phase, action)| Cue { phase: *phase, action: action.clone() }).collect(),
                transitions: l.transitions.clone(),
            })
            .collect();
        let animations: Vec<usize> = d.movements.iter().map(|(_, _, a)| *a).collect();
        let names = (d.anim_names.0.as_deref(), d.anim_names.1.as_deref());
        if d.object {
            // A scripted object (3e-5).
            let o = ActiveObject {
                // anim-scripts.md rule 3 corrected (M4 4d-8).
                player: NamedAnimPlayer::new(&p.name, movements, d.current, base).with_stop_names(names.0, names.1),
                script: d.script.clone(),
                bounds: d.bounds,
                collision_type: d.collider_type,
                collision_mask: d.collider_mask,
                accepts_inventory: d.accepts_inventory.clone(),
                accept_all_items: d.accept_all_items,
                too_far_away_action: d.too_far_away_action,
                do_not_point_at: d.do_not_point_at,
                hidden: !p.drawn(),
                gone: false,
                delete_when_done: false,
            };
            main_log.summary(format!(
                "objects: [{}] scripted at ({:.0}, {:.0}, {:.0}), type {:#x}, script {:?}, box {}, shows '{}', movements {:?}",
                p.name,
                p.position[0],
                p.position[1],
                p.position[2],
                o.collision_type,
                o.script,
                d.bounds.map_or("NONE".to_string(), |(a, b)| format!("{a:?}..{b:?}")),
                o.player.movement_name(),
                o.player.movements.iter().map(|m| m.name.as_str()).collect::<Vec<_>>()
            ));
            object_collider.push(if o.collides_with_connor() {
                colliders.push(o.collider().expect("an object that collides has a box"));
                Some(colliders.len() - 1)
            } else {
                None
            });
            object_drawn.push(spawn_body(commands, &o.player.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting, gpu, &p.name));
            active_objects.push(o);
            continue;
        }
        if let Some(settings) = d.chest.clone() {
            // A chest (3e-4).
            let mut chest = Chest::new(&p.name, settings, movements, d.current, base);
            chest.player = chest.player.with_names_at_load(names.0, names.1);
            let open_turn = chest.player.movements.iter().find(|m| m.name.eq_ignore_ascii_case("open")).map(|m| turn_degrees(&m.turn_at_frame(m.frames() - 1)));
            let c = ActiveChest {
                bounds: d.bounds,
                collision_type: d.collider_type,
                collision_mask: d.collider_mask,
                // objects.md "Hidden objects".
                hidden: !p.drawn(),
                do_not_point_at: d.do_not_point_at,
                script: d.script.clone(),
                chest,
                gone: false,
            };
            main_log.summary(format!(
                "chests: [{}] at ({:.0}, {:.0}, {:.0}), type {:#x}, hidden {}, script {:?}, box {}, holds {:?} x{} at {:?}, Take {:?}, Empty {:?}; 'open' turns {open_turn:?} deg by its last frame",
                p.name,
                p.position[0],
                p.position[1],
                p.position[2],
                c.collision_type,
                c.hidden,
                c.script,
                d.bounds.map_or("NONE".to_string(), |(a, b)| format!("{a:?}..{b:?}")),
                c.chest.settings.item,
                c.chest.settings.quantity,
                c.chest.settings.item_position,
                c.chest.settings.take.map(|r| r.0),
                c.chest.settings.empty.map(|r| r.0),
            ));
            chest_collider.push(if c.collides_with_connor() {
                colliders.push(c.collider().expect("a chest that collides has a box"));
                Some(colliders.len() - 1)
            } else {
                None
            });
            chest_drawn.push(spawn_body(commands, &c.chest.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting, gpu, &p.name));
            active_chests.push(c);
            continue;
        }
        let mut door = Door::new(&p.name, movements, d.current, base);
        door.player = door.player.with_names_at_load(names.0, names.1);
        let DrawnDoor { body, frames, slots } =
            spawn_body(commands, &door.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting, gpu, &p.name);
        let active = ActiveDoor {
            door,
            script: d.script.clone(),
            bounds: d.bounds,
            collision_type: d.collider_type,
            collision_mask: d.collider_mask,
            accepts_inventory: d.accepts_inventory.clone(),
            accept_all_items: d.accept_all_items,
            too_far_away_action: d.too_far_away_action,
            do_not_point_at: d.do_not_point_at,
            gone: false,
        };
        // The door's script, read through the block counter (DESIGN 3a-1).
        let balance = d.script.as_ref().map(|name| {
            match sources.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
                Some((_, bytes)) => {
                    let text: String = bytes.iter().map(|&b| b as char).collect();
                    let (ifs, endifs, problems) = kq8_core::script::block_balance(&text);
                    format!("{name}: {} lines, {ifs} if / {endifs} endif{}", text.lines().count(), if problems.is_empty() { ", balanced".to_string() } else { format!(", PROBLEMS {problems:?}") })
                }
                None => format!("{name}: NOT FOUND in the world's scripts"),
            }
        });
        let open = active.door.player.movements.iter().find(|m| m.name.eq_ignore_ascii_case("open"));
        let open_note = open.map_or("no 'open' movement".to_string(), |m| {
            let last = m.frames() - 1;
            let t = m.turn_at_frame(last);
            format!("'open' {} frames over {:.3} s, turns {:.2} deg by its last frame", m.frames(), m.play_time, turn_degrees(&t))
        });
        main_log.summary(format!(
            "doors: [{}] at ({:.0}, {:.0}, {:.0}) dirZ {:.2}, type {:#x}, script {}, box {}, starts in '{}' (state closed); {open_note}; movements: {}",
            p.name,
            p.position[0],
            p.position[1],
            p.position[2],
            p.angles[2],
            d.collider_type,
            balance.unwrap_or_else(|| "none".into()),
            d.bounds.map_or("NONE".to_string(), |(a, b)| format!("{a:?}..{b:?}")),
            active.door.movement_name(),
            active
                .door
                .player.movements
                .iter()
                .map(|m| format!(
                    "{} (start {} speed {} cues {:?})",
                    m.name,
                    m.start,
                    m.speed,
                    m.cues.iter().map(|c| format!("{} {}", c.phase, c.action)).collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        collider.push(if active.collides_with_connor() {
            colliders.push(active.collider().expect("a door that collides has a box"));
            Some(colliders.len() - 1)
        } else {
            main_log.summary(format!("doors: [{}] does not collide with Connor (no box, or type and mask)", p.name));
            None
        });
        drawn.push(DrawnDoor { body, frames, slots });
        active_doors.push(active);
    }
    let test_item = item;
    let all = drawn.iter().chain(&object_drawn).chain(&chest_drawn);
    let (bodies, new_way, slots) = all.fold((0, 0, 0), |(b, n, s), d| (b + 1, n + usize::from(d.slots.is_some()), s + d.slots.as_ref().map_or(0, |x| x.len())));
    main_log.summary(format!("doors, chests and scripted objects drawn from the compact data (Animation memory step 4): {new_way} of {bodies}, {slots} part entities"));
    main_log.summary(format!(
        "doors: {} doors; Connor's reach 2 x attackDist {attack_dist} = {}; {} other clickable objects; {} scripts; test --item: {:?}; log in {}",
        active_doors.len(),
        2.0 * attack_dist,
        objects.clickables.len(),
        sources.len(),
        test_item,
        log.path().display()
    ));
    let mut active = Active::new(
        std::mem::take(&mut objects.vars),
        active_doors,
        std::mem::take(&mut objects.clickables),
        std::mem::take(&mut objects.known).into_iter().chain(["Connor".to_string()]),
        sources,
    );
    // Traps (3d-2): loaded where the start-up scripts left them.
    let mut trap_log = match Log::create("traps") {
        Ok(l) => Some(l),
        Err(e) => {
            main_log.summary(format!("traps: cannot create traps.log: {e}; traps still run, not logged"));
            None
        }
    };
    // A trap a script loads while playing is switched off until then
    // (`objects.md` rule 6; `ax.cs`'s `henchtp.kq`).
    let traps: Vec<Trap> = std::mem::take(&mut objects.traps)
        .into_iter()
        .map(|s| {
            let off = objects.dormant_traps.contains(&s.name.to_ascii_lowercase());
            Trap { gone: off, ..Trap::new(s) }
        })
        .collect();
    let sticky = traps.iter().filter(|t| t.shrunk).count();
    main_log.summary(format!(
        "traps: {} traps ({sticky} sticky, start shrunk); log in {}",
        traps.len(),
        trap_log.as_ref().map_or("(none)".into(), |l| l.path().display().to_string())
    ));
    if let Some(log) = &mut trap_log {
        log.detail("traps at the start: name, box in use (world), full box, mask, trapType, innerPercent, script");
        for t in &traps {
            let ((a, b), (fa, fb)) = (t.current_box(), t.full_box());
            log.detail(format!(
                "trap {} box ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0}) full ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0}) mask {:#x} trapType {} innerPercent {:?} script {:?}",
                t.settings.name, a[0], a[1], a[2], b[0], b[1], b[2], fa[0], fa[1], fa[2], fb[0], fb[1], fb[2],
                t.settings.mask, t.settings.trap_type, t.settings.inner_percent, t.settings.script
            ));
        }
    }
    active.add_traps(traps);
    main_log.summary(format!("chests: {} chests", active_chests.len()));
    active.add_chests(active_chests);
    main_log.summary(format!("objects: {} scripted clickable objects", active_objects.len()));
    active.add_objects(active_objects);
    // fighting.md rule 3 (M5 5c-10): the combat level, before the
    // characters enter the world.
    if let Some(level) = combat {
        let line = active.test_command(&format!("setCombatLevel {level}"), "test --combat-level");
        main_log.summary(format!("fight: --combat-level {level}: {line:?} (factor {})", active.world.combat_level));
    }
    // The other animated objects (3i-4), drawn by anim_view.
    active.add_anims(anims);
    // Copies prepared for a script's loadKQ (3k-4).
    active.set_dormant(std::mem::take(&mut objects.dormant));
    // M5 5d-3, 5d-4: the templates for new copies: still ones, and
    // animated ones with their player as built (at the start).
    let templates: Vec<(String, kq8_core::active::Template)> = objects
        .templates
        .iter()
        .filter_map(|t| {
            let p = &objects.placed[t.placed].0;
            let placement = kq8_core::collide::Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position };
            let kind = if t.animated {
                let player = own_players.iter().find(|o| o.name.eq_ignore_ascii_case(&t.name))?;
                kq8_core::active::TemplateKind::Anim(Box::new(player.clone()))
            } else {
                kq8_core::active::TemplateKind::Still { bounds: t.bounds, collision_type: t.collider_type }
            };
            Some((t.file.clone(), kq8_core::active::Template { name: t.name.clone(), script: t.script.clone(), placement, kind }))
        })
        .collect();
    let animated = templates.iter().filter(|(_, t)| matches!(t.kind, kq8_core::active::TemplateKind::Anim(_))).count();
    main_log.summary(format!("templates: {} for loadKQ while playing ({animated} animated; M5 5d-3, 5d-4)", templates.len()));
    active.world.set_templates(templates);
    // Stats (3i-6, anim-scripts.md rule 19): doors without a line have
    // 10,000 health (doors.md rule 18); Connor from Connor.kq.
    for d in &active.world.doors.iter().map(|d| d.door.name.clone()).collect::<Vec<_>>() {
        active.set_stats(d, kq8_core::stats::Stats([kq8_core::stats::DOOR_HEALTH, 0, 0, 0, 0]));
    }
    for (name, line) in &objects.object_stats {
        let words: Vec<&str> = line.split_whitespace().collect();
        let (s, problems) = kq8_core::stats::Stats::parse(&words);
        for p in problems {
            main_log.summary(format!("stats: [{name}] stats={line}: {p}"));
        }
        active.set_stats(name, s);
    }
    // fighting.md (M5 5c-4): what Connor.kq gives the fight.
    if let Some(f) = &objects.connor_fight {
        main_log.summary(format!(
            "fight: Connor's ouch sound {}, big swing x {} to {}, {} hand weapon lists ({})",
            f.ouch_sound,
            f.min_damage_factor,
            f.max_damage_factor,
            f.short_lists.len(),
            f.short_lists.iter().map(|(w, a)| format!("{w} {a}")).collect::<Vec<_>>().join(", ")
        ));
    }
    // potions.md rule 31 (M5 5g-7): the objects Clarity reveals.
    active.world.reveal_objects = std::mem::take(&mut objects.reveal_objects);
    // M6 6e-6c: the teleporters' portal files in GAME/Kq, which the
    // teleport scripts load while playing (magic-map.md rule 39).
    let files = std::mem::take(&mut objects.portal_files);
    main_log.summary(format!("portal files in GAME/Kq: {:?} (magic-map.md rule 39)", files.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()));
    active.world.set_portal_files(files);
    // M6 6e-6: the portals, for the Portal state.
    for p in std::mem::take(&mut objects.portals) {
        main_log.summary(format!("portal [{}]: to {:?}, enterAnim {:?}, exitAnim {:?}, noLoc {:?}, script {:?} (world-change.md rule 1)", p.name, p.world, p.enter_anim, p.exit_anim, p.no_loc, p.exit_script));
        active.world.portals.insert(p.name.to_ascii_lowercase(), p);
    }
    main_log.summary(format!("clarity: objects with groupsAlways bit 2048: {:?} (potions.md rule 31)", active.world.reveal_objects));
    match &objects.connor_stats {
        Some((line, level)) => {
            let words: Vec<&str> = line.split_whitespace().collect();
            let (s, _) = kq8_core::stats::Stats::parse(&words);
            let level = level.as_deref().and_then(|l| l.trim().parse::<i64>().ok()).unwrap_or(1);
            main_log.summary(format!("stats: Connor stats={line} level={level}; {} objects with a stats= line", objects.object_stats.len()));
            active.set_connor_stats(kq8_core::stats::ConnorStats { stats: s, level });
        }
        None => main_log.summary("stats: Connor.kq gives no stats= line; Connor's stats unknown"),
    }
    // Buildings as click targets (3g-2, clicking.md rules 3-4, 10).
    let buildings = std::mem::take(&mut objects.building_clicks);
    let not_clickable: Vec<String> =
        buildings.iter().filter(|b| b.collision_type & kq8_core::click::CLICKABLE == 0).map(|b| format!("{} ({:#x})", b.name, b.collision_type)).collect();
    let scripted: Vec<String> = buildings.iter().filter_map(|b| Some(format!("{} ({})", b.name, b.script.as_ref()?))).collect();
    main_log.summary(format!(
        "clicks: {} of {} buildings take clicks (not: {not_clickable:?}); with a script: {scripted:?}; the ground is not tested (clicking.md rule 6)",
        buildings.len() - not_clickable.len(),
        buildings.len()
    ));
    active.add_buildings(buildings);
    // Items (3e-3): those the start-up scripts made, and the ground for
    // z -1 / -2 of items made while playing.
    let mut item_log = match Log::create("items") {
        Ok(l) => Some(l),
        Err(e) => {
            main_log.summary(format!("items: cannot create items.log: {e}; items still work, not logged"));
            None
        }
    };
    active.world.items = std::mem::take(&mut objects.items);
    // M4 4c-2: Connor's start inventory (inventory.md rule 4), named after
    // its sections.
    let (start, problems) = std::mem::take(&mut objects.connor_inventory);
    let mut start_lines = problems.iter().map(|p| format!("Connor's start inventory: {p}")).collect::<Vec<_>>();
    for it in start {
        let line = match active.world.items.add_named(&it.name, &it.kind, it.quantity, it.user_id, kq8_core::item::Place::Carried("Connor".into())) {
            Ok(_) => format!("Connor's start inventory: {}", active.world.items.describe(&it.name)),
            Err(e) => format!("Connor's start inventory: [{}] {}: {e}; left out", it.name, it.kind),
        };
        start_lines.push(line);
    }
    // `--item TYPE` (4c-4): a real item, carried and in his hand.
    if let Some(kind) = &test_item {
        let line = match active.world.items.add_named("TestItem", kind, 1, -1, kq8_core::item::Place::Carried("Connor".into())) {
            Ok(_) => {
                active.world.hand = Some("TestItem".into());
                format!("test --item: {}; in Connor's hand", active.world.items.describe("TestItem"))
            }
            Err(e) => format!("test --item {kind}: {e}; nothing in hand"),
        };
        main_log.summary(&line);
        start_lines.push(line);
    }
    for o in &active.world.items.objects {
        active.world.known.insert(o.name.to_ascii_lowercase());
    }
    // fighting.md (M5 5c-4), ranged-weapons.md rules 17, 21: Connor's
    // fight, after the items the start-up scripts gave him (Swamp's
    // `constart.cs` gives Azriel's Hammer; M6 6b-1), so a ranged weapon he
    // carries gives his bolt file and shoot offset.
    active.world.projectiles = std::mem::take(&mut objects.projectiles);
    active.world.set_connor_fight(objects.connor_fight.clone());
    for (name, mode) in file_dithered {
        active.world.dithered.insert(name.to_ascii_lowercase(), mode);
        main_log.summary(format!("{name}: dithered={mode} from its file (objects.md \"Dithered objects\" rule 8)"));
    }
    // DESIGN "Connor's levels checked" (L-1): what `conStart.cs` gives
    // him (items put on, his level, a stats event), in script order.
    let start = std::mem::take(&mut objects.connor_start);
    for line in kq8_core::connor_start::run(&mut active.world, &start) {
        main_log.summary(format!("connor start: {line}"));
    }
    if let Some(c) = active.world.connor_stats {
        main_log.summary(format!("connor start: Connor {:?} level {} after the start-up scripts (health / hand / ranged / armour / experience)", c.stats.0, c.level));
    }
    // character-movement.md rule 20 (M5 5a-4).
    if let Some(n) = objects.max_monsters {
        active.world.max_monsters = n;
    }
    start_lines.push(format!("KQWorld::MaxMonsters {} ({}; character-movement.md rule 20)", active.world.max_monsters, if objects.max_monsters.is_some() { "world.cs" } else { "not set: 6, ours" }));
    if let Some(ground) = objects.ground.take() {
        use kq8_data::terrain::Level;
        active.world.heights = Some(Box::new(move |x, y, floor| ground.ground_at(if floor { Level::Floor } else { Level::Surface }, x, y).map(|g| g.height)));
    }
    if let Some(log) = &mut item_log {
        for line in std::mem::take(&mut objects.item_lines) {
            log.detail(line);
        }
        for line in &start_lines {
            log.detail(line);
        }
        log.detail("items at the start:");
        let items = &active.world.items;
        for o in &items.objects {
            let click = items.click_box(o).map_or("NO click box".to_string(), |(a, b)| format!("click box {a:?}..{b:?}"));
            log.detail(format!("  {}; {click}", items.describe(&o.name)));
        }
    }
    // Remarks and speech (M4 4a-2): the lines, the monsters, Options.cs.
    let talk_log = match Log::create("talk") {
        Ok(mut l) => {
            l.detail("talk.log: speak / clearSpeak / stateChangeNotify, lines starting and ending, mouth changes, TalkComplete (docs/behaviour/talking.md); t = game time in seconds");
            Some(l)
        }
        Err(e) => {
            main_log.summary(format!("talk: cannot create talk.log: {e}; speech still runs, not logged"));
            None
        }
    };
    let conversation_log = match Log::create("conversation") {
        Ok(mut l) => {
            l.detail("conversation.log: KQConner::conversation, Connor's talk state (case, approach, stay test, the end), the partner's Talk state, TalkExit (docs/behaviour/conversation.md)");
            Some(l)
        }
        Err(e) => {
            main_log.summary(format!("talk: cannot create conversation.log: {e}; conversations still run, not logged"));
            None
        }
    };
    if let Some((talk, sets)) = objects.talk.take() {
        active.world.talk = talk;
        for (k, v) in sets {
            active.scripts.vars.set(&k, v);
        }
    }
    active.world.monsters = std::mem::take(&mut objects.monsters);
    active.world.dormant_monsters = std::mem::take(&mut objects.dormant_monsters);
    let monsters: Vec<String> = active.world.monsters.iter().map(|m| format!("{} ({})", m.name, m.script.as_deref().unwrap_or("no script"))).collect();
    main_log.summary(format!("talk: {} KQMonsters: {}; captions {:?}", monsters.len(), monsters.join(", "), active.scripts.vars.get("Options::Captions")));
    // M4 4e-3: the conversation info objects (conversation.md rule 3).
    active.world.conv_infos = std::mem::take(&mut objects.conv_infos);
    main_log.summary(format!("talk: {} conversation info object(s): {:?}", active.world.conv_infos.len(), active.world.conv_infos.iter().map(|i| i.name.as_str()).collect::<Vec<_>>()));
    // Rule 22: each door's script gets Init as the door enters the world;
    // then each trap's (traps.md rule 13).
    active.start();
    let mut sim = DoorSim { active, spawns: Vec::new(), drawn, collider, attack_dist, pending: Vec::new(), world_clicks: Vec::new(), swing_clicks: Vec::new(), shot_clicks: Vec::new(), click_states: Default::default(), log, trap_log, connor_checked_at: None, item_log, talk_log, conversation_log, chest_drawn, chest_collider, object_drawn, object_collider, rope_log: Log::create("rope").ok(), rope_clicks: Vec::new(), rope_asked: None };
    for line in sim.active.take_log() {
        sim.log.detail(line);
    }
    if let Some(log) = &mut sim.trap_log {
        for line in sim.active.take_trap_log() {
            log.detail(line);
        }
    }
    if let Some(log) = &mut sim.item_log {
        for line in sim.active.take_item_log() {
            log.detail(line);
        }
    }
    commands.insert_resource(sim);
}

/// Names of deleted objects whose entities are still to be hidden (3f).
#[derive(Resource, Default)]
pub struct Deleted(pub Vec<String>);

/// A collider that no longer touches anything (3f: `deleteObject`): size
/// 0, far below the world.
/// No ground at all (a world without terrain).
struct NoGround;

impl kq8_core::connor_mover::GroundLookup for NoGround {
    fn ground(&self, _: f32, _: f32) -> Option<(f32, [f32; 3])> {
        None
    }
}

pub fn switch_off(c: &mut BoxCollider) {
    c.min = [0.0; 3];
    c.max = [0.0; 3];
    c.placement = Placement::at([0.0, 0.0, -1.0e7]);
}

/// After Connor's update (M3 step 3d-2): if he moved, every trap checks
/// him (traps.md rules 6-9) and runs its script. Then what the scripts
/// asked of Connor and the camera this update is applied (3d-3), before
/// the camera's update; and deleted objects lose their collision (3f).
#[allow(clippy::too_many_arguments)]
pub fn trap_step(
    sim: Option<ResMut<DoorSim>>,
    connor: Option<ResMut<ConnorSim>>,
    camera: Option<ResMut<GameCam>>,
    colliders: Option<ResMut<ObjectColliders>>,
    view: Option<ResMut<ViewBlockerSet>>,
    anim: Option<ResMut<crate::anim_view::AnimSim>>,
    mut deleted: ResMut<Deleted>,
    mut rules: ResMut<crate::draw_distance::ActiveRules>,
    mut special_at: ResMut<SpecialAt>,
    mut loads: ResMut<LoadQueue>,
    mut drawn: Query<(&crate::terrain_view::ObjectName, &mut Transform)>,
    lookup: Option<Res<crate::terrain_view::GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    order: Option<Res<crate::terrain_view::ColliderOrder>>,
    (buttons, right_at): (Res<ButtonInput<MouseButton>>, Res<RightAt>),
    shared: Res<Shared>,
) {
    let (mut colliders, mut view, mut anim) = (colliders, view, anim);
    let (Some(mut sim), Some(mut connor)) = (sim, connor) else { return };
    let sim = &mut *sim;
    // `--special`: as if a script asked for it now.
    if let Some((items, _)) = special_at.0.take_if(|(_, n)| sim.active.world.updates >= *n) {
        let at = "test --special".to_string();
        sim.active.world.log.push(format!("update {}: {at}: KQMonster::setState Connor special {items}", sim.active.world.updates));
        sim.active.world.requests.push(Request::ConnorSpecial { items, at });
    }
    // What his trap touches ask counts as asked inside his update (rule
    // 30 of anim-scripts.md: a trap's script runs inside his move; 3m-1),
    // as do the `Push` calls below (3l-1).
    let inside_from = sim.active.world.requests.len();
    sim.active.world.connor_jump = connor.jump_state();
    sim.active.world.connor_can_turn = connor.can_turn_to_listener();
    // characters.md rule 16 (Q156): his state for StateChange messages.
    let portal = sim.active.world.connor_portal.is_some();
    sim.active.world.connor_state = Some(if portal { kq8_core::character::state_number::PORTAL } else { connor.state_number() });
    sim.active.world.connor_at = Some(connor.mover.position);
    // potions.md rule 26 (M5 5g-9, Q146): a cutscene hides him from
    // characters; the hammer throw's handsOff keeps the pointer and doesn't.
    sim.active.world.connor_hands_off = connor.hands_off && connor.throw.is_none();
    sim.active.world.connor_move_key = connor.intent.forward != 0;
    sim.active.world.connor_back_key = connor.intent.forward == -1;
    let position = connor.mover.position;
    if sim.connor_checked_at != Some(position) {
        // His move since the last check (traps.md rule 6: swept).
        let from = sim.connor_checked_at.replace(position).unwrap_or(position);
        let (min, max) = kq8_core::connor_mover::CONNOR_BOX;
        let v = Visitor {
            name: &connor.placement.name,
            collision_type: kq8_core::collide::CONNOR_TYPE,
            from,
            position,
            min,
            max,
            balls: &connor.mover.balls,
        };
        sim.active.connor_moved(&v);
    }
    // His play-once movements that ended (3i-7): getEndLoop listeners.
    for m in std::mem::take(&mut connor.ended) {
        sim.active.connor_movement_ended(&m);
    }
    // His scripted turns and walks that ended (3l-2): their cues.
    for end in std::mem::take(&mut connor.auto_ends) {
        match &end.cue {
            Some(cue) => sim.active.connor_auto_cue(cue, &end.why),
            None => sim.active.world.log.push(format!("update {}: Connor's scripted movement ended ({}): no cue", sim.active.world.updates, end.why)),
        }
    }
    // Scripted pushables he pushed (3j-3): their scripts get Push now,
    // right after his move (pushing.md rule 3 says mid-move: our order).
    // What those calls ask counts as asked inside his update (3l-1): a
    // `move Connor` among it is followed by the rest of his update.
    let pushed = std::mem::take(&mut connor.push_calls);
    for name in &pushed {
        if !sim.active.push_call(name) && !connor.not_pushable.contains(name) {
            connor.not_pushable.push(name.clone());
        }
    }
    // M4 4e-3: Connor's side of a conversation (conversation.md rules
    // 6-15), after his move and this update's scripts; what it asks of him
    // and the camera joins the requests below.
    if sim.active.world.conversation.is_some() || sim.active.world.connor_leave.is_some() {
        let asked = sim.active.world.conversation.as_ref().is_some_and(|c| !c.entered());
        let may_enter = if !asked {
            Ok(())
        } else if matches!(connor.walker.mode, Mode::Dead | Mode::Dying) {
            Err(format!("he is {}", connor.walker.mode.name()))
        } else if connor.state_request("talk state", "conversation") {
            Ok(())
        } else {
            Err("his jump is not finished".to_string())
        };
        let position = connor.mover.position;
        // conversation.md rule 14: the camera-move button (not during
        // handsOff, which ignores the mouse look); `--right` for tests.
        let button = camera.as_ref().and_then(|c| c.mouse_button()).is_some_and(|b| buttons.pressed(b));
        let test = right_at.0.is_some_and(|n| sim.active.world.updates >= n);
        let right_held = (button || test) && !connor.hands_off;
        let ConnorSim { walker, .. } = &mut *connor;
        sim.active.conversation_tick(walker, position, may_enter, right_held);
        sim.flush_conversation(&shared);
    }
    // M4 4e-7: Connor's gesture codes (conversation.md rule 18).
    for code in std::mem::take(&mut sim.active.world.connor_gestures) {
        let wanted = match code {
            8..=15 => kq8_data::characters::GESTURES[(code - 8) as usize],
            _ => "stop",
        };
        let name = connor.movements.iter().find(|m| m.name.eq_ignore_ascii_case(wanted)).map(|m| m.name);
        let line = connor.walker.gesture(code, name);
        let n = connor.updates;
        connor.log.detail(format!("connor update {n}: lip-sync {line}"));
        shared.log.lock().unwrap().detail(format!("connor update {n}: lip-sync {line}"));
    }
    // State requests a jump refused at an earlier update go first (3m-1).
    let retried = connor.refused.len();
    let inside = inside_from + retried..sim.active.world.requests.len() + retried;
    sim.flush_traps(&shared);
    sim.flush_items(&shared);
    sim.flush_rope(&shared);
    sim.flush_talk(&shared);
    let mut camera = camera;
    // characters.md rules 9-11 (Q155): the Choice state's calls from his
    // update (DoChoice, StartChoice after a swap); what they ask is applied
    // below with the rest.
    let choice_script = connor.choice.as_ref().map(|c| c.list.script.clone());
    for args in std::mem::take(&mut connor.choice_calls) {
        if let Some(script) = &choice_script {
            sim.active.run_script(script, &args);
        }
    }
    let requests: std::collections::VecDeque<(usize, Request)> =
        std::mem::take(&mut connor.refused).into_iter().chain(std::mem::take(&mut sim.active.world.requests)).enumerate().collect();
    let mut requests = requests;
    // clicking.md rule 35 (Q105 b): a state a script asked for (and got)
    // before a click's turn decides it; otherwise his state at the click.
    let mut asked = false;
    while let Some((i, r)) = requests.pop_front() {
        let again = if i < retried { " (tried again)" } else { "" };
        // characters.md rules 3, 11 (Q155): before the choice, the Choice
        // state refuses every other state (kept as his waiting request,
        // thrown away at DoChoice: so dropped).
        let refused_by_choice = connor.choice.as_ref().is_some_and(|c| !c.made)
            && match &r {
                Request::ConnorSpecial { .. } | Request::ConnorWait { .. } | Request::ConnorDies { .. } => true,
                Request::ConnorAuto { auto, .. } => matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. }),
                _ => false,
            };
        if refused_by_choice {
            let line = format!("update {}: {r:?}: refused, Connor is in the Choice state before the choice; dropped (characters.md rules 3, 11)", sim.active.world.updates);
            connor.log.detail(line.clone());
            if let Some(log) = &mut sim.trap_log {
                log.detail(line);
            }
            continue;
        }
        // anim-scripts.md rule 28: during an unfinished jump a state
        // request finishes it first (handsOff on) or is refused.
        let state = match &r {
            Request::ConnorSpecial { at, .. } => Some(("setState special", at.clone())),
            Request::ConnorWait { at } => Some(("setState Wait", at.clone())),
            // fighting.md rule 29 (M5 5c-4): his dying state is a state
            // change too (**ours**: during a jump the same rule 28 applies).
            Request::ConnorDies { at, .. } => Some(("dying state", at.clone())),
            Request::ConnorAuto { auto, at } if matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. }) => Some(("scripted walk (Wait)", at.clone())),
            _ => None,
        };
        if let Some((what, at)) = state {
            // ropeandhook.md rule 28 (M6 6a-6): refused and dropped.
            if connor.roping() {
                connor.state_request(what, &at);
                let line = format!("{at}: {what}{again}: refused, Connor is in the rope state; dropped (ropeandhook.md rule 28)");
                sim.active.world.rope_note(line.clone());
                if let Some(log) = &mut sim.trap_log {
                    log.detail(format!("update {}: {line}", sim.active.world.updates));
                }
                continue;
            }
            if !connor.state_request(what, &at) {
                let why = if connor.throw.is_some() { "Connor's hammer throw goes on, azriel-hammer.md rule 3; stand-in, Q148 d" } else { "Connor's jump is not finished" };
                // azriel-hammer.md "Connor's waiting request" (Q148 d): one
                // slot; a later refused request replaces it (the "strong
                // states" exception not done).
                let replaced = if connor.refused.is_empty() { "" } else { "; replaces the one waiting" };
                let line = format!("update {}: {at}: {what}{again}: refused ({why}); kept as his waiting request{replaced}, tried again next update", sim.active.world.updates);
                if let Some(log) = &mut sim.trap_log {
                    log.detail(line);
                }
                connor.refused.clear();
                connor.refused.push(r);
                continue;
            }
            asked = true;
        }
        let line = match r {
            Request::ConnorHurt { roll_over, at } => {
                let ouch = sim.active.world.connor_fight.as_ref().map_or(kq8_core::fight::DEFAULT_OUCH.to_string(), |f| f.ouch_sound.clone());
                let now = sim.active.world.now();
                let line = connor.hurt(roll_over, &ouch, now);
                sim.active.world.fight_note("Connor", format!("{at}: {line}"));
                format!("{at}: applied: Connor {line}")
            }
            Request::ConnorDies { kind, at } => {
                let line = connor.die_from_hit(kind);
                sim.active.world.fight_note("Connor", format!("{at}: {line}"));
                format!("{at}: applied: Connor {line}")
            }
            Request::EndJump { at } => format!("{at}: applied: {}", connor.end_jump(&at)),
            Request::ConnorChoice { list, at } => {
                // characters.md rules 3-5 (Q155): entered at once; the script
                // gets StartChoice at once and what it asks (his `move`) is
                // applied now, before his next update measures the points.
                let script = list.script.clone();
                let line = connor.enter_choice(list, &at);
                let from = sim.active.world.requests.len();
                sim.active.run_script(&script, &["Connor".into(), "StartChoice".into()]);
                let asked_now: Vec<Request> = sim.active.world.requests.drain(from..).collect();
                let n = asked_now.len();
                for r in asked_now.into_iter().rev() {
                    requests.push_front((usize::MAX, r));
                }
                format!("{at}: applied: Connor {line}; StartChoice asked {n} thing(s), applied next")
            }
            Request::ConnorStandNow { at } => {
                connor.walker.stand_now();
                connor.intent = Default::default();
                format!("{at}: applied: Connor straight to standing (stop from its start)")
            }
            Request::SetRun { run, at } => {
                connor.set_run(run, &at);
                format!("{at}: applied: Connor's run switch {}", if run { "on" } else { "off" })
            }
            Request::SetRange { min, max, at } => match camera.as_mut() {
                Some(c) => format!("{at}: applied: camera range ({min:.0}, {max:.0}){}", if c.set_range(min, max) { "" } else { " REFUSED" }),
                None => format!("{at}: camera range ({min:.0}, {max:.0}): no game camera; not applied"),
            },
            Request::ActiveCamera { camera, at } => {
                rules.camera = camera;
                format!("{at}: applied: draw distance measured from {}", if camera { "the camera" } else { "Connor" })
            }
            Request::MakeActive { name, on, at } => {
                if on {
                    rules.always_near.insert(name.to_ascii_lowercase());
                } else {
                    rules.always_near.remove(&name.to_ascii_lowercase());
                }
                format!("{at}: applied: [{name}] {}", if on { "always near" } else { "back to the distance test" })
            }
            Request::ConnorSpecial { items, at } => {
                let line = connor.start_special(&items, &at);
                format!("{at}: applied{again}: Connor {line}")
            }
            Request::ConnorAnim { movement, at } => {
                let line = connor.script_anim(&movement, &at);
                format!("{at}: applied: Connor {line}")
            }
            Request::ConnorWait { at } => {
                let line = connor.enter_wait(&at);
                format!("{at}: applied{again}: Connor {line}")
            }
            Request::Move { who, flags, position, turn, at } if who.eq_ignore_ascii_case("Connor") => {
                // The scene his update used; the objects he pushed this
                // update are still passed through (pushing.md rule 3).
                let objects = colliders.as_ref().map_or(&[][..], |c| &c.0[..]);
                let line = match &lookup {
                    Some(lookup) => {
                        let mut scene = kq8_core::connor_mover::Scene::new(&lookup.0, buildings.as_ref().map_or(&[][..], |b| &b.0[..]), objects);
                        scene.order = order.as_ref().map(|o| &o.0[..]);
                        scene.passed.borrow_mut().extend(pushed.iter().cloned());
                        connor.move_to(flags, position, turn, &at, &scene, inside.contains(&i))
                    }
                    None => connor.move_to(flags, position, turn, &at, &NoGround, inside.contains(&i)),
                };
                // Its "Connor moved" reaches the camera after the script
                // (camera.md "When the message arrives").
                if let Some(c) = camera.as_mut() {
                    c.connor_moved_by_script(&at);
                }
                // A `move` is a jump to the new spot, not a walk along the
                // line to it (traps.md rule 5 "Placed by a script", Q102):
                // the trap he is in runs its stay test now (b); no trap is
                // entered now, his next ordinary move's sweep starts from
                // here (a).
                let p = connor.mover.position;
                sim.connor_checked_at = Some(p);
                let (min, max) = kq8_core::connor_mover::CONNOR_BOX;
                let v = Visitor { name: &connor.placement.name, collision_type: kq8_core::collide::CONNOR_TYPE, from: p, position: p, min, max, balls: &connor.mover.balls };
                let calls = sim.active.connor_placed(&v);
                sim.flush_traps(&shared);
                let left = if calls.is_empty() { "no trap left".to_string() } else { format!("stay test: {}", calls.join("; ")) };
                format!("{at}: applied: {line}; {left}; the traps' sweep starts again from here (the jump's path is not swept)")
            }
            Request::Move { who, flags, position, turn, at } if who.eq_ignore_ascii_case("KQCamera") => match camera.as_mut() {
                Some(c) => format!("{at}: applied: {}", c.move_to(flags, position, turn)),
                None => format!("{at}: move KQCamera: no game camera; not applied"),
            },
            // ropeandhook.md rule 9 (M6 6a-2): as a `move KQCamera` while
            // following (Q147 c).
            Request::RopeCamera { spot, aim, at } => match camera.as_mut() {
                Some(c) => {
                    let (tilt, heading) = kq8_core::camera::view_angles(std::array::from_fn(|k| aim[k] - spot[k]));
                    // Q147 c: it keeps following, but Connor's first move is
                    // not followed.
                    c.skip_next_move = true;
                    let line = format!("{at}: applied: rope camera looking at ({:.0}, {:.0}, {:.0}): {}; his first move will not be followed (rule 9)", aim[0], aim[1], aim[2], c.move_to(0, spot, [tilt, 0.0, heading]));
                    sim.active.world.rope_note(line.clone());
                    line
                }
                None => format!("{at}: rope camera: no game camera; not applied"),
            },
            Request::CamSpot { spot, newlist, at } => match camera.as_mut() {
                Some(c) => format!("{at}: applied: {}", c.add_spot(spot, newlist)),
                None => format!("{at}: addCamLoc: no game camera; not applied"),
            },
            Request::CamFly { aim, seconds, end_roll, at } => match camera.as_mut() {
                Some(c) => format!("{at}: applied: {}", c.fly(aim, seconds, end_roll)),
                None => format!("{at}: flyToCamLoc: no game camera; not applied"),
            },
            Request::CamCut { aim, roll, at } => match (camera.as_mut(), &lookup) {
                (Some(c), Some(l)) => {
                    let ground = |x: f32, y: f32| kq8_core::connor_mover::GroundLookup::ground(&l.0, x, y).map(|h| h.0);
                    format!("{at}: applied: {}", c.cut_to(aim, roll, &ground, &crate::game_camera::surface_line(&l.0)))
                }
                (Some(c), None) => format!("{at}: applied: {}", c.cut_to(aim, roll, &|_, _| None, &|_, _| None)),
                (None, _) => format!("{at}: cutToCamLoc: no game camera; not applied"),
            },
            Request::CameraFollow { connor: follow, at } => match camera.as_mut() {
                Some(c) => format!("{at}: applied: {}", c.follow(follow, connor.mover.position)),
                None => format!("{at}: KQCamera::follow: no game camera; not applied"),
            },
            Request::CameraAround { point, at } => match camera.as_mut() {
                Some(c) => format!("{at}: applied: {}", c.around(point)),
                None => format!("{at}: camera around a point: no game camera; not applied"),
            },
            // M5 5d-2, 5d-3: a new copy, drawn by copies_view.
            Request::Spawn { file, name, placement, at } => {
                let p = placement.position;
                sim.spawns.push((file.clone(), name.clone(), placement));
                format!("{at}: loadKQ {file}: new copy {name} at ({:.0}, {:.0}, {:.0}): to be drawn", p[0], p[1], p[2])
            }
            Request::Load { file, name, at } => {
                loads.0.push(name.clone());
                format!("{at}: applied: loadKQ {file}: {name} comes in (next system)")
            }
            Request::ConnorAuto { auto, at } => {
                // A walk switches handsOff on by itself (rule 24).
                let walk = matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. });
                if walk {
                    connor.hands_off = true;
                }
                let line = connor.walker.start_auto(auto);
                let n = connor.updates;
                connor.log.detail(format!("connor update {n}: {line}{} ({at})", if walk { "; handsOff on" } else { "" }));
                format!("{at}: applied: Connor {line}{}", if walk { "; handsOff on" } else { "" })
            }
            Request::ClickTurn { name, target, at } => {
                // clicking.md rules 27-28, 35: his state after the script:
                // the one it asked for (its requests above are applied
                // already), else the one at the click.
                let at_click = sim.click_states.pop_front();
                let ((turns, state), whence) = match (std::mem::take(&mut asked), at_click) {
                    (false, Some(s)) => (s, "his state at the click"),
                    (true, _) => (connor.click_turn_state(), "the state the script asked for"),
                    (false, None) => (connor.click_turn_state(), "his state now (no click recorded)"),
                };
                let state = format!("{state} ({whence}, rule 35)");
                let n = connor.updates;
                if turns {
                    let auto = kq8_core::connor_auto::AutoMove::new(kq8_core::connor_auto::AutoKind::Point, target, None);
                    let line = connor.walker.start_auto(auto);
                    connor.log.detail(format!("connor update {n}: Connor's state after the script {state}: turn to {name}: {line} ({at})"));
                    format!("{at}: applied: Connor's state after the script {state}: turn to {name} (clicking.md rules 25-32): {line}")
                } else {
                    connor.log.detail(format!("connor update {n}: Connor's state after the script {state}: no turn to {name} (rule 28) ({at})"));
                    format!("{at}: Connor's state after the script {state}: no turn (clicking.md rule 28)")
                }
            }
            Request::WalkExit { at } => match connor.walker.auto.take_if(|a| matches!(a.kind, kq8_core::connor_auto::AutoKind::Walk { .. })) {
                Some(a) => {
                    connor.walker.stop_at_once();
                    let n = connor.updates;
                    connor.log.detail(format!("connor update {n}: walkExit: the scripted walk ends now ({at})"));
                    if let Some(cue) = &a.cue {
                        sim.active.connor_auto_cue(cue, "walkExit");
                    }
                    format!("{at}: applied: walkExit: the scripted walk ended{}", if a.cue.is_some() { ", cue sent" } else { "" })
                }
                None => format!("{at}: walkExit: no scripted walk going on; nothing"),
            },
            Request::HandsOff { on, at } => {
                // anim-scripts.md rules 21-22: on, the keys count as let go
                // (presses swallowed, releases counted) and the camera's
                // fixed mode ends; off, the held movement keys are sent
                // again.
                let was = connor.hands_off;
                connor.hands_off = on;
                let mut extra = Vec::new();
                if on {
                    extra.extend(camera.as_mut().and_then(|c| c.hands_off()));
                } else if was {
                    extra.push(connor.keys_sent_again());
                }
                let n = connor.updates;
                let extra = if extra.is_empty() { String::new() } else { format!("; {}", extra.join("; ")) };
                connor.log.detail(format!("connor update {n}: handsOff {on}{extra} ({at})"));
                format!("{at}: applied: handsOff {on}: movement, turn, run and Jump keys, the mouse look and clicks {}{extra}", if on { "ignored" } else { "work again" })
            }
            Request::Move { who, flags, position, turn, at } => {
                // A plain placed object (3k-1): its colliders and drawn
                // parts. Its old place is its collider's.
                let old = colliders.as_ref().and_then(|c| c.0.iter().find(|b| b.name.eq_ignore_ascii_case(&who))).map(|b| b.placement);
                match old {
                    Some(old) => {
                        let (new, note) = kq8_core::active::moved_placement(old, flags, position, turn);
                        let d: [f32; 3] = std::array::from_fn(|k| new.position[k] - old.position[k]);
                        let mut boxes = 0;
                        for list in [colliders.as_mut().map(|c| &mut c.0), view.as_mut().map(|v| &mut v.boxes)].into_iter().flatten() {
                            for b in list.iter_mut().filter(|b| b.name.eq_ignore_ascii_case(&who)) {
                                b.placement = new;
                                boxes += 1;
                            }
                        }
                        let shift = Vec3::from(crate::to_bevy(d, [0.0; 3]));
                        let mut parts = 0;
                        for (_, mut t) in drawn.iter_mut().filter(|(n, _)| n.0.eq_ignore_ascii_case(&who)) {
                            t.translation += shift;
                            parts += 1;
                        }
                        let turned = if new.rotation != old.rotation { "; its drawn turn not changed (not done)" } else { "" };
                        format!(
                            "{at}: applied: moved {who} by ({:.0}, {:.0}, {:.0}): {boxes} collider(s), {parts} drawn part(s){turned}{}",
                            d[0],
                            d[1],
                            d[2],
                            note.map_or(String::new(), |n| format!("; {n}"))
                        )
                    }
                    None => format!("{at}: move {who}: no collider to know its place; not moved (not done)"),
                }
            }
            Request::Delete { name, at } => {
                let mut off = 0;
                if let Some(c) = colliders.as_mut() {
                    for b in c.0.iter_mut().filter(|b| b.name.eq_ignore_ascii_case(&name)) {
                        switch_off(b);
                        off += 1;
                    }
                }
                let before = view.as_ref().map_or(0, |v| v.boxes.len());
                if let Some(v) = view.as_mut() {
                    v.boxes.retain(|b| !b.name.eq_ignore_ascii_case(&name));
                }
                let view_dropped = before - view.as_ref().map_or(0, |v| v.boxes.len());
                if let Some(a) = anim.as_mut() {
                    a.forget_collider(&name);
                }
                deleted.0.push(name.clone());
                format!("{at}: applied: {name} deleted: {off} collider(s) switched off, {view_dropped} view box(es) dropped, hidden")
            }
        };
        let line = format!("update {}: {line}", sim.active.world.updates);
        shared.log.lock().unwrap().summary(format!("traps: {line}"));
        if let Some(log) = &mut sim.trap_log {
            log.detail(line);
        }
    }
    sim.flush_rope(&shared);
}

/// The clickable buildings' BSP for the click line (3g-2): the same line
/// test the camera uses, only for buildings whose type has bit 0x20 now.
struct ClickBuildings<'a> {
    buildings: &'a [kq8_core::collide::BuildingCollider],
    active: &'a Active,
}

impl kq8_core::active::ClickWalls for ClickBuildings<'_> {
    fn first_hit(&self, from: [f32; 3], to: [f32; 3]) -> Option<(String, f32)> {
        let dist = |p: [f32; 3]| (0..3).map(|k| (p[k] - from[k]).powi(2)).sum::<f32>().sqrt();
        self.buildings
            .iter()
            .enumerate()
            .filter(|(i, _)| self.active.clickable_building(*i))
            .filter_map(|(_, b)| Some((b.name.clone(), dist(b.line_hit(from, to)?))))
            .min_by(|a, b| a.1.total_cmp(&b.1))
    }
}

/// `characters.md` rule 8 (M4 4d-6b-2): each character's mover, after
/// the characters' own update in `door_step`; buildings only for a mask
/// with 0x8; no objects, characters or Connor yet (Q128).
pub fn character_gravity(
    sim: Option<ResMut<DoorSim>>,
    lookup: Option<Res<crate::terrain_view::GroundLookup>>,
    buildings: Option<Res<crate::terrain_view::Buildings>>,
    objects: Option<Res<crate::terrain_view::ObjectColliders>>,
    mut solid: ResMut<CharacterColliders>,
) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let b = buildings.as_ref().map_or(&[][..], |b| &b.0[..]);
    // M5 5a-3: their own moves meet objects, each other and Connor too.
    let o = objects.as_ref().map_or(&[][..], |o| &o.0[..]);
    // M5 5a-4: sight and the active list, then the movers.
    sim.active.world.characters_ai(&lookup.0, b, o);
    sim.active.world.characters_gravity(&lookup.0, b, o);
    // thrown-items.md (M5 5e-1): thrown items, with the same buildings
    // and objects.
    sim.active.world.flights_step(&lookup.0, b, o);
    // M4 4d-12-3: the characters solid for Connor where they stand now,
    // for his scene; his hits on them since the last update to the log.
    solid.list = sim.active.world.character_colliders.clone();
    let hits = std::mem::take(&mut solid.hits);
    sim.active.world.character_log.extend(hits);
}

/// The characters solid for Connor now (`connor-collision.md` rules 2–4,
/// 18–19; M4 4d-12-3), copied from the running game after the characters'
/// gravity, for Connor's scene; and his hits on them, for `characters.log`.
#[derive(Resource, Default)]
pub struct CharacterColliders {
    pub list: Vec<kq8_core::collide::CylinderCollider>,
    pub hits: Vec<String>,
}

/// Left click: the click line from the camera through the mouse picks the
/// nearest clickable box or clickable building (clicking.md rules 1-8;
/// the ground is not tested), handled at the next update. `--aim NAME,N`
/// does the same at update N with a line from the camera to the middle of
/// NAME's box (a test option; `--click` skips the line).
#[allow(clippy::too_many_arguments)]
pub fn door_click_input(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    sim: Option<ResMut<DoorSim>>,
    buildings: Option<Res<Buildings>>,
    mut aim: ResMut<AimAt>,
    cover: Res<crate::gui_view::GuiCover>,
    shared: Res<Shared>,
    keys: Res<ButtonInput<KeyCode>>,
    (objects, lookup): (Option<Res<crate::terrain_view::ObjectColliders>>, Option<Res<crate::terrain_view::GroundLookup>>),
) {
    let Some(mut sim) = sim else { return };
    let updates = sim.active.world.updates;
    let due = aim.0.iter().position(|(_, n)| updates >= *n).map(|i| aim.0.remove(i).0);
    if due.is_none() && !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    // M4 4c-2: a click on a shown panel is the interface's, not the world's.
    if due.is_none() {
        let on_panel = window.single().ok().and_then(|w| {
            let c = w.cursor_position()?;
            let (x, y) = (c.x / w.width(), c.y / w.height());
            cover.0.iter().find(|(_, [l, t, r, b])| x >= *l && x < *r && y >= *t && y < *b).map(|(n, _)| *n)
        });
        if let Some(panel) = on_panel {
            sim.log.detail(format!("update {updates}: mouse click on the panel {panel}: not a world click"));
            return;
        }
    }
    // talking.md rule 16: any click in the world counts, picked or not.
    sim.world_clicks.push(if due.is_some() { "test --aim click".to_string() } else { "mouse click".to_string() });
    let Ok((camera, at)) = camera.single() else { return };
    // Bevy (x, y, z) = game (x, z, -y) / 100, so back: (x, -z, y) x 100.
    let to_game = |v: Vec3| [v.x, -v.z, v.y];
    let (origin, dir, how) = match due {
        // `--aim-point X,Y,Z,N` (M5 5f-5 test): the line from the camera to
        // that point.
        Some(name) if name.starts_with('@') => {
            let origin = to_game(at.translation() * UNITS_PER_METRE);
            let p: Vec<f32> = name[1..].split(':').filter_map(|v| v.parse().ok()).collect();
            let target = [p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0), p.get(2).copied().unwrap_or(0.0)];
            let d: [f32; 3] = std::array::from_fn(|k| target[k] - origin[k]);
            let len = d.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
            (origin, d.map(|v| v / len), format!("test --aim-point ({:.0}, {:.0}, {:.0})", target[0], target[1], target[2]))
        }
        Some(name) => {
            let origin = to_game(at.translation() * UNITS_PER_METRE);
            let Some(target) = sim.active.target_point(&name) else {
                let line = format!("update {updates}: test --aim {name}: no clickable object by that name with a box");
                shared.log.lock().unwrap().summary(&line);
                sim.log.detail(line);
                return;
            };
            let d: [f32; 3] = std::array::from_fn(|k| target[k] - origin[k]);
            let len = d.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
            (origin, d.map(|v| v / len), format!("test --aim {name} (camera to the middle of its box)"))
        }
        None => {
            let Ok(window) = window.single() else { return };
            let Some(cursor) = window.cursor_position() else { return };
            let Ok(ray) = camera.viewport_to_world(at, cursor) else { return };
            (to_game(ray.origin * UNITS_PER_METRE), to_game(*ray.direction), format!("mouse click at ({:.0}, {:.0}) px", cursor.x, cursor.y))
        }
    };
    let walls = ClickBuildings { buildings: buildings.as_deref().map_or(&[][..], |b| &b.0[..]), active: &sim.active };
    let pick = sim.active.pick_line(origin, dir, &walls);
    let from = format!("update {updates}: {how}: line from ({:.0}, {:.0}, {:.0})", origin[0], origin[1], origin[2]);
    let (line, picked) = match pick {
        Pick::Seen { name, t } => (format!("{from} hits {name} first, at {t:.0} units (box)"), Some(name)),
        Pick::Building { name, t, behind } => {
            let hidden = behind.map_or(String::new(), |(b, tb)| format!("; it hides {b} at {tb:.0}"));
            (format!("{from} hits building {name} first, at {t:.0} units{hidden}"), Some(name))
        }
        Pick::Nothing => (format!("{from} meets nothing clickable"), None),
    };
    // fighting.md rule 39 (M5 5c-5; the input side is a stand-in from the
    // manual): with a hand weapon ready a left click swings toward the
    // click (Shift: the big swing); Ctrl held gives the plain click.
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    // ranged-weapons.md rule 13 (M5 5f-5): an item on the pointer wins:
    // the click uses it, whatever is ready.
    let held = sim.active.world.hand.is_some();
    // ropeandhook.md rule 10 (M6 6a-3): with the rope in hand any click in
    // the world, whatever it meets, starts the climb.
    let w = &sim.active.world;
    let rope_held = w.hand.as_ref().and_then(|h| w.items.get(h)).and_then(|o| w.items.kind(&o.kind)).is_some_and(|t| t.category == kq8_core::rope::ROPE_CATEGORY);
    if rope_held {
        let line = format!("{line}; the rope is in hand: the click starts the climb, whatever it meets (ropeandhook.md rule 10)");
        sim.rope_clicks.push(how.clone());
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
        return;
    }
    // Rules 14-16: the shot click (Ctrl: the plain click).
    let mut line = line;
    if sim.active.world.weapon_mode == kq8_core::weapons::WeaponMode::Ranged && !ctrl && !held {
        use kq8_core::weapons::{AIM_PICK, ICEWATER, PLAIN_CLICK_BIT, TALK_PICK};
        let w = &sim.active.world;
        let objs = objects.as_deref().map_or(&[][..], |o| &o.0[..]);
        let blds = buildings.as_deref().map_or(&[][..], |b| &b.0[..]);
        let talk = w.mask_pick(origin, dir, TALK_PICK, objs, blds);
        let plain = talk.as_ref().is_some_and(|h| (h.character.is_some() && h.collision_type & PLAIN_CLICK_BIT != 0) || h.name.eq_ignore_ascii_case(ICEWATER));
        if plain {
            line += &format!("; the ranged weapon is ready, the talk pick meets {} (type 0x{:X}): the plain click (ranged-weapons.md rule 15)", talk.as_ref().map_or("", |h| h.name.as_str()), talk.as_ref().map_or(0, |h| h.collision_type));
        } else {
            let far: [f32; 3] = std::array::from_fn(|k| origin[k] + dir[k] * kq8_core::active::CLICK_LINE_LENGTH);
            let (aim, what) = match w.mask_pick(origin, dir, AIM_PICK, objs, blds) {
                Some(h) => match h.character {
                    // Rule 16: a character: its x, y, the height where the line met its box.
                    Some(j) => {
                        let p = w.anims[j].player.base.position;
                        ([p[0], p[1], h.point[2]], format!("character {} (type 0x{:X})", h.name, h.collision_type))
                    }
                    None => (h.point, format!("{} {} (type 0x{:X})", if h.building { "building" } else { "object" }, h.name, h.collision_type)),
                },
                None => match lookup.as_deref().and_then(|g| kq8_core::connor_mover::GroundLookup::drawn_line_hit(&g.0, origin, far)) {
                    Some(p) => (p, "the drawn ground".to_string()),
                    None => (far, "nothing: the line's far end (the sky)".to_string()),
                },
            };
            let line = format!("{line}; the ranged weapon is ready: the aim pick meets {what} at ({:.0}, {:.0}, {:.0}) (ranged-weapons.md rule 16)", aim[0], aim[1], aim[2]);
            sim.shot_clicks.push(ShotClick { aim, what, how: how.clone() });
            shared.log.lock().unwrap().summary(&line);
            sim.log.detail(line);
            return;
        }
    }
    // ranged-weapons.md rule 32 (M5 5f-7): the swing click. The click line
    // against types with bit 0x40 or 0x20 (characters, plain objects,
    // barrels, clickables, the clickable buildings); a character with bit
    // 0x8000 (or `icewater`) gets the plain click (rule 15).
    if sim.active.world.weapon_mode == kq8_core::weapons::WeaponMode::Hand && !ctrl && !held {
        use kq8_core::weapons::{ICEWATER, PLAIN_CLICK_BIT, SWING_PICK};
        let w = &sim.active.world;
        let objs = objects.as_deref().map_or(&[][..], |o| &o.0[..]);
        let blds = buildings.as_deref().map_or(&[][..], |b| &b.0[..]);
        let hit = w.mask_pick(origin, dir, SWING_PICK, objs, blds);
        let plain = hit.as_ref().is_some_and(|h| (h.character.is_some() && h.collision_type & PLAIN_CLICK_BIT != 0) || h.name.eq_ignore_ascii_case(ICEWATER));
        if !plain {
            // Its position (a character's feet, an object's place); nothing
            // or a building: no pick, he turns toward the click point.
            let picked = hit.as_ref().filter(|h| !h.building).map(|h| {
                let at = match h.character {
                    Some(j) => w.anims[j].player.base.position,
                    None => objs.iter().find(|o| o.name == h.name).map_or(h.point, |o| o.placement.position),
                };
                (h.name.clone(), at)
            });
            let point = match (&picked, &hit) {
                (Some((_, at)), _) => [at[0], at[1]],
                (None, Some(h)) => [h.point[0], h.point[1]],
                // **Stand-in** (the note gives no click point when the line
                // meets nothing): where it meets the level of his feet;
                // straight ahead along it if it never comes down.
                (None, None) => {
                    let z = w.connor_at.map_or(origin[2], |c| c[2]);
                    let t = if dir[2] < -1e-4 { (z - origin[2]) / dir[2] } else { 10_000.0 };
                    [origin[0] + dir[0] * t, origin[1] + dir[1] * t]
                }
            };
            let met = match &hit {
                Some(h) => format!("{} {} (type 0x{:X})", if h.building { "building" } else if h.character.is_some() { "character" } else { "object" }, h.name, h.collision_type),
                None => "nothing".into(),
            };
            let big = keys.pressed(KeyCode::ShiftLeft) || keys.pressed(KeyCode::ShiftRight);
            let line = format!("{line}; a hand weapon is ready: the swing pick meets {met}: a swing toward ({:.0}, {:.0}){} (ranged-weapons.md rule 32)", point[0], point[1], if big { ", Shift" } else { "" });
            sim.swing_clicks.push(SwingClick { picked, point, big, how: how.clone() });
            shared.log.lock().unwrap().summary(&line);
            sim.log.detail(line);
            return;
        }
    }
    if let Some(name) = picked {
        sim.pending.push((name, "mouse".into()));
    }
    shared.log.lock().unwrap().summary(&line);
    sim.log.detail(line);
}

/// The ready weapon's movement list (`ranged-weapons.md` rule 10): a hand
/// weapon's from `Connor.kq`'s `ShortAnimList` (`fighting.md` rule 40:
/// `confists.anm` for `INVITEM_Fists`); a ranged one's from
/// `LongAnimList` (M5 5f-4); none: his own (`None`).
pub(crate) fn weapon_list(w: &kq8_core::active::WorldState) -> Option<String> {
    let ready = w.weapon_ready.as_deref()?;
    let short = ready.strip_prefix(kq8_data::items::PREFIX).unwrap_or(ready);
    let f = w.connor_fight.as_ref()?;
    let lists = match w.weapon_mode {
        kq8_core::weapons::WeaponMode::Hand => &f.short_lists,
        kq8_core::weapons::WeaponMode::Ranged => &f.long_lists,
        kq8_core::weapons::WeaponMode::None => return None,
    };
    lists.iter().find(|(n, _)| n.eq_ignore_ascii_case(short)).map(|(_, f)| f.clone())
}

/// One update of the doors (15 per second), before Connor's: clicks,
/// events, door movements; then the door colliders are moved.
pub fn door_step(
    sim: Option<ResMut<DoorSim>>,
    connor: Option<ResMut<ConnorSim>>,
    colliders: Option<ResMut<ObjectColliders>>,
    mut click_at: ResMut<ClickAt>,
    shared: Res<Shared>,
    (in_range, solid): (Res<crate::draw_distance::CharactersInRange>, Res<crate::draw_distance::CharactersSolid>),
    (mut command_at, mut swing_at): (ResMut<CommandAt>, ResMut<SwingAt>),
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    // `--command`: one script command, as if a script ran it now.
    let updates = sim.active.world.updates;
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut command_at.0).into_iter().partition(|(_, n)| updates >= *n);
    command_at.0 = later;
    for (line, _) in due {
        let result = sim.active.test_command(&line, "test --command");
        shared.log.lock().unwrap().summary(format!("update {updates}: test --command {line}: {result:?}"));
    }
    // characters.md rule 11: which characters update now.
    sim.active.world.characters_in_range = in_range.0.clone();
    // Rule 33 (M4 4d-12-3): which have their collider on.
    sim.active.world.characters_solid = solid.0.clone();
    // KQConner::JumpDir / endJump in timed cues (3m-1).
    sim.active.world.connor_jump = connor.as_ref().and_then(|c| c.jump_state());
    // talking.md rule 7: whether he can turn to a listener, and where.
    sim.active.world.connor_can_turn = connor.as_ref().is_some_and(|c| c.can_turn_to_listener());
    sim.active.world.connor_at = connor.as_ref().map(|c| c.mover.position);
    // potions.md rule 26: a cutscene's handsOff hides the pointer; the
    // hammer throw's keeps it (azriel-hammer.md rule 3), so characters
    // still see him then (M6 6b-3).
    sim.active.world.connor_hands_off = connor.as_ref().is_some_and(|c| c.hands_off && c.throw.is_none());
    // character-movement.md rule 25 (M5 5b-4): his forward / back key.
    sim.active.world.connor_move_key = connor.as_ref().is_some_and(|c| c.intent.forward != 0);
    // fighting.md rule 13 (M5 5c-3): his back key.
    sim.active.world.connor_back_key = connor.as_ref().is_some_and(|c| c.intent.forward == -1);
    let updates = sim.active.world.updates;
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut click_at.0).into_iter().partition(|(_, n)| updates >= *n);
    click_at.0 = later;
    // talking.md rule 16 (M4 4a-5): presses and clicks since the last
    // update cut Connor's line short unless it has '#'; handsOff
    // swallows them (anim-scripts.md rule 21).
    let mut connor = connor;
    let presses = connor.as_mut().map(|c| std::mem::take(&mut c.cut_presses)).unwrap_or_default();
    let clicks: Vec<String> = std::mem::take(&mut sim.world_clicks).into_iter().chain(due.iter().map(|(n, _)| format!("test --click {n}"))).collect();
    let hands_off = connor.as_ref().is_some_and(|c| c.hands_off);
    // conversation.md rule 12: nothing cuts a line short in the talk state.
    let talking = sim.active.world.connor_in_talk();
    for why in presses.into_iter().chain(clicks.into_iter().filter(|_| !hands_off)) {
        let now = sim.active.world.now();
        if talking {
            sim.log.detail(format!("update {updates}: {why}: Connor is in his talk state: no line is cut short (conversation.md rule 12)"));
            continue;
        }
        sim.active.world.talk.cut_short(&why, now);
    }
    // `--swing NAME,N[,big]` (M5 5c-5 test): ready the hand weapon if none
    // is, then a swing click on NAME.
    let (swing_due, swing_later): (Vec<_>, Vec<_>) = std::mem::take(&mut swing_at.0).into_iter().partition(|(_, n, _)| updates >= *n);
    swing_at.0 = swing_later;
    for (name, _, big) in swing_due {
        let w = &mut sim.active.world;
        if w.weapon_ready.is_none() {
            let line = match kq8_core::connor_inventory::hand_weapon(w) {
                Some(kind) => kq8_core::connor_inventory::ready_hand_weapon(w, &kind),
                None => "he carries no hand weapon".to_string(),
            };
            w.fight_note("Connor", format!("test --swing: {line}"));
        }
        // As a click that picks NAME (rule 32): its position (a
        // character's feet; an object's place, else the middle of its box).
        let at = w.character_index(&name).map(|j| w.anims[j].player.base.position);
        let at = at.or_else(|| colliders.as_ref().and_then(|c| c.0.iter().find(|o| o.name.eq_ignore_ascii_case(&name)).map(|o| o.placement.position)));
        let at = at.or_else(|| sim.active.target_point(&name));
        match at {
            Some(p) => sim.swing_clicks.push(SwingClick { picked: Some((name.clone(), p)), point: [p[0], p[1]], big, how: format!("test --swing {name}") }),
            None => sim.log.detail(format!("update {updates}: test --swing {name}: no such character or object")),
        }
    }
    // fighting.md rules 39-41 (M5 5c-5): swings asked for, the swing that
    // waited for his turn, and his `Do` cues.
    if let Some(c) = connor.as_mut() {
        // ranged-weapons.md rules 1-9 (M5 5f-3): Delete, Alt, slot clicks
        // and setDoMode ask for a mode; taken only while he stands or
        // moves (rule 5), then the swap (rule 6).
        let w = &mut sim.active.world;
        for (ask, how) in std::mem::take(&mut c.weapon_keys) {
            w.ask_weapon(ask, &format!("update {updates}: {how}"));
        }
        let mut later = Vec::new();
        for r in std::mem::take(&mut w.weapon_requests) {
            // Q138 (ranged-weapons.md open question 1, traced): moving (a
            // movement key held: his Move state), the swap is taken at once
            // and its away / out cuts the swing short (before `do`: no hit).
            if c.swing.is_some() && c.intent.forward != 0 && !c.hands_off {
                let cut = c.swing.take().map(|s| s.movement).unwrap_or_default();
                c.walker.stand_now();
                w.fight_note("Connor", format!("weapon request ({}): moving: the swap cuts the swing '{cut}' short (Q138)", r.how));
            }
            // Standing refuses every request while a swing plays: it waits
            // for the swing's end (Q138, traced).
            if c.swing.is_some() && !matches!(r.ask, kq8_core::weapons::Ask::Script(_)) {
                if !r.how.ends_with(" (waits for the swing)") {
                    w.fight_note("Connor", format!("weapon request ({}): standing, a swing plays: it waits for its end (Q138)", r.how));
                }
                later.push(kq8_core::weapons::Request { how: if r.how.ends_with(" (waits for the swing)") { r.how } else { format!("{} (waits for the swing)", r.how) }, ..r });
                continue;
            }
            if let Some(ch) = w.take_weapon_request(&r, c.may_ready(), c.hands_off) {
                let list = weapon_list(w);
                let shift = c.shift_held;
                let line = c.ready_set(list, ch.to, ch.to_item.clone(), false, shift, ch.script);
                w.fight_note("Connor", format!("swap ({} -> {}): {line}", ch.from.name(), ch.to.name()));
            }
        }
        w.weapon_requests.splice(0..0, later);
        // Readied or put away another way (picked up while one is ready,
        // 5d fix 11; a conversation's put-away, `conversation.md` rule 7;
        // the `--swing` test): the set follows.
        let list = weapon_list(w);
        if list != c.weapon_target || w.weapon_ready.is_none() != c.weapon_target.is_none() {
            let at_once = std::mem::take(&mut w.weapon_swapped);
            let line = c.ready_set(list, w.weapon_mode, w.weapon_ready.clone(), at_once, false, false);
            w.fight_note("Connor", format!("weapon {:?}: {line}", w.weapon_ready));
        }
        for s in std::mem::take(&mut sim.swing_clicks) {
            let w = &mut sim.active.world;
            let list = weapon_list(w);
            let line = match list {
                Some(list) => {
                    let coin = w.fight_rng.fraction();
                    c.swing_click(&list, s.picked, s.point, s.big, coin, sim.attack_dist)
                }
                None => format!("no movement list for {:?} in Connor.kq's ShortAnimList: no swing", w.weapon_ready),
            };
            w.fight_note("Connor", format!("{}: {line}", s.how));
        }
        if let Some(line) = c.swing_step() {
            sim.active.world.fight_note("Connor", line);
        }
        // ranged-weapons.md rules 18, 20 (M5 5f-5): the shot clicks, the
        // shot that waits for the turn, the bolt at the `do` cue.
        for s in std::mem::take(&mut sim.shot_clicks) {
            let w = &mut sim.active.world;
            let feet = c.mover.position;
            let (aim, moved) = kq8_core::weapons::steepness_limit(feet, w.connor_shoot_offset[2], s.aim);
            let steep = if moved { format!("; steeper than 1 rad from the vertical: moved out to ({:.0}, {:.0}, {:.0}) (rule 18)", aim[0], aim[1], aim[2]) } else { String::new() };
            // azriel-hammer.md rule 2 (M6 6b-3): with the hammer (a
            // `KQHammer` bolt file) the throw instead of the shot.
            let hammer = w.connor_projectile.as_ref().and_then(|f| w.projectiles.get(&f.to_ascii_lowercase())).is_some_and(|p| p.hammer);
            let line = match weapon_list(w) {
                Some(list) if hammer => {
                    let mut line = c.throw_click(&list, aim);
                    if c.throw.is_some() {
                        if let Some(held) = w.hand.take() {
                            line.push_str(&format!("; {held} on the pointer is let go (rule 2)"));
                        }
                    }
                    line
                }
                Some(list) => c.shot_click(&list, aim),
                None => format!("no movement list for {:?} in Connor.kq's LongAnimList: no shot", w.weapon_ready),
            };
            w.fight_note("Connor", format!("{}: shot click at {} ({:.0}, {:.0}, {:.0}){steep}: {line}", s.how, s.what, s.aim[0], s.aim[1], s.aim[2]));
        }
        if let Some(line) = c.shot_step() {
            sim.active.world.fight_note("Connor", line);
        }
        if let Some(line) = c.run_back_step() {
            sim.active.world.fight_note("Connor", line);
        }
        // azriel-hammer.md rules 3, 5, 9 (M6 6b-3): the hammer leaves at the
        // `do` cue from his shoot offset (the part in the frame is not used:
        // the user's check, rule 5), aimed at the aim point; it came back:
        // `catch`; the throw's movements follow each other.
        for (m, phase) in std::mem::take(&mut c.throw_cues) {
            let w = &mut sim.active.world;
            let aim = c.throw.as_ref().map_or(c.mover.position, |t| t.aim);
            let what = format!("{m} at the cue's phase {phase:.4}; the hammer starts at his shoot offset, never the part (azriel-hammer.md rule 5, the user's check)");
            w.connor_shoot(c.mover.position, c.turn_matrix(), None, aim, &what);
        }
        for (who, how) in std::mem::take(&mut sim.active.world.hammers_back) {
            let line = if who.eq_ignore_ascii_case(kq8_core::talk::CONNOR) { c.throw_caught(&how) } else { format!("{how}: thrown by {who}, not Connor: nothing") };
            sim.active.world.fight_note("Connor", line);
        }
        if let Some(line) = c.throw_step() {
            sim.active.world.fight_note("Connor", line);
        }
        let aim_now = c.shot.as_ref().map(|s| s.aim);
        for (m, phase, moving) in std::mem::take(&mut c.shot_cues) {
            let w = &mut sim.active.world;
            let mesh = w.connor_projectile.as_ref().and_then(|f| w.projectiles.get(&f.to_ascii_lowercase())).and_then(|p| p.mesh_name.clone()).unwrap_or_default();
            let part = c.part_at(m, phase, &mesh);
            let (aim, what) = if moving {
                // Rule 20: a moving shot: the item's use sound, flat.
                let sound = w.weapon_ready.as_deref().and_then(|k| w.items.kind(k)).and_then(|t| t.use_sound.clone());
                if let Some(snd) = &sound {
                    w.sounds.push(kq8_core::sound::Start::plain_one_shot(snd, "Connor's moving shot (ranged-weapons.md rule 20)".into()));
                }
                (c.moving_aim.take().unwrap_or(c.mover.position), format!("{m} at {phase:.4}, a moving shot (use sound {})", sound.as_deref().unwrap_or("none")))
            } else {
                (aim_now.unwrap_or(c.mover.position), format!("{m} at {phase:.4}, the phase after the update's step (Q140 c)"))
            };
            w.connor_shoot(c.mover.position, c.turn_matrix(), part, aim, &what);
        }
        for m in std::mem::take(&mut c.do_cues) {
            let (target, big) = c.swing.as_ref().map_or((None, false), |s| (s.target.clone(), s.big));
            let (from, heading, reach) = (c.mover.position, c.walker.heading, sim.attack_dist);
            sim.active.world.connor_do(from, heading, reach, target.as_deref(), big, &m);
        }
    }
    // ropeandhook.md rules 10, 12 (M6 6a-3): a world click with the rope in
    // hand: ignored in handsOff and while dying or dead; otherwise the
    // rope leaves the pointer and the rope state is asked for.
    for how in std::mem::take(&mut sim.rope_clicks) {
        let w = &mut sim.active.world;
        let line = if hands_off {
            format!("world click with the rope ({how}): ignored (handsOff; rule 10)")
        } else if w.connor_dying {
            format!("world click with the rope ({how}): ignored (Connor is dying or dead; rule 10)")
        } else {
            let rope = w.hand.take().unwrap_or_default();
            sim.rope_asked = Some(how.clone());
            format!("world click with the rope ({how}): {rope} leaves the pointer; the camera aim test passes always (\"Do not reproduce\"); the rope state is asked for (rules 10-12)")
        };
        sim.active.world.rope_note(line);
    }
    for (name, _) in due {
        let line = format!("update {updates}: test --click: clicks {name}");
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
        sim.pending.push((name, "test --click".into()));
    }
    for (name, how) in std::mem::take(&mut sim.pending) {
        match &connor {
            // handsOff (3k-3, Q89 a stand-in): clicks are ignored.
            Some(c) if c.hands_off => sim.log.detail(format!("update {updates}: click on {name} ({how}): ignored (handsOff)")),
            Some(c) => {
                let feet = c.mover.position;
                // 4c-4: the item in hand (inventory.md rule 12).
                let held = kq8_core::connor_inventory::hand(&mut sim.active.world).map(|(object, kind)| Item { object, kind });
                let turns = |r: &[Request]| r.iter().filter(|r| matches!(r, Request::ClickTurn { .. })).count();
                let before = turns(&sim.active.world.requests);
                let result = sim.active.click(&name, feet, sim.attack_dist, held.as_ref());
                // clicking.md rule 35 (Q105 b): his state at the click,
                // before his next update, for the turn it asked for.
                for _ in before..turns(&sim.active.world.requests) {
                    sim.click_states.push_back(c.click_turn_state());
                }
                if let Some(h) = &held {
                    let accepted = match &result {
                        kq8_core::active::ClickResult::Script(args) => args.get(2) == Some(&h.object),
                        kq8_core::active::ClickResult::Swallowed => true,
                        _ => false,
                    };
                    kq8_core::connor_inventory::after_world_click(&mut sim.active.world, &h.object, accepted);
                }
                sim.log.detail(format!("update {updates}: click on {name} ({how}) -> {result:?}"));
            }
            None => sim.log.detail(format!("update {updates}: click on {name} ({how}): no Connor in this world; nothing")),
        }
    }
    sim.active.update();
    if let Some(mut colliders) = colliders {
        // A collider out of the collision world (M1 7b, rule 41) stays out.
        let w = &sim.active.world;
        for (door, slot) in w.doors.iter().zip(&sim.collider) {
            if let (Some(i), Some(c), false) = (slot, door.collider(), w.is_switched_off(&door.door.name)) {
                colliders.0[*i] = c;
            }
        }
        for (chest, slot) in w.chests.iter().zip(&sim.chest_collider) {
            if let (Some(i), Some(c), false) = (slot, chest.collider(), w.is_switched_off(&chest.chest.name)) {
                colliders.0[*i] = c;
            }
        }
        for (object, slot) in w.objects.iter().zip(&sim.object_collider) {
            if object.gone {
                // Switched off by `trap_step` (3f).
                continue;
            }
            if let (Some(i), Some(c), false) = (slot, object.collider(), w.is_switched_off(object.name())) {
                colliders.0[*i] = c;
            }
        }
    }
    sim.flush_traps(&shared);
    sim.flush_talk(&shared);
    sim.flush_conversation(&shared);
    let item_lines = sim.flush_items(&shared);
    sim.flush_rope(&shared);
    let lines = sim.active.take_log();
    if !lines.is_empty() {
        let mut main = shared.log.lock().unwrap();
        for line in lines {
            // The main log gets the clicks, door events and finished
            // movements; item lines it already has (`flush_items`).
            let wanted = line.contains(": click ") || line.contains("KQDoorEvent") || line.contains("finished;");
            if wanted && !item_lines.contains(&line) {
                main.summary(format!("doors: {line}"));
            }
            sim.log.detail(line);
        }
    }
}

/// Hides the entities of deleted objects (3f), also from the H key.
pub fn hide_deleted(mut commands: Commands, mut deleted: ResMut<Deleted>, mut tagged: Query<(Entity, &crate::terrain_view::ObjectName, &mut Visibility)>) {
    if deleted.0.is_empty() {
        return;
    }
    let names = std::mem::take(&mut deleted.0);
    for (e, name, mut v) in &mut tagged {
        if names.iter().any(|n| n.eq_ignore_ascii_case(&name.0)) {
            *v = Visibility::Hidden;
            commands.entity(e).remove::<crate::terrain_view::ShownWithH>();
        }
    }
}

/// Puts `drawn` at `placement`, showing frame `frame` of movement
/// `current`.
fn show_frame(
    drawn: &mut DrawnDoor,
    placement: &Placement,
    current: usize,
    frame: usize,
    transforms: &mut Query<&mut Transform>,
    visibility: &mut Query<&mut Visibility>,
    (commands, gpu): (&mut Commands, Option<&crate::anim_gpu::AnimGpu>),
) {
    if let Ok(mut t) = transforms.get_mut(drawn.body) {
        *t = bevy_transform_from(placement.rotation, placement.position);
    }
    // Step 4: past the last frame, the last one shows.
    if let (Some(slots), Some(gpu)) = (drawn.slots.as_mut(), gpu) {
        let frames = drawn.frames.get(current).copied().unwrap_or(0);
        let frame = frame.min(frames.saturating_sub(1));
        if slots.stale(current, frame) {
            slots.show(commands, gpu, current, frame, visibility);
        }
    }
}

/// Each door and chest where it stands, showing the frame of its
/// movement; hidden chests are not drawn.
pub fn door_draw(
    mut commands: Commands,
    sim: Option<ResMut<DoorSim>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut carried: Query<&mut crate::scene_lights::CarriedLights>,
    gpu: Option<Res<crate::anim_gpu::AnimGpu>>,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    let gpu = gpu.as_deref();
    // The frame whose lights shine (M1 7c).
    let mut lit = |body: Entity, movement: usize, frame: usize| {
        if let Ok(mut c) = carried.get_mut(body) {
            if (c.movement, c.frame) != (movement, frame) {
                (c.movement, c.frame) = (movement, frame);
            }
        }
    };
    for (door, drawn) in sim.active.world.doors.iter().zip(sim.drawn.iter_mut()) {
        let d = &door.door;
        lit(drawn.body, d.player.current, d.frame());
        show_frame(drawn, &d.placement(), d.player.current, d.frame(), &mut transforms, &mut visibility, (&mut commands, gpu));
        // Deleted (3g-5): not drawn.
        if door.gone {
            if let Ok(mut v) = visibility.get_mut(drawn.body) {
                if *v != Visibility::Hidden {
                    *v = Visibility::Hidden;
                }
            }
        }
    }
    for (object, drawn) in sim.active.world.objects.iter().zip(sim.object_drawn.iter_mut()) {
        let p = &object.player;
        lit(drawn.body, p.current, p.frame());
        show_frame(drawn, &p.placement(), p.current, p.frame(), &mut transforms, &mut visibility, (&mut commands, gpu));
        let want = if object.hidden { Visibility::Hidden } else { Visibility::Inherited };
        if let Ok(mut v) = visibility.get_mut(drawn.body) {
            if *v != want {
                *v = want;
            }
        }
    }
    for (chest, drawn) in sim.active.world.chests.iter().zip(sim.chest_drawn.iter_mut()) {
        let c = &chest.chest;
        lit(drawn.body, c.player.current, c.frame());
        show_frame(drawn, &c.placement(), c.player.current, c.frame(), &mut transforms, &mut visibility, (&mut commands, gpu));
        let want = if chest.hidden { Visibility::Hidden } else { Visibility::Inherited };
        if let Ok(mut v) = visibility.get_mut(drawn.body) {
            if *v != want {
                *v = want;
            }
        }
    }
}
