//! The Rope and Hook (M6 6a): the rope places' settings, Connor's rope
//! status and the hook icons.
//!
//! Rules from `docs/behaviour/ropeandhook.md` (session A, 2026-10-21;
//! checked by the user in the original):
//!
//! - Rule 1: a rope place is a pair of trap boxes whose `On` scripts run
//!   `KQConner::setRope Bottom|Top x y z rx ry rz` and whose `Off`
//!   scripts run `KQConner::setRope None`.
//! - Rule 2: `setRope` takes `None`, `Top` or `Bottom` (any case) and 1,
//!   4 or 7 words after the command (place and direction 0 when left
//!   out); anything else prints the usage line and changes nothing. The
//!   place and direction are stored only when the change is accepted
//!   (rule 4); the two `setRopeCam` spots are cleared on every `setRope`.
//! - Rule 3: `setRopeCam x1 y1 z1 x2 y2 z2` (a spot counts as set only
//!   with all three coordinates non-zero), `setRopeCam2 x y z` (the
//!   camera when the rope is taken at the top), `setRopeCamTopLimit z` /
//!   `setRopeCamBottomLimit z` (0 = no limit), `setRopeCamAngle angle
//!   range` (kept, never used: "Do not reproduce", the aim test always
//!   passes). Only `setRope None` clears the limits; the rest is sticky.
//! - Rule 4: the status changes only if Connor carries an item of
//!   category `Rope`; otherwise both hook icons are hidden and nothing
//!   else changes (`None` too).
//! - Rule 5: the game time of each accepted change is noted (nothing
//!   reads it: "Do not reproduce"; kept for the log only).
//! - Rule 6: an accepted `setRope None` also puts a held rope back and
//!   clears both camera limits.
//! - Rule 7: `HookUp` while the status is Bottom, `HookDown` while Top,
//!   none for None or while Connor is on the rope.
//!
//! The record is one plain struct of Connor's, so a saved game can keep
//! it later (rule 32; we have no saved games yet).

use crate::potions::StatusIcon;

/// Connor's rope status (rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Status {
    #[default]
    None,
    /// At the top of a rope: the rope can be let down.
    Top,
    /// At the foot of a rope: the rope can be thrown up.
    Bottom,
}

impl Status {
    /// The word as `setRope` takes it, any case.
    pub fn from_word(w: &str) -> Option<Status> {
        match w.to_ascii_lowercase().as_str() {
            "none" => Some(Status::None),
            "top" => Some(Status::Top),
            "bottom" => Some(Status::Bottom),
            _ => None,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Status::None => "none",
            Status::Top => "top",
            Status::Bottom => "bottom",
        }
    }
}

/// The two hook icons (rule 7), shown in the status icon places.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hook {
    /// At the foot: `INVITEM_HookUp` (`hookup.pba`).
    Up,
    /// At the top: `INVITEM_HookDown` (`hookdown.pba`).
    Down,
}

impl Hook {
    /// The item type whose picture the icon shows (`Invinfo.itl`).
    pub fn item_type(self) -> &'static str {
        match self {
            Hook::Up => "INVITEM_HookUp",
            Hook::Down => "INVITEM_HookDown",
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Hook::Up => "HookUp",
            Hook::Down => "HookDown",
        }
    }
}

/// The category of the rope's item type (rule 4; `INVITEM_RopeAndHook`).
pub const ROPE_CATEGORY: &str = "Rope";

/// Connor's rope settings (rules 1–6): set by the rope places' scripts.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Rope {
    pub status: Status,
    /// The rope's place and direction (radians, as an object's
    /// `dirX/dirY/dirZ`), stored only with an accepted change.
    pub place: [f32; 3],
    pub dir: [f32; 3],
    /// `setRopeCam`'s two spots; `None` when not set (a 0 coordinate).
    pub cam: [Option<[f32; 3]>; 2],
    /// `setRopeCam2`: the camera when the rope is taken at the top.
    pub top_cam: Option<[f32; 3]>,
    /// `setRopeCamAngle`: heading and tolerance (unused, "Do not
    /// reproduce").
    pub angle: Option<(f32, f32)>,
    /// `setRopeCamTopLimit` / `BottomLimit`; 0 = no limit.
    pub top_limit: f32,
    pub bottom_limit: f32,
    /// Game time of the last accepted change (rule 5; only logged).
    pub changed_at: Option<f64>,
}

/// What a command did, for the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Done {
    /// The result for `$Console::Return`.
    pub result: &'static str,
    /// The line for `rope.log`.
    pub line: String,
    /// An accepted `setRope None`: a held rope goes back (rule 6).
    pub put_back: bool,
}

fn done(result: &'static str, line: String) -> Done {
    Done { result, line, put_back: false }
}

/// The numbers of `words`, or `None` if one isn't a number.
fn numbers(words: &[String]) -> Option<Vec<f32>> {
    words.iter().map(|w| crate::script::looks_like_number(w).then(|| crate::script::number_value(w) as f32)).collect()
}

fn spot(v: &[f32]) -> Option<[f32; 3]> {
    let s = [v[0], v[1], v[2]];
    s.iter().all(|c| *c != 0.0).then_some(s)
}

fn fmt3(v: [f32; 3]) -> String {
    format!("({:.0}, {:.0}, {:.0})", v[0], v[1], v[2])
}

fn fmt_spot(s: Option<[f32; 3]>) -> String {
    s.map_or("not set".to_string(), fmt3)
}

impl Rope {
    /// One of the six `KQConner::setRope*` commands (`words[0]` is the
    /// command); `carries`: Connor carries an item of category `Rope`;
    /// `now`: game time. `None` for another command.
    pub fn command(&mut self, words: &[String], carries: bool, now: f64) -> Option<Done> {
        let args = &words[1..];
        let usage = |u: &str| done("1", format!("usage: {u}: '{}'; nothing changes", words.join(" ")));
        Some(match words[0].as_str() {
            "KQConner::setRope" => {
                const USAGE: &str = "KQConner::setRope None|Top|Bottom [x y z [rx ry rz]]";
                let Some(status) = args.first().and_then(|w| Status::from_word(w)) else { return Some(usage(USAGE)) };
                if !matches!(args.len(), 1 | 4 | 7) {
                    return Some(usage(USAGE));
                }
                let Some(v) = numbers(&args[1..]) else { return Some(usage(USAGE)) };
                let place = if v.len() >= 3 { [v[0], v[1], v[2]] } else { [0.0; 3] };
                let dir = if v.len() == 6 { [v[3], v[4], v[5]] } else { [0.0; 3] };
                // Rule 2: cleared on every setRope, accepted or not.
                let cleared = self.cam.iter().any(Option::is_some);
                self.cam = [None, None];
                let cleared = if cleared { "; the setRopeCam spots cleared" } else { "" };
                let old = self.status;
                if !carries {
                    // Rule 4: nothing changes; the icons are hidden.
                    return Some(done("0", format!("setRope {}: refused: Connor carries no rope (rule 4); status stays {}, the hook icons hidden{cleared}", status.name(), old.name())));
                }
                self.status = status;
                self.place = place;
                self.dir = dir;
                self.changed_at = Some(now);
                let mut line = format!("status {} -> {} at game time {now:.3} (rule 2), place {}, direction ({:.3}, {:.3}, {:.3}){cleared}", old.name(), status.name(), fmt3(place), dir[0], dir[1], dir[2]);
                let mut put_back = false;
                if status == Status::None {
                    // Rule 6.
                    line += &format!("; the camera limits cleared (were top {:.0}, bottom {:.0}; rule 6)", self.top_limit, self.bottom_limit);
                    self.top_limit = 0.0;
                    self.bottom_limit = 0.0;
                    put_back = true;
                }
                Done { result: "0", line, put_back }
            }
            "KQConner::setRopeCam" => {
                const USAGE: &str = "KQConner::setRopeCam x1 y1 z1 x2 y2 z2";
                let Some(v) = numbers(args).filter(|v| v.len() == 6) else { return Some(usage(USAGE)) };
                self.cam = [spot(&v[0..3]), spot(&v[3..6])];
                done("0", format!("camera spots for the climb: 1 {}, 2 {} (rule 3: set only with no 0 coordinate)", fmt_spot(self.cam[0]), fmt_spot(self.cam[1])))
            }
            "KQConner::setRopeCam2" => {
                const USAGE: &str = "KQConner::setRopeCam2 x y z";
                let Some(v) = numbers(args).filter(|v| v.len() == 3) else { return Some(usage(USAGE)) };
                self.top_cam = Some([v[0], v[1], v[2]]);
                done("0", format!("the camera when the rope is taken at the top: {} (rule 3; kept until set again)", fmt3([v[0], v[1], v[2]])))
            }
            "KQConner::setRopeCamAngle" => {
                const USAGE: &str = "KQConner::setRopeCamAngle angle range";
                let Some(v) = numbers(args).filter(|v| v.len() == 2) else { return Some(usage(USAGE)) };
                self.angle = Some((v[0], v[1]));
                done("0", format!("camera angle {:.3}, range {:.3}: kept, not used (the aim test always passes: \"Do not reproduce\")", v[0], v[1]))
            }
            c @ ("KQConner::setRopeCamTopLimit" | "KQConner::setRopeCamBottomLimit") => {
                let top = c.ends_with("TopLimit");
                let Some(v) = numbers(args).filter(|v| v.len() == 1) else { return Some(usage(&format!("{c} z"))) };
                if top {
                    self.top_limit = v[0];
                } else {
                    self.bottom_limit = v[0];
                }
                done("0", format!("camera {} limit {:.0}{} (rule 3)", if top { "top" } else { "bottom" }, v[0], if v[0] == 0.0 { " (no limit)" } else { "" }))
            }
            _ => return None,
        })
    }

    /// Rule 7: the hook icon wanted now; `carries`: Connor carries the
    /// rope (rule 4); `on_rope`: in the rope state's hanging part.
    pub fn icon(&self, carries: bool, on_rope: bool) -> Option<Hook> {
        if !carries || on_rope {
            return None;
        }
        match self.status {
            Status::None => None,
            Status::Bottom => Some(Hook::Up),
            Status::Top => Some(Hook::Down),
        }
    }
}

/// Rule 7: puts `wanted` in the status icon places (the first free one:
/// STAND-IN, Q147 a) and takes the other hook icon out. Returns the log
/// lines (the places as `IDCTL_TIMER_<n>`).
pub fn sync_icon(places: &mut [Option<StatusIcon>; crate::potions::ICON_PLACES], wanted: Option<Hook>) -> Vec<String> {
    use crate::potions::{free_place, take_place};
    let mut out = Vec::new();
    for h in [Hook::Up, Hook::Down] {
        let icon = StatusIcon::Hook(h);
        let has = places.contains(&Some(icon.clone()));
        if Some(h) == wanted && !has {
            match take_place(places, icon) {
                Some(k) => out.push(format!("{} shown in IDCTL_TIMER_{} (rule 7; the first free place: stand-in, Q147 a)", h.name(), k + 1)),
                None => out.push(format!("{} not shown: all 7 status places are taken", h.name())),
            }
        } else if Some(h) != wanted && has {
            if let Some(k) = free_place(places, &icon) {
                out.push(format!("{} hidden (IDCTL_TIMER_{} free; rule 7)", h.name(), k + 1));
            }
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn w(line: &str) -> Vec<String> {
        line.split_whitespace().map(str::to_string).collect()
    }

    #[test]
    fn set_rope_needs_the_rope_and_stores_the_place_only_when_accepted() {
        let mut r = Rope::default();
        let d = r.command(&w("KQConner::setRope Bottom 147646 207586 7800 0 0 -1.57"), false, 1.0).unwrap();
        assert!(d.line.contains("refused"), "{}", d.line);
        assert_eq!(r.status, Status::None);
        assert_eq!(r.place, [0.0; 3], "not stored when refused");
        let d = r.command(&w("KQConner::setRope bottom 147646 207586 7800 0 0 -1.57"), true, 2.0).unwrap();
        assert_eq!(d.result, "0");
        assert_eq!(r.status, Status::Bottom, "any case");
        assert_eq!(r.place, [147646.0, 207586.0, 7800.0]);
        assert_eq!(r.dir, [0.0, 0.0, -1.57]);
        assert_eq!(r.changed_at, Some(2.0));
        // Without the rope, None leaves the status as it was (rule 4).
        r.command(&w("KQConner::setRope None"), false, 3.0);
        assert_eq!(r.status, Status::Bottom);
    }

    #[test]
    fn set_rope_takes_one_four_or_seven_words() {
        let mut r = Rope::default();
        assert_eq!(r.command(&w("KQConner::setRope Top 1 2"), true, 0.0).unwrap().result, "1");
        assert_eq!(r.command(&w("KQConner::setRope Sideways"), true, 0.0).unwrap().result, "1");
        assert_eq!(r.status, Status::None);
        r.command(&w("KQConner::setRope Top 1 2 3"), true, 0.0);
        assert_eq!((r.status, r.place, r.dir), (Status::Top, [1.0, 2.0, 3.0], [0.0; 3]));
        r.command(&w("KQConner::setRope Bottom"), true, 0.0);
        assert_eq!((r.status, r.place), (Status::Bottom, [0.0; 3]));
    }

    #[test]
    fn camera_spots_limits_and_their_clearing() {
        let mut r = Rope::default();
        r.command(&w("KQConner::setRopeCam 1 2 3 4 0 6"), true, 0.0);
        assert_eq!(r.cam, [Some([1.0, 2.0, 3.0]), None], "a 0 coordinate: not set");
        r.command(&w("KQConner::setRopeCam2 151584 210996 8520"), true, 0.0);
        r.command(&w("KQConner::setRopeCamTopLimit 9000"), true, 0.0);
        r.command(&w("KQConner::setRopeCamBottomLimit 4000"), true, 0.0);
        // A refused setRope still clears the spots (rule 2).
        r.command(&w("KQConner::setRope Top 1 2 3 0 0 0"), false, 0.0);
        assert_eq!(r.cam, [None, None]);
        r.command(&w("KQConner::setRope Top 1 2 3 0 0 0"), true, 0.0);
        assert_eq!((r.top_limit, r.bottom_limit), (9000.0, 4000.0), "Top/Bottom keep the limits");
        let d = r.command(&w("KQConner::setRope None"), true, 0.0).unwrap();
        assert!(d.put_back);
        assert_eq!((r.top_limit, r.bottom_limit), (0.0, 0.0), "None clears them (rule 6)");
        assert_eq!(r.top_cam, Some([151584.0, 210996.0, 8520.0]), "sticky");
    }

    #[test]
    fn the_hook_icon_follows_the_status() {
        let mut r = Rope { status: Status::Bottom, ..Default::default() };
        assert_eq!(r.icon(true, false), Some(Hook::Up));
        assert_eq!(r.icon(false, false), None, "no rope, no icon (rule 4)");
        assert_eq!(r.icon(true, true), None, "not while on the rope");
        r.status = Status::Top;
        assert_eq!(r.icon(true, false), Some(Hook::Down));
        let mut places: [Option<StatusIcon>; crate::potions::ICON_PLACES] = Default::default();
        places[0] = Some(StatusIcon::Mask);
        assert_eq!(sync_icon(&mut places, Some(Hook::Up)).len(), 1);
        assert_eq!(places[1], Some(StatusIcon::Hook(Hook::Up)), "the first free place");
        sync_icon(&mut places, Some(Hook::Down));
        assert_eq!(places[1], Some(StatusIcon::Hook(Hook::Down)), "the old one goes first");
        sync_icon(&mut places, None);
        assert_eq!(places[1], None);
    }
}
