//! `kq8 terrain-verify`: reads the outdoor ground of every world
//! (`data/<World>/KQ/Terrain/*.kq` and the files they name) and logs
//! sizes, value ranges and cross-checks. Then it compares the ground
//! height with the height of every placed object in the world, as the
//! behaviour note `docs/behaviour/terrain.md` suggests ("How to check").

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::kq_text;
use kq8_data::terrain::{parse_directions, Grid};
use kq8_data::{Bitmap, GameDir, MaterialKind, MaterialList, Terrain, TerrainSettings, VolArchive};

use crate::log::Log;

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("terrain-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("terrain-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let (mut ok, mut failures) = (0, 0);
    for world in WORLDS {
        let Some(dir) = game.find(&format!("data/{world}/KQ/Terrain")) else {
            log.summary(format!("{world}: no KQ/Terrain folder"));
            continue;
        };
        let mut files: Vec<_> = std::fs::read_dir(&dir)
            .map(|rd| rd.flatten().map(|e| e.path()).collect())
            .unwrap_or_default();
        files.retain(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("kq")));
        files.sort();
        let vols = open_world(game, world, &mut log);
        for path in files {
            let label = format!("{world}/{}", path.file_name().unwrap().to_string_lossy());
            log.summary("");
            match check(game, world, &vols, &path, &label, &mut log) {
                Ok(()) => ok += 1,
                Err(e) => {
                    failures += 1;
                    log.summary(format!("FAIL {label}: {e}"));
                }
            }
        }
    }
    log.summary("");
    log.summary(format!("TOTAL terrains_ok={ok} failures={failures} time_ms={}", started.elapsed().as_millis()));
    let pass = failures == 0 && ok > 0;
    log.summary(if pass { "RESULT: PASS" } else { "RESULT: FAIL" });
    if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

pub(crate) fn open_world(game: &GameDir, world: &str, log: &mut Log) -> Vec<(String, VolArchive)> {
    let mut out = Vec::new();
    for v in VOLUMES {
        if let Some(path) = game.volume_path(world, v) {
            match VolArchive::open(&path) {
                Ok(a) => out.push((v.to_string(), a)),
                Err(e) => log.summary(format!("{world}/{v}.vol: {e}")),
            }
        }
    }
    out
}

/// Finds a file by its last path part: first next to the terrain file,
/// then in the world's archives. Returns (where, bytes).
pub(crate) fn load(game: &GameDir, world: &str, vols: &[(String, VolArchive)], name: &str) -> Result<(String, Vec<u8>), String> {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    for rel in [format!("data/{world}/KQ/Terrain/{base}"), format!("data/{world}/KQ/{name}")] {
        if let Some(p) = game.find(&rel).filter(|p| p.is_file()) {
            return std::fs::read(&p).map(|b| (rel, b)).map_err(|e| e.to_string());
        }
    }
    for (v, a) in vols {
        if let Some(e) = a.find(base) {
            return a
                .unpack(e)
                .map(|u| (format!("vol:{world}/{v}/{}", e.name), u.data))
                .map_err(|e| e.to_string());
        }
    }
    Err(format!("'{name}' not found next to the terrain file or in {world}'s archives"))
}

fn grid(game: &GameDir, world: &str, vols: &[(String, VolArchive)], name: &str, log: &mut Log) -> Result<Grid, String> {
    let (at, bytes) = load(game, world, vols, name)?;
    let bmp = Bitmap::parse(&bytes).map_err(|e| format!("{at}: {e}"))?;
    let grey = bmp
        .embedded_palette
        .as_ref()
        .map(|p| p.iter().enumerate().all(|(i, c)| *c == [i as u8; 3]));
    let g = Grid::from_bitmap(&bmp).map_err(|e| format!("{at}: {e}"))?;
    let (lo, hi) = (g.values().iter().min().unwrap(), g.values().iter().max().unwrap());
    let distinct = {
        let mut seen = [false; 256];
        g.values().iter().for_each(|&v| seen[v as usize] = true);
        seen.iter().filter(|&&s| s).count()
    };
    log.summary(format!(
        "  {name}: {at} {:?} {}x{} values {lo}..{hi} distinct={distinct} grey_ramp_palette={}",
        bmp.format,
        g.width,
        g.height,
        grey.map_or("none".into(), |b| b.to_string())
    ));
    Ok(g)
}

fn check(
    game: &GameDir,
    world: &str,
    vols: &[(String, VolArchive)],
    path: &std::path::Path,
    label: &str,
    log: &mut Log,
) -> Result<(), String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let sections = kq_text::parse(&bytes);
    let section = sections
        .iter()
        .find(|s| s.get("classType").is_some_and(|c| c.eq_ignore_ascii_case("KQTerrain")))
        .ok_or("no section with classType=KQTerrain")?;
    let settings = TerrainSettings::from_section(section)?;
    log.summary(format!(
        "{label}: [{}] groundScale={} (square {} units) heightBMPScale={} heightBMP={} floorBMP={:?} materialBMP={} materialList={} materialDirection={:?}",
        section.name,
        settings.ground_scale,
        settings.square_size(),
        settings.height_scale,
        settings.height_image,
        settings.floor_image,
        settings.material_image,
        settings.material_list,
        settings.material_direction
    ));
    for (k, v) in &settings.all {
        log.detail(format!("  setting {k}={v}"));
    }

    let heights = grid(game, world, vols, &settings.height_image, log)?;
    if let Some(floor) = &settings.floor_image {
        match grid(game, world, vols, floor, log) {
            Ok(f) if f.width == heights.width && f.height == heights.height => {
                let differ: Vec<i32> = f
                    .values()
                    .iter()
                    .zip(heights.values())
                    .filter(|(a, b)| a != b)
                    .map(|(a, b)| *a as i32 - *b as i32)
                    .collect();
                log.summary(format!(
                    "  floor vs height image: {} of {} points differ (floor minus height: min {:?} max {:?})",
                    differ.len(),
                    f.values().len(),
                    differ.iter().min(),
                    differ.iter().max()
                ));
            }
            Ok(f) => log.summary(format!("  floor image size {}x{} differs from height image", f.width, f.height)),
            Err(e) => log.summary(format!("  floor image: {e}")),
        }
    }
    let terrain = Terrain::new(settings.clone(), heights)?;
    let (nx, ny) = terrain.squares();
    let size = settings.square_size();
    log.summary(format!(
        "  grid: {nx}x{ny} squares, ground covers 0..{} x 0..{} units, heights {}..{} units",
        nx as f32 * size,
        ny as f32 * size,
        terrain.heights.values().iter().min().copied().unwrap_or(0) as f32 * settings.height_scale,
        terrain.heights.values().iter().max().copied().unwrap_or(0) as f32 * settings.height_scale
    ));

    // Materials: list, image, direction table. A list missing from the
    // world is a fact about the data (the temple worlds' terrain2.kq name
    // Ice World's ice.dml), not a reader failure: note it and go on.
    let (list_at, list_bytes) = match load(game, world, vols, &settings.material_list) {
        Ok(x) => x,
        Err(e) => {
            log.summary(format!("  NOTE material list: {e}; material checks skipped"));
            heights_vs_objects(world, vols, &terrain, log);
            return Ok(());
        }
    };
    let list = MaterialList::parse(&list_bytes).map_err(|e| format!("{list_at}: {e}"))?;
    let missing_textures = list
        .materials
        .iter()
        .filter(|m| m.kind() == MaterialKind::Texture && !m.texture.is_empty())
        .filter(|m| !vols.iter().any(|(_, a)| a.find(&m.texture).is_some()))
        .count();
    log.summary(format!(
        "  material list {list_at}: v{} materials={} textures_not_in_world_archives={missing_textures}",
        list.version,
        list.materials.len()
    ));
    let materials = grid(game, world, vols, &settings.material_image, log)?;
    if (materials.width, materials.height) != (nx, ny) {
        log.summary(format!(
            "  NOTE material image {}x{} is not one value per square ({nx}x{ny})",
            materials.width, materials.height
        ));
    }
    let mut use_count: BTreeMap<u8, usize> = BTreeMap::new();
    materials.values().iter().for_each(|&v| *use_count.entry(v).or_default() += 1);
    let past_list = materials.values().iter().filter(|&&v| v as usize >= list.materials.len()).count();
    let mut top: Vec<_> = use_count.iter().map(|(v, n)| (*n, *v)).collect();
    top.sort_unstable_by(|a, b| b.cmp(a));
    let describe = |v: u8| {
        list.materials
            .get(v as usize)
            .map_or("(not in list)".to_string(), |m| format!("{} flags=0x{:X}", m.texture, m.flags))
    };
    log.summary(format!(
        "  material image: {} different values, {past_list} squares use a number past the list; most used: {}",
        use_count.len(),
        top.iter().take(5).map(|(n, v)| format!("{v}={} x{n}", describe(*v))).collect::<Vec<_>>().join(", ")
    ));
    if let Some(dir_name) = &settings.material_direction {
        match load(game, world, vols, dir_name).and_then(|(at, b)| parse_directions(&b).map(|r| (at, r))) {
            Ok((at, recs)) => {
                let same_order = recs.iter().all(|r| r.index as usize == recs.iter().position(|x| std::ptr::eq(x, r)).unwrap());
                let names_match = recs
                    .iter()
                    .zip(&list.materials)
                    .filter(|(r, m)| r.texture.eq_ignore_ascii_case(&m.texture))
                    .count();
                let mut unknown: BTreeMap<u32, usize> = BTreeMap::new();
                let mut direction: BTreeMap<u32, usize> = BTreeMap::new();
                for r in &recs {
                    *unknown.entry(r.unknown).or_default() += 1;
                    *direction.entry(r.direction).or_default() += 1;
                }
                log.summary(format!(
                    "  direction table {at}: records={} index_equals_position={same_order} names_match_list={names_match}/{} always-0_field={unknown:?} direction_values={direction:?}",
                    recs.len(),
                    list.materials.len()
                ));
            }
            Err(e) => log.summary(format!("  direction table: {e}")),
        }
    }

    heights_vs_objects(world, vols, &terrain, log);
    Ok(())
}

/// Ground height at every placed object (non-zero position) in the
/// world's `.kq` files, compared with the object's own height.
fn heights_vs_objects(world: &str, vols: &[(String, VolArchive)], terrain: &Terrain, log: &mut Log) {
    let mut diffs: Vec<(f32, String)> = Vec::new();
    let (mut outside, mut placed) = (0, 0);
    for (_, a) in vols {
        for e in a.entries().iter().filter(|e| e.name.to_ascii_lowercase().ends_with(".kq")) {
            let Ok(u) = a.unpack(e) else { continue };
            for s in kq_text::parse(&u.data) {
                let num = |k| s.number(k).and_then(Result::ok);
                let (Some(x), Some(y), Some(z)) = (num("locX"), num("locY"), num("locZ")) else { continue };
                if x == 0.0 && y == 0.0 && z == 0.0 {
                    continue;
                }
                placed += 1;
                match terrain.height_at(x as f32, y as f32) {
                    None => outside += 1,
                    Some(g) => {
                        let d = z as f32 - g.height;
                        log.detail(format!(
                            "  object {}[{}] class={} at ({x}, {y}, {z}) ground={:.1} object_minus_ground={d:.1} keeponground={}",
                            e.name,
                            s.name,
                            s.get("classType").unwrap_or("?"),
                            g.height,
                            s.get("keeponground").unwrap_or("-")
                        ));
                        diffs.push((d, s.get("keeponground").unwrap_or("-").to_string()));
                    }
                }
            }
        }
    }
    let within = |t: f32| diffs.iter().filter(|(d, _)| d.abs() <= t).count();
    let mut abs: Vec<f32> = diffs.iter().map(|(d, _)| d.abs()).collect();
    abs.sort_by(f32::total_cmp);
    let below = diffs.iter().filter(|(d, _)| *d < -100.0).count();
    log.summary(format!(
        "  objects ({world}): placed={placed} outside_grid={outside} on_grid={} | object height minus ground: exactly_on_ground(0.5)={} within 5 units={} within 20={} within 100={} more_than_100_below_ground={below} median_abs={:.1}",
        diffs.len(),
        within(0.5),
        within(5.0),
        within(20.0),
        within(100.0),
        abs.get(abs.len() / 2).copied().unwrap_or(f32::NAN)
    ));
    let kog: Vec<String> = diffs.iter().filter(|(_, k)| k == "1").map(|(d, _)| format!("{d:.0}")).collect();
    log.summary(format!("  objects with keeponground=1: {} differences: [{}]", kog.len(), kog.join(", ")));
}
