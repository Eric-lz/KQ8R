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
//! `--item TYPE` puts an item of that type in Connor's hand. Everything is
//! logged to `kq8-artifacts/logs/doors.log`.
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
    item: Option<Item>,
    /// Objects clicked since the last update, with how (mouse, test).
    pending: Vec<(String, String)>,
    log: Log,
    /// `traps.log`; `None` if it could not be made.
    trap_log: Option<Log>,
    /// Where the traps last checked Connor.
    connor_checked_at: Option<[f32; 3]>,
    /// `items.log`; `None` if it could not be made.
    item_log: Option<Log>,
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
            })
            .collect();
        let animations: Vec<usize> = d.movements.iter().map(|(_, _, a)| *a).collect();
        let names = (d.anim_names.0.as_deref(), d.anim_names.1.as_deref());
        if d.object {
            // A scripted object (3e-5).
            let o = ActiveObject {
                player: NamedAnimPlayer::new(&p.name, movements, d.current, base).with_names_at_load(names.0, names.1),
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
    let item = item.map(|kind| Item { object: kind.clone(), kind });
    main_log.summary(format!(
        "doors: {} doors; Connor's reach 2 x attackDist {attack_dist} = {}; {} other clickable objects; {} scripts; item in hand: {:?}; log in {}",
        active_doors.len(),
        2.0 * attack_dist,
        objects.clickables.len(),
        sources.len(),
        item,
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
    for o in &active.world.items.objects {
        active.world.known.insert(o.name.to_ascii_lowercase());
    }
    if let Some(ground) = objects.ground.take() {
        use kq8_data::terrain::Level;
        active.world.heights = Some(Box::new(move |x, y, floor| ground.ground_at(if floor { Level::Floor } else { Level::Surface }, x, y).map(|g| g.height)));
    }
    if let Some(log) = &mut item_log {
        for line in std::mem::take(&mut objects.item_lines) {
            log.detail(line);
        }
        log.detail("items at the start:");
        let items = &active.world.items;
        for o in &items.objects {
            let click = items.click_box(o).map_or("NO click box".to_string(), |(a, b)| format!("click box {a:?}..{b:?}"));
            log.detail(format!("  {}; {click}", items.describe(&o.name)));
        }
    }
    // Rule 22: each door's script gets Init as the door enters the world;
    // then each trap's (traps.md rule 13).
    active.start();
    let mut sim = DoorSim { active, drawn, collider, attack_dist, item, pending: Vec::new(), log, trap_log, connor_checked_at: None, item_log, chest_drawn, chest_collider, object_drawn, object_collider };
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
    shared: Res<Shared>,
) {
    let (mut colliders, mut view, mut anim) = (colliders, view, anim);
    let (Some(mut sim), Some(mut connor)) = (sim, connor) else { return };
    let sim = &mut *sim;
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
    sim.flush_traps(&shared);
    sim.flush_items(&shared);
    let mut camera = camera;
    for r in std::mem::take(&mut sim.active.world.requests) {
        let line = match r {
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

/// Left click: the click line from the camera through the mouse picks the
/// nearest clickable box or clickable building (clicking.md rules 1-8;
/// the ground is not tested), handled at the next update. `--aim NAME,N`
/// does the same at update N with a line from the camera to the middle of
/// NAME's box (a test option; `--click` skips the line).
pub fn door_click_input(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    sim: Option<ResMut<DoorSim>>,
    buildings: Option<Res<Buildings>>,
    mut aim: ResMut<AimAt>,
    shared: Res<Shared>,
) {
    let Some(mut sim) = sim else { return };
    let updates = sim.active.world.updates;
    let due = aim.0.iter().position(|(_, n)| updates >= *n).map(|i| aim.0.remove(i).0);
    if due.is_none() && !buttons.just_pressed(MouseButton::Left) {
        return;
    }
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
    connor: Option<Res<ConnorSim>>,
    colliders: Option<ResMut<ObjectColliders>>,
    mut click_at: ResMut<ClickAt>,
    shared: Res<Shared>,
) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    let updates = sim.active.world.updates;
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut click_at.0).into_iter().partition(|(_, n)| updates >= *n);
    click_at.0 = later;
    for (name, _) in due {
        let line = format!("update {updates}: test --click: clicks {name}");
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
        sim.pending.push((name, "test --click".into()));
    }
    for (name, how) in std::mem::take(&mut sim.pending) {
        match &connor {
            Some(c) => {
                let feet = c.mover.position;
                let result = sim.active.click(&name, feet, sim.attack_dist, sim.item.as_ref());
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
