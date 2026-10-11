//! Object placements from the `.kq` files: which model goes where, turned
//! how.
//!
//! Rules from the behaviour note `docs/behaviour/objects.md` (session A,
//! answers Q9/Q10); key names observed in the data:
//!
//! - Position `locX`, `locY`, `locZ` in game units, used exactly as written
//!   (`isGroundRelative` is never read by the game).
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
        })
    }

    /// The placement as a row-vector transform: point × rotation + position.
    pub fn transform(&self) -> Transform {
        Transform { flags: 0, m: rotation(self.angles), p: self.position }
    }
}

/// The model of an animated object's current animation.
///
/// Layout observed in the data (e.g. Daventry `alchdoor.kq`): the object
/// section has `animLoops=<list>` and `currAnimName=<name>`; the section
/// `[<list>]` (classType `KQAnimLoopList`) has `nMovement=N` and
/// `Movement0..N-1=<name>`; section `[movement<k>]` has
/// `loop=<model.kqs> <loop name> <numbers...>`. The model is the first
/// word of the `loop` line of the movement whose name is `currAnimName`.
/// What the numbers mean (which frame is shown) is not known (Q12).
///
/// Returns `Ok(None)` if the section is not animated this way.
pub fn current_loop_shape(file: &[Section], s: &Section) -> Result<Option<String>, String> {
    let Some(list_name) = s.get("animLoops") else { return Ok(None) };
    let current = s.get("currAnimName").ok_or("animLoops but no currAnimName")?;
    let find = |name: &str| file.iter().find(|x| x.name.eq_ignore_ascii_case(name));
    let list = find(list_name).ok_or(format!("animation list [{list_name}] not in the file"))?;
    let count = list
        .number("nMovement")
        .ok_or(format!("[{list_name}] has no nMovement"))??;
    let k = (0..count as usize)
        .find(|k| list.get(&format!("Movement{k}")).is_some_and(|m| m.eq_ignore_ascii_case(current)))
        .ok_or(format!("currAnimName={current} is not one of [{list_name}]'s {count} movements"))?;
    let movement = find(&format!("movement{k}")).ok_or(format!("section [movement{k}] not in the file"))?;
    let line = movement.get("loop").ok_or(format!("[movement{k}] has no loop line"))?;
    let shape = line.split_whitespace().next().ok_or(format!("[movement{k}] loop line is empty"))?;
    Ok(Some(shape.to_string()))
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
        assert_eq!(current_loop_shape(&file, &file[0]).unwrap().as_deref(), Some("door3.kqs"));
        // Not animated: no answer, no error.
        let plain = crate::kq_text::parse(b"[a]\nshapeName=x.kqs\n");
        assert_eq!(current_loop_shape(&plain, &plain[0]).unwrap(), None);
        // A name that is not in the list is an error.
        let bad = crate::kq_text::parse(b"[a]\nanimLoops=l\ncurrAnimName=gone\n[l]\nnMovement=1\nMovement0=stop\n");
        assert!(current_loop_shape(&bad, &bad[0]).is_err());
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
