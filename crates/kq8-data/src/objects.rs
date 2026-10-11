//! Object placements from the `.kq` files: which model goes where, turned
//! how.
//!
//! Rules from the behaviour note `docs/behaviour/objects.md` (session A,
//! answers Q9/Q10); key names observed in the data:
//!
//! - Position `locX`, `locY`, `locZ` in game units (`isGroundRelative` is
//!   never read by the game). `keepOnGround` is on unless the section says
//!   `keepOnGround=0`; then an object below the ground is raised onto it,
//!   never lowered (docs/behaviour/buildings.md, Q10 follow-up).
//! - Angles `dirX`, `dirY`, `dirZ` in radians, right-hand rule (positive
//!   `dirZ` turns +X towards +Y). A point is turned about Y first, then X,
//!   then Z: with row vectors the matrix is `RotY × RotX × RotZ`.
//! - No scale. (A full matrix key `<prefix>RMat` exists in the game but no
//!   file uses it; such sections are reported, not placed.)

use crate::kq_text::Section;
use crate::shape::Transform;

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
    /// `keepOnGround`: true unless the section sets it to 0.
    pub keep_on_ground: bool,
    /// `flags` as written, if present (animation start options; see
    /// objects.md "Animated objects at load" rule 4).
    pub flags: Option<String>,
}

impl Placement {
    /// `None` if the section has no position at all.
    pub fn from_section(s: &Section) -> Option<Self> {
        let num = |k: &str| s.number(k).and_then(Result::ok).map(|v| v as f32);
        let position = [num("locX")?, num("locY")?, num("locZ")?];
        let angles = [num("dirX").unwrap_or(0.0), num("dirY").unwrap_or(0.0), num("dirZ").unwrap_or(0.0)];
        Some(Placement {
            name: s.name.clone(),
            class: s.get("classType").map(str::to_string),
            shape: s.get("shapeName").filter(|v| !v.is_empty()).map(str::to_string),
            position,
            angles,
            has_matrix_key: s.entries.iter().any(|(k, _)| k.to_ascii_lowercase().ends_with("rmat")),
            keep_on_ground: s.number("keepOnGround").and_then(Result::ok).is_none_or(|v| v != 0.0),
            flags: s.get("flags").map(str::to_string),
        })
    }

    /// Position after the ground rule: with `keepOnGround`, an object below
    /// `ground` (the ground height under locX, locY) is raised onto it.
    pub fn grounded_position(&self, ground: Option<f32>) -> [f32; 3] {
        let [x, y, z] = self.position;
        match ground {
            Some(g) if self.keep_on_ground && z < g => [x, y, g],
            _ => self.position,
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
    /// Fourth value; effect not traced.
    pub unknown: i64,
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
    let section = find(&format!("movement{movement}")).ok_or(format!("section [movement{movement}] not in the file"))?;
    let line = section.get("loop").ok_or(format!("[movement{movement}] has no loop line"))?;
    let words: Vec<&str> = line.split_whitespace().collect();
    let [shape, sequence, start, speed, repeat, unknown] = words[..] else {
        return Err(format!("[movement{movement}] loop line '{line}' does not have 6 parts"));
    };
    let bad = |what: &str| format!("[movement{movement}] loop line '{line}': {what} is not a number");
    Ok(Some(CurrentLoop {
        movement,
        fell_back,
        shape: shape.to_string(),
        sequence: sequence.to_string(),
        start: start.parse().map_err(|_| bad("start"))?,
        speed: speed.parse().map_err(|_| bad("speed"))?,
        repeat: repeat.parse::<i64>().map_err(|_| bad("repeat"))? != 0,
        unknown: unknown.parse().map_err(|_| bad("fourth value"))?,
    }))
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
    fn missing_or_unknown_current_animation_uses_movement_0() {
        let list = "[l]\nnMovement=2\nMovement0=stop\nMovement1=go\n[movement0]\nloop=a.kqs a 0.0 1.0 1 1\n[movement1]\nloop=b.kqs b 0 1 1 1\n";
        for head in ["[o]\nanimLoops=l\n", "[o]\nanimLoops=l\ncurrAnimName=gone\n", "[o]\nanimLoops=l\ncurAnimName=go\n"] {
            let file = crate::kq_text::parse(format!("{head}{list}").as_bytes());
            let c = current_loop(&file, &file[0]).unwrap().unwrap();
            assert_eq!((c.movement, c.fell_back, c.shape.as_str()), (0, true, "a.kqs"), "{head}");
        }
    }

    #[test]
    fn keep_on_ground_raises_never_lowers() {
        let on = &crate::kq_text::parse(b"[a]\nlocX=1\nlocY=2\nlocZ=-1\n")[0];
        let p = Placement::from_section(on).unwrap();
        assert!(p.keep_on_ground);
        assert_eq!(p.grounded_position(Some(500.0)), [1.0, 2.0, 500.0]);
        assert_eq!(p.grounded_position(None), [1.0, 2.0, -1.0]);
        let high = Placement { position: [1.0, 2.0, 900.0], ..p.clone() };
        assert_eq!(high.grounded_position(Some(500.0)), [1.0, 2.0, 900.0]);
        let off = &crate::kq_text::parse(b"[a]\nlocX=1\nlocY=2\nlocZ=-1\nkeeponground=0\n")[0];
        assert_eq!(Placement::from_section(off).unwrap().grounded_position(Some(500.0)), [1.0, 2.0, -1.0]);
    }

    #[test]
    fn reads_a_section() {
        let s = &crate::kq_text::parse(b"[barn]\nclassType=KQInterior\nshapeName=barn.dig\nlocX=1\nlocY=2\nlocZ=3\ndirZ=3.14\n")[0];
        let p = Placement::from_section(s).unwrap();
        assert_eq!(p.position, [1.0, 2.0, 3.0]);
        assert_eq!(p.angles, [0.0, 0.0, 3.14]);
        assert_eq!(p.shape.as_deref(), Some("barn.dig"));
        assert!(!p.has_matrix_key);
        let none = &crate::kq_text::parse(b"[x]\nclassType=KQObject\n")[0];
        assert!(Placement::from_section(none).is_none());
    }
}
