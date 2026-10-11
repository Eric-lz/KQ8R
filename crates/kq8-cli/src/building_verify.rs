//! `kq8 building-verify`: parses every building shape (`.dig`) the game
//! has, in all archives and loose under `GAME/` and `data/`, and logs
//! counts and checks against the rules in docs/behaviour/buildings.md.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::building::Building;
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
    let mut t = Tally::default();
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
    log.summary(format!("TOTAL buildings_ok={} failures={} time_ms={}", t.ok, t.failures, started.elapsed().as_millis()));
    let pass = t.failures == 0 && t.ok > 0;
    log.summary(if pass { "RESULT: PASS" } else { "RESULT: FAIL" });
    if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
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
        "ok {label}: v{} scale={} surfaces={} points={} triangles={} order_agrees={agree}/{} lowest_z={low:.1} box=({:.0},{:.0},{:.0})..({:.0},{:.0},{:.0}) trailing={}",
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
        b.trailing_bytes
    ));
}
