//! `kq8 bitmap-png <file> [--palette <file>]`: converts one game image to
//! a PNG in `kq8-artifacts/png/` and logs what was found in it.

use std::fs::{self, File};
use std::io::BufWriter;
use std::path::Path;
use std::process::ExitCode;

use kq8_data::bitmap::{FLAG_DITHER, FLAG_TRANSLUCENT, FLAG_TRANSPARENT};
use kq8_data::palette::Rgb;
use kq8_data::{Bitmap, GameDir, Palette};

use crate::log::{Log, ARTIFACTS_DIR};
use crate::source;

pub fn run(game: &GameDir, image: &str, palette: Option<&str>) -> ExitCode {
    let mut log = match Log::create("bitmap-png") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    match convert(game, image, palette, &mut log) {
        Ok(()) => {
            log.summary("RESULT: PASS");
            ExitCode::SUCCESS
        }
        Err(e) => {
            log.summary(format!("error: {e}"));
            log.summary("RESULT: FAIL");
            ExitCode::FAILURE
        }
    }
}

fn convert(game: &GameDir, image: &str, palette: Option<&str>, log: &mut Log) -> Result<(), String> {
    log.summary(format!("log file: {}", log.path().display()));
    let bytes = source::load(game, image)?;
    // A bitmap array (`.pba`: cursors, buttons): its first bitmap.
    let bmp = if bytes.starts_with(b"PBMA") {
        let all = kq8_data::font::bitmap_array(&bytes).map_err(|e| format!("{image}: {e}"))?;
        log.summary(format!("bitmap array: {} bitmaps, writing the first", all.len()));
        all.into_iter().next().ok_or(format!("{image}: empty bitmap array"))?
    } else {
        Bitmap::parse(&bytes).map_err(|e| format!("{image}: {e}"))?
    };

    log.summary(format!("image: {image} ({} bytes)", bytes.len()));
    log.summary(format!(
        "  format={:?} size={}x{} bits={} mipmaps={} padding_bytes={}",
        bmp.format, bmp.width, bmp.height, bmp.bit_count, bmp.mip_count, bmp.data_padding
    ));
    log.summary(format!("  flags=0x{:X} ({})", bmp.flags, describe_flags(bmp.flags)));
    if let Some(n) = bmp.declared_chunks {
        log.summary(format!(
            "  chunks={:?} head_says_following={} actually_following={}",
            bmp.chunk_tags,
            n,
            chunks_after_head(&bmp.chunk_tags)
        ));
    }

    let colors: Option<[Rgb; 256]> = if bmp.bit_count != 8 {
        None
    } else if let Some(spec) = palette {
        let pal = Palette::parse(&source::load(game, spec)?).map_err(|e| format!("{spec}: {e}"))?;
        log.summary(format!(
            "palette: {spec} format={:?} entries={} head_says_following={:?} chunks={:?}",
            pal.format, pal.entry_count, pal.declared_chunks, pal.chunk_tags
        ));
        // As stored, entry 0 black (models-and-textures.md Q3, Q22).
        Some(pal.shown_colors())
    } else if let Some(embedded) = &bmp.embedded_palette {
        log.summary(format!("palette: embedded in image ({} entries)", embedded.len()));
        let mut c = [[0u8; 3]; 256];
        c.iter_mut().zip(embedded).for_each(|(d, s)| *d = *s);
        Some(kq8_data::palette::as_shown(c))
    } else {
        log.summary("palette: none given, writing greyscale (pass --palette <file>)");
        Some(std::array::from_fn(|i| [i as u8; 3]))
    };

    if let Some(h) = bmp.index_histogram() {
        let used = h.iter().filter(|&&n| n > 0).count();
        let low: u32 = h[..8].iter().sum();
        let high: u32 = h[248..].iter().sum();
        let (top_index, top_count) = h.iter().enumerate().max_by_key(|(_, n)| **n).unwrap();
        log.summary(format!(
            "  colours used={used}/256 pixels_in_0..7={low} pixels_in_248..255={high} most_common=index {top_index} ({top_count} px)"
        ));
    }

    if let (Some(c), 8) = (&colors, bmp.bit_count) {
        log.summary(format!(
            "  neighbour_colour_diff={:.1} (average RGB change between side-by-side pixels; low = smooth picture, high = palette probably wrong)",
            neighbour_diff(&bmp, c)
        ));
    }

    let out_dir = Path::new(ARTIFACTS_DIR).join("png");
    fs::create_dir_all(&out_dir).map_err(|e| e.to_string())?;
    let out = out_dir.join(format!("{}.png", source::stem(image)));
    write_png(&out, &bmp, colors.as_ref()).map_err(|e| format!("{}: {e}", out.display()))?;
    log.summary(format!(
        "wrote {} ({}x{}, level 0 only)",
        out.display(),
        bmp.width,
        bmp.height
    ));
    Ok(())
}

pub fn chunks_after_head(tags: &[String]) -> usize {
    tags.iter().position(|t| t == "head").map(|i| tags.len() - i - 1).unwrap_or(0)
}

/// Mean absolute RGB difference between horizontally adjacent pixels.
fn neighbour_diff(bmp: &Bitmap, colors: &[Rgb; 256]) -> f64 {
    let w = bmp.width as usize;
    let mut total = 0u64;
    let mut pairs = 0u64;
    for row in bmp.pixels.chunks_exact(w) {
        for pair in row.windows(2) {
            let (a, b) = (colors[pair[0] as usize], colors[pair[1] as usize]);
            total += (0..3).map(|i| a[i].abs_diff(b[i]) as u64).sum::<u64>();
            pairs += 1;
        }
    }
    if pairs == 0 {
        0.0
    } else {
        total as f64 / pairs as f64
    }
}

fn describe_flags(flags: u32) -> String {
    let mut parts = Vec::new();
    if flags & FLAG_TRANSPARENT != 0 {
        parts.push("transparent");
    }
    if flags & FLAG_DITHER != 0 {
        parts.push("dither");
    }
    if flags & FLAG_TRANSLUCENT != 0 {
        parts.push("translucent");
    }
    if flags & 0x08 != 0 {
        parts.push("owns-memory");
    }
    if flags & !0x0F != 0 {
        parts.push("unknown-bits");
    }
    if parts.is_empty() {
        "none".into()
    } else {
        parts.join(",")
    }
}

fn write_png(path: &Path, bmp: &Bitmap, colors: Option<&[Rgb; 256]>) -> Result<(), String> {
    let file = File::create(path).map_err(|e| e.to_string())?;
    let mut enc = png::Encoder::new(BufWriter::new(file), bmp.width, bmp.height);
    enc.set_depth(png::BitDepth::Eight);
    match colors {
        Some(c) => {
            enc.set_color(png::ColorType::Indexed);
            enc.set_palette(c.iter().flatten().copied().collect::<Vec<u8>>());
        }
        None => enc.set_color(png::ColorType::Rgb),
    }
    let mut w = enc.write_header().map_err(|e| e.to_string())?;
    w.write_image_data(&bmp.pixels).map_err(|e| e.to_string())
}
