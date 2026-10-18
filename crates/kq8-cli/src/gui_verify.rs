//! `kq8 gui-verify` (M4 step 4b-2): reads the tag file
//! `GAME/Resource/kqGuiTag.TTAG` and every layout (`.gui`) in
//! `GAME/Resource`, and logs each layout as a tree: kind, id and its
//! name, flags, rectangle, and the fonts, pictures and texts it names
//! (turned into their names and strings by the tag file).

use std::fs;
use std::process::ExitCode;

use kq8_data::gui::{self, Control, Kind, Picture, Tags, Text};
use kq8_data::GameDir;

use crate::log::Log;

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("gui-verify") {
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

fn named(tags: &Tags, id: i32) -> String {
    match tags.by_id.get(&id) {
        Some((name, s)) if s.is_empty() => format!("{id} {name}"),
        Some((name, s)) => format!("{id} {name} '{s}'"),
        None if id == 0 => "0".into(),
        None => format!("{id} (no tag)"),
    }
}

fn text(tags: &Tags, t: &Text) -> String {
    format!("font {}, justification 0x{:X}, text {}", named(tags, t.font), t.justification, named(tags, t.text))
}

fn picture(tags: &Tags, p: &Picture) -> String {
    format!("picture {}, modifier 0x{:X}, attribute 0x{:X}, reserved {}", named(tags, p.bitmap), p.modifier, p.attribute, p.reserved)
}

fn describe(tags: &Tags, c: &Control) -> String {
    let kind = match &c.kind {
        Kind::Dialog { caption, picture: p } => {
            let mut s = format!("{} child dialogs, {} controls", c.dialogs.len(), c.controls.len());
            if let Some(cap) = caption {
                s += &format!("; caption y {}, {}", cap.y, text(tags, &cap.text));
            }
            if let Some(p) = p {
                s += &format!("; {}", picture(tags, p));
            }
            s
        }
        Kind::Bitmap(p) => picture(tags, p),
        Kind::Text { text: t, pictures } => {
            let mut s = text(tags, t);
            if let Some(p) = pictures {
                s += &format!("; {}", picture(tags, p));
            }
            s
        }
        Kind::List { text: t, items } => format!("font {}, justification 0x{:X}, items {:?}", named(tags, t.font), t.justification, items),
        Kind::EditBox { font, blink_rate, max_chars } => format!("font {}, blink {blink_rate}, max {max_chars}", named(tags, *font)),
        Kind::Flic { flic, attribute, modifier, speed } => format!("flic {flic}, attribute 0x{attribute:X}, modifier 0x{modifier:X}, speed {speed}"),
        Kind::Scroll(v) => format!("scroll {v:?}"),
    };
    let [l, t, r, b] = c.rect;
    format!("{} v{} id {} flags 0x{:X} rect ({l}, {t})-({r}, {b}) [{}x{}]: {kind}", c.tag, c.version, named(tags, c.id as i32), c.flags, r - l, b - t)
}

fn verify(game: &GameDir, log: &mut Log) -> bool {
    let mut ok = true;
    let Some(dir) = game.find("GAME/Resource") else {
        log.summary("GAME/Resource not found");
        return false;
    };
    let tags = match game.find("GAME/Resource/kqGuiTag.TTAG").map(fs::read) {
        Some(Ok(bytes)) => {
            let (tags, bad) = Tags::parse(&String::from_utf8_lossy(&bytes));
            log.summary(format!("kqGuiTag.TTAG: {} tags, {} lines not read", tags.by_id.len(), bad.len()));
            for line in bad {
                log.detail(format!("  not read: {line}"));
            }
            tags
        }
        _ => {
            log.summary("kqGuiTag.TTAG not found: names not shown");
            ok = false;
            Tags::default()
        }
    };
    let mut files: Vec<_> = fs::read_dir(&dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
    files.retain(|p| p.extension().is_some_and(|e| e.eq_ignore_ascii_case("gui")));
    files.sort();
    for path in files {
        let name = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
        let bytes = match fs::read(&path) {
            Ok(b) => b,
            Err(e) => {
                log.summary(format!("{name}: ERROR cannot read: {e}"));
                ok = false;
                continue;
            }
        };
        match gui::parse(&bytes) {
            Ok(g) => {
                let all = g.walk();
                let untagged = all.iter().filter(|(_, c)| !tags.by_id.contains_key(&(c.id as i32))).count();
                log.summary(format!("{name}: OK, {} bytes read to the end, {} blocks, {} ids without a tag", bytes.len(), all.len(), untagged));
                for (depth, c) in all {
                    log.detail(format!("  {}{}", "  ".repeat(depth), describe(&tags, c)));
                }
            }
            Err(e) => {
                log.summary(format!("{name}: ERROR {e}"));
                ok = false;
            }
        }
    }
    ok
}
