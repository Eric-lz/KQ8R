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
