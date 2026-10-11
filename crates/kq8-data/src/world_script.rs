//! Following a world's start-up scripts (`newWorld.cs` and the scripts it
//! names) to find which objects the world loads.
//!
//! Rules from the behaviour note `docs/behaviour/objects.md` ("Where a
//! world's objects come from", answer to Q16); script syntax observed in
//! the data (Daventry `kq/newworld.cs`, `kq/permwrld.cs`):
//!
//! - Each line runs in order. `#` starts a comment.
//! - A line whose first word is a `.cs` file runs that script fully at that
//!   point (e.g. `WorldVar.cs`, `permwrld.cs $KQGame::runOptimal`).
//! - `loadKQ <file> [<name> [x y z [dirX dirY dirZ]]]` loads an object.
//! - Everything else (`set`, `alias`, `lockShape`, ...) is counted, not run.
//!   `if ... endif` is not evaluated; `loadKQ` lines inside such a block are
//!   still recorded but marked, so the caller can report them.

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
    /// `loadKQ` lines that could not be read, with the reason.
    pub bad_lines: Vec<String>,
}

/// Runs `start` (e.g. `newWorld.cs`). `read` returns a script's bytes by
/// file name (any case), or `None` if it does not exist.
pub fn run(start: &str, read: &dyn Fn(&str) -> Option<Vec<u8>>) -> ScriptRun {
    let mut out = ScriptRun::default();
    run_script(start, read, &mut out, 0);
    out
}

fn run_script(name: &str, read: &dyn Fn(&str) -> Option<Vec<u8>>, out: &mut ScriptRun, depth: usize) {
    if depth > 16 {
        out.bad_lines.push(format!("{name}: scripts nested more than 16 deep; stopped"));
        return;
    }
    let Some(bytes) = read(name) else {
        out.missing_scripts.push(name.to_string());
        return;
    };
    out.scripts.push(name.to_string());
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let mut if_depth = 0usize;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.split('#').next().unwrap_or("").trim();
        let words: Vec<&str> = line.split_whitespace().collect();
        let Some(first) = words.first() else { continue };
        let command = first.to_ascii_lowercase();
        match command.as_str() {
            "if" => if_depth += 1,
            "endif" => if_depth = if_depth.saturating_sub(1),
            "loadkq" => match parse_load(&words[1..]) {
                Ok((file, name_arg, position, angles)) => out.loads.push(LoadKq {
                    file,
                    name: name_arg,
                    position,
                    angles,
                    from: name.to_string(),
                    line: n + 1,
                    inside_if: if_depth > 0,
                }),
                Err(e) => out.bad_lines.push(format!("{name} line {}: {e}: '{line}'", n + 1)),
            },
            c if c.ends_with(".cs") => run_script(first, read, out, depth + 1),
            _ => *out.other_commands.entry(command).or_default() += 1,
        }
    }
}

type Parsed = (String, LoadName, Option<[f32; 3]>, Option<[f32; 3]>);

fn parse_load(args: &[&str]) -> Result<Parsed, String> {
    let file = args.first().ok_or("loadKQ without a file")?.to_string();
    let name = match args.get(1) {
        None => LoadName::Keep,
        Some(n) if n.eq_ignore_ascii_case("same") => LoadName::Keep,
        Some(&"@") => LoadName::None,
        Some(n) => LoadName::Rename(n.to_string()),
    };
    let nums = args.get(2..).unwrap_or(&[]);
    let parse = |s: &str| s.parse::<f32>().map_err(|_| format!("'{s}' is not a number"));
    let position = match nums.len() {
        0 => None,
        1 | 2 => return Err(format!("{} position values, need 3", nums.len())),
        _ => Some([parse(nums[0])?, parse(nums[1])?, parse(nums[2])?]),
    };
    let angles = match nums.len() {
        0..=3 => None,
        6 => Some([parse(nums[3])?, parse(nums[4])?, parse(nums[5])?]),
        n => return Err(format!("{n} numbers; expected 3 or 6")),
    };
    Ok((file, name, position, angles))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read(name: &str) -> Option<Vec<u8>> {
        let files: [(&str, &str); 2] = [
            (
                "newworld.cs",
                "# start\nWorldVar.cs\nloadKQ terrain/terrain.kq\nloadKQ connor.kq same 1 2 -1 0 0 0 # by cottage\npermwrld.cs $KQGame::runOptimal\nloadKQ ax.kq\nalias x \"move\"\n",
            ),
            (
                "permwrld.cs",
                "loadKQ\tstool.kq\tstool.02\t260045\t225746\t13981\t0\t0\t-0.81\nloadKQ a.kq @ 5 6 7\nif test $1 == yes\nloadKQ hidden.kq\nendif\nloadKQ bad.kq x 1 2\n",
            ),
        ];
        files.iter().find(|(n, _)| n.eq_ignore_ascii_case(name)).map(|(_, t)| t.as_bytes().to_vec())
    }

    #[test]
    fn follows_scripts_in_order() {
        let r = run("newWorld.cs", &read);
        let files: Vec<&str> = r.loads.iter().map(|l| l.file.as_str()).collect();
        assert_eq!(files, ["terrain/terrain.kq", "connor.kq", "stool.kq", "a.kq", "hidden.kq", "ax.kq"]);
        assert_eq!(r.scripts, ["newWorld.cs", "permwrld.cs"]);
        assert_eq!(r.missing_scripts, ["WorldVar.cs"]);
        assert_eq!(r.other_commands.get("alias"), Some(&1));
        assert_eq!(r.bad_lines.len(), 1, "{:?}", r.bad_lines);
    }

    #[test]
    fn name_position_and_angle_rules() {
        let r = run("newWorld.cs", &read);
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
}
