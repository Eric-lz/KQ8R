//! Finding a model, its textures and the palette in the game files, and
//! turning them into plain triangle lists and RGBA images. No Bevy here.

use std::collections::BTreeMap;
use std::path::Path;

use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::palette::Rgb;
use kq8_data::shape::{bounds, Triangle};
use kq8_data::{Bitmap, GameDir, MaterialKind, Palette, Shape, VolArchive};

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
        let flags = usize::try_from(index).ok().and_then(|i| list.get(i)).map(|m| m.flags);
        log.summary(format!(
            "  material {index}: flags={} triangles={} -> {what}",
            flags.map_or("-".into(), |f| format!("0x{f:X}")),
            triangles.len()
        ));
        groups.push(Group { material: index, triangles, look });
    }
    Ok(Model { label, groups, min, max, frame_note })
}

/// The movements M2 step 2b uses (connor-movement.md "Which movement plays").
pub const CONNOR_MOVEMENTS: [&str; 6] = ["stop", "walk", "run", "back", "left", "right"];

/// One of Connor's movements: its animation and playback speed.
pub struct ConnorMovement {
    pub name: &'static str,
    /// Index into `ConnorAssets::animations`.
    pub animation: usize,
    pub speed: f32,
}

pub struct ConnorAssets {
    pub movements: Vec<ConnorMovement>,
    pub animations: Vec<Animation>,
    /// `MASK/Connor.sae`.
    pub keys: Vec<kq8_data::action_map::Binding>,
    /// `MASK/Camera.sae`.
    pub camera_keys: Vec<kq8_data::action_map::Binding>,
}

/// Every frame of one animation loop, for drawing it in turn (M2 step 2b).
pub struct Animation {
    /// One look per material index used by any frame.
    pub looks: Vec<(i32, Look)>,
    /// Per frame: triangles grouped by material index (model space).
    pub frames: Vec<Vec<(i32, Vec<Triangle>)>>,
    /// Loop length in seconds.
    pub duration: f32,
    /// The frames' whole-body steps added up (model space).
    pub step_sum: [f32; 3],
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
    let l = shape
        .loops
        .iter()
        .find(|l| usize::try_from(l.name).ok().and_then(|n| shape.names.get(n)).is_some_and(|n| n.eq_ignore_ascii_case(loop_name)))
        .ok_or(format!("{label}: no loop called '{loop_name}'"))?;
    let (first, count) = (l.first_sequence.max(0) as usize, l.sequence_count.max(0) as usize);
    let mut frames = Vec::with_capacity(count);
    let mut step_sum = [0.0f32; 3];
    let mut turning = 0;
    for seq in first..first + count {
        let s = shape.sequences.get(seq).ok_or(format!("{label}: loop '{loop_name}' runs past the sequences"))?;
        if s.transform.rotates() {
            turning += 1;
        }
        for (sum, v) in step_sum.iter_mut().zip(s.transform.p) {
            *sum += v;
        }
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
            let look = match usize::try_from(index).ok().and_then(|i| list.get(i)) {
                None => Look::Missing(format!("material index {index} not in list of {}", list.len())),
                Some(m) => match m.kind() {
                    MaterialKind::Texture => load_texture(world, game, &m.texture, palette, log),
                    MaterialKind::Rgb => Look::Color(m.rgb),
                    k => Look::Missing(format!("material kind {k:?}")),
                },
            };
            (index, look)
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
    Ok(Animation { looks, frames, duration: l.duration, step_sum })
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
    // Used exactly as stored (models-and-textures.md Q3).
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
    // 246-255; Q3), and entry 0 alone (stored blue in the world palettes).
    let edge = histogram.map(|h| h[..10].iter().chain(&h[246..]).sum::<u32>());
    let index0 = histogram.map_or(0, |h| h[0]);
    // For images with their own colour table: how many of the colours the
    // image actually uses differ from the world palette (0 = no difference).
    let own_vs_world = match (&bmp.embedded_palette, histogram) {
        (Some(own), Some(h)) => {
            let used = (0..256).filter(|&i| h[i] > 0);
            let differ = used.filter(|&i| own.get(i) != Some(&palette[i])).count();
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
                c
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
    let terrain = Terrain::new(settings.clone(), grid(&settings.height_image)?)?;
    let materials = grid(&settings.material_image)?;
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
    Ok(Ground { terrain, groups })
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
        markers: Vec::new(),
        connor: None,
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
    placer.finish()
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
    markers: Vec<Marker>,
    connor: Option<(kq8_data::objects::Placement, ConnorAssets)>,
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
        self.hide(name, hidden, at);
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
            let ground = self.terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
            p.position = p.grounded_position(ground);
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
            let mut assets =
                ConnorAssets { movements: Vec::new(), animations: Vec::new(), keys, camera_keys };
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
                    "connor: movement {name} = movement {} loop={} {} speed {}: {} frames, {:.4} s, steps ({:.1}, {:.1}, {:.1}) -> {speed_now:.1} units/s",
                    c.movement, c.shape, c.sequence, c.speed, a.frames.len(), a.duration, a.step_sum[0], a.step_sum[1], a.step_sum[2]
                ));
                assets.movements.push(ConnorMovement { name, animation, speed: c.speed });
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

        // Ground rules, when the object is loaded, so `getLoc` sees the
        // result: loadKQ z of -1/-2 = on the ground (objects.md Q17);
        // otherwise keepOnGround raises, never lowers (Q10).
        let ground = self.terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
        let grounded = p.grounded_position(ground);
        if p.z_on_ground {
            self.log.detail(format!(
                "put [{}] on the ground (loadKQ z={:.0}, keepOnGround={}): z={}",
                p.name,
                p.position[2],
                p.keep_on_ground,
                ground.map_or("none (outside the grid, kept as written)".into(), |g| format!("{g:.0}"))
            ));
            self.on_ground_marker += usize::from(!unused);
            p.position = grounded;
        } else if grounded != p.position {
            self.log.detail(format!("raised [{}] from z={:.0} to the ground at z={:.0}", p.name, p.position[2], grounded[2]));
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
                p.flags.as_ref().map_or(String::new(), |f| format!(" flags={f} (random start not applied)"))
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
        self.placed.push((p, i, unused));
        if let (false, Some(l)) = (unused, self.located.get_mut(&key)) {
            l.drawn = Drawn::Model(self.placed.len() - 1);
        }
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
            let ground = self.terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
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
        Objects { shapes: self.shapes, placed: self.placed, markers: self.markers, connor: self.connor, camera_start }
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
        groups.push(Group { material: index as i32, triangles, look });
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
