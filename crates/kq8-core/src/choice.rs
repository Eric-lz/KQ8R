//! Connor's Choice state (`characters.md` "Connor's Choice state" rules
//! 1–13, Q155): the player picks one of two or three directions by
//! pressing forward or back while facing one of them. Daventry's roof jump
//! (`rooftrap.cs`), Barren's basilisk, Ice World's orcs.
//!
//! This module has the list, the points and the pick; the viewer runs the
//! state on Connor (`connor_view.rs`).

/// The probe's distance ahead of or behind Connor (rule 7).
pub const PROBE: f32 = 200.0;

/// `KQMonster::setState Connor Choice <list>`'s list (rule 2): a script
/// name, then triples `<number>,<x>,<y>`.
#[derive(Debug, Clone, PartialEq)]
pub struct ChoiceList {
    pub script: String,
    /// The triples as written: the number (`DoChoice`'s `$3`) and the point.
    pub points: Vec<(String, [f32; 2])>,
}

impl ChoiceList {
    /// Reads the list (rule 2). `Err`: why not.
    pub fn parse(list: &str) -> Result<Self, String> {
        let mut words = list.split(',').map(str::trim);
        let script = words.next().filter(|s| !s.is_empty()).ok_or("no script name")?.to_string();
        let rest: Vec<&str> = words.collect();
        if rest.is_empty() || rest.len() % 3 != 0 {
            return Err(format!("{} words after the script, not triples", rest.len()));
        }
        let mut points = Vec::new();
        for t in rest.chunks(3) {
            let x = t[1].parse::<f32>().map_err(|_| format!("x '{}' is not a number", t[1]))?;
            let y = t[2].parse::<f32>().map_err(|_| format!("y '{}' is not a number", t[2]))?;
            points.push((t[0].to_string(), [x, y]));
        }
        Ok(ChoiceList { script, points })
    }
}

/// The state while it lasts.
#[derive(Debug, Clone, PartialEq)]
pub struct Choice {
    pub list: ChoiceList,
    /// The points on the circle (rule 6), measured at the first update
    /// after `StartChoice` ran (rule 5); `None` until then.
    pub circle: Option<Vec<(String, [f32; 2])>>,
    /// `DoChoice` was sent (rule 10): any request is taken now and ends
    /// the state (rule 12).
    pub made: bool,
}

impl Choice {
    pub fn new(list: ChoiceList) -> Self {
        Choice { list, circle: None, made: false }
    }
}

/// Rule 6: the level distance from Connor to the first point is the
/// radius; every other point is moved along its own direction from Connor
/// until it is that far too. A point on Connor stays where it is.
pub fn on_circle(at: [f32; 2], points: &[(String, [f32; 2])]) -> Vec<(String, [f32; 2])> {
    let dist = |p: [f32; 2]| ((p[0] - at[0]).powi(2) + (p[1] - at[1]).powi(2)).sqrt();
    let Some(r) = points.first().map(|p| dist(p.1)) else { return Vec::new() };
    points
        .iter()
        .map(|(n, p)| {
            let d = dist(*p);
            let q = if d > 0.0 { [at[0] + (p[0] - at[0]) * r / d, at[1] + (p[1] - at[1]) * r / d] } else { *p };
            (n.clone(), q)
        })
        .collect()
}

/// Rule 7: the point nearest to a probe [`PROBE`] ahead of Connor
/// (`forward`) or behind him; on a tie the earlier one. `heading`: his
/// heading (0 = facing −Y). Returns its place in the list.
pub fn pick(at: [f32; 2], heading: f32, forward: bool, circle: &[(String, [f32; 2])]) -> Option<usize> {
    let s = if forward { PROBE } else { -PROBE };
    let probe = [at[0] + s * heading.sin(), at[1] - s * heading.cos()];
    let d2 = |p: [f32; 2]| (p[0] - probe[0]).powi(2) + (p[1] - probe[1]).powi(2);
    let mut best: Option<(usize, f32)> = None;
    for (i, (_, p)) in circle.iter().enumerate() {
        if best.is_none_or(|(_, b)| d2(*p) < b) {
            best = Some((i, d2(*p)));
        }
    }
    best.map(|(i, _)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    const ROOF: &str = "rooftrap.cs,0,220000,183200,1,230000,183200";

    #[test]
    fn the_list_is_a_script_then_triples() {
        let l = ChoiceList::parse(ROOF).unwrap();
        assert_eq!(l.script, "rooftrap.cs");
        assert_eq!(l.points, vec![("0".to_string(), [220000.0, 183200.0]), ("1".to_string(), [230000.0, 183200.0])]);
        assert!(ChoiceList::parse("rooftrap.cs,0,220000").is_err());
        assert!(ChoiceList::parse("rooftrap.cs").is_err());
    }

    #[test]
    fn daventry_s_roof_points_and_picks() {
        // Rule 6: after StartChoice he stands at (228239, 183179): R = 8,239,
        // point 1 becomes about (236478, 183277).
        let at = [228239.0, 183179.0];
        let c = on_circle(at, &ChoiceList::parse(ROOF).unwrap().points);
        let r = ((c[0].1[0] - at[0]).powi(2) + (c[0].1[1] - at[1]).powi(2)).sqrt();
        assert!((r - 8239.0).abs() < 1.0, "{r}");
        assert!((c[1].1[0] - 236478.0).abs() < 2.0 && (c[1].1[1] - 183277.0).abs() < 2.0, "{:?}", c[1]);
        // Rule 7: facing -1.57, forward picks 0 (west), back picks 1.
        assert_eq!(pick(at, -1.57, true, &c), Some(0));
        assert_eq!(pick(at, -1.57, false, &c), Some(1));
        // Turned round: the other way.
        assert_eq!(pick(at, 1.57, true, &c), Some(1));
    }
}
