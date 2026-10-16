//! Running the game's `.cs` scripts while the game plays (M3 step 3a-1),
//! only as much of the language as the door scripts need.
//!
//! Rules from `docs/behaviour/doors.md` "The script language" (traced):
//!
//! - A script runs line by line, to the end, when it is called. A line is
//!   split into words at spaces and tabs; `#` starts a comment; a
//!   double-quoted part is one word.
//! - `$1` … `$n` are the call's arguments (missing ones are empty);
//!   `$Some::Name` is a variable. Both are replaced by their text, split
//!   into words again, so an unset or empty one **vanishes** from the line.
//!   Quoted (`"$X"`) it stays one word, possibly empty.
//! - `if <command…>` runs the rest of the line and takes the branch when
//!   `$Console::Return` is 0 or empty; `else` switches branch; `endif`
//!   closes. `if`s inside a skipped part are counted, not run. `else if` is
//!   an `else` followed by a nested `if`, which needs its own `endif`.
//! - `test <a> <op> <b>`: exactly three words, else a usage line and
//!   false. Both sides look like numbers (a digit first, or `-`/`.` then a
//!   digit): compared as numbers; otherwise as text, capitals counting.
//!   `&&` / `||` work on whole numbers only.
//! - Command names are matched exactly. A first word that is no command
//!   but names a script (`.cs` added when it has no dot) runs that script
//!   with the rest of the line as its arguments. An unknown command logs
//!   an error and fails.
//! - Every command leaves a result in `$Console::Return`.
//!
//! Confirmed by the study session (`doors.md` "Q45" e, f): the result is
//! `0` for true or success and `1` for false or failure; an unknown command
//! leaves `1`; a script call sets nothing itself (its last command's result
//! stays); a word that looks like a number is read up to where the number
//! ends (`-.5` is text). A false text `==` leaves `-1` when the left word
//! comes first in byte order, `1` otherwise; every other false test leaves
//! `1` (Q54, traced). `set` and `echo` leave `0` (Q54 b, traced).
//!
//! Commands that act on the world (`sendEvent`, `KQObject::collisionType`)
//! go to the caller ([`ScriptHost::world_command`]). Commands of later
//! milestones are stubs: one log line ([`STUBS`]).

use kq8_data::world_script::Vars;

/// Commands of later milestones: the door scripts' (dialogue, GUI,
/// inventory, cutscenes; `doors.md` "Commands door scripts use"), then
/// those of Daventry's trap scripts that only act (music, monsters, ropes,
/// cutscene camera, moving objects while playing; M3 step 3d-3), then the
/// item commands of later milestones (M3 step 3e-2: `tossItems` needs
/// weapons to smash barrels and its flight is not fully traced, M5;
/// `dropItem` is only used by `rockHit.cs`, M5). Each only writes a log
/// line and leaves 0 in `$Console::Return`. `KQMonster::checkInventory`,
/// `deleteObject` and `KQObject::setHidden` were stubs until 3e-2; the
/// world handles them now. Commands that answer a question (`getloc`,
/// `checkDistance`, `exist`, `checkSave`, ...) are not stubs: they stay
/// unknown (error line, result 1) until they are studied.
pub const STUBS: [&str; 41] = [
    "KQMonster::speak",
    "doPopup",
    "addMapIcon",
    "earnTeleport",
    "teleportersEarned",
    "handsOff",
    "KQMonster::setState",
    "pauseAnim",
    "preloadShape",
    "loadKQ",
    // Daventry's trap scripts (3d-3).
    "KQMusic::push",
    "KQMusic::pop",
    "KQMonster::setHome",
    "KQMonster::setHomeRadius",
    "KQMonster::setChaseRadius",
    "KQMonster::setAlarmRadius",
    "KQMonster::goHome",
    "KQMonster::stateChangeNotify",
    "KQMonster::clearStateChangeNotify",
    "KQConner::setRope",
    "KQConner::setRopeCam",
    "KQConner::setRopeCam2",
    "KQConner::setRopeCamAngle",
    "KQConner::setDoMode",
    "KQConner::walkTo",
    "KQConner::endJump",
    "KQConner::JumpDir",
    "KQCamera::follow",
    "KQCamera::flyToCamLoc",
    "KQCamera::addCamLoc",
    "KQGame::letCatchUp",
    "setThirdPerson",
    "move",
    "onTeleport",
    "offTeleport",
    "pointAndCue",
    // Item commands of later milestones (3e-2).
    "KQObject::tossItems",
    "KQMonster::dropItem",
    "KQObject::changeBmp",
    "KQObject::setGroupsAlways",
    // No audio yet (3f: `fishies.cs`).
    "KQSound::play",
];

/// Scripts calling scripts deeper than this stop (a guard of ours).
const MAX_DEPTH: usize = 16;

/// The variable every command leaves its result in.
pub const RETURN: &str = "Console::Return";

/// What a running script needs from the game.
pub trait ScriptHost {
    /// A script's bytes by file name (any case), `None` if there is none.
    fn read(&self, name: &str) -> Option<Vec<u8>>;
    /// A command that acts on the world (`sendEvent`, ...), with its words
    /// already expanded (`words[0]` is the command). Returns its result
    /// for `$Console::Return`, or `None` if it is not such a command.
    fn world_command(&mut self, words: &[String], at: &str) -> Option<String>;
    /// One line for the log.
    fn log(&mut self, line: String);
    /// Variables a world command set (`NewInvItem`, `checkInventory`'s
    /// fourth word), taken right after it ran.
    fn take_vars(&mut self) -> Vec<(String, String)> {
        Vec::new()
    }
}

/// The scripts' shared state: the variables (set by the start-up scripts
/// and by every script since).
#[derive(Debug, Clone, Default)]
pub struct Scripts {
    pub vars: Vars,
}

/// One `if` level: whether the lines in it run.
#[derive(Debug, Clone, Copy)]
struct Block {
    /// The level around it was running.
    outer_runs: bool,
    /// The branch being read is the one taken.
    taking: bool,
}

impl Block {
    fn runs(&self) -> bool {
        self.outer_runs && self.taking
    }
}

impl Scripts {
    pub fn new(vars: Vars) -> Self {
        Scripts { vars }
    }

    /// Runs script `name` with `args` as `$1`, `$2`, ... Returns false if
    /// the script was not found (logged).
    pub fn call(&mut self, name: &str, args: &[String], host: &mut dyn ScriptHost) -> bool {
        self.run(name, args, host, 0)
    }

    fn run(&mut self, name: &str, args: &[String], host: &mut dyn ScriptHost, depth: usize) -> bool {
        if depth > MAX_DEPTH {
            host.log(format!("script {name}: scripts nested more than {MAX_DEPTH} deep; not run"));
            return false;
        }
        let Some(bytes) = host.read(name) else {
            host.log(format!("script {name}: not found"));
            return false;
        };
        let text: String = bytes.iter().map(|&b| b as char).collect();
        let mut blocks: Vec<Block> = Vec::new();
        for (n, raw) in text.lines().enumerate() {
            let line = raw.split('#').next().unwrap_or("");
            let words = split_words(line);
            if words.is_empty() {
                continue;
            }
            let at = format!("{name} line {}", n + 1);
            self.line(&words, args, &mut blocks, host, &at, depth);
        }
        if !blocks.is_empty() {
            host.log(format!("script {name}: {} if-block(s) not closed at the end", blocks.len()));
        }
        true
    }

    /// One line (or the rest of an `else` line).
    fn line(
        &mut self,
        words: &[(String, bool)],
        args: &[String],
        blocks: &mut Vec<Block>,
        host: &mut dyn ScriptHost,
        at: &str,
        depth: usize,
    ) {
        let runs = blocks.last().is_none_or(Block::runs);
        match words[0].0.as_str() {
            "if" => {
                if !runs {
                    // Counted, not run, so its else/endif pair up.
                    blocks.push(Block { outer_runs: false, taking: false });
                    return;
                }
                let command = self.expand(&words[1..], args);
                if command.is_empty() {
                    host.log(format!("{at}: if without a command"));
                    self.vars.set(RETURN, "1".into());
                } else {
                    self.command(&command, host, at, depth);
                }
                let result = self.vars.get(RETURN).unwrap_or("");
                let taking = result.is_empty() || result == "0";
                blocks.push(Block { outer_runs: true, taking });
            }
            "else" => {
                match blocks.last_mut() {
                    Some(b) => b.taking = !b.taking,
                    None => host.log(format!("{at}: else without an if (ignored)")),
                }
                // `else if ...`: the rest is a line of its own, a nested if.
                if words.len() > 1 {
                    self.line(&words[1..], args, blocks, host, at, depth);
                }
            }
            "endif" => {
                if blocks.pop().is_none() {
                    host.log(format!("{at}: endif without an if (ignored)"));
                }
            }
            _ if runs => {
                let mut command = vec![words[0].0.clone()];
                command.extend(self.expand(&words[1..], args));
                self.command(&command, host, at, depth);
            }
            _ => {}
        }
    }

    /// Replaces `$` words: arguments and variables, split into words again
    /// unless quoted.
    fn expand(&self, words: &[(String, bool)], args: &[String]) -> Vec<String> {
        let mut out = Vec::new();
        for (w, quoted) in words {
            let Some(name) = w.strip_prefix('$') else {
                out.push(w.clone());
                continue;
            };
            let value = match name.parse::<usize>() {
                Ok(n) if n >= 1 => args.get(n - 1).map_or("", String::as_str),
                _ => self.vars.get(name).unwrap_or(""),
            };
            if *quoted {
                out.push(value.to_string());
            } else {
                out.extend(value.split_whitespace().map(str::to_string));
            }
        }
        out
    }

    /// Runs one command (words expanded) and leaves its result in
    /// `$Console::Return`.
    fn command(&mut self, words: &[String], host: &mut dyn ScriptHost, at: &str, depth: usize) {
        let result = match words[0].as_str() {
            "test" => {
                let (result, problem) = test_return(&words[1..]);
                if let Some(p) = problem {
                    host.log(format!("{at}: {p}"));
                }
                Some(result.to_string())
            }
            "set" => match words.get(1) {
                Some(var) => {
                    let value = words[2..].join(" ");
                    host.log(format!("{at}: set {var} = '{value}'"));
                    self.vars.set(var, value);
                    Some("0".into())
                }
                None => {
                    host.log(format!("{at}: set without a name"));
                    Some("1".into())
                }
            },
            "echo" => {
                host.log(format!("{at}: echo {}", words[1..].join(" ")));
                Some("0".into())
            }
            c if STUBS.contains(&c) => {
                host.log(format!("{at}: stub (later milestone, nothing done): {}", words.join(" ")));
                Some("0".into())
            }
            _ => {
                let r = host.world_command(words, at);
                for (var, value) in host.take_vars() {
                    self.vars.set(&var, value);
                }
                r
            }
        };
        if let Some(r) = result {
            self.vars.set(RETURN, r);
            return;
        }
        // Not a command: a script by that name?
        let first = &words[0];
        let script = if first.contains('.') { first.clone() } else { format!("{first}.cs") };
        if host.read(&script).is_some() {
            host.log(format!("{at}: runs {script} {}", words[1..].join(" ")));
            self.run(&script, &words[1..], host, depth + 1);
        } else {
            host.log(format!("{at}: ERROR unknown command '{first}': {}", words.join(" ")));
            self.vars.set(RETURN, "1".into());
        }
    }
}

/// Splits a line (comment removed) into words; a double-quoted part is one
/// word without its quotes, flagged as quoted.
pub fn split_words(line: &str) -> Vec<(String, bool)> {
    let mut words = Vec::new();
    let mut chars = line.chars().peekable();
    while let Some(&c) = chars.peek() {
        if c == ' ' || c == '\t' || c.is_whitespace() {
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

/// The note's "looks like a number": a digit first, or `-` or `.`
/// followed by a digit.
pub fn looks_like_number(s: &str) -> bool {
    let b = s.as_bytes();
    match b.first() {
        Some(c) if c.is_ascii_digit() => true,
        Some(b'-' | b'.') => b.get(1).is_some_and(u8::is_ascii_digit),
        _ => false,
    }
}

/// The value of a word that looks like a number: its leading number part
/// (`doors.md` Q45 f).
pub fn number_value(s: &str) -> f64 {
    (1..=s.len()).rev().filter(|&n| s.is_char_boundary(n)).find_map(|n| s[..n].parse::<f64>().ok()).unwrap_or(0.0)
}

/// `test <a> <op> <b>`: (true?, a problem to log).
pub fn test(words: &[String]) -> (bool, Option<String>) {
    let [a, op, b] = words else {
        return (false, Some(format!("usage: test <a> <op> <b> (got {} word(s): {:?}); false", words.len(), words)));
    };
    let numbers = looks_like_number(a) && looks_like_number(b);
    if op == "&&" || op == "||" {
        if !numbers {
            return (false, Some(format!("test {a} {op} {b}: && and || need whole numbers; false")));
        }
        let (x, y) = (number_value(a).trunc() != 0.0, number_value(b).trunc() != 0.0);
        return (if op == "&&" { x && y } else { x || y }, None);
    }
    let order = if numbers {
        number_value(a).partial_cmp(&number_value(b)).unwrap_or(std::cmp::Ordering::Less)
    } else {
        a.as_bytes().cmp(b.as_bytes())
    };
    use std::cmp::Ordering::*;
    let ok = match op.as_str() {
        "==" => order == Equal,
        "!=" => order != Equal,
        "<" => order == Less,
        "<=" => order != Greater,
        ">" => order == Greater,
        ">=" => order != Less,
        _ => return (false, Some(format!("test {a} {op} {b}: unknown operator; false"))),
    };
    (ok, None)
}

/// What `test` leaves in `$Console::Return` (`doors.md` Q45 e, Q54):
/// `0` when true; when false `1`, except a false text `==` whose left
/// word comes first in byte order, which leaves `-1`.
pub fn test_return(words: &[String]) -> (&'static str, Option<String>) {
    let (ok, problem) = test(words);
    if ok {
        return ("0", problem);
    }
    let left_first = match words {
        [a, op, b] if op == "==" && !(looks_like_number(a) && looks_like_number(b)) => a.as_bytes() < b.as_bytes(),
        _ => false,
    };
    (if left_first { "-1" } else { "1" }, problem)
}

/// How a script's `if` / `else` / `endif` lines pair up, counted without
/// running anything (for the load log): (if count, endif count, problems).
pub fn block_balance(text: &str) -> (usize, usize, Vec<String>) {
    let (mut ifs, mut endifs, mut depth) = (0usize, 0usize, 0i64);
    let mut problems = Vec::new();
    for (n, raw) in text.lines().enumerate() {
        let words = split_words(raw.split('#').next().unwrap_or(""));
        let mut i = 0;
        while let Some((w, _)) = words.get(i) {
            match w.as_str() {
                "else" => {
                    if depth == 0 {
                        problems.push(format!("line {}: else without an if", n + 1));
                    }
                    i += 1;
                    continue;
                }
                "if" => {
                    ifs += 1;
                    depth += 1;
                }
                "endif" => {
                    endifs += 1;
                    depth -= 1;
                    if depth < 0 {
                        problems.push(format!("line {}: endif without an if", n + 1));
                        depth = 0;
                    }
                }
                _ => {}
            }
            break;
        }
    }
    if depth > 0 {
        problems.push(format!("{depth} if-block(s) not closed at the end"));
    }
    (ifs, endifs, problems)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Made-up scripts (never copies of the game's files); records world
    /// commands and log lines.
    struct Host {
        files: Vec<(&'static str, &'static str)>,
        commands: Vec<String>,
        log: Vec<String>,
    }

    impl Host {
        fn new(files: Vec<(&'static str, &'static str)>) -> Self {
            Host { files, commands: Vec::new(), log: Vec::new() }
        }
    }

    impl ScriptHost for Host {
        fn read(&self, name: &str) -> Option<Vec<u8>> {
            self.files.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, t)| t.as_bytes().to_vec())
        }
        fn world_command(&mut self, words: &[String], _at: &str) -> Option<String> {
            (words[0] == "sendEvent" || words[0] == "KQObject::collisionType").then(|| {
                self.commands.push(words.join(" "));
                "0".to_string()
            })
        }
        fn log(&mut self, line: String) {
            self.log.push(line);
        }
    }

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    fn run(script: &'static str, a: &[&str], vars: &[(&str, &str)]) -> (Scripts, Host) {
        let mut host = Host::new(vec![("s.cs", script)]);
        let mut v = Vars::default();
        for (k, val) in vars {
            v.set(k, val.to_string());
        }
        let mut s = Scripts::new(v);
        assert!(s.call("s.cs", &args(a), &mut host));
        (s, host)
    }

    #[test]
    fn test_compares_numbers_and_text() {
        let t = |a: &str, op: &str, b: &str| test(&args(&[a, op, b])).0;
        assert!(t("1.0", "==", "1"));
        assert!(t("-2", "<", "1"));
        assert!(t(".5", ">", "0"));
        assert!(!t("true", "==", "True"), "capitals count");
        assert!(t("true", "==", "true"));
        assert!(t("abc", "<", "abd"));
        assert!(t("10", ">", "9"), "numbers, not text");
        assert!(t("1", "&&", "2"));
        assert!(!t("1", "&&", "0"));
        assert!(t("0", "||", "3"));
        let (ok, problem) = test(&args(&["a", "&&", "1"]));
        assert!(!ok && problem.is_some(), "&& needs numbers");
        // Wrong word count: usage, false.
        let (ok, problem) = test(&args(&["==", "0"]));
        assert!(!ok && problem.is_some());
        // A word looks like a number by its first characters only.
        assert!(looks_like_number("12abc") && !looks_like_number("-x") && !looks_like_number("-.5"));
        assert_eq!(number_value("12abc"), 12.0);
    }

    #[test]
    fn a_false_text_equal_leaves_minus_one_when_the_left_word_comes_first() {
        let r = |a: &str, op: &str, b: &str| test_return(&args(&[a, op, b])).0;
        assert_eq!(r("abc", "==", "abd"), "-1");
        assert_eq!(r("abd", "==", "abc"), "1");
        assert_eq!(r("ab", "==", "abc"), "-1", "a word that runs out first comes first");
        assert_eq!(r("Abc", "==", "abc"), "-1", "capitals before small letters");
        assert_eq!(r("abc", "==", "abc"), "0");
        // Only `==` gives -1; numbers never do.
        assert_eq!(r("abc", "!=", "abc"), "1");
        assert_eq!(r("abd", "<", "abc"), "1");
        assert_eq!(r("abc", ">", "abd"), "1");
        assert_eq!(r("1", "==", "2"), "1");
        // Run through a script: `if` treats -1 as false.
        let (s, _) = run("test abc == abd
set R $Console::Return
if test abc == abd
 set T yes
else
 set T no
endif
", &[], &[]);
        assert_eq!(s.vars.get("R"), Some("-1"));
        assert_eq!(s.vars.get("T"), Some("no"));
    }

    #[test]
    fn unset_variables_vanish_quoted_ones_stay() {
        // `test $X == 0` with X unset becomes `test == 0`: false.
        let (s, h) = run("if test $X == 0\nset A yes\nelse\nset A no\nendif\n", &[], &[]);
        assert_eq!(s.vars.get("A"), Some("no"));
        assert!(h.log.iter().any(|l| l.contains("usage")), "{:?}", h.log);
        // Quoted: one (empty) word, so the test has three words.
        let (s, _) = run("if test \"$X\" == 0\nset A yes\nelse\nset A no\nendif\n", &[], &[]);
        assert_eq!(s.vars.get("A"), Some("no"));
        let (s, _) = run("if test \"$X\" == 0\nset A yes\nendif\n", &[], &[("X", "0")]);
        assert_eq!(s.vars.get("A"), Some("yes"));
        // An empty variable vanishes too; arguments past the given ones are empty.
        let (s, _) = run("set B $X$\nset C $5 end\n", &["one"], &[("X", "")]);
        assert_eq!(s.vars.get("C"), Some("end"));
    }

    #[test]
    fn else_if_needs_its_own_endif() {
        // The shape of the door scripts: else if ... endif endif.
        let script = "if test $2 == ConnorAction\nset R action\nelse if test $2 == Die\nset R die\nelse\nset R other\nendif\nendif\nset After 1\n";
        for (arg, want) in [("ConnorAction", "action"), ("Die", "die"), ("Open", "other")] {
            let (s, h) = run(script, &["door", arg], &[]);
            assert_eq!(s.vars.get("R"), Some(want), "{arg}");
            assert_eq!(s.vars.get("After"), Some("1"));
            assert!(!h.log.iter().any(|l| l.contains("not closed") || l.contains("without")), "{:?}", h.log);
        }
        assert_eq!(block_balance(script), (2, 2, vec![]));
        assert_eq!(block_balance("if a\nendif\nendif\n").2.len(), 1);
    }

    #[test]
    fn skipped_ifs_are_counted_not_run() {
        let script = "if test 1 == 2\n if test 1 == 1\n  set A inner\n else\n  set A inner_else\n endif\n set B skipped\nelse\n set C taken\nendif\n";
        let (s, h) = run(script, &[], &[]);
        assert_eq!((s.vars.get("A"), s.vars.get("B"), s.vars.get("C")), (None, None, Some("taken")));
        // The skipped `if` ran no test (nothing logged for it).
        assert!(h.log.iter().all(|l| !l.contains("line 2")), "{:?}", h.log);
    }

    #[test]
    fn console_return_and_if_on_any_command() {
        let (s, _) = run("test a == b\nset Saved $Console::Return\nif KQMonster::speak Connor Connor 0 1 2 3 1 1\nset Inv zero\nendif\n", &[], &[]);
        assert_eq!(s.vars.get("Saved"), Some("-1"), "a false text == with the left word first (Q54)");
        assert_eq!(s.vars.get("Inv"), Some("zero"), "a stub leaves 0, so the branch is taken");
        // An unknown command fails; its `if` branch is skipped.
        let (s, h) = run("if noSuchCommand 1\nset A ran\nendif\n", &[], &[]);
        assert_eq!(s.vars.get("A"), None);
        assert!(h.log.iter().any(|l| l.contains("ERROR unknown command 'noSuchCommand'")));
        // Two words after `test` with a `>1` stuck together: false.
        let (s, _) = run("set N 5\nif test $N >1\nset A yes\nelse\nset A no\nendif\n", &[], &[]);
        assert_eq!(s.vars.get("A"), Some("no"));
    }

    #[test]
    fn scripts_call_scripts_with_arguments_and_world_commands_go_to_the_host() {
        let mut host = Host::new(vec![
            ("door.cs", "echo $1 $2 $3 $4\nhelper.cs $1 KQDoorEvent\nhelper $1 again\nKQObject::collisionType $1 -32\n"),
            ("helper.cs", "sendEvent 0 $1 $2 1\n"),
        ]);
        let mut s = Scripts::default();
        s.call("door.cs", &args(&["Condoor", "ConnorAction", "do", "do"]), &mut host);
        assert_eq!(host.commands, ["sendEvent 0 Condoor KQDoorEvent 1", "sendEvent 0 Condoor again 1", "KQObject::collisionType Condoor -32"]);
        assert!(host.log.iter().any(|l| l.ends_with("echo Condoor ConnorAction do do")), "{:?}", host.log);
        // A missing script is reported.
        assert!(!s.call("gone.cs", &[], &mut host));
    }

    #[test]
    fn set_joins_words_and_comments_are_dropped() {
        let (s, h) = run("set Daventry::Door::Locked 1 # comment\nset Long a  b\tc\necho \"Keep Door\" x # not this\n", &[], &[]);
        assert_eq!(s.vars.get("Daventry::Door::Locked"), Some("1"));
        assert_eq!(s.vars.get("Long"), Some("a b c"));
        assert!(h.log.iter().any(|l| l.ends_with("echo Keep Door x")), "{:?}", h.log);
    }
}
