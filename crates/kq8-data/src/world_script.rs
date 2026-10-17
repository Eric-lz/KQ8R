//! Following a world's start-up scripts (`newWorld.cs` and the scripts it
//! names) to find which objects the world loads.
//!
//! Rules from the behaviour note `docs/behaviour/objects.md` ("Where a
//! world's objects come from", answer to Q16; "Script variables, `getLoc`
//! and `move`", Q20); script syntax observed in the data (Daventry
//! `kq/newworld.cs`, `kq/permwrld.cs`, Barren and IceWorld `newWorld.cs`):
//!
//! - Each line runs in order. `#` starts a comment.
//! - A `$name` word that is not the first word of a line is replaced by
//!   the variable's text, split into words again. A word in double quotes
//!   stays one word. `$1`, `$2`... are the arguments of the running script.
//! - A line whose first word is a `.cs` file runs that script fully at that
//!   point (e.g. `WorldVar.cs`, `permwrld.cs $KQGame::runOptimal`).
//! - `set <name> <words...>`: the variable holds the words joined by spaces.
//! - `loadKQ <file> [<name> [x y z [dirX dirY dirZ]]]` loads an object.
//!   With 4 or 5 numbers the extra ones are ignored (angles 0 0 0).
//! - `getLoc <name>` / `KQObject::getLoc <name>` sets `$KQObject::loc`
//!   (`x y z dirX dirY dirZ`) and `$KQObject::locOnly` (`x y z`) from the
//!   object's current place, which the caller knows.
//! - `move <who> <flags> x y z [rx ry rz]` (missing numbers 0) is passed to
//!   the caller.
//! - `KQMonster::inventory`, `give`, `KQObject::setScript` and
//!   `KQObject::setDontLook` (items, M3 step 3e-3, `items.md` rule 9) go to
//!   the caller, which makes or changes the item; a new item's name is put
//!   in `$NewInvItem`.
//! - Everything else (`alias`, `lockShape`, ...) is counted, not run.
//!   `if ... endif` is not evaluated; lines inside such a block are still
//!   run but marked, so the caller can report them.

use std::collections::BTreeMap;

/// What `loadKQ`'s name argument says.
#[derive(Debug, Clone, PartialEq)]
pub enum LoadName {
    /// Missing or `same`: keep the section name from the `.kq` file.
    Keep,
    /// `@`: no name.
    None,
    /// Anything else: the object is renamed to it.
    Rename(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct LoadKq {
    /// File as written, e.g. `terrain/terrain.kq` or `stool.kq`.
    pub file: String,
    pub name: LoadName,
    /// Given position; replaces the `.kq`'s `locX/Y/Z`.
    pub position: Option<[f32; 3]>,
    /// Given angles; when a position is given and these are missing, the
    /// angles are 0 0 0 (the `.kq`'s own `dirX/Y/Z` are not used).
    pub angles: Option<[f32; 3]>,
    /// Numbers after x y z that were ignored (4 or 5 numbers, Q20 e).
    pub ignored_numbers: usize,
    /// Script that holds the line, and its line number (from 1).
    pub from: String,
    pub line: usize,
    /// True if the line sits inside an `if ... endif` block.
    pub inside_if: bool,
}

impl LoadKq {
    /// Angles to use when a position is given (objects.md Q16 rule 2).
    pub fn angles_or_zero(&self) -> [f32; 3] {
        self.angles.unwrap_or([0.0; 3])
    }
}

/// A `move` line (objects.md Q20 d).
#[derive(Debug, Clone, PartialEq)]
pub struct Move {
    pub who: String,
    /// 0 = position and turn absolute, 1 = position relative, 2 = turn
    /// relative, 3 = both relative.
    pub flags: u32,
    pub position: [f32; 3],
    pub turn: [f32; 3],
    pub from: String,
    pub line: usize,
}

/// The script variables (`set`, `getLoc`).
#[derive(Debug, Clone, Default)]
pub struct Vars {
    values: BTreeMap<String, String>,
}

impl Vars {
    pub fn set(&mut self, name: &str, value: String) {
        self.values.insert(name.to_string(), value);
    }

    /// Exact name first; then the same name in other letter case (whether
    /// the game ignores case here is not traced).
    pub fn get(&self, name: &str) -> Option<&str> {
        self.values
            .get(name)
            .or_else(|| self.values.iter().find(|(k, _)| k.eq_ignore_ascii_case(name)).map(|(_, v)| v))
            .map(String::as_str)
    }

    /// A `.kq` value: `$name` is replaced by the variable's text, anything
    /// else is returned as it is. `Err` names an unknown variable.
    pub fn expand_value(&self, value: &str) -> Result<String, String> {
        match value.trim().strip_prefix('$') {
            Some(name) => self.get(name).map(str::to_string).ok_or_else(|| format!("${name}")),
            None => Ok(value.to_string()),
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

/// What the scripts need from the caller.
pub trait Host {
    /// A script's bytes by file name (any case), or `None` if it does not exist.
    fn read(&self, name: &str) -> Option<Vec<u8>>;
    /// Place an object now (in script order). `vars` is for `$variables`
    /// inside the `.kq` file.
    fn load(&mut self, load: &LoadKq, vars: &Vars);
    /// Current position and angles of a named object.
    fn locate(&self, name: &str) -> Option<([f32; 3], [f32; 3])>;
    fn move_object(&mut self, m: &Move);
    /// `KQObject::setHidden <name> 0|1` (objects.md Q18); `at` names the
    /// script line, for the log.
    fn set_hidden(&mut self, name: &str, hidden: bool, at: &str);
    /// `KQMonster::inventory`, `give`, `KQObject::setScript`,
    /// `KQObject::setDontLook` with their words expanded (`words[0]` as
    /// written). Returns a new item's name, for `$NewInvItem`. By default
    /// nothing is done.
    fn item_command(&mut self, _words: &[String], _at: &str) -> Option<String> {
        None
    }
}

#[derive(Debug, Clone, Default)]
pub struct ScriptRun {
    /// `loadKQ` lines in the order the game runs them.
    pub loads: Vec<LoadKq>,
    /// Scripts run, in order.
    pub scripts: Vec<String>,
    /// Scripts named but not found.
    pub missing_scripts: Vec<String>,
    /// Other commands -> how many times they appear (not run).
    pub other_commands: BTreeMap<String, usize>,
    /// Lines that could not be run, with the reason.
    pub bad_lines: Vec<String>,
    /// Unknown variables -> how many times they were used.
    pub unknown_vars: BTreeMap<String, usize>,
    /// `set` lines run.
    pub sets: usize,
    /// `getLoc` lines run, and those whose object was not found.
    pub getlocs: usize,
    pub getloc_not_found: Vec<String>,
    /// `move` lines passed to the caller.
    pub moves: usize,
    /// `KQObject::setHidden` lines passed to the caller.
    pub set_hidden: usize,
    /// Item lines passed to the caller, and how many of them sit inside
    /// an `if` block (run anyway, as every line here).
    pub item_commands: usize,
    pub item_commands_inside_if: usize,
    /// The variables at the end.
    pub vars: Vars,
}

/// Runs `start` (e.g. `newWorld.cs`) with `host`.
pub fn run(start: &str, host: &mut dyn Host) -> ScriptRun {
    let mut out = ScriptRun::default();
    run_script(start, &[], host, &mut out, 0);
    out
}

/// Splits a line (comment already removed) into words; a double-quoted
/// part is one word, without its quotes. The flag says it was quoted.
fn split_words(line: &str) -> Vec<(String, bool)> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c.is_whitespace() {
            chars.next();
        } else if c == '"' {
            chars.next();
            let w: String = chars.by_ref().take_while(|&c| c != '"').collect();
            words.push((w, true));
        } else {
            let mut w = String::new();
            while let Some(&c) = chars.peek() {
                if c.is_whitespace() {
                    break;
                }
                w.push(c);
                chars.next();
            }
            words.push((w, false));
        }
    }
    words
}

/// Replaces `$name` words after the first (Q20 b). Unknown variables are
/// returned as `Err` with their names.
fn expand(words: &[(String, bool)], args: &[String], vars: &Vars) -> Result<Vec<String>, Vec<String>> {
    let mut out = Vec::new();
    let mut unknown = Vec::new();
    for (i, (w, quoted)) in words.iter().enumerate() {
        let name = w.strip_prefix('$').filter(|_| i > 0);
        let Some(name) = name else {
            out.push(w.clone());
            continue;
        };
        let value = match name.parse::<usize>() {
            Ok(n) if n >= 1 => args.get(n - 1).map(String::as_str),
            _ => vars.get(name),
        };
        match value {
            Some(v) if *quoted => out.push(v.to_string()),
            Some(v) => out.extend(v.split_whitespace().map(str::to_string)),
            None => unknown.push(format!("${name}")),
        }
    }
    if unknown.is_empty() { Ok(out) } else { Err(unknown) }
}

fn run_script(name: &str, args: &[String], host: &mut dyn Host, out: &mut ScriptRun, depth: usize) {
    if depth > 16 {
        out.bad_lines.push(format!("{name}: scripts nested more than 16 deep; stopped"));
        return;
    }
    let Some(bytes) = host.read(name) else {
        out.missing_scripts.push(name.to_string());
        return;
    };
    out.scripts.push(name.to_string());
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let mut if_depth = 0usize;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let raw_words = split_words(line);
        let Some((first, _)) = raw_words.first() else { continue };
        let command = first.to_ascii_lowercase();
        let at = format!("{name} line {}", n + 1);
        // Blocks are tracked whatever their condition says (not evaluated).
        match command.as_str() {
            "if" => {
                if_depth += 1;
                continue;
            }
            "endif" => {
                if_depth = if_depth.saturating_sub(1);
                continue;
            }
            "set" if raw_words.len() == 1 => {
                // A bare `set` names nothing to set.
                *out.other_commands.entry("set (alone)".into()).or_default() += 1;
                continue;
            }
            "set" | "loadkq" | "getloc" | "kqobject::getloc" | "move" | "kqobject::sethidden" => {}
            "kqmonster::inventory" | "give" | "kqobject::setscript" | "kqobject::setdontlook" => {}
            c if c.ends_with(".cs") => {}
            // Not run: counted without looking at its variables.
            _ => {
                *out.other_commands.entry(command).or_default() += 1;
                continue;
            }
        }
        let words = match expand(&raw_words, args, &out.vars) {
            Ok(w) => w,
            Err(unknown) => {
                for u in &unknown {
                    *out.unknown_vars.entry(u.clone()).or_default() += 1;
                }
                if command.ends_with(".cs") {
                    // A script still runs; the unknown argument is left out.
                    let known: Vec<(String, bool)> =
                        raw_words.iter().filter(|(w, _)| !w.starts_with('$')).cloned().collect();
                    expand(&known, args, &out.vars).unwrap_or_default()
                } else {
                    out.bad_lines.push(format!("{at}: unknown variable {}: '{line}'", unknown.join(", ")));
                    continue;
                }
            }
        };
        match command.as_str() {
            "set" => match words.get(1) {
                Some(var) => {
                    out.vars.set(var, words[2..].join(" "));
                    out.sets += 1;
                }
                None => out.bad_lines.push(format!("{at}: set without a name")),
            },
            "loadkq" => match parse_load(&words[1..]) {
                Ok((file, name_arg, position, angles, ignored_numbers)) => {
                    let load = LoadKq {
                        file,
                        name: name_arg,
                        position,
                        angles,
                        ignored_numbers,
                        from: name.to_string(),
                        line: n + 1,
                        inside_if: if_depth > 0,
                    };
                    host.load(&load, &out.vars);
                    out.loads.push(load);
                }
                Err(e) => out.bad_lines.push(format!("{at}: {e}: '{line}'")),
            },
            "getloc" | "kqobject::getloc" => match words.get(1) {
                Some(who) => {
                    out.getlocs += 1;
                    match host.locate(who) {
                        Some((p, a)) => {
                            let f = |v: &[f32]| v.iter().map(|x| x.to_string()).collect::<Vec<_>>().join(" ");
                            out.vars.set("KQObject::loc", format!("{} {}", f(&p), f(&a)));
                            out.vars.set("KQObject::locOnly", f(&p));
                        }
                        None => out.getloc_not_found.push(format!("{at}: {who}")),
                    }
                }
                None => out.bad_lines.push(format!("{at}: getLoc without a name")),
            },
            "kqobject::sethidden" => match (words.get(1), words.get(2).map(|v| v.parse::<i64>())) {
                (Some(who), Some(Ok(v))) => {
                    out.set_hidden += 1;
                    host.set_hidden(who, v != 0, &at);
                }
                _ => out.bad_lines.push(format!("{at}: setHidden needs a name and a whole number: '{line}'")),
            },
            "move" => match parse_move(&words[1..]) {
                Ok((who, flags, position, turn)) => {
                    out.moves += 1;
                    host.move_object(&Move { who, flags, position, turn, from: name.to_string(), line: n + 1 });
                }
                Err(e) => out.bad_lines.push(format!("{at}: {e}: '{line}'")),
            },
            "kqmonster::inventory" | "give" | "kqobject::setscript" | "kqobject::setdontlook" => {
                out.item_commands += 1;
                if if_depth > 0 {
                    out.item_commands_inside_if += 1;
                }
                if let Some(name) = host.item_command(&words, &at) {
                    out.vars.set("NewInvItem", name);
                }
            }
            _ => run_script(first, &words[1..], host, out, depth + 1),
        }
    }
}

type Parsed = (String, LoadName, Option<[f32; 3]>, Option<[f32; 3]>, usize);

fn parse_load(args: &[String]) -> Result<Parsed, String> {
    let file = args.first().ok_or("loadKQ without a file")?.to_string();
    let name = match args.get(1).map(String::as_str) {
        None => LoadName::Keep,
        Some(n) if n.eq_ignore_ascii_case("same") => LoadName::Keep,
        Some("@") => LoadName::None,
        Some(n) => LoadName::Rename(n.to_string()),
    };
    let nums = args.get(2..).unwrap_or(&[]);
    let parse = |s: &str| s.parse::<f32>().map_err(|_| format!("'{s}' is not a number"));
    let position = match nums.len() {
        0 => None,
        1 | 2 => return Err(format!("{} position values, need 3", nums.len())),
        _ => Some([parse(&nums[0])?, parse(&nums[1])?, parse(&nums[2])?]),
    };
    // Angles only with all three (Q20 e); with 4 or 5 numbers the extra
    // ones are ignored.
    let (angles, ignored) = match nums.len() {
        0..=3 => (None, 0),
        4 | 5 => (None, nums.len() - 3),
        6 => (Some([parse(&nums[3])?, parse(&nums[4])?, parse(&nums[5])?]), 0),
        n => return Err(format!("{n} numbers; expected at most 6")),
    };
    Ok((file, name, position, angles, ignored))
}

/// `move <who> <flags> x y z [rx ry rz]` after the command word: up to
/// six numbers, missing ones 0 (`objects.md` "(d)").
pub fn parse_move(args: &[String]) -> Result<(String, u32, [f32; 3], [f32; 3]), String> {
    let who = args.first().ok_or("move without a name")?.to_string();
    let flags = args.get(1).ok_or("move without flags")?;
    let flags = flags.parse::<u32>().map_err(|_| format!("flags '{flags}' is not a whole number"))?;
    let nums = args.get(2..).unwrap_or(&[]);
    if nums.len() > 6 {
        return Err(format!("{} numbers; expected at most 6", nums.len()));
    }
    let mut v = [0.0f32; 6];
    for (slot, s) in v.iter_mut().zip(nums) {
        *slot = s.parse::<f32>().map_err(|_| format!("'{s}' is not a number"))?;
    }
    Ok((who, flags, [v[0], v[1], v[2]], [v[3], v[4], v[5]]))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Records loads; objects are where their `loadKQ` line put them.
    struct TestHost {
        files: Vec<(&'static str, &'static str)>,
        placed: Vec<(String, [f32; 3], [f32; 3])>,
        moves: Vec<Move>,
        kq_values: Vec<String>,
        hidden: Vec<(String, bool)>,
        items: Vec<String>,
    }

    impl TestHost {
        fn new(files: Vec<(&'static str, &'static str)>) -> Self {
            TestHost { files, placed: Vec::new(), moves: Vec::new(), kq_values: Vec::new(), hidden: Vec::new(), items: Vec::new() }
        }
    }

    impl Host for TestHost {
        fn read(&self, name: &str) -> Option<Vec<u8>> {
            self.files.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, t)| t.as_bytes().to_vec())
        }
        fn load(&mut self, load: &LoadKq, vars: &Vars) {
            let name = match &load.name {
                LoadName::Rename(n) => n.clone(),
                _ => load.file.trim_end_matches(".kq").to_string(),
            };
            self.placed.push((name, load.position.unwrap_or([0.0; 3]), load.angles_or_zero()));
            self.kq_values.push(vars.expand_value("$Seller::xPos").unwrap_or_default());
        }
        fn locate(&self, name: &str) -> Option<([f32; 3], [f32; 3])> {
            self.placed.iter().rev().find(|(n, _, _)| n.eq_ignore_ascii_case(name)).map(|(_, p, a)| (*p, *a))
        }
        fn move_object(&mut self, m: &Move) {
            self.moves.push(m.clone());
        }
        fn set_hidden(&mut self, name: &str, hidden: bool, _at: &str) {
            self.hidden.push((name.to_string(), hidden));
        }
        fn item_command(&mut self, words: &[String], _at: &str) -> Option<String> {
            self.items.push(words.join(" "));
            let made = words[0].eq_ignore_ascii_case("give") || words[0].eq_ignore_ascii_case("KQMonster::inventory");
            made.then(|| format!("item.{}", self.items.len()))
        }
    }

    fn daventry_like() -> TestHost {
        TestHost::new(vec![
            (
                "newworld.cs",
                "# start\nWorldVar.cs\nloadKQ terrain/terrain.kq\nloadKQ connor.kq same 1 2 -1 0 0 0 # by cottage\npermwrld.cs $KQGame::runOptimal\nloadKQ ax.kq\nalias x \"move\"\n",
            ),
            (
                "permwrld.cs",
                "loadKQ\tstool.kq\tstool.02\t260045\t225746\t13981\t0\t0\t-0.81\nloadKQ a.kq @ 5 6 7\nif test $1 == yes\nloadKQ hidden.kq\nendif\nloadKQ bad.kq x 1 2\n",
            ),
        ])
    }

    #[test]
    fn follows_scripts_in_order() {
        let mut host = daventry_like();
        let r = run("newWorld.cs", &mut host);
        let files: Vec<&str> = r.loads.iter().map(|l| l.file.as_str()).collect();
        assert_eq!(files, ["terrain/terrain.kq", "connor.kq", "stool.kq", "a.kq", "hidden.kq", "ax.kq"]);
        assert_eq!(host.placed.len(), 6);
        assert_eq!(r.scripts, ["newWorld.cs", "permwrld.cs"]);
        assert_eq!(r.missing_scripts, ["WorldVar.cs"]);
        assert_eq!(r.other_commands.get("alias"), Some(&1));
        // `$KQGame::runOptimal` is unknown: permwrld.cs still runs.
        assert_eq!(r.unknown_vars.get("$KQGame::runOptimal"), Some(&1));
        assert_eq!(r.bad_lines.len(), 1, "{:?}", r.bad_lines);
    }

    #[test]
    fn name_position_and_angle_rules() {
        let mut host = daventry_like();
        let r = run("newWorld.cs", &mut host);
        let by = |f: &str| r.loads.iter().find(|l| l.file == f).unwrap().clone();
        let connor = by("connor.kq");
        assert_eq!(connor.name, LoadName::Keep);
        assert_eq!(connor.position, Some([1.0, 2.0, -1.0]));
        let stool = by("stool.kq");
        assert_eq!(stool.name, LoadName::Rename("stool.02".into()));
        assert_eq!(stool.angles, Some([0.0, 0.0, -0.81]));
        let a = by("a.kq");
        assert_eq!(a.name, LoadName::None);
        assert_eq!((a.angles, a.angles_or_zero()), (None, [0.0; 3]));
        assert!(by("hidden.kq").inside_if);
        let ax = by("ax.kq");
        assert_eq!((ax.position, ax.inside_if), (None, false));
    }

    #[test]
    fn variables_getloc_and_move() {
        let mut host = TestHost::new(vec![
            ("worldvar.cs", "set Seller::xPos 144476 # comment\nset ChestX 279587\nset Pos $ChestX 2 3 0 0 -2.9\n"),
            (
                "newworld.cs",
                "worldvar.cs\nloadKQ tw_weird.kq Weirdling 144476 131239 1744 0 0 -1.43\ngetLoc Weirdling\nloadKQ anvil.kq same $KQObject::loc\nKQObject::getLoc nobody\nloadKQ chest.kq chest $Pos\nKQObject::setHidden chest 1\nmove chest 3 0 0 2000\nmove chest 0 $KQObject::locOnly\nloadKQ grass.kq swmp.01 166759 68128 -2 -1\nsub.cs hello.kq\nloadKQ q.kq \"$Pos\"\n",
            ),
            ("sub.cs", "loadKQ $1\n"),
        ]);
        let r = run("newWorld.cs", &mut host);
        assert!(r.bad_lines.is_empty(), "{:?}", r.bad_lines);
        assert_eq!((r.sets, r.getlocs, r.moves, r.set_hidden), (3, 2, 2, 1));
        assert_eq!(host.hidden, [("chest".to_string(), true)]);
        assert_eq!(r.getloc_not_found.len(), 1, "{:?}", r.getloc_not_found);
        // getLoc -> six numbers -> position and angles.
        let anvil = &r.loads[1];
        assert_eq!((anvil.position, anvil.angles), (Some([144476.0, 131239.0, 1744.0]), Some([0.0, 0.0, -1.43])));
        assert_eq!(r.vars.get("KQObject::locOnly"), Some("144476 131239 1744"));
        // `set` joins the words after replacing variables.
        assert_eq!(r.loads[2].position, Some([279587.0, 2.0, 3.0]));
        assert_eq!(r.loads[2].angles, Some([0.0, 0.0, -2.9]));
        // move: flags and up to six numbers, missing ones 0.
        assert_eq!((host.moves[0].flags, host.moves[0].position, host.moves[0].turn), (3, [0.0, 0.0, 2000.0], [0.0; 3]));
        assert_eq!(host.moves[1].position, [144476.0, 131239.0, 1744.0]);
        // 4 numbers: x y z, the extra one ignored, angles 0 0 0.
        let grass = &r.loads[3];
        assert_eq!((grass.position, grass.angles, grass.ignored_numbers), (Some([166759.0, 68128.0, -2.0]), None, 1));
        // A script's argument as $1.
        assert_eq!(r.loads[4].file, "hello.kq");
        // In quotes the variable stays one word (here: the name).
        assert_eq!(r.loads[5].name, LoadName::Rename("279587 2 3 0 0 -2.9".into()));
        // `.kq` values can be variables.
        assert_eq!(host.kq_values[0], "144476");
    }

    #[test]
    fn item_lines_go_to_the_host_and_set_newinvitem() {
        let mut host = TestHost::new(vec![(
            "newworld.cs",
            "KQMonster::inventory\tworld\tDagger\t1\t260784\t229385\t14935\t-1.87\t1.57\t75\nKQObject::setScript $NewInvItem dagger.cs\nKQObject::setDontLook $NewInvItem 1\nKQObject::setHidden $NewInvItem 1\nif test 1 == 2\ngive mushroom 5\nendif\nset Last $NewInvItem\n",
        )]);
        let r = run("newWorld.cs", &mut host);
        assert!(r.bad_lines.is_empty(), "{:?}", r.bad_lines);
        assert_eq!(
            host.items,
            [
                "KQMonster::inventory world Dagger 1 260784 229385 14935 -1.87 1.57 75",
                "KQObject::setScript item.1 dagger.cs",
                "KQObject::setDontLook item.1 1",
                "give mushroom 5"
            ]
        );
        assert_eq!(host.hidden, [("item.1".to_string(), true)]);
        assert_eq!((r.item_commands, r.item_commands_inside_if), (4, 1));
        assert_eq!(r.vars.get("Last"), Some("item.4"));
    }
}
