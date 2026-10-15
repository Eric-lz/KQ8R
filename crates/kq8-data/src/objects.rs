//! Object placements from the `.kq` files: which model goes where, turned
//! how.
//!
//! Rules from the behaviour note `docs/behaviour/objects.md` (session A,
//! answers Q9/Q10); key names observed in the data:
//!
//! - Position `locX`, `locY`, `locZ` in game units (`isGroundRelative` is
//!   never read by the game). Each key is read on its own; a missing one is
//!   0 (objects.md "Missing locX / locY / locZ", Q19). `keepOnGround`
//!   raises an object below the ground onto it, never lowers it
//!   (docs/behaviour/buildings.md, Q10 follow-up): by default onto the
//!   floor grid, with `keepOnGround=1` onto the surface, with `=0` not at
//!   all (water.md "Which grid answers").
//! - Angles `dirX`, `dirY`, `dirZ` in radians, right-hand rule (positive
//!   `dirZ` turns +X towards +Y). A point is turned about Y first, then X,
//!   then Z: with row vectors the matrix is `RotY × RotX × RotZ`.
//! - No scale. (A full matrix key `<prefix>RMat` exists in the game but no
//!   file uses it; such sections are reported, not placed.)

use crate::kq_text::Section;
use crate::shape::Transform;
use crate::terrain::Level;

/// `keepOnGround` (water.md "Which grid answers"): the mover's default
/// (−1) raises onto the floor grid, `1` onto the surface, `0` not at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeepOnGround {
    Floor,
    Surface,
    Off,
}

impl KeepOnGround {
    /// From a `keepOnGround=` value: 0 off, −1 floor, anything else the
    /// surface (the data only uses 0 and 1; other values are our reading).
    pub fn from_value(v: f64) -> Self {
        match v as i64 {
            0 => KeepOnGround::Off,
            -1 => KeepOnGround::Floor,
            _ => KeepOnGround::Surface,
        }
    }

    /// The grid it raises onto, `None` when off.
    pub fn level(self) -> Option<Level> {
        match self {
            KeepOnGround::Floor => Some(Level::Floor),
            KeepOnGround::Surface => Some(Level::Surface),
            KeepOnGround::Off => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Placement {
    /// Section name, e.g. `barn`.
    pub name: String,
    pub class: Option<String>,
    /// `shapeName` as written, e.g. `smlchest.KQS` or `barn.dig`.
    pub shape: Option<String>,
    pub position: [f32; 3],
    /// `dirX`, `dirY`, `dirZ` in radians (0 when missing).
    pub angles: [f32; 3],
    /// True if the section has an `...RMat` key (not supported).
    pub has_matrix_key: bool,
    /// `keepOnGround`: the floor unless the section sets it.
    pub keep_on_ground: KeepOnGround,
    /// `flags` as written, if present (animation start options; see
    /// objects.md "Animated objects at load" rule 4).
    pub flags: Option<String>,
    /// `translucency`: 256 (the default) = solid; below that every
    /// material is drawn at opacity n / 255 (objects.md Q14).
    pub translucency: f32,
    /// Set if a `loadKQ` line gave z = −1 (the floor) or −2 (the
    /// surface): put on that grid, whatever `keepOnGround` says
    /// (objects.md "Ground heights -1 and -2", Q17; water.md).
    pub z_on_ground: Option<Level>,
    /// Which of `locX`, `locY`, `locZ` the section left out (taken as 0).
    pub missing_loc: [bool; 3],
    /// `hidden`: any value other than 0 (objects.md Q18). Scripts can
    /// change it (`KQObject::setHidden`).
    pub hidden: bool,
    /// `groupsAlways` bit mask, default 6 (Q18).
    pub groups_always: u32,
    /// `b_hazeSolid`: not left out at full haze (object-lighting.md
    /// rule 21). Any value other than 0.
    pub haze_solid: bool,
    /// `hazeMultiplier`, default 1 (rule 22). The data's misspelled
    /// `hazeMulitplier` is not read (Q68).
    pub haze_multiplier: f32,
}

impl Placement {
    /// Missing position or angle keys are 0 (Q19).
    pub fn from_section(s: &Section) -> Self {
        let num = |k: &str| s.number(k).and_then(Result::ok).map(|v| v as f32);
        let loc = [num("locX"), num("locY"), num("locZ")];
        let position = loc.map(|v| v.unwrap_or(0.0));
        let angles = [num("dirX").unwrap_or(0.0), num("dirY").unwrap_or(0.0), num("dirZ").unwrap_or(0.0)];
        Placement {
            name: s.name.clone(),
            class: s.get("classType").map(str::to_string),
            shape: s.get("shapeName").filter(|v| !v.is_empty()).map(str::to_string),
            position,
            angles,
            has_matrix_key: s.entries.iter().any(|(k, _)| k.to_ascii_lowercase().ends_with("rmat")),
            keep_on_ground: s.number("keepOnGround").and_then(Result::ok).map_or(KeepOnGround::Floor, KeepOnGround::from_value),
            flags: s.get("flags").map(str::to_string),
            translucency: s.number("translucency").and_then(Result::ok).map_or(256.0, |v| v as f32),
            // A `.kq` file's own locZ never takes the -1/-2 rule (Q17 rule 4).
            z_on_ground: None,
            missing_loc: loc.map(|v| v.is_none()),
            hidden: s.number("hidden").and_then(Result::ok).is_some_and(|v| v as i64 != 0),
            groups_always: s.number("groupsAlways").and_then(Result::ok).map_or(6, |v| v as u32),
            haze_solid: s.number("b_hazeSolid").and_then(Result::ok).is_some_and(|v| v != 0.0),
            haze_multiplier: s.number("hazeMultiplier").and_then(Result::ok).map_or(1.0, |v| v as f32),
        }
    }

    /// The object a `loadKQ` line creates from `section` (objects.md Q16
    /// rule 2): name kept, removed or replaced; with a position, it
    /// replaces `locX/Y/Z` and the angles are the given ones or 0 0 0.
    pub fn from_load(section: &Section, load: &crate::world_script::LoadKq) -> Self {
        use crate::world_script::LoadName;
        let mut p = match load.position {
            Some(position) => {
                // Build from the section, then replace position and angles.
                let mut with_loc = section.clone();
                with_loc.entries.retain(|(k, _)| !["locx", "locy", "locz"].contains(&k.to_ascii_lowercase().as_str()));
                for (k, v) in ["locX", "locY", "locZ"].iter().zip(position) {
                    with_loc.entries.push((k.to_string(), v.to_string()));
                }
                let mut p = Self::from_section(&with_loc);
                p.angles = load.angles_or_zero();
                p.z_on_ground = match position[2] {
                    -1.0 => Some(Level::Floor),
                    -2.0 => Some(Level::Surface),
                    _ => None,
                };
                p
            }
            None => Self::from_section(section),
        };
        match &load.name {
            LoadName::Keep => {}
            LoadName::None => p.name = String::new(),
            LoadName::Rename(n) => p.name = n.clone(),
        }
        p
    }

    /// Drawn at the start: not hidden, or hidden but `groupsAlways` keeps
    /// bit 0x40, which session A infers is "drawn" (objects.md Q18 rule 3,
    /// not traced).
    pub fn drawn(&self) -> bool {
        !self.hidden || self.groups_always & 0x40 != 0
    }

    /// Opacity 0..1 from `translucency` (1 = solid).
    pub fn opacity(&self) -> f32 {
        if self.translucency >= 256.0 { 1.0 } else { (self.translucency / 255.0).clamp(0.0, 1.0) }
    }

    /// The grid the ground rules use: the `loadKQ` z's (−1/−2), else
    /// `keepOnGround`'s; `None` if neither applies.
    pub fn ground_level(&self) -> Option<Level> {
        self.z_on_ground.or(self.keep_on_ground.level())
    }

    /// Position after the ground rules, with `ground(level)` the height of
    /// that grid under locX, locY: a `loadKQ` z of −1/−2 is replaced by the
    /// floor/surface height (Q17, water.md); then `keepOnGround` raises an
    /// object below its grid onto it, never lowers it. The raise also
    /// follows a −2 (the user's check in the original, 2026-10-12: DeadCity's
    /// portal by the flame tower, placed at −2, stands on the higher floor).
    pub fn grounded_position(&self, ground: impl Fn(Level) -> Option<f32>) -> [f32; 3] {
        let mut p = self.position;
        if let Some(g) = self.z_on_ground.and_then(&ground) {
            p[2] = g;
        }
        match self.keep_on_ground.level().and_then(&ground) {
            Some(g) if p[2] < g => [p[0], p[1], g],
            _ => p,
        }
    }

    /// The placement as a row-vector transform: point × rotation + position.
    pub fn transform(&self) -> Transform {
        self.transform_at(self.position)
    }

    /// Same, with a different position (e.g. after the ground rule).
    pub fn transform_at(&self, position: [f32; 3]) -> Transform {
        Transform { flags: 0, m: rotation(self.angles), p: position }
    }
}

/// An animated object's current animation, from its `loop=` line.
#[derive(Debug, Clone, PartialEq)]
pub struct CurrentLoop {
    /// Index k of `[movement<k>]`.
    pub movement: usize,
    /// True if `currAnimName` was missing, empty or not in the list, so
    /// movement 0 was used (objects.md "Animated objects at load" rule 3).
    pub fell_back: bool,
    pub shape: String,
    /// Sequence (loop) name inside the model.
    pub sequence: String,
    /// Start position 0..1 in the sequence.
    pub start: f32,
    /// 1 forward, -1 backward, 0 stopped.
    pub speed: f32,
    /// True = repeats; false = plays once.
    pub repeat: bool,
    /// Fourth value (6th field of the line). For Connor's movements it is
    /// "can be cut short" (`connor-movement.md` "Starting and stopping");
    /// for objects not traced.
    pub unknown: i64,
    /// The movement's `transitionN` lines (`nTransition` of them).
    pub transitions: Vec<TransitionLine>,
    /// Transition lines that could not be read, with the reason.
    pub transition_problems: Vec<String>,
    /// The movement's `cueN=<phase> <action…>` lines (`nCue` of them),
    /// in file order (doors.md rule 9; e.g. `cue0=0.0 playsound door4.aud`).
    pub cues: Vec<(f32, String)>,
    /// Cue lines that could not be read, with the reason.
    pub cue_problems: Vec<String>,
}

/// `transitionN=<from phase> <to movement> <to phase> <transition movement>`
/// (`connor-movement.md` "Starting and stopping"; a from phase of −1 means
/// "at once").
#[derive(Debug, Clone, PartialEq)]
pub struct TransitionLine {
    pub from: f32,
    pub to: String,
    pub to_phase: f32,
    pub via: String,
}

/// The `nTransition` / `transitionN` lines of a `[movement<k>]` section.
fn transition_lines(section: &Section) -> (Vec<TransitionLine>, Vec<String>) {
    let (mut lines, mut problems) = (Vec::new(), Vec::new());
    let count = match section.number("nTransition") {
        None => 0,
        Some(Ok(n)) if n >= 0.0 => n as usize,
        Some(_) => {
            problems.push(format!("[{}] nTransition is not a count", section.name));
            0
        }
    };
    for k in 0..count {
        let key = format!("transition{k}");
        let Some(text) = section.get(&key) else {
            problems.push(format!("[{}] has no {key}", section.name));
            continue;
        };
        let words: Vec<&str> = text.split_whitespace().collect();
        let parsed = match words[..] {
            [from, to, to_phase, via] => from.parse().ok().zip(to_phase.parse().ok()).map(|(from, to_phase)| TransitionLine {
                from,
                to: to.to_string(),
                to_phase,
                via: via.to_string(),
            }),
            _ => None,
        };
        match parsed {
            Some(t) => lines.push(t),
            None => problems.push(format!("[{}] {key}={text} is not '<from> <to> <to phase> <via>'", section.name)),
        }
    }
    (lines, problems)
}

/// The `nCue` / `cueN=<phase> <action…>` lines of a `[movement<k>]`
/// section. Lines commented out with `#` have another key and don't count.
fn cue_lines(section: &Section) -> (Vec<(f32, String)>, Vec<String>) {
    let (mut cues, mut problems) = (Vec::new(), Vec::new());
    let count = match section.number("nCue") {
        None => 0,
        Some(Ok(n)) if n >= 0.0 => n as usize,
        Some(_) => {
            problems.push(format!("[{}] nCue is not a count", section.name));
            0
        }
    };
    for k in 0..count {
        let key = format!("cue{k}");
        let Some(text) = section.get(&key) else {
            problems.push(format!("[{}] has no {key}", section.name));
            continue;
        };
        let mut words = text.split_whitespace();
        match (words.next().map(str::parse::<f32>), words.collect::<Vec<_>>().join(" ")) {
            (Some(Ok(phase)), action) if !action.is_empty() => cues.push((phase, action)),
            _ => problems.push(format!("[{}] {key}={text} is not '<phase> <action>'", section.name)),
        }
    }
    (cues, problems)
}

/// Every movement of an animated object, in list order (index k =
/// `Movement<k>` = `[movement<k>]`): its name and loop. `Ok(None)` if the
/// section is not animated this way.
pub fn movement_loops(file: &[Section], s: &Section) -> Result<Option<Vec<(String, CurrentLoop)>>, String> {
    let Some(list_name) = s.get("animLoops") else { return Ok(None) };
    let find = |name: &str| file.iter().find(|x| x.name.eq_ignore_ascii_case(name));
    let list = find(list_name).ok_or(format!("animation list [{list_name}] not in the file"))?;
    let count = list.number("nMovement").ok_or(format!("[{list_name}] has no nMovement"))??;
    let mut out = Vec::new();
    for k in 0..count as usize {
        let name = list.get(&format!("Movement{k}")).ok_or(format!("[{list_name}] has no Movement{k}"))?;
        out.push((name.to_string(), loop_of(file, k, false)?));
    }
    Ok(Some(out))
}

/// The loop of `[movement<k>]`.
fn loop_of(file: &[Section], movement: usize, fell_back: bool) -> Result<CurrentLoop, String> {
    let find = |name: &str| file.iter().find(|x| x.name.eq_ignore_ascii_case(name));
    let section = find(&format!("movement{movement}")).ok_or(format!("section [movement{movement}] not in the file"))?;
    let line = section.get("loop").ok_or(format!("[movement{movement}] has no loop line"))?;
    let words: Vec<&str> = line.split_whitespace().collect();
    let [shape, sequence, start, speed, repeat, unknown] = words[..] else {
        return Err(format!("[movement{movement}] loop line '{line}' does not have 6 parts"));
    };
    let bad = |what: &str| format!("[movement{movement}] loop line '{line}': {what} is not a number");
    let (transitions, transition_problems) = transition_lines(section);
    let (cues, cue_problems) = cue_lines(section);
    Ok(CurrentLoop {
        movement,
        fell_back,
        shape: shape.to_string(),
        sequence: sequence.to_string(),
        start: start.parse().map_err(|_| bad("start"))?,
        speed: speed.parse().map_err(|_| bad("speed"))?,
        repeat: repeat.parse::<i64>().map_err(|_| bad("repeat"))? != 0,
        unknown: unknown.parse().map_err(|_| bad("fourth value"))?,
        transitions,
        transition_problems,
        cues,
        cue_problems,
    })
}

/// The current animation of an animated object (rules from objects.md
/// "Animated objects at load"; layout observed, e.g. Daventry
/// `alchdoor.kq`): `animLoops=<list>`; `[<list>]` has `nMovement` and
/// `Movement0..` names; `[movement<k>]` (by number) has
/// `loop=<model> <sequence> <start> <speed> <repeat> <unknown>`. The movement
/// is the one named by `currAnimName` (key matched ignoring capitals only,
/// so `curAnimName` does not count), else movement 0.
///
/// Returns `Ok(None)` if the section is not animated this way.
pub fn current_loop(file: &[Section], s: &Section) -> Result<Option<CurrentLoop>, String> {
    let Some(list_name) = s.get("animLoops") else { return Ok(None) };
    let find = |name: &str| file.iter().find(|x| x.name.eq_ignore_ascii_case(name));
    let list = find(list_name).ok_or(format!("animation list [{list_name}] not in the file"))?;
    let count = list
        .number("nMovement")
        .ok_or(format!("[{list_name}] has no nMovement"))??;
    let named = s.get("currAnimName").filter(|c| !c.is_empty()).and_then(|current| {
        (0..count as usize).find(|k| list.get(&format!("Movement{k}")).is_some_and(|m| m.eq_ignore_ascii_case(current)))
    });
    let (movement, fell_back) = match named {
        Some(k) => (k, false),
        None => (0, true),
    };
    loop_of(file, movement, fell_back).map(Some)
}

/// Row-vector rotation `RotY(y) × RotX(x) × RotZ(z)` for angles
/// `[x, y, z]` in radians: a point is turned about Y, then X, then Z.
pub fn rotation(angles: [f32; 3]) -> [[f32; 3]; 3] {
    let [x, y, z] = angles;
    let (sx, cx) = x.sin_cos();
    let (sy, cy) = y.sin_cos();
    let (sz, cz) = z.sin_cos();
    // Row-vector forms: each row is where that unit axis goes.
    let rx = [[1.0, 0.0, 0.0], [0.0, cx, sx], [0.0, -sx, cx]];
    let ry = [[cy, 0.0, -sy], [0.0, 1.0, 0.0], [sy, 0.0, cy]];
    let rz = [[cz, sz, 0.0], [-sz, cz, 0.0], [0.0, 0.0, 1.0]];
    mul(mul(ry, rx), rz)
}

fn mul(a: [[f32; 3]; 3], b: [[f32; 3]; 3]) -> [[f32; 3]; 3] {
    std::array::from_fn(|i| std::array::from_fn(|j| (0..3).map(|k| a[i][k] * b[k][j]).sum()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::f32::consts::FRAC_PI_2;

    fn turn(angles: [f32; 3], p: [f32; 3]) -> [f32; 3] {
        let t = Transform { flags: 0, m: rotation(angles), p: [0.0; 3] };
        t.apply(p).map(|v| (v * 1e5).round() / 1e5 + 0.0)
    }

    #[test]
    fn right_hand_rule_on_each_axis() {
        // positive dirZ turns +X towards +Y
        assert_eq!(turn([0.0, 0.0, FRAC_PI_2], [1.0, 0.0, 0.0]), [0.0, 1.0, 0.0]);
        // positive dirX turns +Y towards +Z
        assert_eq!(turn([FRAC_PI_2, 0.0, 0.0], [0.0, 1.0, 0.0]), [0.0, 0.0, 1.0]);
        // positive dirY turns +Z towards +X
        assert_eq!(turn([0.0, FRAC_PI_2, 0.0], [0.0, 0.0, 1.0]), [1.0, 0.0, 0.0]);
    }

    #[test]
    fn y_first_then_x_then_z() {
        // +X: about Y by 90 deg -> -Z; about X by 90 deg -> +Y; about Z by
        // 90 deg -> -X. Any other order gives a different answer.
        assert_eq!(turn([FRAC_PI_2, FRAC_PI_2, FRAC_PI_2], [1.0, 0.0, 0.0]), [-1.0, 0.0, 0.0]);
        // +Z: Y -> +X; X leaves +X; Z -> +Y.
        assert_eq!(turn([FRAC_PI_2, FRAC_PI_2, FRAC_PI_2], [0.0, 0.0, 1.0]), [0.0, 1.0, 0.0]);
    }

    #[test]
    fn current_animation_model() {
        let file = crate::kq_text::parse(
            b"[Alchdoor]\nclassType=KQDoor\ncurrAnimName=opened\nlocX=1\nlocY=2\nlocZ=3\nanimLoops=loops\n\
              [loops]\nclassType=KQAnimLoopList\nnMovement=2\nMovement0=closed\nMovement1=opened\n\
              [movement0]\nloop=door2.kqs door2 0.0 0.0 0 1\n[movement1]\nloop=door3.kqs door3 1.0 0.0 0 1\n",
        );
        let c = current_loop(&file, &file[0]).unwrap().unwrap();
        assert_eq!((c.movement, c.fell_back, c.shape.as_str(), c.sequence.as_str()), (1, false, "door3.kqs", "door3"));
        assert_eq!((c.start, c.speed, c.repeat, c.unknown), (1.0, 0.0, false, 1));
        // Not animated: no answer, no error.
        let plain = crate::kq_text::parse(b"[a]\nshapeName=x.kqs\n");
        assert_eq!(current_loop(&plain, &plain[0]).unwrap(), None);
    }

    #[test]
    fn transition_lines_as_in_conner_anm() {
        let file = crate::kq_text::parse(
            b"[c]
animLoops=l
currAnimName=stop
[l]
nMovement=1
Movement0=stop
              [movement0]
loop=c_stop.kqs c_stop 0.000000 1.0 1 1
nTransition=3
              transition0=-1 run 0.866667 stoprun
;toFrames0=15
transition1=-1 walk 0.850000 stopwalk
              transition2=0.5 walk
",
        );
        let c = current_loop(&file, &file[0]).unwrap().unwrap();
        let t = |from, to: &str, to_phase, via: &str| TransitionLine { from, to: to.into(), to_phase, via: via.into() };
        assert_eq!(c.transitions, vec![t(-1.0, "run", 0.866667, "stoprun"), t(-1.0, "walk", 0.85, "stopwalk")]);
        assert_eq!(c.transition_problems.len(), 1, "{:?}", c.transition_problems);
        assert_eq!(c.unknown, 1);
    }

    #[test]
    fn every_movement_with_its_cues() {
        let file = crate::kq_text::parse(
            b"[door]
classType=KQDoor
currAnimName=closed
animLoops=loops
[loops]
nMovement=2
Movement0=closed
Movement1=open
              [movement0]
loop=door1.kqs door1 0.0 0.0 0 1
[movement1]
loop=door1.kqs door1 0.0 1.0 0 1
nCue=2
cue0=0.0 playsound door4.aud
#cue1=0.5 sendEvent x
",
        );
        let all = movement_loops(&file, &file[0]).unwrap().unwrap();
        assert_eq!(all.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(), ["closed", "open"]);
        assert_eq!((all[1].1.speed, all[1].1.movement), (1.0, 1));
        assert_eq!(all[1].1.cues, vec![(0.0, "playsound door4.aud".to_string())]);
        // nCue=2 but cue1 is commented out: one problem, not a cue.
        assert_eq!(all[1].1.cue_problems.len(), 1);
        assert!(all[0].1.cues.is_empty());
    }

    #[test]
    fn missing_or_unknown_current_animation_uses_movement_0() {
        let list = "[l]\nnMovement=2\nMovement0=stop\nMovement1=go\n[movement0]\nloop=a.kqs a 0.0 1.0 1 1\n[movement1]\nloop=b.kqs b 0 1 1 1\n";
        for head in ["[o]\nanimLoops=l\n", "[o]\nanimLoops=l\ncurrAnimName=gone\n", "[o]\nanimLoops=l\ncurAnimName=go\n"] {
            let file = crate::kq_text::parse(format!("{head}{list}").as_bytes());
            let c = current_loop(&file, &file[0]).unwrap().unwrap();
            assert_eq!((c.movement, c.fell_back, c.shape.as_str()), (0, true, "a.kqs"), "{head}");
        }
    }

    /// Floor 500, surface 800 (water 300 deep).
    fn water(level: Level) -> Option<f32> {
        Some(match level {
            Level::Floor => 500.0,
            Level::Surface => 800.0,
        })
    }

    #[test]
    fn keep_on_ground_raises_never_lowers() {
        let on = &crate::kq_text::parse(b"[a]\nlocX=1\nlocY=2\nlocZ=-1\n")[0];
        let p = Placement::from_section(on);
        assert_eq!(p.keep_on_ground, KeepOnGround::Floor);
        assert_eq!(p.grounded_position(|_| Some(500.0)), [1.0, 2.0, 500.0]);
        assert_eq!(p.grounded_position(|_| None), [1.0, 2.0, -1.0]);
        let high = Placement { position: [1.0, 2.0, 900.0], ..p.clone() };
        assert_eq!(high.grounded_position(|_| Some(500.0)), [1.0, 2.0, 900.0]);
        let off = &crate::kq_text::parse(b"[a]\nlocX=1\nlocY=2\nlocZ=-1\nkeeponground=0\n")[0];
        assert_eq!(Placement::from_section(off).grounded_position(|_| Some(500.0)), [1.0, 2.0, -1.0]);
    }

    #[test]
    fn keep_on_ground_default_uses_the_floor_and_1_the_surface() {
        // The Swamp shack: locZ=-1, no keepOnGround line -> the floor.
        let shack = &crate::kq_text::parse(b"[shack2]\nlocX=1\nlocY=2\nlocZ=-1\n")[0];
        assert_eq!(Placement::from_section(shack).grounded_position(water), [1.0, 2.0, 500.0]);
        let one = &crate::kq_text::parse(b"[knight]\nlocX=1\nlocY=2\nlocZ=-1\nkeepOnGround=1\n")[0];
        let p = Placement::from_section(one);
        assert_eq!(p.keep_on_ground, KeepOnGround::Surface);
        assert_eq!(p.grounded_position(water), [1.0, 2.0, 800.0]);
    }

    #[test]
    fn load_line_replaces_position_and_angles() {
        use crate::world_script::{LoadKq, LoadName};
        let s = &crate::kq_text::parse(b"[stool]
shapeName=stool.kqs
locX=0
locY=0
locZ=0
dirZ=1.0
translucency=200
")[0];
        let load = |name, position, angles| LoadKq {
            file: "stool.kq".into(), name, position, angles, ignored_numbers: 0, from: "permwrld.cs".into(), line: 1, inside_if: false,
        };
        let p = Placement::from_load(s, &load(LoadName::Rename("stool.02".into()), Some([1.0, 2.0, 3.0]), Some([0.0, 0.0, -0.81])));
        assert_eq!((p.name.as_str(), p.position, p.angles), ("stool.02", [1.0, 2.0, 3.0], [0.0, 0.0, -0.81]));
        // Position without angles: angles 0 0 0, not the section's dirZ.
        let p = Placement::from_load(s, &load(LoadName::Keep, Some([1.0, 2.0, 3.0]), None));
        assert_eq!((p.name.as_str(), p.angles), ("stool", [0.0; 3]));
        // File name only: the section's own position and angles.
        let p = Placement::from_load(s, &load(LoadName::None, None, None));
        assert_eq!((p.name.as_str(), p.position, p.angles), ("", [0.0; 3], [0.0, 0.0, 1.0]));
        assert!((p.opacity() - 200.0 / 255.0).abs() < 1e-6);
    }

    #[test]
    fn load_line_z_minus_1_or_2_means_on_the_ground() {
        use crate::world_script::{LoadKq, LoadName};
        let s = &crate::kq_text::parse(b"[tomb]\nshapeName=tomb.kqs\nlocX=0\nlocY=0\nlocZ=-1\nkeeponground=0\n")[0];
        let load = |z| LoadKq {
            file: "tomb.kq".into(), name: LoadName::Keep, position: Some([1.0, 2.0, z]), angles: None, ignored_numbers: 0,
            from: "permwrld.cs".into(), line: 1, inside_if: false,
        };
        // -1 and -2 move the object onto the ground, up or down, even with keepOnGround off.
        for z in [-1.0, -2.0] {
            let p = Placement::from_load(s, &load(z));
            assert_eq!(p.grounded_position(|_| Some(500.0)), [1.0, 2.0, 500.0]);
            assert_eq!(p.grounded_position(|_| Some(-800.0)), [1.0, 2.0, -800.0]);
        }
        // -1 is the floor, -2 the surface (water.md).
        assert_eq!(Placement::from_load(s, &load(-1.0)).grounded_position(water), [1.0, 2.0, 500.0]);
        assert_eq!(Placement::from_load(s, &load(-2.0)).grounded_position(water), [1.0, 2.0, 800.0]);
        // -2 where the floor is above the surface (DeadCity's portal): on the
        // surface, then the default keepOnGround raise lifts it to the floor;
        // with keepOnGround=0 (this template) it stays on the surface.
        let raised = |level| Some(if level == Level::Floor { 2784.0 } else { 0.0 });
        assert_eq!(Placement::from_load(s, &load(-2.0)).grounded_position(raised), [1.0, 2.0, 0.0]);
        let portal = &crate::kq_text::parse(b"[porttp7]
shapeName=poptop.kqs
locX=0
locY=0
locZ=937
")[0];
        assert_eq!(Placement::from_load(portal, &load(-2.0)).grounded_position(raised), [1.0, 2.0, 2784.0]);
        assert_eq!(Placement::from_load(portal, &load(-1.0)).grounded_position(raised), [1.0, 2.0, 2784.0]);
        // Any other value is used as written (keepOnGround is off here).
        let p = Placement::from_load(s, &load(-3.0));
        assert_eq!(p.grounded_position(|_| Some(500.0)), [1.0, 2.0, -3.0]);
        // A .kq file's own locZ=-1 does not take this rule.
        let p = Placement::from_section(s);
        assert_eq!(p.grounded_position(|_| Some(500.0)), [0.0, 0.0, -1.0]);
    }

    #[test]
    // `3.14` is the value as written in the game's files (`dirZ=3.14`), not π.
    #[allow(clippy::approx_constant)]
    fn reads_a_section() {
        let s = &crate::kq_text::parse(b"[barn]\nclassType=KQInterior\nshapeName=barn.dig\nlocX=1\nlocY=2\nlocZ=3\ndirZ=3.14\n")[0];
        let p = Placement::from_section(s);
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
        assert_eq!(p.angles, [0.0, 0.0, 3.14]);
        assert_eq!(p.shape.as_deref(), Some("barn.dig"));
        assert!(!p.has_matrix_key);
        let none = &crate::kq_text::parse(b"[x]\nclassType=KQObject\n")[0];
        let p = Placement::from_section(none);
        assert_eq!((p.position, p.missing_loc), ([0.0; 3], [true; 3]));
        // Haze keys (object-lighting.md rules 21, 22): defaults, then set;
        // the data's misspelling is not read (Q68).
        assert_eq!((p.haze_solid, p.haze_multiplier), (false, 1.0));
        let hazy = &crate::kq_text::parse(b"[t]\nb_hazeSolid=1\nhazeMultiplier=2.0\n")[0];
        assert_eq!((Placement::from_section(hazy).haze_solid, Placement::from_section(hazy).haze_multiplier), (true, 2.0));
        let misspelt = &crate::kq_text::parse(b"[t]\nhazeMulitplier=2.0\n")[0];
        assert_eq!(Placement::from_section(misspelt).haze_multiplier, 1.0);
    }

    #[test]
    fn hidden_objects_are_not_drawn_unless_groups_always_has_0x40() {
        let parse = |t: &str| Placement::from_section(&crate::kq_text::parse(t.as_bytes())[0]);
        assert!(parse("[a]\nlocX=1\n").drawn());
        assert!(!parse("[a]\nhidden=1\n").drawn());
        assert!(!parse("[a]\nhidden=2\n").drawn());
        assert!(parse("[a]\nhidden=0\n").drawn());
        assert!(parse("[a]\nhidden=1\ngroupsAlways=1097\n").drawn());
        assert_eq!(parse("[a]\n").groups_always, 6);
    }

    #[test]
    fn missing_loc_z_is_0_then_lifted_to_the_ground() {
        // Barren's lookout2 (objects.md Q19): locX, locY, no locZ.
        let s = &crate::kq_text::parse(b"[lookout2]\nshapeName=lookout4.dig\nlocX=71571\nlocY=108848\ndirZ=3.14\n")[0];
        let p = Placement::from_section(s);
        assert_eq!((p.position, p.missing_loc), ([71571.0, 108848.0, 0.0], [false, false, true]));
        assert_eq!(p.grounded_position(|_| Some(11592.0)), [71571.0, 108848.0, 11592.0]);
        // Below-zero ground: z 0 stays (raise only), i.e. max(0, ground).
        assert_eq!(p.grounded_position(|_| Some(-300.0)), [71571.0, 108848.0, 0.0]);
    }
}
