//! `kq8 msg-verify`: reads every message file (`<n>.MSG`) in the worlds'
//! `English` folders and `GAME/ENGLISH`, `TALKERS.SH` and every lip-sync
//! file next to them, and compares the counts with
//! `docs/behaviour/talking.md` (rules 1-5, 17). The copies under
//! `MASK/data/` are not counted (the note's 12 files are the loose ones).

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;
use std::process::ExitCode;

use kq8_data::msg::{parse_lips, MessageFile, Talkers, Tuple, VERSION};
use kq8_data::GameDir;

use crate::log::Log;

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("msg-verify") {
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

/// The note's numbers per file (folder, file, lines, `#` marks, lines
/// without a voice file next to the message file). `None`: not given.
type Expected = (&'static str, &'static str, Option<usize>, Option<usize>, Option<usize>);
const EXPECTED: [Expected; 12] = [
    ("daventry", "1000", Some(254), Some(100), Some(0)),
    ("GAME", "0", Some(140), Some(21), Some(6)),
    ("GAME", "400", None, None, None), // all lines without a voice
    ("GAME", "500", None, None, Some(168)),
    ("deadcity", "2000", None, None, Some(0)),
    ("swamp", "3000", None, None, Some(0)),
    ("gnome", "4000", None, None, Some(0)),
    ("barren", "5000", Some(282), None, Some(50)),
    ("iceworld", "6000", None, None, Some(2)),
    ("temple1", "7000", None, None, Some(19)),
    ("castled", "1000", None, None, None), // none in its own folder
    ("temple2", "7000", None, None, None), // none in its own folder
];

/// Lower-case names of the files in a folder.
fn listing(dir: &Path) -> BTreeSet<String> {
    fs::read_dir(dir).map(|d| d.filter_map(|e| e.ok()?.file_name().to_str().map(str::to_ascii_lowercase)).collect()).unwrap_or_default()
}

fn verify(game: &GameDir, log: &mut Log) -> bool {
    let mut ok = true;
    let mut check = |log: &mut Log, what: String, good: bool| {
        log.summary(format!("{} {what}", if good { "ok  " } else { "FAIL" }));
        ok &= good;
    };
    let talkers = match game.find("GAME/ENGLISH/TALKERS.SH").map(fs::read) {
        Some(Ok(b)) => Talkers::parse(&b),
        _ => {
            check(log, "GAME/ENGLISH/TALKERS.SH read".into(), false);
            Talkers::default()
        }
    };
    log.summary(format!("talkers: {} names; Connor is {:?}", talkers.names.len(), talkers.names.iter().find(|(_, n)| *n == "Connor").map(|(k, _)| *k)));
    check(log, "Connor's talker number is 4".into(), talkers.name(4) == Some("Connor"));
    let (mut total, mut total_marks) = (0, 0);
    let mut bytes_of: BTreeMap<(String, String), Vec<u8>> = BTreeMap::new();
    let (mut lips, mut lip_changes, mut lip_above_7, mut lip_failed) = (0usize, 0usize, 0usize, Vec::new());
    for (folder, number, want_lines, want_marks, want_silent) in EXPECTED {
        let dir_rel = if folder == "GAME" { "GAME/ENGLISH".to_string() } else { format!("{folder}/English") };
        let rel = format!("{dir_rel}/{number}.MSG");
        let Some(path) = game.find(&rel) else {
            check(log, format!("{rel} found"), false);
            continue;
        };
        let bytes = fs::read(&path).unwrap_or_default();
        bytes_of.insert((folder.to_string(), number.to_string()), bytes.clone());
        let m = match MessageFile::parse(number.parse().unwrap_or(0), &bytes) {
            Ok(m) => m,
            Err(e) => {
                check(log, format!("{rel} parses ({e})"), false);
                continue;
            }
        };
        let files = listing(path.parent().unwrap_or(Path::new(".")));
        let marks = m.lines.iter().filter(|l| l.shown().no_stop).count();
        let silent: Vec<Tuple> = m.lines.iter().map(|l| l.tuple).filter(|t| !files.contains(&t.voice_name().to_ascii_lowercase())).collect();
        let no_lip = m.lines.iter().filter(|l| !files.contains(&l.tuple.lip_name().to_ascii_lowercase())).count();
        let unclosed = m.lines.iter().filter(|l| l.shown().unclosed).count();
        let unknown_talkers: BTreeSet<u8> = m.lines.iter().map(|l| l.talker).filter(|&t| talkers.name(t).is_none()).collect();
        total += m.lines.len();
        total_marks += marks;
        log.summary(format!(
            "{rel}: type 0x{:02X}, version {}, {} lines, {marks} with '#', {} without a voice file here, {no_lip} without a lip-sync file, {unclosed} with an unclosed '(', talkers not in TALKERS.SH {unknown_talkers:?}",
            m.kind,
            m.version,
            m.lines.len(),
            silent.len()
        ));
        check(log, format!("{rel}: version {VERSION}"), m.version == VERSION);
        if let Some(w) = want_lines {
            check(log, format!("{rel}: {w} lines (note)"), m.lines.len() == w);
        }
        if let Some(w) = want_marks {
            check(log, format!("{rel}: {w} lines with '#' (note)"), marks == w);
        }
        match (folder, number, want_silent) {
            (_, _, Some(w)) => check(log, format!("{rel}: {w} lines without a voice file (note)"), silent.len() == w),
            ("GAME", "400", None) | ("castled", _, None) | ("temple2", _, None) => {
                check(log, format!("{rel}: no line has a voice file here (note)"), silent.len() == m.lines.len())
            }
            _ => {}
        }
        if folder == "daventry" {
            let mut by: BTreeMap<String, usize> = BTreeMap::new();
            for l in &m.lines {
                *by.entry(talkers.name(l.talker).unwrap_or("?").to_string()).or_default() += 1;
            }
            log.summary(format!("{rel}: lines per talker {by:?}"));
            for (who, n) in [("Connor", 172), ("Wizard", 40), ("Uglybeast", 17), ("Girl", 8), ("Knight", 6)] {
                check(log, format!("{rel}: {who} says {n} lines (note)"), by.get(who) == Some(&n));
            }
            let t = Tuple::new(1000, 55, 0, 0, 1);
            let sign = m.find(t).map(|l| l.shown());
            log.summary(format!("{rel}: {t} = {:?} shows {:?}", t.voice_name(), sign));
            check(
                log,
                format!("{rel}: the alchemist's sign line has '#' and starts with a space (note)"),
                sign.as_ref().is_some_and(|s| s.no_stop && s.text.starts_with(" The alchemist's shop.")),
            );
            check(log, format!("{rel}: {} is there", t.voice_name()), files.contains(&t.voice_name().to_ascii_lowercase()));
        }
        if folder == "GAME" && number == "0" {
            for (t, want) in [(Tuple::new(0, 0, 0, 1, 1), "'Tis beyond my reach."), (Tuple::new(0, 0, 0, 4, 1), "This will not work."), (Tuple::new(0, 109, 14, 33, 1), "'Tis empty.")] {
                let got = m.find(t).map(|l| l.shown().text);
                log.summary(format!("{rel}: {t} ({}) shows {got:?}", t.voice_name()));
                check(log, format!("{rel}: {t} says {want:?} (note)"), got.as_deref().is_some_and(|g| g.trim() == want));
            }
        }
        // Every lip-sync file of this folder's lines.
        for l in &m.lines {
            let name = l.tuple.lip_name();
            if !files.contains(&name.to_ascii_lowercase()) {
                continue;
            }
            match game.find(&format!("{dir_rel}/{name}")).map(fs::read) {
                Some(Ok(b)) => match parse_lips(&b) {
                    Ok(changes) => {
                        lips += 1;
                        lip_changes += changes.len();
                        lip_above_7 += changes.iter().filter(|c| c.shape > 7).count();
                        let sorted = changes.windows(2).all(|w| w[0].time <= w[1].time);
                        if !sorted {
                            lip_failed.push(format!("{dir_rel}/{name}: times not in order"));
                        }
                    }
                    Err(e) => lip_failed.push(format!("{dir_rel}/{name}: {e}")),
                },
                _ => lip_failed.push(format!("{dir_rel}/{name}: not read")),
            }
        }
    }
    log.summary(format!("all files: {total} lines, {total_marks} with '#'"));
    check(log, "2,080 lines in the 12 files (note)".into(), total == 2080);
    check(log, "476 lines with '#' (note)".into(), total_marks == 476);
    let same = |a: (&str, &str), b: (&str, &str)| {
        let get = |k: (&str, &str)| bytes_of.get(&(k.0.to_string(), k.1.to_string()));
        get(a).is_some() && get(a) == get(b)
    };
    check(log, "castled's 1000.MSG is a copy of Daventry's (note)".into(), same(("castled", "1000"), ("daventry", "1000")));
    check(log, "temple2's 7000.MSG is a copy of temple1's (note)".into(), same(("temple2", "7000"), ("temple1", "7000")));
    log.summary(format!("lip-sync files: {lips} read, {lip_changes} mouth changes, {lip_above_7} with a shape above 7, {} failed", lip_failed.len()));
    for f in &lip_failed {
        log.detail(format!("lip-sync: {f}"));
    }
    check(log, "every lip-sync file reads, times in order".into(), lip_failed.is_empty());
    ok
}
