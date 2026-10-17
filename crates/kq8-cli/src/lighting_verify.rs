//! `kq8 lighting-verify`: reads what object and building lighting needs,
//! in every world, and logs what the behaviour note
//! `docs/behaviour/object-lighting.md` says to check (M1 step 5a):
//!
//! - the light files `swtohwlt.cs` loads (rule 1), the ambient sum and the
//!   suns (rules 3-5), and the light a model corner gets facing away from
//!   the sun, up, and into the sun (Daventry: 0.30, 0.366, 0.42);
//! - every `.dil` (rules 12-15): surfaces, texel bytes, build id and
//!   surface count against the `.dig` of the same name, the size check of
//!   rule 13, Daventry's red histograms;
//! - the lighting kinds of all materials (rule 7);
//! - the buildings' `.kq` lighting keys (rule 12);
//! - the lights models carry (rule 26b, M1 step 7c-1): how many shapes
//!   carry lights, their flag values, and `portlite.kqs` frame by frame.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::building::Building;
use kq8_data::dil::BuildingLight;
use kq8_data::game_dir::WORLDS;
use kq8_data::kq_text;
use kq8_data::lightmap::Channels;
use kq8_data::lights::{self, Lights};
use kq8_data::{GameDir, MaterialList, Shape, VolArchive};

use crate::log::Log;
use crate::terrain_verify::{load, open_world};

/// The `.dil` files whose map sizes miss rule 13 on almost every surface
/// (the note; cause not found). World and base name, lower case.
const SIZE_RULE_EXCEPTIONS: [(&str, &str); 5] =
    [("temple4", "level4"), ("swamp", "ramp"), ("daventry", "zedrock1"), ("daventry", "zedrock2"), ("deadcity", "platform")];

/// Light-map placement (`object-lighting.md` "How to check", Q79): surfaces
/// with a light map in the 148 files with 48-byte records, surfaces with a
/// corner exactly on texel 0's centre, surfaces whose P0 is (4, 4) texture
/// texels past the drawn texel (0, 0) to within 0.05.
const PLACEMENT_SURFACES: usize = 164_838;
const PLACEMENT_AT_ZERO: usize = 7_511;
const PLACEMENT_P0_NEAR_4: usize = 160_649;

#[derive(Default)]
struct Tally {
    failures: usize,
    dil_files: usize,
    pairs: usize,
    pairs_matching: usize,
    pairs_all_fit: usize,
    /// Table D record size -> files (Q67).
    record_sizes: BTreeMap<usize, usize>,
    /// Files with bytes after the PERS block.
    with_trailing: usize,
    /// Surfaces whose byte offset says "no light map", per file (Q67).
    no_map: Vec<(String, usize, usize)>,
    /// (world/name, surfaces failing, surfaces) for pairs with misses.
    size_misses: Vec<(String, usize, usize)>,
    /// Model materials (in `.kqs`) and building material lists (`.dml`):
    /// shading bits -> count, and the three extra flags.
    shading: BTreeMap<(&'static str, u32), usize>,
    flags: BTreeMap<(&'static str, u32), usize>,
    /// Shading bits of materials with 0x1000 or 0x2000.
    forced: BTreeMap<(&'static str, u32, u32), usize>,
    material_failures: usize,
    /// Shape names (lower case) that carry light nodes (rule 26b).
    carrying: BTreeSet<String>,
    /// Light node flags -> count, and shape version -> shapes with lights.
    light_flags: BTreeMap<u8, usize>,
    light_versions: BTreeMap<i32, usize>,
    /// `portlite.kqs`'s lights per sequence (logged once).
    portlite: Vec<String>,
    /// Rule 16's rectangles against rule 13's placement (M1 7d).
    rects: RectCheck,
}

/// Rule 16's light-map rectangle (P0, U, V in the `.dil`) against where
/// rule 13 lays the light texels on the surface's texture (M1 step 7d).
#[derive(Default)]
struct RectCheck {
    surfaces: usize,
    /// Surfaces whose texture placement could not be solved (fewer than 3
    /// corners in a usable shape).
    unsolved: usize,
    /// Largest distance between stored and predicted P0, U, V; and how
    /// many surfaces are within 1 unit on all three.
    worst: [f32; 3],
    within_1: usize,
    /// Largest distance of P0 from the surface's plane.
    worst_plane: f32,
    /// Bit 0x80 facing agrees with the corner order's front.
    facing_agree: usize,
    facing_tested: usize,
    /// P0 against rule 13's texel (0, 0) moved by 4 texture texels on x and
    /// y: the largest difference, and how many within 1 unit.
    worst_shifted: f32,
    shifted_within_1: usize,
    /// Light-map placement (M1 7k, Q79): corners, corners outside the
    /// map's first or last texel centre (before the clamp) and the largest
    /// overshoot in light texels, surfaces with a corner at exactly 0 on
    /// x and y / on x or y.
    corners: usize,
    corners_outside: usize,
    worst_outside: f32,
    zero_both: usize,
    zero_either: usize,
    /// P0 minus the drawn texel (0, 0), in texture texels: within 0.05 of
    /// (4, 4), the largest distance from (4, 4), and the sum (for the mean).
    p0_near_4: usize,
    p0_worst: f32,
    p0_sum: [f64; 2],
    /// Our painting's texel (0, 0) and last texel against the drawn ones
    /// (`wall_light::texel_position`), in texture texels: maps at least 2
    /// by 2 (within 0.05 on both, largest), and maps 1 texel wide or high
    /// (count, largest difference on the 1-texel axis and on the other).
    paint_tested: usize,
    paint_within: usize,
    paint_worst: f32,
    thin: usize,
    thin_worst_flat: f32,
    thin_worst_other: f32,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("lighting-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("lighting-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let mut t = Tally::default();

    log.summary("");
    log.summary("== Lights for models (rules 1-5)");
    for world in WORLDS {
        let vols = open_world(game, world, &mut log);
        if let Err(e) = world_lights(game, world, &vols, &mut log) {
            t.failures += 1;
            log.summary(format!("FAIL {world}: {e}"));
        }
    }

    log.summary("");
    log.summary("== Building light maps, .dil (rules 12-15)");
    for world in WORLDS {
        let vols = open_world(game, world, &mut log);
        building_lights(world, &vols, &mut log, &mut t);
    }
    log.summary(format!(
        ".dil files={} with a .dig of the same name={} build id and surface count match={} every surface fits rule 13={}",
        t.dil_files, t.pairs, t.pairs_matching, t.pairs_all_fit
    ));
    log.summary(format!(
        "surface record sizes (bytes: files) {:?}; files with bytes after the PERS block: {}",
        t.record_sizes, t.with_trailing
    ));
    for (label, n, all) in &t.no_map {
        log.summary(format!("  {label}: {n} of {all} surfaces have byte offset 0xFFFFFFFF, no light map (Q67)"));
    }
    let mut unexpected = 0;
    for (name, bad, all) in &t.size_misses {
        let expected = SIZE_RULE_EXCEPTIONS.iter().any(|(w, n)| name.eq_ignore_ascii_case(&format!("{w}/{n}")));
        unexpected += usize::from(!expected);
        log.summary(format!(
            "  size rule missed: {name} {bad} of {all} surfaces{}",
            if expected { " (one of the note's 5)" } else { "  NOT one of the note's 5" }
        ));
    }
    if unexpected > 0 {
        t.failures += 1;
        log.summary(format!("FAIL {unexpected} file(s) miss the size rule that the note does not name"));
    }

    let r = &t.rects;
    log.summary(format!(
        "rule 16 rectangles vs rule 13 placement: {} surfaces with P0/U/V, {} unsolved; within 1 unit on P0, U and V: {}; largest differences P0 {:.2}, U {:.2}, V {:.2}; P0 farthest from its plane {:.2}",
        r.surfaces, r.unsolved, r.within_1, r.worst[0], r.worst[1], r.worst[2], r.worst_plane
    ));
    log.summary(format!(
        "  P0 against rule 13's texel (0, 0) + 4 texture texels on x and y: within 1 unit on {} surfaces, largest difference {:.2}",
        r.shifted_within_1, r.worst_shifted
    ));
    log.summary(format!(
        "light-map placement (Q79): {} corners of {} surfaces; outside the first/last texel centre before the clamp: {} (largest by {:.4} light texel); surfaces with a corner at exactly 0 on x and y: {}, on x or y: {}",
        r.corners, r.surfaces, r.corners_outside, r.worst_outside, r.zero_both, r.zero_either
    ));
    let n = r.surfaces.max(1) as f64;
    log.summary(format!(
        "  P0 minus the drawn texel (0, 0), texture texels: mean ({:.3}, {:.3}); within 0.05 of (4, 4) on {} surfaces; farthest {:.3}",
        r.p0_sum[0] / n,
        r.p0_sum[1] / n,
        r.p0_near_4,
        r.p0_worst
    ));
    log.summary(format!(
        "  our painting's texels (0, 0) and last vs drawn, texture texels: maps 2x2 or more {} surfaces, within 0.05 on {}, largest {:.3}; maps 1 texel wide or high {} surfaces, largest on the 1-texel axis {:.3}, on the other {:.3}",
        r.paint_tested, r.paint_within, r.paint_worst, r.thin, r.thin_worst_flat, r.thin_worst_other
    ));
    // The note's main claims: no corner outside the map, and P0 a quarter
    // light texel past the drawn texel (0, 0). A miss is a failure.
    let mean_ok = r.p0_sum.iter().all(|&v| (v / n - 4.0).abs() <= 0.05);
    if r.surfaces != PLACEMENT_SURFACES || r.corners_outside != 0 || !mean_ok {
        t.failures += 1;
        log.summary(format!(
            "FAIL light-map placement differs from the note (expect {PLACEMENT_SURFACES} surfaces, 0 corners outside, mean P0 offset (4, 4))"
        ));
    } else {
        log.summary("  light-map placement: surfaces, 0 corners outside and mean P0 offset (4, 4) match the note (Q79)");
    }
    // Two counts in the note we have not reproduced (QUESTIONS.md Q80):
    // logged, not failed, until the study session says how it counted.
    let zero_ok = r.zero_both == PLACEMENT_AT_ZERO || r.zero_either == PLACEMENT_AT_ZERO;
    if !zero_ok || r.p0_near_4 != PLACEMENT_P0_NEAR_4 {
        log.summary(format!(
            "  DIFFERS from the note (Q80 open): surfaces with a corner at 0 {} / {} (note {PLACEMENT_AT_ZERO}), P0 within 0.05 of (4, 4) {} (note {PLACEMENT_P0_NEAR_4})",
            r.zero_both, r.zero_either, r.p0_near_4
        ));
    }
    log.summary(format!(
        "facing by bit 0x80 (rule 16) agrees with the corner order's front on {} of {} surfaces",
        r.facing_agree, r.facing_tested
    ));

    log.summary("");
    log.summary("== Material lighting kinds (rule 7)");
    material_kinds(game, &mut log, &mut t);

    log.summary("");
    log.summary("== Lights carried by models (rule 26b)");
    log.summary(format!(
        "shapes carrying light nodes: {} (A measured 174); light flags (value: nodes) {:?}; shape version: shapes {:?}",
        t.carrying.len(),
        t.light_flags,
        t.light_versions
    ));
    for line in &t.portlite {
        log.summary(format!("  {line}"));
    }

    log.summary("");
    log.summary("== Buildings' .kq lighting keys (rule 12)");
    for world in WORLDS {
        let vols = open_world(game, world, &mut log);
        building_keys(world, &vols, &mut log);
    }

    log.summary("");
    log.summary(format!("TOTAL failures={} time_ms={}", t.failures, started.elapsed().as_millis()));
    let pass = t.failures == 0 && t.dil_files > 0;
    log.summary(if pass { "RESULT: PASS" } else { "RESULT: FAIL" });
    if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn length(v: [f32; 3]) -> f32 {
    (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
}

fn rgb(v: [f32; 3]) -> String {
    if v[0] == v[1] && v[1] == v[2] {
        format!("{:.3}", v[0])
    } else {
        format!("({:.3}, {:.3}, {:.3})", v[0], v[1], v[2])
    }
}

/// The hardware lights of one world (rule 1) and what a model corner gets.
fn world_lights(game: &GameDir, world: &str, vols: &[(String, VolArchive)], log: &mut Log) -> Result<(), String> {
    let (at, script) = load(game, world, vols, "swtohwlt.cs")?;
    let files = lights::load_kq_lines(&script);
    log.summary(format!("{world}: {at} loads {files:?}"));
    let mut l = Lights::default();
    for f in &files {
        let (fat, bytes) = load(game, world, vols, f)?;
        l.add_sections(f, &kq_text::parse(&bytes));
        log.detail(format!("  {f}: {fat}"));
    }
    for p in &l.problems {
        log.summary(format!("  problem: {p}"));
    }
    log.summary(format!("  ambient {} (from {} light(s))", rgb(l.ambient), l.ambient_count));
    for s in &l.suns {
        log.summary(format!(
            "  sun {} aim ({}, {}, {}) length {:.3}",
            rgb(s.colour),
            s.aim[0],
            s.aim[1],
            s.aim[2],
            length(s.aim)
        ));
    }
    if !l.points.is_empty() {
        log.summary(format!("  point lights: {} (not drawn)", l.points.len()));
    }
    // What a corner gets: the first sun's direction, away / up / into it.
    let (away, into) = match l.suns.first() {
        Some(s) if length(s.aim) > 0.0 => {
            let n = s.aim.map(|v| v / length(s.aim));
            (n, n.map(|v| -v))
        }
        _ => ([0.0, 0.0, -1.0], [0.0, 0.0, 1.0]),
    };
    let values = [l.corner_light(away), l.corner_light([0.0, 0.0, 1.0]), l.corner_light(into)];
    log.summary(format!(
        "  model corner light: facing away from the sun {}, facing up {}, facing the sun {}",
        rgb(values[0]),
        rgb(values[1]),
        rgb(values[2])
    ));
    // The note's table for Daventry.
    if world == "Daventry" {
        let want = [0.3, 0.366, 0.3 + 0.1 * (1.0f32 + 0.66 * 0.66).sqrt()];
        let ok = values.iter().zip(want).all(|(v, w)| v.iter().all(|c| (c - w).abs() < 1e-3));
        log.summary(format!(
            "  Daventry check (note: 0.30, 0.366, 0.42): {}",
            if ok { "matches" } else { "DIFFERS" }
        ));
        if !ok {
            return Err("Daventry's corner light differs from the note's table".into());
        }
    }
    // Every file in the Light folder, for reference (only the two above
    // are loaded on hardware).
    if let Some(dir) = game.find(&format!("data/{world}/KQ/Light")) {
        let mut names: Vec<String> = fs::read_dir(&dir)
            .map(|rd| rd.flatten().map(|e| e.file_name().to_string_lossy().into_owned()).collect())
            .unwrap_or_default();
        names.sort();
        let listed: Vec<String> = names
            .iter()
            .map(|n| {
                let class = fs::read(dir.join(n))
                    .ok()
                    .and_then(|b| kq_text::parse(&b).first().and_then(|s| s.get("classType").map(str::to_string)))
                    .unwrap_or("-".into());
                format!("{n} ({class})")
            })
            .collect();
        log.detail(format!("  Light folder: {}", listed.join(", ")));
    }
    Ok(())
}

/// Every `.dil` of one world, paired with the `.dig` of the same name.
fn building_lights(world: &str, vols: &[(String, VolArchive)], log: &mut Log, t: &mut Tally) {
    // Lower-case base name -> (label, bytes).
    let mut digs: BTreeMap<String, (String, Vec<u8>)> = BTreeMap::new();
    let mut dils: Vec<(String, String, Vec<u8>)> = Vec::new();
    for (v, a) in vols {
        for e in a.entries() {
            let lower = e.name.to_ascii_lowercase();
            let (is_dig, is_dil) = (lower.ends_with(".dig"), lower.ends_with(".dil"));
            if !is_dig && !is_dil {
                continue;
            }
            let label = format!("vol:{world}/{v}/{}", e.name);
            let bytes = match a.unpack(e) {
                Ok(u) => u.data,
                Err(err) => {
                    t.failures += 1;
                    log.summary(format!("FAIL {label}: {err}"));
                    continue;
                }
            };
            let base = lower[..lower.len() - 4].to_string();
            if is_dig {
                digs.entry(base).or_insert((label, bytes));
            } else {
                dils.push((base, label, bytes));
            }
        }
    }
    let (mut world_dils, mut world_pairs) = (0, 0);
    for (base, label, bytes) in &dils {
        let l = match BuildingLight::parse(bytes) {
            Ok(l) => l,
            Err(e) => {
                t.failures += 1;
                log.summary(format!("FAIL {label}: {e}"));
                continue;
            }
        };
        t.dil_files += 1;
        world_dils += 1;
        let mut line = format!(
            "{label}: v{} build id {} shift {} unlisted u32 {} counts {:?} surface record {} bytes, texel bytes {} end {} bytes after block {}",
            l.version,
            l.build_id,
            l.scale_shift,
            l.unlisted,
            l.counts,
            l.surface_record_size,
            l.texel_bytes().len(),
            l.end_list_len,
            l.trailing_bytes
        );
        *t.record_sizes.entry(l.surface_record_size).or_default() += 1;
        let no_map = l.surfaces.iter().filter(|s| !s.has_map()).count();
        if no_map > 0 {
            t.no_map.push((label.clone(), no_map, l.surfaces.len()));
        }
        t.with_trailing += usize::from(l.trailing_bytes > 0);
        if l.surface_record_size != 48 {
            log.summary(format!("  {label}: surface records are {} bytes, not 48 (Q67)", l.surface_record_size));
        }
        if l.version != 4 || l.scale_shift != 4 || l.unlisted != 0 || l.counts[..3] != [0, 0, 0] || l.end_list_len != 0 {
            log.summary(format!(
                "  note: {label} differs from the 'every file' values (version 4, shift 4, unlisted u32 0, tables A-C empty, end 0)"
            ));
        }
        if let Some((dig_label, dig_bytes)) = digs.get(base) {
            match Building::parse(dig_bytes) {
                Ok(b) => {
                    t.pairs += 1;
                    world_pairs += 1;
                    let ids = b.build_id == l.build_id;
                    let count = b.surfaces.len() == l.surfaces.len();
                    if ids && count {
                        t.pairs_matching += 1;
                    } else {
                        t.failures += 1;
                        log.summary(format!(
                            "FAIL {label} vs {dig_label}: build id {} vs {}, surfaces {} vs {}",
                            l.build_id,
                            b.build_id,
                            l.surfaces.len(),
                            b.surfaces.len()
                        ));
                    }
                    let misses = (0..l.surfaces.len().min(b.surfaces.len())).filter(|&i| !l.fits(i, &b.surfaces[i])).count();
                    if misses == 0 && count {
                        t.pairs_all_fit += 1;
                    } else {
                        t.size_misses.push((format!("{}/{base}", world.to_ascii_lowercase()), misses, l.surfaces.len()));
                    }
                    line += &format!("; .dig surfaces {} build id {} size rule misses {misses}", b.surfaces.len(), b.build_id);
                    if world == "Daventry" {
                        daventry_histogram(base, &l, log, t);
                    }
                    rect_check(&l, &b, &mut t.rects);
                }
                Err(e) => {
                    t.failures += 1;
                    log.summary(format!("FAIL {dig_label}: {e}"));
                }
            }
        } else {
            line += "; no .dig of this name (software or alternate map)";
        }
        log.detail(line);
    }
    log.summary(format!("{world}: {world_dils} .dil files, {world_pairs} with a .dig of the same name"));
}

/// Rule 16's P0, U, V of every surface against rule 13: light texel (i, j)
/// sits at texture position 16 i - offset_x (from the surface's texture
/// origin), so P0 = the surface's texture placement at texel position
/// (texture offset - map offset), U = 16 (width - 1) texels along the
/// texture's x, V likewise along y. The placement (texel -> building
/// space) is solved from three of the surface's corners.
fn rect_check(l: &BuildingLight, b: &Building, r: &mut RectCheck) {
    for (i, s) in l.surfaces.iter().enumerate() {
        let (Some(rect), Some(dig)) = (s.rect, b.surfaces.get(i)) else { continue };
        if !s.has_map() {
            continue;
        }
        r.surfaces += 1;
        let Some(corners) = b.surface_corners(i) else {
            r.unsolved += 1;
            continue;
        };
        if let (Some(n), Some(plane)) = (b.corner_order_normal(i), b.planes.get(dig.plane as usize)) {
            // Corner order: the normal points away from the front.
            let front = n.map(|v| -v);
            let by_bit = kq8_data::wall_light::facing(plane.normal, dig.flipped());
            r.facing_tested += 1;
            r.facing_agree += usize::from(front.iter().zip(by_bit).map(|(a, b)| a * b).sum::<f32>() > 0.0);
            let off = plane.normal.iter().zip(rect.p0).map(|(a, b)| a * b).sum::<f32>() + plane.d;
            r.worst_plane = r.worst_plane.max(off.abs());
        }
        // The best-shaped corner triangle in texel space.
        let mut best: Option<(f32, [usize; 3])> = None;
        for a in 0..corners.len() {
            for b2 in a + 1..corners.len() {
                for c in b2 + 1..corners.len() {
                    let (ta, tb, tc) = (corners[a].1, corners[b2].1, corners[c].1);
                    let area = ((tb[0] - ta[0]) * (tc[1] - ta[1]) - (tb[1] - ta[1]) * (tc[0] - ta[0])).abs();
                    if best.is_none_or(|(x, _)| area > x) {
                        best = Some((area, [a, b2, c]));
                    }
                }
            }
        }
        let Some((_, [a, b2, c])) = best.filter(|(area, _)| *area > 1e-3) else {
            r.unsolved += 1;
            continue;
        };
        let (p0, p1, p2) = (corners[a].0, corners[b2].0, corners[c].0);
        let (t0, t1, t2) = (corners[a].1, corners[b2].1, corners[c].1);
        let (s1, u1, s2, u2) = (t1[0] - t0[0], t1[1] - t0[1], t2[0] - t0[0], t2[1] - t0[1]);
        let det = s1 * u2 - s2 * u1;
        // Per texel along x (bx) and along y (by), in building space.
        let bx: [f32; 3] = std::array::from_fn(|k| ((p1[k] - p0[k]) * u2 - (p2[k] - p0[k]) * u1) / det);
        let by: [f32; 3] = std::array::from_fn(|k| ((p2[k] - p0[k]) * s1 - (p1[k] - p0[k]) * s2) / det);
        let at = |tx: f32, ty: f32| -> [f32; 3] { std::array::from_fn(|k| p0[k] + bx[k] * (tx - t0[0]) + by[k] * (ty - t0[1])) };
        let step = l.texels_per_light() as f32;
        let first = at(dig.texture_offset[0] as f32 - s.map_offset[0] as f32, dig.texture_offset[1] as f32 - s.map_offset[1] as f32);
        let u: [f32; 3] = std::array::from_fn(|k| bx[k] * step * (s.width as f32 - 1.0));
        let v: [f32; 3] = std::array::from_fn(|k| by[k] * step * (s.height as f32 - 1.0));
        let diff = |x: [f32; 3], y: [f32; 3]| length([x[0] - y[0], x[1] - y[1], x[2] - y[2]]);
        let d = [diff(first, rect.p0), diff(u, rect.u), diff(v, rect.v)];
        for (w, d) in r.worst.iter_mut().zip(d) {
            *w = w.max(d);
        }
        r.within_1 += usize::from(d.iter().all(|&x| x <= 1.0));
        // Measured: the stored P0 sits 4 texture texels further along x and
        // y than rule 13's texel (0, 0) (QUESTIONS.md Q79).
        let shifted = at(
            dig.texture_offset[0] as f32 - s.map_offset[0] as f32 + 4.0,
            dig.texture_offset[1] as f32 - s.map_offset[1] as f32 + 4.0,
        );
        let ds = diff(shifted, rect.p0);
        r.worst_shifted = r.worst_shifted.max(ds);
        r.shifted_within_1 += usize::from(ds <= 1.0);

        // Light-map placement (Q79, "How to check"): each corner's light
        // texel position (texture x + offset_x) / 16, texture x = u (size_x
        // + 1) from the surface's own texture origin, before the clamp.
        let last = [s.width as f32 - 1.0, s.height as f32 - 1.0];
        let (mut both, mut either) = (false, false);
        let first_corner = dig.first_corner as usize;
        for c in first_corner..first_corner + dig.corner_count as usize {
            let Some(uv) = b.corners.get(c).and_then(|c| b.texture_coords.get(c.texture_coord as usize)) else { continue };
            let pos: [f32; 2] = std::array::from_fn(|k| (uv[k] * (dig.texture_size[k] as f32 + 1.0) + s.map_offset[k] as f32) / step);
            r.corners += 1;
            let over = (0..2).map(|k| (-pos[k]).max(pos[k] - last[k])).fold(f32::MIN, f32::max);
            if over > 0.0 {
                r.corners_outside += 1;
                r.worst_outside = r.worst_outside.max(over);
            }
            both |= pos == [0.0, 0.0];
            either |= pos[0] == 0.0 || pos[1] == 0.0;
        }
        r.zero_both += usize::from(both);
        r.zero_either += usize::from(either);

        // A building-space point back in texture texels (least squares on
        // the placement's x and y steps).
        let (xx, xy, yy) = (dot3(bx, bx), dot3(bx, by), dot3(by, by));
        let in_texels = |p: [f32; 3]| -> [f32; 2] {
            let d = [p[0] - p0[0], p[1] - p0[1], p[2] - p0[2]];
            let (dx, dy) = (dot3(bx, d), dot3(by, d));
            let det = xx * yy - xy * xy;
            [t0[0] + (dx * yy - dy * xy) / det, t0[1] + (dy * xx - dx * xy) / det]
        };
        // Rule 13's drawn texel (i, j): 16 i - offset_x from the texture
        // origin (in `at`'s texels, which include the texture offset).
        let drawn = |i: f32, j: f32| -> [f32; 2] {
            [dig.texture_offset[0] as f32 - s.map_offset[0] as f32 + step * i, dig.texture_offset[1] as f32 - s.map_offset[1] as f32 + step * j]
        };
        let pt = in_texels(rect.p0);
        let d0 = drawn(0.0, 0.0);
        let rel = [pt[0] - d0[0], pt[1] - d0[1]];
        r.p0_sum[0] += rel[0] as f64;
        r.p0_sum[1] += rel[1] as f64;
        let off4 = (rel[0] - 4.0).abs().max((rel[1] - 4.0).abs());
        r.p0_near_4 += usize::from(off4 <= 0.05);
        r.p0_worst = r.p0_worst.max(off4);

        // Where our point-light painting puts texels (0, 0) and the last one.
        let (w, h) = (s.width as usize, s.height as usize);
        let ours = |i: usize, j: usize| in_texels(kq8_data::wall_light::texel_position(&rect, w, h, i, j));
        let gap = |a: [f32; 2], b: [f32; 2]| [(a[0] - b[0]).abs(), (a[1] - b[1]).abs()];
        let g0 = gap(ours(0, 0), d0);
        let g1 = gap(ours(w - 1, h - 1), drawn(last[0], last[1]));
        if w > 1 && h > 1 {
            let worst = g0[0].max(g0[1]).max(g1[0]).max(g1[1]);
            r.paint_tested += 1;
            r.paint_within += usize::from(worst <= 0.05);
            r.paint_worst = r.paint_worst.max(worst);
        } else if w >= 1 && h >= 1 {
            r.thin += 1;
            for k in 0..2 {
                let flat = [w, h][k] == 1;
                let worst = g0[k].max(g1[k]);
                if flat {
                    r.thin_worst_flat = r.thin_worst_flat.max(worst);
                } else {
                    r.thin_worst_other = r.thin_worst_other.max(worst);
                }
            }
        }
    }
}

fn dot3(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Red channel of a Daventry building's light map (the note's table), and
/// the Connor's-hut check.
fn daventry_histogram(base: &str, l: &BuildingLight, log: &mut Log, t: &mut Tally) {
    let mut red = [0usize; 16];
    for v in l.values() {
        red[Channels::of(v).red as usize] += 1;
    }
    let total: usize = red.iter().sum();
    let mut order: Vec<usize> = (0..16).collect();
    order.sort_by_key(|&v| std::cmp::Reverse(red[v]));
    let top: Vec<String> = order.iter().take(5).map(|&v| format!("{v} x{}", red[v])).collect();
    log.summary(format!(
        "  Daventry {base}.dil: {} surfaces, {} texels, red most common {}; at 15: {:.1}%",
        l.surfaces.len(),
        total,
        top.join(", "),
        100.0 * red[15] as f32 / total.max(1) as f32
    ));
    if base == "cnnrhut" {
        let ok = l.surfaces.len() == 1129
            && l.texel_bytes().len() == 162_170
            && red[0] == 16_017
            && red[3] == 7_775
            && red[4] == 7_344;
        log.summary(format!(
            "  cnnrhut check (note: 1,129 surfaces, 162,170 bytes, red 0 x16,017, 3 x7,775, 4 x7,344): {}",
            if ok { "matches" } else { "DIFFERS" }
        ));
        if !ok {
            t.failures += 1;
            log.summary("FAIL cnnrhut.dil differs from the note");
        }
    }
}

/// Shading bits (0x0F00) and the 0x1000 / 0x2000 / 0x10000 flags of every
/// material, models (`.kqs`) and building material lists (`.dml`)
/// separately. Counts uses in archives, so a file in several worlds counts
/// once per world.
fn material_kinds(game: &GameDir, log: &mut Log, t: &mut Tally) {
    let mut seen_files = 0;
    for world in WORLDS {
        let vols = open_world(game, world, log);
        for (v, a) in &vols {
            for e in a.entries() {
                let lower = e.name.to_ascii_lowercase();
                if !(lower.ends_with(".kqs") || lower.ends_with(".dml")) {
                    continue;
                }
                if let Ok(u) = a.unpack(e) {
                    let label = format!("vol:{world}/{v}/{}", e.name);
                    seen_files += usize::from(material_file(&label, &e.name, &u.data, log, t));
                }
            }
        }
    }
    // Models outside the archives (Connor and the shared ones in GAME/).
    if let Some(dir) = game.find("GAME") {
        walk(&dir, "GAME", &mut |label, name, bytes| {
            seen_files += usize::from(material_file(label, name, bytes, log, t));
        });
    }
    log.summary(format!(
        "files read: {seen_files}; skipped (not readable here, see shape-verify): {}",
        t.material_failures
    ));
    let name = |s: u32| match s {
        0x100 => "none (0x100)".to_string(),
        0x200 => "flat (0x200)".to_string(),
        0x400 => "smooth (0x400)".to_string(),
        0 => "no shading bits (0x000)".to_string(),
        other => format!("other (0x{other:03X})"),
    };
    for ((group, s), n) in &t.shading {
        log.summary(format!("  {group}: {} x{n}", name(*s)));
    }
    for ((group, f), n) in &t.flags {
        log.summary(format!("  {group}: flag 0x{f:X} x{n}"));
    }
    for ((group, f, s), n) in &t.forced {
        log.summary(format!("  {group}: flag 0x{f:X} with shading {} x{n} (rule 7: drawn as none)", name(*s)));
    }
}

/// Tallies the materials of one `.kqs` or `.dml` file. True if read.
fn material_file(label: &str, name: &str, bytes: &[u8], log: &mut Log, t: &mut Tally) -> bool {
    let lower = name.to_ascii_lowercase();
    let (group, list) = if lower.ends_with(".kqs") {
        match Shape::parse(bytes) {
            Ok(s) => {
                model_lights(name, &s, t);
                ("models (.kqs)", s.materials)
            }
            Err(e) => {
                t.material_failures += 1;
                log.detail(format!("  skipped {label}: {e}"));
                return false;
            }
        }
    } else if lower.ends_with(".dml") {
        match MaterialList::parse(bytes) {
            Ok(list) => ("material lists (.dml)", Some(list)),
            Err(e) => {
                t.material_failures += 1;
                log.detail(format!("  skipped {label}: {e}"));
                return false;
            }
        }
    } else {
        return false;
    };
    for m in list.iter().flat_map(|l| &l.materials) {
        let shading = m.flags & 0xF00;
        *t.shading.entry((group, shading)).or_default() += 1;
        for f in [0x1000, 0x2000, 0x10000] {
            if m.flags & f != 0 {
                *t.flags.entry((group, f)).or_default() += 1;
                if f != 0x10000 {
                    *t.forced.entry((group, f, shading)).or_default() += 1;
                }
            }
        }
    }
    true
}

/// Tallies the lights a shape carries (rule 26b); `portlite.kqs` frame by
/// frame (A's example: 17 frames, a point light circling about 400 units,
/// colour from (1.0, 1.0, 0.07) to (0, 1.0, 0.56) of 255, Begin 1,500, End
/// 3,000).
fn model_lights(name: &str, s: &Shape, t: &mut Tally) {
    if s.lights.is_empty() {
        return;
    }
    let lower = name.to_ascii_lowercase();
    if t.carrying.insert(lower.clone()) {
        *t.light_versions.entry(s.version).or_default() += 1;
        for l in &s.lights {
            *t.light_flags.entry(l.flags).or_default() += 1;
        }
    }
    if lower == "portlite.kqs" && t.portlite.is_empty() {
        t.portlite.push(format!("portlite.kqs v{}: {} light nodes, {} sequences", s.version, s.lights.len(), s.sequences.len()));
        for seq in 0..s.sequences.len() {
            for l in s.sequence_lights(seq) {
                let r = (l.position[0].powi(2) + l.position[1].powi(2)).sqrt();
                t.portlite.push(format!(
                    "sequence {seq}: flags {:#x} {} at ({:.0}, {:.0}, {:.0}) ({r:.0} from the axis), colour ({:.3}, {:.3}, {:.3}), Begin {:.0}, End {:.0}",
                    l.flags,
                    if l.point { "point" } else { "sun-type" },
                    l.position[0],
                    l.position[1],
                    l.position[2],
                    l.colour[0],
                    l.colour[1],
                    l.colour[2],
                    l.begin,
                    l.end
                ));
            }
        }
    }
}

fn walk(dir: &Path, label: &str, f: &mut dyn FnMut(&str, &str, &[u8])) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.flatten().collect();
    entries.sort_by_key(|e| e.file_name());
    for e in entries {
        let name = e.file_name().to_string_lossy().into_owned();
        let sub = format!("{label}/{name}");
        let path = e.path();
        if path.is_dir() {
            walk(&path, &sub, f);
        } else if let Ok(bytes) = fs::read(&path) {
            f(&sub, &name, &bytes);
        }
    }
}

/// `KQInterior` sections in the world's `.kq` files: how many, whether a
/// `<shape>.dil` exists for each, and the alternate-map keys (rule 12).
fn building_keys(world: &str, vols: &[(String, VolArchive)], log: &mut Log) {
    let mut dil_names = BTreeSet::new();
    let mut kq_files = Vec::new();
    for (v, a) in vols {
        for e in a.entries() {
            let lower = e.name.to_ascii_lowercase();
            if lower.ends_with(".dil") {
                dil_names.insert(lower);
            } else if lower.ends_with(".kq") {
                kq_files.push((format!("vol:{world}/{v}/{}", e.name), a.unpack(e).map(|u| u.data)));
            }
        }
    }
    let (mut interiors, mut no_dil) = (0, Vec::new());
    let mut alternates = Vec::new();
    for (label, bytes) in kq_files {
        let Ok(bytes) = bytes else { continue };
        for s in kq_text::parse(&bytes) {
            if !s.get("classType").is_some_and(|c| c.eq_ignore_ascii_case("KQInterior")) {
                continue;
            }
            interiors += 1;
            let shape = s.get("shapeName").unwrap_or("").to_ascii_lowercase();
            let dil = shape.strip_suffix(".dig").map(|b| format!("{b}.dil"));
            if !dil.as_ref().is_some_and(|d| dil_names.contains(d)) {
                no_dil.push(format!("{label} [{}] shapeName={shape}", s.name));
            }
            let alt = s.get("althwLightmapName");
            let using = s.get("usingAltLighting");
            if alt.is_some() || using.is_some() {
                let present = alt.is_some_and(|a| dil_names.contains(&a.to_ascii_lowercase()));
                alternates.push(format!(
                    "{label} [{}] althwLightmapName={} (in archives: {present}) usingAltLighting={}",
                    s.name,
                    alt.unwrap_or("-"),
                    using.unwrap_or("-")
                ));
            }
        }
    }
    log.summary(format!("{world}: {interiors} KQInterior sections; without a <shape>.dil: {}", no_dil.len()));
    for n in &no_dil {
        log.summary(format!("  no .dil: {n}"));
    }
    for a in &alternates {
        log.summary(format!("  alternate: {a}"));
    }
}
