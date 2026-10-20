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
use crate::{model_mesh, Shared, UNITS_PER_METRE};

/// Connor's `attackDist` when `Connor.kq` gives none: the built-in
/// default for characters (doors.md rule 11).
const DEFAULT_ATTACK_DIST: f32 = 1250.0;

/// One frame of an animation, ready to draw: (mesh, material) per
/// material group.
type BuiltFrame = Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>;

/// One door's entities: the body (turned) and, per movement, one entity
/// per frame (only the shown one visible).
struct DrawnDoor {
    body: Entity,
    frames: Vec<Vec<Entity>>,
    shown: Option<Entity>,
}

#[derive(Resource)]
pub struct DoorSim {
    pub active: Active,
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
    /// Connor's state at each click whose turn is waiting (M4 4c-5,
    /// `clicking.md` rule 35): whether it allows the turn, its name.
    click_states: std::collections::VecDeque<(bool, &'static str)>,
    log: Log,
    /// `traps.log`; `None` if it could not be made.
    trap_log: Option<Log>,
    /// Where the traps last checked Connor.
    connor_checked_at: Option<[f32; 3]>,
    /// `items.log`; `None` if it could not be made.
    item_log: Option<Log>,
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
        // A character brought in gets its draw-distance test now (M4
        // 4d-3), so it updates while near or far (characters.md rule 11).
        if let Some((body, test)) = anim.as_mut().and_then(|a| a.take_test(&name)) {
            commands.entity(body).insert(test);
        }
        let sounding = sound.as_mut().is_some_and(|s| s.wake(&name));
        let line = format!(
            "update {}: loadKQ: {name} came in: {shown} drawn part(s) shown{}{} (not clickable, no collider: prepared copies have none; not done)",
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

/// `--item TYPE`.
#[derive(Resource)]
pub struct HeldItem(pub Option<String>);

/// A body entity at `placement` with, per movement (index into `built`),
/// one hidden entity per frame. `haze`: the object's `b_hazeSolid` and
/// `hazeMultiplier` (object-lighting.md rules 21, 22). `test`: its draw
/// distance (M1 6h, rules 34-37).
#[allow(clippy::too_many_arguments)]
fn spawn_body(
    commands: &mut Commands,
    placement: &Placement,
    animations: &[usize],
    built: &[Vec<BuiltFrame>],
    materials: &mut Assets<ObjectMaterial>,
    haze: (bool, f32),
    test: crate::draw_distance::DistanceTest,
    lights: &[std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>],
    dyn_off: bool,
) -> DrawnDoor {
    let transform = bevy_transform_from(placement.rotation, placement.position);
    let body = commands.spawn((transform, Visibility::default(), test)).id();
    // The lights it carries (M1 7c), by movement and frame.
    if let Some(c) = crate::scene_lights::CarriedLights::new(animations.iter().map(|a| lights[*a].clone()).collect(), transform, dyn_off) {
        commands.entity(body).insert(c);
    }
    let mut frames = Vec::new();
    for a in animations {
        let mut entities = Vec::new();
        for frame in &built[*a] {
            let parent = commands.spawn((Transform::default(), Visibility::Hidden)).id();
            commands.entity(body).add_child(parent);
            for (mesh, material) in frame {
                let material = crate::object_light::for_object(materials, material, 1.0, haze.0, haze.1);
                let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material))).id();
                commands.entity(parent).add_child(part);
            }
            entities.push(parent);
        }
        frames.push(entities);
    }
    DrawnDoor { body, frames, shown: None }
}

/// Builds the doors from the loaded objects: their entities, their game
/// state, and their colliders (added to `colliders`).
#[allow(clippy::too_many_arguments)]
pub fn spawn_doors(
    commands: &mut Commands,
    objects: &mut Objects,
    colliders: &mut Vec<BoxCollider>,
    item: Option<String>,
    anims: Vec<kq8_core::active::AnimObject>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    balls: &crate::draw_distance::ShapeBalls,
    main_log: &mut Log,
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
    let built: Vec<Vec<BuiltFrame>> = std::mem::take(&mut objects.door_animations)
        .into_iter()
        .map(|anim| {
            let looks: Vec<(i32, Handle<ObjectMaterial>, crate::load::Shading)> =
                anim.looks.into_iter().map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)), l)).collect();
            anim.frames
                .into_iter()
                .map(|frame| {
                    frame
                        .into_iter()
                        .filter_map(|(m, triangles)| {
                            let (_, material, shading) = looks.iter().find(|(i, _, _)| *i == m)?;
                            Some((meshes.add(model_mesh(&triangles, shading.lighting, [0.0; 3])), material.clone()))
                        })
                        .collect()
                })
                .collect()
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
            object_drawn.push(spawn_body(commands, &o.player.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting));
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
            chest_drawn.push(spawn_body(commands, &c.chest.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting));
            active_chests.push(c);
            continue;
        }
        let mut door = Door::new(&p.name, movements, d.current, base);
        door.player = door.player.with_names_at_load(names.0, names.1);
        let DrawnDoor { body, frames, .. } =
            spawn_body(commands, &door.placement(), &animations, &built, materials, (p.haze_solid, p.haze_multiplier), balls.test_shape(p, &d.shape), &anim_lights, p.turn_off_dyn_lighting);
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
        drawn.push(DrawnDoor { body, frames, shown: None });
        active_doors.push(active);
    }
    let test_item = item;
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
    let traps: Vec<Trap> = std::mem::take(&mut objects.traps).into_iter().map(Trap::new).collect();
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
    // The other animated objects (3i-4), drawn by anim_view.
    active.add_anims(anims);
    // Copies prepared for a script's loadKQ (3k-4).
    active.set_dormant(std::mem::take(&mut objects.dormant));
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
    let monsters: Vec<String> = active.world.monsters.iter().map(|m| format!("{} ({})", m.name, m.script.as_deref().unwrap_or("no script"))).collect();
    main_log.summary(format!("talk: {} KQMonsters: {}; captions {:?}", monsters.len(), monsters.join(", "), active.scripts.vars.get("Options::Captions")));
    // M4 4e-3: the conversation info objects (conversation.md rule 3).
    active.world.conv_infos = std::mem::take(&mut objects.conv_infos);
    main_log.summary(format!("talk: {} conversation info object(s): {:?}", active.world.conv_infos.len(), active.world.conv_infos.iter().map(|i| i.name.as_str()).collect::<Vec<_>>()));
    // Rule 22: each door's script gets Init as the door enters the world;
    // then each trap's (traps.md rule 13).
    active.start();
    let mut sim = DoorSim { active, drawn, collider, attack_dist, pending: Vec::new(), world_clicks: Vec::new(), click_states: Default::default(), log, trap_log, connor_checked_at: None, item_log, talk_log, conversation_log, chest_drawn, chest_collider, object_drawn, object_collider };
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
    sim.active.world.connor_at = Some(connor.mover.position);
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
    sim.flush_talk(&shared);
    let mut camera = camera;
    let requests: Vec<Request> = std::mem::take(&mut connor.refused).into_iter().chain(std::mem::take(&mut sim.active.world.requests)).collect();
    // clicking.md rule 35 (Q105 b): a state a script asked for (and got)
    // before a click's turn decides it; otherwise his state at the click.
    let mut asked = false;
    for (i, r) in requests.into_iter().enumerate() {
        let again = if i < retried { " (tried again)" } else { "" };
        // anim-scripts.md rule 28: during an unfinished jump a state
        // request finishes it first (handsOff on) or is refused.
        let state = match &r {
            Request::ConnorSpecial { at, .. } => Some(("setState special", at.clone())),
            Request::ConnorAuto { auto, at } if matches!(auto.kind, kq8_core::connor_auto::AutoKind::Walk { .. }) => Some(("scripted walk (Wait)", at.clone())),
            _ => None,
        };
        if let Some((what, at)) = state {
            if !connor.state_request(what, &at) {
                let line = format!("update {}: {at}: {what}{again}: refused (Connor's jump is not finished); tried again next update", sim.active.world.updates);
                if let Some(log) = &mut sim.trap_log {
                    log.detail(line);
                }
                connor.refused.push(r);
                continue;
            }
            asked = true;
        }
        let line = match r {
            Request::EndJump { at } => format!("{at}: applied: {}", connor.end_jump(&at)),
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
    if let Some(name) = picked {
        sim.pending.push((name, "mouse".into()));
    }
    shared.log.lock().unwrap().summary(&line);
    sim.log.detail(line);
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
    mut command_at: ResMut<CommandAt>,
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
) {
    if let Ok(mut t) = transforms.get_mut(drawn.body) {
        *t = bevy_transform_from(placement.rotation, placement.position);
    }
    let entity = drawn.frames.get(current).and_then(|f| f.get(frame).or(f.last())).copied();
    if entity != drawn.shown {
        if let Some(mut old) = drawn.shown.and_then(|e| visibility.get_mut(e).ok()) {
            *old = Visibility::Hidden;
        }
        if let Some(mut new) = entity.and_then(|e| visibility.get_mut(e).ok()) {
            *new = Visibility::Inherited;
        }
        drawn.shown = entity;
    }
}

/// Each door and chest where it stands, showing the frame of its
/// movement; hidden chests are not drawn.
pub fn door_draw(
    sim: Option<ResMut<DoorSim>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut carried: Query<&mut crate::scene_lights::CarriedLights>,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
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
        show_frame(drawn, &d.placement(), d.player.current, d.frame(), &mut transforms, &mut visibility);
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
        show_frame(drawn, &p.placement(), p.current, p.frame(), &mut transforms, &mut visibility);
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
        show_frame(drawn, &c.placement(), c.player.current, c.frame(), &mut transforms, &mut visibility);
        let want = if chest.hidden { Visibility::Hidden } else { Visibility::Inherited };
        if let Ok(mut v) = visibility.get_mut(drawn.body) {
            if *v != want {
                *v = want;
            }
        }
    }
}
