//! `kq8 vol-verify`: proves the VOL reader works on the real game data.
//!
//! For every `data/<World>/<volume>.vol` it decompresses every entry in
//! memory and compares it byte-for-byte with the copy the installer
//! already unpacked to `<world>/<volume>/<name>`. Nothing is written to
//! the game folder.

use std::collections::HashSet;
use std::fs;
use std::process::ExitCode;
use std::time::Instant;

use kq8_data::game_dir::{find_ci, VOLUMES, WORLDS};
use kq8_data::{GameDir, VolArchive};

use crate::log::Log;

#[derive(Default)]
struct Totals {
    archives: usize,
    archives_failed: usize,
    archive_bytes: usize,
    entries: usize,
    unpacked_bytes: usize,
    matched: usize,
    mismatched: usize,
    no_reference: usize,
    decompress_errors: usize,
    // compressed bytes read != space the entry occupies
    size_slack: usize,
    // files in the unpacked folder with no matching VOL entry
    extra_reference_files: usize,
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("vol-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    let started = Instant::now();
    log.summary(format!("vol-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));

    let mut t = Totals::default();
    for world in WORLDS {
        for volume in VOLUMES {
            verify_archive(game, world, volume, &mut log, &mut t);
        }
    }

    log.summary("");
    log.summary(format!(
        "TOTAL archives={} (failed={}) archive_bytes={} entries={} unpacked_bytes={}",
        t.archives, t.archives_failed, t.archive_bytes, t.entries, t.unpacked_bytes
    ));
    log.summary(format!(
        "TOTAL matched={} mismatched={} no_reference={} decompress_errors={} size_slack={} extra_reference_files={}",
        t.matched, t.mismatched, t.no_reference, t.decompress_errors, t.size_slack,
        t.extra_reference_files
    ));
    log.summary(format!("TOTAL time_ms={}", started.elapsed().as_millis()));

    let ok = t.archives_failed == 0
        && t.mismatched == 0
        && t.decompress_errors == 0
        && t.archives == WORLDS.len() * VOLUMES.len();
    log.summary(if ok { "RESULT: PASS" } else { "RESULT: FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

fn verify_archive(game: &GameDir, world: &str, volume: &str, log: &mut Log, t: &mut Totals) {
    let label = format!("{world}/{volume}.vol");
    let Some(path) = game.volume_path(world, volume) else {
        log.summary(format!("{label}: MISSING archive"));
        t.archives_failed += 1;
        return;
    };
    let started = Instant::now();
    let vol = match VolArchive::open(&path) {
        Ok(v) => v,
        Err(e) => {
            log.summary(format!("{label}: FAILED to read: {e}"));
            t.archives_failed += 1;
            return;
        }
    };
    t.archives += 1;
    t.archive_bytes += vol.len();

    // Reference copies unpacked by the installer, e.g. MASK/daventry/kq/.
    let ref_dir = find_ci(game.root(), &format!("{world}/{volume}"));
    let mut seen = HashSet::new();

    let (mut matched, mut mismatched, mut no_ref, mut errors, mut slack) = (0, 0, 0, 0, 0);
    let mut unpacked_bytes = 0;
    for entry in vol.entries() {
        let unpacked = match vol.unpack(entry) {
            Ok(u) => u,
            Err(e) => {
                log.detail(format!("  ERROR {e}"));
                errors += 1;
                continue;
            }
        };
        unpacked_bytes += unpacked.data.len();
        if unpacked.compressed_used != entry.compressed_size() {
            slack += 1;
            log.detail(format!(
                "  SLACK {} used={} stored={}",
                entry.name,
                unpacked.compressed_used,
                entry.compressed_size()
            ));
        }
        seen.insert(entry.name.to_ascii_lowercase());

        let reference = ref_dir
            .as_ref()
            .and_then(|d| find_ci(d, &entry.name))
            .and_then(|p| fs::read(p).ok());
        match reference {
            None => {
                no_ref += 1;
                log.detail(format!("  NOREF {} size={}", entry.name, unpacked.data.len()));
            }
            Some(r) if r == unpacked.data => matched += 1,
            Some(r) => {
                mismatched += 1;
                let first_diff = r
                    .iter()
                    .zip(&unpacked.data)
                    .position(|(a, b)| a != b)
                    .unwrap_or(r.len().min(unpacked.data.len()));
                log.detail(format!(
                    "  DIFF {} ours={} reference={} first_diff_at={}",
                    entry.name,
                    unpacked.data.len(),
                    r.len(),
                    first_diff
                ));
            }
        }
    }

    // Top-level files in the reference folder that the VOL does not have.
    // (Sub-folders such as kq/Light come from loose files, not the VOL.)
    let mut extra = 0;
    if let Some(dir) = &ref_dir {
        if let Ok(rd) = fs::read_dir(dir) {
            for e in rd.flatten() {
                let is_file = e.file_type().map(|t| t.is_file()).unwrap_or(false);
                let name = e.file_name().to_string_lossy().to_ascii_lowercase();
                if is_file && !seen.contains(&name) {
                    extra += 1;
                    log.detail(format!("  EXTRA {}", e.file_name().to_string_lossy()));
                }
            }
        }
    }

    log.summary(format!(
        "{label}: bytes={} entries={} unpacked_bytes={} matched={} mismatched={} no_reference={} errors={} slack={} extra={} time_ms={}",
        vol.len(),
        vol.entries().len(),
        unpacked_bytes,
        matched,
        mismatched,
        no_ref,
        errors,
        slack,
        extra,
        started.elapsed().as_millis()
    ));

    t.entries += vol.entries().len();
    t.unpacked_bytes += unpacked_bytes;
    t.matched += matched;
    t.mismatched += mismatched;
    t.no_reference += no_ref;
    t.decompress_errors += errors;
    t.size_slack += slack;
    t.extra_reference_files += extra;
}
