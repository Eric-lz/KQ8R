//! `kq8 shape-verify`: parses every 3D model (`.kqs`) and material list
//! (`.dml`) the game has, in all archives and in the loose files under
//! `GAME/` and `data/`, and logs counts and consistency checks so the
//! format can be judged from numbers.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::pers::read_block;
use kq8_data::shape::{bounds, Shape};
use kq8_data::{GameDir, MaterialKind, MaterialList, VolArchive};

use crate::log::Log;

#[derive(Default)]
struct Tally {
    shapes_ok: usize,
    lists_ok: usize,
    failures: usize,
    /// Files whose class is known but not supported (counted, not failures).
    unsupported: BTreeMap<String, usize>,
    /// "class vN" -> count, for shapes, meshes and material lists.
    versions: BTreeMap<String, usize>,
    unread_bytes: usize,
    trailing_bytes: usize,
    meshes: usize,
    faces: usize,
    /// Smallest and largest face vertex index seen (relative to frame).
    vertex_index_range: Option<(i32, i32)>,
    /// Face corners pointing at frame vertex 0 or 1 (the bounding box).
    corners_on_bbox_vertex: usize,
    /// Face corners at or past `vertices_per_frame`.
    corners_past_frame: usize,
    /// Frames whose vertex 0 / 1 equal the min / max of the other vertices.
    bbox_frames_match: usize,
    bbox_frames_differ: usize,
    frames_implied: usize,
    meshes_without_uvs: usize,
    nodes_without_uv_frame: usize,
    uv_range: Option<([f32; 2], [f32; 2])>,
    material_index_bad: usize,
    shapes_without_materials: usize,
    transforms: usize,
    transforms_rotating: usize,
    transform_flags: BTreeMap<u32, usize>,
    geometry_problems: usize,
    rest_triangles: usize,
    /// Axis with the largest extent in the rest pose -> count.
    tallest_axis: BTreeMap<&'static str, usize>,
    /// Axis whose minimum is (near) zero, i.e. the model stands on it.
    sits_on_zero: BTreeMap<&'static str, usize>,
    /// Shapes with 2+ rotating parts, scored for both matrix conventions.
    convention_shapes: usize,
    /// Part vertices that touch another part: (row-vector, column-vector).
    convention_touching: (usize, usize),
    /// Shapes where one convention clearly wins: (row, column, tie).
    convention_wins: (usize, usize, usize),
    loops_per_shape: BTreeMap<usize, usize>,
    lights: usize,
    material_flags: BTreeMap<u32, usize>,
    material_kinds: BTreeMap<String, usize>,
    textures_referenced: usize,
    textures_found_here: usize,
    textures_found_in_game: usize,
    textures_found_elsewhere: usize,
    /// Texture names not found anywhere -> how many materials use them.
    textures_missing: BTreeMap<String, usize>,
}

/// Lower-case names of every file the game could load, per world.
struct NameIndex {
    world: HashMap<String, BTreeSet<String>>,
    game: BTreeSet<String>,
}

impl NameIndex {
    /// Where a texture is found for a model from `world` (None = GAME/).
    fn locate(&self, world: Option<&str>, name: &str) -> Where {
        let n = name.to_ascii_lowercase();
        if let Some(w) = world {
            if self.world.get(w).is_some_and(|s| s.contains(&n)) {
                return Where::Here;
            }
        }
        if self.game.contains(&n) {
            return Where::Game;
        }
        if self.world.values().any(|s| s.contains(&n)) {
            return Where::Elsewhere;
        }
        Where::Missing
    }
}

enum Where {
    Here,
    Game,
    Elsewhere,
    Missing,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("shape-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("shape-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let mut t = Tally::default();
    let mut index = NameIndex { world: HashMap::new(), game: BTreeSet::new() };

    // Open every archive once; collect names for texture lookups.
    let mut archives = Vec::new();
    for world in WORLDS {
        let names = index.world.entry(world.to_string()).or_default();
        for volume in VOLUMES {
            let Some(path) = game.volume_path(world, volume) else { continue };
            match VolArchive::open(&path) {
                Ok(v) => {
                    names.extend(v.entries().iter().map(|e| e.name.to_ascii_lowercase()));
                    archives.push((world, volume, v));
                }
                Err(e) => {
                    log.summary(format!("FAIL {world}/{volume}.vol: {e}"));
                    t.failures += 1;
                }
            }
        }
        if let Some(dir) = game.find(&format!("data/{world}")) {
            collect_names(&dir, names);
        }
    }
    if let Some(dir) = game.find("GAME") {
        collect_names(&dir, &mut index.game);
    }

    for (world, volume, vol) in &archives {
        for entry in vol.entries() {
            if kind(&entry.name).is_none() {
                continue;
            }
            let label = format!("vol:{world}/{volume}/{}", entry.name);
            match vol.unpack(entry) {
                Ok(u) => check(&label, Some(world), &entry.name, &u.data, &index, &mut log, &mut t),
                Err(e) => {
                    log.summary(format!("FAIL {label}: {e}"));
                    t.failures += 1;
                }
            }
        }
    }
    for top in ["GAME", "data"] {
        if let Some(dir) = game.find(top) {
            walk(&dir, top, &index, &mut log, &mut t);
        }
    }

    report(&mut log, &t);
    log.summary(format!(
        "TOTAL shapes_ok={} material_lists_ok={} unsupported={} failures={} time_ms={}",
        t.shapes_ok,
        t.lists_ok,
        t.unsupported.values().sum::<usize>(),
        t.failures,
        started.elapsed().as_millis()
    ));
    let ok = t.failures == 0 && t.shapes_ok > 0;
    log.summary(if ok { "RESULT: PASS" } else { "RESULT: FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn kind(name: &str) -> Option<&'static str> {
    match name.rsplit_once('.')?.1.to_ascii_lowercase().as_str() {
        "kqs" => Some("kqs"),
        "dml" => Some("dml"),
        _ => None,
    }
}

fn collect_names(dir: &Path, out: &mut BTreeSet<String>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    for e in rd.flatten() {
        let path = e.path();
        if path.is_dir() {
            collect_names(&path, out);
        } else {
            out.insert(e.file_name().to_string_lossy().to_ascii_lowercase());
        }
    }
}

/// World a loose file belongs to (`data/<World>/...`), if any.
fn loose_world(label: &str) -> Option<&'static str> {
    let w = label.strip_prefix("data/")?.split('/').next()?;
    WORLDS.iter().copied().find(|x| x.eq_ignore_ascii_case(w))
}

fn walk(dir: &Path, label: &str, index: &NameIndex, log: &mut Log, t: &mut Tally) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        let sub = format!("{label}/{name}");
        let path = e.path();
        if path.is_dir() {
            walk(&path, &sub, index, log, t);
        } else if kind(&name).is_some() {
            match fs::read(&path) {
                Ok(bytes) => check(&sub, loose_world(&sub), &name, &bytes, index, log, t),
                Err(err) => {
                    log.summary(format!("FAIL {sub}: {err}"));
                    t.failures += 1;
                }
            }
        }
    }
}

fn check(
    label: &str,
    world: Option<&str>,
    name: &str,
    bytes: &[u8],
    index: &NameIndex,
    log: &mut Log,
    t: &mut Tally,
) {
    // Look at the class first so known-but-unsupported classes are
    // counted separately from real failures.
    let class = match read_block(bytes, 0) {
        Ok(b) => format!("{} v{}", b.class, b.version),
        Err(e) => {
            t.failures += 1;
            log.summary(format!("FAIL {label}: {e}"));
            return;
        }
    };
    match kind(name) {
        Some("dml") => match MaterialList::parse(bytes) {
            Ok(list) => {
                t.lists_ok += 1;
                *t.versions.entry(format!("dml {class}")).or_default() += 1;
                materials(label, world, &list, index, t);
                log.detail(format!(
                    "ok {label}: {class} detail_levels={} materials={}",
                    list.detail_levels,
                    list.materials.len()
                ));
            }
            Err(e) => {
                t.failures += 1;
                log.summary(format!("FAIL {label}: {class}: {e}"));
            }
        },
        Some("kqs") if class.starts_with("TS::Shape ") => {
            // Documented as unused development leftovers; layout not known.
            *t.unsupported.entry(class.clone()).or_default() += 1;
            log.detail(format!("unsupported {label}: {class}"));
        }
        Some("kqs") => match Shape::parse(bytes) {
            Ok(s) => {
                t.shapes_ok += 1;
                *t.versions.entry(format!("kqs {class}")).or_default() += 1;
                shape(label, world, &s, index, log, t);
            }
            Err(e) => {
                t.failures += 1;
                log.summary(format!("FAIL {label}: {class}: {e}"));
            }
        },
        _ => {}
    }
}

fn shape(label: &str, world: Option<&str>, s: &Shape, index: &NameIndex, log: &mut Log, t: &mut Tally) {
    t.unread_bytes += s.unread_bytes;
    t.trailing_bytes += s.trailing_bytes;
    if s.unread_bytes != 0 || s.trailing_bytes != 0 {
        log.detail(format!("note {label}: unread={} trailing={}", s.unread_bytes, s.trailing_bytes));
    }
    *t.loops_per_shape.entry(s.loops.len()).or_default() += 1;
    t.lights += s.lights.len();
    let material_count = s.materials.as_ref().map_or(0, |m| m.materials.len());
    match &s.materials {
        Some(list) => {
            *t.versions.entry(format!("kqs-embedded TS::MaterialList v{}", list.version)).or_default() += 1;
            materials(label, world, list, index, t);
        }
        None => t.shapes_without_materials += 1,
    }

    let transforms = s.nodes.iter().map(|n| &n.transform).chain(s.sequences.iter().map(|q| &q.transform));
    for tr in transforms {
        t.transforms += 1;
        t.transforms_rotating += tr.rotates() as usize;
        *t.transform_flags.entry(tr.flags).or_default() += 1;
    }
    t.nodes_without_uv_frame += s.nodes.iter().filter(|n| n.texture_frame < 0).count();

    for m in &s.meshes {
        t.meshes += 1;
        *t.versions.entry(format!("mesh TS::CelAnimMesh v{}", m.version)).or_default() += 1;
        t.frames_implied += m.frame_was_implied as usize;
        t.meshes_without_uvs += (m.texture_vertices_per_frame == 0) as usize;
        for uv in &m.texture_vertices {
            let (lo, hi) = t.uv_range.get_or_insert((*uv, *uv));
            for i in 0..2 {
                lo[i] = lo[i].min(uv[i]);
                hi[i] = hi[i].max(uv[i]);
            }
        }
        for f in &m.faces {
            t.faces += 1;
            if f.material < 0 || f.material as usize >= material_count.max(1) {
                t.material_index_bad += 1;
            }
            for &(vi, _) in &f.corners {
                let r = t.vertex_index_range.get_or_insert((vi, vi));
                r.0 = r.0.min(vi);
                r.1 = r.1.max(vi);
                t.corners_on_bbox_vertex += (vi == 0 || vi == 1) as usize;
                t.corners_past_frame += (vi < 0 || vi as usize >= m.vertices_per_frame) as usize;
            }
        }
        for fr in &m.frames {
            let start = fr.first_vertex.max(0) as usize;
            let Some(vs) = m.vertices.get(start..start + m.vertices_per_frame) else {
                t.bbox_frames_differ += 1;
                continue;
            };
            if vs.len() < 3 {
                continue;
            }
            let rest = &vs[2..];
            let lo: [u8; 3] = std::array::from_fn(|i| rest.iter().map(|v| v[i]).min().unwrap());
            let hi: [u8; 3] = std::array::from_fn(|i| rest.iter().map(|v| v[i]).max().unwrap());
            if vs[0][..3] == lo && vs[1][..3] == hi {
                t.bbox_frames_match += 1;
            } else {
                t.bbox_frames_differ += 1;
            }
        }
    }

    convention_check(label, s, log, t);

    let g = s.geometry(s.rest_sequence());
    t.geometry_problems += g.problems.len();
    t.rest_triangles += g.triangles.len();
    for p in g.problems.iter().take(3) {
        log.detail(format!("problem {label}: {p}"));
    }
    if let Some((lo, hi)) = bounds(&g.triangles) {
        let ext: [f32; 3] = std::array::from_fn(|i| hi[i] - lo[i]);
        const AXES: [&str; 3] = ["x", "y", "z"];
        let tallest = (0..3).max_by(|&a, &b| ext[a].total_cmp(&ext[b])).unwrap();
        *t.tallest_axis.entry(AXES[tallest]).or_default() += 1;
        let size = ext.iter().cloned().fold(0.0, f32::max);
        for i in 0..3 {
            if lo[i].abs() <= size * 0.02 {
                *t.sits_on_zero.entry(AXES[i]).or_default() += 1;
            }
        }
        log.detail(format!(
            "ok {label}: v{} nodes={} seqs={} loops={} meshes={} lights={} materials={} rest_seq={} tris={} min=({:.1},{:.1},{:.1}) max=({:.1},{:.1},{:.1})",
            s.version, s.nodes.len(), s.sequences.len(), s.loops.len(), s.meshes.len(), s.lights.len(),
            material_count, s.rest_sequence(), g.triangles.len(), lo[0], lo[1], lo[2], hi[0], hi[1], hi[2]
        ));
    } else {
        log.detail(format!("ok {label}: v{} no triangles in rest sequence {}", s.version, s.rest_sequence()));
    }
}

/// Which way are the 3x3 matrices applied? Data test: in a model made of
/// several parts, parts usually meet, so with the right convention many
/// vertices of one part sit exactly on vertices of another. Counts such
/// vertices with `Transform::apply` as written (row vector) and with every
/// matrix transposed (column vector).
fn convention_check(label: &str, s: &Shape, log: &mut Log, t: &mut Tally) {
    let rest = s.rest_sequence();
    let Some(seq) = s.sequences.get(rest) else { return };
    let nodes = s
        .subsequences
        .get(seq.mesh_subsequence.max(0) as usize)
        .map(|sub| sub.first_node.max(0) as usize..(sub.first_node.max(0) + sub.node_count.max(0)) as usize);
    let Some(nodes) = nodes else { return };
    let parts = nodes.clone().filter_map(|i| s.nodes.get(i)).count();
    let rotating = nodes.clone().filter_map(|i| s.nodes.get(i)).any(|n| n.transform.rotates());
    if parts < 2 || !rotating {
        return;
    }
    let row = touching(&s.geometry(rest).triangles);
    let mut transposed = s.clone();
    for n in &mut transposed.nodes {
        n.transform.m = transpose(n.transform.m);
    }
    let col = touching(&transposed.geometry(rest).triangles);
    t.convention_shapes += 1;
    t.convention_touching.0 += row;
    t.convention_touching.1 += col;
    // "Clearly" = one side has at least 1.5x the other and 4+ more vertices.
    if row >= col + 4 && row * 2 >= col * 3 {
        t.convention_wins.0 += 1;
    } else if col >= row + 4 && col * 2 >= row * 3 {
        t.convention_wins.1 += 1;
    } else {
        t.convention_wins.2 += 1;
    }
    log.detail(format!("convention {label}: parts={parts} touching row={row} column={col}"));
}

fn transpose(m: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| m[j][i]))
}

/// Number of distinct (part, vertex) positions that lie within a small
/// distance of a vertex of a different part.
fn touching(tris: &[kq8_data::shape::Triangle]) -> usize {
    let Some((lo, hi)) = bounds(tris) else { return 0 };
    let size = (0..3).map(|i| hi[i] - lo[i]).fold(0.0, f32::max);
    let cell = (size * 0.002).max(1e-4);
    let key = |p: [f32; 3]| -> [i64; 3] { std::array::from_fn(|i| (p[i] / cell).round() as i64) };
    let mut grid: HashMap<[i64; 3], BTreeSet<usize>> = HashMap::new();
    let mut points = BTreeSet::new();
    for t in tris {
        for p in t.positions {
            let k = key(p);
            grid.entry(k).or_default().insert(t.node);
            points.insert((t.node, k));
        }
    }
    points
        .iter()
        .filter(|(node, k)| {
            (-1..=1).any(|dx| (-1..=1).any(|dy| (-1..=1).any(|dz| {
                grid.get(&[k[0] + dx, k[1] + dy, k[2] + dz]).is_some_and(|s| s.iter().any(|n| n != node))
            })))
        })
        .count()
}

fn materials(_label: &str, world: Option<&str>, list: &MaterialList, index: &NameIndex, t: &mut Tally) {
    for m in &list.materials {
        *t.material_flags.entry(m.flags).or_default() += 1;
        *t.material_kinds.entry(format!("{:?}", m.kind())).or_default() += 1;
        if m.kind() != MaterialKind::Texture || m.texture.is_empty() {
            continue;
        }
        t.textures_referenced += 1;
        match index.locate(world, &m.texture) {
            Where::Here => t.textures_found_here += 1,
            Where::Game => t.textures_found_in_game += 1,
            Where::Elsewhere => t.textures_found_elsewhere += 1,
            Where::Missing => *t.textures_missing.entry(m.texture.to_ascii_lowercase()).or_default() += 1,
        }
    }
}

fn report(log: &mut Log, t: &Tally) {
    log.summary("");
    log.summary("classes and versions:");
    for (k, n) in &t.versions {
        log.summary(format!("  {n:6}  {k}"));
    }
    for (k, n) in &t.unsupported {
        log.summary(format!("  {n:6}  {k} (not supported, skipped)"));
    }
    log.summary(format!("bytes not read inside shape blocks: {}; bytes after shape blocks: {}", t.unread_bytes, t.trailing_bytes));
    log.summary(format!("loops per shape: {:?}", t.loops_per_shape));
    log.summary(format!("light nodes: {}", t.lights));
    log.summary(format!(
        "meshes={} faces={} vertex_index_range={:?} corners_on_bbox_vertex(0/1)={} corners_past_frame={}",
        t.meshes, t.faces, t.vertex_index_range, t.corners_on_bbox_vertex, t.corners_past_frame
    ));
    log.summary(format!(
        "frames: bbox vertices match min/max of the rest={} differ={} implied_frames={}",
        t.bbox_frames_match, t.bbox_frames_differ, t.frames_implied
    ));
    log.summary(format!(
        "texture coords: range={:?} meshes_without_uvs={} nodes_without_uv_frame={}",
        t.uv_range, t.meshes_without_uvs, t.nodes_without_uv_frame
    ));
    log.summary(format!(
        "material index out of range: {} faces; shapes without material list: {}",
        t.material_index_bad, t.shapes_without_materials
    ));
    log.summary(format!(
        "transforms={} rotating={} flags={:?}",
        t.transforms, t.transforms_rotating, t.transform_flags
    ));
    log.summary(format!(
        "rest pose: triangles={} geometry_problems={} tallest_axis={:?} min_near_zero_on_axis={:?}",
        t.rest_triangles, t.geometry_problems, t.tallest_axis, t.sits_on_zero
    ));
    log.summary(format!(
        "matrix convention test on {} multi-part shapes: touching vertices row-vector={} column-vector={}; clear wins row={} column={} unclear={}",
        t.convention_shapes, t.convention_touching.0, t.convention_touching.1,
        t.convention_wins.0, t.convention_wins.1, t.convention_wins.2
    ));
    log.summary(format!("material kinds: {:?}", t.material_kinds));
    log.summary("material flags (value: count):");
    for (k, n) in &t.material_flags {
        log.summary(format!("  0x{k:05X}: {n}"));
    }
    log.summary(format!(
        "textures: referenced={} found_in_same_world={} found_in_GAME={} found_only_in_other_world={} missing_names={} (materials={})",
        t.textures_referenced,
        t.textures_found_here,
        t.textures_found_in_game,
        t.textures_found_elsewhere,
        t.textures_missing.len(),
        t.textures_missing.values().sum::<usize>()
    ));
    for (name, n) in t.textures_missing.iter().take(30) {
        log.summary(format!("  missing {name} ({n})"));
    }
    for (name, n) in t.textures_missing.iter().skip(30) {
        log.detail(format!("  missing {name} ({n})"));
    }
}
