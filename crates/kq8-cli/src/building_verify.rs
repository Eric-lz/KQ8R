//! `kq8 building-verify`: parses every building shape (`.dig`) the game
//! has, in all archives and loose under `GAME/` and `data/`, and logs
//! counts and checks against the rules in docs/behaviour/buildings.md,
//! the BSP tree (tables 2 and 3) and the collision box table
//! `GAME/patch/BOUNDS.KQ` (docs/behaviour/connor-physics.md, Q31, Q33).

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::bounds::BoundingBoxes;
use kq8_data::building::{BspChild, Building, BSP_ROOT};
use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::{GameDir, VolArchive};

use crate::log::Log;

#[derive(Default)]
struct Tally {
    ok: usize,
    failures: usize,
    versions: BTreeMap<i32, usize>,
    texture_scales: BTreeMap<String, usize>,
    with_trailing_bytes: usize,
    surfaces: usize,
    bad_surfaces: usize,
    corner_counts: BTreeMap<u8, usize>,
    /// Corner-order normal (right-hand rule) agrees with the plane normal,
    /// taking the 0x80 "flipped" bit into account.
    order_agrees: usize,
    order_disagrees: usize,
    /// Largest distance of a surface corner from its plane.
    worst_plane_distance: f32,
    uv_outside_0_1: usize,
    max_material: u8,
    triangles: usize,
    /// Surfaces lying on a side of the building's box (outer walls, roof,
    /// bottom): does the right-hand-rule corner-order normal point out of
    /// the box or into it? Per side: (out, in).
    outer_sides: BTreeMap<&'static str, (usize, usize)>,
    /// Lowest z of each building relative to its own box: histogram.
    lowest_z: BTreeMap<String, usize>,
    /// BSP: buildings whose tree breaks a rule (index out of range, node 0
    /// a child, a node without exactly one parent, unreachable nodes).
    bsp_bad: usize,
    bsp_nodes: usize,
    bsp_leaves: usize,
    solid_leaves: usize,
    /// Leaves no node leads to.
    unreachable_leaves: usize,
    /// Buildings without any BSP node.
    bsp_empty: usize,
    /// Smallest and largest plane normal length.
    normal_length: (f32, f32),
    /// Distinct first bytes of the leaves, with counts.
    leaf_first_bytes: BTreeMap<u8, usize>,
    /// (first byte, number of nodes leading to the leaf): count.
    leaf_byte_by_parents: BTreeMap<(u8, usize), usize>,
    /// Shadow receivers (characters.md rule 37 (2), M4 4d-7b-3): surfaces
    /// whose visible side faces up (more than 45 degrees), those wider than
    /// 1,800 on x or y, and the widest (width, label, surface).
    floors: usize,
    floors_wider: usize,
    widest_floor: (f32, String, usize),
    /// Ball radius (half the `.dig` box diagonal, object-lighting.md rule
    /// 40): smallest and largest (radius, label).
    radius_range: Option<((f32, String), (f32, String))>,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("building-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("building-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let mut t = Tally { normal_length: (f32::INFINITY, 0.0), ..Default::default() };
    let bounds_ok = check_bounds(game, &mut log);
    for world in WORLDS {
        for volume in VOLUMES {
            let Some(path) = game.volume_path(world, volume) else { continue };
            let vol = match VolArchive::open(&path) {
                Ok(v) => v,
                Err(e) => {
                    log.summary(format!("FAIL {world}/{volume}.vol: {e}"));
                    t.failures += 1;
                    continue;
                }
            };
            for entry in vol.entries().iter().filter(|e| is_dig(&e.name)) {
                let label = format!("vol:{world}/{volume}/{}", entry.name);
                match vol.unpack(entry) {
                    Ok(u) => check(&label, &u.data, &mut log, &mut t),
                    Err(e) => {
                        log.summary(format!("FAIL {label}: {e}"));
                        t.failures += 1;
                    }
                }
            }
        }
    }
    for top in ["GAME", "data"] {
        if let Some(dir) = game.find(top) {
            walk(&dir, top, &mut log, &mut t);
        }
    }

    log.summary("");
    log.summary(format!("versions: {:?}; texture scales: {:?}; files with bytes after the block: {}", t.versions, t.texture_scales, t.with_trailing_bytes));
    log.summary(format!(
        "surfaces={} bad_indices={} triangles={} corners_per_surface={:?}",
        t.surfaces, t.bad_surfaces, t.triangles, t.corner_counts
    ));
    log.summary(format!(
        "corner-order normal (right-hand rule) agrees with plane and 0x80 bit: {} disagrees: {}",
        t.order_agrees, t.order_disagrees
    ));
    log.summary(format!(
        "largest corner distance from its plane: {:.2} units; texture coords outside 0..1: {}; largest material index: {}",
        t.worst_plane_distance, t.uv_outside_0_1, t.max_material
    ));
    log.summary(format!("lowest point in own space (z): {:?}", t.lowest_z));
    log.summary(format!(
        "surfaces on a side of the building's box, right-hand-rule corner-order normal pointing (out of the box, into the box): {:?} (into = clockwise seen from outside)",
        t.outer_sides
    ));
    log.summary(format!(
        "BSP: nodes={} leaves={} solid_leaves={} unreachable_leaves={} buildings_without_nodes={} buildings_breaking_tree_rules={}",
        t.bsp_nodes, t.bsp_leaves, t.solid_leaves, t.unreachable_leaves, t.bsp_empty, t.bsp_bad
    ));
    log.summary(format!("BSP leaf first bytes (value: count): {:?}", t.leaf_first_bytes));
    log.summary(format!("BSP leaves by (first byte, nodes leading to it): {:?}", t.leaf_byte_by_parents));
    log.summary(format!("plane normal length: {:.5} .. {:.5}", t.normal_length.0, t.normal_length.1));
    log.summary(format!(
        "shadow receivers (characters.md rule 37 (2)): {} surfaces face up (floors), {} of them wider than 1,800 on x or y; widest {:.0} ({} surface {})",
        t.floors, t.floors_wider, t.widest_floor.0, t.widest_floor.1, t.widest_floor.2
    ));
    if let Some(((lo, lo_at), (hi, hi_at))) = &t.radius_range {
        log.summary(format!("building balls (half the .dig box diagonal, object-lighting.md rule 40): radius {lo:.0} ({lo_at}) .. {hi:.0} ({hi_at})"));
    }
    log.summary(format!("TOTAL buildings_ok={} failures={} time_ms={}", t.ok, t.failures, started.elapsed().as_millis()));
    let pass = t.failures == 0 && t.ok > 0 && t.bsp_bad == 0 && bounds_ok;
    log.summary(if pass { "RESULT: PASS" } else { "RESULT: FAIL" });
    if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Reads `GAME/patch/BOUNDS.KQ` and logs what it holds. False if it is
/// missing or has problems.
fn check_bounds(game: &GameDir, log: &mut Log) -> bool {
    let Some(path) = game.find("GAME/patch/BOUNDS.KQ") else {
        log.summary("FAIL GAME/patch/BOUNDS.KQ not found");
        return false;
    };
    let bytes = match fs::read(&path) {
        Ok(b) => b,
        Err(e) => {
            log.summary(format!("FAIL {}: {e}", path.display()));
            return false;
        }
    };
    let (b, problems) = BoundingBoxes::parse(&bytes);
    for p in &problems {
        log.summary(format!("BOUNDS.KQ problem: {p}"));
    }
    let numbered_in_order = b.entries.iter().enumerate().all(|(i, e)| i == e.item);
    log.summary(format!(
        "BOUNDS.KQ: nItems={:?} entries={} numbered 0..{} each once: {numbered_in_order}",
        b.declared,
        b.entries.len(),
        b.entries.len().saturating_sub(1)
    ));
    let inside_out: Vec<String> =
        b.entries.iter().filter(|e| (0..3).any(|k| e.min[k] > e.max[k])).map(|e| format!("item{}={}", e.item, e.name)).collect();
    log.summary(format!("BOUNDS.KQ: boxes with min > max on some axis: {inside_out:?}"));
    log.summary(format!("BOUNDS.KQ: names used more than once, ignoring case (name, boxes differ): {:?}", b.duplicates()));
    let connor: Vec<_> = b.find_all("Connor").collect();
    for e in &connor {
        log.summary(format!("BOUNDS.KQ: item{}={} min={:?} max={:?}", e.item, e.name, e.min, e.max));
    }
    let connor_ok = connor.len() == 1 && connor[0].min == [-200.0, -200.0, 0.0] && connor[0].max == [200.0, 200.0, 1300.0];
    if !connor_ok {
        log.summary("FAIL BOUNDS.KQ: expected one Connor entry -200 -200 0 200 200 1300 (connor-physics.md)");
    }
    problems.is_empty() && b.declared == Some(b.entries.len()) && numbered_in_order && connor_ok
}

/// Checks one building's BSP tree: indices in range, node 0 is the root
/// (nobody's child), every other node has exactly one parent, and every
/// node can be reached from node 0. Returns the problems found.
fn check_bsp(b: &Building, t: &mut Tally) -> Vec<String> {
    let mut problems = Vec::new();
    t.bsp_nodes += b.nodes.len();
    t.bsp_leaves += b.leaf_flags.len();
    for &f in &b.leaf_flags {
        *t.leaf_first_bytes.entry(f).or_default() += 1;
        t.solid_leaves += (f & 1) as usize;
    }
    for p in &b.planes {
        let len = p.normal.iter().map(|v| v * v).sum::<f32>().sqrt();
        t.normal_length = (t.normal_length.0.min(len), t.normal_length.1.max(len));
    }
    if b.nodes.is_empty() {
        t.bsp_empty += 1;
        return problems;
    }
    let mut node_parents = vec![0usize; b.nodes.len()];
    let mut leaf_parents = vec![0usize; b.leaf_flags.len()];
    for (i, n) in b.nodes.iter().enumerate() {
        if n.plane as usize >= b.planes.len() {
            problems.push(format!("node {i}: plane {} of {}", n.plane, b.planes.len()));
        }
        for child in [n.front, n.back] {
            match child {
                BspChild::Node(c) if c < b.nodes.len() => node_parents[c] += 1,
                BspChild::Leaf(l) if l < b.leaf_flags.len() => leaf_parents[l] += 1,
                other => problems.push(format!("node {i}: child {other:?} out of range")),
            }
        }
    }
    if node_parents[BSP_ROOT] != 0 {
        problems.push(format!("node {BSP_ROOT} is the child of {} node(s)", node_parents[BSP_ROOT]));
    }
    let bad_parents = node_parents.iter().enumerate().filter(|&(i, &n)| i != BSP_ROOT && n != 1).count();
    if bad_parents > 0 {
        problems.push(format!("{bad_parents} node(s) other than {BSP_ROOT} without exactly one parent"));
    }
    let mut seen = vec![false; b.nodes.len()];
    let mut stack = vec![BSP_ROOT];
    while let Some(i) = stack.pop() {
        if std::mem::replace(&mut seen[i], true) {
            continue;
        }
        for child in [b.nodes[i].front, b.nodes[i].back] {
            if let BspChild::Node(c) = child {
                if c < b.nodes.len() {
                    stack.push(c);
                }
            }
        }
    }
    let unreached = seen.iter().filter(|s| !**s).count();
    if unreached > 0 {
        problems.push(format!("{unreached} node(s) not reachable from node {BSP_ROOT}"));
    }
    for (l, &n) in leaf_parents.iter().enumerate() {
        t.unreachable_leaves += (n == 0) as usize;
        *t.leaf_byte_by_parents.entry((b.leaf_flags[l], n)).or_default() += 1;
    }
    problems
}

fn is_dig(name: &str) -> bool {
    name.to_ascii_lowercase().ends_with(".dig")
}

fn walk(dir: &Path, label: &str, log: &mut Log, t: &mut Tally) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        let sub = format!("{label}/{name}");
        let path = e.path();
        if path.is_dir() {
            walk(&path, &sub, log, t);
        } else if is_dig(&name) {
            match fs::read(&path) {
                Ok(bytes) => check(&sub, &bytes, log, t),
                Err(err) => {
                    log.summary(format!("FAIL {sub}: {err}"));
                    t.failures += 1;
                }
            }
        }
    }
}

fn check(label: &str, bytes: &[u8], log: &mut Log, t: &mut Tally) {
    let b = match Building::parse(bytes) {
        Ok(b) => b,
        Err(e) => {
            t.failures += 1;
            log.summary(format!("FAIL {label}: {e}"));
            return;
        }
    };
    t.ok += 1;
    *t.versions.entry(b.version).or_default() += 1;
    *t.texture_scales.entry(b.texture_scale.to_string()).or_default() += 1;
    t.with_trailing_bytes += (b.trailing_bytes > 0) as usize;
    let mut agree = 0;
    for (i, s) in b.surfaces.iter().enumerate() {
        t.surfaces += 1;
        *t.corner_counts.entry(s.corner_count).or_default() += 1;
        t.max_material = t.max_material.max(s.material);
        let (Some(corners), Some(plane)) = (b.surface_corners(i), b.planes.get(s.plane as usize)) else {
            t.bad_surfaces += 1;
            continue;
        };
        for (p, _) in &corners {
            let d = (0..3).map(|k| plane.normal[k] * p[k]).sum::<f32>() + plane.d;
            t.worst_plane_distance = t.worst_plane_distance.max(d.abs());
        }
        let uvs_out = s.first_corner as usize..s.first_corner as usize + s.corner_count as usize;
        t.uv_outside_0_1 += uvs_out
            .filter_map(|c| b.corners.get(c))
            .filter_map(|c| b.texture_coords.get(c.texture_coord as usize))
            .filter(|uv| uv.iter().any(|v| !(0.0..=1.0).contains(v)))
            .count();
        if let Some(n) = b.corner_order_normal(i) {
            let dot: f32 = (0..3).map(|k| n[k] * plane.normal[k]).sum();
            // Expected: same way as the plane, or the opposite way if 0x80.
            if (dot > 0.0) != s.flipped() {
                agree += 1;
                t.order_agrees += 1;
            } else {
                t.order_disagrees += 1;
            }
        }
    }
    // Outer-wall test: surfaces whose corners all lie on one side of the box.
    let tol = |lo: f32, hi: f32| ((hi - lo) * 0.002).max(0.5);
    for i in 0..b.surfaces.len() {
        let (Some(c), Some(n)) = (b.surface_corners(i), b.corner_order_normal(i)) else { continue };
        for (axis, name_lo, name_hi) in [(0, "west (-x)", "east (+x)"), (1, "south (-y)", "north (+y)"), (2, "bottom (-z)", "top (+z)")] {
            let (lo, hi) = (b.bounds_min[axis], b.bounds_max[axis]);
            let e = tol(lo, hi);
            for (side, edge, outward) in [(name_lo, lo, -1.0f32), (name_hi, hi, 1.0)] {
                if c.iter().all(|(p, _)| (p[axis] - edge).abs() <= e) {
                    let entry = t.outer_sides.entry(side).or_default();
                    if n[axis] * outward > 0.0 {
                        entry.0 += 1;
                    } else {
                        entry.1 += 1;
                    }
                }
            }
        }
    }
    // Rule 37 (2) against the data: floors and their size, the ball.
    let (mut floors, mut wider, mut widest) = (0, 0, 0.0f32);
    for i in 0..b.surfaces.len() {
        let (Some(c), Some(n)) = (b.surface_corners(i), b.corner_order_normal(i)) else { continue };
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        // The right-hand-rule normal points away from the visible side.
        if len <= 0.0 || -n[2] / len < std::f32::consts::FRAC_1_SQRT_2 {
            continue;
        }
        t.floors += 1;
        floors += 1;
        let width = (0..2)
            .map(|k| c.iter().map(|(p, _)| p[k]).fold(f32::MIN, f32::max) - c.iter().map(|(p, _)| p[k]).fold(f32::MAX, f32::min))
            .fold(0.0, f32::max);
        t.floors_wider += usize::from(width > 1800.0);
        wider += usize::from(width > 1800.0);
        widest = widest.max(width);
        if width > t.widest_floor.0 {
            t.widest_floor = (width, label.to_string(), i);
        }
    }
    let radius = (0..3).map(|k| (b.bounds_max[k] - b.bounds_min[k]).powi(2)).sum::<f32>().sqrt() / 2.0;
    let r = t.radius_range.get_or_insert_with(|| ((radius, label.to_string()), (radius, label.to_string())));
    if radius < r.0 .0 {
        r.0 = (radius, label.to_string());
    }
    if radius > r.1 .0 {
        r.1 = (radius, label.to_string());
    }
    let bsp_problems = check_bsp(&b, t);
    if !bsp_problems.is_empty() {
        t.bsp_bad += 1;
        log.summary(format!("FAIL {label}: BSP: {}", bsp_problems.join("; ")));
    }
    let (tris, _) = b.triangles();
    t.triangles += tris.len();
    let low = b.points.iter().map(|p| p[2]).fold(f32::INFINITY, f32::min);
    let bucket = match low {
        z if z.abs() <= 2.0 => "about 0 (|z| <= 2)",
        z if z < -2.0 => "below 0",
        _ => "above 2",
    };
    *t.lowest_z.entry(bucket.to_string()).or_default() += 1;
    log.detail(format!(
        "ok {label}: v{} scale={} surfaces={} points={} triangles={} order_agrees={agree}/{} lowest_z={low:.1} box=({:.0},{:.0},{:.0})..({:.0},{:.0},{:.0}) trailing={} bsp_nodes={} leaves={} solid={} floors={floors} floors_wider_than_1800={wider} widest_floor={widest:.0} ball_radius={radius:.0}",
        b.version,
        b.texture_scale,
        b.surfaces.len(),
        b.points.len(),
        tris.len(),
        b.surfaces.len(),
        b.bounds_min[0],
        b.bounds_min[1],
        b.bounds_min[2],
        b.bounds_max[0],
        b.bounds_max[1],
        b.bounds_max[2],
        b.trailing_bytes,
        b.nodes.len(),
        b.leaf_flags.len(),
        b.leaf_flags.iter().filter(|f| *f & 1 != 0).count()
    ));
}
