//! Connor in the ground view (M2 steps 2a, 2b, 2c and the falls of 2e,
//! docs/DESIGN.md).
//!
//! - His mover (`kq8_core::connor_mover`, docs/behaviour/connor-physics.md) and his
//!   walking (`kq8_core::connor`, docs/behaviour/connor-movement.md) run at
//!   a fixed 15 updates per second, gravity on.
//! - Keys from `MASK/Connor.sae`; the intent comes from the keys held (the
//!   note's recommendation), so releasing one of two opposite keys gives the
//!   other back.
//! - Every frame of his current movement is drawn in turn; position,
//!   heading, lean and loop position are blended between the last two
//!   updates (display only).
//! - Banking (M2 step 2i, connor-movement.md "Turning" rule 3): the
//!   walker's lean is turned in before his heading, about his forward axis
//!   through his feet (pivot: stand-in, Q56 b). Drawing only.
//!
//! - Falling: `jumphold` while the mover is in Fall, then `jumpend` or
//!   `jumpdie` once (connor-physics.md "Falling off an edge").
//! - Walls: walking head-on stops (`stop`), running head-on plays `crash`
//!   once (connor-physics.md "Walls"; M2 steps 2d-3, 2d-3b, 2d-4).
//!
//! Test keys: G lifts him 5,000 (should die on landing), B lifts him 2,000
//! (safe). `--at X,Y,H` starts him somewhere else. The camera that follows
//! him is in `game_camera.rs`.

use kq8_core::weapons::WeaponMode;
use bevy::prelude::*;
use kq8_data::objects::Placement;
use kq8_core::connor::{Intent, Mode, MoveKey, MovementLoop, Walker};
use kq8_core::connor_mover::{Mover, MoverWorld, Scene as MoverScene, State};

use crate::load::ConnorAssets;
use crate::log::Log;
use crate::terrain_view::{bevy_transform, bevy_transform_from, Buildings, GroundLookup, ObjectColliders};
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::{model_mesh, Shared};

/// One drawable movement: its loop for the walker and one entity per frame.
pub struct DrawnMovement {
    pub name: &'static str,
    pub movement_loop: MovementLoop,
    pub frames: Vec<Entity>,
    /// Each frame's named model parts and their places (the `arrow` part
    /// his bolt starts at, `ranged-weapons.md` rule 24; M5 5f-5).
    pub parts: Vec<Vec<(String, [f32; 3])>>,
}

/// Keys from `Connor.sae`, and which of them are held, oldest first.
pub struct Controls {
    forward: Option<KeyCode>,
    back: Option<KeyCode>,
    left: Option<KeyCode>,
    right: Option<KeyCode>,
    run: Option<KeyCode>,
    jump: Option<KeyCode>,
    /// `Draw` (Delete), `Away` (Alt), `ModifyDn` (Shift)
    /// (`ranged-weapons.md` rule 1; M5 5f-3).
    draw: Option<KeyCode>,
    away: Option<KeyCode>,
    shift: Option<KeyCode>,
    held: Vec<KeyCode>,
}

impl Controls {
    /// The value of the pair (`plus` gives +1, `minus` −1) from the keys
    /// held: the one pressed last wins.
    fn axis(&self, plus: Option<KeyCode>, minus: Option<KeyCode>) -> i8 {
        for k in self.held.iter().rev() {
            if Some(*k) == plus {
                return 1;
            }
            if Some(*k) == minus {
                return -1;
            }
        }
        0
    }
}

/// Connor's mover, walker, drawing and logs.
#[derive(Resource)]
pub struct ConnorSim {
    /// His mouth (M4 4a-6, `talking.md` rule 18): the materials that show
    /// the face texture `baseLip`, and the image of each mouth shape
    /// 0-7 (`None`: not found, the face stays).
    pub face: Vec<Handle<ObjectMaterial>>,
    pub lip_images: Vec<Option<Handle<Image>>>,
    /// What the potions change on him (M5 5g-5).
    pub looks: crate::potion_view::ConnorLooks,
    /// The mouth shape shown now (0 = rest).
    pub mouth: u16,
    /// Presses that cut his line short (`talking.md` rule 16, M4 4a-5):
    /// forward, back, a turn key or Jump, not during `handsOff`; taken
    /// by the door simulation.
    pub cut_presses: Vec<String>,
    pub mover: Mover,
    pub walker: Walker,
    pub intent: Intent,
    pub controls: Controls,
    pub movements: Vec<DrawnMovement>,
    /// Placement (name, angles) used to build his transform; angle Z is
    /// the walker's heading.
    pub placement: Placement,
    /// Position, heading, loop position and lean after the last two updates.
    previous: ([f32; 3], f32, f32, f32),
    current: ([f32; 3], f32, f32, f32),
    /// Share of the loop played per update (for blending the loop position).
    phase_step: f32,
    pub updates: u64,
    /// The Jump key was pressed since the last update.
    jump_pressed: bool,
    /// Height where the last jump started (for the log).
    jump_start_z: Option<f32>,
    /// The turn keys as events (M4 4a-7, `kq8_core::turn_keys`): the
    /// turn direction changes only on a key going down or up.
    pub turn_keys: kq8_core::turn_keys::TurnKeys,
    /// `--hold`'s turn key held now: its direction, 0 for none.
    hold_turn_down: i8,
    /// `--hold`'s run switch has been set (once, at the start).
    hold_run_set: bool,
    /// Keyboard auto-repeat: the movement key pressed last, while it is
    /// still held (its key, `None` for the `--turn` test). Each update it
    /// counts as pressed again (user's check in the original, 2026-10-12:
    /// a held turn key keeps clearing the wall blocks).
    repeat: Option<(Option<KeyCode>, MoveKey)>,
    /// One line per update: `kq8-artifacts/logs/connor.log`.
    pub log: Log,
    last_state: &'static str,
    /// What the last down-pass contact was (`terrain`, a building, or
    /// `none`), to log only when it changes.
    last_down: String,
    /// The frame entity shown now (its children are the model parts; the
    /// shadow reads them, M4 4d-7b-2).
    pub(crate) shown: Option<Entity>,
    /// Sounds his cues started (M3 3h-4), taken by `sound_view`.
    pub sounds: Vec<kq8_core::sound::Start>,
    /// Picks the footstep file (`sound.md` rule 38).
    dice: kq8_core::sound::Dice,
    /// The building his steps sound like (`sound.md` rule 39a).
    step_ground: kq8_core::sound::StepGround,
    /// His play-once movements that ended (plain names), for `getEndLoop`
    /// listeners; taken by `door_view::trap_step` (M3 3i-7).
    pub ended: Vec<String>,
    /// Physical pushables he moved this update (`pushing.md` rule 4):
    /// name and movement; applied by [`push_apply`] (M3 3j-2).
    pub pushed: Vec<(String, [f32; 3])>,
    /// Scripted pushables he pushed this update, whose scripts get `Push`
    /// (`door_view::trap_step`, M3 3j-3).
    pub push_calls: Vec<String>,
    /// Scripted pushables whose script didn't take control: walls for him
    /// from then on (`pushing.md` "Do not reproduce").
    pub not_pushable: Vec<String>,
    /// `handsOff 1` (M3 3k-3, Q89 a stand-in): movement keys and Jump
    /// count as let go.
    pub hands_off: bool,
    /// Scripted turns and walks that ended (M3 3l-2), whose cues
    /// `door_view::trap_step` sends.
    pub auto_ends: Vec<kq8_core::connor_auto::AutoEnd>,
    /// State requests a jump refused (M3 3m-1, `anim-scripts.md` rule
    /// 28): tried again at each later update by `door_view::trap_step`
    /// (STAND-IN: when the original tries again was not traced).
    pub refused: Vec<kq8_core::active::Request>,
    /// The end of a scripted walk being watched (M3 3n-5, Q98: logs only).
    walk_end: Option<WalkEnd>,
    /// `fighting.md` rules 23, 24, 29 (M5 5c-4): when his last flinch ends
    /// (game time) and when his ouch sound last played.
    flinch_end: f64,
    last_ouch: f64,
    /// The swing playing or waiting for his turn (M5 5c-5, `fighting.md`
    /// rules 39-41).
    pub swing: Option<Swing>,
    /// His swing passed its `Do` cue this update: taken by `door_view`.
    pub do_cues: Vec<String>,
    /// The shot asked or playing (`ranged-weapons.md` rule 20; M5 5f-5).
    pub shot: Option<Shot>,
    /// His swing's target, kept until another pick within reach replaces
    /// it (`ranged-weapons.md` rule 32; M5 5f-7).
    pub melee_target: Option<String>,
    /// His shot's `do` cue passed this update, or a moving shot left
    /// (movement, the phase of the frame, moving): taken by `door_view`.
    pub shot_cues: Vec<(&'static str, f32, bool)>,
    /// `ranged-weapons.md` rule 20 (M5 5f-6): when the next moving shot
    /// may leave (seconds of his updates), and when the run comes back on
    /// after a moving shot made him walk.
    next_moving_shot: f64,
    run_back_at: Option<f64>,
    /// A moving shot's aim point, for `door_view`.
    pub moving_aim: Option<[f32; 3]>,
    /// The hand weapon's movement list in use (`confists.anm`), while one
    /// is ready (M5 5c fix, **stand-in**, Q138): his movements come from
    /// it where it has them.
    pub weapon_set: Option<String>,
    /// The weapon list he is readying or has ready (switched at the click;
    /// `weapon_set` follows when the draw part starts), the switch still
    /// to come (set, list item it waits for), and whether a draw or
    /// put-away is playing (no slot clicks meanwhile: the user's check,
    /// 5d fix 10).
    pub weapon_target: Option<String>,
    pending_set: Option<(Option<String>, WeaponMode, usize)>,
    readying: bool,
    /// The mode and item of `weapon_target`, and the mode of `weapon_set`,
    /// for the pointer (`ranged-weapons.md` rules 6, 11; M5 5f-3).
    target_mode: WeaponMode,
    ready_item: Option<String>,
    pub set_mode: WeaponMode,
    /// Delete / Alt pressed since the last update: taken by `door_view`
    /// (rule 1).
    pub weapon_keys: Vec<(kq8_core::weapons::Ask, String)>,
    /// The big-swing key (Shift) is held (rule 6's `away2`).
    pub shift_held: bool,
    /// A script's walk asked while a script's swap starts: it waits for
    /// the swap's end (Q141, the user's check).
    held_walk: Option<kq8_core::connor_auto::AutoMove>,
    /// The rope state (M6 6a-3, `ropeandhook.md` rules 12-26), run by
    /// `rope_view::rope_step`.
    pub rope: Option<kq8_core::rope::Climb>,
    /// The separate rope model playing (rule 15): its movement and phase
    /// now and after the last update (for drawing), shown frame.
    pub rope_model: Option<(&'static str, f32, f32)>,
    rope_shown: Option<Entity>,
    /// His rope-state movements that ended this update, for `rope_step`.
    pub rope_ended: Vec<&'static str>,
    /// He climbed last update (for the log's "frozen" line, rule 22).
    pub rope_was_climbing: bool,
    /// The hammer-throw state (`azriel-hammer.md` rules 2-4; M6 6b-3).
    pub throw: Option<Throw>,
    /// His throw's `do` cue passed this update (movement, the cue's
    /// phase): the hammer leaves, taken by `door_view`.
    pub throw_cues: Vec<(&'static str, f32)>,
    /// His throw-state movements that ended this update.
    throw_ended: Vec<&'static str>,
    /// The weapon sets not loaded at start-up (M6 6c-2, `late_set_step`).
    pub late: Option<crate::load::LateSets>,
}

/// Connor's `attackDelay` (`ranged-weapons.md` rule 20: the default 0.5 s;
/// `Connor.kq` gives none): moving shots at most this often, seconds.
pub const MOVING_SHOT_DELAY: f64 = 0.5;

/// Connor's shot (`ranged-weapons.md` rule 20; M5 5f-5).
#[derive(Debug, Clone)]
pub struct Shot {
    /// The ranged list's `attack` (`conscbow.anm/attack`).
    pub movement: &'static str,
    /// The aim point (after the steepness limit).
    pub aim: [f32; 3],
    /// Waiting for the turn (more than 45° off at the click).
    pub waiting: bool,
    /// `attack` started.
    pub started: bool,
}

/// Connor's throw of Azriel's Hammer (`azriel-hammer.md` rules 2-4; M6
/// 6b-3): the hammer list's movements, the aim point, where it stands.
#[derive(Debug, Clone)]
pub struct Throw {
    pub attack: &'static str,
    pub wait: &'static str,
    pub catch: &'static str,
    pub aim: [f32; 3],
    /// The hammer came back (rule 9): `catch` plays next.
    pub back: bool,
    /// His update at the click, for the log's times.
    pub from_update: u64,
}

/// One of Connor's swings (M5 5c-5).
#[derive(Debug, Clone)]
pub struct Swing {
    /// The movement as loaded (`confists.anm/attack`).
    pub movement: &'static str,
    /// His target at the swing (`ranged-weapons.md` rule 32: kept from
    /// the last pick within reach), if any.
    pub target: Option<String>,
    pub big: bool,
    /// It waits until his turn toward `point` has 8 updates or fewer to
    /// go (rule 32: the picked thing was more than 45° off).
    pub waiting: bool,
    pub point: [f32; 2],
}

/// `ranged-weapons.md` rule 32: a waiting swing starts when the turn has
/// this many updates or fewer to go.
pub const SWING_TURN_UPDATES: f32 = 8.0;

/// Q98 (M3 3n-5, logs only): from the update a scripted walk ends until
/// its stop transition has ended, what he did.
struct WalkEnd {
    update: u64,
    at: [f32; 3],
    heading: f32,
    /// The stop transition seen playing, and the updates it played in.
    via: Option<&'static str>,
    first: u64,
    last: u64,
    /// Flat distance along his path since the walk ended, and while the
    /// transition played.
    path: f32,
    via_path: f32,
    via_from: [f32; 3],
    previous: [f32; 3],
}

/// Each building's `footsoundType` by lower-case name (`sound.md` rule 39).
#[derive(Resource, Default)]
pub struct FootSounds(pub std::collections::BTreeMap<String, Option<i64>>);

/// A movement by the name the walker knows it by: with a hand weapon
/// ready (`set`, its list file) the weapon's own version first
/// (`confists.anm/stop` for `stop`), else his own (M5 5c fix,
/// **stand-in**, Q138).
fn find<'a>(movements: &'a [DrawnMovement], set: Option<&str>, name: &str) -> Option<&'a DrawnMovement> {
    set.and_then(|f| {
        let wanted = kq8_core::connor::special_movement_name(f, name);
        movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted))
    })
    .or_else(|| movements.iter().find(|m| m.name == name))
}

impl ConnorSim {
    fn movement_loop(&self, name: &str) -> Option<&MovementLoop> {
        self.drawn(name).map(|m| &m.movement_loop)
    }

    /// See [`find`].
    fn drawn(&self, name: &str) -> Option<&DrawnMovement> {
        find(&self.movements, self.weapon_set.as_deref(), name).or_else(|| self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(name)))
    }
}

#[derive(Component)]
pub struct ConnorBody;

/// `--lift N`: lift Connor this much once at the start (a test, like G/B).
#[derive(Resource)]
pub struct StartLift(pub Option<f32>);

/// `--hold F,T,R[,N]`: a test that holds this intent (forward, turn, run
/// 0/1) instead of the keys; with N, only for the first N updates, then
/// forward and turn are let go (run stays as it was).
#[derive(Resource)]
pub struct HoldIntent(pub Vec<([i8; 3], u64, Option<u64>)>);

/// `--jump N` (repeatable): a test that presses Jump once, before update
/// N + 1, for each N given.
#[derive(Resource)]
pub struct JumpAt(pub Vec<u64>);

/// `--turn N[,T]`: a test that presses a turn key (T = 1 left, the
/// default, or −1 right) before update N + 1 and holds it from then on
/// (with `--hold`, on top of its intent).
#[derive(Resource)]
pub struct TurnAt(pub Option<(u64, i8)>);

/// `--at X,Y,H`: a test that starts Connor on the ground at X, Y facing
/// heading H (radians) instead of where the world script puts him.
#[derive(Resource)]
pub struct StartAt(pub Option<[f32; 3]>);

/// Bevy key for a key name in a `.sae` file. Only the names Connor's
/// movement uses so far, plus letters and function keys.
pub(crate) fn key_code(name: &str) -> Option<KeyCode> {
    Some(match name {
        "up" => KeyCode::ArrowUp,
        "down" => KeyCode::ArrowDown,
        "left" => KeyCode::ArrowLeft,
        "right" => KeyCode::ArrowRight,
        "end" => KeyCode::End,
        "home" => KeyCode::Home,
        "insert" => KeyCode::Insert,
        "delete" => KeyCode::Delete,
        "pageup" | "prior" => KeyCode::PageUp,
        "pagedown" | "next" => KeyCode::PageDown,
        "f2" => KeyCode::F2,
        "f11" => KeyCode::F11,
        "shift" => KeyCode::ShiftLeft,
        "control" => KeyCode::ControlLeft,
        "alt" => KeyCode::AltLeft,
        _ => return None,
    })
}

/// An animation's loop data (the movements add speed, flags, transitions).
fn animation_loop(a: &crate::load::Animation) -> MovementLoop {
    MovementLoop { duration: a.duration, step_sum: a.step_sum, frames: a.frames.len(), frame_steps: a.frame_steps.clone(), ..Default::default() }
}

/// A movement drawn with its animation's loop, frames and parts.
#[allow(clippy::type_complexity)]
fn drawn_movement(m: &crate::load::ConnorMovement, loops: &[MovementLoop], frames: &[Vec<Entity>], parts: &[Vec<Vec<(String, [f32; 3])>>]) -> DrawnMovement {
    DrawnMovement {
        name: m.name,
        movement_loop: MovementLoop {
            speed: m.speed,
            repeat: m.repeat,
            cut_short: m.cut_short,
            transitions: m.transitions.clone(),
            cues: m.cues.clone(),
            ..loops[m.animation].clone()
        },
        frames: frames[m.animation].clone(),
        parts: parts[m.animation].clone(),
    }
}

/// One animation drawn under his body: one material per look (its texture
/// file and own picture go to `all_materials`, the face's to `face`), one
/// hidden entity per frame. With `record`, also the first frame's smooth
/// normals (for the light log, M1 5b).
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
fn spawn_frames(
    commands: &mut Commands,
    body: Entity,
    anim: crate::load::Animation,
    lip_base: Option<&str>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    face: &mut Vec<Handle<ObjectMaterial>>,
    all_materials: &mut Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>,
    record: bool,
) -> (Vec<Entity>, Option<Vec<Vec3>>) {
    let looks: Vec<(i32, Handle<ObjectMaterial>, crate::load::Shading)> = anim
        .looks
        .into_iter()
        .map(|(m, look, l)| {
            // The look's name is where it was found (`vol:…/CF0.BMP`).
            let file = |name: &str| name.rsplit(['/', '\\']).next().unwrap_or(name).to_string();
            let is_face = matches!(&look, crate::load::Look::Texture { name, .. } if lip_base.is_some_and(|b| b.eq_ignore_ascii_case(&file(name))));
            let texture = match &look {
                crate::load::Look::Texture { name, .. } => file(name).to_ascii_lowercase(),
                _ => String::new(),
            };
            let handle = materials.add(object_material(look, l, light, images));
            all_materials.push((handle.clone(), texture, materials.get(&handle).and_then(|m| m.texture.clone())));
            if is_face {
                face.push(handle.clone());
            }
            (m, handle, l)
        })
        .collect();
    let mut entities = Vec::new();
    let mut first = None;
    // Its `.kqs` ball, for shadows on buildings (rule 37 (2), M4 4d-7b-5b).
    let ball = crate::shadow_view::ModelBall::of(anim.ball);
    for frame in anim.frames {
        let parent = commands.spawn((Transform::default(), Visibility::Hidden, ball)).id();
        commands.entity(body).add_child(parent);
        let record = record && first.is_none();
        let mut normals = Vec::new();
        for (m, triangles) in frame {
            let Some((_, material, shading)) = looks.iter().find(|(i, _, _)| *i == m) else { continue };
            if record {
                normals.extend(crate::object_light::smooth_normals(&triangles, shading.lighting));
            }
            let mesh = model_mesh(&triangles, shading.lighting, [0.0; 3]);
            let part = commands.spawn((Mesh3d(meshes.add(mesh)), MeshMaterial3d(material.clone()))).id();
            commands.entity(parent).add_child(part);
        }
        entities.push(parent);
        if record {
            first = Some(normals);
        }
    }
    (entities, first)
}

/// M6 6c-2 (DESIGN "6c"; not from a behaviour note): a weapon Connor
/// carries whose movement set was not loaded at start-up (a console `give`
/// of a weapon no file of this world names) gets it now: the world's
/// archives opened again, the set read as at start-up (`connor_set`), drawn
/// as at start-up, its materials added to the potion looks and the mouth.
/// Logged in `viewer.log` and `connor.log` with the time it took.
#[allow(clippy::too_many_arguments)]
pub fn late_set_step(
    mut commands: Commands,
    connor: Option<ResMut<ConnorSim>>,
    sim: Option<Res<crate::door_view::DoorSim>>,
    body: Query<Entity, With<ConnorBody>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut images: ResMut<Assets<Image>>,
    light: Option<Res<ObjectLight>>,
    shared: Res<Shared>,
) {
    let (Some(mut c), Some(sim), Some(light)) = (connor, sim, light) else { return };
    let Ok(body) = body.single() else { return };
    let c = &mut *c;
    let carried: Vec<String> = sim.active.world.items.objects.iter().filter(|o| o.carried_by(kq8_core::talk::CONNOR)).map(|o| o.kind.to_ascii_lowercase()).collect();
    let Some(mut late) = c.late.take_if(|l| l.sets.iter().any(|(k, _)| carried.contains(k))) else { return };
    let due: Vec<(String, String)> = late.sets.iter().filter(|(k, _)| carried.contains(k)).cloned().collect();
    late.sets.retain(|(k, _)| !carried.contains(k));
    let started = std::time::Instant::now();
    let mut log = shared.log.lock().unwrap();
    match crate::load::World::open(&late.game, &late.world) {
        Err(e) => log.summary(format!("connor: weapon sets {due:?}: world {} can't be opened again: {e}; not loaded", late.world)),
        Ok(world) => {
            for (kind, file) in due {
                let sections = crate::load::read_set(&late.game, &world, &file);
                let Some(list) = sections.first().map(|s| s.name.clone()) else {
                    log.summary(format!("connor: weapon set {file} ({kind}): not found; not loaded"));
                    continue;
                };
                let (mut movements, mut animations, mut index_of) = (Vec::new(), Vec::new(), std::collections::BTreeMap::new());
                if let Err(e) = crate::load::connor_set(&late.game, &world, &late.palette, &mut log, &sections, &list, Some(&file), &mut movements, &mut animations, &mut index_of) {
                    log.summary(format!("connor: weapon set {file} ({kind}): {e}; not loaded"));
                    continue;
                }
                // potions.md rule 29: the `I_` versions of its textures.
                let mut textures: Vec<String> = animations.iter().flat_map(|a| a.looks.iter()).filter_map(|(_, l, _)| match l {
                    crate::load::Look::Texture { name, .. } => Some(name.rsplit(['/', '\\']).next().unwrap_or(name).to_ascii_lowercase()),
                    _ => None,
                }).collect();
                textures.sort();
                textures.dedup();
                let ghosts: Vec<(String, crate::load::Look)> = textures.into_iter().filter_map(|t| {
                    let ghost = format!("i_{}", t.get(2..)?);
                    Some((t, crate::load::ghost_texture(&world, &ghost, &late.palette)?))
                }).collect();
                let loops: Vec<MovementLoop> = animations.iter().map(animation_loop).collect();
                let parts: Vec<_> = animations.iter().map(|a| a.frame_parts.clone()).collect();
                let mut added = Vec::new();
                let frames: Vec<Vec<Entity>> = animations
                    .into_iter()
                    .map(|a| spawn_frames(&mut commands, body, a, late.lip_base.as_deref(), &mut meshes, &mut materials, &light, &mut images, &mut c.face, &mut added, false).0)
                    .collect();
                let mut new = 0;
                for m in &movements {
                    if !c.movements.iter().any(|d| d.name == m.name) {
                        c.movements.push(drawn_movement(m, &loops, &frames, &parts));
                        new += 1;
                    }
                }
                let (n_materials, n_ghosts) = (added.len(), ghosts.len());
                c.looks.add(added, ghosts, &mut images);
                let line = format!(
                    "weapon set {file} ({kind}) loaded while playing: {new} movements, {} animations, {n_materials} materials ({n_ghosts} I_ pictures), {} ms so far (M6 6c-2)",
                    frames.len(),
                    started.elapsed().as_millis()
                );
                log.summary(format!("connor: {line}"));
                c.log.detail(format!("connor update {}: {line}", c.updates));
            }
        }
    }
    c.late = Some(late);
}

/// Spawns Connor (every frame of every movement, hidden but one) and
/// starts his mover with gravity on (connor-physics.md "Is gravity on at
/// the start?", recommendation).
#[allow(clippy::too_many_arguments)]
pub fn spawn_connor(
    commands: &mut Commands,
    (placement, mut assets): (Placement, ConnorAssets),
    camera_start: Option<[f32; 3]>,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    main_log: &mut Log,
) {
    let camera_keys = assets.camera_keys.clone();
    let connor_keys = assets.keys.clone();
    // His mouth (M4 4a-6): the face materials are found below.
    let lip_base = assets.lips.as_ref().map(|(b, _)| b.clone());
    let lip_images: Vec<Option<Handle<Image>>> = assets
        .lips
        .take()
        .map(|(_, looks)| {
            looks
                .into_iter()
                .map(|l| match l {
                    Some(crate::load::Look::Texture { width, height, rgba, .. }) => {
                        Some(images.add(crate::object_light::texture_image(width, height, rgba, bevy::image::ImageAddressMode::Repeat)))
                    }
                    _ => None,
                })
                .collect()
        })
        .unwrap_or_default();
    let mut face = Vec::new();
    // M5 5g-5: every material with its texture file and own picture.
    let mut all_materials = Vec::new();
    let potion_looks = std::mem::take(&mut assets.potion_looks);
    let late = assets.late.take();
    // Each animation's loop data; the movements add speed, flags and
    // transitions.
    let loops: Vec<MovementLoop> = assets.animations.iter().map(animation_loop).collect();
    // Each animation: one material per look, one entity per frame.
    let mut frame_entities: Vec<Vec<Entity>> = Vec::new();
    let body = commands.spawn((bevy_transform(&placement), Visibility::default(), ConnorBody)).id();
    // His first animation's first frame, for the light log (M1 5b).
    let mut first_frame_normals: Option<Vec<Vec3>> = None;
    // ranged-weapons.md rule 24 (M5 5f-5): each animation's named parts.
    let frame_parts: Vec<_> = assets.animations.iter().map(|a| a.frame_parts.clone()).collect();
    for anim in assets.animations {
        let record = first_frame_normals.is_none();
        let (entities, normals) = spawn_frames(commands, body, anim, lip_base.as_deref(), meshes, materials, light, images, &mut face, &mut all_materials, record);
        if normals.is_some() {
            first_frame_normals = normals;
        }
        frame_entities.push(entities);
    }
    let rotation = bevy_transform(&placement).rotation;
    match first_frame_normals.as_deref().and_then(|n| crate::object_light::light_stats(light, n, rotation)) {
        Some(stats) => main_log.summary(format!("object light: Connor at his start (first frame): {stats}")),
        None => main_log.summary("object light: Connor has no smooth corners in his first frame"),
    }
    let movements: Vec<DrawnMovement> = assets.movements.iter().map(|m| drawn_movement(m, &loops, &frame_entities, &frame_parts)).collect();
    for d in &movements {
        main_log.summary(format!(
            "connor: {} moves at {:.1} units/s ({} frames)",
            d.name,
            d.movement_loop.ground_speed(),
            d.frames.len()
        ));
    }

    let mut key = |action: &str, value: f32| {
        kq8_data::action_map::key_for(&assets.keys, action, value).and_then(|name| {
            let code = key_code(name);
            if code.is_none() {
                main_log.summary(format!("connor: key '{name}' for {action} {value} is not known to the viewer"));
            }
            code
        })
    };
    let controls = Controls {
        forward: key("MoveY", 1.0),
        back: key("MoveY", -1.0),
        left: key("RotateZ", 1.0),
        right: key("RotateZ", -1.0),
        run: key("Run", 0.0),
        jump: key("Jump", 0.0),
        draw: key("Draw", 0.0),
        away: key("Away", 0.0),
        shift: key("ModifyDn", 0.0),
        held: Vec::new(),
    };
    main_log.summary(format!(
        "connor: keys from Connor.sae: forward {:?}, back {:?}, turn left {:?}, turn right {:?}, run on/off {:?}, jump {:?}, draw {:?}, away {:?}, modifier {:?}",
        controls.forward, controls.back, controls.left, controls.right, controls.run, controls.jump, controls.draw, controls.away, controls.shift
    ));

    let mut mover = Mover::new(placement.position, true);
    if let Some((min, max)) = assets.collision_box {
        mover.balls = kq8_core::collide::BallStack::from_box(min, max);
    }
    main_log.summary(format!("connor: collision balls radius {} at heights {:?}", mover.balls.radius, mover.balls.heights));
    let log = match Log::create("connor") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("connor: cannot create connor.log: {e}"));
            return;
        }
    };
    main_log.summary(format!(
        "connor: mover started at ({:.0}, {:.0}, {:.0}), gravity {}, step up/down {}/{}, {} updates per second, run on; per-update log in {}",
        mover.position[0],
        mover.position[1],
        mover.position[2],
        if mover.use_gravity { "on" } else { "off" },
        mover.step_up,
        mover.step_down,
        kq8_core::UPDATES_PER_SECOND,
        log.path().display()
    ));
    let heading = placement.angles[2];
    let start = (mover.position, heading, 0.0, 0.0);
    let mut sim = ConnorSim {
        mover,
        walker: Walker::new(heading),
        // A freshly made Connor runs (connor-movement.md rule 4).
        intent: Intent { forward: 0, turn: 0, run: true },
        controls,
        movements,
        placement,
        previous: start,
        current: start,
        phase_step: 0.0,
        updates: 0,
        jump_pressed: false,
        cut_presses: Vec::new(),
        face,
        lip_images,
        looks: crate::potion_view::ConnorLooks::new(potion_looks, all_materials, images),
        mouth: 0,
        jump_start_z: None,
        turn_keys: Default::default(),
        hold_turn_down: 0,
        hold_run_set: false,
        repeat: None,
        log,
        last_state: "normal",
        last_down: "terrain".to_string(),
        shown: None,
        sounds: Vec::new(),
        dice: Default::default(),
        step_ground: Default::default(),
        ended: Vec::new(),
        pushed: Vec::new(),
        push_calls: Vec::new(),
        not_pushable: Vec::new(),
        hands_off: false,
        auto_ends: Vec::new(),
        refused: Vec::new(),
        walk_end: None,
        flinch_end: f64::NEG_INFINITY,
        last_ouch: f64::NEG_INFINITY,
        swing: None,
        do_cues: Vec::new(),
        shot: None,
        melee_target: None,
        shot_cues: Vec::new(),
        next_moving_shot: 0.0,
        run_back_at: None,
        moving_aim: None,
        weapon_set: None,
        weapon_target: None,
        pending_set: None,
        readying: false,
        target_mode: WeaponMode::None,
        ready_item: None,
        set_mode: WeaponMode::None,
        weapon_keys: Vec::new(),
        shift_held: false,
        held_walk: None,
        rope: None,
        rope_model: None,
        rope_shown: None,
        rope_ended: Vec::new(),
        rope_was_climbing: false,
        throw: None,
        throw_cues: Vec::new(),
        throw_ended: Vec::new(),
        late,
    };
    sim.log.detail(
        "update time_s x y z on_ground gravity fall_timer air_updates state drop ground_z ground_normal_z began_on_ground up_contact_nz down contact_nx contact_ny contact_nz push check pull_z forward turn run mode movement heading phase wanted_x wanted_y across hits down_what touch_gap blocked played played_from played_to wanted_movement step_left step_forward above_jump_start submerged surface_z bank",
    );
    sim.log.detail(
        "  (played = the animation that moved him this update, with its phase before and after; wanted_movement = asked for and not reached yet; step_left / step_forward = that animation's step in his own axes; movement / phase = where he is after the update)",
    );
    if let Some(cam) = crate::game_camera::make(
        sim.mover.position,
        heading,
        camera_start,
        &camera_keys,
        &connor_keys,
        main_log,
    ) {
        commands.insert_resource(cam);
        commands.insert_resource(crate::game_camera::CameraChoice { game: true });
    }
    commands.insert_resource(sim);
}

/// Keys: Connor's movement keys (held order) and the test keys G and B.
#[allow(clippy::too_many_arguments)]
pub fn connor_keys(
    keys: Res<ButtonInput<KeyCode>>,
    sim: Option<ResMut<ConnorSim>>,
    shared: Res<Shared>,
    mut start_lift: ResMut<StartLift>,
    mut start_at: ResMut<StartAt>,
    hold: Res<HoldIntent>,
    mut jump_at: ResMut<JumpAt>,
    mut turn_at: ResMut<TurnAt>,
    lookup: Option<Res<GroundLookup>>,
) {
    let Some(mut sim) = sim else { return };
    if let (Some([x, y, heading]), Some(lookup)) = (start_at.0, &lookup) {
        start_at.0 = None;
        let line = match lookup.0.floor_at(x, y) {
            Some(g) => {
                let balls = sim.mover.balls.clone();
                sim.mover = Mover::new([x, y, g.height], sim.mover.use_gravity);
                sim.mover.balls = balls;
                sim.walker = Walker::new(heading);
                let p = sim.mover.position;
                sim.previous = (p, heading, 0.0, 0.0);
                sim.current = sim.previous;
                format!(
                    "connor update {}: test --at: put on the ground at ({x:.0}, {y:.0}, {:.0}), heading {heading:.3}, ground normal z {:.3}",
                    sim.updates, g.height, g.normal[2]
                )
            }
            None => format!("connor update {}: test --at ({x:.0}, {y:.0}): no ground there; not moved", sim.updates),
        };
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    // Movement keys, in the order they were pressed.
    let c = &sim.controls;
    let movement_keys: Vec<KeyCode> = [c.forward, c.back, c.left, c.right].into_iter().flatten().collect();
    for k in &movement_keys {
        if keys.just_pressed(*k) {
            sim.controls.held.retain(|h| h != k);
            sim.controls.held.push(*k);
        }
        if keys.just_released(*k) {
            sim.controls.held.retain(|h| h != k);
        }
    }
    // Movement key events clear wall blocks (connor-physics.md "Walk held
    // against a wall").
    let c = &sim.controls;
    let mut key_events = Vec::new();
    for (key, what) in [(c.forward, MoveKey::Forward), (c.back, MoveKey::Back), (c.left, MoveKey::Turn), (c.right, MoveKey::Turn)] {
        let Some(key) = key else { continue };
        if keys.just_pressed(key) {
            key_events.push((what, true));
        }
        if keys.just_released(key) {
            key_events.push((what, false));
        }
    }
    // Only the key pressed last repeats, as in Windows; letting it go
    // ends the repeat.
    for (key, what) in [(c.forward, MoveKey::Forward), (c.back, MoveKey::Back), (c.left, MoveKey::Turn), (c.right, MoveKey::Turn)] {
        let Some(key) = key else { continue };
        if keys.just_released(key) && sim.repeat.is_some_and(|(k, _)| k == Some(key)) {
            sim.repeat = None;
        }
        if keys.just_pressed(key) {
            sim.repeat = Some((Some(key), what));
        }
    }
    for (what, pressed) in key_events {
        // handsOff (anim-scripts.md rule 21): presses are swallowed,
        // releases still count.
        if pressed && sim.hands_off {
            continue;
        }
        if pressed {
            sim.cut_presses.push(format!("{what:?} pressed"));
        }
        let cleared = sim.walker.key_event(what, pressed);
        if cleared != (false, false) {
            let block = match cleared {
                (true, true) => "forward and back blocks",
                (true, false) => "forward block",
                _ => "back block",
            };
            let line = format!("connor update {}: {what:?} {}: {block} cleared", sim.updates, if pressed { "pressed" } else { "released" });
            shared.log.lock().unwrap().detail(&line);
            sim.log.detail(line);
        }
    }
    // The turn keys as events (connor-movement.md "Turning" rule 2, Q104
    // a): releases first, then presses.
    let c = &sim.controls;
    let mut turn_events = Vec::new();
    for (key, dir) in [(c.left, 1i8), (c.right, -1i8)] {
        if key.is_some_and(|k| keys.just_released(k)) {
            turn_events.push((dir, false));
        }
    }
    for (key, dir) in [(c.left, 1i8), (c.right, -1i8)] {
        if key.is_some_and(|k| keys.just_pressed(k)) {
            turn_events.push((dir, true));
        }
    }
    for (dir, down) in turn_events {
        let what = if down { "key" } else { "key released" };
        turn_key_event(&mut sim, dir, down, what, &shared);
    }
    // ranged-weapons.md rule 1 (M5 5f-3): Delete and Alt ask for a mode;
    // Shift held for rule 6's `away2`.
    let c = &sim.controls;
    let (draw, away) = (c.draw.is_some_and(|k| keys.just_pressed(k)), c.away.is_some_and(|k| keys.just_pressed(k)));
    sim.shift_held = c.shift.is_some_and(|k| keys.pressed(k)) || keys.pressed(KeyCode::ShiftRight);
    if draw {
        sim.weapon_keys.push((kq8_core::weapons::Ask::Draw, "Delete (Draw) pressed".into()));
    }
    if away {
        sim.weapon_keys.push((kq8_core::weapons::Ask::Away, "Alt (Away) pressed".into()));
    }
    let c = &sim.controls;
    let forward = c.axis(c.forward, c.back);
    let run_pressed = c.run.is_some_and(|k| keys.just_pressed(k));
    if c.jump.is_some_and(|k| keys.just_pressed(k)) {
        sim.jump_pressed = true;
        if !sim.hands_off {
            sim.cut_presses.push("Jump pressed".into());
        }
    }
    let due = jump_at.0.len();
    jump_at.0.retain(|n| sim.updates < *n);
    if jump_at.0.len() < due {
        sim.jump_pressed = true;
        if !sim.hands_off {
            sim.cut_presses.push("test --jump: Jump pressed".into());
        }
        let line = format!("connor update {}: test --jump: Jump pressed", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    sim.intent.forward = forward;
    if let Some(&([_, _, r], _, _)) = hold.0.first() {
        // The hold whose updates these are (M6 6a-4: several, each from
        // its own start), if any.
        let now = hold.0.iter().find(|(_, from, until)| sim.updates >= *from && until.is_none_or(|n| sim.updates < n)).map(|(k, _, _)| *k);
        let [f, t, _] = now.unwrap_or([0; 3]);
        sim.intent.forward = f;
        // The held turn: pressed at the start, let go at the end (events).
        if t != sim.hold_turn_down {
            let was = std::mem::replace(&mut sim.hold_turn_down, t);
            if was != 0 {
                turn_key_event(&mut sim, was, false, "test --hold", &shared);
            }
            if t != 0 {
                turn_key_event(&mut sim, t, true, "test --hold", &shared);
            }
        }
        // The run switch only at the start, so scripts can change it
        // (`KQConner::setRun`, M3 step 3d-3).
        if !sim.hold_run_set {
            sim.hold_run_set = true;
            sim.intent.run = r != 0;
        }
    }
    if let Some((_, t)) = turn_at.0.filter(|&(n, _)| sim.updates >= n) {
        turn_at.0 = None;
        turn_key_event(&mut sim, t, true, "test --turn (pressed and held)", &shared);
        sim.repeat = Some((None, MoveKey::Turn));
        if !sim.hands_off {
            sim.cut_presses.push("test --turn: turn pressed".into());
        }
        let cleared = sim.walker.key_event(MoveKey::Turn, true);
        let side = if t > 0 { "left" } else { "right" };
        let line = format!("connor update {}: test --turn: {side} pressed and held; blocks cleared (forward, back): {cleared:?}", sim.updates);
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
    sim.intent.turn = sim.turn_keys.turn;
    if run_pressed && sim.hands_off {
        let line = format!("connor update {}: run key pressed: ignored (handsOff)", sim.updates);
        sim.log.detail(line);
    } else if run_pressed {
        sim.intent.run = !sim.intent.run;
        let line = format!("connor update {}: run switch {}", sim.updates, if sim.intent.run { "on" } else { "off" });
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }

    let start = start_lift.0.take().map(|h| ("--lift".to_string(), h));
    let pressed = [(KeyCode::KeyG, 5000.0), (KeyCode::KeyB, 2000.0)]
        .into_iter()
        .filter(|(k, _)| keys.just_pressed(*k))
        .map(|(k, h)| (format!("key {k:?}"), h));
    for (what, height) in start.into_iter().chain(pressed) {
        let was = sim.mover.state;
        sim.mover.lift(height);
        sim.walker.set_mode(Mode::Free);
        let p = sim.mover.position;
        sim.previous.0 = p;
        sim.current.0 = p;
        let line = format!(
            "connor update {}: test {what}: lifted {height} to z={:.0}{}",
            sim.updates,
            p[2],
            if was == State::Dead { " (was dead: alive again)" } else { "" }
        );
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
}

/// One turn key event (+1 left, −1 right; `down` or up) reaching Connor
/// (`connor-movement.md` "Turning" rule 2, Q104 a): sets the turn
/// direction; a down that reaches him (not swallowed by `handsOff`)
/// cancels an automatic turn at his next update. Logged in `connor.log`,
/// with "auto-turn cancelled" only on a down during an automatic turn.
fn turn_key_event(sim: &mut ConnorSim, dir: i8, down: bool, what: &str, shared: &Shared) {
    let hands_off = sim.hands_off;
    let e = if down { sim.turn_keys.press(dir, hands_off) } else { sim.turn_keys.release(dir, hands_off) };
    let turning = sim.walker.auto.as_ref().is_some_and(|a| a.phase == kq8_core::connor_auto::AutoPhase::Turning);
    if e.is_down() {
        sim.walker.turn_press = true;
    }
    let cancel = if e.is_down() && turning { "; auto-turn cancelled (at the next update)" } else if turning { "; the automatic turn goes on" } else { "" };
    sim.intent.turn = sim.turn_keys.turn;
    let line = format!(
        "connor update {}: turn key {} ({what}): {e}; turn direction {:+}{cancel}",
        sim.updates,
        if dir > 0 { "left" } else { "right" },
        sim.turn_keys.turn
    );
    shared.log.lock().unwrap().detail(&line);
    sim.log.detail(line);
}

impl ConnorSim {
    /// He can turn to face a listener (`talking.md` rule 7): in Wait,
    /// Move or the talk state.
    pub fn can_turn_to_listener(&self) -> bool {
        matches!(self.walker.mode, Mode::Free | Mode::Talk)
    }

    /// His state for the turn after a click (`clicking.md` rule 28): Wait
    /// or Move turn (talk, weapon draw, throwing and the tile puzzle are
    /// not built); anything else doesn't. The state's name, for the log.
    pub fn click_turn_state(&self) -> (bool, &'static str) {
        match self.walker.mode {
            Mode::Free if self.walker.moving() => (true, "Move"),
            Mode::Free => (true, "Wait"),
            // Rule 28: the talk state turns too (M4 4e-3).
            Mode::Talk => (true, "Talk"),
            m => (false, m.name()),
        }
    }

    /// The jump he is in (`jump`, `runJump`, `backJump`) for the scripts'
    /// `KQConner::JumpDir` / `endJump` (M3 3m-1), `None` outside the Jump
    /// state.
    pub fn jump_state(&self) -> Option<&'static str> {
        (self.walker.mode == Mode::Jump).then(|| self.walker.kind.jump())
    }

    /// `KQConner::endJump` (`anim-scripts.md` rule 28): the jump finished,
    /// no state change. Returns a line for the script logs.
    pub fn end_jump(&mut self, at: &str) -> String {
        let line = kq8_core::connor_update::finish_jump(&mut self.mover, &mut self.walker)
            .unwrap_or_else(|| "not in the Jump state any more; nothing".into());
        self.log.detail(format!("connor update {}: endJump: {line} ({at})", self.updates));
        format!("endJump: {line}")
    }

    /// A script asks for another state (`anim-scripts.md` rule 28): during
    /// an unfinished jump, with `handsOff` on the jump is finished first;
    /// without it the request is refused (`false`). Logged either way.
    pub fn state_request(&mut self, what: &str, at: &str) -> bool {
        // ropeandhook.md rule 28 (M6 6a-6): the rope state refuses every
        // other state; the caller drops the request (`roping`).
        if self.roping() {
            self.log.detail(format!("connor update {}: {what}: refused, he is in the rope state (ropeandhook.md rule 28); dropped ({at})", self.updates));
            return false;
        }
        // azriel-hammer.md rule 3 (M6 6b-3): the throw state can't be left
        // before `catch` has ended; tried again next update (STAND-IN, as a
        // jump's refusal; Q148 d).
        if self.throw.is_some() {
            self.log.detail(format!("connor update {}: {what}: refused, the hammer throw goes on (azriel-hammer.md rule 3); tried again next update (stand-in, Q148 d) ({at})", self.updates));
            return false;
        }
        match kq8_core::connor_update::state_request(&mut self.mover, &mut self.walker, self.hands_off) {
            Ok(None) => true,
            Ok(Some(note)) => {
                self.log.detail(format!("connor update {}: {what}: {note} ({at})", self.updates));
                true
            }
            Err(why) => {
                self.log.detail(format!("connor update {}: {what}: refused, {why}; tried again next update (stand-in) ({at})", self.updates));
                false
            }
        }
    }

    /// `fighting.md` rules 23, 24, 29 (M5 5c-4): a hit took health and he
    /// lives. He flinches (his `react`, keys ignored while it plays, as a
    /// Special list) when the roll was over 0.9 *D*, his movement can be
    /// cut short (the keys choose it, no start or stop transition plays,
    /// its `loop=` line says so or it has ended) and his last flinch ended
    /// over 2 s ago; otherwise his ouch sound, at most once a second (a
    /// flinch also starts that second). Returns the line for `fight.log`.
    pub fn hurt(&mut self, roll_over: bool, ouch: &str, now: f64) -> String {
        use kq8_core::fight::{FLINCH_PAUSE, OUCH_EVERY};
        let can_cut = self.can_be_cut_short();
        let w = &self.walker;
        let react = self.drawn("react").map(|m| (m.name, m.movement_loop.duration / m.movement_loop.speed.abs().max(1e-3)));
        let why_not = if !roll_over {
            Some("the roll is not above 0.9 x D".to_string())
        } else if !can_cut {
            Some(format!("'{}' can't be cut short now (mode {})", w.movement, w.mode.name()))
        } else if now < self.flinch_end + FLINCH_PAUSE {
            Some(format!("his last flinch ended {:.2} s ago (2 s pause)", now - self.flinch_end))
        } else if react.is_none() {
            Some("no react in his list".to_string())
        } else {
            None
        };
        let line = match (why_not, react) {
            (None, Some((name, length))) => {
                self.walker.start_special(vec![kq8_core::connor::SpecialStep::Play(name, "react".into())]);
                self.flinch_end = now + f64::from(length);
                self.last_ouch = now;
                format!("flinches: '{name}' ({length:.2} s, keys ignored meanwhile; rules 23, 29)")
            }
            (why, _) => {
                let why = why.unwrap_or_default();
                if now + 1e-6 >= self.last_ouch + OUCH_EVERY {
                    self.last_ouch = now;
                    self.sounds.push(kq8_core::sound::Start::placed_one_shot(ouch, self.mover.position, "his ouch sound (fighting.md rule 24)".into()));
                    format!("no flinch ({why}): his ouch sound {ouch} (rule 24)")
                } else {
                    format!("no flinch ({why}); no ouch sound, the last was {:.2} s ago (at most once a second, rule 24)", now - self.last_ouch)
                }
            }
        };
        self.log.detail(format!("connor update {}: hurt: {line}", self.updates));
        line
    }

    /// His running movement can be cut short now (`fighting.md` rules 23,
    /// 40): the keys choose it (or it is his swing) and no start or stop
    /// transition plays, and its `loop=` line with its `interruptable`
    /// cues says so, or it has ended.
    fn can_be_cut_short(&self) -> bool {
        let w = &self.walker;
        let swinging = self.swing.as_ref().is_some_and(|s| !s.waiting && w.mode == Mode::Special && w.movement == s.movement);
        if !(w.keys_choose() || swinging) || w.playing.is_some() {
            return false;
        }
        self.movement_loop(w.movement).is_none_or(|l| kq8_core::fight::cut_short_at(l.cut_short, &l.cues, w.phase))
    }

    /// `ranged-weapons.md` rule 32 (M5 5f-7; `fighting.md` rules 39-40): a
    /// click with a hand weapon ready. `picked`: what the click line met
    /// (types with bit 0x40 or 0x20) and its position, `None` for nothing
    /// or a building; `point`: where he turns (the picked thing's
    /// position, else the click point); `reach`: his `attackDist`.
    /// Nothing while a swing plays or his movement can't be cut short.
    /// A picked thing within reach (straight 3D distance between the two
    /// positions) becomes his target, which stays until another pick
    /// replaces it. He turns toward the point (the automatic turn); for a
    /// picked thing more than 45° off his facing the swing waits until
    /// the turn has 8 updates or fewer to go; otherwise it starts at once
    /// and he turns during it. `Attack` / `AttackB` 50/50, Shift
    /// `Attack2`, from the weapon's list (`list`). Returns the line for
    /// `fight.log`.
    #[allow(clippy::too_many_arguments)]
    pub fn swing_click(&mut self, list: &str, picked: Option<(String, [f32; 3])>, point: [f32; 2], big: bool, coin: f32, reach: f32) -> String {
        use kq8_core::connor_auto::{heading_towards, wrap, AutoKind, AutoMove};
        if self.swing.is_some() {
            return "a swing is already playing or waiting: nothing (rule 40)".into();
        }
        if !self.can_be_cut_short() {
            return format!("'{}' can't be cut short (mode {}): no swing (rule 40)", self.walker.movement, self.walker.mode.name());
        }
        let word = if big { "attack2" } else if coin < 0.5 { "attack" } else { "attackb" };
        let wanted = kq8_core::connor::special_movement_name(list, word);
        let Some(movement) = self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name) else {
            return format!("no {wanted} loaded: no swing");
        };
        let feet = self.mover.position;
        let pick = match &picked {
            Some((name, at)) => {
                let d = (0..3).map(|k| (at[k] - feet[k]).powi(2)).sum::<f32>().sqrt();
                if d <= reach {
                    self.melee_target = Some(name.clone());
                    format!("{name} picked, {d:.0} away: within {reach:.0}, his target now")
                } else {
                    format!("{name} picked, {d:.0} away: beyond {reach:.0}, his target stays {:?}", self.melee_target)
                }
            }
            None => format!("nothing or a building picked: his target stays {:?}", self.melee_target),
        };
        let off = wrap(heading_towards(feet, point) - self.walker.heading);
        let waits = picked.is_some() && off.abs() > kq8_core::weapons::SHOT_WAIT_ANGLE;
        let turn = self.walker.start_auto(AutoMove::new(AutoKind::Point, point, None));
        self.swing = Some(Swing { movement, target: self.melee_target.clone(), big, waiting: waits, point });
        let start = if waits {
            "more than 45 deg off: the swing waits until 8 updates of turn are left".to_string()
        } else {
            self.walker.start_special(vec![kq8_core::connor::SpecialStep::Play(movement, word.into())]);
            format!("'{movement}' starts at once, he turns during it")
        };
        let line = format!(
            "swing ({}): {pick} (rule 32); {start}; turn toward ({:.0}, {:.0}), {:.0} deg off his heading; {turn}",
            if big { "Shift: Attack2" } else { "Attack or AttackB, 50/50" },
            point[0],
            point[1],
            off.to_degrees()
        );
        self.log.detail(format!("connor update {}: {line}", self.updates));
        line
    }

    /// The weapon swap (`ranged-weapons.md` rule 6; M5 5f-3), or a weapon
    /// readied or put away outside a request (`at_once`: picked up while
    /// another is ready, 5d fix 11: straight into its stance). `list`: the
    /// new mode's movement list (`None`: his own); `mode`, `item`: the new
    /// mode and item; `shift`: the big-swing key held; `script`: asked by
    /// `KQConner::setDoMode` (rule 9: he is put into standing first). In
    /// rule 6's order:
    /// 1. none → the Fists: the list switches first, then `draw`;
    /// 2. the Fists → none: `undraw`, the switch back when it ends;
    /// 3. from a hand weapon with Shift: `away2` (else `away`), then as 4;
    /// 4. otherwise `away` in the list in use now; when it ends, the switch
    ///    (the mode, the list, the pointer), and `out` from the new list.
    ///
    /// Then standing. The movement playing is asked again by name in the
    /// new list (rule 8): our movements are looked up by name in the set
    /// in use. Plays only while he stands or moves (or for a script);
    /// otherwise the set switches at once. Returns the line for `fight.log`.
    #[allow(clippy::too_many_arguments)]
    pub fn ready_set(&mut self, list: Option<String>, mode: WeaponMode, item: Option<String>, at_once: bool, shift: bool, script: bool) -> String {
        use kq8_core::connor::SpecialStep;
        let old = self.weapon_target.clone();
        let old_mode = self.target_mode;
        let old_item = self.ready_item.clone();
        if old == list && old_mode == mode && old_item == item {
            return "no change".into();
        }
        self.weapon_target = list.clone();
        self.target_mode = mode;
        self.ready_item = item.clone();
        if at_once {
            self.pending_set = None;
            self.switch_set(list.clone(), mode);
            return format!("picked up while another was ready: {list:?} at once, nothing plays (the user's check)");
        }
        let named = |file: &str, w: &str| {
            let wanted = kq8_core::connor::special_movement_name(file, w);
            self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
        };
        let own = |w: &str| self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(w)).map(|m| m.name);
        // A movement of a list (`None`: his own).
        let of = |l: &Option<String>, w: &str| match l {
            Some(f) => named(f, w),
            None => own(w),
        };
        let fists = |i: &Option<String>| i.as_deref().is_some_and(|k| k.eq_ignore_ascii_case(kq8_core::weapons::FISTS));
        let mut steps: Vec<(&'static str, String)> = Vec::new();
        let switch_at;
        let case;
        if old_mode == WeaponMode::None && mode == WeaponMode::Hand && fists(&item) {
            case = "1: none to the Fists, the list first, then draw";
            switch_at = 0;
            if let Some(m) = of(&list, "draw") {
                steps.push((m, "draw".into()));
            }
        } else if old_mode == WeaponMode::Hand && fists(&old_item) && mode == WeaponMode::None {
            case = "2: the Fists to none, undraw, then the list";
            if let Some(m) = of(&old, "undraw") {
                steps.push((m, "undraw".into()));
            }
            switch_at = steps.len();
        } else {
            let big = old_mode == WeaponMode::Hand && shift;
            case = if big { "3: from a hand weapon with Shift: away2, the list, out" } else { "4: away, the list, out" };
            let away = if big { of(&old, "away2").map(|m| (m, "away2")).or_else(|| of(&old, "away").map(|m| (m, "away"))) } else { of(&old, "away").map(|m| (m, "away")) };
            if let Some((m, w)) = away {
                steps.push((m, format!("{w} ({})", old.as_deref().unwrap_or("his own"))));
            }
            switch_at = steps.len();
            if let Some(m) = of(&list, "out") {
                steps.push((m, format!("out ({})", list.as_deref().unwrap_or("his own"))));
            }
        }
        let set = match &list {
            Some(l) => format!("his movements come from {l} where it has them"),
            None => "his own movements again (Conner.anm)".into(),
        };
        let played: Vec<&str> = steps.iter().map(|(m, _)| *m).collect();
        let plays = self.walker.mode == Mode::Free || (script && matches!(self.walker.mode, Mode::Special | Mode::Push | Mode::Talk));
        // Q141 (the user's check in the original, 2026-10-21): a script's
        // request while a scripted walk runs (`ax.cs`: `setDoMode 0`, then
        // `walkTo` in the same event): he stops, the swap plays, then the
        // walk starts (held until the swap is over, `weapon_step`).
        let scripted_walk = self.walker.auto.as_ref().is_some_and(|a| matches!(a.kind, kq8_core::connor_auto::AutoKind::Walk { .. }));
        let held = if script && scripted_walk && !steps.is_empty() {
            self.held_walk = self.walker.auto.take();
            "; the scripted walk waits until the swap is over (Q141, the user's check)"
        } else {
            ""
        };
        if !plays || steps.is_empty() {
            self.pending_set = None;
            self.switch_set(list, mode);
            return format!("rule 6 case {case}; {set} at once; mode {}: {played:?} not played", self.walker.mode.name());
        }
        if switch_at == 0 {
            self.pending_set = None;
            self.switch_set(list, mode);
        } else {
            self.pending_set = Some((list, mode, switch_at));
        }
        self.walker.auto = None;
        self.readying = true;
        let notes = self.walker.start_special(steps.iter().map(|(m, w)| SpecialStep::Play(m, w.clone())).collect());
        let line = format!("rule 6 case {case}: plays {played:?}; {set} from item {switch_at} ({}){held}", notes.join("; "));
        self.log.detail(format!("connor update {}: weapon: {line}", self.updates));
        line
    }

    /// The movement set switches (5d fix 10; `ranged-weapons.md` rule 8),
    /// with his hand mode (`pushing.md`: sword 0, bow 1, empty hands 4;
    /// `conversation.md` rule 7, gestures, pushing) and the pointer's mode
    /// (rule 11).
    fn switch_set(&mut self, list: Option<String>, mode: WeaponMode) {
        self.walker.hand_mode = if mode == WeaponMode::None { kq8_core::connor::EMPTY_HANDS } else { mode.number() };
        self.weapon_set = list;
        self.set_mode = mode;
    }

    /// After his own update: the waiting set switch when its list item
    /// starts (or the list ended or was replaced); the swap is over when
    /// the Special state is.
    fn weapon_step(&mut self) {
        if let Some((set, mode, at)) = self.pending_set.clone() {
            if self.walker.mode != Mode::Special || self.walker.special_at >= at {
                self.pending_set = None;
                self.switch_set(set, mode);
                let n = self.updates;
                self.log.detail(format!("connor update {n}: weapon: the set switches now ({:?}, {})", self.weapon_set, mode.name()));
            }
        }
        if self.readying && self.walker.mode != Mode::Special {
            self.readying = false;
            // Q141: the scripted walk held during the swap starts now.
            if let Some(a) = self.held_walk.take() {
                let line = self.walker.start_auto(a);
                self.log.detail(format!("connor update {}: weapon: the swap is over: {line} (Q141)", self.updates));
            }
        }
    }

    /// `ranged-weapons.md` rule 5 (M5 5f-3): a request is taken only while
    /// he stands or moves (our Free mode), not while a swap plays; the
    /// landings `jumprun` / `jumpstop` play in Free mode but are the
    /// landing (the user's checks, 5c fixes 2, 3). `Err`: why not.
    pub fn may_ready(&self) -> Result<(), String> {
        let w = &self.walker;
        if self.readying {
            return Err("a weapon swap plays".into());
        }
        if w.mode != Mode::Free {
            return Err(format!("mode {}", w.mode.name()));
        }
        let landing = |n: &str| n.eq_ignore_ascii_case("jumprun") || n.eq_ignore_ascii_case("jumpstop");
        if landing(w.movement) || w.playing.is_some_and(|t| landing(t.via)) {
            return Err(format!("the landing {} plays", w.movement));
        }
        Ok(())
    }

    /// His full turn now: the lean about his forward axis, then his
    /// heading (as he is drawn; `ranged-weapons.md` rule 23).
    pub fn turn_matrix(&self) -> [[f32; 3]; 3] {
        let (_, heading, _, bank) = self.current;
        let lean = kq8_data::objects::rotation([0.0, bank, 0.0]);
        let r = kq8_data::objects::rotation([0.0, 0.0, heading]);
        std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| lean[i][k] * r[k][j]).sum()))
    }

    /// The named part `part` of `movement`'s frame at `phase` (rule 24).
    pub fn part_at(&self, movement: &str, phase: f32, part: &str) -> Option<([f32; 3], String)> {
        let m = self.drawn(movement)?;
        let frame = m.movement_loop.frame_at(phase);
        let place = m.parts.get(frame)?.iter().find(|(n, _)| n.eq_ignore_ascii_case(part))?.1;
        Some((place, format!("its part '{part}' in frame {frame} of '{movement}'")))
    }

    /// Where his named part `part` is in the world now, in the frame
    /// playing (`potions.md` rule 20: the shield's orbit centre, M5 5g-4).
    pub fn part_now(&self, part: &str) -> Option<[f32; 3]> {
        let (o, _) = self.part_at(self.walker.movement, self.walker.phase, part)?;
        let (r, feet) = (self.turn_matrix(), self.mover.position);
        Some(std::array::from_fn(|k| feet[k] + o[0] * r[0][k] + o[1] * r[1][k] + o[2] * r[2][k]))
    }

    /// `ranged-weapons.md` rule 20 (M5 5f-5): a shot click with the ranged
    /// weapon ready, aim point `aim` (rules 16, 18). He turns toward it
    /// (the automatic turn, 3.5 rad/s; we are always in third person);
    /// more than 45° off his facing the shot waits until the turn has 3
    /// updates or fewer to go; otherwise it is asked at once. Returns the
    /// line for `fight.log`.
    pub fn shot_click(&mut self, list: &str, aim: [f32; 3]) -> String {
        use kq8_core::connor_auto::{heading_towards, wrap, AutoKind, AutoMove};
        let wanted = kq8_core::connor::special_movement_name(list, "attack");
        let Some(movement) = self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name) else {
            return format!("no {wanted} loaded: no shot");
        };
        let off = wrap(heading_towards(self.mover.position, [aim[0], aim[1]]) - self.walker.heading);
        let turn = self.walker.start_auto(AutoMove::new(AutoKind::Point, [aim[0], aim[1]], None));
        let waits = off.abs() > kq8_core::weapons::SHOT_WAIT_ANGLE;
        if self.shot.as_ref().is_some_and(|s| s.waiting) {
            self.log.detail(format!("connor update {}: a new click: the shot waiting for the turn is lost (rule 20)", self.updates));
        }
        // A shot already playing stays; only a waiting one is replaced.
        if self.shot.as_ref().is_some_and(|s| s.started) {
            let line = self.ask_shot(movement, aim);
            return format!("{:.0} deg off his facing; {turn}; {line}", off.to_degrees());
        }
        self.shot = Some(Shot { movement, aim, waiting: waits, started: false });
        let line = if waits { "more than 45 deg off: the shot waits until 3 updates of turn are left (rule 20)".to_string() } else { self.ask_shot(movement, aim) };
        format!("{:.0} deg off his facing; {turn}; {line}", off.to_degrees())
    }

    /// `azriel-hammer.md` rules 2-4 (M6 6b-3): a throw click with the hammer
    /// ready, aim point `aim` (after `ranged-weapons.md` rules 13-18). Only
    /// from standing or moving, not kept otherwise; he turns toward the aim
    /// point (the automatic turn) with no waiting; handsOff on; `attack`.
    /// Returns the line for `fight.log`.
    pub fn throw_click(&mut self, list: &str, aim: [f32; 3]) -> String {
        use kq8_core::connor_auto::{heading_towards, wrap, AutoKind, AutoMove};
        if self.throw.is_some() {
            return "a throw goes on: nothing (rule 4)".into();
        }
        if let Err(why) = self.may_ready() {
            return format!("not standing or moving ({why}): no throw, the click is not kept (rule 2)");
        }
        let find = |word: &str| {
            let wanted = kq8_core::connor::special_movement_name(list, word);
            self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
        };
        let (Some(attack), Some(wait), Some(catch)) = (find("attack"), find("wait"), find("catch")) else {
            return format!("{list} lacks attack, wait or catch: no throw");
        };
        let off = wrap(heading_towards(self.mover.position, [aim[0], aim[1]]) - self.walker.heading);
        let turn = self.walker.start_auto(AutoMove::new(AutoKind::Point, [aim[0], aim[1]], None));
        let was = self.walker.movement;
        self.walker.throw_play(attack);
        self.hands_off = true;
        self.throw = Some(Throw { attack, wait, catch, aim, back: false, from_update: self.updates });
        let line = format!(
            "{:.0} deg off his facing; {turn}, no waiting (rule 2); from '{was}': the hammer-throw state, handsOff on (keys and clicks; the panels and the pointer unchanged: stand-in, Q148 e), '{attack}' starts (rule 3)",
            off.to_degrees()
        );
        self.log.detail(format!("connor update {}: throw: {line}", self.updates));
        line
    }

    /// Rule 9 (M6 6b-3): the hammer came back: `catch` at once, from
    /// `wait` or cutting `attack` short (the user's check in the original,
    /// Q148 c: against a wall he catches it as soon as it leaves his hand).
    /// Returns the line for `fight.log`.
    pub fn throw_caught(&mut self, how: &str) -> String {
        let Some(t) = self.throw.as_mut() else {
            return format!("{how}: he is not throwing: nothing");
        };
        t.back = true;
        let secs = (self.updates - t.from_update) as f64 / kq8_core::UPDATES_PER_SECOND;
        if self.walker.mode != Mode::Throw || self.walker.movement == t.catch {
            return format!("{how}: '{}' plays already or he is not in the throw state ({secs:.2} s after the click)", t.catch);
        }
        let (c, attack, was, phase) = (t.catch, t.attack, self.walker.movement, self.walker.phase);
        self.walker.throw_play(c);
        // Drawn from its start at once (as the rope's, 6a fix).
        self.redraw_from_walker();
        let cut = if was == attack { format!(", '{was}' cut short at {phase:.3} (the user's check, Q148 c)") } else { String::new() };
        format!("{how}: '{c}' starts{cut}, {secs:.2} s after the click (rule 3)")
    }

    /// Rule 3 (M6 6b-3), each update after his own: `attack` ended ->
    /// `wait` (or `catch` if the hammer is back); `catch` ended -> handsOff
    /// off, standing. Returns the line for `fight.log`.
    pub fn throw_step(&mut self) -> Option<String> {
        let ended = std::mem::take(&mut self.throw_ended);
        let t = self.throw.clone()?;
        let mut lines = Vec::new();
        for e in ended {
            // Not when the hammer's return already cut it short (Q148 c).
            if e == t.attack && self.walker.movement == t.attack {
                let next = if t.back { t.catch } else { t.wait };
                self.walker.throw_play(next);
                self.redraw_from_walker();
                lines.push(format!("'{e}' ended: '{next}' (rule 3)"));
            } else if e == t.catch {
                self.walker.throw_leave();
                self.throw = None;
                self.hands_off = false;
                let again = self.keys_sent_again();
                let secs = (self.updates - t.from_update) as f64 / kq8_core::UPDATES_PER_SECOND;
                lines.push(format!("'{e}' ended: handsOff off ({again}), standing; the throw took {secs:.2} s (rule 3)"));
            }
        }
        if lines.is_empty() {
            return None;
        }
        let line = lines.join("; ");
        self.log.detail(format!("connor update {}: throw: {line}", self.updates));
        Some(line)
    }

    /// Rule 20 (M5 5f-6): his updates as seconds.
    fn clock(&self) -> f64 {
        self.updates as f64 / kq8_core::UPDATES_PER_SECOND
    }

    /// Rule 20: the shot is asked. Standing (our Free mode, no movement
    /// key): `attack` only if his movement can be cut short (a click
    /// during `attack` does nothing). Moving (M5 5f-6): no `attack`; if
    /// his `attackDelay` (0.5 s) has passed since the last shot the bolt
    /// leaves at once from the frame playing, and a run becomes a walk
    /// for 1 s.
    fn ask_shot(&mut self, movement: &'static str, aim: [f32; 3]) -> String {
        use kq8_core::connor::SpecialStep;
        if self.intent.forward != 0 && self.walker.mode == Mode::Free {
            self.shot = None;
            let now = self.clock();
            if now + 1e-6 < self.next_moving_shot {
                return format!("moving: the last shot was less than 0.5 s ago (next at {:.2} s, now {now:.2} s): nothing (rule 20)", self.next_moving_shot);
            }
            self.next_moving_shot = now + MOVING_SHOT_DELAY;
            let run = if self.intent.run {
                self.intent.run = false;
                self.run_back_at = Some(now + 2.0 * MOVING_SHOT_DELAY);
                "; he was running: he walks, the run comes back 1 s after the last moving shot".to_string()
            } else if self.run_back_at.is_some() {
                self.run_back_at = Some(now + 2.0 * MOVING_SHOT_DELAY);
                "; the run comes back 1 s from now".to_string()
            } else {
                String::new()
            };
            self.shot_cues.push((self.walker.movement, self.walker.phase, true));
            self.moving_aim = Some(aim);
            let line = format!("moving ('{}' at {:.4}): the bolt leaves at once, no attack; the next moving shot at {:.2} s{run} (rule 20)", self.walker.movement, self.walker.phase, self.next_moving_shot);
            self.log.detail(format!("connor update {}: shot: {line}", self.updates));
            return line;
        }
        let can = {
            let w = &self.walker;
            let shooting = self.shot.as_ref().is_some_and(|s| s.started && w.mode == Mode::Special && w.movement == s.movement);
            !shooting && w.keys_choose() && w.playing.is_none() && self.movement_loop(w.movement).is_none_or(|l| kq8_core::fight::cut_short_at(l.cut_short, &l.cues, w.phase))
        };
        if !can {
            if !self.shot.as_ref().is_some_and(|s| s.started) {
                self.shot = None;
            }
            return format!("standing: '{}' can't be cut short (mode {}): no shot (rule 20)", self.walker.movement, self.walker.mode.name());
        }
        let notes = self.walker.start_special(vec![SpecialStep::Play(movement, "attack".into())]);
        self.shot = Some(Shot { movement, aim, waiting: false, started: true });
        let line = format!("standing: '{movement}' starts ({}); the bolt leaves at its do cue (rule 20)", notes.join("; "));
        self.log.detail(format!("connor update {}: shot: {line}", self.updates));
        line
    }

    /// Each update after his own (rule 20): a waiting shot is asked when
    /// the turn has 3 updates or fewer to go (lost if the turn was
    /// replaced or cancelled); a shot whose `attack` ended is over.
    pub fn shot_step(&mut self) -> Option<String> {
        use kq8_core::connor_auto::{heading_towards, wrap};
        let s = self.shot.clone()?;
        if s.waiting {
            let Some(auto) = &self.walker.auto else {
                self.shot = None;
                return Some("the turn was replaced or cancelled: the waiting shot is lost (rule 20)".into());
            };
            if auto.target != [s.aim[0], s.aim[1]] {
                self.shot = None;
                return Some("another turn order: the waiting shot is lost (rule 20)".into());
            }
            let left = wrap(heading_towards(self.mover.position, [s.aim[0], s.aim[1]]) - self.walker.heading);
            let per_update = kq8_core::connor_auto::AUTO_TURN_RATE / kq8_core::UPDATES_PER_SECOND as f32;
            if left.abs() > kq8_core::weapons::SHOT_TURN_UPDATES * per_update + 1e-4 {
                return None;
            }
            let line = self.ask_shot(s.movement, s.aim);
            return Some(format!("{:.2} rad of turn left (3 updates or fewer): {line}", left.abs()));
        }
        if s.started && (self.walker.mode != Mode::Special || self.walker.movement != s.movement) {
            self.shot = None;
            // Rule 20: the end of a standing `attack` also sets the next
            // moving shot.
            self.next_moving_shot = self.clock() + MOVING_SHOT_DELAY;
            return Some(format!("'{}' is over: the next shot can start (no extra delay); the next moving shot at {:.2} s (rule 20)", s.movement, self.next_moving_shot));
        }
        None
    }

    /// Rule 20, "Do not reproduce" (the simpler way): the run comes back
    /// 1 s after the last moving shot.
    pub fn run_back_step(&mut self) -> Option<String> {
        let at = self.run_back_at?;
        if self.clock() + 1e-6 < at {
            return None;
        }
        self.run_back_at = None;
        self.intent.run = true;
        Some(format!("1 s after the last moving shot: the run is switched on again (rule 20, the simpler way of Do not reproduce) at {:.2} s", self.clock()))
    }

    /// Each update after his own (M5 5c-5): a swing that waited for his
    /// turn starts when the turn is over; a swing that ended (or was
    /// replaced) is forgotten. Returns a line for `fight.log`.
    pub fn swing_step(&mut self) -> Option<String> {
        let s = self.swing.as_mut()?;
        if s.waiting {
            // Rule 32: when the turn has 8 updates or fewer to go (or is
            // over).
            let left = kq8_core::connor_auto::wrap(kq8_core::connor_auto::heading_towards(self.mover.position, s.point) - self.walker.heading);
            let per_update = kq8_core::connor_auto::AUTO_TURN_RATE / kq8_core::UPDATES_PER_SECOND as f32;
            if self.walker.auto.is_some() && left.abs() > SWING_TURN_UPDATES * per_update + 1e-4 {
                return None;
            }
            s.waiting = false;
            let m = s.movement;
            self.walker.start_special(vec![kq8_core::connor::SpecialStep::Play(m, kq8_core::connor::plain_movement_name(m).to_string())]);
            return Some(format!("{:.2} rad of turn left (8 updates or fewer): '{m}' starts (rule 32)", left.abs()));
        }
        if self.walker.mode != Mode::Special || self.walker.movement != s.movement {
            let m = s.movement;
            self.swing = None;
            return Some(format!("'{m}' is over: the next click can swing"));
        }
        None
    }

    /// `fighting.md` rule 29 (M5 5c-4): his health fell below 0: his dying
    /// state by the hit's kind (`die`; fire and ice: `dropweapon`, then
    /// `firedie` / `icedie`), played as a Special list ending in his Die
    /// state, which holds the last frame. **Stand-in**: game over is
    /// another study; he stays there. Returns the line for `fight.log`.
    pub fn die_from_hit(&mut self, kind: kq8_core::fight::HitKind) -> String {
        use kq8_core::connor::SpecialStep;
        use kq8_core::fight::HitKind;
        let names: &[&str] = match kind {
            HitKind::Normal | HitKind::Projectile => &["die"],
            HitKind::Fire => &["dropweapon", "firedie"],
            HitKind::Ice => &["dropweapon", "icedie"],
        };
        let mut steps: Vec<SpecialStep> = names
            .iter()
            .map(|n| match self.drawn(n) {
                Some(m) => SpecialStep::Play(m.name, n.to_string()),
                None => SpecialStep::Note(format!("{n} not in his list: skipped")),
            })
            .collect();
        self.walker.death = steps.iter().rev().find_map(|s| if let SpecialStep::Play(n, _) = s { Some(*n) } else { None });
        steps.push(SpecialStep::Kill);
        self.walker.auto = None;
        let notes = self.walker.start_special(steps);
        let line = format!("dies ({}): plays {names:?}, then his Die state holding the last frame; game over is another study (stand-in: he stays dead) ({})", kind.name(), notes.join("; "));
        self.log.detail(format!("connor update {}: {line}", self.updates));
        line
    }

    /// `KQMonster::setState Connor special <items>` (`anim-scripts.md`
    /// rule 16): the items worked out against the movements loaded (his
    /// own and the Special lists'), then the Special state. Returns a
    /// line for the script logs.
    pub fn start_special(&mut self, items: &str, at: &str) -> String {
        let movements = &self.movements;
        let resolve = |list: Option<&str>, word: &str| -> Option<&'static str> {
            let wanted = match list {
                Some(l) => kq8_core::connor::special_movement_name(l, word),
                None => word.to_string(),
            };
            movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
        };
        let steps = kq8_core::connor::special_steps(items, &resolve);
        let plays: Vec<&str> = steps.iter().filter_map(|s| if let kq8_core::connor::SpecialStep::Play(n, _) = s { Some(*n) } else { None }).collect();
        let line = format!("special {items}: plays {plays:?}");
        let notes = self.walker.start_special(steps);
        self.log.detail(format!("connor update {}: KQMonster::setState Connor special {items} ({at}): {}", self.updates, notes.join("; ")));
        line
    }

    /// `move Connor <flags> x y z rx ry rz` from a script (M3 3k-1,
    /// `objects.md` "(d)"): his position set or added, his heading (rz)
    /// set or added; no collision passes; the mover's state is kept. Then
    /// the move step's own vertical bookkeeping (one gravity step, the
    /// terrain raise; refined Q92), and when the script ran **inside his
    /// update** (a `Push` call, `pushing.md` rule 11) the rest of that
    /// update: the down pass and the ground check (M3 3l-1). Returns a
    /// line for the script logs.
    #[allow(clippy::too_many_arguments)]
    pub fn move_to(&mut self, flags: u32, position: [f32; 3], turn: [f32; 3], at: &str, world: &dyn MoverWorld, inside: bool) -> String {
        let old = (self.mover.position, self.walker.heading);
        self.mover.position = if flags & 1 != 0 { std::array::from_fn(|k| old.0[k] + position[k]) } else { position };
        self.walker.heading = if flags & 2 != 0 { old.1 + turn[2] } else { turn[2] };
        let tilt = if turn[0] != 0.0 || turn[1] != 0.0 { " (his turn about x and y ignored: he only turns about z)" } else { "" };
        let set = self.mover.position;
        let drop = self.mover.after_move(world);
        let mut rest = format!("; the move step: gravity step {drop:.1} -> z {:.1}", self.mover.position[2]);
        if inside {
            let r = self.mover.rest_of_update(world);
            let on = r.hits.iter().find(|h| h.pass == "down").map_or(r.check.to_string(), |h| format!("{} ({})", h.hit.what, h.result));
            rest += &format!(
                "; inside his update (a trap touch or a Push call): the rest of it: drop {:.1}, step-down 325, down pass on {on} -> z {:.1}, on ground {}",
                r.drop, self.mover.position[2], self.mover.on_ground
            );
        } else {
            rest += "; outside his update: he falls from there";
        }
        // No blending from the old place in the drawing.
        self.current = (self.mover.position, self.walker.heading, self.walker.phase, self.walker.bank);
        self.previous = self.current;
        let line = format!(
            "moved Connor from ({:.0}, {:.0}, {:.0}) heading {:.3} to ({:.0}, {:.0}, {:.0}) heading {:.3}{tilt}{rest}",
            old.0[0], old.0[1], old.0[2], old.1, set[0], set[1], set[2], self.walker.heading
        );
        self.log.detail(format!("connor update {}: {line} ({at})", self.updates));
        line
    }

    /// The share of the movement playing now that one update plays, for
    /// blending the drawn frame between updates.
    fn phase_step_now(&self) -> f32 {
        self.movement_loop(self.walker.movement).map_or(0.0, |l| {
            let time = l.play_time();
            // A one-shot's phase always grows, even played backwards
            // (`undraw`; `frame_at` shows those last frame first): blend
            // forward too (M5 5c fix 2: the put-away jittered).
            let speed = if l.repeat { l.speed } else { l.speed.abs() };
            // Hanging on the rope: the climb is moved by the keys, not
            // blended (rule 22).
            let hanging = self.walker.mode == Mode::Rope && self.walker.rope_hang;
            if time > 0.0 && self.walker.mode != Mode::Dead && !hanging { (1.0 / kq8_core::UPDATES_PER_SECOND) as f32 * speed / time } else { 0.0 }
        })
    }

    /// The movement was switched after his own update (the rope state,
    /// M6 6a fix): what is drawn starts from the walker's movement and
    /// phase now, not from where the old movement ended (which drew the
    /// new one's last frame for one update: the user's check, a snap).
    /// The phase the drawing uses now.
    pub fn drawn_phase(&self) -> f32 {
        self.current.2
    }

    pub fn redraw_from_walker(&mut self) {
        let p = self.walker.phase;
        self.current.2 = p;
        self.previous.2 = p;
        self.phase_step = self.phase_step_now();
    }

    /// He is in the rope state (`ropeandhook.md` rules 13–26).
    pub fn roping(&self) -> bool {
        self.rope.is_some()
    }

    /// Puts him at `position` with `heading` at once, not blended from his
    /// old place (the rope state, `ropeandhook.md` rules 15–23).
    pub fn put(&mut self, position: [f32; 3], heading: f32) {
        self.mover.position = position;
        self.walker.heading = heading;
        self.current = (position, heading, self.walker.phase, self.walker.bank);
        self.previous = self.current;
    }

    /// A movement of his by the name the rope state asks for: his own
    /// list's in use (the weapon's first), or `Conrope.anm`'s.
    pub fn rope_movement(&self, anim: kq8_core::rope::Anim) -> Option<&'static str> {
        match anim {
            kq8_core::rope::Anim::Own(n) => self.drawn(n).map(|m| m.name),
            kq8_core::rope::Anim::Rope(n) => {
                let wanted = kq8_core::connor::special_movement_name(kq8_core::rope::ROPE_LIST, n);
                self.movements.iter().find(|m| m.name.eq_ignore_ascii_case(&wanted)).map(|m| m.name)
            }
        }
    }

    /// A movement's loop by its loaded name (for the rope model).
    pub fn loop_of(&self, name: &str) -> Option<&MovementLoop> {
        self.movements.iter().find(|m| m.name == name).map(|m| &m.movement_loop)
    }

    /// `handsOff 0` (`anim-scripts.md` rule 22): every movement key still
    /// held is sent to him again as a press (which clears wall blocks as a
    /// press does), so it acts at once. Returns a line for the log.
    pub fn keys_sent_again(&mut self) -> String {
        let mut sent = Vec::new();
        let keys = [(self.intent.forward == 1, MoveKey::Forward), (self.intent.forward == -1, MoveKey::Back)];
        for (held, key) in keys {
            if held {
                let cleared = self.walker.key_event(key, true);
                sent.push(format!("{key:?} (blocks cleared {cleared:?})"));
            }
        }
        // The held turn key goes down again (Q104 a: a down event).
        if let Some(dir) = self.turn_keys.held_last() {
            let e = self.turn_keys.press(dir, false);
            self.walker.turn_press = true;
            self.intent.turn = self.turn_keys.turn;
            let cleared = self.walker.key_event(MoveKey::Turn, true);
            sent.push(format!("Turn: {e} (blocks cleared {cleared:?})"));
        }
        if sent.is_empty() { "no movement key held".into() } else { format!("held keys sent again: {}", sent.join(", ")) }
    }

    /// `KQConner::setRun` from a script (M3 step 3d-3).
    pub fn set_run(&mut self, run: bool, at: &str) {
        let was = self.intent.run;
        self.intent.run = run;
        self.log.detail(format!(
            "connor update {}: run switch {} -> {} ({at})",
            self.updates,
            if was { "on" } else { "off" },
            if run { "on" } else { "off" }
        ));
    }
}

/// Moves the physical pushables Connor pushed this update (M3 3j-2,
/// `pushing.md` rule 4): their colliders (his and the camera's) and their
/// drawn entities. Runs right after `connor_step`.
pub fn push_apply(
    sim: Option<ResMut<ConnorSim>>,
    colliders: Option<ResMut<ObjectColliders>>,
    view: Option<ResMut<crate::terrain_view::ViewBlockerSet>>,
    mut drawn: Query<(&crate::terrain_view::ObjectName, &mut Transform)>,
    mut doors: Option<ResMut<crate::door_view::DoorSim>>,
    shared: Res<Shared>,
) {
    let Some(mut sim) = sim else { return };
    let (mut colliders, mut view) = (colliders, view);
    for (name, d) in std::mem::take(&mut sim.pushed) {
        let mut boxes = 0;
        let mut at = None;
        for list in [colliders.as_mut().map(|c| &mut c.0), view.as_mut().map(|v| &mut v.boxes)].into_iter().flatten() {
            for b in list.iter_mut().filter(|b| b.name == name) {
                for (p, v) in b.placement.position.iter_mut().zip(d) {
                    *p += v;
                }
                at = Some(b.placement.position);
                boxes += 1;
            }
        }
        // The world's own box of it moves too (its clicks, Connor's
        // swings and `getLoc` go by it; the user's check after M5 5d: a
        // pushed barrel was hit where it had been).
        if let Some(c) = doors.as_mut().and_then(|s| s.active.world.clickables.iter_mut().find(|c| c.name.eq_ignore_ascii_case(&name))) {
            for (p, v) in c.placement.position.iter_mut().zip(d) {
                *p += v;
            }
            boxes += 1;
        }
        let shift = Vec3::from(crate::to_bevy(d, [0.0; 3]));
        let mut entities = 0;
        for (_, mut t) in drawn.iter_mut().filter(|(n, _)| n.0 == name) {
            t.translation += shift;
            entities += 1;
        }
        let line = format!(
            "connor update {}: pushed {name} by ({:.1}, {:.1}, {:.1}): {boxes} collider(s), {entities} drawn part(s) moved; now at {}",
            sim.updates,
            d[0],
            d[1],
            d[2],
            at.map_or("?".into(), |p| format!("({:.1}, {:.1}, {:.1})", p[0], p[1], p[2]))
        );
        if sim.updates % 15 == 0 {
            shared.log.lock().unwrap().detail(&line);
        }
        sim.log.detail(line);
    }
}

/// One update; runs in Bevy's fixed-rate schedule (15 per second).
/// Q98 (`anim-scripts.md` rules 24–25, open question 6; logs only): in
/// the update a scripted walk ends, the walk's phase, its stop transitions
/// and which comes next from that phase; then, once the stop transition
/// has played, the updates it took and how far it carried him.
fn watch_walk_end(sim: &mut ConnorSim, ended: bool) {
    let p = sim.mover.position;
    // This update's number (`sim.updates` grows later in `connor_step`).
    let n = sim.updates + 1;
    if ended {
        let movement = sim.walker.movement;
        let phase = sim.walker.phase;
        let lp = sim.movement_loop(movement);
        let stops: Vec<String> = lp.map_or(Vec::new(), |l| l.transitions.iter().filter(|t| t.to == "stop").map(|t| format!("{} from {:.2}", t.via, t.from)).collect());
        let next = lp.and_then(|l| l.choose_transition(phase, "stop")).map_or("none".to_string(), |(t, d)| format!("{} from {:.2} ({d:.4} of a loop on)", t.via, t.from));
        sim.log.detail(format!(
            "connor update {n}: Q98: the scripted walk ends at ({:.1}, {:.1}), heading {:.3}: {movement} at phase {phase:.4} after this update's move; {movement}'s stop transitions: [{}]; the next from this phase: {next}",
            p[0],
            p[1],
            sim.walker.heading,
            stops.join(", ")
        ));
        sim.walk_end = Some(WalkEnd { update: n, at: p, heading: sim.walker.heading, via: None, first: 0, last: 0, path: 0.0, via_path: 0.0, via_from: p, previous: p });
        return;
    }
    let Some(w) = sim.walk_end.as_mut() else { return };
    let moved = ((p[0] - w.previous[0]).powi(2) + (p[1] - w.previous[1]).powi(2)).sqrt();
    w.path += moved;
    let playing = sim.walker.playing.map(|t| t.via);
    match (w.via, playing) {
        (None, Some(via)) => {
            // It started this update; its share of this update's move is
            // counted with it (the walk played on to the from point first).
            w.via = Some(via);
            w.first = n;
            w.last = n;
            w.via_from = w.previous;
            w.via_path = moved;
        }
        (Some(v), Some(via)) if v == via => {
            w.last = n;
            w.via_path += moved;
        }
        _ => {}
    }
    w.previous = p;
    let done = w.via.is_some() && playing.is_none();
    if done || n > w.update + 60 {
        let w = sim.walk_end.take().expect("watched");
        let straight = ((p[0] - w.at[0]).powi(2) + (p[1] - w.at[1]).powi(2)).sqrt();
        // The transition's own steps in the data (its frames added up, flat).
        let own = |via: &str| {
            sim.drawn(via).map_or("unknown".to_string(), |m| {
                let l = &m.movement_loop;
                let sum = if l.frame_steps.is_empty() { l.step_sum } else { l.frame_steps.iter().fold([0.0; 3], |a, f| [a[0] + f[0], a[1] + f[1], a[2] + f[2]]) };
                format!("{:.1} (left {:.1}, forward {:.1}) over {:.3} s", (sum[0] * sum[0] + sum[1] * sum[1]).sqrt(), sum[0], -sum[1], l.duration)
            })
        };
        let line = match w.via {
            Some(via) => format!(
                "connor update {n}: Q98: {via} (its own steps in the data: {}) played in updates {}-{} ({} updates; the update it started in also played the walk up to its from point): {:.1} along his path from ({:.1}, {:.1}); since the walk ended (update {}): {:.1} along his path, {straight:.1} straight, now at ({:.1}, {:.1}), heading {:.3} -> {:.3}",
                own(via),
                w.first,
                w.last,
                w.last - w.first + 1,
                w.via_path,
                w.via_from[0],
                w.via_from[1],
                w.update,
                w.path,
                p[0],
                p[1],
                w.heading,
                sim.walker.heading
            ),
            None => format!("connor update {n}: Q98: no stop transition seen in 60 updates after the walk ended (update {}); now {}", w.update, sim.walker.movement),
        };
        sim.log.detail(line);
    }
}

#[allow(clippy::too_many_arguments)]
pub fn connor_step(
    sim: Option<ResMut<ConnorSim>>,
    lookup: Option<Res<GroundLookup>>,
    buildings: Option<Res<Buildings>>,
    objects: Option<Res<ObjectColliders>>,
    order: Option<Res<crate::terrain_view::ColliderOrder>>,
    footsounds: Option<Res<FootSounds>>,
    shared: Res<Shared>,
    mut characters: ResMut<crate::door_view::CharacterColliders>,
) {
    let (Some(mut sim), Some(lookup)) = (sim, lookup) else { return };
    let sim = &mut *sim;
    let mut scene = MoverScene::new(
        &lookup.0,
        buildings.as_ref().map_or(&[][..], |b| &b.0),
        objects.as_ref().map_or(&[][..], |o| &o.0),
    );
    scene.not_pushable = sim.not_pushable.clone();
    scene.order = order.as_ref().map(|o| &o.0[..]);
    // M4 4d-12-3: the characters solid for him now.
    let solid = characters.list.clone();
    scene.characters = &solid;
    if let Some((_, what)) = sim.repeat.filter(|_| !sim.hands_off) {
        let cleared = sim.walker.key_event(what, true);
        if cleared != (false, false) {
            let line = format!("connor update {}: {what:?} held (repeat): blocks cleared (forward, back): {cleared:?}", sim.updates);
            shared.log.lock().unwrap().detail(&line);
            sim.log.detail(line);
        }
    }
    // handsOff (Q89 a stand-in): the keys count as let go (the run
    // switch stays as it is).
    let intent = if sim.hands_off { Intent { forward: 0, turn: 0, ..sim.intent } } else { sim.intent };
    let jump = std::mem::take(&mut sim.jump_pressed) && !sim.hands_off;
    let z_before = sim.mover.position[2];
    let tick = {
        let ConnorSim { mover, walker, movements, weapon_set, .. } = &mut *sim;
        let loops = |name: &str| find(movements, weapon_set.as_deref(), name).map(|m| &m.movement_loop);
        kq8_core::connor_update::tick(mover, walker, intent, jump, &loops, &scene)
    };
    // 5d fix 10: the weapon set switch waiting for its list item.
    sim.weapon_step();
    if sim.walker.mode == Mode::Jump && sim.jump_start_z.is_none() {
        sim.jump_start_z = Some(z_before);
    }
    let kq8_core::connor_update::Tick { step, report, wall_stop, notes: jump_notes, auto_end, auto_turn_ended } = tick;
    // Q104 a: the end of an automatic turn sets the turn direction to 0;
    // a held turn key is not sent again.
    if auto_turn_ended {
        sim.turn_keys.auto_turn_ended();
        sim.intent.turn = 0;
    }
    // M3 3n-5 (Q98): the end of a scripted walk, logged.
    let walk_ended = auto_end.as_ref().is_some_and(|a| a.why.starts_with("walked") || a.why == "head-on");
    watch_walk_end(sim, walk_ended);
    // M3 3l-2: a scripted turn or walk ended: its cue.
    sim.auto_ends.extend(auto_end);
    // M3 3j-2: physical pushables moved by his move.
    sim.pushed.extend(scene.moved.take());
    // M3 3j-3: scripted pushables pushed: their scripts get Push.
    for t in report.pushes.iter().filter(|t| t.scripted && t.result == kq8_core::connor_mover::PushResult::Push) {
        if !sim.push_calls.contains(&t.object) {
            sim.push_calls.push(t.object.clone());
        }
    }
    // M3 3i-7: a play-once movement ended: for getEndLoop listeners.
    if let Some(e) = step.as_ref().and_then(|s| s.ended) {
        sim.ended.push(kq8_core::connor::plain_movement_name(e).to_string());
        // M6 6a-3: the rope state's next step (rope_view).
        if sim.walker.mode == Mode::Rope {
            sim.rope_ended.push(e);
        }
        // M6 6b-3: the throw state's next movement (`throw_step`).
        if sim.walker.mode == Mode::Throw {
            sim.throw_ended.push(e);
        }
    }
    sim.phase_step = sim.phase_step_now();
    let wanted = step.as_ref().map_or([0.0; 3], |s| s.wanted);
    sim.updates += 1;
    sim.previous = sim.current;
    let bank_before = sim.current.3;
    sim.current = (sim.mover.position, sim.walker.heading, sim.walker.phase, sim.walker.bank);
    // M4 4d-12-3: his hits on characters, for characters.log.
    for h in report.hits.iter().filter(|h| h.hit.monster) {
        let at = solid.iter().find(|c| c.name == h.hit.what).map_or([0.0; 3], |c| c.position);
        let (t, n, p) = (h.hit.touch, h.hit.normal, h.hit.position);
        let moved = [sim.current.0[0] - sim.previous.0[0], sim.current.0[1] - sim.previous.0[1]];
        let (want, got) = ((wanted[0] * wanted[0] + wanted[1] * wanted[1]).sqrt(), (moved[0] * moved[0] + moved[1] * moved[1]).sqrt());
        characters.hits.push(format!(
            "update {}: {}: Connor's hit ({} pass, {}): touched with his feet at ({:.1}, {:.1}, {:.1}), {:.1} level from its position; put at ({:.1}, {:.1}, {:.1}); surface direction ({:.3}, {:.3}, {:.3}); angle from head-on {}; this update he wanted {want:.1} and moved {got:.1}: {}",
            sim.updates,
            h.hit.what,
            h.pass,
            h.result,
            t[0],
            t[1],
            t[2],
            ((t[0] - at[0]).powi(2) + (t[1] - at[1]).powi(2)).sqrt(),
            p[0],
            p[1],
            p[2],
            n[0],
            n[1],
            n[2],
            h.angle.map_or("none".into(), |a| format!("{:.1} deg", a.to_degrees())),
            if want < 0.001 { "not walking" } else if got < 0.05 * want { "stayed" } else { "slid" }
        ));
    }
    let m = &sim.mover;
    let down_hit = report.hits.iter().find(|h| h.pass == "down");
    // The drawn ground (the water's surface where there is water).
    let surface_z = scene.water(m.position[0], m.position[1]).map_or("none".into(), |(s, _)| format!("{s:.1}"));
    let normal = |n: Option<[f32; 3]>, i: usize| n.map_or("none".into(), |n| format!("{:.3}", n[i]));
    let line = format!(
        "{} {:.3} {:.1} {:.1} {:.1} {} {} {:.2} {} {} {:.1} {} {} {} {} {} {} {} {} {:.1} {} {:.3} {} {} {} {} {} {:.4} {:.3} {:.1} {:.1} {} {} {} {} {} {} {} {} {} {:.2} {:.2} {} {} {} {:.3}",
        sim.updates,
        sim.updates as f64 / kq8_core::UPDATES_PER_SECOND,
        m.position[0],
        m.position[1],
        m.position[2],
        m.on_ground,
        m.use_gravity,
        m.fall_timer,
        m.air_updates,
        m.state.name(),
        report.drop,
        report.ground.map_or("none".into(), |g| format!("{:.1}", g.0)),
        report.ground.map_or("none".into(), |g| format!("{:.3}", g.1[2])),
        report.began_on_ground,
        normal(report.up_contact, 2),
        report.down.name(),
        normal(report.down_normal, 0),
        normal(report.down_normal, 1),
        normal(report.down_normal, 2),
        report.push,
        report.check,
        m.pull[2],
        intent.forward,
        intent.turn,
        intent.run,
        sim.walker.mode.name(),
        sim.walker.movement,
        sim.walker.heading,
        sim.walker.phase,
        wanted[0],
        wanted[1],
        report.across,
        report.hits.len(),
        down_hit.map_or("none".into(), |h| h.hit.what.replace(' ', "_")),
        down_hit.map_or("none".into(), |h| format!("{:.1}", h.hit.position[2] - h.hit.touch[2])),
        sim.walker.blocked_name(),
        step.as_ref().map_or("none", |s| s.played),
        step.as_ref().map_or("none".into(), |s| format!("{:.4}", s.played_from)),
        step.as_ref().map_or("none".into(), |s| format!("{:.4}", s.played_to)),
        sim.walker.wanted.unwrap_or("none"),
        step.as_ref().map_or(0.0, |s| s.own_step[0]),
        step.as_ref().map_or(0.0, |s| -s.own_step[1]),
        sim.jump_start_z.map_or("none".into(), |z| format!("{:.1}", sim.mover.position[2] - z)),
        m.submerged,
        surface_z,
        sim.walker.bank,
    );
    if sim.walker.mode == Mode::Free {
        sim.jump_start_z = None;
    }
    sim.log.detail(line);
    let mut events = jump_notes;
    events.extend(report.events.iter().cloned());
    if report.up_contact.is_some() {
        events.push(format!("up pass hit the terrain (normal z {})", normal(report.up_contact, 2)));
    }
    if let Some(s) = &step {
        if let Some((from, to)) = s.mode_changed {
            events.push(format!("animation mode {} -> {}", from.name(), to.name()));
            if to == Mode::Dead {
                events.push(format!("{} finished: Die state", sim.walker.death.unwrap_or("jumpdie")));
            }
        }
        events.extend(s.notes.iter().cloned());
        if let Some((from, to)) = s.changed {
            // The version drawn (M5 5c fix): the weapon's with one ready.
            let drawn = sim.drawn(to).map_or("none", |m| m.name);
            events.push(if drawn == to { format!("movement {from} -> {to}") } else { format!("movement {from} -> {to} (drawn: {drawn})") });
        }
        if s.missing {
            events.push(format!("movement {} has no loop loaded; not moving", sim.walker.movement));
        }
    }
    let bank = sim.walker.bank;
    if bank_before == 0.0 && bank != 0.0 {
        events.push(format!("lean starts: {bank:.3} rad ({})", if bank > 0.0 { "left" } else { "right" }));
    } else if bank_before != 0.0 && bank == 0.0 {
        events.push(format!("lean back upright (was {bank_before:.3} rad)"));
    }
    let down_now = down_hit.map_or("none".to_string(), |h| h.hit.what.clone());
    if down_now != sim.last_down && down_now != "none" {
        events.push(format!("standing on {down_now} (was {})", sim.last_down));
    }
    // sound.md rule 39a: the building his steps sound like.
    let down = (down_now != "none").then_some(down_now.as_str());
    let is_building = down.is_some_and(|d| footsounds.as_ref().is_some_and(|f| f.0.contains_key(&d.to_ascii_lowercase())));
    if let Some(line) = sim.step_ground.after_update(down, is_building, sim.mover.use_gravity) {
        events.push(line);
    }
    if down_now != "none" {
        sim.last_down = down_now;
    }
    for h in &report.hits {
        // Floor landings are in the columns; log the other building and
        // object hits.
        if h.hit.what != "terrain" && h.result != "landing" {
            events.push(format!(
                "{} pass hit {} ({}): normal ({:.3}, {:.3}, {:.3}), angle {}, put at ({:.1}, {:.1}, {:.1}), touched at ({:.1}, {:.1}, {:.1})",
                h.pass,
                h.hit.what,
                h.result,
                h.hit.normal[0],
                h.hit.normal[1],
                h.hit.normal[2],
                h.angle.map_or("none".into(), |a| format!("{:.1} deg", a.to_degrees())),
                h.hit.position[0],
                h.hit.position[1],
                h.hit.position[2],
                h.hit.touch[0],
                h.hit.touch[1],
                h.hit.touch[2]
            ));
        }
    }
    if let Some((from, to)) = wall_stop {
        events.push(if to == "crash" {
            format!("head-on into a wall: {from} -> {to} from the next update (turn keys turn him; forward, back and run wait until it ends)")
        } else {
            format!("head-on into a wall: movement {from} -> {to}; blocked: {}", sim.walker.blocked_name())
        });
    }
    // M3 3h-4: the cues his movement passed this update (doors.md rule 9,
    // sound.md rules 34-39).
    if let Some(s) = &step {
        let fired: Vec<String> =
            sim.movement_loop(s.played).map(|l| l.cues_between(s.played_from, s.played_to).into_iter().map(String::from).collect()).unwrap_or_default();
        for cue in fired {
            let mut words = cue.split_whitespace();
            let action = words.next().unwrap_or("").to_ascii_lowercase();
            let what = match action.as_str() {
                "footstep" => footstep(sim, &lookup.0, footsounds.as_deref()),
                "playsound" => match words.next() {
                    Some(name) => {
                        let at = sim.mover.position;
                        sim.sounds.push(kq8_core::sound::Start::placed_one_shot(name, at, format!("Connor {} cue", s.played)));
                        "sound started".to_string()
                    }
                    None => "playsound without a file name; nothing".to_string(),
                },
                // fighting.md rule 41 (M5 5c-5): his swing's hit test.
                "do" if sim.swing.as_ref().is_some_and(|w| w.movement == s.played) => {
                    sim.do_cues.push(s.played.to_string());
                    "his hit test (fighting.md rule 41)".to_string()
                }
                // ranged-weapons.md rule 20 (M5 5f-5): the bolt leaves.
                "do" if sim.shot.as_ref().is_some_and(|w| w.started && w.movement == s.played) => {
                    let phase = sim.movement_loop(s.played).and_then(|l| l.cues.iter().find(|(p, a)| a.trim().eq_ignore_ascii_case("do") && *p >= s.played_from.min(s.played_to) - 1e-4).map(|(p, _)| *p)).unwrap_or(s.played_to);
                    sim.shot_cues.push((s.played, phase, false));
                    "his bolt leaves (ranged-weapons.md rule 23)".to_string()
                }
                // azriel-hammer.md rules 3, 5 (M6 6b-3): the hammer leaves.
                "do" if sim.throw.as_ref().is_some_and(|t| t.attack == s.played) && sim.walker.mode == Mode::Throw => {
                    let phase = sim.movement_loop(s.played).and_then(|l| l.cues.iter().find(|(p, a)| a.trim().eq_ignore_ascii_case("do") && *p >= s.played_from.min(s.played_to) - 1e-4).map(|(p, _)| *p)).unwrap_or(s.played_to);
                    sim.throw_cues.push((s.played, phase));
                    "the hammer leaves (azriel-hammer.md rule 3)".to_string()
                }
                "interruptable" => "can be cut short or not from here (anim-scripts.md rule 7)".to_string(),
                _ => "cue action not done for Connor".to_string(),
            };
            events.push(format!("cue '{cue}' of {} fires (phase {:.4} -> {:.4}): {what}", s.played, s.played_from, s.played_to));
        }
    }
    let state = sim.mover.state.name();
    if state != sim.last_state || !events.is_empty() {
        let mut main = shared.log.lock().unwrap();
        for e in &events {
            let e = format!("connor update {}: {e}", sim.updates);
            main.detail(&e);
            sim.log.detail(e);
        }
        if state != sim.last_state {
            main.summary(format!("connor update {}: state {} -> {state}", sim.updates, sim.last_state));
        }
        sim.last_state = state;
    }
}

/// A `FootStep` cue (`sound.md` rules 36-39a): the remembered building's
/// `footsoundType`, else the terrain's flags under him now, give the flags,
/// the flags the set, a roll the file; a plain one-shot. Returns the log
/// text.
fn footstep(sim: &mut ConnorSim, terrain: &kq8_data::Terrain, footsounds: Option<&FootSounds>) -> String {
    use kq8_core::sound::{building_floor_flags, footstep_file, footstep_set, Start};
    let [x, y, _] = sim.mover.position;
    let building = sim.step_ground.building().map(str::to_string);
    let t = building.as_ref().and_then(|b| footsounds.and_then(|f| f.0.get(&b.to_ascii_lowercase())));
    let (on, flags, from) = match (building, t) {
        (Some(b), Some(t)) => {
            let from = match t {
                Some(n) => format!("building footsoundType={n}"),
                None => "building without footsoundType: 0, so 0x1 (rule 39)".to_string(),
            };
            (b, building_floor_flags(*t), from)
        }
        _ => ("terrain".to_string(), terrain.flags_at(x, y), "the terrain's envInfo.kq flags under him (rule 39a)".to_string()),
    };
    let Some(set) = footstep_set(flags) else {
        return format!("footstep on {on} ({from}), flags {flags:#x}: no step sound (rule 37)");
    };
    let roll = sim.dice.roll4();
    let file = footstep_file(set, roll);
    sim.sounds.push(Start::plain_one_shot(&file, format!("Connor's footstep on {on}, flags {flags:#x} -> {set}, roll {roll}")));
    format!("footstep on {on} ({from}), flags {flags:#x} -> {set}, roll {roll} -> {file}")
}

/// Draws Connor between his last two updates (display only): position,
/// heading, and the frame of his current movement.
pub fn connor_draw(
    sim: Option<ResMut<ConnorSim>>,
    fixed: Res<Time<Fixed>>,
    mut body: Query<&mut Transform, (With<ConnorBody>, Without<Camera3d>)>,
    mut visibility: Query<&mut Visibility>,
    mut frames: Query<&mut Transform, (Without<ConnorBody>, Without<Camera3d>)>,
) {
    let Some(mut sim) = sim else { return };
    let a = fixed.overstep_fraction();
    let (p0, h0, _, b0) = sim.previous;
    let (p1, h1, phase, b1) = sim.current;
    let p: [f32; 3] = std::array::from_fn(|i| p0[i] + (p1[i] - p0[i]) * a);
    let heading = h0 + (h1 - h0) * a;
    let bank = b0 + (b1 - b0) * a;
    let mut placement = sim.placement.clone();
    placement.position = p;
    placement.angles[2] = heading;
    // The lean first, about the model's forward axis (−Y; a turn about Y,
    // positive tips his head towards +X, his left), then his placement.
    let lean = kq8_data::objects::rotation([0.0, bank, 0.0]);
    let r = placement.transform().m;
    let m: [[f32; 3]; 3] = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| lean[i][k] * r[k][j]).sum()));
    let t = bevy_transform_from(m, p);
    for mut b in &mut body {
        *b = t;
    }
    // The frame: loop position blended forward from the last update.
    let movement = sim.walker.movement;
    let frame = sim.drawn(movement).and_then(|m| {
        let n = m.frames.len();
        (n > 0).then(|| m.frames[m.movement_loop.frame_at(phase + sim.phase_step * a).min(n - 1)])
    });
    if frame != sim.shown {
        if let Some(old) = sim.shown {
            if let Ok(mut v) = visibility.get_mut(old) {
                *v = Visibility::Hidden;
            }
        }
        if let Some(new) = frame {
            if let Ok(mut v) = visibility.get_mut(new) {
                *v = Visibility::Inherited;
            }
        }
        sim.shown = frame;
    }
    // M6 6a-3 (ropeandhook.md rule 15, Q147 f): the separate rope model,
    // 2,000 along the rope's +y from his place (he stands at the rope's
    // place and direction while it plays, so that is his own +y, his back).
    let rope = sim.rope_model.and_then(|(name, p0, p1)| {
        let m = sim.movements.iter().find(|m| m.name == name)?;
        let n = m.frames.len();
        (n > 0).then(|| m.frames[m.movement_loop.frame_at(p0 + (p1 - p0) * a).min(n - 1)])
    });
    if rope != sim.rope_shown {
        if let Some(old) = sim.rope_shown {
            if let Ok(mut v) = visibility.get_mut(old) {
                *v = Visibility::Hidden;
            }
        }
        if let Some(new) = rope {
            if let Ok(mut v) = visibility.get_mut(new) {
                *v = Visibility::Inherited;
            }
            if let Ok(mut t) = frames.get_mut(new) {
                t.translation = Vec3::from(crate::to_bevy(kq8_core::rope::ROPE_MODEL_OFFSET, [0.0; 3]));
            }
        }
        sim.rope_shown = rope;
    }
}
