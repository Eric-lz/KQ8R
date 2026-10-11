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

    /// Every archive entry whose name ends with `ext` (any case), unpacked,
    /// as (label, bytes). Entries that fail to unpack are returned as errors.
    pub fn entries_ending(&self, ext: &str) -> Vec<(String, Result<Vec<u8>, String>)> {
        let ext = ext.to_ascii_lowercase();
        self.vols
            .iter()
            .flat_map(|(v, a)| {
                a.entries().iter().filter(|e| e.name.to_ascii_lowercase().ends_with(&ext)).map(move |e| {
                    let label = format!("vol:{}/{v}/{}", self.name, e.name);
                    (label, a.unpack(e).map(|u| u.data).map_err(|e| e.to_string()))
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
}

pub struct Group {
    pub material: i32,
    pub triangles: Vec<Triangle>,
    pub look: Look,
}

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
    log: &mut Log,
) -> Result<Model, String> {
    let (label, bytes) = find_model(game, world, model)?;
    log.summary(format!("model: {label} ({} bytes)", bytes.len()));
    let shape = Shape::parse(&bytes).map_err(|e| format!("{label}: {e}"))?;
    let rest = shape.rest_sequence();
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
    let (min, max) = bounds(&geometry.triangles).ok_or(format!("{label}: rest sequence {rest} has no triangles"))?;
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
    Ok(Model { label, groups, min, max })
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
    /// A placement and the index of its model in `shapes`.
    pub placed: Vec<(kq8_data::objects::Placement, usize)>,
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
    let mut skipped: BTreeMap<String, usize> = BTreeMap::new();
    let mut by_class: BTreeMap<String, usize> = BTreeMap::new();
    let mut sections = 0;
    for (label, bytes) in world.entries_ending(".kq") {
        let Ok(bytes) = bytes else {
            *skipped.entry("file failed to unpack".into()).or_default() += 1;
            continue;
        };
        let file = kq_text::parse(&bytes);
        for s in &file {
            let Some(mut p) = Placement::from_section(s) else { continue };
            sections += 1;
            // Animated objects name their model through their current
            // animation; shown in its first frame (which frame the game
            // shows is open, QUESTIONS.md Q12).
            let mut animated = false;
            if p.shape.is_none() {
                match kq8_data::objects::current_loop_shape(&file, s) {
                    Ok(Some(sh)) => {
                        p.shape = Some(sh);
                        animated = true;
                    }
                    Ok(None) => {}
                    Err(e) => {
                        log.detail(format!("skip {label}[{}]: animation lookup failed: {e}", p.name));
                        *skipped.entry("animation lookup failed".into()).or_default() += 1;
                        continue;
                    }
                }
            }
            let reason = if p.position == [0.0; 3] {
                Some("position (0, 0, 0), probably a template".to_string())
            } else if p.has_matrix_key {
                Some("uses an RMat key (not supported)".to_string())
            } else {
                match &p.shape {
                    None => Some(format!(
                        "no model ({})",
                        if s.get("animFile").is_some() { "uses an .anm file" } else { "no shapeName or animLoops" }
                    )),
                    Some(sh) if sh.to_ascii_lowercase().ends_with(".dig") => Some("building (.dig, step 3b)".to_string()),
                    Some(sh) if !sh.to_ascii_lowercase().ends_with(".kqs") => {
                        Some(format!("shapeName is not a .kqs ({})", sh.rsplit('.').next().unwrap_or("")))
                    }
                    Some(_) => None,
                }
            };
            if let Some(r) = reason {
                log.detail(format!("skip {label}[{}]: {r}", p.name));
                *skipped.entry(r).or_default() += 1;
                continue;
            }
            let shape = p.shape.clone().unwrap();
            let key = shape.to_ascii_lowercase();
            let slot = *index_of.entry(key).or_insert_with(|| {
                // Per-model lines go to the file only.
                log.quiet = true;
                let r = load_model(game, world, &shape, Some(&palette_name), log);
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
            *by_class.entry(if animated { format!("{class} (animated, first frame)") } else { class }).or_default() += 1;
            placed.push((p, i));
        }
    }

    // Check from the numbers: lowest point of each placed model vs ground.
    let mut gaps = Vec::new();
    for (p, i) in &placed {
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
        "objects: {sections} sections with a position, {} placed using {} different models; skipped: {skipped:?}",
        placed.len(),
        shapes.len()
    ));
    log.summary(format!("  placed per class: {by_class:?}"));
    log.summary(format!(
        "  lowest point minus ground: within 50 units={} 50..500 above={} more than 500 above={} 50..500 below={} more than 500 below={}",
        count(&|g| g.abs() <= 50.0),
        count(&|g| g > 50.0 && g <= 500.0),
        count(&|g| g > 500.0),
        count(&|g| (-500.0..-50.0).contains(&g)),
        count(&|g| g < -500.0)
    ));
    Objects { shapes, placed }
}
