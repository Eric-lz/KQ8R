//! Characters (M4 step 4d-1): the people and creatures of class
//! `KQMonster` and `KQGeometryTalker`, as their files describe them.
//!
//! Rules from `docs/behaviour/characters.md` (session A, 2026-10-19):
//!
//! - Rule 1: the two character classes.
//! - Rule 3: a world's monster list (`monsters.kq`, `[Monsters]`,
//!   `KQMonsterManager`, `numMonsters`, `monster0`…): one line per
//!   character, fields separated by tabs, in the order name, type, team,
//!   extra health, extra damage, extra protection, extra experience, x,
//!   y, z, heading, dependents, movement, items held, items dropped, chase
//!   mode, alarm radius, birth delay, attack distance, attack delay. `@` =
//!   the type's value, `none` = none.
//! - Rule 4: the type names a line of `GAME/Kq/Monstats.mtl`
//!   (`KQMonsterTypeList`), whose `KQFile` column names the character's
//!   own `.kq` file; `type(a.bmp,b.bmp)` is a skin swap (draw `b` where
//!   the model uses `a`); quotes around a field are dropped; z −1 = on the
//!   ground; the movement field `none` (never wanders), `loop n x y z …`,
//!   `patrol n …`, `random`.
//! - Rule 6: the keys a character reads, with their defaults (rule 13:
//!   `flags` 0x10; rule 8: gravity on; rule 36: `shadowMask` 2; "Numbers
//!   and units": `wanders` 1, `birthDelay` 0).
//! - Rule 25: `lips=<section>` names a `KQLips` section with
//!   `lip0`…`lip7`; the face texture is `baseLip`, else `lip0`.
//! - Rule 27: the geometry mouth's movements.
//!
//! The column layout of the two lists was checked against the files'
//! own commented header lines (`;monsterX=` and `;typeX=`, data).

use crate::kq_text::{self, Section};
use crate::objects::{current_loop, movement_loops, CurrentLoop};

/// `flags` bit: `Stop` starts at a random frame and speed when Wait
/// begins (rule 13).
pub const RANDOM_STOP: u32 = 0x10;
/// A character's `flags` when its file has no `flags=` key (rule 13).
pub const DEFAULT_FLAGS: u32 = RANDOM_STOP;
/// A character's `shadowMask` when its file has none (rule 36).
pub const DEFAULT_SHADOW_MASK: u32 = 2;

/// The geometry mouth (rule 27): lip-sync code n (0–7) starts movement
/// `GEOMETRY_MOUTH[n]`; code 16 starts `rest` too.
pub const GEOMETRY_MOUTH: [&str; 8] = ["rest", "f", "m", "r", "w", "ih", "ah", "e"];

/// The movements gesture codes 8–15 ask for (rule 28); code 16 asks for
/// `stop`.
pub const GESTURES: [&str; 8] = ["gesturea", "gestureb", "gesturec", "gestured", "gesturee", "gesturef", "gestureg", "gestureh"];

/// The world's shadow mask when no script sets one (rule 36: the
/// program's own value).
pub const BUILT_IN_SHADOW_MASK: u32 = 1;

/// `setShadowMask <n>` in a start-up script such as `MASK/Options.cs`
/// (rule 36): the last value given, `None` if there is none.
pub fn shadow_mask_setting(script: &[u8]) -> Option<u32> {
    let text: String = script.iter().map(|&b| b as char).collect();
    let mut found = None;
    for line in text.lines() {
        let line = line.split("//").next().unwrap_or("").trim();
        let mut words = line.split_whitespace();
        if words.next().is_some_and(|w| w.eq_ignore_ascii_case("setShadowMask")) {
            if let Some(v) = words.next().and_then(|v| v.parse::<f64>().ok()) {
                found = Some(v as u32);
            }
        }
    }
    found
}

/// Rule 36: an object casts a shadow when (the world's mask) AND (its own
/// `shadowMask`) is not 0, shadows are allowed (`allowShadows`), and it
/// is solid (`translucency` 256).
pub fn casts_shadow(world_mask: u32, own_mask: u32, allowed: bool, translucency: f32) -> bool {
    world_mask & own_mask != 0 && allowed && translucency >= 256.0
}

/// The two character classes (rule 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Class {
    Monster,
    GeometryTalker,
}

impl Class {
    /// `classType=` value → class, ignoring capitals; `None` for any
    /// other class.
    pub fn of(class_type: &str) -> Option<Self> {
        let c = class_type.trim();
        if c.eq_ignore_ascii_case("KQMonster") {
            Some(Class::Monster)
        } else if c.eq_ignore_ascii_case("KQGeometryTalker") {
            Some(Class::GeometryTalker)
        } else {
            None
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Class::Monster => "KQMonster",
            Class::GeometryTalker => "KQGeometryTalker",
        }
    }
}

/// A field that may say `@` (the type's value).
#[derive(Debug, Clone, PartialEq)]
pub enum Field<T> {
    /// `@`.
    FromType,
    Value(T),
}

impl<T: Clone> Field<T> {
    /// The value, or `from_type` for `@`.
    pub fn or(&self, from_type: T) -> T {
        match self {
            Field::FromType => from_type,
            Field::Value(v) => v.clone(),
        }
    }
}

/// The movement field (rule 4). Wandering itself is M5; only whether the
/// character may wander and its points are kept.
#[derive(Debug, Clone, PartialEq)]
pub enum Wander {
    /// `none`: never wanders.
    None,
    /// `loop n x y z …`: the n points in order.
    Loop(Vec<[f32; 3]>),
    /// `patrol n x y z …`: the n points, then back again (M5).
    Patrol(Vec<[f32; 3]>),
    /// `random`: no points.
    Random,
}

impl Wander {
    /// Reads the field; `Err` names what is wrong.
    pub fn parse(text: &str) -> Result<Self, String> {
        let words: Vec<&str> = text.split_whitespace().collect();
        let Some(first) = words.first() else { return Err("empty movement field".into()) };
        if first.eq_ignore_ascii_case("none") {
            return Ok(Wander::None);
        }
        if first.eq_ignore_ascii_case("random") {
            return Ok(Wander::Random);
        }
        let patrol = first.eq_ignore_ascii_case("patrol");
        if !patrol && !first.eq_ignore_ascii_case("loop") {
            return Err(format!("movement '{text}' is not none, loop, patrol or random"));
        }
        let n: usize = words.get(1).and_then(|w| w.parse().ok()).ok_or(format!("movement '{text}': no point count"))?;
        let numbers: Vec<f32> = words[2..].iter().map(|w| w.parse::<f32>()).collect::<Result<_, _>>().map_err(|_| format!("movement '{text}': a point is not numbers"))?;
        if numbers.len() < 3 * n {
            return Err(format!("movement '{text}': {n} points named, {} numbers given", numbers.len()));
        }
        let points = numbers.chunks(3).take(n).map(|c| [c[0], c[1], c[2]]).collect();
        Ok(if patrol { Wander::Patrol(points) } else { Wander::Loop(points) })
    }

    /// The character may wander once active (`wanders` 1, rule 4).
    pub fn wanders(&self) -> bool {
        !matches!(self, Wander::None)
    }
}

/// Field positions in a `monsterN=` value (rule 3).
mod line {
    pub const NAME: usize = 0;
    pub const TYPE: usize = 1;
    pub const X: usize = 7;
    pub const HEADING: usize = 10;
    pub const MOVEMENT: usize = 12;
    pub const ALARM_RADIUS: usize = 16;
    pub const BIRTH_DELAY: usize = 17;
    pub const COUNT: usize = 20;
}

/// Field positions in a `typeN=` value of `Monstats.mtl` (its own header
/// line `;typeX=`, data).
mod kind {
    pub const CLASS: usize = 0;
    pub const NAME: usize = 1;
    pub const FILE: usize = 6;
    pub const MOVEMENT: usize = 7;
    pub const ALARM_RADIUS: usize = 11;
    pub const BIRTH_DELAY: usize = 12;
    pub const COUNT: usize = 15;
}

/// A field with surrounding quotes dropped (rule 4).
fn unquote(field: &str) -> &str {
    let f = field.trim();
    f.strip_prefix('"').and_then(|f| f.strip_suffix('"')).unwrap_or(f).trim()
}

/// A number field that may be `@`.
fn number_field(text: &str, what: &str) -> Result<Field<f32>, String> {
    if text == "@" {
        return Ok(Field::FromType);
    }
    text.parse().map(Field::Value).map_err(|_| format!("{what} '{text}' is not a number"))
}

/// One line of a world's monster list (rules 3, 4).
#[derive(Debug, Clone, PartialEq)]
pub struct MonsterLine {
    /// The key it was read from, e.g. `monster17`.
    pub key: String,
    pub name: String,
    /// The type's name, before any bracket (`sprigan` for
    /// `"sprigan(test2.bmp,test3.bmp)"`).
    pub type_name: String,
    /// The skin swap of the type field: (texture in the model, texture
    /// drawn instead).
    pub skin: Option<(String, String)>,
    /// x, y, z; z −1 = on the ground (rule 4).
    pub position: [f32; 3],
    pub heading: f32,
    /// The movement field; `@` = the type's.
    pub movement: Field<Wander>,
    pub alarm_radius: Field<f32>,
    pub birth_delay: Field<f32>,
}

impl MonsterLine {
    /// Reads one `monsterN=` value.
    pub fn parse(key: &str, value: &str) -> Result<Self, String> {
        let fields: Vec<&str> = value.split('\t').map(unquote).collect();
        if fields.len() < line::COUNT {
            return Err(format!("{key}: {} fields, {} expected", fields.len(), line::COUNT));
        }
        let (type_name, skin) = split_skin(fields[line::TYPE]).map_err(|e| format!("{key}: {e}"))?;
        let mut position = [0.0; 3];
        for (k, v) in position.iter_mut().enumerate() {
            let text = fields[line::X + k];
            *v = text.parse().map_err(|_| format!("{key}: {} '{text}' is not a number", ["x", "y", "z"][k]))?;
        }
        let heading_text = fields[line::HEADING];
        let heading = heading_text.parse().map_err(|_| format!("{key}: heading '{heading_text}' is not a number"))?;
        let movement = match fields[line::MOVEMENT] {
            "@" => Field::FromType,
            m => Field::Value(Wander::parse(m).map_err(|e| format!("{key}: {e}"))?),
        };
        Ok(MonsterLine {
            key: key.to_string(),
            name: fields[line::NAME].to_string(),
            type_name,
            skin,
            position,
            heading,
            movement,
            alarm_radius: number_field(fields[line::ALARM_RADIUS], "alarm radius").map_err(|e| format!("{key}: {e}"))?,
            birth_delay: number_field(fields[line::BIRTH_DELAY], "birth delay").map_err(|e| format!("{key}: {e}"))?,
        })
    }
}

/// `type(a.bmp,b.bmp)` → (`type`, Some((a.bmp, b.bmp))) (rule 4).
fn split_skin(field: &str) -> Result<(String, Option<(String, String)>), String> {
    let Some((name, rest)) = field.split_once('(') else { return Ok((field.trim().to_string(), None)) };
    let inner = rest.strip_suffix(')').ok_or(format!("type '{field}' has no closing bracket"))?;
    let (from, to) = inner.split_once(',').ok_or(format!("type '{field}': skin swap without a comma"))?;
    Ok((name.trim().to_string(), Some((from.trim().to_string(), to.trim().to_string()))))
}

/// A world's monster list (rule 3), with the lines that could not be read.
#[derive(Debug, Clone, Default)]
pub struct MonsterList {
    /// The section's name, e.g. `Monsters`.
    pub name: String,
    /// `numMonsters`, if given.
    pub declared: Option<usize>,
    pub lines: Vec<MonsterLine>,
}

impl MonsterList {
    /// Reads `monster0` … `monster<numMonsters − 1>` of the section.
    pub fn from_section(s: &Section) -> (Self, Vec<String>) {
        let mut problems = Vec::new();
        let declared = match s.number("numMonsters") {
            Some(Ok(n)) if n >= 0.0 => Some(n as usize),
            Some(_) => {
                problems.push(format!("[{}] numMonsters is not a count", s.name));
                None
            }
            None => {
                problems.push(format!("[{}] has no numMonsters", s.name));
                None
            }
        };
        let mut lines = Vec::new();
        for k in 0..declared.unwrap_or(0) {
            let key = format!("monster{k}");
            match s.get(&key) {
                None => problems.push(format!("[{}] has no {key}", s.name)),
                Some(v) => match MonsterLine::parse(&key, v) {
                    Ok(l) => lines.push(l),
                    Err(e) => problems.push(e),
                },
            }
        }
        (MonsterList { name: s.name.clone(), declared, lines }, problems)
    }
}

/// A line of `GAME/Kq/Monstats.mtl` (rule 4).
#[derive(Debug, Clone, PartialEq)]
pub struct MonsterType {
    /// The key it was read from, e.g. `type4`.
    pub key: String,
    /// `KQMonsterType` in every line of the data.
    pub class: String,
    /// `goblin2`.
    pub name: String,
    /// The character's own `.kq` file (`KQFile`), e.g. `goblin2.kq`.
    pub file: String,
    pub movement: Wander,
    pub alarm_radius: f32,
    pub birth_delay: f32,
}

/// The type list (rule 4).
#[derive(Debug, Clone, Default)]
pub struct MonsterTypes {
    pub declared: Option<usize>,
    pub types: Vec<MonsterType>,
}

impl MonsterTypes {
    /// Reads `type0` … `type<numTypes − 1>` of the file's first section.
    pub fn parse(bytes: &[u8]) -> (Self, Vec<String>) {
        let file = kq_text::parse(bytes);
        let mut problems = Vec::new();
        let Some(s) = file.first() else { return (Self::default(), vec!["the type list has no section".into()]) };
        let declared = s.number("numTypes").and_then(Result::ok).filter(|n| *n >= 0.0).map(|n| n as usize);
        if declared.is_none() {
            problems.push(format!("[{}] has no numTypes count", s.name));
        }
        let mut types = Vec::new();
        for k in 0..declared.unwrap_or(0) {
            let key = format!("type{k}");
            let Some(v) = s.get(&key) else {
                problems.push(format!("[{}] has no {key}", s.name));
                continue;
            };
            let f: Vec<&str> = v.split('\t').map(unquote).collect();
            if f.len() < kind::COUNT {
                problems.push(format!("{key}: {} fields, {} expected", f.len(), kind::COUNT));
                continue;
            }
            let number = |i: usize, what: &str| f[i].parse::<f32>().map_err(|_| format!("{key}: {what} '{}' is not a number", f[i]));
            let read = || -> Result<MonsterType, String> {
                Ok(MonsterType {
                    key: key.clone(),
                    class: f[kind::CLASS].to_string(),
                    name: f[kind::NAME].to_string(),
                    file: f[kind::FILE].to_string(),
                    movement: Wander::parse(f[kind::MOVEMENT]).map_err(|e| format!("{key}: {e}"))?,
                    alarm_radius: number(kind::ALARM_RADIUS, "alarm radius")?,
                    birth_delay: number(kind::BIRTH_DELAY, "birth delay")?,
                })
            };
            match read() {
                Ok(t) => types.push(t),
                Err(e) => problems.push(e),
            }
        }
        (MonsterTypes { declared, types }, problems)
    }

    /// The type called `name`: exact first, then ignoring capitals (the
    /// note does not say; `character-verify` counts how often the second
    /// is needed). The bool is true when only the second matched.
    pub fn find(&self, name: &str) -> Option<(&MonsterType, bool)> {
        if let Some(t) = self.types.iter().find(|t| t.name == name) {
            return Some((t, false));
        }
        self.types.iter().find(|t| t.name.eq_ignore_ascii_case(name)).map(|t| (t, true))
    }
}

/// The keys a character reads (rule 6), with their defaults.
#[derive(Debug, Clone, PartialEq)]
pub struct CharacterKeys {
    pub class: Class,
    /// `animFile=`: its movement list.
    pub anim_file: Option<String>,
    /// `idleAnimFile=`: fidgets, only for active characters (M5).
    pub idle_anim_file: Option<String>,
    /// `lips=`: the `KQLips` section's name.
    pub lips: Option<String>,
    /// `baseLip=`: the face texture, if not `lip0`.
    pub base_lip: Option<String>,
    /// `flags=` (replaces the default 0x10, rule 13); a value that is not
    /// a whole number counts as 0, as for objects.
    pub flags: u32,
    /// `monUseGravity=` (default on, rule 8).
    pub gravity: bool,
    /// `noTurn=1` blocks every turn (rule 23).
    pub no_turn: bool,
    /// `wanders=` (default 1).
    pub wanders: bool,
    /// `monsterType=`: the box's name (rule 9).
    pub monster_type: Option<String>,
    /// `shadowMask=` (default 2, rule 36).
    pub shadow_mask: u32,
    /// `flyingObject=1` (M5).
    pub flying: bool,
    /// `birthDelay=` (default 0, rule 12).
    pub birth_delay: f32,
    /// `alarmRadius=`, if given (M5).
    pub alarm_radius: Option<f32>,
    /// `originalBmp=` / `alternateBmp=` (rule 4): the skin swap.
    pub skin: Option<(String, String)>,
    /// `stepUpAmount=` / `stepDownAmount=`, if given (rule 8).
    pub step_up: Option<f32>,
    pub step_down: Option<f32>,
    /// `colliderType=` / `colliderMask=`, else the character's own
    /// defaults 0x42 / 0x281F (rule 8, Q122).
    pub collider_type: u32,
    pub collider_mask: u32,
    /// `GBMsg`, `GBNoun`, `GBVerb`, `GBCase`, `GBSeq`: the goodbye line
    /// (`conversation.md` rule 20); `None` when file and noun are both 0
    /// (a missing key counts as 0).
    pub goodbye: Option<crate::msg::Tuple>,
    /// `WBMsg`…`WBSeq`: the welcome-back line (rule 20).
    pub welcome_back: Option<crate::msg::Tuple>,
}

/// A character's own collision type and mask when its file sets none
/// (`characters.md` rule 8, Q122).
pub const CHARACTER_TYPE: u32 = 0x42;
pub const CHARACTER_MASK: u32 = 0x281F;

impl CharacterKeys {
    /// `None` when the section is not a character.
    pub fn from_section(s: &Section) -> Option<Self> {
        let class = Class::of(s.get("classType")?)?;
        let text = |k: &str| s.get(k).map(|v| v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("").to_string()).filter(|v| !v.is_empty());
        let number = |k: &str| s.number(k).and_then(Result::ok);
        let switch = |k: &str, default: bool| number(k).map_or(default, |v| v != 0.0);
        Some(CharacterKeys {
            class,
            anim_file: text("animFile"),
            idle_anim_file: text("idleAnimFile"),
            lips: text("lips"),
            base_lip: text("baseLip"),
            flags: s.get("flags").map_or(DEFAULT_FLAGS, |f| f.trim().parse::<i64>().unwrap_or(0) as u32),
            gravity: switch("monUseGravity", true),
            no_turn: switch("noTurn", false),
            wanders: switch("wanders", true),
            monster_type: text("monsterType"),
            shadow_mask: number("shadowMask").map_or(DEFAULT_SHADOW_MASK, |v| v as u32),
            flying: switch("flyingObject", false),
            birth_delay: number("birthDelay").map_or(0.0, |v| v as f32),
            alarm_radius: number("alarmRadius").map(|v| v as f32),
            skin: text("originalBmp").zip(text("alternateBmp")),
            step_up: number("stepUpAmount").map(|v| v as f32),
            step_down: number("stepDownAmount").map(|v| v as f32),
            collider_type: number("colliderType").map_or(CHARACTER_TYPE, |v| v as i64 as u32),
            collider_mask: number("colliderMask").map_or(CHARACTER_MASK, |v| v as i64 as u32),
            goodbye: line_keys(s, "GB"),
            welcome_back: line_keys(s, "WB"),
        })
    }

    /// `flags` bit 0x10: `Stop` at a random frame and speed (rule 13).
    pub fn random_stop(&self) -> bool {
        self.flags & RANDOM_STOP != 0
    }
}

/// `<p>Msg`, `<p>Noun`, `<p>Verb`, `<p>Case`, `<p>Seq` as a line
/// (`conversation.md` rule 20): a missing key is 0; file and noun both 0
/// = no line.
fn line_keys(s: &Section, p: &str) -> Option<crate::msg::Tuple> {
    let n = |k: &str| s.number(&format!("{p}{k}")).and_then(Result::ok).unwrap_or(0.0) as i64;
    let t = crate::msg::Tuple::new(n("Msg") as u16, n("Noun") as u8, n("Verb") as u8, n("Case") as u8, n("Seq") as u8);
    (t.file != 0 || t.noun != 0).then_some(t)
}

/// The mouth (rule 25): the face texture and `lip0`…`lip7`.
#[derive(Debug, Clone, PartialEq)]
pub struct Lips {
    /// The section named by `lips=`.
    pub section: String,
    /// `baseLip`, else `lip0`.
    pub face: String,
    /// `lip0`…`lip7`; `None` where the section has no such key.
    pub shapes: [Option<String>; 8],
}

impl Lips {
    /// The `KQLips` section `keys.lips` names, looked up in `file`;
    /// `None` without `lips=`, `Err` when the section is missing or has
    /// neither `baseLip` nor `lip0`.
    pub fn of(file: &[Section], keys: &CharacterKeys) -> Option<Result<Self, String>> {
        let name = keys.lips.as_deref()?;
        let Some(s) = file.iter().find(|s| s.name.eq_ignore_ascii_case(name)) else {
            return Some(Err(format!("lips={name}: no section [{name}] in the file")));
        };
        let shapes: [Option<String>; 8] = std::array::from_fn(|n| s.get(&format!("lip{n}")).map(|v| v.trim().to_string()).filter(|v| !v.is_empty()));
        let face = keys.base_lip.clone().or_else(|| shapes[0].clone());
        Some(match face {
            Some(face) => Ok(Lips { section: s.name.clone(), face, shapes }),
            None => Err(format!("[{name}] has no lip0 and the character no baseLip")),
        })
    }

    /// The texture drawn for mouth shape `n`: `lip<n>`, `lip0` for 8 or
    /// more (rule 25); `None` if the section lacks it.
    pub fn texture(&self, n: u16) -> Option<&str> {
        let n = if n > 7 { 0 } else { n as usize };
        self.shapes[n].as_deref()
    }
}

/// A `.anm` movement list (`KQAnimLoopList`, the same format as
/// `Conner.anm`, rule 6): its section's name and every movement in list
/// order.
#[derive(Debug, Clone)]
pub struct AnimList {
    pub name: String,
    pub movements: Vec<(String, CurrentLoop)>,
    /// The parsed file, kept to look up the current movement.
    file: Vec<Section>,
}

impl AnimList {
    /// Reads a `.anm` file: the list is its first section.
    pub fn parse(bytes: &[u8]) -> Result<Self, String> {
        let file = kq_text::parse(bytes);
        let name = file.first().map(|s| s.name.clone()).ok_or("the .anm file has no section")?;
        let asks = Self::asking(&name, None);
        let movements = movement_loops(&file, &asks)?.ok_or("no animation list")?;
        Ok(AnimList { name, movements, file })
    }

    /// A section naming the list (and a current movement), as an object
    /// that carries the list itself would (`objects.md`).
    fn asking(list: &str, current: Option<&str>) -> Section {
        let mut entries = vec![("animLoops".to_string(), list.to_string())];
        if let Some(c) = current {
            entries.push(("currAnimName".to_string(), c.to_string()));
        }
        Section { name: "CharacterAsks".into(), entries }
    }

    /// The movement shown at load: `currAnimName`, or movement 0 if it is
    /// empty or unknown (rule 12.2; `fell_back` says which).
    pub fn current(&self, curr: Option<&str>) -> Result<CurrentLoop, String> {
        current_loop(&self.file, &Self::asking(&self.name, curr.map(str::trim).filter(|c| !c.is_empty())))?.ok_or("no animation list".into())
    }

    /// Index of the movement called `name`, ignoring capitals (first
    /// match, as `anim-scripts.md` rule 5).
    pub fn index(&self, name: &str) -> Option<usize> {
        self.movements.iter().position(|(n, _)| n.eq_ignore_ascii_case(name))
    }

    pub fn has(&self, name: &str) -> bool {
        self.index(name).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LIST: &str = "[Monsters]\nclassType=KQMonsterManager\n;monsterX=\tname\ttype\nnumMonsters=4\n\
monster0=\tsprgnbow\tsprgnbowtype\t@\t100\t-13\t10\t8\t226700\t183000\t8370\t-1.57\t@\t@\t@\tINVITEM_SmCrossbow\tguard\t12000\t@\t12000\t1\n\
monster1=\tgobcon1\tgoblin\t@\t-5\t-3\t-4\t1\t279735\t227945\t-1\t6.1\t@\tloop 3 279735 227945 -1 270219 245074 -1 256417 241119 -1\t@\t\"INVITEM_SilverCoins(3),INVITEM_mushroom\"\tchase 4000 9000\t6000\t@\t@\t@\n\
monster2=\tbossprig\t\"sprigan(test2.bmp,test3.bmp)\"\t@\t200\t13\t25\t@\t217361\t284013\t24650\t1.8\t@\tnone\tx\ty\tchase 4000 6000\t5000\t@\t@\t0.2\n\
monster3=\tchicken3\tchicken\t@\t@\t@\t1\t-1\t244688\t216198\t-2\t0\t@\tpatrol 2 1 2 -2 3 4 -2\t@\t@\t@\t@\t1\t@\t1.5\t\t\n";

    #[test]
    fn monster_lines_are_read_field_by_field() {
        let file = kq_text::parse(LIST.as_bytes());
        let (list, problems) = MonsterList::from_section(&file[0]);
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(list.lines.len(), 4);
        let bow = &list.lines[0];
        assert_eq!((bow.name.as_str(), bow.type_name.as_str()), ("sprgnbow", "sprgnbowtype"));
        assert_eq!(bow.position, [226700.0, 183000.0, 8370.0]);
        assert_eq!(bow.heading, -1.57);
        assert_eq!(bow.movement, Field::FromType);
        assert_eq!(bow.alarm_radius, Field::Value(12000.0));
        assert_eq!(bow.birth_delay, Field::FromType);
        let gob = &list.lines[1];
        assert_eq!(gob.position[2], -1.0);
        assert_eq!(gob.movement, Field::Value(Wander::Loop(vec![[279735.0, 227945.0, -1.0], [270219.0, 245074.0, -1.0], [256417.0, 241119.0, -1.0]])));
        let boss = &list.lines[2];
        assert_eq!(boss.type_name, "sprigan");
        assert_eq!(boss.skin, Some(("test2.bmp".into(), "test3.bmp".into())));
        assert_eq!(boss.movement, Field::Value(Wander::None));
        let chick = &list.lines[3];
        assert_eq!(chick.movement, Field::Value(Wander::Patrol(vec![[1.0, 2.0, -2.0], [3.0, 4.0, -2.0]])));
        assert_eq!(chick.birth_delay, Field::Value(1.0));
    }

    #[test]
    fn bad_lines_are_reported() {
        assert!(MonsterLine::parse("m", "a\tb").is_err());
        assert!(MonsterLine::parse("m", "a\tt(x.bmp\t@\t@\t@\t@\t@\t1\t2\t3\t0\t@\tnone\t@\t@\t@\t@\t@\t@\t@").is_err());
        assert!(Wander::parse("loop 2 1 2 3").is_err());
        assert!(Wander::parse("walk").is_err());
        assert_eq!(Wander::parse("random"), Ok(Wander::Random));
        assert!(!Wander::None.wanders() && Wander::Random.wanders());
    }

    #[test]
    fn types_name_their_files() {
        let text = "[MonsterTypeList]\nclassType=KQMonsterTypeList\nnumTypes=2\n\
type0=\tKQMonsterType\tsprigan\t30\t10\t5\t7\tsprigan.kq\tnone\tnone\tnone\tchase 4000 8000\t4000\t0\t1200\t1\n\
type1=\tKQMonsterType\tslime\t100\t20\t20\t5000\tslime.kq\tnone\tINVITEM_SlmSlime\tnone\tchase 3000 6000\t9000\t2\t5000 0.4 3.0\t0.5\n";
        let (t, problems) = MonsterTypes::parse(text.as_bytes());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(t.types.len(), 2);
        let (s, folded) = t.find("sprigan").unwrap();
        assert_eq!((s.file.as_str(), folded), ("sprigan.kq", false));
        assert_eq!(t.find("SLIME").map(|(t, f)| (t.birth_delay, f)), Some((2.0, true)));
        assert!(t.find("nothing").is_none());
    }

    #[test]
    fn shadows_follow_the_masks_and_solidity() {
        assert_eq!(shadow_mask_setting(b"KQSound::setSfxVol 0.14
setShadowMask 15
useDynamicLighting 1
"), Some(15));
        assert_eq!(shadow_mask_setting(b"set x 1
"), None);
        // Characters (2) and plain objects (8) cast with 15; not with the
        // built-in 1; shadowMask 0 never; see-through never; all off.
        assert!(casts_shadow(15, DEFAULT_SHADOW_MASK, true, 256.0));
        assert!(casts_shadow(15, 8, true, 256.0));
        assert!(!casts_shadow(BUILT_IN_SHADOW_MASK, DEFAULT_SHADOW_MASK, true, 256.0));
        assert!(!casts_shadow(15, 0, true, 256.0));
        assert!(!casts_shadow(15, 2, true, 165.0));
        assert!(!casts_shadow(15, 2, false, 256.0));
    }

    #[test]
    fn keys_have_the_note_defaults() {
        let file = kq_text::parse(b"[Chicken]\nclassType=KQMonster\nanimFile=chicken.anm\nshadowMask=0\n");
        let k = CharacterKeys::from_section(&file[0]).unwrap();
        assert_eq!(k.class, Class::Monster);
        assert_eq!(k.flags, 0x10);
        assert!(k.random_stop() && k.gravity && k.wanders && !k.no_turn);
        assert_eq!(k.shadow_mask, 0);
        assert_eq!(k.anim_file.as_deref(), Some("chicken.anm"));
        let file = kq_text::parse(b"[Uglybeast]\nclassType=KQGeometryTalker\nflags=32\nnoTurn=1\nwanders=0\nmonUseGravity=0\n");
        let k = CharacterKeys::from_section(&file[0]).unwrap();
        assert_eq!((k.class, k.flags, k.shadow_mask), (Class::GeometryTalker, 32, 2));
        assert!(!k.random_stop() && !k.gravity && !k.wanders && k.no_turn);
        let file = kq_text::parse(b"[Door]\nclassType=KQDoor\n");
        assert!(CharacterKeys::from_section(&file[0]).is_none());
    }

    #[test]
    fn goodbye_and_welcome_back_keys() {
        // conversation.md rule 20: a missing key is 0; file and noun 0 = none.
        let file = kq_text::parse(b"[Bob]\nclassType=KQMonster\nGBMsg=1000\nGBNoun=4\nGBCase=7\nGBSeq=1\nWBMsg=0\nWBNoun=0\nWBSeq=1\n");
        let k = CharacterKeys::from_section(&file[0]).unwrap();
        assert_eq!(k.goodbye, Some(crate::msg::Tuple::new(1000, 4, 0, 7, 1)));
        assert_eq!(k.welcome_back, None);
    }

    #[test]
    fn lips_give_the_face_and_shapes() {
        let file = kq_text::parse(b"[Wizard]\nclassType=KQMonster\nlips=WizardLips\n[WizardLips]\nclassType=KQLips\nlip0=WF0.bmp\nlip1=WF1.bmp\nlip7=WF7.bmp\n");
        let keys = CharacterKeys::from_section(&file[0]).unwrap();
        let lips = Lips::of(&file, &keys).unwrap().unwrap();
        assert_eq!(lips.face, "WF0.bmp");
        assert_eq!(lips.texture(1), Some("WF1.bmp"));
        assert_eq!(lips.texture(9), Some("WF0.bmp"));
        assert_eq!(lips.texture(3), None);
        let file = kq_text::parse(b"[X]\nclassType=KQMonster\nlips=Nope\n");
        let keys = CharacterKeys::from_section(&file[0]).unwrap();
        assert!(Lips::of(&file, &keys).unwrap().is_err());
    }

    #[test]
    fn anm_lists_give_movements_and_the_current_one() {
        let anm = b"[chickenAnimLoops]\nclassType=KQAnimLoopList\nnMovement=2\nMovement0=attack\nMovement1=stop\n\
[movement0]\nloop=chk_atk.kqs chk_atk 0.000000 1.0 0 0\n[movement1]\nloop=chk_stop.kqs chk_stop 0.000000 1.0 1 1\n";
        let list = AnimList::parse(anm).unwrap();
        assert_eq!(list.movements.len(), 2);
        assert_eq!(list.index("STOP"), Some(1));
        let c = list.current(Some("stop")).unwrap();
        assert_eq!((c.movement, c.fell_back, c.shape.as_str()), (1, false, "chk_stop.kqs"));
        let c = list.current(None).unwrap();
        assert_eq!((c.movement, c.fell_back), (0, true));
        let c = list.current(Some("dance")).unwrap();
        assert_eq!((c.movement, c.fell_back), (0, true));
    }
}
