//! `kq8 aud-verify [--wav <file>]`: decodes every `.aud` file in the game
//! folder and every SOL file in its `English` folders (speech), and
//! compares the counts with `docs/behaviour/sound.md`. `--wav <file>` also
//! writes that one file, decoded, to `kq8-artifacts/wav/<name>.wav`.
//!
//! The note counts the copies under `MASK/data/` too (1,750 `.aud` =
//! 1,007 loose + 743 copies), so the checks do; loose-only numbers are
//! logged next to them.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use kq8_data::aud::{self, Kind, Sound, SOL_HEADER};
use kq8_data::GameDir;

use crate::log::{Log, ARTIFACTS_DIR};
use crate::source;

pub fn run(game: &GameDir, wav: Option<&str>) -> ExitCode {
    let mut log = match Log::create("aud-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("log file: {}", log.path().display()));
    let mut ok = verify(game, &mut log);
    if let Some(spec) = wav {
        match write_wav(game, spec, &mut log) {
            Ok(()) => {}
            Err(e) => {
                log.summary(format!("--wav {spec}: error: {e}"));
                ok = false;
            }
        }
    }
    log.summary(if ok { "RESULT: PASS" } else { "RESULT: FAIL" });
    if ok {
        ExitCode::SUCCESS
    } else {
        ExitCode::FAILURE
    }
}

/// Counts for one group of files (`.aud`, or speech).
#[derive(Default)]
struct Tally {
    files: usize,
    loose: usize,
    sol: usize,
    sol_loose: usize,
    wav: usize,
    refused: Vec<String>,
    /// SOL files only (the note's rate counts are of SOL files).
    rates: BTreeMap<u32, usize>,
    formats: BTreeMap<u8, usize>,
    stereo: usize,
    length_mismatch: usize,
    wraps: u64,
    files_with_wraps: usize,
    peak: u32,
    seconds: f64,
    seconds_loose: f64,
    shortest: Option<(usize, String)>,
    longest: Option<(f64, String)>,
}

impl Tally {
    fn add(&mut self, rel: &str, bytes: &[u8], loose: bool, log: &mut Log) {
        self.files += 1;
        self.loose += usize::from(loose);
        let s = match aud::parse(bytes) {
            Ok(s) => s,
            Err(e) => {
                log.detail(format!("{rel}: REFUSED: {e}"));
                self.refused.push(format!("{rel} ({e})"));
                return;
            }
        };
        let length = match s.kind {
            Kind::Sol { format, header_byte_1 } => {
                self.sol += 1;
                self.sol_loose += usize::from(loose);
                *self.rates.entry(s.rate).or_default() += 1;
                *self.formats.entry(format).or_default() += 1;
                let matches = SOL_HEADER as u64 + s.declared_len == bytes.len() as u64;
                self.length_mismatch += usize::from(!matches);
                format!(
                    "SOL format 0x{format:02X} byte1 0x{header_byte_1:02X}; header + data {} {} file {}",
                    SOL_HEADER as u64 + s.declared_len,
                    if matches { "=" } else { "!=" },
                    bytes.len()
                )
            }
            Kind::Wav { bits } => {
                self.wav += 1;
                log.summary(format!("{rel}: WAV, {} Hz, {} ch, {bits}-bit", s.rate, s.channels));
                format!("WAV {bits}-bit, data chunk {} of {} bytes present", s.present_len, s.declared_len)
            }
        };
        self.stereo += usize::from(s.channels == 2);
        self.wraps += s.wraps;
        self.files_with_wraps += usize::from(s.wraps > 0);
        self.peak = self.peak.max(s.peak);
        self.seconds += s.seconds();
        if loose {
            self.seconds_loose += s.seconds();
        }
        if self.shortest.as_ref().is_none_or(|(n, _)| s.frames() < *n) {
            self.shortest = Some((s.frames(), rel.to_string()));
        }
        if self.longest.as_ref().is_none_or(|(t, _)| s.seconds() > *t) {
            self.longest = Some((s.seconds(), rel.to_string()));
        }
        log.detail(format!(
            "{rel}: {length}; {} Hz, {} ch, {} samples, {:.3} s, peak {}, wraps {}",
            s.rate,
            s.channels,
            s.frames(),
            s.seconds(),
            s.peak,
            s.wraps
        ));
    }

    fn report(&self, what: &str, log: &mut Log) {
        log.summary(format!(
            "{what}: {} files ({} loose, {} copies under data/): {} SOL, {} WAV, {} refused",
            self.files,
            self.loose,
            self.files - self.loose,
            self.sol,
            self.wav,
            self.refused.len()
        ));
        for r in &self.refused {
            log.summary(format!("  refused: {r}"));
        }
        log.summary(format!("  SOL samples per second: {:?}", self.rates));
        let formats: Vec<String> = self.formats.iter().map(|(f, n)| format!("0x{f:02X}: {n}")).collect();
        log.summary(format!("  SOL format bytes: {}; stereo files {}", formats.join(", "), self.stereo));
        log.summary(format!("  SOL files where header + data length != file length: {}", self.length_mismatch));
        log.summary(format!(
            "  wrap-rounds: {} in {} files; largest peak {}",
            self.wraps, self.files_with_wraps, self.peak
        ));
        log.summary(format!(
            "  total length {} (loose only {})",
            hms(self.seconds),
            hms(self.seconds_loose)
        ));
        if let (Some((n, short)), Some((t, long))) = (&self.shortest, &self.longest) {
            log.summary(format!("  shortest {short} ({n} samples), longest {long} ({t:.1} s)"));
        }
    }
}

fn hms(seconds: f64) -> String {
    let s = seconds.round() as u64;
    format!("{} h {:02} min {:02} s", s / 3600, s / 60 % 60, s % 60)
}

fn verify(game: &GameDir, log: &mut Log) -> bool {
    let mut files = Vec::new();
    walk(game.root(), &mut files);
    files.sort();
    let (mut auds, mut speech) = (Tally::default(), Tally::default());
    let mut english_other = 0usize;
    let mut sounds: BTreeMap<String, Sound> = BTreeMap::new();
    log.detail("per file: path; kind, format byte, header byte 1, length check; rate, channels, samples per channel, seconds, peak, wrap-rounds");
    for path in &files {
        let rel = path.strip_prefix(game.root()).unwrap_or(path).to_string_lossy().replace('\\', "/");
        let lower = rel.to_ascii_lowercase();
        let is_aud = lower.ends_with(".aud");
        let in_english = lower.split('/').any(|p| p == "english");
        if !is_aud && !in_english {
            continue;
        }
        let bytes = match fs::read(path) {
            Ok(b) => b,
            Err(e) => {
                log.summary(format!("{rel}: cannot read: {e}"));
                continue;
            }
        };
        let loose = !lower.starts_with("data/");
        if is_aud {
            auds.add(&rel, &bytes, loose, log);
            // Loose files the "How to check" list names.
            let name = lower.rsplit('/').next().unwrap_or("");
            if loose && ["axedoor.aud", "forest.aud", "none.aud", "ambdead.aud"].contains(&name) {
                if let Ok(s) = aud::parse(&bytes) {
                    sounds.insert(lower.clone(), s);
                }
            }
        } else if aud::is_sol(&bytes) {
            speech.add(&rel, &bytes, loose, log);
        } else {
            english_other += 1;
        }
    }

    auds.report(".aud", log);
    speech.report("speech (SOL files in English folders)", log);
    log.summary(format!("  other files in English folders (not SOL, not counted): {english_other}"));

    log.summary("checks against docs/behaviour/sound.md:");
    let mut all = true;
    let mut check = |what: &str, got: String, want: &str| {
        let same = got == want;
        all &= same;
        log.summary(format!("  {} {what}: {got} (note: {want})", if same { "MATCHES" } else { "DIFFERS" }));
    };
    check(".aud files", auds.files.to_string(), "1750");
    check(".aud SOL / WAV", format!("{} / {}", auds.sol, auds.wav), "1749 / 1");
    check(".aud refused", auds.refused.len().to_string(), "0");
    check(
        ".aud at 22050 / 44100 Hz",
        format!("{} / {}", auds.rates.get(&22050).copied().unwrap_or(0), auds.rates.get(&44100).copied().unwrap_or(0)),
        "1745 / 4",
    );
    check(
        ".aud format 0x0D / 0x1D",
        format!("{} / {}", auds.formats.get(&0x0D).copied().unwrap_or(0), auds.formats.get(&0x1D).copied().unwrap_or(0)),
        "1657 / 92",
    );
    check("speech SOL files", speech.sol.to_string(), "2849");
    check(
        "speech at 22050 / 44100 Hz",
        format!("{} / {}", speech.rates.get(&22050).copied().unwrap_or(0), speech.rates.get(&44100).copied().unwrap_or(0)),
        "2839 / 10",
    );
    check("speech format 0x0D", speech.formats.get(&0x0D).copied().unwrap_or(0).to_string(), &speech.sol.to_string());
    check(
        "length mismatches (all SOL)",
        (auds.length_mismatch + speech.length_mismatch).to_string(),
        "0",
    );
    check(
        "loose SOL files (.aud + speech) decoded",
        format!("{} + {} = {}", auds.sol_loose, speech.sol_loose, auds.sol_loose + speech.sol_loose),
        &format!("{} + {} = 2498", auds.sol_loose, speech.sol_loose),
    );
    check("wrap-rounds (all SOL)", (auds.wraps + speech.wraps).to_string(), "0");
    check("largest peak", auds.peak.max(speech.peak).to_string(), "32768");
    let find = |name: &str| sounds.iter().find(|(k, _)| k.ends_with(name)).map(|(_, s)| s);
    check(
        "daventry AXEDOOR.AUD samples",
        find("daventry/sound/axedoor.aud").map_or("missing".into(), |s| format!("{} ({:.3} s)", s.frames(), s.seconds())),
        "11847 (0.537 s)",
    );
    check(
        "daventry FOREST.AUD",
        find("daventry/sound/forest.aud").map_or("missing".into(), |s| format!("{} ch, {:.1} s", s.channels, s.seconds())),
        "2 ch, 178.5 s",
    );
    check(
        "GAME/SOUND/NONE.AUD samples",
        find("game/sound/none.aud").map_or("missing".into(), |s| s.frames().to_string()),
        "8",
    );
    check(
        "longest .aud",
        auds.longest.as_ref().map_or("none".into(), |(t, n)| {
            let name = n.rsplit('/').next().unwrap_or(n).to_ascii_uppercase();
            format!("{name} {t:.1} s")
        }),
        "AMBDEAD.AUD 187.6 s",
    );
    // "How to check", file lookup (rule 13), for Daventry.
    for (name, want) in [
        ("raven.snd", "daventry/sound/RAVEN.AUD"),
        ("sprgouch.wav", "GAME/SOUND/SPRGOUCH.AUD"),
        ("chstopen", "GAME/SOUND/CHSTOPEN.AUD"),
        ("door4.aud", "GAME/SOUND/DOOR4.AUD"),
        ("dirt1", "GAME/SOUND/DIRT1.AUD"),
        ("poffup.aud", "not found"),
    ] {
        let got = aud::find(game, "Daventry", name).map_or("not found".to_string(), |p| {
            p.strip_prefix(game.root()).unwrap_or(&p).to_string_lossy().replace('\\', "/")
        });
        check(&format!("Daventry lookup of {name}"), got, want);
    }
    all
}

fn walk(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else { return };
    for e in entries.flatten() {
        let p = e.path();
        match e.file_type() {
            Ok(t) if t.is_dir() => walk(&p, out),
            Ok(t) if t.is_file() => out.push(p),
            _ => {}
        }
    }
}

/// Decodes one file and writes it as a plain 16-bit PCM WAV file.
fn write_wav(game: &GameDir, spec: &str, log: &mut Log) -> Result<(), String> {
    let bytes = source::load(game, spec)?;
    let s = aud::parse(&bytes).map_err(|e| format!("{spec}: {e}"))?;
    let dir = Path::new(ARTIFACTS_DIR).join("wav");
    fs::create_dir_all(&dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let out = dir.join(format!("{}.wav", source::stem(spec)));
    fs::write(&out, wav_bytes(&s)).map_err(|e| format!("{}: {e}", out.display()))?;
    log.summary(format!(
        "wrote {} ({} Hz, {} ch, {} samples, {:.3} s, peak {}) from {spec}",
        out.display(),
        s.rate,
        s.channels,
        s.frames(),
        s.seconds(),
        s.peak
    ));
    Ok(())
}

fn wav_bytes(s: &Sound) -> Vec<u8> {
    let data_len = (s.samples.len() * 2) as u32;
    let block = s.channels * 2;
    let mut b = Vec::with_capacity(44 + data_len as usize);
    b.extend_from_slice(b"RIFF");
    b.extend_from_slice(&(36 + data_len).to_le_bytes());
    b.extend_from_slice(b"WAVEfmt ");
    b.extend_from_slice(&16u32.to_le_bytes());
    b.extend_from_slice(&1u16.to_le_bytes());
    b.extend_from_slice(&s.channels.to_le_bytes());
    b.extend_from_slice(&s.rate.to_le_bytes());
    b.extend_from_slice(&(s.rate * u32::from(block)).to_le_bytes());
    b.extend_from_slice(&block.to_le_bytes());
    b.extend_from_slice(&16u16.to_le_bytes());
    b.extend_from_slice(b"data");
    b.extend_from_slice(&data_len.to_le_bytes());
    for v in &s.samples {
        b.extend_from_slice(&v.to_le_bytes());
    }
    b
}
