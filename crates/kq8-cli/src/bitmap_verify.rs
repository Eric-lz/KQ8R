//! `kq8 bitmap-verify`: parses every image and palette the game has, in
//! all archives and in the loose files under `GAME/` and `data/`, and logs
//! counts per variant plus every failure.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::{Bitmap, GameDir, Palette, VolArchive};

use crate::bitmap_png::chunks_after_head;
use crate::log::Log;

#[derive(Default)]
struct Tally {
    bitmaps_ok: usize,
    palettes_ok: usize,
    failures: usize,
    /// variant description -> count
    variants: BTreeMap<String, usize>,
    /// "head count vs real count" observations -> count
    chunk_counts: BTreeMap<String, usize>,
    /// unexpected chunk tags -> count
    other_chunks: BTreeMap<String, usize>,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("bitmap-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("bitmap-verify: game folder {}", game.root().display()));
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
            for entry in vol.entries() {
                if kind(&entry.name).is_none() {
                    continue;
                }
                let label = format!("vol:{world}/{volume}/{}", entry.name);
                match vol.unpack(entry) {
                    Ok(u) => check(&label, &entry.name, &u.data, &mut log, &mut t),
                    Err(e) => {
                        log.detail(format!("FAIL {label}: {e}"));
                        t.failures += 1;
                    }
                }
            }
        }
    }

    // Loose files the game reads directly (not the installer's unpacked
    // copies of the archives, which vol-verify already covers).
    for top in ["GAME", "data"] {
        if let Some(dir) = game.find(top) {
            walk(&dir, top, &mut log, &mut t);
        }
    }

    log.summary("");
    log.summary("variants:");
    for (k, n) in &t.variants {
        log.summary(format!("  {n:6}  {k}"));
    }
    log.summary("chunk count in 'head' vs chunks actually following it:");
    for (k, n) in &t.chunk_counts {
        log.summary(format!("  {n:6}  {k}"));
    }
    if !t.other_chunks.is_empty() {
        log.summary("other chunk tags seen:");
        for (k, n) in &t.other_chunks {
            log.summary(format!("  {n:6}  {k}"));
        }
    }
    log.summary(format!(
        "TOTAL bitmaps_ok={} palettes_ok={} failures={} time_ms={}",
        t.bitmaps_ok,
        t.palettes_ok,
        t.failures,
        started.elapsed().as_millis()
    ));
    let ok = t.failures == 0 && t.bitmaps_ok > 0;
    log.summary(if ok { "RESULT: PASS" } else { "RESULT: FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

enum Kind {
    Bitmap,
    Palette,
}

fn kind(name: &str) -> Option<Kind> {
    let ext = name.rsplit_once('.')?.1.to_ascii_lowercase();
    match ext.as_str() {
        "bmp" | "pbm" | "dib" => Some(Kind::Bitmap),
        "ppl" | "pal" => Some(Kind::Palette),
        _ => None,
    }
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
        } else if kind(&name).is_some() {
            match fs::read(&path) {
                Ok(bytes) => check(&sub, &name, &bytes, log, t),
                Err(err) => {
                    log.detail(format!("FAIL {sub}: {err}"));
                    t.failures += 1;
                }
            }
        }
    }
}

fn check(label: &str, name: &str, bytes: &[u8], log: &mut Log, t: &mut Tally) {
    let ext = name.rsplit_once('.').map(|(_, e)| e.to_ascii_lowercase()).unwrap_or_default();
    match kind(name) {
        Some(Kind::Bitmap) => match Bitmap::parse(bytes) {
            Ok(b) => {
                t.bitmaps_ok += 1;
                *t.variants
                    .entry(format!(
                        ".{ext} {:?} {}-bit mipmaps={} flags=0x{:X}{}",
                        b.format,
                        b.bit_count,
                        b.mip_count,
                        b.flags,
                        if b.embedded_palette.is_some() { " +palette" } else { "" }
                    ))
                    .or_default() += 1;
                if let Some(n) = b.declared_chunks {
                    let real = chunks_after_head(&b.chunk_tags);
                    *t.chunk_counts
                        .entry(format!("bitmap head={n} real={real}"))
                        .or_default() += 1;
                }
                for (tag, _) in &b.other_chunks {
                    *t.other_chunks.entry(format!("bitmap '{tag}'")).or_default() += 1;
                }
                log.detail(format!(
                    "ok {label}: {:?} {}x{} {}-bit mips={} flags=0x{:X} chunks={:?} padding={}",
                    b.format, b.width, b.height, b.bit_count, b.mip_count, b.flags, b.chunk_tags,
                    b.data_padding
                ));
            }
            Err(e) => {
                t.failures += 1;
                log.summary(format!("FAIL {label}: {e}"));
            }
        },
        Some(Kind::Palette) => match Palette::parse(bytes) {
            Ok(p) => {
                t.palettes_ok += 1;
                *t.variants
                    .entry(format!(".{ext} {:?} entries={}", p.format, p.entry_count))
                    .or_default() += 1;
                if let Some(n) = p.declared_chunks {
                    let real = chunks_after_head(&p.chunk_tags);
                    *t.chunk_counts
                        .entry(format!("palette head={n} real={real}"))
                        .or_default() += 1;
                }
                log.detail(format!(
                    "ok {label}: {:?} entries={} chunks={:?}",
                    p.format, p.entry_count, p.chunk_tags
                ));
            }
            Err(e) => {
                t.failures += 1;
                log.summary(format!("FAIL {label}: {e}"));
            }
        },
        None => {}
    }
}
