//! Remarks and speech in the viewer (M4 step 4a, docs/DESIGN.md; rules:
//! docs/behaviour/talking.md).
//!
//! 4a-2: [`load_talk`] reads the message files of the world's `English`
//! folder, then `GAME/ENGLISH` (rule 1: the world's first, our reading of
//! the data), `TALKERS.SH`, and gives the world's rules a way to find a
//! line's voice and lip-sync files next to its message file (rule 4).
//! The queues themselves run in `kq8_core::talk`, inside the door
//! simulation; their lines go to `kq8-artifacts/logs/talk.log`.

use kq8_core::talk::{Book, Talk, Voice};
use kq8_data::msg::{parse_lips, MessageFile, Talkers, Tuple};
use kq8_data::{aud, GameDir};

use crate::log::Log;

/// The message files, talkers and voice lookup for `world`, and the
/// `set` lines of `MASK/Options.cs` (the console variables it sets, e.g.
/// `Options::Captions`, rule 10).
pub fn load_talk(game: &GameDir, world: &str, log: &mut Log) -> (Talk, Vec<(String, String)>) {
    let mut book = Book::default();
    for folder in [format!("{world}/English"), "GAME/ENGLISH".to_string()] {
        let Some(dir) = game.find(&folder) else {
            log.summary(format!("talk: no folder {folder}"));
            continue;
        };
        let rel = dir.strip_prefix(game.root()).unwrap_or(&dir).to_string_lossy().replace('\\', "/");
        let mut names: Vec<_> = std::fs::read_dir(&dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
        names.sort();
        for path in names {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let is_msg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("msg"));
            let Some(number) = stem.parse::<u16>().ok().filter(|_| is_msg) else { continue };
            match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| MessageFile::parse(number, &b)) {
                Ok(m) => {
                    let lines = m.lines.len();
                    let added = book.add(&rel, m, number);
                    log.summary(format!("talk: {rel}/{stem}.MSG: {lines} lines{}", if added { "" } else { " (number already read from another folder: not used)" }));
                }
                Err(e) => log.summary(format!("talk: {rel}/{stem}.MSG: {e}; not used")),
            }
        }
    }
    match game.find("GAME/ENGLISH/TALKERS.SH").map(std::fs::read) {
        Some(Ok(b)) => book.talkers = Talkers::parse(&b),
        _ => log.summary("talk: GAME/ENGLISH/TALKERS.SH not read: every line is spoken by its issuer"),
    }
    log.summary(format!("talk: {} message files, {} talkers", book.files.len(), book.talkers.names.len()));
    let mut talk = Talk::default();
    talk.book = book;
    let g = game.clone();
    talk.voices = Some(Box::new(move |folder: &str, t: Tuple| voice(&g, folder, t)));
    let sets = match game.find("Options.cs").map(std::fs::read) {
        Some(Ok(b)) => options_sets(&b),
        _ => Vec::new(),
    };
    for (k, v) in &sets {
        log.summary(format!("talk: Options.cs sets {k} = '{v}'"));
    }
    (talk, sets)
}

/// A line's voice file and its lip-sync file (rules 4, 17), both next to
/// its message file; `None` without a readable voice file.
fn voice(game: &GameDir, folder: &str, t: Tuple) -> Option<Voice> {
    let path = game.find(&format!("{folder}/{}", t.voice_name()))?;
    let sound = std::fs::read(&path).ok().and_then(|b| aud::parse(&b).ok())?;
    let rel = path.strip_prefix(game.root()).unwrap_or(&path).to_string_lossy().replace('\\', "/");
    let mouths = game.find(&format!("{folder}/{}", t.lip_name())).and_then(|p| std::fs::read(p).ok()).and_then(|b| parse_lips(&b).ok()).unwrap_or_default();
    Some(Voice { path: rel, seconds: sound.seconds(), mouths })
}

/// The `set <name> <value…>` lines of a console script such as
/// `Options.cs` (the value is the rest of the line, words joined by one
/// space, as the script runner's `set`).
pub fn options_sets(bytes: &[u8]) -> Vec<(String, String)> {
    let text: String = bytes.iter().map(|&b| b as char).collect();
    text.lines()
        .filter_map(|line| {
            let mut w = line.split('#').next().unwrap_or("").split_whitespace();
            (w.next()? == "set").then_some(())?;
            let name = w.next()?.to_string();
            Some((name, w.collect::<Vec<_>>().join(" ")))
        })
        .collect()
}
