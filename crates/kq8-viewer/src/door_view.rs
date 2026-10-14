//! Doors in the ground view (M3 step 3a, docs/DESIGN.md; rules:
//! docs/behaviour/doors.md).
//!
//! - The doors, their scripts and the event queue live in
//!   `kq8_game::active` and run at 15 updates per second, before Connor's
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

use bevy::image::ImageAddressMode;
use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use kq8_game::active::{Active, ActiveDoor, Item, Request};
use kq8_game::trap::{Trap, Visitor};
use kq8_game::collide::{BoxCollider, Placement};
use kq8_game::door::{turn_degrees, Cue, Door, DoorMovement};

use crate::connor_view::ConnorSim;
use crate::game_camera::GameCam;
use crate::load::Objects;
use crate::log::Log;
use crate::terrain_view::{bevy_transform_from, ObjectColliders};
use crate::{material_for, mesh_from, to_bevy, Shared, UNITS_PER_METRE};

/// Connor's `attackDist` when `Connor.kq` gives none: the built-in
/// default for characters (doors.md rule 11).
const DEFAULT_ATTACK_DIST: f32 = 1250.0;

/// One frame of an animation, ready to draw: (mesh, material) per
/// material group.
type BuiltFrame = Vec<(Handle<Mesh>, Handle<StandardMaterial>)>;

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

/// `--item TYPE`.
#[derive(Resource)]
pub struct HeldItem(pub Option<String>);

/// Builds the doors from the loaded objects: their entities, their game
/// state, and their colliders (added to `colliders`).
#[allow(clippy::too_many_arguments)]
pub fn spawn_doors(
    commands: &mut Commands,
    objects: &mut Objects,
    colliders: &mut Vec<BoxCollider>,
    item: Option<String>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<StandardMaterial>,
    images: &mut Assets<Image>,
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
    // Each animation once: per frame, (mesh, material) per material group.
    let built: Vec<Vec<BuiltFrame>> = std::mem::take(&mut objects.door_animations)
        .into_iter()
        .map(|anim| {
            let looks: Vec<(i32, Handle<StandardMaterial>)> =
                anim.looks.into_iter().map(|(m, look)| (m, materials.add(material_for(look, ImageAddressMode::Repeat, images)))).collect();
            anim.frames
                .into_iter()
                .map(|frame| {
                    frame
                        .into_iter()
                        .filter_map(|(m, triangles)| {
                            let (_, material) = looks.iter().find(|(i, _)| *i == m)?;
                            let mut positions = Vec::with_capacity(triangles.len() * 3);
                            let mut uvs = Vec::with_capacity(triangles.len() * 3);
                            for t in &triangles {
                                // Clockwise front (the game) -> counter-clockwise (Bevy).
                                for k in [0, 2, 1] {
                                    positions.push(to_bevy(t.positions[k], [0.0; 3]));
                                    uvs.push(t.uvs[k]);
                                }
                            }
                            Some((meshes.add(mesh_from(positions, uvs)), material.clone()))
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
                play_time: timing[*a].0,
                frame_steps: timing[*a].1.clone(),
                cues: l.cues.iter().map(|(phase, action)| Cue { phase: *phase, action: action.clone() }).collect(),
            })
            .collect();
        let door = Door::new(&p.name, movements, d.current, base);
        let body = commands.spawn((bevy_transform_from(door.placement().rotation, door.placement().position), Visibility::default())).id();
        let mut frames = Vec::new();
        for (_, _, a) in &d.movements {
            let mut entities = Vec::new();
            for frame in &built[*a] {
                let parent = commands.spawn((Transform::default(), Visibility::Hidden)).id();
                commands.entity(body).add_child(parent);
                for (mesh, material) in frame {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(parent).add_child(part);
                }
                entities.push(parent);
            }
            frames.push(entities);
        }
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
        };
        // The door's script, read through the block counter (DESIGN 3a-1).
        let balance = d.script.as_ref().map(|name| {
            match sources.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)) {
                Some((_, bytes)) => {
                    let text: String = bytes.iter().map(|&b| b as char).collect();
                    let (ifs, endifs, problems) = kq8_game::script::block_balance(&text);
                    format!("{name}: {} lines, {ifs} if / {endifs} endif{}", text.lines().count(), if problems.is_empty() { ", balanced".to_string() } else { format!(", PROBLEMS {problems:?}") })
                }
                None => format!("{name}: NOT FOUND in the world's scripts"),
            }
        });
        let open = active.door.movements.iter().find(|m| m.name.eq_ignore_ascii_case("open"));
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
                .movements
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
    let mut sim = DoorSim { active, drawn, collider, attack_dist, item, pending: Vec::new(), log, trap_log, connor_checked_at: None, item_log };
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

/// After Connor's update (M3 step 3d-2): if he moved, every trap checks
/// him (traps.md rules 6-9) and runs its script. Then what the scripts
/// asked of Connor and the camera this update is applied (3d-3), before
/// the camera's update.
pub fn trap_step(sim: Option<ResMut<DoorSim>>, connor: Option<ResMut<ConnorSim>>, camera: Option<ResMut<GameCam>>, shared: Res<Shared>) {
    let (Some(mut sim), Some(mut connor)) = (sim, connor) else { return };
    let sim = &mut *sim;
    let position = connor.mover.position;
    if sim.connor_checked_at != Some(position) {
        // His move since the last check (traps.md rule 6: swept).
        let from = sim.connor_checked_at.replace(position).unwrap_or(position);
        let (min, max) = kq8_game::mover::CONNOR_BOX;
        let v = Visitor {
            name: &connor.placement.name,
            collision_type: kq8_game::collide::CONNOR_TYPE,
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
        };
        let line = format!("update {}: {line}", sim.active.world.updates);
        shared.log.lock().unwrap().summary(format!("traps: {line}"));
        if let Some(log) = &mut sim.trap_log {
            log.detail(line);
        }
    }
}

/// Left click: the nearest clickable box under the mouse (rule 10), handled
/// at the next update.
pub fn door_click_input(
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<PrimaryWindow>>,
    camera: Query<(&Camera, &GlobalTransform), With<Camera3d>>,
    sim: Option<ResMut<DoorSim>>,
    shared: Res<Shared>,
) {
    let Some(mut sim) = sim else { return };
    if !buttons.just_pressed(MouseButton::Left) {
        return;
    }
    let (Ok(window), Ok((camera, at))) = (window.single(), camera.single()) else { return };
    let Some(cursor) = window.cursor_position() else { return };
    let Ok(ray) = camera.viewport_to_world(at, cursor) else { return };
    // Bevy (x, y, z) = game (x, z, -y) / 100, so back: (x, -z, y) x 100.
    let o = ray.origin * UNITS_PER_METRE;
    let d = *ray.direction;
    let origin = [o.x, -o.z, o.y];
    let dir = [d.x, -d.z, d.y];
    let updates = sim.active.world.updates;
    let line = match sim.active.pick(origin, dir) {
        Some((name, t)) => {
            let line = format!(
                "update {updates}: mouse click at ({:.0}, {:.0}) px: ray from ({:.0}, {:.0}, {:.0}) picks {name} at {t:.0} units",
                cursor.x, cursor.y, origin[0], origin[1], origin[2]
            );
            sim.pending.push((name, "mouse".into()));
            line
        }
        None => format!(
            "update {updates}: mouse click at ({:.0}, {:.0}) px: ray from ({:.0}, {:.0}, {:.0}) meets no clickable object",
            cursor.x, cursor.y, origin[0], origin[1], origin[2]
        ),
    };
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
        for (door, slot) in sim.active.world.doors.iter().zip(&sim.collider) {
            if let (Some(i), Some(c)) = (slot, door.collider()) {
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

/// Each door where it stands, showing the frame of its movement.
pub fn door_draw(sim: Option<ResMut<DoorSim>>, mut transforms: Query<&mut Transform>, mut visibility: Query<&mut Visibility>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    for (door, drawn) in sim.active.world.doors.iter().zip(sim.drawn.iter_mut()) {
        let d = &door.door;
        let placement = d.placement();
        if let Ok(mut t) = transforms.get_mut(drawn.body) {
            *t = bevy_transform_from(placement.rotation, placement.position);
        }
        let frame = drawn.frames.get(d.current).and_then(|f| f.get(d.frame()).or(f.last())).copied();
        if frame != drawn.shown {
            if let Some(old) = drawn.shown.and_then(|e| visibility.get_mut(e).ok()) {
                let mut old = old;
                *old = Visibility::Hidden;
            }
            if let Some(mut new) = frame.and_then(|e| visibility.get_mut(e).ok()) {
                *new = Visibility::Inherited;
            }
            drawn.shown = frame;
        }
    }
}
