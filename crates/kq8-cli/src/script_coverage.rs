//! `kq8 script-coverage [--out FILE]`: what the game's scripts and object
//! files ask of the program, world by world, and whether our build has it
//! (`docs/DESIGN.md`, "script coverage checklist", SC-1). Ours, a tool:
//! it reads the game's archives and our own code, never `Mask.exe`.
//!
//! - **Commands** (the first word of each `.cs` line; after `if` /
//!   `else if`, the word after `eval`) and **Connor's states**
//!   (`KQMonster::setState Connor <state>`): each is run once, as a
//!   one-line script made from a real line of the data, in an empty
//!   `Active`; its log decides: `ERROR unknown command` = missing,
//!   `stub (later milestone` = stub, otherwise built (built can still be
//!   partial or a stand-in).
//! - **Event kinds** (`sendEvent … <kind>`, also in `.kq` movement cues),
//!   **script calls** the scripts expect (`test $2 == <word>`) and
//!   **classes** (`classType=`): "named" when the exact word appears
//!   quoted somewhere in `crates/` (a hint only).
//!
//! - **Start-up scripts** (the world's `newWorld.cs` and every script it
//!   names, followed by name) are read by the world loader
//!   (`kq8_data::world_script`), which runs only a few commands and counts
//!   the rest; a command used only there gets its status from that list.
//!
//! Only names, counts and file names are written, no script text.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;
use std::process::ExitCode;

use kq8_core::active::Active;
use kq8_data::world_script::Vars;
use kq8_data::{GameDir, VolArchive};

/// Worlds in the order the game visits them, then the shared scripts.
const ORDER: [&str; 12] = ["Daventry", "CastleD", "DeadCity", "Swamp", "Gnome", "Barren", "IceWorld", "Snowexit", "temple1", "temple2", "temple3", "temple4"];
const COMMON: &str = "Common";

/// Words that are the script language itself (`script.rs`), not commands.
const CONTROL: [&str; 6] = ["if", "else", "endif", "echo", "set", "test"];

/// What the world loader acts on in start-up scripts (`world_script.rs`;
/// `setLoadProgress` drives the loading meter). Lower case.
const LOADER: [&str; 13] = [
    "set",
    "loadkq",
    "getloc",
    "kqobject::getloc",
    "move",
    "kqobject::sethidden",
    "kqmonster::inventory",
    "give",
    "kqobject::setscript",
    "kqobject::setdontlook",
    "applycombatlevel",
    "kqconner::setlevel",
    "sendevent",
];

/// Names that look like loading or memory hints (by name only: to be
/// confirmed by session A).
const LOADING_HINTS: [&str; 12] = [
    "lockShape",
    "preloadShape",
    "purgeResource",
    "lockInterior",
    "preloadResource",
    "KQObject::preloadResources",
    "lockResource",
    "loadVolume",
    "activatePersistMgr",
    "loadAnim",
    "setLoadProgress",
    "loadPalette",
];

#[derive(Default)]
struct Seen {
    /// Uses per world.
    uses: BTreeMap<&'static str, usize>,
    /// Of those, uses in start-up scripts.
    startup: usize,
    /// First file it was seen in (`World/file`), and a sample line.
    example: Option<(String, Vec<String>)>,
}

impl Seen {
    fn add(&mut self, world: &'static str, file: &str, line: &[String]) {
        *self.uses.entry(world).or_default() += 1;
        if STARTUP.with(|s| s.borrow().contains(&file.to_ascii_lowercase())) {
            self.startup += 1;
        }
        if self.example.is_none() {
            self.example = Some((format!("{world}/{file}"), line.to_vec()));
        }
    }

    fn total(&self) -> usize {
        self.uses.values().sum()
    }

    fn worlds(&self) -> String {
        ORDER.iter().chain([&COMMON]).filter_map(|w| self.uses.get(w).map(|n| format!("{w} {n}"))).collect::<Vec<_>>().join(", ")
    }

    fn file(&self) -> &str {
        self.example.as_ref().map_or("", |(f, _)| f.as_str())
    }
}

thread_local! {
    /// The start-up scripts of the world being read (lower case).
    static STARTUP: std::cell::RefCell<BTreeSet<String>> = Default::default();
}

#[derive(Default)]
struct Found {
    commands: BTreeMap<String, Seen>,
    states: BTreeMap<String, Seen>,
    events: BTreeMap<String, Seen>,
    calls: BTreeMap<String, Seen>,
    classes: BTreeMap<String, Seen>,
    scripts_per_world: BTreeMap<&'static str, usize>,
    script_calls: usize,
    dynamic: usize,
}

pub fn run(game: &GameDir, out: Option<&str>) -> ExitCode {
    let mut found = Found::default();
    for world in ORDER {
        let Some(path) = game.volume_path(world, "kq") else {
            eprintln!("warning: no data/{world}/kq.vol");
            continue;
        };
        let vol = match VolArchive::open(&path) {
            Ok(v) => v,
            Err(e) => {
                eprintln!("error: {}: {e}", path.display());
                return ExitCode::FAILURE;
            }
        };
        let files: Vec<(String, Vec<u8>)> = vol.entries().iter().filter_map(|e| vol.unpack(e).ok().map(|u| (e.name.clone(), u.data))).collect();
        read_world(world, &files, &mut found);
    }
    // The shared scripts and object files (`GAME/Kq`).
    if let Some(dir) = game.find("GAME/Kq") {
        let files: Vec<(String, Vec<u8>)> = std::fs::read_dir(&dir)
            .into_iter()
            .flatten()
            .flatten()
            .filter_map(|e| Some((e.file_name().to_string_lossy().into_owned(), std::fs::read(e.path()).ok()?)))
            .collect();
        read_world(COMMON, &files, &mut found);
    }
    let quoted = quoted_words(Path::new("crates"));
    let report = report(&mut found, &quoted);
    match out {
        Some(file) => match std::fs::write(file, &report) {
            Ok(()) => {
                println!("written: {file}");
                ExitCode::SUCCESS
            }
            Err(e) => {
                eprintln!("error: {file}: {e}");
                ExitCode::FAILURE
            }
        },
        None => {
            print!("{report}");
            ExitCode::SUCCESS
        }
    }
}

fn text(bytes: &[u8]) -> String {
    bytes.iter().map(|&b| b as char).collect()
}

fn words(line: &str) -> Vec<String> {
    let line = line.split('#').next().unwrap_or("");
    line.split_whitespace().map(|w| w.trim_matches('\'').to_string()).collect()
}

fn read_world(world: &'static str, files: &[(String, Vec<u8>)], found: &mut Found) {
    let scripts: BTreeSet<String> = files.iter().filter(|(n, _)| n.to_ascii_lowercase().ends_with(".cs")).map(|(n, _)| n.to_ascii_lowercase()).collect();
    STARTUP.with(|s| *s.borrow_mut() = startup_scripts(files, &scripts));
    for (name, bytes) in files {
        let lower = name.to_ascii_lowercase();
        if lower.ends_with(".cs") {
            *found.scripts_per_world.entry(world).or_default() += 1;
            for line in text(bytes).lines() {
                read_script_line(world, name, &words(line), &scripts, found);
            }
        } else if lower.ends_with(".kq") {
            for line in text(bytes).lines() {
                let line = line.split(';').next().unwrap_or("").trim();
                if let Some((key, value)) = line.split_once('=') {
                    if key.trim().eq_ignore_ascii_case("classType") {
                        let class = value.split_whitespace().next().unwrap_or("").to_string();
                        if !class.is_empty() {
                            found.classes.entry(class).or_default().add(world, name, &[]);
                        }
                    }
                }
                // Movement cues: `cue0=<phase> sendEvent @ <kind> ...`.
                let w = words(line);
                if let Some(i) = w.iter().position(|x| x.eq_ignore_ascii_case("sendEvent")) {
                    if let Some(kind) = w.get(i + 2).filter(|k| !k.starts_with('$')) {
                        found.events.entry(kind.clone()).or_default().add(world, name, &w);
                    }
                }
            }
        }
    }
}

/// `newWorld.cs` and every script it names (a line starting with a script's
/// name), followed by name, lower case.
fn startup_scripts(files: &[(String, Vec<u8>)], scripts: &BTreeSet<String>) -> BTreeSet<String> {
    let by_name: BTreeMap<String, &Vec<u8>> = files.iter().map(|(n, b)| (n.to_ascii_lowercase(), b)).collect();
    let mut out = BTreeSet::new();
    let mut todo = vec!["newworld.cs".to_string()];
    while let Some(name) = todo.pop() {
        if !out.insert(name.clone()) {
            continue;
        }
        let Some(bytes) = by_name.get(&name) else { continue };
        for line in text(bytes).lines() {
            let Some(first) = words(line).first().map(|w| w.to_ascii_lowercase()) else { continue };
            let as_script = if first.ends_with(".cs") { first } else { format!("{first}.cs") };
            if scripts.contains(&as_script) {
                todo.push(as_script);
            }
        }
    }
    out.retain(|n| by_name.contains_key(n));
    out
}

fn read_script_line(world: &'static str, file: &str, w: &[String], scripts: &BTreeSet<String>, found: &mut Found) {
    let Some(first) = w.first() else { return };
    // `if <cmd>`, `else if <cmd>`: the condition's command.
    let rest: &[String] = match first.to_ascii_lowercase().as_str() {
        "if" => &w[1..],
        "else" if w.get(1).is_some_and(|x| x.eq_ignore_ascii_case("if")) => &w[2..],
        _ => w,
    };
    let rest = if rest.first().is_some_and(|x| x.eq_ignore_ascii_case("eval")) { &rest[1..] } else { rest };
    // `test $2 == <word>`: a call the script expects from the program.
    if rest.first().is_some_and(|x| x.eq_ignore_ascii_case("test")) {
        if let [_, a, op, b, ..] = rest {
            if a.trim_matches('"') == "$2" && op == "==" && !b.starts_with('$') {
                found.calls.entry(b.trim_matches('"').to_string()).or_default().add(world, file, rest);
            }
        }
        return;
    }
    let Some(cmd) = rest.first() else { return };
    if CONTROL.iter().any(|c| c.eq_ignore_ascii_case(cmd)) {
        return;
    }
    if cmd.starts_with('$') {
        found.dynamic += 1;
        return;
    }
    let as_script = if cmd.contains('.') { cmd.to_ascii_lowercase() } else { format!("{}.cs", cmd.to_ascii_lowercase()) };
    if scripts.contains(&as_script) || cmd.to_ascii_lowercase().ends_with(".cs") {
        found.script_calls += 1;
        return;
    }
    found.commands.entry(cmd.clone()).or_default().add(world, file, rest);
    if cmd.eq_ignore_ascii_case("sendEvent") {
        if let Some(kind) = rest.get(3).filter(|k| !k.starts_with('$')) {
            found.events.entry(kind.clone()).or_default().add(world, file, rest);
        }
    }
    if cmd.eq_ignore_ascii_case("KQMonster::setState") && rest.get(1).is_some_and(|x| x.eq_ignore_ascii_case("Connor")) {
        if let Some(state) = rest.get(2).filter(|s| !s.starts_with('$')) {
            // Capitals are ignored by the game (characters.md rule 22).
            let key = found.states.keys().find(|k| k.eq_ignore_ascii_case(state)).cloned().unwrap_or_else(|| state.clone());
            found.states.entry(key).or_default().add(world, file, rest);
        }
    }
}

/// Every word quoted in our Rust code under `dir`, lower case.
fn quoted_words(dir: &Path) -> BTreeSet<String> {
    let mut out = BTreeSet::new();
    let mut stack = vec![dir.to_path_buf()];
    while let Some(d) = stack.pop() {
        for e in std::fs::read_dir(&d).into_iter().flatten().flatten() {
            let p = e.path();
            if p.is_dir() {
                stack.push(p);
            } else if p.extension().is_some_and(|x| x == "rs") {
                let Ok(src) = std::fs::read_to_string(&p) else { continue };
                for part in src.split('"').skip(1).step_by(2) {
                    if !part.is_empty() && part.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == ':') {
                        out.insert(part.to_ascii_lowercase());
                    }
                }
            }
        }
    }
    out
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Status {
    Missing,
    Crashed,
    Stub,
    /// Only in start-up scripts, which the world loader counts but doesn't run.
    StartUpSkipped,
    Built,
    /// Only in start-up scripts, acted on by the world loader.
    StartUpRead,
}

impl Status {
    fn word(self) -> &'static str {
        match self {
            Status::Missing => "**missing**",
            Status::Crashed => "**probe crashed**",
            Status::Stub => "stub",
            Status::StartUpSkipped => "start-up only: counted, not run",
            Status::Built => "built",
            Status::StartUpRead => "start-up only: read by the loader",
        }
    }
}

/// Runs `line` as a one-line script in an empty world and reads the log.
fn probe(line: &[String]) -> Status {
    let source = line.join(" ");
    let result = std::panic::catch_unwind(|| {
        let mut a = Active::new(Vars::default(), vec![], vec![], ["Connor".to_string()], [("probe.cs".to_string(), format!("{source}\n").into_bytes())]);
        a.run_script("probe.cs", &[]);
        a.take_log()
    });
    match result {
        Err(_) => Status::Crashed,
        Ok(log) => {
            if log.iter().any(|l| l.contains("ERROR unknown command")) {
                Status::Missing
            } else if log.iter().any(|l| l.contains("stub (later milestone")) {
                Status::Stub
            } else {
                Status::Built
            }
        }
    }
}

fn report(found: &mut Found, quoted: &BTreeSet<String>) -> String {
    // The probes print nothing: a crash is a status, not a message.
    let hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let commands: Vec<(String, Status)> = found
        .commands
        .iter()
        .map(|(c, s)| {
            let status = if s.startup == s.total() {
                if LOADER.contains(&c.to_ascii_lowercase().as_str()) || c.eq_ignore_ascii_case("setLoadProgress") {
                    Status::StartUpRead
                } else {
                    Status::StartUpSkipped
                }
            } else {
                probe(&s.example.as_ref().unwrap().1)
            };
            (c.clone(), status)
        })
        .collect();
    let states: Vec<(String, Status)> = found.states.iter().map(|(c, s)| (c.clone(), probe(&s.example.as_ref().unwrap().1))).collect();
    std::panic::set_hook(hook);

    let mut o = String::new();
    let mut line = |s: String| {
        o.push_str(&s);
        o.push('\n');
    };
    line("# Script coverage".into());
    line(String::new());
    line("Generated by `kq8 script-coverage --out docs/SCRIPT-COVERAGE.md` (`docs/DESIGN.md`, \"script coverage checklist\"). Do not edit by hand: run it again after building more.".into());
    line(String::new());
    line("What the game's scripts (`.cs`) and object files (`.kq`) ask of the program, from every world's `kq.vol` and `GAME/Kq` (\"Common\"). Worlds in the order the game visits them.".into());
    line(String::new());
    line("- **Commands and Connor's states**: each was run once in an empty world, with a real line from the data. *built* = our build handles the word (it can still be partial or a stand-in); *stub* = accepted, does nothing yet; **missing** = \"unknown command\" in our log.".into());
    line("- **Event kinds, script calls, classes**: *named* = the exact word appears quoted in our code (a hint only); **not named** = nowhere in our code.".into());
    line("- **Start-up scripts** (`newWorld.cs` and the scripts it names, e.g. `permwrld.cs`, `worldvar.cs`) are read by our world loader, which acts on a few commands (`loadKQ`, `set`, `move`, items, …) and only counts the rest; a command used only there is marked *start-up only*. A command also used during play was probed as above.".into());
    line("- \"loading hint?\": the name looks like a loading or memory instruction with nothing to see; to be confirmed by session A.".into());
    line(String::new());

    // Summary per world.
    let status_of: BTreeMap<&str, Status> = commands.iter().map(|(c, s)| (c.as_str(), *s)).collect();
    line("## Summary per world".into());
    line(String::new());
    line("| World | Scripts | Commands used | Missing (uses) | Stub (uses) | Connor states not built |".into());
    line("| --- | ---: | ---: | --- | --- | --- |".into());
    for w in ORDER.iter().chain([&COMMON]) {
        let used: Vec<(&String, usize)> = found.commands.iter().filter_map(|(c, s)| s.uses.get(w).map(|n| (c, *n))).collect();
        let count = |st: Status| {
            let hits: Vec<usize> = used.iter().filter(|(c, _)| status_of.get(c.as_str()) == Some(&st) && !is_hint(c)).map(|(_, n)| *n).collect();
            format!("{} ({})", hits.len(), hits.iter().sum::<usize>())
        };
        let open_states: Vec<&str> = states.iter().filter(|(st, s)| *s != Status::Built && found.states[st].uses.contains_key(w)).map(|(st, _)| st.as_str()).collect();
        line(format!(
            "| {w} | {} | {} | {} | {} | {} |",
            found.scripts_per_world.get(w).copied().unwrap_or(0),
            used.len(),
            count(Status::Missing),
            count(Status::Stub),
            if open_states.is_empty() { "—".to_string() } else { open_states.join(", ") }
        ));
    }
    line(String::new());
    line(format!(
        "Not counted as commands: {} calls of other scripts by name, {} lines starting with a variable. Loading hints are left out of the summary's counts.",
        found.script_calls, found.dynamic
    ));
    line(String::new());

    // Commands, missing first, then by uses.
    let mut rows: Vec<(&String, Status)> = commands.iter().map(|(c, s)| (c, *s)).collect();
    rows.sort_by_key(|(c, s)| (*s, std::cmp::Reverse(found.commands[*c].total())));
    line("## Commands".into());
    line(String::new());
    line("| Command | Status | Uses (start-up) | Worlds | First seen in |".into());
    line("| --- | --- | ---: | --- | --- |".into());
    for (c, s) in rows {
        let seen = &found.commands[c];
        let hint = if is_hint(c) { " (loading hint?)" } else { "" };
        let typo = if typo(c) { " (typo in the original?)" } else { "" };
        line(format!("| `{c}` | {}{hint}{typo} | {} ({}) | {} | `{}` |", s.word(), seen.total(), seen.startup, seen.worlds(), seen.file()));
    }
    line(String::new());

    line("## Connor's states (`KQMonster::setState Connor <state>`)".into());
    line(String::new());
    line("| State | Status | Uses | Worlds | First seen in |".into());
    line("| --- | --- | ---: | --- | --- |".into());
    let mut rows: Vec<&(String, Status)> = states.iter().collect();
    rows.sort_by_key(|(c, s)| (*s, std::cmp::Reverse(found.states[c].total())));
    for (c, s) in rows {
        let seen = &found.states[c];
        line(format!("| `{c}` | {} | {} | {} | `{}` |", s.word(), seen.total(), seen.worlds(), seen.file()));
    }
    line(String::new());

    for (title, map) in [
        ("Event kinds (`sendEvent … <kind>`, also in `.kq` movement cues)", &found.events),
        ("Calls the scripts expect from the program (`test $2 == <word>`)", &found.calls),
        ("Object classes (`classType=` in `.kq` sections)", &found.classes),
    ] {
        line(format!("## {title}"));
        line(String::new());
        line("| Word | In our code | Uses | Worlds | First seen in |".into());
        line("| --- | --- | ---: | --- | --- |".into());
        let mut rows: Vec<(&String, &Seen, bool)> = map.iter().map(|(k, s)| (k, s, quoted.contains(&k.to_ascii_lowercase()))).collect();
        rows.sort_by_key(|(_, s, named)| (*named, std::cmp::Reverse(s.total())));
        for (k, s, named) in rows {
            line(format!("| `{k}` | {} | {} | {} | `{}` |", if named { "named" } else { "**not named**" }, s.total(), s.worlds(), s.file()));
        }
        line(String::new());
    }
    o
}

fn is_hint(c: &str) -> bool {
    LOADING_HINTS.iter().any(|h| h.eq_ignore_ascii_case(c))
}

/// A word that looks like a misspelt control word (`endifif`) or a line
/// commented out the `.kq` way (`;`).
fn typo(c: &str) -> bool {
    let l = c.to_ascii_lowercase();
    l.starts_with(';') || ["endif", "else", "echo"].iter().any(|k| l.starts_with(k) && l != *k)
}
