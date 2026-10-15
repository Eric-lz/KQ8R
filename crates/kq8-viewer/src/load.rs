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
}

pub struct Group {
    pub material: i32,
    pub triangles: Vec<Triangle>,
    pub look: Look,
    /// How the material is lit (object-lighting.md rule 7).
    pub lighting: Lighting,
}

/// A model material's lighting kind (object-lighting.md rule 7). Shading
/// bits the note doesn't cover, or no material: full brightness
/// (STAND-IN; no model material in the data has other bits).
pub fn material_lighting(m: Option<&kq8_data::Material>) -> Lighting {
    m.and_then(|m| m.lighting()).unwrap_or(Lighting::None)
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
        let lighting = material_lighting(m);
        log.summary(format!(
            "  material {index}: flags={} triangles={} lighting={lighting:?} -> {what}",
            m.map_or("-".into(), |m| format!("0x{:X}", m.flags)),
            triangles.len()
        ));
        groups.push(Group { material: index, triangles, look, lighting });
    }
    Ok(Model { label, groups, min, max, frame_note })
}

/// The movements M2 uses: step 2b (connor-movement.md "Which movement
/// plays"), the falls of step 2c/2e (connor-physics.md "Falling off an
/// edge"), the crash (2d-4), the jumps (2e), the start and stop transitions (2g,
/// connor-movement.md "Starting and stopping") and wading (2h, water.md).
/// Transitions to or via a movement not listed here are skipped.
pub const CONNOR_MOVEMENTS: [&str; 34] = [
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
}

/// Every frame of one animation loop, for drawing it in turn (M2 step 2b).
pub struct Animation {
    /// One look and lighting kind per material index used by any frame.
    pub looks: Vec<(i32, Look, Lighting)>,
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
                    MaterialKind::Texture => load_texture(world, game, &m.texture, palette, log),
                    MaterialKind::Rgb => Look::Color(m.rgb),
                    k => Look::Missing(format!("material kind {k:?}")),
                },
            };
            (index, look, material_lighting(material))
        })
        .collect();
    log.detail(format!(
        "animation {label} loop '{loop_name}': {count} frames, {:.4} s, steps add up to ({:.1}, {:.1}, {:.1}){}",
        l.duration,
        step_sum[0],
        step_sum[1],
        step_sum[2],
        if turning > 0 { format!("; {turning} frame steps also turn (not applied)") } else { String::new() }
    ));
    Ok(Animation { looks, frames, duration: l.duration, step_sum, frame_steps, frame_transforms })
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
    let light = load_light_map(game, world, &settings, &fetch, log);
    Ok(Ground { terrain, groups, materials, orientation, textures, palette, animated: Default::default(), light })
}

/// A file found by name: (where, bytes).
type Fetched = Result<(String, Vec<u8>), String>;

/// The ground's light map (terrain-lighting.md rules 1-6): the hardware
/// pair (we draw like the Glide/Direct3D renderers), the shadow map if
/// `Options.cs` says `useDynamicLighting 1`, else the light map. No name,
/// or a file that can't be used: full brightness (rule 4).
fn load_light_map(
    game: &GameDir,
    world: &World,
    settings: &kq8_data::TerrainSettings,
    fetch: &dyn Fn(&str) -> Fetched,
    log: &mut Log,
) -> kq8_data::lightmap::LightMap {
    use kq8_data::lightmap::{self, Kind, LightMap};
    let dynamic = game.find("Options.cs").and_then(|p| std::fs::read(p).ok()).and_then(|b| lightmap::use_dynamic_lighting(&b));
    let kind = Kind::for_dynamic_lighting(dynamic.unwrap_or(false));
    let why = match dynamic {
        Some(on) => format!("Options.cs useDynamicLighting {}", u8::from(on)),
        None => "no useDynamicLighting in Options.cs, the game's start value 0".into(),
    };
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

/// Daventry's (or any world's) placed objects: each different model once,
/// plus where each copy goes.
pub struct Objects {
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
    /// The emitters the start-up scripts load (M3 step 3b-1).
    pub emitters: Vec<EmitterAssets>,
    /// The particle catchers (M3 step 3b-2): name and settings.
    pub catchers: Vec<(String, kq8_data::effects::CatcherSettings)>,
    /// The traps (M3 step 3d-2), where the start-up scripts left them.
    pub traps: Vec<kq8_core::trap::TrapSettings>,
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
    /// Lines about items at load, for `items.log`.
    pub item_lines: Vec<String>,
    /// The ground, for z -1 / -2 of items made while playing.
    pub ground: Option<kq8_data::Terrain>,
}

/// A placed object whose movement plays (M3 step 3c; objects.md "Animated
/// objects at load"). It stays in `Objects::placed` (collision, logs) but
/// is drawn by `anim_view.rs`, not as a still model.
pub struct AnimatedAssets {
    /// Index into `Objects::placed`.
    pub placed: usize,
    /// Index into `Objects::object_animations`.
    pub animation: usize,
    /// `loop=` start, speed, repeat.
    pub start: f32,
    pub speed: f32,
    pub repeat: bool,
    /// `flags` bit 8: random start and speed.
    pub random: bool,
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
pub fn load_objects(game: &GameDir, world: &World, terrain: &kq8_data::Terrain, log: &mut Log) -> Objects {
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
    let items = item_types(game, log);
    let mut placer = Placer {
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
        doors: Vec::new(),
        door_animations: Vec::new(),
        door_animation_index: BTreeMap::new(),
        emitters: Vec::new(),
        catchers: Vec::new(),
        traps: Vec::new(),
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
        item_lines: Vec::new(),
    };

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
        if loaded_files.contains(&name.to_ascii_lowercase()) {
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
    game: &'a GameDir,
    world: &'a World,
    terrain: &'a kq8_data::Terrain,
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
    doors: Vec<DoorAssets>,
    door_animations: Vec<Animation>,
    door_animation_index: BTreeMap<String, usize>,
    emitters: Vec<EmitterAssets>,
    catchers: Vec<(String, kq8_data::effects::CatcherSettings)>,
    traps: Vec<kq8_core::trap::TrapSettings>,
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
    /// Lines about items, for `items.log`.
    item_lines: Vec<String>,
}

impl kq8_data::world_script::Host for Placer<'_> {
    fn read(&self, name: &str) -> Option<Vec<u8>> {
        let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
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

impl Placer<'_> {
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
            let mut assets =
                ConnorAssets { movements: Vec::new(), animations: Vec::new(), keys, camera_keys, collision_box };
            let mut index_of: BTreeMap<String, usize> = BTreeMap::new();
            for name in CONNOR_MOVEMENTS {
                let mut with_ask = anm.clone();
                with_ask.extend(kq_text::parse(format!("[ConnorAsks]\nanimLoops={list}\ncurrAnimName={name}\n").as_bytes()));
                let c = kq8_data::objects::current_loop(&with_ask, with_ask.last().unwrap())?.ok_or("no animation list")?;
                if c.fell_back {
                    return Err(format!("Conner.anm has no movement called {name}"));
                }
                let key = format!("{}|{}", c.shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase());
                let animation = match index_of.get(&key) {
                    Some(&i) => i,
                    None => {
                        self.log.quiet = true;
                        let a = load_animation(self.game, self.world, &c.shape, &c.sequence, &palette, self.log);
                        self.log.quiet = false;
                        assets.animations.push(a?);
                        index_of.insert(key, assets.animations.len() - 1);
                        assets.animations.len() - 1
                    }
                };
                let a = &assets.animations[animation];
                let speed_now = if a.duration > 0.0 {
                    (a.step_sum[0].powi(2) + a.step_sum[1].powi(2)).sqrt() / a.duration * c.speed
                } else {
                    0.0
                };
                self.log.summary(format!(
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
                            self.log.summary(format!("connor: {name}: transition {line}"));
                            transitions.push(kq8_core::connor::Transition { from: t.from, to, to_phase: t.to_phase, via });
                        }
                        _ => self.log.summary(format!("connor: {name}: transition {line} skipped (movement not loaded)")),
                    }
                }
                for p in &c.transition_problems {
                    self.log.summary(format!("connor: {name}: {p}"));
                }
                if !c.repeat || !c.transitions.is_empty() {
                    let steps: Vec<String> =
                        a.frame_steps.iter().map(|p| format!("({:.2}, {:.2}, {:.2})", p[0], p[1], p[2])).collect();
                    self.log.summary(format!("connor: {name}: can be cut short {}; frame steps {}", c.unknown != 0, steps.join(" ")));
                }
                assets.movements.push(ConnorMovement {
                    name,
                    animation,
                    speed: c.speed,
                    repeat: c.repeat,
                    cut_short: c.unknown != 0,
                    transitions,
                });
            }
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
        let (label, bytes) = match preloaded.map(Ok).or_else(|| self.world.load(base)) {
            Some(Ok(x)) => x,
            _ => {
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
        if file.iter().filter(|x| x.get("locX").is_some()).count() > 1 {
            self.several_positions += 1;
            self.log.detail(format!("note {label}: several sections have a position; the first section is the object"));
        }
        // The object is the file's first section; the other sections
        // (animation lists, movements) belong to it.
        let Some(s) = file.first() else { return };
        let mut p = Placement::from_load(s, load);
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
            // Templates for script copies sit at (0, 0, 0): nothing to see there.
            if p.position == [0.0; 3] {
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
        if !unused {
            self.located.insert(key.clone(), Located { position: p.position, angles: p.angles, drawn: Drawn::Nothing });
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
            too_far_away_action: number("tooFarAwayAction").map_or(0, |v| v as i64),
            do_not_point_at: number("doNotPointAt").is_some_and(|v| v != 0.0),
        });
        let flags = p.flags.clone();
        self.placed.push((p, i, unused));
        // A movement that plays (speed not 0): all its frames (3c). Not a
        // `KQBoulder`: it controls its own animation (shown still in the
        // original), not studied yet (objects.md Q51 rule 8, step 3r-2).
        if let Some(c) = animation.as_ref().filter(|c| c.speed != 0.0) {
            if boulder {
                self.log.summary(format!(
                    "animated {label}[{}]: class KQBoulder: movement {} not played, drawn still at its start (objects.md Q51 rule 8)",
                    self.placed.last().map_or("?", |(p, _, _)| p.name.as_str()),
                    c.movement
                ));
            } else {
                self.place_animated(&label, c, flags.as_deref());
            }
        }
        if let (false, Some(l)) = (unused, self.located.get_mut(&key)) {
            l.drawn = Drawn::Model(self.placed.len() - 1);
        }
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
        let door = DoorAssets {
            movements: with_index,
            current,
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
            shape,
            ground: [kq8_data::terrain::Level::Floor, kq8_data::terrain::Level::Surface]
                .map(|level| self.terrain.ground_at(level, p.position[0], p.position[1]).map(|g| g.height)),
        });
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
    fn place_animated(&mut self, label: &str, c: &kq8_data::objects::CurrentLoop, flags: Option<&str>) {
        let key = format!("{}|{}", c.shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase());
        let index = match self.object_animation_index.get(&key) {
            Some(i) => *i,
            None => {
                let loaded = self.object_palette().and_then(|palette| {
                    self.log.quiet = true;
                    let a = load_animation(self.game, self.world, &c.shape, &c.sequence, &palette, self.log);
                    self.log.quiet = false;
                    a
                });
                let i = match loaded {
                    Ok(a) => {
                        self.object_animations.push(a);
                        Some(self.object_animations.len() - 1)
                    }
                    Err(e) => {
                        self.log.summary(format!("animated {label}: {} {}: {e}; drawn still", c.shape, c.sequence));
                        None
                    }
                };
                self.object_animation_index.insert(key, i);
                i
            }
        };
        let Some(animation) = index else { return };
        // `flags` bit 8 (objects.md rule 4); a value that is not a whole
        // number counts as no flags.
        let flag_bits = flags.and_then(|f| f.trim().parse::<i64>().ok()).unwrap_or(0) as u32;
        self.animated.push(AnimatedAssets {
            placed: self.placed.len() - 1,
            animation,
            start: c.start,
            speed: c.speed,
            repeat: c.repeat,
            random: flag_bits & kq8_core::anim::RANDOM_START != 0,
        });
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
            buildings.push(c);
            building_types.push(self.collision[k].collider_type);
        }
        log.summary(format!("collision: {} building colliders", buildings.len()));
        let (objects, clickables, view_boxes) = object_colliders(self.game, &self.shapes, &self.placed, &self.collision, &self.markers, log);
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
        let mut known: Vec<String> = self.located.keys().cloned().collect();
        known.extend(self.markers.iter().map(|m| m.name.to_ascii_lowercase()));
        Objects {
            shapes: self.shapes,
            placed: self.placed,
            markers: self.markers,
            connor: self.connor,
            camera_start,
            buildings,
            building_types,
            view_boxes,
            objects,
            doors,
            door_animations: self.door_animations,
            clickables,
            known,
            vars: Default::default(),
            scripts: Vec::new(),
            attack_dist: self.attack_dist,
            emitters,
            catchers: self.catchers,
            traps,
            animators: self.animators,
            animated: self.animated,
            object_animations: self.object_animations,
            items: Default::default(),
            item_shapes: BTreeMap::new(),
            item_lines: Vec::new(),
            ground: None,
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
    use kq8_core::collide::{pair_collides, BoxCollider, Placement, CONNOR_MASK, CONNOR_TYPE};
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
        if class == "KQMonster" {
            *left_out.entry("KQMonster (ball stack, not traced against Connor)").or_default() += 1;
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
            });
        }
        if info.collider_type & kq8_core::click::CLICKABLE != 0 {
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
        objects.push(BoxCollider { name: p.name.clone(), min: entry.min, max: entry.max, placement });
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
    let mut by_material: BTreeMap<u8, Vec<kq8_data::building::BuildingTriangle>> = BTreeMap::new();
    for t in tris {
        by_material.entry(t.material).or_default().push(t);
    }
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
        // Buildings are lit by their `.dil` light maps (M1 5c, not done):
        // full brightness until then.
        groups.push(Group { material: index as i32, triangles, look, lighting: Lighting::None });
    }
    if not_drawn > 0 {
        log.summary(format!("  {not_drawn} triangles with kind-0 materials not drawn"));
    }
    Ok(Model {
        label,
        groups,
        min: b.bounds_min,
        max: b.bounds_max,
        frame_note: "building".into(),
    })
}
