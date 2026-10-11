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
    Ok(pal.engine_colors())
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
    let edge = histogram.map(|h| h[..8].iter().chain(&h[248..]).sum::<u32>());
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
        "  texture {label}: {:?} {}x{} {}-bit mipmaps={} flags=0x{:X} pixels_using_system_colours={}{own_vs_world}",
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

/// Reads every `.kq` section with a position from the world's archives
/// and loads the models they name (rules: docs/behaviour/objects.md).
/// `terrain` is only used for the log: how far each placed model's lowest
/// point is from the ground under it.
pub fn load_objects(game: &GameDir, world: &World, terrain: &kq8_data::Terrain, log: &mut Log) -> Objects {
    use kq8_data::kq_text;
    use kq8_data::objects::Placement;

    let palette_name = world_palette_name(world, "objectPalette").unwrap_or_else(|| {
        let fallback = format!("{}.ppl", world.name);
        log.summary(format!("objects: world.cs names no KQWorld::objectPalette; using {fallback}"));
        fallback
    });
    log.summary(format!("objects: palette {palette_name} (KQWorld::objectPalette)"));

    let mut shapes: Vec<(String, Model)> = Vec::new();
    let mut index_of: BTreeMap<String, Option<usize>> = BTreeMap::new();
    let mut placed = Vec::new();
    let mut markers = Vec::new();
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    let mut flag_values: BTreeMap<String, usize> = BTreeMap::new();
    let mut hidden_values: BTreeMap<String, usize> = BTreeMap::new();
    let (mut sections, mut raised, mut fell_back, mut on_ground_marker) = (0, 0, 0, 0);
    // Buildings use KQWorld::interiorPalette (buildings.md); loaded on first use.
    let mut interior_palette: Option<Result<[Rgb; 256], String>> = None;
    // Only what the world's start-up scripts load is in the world
    // (objects.md "Where a world's objects come from", Q16).
    let script = kq8_data::world_script::run("newWorld.cs", &|name: &str| {
        let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
        world.load(base).and_then(Result::ok).map(|(_, b)| b)
    });
    log.summary(format!(
        "world scripts: ran {:?}; missing scripts {:?}; {} loadKQ lines ({} inside if-blocks); unreadable lines {}; other commands (not run): {:?}",
        script.scripts,
        script.missing_scripts,
        script.loads.len(),
        script.loads.iter().filter(|l| l.inside_if).count(),
        script.bad_lines.len(),
        script.other_commands
    ));
    for b in &script.bad_lines {
        log.summary(format!("  unreadable: {b}"));
    }
    let mut several_positions = 0;
    // Unused objects: `.kq` files that no start-up script loads. Not in
    // the game's world at the start; drawn only with the H key so they can
    // be looked at (viewer only, not a game rule). Same rules as loaded
    // objects otherwise, with the file's own position and angles.
    let loaded_files: std::collections::BTreeSet<String> = script
        .loads
        .iter()
        .map(|l| l.file.rsplit(['/', '\\']).next().unwrap_or(&l.file).to_ascii_lowercase())
        .collect();
    let all_scripts: Vec<(String, String)> = world
        .entries_ending(".cs")
        .into_iter()
        .filter_map(|(n, _, b)| b.ok().map(|b| (n, b.iter().map(|&c| (c as char).to_ascii_lowercase()).collect())))
        .collect();
    let mut work: Vec<(kq8_data::world_script::LoadKq, Option<(String, Vec<u8>)>, bool)> =
        script.loads.iter().map(|l| (l.clone(), None, false)).collect();
    for (name, label, bytes) in world.entries_ending(".kq") {
        if loaded_files.contains(&name.to_ascii_lowercase()) {
            continue;
        }
        let Ok(bytes) = bytes else { continue };
        let load = kq8_data::world_script::LoadKq {
            file: name.clone(),
            name: kq8_data::world_script::LoadName::Keep,
            position: None,
            angles: None,
            from: "(no start-up script)".into(),
            line: 0,
            inside_if: false,
        };
        work.push((load, Some((label, bytes)), true));
    }
    let mut unused_skipped: BTreeMap<String, usize> = BTreeMap::new();
    let mut unused_placed = 0;
    for (load, preloaded, unused) in &work {
        let unused = *unused;
        let skipped = if unused { &mut unused_skipped } else { &mut skipped };
        let base = load.file.rsplit(['/', '\\']).next().unwrap_or(&load.file);
        let (label, bytes) = match preloaded.clone().map(Ok).or_else(|| world.load(base)) {
            Some(Ok(x)) => x,
            _ => {
                log.detail(format!("skip {} line {}: {} not found", load.from, load.line, load.file));
                *skipped.entry("loaded file not found".into()).or_default() += 1;
                continue;
            }
        };
        let file = kq_text::parse(&bytes);
        if file.iter().filter(|x| x.get("locX").is_some()).count() > 1 {
            several_positions += 1;
            log.detail(format!("note {label}: several sections have a position; the first section is the object"));
        }
        // The object is the file's first section; the other sections
        // (animation lists, movements) belong to it.
        if let Some(s) = file.first() {
            let Some(mut p) = Placement::from_load(s, load) else {
                log.detail(format!("skip {label} ({} line {}): no position in the line or the file", load.from, load.line));
                *skipped.entry("no position".into()).or_default() += 1;
                continue;
            };
            if unused {
                // Templates for script copies sit at (0, 0, 0): nothing to see there.
                if p.position == [0.0; 3] {
                    *skipped.entry("at (0, 0, 0) (template)".into()).or_default() += 1;
                    continue;
                }
                let base_lc = base.to_ascii_lowercase();
                let named_in: Vec<&str> =
                    all_scripts.iter().filter(|(_, t)| t.contains(&base_lc)).map(|(n, _)| n.as_str()).collect();
                log.detail(format!(
                    "unused object {label}[{}] at ({:.0}, {:.0}, {:.0}); file named in scripts: {}",
                    p.name,
                    p.position[0],
                    p.position[1],
                    p.position[2],
                    if named_in.is_empty() { "none".to_string() } else { named_in.join(", ") }
                ));
            } else {
                sections += 1;
            }
            if let Some(h) = s.get("hidden") {
                *hidden_values.entry(format!("{h} ({})", if p.shape.is_some() || s.get("animLoops").is_some() { "with a model" } else { "no model" })).or_default() += 1;
                log.detail(format!("[{}] hidden={h}", p.name));
            }
            if load.inside_if {
                log.summary(format!("  NOTE {label}: loaded inside an if-block ({} line {}); placed anyway", load.from, load.line));
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
                        log.detail(format!("skip {label}[{}]: animation lookup failed: {e}", p.name));
                        *skipped.entry("animation lookup failed".into()).or_default() += 1;
                        continue;
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
                log.detail(format!("skip {label}[{}]: {r}", p.name));
                if p.shape.is_none() && !unused {
                    let ground = terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
                    markers.push(Marker {
                        name: p.name.clone(),
                        class: p.class.clone().unwrap_or_else(|| "?".into()),
                        position: p.grounded_position(ground),
                        anm: s.get("animFile").is_some(),
                    });
                }
                *skipped.entry(r).or_default() += 1;
                continue;
            }
            let shape = p.shape.clone().unwrap();
            // One model per (file, start frame) combination.
            let key = match &animation {
                Some(c) => format!("{}|{}|{}|{}", shape.to_ascii_lowercase(), c.sequence.to_ascii_lowercase(), c.start, c.repeat),
                None => shape.to_ascii_lowercase(),
            };
            let start = animation.as_ref().map(|c| (c.sequence.as_str(), c.start, c.repeat));
            let material_list = s.get("material").map(str::to_string);
            let key = match &material_list {
                Some(m) => format!("{key}|{}", m.to_ascii_lowercase()),
                None => key,
            };
            let slot = *index_of.entry(key).or_insert_with(|| {
                // Per-model lines go to the file only.
                log.quiet = true;
                let r = if shape.to_ascii_lowercase().ends_with(".dig") {
                    let palette = interior_palette.get_or_insert_with(|| {
                        let name = world_palette_name(world, "interiorPalette").unwrap_or_else(|| format!("{}.ppl", world.name));
                        log.summary(format!("buildings: palette {name} (KQWorld::interiorPalette)"));
                        load_palette(world, Some(&name), game, log)
                    });
                    match palette {
                        Ok(pal) => load_building(game, world, &shape, material_list.as_deref().unwrap(), pal, log),
                        Err(e) => Err(e.clone()),
                    }
                } else {
                    load_model(game, world, &shape, Some(&palette_name), start, log)
                };
                log.quiet = false;
                match r {
                    Ok(m) => {
                        shapes.push((shape.clone(), m));
                        Some(shapes.len() - 1)
                    }
                    Err(e) => {
                        log.detail(format!("model {shape} failed: {e}"));
                        None
                    }
                }
            });
            let Some(i) = slot else {
                log.detail(format!("skip {label}[{}]: model {shape} could not be loaded", p.name));
                *skipped.entry("model could not be loaded".into()).or_default() += 1;
                continue;
            };
            let class = p.class.clone().unwrap_or("?".into());
            if let Some(c) = &animation {
                log.detail(format!(
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
                    shapes[i].1.frame_note,
                    p.flags.as_ref().map_or(String::new(), |f| format!(" flags={f} (random start not applied)"))
                ));
                if c.fell_back && !unused {
                    fell_back += 1;
                }
            }
            if p.flags.is_some() && !unused {
                *flag_values.entry(p.flags.clone().unwrap()).or_default() += 1;
            }
            // Ground rules: loadKQ z of -1/-2 = on the ground (objects.md
            // Q17); otherwise keepOnGround raises, never lowers (Q10).
            let ground = terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
            let grounded = p.grounded_position(ground);
            if p.z_on_ground {
                log.detail(format!(
                    "put [{}] on the ground (loadKQ z={:.0}, keepOnGround={}): z={}",
                    p.name,
                    p.position[2],
                    p.keep_on_ground,
                    ground.map_or("none (outside the grid, kept as written)".into(), |g| format!("{g:.0}"))
                ));
                on_ground_marker += usize::from(!unused);
                p.position = grounded;
            } else if grounded != p.position {
                log.detail(format!("raised [{}] from z={:.0} to the ground at z={:.0}", p.name, p.position[2], grounded[2]));
                raised += usize::from(!unused);
                p.position = grounded;
            }
            if unused {
                unused_placed += 1;
            } else {
                *by_class.entry(if animation.is_some() { format!("{class} (animated)") } else { class }).or_default() += 1;
            }
            placed.push((p, i, unused));
        }
    }

    // Check from the numbers: lowest point of each placed model vs ground.
    let mut gaps = Vec::new();
    for (p, i, _) in placed.iter().filter(|x| !x.2) {
        let t = p.transform();
        let lowest = shapes[*i]
            .1
            .groups
            .iter()
            .flat_map(|g| g.triangles.iter().flat_map(|tri| tri.positions))
            .map(|v| t.apply(v)[2])
            .fold(f32::INFINITY, f32::min);
        let ground = terrain.height_at(p.position[0], p.position[1]).map(|g| g.height);
        let gap = ground.map(|h| lowest - h);
        log.detail(format!(
            "object [{}] class={} shape={} at ({:.0}, {:.0}, {:.0}) angles=({:.2}, {:.2}, {:.2}) lowest_point={lowest:.0} ground={} gap={}",
            p.name,
            p.class.as_deref().unwrap_or("?"),
            shapes[*i].0,
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
        "objects: {sections} loaded objects with a position, {} placed using {} different models; skipped: {skipped:?}",
        placed.len() - unused_placed,
        shapes.len()
    ));
    log.summary(format!(
        "  unused objects (in .kq files no start-up script loads; shown with key H): {unused_placed} placed; skipped: {unused_skipped:?}"
    ));
    log.summary(format!("  placed per class: {by_class:?}"));
    log.summary(format!("  'hidden' values (not used yet): {hidden_values:?}"));
    log.summary(format!("  files where several sections have a position (first used): {several_positions}"));
    log.summary(format!(
        "  put on the ground by a loadKQ z of -1/-2: {on_ground_marker}; raised onto the ground (keepOnGround): {raised}; animated objects without a usable currAnimName (movement 0 used): {fell_back}; 'flags' values: {flag_values:?}"
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
        markers.iter().filter(|m| !m.anm).count(),
        markers.iter().filter(|m| m.anm).count()
    ));
    for m in &markers {
        log.detail(format!(
            "marker [{}] class={} at ({:.0}, {:.0}, {:.0}){}",
            m.name, m.class, m.position[0], m.position[1], m.position[2], if m.anm { " (.anm character)" } else { "" }
        ));
    }
    Objects { shapes, placed, markers }
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
