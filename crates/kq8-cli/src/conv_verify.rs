//! `kq8 conv-verify` (M4 step 4e-1): reads every world's conversation info
//! objects (`classType=KQConvInfo`) and the data around conversations, and
//! compares the counts with `docs/behaviour/conversation.md` (rules 1, 3,
//! 14, 20, "How to check"):
//!
//! - every info object: its talking spot, areas and cameras;
//! - every `KQConner::conversation` line of the world's scripts: its
//!   number of words, the `<enemy>` word, the partner (`$1` is the object
//!   whose `script=` is that file) and the largest camera number `[n]` of
//!   its lines, against the partner's camera count;
//! - the characters with goodbye / welcome-back keys (rule 20);
//! - the note's three Daventry points (inside which areas, distance to the
//!   spot).
//!
//! Log: `kq8-artifacts/logs/conv-verify.log`.

use std::collections::BTreeMap;
use std::process::ExitCode;

use kq8_data::characters::CharacterKeys;
use kq8_data::conv_info::ConvInfo;
use kq8_data::game_dir::{VOLUMES, WORLDS};
use kq8_data::msg::{MessageFile, Tuple};
use kq8_data::{kq_text, GameDir, VolArchive};

use crate::log::Log;

/// The note's Daventry checks: (point, areas it should be in).
const DAVENTRY_POINTS: [([f32; 2], &[usize]); 3] = [([193000.0, 198000.0], &[0, 1]), ([191000.0, 197000.0], &[0, 2]), ([198000.0, 198000.0], &[])];

#[derive(Default)]
struct Totals {
    infos: usize,
    calls: usize,
    enemy_not_0: usize,
    short_calls: usize,
    goodbye_keys: usize,
    camera_over: usize,
}

/// A world's archives.
struct World {
    name: String,
    vols: Vec<VolArchive>,
}

impl World {
    /// Every entry ending in `ext` (lower case): name and bytes.
    fn files(&self, ext: &str) -> Vec<(String, Vec<u8>)> {
        let mut out = Vec::new();
        for a in &self.vols {
            for e in a.entries() {
                if e.name.to_ascii_lowercase().ends_with(ext) {
                    if let Ok(u) = a.unpack(e) {
                        out.push((e.name.clone(), u.data));
                    }
                }
            }
        }
        out
    }
}

pub fn run(game: &GameDir) -> ExitCode {
    let mut log = match Log::create("conv-verify") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("error: cannot create log file: {e}");
            return ExitCode::FAILURE;
        }
    };
    log.summary(format!("conv-verify: game folder {}", game.root().display()));
    log.summary(format!("log file: {}", log.path().display()));
    let mut totals = Totals::default();
    for name in WORLDS {
        let vols = VOLUMES.iter().filter_map(|v| game.volume_path(name, v)).filter_map(|p| VolArchive::open(&p).ok()).collect();
        verify_world(game, &World { name: name.to_string(), vols }, &mut log, &mut totals);
    }
    log.summary(format!(
        "ALL WORLDS: info objects {}, KQConner::conversation calls {} ({} with <enemy> not 0, {} with fewer than 9 words), characters with goodbye/welcome-back keys {}, calls whose largest camera number is not below the partner's nCam {}",
        totals.infos, totals.calls, totals.enemy_not_0, totals.short_calls, totals.goodbye_keys, totals.camera_over
    ));
    log.summary("expected (conversation.md): 16 info objects, 169 calls all with <enemy> 0, 19 characters with the keys, every camera number within nCam");
    ExitCode::SUCCESS
}

/// A message file of the world (its `English` folder), else `GAME/ENGLISH`.
fn message_file(game: &GameDir, world: &str, number: u16, cache: &mut BTreeMap<u16, Option<MessageFile>>) -> Option<MessageFile> {
    cache
        .entry(number)
        .or_insert_with(|| {
            [format!("{}/English/{number}.MSG", world.to_ascii_lowercase()), format!("GAME/ENGLISH/{number}.MSG")]
                .iter()
                .find_map(|rel| game.find(rel).and_then(|p| std::fs::read(p).ok()))
                .and_then(|b| MessageFile::parse(number, &b).ok())
        })
        .clone()
}

fn verify_world(game: &GameDir, world: &World, log: &mut Log, totals: &mut Totals) {
    log.detail(format!("== {} ==", world.name));
    // Info objects, characters and which object has which script.
    let mut infos: Vec<ConvInfo> = Vec::new();
    let mut script_owner: BTreeMap<String, Vec<String>> = BTreeMap::new();
    let mut keys = 0;
    for (file_name, bytes) in world.files(".kq") {
        let file = kq_text::parse(&bytes);
        for s in &file {
            if let Some(info) = ConvInfo::from_section(s) {
                log.detail(format!(
                    "{file_name} [{}]: partner {}, spot ({:.0}, {:.0}, {:.0}), {} areas, {} cameras{}",
                    info.name,
                    info.partner(),
                    info.spot[0],
                    info.spot[1],
                    info.spot[2],
                    info.areas.len(),
                    info.cameras.len(),
                    if info.problems.is_empty() { String::new() } else { format!("; PROBLEMS: {}", info.problems.join("; ")) }
                ));
                for (k, a) in info.areas.iter().enumerate() {
                    let what = match k {
                        0 => "start area",
                        1 => "stay area",
                        _ => "special area",
                    };
                    log.detail(format!("  box{k} ({what}): {a:?}"));
                }
                for (k, c) in info.cameras.iter().enumerate() {
                    log.detail(format!("  cam{k}: at {:?}, angles {:?}", c.position, c.turn));
                }
                infos.push(info);
            }
            if let Some(script) = s.get("script").map(|v| v.split_whitespace().next().unwrap_or("").to_ascii_lowercase()).filter(|v| !v.is_empty()) {
                script_owner.entry(script).or_default().push(s.name.clone());
            }
            if let Some(k) = CharacterKeys::from_section(s) {
                if k.goodbye.is_some() || k.welcome_back.is_some() {
                    keys += 1;
                    log.detail(format!("{file_name} [{}]: goodbye {:?}, welcome back {:?}", s.name, k.goodbye.map(|t| t.to_string()), k.welcome_back.map(|t| t.to_string())));
                }
            }
        }
    }
    // The conversation lines of the scripts.
    let mut cache = BTreeMap::new();
    let (mut calls, mut enemy_not_0, mut short) = (0, 0, 0);
    let mut largest: BTreeMap<String, (u32, String)> = BTreeMap::new();
    for (file_name, bytes) in world.files(".cs") {
        let text: String = bytes.iter().map(|&b| b as char).collect();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.trim();
            if line.starts_with('#') || !line.starts_with("KQConner::conversation") {
                continue;
            }
            calls += 1;
            let words: Vec<&str> = line.split_whitespace().collect();
            if words.len() < 9 {
                short += 1;
                log.detail(format!("{file_name} line {}: {line}: fewer than 9 words (rule 1: usage, nothing happens)", n + 1));
                continue;
            }
            if words[8] != "0" {
                enemy_not_0 += 1;
            }
            // `$1`: the object running the script; several objects can share
            // one (`wizard.cs` is also `magicmap`'s): the one with an info
            // object first.
            let partner = if words[1] == "$1" {
                script_owner
                    .get(&file_name.to_ascii_lowercase())
                    .and_then(|o| o.iter().find(|n| infos.iter().any(|i| i.partner().eq_ignore_ascii_case(n))).or(o.first()).cloned())
                    .unwrap_or_else(|| "$1 (owner not found)".into())
            } else {
                words[1].to_string()
            };
            let num = |k: usize| words[k].parse::<i64>().ok();
            let (Some(f), Some(no), Some(v), Some(c), Some(first), Some(last)) = (num(2), num(3), num(4), num(5), num(6), num(7)) else {
                log.detail(format!("{file_name} line {}: {line}: not all numbers", n + 1));
                continue;
            };
            let first_t = Tuple::new(f as u16, no as u8, v as u8, c as u8, first as u8);
            let Some(mf) = message_file(game, &world.name, f as u16, &mut cache) else {
                log.detail(format!("{file_name} line {}: {line}: message file {f} not found", n + 1));
                continue;
            };
            let last = if last == 0 { mf.last_of_run(first_t).unwrap_or(first as u8) } else { last as u8 };
            let cameras: Vec<u32> = (first as u8..=last).filter_map(|s| mf.find(first_t.with_seq(s))).filter_map(|l| l.shown().camera).collect();
            let top = cameras.iter().copied().max();
            log.detail(format!(
                "{file_name} line {}: partner {partner}, lines {first_t}…{last}, <enemy> {}, camera numbers {cameras:?}",
                n + 1,
                words[8]
            ));
            if let Some(t) = top {
                let e = largest.entry(partner.to_ascii_lowercase()).or_insert((0, partner.clone()));
                e.0 = e.0.max(t);
            }
        }
    }
    for (key, (top, partner)) in &largest {
        let info = infos.iter().find(|i| i.partner().eq_ignore_ascii_case(key));
        let n_cam = info.map(|i| i.cameras.len());
        let ok = n_cam.is_some_and(|n| (*top as usize) < n);
        if !ok {
            totals.camera_over += 1;
        }
        log.summary(format!(
            "{}: {partner}: largest camera number {top}, cameras {}: {}",
            world.name,
            n_cam.map_or("none (no info object)".into(), |n| n.to_string()),
            if ok { "within" } else { "NOT WITHIN" }
        ));
    }
    if world.name == "Daventry" {
        if let Some(w) = infos.iter().find(|i| i.partner().eq_ignore_ascii_case("Wizard")) {
            for (p, want) in DAVENTRY_POINTS {
                let got: Vec<usize> = (0..w.areas.len()).filter(|&k| w.in_area(k, p)).collect();
                let d = ((p[0] - w.spot[0]).powi(2) + (p[1] - w.spot[1]).powi(2)).sqrt();
                log.summary(format!(
                    "Daventry: Wizard: ({:.0}, {:.0}) is in areas {got:?} (note: {want:?}): {}; {d:.0} from the spot (flat)",
                    p[0],
                    p[1],
                    if got == want { "as the note" } else { "DIFFERENT" }
                ));
            }
        }
    }
    log.summary(format!(
        "{}: {} info objects ({}), {calls} conversation calls ({enemy_not_0} with <enemy> not 0, {short} short), {keys} characters with goodbye/welcome-back keys",
        world.name,
        infos.len(),
        infos.iter().map(|i| format!("{} {}/{}", i.partner(), i.areas.len(), i.cameras.len())).collect::<Vec<_>>().join(", ")
    ));
    totals.infos += infos.len();
    totals.calls += calls;
    totals.enemy_not_0 += enemy_not_0;
    totals.short_calls += short;
    totals.goodbye_keys += keys;
}
