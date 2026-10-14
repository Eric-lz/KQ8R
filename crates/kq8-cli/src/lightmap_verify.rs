//! `kq8 lightmap-verify`: reads the light and shadow maps of every world
//! (the four names in `data/<World>/KQ/Terrain/TERRAIN.KQ`) and logs what
//! the behaviour note `docs/behaviour/terrain-lighting.md` says to check:
//! count, side, spacing, a histogram of each channel, how the light and
//! shadow maps of a pair differ, and how the hardware and software maps
//! compare. Also which map the game shows (`Options.cs`).

use std::collections::BTreeMap;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::game_dir::WORLDS;
use kq8_data::kq_text;
use kq8_data::lightmap::{self, Channels, Kind, LightMap};
use kq8_data::{GameDir, TerrainSettings};

use crate::log::Log;
use crate::terrain_verify::{load, open_world};

const CHANNELS: [&str; 4] = ["brightness", "red", "green", "blue"];

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("lightmap-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("lightmap-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));

    // Rule 2: which map of a pair is shown.
    let dynamic = match game.find("Options.cs").map(std::fs::read) {
        Some(Ok(bytes)) => {
            let v = lightmap::use_dynamic_lighting(&bytes);
            log.summary(format!(
                "Options.cs: useDynamicLighting {}",
                v.map_or("not set (game keeps 0)".into(), |b| u8::from(b).to_string())
            ));
            v.unwrap_or(false)
        }
        _ => {
            log.summary("Options.cs: not found; useDynamicLighting stays 0");
            false
        }
    };
    let shown = Kind::for_dynamic_lighting(dynamic);
    log.summary(format!("shown map of each pair: {shown:?} (rule 2)"));

    let (mut worlds_with_maps, mut files_ok, mut failures) = (0, 0, 0);
    for world in WORLDS {
        log.summary("");
        match check(game, world, shown, &mut log) {
            Ok((with_maps, ok)) => {
                worlds_with_maps += usize::from(with_maps);
                files_ok += ok;
            }
            Err(e) => {
                failures += 1;
                log.summary(format!("FAIL {world}: {e}"));
            }
        }
    }
    log.summary("");
    log.summary(format!(
        "TOTAL worlds_with_maps={worlds_with_maps} files_read={files_ok} failures={failures} time_ms={}",
        started.elapsed().as_millis()
    ));
    let pass = failures == 0 && files_ok > 0;
    log.summary(if pass { "RESULT: PASS" } else { "RESULT: FAIL" });
    if pass {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// One world: (has any map name, files read). An error is a named file
/// that is missing or can't be read.
fn check(game: &GameDir, world: &str, shown: Kind, log: &mut Log) -> Result<(bool, usize), String> {
    let rel = format!("data/{world}/KQ/Terrain/TERRAIN.KQ");
    let path = game.find(&rel).ok_or(format!("{rel} not found"))?;
    let bytes = std::fs::read(&path).map_err(|e| format!("{rel}: {e}"))?;
    let sections = kq_text::parse(&bytes);
    let section = sections
        .iter()
        .find(|s| s.get("classType").is_some_and(|c| c.eq_ignore_ascii_case("KQTerrain")))
        .ok_or(format!("{rel}: no section with classType=KQTerrain"))?;
    let settings = TerrainSettings::from_section(section)?;
    let side = lightmap::side_for(settings.light_scale);
    let full = LightMap::full(&settings);
    log.summary(format!(
        "{world}: lightScale={} groundScale={} -> side {side} points ({} values, file {} bytes), spacing {} units",
        settings.light_scale,
        settings.ground_scale,
        side * side,
        4 + 2 * side * side,
        full.spacing
    ));

    let vols = open_world(game, world, log);
    // (hardware, kind) -> map, for the comparisons below.
    let mut maps: BTreeMap<(bool, u8), (String, LightMap)> = BTreeMap::new();
    let mut any_name = false;
    for hardware in [true, false] {
        for kind in [Kind::Light, Kind::Shadow] {
            let key = match (hardware, kind) {
                (true, Kind::Light) => "lightmapName",
                (true, Kind::Shadow) => "shadowmapName",
                (false, Kind::Light) => "SWlightmapName",
                (false, Kind::Shadow) => "SWshadowmapName",
            };
            let Some(name) = lightmap::file_name(&settings, hardware, kind) else {
                log.summary(format!("  {key}: empty -> full brightness, every channel 15 (rule 4)"));
                continue;
            };
            any_name = true;
            let (at, bytes) = load(game, world, &vols, name)?;
            let map = LightMap::parse(&bytes, &settings).map_err(|e| format!("{key}={name} ({at}): {e}"))?;
            let count = u32::from_le_bytes(bytes[..4].try_into().unwrap());
            let shown_mark = if hardware && kind == shown { "  <- shown (hardware)" } else { "" };
            log.summary(format!(
                "  {key}={name}: {at} {} bytes, count {count} = {side} x {side}{shown_mark}",
                bytes.len()
            ));
            let hist = map.histograms();
            let grey = map.values().iter().filter(|&&v| Channels::of(v).is_grey()).count();
            for (k, h) in hist.iter().enumerate() {
                log.summary(format!("    {:<10} {}", CHANNELS[k], histogram(h)));
            }
            log.summary(format!("    grey (red = green = blue): {grey} of {} points", map.values().len()));
            maps.insert((hardware, kind as u8), (name.to_string(), map));
        }
    }

    // Light vs shadow map of each pair (rule 3).
    for hardware in [true, false] {
        let pair = (maps.get(&(hardware, Kind::Light as u8)), maps.get(&(hardware, Kind::Shadow as u8)));
        if let (Some((ln, l)), Some((sn, s))) = pair {
            log.summary(format!("  {} light vs shadow ({ln} vs {sn}): {}", label(hardware), compare(l, s)));
        }
    }
    // Hardware vs software, same kind (rule 9: 7 / 4 vs 15 / 9).
    for kind in [Kind::Light, Kind::Shadow] {
        let pair = (maps.get(&(true, kind as u8)), maps.get(&(false, kind as u8)));
        if let (Some((hn, h)), Some((sn, s))) = pair {
            let mut pairs: BTreeMap<(u8, u8), usize> = BTreeMap::new();
            for (&a, &b) in h.values().iter().zip(s.values()) {
                *pairs.entry((Channels::of(a).red, Channels::of(b).red)).or_default() += 1;
            }
            let mut list: Vec<_> = pairs.into_iter().collect();
            list.sort_by_key(|(_, n)| std::cmp::Reverse(*n));
            let text: Vec<String> = list.iter().map(|((a, b), n)| format!("{a}->{b} x{n}")).collect();
            log.summary(format!(
                "  {kind:?} map, hardware vs software red ({hn} vs {sn}), value pairs: {}",
                text.join(", ")
            ));
        }
    }
    Ok((any_name, maps.len()))
}

fn label(hardware: bool) -> &'static str {
    if hardware {
        "hardware"
    } else {
        "software"
    }
}

/// Non-zero entries as `value xcount`.
fn histogram(h: &[usize; 16]) -> String {
    let parts: Vec<String> = h.iter().enumerate().filter(|(_, n)| **n > 0).map(|(v, n)| format!("{v} x{n}")).collect();
    parts.join(", ")
}

/// Points whose value differs, and the largest difference in any channel.
fn compare(a: &LightMap, b: &LightMap) -> String {
    let (mut differ, mut largest) = (0usize, 0i32);
    let mut by_step: BTreeMap<i32, usize> = BTreeMap::new();
    for (&x, &y) in a.values().iter().zip(b.values()) {
        if x == y {
            continue;
        }
        differ += 1;
        let (cx, cy) = (Channels::of(x), Channels::of(y));
        let d = [
            cx.brightness as i32 - cy.brightness as i32,
            cx.red as i32 - cy.red as i32,
            cx.green as i32 - cy.green as i32,
            cx.blue as i32 - cy.blue as i32,
        ];
        let m = d.iter().map(|v| v.abs()).max().unwrap();
        largest = largest.max(m);
        *by_step.entry(d[1]).or_default() += 1;
    }
    let steps: Vec<String> = by_step.iter().map(|(d, n)| format!("{d:+} x{n}")).collect();
    format!(
        "{differ} of {} points differ, largest step in any channel {largest}; red (light minus shadow): {}",
        a.values().len(),
        if steps.is_empty() { "-".into() } else { steps.join(", ") }
    )
}
