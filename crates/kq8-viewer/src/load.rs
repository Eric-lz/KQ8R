//! Finding a model, its textures and the palette in the game files, and
//! turning them into plain triangle lists and RGBA images. No Bevy here.

use std::collections::BTreeMap;
use std::path::Path;

use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::palette::Rgb;
use kq8_data::shape::{bounds, Triangle};
use kq8_data::{Bitmap, GameDir, Lighting, MaterialKind, Palette, Shape, VolArchive};

use crate::log::Log;

/// The archives of one world, opened once.
pub struct World {
    pub name: String,
    vols: Vec<(String, VolArchive)>,
}

impl World {
    pub fn open(game: &GameDir, name: &str) -> Result<Self, String> {
        let canonical = WORLDS
            .iter()
            .find(|w| w.eq_ignore_ascii_case(name))
            .ok_or(format!("unknown world '{name}'; known: {}", WORLDS.join(", ")))?;
        let mut vols = Vec::new();
        for v in VOLUMES {
            if let Some(path) = game.volume_path(canonical, v) {
                let a = VolArchive::open(&path).map_err(|e| format!("{}: {e}", path.display()))?;
                vols.push((v.to_string(), a));
            }
        }
        Ok(World { name: canonical.to_string(), vols })
    }

    /// Unpacks the first entry called `name` (any case) from any archive.
    /// Returns the archive it came from as `vol:<World>/<volume>/<NAME>`.
    pub fn load(&self, name: &str) -> Option<Result<(String, Vec<u8>), String>> {
        self.vols.iter().find_map(|(v, a)| {
            let e = a.find(name)?;
            let label = format!("vol:{}/{v}/{}", self.name, e.name);
            Some(a.unpack(e).map(|u| (label, u.data)).map_err(|e| e.to_string()))
        })
    }

    /// Every entry whose name ends with `ext` (any case), unpacked, with
    /// its file name and archive label.
    pub fn entries_ending(&self, ext: &str) -> Vec<(String, String, Result<Vec<u8>, String>)> {
        let ext = ext.to_ascii_lowercase();
        self.vols
            .iter()
            .flat_map(|(v, a)| {
                a.entries().iter().filter(|e| e.name.to_ascii_lowercase().ends_with(&ext)).map(move |e| {
                    let label = format!("vol:{}/{v}/{}", self.name, e.name);
                    (e.name.clone(), label, a.unpack(e).map(|u| u.data).map_err(|e| e.to_string()))
                })
            })
            .collect()
    }
}

/// Everything the window needs, already converted.
pub struct Model {
    pub label: String,
    /// One group per material index that has triangles.
    pub groups: Vec<Group>,
    /// Model-space bounds in game units (Z up).
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Which sequence was drawn and why, for the log.
    pub frame_note: String,
    /// Buildings only: the light maps of all surfaces in one picture (M1
    /// 5c, `building_light`); `None` for models.
    pub light_atlas: Option<crate::building_light::LightAtlas>,
    /// Buildings only: each surface's corners in model space (game units),
    /// for the walls' haze (M1 6f, object-lighting.md rule 29); empty for
    /// models.
    pub surfaces: Vec<Vec<[f32; 3]>>,
    /// The lights the model carries in the shown frame (object-lighting.md
    /// rule 26b, M1 7c-1), model space.
    pub lights: Vec<kq8_data::shape::ModelLight>,
    /// Buildings with a usable `.dil` only: what point lights on its
    /// surfaces need (M1 7d, rule 16).
    pub wall_light: Option<std::sync::Arc<crate::wall_lights::WallData>>,
    /// The ball stored in its `.kqs` file (radius, centre in its own
    /// space; `characters.md` rule 37 (2), Q135); `None` for buildings.
    pub ball: Option<(f32, [f32; 3])>,
}

pub struct Group {
    pub material: i32,
    pub triangles: Vec<Triangle>,
    pub look: Look,
    /// How the material is lit and hazed (object-lighting.md rules 7, 20).
    pub shading: Shading,
    /// Buildings only: per triangle, its corners' coordinates in the
    /// building's light picture (same corner order); empty for models.
    pub light_uvs: Vec<[[f32; 2]; 3]>,
}

/// How a model material is lit and hazed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Shading {
    /// object-lighting.md rule 7.
    pub lighting: Lighting,
    /// Material flag `0x10000`: half the haze (rule 20).
    pub haze_halved: bool,
}

impl Shading {
    /// Full brightness, normal haze (buildings, missing materials).
    pub const PLAIN: Shading = Shading { lighting: Lighting::None, haze_halved: false };
}

/// A model material's lighting kind (object-lighting.md rule 7) and haze
/// flag (rule 20). Shading bits the note doesn't cover, or no material:
/// full brightness (STAND-IN; no model material in the data has other
/// bits).
pub fn material_shading(m: Option<&kq8_data::Material>) -> Shading {
    Shading {
        lighting: m.and_then(|m| m.lighting()).unwrap_or(Lighting::None),
        haze_halved: m.is_some_and(|m| m.flags & 0x10000 != 0),
    }
}

/// Which animation frame to draw: (sequence/loop name, start 0..1, repeats).
pub type StartAt<'a> = Option<(&'a str, f32, bool)>;

pub enum Look {
    Texture { name: String, width: u32, height: u32, rgba: Vec<u8> },
    Color([u8; 3]),
    /// No usable material: drawn in a warning colour.
    Missing(String),
}

pub fn load_model(
    game: &GameDir,
    world: &World,
    model: &str,
    palette_spec: Option<&str>,
    start: StartAt,
    log: &mut Log,
) -> Result<Model, String> {
    let (label, bytes) = find_model(game, world, model)?;
    log.summary(format!("model: {label} ({} bytes)", bytes.len()));
    let shape = Shape::parse(&bytes).map_err(|e| format!("{label}: {e}"))?;
    let (rest, frame_note) = match start {
        None => (shape.rest_sequence(), "first frame".to_string()),
        Some((name, at, repeat)) => match shape.start_sequence(name, at, repeat) {
            Some(f) => {
                // Frame transforms are steps the whole object makes during
                // an animation (models-and-textures.md Q1 b). Starting past
                // frame 0 would mean the earlier steps have been taken;
                // they are not applied here, so say whether there are any.
                let first = f.sequence - f.frame;
                let moving = (first + 1..=f.sequence).any(|q| {
                    shape.sequences.get(q).is_some_and(|s| s.transform.p != [0.0; 3] || s.transform.rotates())
                });
                (
                    f.sequence,
                    format!(
                        "frame {} of {} in loop '{name}'{}{}",
                        f.frame,
                        f.frames,
                        if f.fell_back { " (no loop by that name; first loop used)" } else { "" },
                        if moving { " WARNING: earlier frames move the object; those steps are not applied" } else { "" }
                    ),
                )
            }
            None => (shape.rest_sequence(), format!("loop '{name}' has no frames; first frame")),
        },
    };
    let geometry = shape.geometry(rest);
    log.summary(format!(
        "shape: v{} nodes={} sequences={} loops={} meshes={} lights={} rest_sequence={}",
        shape.version,
        shape.nodes.len(),
        shape.sequences.len(),
        shape.loops.len(),
        shape.meshes.len(),
        shape.lights.len(),
        rest
    ));
    for p in &geometry.problems {
        log.summary(format!("  geometry problem: {p}"));
    }
    let (min, max) = bounds(&geometry.triangles).ok_or(format!("{label}: sequence {rest} ({frame_note}) has no triangles"))?;
    log.summary(format!(
        "rest pose: triangles={} min=({:.1}, {:.1}, {:.1}) max=({:.1}, {:.1}, {:.1}) (game units, Z up)",
        geometry.triangles.len(),
        min[0], min[1], min[2], max[0], max[1], max[2]
    ));

    let palette = load_palette(world, palette_spec, game, log)?;
    let list = shape.materials.as_ref().map(|m| m.materials.as_slice()).unwrap_or(&[]);
    let mut by_material: BTreeMap<i32, Vec<Triangle>> = BTreeMap::new();
    for t in geometry.triangles {
        by_material.entry(t.material).or_default().push(t);
    }
    let mut groups = Vec::new();
    for (index, triangles) in by_material {
        let look = match usize::try_from(index).ok().and_then(|i| list.get(i)) {
            None => Look::Missing(format!("material index {index} not in list of {}", list.len())),
            Some(m) => match m.kind() {
                MaterialKind::Texture => load_texture(world, game, &m.texture, &palette, log),
                MaterialKind::Rgb => Look::Color(m.rgb),
                k => Look::Missing(format!("material kind {k:?}")),
            },
        };
        let what = match &look {
            Look::Texture { name, width, height, .. } => format!("texture {name} {width}x{height}"),
            Look::Color(c) => format!("colour {c:?}"),
            Look::Missing(why) => format!("MISSING ({why}), drawn magenta"),
        };
        let m = usize::try_from(index).ok().and_then(|i| list.get(i));
        let shading = material_shading(m);
        log.summary(format!(
            "  material {index}: flags={} triangles={} {shading:?} -> {what}",
            m.map_or("-".into(), |m| format!("0x{:X}", m.flags)),
            triangles.len()
        ));
        groups.push(Group { material: index, triangles, look, shading, light_uvs: Vec::new() });
    }
    let lights = shape.sequence_lights(rest);
    if !lights.is_empty() {
        log.summary(format!("  carries {} light(s) in its shown frame (rule 26b)", lights.len()));
    }
    Ok(Model { label, groups, min, max, frame_note, light_atlas: None, surfaces: Vec::new(), lights, wall_light: None, ball: Some((shape.radius, shape.center)) })
}

/// The movements M2 uses: step 2b (connor-movement.md "Which movement
/// plays"), the falls of step 2c/2e (connor-physics.md "Falling off an
/// edge"), the crash (2d-4), the jumps (2e), the start and stop transitions (2g,
/// connor-movement.md "Starting and stopping") and wading (2h, water.md).
/// Transitions to or via a movement not listed here are skipped. M4 4e-7
/// adds his gestures; M5 5c-4 his flinch and deaths from a hit.
pub const CONNOR_MOVEMENTS: [&str; 56] = [
    "stop",
    "walk",
    "run",
    "back",
    "left",
    "right",
    "jumphold",
    "jumpend",
    "jumpdie",
    "crash",
    "stopwalk",
    "stoprun",
    "walkstopleft",
    "stopwalkstop",
    "walkstopright",
    "runstopright",
    "runstopleft",
    "stoprunstop",
    "jump",
    "runJump",
    "runJumpHold",
    "runJumpDie",
    "jumprun",
    "jumpstop",
    "backJump",
    "backJumpHold",
    "backJumpEnd",
    "backJumpDie",
    "waterwalk",
    "waterrun",
    "walkwaterwalk",
    "waterwalkwalk",
    "runwaterrun",
    "waterrunrun",
    // Pushing (M3 3j, pushing.md rule 6) and its stops.
    "push",
    "pushleft",
    "pushright",
    // His gestures (M4 4e-7, conversation.md rule 18: lip-sync codes 8-15).
    "gesturea",
    "gestureb",
    "gesturec",
    "gestured",
    "gesturee",
    "gesturef",
    "gestureg",
    "gestureh",
    // Hit (M5 5c-4, fighting.md rule 29): the flinch, the deaths.
    "react",
    "die",
    "dropweapon",
    "firedie",
    "icedie",
    // Leaving and arriving through portals (M6 6e-4b, world-change.md
    // rules 4, 20; the portal files' enterAnim / exitAnim from his own
    // list).
    "teleportdown",
    "teleportup",
    "out",
    "portal",
    "portalx",
    "walkend",
];

/// One of Connor's movements: its animation and playback speed.
pub struct ConnorMovement {
    pub name: &'static str,
    /// Index into `ConnorAssets::animations`.
    pub animation: usize,
    pub speed: f32,
    /// True = repeats; false = plays once.
    pub repeat: bool,
    /// `loop=` 6th field: can be cut short.
    pub cut_short: bool,
    /// Its `transitionN` lines whose movements are all loaded.
    pub transitions: Vec<kq8_core::connor::Transition>,
    /// Its `cueN` lines (M3 3h-4).
    pub cues: Vec<(f32, String)>,
}

pub struct ConnorAssets {
    pub movements: Vec<ConnorMovement>,
    pub animations: Vec<Animation>,
    /// `MASK/Connor.sae`.
    pub keys: Vec<kq8_data::action_map::Binding>,
    /// `MASK/Camera.sae`.
    pub camera_keys: Vec<kq8_data::action_map::Binding>,
    /// His collision box, the `Connor` entry of `GAME/patch/BOUNDS.KQ`
    /// (connor-physics.md "Collision shape"); `None` if it can't be read.
    pub collision_box: Option<([f32; 3], [f32; 3])>,
    /// His mouth (M4 4a-6, `talking.md` rule 18): the face texture
    /// `baseLip=` and the looks of `lip0`..`lip7` of his `lips=` list
    /// (`None` where missing); `None` without them.
    pub lips: Option<(String, Vec<Option<Look>>)>,
    /// What the potions change on him (M5 5g-5, `potions.md` rules 25,
    /// 30).
    pub potion_looks: PotionLooks,
    /// The weapons whose sets were not loaded, and what loading one while
    /// playing needs (M6 6c-2).
    pub late: Option<LateSets>,
}

/// M6 6c-2: what the viewer needs to load a weapon's movement set while
/// playing (a weapon given that no file of this world names): the game
/// folder, the world, Connor's palette and face texture, and for each such
/// weapon its item type (lower case, `invitem_…`) and list file.
pub struct LateSets {
    pub game: GameDir,
    pub world: String,
    pub palette: [Rgb; 256],
    pub lip_base: Option<String>,
    pub sets: Vec<(String, String)>,
}

/// A movement list file (`conSword.anm`): from the world's archives, else
/// `GAME/Kq` (M5 5c fix; M6 6c-2).
pub fn read_set(game: &GameDir, world: &World, file: &str) -> Vec<kq8_data::kq_text::Section> {
    let bytes = match world.load(file) {
        Some(Ok((_, b))) => Some(b),
        _ => game.find(&format!("GAME/Kq/{file}")).and_then(|p| std::fs::read(p).ok()),
    };
    bytes.map(|b| kq8_data::kq_text::parse(&b)).unwrap_or_default()
}

/// The `I_` version of a texture this world has (`potions.md` rule 29):
/// for a set loaded while playing (M6 6c-2).
pub fn ghost_texture(world: &World, texture: &str, palette: &[Rgb; 256]) -> Option<Look> {
    load_ghost_texture(world, texture, palette)
}

/// Connor's skin and face lists and the pictures the potions need (M5
/// 5g-5): his `skinList` and `lipsList`, his body skin's texture (the
/// skin list's entry for no bits, `curSkin=0`), the Strength entry's
/// texture, and every face set of the face list (`<name>0`..`<name>7`).
#[derive(Default)]
pub struct PotionLooks {
    pub skins: kq8_data::characters::SkinList,
    pub faces: kq8_data::characters::SkinList,
    pub body_skin: Option<String>,
    pub iron: Option<(String, Look)>,
    pub face_sets: Vec<(String, Vec<Option<Look>>)>,
    /// M5 5g-6: his textures' `I_` versions in this world (his texture's
    /// name, lower case; the `I_` picture with entry 0 see-through).
    pub ghosts: Vec<(String, Look)>,
}

/// Every frame of one animation loop, for drawing it in turn (M2 step 2b).
pub struct Animation {
    /// One look and shading per material index used by any frame.
    pub looks: Vec<(i32, Look, Shading)>,
    /// Per frame: triangles grouped by material index (model space).
    pub frames: Vec<Vec<(i32, Vec<Triangle>)>>,
    /// Loop length in seconds.
    pub duration: f32,
    /// The frames' whole-body steps added up (model space).
    pub step_sum: [f32; 3],
    /// Each frame's own step (model space), frame 0 first.
    pub frame_steps: Vec<[f32; 3]>,
    /// Each frame's whole step, turn included (doors turn by these;
    /// doors.md rule 24).
    pub frame_transforms: Vec<kq8_data::shape::Transform>,
    /// Each frame's lights (object-lighting.md rule 26b, M1 7c-1), model
    /// space.
    pub frame_lights: Vec<Vec<kq8_data::shape::ModelLight>>,
    /// The ball stored in its `.kqs` file (radius, centre in its own
    /// space; `characters.md` rule 37 (2), Q135).
    pub ball: (f32, [f32; 3]),
    /// Each frame's named model parts and their own places
    /// (`ranged-weapons.md` rule 24; M5 5f-2).
    pub frame_parts: Vec<Vec<(String, [f32; 3])>>,
}

/// One movement set's own versions of Connor's movements (M2; the
/// weapons' sets: M5 5c fix, 5d fix 9): each of [`CONNOR_MOVEMENTS`] the
/// list `set_list` (in `sections`) has, its animation read once (shared by
/// shape and sequence through `index_of`), named `<file>/<name>` for a
/// weapon's list (`file`), plain for his own. Used at start-up and, for a
/// weapon given while playing, by the viewer (M6 6c).
#[allow(clippy::too_many_arguments)]
pub fn connor_set(
    game: &GameDir,
    world: &World,
    palette: &[Rgb; 256],
    log: &mut Log,
    sections: &[kq8_data::kq_text::Section],
    set_list: &str,
    file: Option<&str>,
    movements: &mut Vec<ConnorMovement>,
    animations: &mut Vec<Animation>,
    index_of: &mut BTreeMap<String, usize>,
) -> Result<(), String> {
    use kq8_data::kq_text;
    let mut missing = Vec::new();
    for word in CONNOR_MOVEMENTS {
        let mut with_ask = sections.to_vec();
        with_ask.extend(kq_text::parse(format!("[ConnorAsks]\nanimLoops={set_list}\ncurrAnimName={word}\n").as_bytes()));
        let c = kq8_data::objects::current_loop(&with_ask, with_ask.last().unwrap())?.ok_or("no animation list")?;
        if c.fell_back {
            if file.is_none() {
                return Err(format!("Conner.anm has no movement called {word}"));
            }
            missing.push(word);
            continue;
        }
        let name = match file {
            Some(f) => kq8_core::connor::intern(&kq8_core::connor::special_movement_name(f, word)),
            None => word,
        };
        let key = format!("{}|{}", c.shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase());
        let animation = match index_of.get(&key) {
            Some(&i) => i,
            None => {
                log.quiet = true;
                let a = load_animation(game, world, &c.shape, &c.sequence, palette, log);
                log.quiet = false;
                animations.push(a?);
                index_of.insert(key, animations.len() - 1);
                animations.len() - 1
            }
        };
        let a = &animations[animation];
        let speed_now = if a.duration > 0.0 {
            (a.step_sum[0].powi(2) + a.step_sum[1].powi(2)).sqrt() / a.duration * c.speed
        } else {
            0.0
        };
        log.summary(format!(
            "connor: movement {name} = movement {} loop={} {} speed {} repeat {}: {} frames, {:.4} s, steps ({:.1}, {:.1}, {:.1}) -> {speed_now:.1} units/s",
            c.movement, c.shape, c.sequence, c.speed, c.repeat, a.frames.len(), a.duration, a.step_sum[0], a.step_sum[1], a.step_sum[2]
        ));
        // Transitions: only those whose movements are loaded
        // (connor-movement.md "Starting and stopping").
        let known = |n: &str| CONNOR_MOVEMENTS.into_iter().find(|k| k.eq_ignore_ascii_case(n));
        let mut transitions = Vec::new();
        for t in &c.transitions {
            let line = format!("{} {} {} {}", t.from, t.to, t.to_phase, t.via);
            match (known(&t.to), known(&t.via)) {
                (Some(to), Some(via)) => {
                    log.summary(format!("connor: {name}: transition {line}"));
                    transitions.push(kq8_core::connor::Transition { from: t.from, to, to_phase: t.to_phase, via });
                }
                _ => log.summary(format!("connor: {name}: transition {line} skipped (movement not loaded)")),
            }
        }
        for p in &c.transition_problems {
            log.summary(format!("connor: {name}: {p}"));
        }
        if !c.cues.is_empty() {
            let cues: Vec<String> = c.cues.iter().map(|(ph, a)| format!("{ph} {a}")).collect();
            log.detail(format!("connor: {name}: cues {}", cues.join(", ")));
        }
        for p in &c.cue_problems {
            log.summary(format!("connor: {name}: {p}"));
        }
        if !c.repeat || !c.transitions.is_empty() {
            let steps: Vec<String> =
                a.frame_steps.iter().map(|p| format!("({:.2}, {:.2}, {:.2})", p[0], p[1], p[2])).collect();
            log.detail(format!("connor: {name}: can be cut short {}; frame steps {}", c.unknown != 0, steps.join(" ")));
        }
        movements.push(ConnorMovement {
            name,
            animation,
            speed: c.speed,
            repeat: c.repeat,
            cut_short: c.unknown != 0,
            transitions,
            cues: c.cues.clone(),
        });
    }
    if let Some(f) = file {
        log.summary(format!("connor: weapon set {f}: its own versions of his movements loaded; not in it (his own play): {missing:?}"));
    }
    Ok(())
}

/// Loads all frames of loop `loop_name` of `model`. Frame transforms are
/// whole-body steps, not drawn (models-and-textures.md Q1 b); they are
/// added up into `step_sum` (connor-movement.md "Forward speed comes from
/// the animation").
pub fn load_animation(
    game: &GameDir,
    world: &World,
    model: &str,
    loop_name: &str,
    palette: &[Rgb; 256],
    log: &mut Log,
) -> Result<Animation, String> {
    load_animation_skinned(game, world, model, loop_name, palette, None, log)
}

/// [`load_animation`] with a character's skin swap (`characters.md` rule
/// 4): wherever the model uses texture `skin.0`, `skin.1` is drawn instead
/// (names compared ignoring capitals, as for the mouth textures, rule 25).
pub fn load_animation_skinned(
    game: &GameDir,
    world: &World,
    model: &str,
    loop_name: &str,
    palette: &[Rgb; 256],
    skin: Option<&(String, String)>,
    log: &mut Log,
) -> Result<Animation, String> {
    let (label, bytes) = find_model(game, world, model)?;
    let shape = Shape::parse(&bytes).map_err(|e| format!("{label}: {e}"))?;
    let named = shape
        .loops
        .iter()
        .find(|l| usize::try_from(l.name).ok().and_then(|n| shape.names.get(n)).is_some_and(|n| n.eq_ignore_ascii_case(loop_name)));
    // A name not in the model: the first loop plays (objects.md Q13 b).
    let l = match named {
        Some(l) => l,
        None => {
            let first = shape.loops.first().ok_or(format!("{label}: no loops"))?;
            let name = usize::try_from(first.name).ok().and_then(|n| shape.names.get(n)).map_or("?", |n| n.as_str());
            log.summary(format!("{label}: no loop called '{loop_name}'; using the first loop '{name}' (objects.md Q13 b)"));
            first
        }
    };
    let (first, count) = (l.first_sequence.max(0) as usize, l.sequence_count.max(0) as usize);
    let mut frames = Vec::with_capacity(count);
    let mut step_sum = [0.0f32; 3];
    let mut frame_steps = Vec::with_capacity(count);
    let mut frame_transforms = Vec::with_capacity(count);
    let mut frame_lights = Vec::with_capacity(count);
    let mut frame_parts = Vec::with_capacity(count);
    let mut turning = 0;
    for seq in first..first + count {
        let s = shape.sequences.get(seq).ok_or(format!("{label}: loop '{loop_name}' runs past the sequences"))?;
        if s.transform.rotates() {
            turning += 1;
        }
        for (sum, v) in step_sum.iter_mut().zip(s.transform.p) {
            *sum += v;
        }
        frame_steps.push(s.transform.p);
        frame_transforms.push(s.transform);
        frame_lights.push(shape.sequence_lights(seq));
        frame_parts.push(shape.part_places(seq));
        let geometry = shape.geometry(seq);
        for p in &geometry.problems {
            log.detail(format!("  {label} frame {}: geometry problem: {p}", seq - first));
        }
        let mut by_material: BTreeMap<i32, Vec<Triangle>> = BTreeMap::new();
        for t in geometry.triangles {
            by_material.entry(t.material).or_default().push(t);
        }
        frames.push(by_material.into_iter().collect::<Vec<_>>());
    }
    let list = shape.materials.as_ref().map(|m| m.materials.as_slice()).unwrap_or(&[]);
    let used: std::collections::BTreeSet<i32> = frames.iter().flatten().map(|(m, _)| *m).collect();
    let looks = used
        .into_iter()
        .map(|index| {
            let material = usize::try_from(index).ok().and_then(|i| list.get(i));
            let look = match material {
                None => Look::Missing(format!("material index {index} not in list of {}", list.len())),
                Some(m) => match m.kind() {
                    MaterialKind::Texture => match skin.filter(|(from, _)| from.eq_ignore_ascii_case(&m.texture)) {
                        Some((from, to)) => {
                            log.summary(format!("{model}: skin swap: texture {from} drawn as {to} (characters.md rule 4)"));
                            load_texture(world, game, to, palette, log)
                        }
                        None => load_texture(world, game, &m.texture, palette, log),
                    },
                    MaterialKind::Rgb => Look::Color(m.rgb),
                    k => Look::Missing(format!("material kind {k:?}")),
                },
            };
            (index, look, material_shading(material))
        })
        .collect();
    let lit = frame_lights.iter().filter(|l| !l.is_empty()).count();
    log.detail(format!(
        "animation {label} loop '{loop_name}': {count} frames, {:.4} s, steps add up to ({:.1}, {:.1}, {:.1}){}{}",
        l.duration,
        step_sum[0],
        step_sum[1],
        step_sum[2],
        if turning > 0 { format!("; {turning} frame steps also turn (not applied)") } else { String::new() },
        if lit > 0 { format!("; carries lights in {lit} frame(s) (rule 26b)") } else { String::new() }
    ));
    Ok(Animation { looks, frames, duration: l.duration, step_sum, frame_steps, frame_transforms, frame_lights, ball: (shape.radius, shape.center), frame_parts })
}

/// The world's haze (object-lighting.md rules 18, 19): `hazeDistance` and
/// `visibleDistance` from `TERRAIN.KQ` (no terrain or missing: 20,000 and
/// 25,500, the note's defaults) and `resetHazeColor` from `world.cs`
/// (missing: black, STAND-IN), snapped to the nearest colour of the
/// palette `world.cs` loads (`KQWorld::loadingPalette`, else
/// `<world>.ppl`; rule 23); the background is that palette's entry
/// `newSky n` (rule 31; missing: black, STAND-IN).
pub fn load_haze(game: &GameDir, world: &World, settings: Option<&kq8_data::TerrainSettings>, log: &mut Log) -> kq8_data::haze::Haze {
    use kq8_data::haze::{new_sky, palette_haze_colour, reset_haze_color, Haze, NO_TERRAIN_HAZE_DISTANCE, NO_TERRAIN_VISIBLE_DISTANCE};
    let start = settings.and_then(|s| s.haze_distance).unwrap_or(NO_TERRAIN_HAZE_DISTANCE);
    let visible = settings.and_then(|s| s.visible_distance).unwrap_or(NO_TERRAIN_VISIBLE_DISTANCE);
    let script = match world.load("world.cs") {
        Some(Ok((label, bytes))) => {
            log.detail(format!("haze colour and background from {label}"));
            Some(bytes)
        }
        _ => {
            log.summary(format!("haze: no world.cs in {}; haze colour and background black", world.name));
            None
        }
    };
    let palette_name = world_palette_name(world, "loadingPalette").unwrap_or_else(|| format!("{}.ppl", world.name));
    let palette = match load_palette(world, Some(&palette_name), game, log) {
        Ok(p) => Some(p),
        Err(e) => {
            log.summary(format!("haze: palette {palette_name}: {e}; haze colour and background black"));
            None
        }
    };
    let screen = |c: [u8; 3]| c.map(|v| v as f32 / 255.0);
    let mut colour = [0.0; 3];
    match (script.as_deref().and_then(reset_haze_color), &palette) {
        (Some(want), Some(p)) => {
            let i = palette_haze_colour(p, want);
            colour = screen(p[i]);
            log.summary(format!(
                "haze colour: resetHazeColor {} {} {} -> {palette_name} entry {i} = ({}, {}, {}) (nearest of entries 10-244)",
                want[0], want[1], want[2], p[i][0], p[i][1], p[i][2]
            ));
        }
        (None, _) => log.summary("haze colour: no resetHazeColor line; black"),
        _ => {}
    }
    let mut background = [0.0; 3];
    match (script.as_deref().and_then(new_sky), &palette) {
        (Some(n), Some(p)) => {
            background = screen(p[n]);
            log.summary(format!("background: newSky {n} -> {palette_name} entry {n} = ({}, {}, {})", p[n][0], p[n][1], p[n][2]));
        }
        (None, _) => log.summary("background: no newSky line; black"),
        _ => {}
    }
    let mut haze = Haze::new(start, visible, colour);
    haze.background = background;
    log.summary(format!(
        "haze: hazeDistance {start:.0}, visibleDistance {visible:.0} -> haze full at {:.0}{}",
        haze.end,
        if settings.is_none() { " (no terrain: the note's defaults)" } else { "" }
    ));
    haze
}

/// The lights for models on hardware (object-lighting.md rule 1): the
/// files the world's `swtohwlt.cs` loads with `loadKQ`, from
/// `data/<World>/KQ/<path>` or, failing that, the world's archives. No
/// script or no file: logged, and that part stays dark (no light).
pub fn load_object_lights(game: &GameDir, world: &World, log: &mut Log) -> kq8_data::lights::Lights {
    use kq8_data::{kq_text, lights};
    let mut out = lights::Lights::default();
    let script = match world.load("swtohwlt.cs") {
        Some(Ok((label, bytes))) => {
            let files = lights::load_kq_lines(&bytes);
            log.summary(format!("object light: {label} loads {files:?}"));
            files
        }
        Some(Err(e)) => {
            log.summary(format!("object light: swtohwlt.cs: {e}; no lights"));
            return out;
        }
        None => {
            log.summary(format!("object light: no swtohwlt.cs in {}; no lights", world.name));
            return out;
        }
    };
    for path in script {
        let rel = format!("data/{}/KQ/{path}", world.name);
        let found = match game.find(&rel).filter(|p| p.is_file()) {
            Some(p) => std::fs::read(&p).map(|b| (rel, b)).map_err(|e| e.to_string()),
            None => {
                let base = path.rsplit(['/', '\\']).next().unwrap_or(&path);
                world.load(base).unwrap_or_else(|| Err(format!("{path} not found")))
            }
        };
        match found {
            Ok((label, bytes)) => {
                out.add_sections(&path, &kq_text::parse(&bytes));
                log.detail(format!("  object light file {path}: {label}"));
            }
            Err(e) => log.summary(format!("  object light file {path}: {e}")),
        }
    }
    for p in &out.problems {
        log.summary(format!("  object light problem: {p}"));
    }
    out
}

/// A model name is looked up in the world's archives, then in
/// `GAME/Common`; a path to a loose file also works.
fn find_model(game: &GameDir, world: &World, model: &str) -> Result<(String, Vec<u8>), String> {
    if let Some(found) = world.load(model) {
        return found;
    }
    for rel in [format!("GAME/Common/{model}"), model.to_string()] {
        if let Some(path) = game.find(&rel).filter(|p| p.is_file()) {
            let bytes = std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?;
            return Ok((rel, bytes));
        }
    }
    if Path::new(model).is_file() {
        let bytes = std::fs::read(model).map_err(|e| format!("{model}: {e}"))?;
        return Ok((model.to_string(), bytes));
    }
    Err(format!("model '{model}' not found in world {} or GAME/Common", world.name))
}

/// Default: `<world>.ppl` from the world's archives (measured best for
/// Daventry; see docs/behaviour/QUESTIONS.md Q3).
fn load_palette(world: &World, spec: Option<&str>, game: &GameDir, log: &mut Log) -> Result<[Rgb; 256], String> {
    let (label, bytes) = match spec {
        Some(s) => match world.load(s) {
            Some(r) => r?,
            None => {
                let path = game.find(s).ok_or(format!("palette '{s}' not found"))?;
                (s.to_string(), std::fs::read(&path).map_err(|e| e.to_string())?)
            }
        },
        None => {
            let name = format!("{}.ppl", world.name);
            world.load(&name).ok_or(format!("default palette {name} not found; pass --palette"))??
        }
    };
    let pal = Palette::parse(&bytes).map_err(|e| format!("{label}: {e}"))?;
    log.summary(format!("palette: {label} ({:?}, {} entries)", pal.format, pal.entry_count));
    // As stored, entry 0 black (models-and-textures.md Q3, Q22).
    Ok(pal.shown_colors())
}

/// Textures are looked up in the current world first, then in every other
/// world (models in GAME/Common have no textures next to them; Q4).
fn load_texture(world: &World, game: &GameDir, name: &str, palette: &[Rgb; 256], log: &mut Log) -> Look {
    let mut found = world.load(name);
    if found.is_none() {
        for other in WORLDS.iter().filter(|w| **w != world.name) {
            if let Ok(w) = World::open(game, other) {
                found = w.load(name);
                if found.is_some() {
                    log.summary(format!("  texture {name}: not in {}, using {other}'s copy", world.name));
                    break;
                }
            }
        }
    }
    let (label, bytes) = match found {
        None => return Look::Missing(format!("texture '{name}' not found")),
        Some(Err(e)) => return Look::Missing(format!("texture '{name}': {e}")),
        Some(Ok(x)) => x,
    };
    let bmp = match Bitmap::parse(&bytes) {
        Ok(b) => b,
        Err(e) => return Look::Missing(format!("{label}: {e}")),
    };
    let rgba = to_rgba(&bmp, palette);
    let histogram = bmp.index_histogram();
    // Pixels using the 20 entries a 256-colour screen would swap (0-9,
    // 246-255; Q3), and entry 0 alone (drawn black, Q22).
    let edge = histogram.map(|h| h[..10].iter().chain(&h[246..]).sum::<u32>());
    let index0 = histogram.map_or(0, |h| h[0]);
    // For images with their own colour table: how many of the colours the
    // image actually uses differ from the world palette (0 = no difference),
    // both as drawn (entry 0 black, Q22).
    let own_vs_world = match (&bmp.embedded_palette, histogram) {
        (Some(own), Some(h)) => {
            let used = (0..256).filter(|&i| h[i] > 0);
            let drawn = |i: usize| if i == 0 { Some([0, 0, 0]) } else { own.get(i).copied() };
            let differ = used.filter(|&i| drawn(i) != Some(palette[i])).count();
            format!(" own_palette=yes used_colours_differing_from_world_palette={differ}")
        }
        _ => " own_palette=no".into(),
    };
    log.summary(format!(
        "  texture {label}: {:?} {}x{} {}-bit mipmaps={} flags=0x{:X} pixels_using_entries_0-9_246-255={} pixels_using_entry_0={index0}{own_vs_world}",
        bmp.format,
        bmp.width,
        bmp.height,
        bmp.bit_count,
        bmp.mip_count,
        bmp.flags,
        edge.map_or("-".into(), |n| n.to_string())
    ));
    Look::Texture { name: label, width: bmp.width, height: bmp.height, rgba }
}

/// `potions.md` rule 29 (M5 5g-6): an `I_` texture from this world only,
/// its entry-0 pixels see-through (alpha 0: the holes of its mesh).
fn load_ghost_texture(world: &World, name: &str, palette: &[Rgb; 256]) -> Option<Look> {
    let (label, bytes) = world.load(name)?.ok()?;
    let bmp = Bitmap::parse(&bytes).ok()?;
    let mut rgba = to_rgba(&bmp, palette);
    if bmp.bit_count == 8 {
        for (px, &i) in rgba.chunks_mut(4).zip(&bmp.pixels) {
            if i == 0 {
                px[3] = 0;
            }
        }
    }
    Some(Look::Texture { name: label, width: bmp.width, height: bmp.height, rgba })
}

/// Level 0 as RGBA, top row first. 8-bit images use the given palette
/// unless they carry their own.
fn to_rgba(bmp: &Bitmap, palette: &[Rgb; 256]) -> Vec<u8> {
    match bmp.bit_count {
        8 => {
            let own: Option<[Rgb; 256]> = bmp.embedded_palette.as_ref().map(|e| {
                let mut c = [[0; 3]; 256];
                c.iter_mut().zip(e).for_each(|(d, s)| *d = *s);
                // Entry 0 black, as for world palettes (Q22).
                kq8_data::palette::as_shown(c)
            });
            let colors = own.as_ref().unwrap_or(palette);
            bmp.pixels.iter().flat_map(|&i| {
                let c = colors[i as usize];
                [c[0], c[1], c[2], 255]
            }).collect()
        }
        _ => bmp.pixels.as_chunks::<3>().0.iter().flat_map(|p| [p[0], p[1], p[2], 255]).collect(),
    }
}

/// The ground of a world, ready to draw: triangles grouped by texture.
pub struct Ground {
    pub terrain: kq8_data::Terrain,
    pub groups: Vec<GroundGroup>,
    /// The material image (`terrm.dib`): one material per square.
    pub materials: kq8_data::terrain::Grid,
    /// Orientation code per material (direction table).
    pub orientation: Vec<u32>,
    /// Texture name per material, if it is a texture.
    pub textures: Vec<Option<String>>,
    /// `KQWorld::terrainPalette`.
    pub palette: [Rgb; 256],
    /// Squares a ground animator draws instead (M3 step 3b-3).
    pub animated: std::collections::BTreeSet<(usize, usize)>,
    /// The light or shadow map drawn on the ground (M1 step 4c,
    /// terrain-lighting.md); full brightness if none.
    pub light: kq8_data::lightmap::LightMap,
    /// `useDynamicLighting` from `Options.cs` (terrain-lighting.md rule 2;
    /// the game's start value 0 if missing): point lights paint the ground
    /// (object-lighting.md rule 26, M1 7e).
    pub dynamic: bool,
    /// The other map of the pair (light map when `dynamic`, else the
    /// shadow map), for F9 (M1 7j); full brightness if none.
    pub other_light: kq8_data::lightmap::LightMap,
}

/// The ground animators of a world (M3 step 3b-3, emitters.md "Ground
/// animators"), ready to run and draw.
pub struct GroundAnimAssets {
    pub animators: Vec<kq8_core::ground_anim::GroundAnimator>,
    /// The material image as loaded; the animators change their copy.
    pub materials: kq8_data::terrain::Grid,
    /// Triangles of each animated square for each material it can show.
    pub triangles: BTreeMap<((usize, usize), u8), Vec<kq8_data::terrain::GroundTriangle>>,
    /// The look of each of those materials (`None`: not a texture, the
    /// square is not drawn while it shows it).
    pub looks: BTreeMap<u8, Option<Look>>,
    /// Animated squares where the floor grid differs from the surface
    /// (see-through in the U key's "both" view).
    pub floor_differs: std::collections::BTreeSet<(usize, usize)>,
}

pub struct GroundGroup {
    pub label: String,
    pub triangles: Vec<kq8_data::terrain::GroundTriangle>,
    pub look: Look,
}

/// Reads `data/<World>/KQ/Terrain/TERRAIN.KQ` and everything it names
/// (rules: docs/behaviour/terrain.md).
pub fn load_ground(game: &GameDir, world: &World, log: &mut Log) -> Result<Ground, String> {
    use kq8_data::terrain::{parse_directions, Grid};
    use kq8_data::{kq_text, MaterialList, Terrain, TerrainSettings};

    let rel = format!("data/{}/KQ/Terrain/TERRAIN.KQ", world.name);
    let path = game.find(&rel).ok_or(format!("{rel} not found"))?;
    let text = std::fs::read(&path).map_err(|e| format!("{rel}: {e}"))?;
    let sections = kq_text::parse(&text);
    let section = sections
        .iter()
        .find(|s| s.get("classType").is_some_and(|c| c.eq_ignore_ascii_case("KQTerrain")))
        .ok_or(format!("{rel}: no section with classType=KQTerrain"))?;
    let settings = TerrainSettings::from_section(section)?;
    log.summary(format!(
        "terrain: {rel} groundScale={} (square {} units) heightBMPScale={}",
        settings.ground_scale,
        settings.square_size(),
        settings.height_scale
    ));

    // Images, list and direction table: next to TERRAIN.KQ first, then the
    // world's archives (where the images and the list are found).
    let fetch = |name: &str| -> Result<(String, Vec<u8>), String> {
        let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
        let loose = format!("data/{}/KQ/Terrain/{base}", world.name);
        if let Some(p) = game.find(&loose).filter(|p| p.is_file()) {
            return std::fs::read(&p).map(|b| (loose, b)).map_err(|e| e.to_string());
        }
        world.load(base).ok_or(format!("'{name}' not found for {}", world.name))?
    };
    let grid = |name: &str| -> Result<Grid, String> {
        let (at, bytes) = fetch(name)?;
        let b = Bitmap::parse(&bytes).map_err(|e| format!("{at}: {e}"))?;
        Grid::from_bitmap(&b).map_err(|e| format!("{at}: {e}"))
    };
    let mut terrain = Terrain::new(settings.clone(), grid(&settings.height_image)?)?;
    // The floor grid, the ground under water (water.md "Two ground grids").
    match &settings.floor_image {
        Some(name) => match grid(name).and_then(|floor| terrain.clone().with_floor(floor)) {
            Ok(t) => {
                terrain = t;
                let floor = terrain.floor.as_ref().unwrap();
                let pairs = terrain.heights.values().iter().zip(floor.values());
                let (mut higher, mut lower) = (0, 0);
                for (s, f) in pairs {
                    higher += usize::from(f > s);
                    lower += usize::from(f < s);
                }
                log.summary(format!(
                    "  floor grid: {name}; corners where the floor is lower than the surface {lower}, higher {higher}"
                ));
            }
            Err(e) => log.summary(format!("  floor grid: {name} NOT loaded ({e}); floor = surface")),
        },
        None => log.summary("  floor grid: none (no floorBMPName); floor = surface"),
    }
    let materials = grid(&settings.material_image)?;
    // Ground flags per material (water.md "Ground flags from envInfo.kq").
    let env_name = settings
        .all
        .iter()
        .find(|(k, v)| k.eq_ignore_ascii_case("envinfoName") && !v.is_empty())
        .map_or("envInfo.kq".to_string(), |(_, v)| v.clone());
    match fetch(&env_name) {
        Ok((at, bytes)) => {
            let flags = kq8_data::envinfo::parse(&bytes);
            match terrain.clone().with_ground_flags(&materials, &flags) {
                Ok(t) => {
                    terrain = t;
                    let count = |mask: u32| terrain.square_flags.iter().filter(|f| *f & mask != 0).count();
                    use kq8_data::envinfo::{FAST_POISON, LIQUID, POISON, SWAMP, WATER};
                    // Liquid squares with the surface above the floor somewhere.
                    let (nx, _) = terrain.squares();
                    let size = settings.square_size();
                    let deep = terrain
                        .square_flags
                        .iter()
                        .enumerate()
                        .filter(|(_, f)| *f & LIQUID != 0)
                        .filter(|(k, _)| {
                            let (x, y) = (((k % nx) as f32 + 0.5) * size, ((k / nx) as f32 + 0.5) * size);
                            let (s, f) = (terrain.height_at(x, y), terrain.floor_at(x, y));
                            s.zip(f).is_some_and(|(s, f)| s.height > f.height)
                        })
                        .count();
                    log.summary(format!(
                        "  ground flags: {at}: {} materials; liquid squares {} (water {}, poison {}, fastpoison/lava {}, swamp {}), {deep} with the surface above the floor at their centre",
                        flags.len(),
                        count(LIQUID),
                        count(WATER),
                        count(POISON),
                        count(FAST_POISON),
                        count(SWAMP)
                    ));
                }
                Err(e) => log.summary(format!("  ground flags: {at}: NOT used ({e}); no water anywhere")),
            }
        }
        Err(e) => log.summary(format!("  ground flags: {env_name} NOT found ({e}); no water anywhere")),
    }
    let (list_at, list_bytes) = fetch(&settings.material_list)?;
    let list = MaterialList::parse(&list_bytes).map_err(|e| format!("{list_at}: {e}"))?;
    let mut orientation = vec![0u32; list.materials.len()];
    match &settings.material_direction {
        Some(name) => {
            let (at, bytes) = fetch(name)?;
            let records = parse_directions(&bytes).map_err(|e| format!("{at}: {e}"))?;
            for r in &records {
                if let Some(slot) = orientation.get_mut(r.index as usize) {
                    *slot = r.direction;
                }
            }
            log.summary(format!("  direction table {at}: {} records", records.len()));
        }
        None => log.summary("  no direction table: every texture drawn as stored (code 0)"),
    }

    let palette_name = world_palette_name(world, "terrainPalette").unwrap_or_else(|| {
        let fallback = format!("{}.ppl", world.name);
        log.summary(format!("  world.cs names no KQWorld::terrainPalette; using {fallback}"));
        fallback
    });
    let palette = load_palette(world, Some(&palette_name), game, log)?;

    let triangles = terrain.ground_triangles(&materials, &orientation)?;
    let textures = list
        .materials
        .iter()
        .map(|m| (m.kind() == MaterialKind::Texture && !m.texture.is_empty()).then(|| m.texture.clone()))
        .collect();
    let mut by_texture: BTreeMap<String, (String, Vec<kq8_data::terrain::GroundTriangle>)> = BTreeMap::new();
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    for t in triangles {
        let m = list.materials.get(t.material as usize);
        match m {
            Some(m) if m.kind() == MaterialKind::Texture && !m.texture.is_empty() => {
                by_texture
                    .entry(m.texture.to_ascii_lowercase())
                    .or_insert_with(|| (m.texture.clone(), Vec::new()))
                    .1
                    .push(t);
            }
            other => {
                let why = other.map_or("material not in list".into(), |m| format!("kind {:?} flags 0x{:X}", m.kind(), m.flags));
                *skipped.entry(why).or_default() += 1;
            }
        }
    }
    let drawn: usize = by_texture.values().map(|(_, v)| v.len()).sum();
    log.summary(format!(
        "  ground: {} triangles drawn ({} squares) using {} textures; skipped triangles: {:?}",
        drawn,
        drawn / 2,
        by_texture.len(),
        skipped
    ));
    let mut groups = Vec::new();
    for (_, (name, tris)) in by_texture {
        let look = load_texture(world, game, &name, &palette, log);
        if let Look::Missing(why) = &look {
            log.summary(format!("  MISSING {why} ({} triangles drawn magenta)", tris.len()));
        }
        groups.push(GroundGroup { label: name, triangles: tris, look });
    }
    let dynamic = use_dynamic_lighting(game);
    let why = match options_dynamic_lighting(game) {
        Some(on) if DYNAMIC_OVERRIDE.get().is_some() => format!("--dynamic {}", if on { "on" } else { "off" }),
        Some(on) => format!("Options.cs useDynamicLighting {}", u8::from(on)),
        None => "no useDynamicLighting in Options.cs, the game's start value 0".into(),
    };
    let light = load_light_map(world, &settings, &fetch, dynamic, &why, log);
    // The other one, for F9 (M1 7j).
    let other_light = load_light_map(world, &settings, &fetch, !dynamic, "kept for F9", log);
    Ok(Ground { terrain, groups, materials, orientation, textures, palette, animated: Default::default(), light, dynamic, other_light })
}

/// The viewer's `--dynamic on|off` (M1 7d), which stands in for
/// `Options.cs` (the game's files are never written).
static DYNAMIC_OVERRIDE: std::sync::OnceLock<bool> = std::sync::OnceLock::new();

/// Sets `--dynamic`; call before loading.
pub fn override_dynamic_lighting(on: bool) {
    let _ = DYNAMIC_OVERRIDE.set(on);
}

/// `useDynamicLighting` as `Options.cs` sets it, if it does (or as
/// `--dynamic` says).
fn options_dynamic_lighting(game: &GameDir) -> Option<bool> {
    if let Some(on) = DYNAMIC_OVERRIDE.get() {
        return Some(*on);
    }
    game.find("Options.cs").and_then(|p| std::fs::read(p).ok()).and_then(|b| kq8_data::lightmap::use_dynamic_lighting(&b))
}

/// `useDynamicLighting` in effect: `Options.cs`, else the game's start
/// value 0 (terrain-lighting.md rule 2).
pub fn use_dynamic_lighting(game: &GameDir) -> bool {
    options_dynamic_lighting(game).unwrap_or(false)
}

/// A file found by name: (where, bytes).
type Fetched = Result<(String, Vec<u8>), String>;

/// A ground light map (terrain-lighting.md rules 1-6): the hardware pair
/// (we draw like the Glide/Direct3D renderers), the shadow map for
/// `dynamic` (`useDynamicLighting 1`), else the light map; `why` says why
/// (for the log). No name, or a file that can't be used: full brightness
/// (rule 4).
fn load_light_map(
    world: &World,
    settings: &kq8_data::TerrainSettings,
    fetch: &dyn Fn(&str) -> Fetched,
    dynamic: bool,
    why: &str,
    log: &mut Log,
) -> kq8_data::lightmap::LightMap {
    use kq8_data::lightmap::{self, Kind, LightMap};
    let kind = Kind::for_dynamic_lighting(dynamic);
    let full = LightMap::full(settings);
    let Some(name) = lightmap::file_name(settings, true, kind) else {
        log.summary(format!(
            "  ground light: {kind:?} map ({why}); {} has no file name: full brightness (side {}, spacing {})",
            world.name, full.side, full.spacing
        ));
        return full;
    };
    let map = fetch(name).and_then(|(at, bytes)| match LightMap::parse(&bytes, settings) {
        Ok(m) => Ok((at, m)),
        Err(e) => Err(format!("{at}: {e}")),
    });
    match map {
        Ok((at, map)) => {
            let red = map.histograms()[1];
            let hist: Vec<String> = red.iter().enumerate().filter(|(_, n)| **n > 0).map(|(v, n)| format!("{v} x{n}")).collect();
            log.summary(format!(
                "  ground light: {kind:?} map {name} ({why}): {at}, count {} = {} x {}, spacing {} units; red {}",
                map.values().len(),
                map.side,
                map.side,
                map.spacing,
                hist.join(", ")
            ));
            map
        }
        Err(e) => {
            log.summary(format!("  ground light: {kind:?} map {name} NOT used ({e}); full brightness"));
            full
        }
    }
}

/// Chooses each ground animator's squares (in load order, from the
/// materials as loaded), starts them at game time 0, and loads what drawing
/// their squares needs (M3 step 3b-3). Marks the squares in
/// `ground.animated`, so the normal ground mesh leaves their surface out.
pub fn prepare_ground_animation(
    game: &GameDir,
    world: &World,
    ground: &mut Ground,
    animators: Vec<(String, kq8_data::effects::AnimatorSettings)>,
    log: &mut Log,
) -> GroundAnimAssets {
    use kq8_core::ground_anim::{choose_squares, GroundAnimator};
    let mut running = Vec::new();
    for (name, settings) in animators {
        let (squares, mut notes) = choose_squares(&settings, &ground.materials);
        let (a, more) = GroundAnimator::new(&name, &settings, squares, 0.0);
        notes.extend(more);
        notes.extend(settings.problems.iter().cloned());
        // Which materials its squares had, for the log.
        let mut had: BTreeMap<u8, usize> = BTreeMap::new();
        for &(i, j) in &a.squares {
            *had.entry(ground.materials.get(i, j).unwrap()).or_default() += 1;
        }
        let how = match &settings.grid_blocks {
            Some(b) => format!("{} listed squares", b.len()),
            None => format!("materials {:?} minus {} excluded squares", settings.orig_textures, settings.exclude.len()),
        };
        log.summary(format!(
            "ground animator [{name}]: {} squares ({how}; their materials {had:?}); cycle {:?} every {} s, repeat {}, {}{}",
            a.squares.len(),
            a.cycle,
            a.delay,
            a.repeat,
            if a.running { "started" } else { "NOT started" },
            if notes.is_empty() { String::new() } else { format!("; notes: {}", notes.join("; ")) }
        ));
        running.push((a, settings.exclude.clone()));
    }
    // Excluded squares: does any animator draw them anyway? (emitters.md
    // "Edge cases": (45, 57) at Daventry's stream never animates.)
    let mut excluded_by: BTreeMap<(i64, i64), Vec<&str>> = BTreeMap::new();
    for (a, exclude) in &running {
        for &sq in exclude {
            excluded_by.entry(sq).or_default().push(a.name.as_str());
        }
    }
    for ((i, j), names) in &excluded_by {
        let by: Vec<&str> = running
            .iter()
            .filter(|(b, _)| b.squares.iter().any(|&(x, y)| (x as i64, y as i64) == (*i, *j)))
            .map(|(b, _)| b.name.as_str())
            .collect();
        log.summary(format!("  excluded square ({i}, {j}) (by {} animators): animated by {by:?}", names.len()));
    }
    let animators: Vec<GroundAnimator> = running.into_iter().map(|(a, _)| a).collect();
    // Every material each animated square can show: its own, and the
    // cycles of the animators that change it.
    let mut triangles = BTreeMap::new();
    let mut needed = std::collections::BTreeSet::new();
    let mut floor_differs = std::collections::BTreeSet::new();
    // Squares whose own turn differs from a cycle material's (where the Q50
    // stand-in changes the picture), per animator.
    let mut kept_turn: Vec<String> = Vec::new();
    let terrain = &ground.terrain;
    for a in &animators {
        for &(i, j) in &a.squares {
            ground.animated.insert((i, j));
            let own = ground.materials.get(i, j).unwrap();
            // The square keeps its own material's orientation whatever it
            // shows: the user's check of our viewer against the data (shore
            // and corner squares are one texture in up to four turns, the
            // animation frames are stored unturned). Stand-in until Q50;
            // emitters.md rules 2 and 6 read as "the new material's turn".
            let code = ground.orientation.get(own as usize).copied().unwrap_or(0);
            if a.cycle.iter().any(|&m| ground.orientation.get(m as usize).copied().unwrap_or(0) != code) {
                kept_turn.push(a.name.clone());
            }
            for m in std::iter::once(own).chain(a.cycle.iter().copied()) {
                needed.insert(m);
                // Cut into the light map's cells, like the rest of the ground (M1 step 4c).
                let cells = 1usize << terrain.settings.light_scale;
                triangles.entry(((i, j), m)).or_insert_with(|| terrain.square_cells(kq8_data::terrain::Level::Surface, i, j, m, code, cells));
            }
            let corners = [(i, j), (i + 1, j), (i, j + 1), (i + 1, j + 1)];
            if corners.iter().any(|&(x, y)| terrain.point_height_on(kq8_data::terrain::Level::Floor, x, y) != terrain.point_height(x, y)) {
                floor_differs.insert((i, j));
            }
        }
    }
    let mut per: BTreeMap<String, usize> = BTreeMap::new();
    for name in kept_turn {
        *per.entry(name).or_default() += 1;
    }
    log.summary(format!(
        "ground animation: squares keeping their own turn where a cycle material has another (Q50 stand-in): {} {per:?}",
        per.values().sum::<usize>()
    ));
    let mut looks = BTreeMap::new();
    for m in needed {
        let look = match ground.textures.get(m as usize).cloned().flatten() {
            Some(name) => {
                log.quiet = true;
                let look = load_texture(world, game, &name, &ground.palette, log);
                log.quiet = false;
                if let Look::Missing(why) = &look {
                    log.summary(format!("  ground animation: material {m}: MISSING {why} (drawn magenta)"));
                }
                Some(look)
            }
            None => {
                log.summary(format!("  ground animation: material {m} is not a texture; squares showing it are not drawn"));
                None
            }
        };
        looks.insert(m, look);
    }
    log.summary(format!(
        "ground animation: {} animators ({} running), {} animated squares ({} over a different floor), {} materials",
        animators.len(),
        animators.iter().filter(|a| a.running).count(),
        ground.animated.len(),
        floor_differs.len(),
        looks.len()
    ));
    GroundAnimAssets { animators, materials: ground.materials.clone(), triangles, looks, floor_differs }
}

/// `set KQWorld::<setting> <name>` in the world's `world.cs`, e.g.
/// `terrainPalette` or `objectPalette` (models-and-textures.md, Q3). The
/// first such line is used; Daventry's 8-bit and high-colour branches name
/// the same file.
fn world_palette_name(world: &World, setting: &str) -> Option<String> {
    let (_, bytes) = world.load("world.cs")?.ok()?;
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let key = format!("KQWorld::{setting}");
    text.lines().find_map(|l| {
        let mut w = l.split_whitespace();
        (w.next()?.eq_ignore_ascii_case("set") && w.next()?.eq_ignore_ascii_case(&key))
            .then(|| w.next().map(str::to_string))
            .flatten()
    })
}

/// The game-play screen's palette (bars, caption strip, fonts): the
/// paths to try, best first, and a line for the log (`interface.md`
/// rules 1c, 1e, 1f, Q118). The file is the world's
/// `KQWorld::objectPalette` from `world.cs`; if that one is missing the
/// panels keep the last palette that loaded, so `loadingPalette`, then
/// the interior, terrain and Connor ones follow. Each name is looked up
/// in the world's `resource` folder, then `GAME/Resource` (the copies are
/// the same).
pub fn screen_palette_paths(game: &GameDir, world: &str) -> (Vec<String>, String) {
    const SETTINGS: [&str; 5] = ["objectPalette", "loadingPalette", "interiorPalette", "terrainPalette", "connorPalette"];
    let w = match World::open(game, world) {
        Ok(w) => w,
        Err(e) => return (Vec::new(), format!("world.cs not read ({e})")),
    };
    let mut names: Vec<(&str, String)> = Vec::new();
    for s in SETTINGS {
        if let Some(n) = world_palette_name(&w, s).filter(|n| !names.iter().any(|(_, m)| m.eq_ignore_ascii_case(n))) {
            names.push((s, n));
        }
    }
    let note = if names.is_empty() {
        "world.cs names no KQWorld:: palette".to_string()
    } else {
        names.iter().map(|(s, n)| format!("KQWorld::{s} {n}")).collect::<Vec<_>>().join(", ")
    };
    let paths = names.iter().flat_map(|(_, n)| [format!("{world}/resource/{n}"), format!("GAME/Resource/{n}")]).collect();
    (paths, note)
}

/// Daventry's (or any world's) placed objects: each different model once,
/// plus where each copy goes.
/// A world's map file (M6 6d-2, `magic-map.md` rule 3): where it was
/// read, its keys, and the terrain's `hazeDistance` (rule 13).
pub struct MapFile {
    pub label: String,
    pub keys: kq8_data::magic_map::MapKeys,
    pub haze: Option<f64>,
}

pub struct Objects {
    /// The world's shadow mask (`setShadowMask` in `Options.cs`, else the
    /// built-in 1; `characters.md` rule 36).
    pub shadow_world_mask: u32,
    /// (shape name as first written, model in model space).
    pub shapes: Vec<(String, Model)>,
    /// A placement, the index of its model in `shapes`, and whether it is
    /// an unused object (in a `.kq` file no start-up script loads; drawn
    /// only with the H key, viewer only).
    pub placed: Vec<(kq8_data::objects::Placement, usize, bool)>,
    /// Loaded objects that have a position but nothing we can draw: shown
    /// as markers on request (viewer only, not a game rule).
    pub markers: Vec<Marker>,
    /// Connor, if the world script loads him: placement, animations, keys.
    pub connor: Option<(kq8_data::objects::Placement, ConnorAssets)>,
    /// Where the world script puts the camera object `KQCamera`, if anywhere.
    pub camera_start: Option<[f32; 3]>,
    /// Every building the start-up scripts load (hidden ones too: hiding
    /// only changes the drawing), as colliders (M2 step 2d-3).
    pub buildings: Vec<kq8_core::collide::BuildingCollider>,
    /// Each building's collision type (default 0x28), same order.
    pub building_types: Vec<u32>,
    /// Each building as a click target (M3 step 3g-2), same order.
    pub building_clicks: Vec<kq8_core::active::ActiveBuilding>,
    /// Placed objects other than buildings and doors whose collision type
    /// blocks the camera's view (bit 0x8 or 0x1000), by their box (2f-2).
    pub view_boxes: Vec<kq8_core::collide::BoxCollider>,
    /// Every other placed object that collides with Connor (box from
    /// `BOUNDS.KQ`, type and mask rule; hidden ones too), M2 step 2d-5.
    /// Doors are not in it (their boxes move; `door_view.rs`).
    pub objects: Vec<kq8_core::collide::BoxCollider>,
    /// The doors the start-up scripts load (M3 step 3a), and the chests
    /// (3e-4: same movements and drawing; `DoorAssets::chest` is set).
    pub doors: Vec<DoorAssets>,
    /// The doors' animations, each different one once.
    pub door_animations: Vec<Animation>,
    /// Objects other than doors with the clickable bit 0x20 and a box.
    pub clickables: Vec<kq8_core::active::Clickable>,
    /// Every loaded object's name (for scripts: "unknown object").
    pub known: Vec<String>,
    /// The variables the start-up scripts left.
    pub vars: kq8_data::world_script::Vars,
    /// Every `.cs` of the world: (file name, bytes).
    pub scripts: Vec<(String, Vec<u8>)>,
    /// Connor's `attackDist` from `GAME/Kq/Connor.kq` (doors.md rule 11).
    pub attack_dist: Option<f32>,
    /// Connor's `stats=` and `level=` from `GAME/Kq/Connor.kq` (3i-6).
    pub connor_stats: Option<(String, Option<String>)>,
    /// What `Connor.kq` gives the fight (M5 5c-4, `fighting.md` rules 24,
    /// 39–41).
    pub connor_fight: Option<kq8_core::fight::ConnorFight>,
    /// Connor's start inventory from `Connor.kq` (M4 4c-2,
    /// `inventory.md` rule 4): per item its section name, type,
    /// quantity and `userId`, in list order; and the problems met.
    pub connor_inventory: (Vec<ConnorItem>, Vec<String>),
    /// What the start-up scripts give Connor (`conStart.cs`: items,
    /// `setLevel`, a stats event), in script order, done when play starts
    /// (DESIGN "Connor's levels checked", L-1).
    pub connor_start: Vec<kq8_core::connor_start::StartLine>,
    /// Every placed object's `stats=` line (3i-6): name, the line.
    pub object_stats: Vec<(String, String)>,
    /// The placed `KQMonster`s (M4 4a-2, `talking.md` rule 6): name,
    /// place after `move`, script.
    pub monsters: Vec<kq8_core::active::Monster>,
    /// The message files and voice lookup, and `Options.cs`'s `set`
    /// lines (M4 4a-2), filled in by `main.rs`.
    pub talk: Option<(kq8_core::talk::Talk, Vec<(String, String)>)>,
    /// `GAME/patch/BOUNDS.KQ`, for the draw-distance balls (M1 7a);
    /// `None` if it could not be read.
    pub game_bounds: Option<kq8_data::bounds::BoundingBoxes>,
    /// The emitters the start-up scripts load (M3 step 3b-1).
    pub emitters: Vec<EmitterAssets>,
    /// The particle catchers (M3 step 3b-2): name and settings.
    pub catchers: Vec<(String, kq8_data::effects::CatcherSettings)>,
    /// The traps (M3 step 3d-2), where the start-up scripts left them.
    pub traps: Vec<kq8_core::trap::TrapSettings>,
    /// The conversation info objects (`KQConvInfo`, M4 4e-3).
    pub conv_infos: Vec<kq8_data::conv_info::ConvInfo>,
    /// `KQAudibleObject`s the start-up scripts load (M3 step 3h-3,
    /// sound.md rules 43-44).
    pub audibles: Vec<kq8_core::sound::Audible>,
    /// Objects prepared for a script's `loadKQ` while playing (M3 3k-4):
    /// lower-case file name, object name.
    pub dormant: Vec<(String, String, Option<String>)>,
    /// The traps among them (lower-case names): built, switched off until
    /// their `loadKQ`.
    pub dormant_traps: Vec<String>,
    /// The files a script may load while playing as new copies (M5 5d-3):
    /// read and built at the start like an unused object, hidden.
    pub templates: Vec<TemplateAssets>,
    /// The world's music object (`KQMusic`, `music.kq`; M3 step 3h-5,
    /// sound.md rule 57), if a start-up script loads one.
    pub music: Option<kq8_core::music::Music>,
    /// Each building's `footsoundType` by lower-case name, `None` if it
    /// has none (M3 3h-4, sound.md rule 39).
    pub footsounds: BTreeMap<String, Option<i64>>,
    /// `KQPointLight` objects the start-up scripts load (object-lighting.md
    /// rules 24, 25; M1 7c-1): name and light.
    pub point_lights: Vec<(String, kq8_data::lights::PointLight)>,
    /// The ground animators in load order (M3 step 3b-3).
    pub animators: Vec<(String, kq8_data::effects::AnimatorSettings)>,
    /// Placed objects whose current movement plays (M3 step 3c).
    pub animated: Vec<AnimatedAssets>,
    /// Their animations, each (model, loop) once.
    pub object_animations: Vec<Animation>,
    /// The item types and the item objects the start-up scripts made (M3
    /// step 3e-3; items.md rule 9).
    pub items: kq8_core::item::Items,
    /// Each item type that can appear in this world (`IDName`) -> its
    /// model in `shapes`.
    pub item_shapes: BTreeMap<String, usize>,
    /// Corpse shapes of the fighters (`fighting.md` rule 32; M5 5c-7), and
    /// their bolts' (rule 36; 5c-9): lower-case shape name -> model slot.
    pub corpse_shapes: BTreeMap<String, usize>,
    /// The projectile files the fighters shoot (M5 5c-9), by lower-case name.
    pub projectiles: BTreeMap<String, kq8_data::characters::ProjectileKeys>,
    /// The objects whose `groupsAlways` has bit 2048 (M5 5g-7).
    pub reveal_objects: Vec<String>,
    /// The world's placed portals' keys (M6 6e-6).
    pub portals: Vec<kq8_data::magic_map::PortalKeys>,
    /// The `KQPortal` files in `GAME/Kq` (the teleporters'), by file name
    /// (M6 6e-6c).
    pub portal_files: Vec<(String, kq8_data::magic_map::PortalKeys)>,
    /// The shield's orbits (`orbit1.kq`–`orbit3.kq`, M5 5g-4) and their
    /// model slots.
    pub orbit: Vec<(kq8_data::characters::OrbitKeys, Option<usize>)>,
    /// Lines about items at load, for `items.log`.
    pub item_lines: Vec<String>,
    /// The world's magic map (M6 6d-2): the last `<x>map.kq` a start-up
    /// script loads (`magic-map.md` rule 3).
    pub map: Option<MapFile>,
    /// Lines about characters at load, for `characters.log` (M4 4d-2).
    pub character_lines: Vec<String>,
    /// The ground, for z -1 / -2 of items made while playing.
    pub ground: Option<kq8_data::Terrain>,
    /// `KQWorld::MaxMonsters` from `world.cs` (`character-movement.md`
    /// rule 20; M5 5a-4).
    pub max_monsters: Option<usize>,
}

/// A placed object whose movement plays (M3 step 3c; objects.md "Animated
/// objects at load"). It stays in `Objects::placed` (collision, logs) but
/// is drawn by `anim_view.rs`, not as a still model.
pub struct AnimatedAssets {
    /// Index into `Objects::placed`.
    pub placed: usize,
    /// Every movement in list order (M3 step 3i-4): name, `loop=` line
    /// and cues, index into `Objects::object_animations`.
    pub movements: Vec<(String, kq8_data::objects::CurrentLoop, usize)>,
    /// The movement shown at load (`currAnimName`, else movement 0).
    pub current: usize,
    /// `currAnimName` and `nextAnimName` as written (`anim-scripts.md`
    /// rule 3).
    pub anim_names: (Option<String>, Option<String>),
    /// `flags` bit 8: random start and speed.
    pub random: bool,
    /// `script=`, if any.
    pub script: Option<String>,
    /// `classType=KQAnimOnce` (`anim-scripts.md` rule 11; M5 5d-1).
    pub once: bool,
    /// A character's own values (M4 4d-2), `None` for other objects.
    pub character: Option<CharacterAssets>,
    /// A character's mouth textures `lip0`…`lip7` (M4 4d-5, rule 25),
    /// coloured with the object palette; `None` where one is missing.
    pub lip_looks: Vec<Option<Look>>,
}

/// A file a script may load while playing as new copies (M5 5d-3; plan
/// 5d): built at the start (hidden, at its own place), with what core
/// needs for a copy.
pub struct TemplateAssets {
    /// Lower-case file name, as `loadKQ` names it.
    pub file: String,
    /// Index into `Objects::placed` (its model slot is there).
    pub placed: usize,
    pub name: String,
    pub script: Option<String>,
    /// Its `BOUNDS.KQ` box (by shape, then name), its collision type and
    /// mask.
    pub bounds: Option<([f32; 3], [f32; 3])>,
    pub collider_type: u32,
    pub collider_mask: u32,
    /// Animated (in `Objects::animated`): 5d-4.
    pub animated: bool,
}

/// An emitter (`classType=KQEmitter`) as loaded (docs/behaviour/emitters.md).
pub struct EmitterAssets {
    pub name: String,
    /// The position as the file or the `loadKQ` line gives it, and after
    /// the usual placement rules (and any `move`).
    pub written: [f32; 3],
    pub position: [f32; 3],
    pub settings: kq8_data::effects::EmitterSettings,
    /// `translucency` (256 = solid), read like any object's.
    pub translucency: f32,
    /// `b_hazeSolid` and `hazeMultiplier`, read like any object's
    /// (object-lighting.md rule 30).
    pub haze: (bool, f32),
    /// `tooFar` and groupsAlways / groupsActive3D / groupsActive3DFar,
    /// for the draw distance (rules 36, 37).
    pub too_far: f32,
    pub groups: [u32; 3],
    /// The particle model: index into `Objects::shapes`.
    pub shape: Option<usize>,
    /// Floor and surface heights under the emitter, for the log.
    pub ground: [Option<f32>; 2],
}

/// A door (`classType=KQDoor`) as loaded (docs/behaviour/doors.md).
pub struct DoorAssets {
    /// Its place after the ground rules.
    pub placement: kq8_data::objects::Placement,
    /// Every movement in list order: name, `loop=` line and cues, index
    /// into `Objects::door_animations`.
    pub movements: Vec<(String, kq8_data::objects::CurrentLoop, usize)>,
    /// The movement shown when loaded (`currAnimName`).
    pub current: usize,
    /// `currAnimName` and `nextAnimName` as written (`anim-scripts.md`
    /// rule 3).
    pub anim_names: (Option<String>, Option<String>),
    /// Shape of that movement (for the box lookup).
    pub shape: String,
    pub script: Option<String>,
    pub collider_type: u32,
    pub collider_mask: u32,
    pub accepts_inventory: Option<String>,
    pub accept_all_items: bool,
    pub too_far_away_action: i64,
    pub do_not_point_at: bool,
    /// Its `BOUNDS.KQ` box (own space), found in `finish`.
    pub bounds: Option<([f32; 3], [f32; 3])>,
    /// A `KQChest`'s own keys (M3 step 3e-4); `None` for a door.
    pub chest: Option<kq8_core::chest::ChestSettings>,
    /// A scripted animated object (3e-5), not a door.
    pub object: bool,
}

/// What `place_door` places.
#[derive(Clone, Copy, PartialEq)]
enum Mover {
    Door,
    Chest,
    /// A clickable `KQAnimObject` with a script whose movement is still
    /// when loaded (3e-5).
    Object,
}

/// A building's collision type when its `.kq` gives none (camera.md
/// "Obstacles").
pub const BUILDING_TYPE: u32 = 0x28;

/// Collision type bits that block the camera's view line (camera.md
/// "Obstacles").
pub const VIEW_BLOCKING: u32 = 0x8 | 0x1000;

/// What collision needs to know about a placed object (M2 step 2d-5,
/// connor-physics.md "Collision shape").
struct CollisionInfo {
    /// The shape name as the object gives it (an animated object: the
    /// shape of the movement it has when it is loaded).
    shape: String,
    collider_type: u32,
    collider_mask: u32,
    /// `script=` and the click keys (`doors.md` rules 12-16), for a
    /// clickable one (3f).
    script: Option<String>,
    accepts_inventory: Option<String>,
    accept_all_items: bool,
    too_far_away_action: i64,
    do_not_point_at: bool,
    /// `footsoundType=` (buildings, `sound.md` rule 39; M3 3h-4).
    footsound_type: Option<i64>,
    /// `collisionShape=1`: a ball stack (M3 3j).
    ball_stack: bool,
    /// A character (M4 4d-2): the name its box is looked up by first
    /// (characters.md rule 9); `Some(None)` = by its name only.
    character_box: Option<Option<String>>,
}

/// `colliderType` / `colliderMask` from an object's section: a whole
/// number (decimal, or hex with `0x`) at the start of the value; anything
/// after it (`97`, two tabs, `#World, Click, Attack` in Barren and IceWorld) is a
/// comment (our reading). `default` if missing. A value that can't be read
/// gives the default and a problem text.
fn collider_value(s: &kq8_data::kq_text::Section, key: &str, default: u32) -> (u32, Option<String>) {
    let Some(full) = s.get(key) else { return (default, None) };
    let v = full.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("");
    let parsed = match v.strip_prefix("0x").or_else(|| v.strip_prefix("0X")) {
        Some(hex) => u32::from_str_radix(hex, 16).ok(),
        None => v.parse::<i64>().ok().map(|n| n as u32),
    };
    match parsed {
        Some(n) => (n, None),
        None => (default, Some(format!("{key}={full} is not a whole number; default {default:#x} used"))),
    }
}

pub struct Marker {
    pub name: String,
    pub class: String,
    /// After the same ground rules as drawn objects.
    pub position: [f32; 3],
    /// True: a character in an `.anm` file (drawn by the game, not by us
    /// yet). False: no model at all (traps, sounds, paths...).
    pub anm: bool,
}

/// Reads the objects the world's start-up scripts load, in script order,
/// and loads the models they name (rules: docs/behaviour/objects.md).
/// `terrain` gives the ground rules and, for the log, how far each placed
/// model's lowest point is from the ground under it.
/// A world Connor arrives in through a world change (M6 6e-3,
/// `world-change.md` rules 18, 21).
#[derive(Debug, Clone, Copy)]
pub struct Arrival {
    /// The game-wide `NoName<n>` counter carried over (rule 18): the new
    /// world's start names nothing below it.
    pub no_name_from: usize,
}

/// `arrival`: `Some` for a world change: `conStart.cs` is not run (rule
/// 21: only a developer start sets `$Testing`; Connor brings his own
/// things), and the `NoName` counter goes on from the carried one.
/// `progress`: the scripts' `setLoadProgress` percent, for the loading
/// screen (M6 6e-4a, `world-change.md` rule 24).
pub fn load_objects(game: &GameDir, world: &World, terrain: &kq8_data::Terrain, log: &mut Log, arrival: Option<Arrival>, progress: Option<std::sync::Arc<std::sync::atomic::AtomicU32>>) -> Objects {
    use kq8_data::world_script::{LoadKq, LoadName};

    let palette_name = world_palette_name(world, "objectPalette").unwrap_or_else(|| {
        let fallback = format!("{}.ppl", world.name);
        log.summary(format!("objects: world.cs names no KQWorld::objectPalette; using {fallback}"));
        fallback
    });
    log.summary(format!("objects: palette {palette_name} (KQWorld::objectPalette)"));
    let all_scripts: Vec<(String, String)> = world
        .entries_ending(".cs")
        .into_iter()
        .filter_map(|(n, _, b)| b.ok().map(|b| (n, b.iter().map(|&c| (c as char).to_ascii_lowercase()).collect())))
        .collect();
    // Files a script loads while playing with `loadKQ <file>` (M3 3k-4).
    let load_targets: std::collections::BTreeSet<String> = all_scripts
        .iter()
        .flat_map(|(_, t)| t.lines())
        .filter_map(|l| {
            let mut w = l.split('#').next().unwrap_or("").split_whitespace();
            (w.next() == Some("loadkq")).then(|| w.next()).flatten().map(|f| f.rsplit(['/', '\\']).next().unwrap_or(f).to_string())
        })
        .collect();
    let mut items = item_types(game, log);
    if let Some(a) = arrival {
        items.start_no_names_at(a.no_name_from);
        log.summary(format!("world change: NoName counter from {} (world-change.md rule 18); conStart.cs not run (rule 21)", a.no_name_from));
    }
    let mut placer = Placer {
        skip_con_start: arrival.is_some(),
        progress,
        map: None,
        load_targets,
        dormant: Vec::new(),
        dormant_traps: Vec::new(),
        template_files: Vec::new(),
        game,
        world,
        terrain,
        log,
        palette_name,
        all_scripts,
        interior_palette: None,
        shapes: Vec::new(),
        index_of: BTreeMap::new(),
        placed: Vec::new(),
        collision: Vec::new(),
        markers: Vec::new(),
        connor: None,
        attack_dist: None,
        connor_stats: None,
        connor_fight: None,
        connor_inventory: (Vec::new(), Vec::new()),
        connor_start: Vec::new(),
        object_stats: Vec::new(),
        monsters: Vec::new(),
        doors: Vec::new(),
        door_animations: Vec::new(),
        door_animation_index: BTreeMap::new(),
        emitters: Vec::new(),
        catchers: Vec::new(),
        traps: Vec::new(),
        conv_infos: Vec::new(),
        audibles: Vec::new(),
        music: None,
        point_lights: Vec::new(),
        animators: Vec::new(),
        animated: Vec::new(),
        object_animations: Vec::new(),
        object_animation_index: BTreeMap::new(),
        object_palette: None,
        located: BTreeMap::new(),
        skipped: BTreeMap::new(),
        unused_skipped: BTreeMap::new(),
        by_class: BTreeMap::new(),
        flag_values: BTreeMap::new(),
        hidden_values: BTreeMap::new(),
        missing_loc_counts: BTreeMap::new(),
        kq_variables: BTreeMap::new(),
        move_notes: BTreeMap::new(),
        sections: 0,
        raised: 0,
        fell_back: 0,
        on_ground_marker: 0,
        ignored_numbers: 0,
        unused_placed: 0,
        several_positions: 0,
        items,
        corpse_shapes: BTreeMap::new(),
        orbit: Vec::new(),
        reveal_objects: Vec::new(),
        portals: Vec::new(),
        portal_files: Vec::new(),
        drop_kinds: Vec::new(),
        projectiles: BTreeMap::new(),
        combat_apply: false,
        item_lines: Vec::new(),
        monster_types: None,
        pending_line: None,
        list_files: Default::default(),
        character_lines: Vec::new(),
        shadow_world_mask: None,
    };

    // M6 6e-6c: the portal files in GAME/Kq, for a script's loadKQ.
    if let Some(dir) = game.find("GAME/Kq") {
        let mut paths: Vec<_> = std::fs::read_dir(dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
        paths.sort();
        for path in paths {
            let Some(name) = path.file_name().and_then(|n| n.to_str()).filter(|n| n.to_ascii_lowercase().ends_with(".kq")).map(str::to_string) else { continue };
            let Ok(bytes) = std::fs::read(&path) else { continue };
            if let Some(k) = kq8_data::kq_text::parse(&bytes).iter().find_map(kq8_data::magic_map::PortalKeys::from_section) {
                placer.portal_files.push((name, k));
            }
        }
    }
    // Only what the world's start-up scripts load is in the world
    // (objects.md "Where a world's objects come from", Q16), placed while
    // the scripts run so `getLoc` and `move` see them (Q20).
    let script = kq8_data::world_script::run("newWorld.cs", &mut placer);
    let log = &mut *placer.log;
    log.summary(format!(
        "world scripts: ran {:?}; missing scripts {:?}; {} loadKQ lines ({} inside if-blocks); unreadable lines {}; other commands (not run): {:?}",
        script.scripts,
        script.missing_scripts,
        script.loads.len(),
        script.loads.iter().filter(|l| l.inside_if).count(),
        script.bad_lines.len(),
        script.other_commands
    ));
    log.summary(format!(
        "  variables: {} set lines, {} at the end; unknown variables used: {:?}; getLoc lines {} (object not found: {}); move lines {}",
        script.sets,
        script.vars.len(),
        script.unknown_vars,
        script.getlocs,
        script.getloc_not_found.len(),
        script.moves
    ));
    for b in &script.bad_lines {
        log.summary(format!("  unreadable: {b}"));
    }
    for g in &script.getloc_not_found {
        log.summary(format!("  getLoc of an object not loaded (variable left as it was): {g}"));
    }

    // Unused objects: `.kq` files that no start-up script loads. Not in
    // the game's world at the start; drawn only with the H key so they can
    // be looked at (viewer only, not a game rule). Same rules as loaded
    // objects otherwise, with the file's own position and angles.
    let loaded_files: std::collections::BTreeSet<String> = script
        .loads
        .iter()
        .map(|l| l.file.rsplit(['/', '\\']).next().unwrap_or(&l.file).to_ascii_lowercase())
        .collect();
    for (name, label, bytes) in world.entries_ending(".kq") {
        if loaded_files.contains(&name.to_ascii_lowercase()) || placer.list_files.contains(&name.to_ascii_lowercase()) {
            continue;
        }
        let Ok(bytes) = bytes else { continue };
        let load = LoadKq {
            file: name.clone(),
            name: LoadName::Keep,
            position: None,
            angles: None,
            ignored_numbers: 0,
            from: "(no start-up script)".into(),
            line: 0,
            inside_if: false,
        };
        placer.place(&load, Some((label, bytes)), true, &script.vars);
    }
    // The world's scripts, for running them while the game plays (M3).
    let mut scripts: Vec<(String, Vec<u8>)> = world.entries_ending(".cs").into_iter().filter_map(|(n, _, b)| b.ok().map(|b| (n, b))).collect();
    // Then the game-wide ones (`walk_la.cs`, ... in `GAME/Kq`, traps.md
    // "Data used"): searched after the world's folders, so a world's own
    // script of the same name wins (models-and-textures.md "Which one
    // wins"; `GAME/patch` before `GAME/Kq`: the note's likely reading).
    let mut shared = 0;
    for folder in ["GAME/patch", "GAME/Kq"] {
        let Some(dir) = game.find(folder) else { continue };
        let Ok(entries) = std::fs::read_dir(&dir) else { continue };
        let mut files: Vec<_> = entries.filter_map(Result::ok).map(|e| e.path()).collect();
        files.sort();
        for path in files {
            let Some(name) = path.file_name().and_then(|n| n.to_str()).map(str::to_string) else { continue };
            if !name.to_ascii_lowercase().ends_with(".cs") || scripts.iter().any(|(n, _)| n.eq_ignore_ascii_case(&name)) {
                continue;
            }
            if let Ok(bytes) = std::fs::read(&path) {
                scripts.push((name, bytes));
                shared += 1;
            }
        }
    }
    placer.log.summary(format!("scripts for play: {} from the world, {shared} game-wide (GAME/patch, GAME/Kq)", scripts.len() - shared));
    let (item_shapes, item_lines) = placer.item_models(&script);
    let items = std::mem::take(&mut placer.items);
    let mut objects = placer.finish();
    objects.scripts = scripts;
    objects.vars = script.vars;
    objects.items = items;
    objects.item_shapes = item_shapes;
    objects.item_lines = item_lines;
    objects.ground = Some(terrain.clone());
    objects.max_monsters = world_palette_name(world, "MaxMonsters").and_then(|v| v.parse().ok());
    objects
}

/// The item type list (`GAME/Kq/Invinfo.itl`, items.md rule 1) and each
/// type's click box: its shape in `GAME/patch/BOUNDS.KQ`, lower-cased as
/// for objects (M3 step 3e-3).
fn item_types(game: &GameDir, log: &mut Log) -> kq8_core::item::Items {
    let types = match game.find("GAME/Kq/Invinfo.itl").map(std::fs::read) {
        Some(Ok(bytes)) => {
            let (t, problems) = kq8_data::items::ItemTypes::parse(&bytes);
            for p in &problems {
                log.summary(format!("items: Invinfo.itl: {p}"));
            }
            t
        }
        _ => {
            log.summary("items: GAME/Kq/Invinfo.itl could not be read: no item types");
            Default::default()
        }
    };
    let table = match game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) {
        Some(Ok(bytes)) => kq8_data::bounds::BoundingBoxes::parse(&bytes).0,
        _ => Default::default(),
    };
    let boxes: Vec<(String, [f32; 3], [f32; 3])> = types
        .types
        .iter()
        .filter_map(|t| {
            let shape = t.shape.to_ascii_lowercase();
            table.lookup(&shape).map(|e| (shape, e.min, e.max))
        })
        .collect();
    log.summary(format!("items: {} item types, {} with a click box", types.types.len(), boxes.len()));
    kq8_core::item::Items::new(types, boxes)
}

/// Where a remembered object is drawn, so a `move` can change it.
#[derive(Clone, Copy)]
enum Drawn {
    Model(usize),
    Marker(usize),
    Nothing,
}

/// A loaded object by name: its current place (after the ground rules)
/// and what was drawn for it.
struct Located {
    position: [f32; 3],
    angles: [f32; 3],
    drawn: Drawn,
}

/// Places objects one by one as the scripts load them, and keeps the
/// numbers for the log.
struct Placer<'a> {
    /// A world change (M6 6e-3): `conStart.cs` is not run.
    skip_con_start: bool,
    /// The loading meter's percent (M6 6e-4a).
    progress: Option<std::sync::Arc<std::sync::atomic::AtomicU32>>,
    game: &'a GameDir,
    world: &'a World,
    terrain: &'a kq8_data::Terrain,
    /// The world's map file (M6 6d-2).
    map: Option<MapFile>,
    log: &'a mut Log,
    palette_name: String,
    /// Every script of the world (lower case), to tell which name a file.
    all_scripts: Vec<(String, String)>,
    /// Buildings use KQWorld::interiorPalette (buildings.md); loaded on first use.
    interior_palette: Option<Result<[Rgb; 256], String>>,
    shapes: Vec<(String, Model)>,
    index_of: BTreeMap<String, Option<usize>>,
    placed: Vec<(kq8_data::objects::Placement, usize, bool)>,
    /// One for each entry of `placed`.
    collision: Vec<CollisionInfo>,
    markers: Vec<Marker>,
    connor: Option<(kq8_data::objects::Placement, ConnorAssets)>,
    attack_dist: Option<f32>,
    connor_stats: Option<(String, Option<String>)>,
    connor_fight: Option<kq8_core::fight::ConnorFight>,
    connor_inventory: (Vec<ConnorItem>, Vec<String>),
    connor_start: Vec<kq8_core::connor_start::StartLine>,
    object_stats: Vec<(String, String)>,
    monsters: Vec<kq8_core::active::Monster>,
    doors: Vec<DoorAssets>,
    door_animations: Vec<Animation>,
    door_animation_index: BTreeMap<String, usize>,
    emitters: Vec<EmitterAssets>,
    catchers: Vec<(String, kq8_data::effects::CatcherSettings)>,
    traps: Vec<kq8_core::trap::TrapSettings>,
    conv_infos: Vec<kq8_data::conv_info::ConvInfo>,
    audibles: Vec<kq8_core::sound::Audible>,
    /// Files named by `loadKQ <file>` in the world's scripts, lower case
    /// (M3 3k-4).
    load_targets: std::collections::BTreeSet<String>,
    dormant: Vec<(String, String, Option<String>)>,
    dormant_traps: Vec<String>,
    /// (file, index into `placed`) of each template (M5 5d-3).
    template_files: Vec<(String, usize)>,
    music: Option<kq8_core::music::Music>,
    point_lights: Vec<(String, kq8_data::lights::PointLight)>,
    animators: Vec<(String, kq8_data::effects::AnimatorSettings)>,
    animated: Vec<AnimatedAssets>,
    object_animations: Vec<Animation>,
    object_animation_index: BTreeMap<String, Option<usize>>,
    object_palette: Option<Result<[Rgb; 256], String>>,
    /// Loaded objects by lower-case name (the last one loaded wins).
    located: BTreeMap<String, Located>,
    skipped: BTreeMap<String, usize>,
    unused_skipped: BTreeMap<String, usize>,
    by_class: BTreeMap<String, usize>,
    flag_values: BTreeMap<String, usize>,
    hidden_values: BTreeMap<String, usize>,
    missing_loc_counts: BTreeMap<String, usize>,
    /// `.kq` values that were `$variables`: replaced / unknown.
    kq_variables: BTreeMap<String, usize>,
    /// What happened to `move` lines.
    move_notes: BTreeMap<String, usize>,
    sections: usize,
    raised: usize,
    fell_back: usize,
    on_ground_marker: usize,
    ignored_numbers: usize,
    unused_placed: usize,
    several_positions: usize,
    /// Item types and the item objects made so far (3e-3).
    items: kq8_core::item::Items,
    /// Corpse shapes loaded (M5 5c-7).
    corpse_shapes: BTreeMap<String, usize>,
    /// The objects Clarity reveals (M5 5g-7).
    reveal_objects: Vec<String>,
    /// The world's portals (M6 6e-6).
    portals: Vec<kq8_data::magic_map::PortalKeys>,
    /// The portal files in GAME/Kq (M6 6e-6c).
    portal_files: Vec<(String, kq8_data::magic_map::PortalKeys)>,
    /// The shield's orbits (M5 5g-4).
    orbit: Vec<(kq8_data::characters::OrbitKeys, Option<usize>)>,
    /// The item types the fighters drop (M5 5c-8).
    drop_kinds: Vec<String>,
    /// The projectile files read (M5 5c-9).
    projectiles: BTreeMap<String, kq8_data::characters::ProjectileKeys>,
    /// `applyCombatLevel` is on (M5 5c-10, `fighting.md` rule 3).
    combat_apply: bool,
    /// Lines about items, for `items.log`.
    item_lines: Vec<String>,
    /// `GAME/Kq/Monstats.mtl`, read when a monster list is first loaded
    /// (characters.md rule 4).
    monster_types: Option<kq8_data::characters::MonsterTypes>,
    /// The monster-list line whose type file `place` is placing now.
    pending_line: Option<PendingLine>,
    /// Type files placed through the monster list (lower case): not
    /// shown again as unused objects.
    list_files: std::collections::BTreeSet<String>,
    /// Lines for `characters.log` written while loading.
    character_lines: Vec<String>,
    /// The world's shadow mask (`setShadowMask` in `Options.cs`, else
    /// 1), read when first needed (characters.md rule 36).
    shadow_world_mask: Option<u32>,
}

/// A monster-list line being placed (characters.md rule 4): the line, and
/// the values its `@` fields take from the type.
struct PendingLine {
    line: kq8_data::characters::MonsterLine,
    type_name: String,
    wander: kq8_data::characters::Wander,
    birth_delay: f32,
    from: String,
    /// Its type (for the `@` fields of the movement rules, M5 5a-1).
    kind: kq8_data::characters::MonsterType,
}

/// A character's `.anm` file, read: the list, and the bytes to attach.
struct LoadedAnm {
    list: kq8_data::characters::AnimList,
    bytes: Vec<u8>,
}

/// What the viewer and core need to know about a character (M4 4d-2,
/// characters.md), beyond what every animated object has.
#[derive(Debug, Clone)]
pub struct CharacterAssets {
    pub keys: kq8_data::characters::CharacterKeys,
    /// The name its box is looked up by first (rule 9): the monster
    /// list's type, else `monsterType`.
    pub box_type: Option<String>,
    /// Where it came from: its own file, or a monster-list line.
    pub from: String,
    /// May wander once active (M5): the list's movement field, else
    /// `wanders`.
    pub wanders: bool,
    /// Seconds before Wait (rule 12): the list's value, else the key's.
    pub birth_delay: f32,
    /// The list has `Birth` (and no `PreBirth`): not drawn, not solid
    /// until born (rule 16).
    pub unborn: bool,
    /// Its mouth textures (rule 25), if it has `lips=`.
    pub lips: Option<kq8_data::characters::Lips>,
    /// The skin swap (rule 4): the list's, else the keys'.
    pub skin: Option<(String, String)>,
    /// It casts a shadow (rule 36), as tested at load.
    pub casts_shadow: bool,
    /// What its movement rules use (`character-movement.md`; M5 5a-1).
    pub ai: kq8_data::characters::AiSettings,
    /// What it brings to the fight (`fighting.md` rule 2; M5 5c-1): only
    /// for a monster-list line.
    pub fight: Option<kq8_data::characters::FightSettings>,
    /// Loaded while `applyCombatLevel` was on (rule 3; M5 5c-10).
    pub combat_scaled: bool,
}

impl kq8_data::world_script::Host for Placer<'_> {
    fn load_progress(&mut self, percent: Option<f32>) {
        use std::sync::atomic::Ordering;
        if let Some(p) = &self.progress {
            // No number: one step on (rule 24).
            let now = p.load(Ordering::Relaxed);
            let next = percent.map_or(now + 1, |v| v.clamp(0.0, 100.0) as u32).min(100);
            p.store(next, Ordering::Relaxed);
        }
    }

    fn apply_combat_level(&mut self, on: bool, at: &str) {
        self.combat_apply = on;
        self.log.summary(format!("fight: {at}: applyCombatLevel {}: characters loaded from here on get the combat level (fighting.md rule 3)", u8::from(on)));
    }

    fn connor_line(&mut self, words: &[String], at: &str) -> bool {
        // `sendEvent <delay> <target> ...`: only those to Connor are kept.
        let for_connor = !words[0].eq_ignore_ascii_case("sendEvent") || words.get(2).is_some_and(|t| t.eq_ignore_ascii_case("Connor"));
        if for_connor {
            self.connor_start.push(kq8_core::connor_start::StartLine::Command { words: words.to_vec(), at: at.to_string() });
        }
        for_connor
    }

    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
        if self.skip_con_start && base.eq_ignore_ascii_case("conStart.cs") {
            return None;
        }
        self.world.load(base).and_then(Result::ok).map(|(_, b)| b)
    }

    fn load(&mut self, load: &kq8_data::world_script::LoadKq, vars: &kq8_data::world_script::Vars) {
        self.place(load, None, false, vars);
    }

    fn locate(&self, name: &str) -> Option<([f32; 3], [f32; 3])> {
        self.located.get(&name.to_ascii_lowercase()).map(|l| (l.position, l.angles))
    }

    fn set_hidden(&mut self, name: &str, hidden: bool, at: &str) {
        // An item object (`KQObject::setHidden $NewInvItem 1`, Daventry's
        // `BirthdaySuit`): not drawn, can't be clicked.
        if let Some(i) = self.items.index(name) {
            self.items.objects[i].hidden = hidden;
            self.item_lines.push(format!("{at}: setHidden {name} {}", u8::from(hidden)));
            return;
        }
        self.hide(name, hidden, at);
    }

    /// Items made and changed by the start-up scripts (items.md rule 9;
    /// `objects.md` Q17 rule 2 for z -1 / -2).
    fn item_command(&mut self, words: &[String], at: &str) -> Option<String> {
        use kq8_data::terrain::Level;
        use kq8_core::item::{read_make, Place};
        let command = words[0].to_ascii_lowercase();
        if command == "kqmonster::inventory" || command == "give" {
            let m = match read_make(words) {
                Ok(m) => m,
                Err(e) => {
                    self.item_lines.push(format!("{at}: {e}; nothing made"));
                    return None;
                }
            };
            let place = match m.position.filter(|_| m.in_world()) {
                Some(mut p) => {
                    if p[2] == -1.0 || p[2] == -2.0 {
                        let level = if p[2] == -1.0 { Level::Floor } else { Level::Surface };
                        match self.terrain.ground_at(level, p[0], p[1]) {
                            Some(g) => p[2] = g.height,
                            None => self.item_lines.push(format!("{at}: z {} outside the ground; kept", p[2])),
                        }
                    }
                    Place::World { position: p, angles: m.angles }
                }
                None => Place::Carried(m.who.clone()),
            };
            return match self.items.make(&m.kind, m.quantity, place) {
                Ok(o) => {
                    let name = o.name.clone();
                    if o.carried_by("Connor") {
                        self.connor_start.push(kq8_core::connor_start::StartLine::Item { name: name.clone(), at: at.to_string() });
                    }
                    self.item_lines.push(format!("{at}: made {}; NewInvItem = {name}", self.items.describe(&name)));
                    Some(name)
                }
                Err(e) => {
                    self.item_lines.push(format!("{at}: ERROR {}: {e}; nothing made", words.join(" ")));
                    None
                }
            };
        }
        // `KQObject::setScript` / `setDontLook <name> <value>`.
        let (Some(name), Some(value)) = (words.get(1), words.get(2)) else {
            self.item_lines.push(format!("{at}: usage: '{}'", words.join(" ")));
            return None;
        };
        let Some(i) = self.items.index(name) else {
            self.item_lines.push(format!("{at}: {}: not an item; not run at load", words.join(" ")));
            return None;
        };
        let o = &mut self.items.objects[i];
        if command == "kqobject::setscript" {
            o.script = (!value.eq_ignore_ascii_case("none")).then(|| value.clone());
        } else {
            o.do_not_point_at = value.parse::<f64>() != Ok(0.0);
        }
        self.item_lines.push(format!("{at}: {}", words.join(" ")));
        None
    }

    /// `move <who> <flags> x y z [rx ry rz]` (objects.md Q20 d).
    fn move_object(&mut self, m: &kq8_data::world_script::Move) {
        let at = format!("{} line {}", m.from, m.line);
        let note = |placer: &mut Self, what: &str, detail: String| {
            *placer.move_notes.entry(what.to_string()).or_default() += 1;
            placer.log.detail(format!("move {} ({at}): {what}{detail}", m.who));
        };
        let Some(l) = self.located.get(&m.who.to_ascii_lowercase()) else {
            note(self, "object not loaded, not moved", String::new());
            return;
        };
        if m.flags > 3 {
            note(self, "unknown flags, not moved", format!(" (flags {})", m.flags));
            return;
        }
        let (relative_position, relative_turn) = (m.flags & 1 != 0, m.flags & 2 != 0);
        let (old_position, old_angles, drawn) = (l.position, l.angles, l.drawn);
        let position = if relative_position {
            if old_angles != [0.0; 3] {
                // Along the world axes, not the object's own (objects.md Q21).
                note(self, "relative move of a turned object (along the world axes, Q21)", String::new());
            }
            [old_position[0] + m.position[0], old_position[1] + m.position[1], old_position[2] + m.position[2]]
        } else {
            if m.position[2] == -1.0 || m.position[2] == -2.0 {
                // Whether `move` puts -1/-2 on the ground is not traced (Q17 correction).
                note(self, "absolute z of -1/-2 kept as written (not traced)", String::new());
            }
            m.position
        };
        let angles = if !relative_turn {
            m.turn
        } else if m.turn == [0.0; 3] {
            old_angles
        } else if old_angles[..2] == [0.0; 2] && m.turn[..2] == [0.0; 2] {
            // Two turns about the same (vertical) axis add up, whatever the
            // order: the only case applied.
            note(self, "relative turn about Z added", format!(" {:?}", m.turn));
            [0.0, 0.0, old_angles[2] + m.turn[2]]
        } else {
            // How turns add up is not traced.
            note(self, "relative turn not applied (not traced)", format!(" {:?}", m.turn));
            old_angles
        };
        note(
            self,
            "moved",
            format!(
                ": flags {} from ({:.0}, {:.0}, {:.0}) to ({:.0}, {:.0}, {:.0}), angles {:?}",
                m.flags, old_position[0], old_position[1], old_position[2], position[0], position[1], position[2], angles
            ),
        );
        if let Some(l) = self.located.get_mut(&m.who.to_ascii_lowercase()) {
            l.position = position;
            l.angles = angles;
        }
        match drawn {
            Drawn::Model(i) => {
                self.placed[i].0.position = position;
                self.placed[i].0.angles = angles;
            }
            Drawn::Marker(i) => self.markers[i].position = position,
            Drawn::Nothing => {}
        }
    }
}

/// One item of Connor's start inventory (M4 4c-2).
#[derive(Debug, Clone)]
pub struct ConnorItem {
    /// Its section name, which becomes the item object's name.
    pub name: String,
    pub kind: String,
    pub quantity: i64,
    pub user_id: i64,
}

/// `inventory=<section>` of Connor's section, that section's `numItems`
/// and `ItemName<k>`, each naming a `KQInventoryItem` section with
/// `itemType`, `quantity` and `userId` (`inventory.md` rule 4; read from
/// the data).
fn connor_inventory(file: &[kq8_data::kq_text::Section], connor: &kq8_data::kq_text::Section) -> (Vec<ConnorItem>, Vec<String>) {
    let mut problems = Vec::new();
    let find = |name: &str| file.iter().find(|s| s.name.eq_ignore_ascii_case(name));
    let Some(list) = connor.get("inventory").map(str::trim) else { return (Vec::new(), vec!["no inventory= key".into()]) };
    let Some(list) = find(list) else { return (Vec::new(), vec![format!("no section [{list}]")]) };
    let n = list.number("numItems").and_then(Result::ok).unwrap_or(0.0) as usize;
    let mut items = Vec::new();
    for k in 0..n {
        let Some(name) = list.get(&format!("ItemName{k}")).map(str::trim) else {
            problems.push(format!("[{}] has no ItemName{k}", list.name));
            continue;
        };
        let Some(s) = find(name) else {
            problems.push(format!("no section [{name}]"));
            continue;
        };
        let Some(kind) = s.get("itemType").map(str::trim) else {
            problems.push(format!("[{name}] has no itemType"));
            continue;
        };
        let num = |key: &str, default: i64| s.number(key).and_then(Result::ok).map_or(default, |v| v as i64);
        items.push(ConnorItem { name: name.to_string(), kind: kind.to_string(), quantity: num("quantity", 1), user_id: num("userId", -1) });
    }
    (items, problems)
}

impl Placer<'_> {
    /// A shape drawn where the game puts it (a corpse, a bolt; `fighting.md`
    /// rules 32, 36), loaded once into `corpse_shapes`.
    fn load_corpse_shape(&mut self, shape: &str, who: &str) {
        let key = shape.to_ascii_lowercase();
        if self.corpse_shapes.contains_key(&key) {
            return;
        }
        let slot = match self.index_of.get(&key) {
            Some(slot) => *slot,
            None => {
                let slot = self.load_shape(shape, None, None);
                self.index_of.insert(key.clone(), slot);
                slot
            }
        };
        match slot {
            Some(i) => {
                self.corpse_shapes.insert(key, i);
            }
            None => self.character_lines.push(format!("[{who}]: shape {shape} could not be loaded")),
        }
    }

    /// A projectile file (`fighting.md` rule 36; `ranged-weapons.md` rules
    /// 26, 29), read once: its keys, and its shape's `BOUNDS.KQ` box.
    /// Returns its shape's name. `who`: for the log.
    fn load_projectile(&mut self, file: &str, who: &str) -> Option<String> {
        let key = file.to_ascii_lowercase();
        if !self.projectiles.contains_key(&key) {
            let keys = match self.world.load(file) {
                Some(Ok((_, b))) => kq8_data::kq_text::parse(&b).first().and_then(kq8_data::characters::ProjectileKeys::from_section),
                _ => None,
            };
            match keys {
                Some(mut k) => {
                    let table = self.game.find("GAME/patch/BOUNDS.KQ").and_then(|p| std::fs::read(p).ok()).map(|b| kq8_data::bounds::BoundingBoxes::parse(&b).0);
                    k.bounds = k.shape.as_ref().and_then(|s| table.as_ref()?.lookup(&s.to_ascii_lowercase())).map(|e| (e.min, e.max));
                    self.character_lines.push(format!(
                        "[{who}]: shoots {file}: shape {:?} (box {:?}), speed {}, range {}, mask 0x{:X}, weaponType {:?}, part {:?} (ranged-weapons.md rule 26){}",
                        k.shape,
                        k.bounds,
                        k.speed,
                        k.max_distance,
                        k.collider_mask,
                        k.weapon_type,
                        k.mesh_name,
                        if k.hammer { format!("; KQHammer: modifier {}, minDamage {}, maxDamage {} (azriel-hammer.md rule 6)", k.modifier, k.min_damage, k.max_damage) } else { String::new() }
                    ));
                    self.projectiles.insert(key.clone(), k);
                }
                None => self.character_lines.push(format!("[{who}]: projectile file {file} not found or not a KQProjectile / KQHammer")),
            }
        }
        self.projectiles.get(&key).and_then(|k| k.shape.clone())
    }

    /// Connor (M2 steps 2a, 2b; docs/DESIGN.md): the world script loads
    /// `connor.kq`, which is not in the world archives but in
    /// `GAME/Kq/Connor.kq` (connor-physics.md "Data used"). Placed by the
    /// script line with the usual ground rules. His movements come from
    /// `GAME/Kq/Conner.anm` (connor-movement.md), coloured with
    /// `KQWorld::connorPalette`; his keys from `MASK/Connor.sae`.
    fn place_connor(&mut self, load: &kq8_data::world_script::LoadKq) {
        use kq8_data::kq_text;
        let at = format!("{} line {}", load.from, load.line);
        let read = |rel: &str| -> Result<Vec<u8>, String> {
            let path = self.game.find(rel).ok_or(format!("{rel} not found in the game folder"))?;
            std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))
        };
        let result = (|| -> Result<(kq8_data::objects::Placement, ConnorAssets), String> {
            let file = kq_text::parse(&read("GAME/Kq/Connor.kq")?);
            let section = file.first().ok_or("GAME/Kq/Connor.kq has no section")?;
            let mut p = kq8_data::objects::Placement::from_load(section, load);
            self.attack_dist = section.number("attackDist").and_then(Result::ok).map(|v| v as f32);
            self.connor_stats = section.get("stats").map(|s| (s.to_string(), section.get("level").map(str::to_string)));
            // fighting.md rules 24, 39-41 (M5 5c-4): his ouch sound, the big
            // swing's factors, the hand weapons' movement lists.
            let num = |k: &str, d: f32| section.number(k).and_then(Result::ok).map_or(d, |v| v as f32);
            let lists = |key: &str| -> Vec<(String, String)> {
                section
                    .get(key)
                    .and_then(|n| file.iter().find(|s| s.name.eq_ignore_ascii_case(n.trim())))
                    .map(|s| {
                        s.entries
                            .iter()
                            .filter(|(k, _)| k.to_ascii_lowercase().starts_with("animlist") && k.len() > "animlist".len())
                            .filter_map(|(_, v)| {
                                let mut w = v.split_whitespace();
                                Some((w.next()?.to_string(), w.next()?.to_string()))
                            })
                            .collect()
                    })
                    .unwrap_or_default()
            };
            let short_lists = lists("ShortAnimList");
            // ranged-weapons.md rules 10, 17, 21 (M5 5f-4): the ranged
            // weapons' lists, the shoot offsets by item, the start values.
            let long_lists = lists("LongAnimList");
            let shoot_offsets: Vec<(String, [f32; 3])> = section
                .entries
                .iter()
                .filter(|(k, _)| k.to_ascii_lowercase().starts_with("shootoffsets"))
                .filter_map(|(_, v)| {
                    let w: Vec<&str> = v.split_whitespace().collect();
                    let n = |k: usize| w.get(k).and_then(|x| x.parse::<f32>().ok());
                    Some((w.first()?.to_string(), [n(1)?, n(2)?, n(3)?]))
                })
                .collect();
            let start_offset = [num("shootOffsetX", 0.0), num("shootOffsetY", 0.0), num("shootOffsetZ", 0.0)];
            let start_projectile = section.get("projectileFile").map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
            self.log.summary(format!(
                "connor: ranged (ranged-weapons.md rules 10, 17, 21): {} lists ({}); {} shoot offsets; start projectile {start_projectile:?}, offset {start_offset:?}",
                long_lists.len(),
                long_lists.iter().map(|(w, a)| format!("{w} {a}")).collect::<Vec<_>>().join(", "),
                shoot_offsets.len()
            ));
            self.connor_fight = Some(kq8_core::fight::ConnorFight {
                ouch_sound: section.get("ouchSound").map_or(kq8_core::fight::DEFAULT_OUCH.to_string(), |s| s.trim().to_string()),
                min_damage_factor: num("minDamageFactor", 1.0),
                max_damage_factor: num("maxDamageFactor", 1.0),
                short_lists,
                long_lists,
                shoot_offsets,
                start_projectile,
                start_offset,
            });
            self.connor_inventory = connor_inventory(&file, section);
            let (terrain, [x, y, _]) = (&self.terrain, p.position);
            p.position = p.grounded_position(|level| terrain.ground_at(level, x, y).map(|g| g.height));
            let (keys, problems) = kq8_data::action_map::parse(&read("Connor.sae")?);
            for pr in &problems {
                self.log.summary(format!("connor: Connor.sae: {pr}"));
            }
            let anm = kq_text::parse(&read("GAME/Kq/Conner.anm")?);
            let list = anm.first().map(|s| s.name.clone()).ok_or("Conner.anm has no section")?;
            let palette_name = world_palette_name(self.world, "connorPalette").unwrap_or_else(|| format!("{}.ppl", self.world.name));
            self.log.quiet = true;
            let palette = load_palette(self.world, Some(&palette_name), self.game, self.log);
            self.log.quiet = false;
            let palette = palette?;
            let (camera_keys, problems) = kq8_data::action_map::parse(&read("Camera.sae")?);
            for pr in &problems {
                self.log.summary(format!("connor: Camera.sae: {pr}"));
            }
            let collision_box = match self.game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) {
                Some(Ok(bytes)) => {
                    let (table, _) = kq8_data::bounds::BoundingBoxes::parse(&bytes);
                    let found: Vec<_> = table.find_all("Connor").collect();
                    if found.len() != 1 {
                        self.log.summary(format!("connor: BOUNDS.KQ has {} entries called Connor; using the first", found.len()));
                    }
                    found.first().map(|e| (e.min, e.max))
                }
                _ => None,
            };
            match collision_box {
                Some((min, max)) => self.log.summary(format!("connor: collision box from BOUNDS.KQ: {min:?} .. {max:?}")),
                None => self.log.summary("connor: no Connor entry read from GAME/patch/BOUNDS.KQ; the mover's built-in copy is used"),
            }
            // His mouth (M4 4a-6, talking.md rule 18): `baseLip`, and
            // `lip0`..`lip7` of the section `lips=` names.
            let lip_names = section.get("baseLip").zip(section.get("lips")).and_then(|(base, list)| {
                let s = file.iter().find(|s| s.name.eq_ignore_ascii_case(list.trim()))?;
                Some((base.trim().to_string(), (0..8).map(|n| s.get(&format!("lip{n}")).map(|v| v.trim().to_string())).collect::<Vec<_>>()))
            });
            let lips = lip_names.map(|(base, names)| {
                self.log.summary(format!("connor: mouth: baseLip {base}, lips {names:?} (talking.md rule 18)"));
                self.log.quiet = true;
                let looks = names.iter().map(|n| n.as_ref().map(|n| load_texture(self.world, self.game, n, &palette, self.log))).collect();
                self.log.quiet = false;
                (base, looks)
            });
            if lips.is_none() {
                self.log.summary("connor: Connor.kq gives no baseLip / lips: his mouth won't move");
            }
            // potions.md rules 25, 30 (M5 5g-5): his skin and face lists.
            let list_of = |key: &str| section.get(key).and_then(|n| file.iter().find(|s| s.name.eq_ignore_ascii_case(n.trim()))).and_then(kq8_data::characters::SkinList::from_section).unwrap_or_default();
            let mut potion_looks = PotionLooks { skins: list_of("skinList"), faces: list_of("lipsList"), ..Default::default() };
            self.log.quiet = true;
            potion_looks.body_skin = potion_looks.skins.exact(0).map(str::to_string);
            if let Some(iron) = potion_looks.skins.bit_of("INVITEM_Strength").and_then(|b| potion_looks.skins.exact(b)).map(str::to_string) {
                let look = load_texture(self.world, self.game, &iron, &palette, self.log);
                potion_looks.iron = Some((iron, look));
            }
            let mut names: Vec<String> = potion_looks.faces.entries.iter().map(|(n, _)| n.to_ascii_lowercase()).collect();
            names.sort();
            names.dedup();
            for name in names {
                // Only this world's pictures (some sets exist in no world).
                let set = (0..8)
                    .map(|n| format!("{name}{n}.bmp"))
                    .map(|f| self.world.load(&f).is_some().then(|| load_texture(self.world, self.game, &f, &palette, self.log)).filter(|l| matches!(l, Look::Texture { .. })))
                    .collect();
                potion_looks.face_sets.push((name, set));
            }
            self.log.quiet = false;
            self.log.summary(format!(
                "connor: skin list {} entries, body skin {:?}, Strength skin {:?}; face list {} entries, face sets {:?} (pictures found: {:?}) (potions.md rules 25, 30)",
                potion_looks.skins.entries.len(),
                potion_looks.body_skin,
                potion_looks.iron.as_ref().map(|(n, l)| (n.clone(), matches!(l, Look::Texture { .. }))),
                potion_looks.faces.entries.len(),
                potion_looks.face_sets.iter().map(|(n, _)| n.clone()).collect::<Vec<_>>(),
                potion_looks.face_sets.iter().map(|(_, s)| s.iter().filter(|l| l.is_some()).count()).collect::<Vec<_>>()
            ));
            let lip_base = lips.as_ref().map(|(b, _)| b.clone());
            let mut assets =
                ConnorAssets { movements: Vec::new(), animations: Vec::new(), keys, camera_keys, collision_box, lips, potion_looks, late: None };
            let mut index_of: BTreeMap<String, usize> = BTreeMap::new();
            // His own set (Conner.anm), then each hand weapon's (M5 5c fix:
            // **stand-in**, see `weapon_sets`), loaded as `<file>/<name>`.
            let mut sets = vec![(anm.clone(), list.to_string(), None)];
            let (weapons, late) = self.weapon_sets();
            sets.extend(weapons);
            self.log.summary(format!("connor: weapon sets not loaded now, loaded when the weapon is given (M6 6c-2): {:?}", late.iter().map(|(_, f)| f.clone()).collect::<Vec<_>>()));
            assets.late = Some(LateSets { game: self.game.clone(), world: self.world.name.clone(), palette, lip_base, sets: late });
            for (sections, set_list, file) in &sets {
                connor_set(self.game, self.world, &palette, self.log, sections, set_list, file.as_deref(), &mut assets.movements, &mut assets.animations, &mut index_of)?;
            }
            self.special_movements(&anm, &list, &palette, &mut assets, &mut index_of);
            // potions.md rule 29 (M5 5g-6): the `I_` versions of his
            // textures that this world has (the first two letters replaced).
            let mut names: Vec<String> = assets.animations.iter().flat_map(|a| a.looks.iter()).filter_map(|(_, l, _)| match l {
                Look::Texture { name, .. } => Some(name.rsplit(['/', '\\']).next().unwrap_or(name).to_ascii_lowercase()),
                _ => None,
            }).collect();
            names.sort();
            names.dedup();
            for name in names {
                let Some(ghost) = name.get(2..).map(|rest| format!("i_{rest}")) else { continue };
                if let Some(look) = load_ghost_texture(self.world, &ghost, &palette) {
                    assets.potion_looks.ghosts.push((name, look));
                }
            }
            self.log.summary(format!("connor: I_ textures for Invisible: {:?} (potions.md rule 29)", assets.potion_looks.ghosts.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>()));
            self.log.summary(format!("connor: palette {palette_name}; {} key bindings from Connor.sae", assets.keys.len()));
            Ok((p, assets))
        })();
        match result {
            Ok((p, assets)) => {
                self.log.summary(format!(
                    "connor: loaded by {at} at ({:.0}, {:.0}, {:.0}) angles ({:.2}, {:.2}, {:.2})",
                    p.position[0], p.position[1], p.position[2], p.angles[0], p.angles[1], p.angles[2]
                ));
                self.located.insert(
                    p.name.to_ascii_lowercase(),
                    Located { position: p.position, angles: p.angles, drawn: Drawn::Nothing },
                );
                self.connor = Some((p, assets));
            }
            Err(e) => self.log.summary(format!("connor: not loaded ({at}): {e}")),
        }
    }

    /// `KQObject::setHidden` from the scripts (objects.md Q18).
    fn hide(&mut self, name: &str, hidden: bool, at: &str) {
        let drawn = self.located.get(&name.to_ascii_lowercase()).map(|l| l.drawn);
        let what = match drawn {
            Some(Drawn::Model(i)) => {
                self.placed[i].0.hidden = hidden;
                "applied"
            }
            Some(_) => "object has no model (nothing to hide)",
            None => "object not loaded",
        };
        *self.hidden_values.entry(format!("setHidden {}: {what}", u8::from(hidden))).or_default() += 1;
        self.log.detail(format!("setHidden {name} {} ({at}): {what}", u8::from(hidden)));
    }

    /// M6 6d-2 (`magic-map.md` rule 3): a `loadKQ` of a map file, which is
    /// in `GAME/Kq`, not in the world's archives: it becomes the world's
    /// map (one at a time; a later one replaces it). `false` if the file is
    /// not there or not a `KQMap`.
    fn take_map(&mut self, load: &kq8_data::world_script::LoadKq, base: &str) -> bool {
        let rel = format!("GAME/Kq/{base}");
        let Some(bytes) = self.game.find(&rel).and_then(|p| std::fs::read(p).ok()) else { return false };
        let file = kq8_data::kq_text::parse(&bytes);
        match file.first().and_then(kq8_data::magic_map::MapKeys::from_section) {
            Some(Ok(keys)) => {
                let haze = self.terrain.settings.haze_distance.map(f64::from);
                self.log.summary(format!(
                    "map: {} line {}: {rel} [{}] is the world's map (magic-map.md rule 3): index {:?}, pictures {} / {}, fogRadius {:?}, hazeDistance {haze:?}{}",
                    load.from,
                    load.line,
                    keys.name,
                    keys.index(),
                    keys.blank_map,
                    keys.full_map,
                    keys.fog_radius,
                    if self.map.is_some() { "; replaces the one loaded before" } else { "" }
                ));
                self.map = Some(MapFile { label: rel, keys, haze });
                true
            }
            Some(Err(e)) => {
                self.log.summary(format!("map: {rel}: {e}; no map"));
                true
            }
            None => false,
        }
    }

    /// One object from a `loadKQ` line (or an unused `.kq` file).
    fn place(
        &mut self,
        load: &kq8_data::world_script::LoadKq,
        preloaded: Option<(String, Vec<u8>)>,
        unused: bool,
        vars: &kq8_data::world_script::Vars,
    ) {
        use kq8_data::kq_text;
        use kq8_data::objects::Placement;

        let base = load.file.rsplit(['/', '\\']).next().unwrap_or(&load.file);
        if !unused && base.eq_ignore_ascii_case("connor.kq") && self.world.load(base).is_none() {
            self.place_connor(load);
            return;
        }
        // A path like `light/Plight01.kq` may be a loose file in
        // `data/<World>/KQ/` (where the sun files are, object-lighting.md
        // rule 1).
        let loose = || {
            let rel = format!("data/{}/KQ/{}", self.world.name, load.file.replace('\\', "/"));
            let path = self.game.find(&rel).filter(|p| p.is_file())?;
            Some(std::fs::read(&path).map(|b| (rel, b)).map_err(|e| e.to_string()))
        };
        let (label, bytes) = match preloaded.map(Ok).or_else(|| self.world.load(base)).or_else(loose) {
            Some(Ok(x)) => x,
            _ => {
                if !unused && self.take_map(load, base) {
                    return;
                }
                self.log.detail(format!("skip {} line {}: {} not found", load.from, load.line, load.file));
                *self.skipped_map(unused).entry("loaded file not found".into()).or_default() += 1;
                return;
            }
        };
        let mut file = kq_text::parse(&bytes);
        // Values can be `$variables` too (objects.md Q20 b).
        for section in &mut file {
            for (k, v) in &mut section.entries {
                if !v.trim_start().starts_with('$') {
                    continue;
                }
                match vars.expand_value(v) {
                    Ok(text) => {
                        self.log.detail(format!("{label}[{}]: {k}={v} -> {text}", section.name));
                        *self.kq_variables.entry("replaced".into()).or_default() += usize::from(!unused);
                        *v = text;
                    }
                    Err(name) => {
                        self.log.detail(format!("{label}[{}]: {k}={v}: unknown variable {name}, kept as written", section.name));
                        *self.kq_variables.entry(format!("unknown {name}")).or_default() += usize::from(!unused);
                    }
                }
            }
        }
        // A world's monster list (characters.md rules 3, 4): each line
        // places its type's file.
        if !unused && file.first().and_then(|s| s.get("classType")).is_some_and(|c| c.trim().eq_ignore_ascii_case("KQMonsterManager")) {
            self.place_monster_list(&label, &file, load, vars);
            return;
        }
        // A character (M4 4d-2, rule 6): its movement list comes from
        // `animFile`; attached to the file as if the file carried it, so
        // the animated-object rules below apply unchanged.
        let character_keys = file.first().and_then(kq8_data::characters::CharacterKeys::from_section);
        let mut character_list = None;
        if let Some(keys) = &character_keys {
            match keys.anim_file.as_deref().map(|a| (a, self.read_anm(a))) {
                Some((_, Ok(list))) => {
                    file[0].entries.insert(0, ("animLoops".to_string(), list.list.name.clone()));
                    file.extend(kq_text::parse(&list.bytes));
                    character_list = Some(list.list);
                }
                Some((a, Err(e))) => self.character_lines.push(format!("{label}[{}]: animFile {a}: {e}; not drawn (marker)", file[0].name)),
                None => self.character_lines.push(format!("{label}[{}]: no animFile; not drawn (marker)", file[0].name)),
            }
        }
        if file.iter().filter(|x| x.get("locX").is_some()).count() > 1 {
            self.several_positions += 1;
            self.log.detail(format!("note {label}: several sections have a position; the first section is the object"));
        }
        // The object is the file's first section; the other sections
        // (animation lists, movements) belong to it.
        let Some(s) = file.first() else { return };
        let mut p = Placement::from_load(s, load);
        // A monster-list line (rule 4): renamed, its position and heading
        // replace the file's; z −1 = on the ground (the floor grid, as a
        // `loadKQ` −1); any other z as written, then `keepOnGround`.
        let pending = self.pending_line.take();
        if let Some(l) = &pending {
            p.name = l.line.name.clone();
            p.position = l.line.position;
            p.angles[2] = l.line.heading;
            p.z_on_ground = (l.line.position[2] == -1.0).then_some(kq8_data::terrain::Level::Floor);
            p.missing_loc = [false; 3];
        }
        let mut character = character_keys.zip(character_list).map(|(keys, list)| {
            let lips = match kq8_data::characters::Lips::of(&file, &keys) {
                Some(Ok(l)) => Some(l),
                Some(Err(e)) => {
                    self.character_lines.push(format!("[{}]: lips: {e}; no mouth change", p.name));
                    None
                }
                None => None,
            };
            let unborn = list.has("Birth") && !list.has("PreBirth");
            CharacterAssets {
                box_type: pending.as_ref().map(|l| l.type_name.clone()).or_else(|| keys.monster_type.clone()),
                from: pending.as_ref().map_or_else(|| format!("{label} ({} line {})", load.from, load.line), |l| l.from.clone()),
                wanders: pending.as_ref().map_or(keys.wanders, |l| l.wander.wanders()),
                birth_delay: pending.as_ref().map_or(keys.birth_delay, |l| l.birth_delay),
                unborn,
                lips,
                skin: pending.as_ref().and_then(|l| l.line.skin.clone()).or_else(|| keys.skin.clone()),
                ai: pending.as_ref().map_or_else(|| kq8_data::characters::AiSettings::from_keys(&keys), |l| kq8_data::characters::AiSettings::from_line(&keys, &l.line, &l.kind)),
                fight: pending.as_ref().map(|l| kq8_data::characters::FightSettings::from_line(&keys, &l.line, &l.kind, &self.items.types)),
                combat_scaled: self.combat_apply,
                keys,
                casts_shadow: false,
            }
        });
        // fighting.md rule 32 (M5 5c-7): its corpse shapes, ready for its death.
        if let Some(f) = character.as_ref().and_then(|c| c.fight.clone()) {
            self.drop_kinds.extend(f.drops.iter().map(|d| d.name.clone()));
            // fighting.md rule 36 (M5 5c-9): the projectile it shoots.
            let bolt_shape = f.projectile.clone().and_then(|file| self.load_projectile(&file, &p.name));
            for shape in [f.dead_dts, f.ided_kqs, f.projd_kqs, bolt_shape].into_iter().flatten() {
                self.load_corpse_shape(&shape, &p.name);
            }
        }
        // Missing locX/locY/locZ are 0 (objects.md Q19); only the file's
        // own position can have gaps.
        if load.position.is_none() && p.missing_loc.contains(&true) {
            let missing: Vec<&str> =
                ["locX", "locY", "locZ"].iter().zip(p.missing_loc).filter(|(_, m)| *m).map(|(k, _)| *k).collect();
            self.log.detail(format!("{label}[{}]: no {} in the file, taken as 0", p.name, missing.join("/")));
            *self.missing_loc_counts.entry(missing.join("/")).or_default() += usize::from(!unused);
        }
        if load.ignored_numbers > 0 && !unused {
            self.ignored_numbers += 1;
            self.log.detail(format!(
                "{label}[{}] ({} line {}): {} extra number(s) after x y z ignored, angles 0 0 0 (Q20 e)",
                p.name, load.from, load.line, load.ignored_numbers
            ));
        }
        if unused {
            // Templates for script copies sit at (0, 0, 0): nothing to see
            // there; one a script loads is kept as a template (M5 5d-3).
            if p.position == [0.0; 3] && !self.load_targets.contains(&base.to_ascii_lowercase()) {
                *self.unused_skipped.entry("at (0, 0, 0) (template)".into()).or_default() += 1;
                return;
            }
            let base_lc = base.to_ascii_lowercase();
            let named_in: Vec<&str> =
                self.all_scripts.iter().filter(|(_, t)| t.contains(&base_lc)).map(|(n, _)| n.as_str()).collect();
            self.log.detail(format!(
                "unused object {label}[{}] at ({:.0}, {:.0}, {:.0}); file named in scripts: {}",
                p.name,
                p.position[0],
                p.position[1],
                p.position[2],
                if named_in.is_empty() { "none".to_string() } else { named_in.join(", ") }
            ));
        } else {
            self.sections += 1;
        }
        if let Some(h) = s.get("hidden").filter(|_| !unused) {
            let with = if p.shape.is_some() || s.get("animLoops").is_some() { "with a model" } else { "no model" };
            *self.hidden_values.entry(format!("hidden={h} in the file ({with})")).or_default() += 1;
            self.log.detail(format!("[{}] hidden={h} groupsAlways={} -> drawn at the start: {}", p.name, p.groups_always, p.drawn()));
        }
        if load.inside_if {
            self.log.summary(format!("  NOTE {label}: loaded inside an if-block ({} line {}); placed anyway", load.from, load.line));
        }
        // A trap (M3 step 3d-2, traps.md rules 1 and 2): no model, its
        // position as written (keepOnGround ignored), a z of -1 on the
        // floor grid; it stays an invisible-object marker.
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQTrap")) {
            self.place_trap(&label, s, p);
            return;
        }
        // A conversation info object (M4 4e-3, conversation.md rule 3): no
        // model, no place of its own; its spot, areas and cameras.
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case(kq8_data::conv_info::CLASS)) {
            if let Some(info) = kq8_data::conv_info::ConvInfo::from_section(s) {
                self.character_lines.push(format!(
                    "{label}[{}]: conversation info for {}: spot ({:.0}, {:.0}, {:.0}), {} areas, {} cameras{}",
                    info.name,
                    info.partner(),
                    info.spot[0],
                    info.spot[1],
                    info.spot[2],
                    info.areas.len(),
                    info.cameras.len(),
                    if info.problems.is_empty() { String::new() } else { format!("; problems: {}", info.problems.join("; ")) }
                ));
                self.conv_infos.push(info);
                *self.by_class.entry("KQConvInfo (conversation info, M4)".into()).or_default() += 1;
                return;
            }
        }
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQPointLight")) {
            self.place_point_light(&label, s, p);
            return;
        }
        // A copy prepared for a script's `loadKQ <file>` while playing
        // (M3 3k-4): the file is named by such a line and has a place of
        // its own.
        let dormant = unused && p.position != [0.0; 3] && self.load_targets.contains(&base.to_ascii_lowercase());
        if dormant {
            let script = s.get("script").map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
            self.dormant.push((base.to_ascii_lowercase(), p.name.clone(), script));
            self.log.summary(format!("dormant {label}[{}]: prepared for a script's loadKQ (M3 3k-4)", p.name));
        }
        // A trap a script loads while playing (`ax.cs`'s `henchtp.kq`): its
        // box built as at the start (`objects.md` rule 6), switched off
        // until the `loadKQ` (traps.md rules 1, 2).
        if dormant && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQTrap")) {
            self.dormant_traps.push(p.name.to_ascii_lowercase());
            self.place_trap(&label, s, p);
            return;
        }
        if (!unused || dormant) && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQAudibleObject")) {
            self.place_audible(&label, s, p);
            if dormant {
                if let Some(a) = self.audibles.last_mut() {
                    a.dormant = true;
                }
            }
            return;
        }
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQMusic")) {
            self.place_music(&label, s, p);
            return;
        }

        // Ground rules, when the object is loaded, so `getLoc` sees the
        // result: loadKQ z of -1 = on the floor, -2 = on the surface
        // (objects.md Q17, water.md); otherwise keepOnGround raises onto
        // its grid, never lowers (Q10).
        let (terrain, [x, y, _]) = (self.terrain, p.position);
        let height = |level| terrain.ground_at(level, x, y).map(|g| g.height);
        let grounded = p.grounded_position(height);
        let level = p.ground_level();
        if p.z_on_ground.is_some() {
            self.log.detail(format!(
                "put [{}] on the {:?} grid (loadKQ z={:.0}, keepOnGround={:?}): z={}",
                p.name,
                level.unwrap(),
                p.position[2],
                p.keep_on_ground,
                level.and_then(height).map_or("none (outside the grid, kept as written)".into(), |g| format!("{g:.0}"))
            ));
            if level.and_then(height).is_some_and(|g| g != grounded[2]) {
                self.log.detail(format!(
                    "  then raised [{}] by keepOnGround onto the {:?} grid at z={:.0}",
                    p.name,
                    p.keep_on_ground.level().unwrap(),
                    grounded[2]
                ));
            }
            self.on_ground_marker += usize::from(!unused);
            p.position = grounded;
        } else if grounded != p.position {
            self.log.detail(format!(
                "raised [{}] from z={:.0} onto the {:?} grid at z={:.0}",
                p.name,
                p.position[2],
                level.unwrap(),
                grounded[2]
            ));
            self.raised += usize::from(!unused);
            p.position = grounded;
        }
        let key = p.name.to_ascii_lowercase();
        if let Some(v) = s.get("stats").filter(|_| !unused) {
            self.object_stats.push((p.name.clone(), v.to_string()));
        }
        if !unused {
            self.located.insert(key.clone(), Located { position: p.position, angles: p.angles, drawn: Drawn::Nothing });
            if p.class.as_deref().and_then(kq8_data::characters::Class::of).is_some() {
                let script = s.get("script").map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
                // conversation.md rule 20: its goodbye and welcome-back lines.
                let keys = kq8_data::characters::CharacterKeys::from_section(s);
                let (goodbye, welcome_back) = keys.map_or((None, None), |k| (k.goodbye, k.welcome_back));
                self.monsters.push(kq8_core::active::Monster { name: p.name.clone(), position: p.position, script, goodbye, welcome_back });
            }
            // 4d-7a: does it cast a shadow (characters.md rule 36)? The
            // world's mask from Options.cs (else the built-in 1);
            // `allowShadows` taken as on (its default; Daventry's world.cs
            // says 1).
            if let Some(c) = character.as_mut() {
                let world_mask = *self.shadow_world_mask.get_or_insert_with(|| {
                    self.game.find("Options.cs").and_then(|p| std::fs::read(p).ok()).and_then(|b| kq8_data::characters::shadow_mask_setting(&b)).unwrap_or(kq8_data::characters::BUILT_IN_SHADOW_MASK)
                });
                let casts = kq8_data::characters::casts_shadow(world_mask, c.keys.shadow_mask, true, p.translucency);
                c.casts_shadow = casts;
                self.character_lines.push(format!(
                    "shadow [{}]: {} (world mask {world_mask} AND shadowMask {} = {}; translucency {}; allowShadows on) (rule 36; drawn on the terrain, 4d-7b-2)",
                    p.name,
                    if casts { "CASTS a shadow" } else { "no shadow" },
                    c.keys.shadow_mask,
                    world_mask & c.keys.shadow_mask,
                    p.translucency
                ));
            }
            // 4d-6a: how far above the ground grids a character starts
            // (characters.md rule 8: gravity would bring it down).
            if let Some(c) = &character {
                let [x, y, z] = p.position;
                let show = |g: Option<kq8_data::terrain::Ground>| g.map_or("none".to_string(), |g| format!("{:.0} (gap {:.0})", g.height, z - g.height));
                self.character_lines.push(format!(
                    "gravity check [{}]: z {z:.0}; floor grid {}, surface {}; gravity {}, keepOnGround {:?}, flyingObject {}",
                    p.name,
                    show(self.terrain.floor_at(x, y)),
                    show(self.terrain.height_at(x, y)),
                    if c.keys.gravity { "on" } else { "off" },
                    p.keep_on_ground,
                    u8::from(c.keys.flying)
                ));
            }
        }

        // Animated objects name their model through their current
        // animation, shown at its start frame (objects.md "Animated
        // objects at load").
        let mut animation = None;
        if p.shape.is_none() {
            match kq8_data::objects::current_loop(&file, s) {
                Ok(Some(c)) => {
                    p.shape = Some(c.shape.clone());
                    animation = Some(c);
                }
                Ok(None) => {}
                Err(e) => {
                    self.log.detail(format!("skip {label}[{}]: animation lookup failed: {e}", p.name));
                    *self.skipped_map(unused).entry("animation lookup failed".into()).or_default() += 1;
                    return;
                }
            }
        }
        // An emitter has no model of its own (it stays an invisible-object
        // marker); its particles are drawn by `effects_view.rs`.
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQEmitter")) {
            self.place_emitter(&label, s, &p, load);
        }
        // A particle catcher: no model; its names are looked up once all
        // start-up scripts have run (emitters.md "Particle catchers" rule 2).
        // potions.md rule 31 (M5 5g-7): an object Clarity reveals.
        if !unused && p.groups_always & kq8_core::potions::REVEAL_BIT != 0 && !self.reveal_objects.iter().any(|n| n.eq_ignore_ascii_case(&p.name)) {
            self.reveal_objects.push(p.name.clone());
        }
        // M6 6e-6: a portal's keys, under its placed name, for the Portal
        // state (world-change.md rules 1-6).
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQPortal")) {
            if let Some(mut keys) = kq8_data::magic_map::PortalKeys::from_section(s) {
                keys.name = p.name.clone();
                self.portals.push(keys);
            }
        }
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQParticleCatcher")) {
            self.catchers.push((p.name.clone(), kq8_data::effects::CatcherSettings::from_section(s)));
        }
        // A ground animator: no model; it changes ground squares (3b-3).
        if !unused && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQTerrainAnimator")) {
            self.animators.push((p.name.clone(), kq8_data::effects::AnimatorSettings::from_section(s)));
        }
        if !unused && animation.is_some() && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQDoor")) {
            self.place_door(&label, &file, p, Mover::Door);
            return;
        }
        // A chest (3e-4): moves and is drawn like a door.
        if !unused && animation.is_some() && p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQChest")) {
            self.place_door(&label, &file, p, Mover::Chest);
            return;
        }
        // A clickable animated object with a script, still when loaded
        // (3e-5: `smlchst1`, `jar1`, the baskets): its script may play
        // another movement. One that already plays stays with `anim_view`.
        let scripted = p.class.as_deref().is_some_and(|c| c.eq_ignore_ascii_case("KQAnimObject"))
            && s.get("script").is_some_and(|v| !v.trim().is_empty())
            && collider_value(s, "colliderType", kq8_core::collide::DEFAULT_TYPE).0 & kq8_core::click::CLICKABLE != 0;
        if !unused && scripted {
            match &animation {
                Some(c) if c.speed == 0.0 => {
                    self.place_door(&label, &file, p, Mover::Object);
                    return;
                }
                _ => self.log.detail(format!("object {label}[{}]: scripted and clickable, but its movement plays at load: left to anim_view (not clickable yet)", p.name)),
            }
        }
        let reason = if p.has_matrix_key {
            Some("uses an RMat key (not supported)".to_string())
        } else {
            match &p.shape {
                None => Some(format!(
                    "no model ({})",
                    if s.get("animFile").is_some() { "uses an .anm file" } else { "no shapeName or animLoops" }
                )),
                Some(sh) if sh.to_ascii_lowercase().ends_with(".dig") && s.get("material").is_none() => {
                    Some("building without material=".to_string())
                }
                Some(sh) if sh.to_ascii_lowercase().ends_with(".dig") => None,
                Some(sh) if !sh.to_ascii_lowercase().ends_with(".kqs") => {
                    Some(format!("shapeName is not a .kqs ({})", sh.rsplit('.').next().unwrap_or("")))
                }
                Some(_) => None,
            }
        };
        if let Some(r) = reason {
            self.log.detail(format!("skip {label}[{}]: {r}", p.name));
            if p.shape.is_none() && !unused {
                self.markers.push(Marker {
                    name: p.name.clone(),
                    class: p.class.clone().unwrap_or_else(|| "?".into()),
                    position: p.position,
                    anm: s.get("animFile").is_some(),
                });
                if let Some(l) = self.located.get_mut(&key) {
                    l.drawn = Drawn::Marker(self.markers.len() - 1);
                }
            }
            *self.skipped_map(unused).entry(r).or_default() += 1;
            return;
        }
        let shape = p.shape.clone().unwrap();
        // One model per (file, start frame) combination.
        let model_key = match &animation {
            Some(c) => format!("{}|{}|{}|{}", shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase(), c.start, c.repeat),
            None => shape.to_ascii_lowercase(),
        };
        let start = animation.as_ref().map(|c| (c.sequence.as_str(), c.start, c.repeat));
        let material_list = s.get("material").map(str::to_string);
        let model_key = match &material_list {
            Some(m) => format!("{model_key}|{}", m.to_ascii_lowercase()),
            None => model_key,
        };
        let slot = match self.index_of.get(&model_key) {
            Some(slot) => *slot,
            None => {
                let slot = self.load_shape(&shape, material_list.as_deref(), start);
                self.index_of.insert(model_key, slot);
                slot
            }
        };
        let Some(i) = slot else {
            self.log.detail(format!("skip {label}[{}]: model {shape} could not be loaded", p.name));
            *self.skipped_map(unused).entry("model could not be loaded".into()).or_default() += 1;
            return;
        };
        let class = p.class.clone().unwrap_or("?".into());
        let boulder = class.eq_ignore_ascii_case("KQBoulder");
        if let Some(c) = &animation {
            self.log.detail(format!(
                "animated {label}[{}]: movement {}{} loop={} {} start={} speed={} repeat={} fourth={} -> {}{}",
                p.name,
                c.movement,
                if c.fell_back { " (no usable currAnimName: movement 0)" } else { "" },
                c.shape,
                c.sequence,
                c.start,
                c.speed,
                c.repeat,
                c.unknown,
                self.shapes[i].1.frame_note,
                p.flags.as_ref().map_or(String::new(), |f| format!(" flags={f}"))
            ));
            if c.fell_back && !unused {
                self.fell_back += 1;
            }
        }
        if let Some(f) = p.flags.clone().filter(|_| !unused) {
            *self.flag_values.entry(f).or_default() += 1;
        }
        if unused {
            self.unused_placed += 1;
        } else {
            *self.by_class.entry(if animation.is_some() { format!("{class} (animated)") } else { class }).or_default() += 1;
        }
        // Buildings default to type 0x28 (camera.md "Obstacles"), other
        // objects to 0x41.
        let default_type = if shape.to_ascii_lowercase().ends_with(".dig") { BUILDING_TYPE } else { kq8_core::collide::DEFAULT_TYPE };
        let (collider_type, problem_t) = collider_value(s, "colliderType", default_type);
        let (collider_mask, problem_m) = collider_value(s, "colliderMask", kq8_core::collide::DEFAULT_MASK);
        for problem in [problem_t, problem_m].into_iter().flatten() {
            self.log.summary(format!("collision: {label}[{}]: {problem}", p.name));
        }
        let text = |k: &str| s.get(k).map(|v| v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("").to_string()).filter(|v| !v.is_empty());
        let number = |k: &str| s.number(k).and_then(Result::ok);
        self.collision.push(CollisionInfo {
            shape,
            collider_type,
            collider_mask,
            script: text("script"),
            accepts_inventory: text("acceptsInventory"),
            accept_all_items: number("acceptAllItems").is_some_and(|v| v != 0.0),
            // clicking.md rule 10: buildings default to 1, others to 0.
            too_far_away_action: number("tooFarAwayAction").map_or(if default_type == BUILDING_TYPE { kq8_core::active::BUILDING_TOO_FAR_AWAY_ACTION } else { 0 }, |v| v as i64),
            do_not_point_at: number("doNotPointAt").is_some_and(|v| v != 0.0),
            footsound_type: number("footsoundType").map(|v| v as i64),
            ball_stack: number("collisionShape").is_some_and(|v| v == 1.0),
            character_box: character.as_ref().map(|c| c.box_type.clone()),
        });
        let flags = p.flags.clone();
        // M5 5d-3: a file a script loads, with no place of its own.
        if unused && p.position == [0.0; 3] && self.load_targets.contains(&base.to_ascii_lowercase()) && !self.template_files.iter().any(|(f, _)| f.eq_ignore_ascii_case(base)) {
            self.template_files.push((base.to_ascii_lowercase(), self.placed.len()));
            self.log.summary(format!("template {label}[{}]: kept for a script's loadKQ while playing (M5 5d-3)", p.name));
        }
        self.placed.push((p, i, unused));
        // A movement that plays (speed not 0), or more than one movement
        // (a script may switch it, 3i-4): every movement's frames (3c).
        // Not a `KQBoulder`: it controls its own animation (shown still
        // in the original), not studied yet (objects.md Q51 rule 8, step
        // 3r-2).
        let movement_count = kq8_data::objects::movement_loops(&file, s).ok().flatten().map_or(0, |m| m.len());
        if let Some(c) = animation.as_ref().filter(|c| c.speed != 0.0 || movement_count > 1) {
            if boulder {
                self.log.summary(format!(
                    "animated {label}[{}]: class KQBoulder: movement {} not played, drawn still at its start (objects.md Q51 rule 8)",
                    self.placed.last().map_or("?", |(p, _, _)| p.name.as_str()),
                    c.movement
                ));
            } else {
                self.place_animated(&label, &file, c, flags.as_deref(), character.clone());
            }
        }
        if let (false, Some(l)) = (unused, self.located.get_mut(&key)) {
            l.drawn = Drawn::Model(self.placed.len() - 1);
        }
    }

    /// The hand weapons' movement sets Connor switches to when one is
    /// ready (M5 5c fix, **stand-in**: `fighting.md` rule 39 and
    /// `inventory.md` rule 9 say the weapon is readied, not what it plays;
    /// the user saw the fists-up stance, and `confists.anm` holds a whole
    /// set: Q138): for each `ShortAnimList` entry whose weapon is in the
    /// world or named in a world script (`Fists` always), its file's
    /// sections, list name and file.
    /// The weapons left out: their item type (lower case) and list file
    /// (M6 6c-2: loaded when given).
    #[allow(clippy::type_complexity)]
    fn weapon_sets(&mut self) -> (Vec<(Vec<kq8_data::kq_text::Section>, String, Option<String>)>, Vec<(String, String)>) {
        let Some(f) = &self.connor_fight else { return (Vec::new(), Vec::new()) };
        let mut late = Vec::new();
        let kinds: Vec<String> = self.items.objects.iter().map(|o| o.kind.to_ascii_lowercase()).collect();
        // ranged-weapons.md rule 10 (M5 5f-4): the ranged weapons' lists
        // too, for those this world's object files name (a monster's drop:
        // `sprgnbow`'s Small Crossbow in `MONSTERS.KQ`); `NoRanged` is his
        // own list.
        let mut kq_text = String::new();
        for (_, _, bytes) in self.world.entries_ending(".kq") {
            if let Ok(b) = bytes {
                kq_text.push_str(&String::from_utf8_lossy(&b).to_ascii_lowercase());
            }
        }
        let mut sets = Vec::new();
        let all: Vec<(String, String, bool)> = f.short_lists.iter().map(|(w, a)| (w.clone(), a.clone(), false)).chain(f.long_lists.iter().map(|(w, a)| (w.clone(), a.clone(), true))).collect();
        for (weapon, file, ranged) in all {
            if ranged && weapon.eq_ignore_ascii_case("NoRanged") {
                continue;
            }
            let kind = format!("{}{weapon}", kq8_data::items::PREFIX).to_ascii_lowercase();
            // Also a weapon a world script names (made or given after
            // Connor is loaded: `WorldInv.cs`'s dagger and small axe; the
            // user's check, 5d fix 9).
            let w = weapon.to_ascii_lowercase();
            let named = self.all_scripts.iter().find(|(_, t)| t.to_ascii_lowercase().contains(&w)).map(|(n, _)| n.clone());
            let in_files = ranged && kq_text.contains(&kind);
            if !weapon.eq_ignore_ascii_case("Fists") && !kinds.contains(&kind) && named.is_none() && !in_files {
                late.push((kind, file));
                continue;
            }
            if let Some(n) = &named {
                self.log.summary(format!("connor: weapon set {file} ({weapon}): named in {n}"));
            }
            let sections = read_set(self.game, self.world, &file);
            match sections.first().map(|s| s.name.clone()) {
                Some(name) => sets.push((sections, name, Some(file))),
                None => self.log.summary(format!("connor: weapon set {file} ({weapon}): not found")),
            }
        }
        (sets, late)
    }

    /// The movements Connor's Special state can play (M3 step 3i-7,
    /// `anim-scripts.md` rule 16), found in the world's scripts
    /// (`KQMonster::setState Connor special <list>`): his own beyond
    /// [`CONNOR_MOVEMENTS`] from `Conner.anm`, and those of the lists named
    /// by `AnimList=` (read from the world's archives, else `GAME/Kq`),
    /// loaded as `<list>/<movement>`. Their transitions are not used
    /// (logged); their cues are.
    fn special_movements(
        &mut self,
        anm: &[kq8_data::kq_text::Section],
        list: &str,
        palette: &[Rgb; 256],
        assets: &mut ConnorAssets,
        index_of: &mut BTreeMap<String, usize>,
    ) {
        use kq8_data::kq_text;
        // What the scripts ask for: (list file or None, movement word).
        let mut wanted: std::collections::BTreeSet<(Option<String>, String)> = Default::default();
        let mut lines = 0;
        for (_, text) in &self.all_scripts {
            for line in text.lines().map(|l| l.split('#').next().unwrap_or("")) {
                let words: Vec<&str> = line.split_whitespace().collect();
                let Some(i) = words.windows(3).position(|w| w[0] == "kqmonster::setstate" && w[1] == "connor" && w[2] == "special") else { continue };
                lines += 1;
                let items = words[i + 3..].join(",").replace('\'', "");
                let record = std::cell::RefCell::new(Vec::new());
                let resolve = |l: Option<&str>, w: &str| -> Option<&'static str> {
                    record.borrow_mut().push((l.map(str::to_string), w.to_string()));
                    Some("")
                };
                kq8_core::connor::special_steps(&items, &resolve);
                wanted.extend(record.into_inner());
            }
        }
        // fighting.md rules 39-40 (M5 5c-5): the swings of each hand
        // weapon's list (Connor.kq's ShortAnimList); **stand-in**: only
        // these come from the weapon's list, the rest stays Conner.anm.
        if let Some(f) = &self.connor_fight {
            // 5d fix 10 (the user's check: the dagger's draw and sheath were
            // cut short): drawing starts with his own `away` (the hand to
            // the holster) and putting away ends with his own `out`, from
            // Conner.anm, in every world, not only where a script names them
            // (Swamp's sword and hammer had neither: the user's check).
            for m in ["away", "out"] {
                wanted.insert((None, m.to_string()));
            }
            for (_, file) in &f.short_lists {
                for m in kq8_core::fight::SWINGS {
                    wanted.insert((Some(file.clone()), m.to_string()));
                }
                // The swap (ranged-weapons.md rule 6, M5 5f-3); `away`
                // before a conversation (conversation.md rule 7).
                for m in ["draw", "undraw", "away", "away2", "out"] {
                    wanted.insert((Some(file.clone()), m.to_string()));
                }
            }
            // ranged-weapons.md rules 6, 20 (M5 5f-4): the ranged lists'
            // swap and shot.
            for (w, file) in &f.long_lists {
                if w.eq_ignore_ascii_case("NoRanged") {
                    continue;
                }
                for m in ["attack", "away", "away2", "out"] {
                    wanted.insert((Some(file.clone()), m.to_string()));
                }
                // azriel-hammer.md rules 1, 3 (M6 6b-3): the throw's wait
                // and catch.
                if w.eq_ignore_ascii_case("AzrielHammer") {
                    for m in ["wait", "catch"] {
                        wanted.insert((Some(file.clone()), m.to_string()));
                    }
                }
            }
        }
        // ropeandhook.md rule 13 (M6 6a-3): the rope list, in worlds whose
        // scripts have rope places.
        if self.all_scripts.iter().any(|(_, t)| t.to_ascii_lowercase().contains("kqconner::setrope")) {
            for m in kq8_core::rope::ROPE_MOVEMENTS {
                wanted.insert((Some(kq8_core::rope::ROPE_LIST.to_string()), m.to_string()));
            }
        }
        let mut loaded = Vec::new();
        let mut problems = Vec::new();
        let mut files: BTreeMap<String, Option<Vec<kq_text::Section>>> = BTreeMap::new();
        for (file, word) in wanted {
            let (sections, list_name, loaded_name) = match &file {
                None => {
                    if CONNOR_MOVEMENTS.iter().any(|m| m.eq_ignore_ascii_case(&word)) {
                        continue;
                    }
                    (anm.to_vec(), list.to_string(), None)
                }
                Some(f) => {
                    let sections = files.entry(f.clone()).or_insert_with(|| {
                        let bytes = match self.world.load(f) {
                            Some(Ok((_, b))) => Some(b),
                            _ => self.game.find(&format!("GAME/Kq/{f}")).and_then(|p| std::fs::read(p).ok()),
                        };
                        bytes.map(|b| kq_text::parse(&b))
                    });
                    let Some(sections) = sections.clone() else {
                        problems.push(format!("{f}: not found"));
                        continue;
                    };
                    let Some(name) = sections.first().map(|s| s.name.clone()) else {
                        problems.push(format!("{f}: no section"));
                        continue;
                    };
                    (sections, name, Some(f.clone()))
                }
            };
            let mut with_ask = sections;
            with_ask.extend(kq_text::parse(format!("[ConnorAsks]\nanimLoops={list_name}\ncurrAnimName={word}\n").as_bytes()));
            let c = match kq8_data::objects::current_loop(&with_ask, with_ask.last().unwrap()) {
                Ok(Some(c)) if !c.fell_back => c,
                _ => {
                    problems.push(format!("{}: no movement {word}", file.as_deref().unwrap_or("Conner.anm")));
                    continue;
                }
            };
            // The movement's name as its list spells it.
            let spelled = with_ask
                .iter()
                .find(|s| s.name.eq_ignore_ascii_case(&list_name))
                .and_then(|s| s.get(&format!("Movement{}", c.movement)))
                .unwrap_or(&word)
                .to_string();
            let name = match &loaded_name {
                Some(f) => kq8_core::connor::special_movement_name(f, &spelled),
                None => spelled,
            };
            if assets.movements.iter().any(|m| m.name.eq_ignore_ascii_case(&name)) {
                continue;
            }
            let key = format!("{}|{}", c.shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase());
            let animation = match index_of.get(&key) {
                Some(&i) => i,
                None => {
                    self.log.quiet = true;
                    let a = load_animation(self.game, self.world, &c.shape, &c.sequence, palette, self.log);
                    self.log.quiet = false;
                    match a {
                        Ok(a) => {
                            assets.animations.push(a);
                            index_of.insert(key, assets.animations.len() - 1);
                            assets.animations.len() - 1
                        }
                        Err(e) => {
                            problems.push(format!("{name}: {} {}: {e}", c.shape, c.sequence));
                            continue;
                        }
                    }
                }
            };
            if !c.transitions.is_empty() {
                self.log.detail(format!("connor: special {name}: {} transition(s) not used", c.transitions.len()));
            }
            let a = &assets.animations[animation];
            self.log.detail(format!(
                "connor: special {name}: loop={} {} start {} speed {} repeat {} cut short {}: {} frames, {:.4} s, steps ({:.1}, {:.1}, {:.1}), cues {:?}",
                c.shape, c.sequence, c.start, c.speed, c.repeat, c.unknown != 0, a.frames.len(), a.duration, a.step_sum[0], a.step_sum[1], a.step_sum[2], c.cues
            ));
            assets.movements.push(ConnorMovement {
                name: kq8_core::connor::intern(&name),
                animation,
                speed: c.speed,
                repeat: c.repeat,
                cut_short: c.unknown != 0,
                transitions: Vec::new(),
                cues: c.cues.clone(),
            });
            loaded.push(name);
        }
        self.log.summary(format!(
            "connor: special state (anim-scripts.md rule 16): {lines} setState lines in the scripts; {} movements loaded {loaded:?}; problems {problems:?}",
            loaded.len()
        ));
    }

    /// A door (M3 step 3a; docs/behaviour/doors.md): every movement with
    /// its cues and all frames of its shape (with their frame steps, which
    /// turn the door), and the keys clicking needs. Drawn and moved by
    /// `door_view.rs`, not as a still copy.
    fn place_door(&mut self, label: &str, file: &[kq8_data::kq_text::Section], p: kq8_data::objects::Placement, kind: Mover) {
        let chest = kind == Mover::Chest;
        let s = &file[0];
        let movements = match kq8_data::objects::movement_loops(file, s) {
            Ok(Some(m)) => m,
            Ok(None) => return,
            Err(e) => {
                self.log.summary(format!("door {label}[{}]: movements could not be read: {e}; not placed", p.name));
                return;
            }
        };
        let current = kq8_data::objects::current_loop(file, s).ok().flatten().map_or(0, |c| c.movement);
        let (game, world) = (self.game, self.world);
        let palette_name = self.palette_name.clone();
        let log = &mut *self.log;
        let palette = self.object_palette.get_or_insert_with(|| {
            log.quiet = true;
            let r = load_palette(world, Some(&palette_name), game, log);
            log.quiet = false;
            r
        });
        let palette = match palette {
            Ok(pal) => *pal,
            Err(e) => {
                let e = e.clone();
                self.log.summary(format!("door {label}[{}]: palette: {e}; not placed", p.name));
                return;
            }
        };
        let mut with_index = Vec::new();
        for (name, l) in movements {
            let key = format!("{}|{}", l.shape.to_ascii_lowercase(), l.sequence.to_ascii_lowercase());
            let index = match self.door_animation_index.get(&key) {
                Some(&i) => i,
                None => {
                    self.log.quiet = true;
                    let a = load_animation(game, world, &l.shape, &l.sequence, &palette, self.log);
                    self.log.quiet = false;
                    match a {
                        Ok(a) => {
                            self.door_animations.push(a);
                            self.door_animation_index.insert(key, self.door_animations.len() - 1);
                            self.door_animations.len() - 1
                        }
                        Err(e) => {
                            self.log.summary(format!("door {label}[{}]: movement {name}: {e}; door not placed", p.name));
                            return;
                        }
                    }
                }
            };
            for problem in l.cue_problems.iter().chain(&l.transition_problems) {
                self.log.detail(format!("door [{}]: {problem}", p.name));
            }
            with_index.push((name, l, index));
        }
        let text = |k: &str| s.get(k).map(|v| v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("").to_string());
        let number = |k: &str| s.number(k).and_then(Result::ok);
        // A chest's default type is its own (chests.md rule 2).
        let default_type = if chest { kq8_core::chest::CHEST_TYPE } else { kq8_core::collide::DEFAULT_TYPE };
        let (collider_type, problem_t) = collider_value(s, "colliderType", default_type);
        let (collider_mask, problem_m) = collider_value(s, "colliderMask", kq8_core::collide::DEFAULT_MASK);
        for problem in [problem_t, problem_m].into_iter().flatten() {
            self.log.summary(format!("door {label}[{}]: {problem}", p.name));
        }
        let shape = with_index.get(current).map_or(String::new(), |(_, l, _)| l.shape.clone());
        let chest = chest.then(|| {
            // chests.md rule 1.
            let f = |k: &str| number(k).map(|v| v as f32);
            let item_position = match (f("invLocX"), f("invLocY"), f("invLocZ")) {
                (Some(x), Some(y), Some(z)) => [x, y, z],
                _ => {
                    // Our choice: every chest in the data gives invLoc.
                    self.log.summary(format!("chest [{}]: no invLocX/Y/Z; the item would appear at the chest's own place", p.name));
                    p.position
                }
            };
            let remark = |prefix: &str| {
                let keys = ["Msg", "Noun", "Verb", "Case", "Seq", "End"].map(|k| number(&format!("{prefix}{k}")));
                keys.iter().any(Option::is_some).then(|| kq8_core::chest::Remark(keys.map(|v| v.unwrap_or(0.0) as i64)))
            };
            kq8_core::chest::ChestSettings {
                item: text("invItemTypeName").filter(|v| !v.is_empty()),
                quantity: number("invQuantity").map_or(1, |v| v as i64),
                item_position,
                item_angles: [f("invDirX").unwrap_or(0.0), f("invDirY").unwrap_or(0.0), f("invDirZ").unwrap_or(0.0)],
                take: remark("Take"),
                empty: remark("Empty"),
            }
        });
        let name_key = |k: &str| s.get(k).map(|v| v.trim().to_string());
        let door = DoorAssets {
            movements: with_index,
            current,
            anim_names: (name_key("currAnimName"), name_key("nextAnimName")),
            shape,
            script: text("script").filter(|v| !v.is_empty()),
            collider_type,
            collider_mask,
            accepts_inventory: text("acceptsInventory").filter(|v| !v.is_empty()),
            accept_all_items: number("acceptAllItems").is_some_and(|v| v != 0.0),
            too_far_away_action: number("tooFarAwayAction").map_or(0, |v| v as i64),
            do_not_point_at: number("doNotPointAt").is_some_and(|v| v != 0.0),
            bounds: None,
            placement: p,
            chest,
            object: kind == Mover::Object,
        };
        if let Some(v) = s.get("tooFar") {
            self.log.detail(format!("door [{}]: tooFar={v} not used (Q45 a)", door.placement.name));
        }
        let class = match kind {
            Mover::Door => "KQDoor (door, M3)",
            Mover::Chest => "KQChest (chest, M3)",
            Mover::Object => "KQAnimObject (scripted, clickable, M3 3e-5)",
        };
        *self.by_class.entry(class.into()).or_default() += 1;
        self.doors.push(door);
    }

    /// An emitter (M3 step 3b-1; docs/behaviour/emitters.md): its keys and
    /// its particle model, loaded once like any object model.
    fn place_trap(&mut self, label: &str, s: &kq8_data::kq_text::Section, mut p: kq8_data::objects::Placement) {
        let num = |key: &str| s.get(key).and_then(|v| v.trim().parse::<f32>().ok());
        if p.position[2] == -1.0 {
            let [x, y, _] = p.position;
            match self.terrain.ground_at(kq8_data::terrain::Level::Floor, x, y) {
                Some(g) => {
                    self.log.detail(format!("trap [{}]: locZ -1 -> floor grid height {:.0}", p.name, g.height));
                    p.position[2] = g.height;
                }
                None => self.log.detail(format!("trap [{}]: locZ -1 outside the floor grid; kept as -1", p.name)),
            }
        }
        let (mask, problem) = collider_value(s, "colliderMask", 0);
        if let Some(problem) = problem {
            self.log.detail(format!("trap [{}]: {problem}", p.name));
        }
        let trap_type = s.get("trapType").and_then(|v| v.trim().parse::<u32>().ok()).unwrap_or(0);
        let settings = kq8_core::trap::TrapSettings {
            name: p.name.clone(),
            position: p.position,
            min: [num("BBMinX").unwrap_or(0.0), num("BBMinY").unwrap_or(0.0), num("BBMinZ").unwrap_or(0.0)],
            max: [num("BBMaxX").unwrap_or(0.0), num("BBMaxY").unwrap_or(0.0), num("BBMaxZ").unwrap_or(0.0)],
            script: s.get("script").map(|v| v.trim().to_string()).filter(|v| !v.is_empty()),
            mask,
            trap_type,
            inner_percent: num("innerPercent"),
        };
        self.log.detail(format!(
            "trap {label}[{}] at ({:.0}, {:.0}, {:.0}): box {:?}..{:?}, mask {mask:#x}, trapType {trap_type}, innerPercent {:?}, script {:?}",
            p.name, p.position[0], p.position[1], p.position[2], settings.min, settings.max, settings.inner_percent, settings.script
        ));
        self.traps.push(settings);
        self.located.insert(p.name.to_ascii_lowercase(), Located { position: p.position, angles: p.angles, drawn: Drawn::Nothing });
        self.markers.push(Marker { name: p.name.clone(), class: "KQTrap".into(), position: p.position, anm: false });
        if let Some(l) = self.located.get_mut(&p.name.to_ascii_lowercase()) {
            l.drawn = Drawn::Marker(self.markers.len() - 1);
        }
        *self.by_class.entry("KQTrap (trap, M3)".into()).or_default() += 1;
    }

    /// A `KQAudibleObject` (M3 step 3h-3, sound.md rules 43-44): its sound
    /// keys, its position as written; `deltaX/Y/Z` are not used ("Do not
    /// reproduce" 7). A marker at its position (no model).
    fn place_audible(&mut self, label: &str, s: &kq8_data::kq_text::Section, p: kq8_data::objects::Placement) {
        let num = |key: &str| s.get(key).and_then(|v| v.split_whitespace().next()?.parse::<f32>().ok());
        let file = s.get("soundFileName").map(|v| v.trim().to_string()).filter(|v| !v.is_empty());
        let Some(file) = file else {
            self.log.summary(format!("sound object {label}[{}]: no soundFileName; left out", p.name));
            return;
        };
        let placed = num("isSound3D").is_some_and(|v| v != 0.0);
        let (min, max) = (num("minDistance").unwrap_or(0.0), num("maxDistance").unwrap_or(0.0));
        self.log.summary(format!(
            "sound object {label}[{}] at ({:.0}, {:.0}, {:.0}): {file}, {}, min {min:.0} max {max:.0}",
            p.name,
            p.position[0],
            p.position[1],
            p.position[2],
            if placed { "placed (isSound3D=1)" } else { "plain (isSound3D=0)" }
        ));
        self.audibles.push(kq8_core::sound::Audible::new(&p.name, &file, placed, min, max, p.position));
        self.located.insert(p.name.to_ascii_lowercase(), Located { position: p.position, angles: [0.0; 3], drawn: Drawn::Nothing });
        self.markers.push(Marker { name: p.name.clone(), class: "KQAudibleObject".into(), position: p.position, anm: false });
        if let Some(l) = self.located.get_mut(&p.name.to_ascii_lowercase()) {
            l.drawn = Drawn::Marker(self.markers.len() - 1);
        }
        *self.by_class.entry("KQAudibleObject (sound object, M3 3h-3)".into()).or_default() += 1;
    }

    /// The music object (M3 step 3h-5, sound.md rule 57): `nMusic` tunes
    /// `music0` .. `music<n-1>`, bottom first. A marker at its position.
    fn place_music(&mut self, label: &str, s: &kq8_data::kq_text::Section, p: kq8_data::objects::Placement) {
        let n = s.get("nMusic").and_then(|v| v.split_whitespace().next()?.parse::<usize>().ok()).unwrap_or(0);
        let mut names = Vec::new();
        for i in 0..n {
            match s.get(&format!("music{i}")).map(str::trim).filter(|v| !v.is_empty()) {
                Some(v) => names.push(v.to_string()),
                None => self.log.summary(format!("music object {label}[{}]: no music{i}; left out", p.name)),
            }
        }
        self.log.summary(format!("music object {label}[{}]: nMusic={n}, tunes bottom first [{}]", p.name, names.join(", ")));
        if let Some(old) = &self.music {
            self.log.summary(format!("  NOTE a second music object; it replaces [{}]", old.object));
        }
        self.music = Some(kq8_core::music::Music::new(&p.name, &names));
        self.located.insert(p.name.to_ascii_lowercase(), Located { position: p.position, angles: [0.0; 3], drawn: Drawn::Nothing });
        self.markers.push(Marker { name: p.name.clone(), class: "KQMusic".into(), position: p.position, anm: false });
        if let Some(l) = self.located.get_mut(&p.name.to_ascii_lowercase()) {
            l.drawn = Drawn::Marker(self.markers.len() - 1);
        }
        *self.by_class.entry("KQMusic (music, M3 3h-5)".into()).or_default() += 1;
    }

    /// A `KQPointLight` (object-lighting.md rules 24, 25): kept by name with
    /// its light; a marker at the light's position (no model).
    fn place_point_light(&mut self, label: &str, s: &kq8_data::kq_text::Section, p: kq8_data::objects::Placement) {
        let mut lights = kq8_data::lights::Lights::default();
        lights.add_sections(label, std::slice::from_ref(s));
        for problem in &lights.problems {
            self.log.summary(format!("point light {label}[{}]: {problem}", p.name));
        }
        let Some(light) = lights.points.first().copied() else {
            self.log.summary(format!("point light {label}[{}]: not a point light (tsLightType not 2?); left out", p.name));
            return;
        };
        self.log.summary(format!(
            "point light {label}[{}] at ({:.0}, {:.0}, {:.0}), colour ({:.2}, {:.2}, {:.2}), Begin {:.0}, End {:.0}",
            p.name, light.position[0], light.position[1], light.position[2], light.colour[0], light.colour[1], light.colour[2], light.begin, light.end
        ));
        self.point_lights.push((p.name.clone(), light));
        self.located.insert(p.name.to_ascii_lowercase(), Located { position: light.position, angles: [0.0; 3], drawn: Drawn::Nothing });
        self.markers.push(Marker { name: p.name.clone(), class: "KQPointLight".into(), position: light.position, anm: false });
        if let Some(l) = self.located.get_mut(&p.name.to_ascii_lowercase()) {
            l.drawn = Drawn::Marker(self.markers.len() - 1);
        }
        *self.by_class.entry("KQPointLight (point light, M1 7c)".into()).or_default() += 1;
    }

    fn place_emitter(
        &mut self,
        label: &str,
        s: &kq8_data::kq_text::Section,
        p: &kq8_data::objects::Placement,
        load: &kq8_data::world_script::LoadKq,
    ) {
        let settings = kq8_data::effects::EmitterSettings::from_section(s);
        for problem in &settings.problems {
            self.log.summary(format!("emitter {label}[{}]: {problem}", p.name));
        }
        let written = kq8_data::objects::Placement::from_load(s, load).position;
        let shape = settings.shape.clone().and_then(|shape| {
            let key = shape.to_ascii_lowercase();
            match self.index_of.get(&key) {
                Some(slot) => *slot,
                None => {
                    let slot = self.load_shape(&shape, None, None);
                    self.index_of.insert(key, slot);
                    if slot.is_none() {
                        self.log.summary(format!("emitter {label}[{}]: particle model {shape} could not be loaded", p.name));
                    }
                    slot
                }
            }
        });
        self.emitters.push(EmitterAssets {
            name: p.name.clone(),
            written,
            position: p.position,
            settings,
            translucency: p.translucency,
            haze: (p.haze_solid, p.haze_multiplier),
            too_far: p.too_far,
            groups: [p.groups_always, p.groups_near, p.groups_far],
            shape,
            ground: [kq8_data::terrain::Level::Floor, kq8_data::terrain::Level::Surface]
                .map(|level| self.terrain.ground_at(level, p.position[0], p.position[1]).map(|g| g.height)),
        });
    }

    /// A movement's frames, loaded once per (model, loop, skin): its index
    /// in `object_animations`, `None` if it can't be loaded.
    fn movement_animation(&mut self, label: &str, name: &str, l: &kq8_data::objects::CurrentLoop, skin: Option<&(String, String)>) -> Option<usize> {
        let mut key = format!("{}|{}", l.shape.to_ascii_lowercase(), l.sequence.to_ascii_lowercase());
        if let Some((a, b)) = skin {
            key = format!("{key}|{}>{}", a.to_ascii_lowercase(), b.to_ascii_lowercase());
        }
        if let Some(i) = self.object_animation_index.get(&key) {
            return *i;
        }
        let loaded = self.object_palette().and_then(|palette| {
            self.log.quiet = true;
            let a = load_animation_skinned(self.game, self.world, &l.shape, &l.sequence, &palette, skin, self.log);
            self.log.quiet = false;
            a
        });
        let i = match loaded {
            Ok(a) => {
                self.object_animations.push(a);
                Some(self.object_animations.len() - 1)
            }
            Err(e) => {
                self.log.summary(format!("animated {label}: movement {name}: {} {}: {e}; drawn still", l.shape, l.sequence));
                None
            }
        };
        self.object_animation_index.insert(key, i);
        i
    }

    /// The object palette, loaded once.
    fn object_palette(&mut self) -> Result<[Rgb; 256], String> {
        let (game, world) = (self.game, self.world);
        let palette_name = self.palette_name.clone();
        let log = &mut *self.log;
        let palette = self.object_palette.get_or_insert_with(|| {
            log.quiet = true;
            let r = load_palette(world, Some(&palette_name), game, log);
            log.quiet = false;
            r
        });
        palette.clone()
    }

    /// The object just placed plays its movement (M3 step 3c): its loop's
    /// frames, loaded once per (model, loop).
    /// An animated object for `anim_view` and the script rules (3c, 3i-4):
    /// every movement's frames. If one can't be loaded the object is drawn
    /// still (as before for its current movement).
    fn place_animated(
        &mut self,
        label: &str,
        file: &[kq8_data::kq_text::Section],
        c: &kq8_data::objects::CurrentLoop,
        flags: Option<&str>,
        character: Option<CharacterAssets>,
    ) {
        // A character's skin swap (characters.md rule 4) gives its own
        // copies of the animations that use the texture.
        let skin = character.as_ref().and_then(|c| c.skin.clone());
        // Its mouth textures (rule 25).
        let mut lip_looks = Vec::new();
        if let Some(lips) = character.as_ref().and_then(|c| c.lips.clone()) {
            if let Ok(palette) = self.object_palette() {
                self.log.quiet = true;
                lip_looks = lips.shapes.iter().map(|n| n.as_ref().map(|n| load_texture(self.world, self.game, n, &palette, self.log))).collect();
                self.log.quiet = false;
            }
        }
        let s = &file[0];
        let listed = match kq8_data::objects::movement_loops(file, s) {
            Ok(Some(m)) => m,
            Ok(None) => return,
            Err(e) => {
                self.log.summary(format!("animated {label}: movements could not be read: {e}; drawn still"));
                return;
            }
        };
        let mut movements = Vec::with_capacity(listed.len());
        for (name, l) in listed {
            let Some(index) = self.movement_animation(label, &name, &l, skin.as_ref()) else { return };
            for problem in l.cue_problems.iter().chain(&l.transition_problems) {
                self.log.detail(format!("animated {label}: {problem}"));
            }
            if !l.transitions.is_empty() {
                // anim-scripts.md rule 4.2: no placed object has any (data);
                // characters play them (characters.md rule 20, M4 4d-11).
                let who = if character.is_some() { "a character's: played (characters.md rule 20)" } else { "on an object: played as for characters (anim-scripts.md rule 4.2 says none exist)" };
                self.log.summary(format!("animated {label}: movement {name} has transitions, {who}: {:?}", l.transitions.iter().map(|t| format!("{} -> {} via {} from {}", name, t.to, t.via, t.from)).collect::<Vec<_>>()));
            }
            movements.push((name, l, index));
        }
        // character-movement.md rules 15, 18 (M5 5a-6): the idle list's
        // fidgets, as `idle:<name>` after its own movements.
        if let Some(idle) = character.as_ref().and_then(|c| c.keys.idle_anim_file.clone()) {
            match self.read_anm(&idle) {
                Ok(list) => {
                    let file = kq8_data::kq_text::parse(&list.bytes);
                    let head = kq8_data::kq_text::Section { name: "idle".into(), entries: vec![("animLoops".to_string(), list.list.name.clone())] };
                    match kq8_data::objects::movement_loops(&file, &head) {
                        Ok(Some(listed)) => {
                            let mut names = Vec::new();
                            for (name, l) in listed {
                                if let Some(index) = self.movement_animation(label, &name, &l, skin.as_ref()) {
                                    names.push(name.clone());
                                    movements.push((format!("{}{name}", kq8_core::character_ai::IDLE_PREFIX), l, index));
                                }
                            }
                            self.character_lines.push(format!("{label}: idleAnimFile {idle}: fidgets {names:?} (character-movement.md rule 15)"));
                        }
                        Ok(None) => self.character_lines.push(format!("{label}: idleAnimFile {idle}: no movements")),
                        Err(e) => self.character_lines.push(format!("{label}: idleAnimFile {idle}: {e}")),
                    }
                }
                Err(e) => self.character_lines.push(format!("{label}: idleAnimFile {idle}: {e}; no fidgets")),
            }
        }
        // `flags` bit 8 (objects.md rule 4); a value that is not a whole
        // number counts as no flags.
        let flag_bits = flags.and_then(|f| f.trim().parse::<i64>().ok()).unwrap_or(0) as u32;
        let text = |k: &str| s.get(k).map(|v| v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("").to_string()).filter(|v| !v.is_empty());
        let name_key = |k: &str| s.get(k).map(|v| v.trim().to_string());
        self.animated.push(AnimatedAssets {
            placed: self.placed.len() - 1,
            movements,
            current: c.movement,
            anim_names: (name_key("currAnimName"), name_key("nextAnimName")),
            random: flag_bits & kq8_core::anim::RANDOM_START != 0,
            script: text("script"),
            once: s.get("classType").is_some_and(|c| c.trim().eq_ignore_ascii_case("KQAnimOnce")),
            lip_looks,
            character,
        });
    }

    /// A character's `.anm` list (characters.md rule 6): the world's
    /// archives, else `GAME/Kq` (the search order 3i-7 uses for
    /// `AnimList=` files).
    fn read_anm(&mut self, name: &str) -> Result<LoadedAnm, String> {
        let bytes = match self.world.load(name) {
            Some(r) => r.map(|(_, b)| b)?,
            None => {
                let path = self.game.find(&format!("GAME/Kq/{name}")).ok_or("not found in the world or GAME/Kq")?;
                std::fs::read(&path).map_err(|e| format!("{}: {e}", path.display()))?
            }
        };
        let list = kq8_data::characters::AnimList::parse(&bytes)?;
        Ok(LoadedAnm { list, bytes })
    }

    /// `loadKQ monsters.kq` (characters.md rules 3, 4): each line's type
    /// (`GAME/Kq/Monstats.mtl`) names a file of the world, read as if
    /// loaded by `loadKQ`, then renamed and placed by the line. An unknown
    /// type makes nothing.
    fn place_monster_list(&mut self, label: &str, file: &[kq8_data::kq_text::Section], load: &kq8_data::world_script::LoadKq, vars: &kq8_data::world_script::Vars) {
        use kq8_data::characters::{MonsterList, MonsterTypes};
        let (list, problems) = MonsterList::from_section(&file[0]);
        for p in &problems {
            self.log.summary(format!("characters: {label}: {p}"));
        }
        if self.monster_types.is_none() {
            let types = match self.game.find("GAME/Kq/Monstats.mtl").map(std::fs::read) {
                Some(Ok(bytes)) => {
                    let (t, problems) = MonsterTypes::parse(&bytes);
                    for p in &problems {
                        self.log.summary(format!("characters: Monstats.mtl: {p}"));
                    }
                    t
                }
                _ => {
                    self.log.summary("characters: GAME/Kq/Monstats.mtl could not be read: no monster-list character is made");
                    MonsterTypes::default()
                }
            };
            self.monster_types = Some(types);
        }
        let mut made = 0;
        for line in list.lines {
            let found = self.monster_types.as_ref().and_then(|t| t.find(&line.type_name)).map(|(t, folded)| (t.clone(), folded));
            let Some((kind, folded)) = found else {
                self.character_lines.push(format!("{label} {} {}: KQMonster::add - Monster type undefined ('{}'): nothing made", line.key, line.name, line.type_name));
                continue;
            };
            let bytes = match self.world.load(&kind.file) {
                Some(Ok(x)) => x,
                _ => {
                    self.character_lines.push(format!("{label} {} {}: type {} file {} not in the world: nothing made", line.key, line.name, kind.name, kind.file));
                    continue;
                }
            };
            self.list_files.insert(kind.file.to_ascii_lowercase());
            let from = format!("{label} {} (type {}{}, file {})", line.key, kind.name, if folded { ", matched ignoring capitals" } else { "" }, kind.file);
            let sub = kq8_data::world_script::LoadKq {
                file: kind.file.clone(),
                name: kq8_data::world_script::LoadName::Keep,
                position: None,
                angles: None,
                ignored_numbers: 0,
                from: from.clone(),
                line: load.line,
                inside_if: load.inside_if,
            };
            self.pending_line = Some(PendingLine {
                wander: line.movement.or(kind.movement.clone()),
                birth_delay: line.birth_delay.or(kind.birth_delay),
                type_name: kind.name.clone(),
                kind: kind.clone(),
                line,
                from,
            });
            self.place(&sub, Some(bytes), false, vars);
            self.pending_line = None;
            made += 1;
        }
        self.log.summary(format!("characters: {label}: {made} of {:?} monster-list lines placed (characters.md rule 4)", list.declared));
    }

    fn skipped_map(&mut self, unused: bool) -> &mut BTreeMap<String, usize> {
        if unused { &mut self.unused_skipped } else { &mut self.skipped }
    }

    /// Loads a model or building once; `None` if it fails.
    fn load_shape(&mut self, shape: &str, material_list: Option<&str>, start: StartAt) -> Option<usize> {
        // Per-model lines go to the file only.
        self.log.quiet = true;
        let r = if shape.to_ascii_lowercase().ends_with(".dig") {
            let (world, game) = (self.world, self.game);
            let log = &mut *self.log;
            let palette = self.interior_palette.get_or_insert_with(|| {
                let name = world_palette_name(world, "interiorPalette").unwrap_or_else(|| format!("{}.ppl", world.name));
                log.summary(format!("buildings: palette {name} (KQWorld::interiorPalette)"));
                load_palette(world, Some(&name), game, log)
            });
            match palette {
                Ok(pal) => load_building(game, world, shape, material_list.unwrap_or_default(), pal, log),
                Err(e) => Err(e.clone()),
            }
        } else {
            load_model(self.game, self.world, shape, Some(&self.palette_name), start, self.log)
        };
        self.log.quiet = false;
        match r {
            Ok(m) => {
                self.shapes.push((shape.to_string(), m));
                Some(self.shapes.len() - 1)
            }
            Err(e) => {
                self.log.detail(format!("model {shape} failed: {e}"));
                None
            }
        }
    }

    /// Summary lines, then the result.
    /// Loads the model of every item type that can appear in this world:
    /// those the start-up scripts made, those the world's scripts make or
    /// throw (`KQMonster::inventory`, `give`, `KQObject::tossItems`) and
    /// those its chests hold (`invItemTypeName`). Returns `IDName` ->
    /// model, and the lines for `items.log`.
    fn item_models(&mut self, script: &kq8_data::world_script::ScriptRun) -> (BTreeMap<String, usize>, Vec<String>) {
        let mut named: Vec<String> = self.items.objects.iter().map(|o| o.kind.clone()).collect();
        // fighting.md rule 31 (M5 5c-8): what the fighters drop.
        named.extend(self.drop_kinds.iter().cloned());
        for (_, text) in &self.all_scripts {
            for line in text.lines() {
                let w: Vec<&str> = line.split('#').next().unwrap_or("").split_whitespace().collect();
                match w.first().copied() {
                    Some("kqmonster::inventory") => named.extend(w.get(2).map(|s| s.to_string())),
                    Some("give") => named.extend(w.get(1).map(|s| s.to_string())),
                    Some("kqobject::tossitems") => named.extend(w.get(3).into_iter().flat_map(|v| v.split(',')).map(str::to_string)),
                    _ => {}
                }
            }
        }
        for (_, _, bytes) in self.world.entries_ending(".kq") {
            let Ok(bytes) = bytes else { continue };
            for s in kq8_data::kq_text::parse(&bytes) {
                named.extend(s.get("invItemTypeName").map(str::to_string));
            }
        }
        // ranged-weapons.md rule 21 (M5 5f-4): the bolt files of the ranged
        // weapons that can appear here, and their shapes; also those of
        // every other ranged weapon whose file this world has, for a
        // console `give` (azriel-hammer.md rule 10: Daventry's
        // `prhammer.kq`; M6 6b-3).
        let mut ranged: Vec<(String, String)> = named
            .iter()
            .filter_map(|n| self.items.kind(n.strip_prefix("invitem_").unwrap_or(n)))
            .filter(|t| t.category == "LongWeapon")
            .filter_map(|t| Some((t.name.clone(), t.extra.clone()?)))
            .collect();
        for t in self.items.types.types.iter().filter(|t| t.category == "LongWeapon") {
            if let Some(file) = t.extra.clone().filter(|f| !ranged.iter().any(|(_, r)| r.eq_ignore_ascii_case(f)) && matches!(self.world.load(f), Some(Ok(_)))) {
                ranged.push((t.name.clone(), file));
            }
        }
        for (kind, file) in ranged {
            if let Some(shape) = self.load_projectile(&file, &format!("Connor's {kind}")) {
                self.load_corpse_shape(&shape, &kind);
            }
        }
        // potions.md rule 20 (M5 5g-4): the shield's orbits. The game tries
        // `orbit1.kq`, `orbit2.kq` and `orbit3.kq`; all three are in
        // `GAME/Kq` (the note said only the first exists: Q145), the same
        // shield 2.09 rad apart.
        for n in 1..=3 {
            let file = format!("orbit{n}.kq");
            let bytes = match self.world.load(&file) {
                Some(Ok((_, b))) => Some(b),
                _ => self.game.find(&format!("GAME/Kq/{file}")).and_then(|p| std::fs::read(p).ok()),
            };
            match bytes.and_then(|b| kq8_data::kq_text::parse(&b).first().and_then(kq8_data::characters::OrbitKeys::from_section)) {
                Some(k) => {
                    let slot = k.shape.clone().and_then(|s| {
                        self.load_corpse_shape(&s, &format!("orbit{n}"));
                        self.corpse_shapes.get(&s.to_ascii_lowercase()).copied()
                    });
                    self.item_lines.push(format!("{file}: shape {:?} (model slot {slot:?}), radius {}, start {:?}, step {:?} rad per update, z offset {}, translucency {} (potions.md rule 20)", k.shape, k.radius, k.start_angle, k.step_angle, k.z_offset, k.translucency));
                    self.orbit.push((k, slot));
                }
                None => self.item_lines.push(format!("{file} not found or not a KQOrbitObj: no orbit from it (potions.md rule 20)")),
            }
        }
        let mut shapes = BTreeMap::new();
        let mut failed = Vec::new();
        let mut unknown = std::collections::BTreeSet::new();
        for name in named {
            // The scripts' text was lower-cased (`all_scripts`), so the
            // exact `INVITEM_` of rule 2 is gone: strip it to look up.
            let name = name.strip_prefix("invitem_").map_or(name.clone(), str::to_string);
            let Some(t) = self.items.kind(&name).cloned() else {
                if !name.starts_with('$') {
                    unknown.insert(name);
                }
                continue;
            };
            let failure = format!("{} ({})", t.name, t.shape);
            if shapes.contains_key(&t.name) || failed.contains(&failure) {
                continue;
            }
            let key = t.shape.to_ascii_lowercase();
            let slot = match self.index_of.get(&key) {
                Some(slot) => *slot,
                None => {
                    let slot = self.load_shape(&t.shape, None, None);
                    self.index_of.insert(key, slot);
                    slot
                }
            };
            match slot {
                Some(i) => {
                    shapes.insert(t.name.clone(), i);
                }
                // Models are looked up in this world and `GAME/Common`
                // only (as objects').
                None => failed.push(failure),
            }
        }
        let it = &self.items.objects;
        let mut lines = std::mem::take(&mut self.item_lines);
        let summary = format!(
            "items: start-up scripts: {} item lines ({} inside if-blocks, run anyway); {} items made: {} in the world ({} hidden), {} carried; models for {} types; failed {failed:?}; names not in the type list: {unknown:?}",
            script.item_commands,
            script.item_commands_inside_if,
            it.len(),
            it.iter().filter(|o| o.in_world()).count(),
            it.iter().filter(|o| o.hidden).count(),
            it.iter().filter(|o| !o.in_world()).count(),
            shapes.len()
        );
        self.log.summary(&summary);
        lines.insert(0, summary);
        lines.push(format!("item types with a model here: {:?}", shapes.keys().collect::<Vec<_>>()));
        (shapes, lines)
    }

    fn finish(self) -> Objects {
        let log = self.log;
        // Check from the numbers: lowest point of each placed model vs ground.
        let mut gaps = Vec::new();
        for (p, i, _) in self.placed.iter().filter(|x| !x.2) {
            let t = p.transform();
            let lowest = self.shapes[*i]
                .1
                .groups
                .iter()
                .flat_map(|g| g.triangles.iter().flat_map(|tri| tri.positions))
                .map(|v| t.apply(v)[2])
                .fold(f32::INFINITY, f32::min);
            let ground = self.terrain.floor_at(p.position[0], p.position[1]).map(|g| g.height);
            let gap = ground.map(|h| lowest - h);
            log.detail(format!(
                "object [{}] class={} shape={} at ({:.0}, {:.0}, {:.0}) angles=({:.2}, {:.2}, {:.2}) lowest_point={lowest:.0} ground={} gap={}",
                p.name,
                p.class.as_deref().unwrap_or("?"),
                self.shapes[*i].0,
                p.position[0],
                p.position[1],
                p.position[2],
                p.angles[0],
                p.angles[1],
                p.angles[2],
                ground.map_or("none".into(), |h| format!("{h:.0}")),
                gap.map_or("-".into(), |g| format!("{g:.0}"))
            ));
            if let Some(g) = gap {
                gaps.push(g);
            }
        }
        let count = |f: &dyn Fn(f32) -> bool| gaps.iter().filter(|g| f(**g)).count();
        log.summary(format!(
            "objects: {} loaded objects, {} placed using {} different models; skipped: {:?}",
            self.sections,
            self.placed.len() - self.unused_placed,
            self.shapes.len(),
            self.skipped
        ));
        log.summary(format!(
            "  unused objects (in .kq files no start-up script loads; shown with key H): {} placed; skipped: {:?}",
            self.unused_placed, self.unused_skipped
        ));
        log.summary(format!("  placed per class: {:?}", self.by_class));
        let hidden: Vec<&str> =
            self.placed.iter().filter(|(p, _, unused)| !unused && !p.drawn()).map(|(p, _, _)| p.name.as_str()).collect();
        log.summary(format!(
            "  hidden (Q18): {:?}; placed but hidden at the start (shown with key H): {} {hidden:?}",
            self.hidden_values,
            hidden.len()
        ));
        log.summary(format!("  loaded objects with position keys missing (taken as 0, Q19): {:?}", self.missing_loc_counts));
        log.summary(format!(
            "  .kq values that are $variables: {:?}; loadKQ lines with extra numbers ignored (Q20 e): {}; move lines: {:?}",
            self.kq_variables, self.ignored_numbers, self.move_notes
        ));
        log.summary(format!("  files where several sections have a position (first used): {}", self.several_positions));
        log.summary(format!(
            "  put on the ground by a loadKQ z of -1/-2: {}; raised onto the ground (keepOnGround): {}; animated objects without a usable currAnimName (movement 0 used): {}; 'flags' values: {:?}",
            self.on_ground_marker, self.raised, self.fell_back, self.flag_values
        ));
        log.summary(format!(
            "  lowest point minus ground: within 50 units={} 50..500 above={} more than 500 above={} 50..500 below={} more than 500 below={}",
            count(&|g| g.abs() <= 50.0),
            count(&|g| g > 50.0 && g <= 500.0),
            count(&|g| g > 500.0),
            count(&|g| (-500.0..-50.0).contains(&g)),
            count(&|g| g < -500.0)
        ));
        log.summary(format!(
            "  invisible objects (markers, key H): {} with no model, {} characters in .anm files",
            self.markers.iter().filter(|m| !m.anm).count(),
            self.markers.iter().filter(|m| m.anm).count()
        ));
        for m in &self.markers {
            log.detail(format!(
                "marker [{}] class={} at ({:.0}, {:.0}, {:.0}){}",
                m.name, m.class, m.position[0], m.position[1], m.position[2], if m.anm { " (.anm character)" } else { "" }
            ));
        }
        // The camera object the world script loads (`cam3rd.kq`, named
        // KQCamera): where the following camera starts (Q27 d).
        let camera_start = self.located.get("kqcamera").map(|l| l.position);
        // Building colliders, placed exactly as drawn (connor-physics.md
        // "Against a building").
        let mut buildings = Vec::new();
        let mut building_types = Vec::new();
        let mut building_clicks = Vec::new();
        let mut footsounds = BTreeMap::new();
        let mut digs: BTreeMap<String, Option<kq8_data::building::Building>> = BTreeMap::new();
        for (k, (p, i, unused)) in self.placed.iter().enumerate() {
            let shape = &self.shapes[*i].0;
            if *unused || !shape.to_ascii_lowercase().ends_with(".dig") {
                continue;
            }
            let dig = digs.entry(shape.to_ascii_lowercase()).or_insert_with(|| {
                let bytes = self.world.load(shape)?.ok()?;
                kq8_data::building::Building::parse(&bytes.1).ok()
            });
            let Some(dig) = dig else {
                log.summary(format!("collision: [{}] {shape} could not be read; no collider", p.name));
                continue;
            };
            let placement = kq8_core::collide::Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position };
            let c = kq8_core::collide::BuildingCollider::new(&p.name, dig, placement);
            log.detail(format!(
                "collision: building [{}] {shape}: {} nodes, {} leaves, world box ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0})",
                p.name,
                dig.nodes.len(),
                dig.leaf_flags.len(),
                c.world_min[0],
                c.world_min[1],
                c.world_min[2],
                c.world_max[0],
                c.world_max[1],
                c.world_max[2]
            ));
            let info = &self.collision[k];
            building_clicks.push(kq8_core::active::ActiveBuilding {
                name: p.name.clone(),
                collision_type: info.collider_type,
                script: info.script.clone(),
                accepts_inventory: info.accepts_inventory.clone(),
                accept_all_items: info.accept_all_items,
                too_far_away_action: info.too_far_away_action,
                do_not_point_at: info.do_not_point_at,
                bounds: c.bounds,
                placement: c.placement,
            });
            footsounds.insert(p.name.to_ascii_lowercase(), info.footsound_type);
            if let Some(n) = info.footsound_type {
                log.summary(format!("footsteps: building [{}] footsoundType={n} (sound.md rule 39)", p.name));
            }
            buildings.push(c);
            building_types.push(info.collider_type);
        }
        log.summary(format!("collision: {} building colliders", buildings.len()));
        let (objects, clickables, view_boxes) = object_colliders(self.game, &self.shapes, &self.placed, &self.collision, &self.markers, log);
        // M5 5d-3: the templates, with their boxes from BOUNDS.KQ.
        let table = self.game.find("GAME/patch/BOUNDS.KQ").and_then(|p| std::fs::read(p).ok()).map(|b| kq8_data::bounds::BoundingBoxes::parse(&b).0);
        let templates: Vec<TemplateAssets> = self
            .template_files
            .iter()
            .map(|(file, k)| {
                let (p, _, _) = &self.placed[*k];
                let info = &self.collision[*k];
                let bounds = table.as_ref().and_then(|t| t.lookup(&info.shape.to_ascii_lowercase()).or_else(|| t.lookup(&p.name))).map(|e| (e.min, e.max));
                let animated = self.animated.iter().any(|a| a.placed == *k);
                log.summary(format!(
                    "template {file}: [{}] {}, script {:?}, box {bounds:?}, type 0x{:X} mask 0x{:X}",
                    p.name,
                    if animated { "animated" } else { "still" },
                    info.script,
                    info.collider_type,
                    info.collider_mask
                ));
                TemplateAssets { file: file.clone(), placed: *k, name: p.name.clone(), script: info.script.clone(), bounds, collider_type: info.collider_type, collider_mask: info.collider_mask, animated }
            })
            .collect();
        let not_blocking: Vec<String> = buildings
            .iter()
            .zip(&building_types)
            .filter(|(_, t)| *t & VIEW_BLOCKING == 0)
            .map(|(b, t): (&kq8_core::collide::BuildingCollider, &u32)| format!("{} ({t:#x})", b.name))
            .collect();
        log.summary(format!(
            "camera view blockers: {} of {} buildings (not: {not_blocking:?}), {} object boxes (bit 0x8 or 0x1000) {:?}, plus doors as they stand",
            buildings.len() - not_blocking.len(),
            buildings.len(),
            view_boxes.len(),
            view_boxes.iter().map(|b| b.name.as_str()).collect::<Vec<_>>()
        ));
        let mut doors = self.doors;
        door_bounds(self.game, &mut doors, log);
        // Emitters follow any `move` of their object.
        let mut emitters = self.emitters;
        for e in &mut emitters {
            if let Some(l) = self.located.get(&e.name.to_ascii_lowercase()) {
                e.position = l.position;
            }
        }
        let mut traps = self.traps;
        for t in &mut traps {
            if let Some(l) = self.located.get(&t.name.to_ascii_lowercase()) {
                t.position = l.position;
            }
        }
        // Monsters where the start-up scripts' `move` left them.
        let mut monsters = self.monsters;
        for m in &mut monsters {
            if let Some(l) = self.located.get(&m.name.to_ascii_lowercase()) {
                m.position = l.position;
            }
        }
        let mut known: Vec<String> = self.located.keys().cloned().collect();
        known.extend(self.markers.iter().map(|m| m.name.to_ascii_lowercase()));
        let shadow_world_mask = self.shadow_world_mask.unwrap_or_else(|| {
            self.game.find("Options.cs").and_then(|p| std::fs::read(p).ok()).and_then(|b| kq8_data::characters::shadow_mask_setting(&b)).unwrap_or(kq8_data::characters::BUILT_IN_SHADOW_MASK)
        });
        Objects {
            map: self.map,
            shadow_world_mask,
            shapes: self.shapes,
            placed: self.placed,
            markers: self.markers,
            connor: self.connor,
            camera_start,
            buildings,
            building_types,
            building_clicks,
            view_boxes,
            objects,
            doors,
            door_animations: self.door_animations,
            clickables,
            known,
            vars: Default::default(),
            scripts: Vec::new(),
            attack_dist: self.attack_dist,
            connor_stats: self.connor_stats,
            connor_fight: self.connor_fight,
            connor_inventory: self.connor_inventory,
            connor_start: self.connor_start,
            object_stats: self.object_stats,
            monsters,
            talk: None,
            game_bounds: match self.game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) {
                Some(Ok(bytes)) => Some(kq8_data::bounds::BoundingBoxes::parse(&bytes).0),
                _ => None,
            },
            emitters,
            catchers: self.catchers,
            traps,
            conv_infos: self.conv_infos,
            audibles: self.audibles,
            dormant: self.dormant,
            dormant_traps: self.dormant_traps,
            templates,
            music: self.music,
            footsounds,
            point_lights: self.point_lights,
            animators: self.animators,
            animated: self.animated,
            object_animations: self.object_animations,
            items: Default::default(),
            item_shapes: BTreeMap::new(),
            corpse_shapes: self.corpse_shapes,
            projectiles: self.projectiles,
            orbit: self.orbit,
            reveal_objects: self.reveal_objects,
            portals: self.portals,
            portal_files: self.portal_files,
            item_lines: Vec::new(),
            character_lines: self.character_lines,
            ground: None,
            max_monsters: None,
        }
    }
}

/// The object colliders (M2 step 2d-5; connor-physics.md "Collision
/// shape", "Which pairs collide", "When an object takes part"): every
/// object the start-up scripts load and we place, apart from buildings,
/// with its place after `move`, hidden ones too. The box comes from
/// `BOUNDS.KQ`, first by the shape's name, then by the object's own name;
/// none: never collides. Names are compared with case (Q34 a), but the
/// shape's name is lower-cased first: our stand-in until Q39 (every shape
/// entry in the table is lower case; 25 Daventry objects write theirs
/// with capitals). Type 0 and mask 0: never collides. Only pairs
/// with Connor are kept (nothing else moves yet). `KQMonster`s are ball
/// stacks (Connor against them is not traced; none are placed in M2).
fn object_colliders(
    game: &GameDir,
    shapes: &[(String, Model)],
    placed: &[(kq8_data::objects::Placement, usize, bool)],
    collision: &[CollisionInfo],
    markers: &[Marker],
    log: &mut Log,
) -> (Vec<kq8_core::collide::BoxCollider>, Vec<kq8_core::active::Clickable>, Vec<kq8_core::collide::BoxCollider>) {
    use kq8_core::collide::{pair_collides, BoxCollider, ColliderKind, Placement, CONNOR_MASK, CONNOR_TYPE};
    // Its collision settings (3j).
    let kind = |info: &CollisionInfo, p: &kq8_data::objects::Placement| ColliderKind {
        collision_type: info.collider_type,
        collision_mask: info.collider_mask,
        ball_stack: info.ball_stack,
        keep_on_ground: p.keep_on_ground.level().is_some(),
    };
    let table = match game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) {
        Some(Ok(bytes)) => kq8_data::bounds::BoundingBoxes::parse(&bytes).0,
        _ => {
            log.summary("collision: GAME/patch/BOUNDS.KQ could not be read: no object colliders");
            return (Vec::new(), Vec::new(), Vec::new());
        }
    };
    let mut objects = Vec::new();
    let mut clickables = Vec::new();
    let mut view_boxes = Vec::new();
    let mut clickable_without_box = 0;
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    let mut left_out: BTreeMap<&str, usize> = BTreeMap::new();
    let mut by_name = 0;
    let mut case_only = Vec::new();
    let mut lowered = 0;
    let mut fixed = Vec::new();
    for ((p, i, unused), info) in placed.iter().zip(collision) {
        if *unused || shapes[*i].0.to_ascii_lowercase().ends_with(".dig") {
            continue;
        }
        let class = p.class.as_deref().unwrap_or("?");
        let what = format!("[{}] class={class} shape={}", p.name, info.shape);
        if let Some(box_type) = &info.character_box {
            // A character (M4 4d-2): its box by type, else by name
            // (characters.md rule 9). Clickable with it (doors.md rule
            // 10); never solid for Connor yet (a ball stack, M5).
            *left_out.entry("character (not solid for Connor yet)").or_default() += 1;
            let found = box_type.as_deref().and_then(|t| table.lookup(t).map(|e| (e, "type"))).or_else(|| table.lookup(&p.name).map(|e| (e, "name")));
            let Some((entry, from)) = found else {
                log.detail(format!("collision: {what}: character with no box (type {box_type:?}, name): can't be clicked"));
                clickable_without_box += usize::from(info.collider_type & kq8_core::click::CLICKABLE != 0);
                continue;
            };
            if info.collider_type & kq8_core::click::CLICKABLE != 0 {
                let mut c = kq8_core::active::Clickable::plain(
                    &p.name,
                    class,
                    entry.min,
                    entry.max,
                    Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position },
                    info.collider_type,
                );
                c.script = info.script.clone();
                c.accepts_inventory = info.accepts_inventory.clone();
                c.accept_all_items = info.accept_all_items;
                c.too_far_away_action = info.too_far_away_action;
                c.do_not_point_at = info.do_not_point_at;
                log.detail(format!("collision: {what}: character, clickable, box {} by {from}", entry.name));
                clickables.push(c);
            }
            continue;
        }
        if info.collider_type == 0 && info.collider_mask == 0 {
            *left_out.entry("type 0 and mask 0").or_default() += 1;
            log.detail(format!("collision: {what}: type 0 and mask 0: never collides"));
            continue;
        }
        let shape_lc = info.shape.to_ascii_lowercase();
        let found = table.lookup(&shape_lc).map(|e| (e, "shape")).or_else(|| table.lookup(&p.name).map(|e| (e, "name")));
        if found.as_ref().is_some_and(|(_, from)| *from == "shape") && shape_lc != info.shape && table.lookup(&info.shape).is_none() {
            lowered += 1;
            log.detail(format!("collision: {what}: found only with the shape name lower-cased (Q39 stand-in)"));
        }
        let Some((entry, from)) = found else {
            let other_case = table.find_all(&p.name).next();
            if let Some(e) = other_case {
                case_only.push(format!("{} ({})", p.name, e.name));
                *left_out.entry("not in BOUNDS.KQ (only with other case)").or_default() += 1;
            } else {
                *left_out.entry("not in BOUNDS.KQ").or_default() += 1;
            }
            log.detail(format!("collision: {what}: no box in BOUNDS.KQ: never collides"));
            clickable_without_box += usize::from(info.collider_type & kq8_core::click::CLICKABLE != 0);
            continue;
        };
        // Clickable (doors.md rule 10): picked by its box, whatever it
        // collides with.
        // Blocks the camera's view (camera.md "Obstacles"), whatever it
        // collides with.
        if info.collider_type & VIEW_BLOCKING != 0 {
            view_boxes.push(BoxCollider {
                name: p.name.clone(),
                min: entry.min,
                max: entry.max,
                placement: Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position },
                kind: kind(info, p),
            });
        }
        // Not clickable but with a script (the barrels, type 0x141): kept
        // too, for its script (`clicking.md` rule 21: every object with a
        // script gets `Init`) and Connor's hits (M5 5c fix 5); the click
        // pick tests the clickable bit itself.
        if info.collider_type & kq8_core::click::CLICKABLE != 0 || info.script.is_some() {
            let mut c = kq8_core::active::Clickable::plain(
                &p.name,
                class,
                entry.min,
                entry.max,
                Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position },
                info.collider_type,
            );
            // 3f: its script and click keys.
            c.script = info.script.clone();
            c.accepts_inventory = info.accepts_inventory.clone();
            c.accept_all_items = info.accept_all_items;
            c.too_far_away_action = info.too_far_away_action;
            c.do_not_point_at = info.do_not_point_at;
            clickables.push(c);
        }
        if !pair_collides(CONNOR_TYPE, CONNOR_MASK, info.collider_type, info.collider_mask) {
            *left_out.entry("type and mask: not tested against Connor").or_default() += 1;
            log.detail(format!(
                "collision: {what}: type {:#x} mask {:#x}: not tested against Connor (type 4, mask 0x281b)",
                info.collider_type, info.collider_mask
            ));
            continue;
        }
        by_name += usize::from(from == "name");
        let written = table.entries.iter().find(|e| e.item == entry.item);
        if written.is_some_and(|w| w.min != entry.min) {
            fixed.push(format!("{} ({})", p.name, entry.name));
        }
        let placement = Placement { rotation: kq8_data::objects::rotation(p.angles), position: p.position };
        log.detail(format!(
            "collision: {what}: box item{}={} by its {from} ({:.0}, {:.0}, {:.0})..({:.0}, {:.0}, {:.0}), type {:#x} mask {:#x}, at ({:.0}, {:.0}, {:.0}) angles ({:.2}, {:.2}, {:.2}){}",
            entry.item,
            entry.name,
            entry.min[0],
            entry.min[1],
            entry.min[2],
            entry.max[0],
            entry.max[1],
            entry.max[2],
            info.collider_type,
            info.collider_mask,
            p.position[0],
            p.position[1],
            p.position[2],
            p.angles[0],
            p.angles[1],
            p.angles[2],
            if p.drawn() { "" } else { " (hidden: still collides)" }
        ));
        *by_class.entry(class.to_string()).or_default() += 1;
        objects.push(BoxCollider { name: p.name.clone(), min: entry.min, max: entry.max, placement, kind: kind(info, p) });
    }
    log.summary(format!(
        "collision: {} object colliders (per class {by_class:?}; {by_name} found by the object's own name, the rest by shape; {lowered} only with the shape name lower-cased, Q39); left out: {left_out:?}",
        objects.len()
    ));
    if !case_only.is_empty() {
        log.summary(format!("collision: in BOUNDS.KQ only with other case (left out, Q34 a): {case_only:?}"));
    }
    if !fixed.is_empty() {
        log.summary(format!("collision: inside-out boxes fixed (Q34 c): {fixed:?}"));
    }
    log.summary(format!(
        "collision: no collider (no model of ours): {} markers, {} of them .anm characters",
        markers.len(),
        markers.iter().filter(|m| m.anm).count()
    ));
    log.summary(format!(
        "click: {} clickable objects other than doors (collision type bit 0x20, with a box); {clickable_without_box} clickable without a box (can't be picked)",
        clickables.len()
    ));
    for c in &clickables {
        log.detail(format!("click: clickable [{}] class {} type {:#x}", c.name, c.class, c.collision_type));
    }
    (objects, clickables, view_boxes)
}

/// Each door's box from `BOUNDS.KQ`, by the same rule as other objects
/// (shape name lower-cased first, then the object's own name).
fn door_bounds(game: &GameDir, doors: &mut [DoorAssets], log: &mut Log) {
    let Some(Ok(bytes)) = game.find("GAME/patch/BOUNDS.KQ").map(std::fs::read) else {
        log.summary("doors: GAME/patch/BOUNDS.KQ could not be read: doors have no box");
        return;
    };
    let table = kq8_data::bounds::BoundingBoxes::parse(&bytes).0;
    for d in doors {
        let found = table.lookup(&d.shape.to_ascii_lowercase()).or_else(|| table.lookup(&d.placement.name));
        d.bounds = found.as_ref().map(|e| (e.min, e.max));
        match found {
            Some(e) => log.detail(format!("doors: [{}] box item{}={} ({:?}..{:?})", d.placement.name, e.item, e.name, e.min, e.max)),
            None => log.summary(format!("doors: [{}] {} has no box in BOUNDS.KQ: can't be clicked, doesn't collide", d.placement.name, d.shape)),
        }
    }
}

/// A building (`.dig`) as a `Model`, so it is placed and drawn like any
/// object (rules: docs/behaviour/buildings.md). Not used: lighting
/// (`.dil`), the visibility tables.
pub fn load_building(
    game: &GameDir,
    world: &World,
    dig: &str,
    material_list: &str,
    palette: &[Rgb; 256],
    log: &mut Log,
) -> Result<Model, String> {
    use kq8_data::building::Building;
    let (label, bytes) = world.load(dig).ok_or(format!("building '{dig}' not found in {}", world.name))??;
    let b = Building::parse(&bytes).map_err(|e| format!("{label}: {e}"))?;
    let (list_at, list_bytes) = world.load(material_list).ok_or(format!("material list '{material_list}' not found"))??;
    let list = kq8_data::MaterialList::parse(&list_bytes).map_err(|e| format!("{list_at}: {e}"))?;
    let (tris, bad) = b.triangles();
    log.summary(format!(
        "building: {label} v{} surfaces={} triangles={} bad_surfaces={bad} materials from {list_at} ({})",
        b.version,
        b.surfaces.len(),
        tris.len(),
        list.materials.len()
    ));
    // Its light maps (M1 5c, object-lighting.md rules 12-16).
    let light = building_light_maps(world, dig, &b, log);
    let atlas = match &light {
        Some(l) => crate::building_light::LightAtlas::build(l),
        None => crate::building_light::LightAtlas::white(b.surfaces.len()),
    };
    // Surfaces with no light map and material 255 (Castle D's 96, which
    // have no material either) are not drawn (Q67 c).
    let no_map = |i: usize| light.as_ref().and_then(|l| l.surfaces.get(i)).is_some_and(|s| !s.has_map());
    let mut skipped_surfaces = std::collections::BTreeSet::new();
    let mut by_material: BTreeMap<u8, Vec<kq8_data::building::BuildingTriangle>> = BTreeMap::new();
    for t in tris {
        if t.material == 255 && no_map(t.surface) {
            skipped_surfaces.insert(t.surface);
            continue;
        }
        by_material.entry(t.material).or_default().push(t);
    }
    let lit = (0..b.surfaces.len()).filter(|&i| atlas.places.get(i).is_some_and(|p| p.is_some())).count();
    let without = (0..b.surfaces.len()).filter(|&i| no_map(i) && !skipped_surfaces.contains(&i)).count();
    log.summary(format!(
        "  light: {lit} of {} surfaces lit by their map; {without} without a map (full bright); {} not drawn (no map, material 255); light picture {}x{}",
        b.surfaces.len(),
        skipped_surfaces.len(),
        atlas.width,
        atlas.height
    ));
    let mut groups = Vec::new();
    let mut not_drawn = 0;
    for (index, tris) in by_material {
        let m = list.materials.get(index as usize);
        let look = match m {
            None => Look::Missing(format!("material index {index} not in list of {}", list.materials.len())),
            Some(m) if m.kind() == MaterialKind::Texture && !m.texture.is_empty() => {
                load_texture(world, game, &m.texture, palette, log)
            }
            Some(m) if m.kind() == MaterialKind::Rgb => Look::Color(m.rgb),
            Some(m) => {
                // Kind 0 = nothing to draw (models-and-textures.md
                // "Material flags"): leave these surfaces out.
                log.summary(format!("  material {index}: kind {:?} flags 0x{:X}: {} triangles not drawn", m.kind(), m.flags, tris.len()));
                not_drawn += tris.len();
                continue;
            }
        };
        // Texels -> 0..1 by the texture's size; the texture repeats
        // (buildings.md rule 3, "likely").
        let size = match &look {
            Look::Texture { width, height, .. } => [*width as f32, *height as f32],
            _ => [1.0, 1.0],
        };
        // Each corner's place in the light picture (rule 13).
        let light_uvs: Vec<[[f32; 2]; 3]> = tris
            .iter()
            .map(|t| match (&light, b.surfaces.get(t.surface)) {
                (Some(l), Some(s)) => t.texels.map(|texel| atlas.light_uv(l, t.surface, s, texel)),
                _ => [atlas.white_uv(); 3],
            })
            .collect();
        let triangles: Vec<Triangle> = tris
            .iter()
            .map(|t| {
                // Front faces are clockwise, as for models: measured on the
                // outer sides of every building's box (99.7% of ~23,000
                // surfaces) and seen by eye. This overrides buildings.md
                // rule 2 (anticlockwise); see QUESTIONS.md Q15.
                let order = [0, 1, 2];
                Triangle {
                    positions: order.map(|k| t.positions[k]),
                    uvs: order.map(|k| [t.texels[k][0] / size[0], t.texels[k][1] / size[1]]),
                    material: index as i32,
                    node: t.surface,
                    normals: [kq8_data::shape::front_normal(&t.positions); 3],
                    // Walls get point lights by their own rule (rule 16).
                    part_ball: [0.0; 4],
                }
            })
            .collect();
        let what = match &look {
            Look::Texture { name, width, height, .. } => format!("texture {name} {width}x{height}"),
            Look::Color(c) => format!("colour {c:?}"),
            Look::Missing(why) => format!("MISSING ({why}), drawn magenta"),
        };
        log.summary(format!(
            "  material {index}: flags={} triangles={} -> {what}",
            m.map_or("-".into(), |m| format!("0x{:X}", m.flags)),
            triangles.len()
        ));
        // Lit by the building's light maps, not by the sun (M1 5c).
        groups.push(Group { material: index as i32, triangles, look, shading: Shading::PLAIN, light_uvs });
    }
    if not_drawn > 0 {
        log.summary(format!("  {not_drawn} triangles with kind-0 materials not drawn"));
    }
    // Point lights on its surfaces (M1 7d, rule 16).
    let wall_light = light.map(|l| {
        let (data, agree) = crate::wall_lights::WallData::new(l, &b);
        log.summary(format!(
            "  point lights (rule 16): {} of {} surfaces can be lit (a map and a P0/U/V rectangle); facing by bit 0x80 agrees with the corner order on {agree}",
            data.lightable.len(),
            b.surfaces.len()
        ));
        std::sync::Arc::new(data)
    });
    Ok(Model {
        label,
        groups,
        min: b.bounds_min,
        max: b.bounds_max,
        frame_note: "building".into(),
        light_atlas: Some(atlas),
        surfaces: (0..b.surfaces.len()).map(|i| b.surface_corners(i).map(|c| c.into_iter().map(|(p, _)| p).collect()).unwrap_or_default()).collect(),
        lights: Vec::new(),
        wall_light,
        ball: None,
    })
}

/// A building's light maps: `<shape>.dil` from the world's archives
/// (object-lighting.md rule 12; no world switches the alternate maps on
/// at the start). `None`, logged, if it is missing, unreadable, or does not
/// match the `.dig` (build id, surface count): the building is then drawn
/// full bright (note: edge cases).
fn building_light_maps(
    world: &World,
    dig: &str,
    b: &kq8_data::building::Building,
    log: &mut Log,
) -> Option<kq8_data::dil::BuildingLight> {
    use kq8_data::lightmap::Channels;
    let base = dig.rsplit(['/', '\\']).next().unwrap_or(dig);
    let name = format!("{}.dil", base.rsplit_once('.').map_or(base, |(stem, _)| stem));
    let (label, bytes) = match world.load(&name) {
        None => {
            log.summary(format!("  light: {name} not in {}; drawn full bright", world.name));
            return None;
        }
        Some(Err(e)) => {
            log.summary(format!("  light: {name}: {e}; drawn full bright"));
            return None;
        }
        Some(Ok(x)) => x,
    };
    let l = match kq8_data::dil::BuildingLight::parse(&bytes) {
        Ok(l) => l,
        Err(e) => {
            log.summary(format!("  light: {label}: {e}; drawn full bright"));
            return None;
        }
    };
    if l.build_id != b.build_id || l.surfaces.len() != b.surfaces.len() {
        log.summary(format!(
            "  light: {label} doesn't match the .dig (build id {} vs {}, surfaces {} vs {}); drawn full bright",
            l.build_id,
            b.build_id,
            l.surfaces.len(),
            b.surfaces.len()
        ));
        return None;
    }
    let mut red = [0usize; 16];
    l.values().for_each(|v| red[Channels::of(v).red as usize] += 1);
    let mut order: Vec<usize> = (0..16).collect();
    order.sort_by_key(|&v| std::cmp::Reverse(red[v]));
    let top: Vec<String> = order.iter().take(3).map(|&v| format!("{v} x{}", red[v])).collect();
    let misses = (0..l.surfaces.len()).filter(|&i| !l.fits(i, &b.surfaces[i])).count();
    log.summary(format!(
        "  light: {label}, {} texels, red most common {}; surfaces failing the size rule: {misses}",
        red.iter().sum::<usize>(),
        top.join(", ")
    ));
    Some(l)
}
