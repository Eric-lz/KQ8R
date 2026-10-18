//! `kq8 font-verify` (M4 step 4b-1): reads every font (`.pft`) in the
//! install (not the copies under `MASK/data/`) and logs, per font, its
//! settings, the characters it covers, the extra chunks of its bitmap
//! array, the pixel values its glyphs use, and any glyph whose rectangle
//! doesn't fit its picture. Each font also gets a sheet of its glyphs,
//! characters 32-255 in rows of 16, in `kq8-artifacts/png/fonts/`
//! (pixel value 0 dark blue, other values grey = the value itself).

use std::collections::BTreeMap;
use std::fs::{self, File};
use std::io::BufWriter;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kq8_data::font::Font;
use kq8_data::GameDir;

use crate::log::{Log, ARTIFACTS_DIR};

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("font-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("log file: {}", log.path().display()));
    let ok = verify(game, &mut log);
    log.summary(if ok { "RESULT: PASS" } else { "RESULT: FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Every `.pft` under `dir`, skipping `data` and `save` folders.
fn fonts(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(rd) = fs::read_dir(dir) else { return };
    let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
    entries.sort();
    for p in entries {
        let name = p.file_name().and_then(|n| n.to_str()).unwrap_or("").to_ascii_lowercase();
        if p.is_dir() {
            if name != "data" && name != "save" {
                fonts(&p, out);
            }
        } else if name.ends_with(".pft") {
            out.push(p);
        }
    }
}

fn verify(game: &GameDir, log: &mut Log) -> bool {
    let mut files = Vec::new();
    fonts(game.root(), &mut files);
    let sheets = Path::new(ARTIFACTS_DIR).join("png").join("fonts");
    let _ = fs::create_dir_all(&sheets);
    let (mut read, mut failed) = (0, 0);
    // Identical files (the worlds' copies) are summarised once.
    let mut by_content: BTreeMap<Vec<u8>, Vec<String>> = BTreeMap::new();
    for path in &files {
        let rel = path.strip_prefix(game.root()).unwrap_or(path).to_string_lossy().replace('\\', "/");
        match fs::read(path) {
            Ok(bytes) => by_content.entry(bytes).or_default().push(rel),
            Err(e) => {
                log.summary(format!("{rel}: ERROR cannot read: {e}"));
                failed += 1;
            }
        }
    }
    log.summary(format!("{} font files, {} different contents", files.len(), by_content.len()));
    for (bytes, names) in &by_content {
        let first = &names[0];
        let same = if names.len() > 1 { format!(" (same bytes: {} files)", names.len()) } else { String::new() };
        match Font::parse(bytes) {
            Ok(f) => {
                read += names.len();
                let bad: Vec<usize> = f.glyphs.iter().enumerate().filter(|(_, g)| f.glyph_pixels(g).is_none()).map(|(k, _)| k).collect();
                let mut values: BTreeMap<u8, usize> = BTreeMap::new();
                for g in &f.glyphs {
                    for v in f.glyph_pixels(g).unwrap_or_default() {
                        *values.entry(v).or_default() += 1;
                    }
                }
                let widths = f.glyphs.iter().map(|g| g.width).fold((u8::MAX, 0), |(lo, hi), w| (lo.min(w), hi.max(w)));
                let heights = f.glyphs.iter().map(|g| g.height).fold((u8::MAX, 0), |(lo, hi), h| (lo.min(h), hi.max(h)));
                let shifts = f.glyphs.iter().map(|g| g.baseline_shift).fold((i8::MAX, i8::MIN), |(lo, hi), s| (lo.min(s), hi.max(s)));
                let tops: BTreeMap<u8, usize> = f.glyphs.iter().fold(BTreeMap::new(), |mut m, g| {
                    *m.entry(g.top).or_default() += 1;
                    m
                });
                let sizes: BTreeMap<(u32, u32), usize> = f.bitmaps.iter().fold(BTreeMap::new(), |mut m, b| {
                    *m.entry((b.width, b.height)).or_default() += 1;
                    m
                });
                let ok = bad.is_empty() && f.trailing == 0 && f.declared_bitmaps as usize == f.bitmaps.len();
                if !ok {
                    failed += 1;
                }
                log.summary(format!(
                    "{first}{same}: {} v{}, flags 0x{:X}, text flags 0x{:X}, {} glyphs, {} characters 32-255 covered, height {}, width {}, baseline {}, colours text 0x{:X} back 0x{:X}, scale {}x{}, spacing {}; {} bitmaps (head says {}), array chunks {:?}, palette {}, trailing {} bytes; glyph widths {}..{}, heights {}..{}, baseline shifts {}..{}; glyphs not fitting their picture: {:?}",
                    if ok { "OK" } else { "PROBLEM" },
                    f.version,
                    f.font_flags,
                    f.text_flags,
                    f.glyphs.len(),
                    f.covered(),
                    f.char_height,
                    f.char_width,
                    f.baseline,
                    f.text_color,
                    f.back_color,
                    f.h_scale,
                    f.v_scale,
                    f.char_spacing,
                    f.bitmaps.len(),
                    f.declared_bitmaps,
                    f.array_chunks,
                    if f.palette.is_some() { "yes" } else { "no" },
                    f.trailing,
                    widths.0,
                    widths.1,
                    heights.0,
                    heights.1,
                    shifts.0,
                    shifts.1,
                    bad
                ));
                log.detail(format!("  pixel values (value: count): {values:?}"));
                log.detail(format!("  glyph tops in their pictures (top: count): {tops:?}; picture sizes (w, h): count: {sizes:?}"));
                for n in &names[1..] {
                    log.detail(format!("  also {n}"));
                }
                let png = sheets.join(format!("{}.png", first.replace('/', "_")));
                match sheet(&f, &png) {
                    Ok(()) => log.detail(format!("  sheet: {}", png.display())),
                    Err(e) => log.summary(format!("  sheet {}: ERROR {e}", png.display())),
                }
            }
            Err(e) => {
                failed += names.len();
                log.summary(format!("{first}{same}: ERROR {e}"));
            }
        }
    }
    log.summary(format!("read {read} of {} font files; {failed} with problems", files.len()));
    failed == 0
}

/// Characters 32-255 in rows of 16, each glyph at the top-left of its
/// cell (the baseline shift is not applied: its meaning is not checked).
fn sheet(f: &Font, path: &Path) -> Result<(), String> {
    let cw = f.glyphs.iter().map(|g| g.width as usize).max().unwrap_or(1) + 2;
    let ch = f.glyphs.iter().map(|g| g.height as usize).max().unwrap_or(1) + 2;
    let (w, h) = (16 * cw, 14 * ch);
    let mut rgb = vec![0u8; w * h * 3];
    for p in rgb.chunks_mut(3) {
        p.copy_from_slice(&[0, 0, 60]);
    }
    for c in 32u8..=255 {
        let Some(g) = f.glyph(c) else { continue };
        let Some(px) = f.glyph_pixels(g) else { continue };
        let k = (c - 32) as usize;
        let (x0, y0) = ((k % 16) * cw + 1, (k / 16) * ch + 1);
        for y in 0..g.height as usize {
            for x in 0..g.width as usize {
                let v = px[y * g.width as usize + x];
                if v != 0 {
                    let at = ((y0 + y) * w + x0 + x) * 3;
                    rgb[at..at + 3].copy_from_slice(&[v, v, v]);
                }
            }
        }
    }
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(BufWriter::new(file), w as u32, h as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    let mut wr = enc.write_header().map_err(|e| e.to_string())?;
    wr.write_image_data(&rgb).map_err(|e| e.to_string())
}
