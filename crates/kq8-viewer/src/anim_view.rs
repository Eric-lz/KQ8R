//! Animated objects playing their movements in the ground view (M3 steps
//! 3c and 3i-4, docs/DESIGN.md; rules: docs/behaviour/objects.md
//! "Animated objects at load", docs/behaviour/anim-scripts.md).
//!
//! - Since 3i-4 the movements are played by `kq8_core` (`Active`'s
//!   `anims`, the same player as doors and chests), so scripts reach them
//!   (`KQAnimPlayEvent`, `pauseAnim`, `deleteObject`, `EndLoop`). This
//!   file only draws what core says. Unused objects (shown only with H)
//!   are not in the world's scripts: they keep a player of their own here.
//! - Each object is a body entity at its placement with one child per
//!   frame of every movement; only the shown frame is visible (as doors).
//!   If the shown movement has whole-object steps (frame transforms), the
//!   body is moved by the shown frame's chained steps, as doors are
//!   turned. The original adds the steps up and the data's cue puts the
//!   object back at each wrap; for Gnome's rolling rocks both give the
//!   same picture, so this stays (objects.md Q51 rules 1–7, checked by the
//!   user). Such an object's collider is moved to the same pose every
//!   update (Q51 rule 5, M3 step 3r-4).
//! - Hidden and unused objects keep their H-key visibility; see-through
//!   objects keep their opacity.
//!
//! Log: `kq8-artifacts/logs/anim.log`.

use bevy::prelude::*;
use kq8_core::active::AnimObject;
use kq8_core::anim::pose_at_frame;
use kq8_core::collide::Placement;
use kq8_core::door::{Cue, DoorMovement};
use kq8_core::emitter::Rng;
use kq8_core::named_anim::NamedAnimPlayer;
use kq8_data::shape::Transform as GameTransform;

use crate::door_view::DoorSim;
use crate::load::{AnimatedAssets, Animation};
use crate::log::Log;
use crate::object_light::{object_material, ObjectLight, ObjectMaterial};
use crate::terrain_view::{bevy_transform_from, ObjectColliders, ShownWithH};

/// Seed for the `flags` bit 8 random starts and speeds.
const SEED: u64 = 0x0A11_3C00_5EED_0001;

/// Objects whose phase and frame are logged every update (for a while).
const TRACKED: [&str; 4] = ["wellwatr", "windprop", "millwheel", "Davtorch.01"];
const TRACK_UPDATES: u64 = 120;

/// Who plays an object's movements.
enum Source {
    /// `Active`'s `anims[i]` (the world's scripts reach it).
    Core(usize),
    /// Its own player (unused objects, shown only with H).
    Own(Box<NamedAnimPlayer>),
}

/// One movement as drawn: its number of frames and whole-object steps
/// (empty when no frame has one).
struct DrawnMovement {
    frames: usize,
    steps: Vec<GameTransform>,
}

struct Playing {
    name: String,
    source: Source,
    /// `script=` (for a copy brought in by `loadKQ`, M3 3k-4).
    script: Option<String>,
    /// The object's placement (rotation, position).
    rotation: [[f32; 3]; 3],
    position: [f32; 3],
    /// Smooth animation (S-5): its place between the last two updates.
    glide: crate::smooth_view::Glide,
    movements: Vec<DrawnMovement>,
    body: Entity,
    /// (movement, frame) shown.
    shown: Option<(usize, usize)>,
    tracked: bool,
    /// Its entry in `ObjectColliders` (only for objects with steps):
    /// `None` = not looked up yet, `Some(None)` = it has none.
    collider: Option<Option<usize>>,
    /// A character prepared for a script's `loadKQ` (M4 4d-3): its states
    /// and its draw-distance test, handed over when it comes in.
    character: Option<Box<kq8_core::character::Character>>,
    test: Option<crate::draw_distance::DistanceTest>,
    /// Its own copies of the materials showing its face texture, the
    /// image of each mouth shape, and the shape drawn (M4 4d-5, rule 25).
    face: Vec<Handle<ObjectMaterial>>,
    lip_images: Vec<Option<Handle<Image>>>,
    mouth_shown: u16,
    /// Its drawn parts (for `makeTranslucent`, rule 22), the opacity drawn
    /// now, and its own material copies with their opacity at 1 (the
    /// alpha of the colour before any see-through).
    parts: Vec<Entity>,
    opacity_shown: f32,
    own: std::collections::HashMap<AssetId<ObjectMaterial>, (Handle<ObjectMaterial>, f32)>,
    /// A character that casts a shadow (rule 36), and its shadow mesh once
    /// made (M4 4d-7b-2).
    casts: bool,
    shadow: Option<crate::shadow_view::ShadowSlot>,
    /// A character loaded underground: whether its body is shown now
    /// (`character-movement.md` rules 28, 30; M5 5a-8).
    birth_shown: Option<bool>,
    /// Its drawing (Animation memory step 4); `None` if its animations
    /// couldn't be packed.
    new_draw: Option<crate::anim_gpu::Slots>,
}

/// What a new copy of an unused object needs (M5 5d-4): each movement's
/// frames as (mesh, material) lists, its whole-object steps, its script.
struct Recipe {
    /// Per movement: its frames and whole-object steps.
    movements: Vec<(usize, Vec<GameTransform>)>,
    script: Option<String>,
    /// What `Slots::spawn` needs for a copy drawn the new way (step 4e):
    /// per movement its animation, model and ball; its materials.
    slots: (Vec<SlotMovement>, SlotMaterials),
}

/// A copy's materials for `Slots::spawn`: (animation, material index).
type SlotMaterials = Vec<((usize, i32), Handle<ObjectMaterial>)>;

/// A movement for `Slots::spawn`: its animation's index, model and ball.
type SlotMovement = (usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall);

#[derive(Resource)]
pub struct AnimSim {
    playing: Vec<Playing>,
    /// The unused objects' recipes, by lower-case name (M5 5d-4).
    recipes: std::collections::BTreeMap<String, Recipe>,
    updates: u64,
    log: Log,
    /// `characters.log` (M4 4d), if it could be made.
    characters: Option<Log>,
    /// `fight.log` (M5 5c), if it could be made.
    fight: Option<Log>,
}

/// A step that does nothing (no move, no turn), within rounding.
fn is_still(t: &GameTransform) -> bool {
    let id = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    t.p.iter().all(|v| v.abs() < 0.001) && (0..3).all(|i| (0..3).all(|j| (t.m[i][j] - id[i][j]).abs() < 1e-5))
}

/// The pose `pose` (own space) followed by the placement: m = pose.m × R,
/// p = pose.p × R + position (row vectors).
fn combine(pose: &GameTransform, r: &[[f32; 3]; 3], position: [f32; 3]) -> ([[f32; 3]; 3], [f32; 3]) {
    let m = std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| pose.m[i][k] * r[k][j]).sum()));
    let p = std::array::from_fn(|j| (0..3).map(|k| pose.p[k] * r[k][j]).sum::<f32>() + position[j]);
    (m, p)
}

impl AnimSim {
    /// A copy prepared for a script's `loadKQ` comes into the world (M3
    /// 3k-4): its player goes to `active`, which plays it from now on.
    /// `false` if no unused copy by that name plays here.
    pub fn bring_in(&mut self, name: &str, active: &mut kq8_core::active::Active) -> bool {
        let Some(p) = self.playing.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name) && matches!(p.source, Source::Own(_))) else {
            return false;
        };
        let index = active.world.anims.len();
        let Source::Own(player) = std::mem::replace(&mut p.source, Source::Core(index)) else { unreachable!() };
        active.add_anims([AnimObject { player: *player, script: p.script.clone(), gone: false, character: p.character.take(), translucency: None }]);
        true
    }

    /// A new copy of an unused object (M5 5d-4, a script's `loadKQ` while
    /// playing): its body and frames from the template's recipe, at its
    /// place, played by core's `anims[index]`. Returns the log line.
    pub fn spawn_copy(
        &mut self,
        commands: &mut Commands,
        template: &str,
        name: &str,
        placement: Placement,
        index: usize,
        (gpu, materials): (Option<&crate::anim_gpu::AnimGpu>, &mut Assets<ObjectMaterial>),
    ) -> String {
        let Some(r) = self.recipes.get(&template.to_ascii_lowercase()) else {
            return format!("copy {name}: no recipe for {template}; not drawn");
        };
        let body = commands.spawn((bevy_transform_from(placement.rotation, placement.position), Visibility::default(), crate::terrain_view::ObjectName(name.to_string()))).id();
        // Step 4e: drawn the new way too.
        let new_draw = gpu.and_then(|gpu| crate::anim_gpu::Slots::spawn(commands, body, gpu, crate::anim_gpu::List::Objects, r.slots.0.clone(), r.slots.1.clone(), materials, &mut Default::default()));
        let movements: Vec<DrawnMovement> = r.movements.iter().map(|(frames, steps)| DrawnMovement { frames: *frames, steps: steps.clone() }).collect();
        let parts = Vec::new();
        let line = format!(
            "copy {name} of {template}: drawn, {} movement(s), {} part entities, played by the world's rules (M5 5d-4)",
            movements.len(),
            new_draw.as_ref().map_or(0, |s| s.len())
        );
        self.playing.push(Playing {
            name: name.to_string(),
            source: Source::Core(index),
            script: r.script.clone(),
            glide: Default::default(),
            rotation: placement.rotation,
            position: placement.position,
            movements,
            body,
            shown: None,
            tracked: false,
            collider: Some(None),
            character: None,
            test: None,
            face: Vec::new(),
            lip_images: Vec::new(),
            mouth_shown: 0,
            parts,
            opacity_shown: 1.0,
            own: Default::default(),
            casts: false,
            shadow: None,
            birth_shown: None,
            new_draw,
        });
        line
    }

    /// The draw-distance test of a character that just came in (M4 4d-3),
    /// with its body, to be put on it.
    pub fn take_test(&mut self, name: &str) -> Option<(Entity, crate::draw_distance::DistanceTest)> {
        self.playing.iter_mut().find(|p| p.name.eq_ignore_ascii_case(name) && p.test.is_some()).and_then(|p| Some((p.body, p.test.take()?)))
    }

    /// `deleteObject` (3f): its collider is no longer moved.
    pub fn forget_collider(&mut self, name: &str) {
        for p in self.playing.iter_mut().filter(|p| p.name.eq_ignore_ascii_case(name)) {
            p.collider = Some(None);
        }
    }
}

/// The player of `p`: in core, or its own.
fn player<'a>(p: &'a Playing, doors: Option<&'a DoorSim>) -> Option<&'a NamedAnimPlayer> {
    match &p.source {
        Source::Core(i) => doors.and_then(|d| d.active.world.anims.get(*i)).map(|a| &a.player),
        Source::Own(player) => Some(player),
    }
}

/// Builds the objects and returns the ones the world's scripts can reach,
/// in order, for `Active::add_anims` (3i-4). `placed` is
/// `Objects::placed`.
#[allow(clippy::too_many_arguments)]
pub fn spawn_animated(
    commands: &mut Commands,
    animated: Vec<AnimatedAssets>,
    animations: Vec<Animation>,
    placed: &[(kq8_data::objects::Placement, usize, bool)],
    materials: &mut Assets<ObjectMaterial>,
    light: &ObjectLight,
    images: &mut Assets<Image>,
    balls: &crate::draw_distance::ShapeBalls,
    character_lines: &[String],
    main_log: &mut Log,
    gpu: &crate::anim_gpu::AnimGpu,
) -> (Vec<AnimObject>, Vec<NamedAnimPlayer>) {
    let mut log = match Log::create("anim") {
        Ok(l) => l,
        Err(e) => {
            main_log.summary(format!("animated objects: cannot create anim.log: {e}; not started"));
            return (Vec::new(), Vec::new());
        }
    };
    // Per animation: its number of frames, the materials by index, its
    // length and steps.
    struct Built {
        frames: usize,
        materials: Vec<(i32, Handle<ObjectMaterial>)>,
        duration: f32,
        steps: Vec<GameTransform>,
        moves: bool,
        /// The loop starts at the file's first frame (Q164).
        first_in_file: bool,
        /// Each frame's lights (M1 7c, rule 26b).
        lights: std::sync::Arc<Vec<Vec<kq8_data::shape::ModelLight>>>,
        /// Each material's texture file name (for a character's face).
        textures: Vec<(i32, Option<String>)>,
        /// Its `.kqs` ball (rule 37 (2), M4 4d-7b-5b).
        ball: crate::shadow_view::ModelBall,
        /// Each frame's named parts (`ranged-weapons.md` rule 24, M5 5f-2).
        parts: Vec<kq8_data::shape::FrameParts>,
        /// Its model (step 4).
        source: Option<crate::load::FrameShape>,
    }
    let built: Vec<Built> = animations
        .into_iter()
        .map(|a| {
            let textures = a
                .looks
                .iter()
                .map(|(m, look, _)| match look {
                    crate::load::Look::Texture { name, .. } => (*m, Some(name.rsplit(['/', '\\']).next().unwrap_or(name).to_string())),
                    _ => (*m, None),
                })
                .collect();
            let materials = a.looks.into_iter().map(|(m, look, l)| (m, materials.add(object_material(look, l, light, images)))).collect();
            let frames = a.frame_count;
            let source = a.source.clone();
            let moves = a.frame_transforms.iter().skip(1).any(|t| !is_still(t));
            let ball = crate::shadow_view::ModelBall::of(a.ball);
            Built { frames, materials, duration: a.duration, steps: a.frame_transforms, moves, first_in_file: a.first_in_file, lights: std::sync::Arc::new(a.frame_lights), textures, ball, parts: a.frame_parts, source }
        })
        .collect();
    let mut rng = Rng::new(SEED);
    let mut characters: Vec<String> = Vec::new();
    let mut playing = Vec::new();
    let mut core = Vec::new();
    // M5 5d-4: the unused objects' players and recipes (templates).
    let mut own_players = Vec::new();
    let mut recipes = std::collections::BTreeMap::new();
    let mut moving = Vec::new();
    let mut characters_new = 0;
    let (mut random, mut switchable) = (0, 0);
    let mut stop_starts: Vec<String> = Vec::new();
    for a in animated {
        let (p, shape_index, unused) = &placed[a.placed];
        random += usize::from(a.random);
        let opacity = p.opacity();
        let rotation = kq8_data::objects::rotation(p.angles);
        // The movements as core plays them (anim-scripts.md rules 1–10).
        let movements: Vec<DoorMovement> = a
            .movements
            .iter()
            .map(|(name, l, i)| DoorMovement {
                name: name.clone(),
                start: l.start,
                speed: l.speed,
                repeat: l.repeat,
                cut_short: l.unknown != 0,
                play_time: built[*i].duration,
                frame_steps: built[*i].steps.clone(),
                first_in_file: built[*i].first_in_file,
                frame_parts: built[*i].parts.clone(),
                cues: l.cues.iter().map(|(phase, action)| Cue { phase: *phase, action: action.clone() }).collect(),
                // characters.md rule 20 (M4 4d-11).
                transitions: l.transitions.clone(),
            })
            .collect();
        // ranged-weapons.md rule 24 (M5 5f-2): where a bolt would start, per
        // frame of each movement that has an `arrow` part (for the checks).
        for m in &movements {
            let arrow: Vec<String> = m
                .frame_parts
                .iter()
                .enumerate()
                .filter_map(|(k, parts)| parts.iter().find(|(n, _)| n.eq_ignore_ascii_case("arrow")).map(|(_, q)| format!("{k}: ({:.0}, {:.0}, {:.0})", q.p[0], q.p[1], q.p[2])))
                .collect();
            if !arrow.is_empty() {
                log.detail(format!("[{}] '{}' ({} frames) part arrow by frame: {}", p.name, m.name, m.frames(), arrow.join(", ")));
            }
        }
        let names = (a.anim_names.0.as_deref(), a.anim_names.1.as_deref());
        // anim-scripts.md rule 3 as corrected 2026-10-19 (M4 4d-8): both
        // names start as `Stop`, the keys replace them.
        let mut player = NamedAnimPlayer::new(&p.name, movements, a.current, Placement { rotation, position: p.position }).with_stop_names(names.0, names.1).with_anim_pos(a.anim_pos);
        // anim-scripts.md rule 11 (M5 5d-1).
        player.once = a.once;
        if player.current != a.current {
            let was = player.movements.get(a.current).map_or("none", |m| m.name.as_str()).to_string();
            stop_starts.push(format!("{} starts on '{}' (movement {}), not '{was}' (movement {}): no currAnimName, so the name Stop (anim-scripts.md rule 3 corrected)", p.name, player.movement().map_or("none", |m| m.name.as_str()), player.current, a.current));
        }
        if a.random {
            player = player.with_random_start(&mut rng);
        }
        // character-movement.md rule 1 (M5 5a-2).
        player.loops_move_base = a.character.is_some();
        // Q164: its walk and run speeds by `objects.md` "Loops" rule 6a.
        if player.loops_move_base {
            let speeds: Vec<String> = player
                .movements
                .iter()
                .filter(|m| m.repeat && (m.name.eq_ignore_ascii_case("walk") || m.name.eq_ignore_ascii_case("run")))
                .map(|m| format!("{} {:.0} units/s ({} frames, {:.4} s, from the file's first frame: {})", m.name, m.loop_ground_speed(m.speed), m.frames(), m.play_time, m.first_in_file))
                .collect();
            if !speeds.is_empty() {
                log.detail(format!("[{}] character speeds (objects.md \"Loops\" rule 6a): {}", p.name, speeds.join("; ")));
            }
        }
        let mut body = commands.spawn((bevy_transform_from(rotation, p.position), Visibility::default(), crate::terrain_view::ObjectName(p.name.clone())));
        // A character not born yet (characters.md rule 16): not drawn;
        // only with H, as hidden objects.
        let unborn = a.character.as_ref().is_some_and(|c| c.unborn);
        if *unused || !p.drawn() || unborn {
            body.insert((Visibility::Hidden, ShownWithH));
        }
        // Draw distance (M1 6h); unused objects are only shown with H.
        // A character's ball is its rule-9 box.
        // A character's states (M4 4d-3).
        let mut character = a.character.as_ref().map(|c| {
            let has_birth = a.movements.iter().any(|(n, _, _)| n.eq_ignore_ascii_case("Birth"));
            Box::new(kq8_core::character::Character::new(
                c.keys.class == kq8_data::characters::Class::GeometryTalker,
                c.keys.flags,
                c.birth_delay,
                has_birth,
                c.keys.no_turn,
                c.keys.gravity,
                kq8_core::character::Character::seed_for(&p.name),
            ))
        });
        // characters.md rule 8 (M4 4d-6b-2): its mover.
        if let (Some(ch), Some(c)) = (character.as_deref_mut(), &a.character) {
            let k = &c.keys;
            ch.flying = k.flying;
            ch.hits_buildings = k.collider_mask & 0x8 != 0;
            // M4 4d-12-3: what Connor's scene needs to make it solid.
            (ch.collision_type, ch.collision_mask) = (k.collider_type, k.collider_mask);
            ch.bounds = balls.character_box(&p.name, c.box_type.as_deref());
            // character-movement.md (M5 5a-4): its rules; home = where it
            // was placed; the eye at the top of its box (rule 20).
            let eye = ch.bounds.map_or(1000.0, |(_, max)| max[2]);
            let has_hide = a.movements.iter().any(|(n, _, _)| n.eq_ignore_ascii_case("hide"));
            ch.unborn = c.unborn;
            let mut ai = kq8_core::character_ai::Ai::new(c.ai.clone(), p.position, eye, has_hide);
            // fighting.md rule 2 (M5 5c-1): its stats.
            ch.fight = c.fight.clone().map(|s| {
                let mut f = kq8_core::fight::Fight::new(s);
                f.combat_scaled = c.combat_scaled;
                f
            });
            ai.has_birth = a.movements.iter().any(|(n, _, _)| n.eq_ignore_ascii_case("Birth"));
            ch.ai = Some(ai);
            if let Some((min, max)) = ch.bounds {
                let keep = match p.keep_on_ground {
                    kq8_data::objects::KeepOnGround::Floor => -1,
                    kq8_data::objects::KeepOnGround::Surface => 1,
                    kq8_data::objects::KeepOnGround::Off => 0,
                };
                let step = kq8_core::connor_mover::STEP;
                ch.mover = Some(Box::new(kq8_core::connor_mover::Mover::character(
                    p.position,
                    k.gravity,
                    min,
                    max,
                    (k.collider_type, k.collider_mask),
                    (k.step_up.unwrap_or(step), k.step_down.unwrap_or(step)),
                    keep,
                )));
            }
        }
        let mut prepared_test = None;
        if *unused {
            if let Some(c) = &a.character {
                prepared_test = Some(balls.test_character(p, c.box_type.as_deref()));
            }
        }
        if !*unused {
            let test = match &a.character {
                Some(c) => {
                    let mut test = balls.test_character(p, c.box_type.as_deref());
                    if c.unborn {
                        // Rule 16: groupsAlways plus bit 1 only, no draw
                        // or collider bits (its own again once risen).
                        test.born_groups = Some(test.groups);
                        test.groups = [p.groups_always | 1, 0, 0];
                    }
                    test
                }
                None => balls.test_at(p, *shape_index),
            };
            if let Some(c) = &a.character {
                let l = &a.movements[player.current].1;
                characters.push(format!(
                    "[{}] {}: from {}; at ({:.0}, {:.0}, {:.0}) heading {:.2}; {} movements, shown at load '{}' ({} {} speed {} repeat {} cut short {}); flags {:#x}{}{}; ball from {} (centre ({:.0}, {:.0}, {:.0}) m, radius {:.0}); wanders {}; birthDelay {}{}{}{}; movement rules: {}",
                    p.name,
                    c.keys.class.name(),
                    c.from,
                    p.position[0],
                    p.position[1],
                    p.position[2],
                    p.angles[2],
                    a.movements.len(),
                    a.movements[player.current].0,
                    l.shape,
                    l.sequence,
                    l.speed,
                    l.repeat,
                    l.unknown != 0,
                    c.keys.flags,
                    if c.keys.no_turn { ", noTurn" } else { "" },
                    if c.keys.gravity { "" } else { ", gravity off" },
                    test.ball_from,
                    test.centre.x,
                    test.centre.y,
                    test.centre.z,
                    test.radius,
                    c.wanders,
                    c.birth_delay,
                    if c.unborn { "; has Birth: NOT DRAWN until born (rule 16)" } else { "" },
                    c.lips.as_ref().map_or(String::new(), |l| format!("; lips [{}] face {}", l.section, l.face)),
                    c.skin.as_ref().map_or(String::new(), |(f, t)| format!("; skin {f} -> {t}")),
                    c.ai.describe(),
                ));
            }
            body.insert(test);
            // The lights it carries (M1 7c), by the movement and frame shown.
            let lights = a.movements.iter().map(|(_, _, i)| built[*i].lights.clone()).collect();
            if let Some(c) = crate::scene_lights::CarriedLights::new(lights, bevy_transform_from(rotation, p.position), p.turn_off_dyn_lighting) {
                body.insert(c);
            }
        }
        let body = body.id();
        // A character's mouth (M4 4d-5, rule 25): its own copy of every
        // material showing the face texture, in every movement, so the
        // shape is kept across movements and not shared with others.
        let face_name = a.character.as_ref().and_then(|c| c.lips.as_ref()).map(|l| l.face.clone());
        let lip_images: Vec<Option<Handle<Image>>> = a
            .lip_looks
            .iter()
            .map(|l| match l {
                Some(crate::load::Look::Texture { width, height, rgba, .. }) => {
                    Some(images.add(crate::object_light::texture_image(*width, *height, rgba.clone(), bevy::image::ImageAddressMode::Repeat)))
                }
                _ => None,
            })
            .collect();
        let parts = Vec::new();
        // Steps 4b, 4d, 4e: every animated object is drawn the new way
        // too (the unused ones, copy templates, only with H; their copies
        // from the recipe).
        let own: SlotMaterials = a
            .movements
            .iter()
            .flat_map(|(_, _, i)| built[*i].materials.iter().map(move |(m, h)| ((*i, *m), h.clone())))
            .map(|(k, h)| (k, crate::object_light::for_object(materials, &h, opacity, p.haze_solid, p.haze_multiplier)))
            .collect();
        let slot_movements: Vec<SlotMovement> = a.movements.iter().map(|(_, _, i)| (*i, built[*i].source.clone(), built[*i].ball)).collect();
        let new_draw = crate::anim_gpu::Slots::spawn(commands, body, gpu, crate::anim_gpu::List::Objects, slot_movements.clone(), own.clone(), materials, &mut Default::default());
        characters_new += usize::from(a.character.is_some() && new_draw.is_some());
        // Step 4d: the new way's face materials take the mouth shapes too.
        let new_faces: Vec<Handle<ObjectMaterial>> = match (&new_draw, &face_name) {
            (Some(nd), Some(f)) => a
                .movements
                .iter()
                .flat_map(|(_, _, i)| built[*i].textures.iter().filter(|(_, t)| t.as_ref().is_some_and(|t| t.eq_ignore_ascii_case(f))).map(move |(m, _)| (*i, *m)))
                .filter_map(|k| nd.materials.get(&k).cloned())
                .collect(),
            _ => Vec::new(),
        };
        let drawn: Vec<DrawnMovement> = a
            .movements
            .iter()
            .map(|(_, _, i)| DrawnMovement { frames: built[*i].frames, steps: if built[*i].moves { built[*i].steps.clone() } else { Vec::new() } })
            .collect();
        let recipe: Vec<(usize, Vec<GameTransform>)> = drawn.iter().map(|d| (d.frames, d.steps.clone())).collect();
        if *unused && a.character.is_none() {
            recipes.insert(p.name.to_ascii_lowercase(), Recipe { movements: recipe, script: a.script.clone(), slots: (slot_movements, own) });
        }
        // Whole-object steps of the movement shown at load: how far the
        // chained pose goes over the loop.
        let b = &built[a.movements[player.current].2];
        let mut reach = String::from("none");
        if b.moves {
            let (mut far, mut turn) = (0.0f32, 0.0f32);
            for k in 0..b.steps.len() {
                let pose = pose_at_frame(&b.steps, k);
                far = far.max(pose.p.iter().map(|v| v * v).sum::<f32>().sqrt());
                turn = turn.max(kq8_core::door::turn_degrees(&pose).abs());
            }
            let last = pose_at_frame(&b.steps, b.steps.len().saturating_sub(1));
            reach = format!(
                "up to {far:.1} units and {turn:.1} deg about z; at the last frame ({:.1}, {:.1}, {:.1}), {:.1} deg",
                last.p[0],
                last.p[1],
                last.p[2],
                kq8_core::door::turn_degrees(&last)
            );
            moving.push(p.name.clone());
        }
        let (_, l, _) = &a.movements[player.current];
        let more = if a.movements.len() > 1 {
            switchable += 1;
            format!("; movements {:?}", a.movements.iter().map(|(n, _, _)| n.as_str()).collect::<Vec<_>>())
        } else {
            String::new()
        };
        log.detail(format!(
            "[{}] {} frames, {:.4} s, start {} speed {} repeat {} random {} -> phase {:.4} speed {:.4}, frame {}; whole-object steps: {reach}{}{more}",
            p.name,
            b.frames,
            b.duration,
            l.start,
            l.speed,
            l.repeat,
            a.random,
            player.phase,
            player.speed(),
            player.frame(),
            if *unused || !p.drawn() { "; hidden (H)" } else { "" }
        ) + if player.current_name.is_empty() && player.waiting.is_some() {
            // anim-scripts.md rule 3 (Q85).
            "; no currAnimName: the first update starts the waiting movement plainly (phase from its loop= line, its own speed; the random part is lost)"
        } else {
            ""
        });
        let tracked = TRACKED.iter().any(|t| t.eq_ignore_ascii_case(&p.name));
        let source = if *unused {
            if a.character.is_none() {
                own_players.push(player.clone());
            }
            Source::Own(Box::new(player))
        } else {
            core.push(AnimObject { player, script: a.script.clone(), gone: false, character: character.take(), translucency: None });
            Source::Core(core.len() - 1)
        };
        playing.push(Playing {
            name: p.name.clone(),
            source,
            script: a.script.clone(),
            glide: Default::default(),
            rotation,
            position: p.position,
            movements: drawn,
            body,
            shown: None,
            tracked,
            collider: None,
            character,
            test: prepared_test,
            face: new_faces,
            lip_images,
            mouth_shown: 0,
            parts,
            opacity_shown: opacity,
            own: Default::default(),
            casts: a.character.as_ref().is_some_and(|c| c.casts_shadow),
            shadow: None,
            birth_shown: unborn.then_some(false),
            new_draw,
        });
    }
    let new_way = playing.iter().filter(|p| p.new_draw.is_some()).count();
    let slots: usize = playing.iter().filter_map(|p| p.new_draw.as_ref()).map(|n| n.len()).sum();
    main_log.summary(format!(
        "animated objects drawn from the compact data (Animation memory step 4): {new_way} of {} (characters among them: {characters_new}), {slots} part entities",
        playing.len()
    ));
    main_log.summary(format!(
        "animated objects: {} playing ({} run by the world's scripts, {} with more than one movement, {} with a random start and speed, flags bit 8), {} different loops; whole-object steps in {} objects {moving:?}; log in {}",
        playing.len(),
        core.len(),
        switchable,
        random,
        built.len(),
        moving.len(),
        log.path().display()
    ));
    main_log.summary(format!("animated objects and characters: {} start on another movement than the loader's because the names start as Stop (M4 4d-8); listed in the log", stop_starts.len()));
    for line in &stop_starts {
        main_log.detail(format!("names Stop: {line}"));
    }
    // Characters (M4 4d-2): one line each, and the loader's lines.
    let mut clog = Log::create("characters").ok();
    let drawn = characters.len();
    if let Some(c) = clog.as_mut() {
        for line in character_lines.iter().chain(&characters) {
            c.detail(line);
        }
    }
    main_log.summary(format!(
        "characters: {drawn} drawn from their movement lists ({} not born yet: hidden), {} loader notes; log in kq8-artifacts/logs/characters.log",
        characters.iter().filter(|l| l.contains("NOT DRAWN")).count(),
        character_lines.len()
    ));
    let fight = Log::create("fight").ok();
    if let Some(f) = fight.as_ref() {
        main_log.summary(format!("fight: log in {}", f.path().display()));
    }
    commands.insert_resource(AnimSim { playing, recipes, updates: 0, log, characters: clog, fight });
    (core, own_players)
}

/// One update (15 per second), after `door_step` has run core's update:
/// unused objects move their own loops; an object whose shown movement
/// moves it takes its collider along (3r-4).
pub fn anim_step(sim: Option<ResMut<AnimSim>>, colliders: Option<ResMut<ObjectColliders>>, doors: Option<ResMut<DoorSim>>) {
    let Some(mut sim) = sim else { return };
    let sim = &mut *sim;
    sim.updates += 1;
    let n = sim.updates;
    let mut doors = doors;
    // characters.log (M4 4d-3): core's lines, and every 10 s how many
    // characters are in each state.
    if let Some(d) = doors.as_mut() {
        // fight.log (M5 5c): core's fight lines.
        let fight = std::mem::take(&mut d.active.world.fight_log);
        if let Some(log) = sim.fight.as_mut() {
            for l in fight {
                log.detail(l);
            }
        }
        let lines = std::mem::take(&mut d.active.world.character_log);
        if let Some(log) = sim.characters.as_mut() {
            for l in lines {
                log.detail(l);
            }
            if n % 150 == 0 {
                let w = &d.active.world;
                let mut states: std::collections::BTreeMap<String, usize> = Default::default();
                for c in w.anims.iter().filter(|a| !a.gone).filter_map(|a| a.character.as_deref()) {
                    let key = if c.first_update.is_none() { "not updated yet".to_string() } else { format!("{:?}", c.state) };
                    *states.entry(key).or_default() += 1;
                }
                log.detail(format!("update {}: characters near or far now: {}; by state {states:?}", w.updates, w.characters_in_range.len()));
            }
        }
    }
    let doors = doors.as_deref();
    let mut colliders = colliders;
    for p in &mut sim.playing {
        if let Source::Own(player) = &mut p.source {
            player.advance(1.0 / kq8_core::UPDATES_PER_SECOND as f32);
        }
        let Some(player) = player(p, doors) else { continue };
        let (movement, frame) = (player.current, player.frame());
        if p.tracked && n <= TRACK_UPDATES {
            sim.log.detail(format!("update {n} [{}] phase {:.4} frame {}{}", p.name, player.phase, frame, if player.has_ended() { " finished" } else { "" }));
        }
        let Some(drawn) = p.movements.get(movement) else { continue };
        if drawn.steps.is_empty() {
            continue;
        }
        let Some(colliders) = colliders.as_mut() else { continue };
        let slot = *p.collider.get_or_insert_with(|| {
            let found: Vec<usize> = colliders.0.iter().enumerate().filter(|(_, c)| c.name == p.name).map(|(i, _)| i).collect();
            match found[..] {
                [i] => {
                    sim.log.detail(format!("[{}] its collider moves with it (objects.md Q51 rule 5)", p.name));
                    Some(i)
                }
                [] => {
                    sim.log.detail(format!("[{}] has no collider: nothing to move", p.name));
                    None
                }
                _ => {
                    sim.log.detail(format!("[{}] {} colliders by that name: none moved", p.name, found.len()));
                    None
                }
            }
        });
        let Some(i) = slot else { continue };
        // Switched off (out of the collision world, or deleted): stays off.
        if colliders.0[i].min == colliders.0[i].max {
            continue;
        }
        let frame = frame.min(drawn.frames.saturating_sub(1));
        let (rotation, position) = combine(&pose_at_frame(&drawn.steps, frame), &p.rotation, p.position);
        colliders.0[i].placement = kq8_core::collide::Placement { rotation, position };
        if n % 15 == 0 && n <= 150 {
            sim.log.detail(format!(
                "update {n} [{}] frame {frame}: collider at ({:.0}, {:.0}, {:.0})",
                p.name, position[0], position[1], position[2]
            ));
        }
    }
}

/// Each object shows its current movement's frame (and pose, if that
/// movement moves it).
#[allow(clippy::too_many_arguments)]
pub fn anim_draw(
    mut commands: Commands,
    sim: Option<ResMut<AnimSim>>,
    doors: Option<Res<DoorSim>>,
    mut transforms: Query<&mut Transform>,
    mut visibility: Query<&mut Visibility>,
    mut carried: Query<&mut crate::scene_lights::CarriedLights>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut part_materials: Query<&mut MeshMaterial3d<ObjectMaterial>>,
    gpu: Option<Res<crate::anim_gpu::AnimGpu>>,
    fixed: Res<Time<Fixed>>,
) {
    let Some(mut sim) = sim else { return };
    let doors = doors.as_deref();
    let sim = &mut *sim;
    let ahead = crate::smooth_view::ahead(&fixed);
    let smooth = crate::smooth_view::on();
    let (update, a) = (doors.map_or(sim.updates, |d| d.active.world.updates), fixed.overstep_fraction());
    for p in &mut sim.playing {
        // makeTranslucent (characters.md rule 22, M4): every material of
        // every part at n / 255, at once (objects.md "See-through
        // objects" rule 2); the object gets its own copies first.
        let wanted = match (&p.source, doors) {
            (Source::Core(i), Some(d)) => d.active.world.anims.get(*i).and_then(|a| a.translucency),
            _ => None,
        };
        if let Some(n) = wanted {
            let opacity = if n >= 256.0 { 1.0 } else { (n / 255.0).clamp(0.0, 1.0) };
            if opacity != p.opacity_shown {
                let mut changed = 0;
                // Shared material → this object's copy (one per material).
                let mut copies: std::collections::HashMap<AssetId<ObjectMaterial>, Handle<ObjectMaterial>> = Default::default();
                for e in &p.parts {
                    let Ok(mut mm) = part_materials.get_mut(*e) else { continue };
                    let id = mm.0.id();
                    if !p.own.contains_key(&id) {
                        let Some(m) = materials.get(&mm.0).cloned() else { continue };
                        // Its alpha at opacity 1: the colour's, undone.
                        let full = if p.opacity_shown > 0.0 && m.alpha_mode == AlphaMode::Blend { m.settings.base_colour.w / p.opacity_shown } else { m.settings.base_colour.w };
                        if p.face.iter().any(|h| h.id() == id) {
                            // Already its own (the mouth's copies, 4d-5).
                            p.own.insert(id, (mm.0.clone(), full));
                        } else {
                            let h = copies.entry(id).or_insert_with(|| materials.add(m)).clone();
                            p.own.insert(h.id(), (h.clone(), full));
                            mm.0 = h;
                        }
                    }
                    changed += 1;
                }
                // Step 4: the new way's materials get own copies too.
                if let Some(nd) = p.new_draw.as_mut() {
                    for h in nd.materials.values_mut() {
                        if p.own.contains_key(&h.id()) {
                            continue;
                        }
                        let Some(m) = materials.get(&*h).cloned() else { continue };
                        let full = if p.opacity_shown > 0.0 && m.alpha_mode == AlphaMode::Blend { m.settings.base_colour.w / p.opacity_shown } else { m.settings.base_colour.w };
                        if p.face.iter().any(|f| f.id() == h.id()) {
                            // Already its own (the mouth's, 4d).
                            p.own.insert(h.id(), (h.clone(), full));
                            continue;
                        }
                        let copy = materials.add(m);
                        p.own.insert(copy.id(), (copy.clone(), full));
                        *h = copy;
                    }
                    nd.forget_shown();
                }
                for (h, full) in p.own.values() {
                    if let Some(mut m) = materials.get_mut(h) {
                        m.settings.base_colour.w = full * opacity;
                        m.alpha_mode = if opacity < 1.0 { AlphaMode::Blend } else { AlphaMode::Opaque };
                    }
                }
                if let Some(log) = sim.characters.as_mut() {
                    log.detail(format!("update {}: {}: makeTranslucent {n}: opacity {:.3} -> {opacity:.3} on {changed} part(s), {} own material(s) (rule 22)", sim.updates, p.name, p.opacity_shown, p.own.len()));
                }
                p.opacity_shown = opacity;
            }
        }
        // A character's mouth shape (M4 4d-5, rule 25): lip<n> on its face.
        let mouth = match (&p.source, doors) {
            (Source::Core(i), Some(d)) => d.active.world.anims.get(*i).and_then(|a| a.character.as_deref()).map(|c| c.mouth),
            _ => None,
        };
        if let Some(shape) = mouth.filter(|m| *m != p.mouth_shown && !p.face.is_empty()) {
            let n = if shape > 7 { 0 } else { shape as usize };
            let what = match p.lip_images.get(n).cloned().flatten() {
                Some(img) => {
                    for h in &p.face {
                        if let Some(mut m) = materials.get_mut(h) {
                            m.texture = Some(img.clone());
                        }
                    }
                    format!("lip{n} drawn on {} face material(s)", p.face.len())
                }
                None => format!("lip{n} has no texture: face unchanged"),
            };
            p.mouth_shown = shape;
            if let Some(log) = sim.characters.as_mut() {
                log.detail(format!("update {}: {}: mouth shape {shape}: {what} (rule 25)", sim.updates, p.name));
            }
        }
        // M5 5a-8: a zombie rises (shown) or goes back underground.
        if let (Some(shown), Source::Core(i), Some(d)) = (p.birth_shown, &p.source, doors) {
            let up = d.active.world.anims.get(*i).and_then(|a| a.character.as_deref()).is_some_and(|c| !c.unborn);
            if up != shown {
                if let Ok(mut v) = visibility.get_mut(p.body) {
                    *v = if up { Visibility::Inherited } else { Visibility::Hidden };
                }
                p.birth_shown = Some(up);
                if let Some(log) = sim.characters.as_mut() {
                    log.detail(format!("update {}: {}: {} (character-movement.md rules 28, 30)", sim.updates, p.name, if up { "risen: drawn" } else { "underground: not drawn" }));
                }
            }
        }
        let Some(player) = player(p, doors) else { continue };
        let movement = player.current.min(p.movements.len().saturating_sub(1));
        let Some(drawn) = p.movements.get(movement) else { continue };
        // Smooth animation (S-4): the frame and how far toward the next.
        let (frame, mix) = crate::smooth_view::frame_of(player, ahead);
        let frame = frame.min(drawn.frames.saturating_sub(1));
        // A character's repeating movement moves its place, not the
        // drawing (character-movement.md rule 1; M5 5a-2).
        let loop_moves_base = player.loops_move_base && player.movement().is_some_and(|m| m.repeat);
        // Its place may have changed (a script's `move`, objects.md Q20 d;
        // core keeps it): the body follows (M4 4d-2).
        let base = player.base;
        let moved = base.rotation != p.rotation || base.position != p.position;
        if moved {
            (p.rotation, p.position) = (base.rotation, base.position);
        }
        let same = p.shown == Some((movement, frame)) && !moved && p.new_draw.as_ref().is_none_or(|nd| !nd.stale(movement, frame));
        if same {
            if let (Some(nd), Some(gpu)) = (p.new_draw.as_mut(), gpu.as_deref()) {
                nd.set_amount(&mut commands, gpu, mix);
            }
            // Smooth animation (S-5) glides its place every drawn frame.
            if !smooth {
                continue;
            }
        } else {
            if let Ok(mut c) = carried.get_mut(p.body) {
                (c.movement, c.frame) = (movement, frame);
            }
            p.shown = Some((movement, frame));
            let join = match (p.new_draw.as_mut(), gpu.as_deref()) {
                (Some(nd), Some(gpu)) => nd.show_mixed(&mut commands, gpu, movement, frame, mix, &mut visibility),
                _ => None,
            };
            // Smooth joins (J-3): a change of movement drawn mixed.
            if let (Some(j), Some(now), Some(log)) = (join, self::player(p, doors), sim.characters.as_mut()) {
                let name = |k: usize| now.movements.get(k).map_or("?", |m| m.name.as_str());
                log.detail(format!("update {}: {}: {}", sim.updates, p.name, j.line(name(j.from), name(j.to))));
            }
        }
        let mut pose = if drawn.steps.is_empty() { kq8_core::door::identity() } else { pose_at_frame(&drawn.steps, frame) };
        // Smooth animation (S-5): the frame's steps mixed toward the next
        // frame's (not across a loop's wrap).
        if smooth && mix > 0 && frame + 1 < drawn.steps.len() {
            pose = kq8_core::door::mix_transform(&pose, &pose_at_frame(&drawn.steps, frame + 1), mix as f32 / 255.0);
        }
        if loop_moves_base {
            pose.p = [0.0; 3];
        }
        // Only bodies that move or just stopped moving need a new pose.
        if moved || smooth || !drawn.steps.is_empty() || p.movements.iter().any(|m| !m.steps.is_empty()) {
            let place = p.glide.drawn(update, kq8_core::collide::Placement { rotation: p.rotation, position: p.position }, a);
            let (m, at) = combine(&pose, &place.rotation, place.position);
            if let Ok(mut t) = transforms.get_mut(p.body) {
                *t = bevy_transform_from(m, at);
            }
        }
    }
}

/// `characters.md` rules 36, 37 (M4 4d-7b-2): each character that casts a
/// shadow and is drawn (near or far, not hidden), and Connor (4d-7b-2b),
/// get, every frame, the triangles of their shown model frame thrown along
/// the sun onto the terrain (`kq8_core::shadow`), drawn unlit with multiply
/// blending at the strength 0.5, so each layer halves what is under it and
/// overlaps darken twice (the user's check); no haze. Runs after the
/// transforms are propagated, so it sees this frame's pose. Buildings do
/// not receive them yet (4d-7b-3). Once a second a line per shadow in
/// `characters.log`.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn shadow_draw(
    mut commands: Commands,
    sim: Option<ResMut<AnimSim>>,
    doors: Option<Res<DoorSim>>,
    connor: Option<Res<crate::connor_view::ConnorSim>>,
    connor_body: Query<(&GlobalTransform, &Visibility), With<crate::connor_view::ConnorBody>>,
    light: Option<Res<ObjectLight>>,
    ground: Option<Res<crate::terrain_view::GroundLookup>>,
    children: Query<&Children>,
    parts: crate::shadow_view::PartQuery,
    visibility: Query<&Visibility>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut material: Local<Option<Handle<StandardMaterial>>>,
    mut connor_shadow: Local<Option<crate::shadow_view::ShadowSlot>>,
    mut frames: Local<u64>,
    (buildings, activity, mut spent, balls, change, mut seen, sources): (
        Option<Res<crate::shadow_view::BuildingShadows>>,
        Option<Res<crate::draw_distance::ObjectActivity>>,
        Local<std::time::Duration>,
        Query<(&GlobalTransform, &crate::shadow_view::ModelBall)>,
        Res<crate::world_build::WorldChange>,
        Local<u64>,
        Option<Res<crate::shadow_view::FrameSources>>,
    ),
) {
    use kq8_core::shadow;
    let no_sources = crate::shadow_view::FrameSources::default();
    let sources = sources.as_deref().unwrap_or(&no_sources);
    // A new world (M6 6e-2): Connor's shadow is made again.
    if crate::world_build::new_world(&change, &mut seen) {
        *connor_shadow = None;
        *frames = 0;
        *spent = std::time::Duration::ZERO;
    }
    let (Some(mut sim), Some(ground)) = (sim, ground) else { return };
    let started = std::time::Instant::now();
    let sim = &mut *sim;
    *frames += 1;
    let report = (*frames).is_multiple_of(60);
    let dir = shadow::direction(light.as_ref().and_then(|l| l.lights.suns.first()).map(|s| s.aim));
    let material = material.get_or_insert_with(|| crate::shadow_view::shadow_material(&mut materials)).clone();
    // Rule 37 (2): the near and far buildings receive too (M4 4d-7b-3).
    let active = buildings.as_deref().map(|b| b.active(activity.as_deref())).unwrap_or_default();
    let mut lines = Vec::new();
    // On log frames only: how many of the pieces lie on buildings.
    let line = |name: &str, model: &[[shadow::V3; 3]], on: &crate::shadow_view::Receivers, cast: usize, above: f32| {
        let pick = &on.pick;
        let search = pick.search.map_or("none (no downward sun)".to_string(), |(lo, hi)| format!("{:.0} x {:.0} x {:.0} (own space, z {:.0}..{:.0})", hi[0] - lo[0], hi[1] - lo[1], hi[2] - lo[2], lo[2], hi[2]));
        format!(
            "update {}: {name}: shadow: {} model triangles, {} facing the sun, {} terrain triangles; buildings: s {:.0} ({}), box {search}, {} of {} active buildings in reach {:?}, {} building surfaces ({} triangles); {cast} pieces drawn, {} of them on buildings; {above:.0} above the drawn ground{} (rule 37)",
            sim.updates,
            model.len(),
            model.iter().filter(|t| shadow::faces_sun(t, dir)).count(),
            on.terrain,
            pick.s,
            if on.fallback { "OUR FALLBACK: box around its points" } else { "its model's stored ball" },
            pick.in_reach.len(),
            active.len(),
            pick.in_reach,
            pick.surfaces,
            on.triangles.len() - on.terrain,
            shadow::cast(model, dir, &on.triangles[on.terrain..]).len(),
            if above.abs() > shadow::TERRAIN_REACH { ": terrain beyond 1,500, none there" } else { "" }
        )
    };
    let height_above = |p: shadow::V3| p[2] - ground.0.height_at(p[0], p[1]).map_or(f32::NAN, |g| g.height);
    // Connor (the user's check: he casts one; his own cast test is not in
    // the notes, Q130): while his body is shown.
    if let Some(c) = connor.as_ref() {
        let body = connor_body.iter().next();
        let drawn = body.is_some_and(|(_, v)| *v != Visibility::Hidden);
        let shown = c.shown;
        let model = match shown.filter(|_| drawn) {
            Some(frame) => crate::shadow_view::frame_triangles(frame, &children, &parts, &meshes, sources),
            None => Vec::new(),
        };
        let at = body.map_or([0.0; 3], |(t, _)| {
            let v = t.translation();
            [v.x * 100.0, -v.z * 100.0, v.y * 100.0]
        });
        let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
        let caster = shown.filter(|_| drawn).and_then(|f| balls.get(f).ok()).map(|(at, b)| crate::shadow_view::caster(at, b));
        let on = crate::shadow_view::receivers(&ground.0, at, &corners, dir, caster, &active);
        let cast = shadow::cast(&model, dir, &on.triangles);
        if report && drawn {
            lines.push(line("Connor", &model, &on, cast.len(), height_above(at)));
        }
        crate::shadow_view::show_shadow(&cast, &mut connor_shadow, &mut commands, &mut meshes, &material);
    }
    let Some(doors) = doors else { return };
    let world = &doors.active.world;
    for p in sim.playing.iter_mut().filter(|p| p.casts) {
        let Source::Core(i) = p.source else { continue };
        let Some(a) = world.anims.get(i) else { continue };
        let drawn = !a.gone && world.characters_in_range.contains(&p.name.to_ascii_lowercase()) && visibility.get(p.body).is_ok_and(|v| *v != Visibility::Hidden);
        // Its parts shown (step 4: the slots).
        let frame = p.shown.filter(|_| drawn).and(p.new_draw.as_ref().map(|nd| nd.root()));
        let model = match frame {
            Some(frame) => crate::shadow_view::frame_triangles(frame, &children, &parts, &meshes, sources),
            None => Vec::new(),
        };
        let corners: Vec<shadow::V3> = model.iter().flatten().copied().collect();
        let caster = frame.and_then(|f| balls.get(f).ok()).map(|(at, b)| crate::shadow_view::caster(at, b));
        let on = crate::shadow_view::receivers(&ground.0, a.player.base.position, &corners, dir, caster, &active);
        let cast = shadow::cast(&model, dir, &on.triangles);
        if report && drawn {
            lines.push(line(&p.name, &model, &on, cast.len(), height_above(a.player.base.position)));
        }
        crate::shadow_view::show_shadow(&cast, &mut p.shadow, &mut commands, &mut meshes, &material);
    }
    *spent += started.elapsed();
    if report {
        lines.push(format!("update {}: shadows of Connor and the characters: {:.3} ms a frame on average (rule 37); {}", sim.updates, spent.as_secs_f64() * 1000.0 / *frames as f64, sources.line()));
    }
    if let Some(log) = sim.characters.as_mut() {
        for l in lines {
            log.detail(l);
        }
    }
}
