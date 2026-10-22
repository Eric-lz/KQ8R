//! The magic map's files (M6 6d-1, `docs/behaviour/magic-map.md`): a
//! world's map (`classType=KQMap`, `GAME/Kq/<x>map.kq`), a teleporter's
//! portal (`classType=KQPortal`, `GAME/Kq/<x>tele.kq`), and the fog's run
//! lengths (`rle0`, `rle1`, …) as the map files and saved games store them.
//!
//! From the note:
//!
//! - "Data used": the keys of each file.
//! - Rule 3: `mapName=#n` is text n of `500.MSG`; the map index is
//!   n − 38.
//! - Rule 16: the fog is stored as run lengths over the 460 rows, each of
//!   **464** places (the 460 pixels and 4 unused places), top to bottom;
//!   `rle0` counts places still in the fog, `rle1` the next run out of it,
//!   alternating, runs going on from one row into the next; one more key
//!   with −1 ends the list (`rle0=-1`: nothing revealed). The unused
//!   places are written as in the fog and ignored on reading.
//! - "Do not reproduce" 7: our own fog is one yes/no per pixel; the
//!   464-place rows are kept only for the original's form.

use crate::kq_text::Section;

/// Rule 4: the map pictures' width and height in pixels.
pub const MAP_SIZE: usize = 460;
/// Rule 16: places per stored row (460 pixels and 4 unused).
pub const ROW_PLACES: usize = 464;
/// Rule 3: `mapName` #38 is map index 0.
pub const FIRST_NAME: u32 = 38;
/// Rule 3: map indexes 0–15.
pub const MAP_COUNT: usize = 16;

/// A `KQMap` file's keys ("Data used").
#[derive(Debug, Clone, PartialEq)]
pub struct MapKeys {
    /// The section's name (`davMap`).
    pub name: String,
    pub north: f64,
    pub south: f64,
    pub east: f64,
    pub west: f64,
    /// `useFog` (rule 14; all shipped maps 1).
    pub use_fog: bool,
    /// `fogRadius` in world units; `None` when missing (rule 13: then the
    /// world's `hazeDistance`).
    pub fog_radius: Option<f64>,
    /// `holdMapLoc` (rule 15).
    pub hold_map_loc: bool,
    /// `isActive` (rule 2; all shipped files 0).
    pub is_active: bool,
    pub blank_map: String,
    pub full_map: String,
    pub arrow_pba: String,
    /// `mapName=#n`: n (rule 8).
    pub map_name: Option<u32>,
    /// `earnedTeleport` (rule 37; only in saved maps).
    pub earned_teleport: bool,
    /// `TeleportScript` / `TeleportPortalKQ` (rule 39), empty → `None`.
    pub teleport_script: Option<String>,
    pub teleport_portal: Option<String>,
    /// `cMapIcons` and `mapIcon<n>`, `srcIcon<n>`, `xIcon<n>`, `yIcon<n>`
    /// (rule 21).
    pub icons: Vec<MapIcon>,
    /// `rle0`, `rle1`, … up to (not including) the −1 (rule 16).
    pub runs: Vec<i64>,
}

/// One icon on a map picture (rules 19–21).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MapIcon {
    /// The map picture it belongs to (`davmap.pbm`).
    pub picture: String,
    /// The icon file (`dv_tree2.bmp`).
    pub file: String,
    pub x: i32,
    pub y: i32,
}

fn text(s: &Section, key: &str) -> Option<String> {
    s.get(key).map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

impl MapKeys {
    /// `None` when the section is not a `KQMap`; `Some(Err)` when a key the
    /// map needs is missing or not a number.
    pub fn from_section(s: &Section) -> Option<Result<Self, String>> {
        if !s.get("classType")?.trim().eq_ignore_ascii_case("KQMap") {
            return None;
        }
        Some(Self::read(s))
    }

    fn read(s: &Section) -> Result<Self, String> {
        let number = |k: &str| s.number(k).ok_or_else(|| format!("no {k}")).and_then(|r| r);
        let flag = |k: &str| s.number(k).and_then(Result::ok).is_some_and(|v| v != 0.0);
        let count = s.number("cMapIcons").and_then(Result::ok).map_or(0, |v| v.max(0.0) as usize);
        let mut icons = Vec::new();
        for n in 0..count {
            let picture = text(s, &format!("mapIcon{n}")).ok_or_else(|| format!("no mapIcon{n}"))?;
            let file = text(s, &format!("srcIcon{n}")).ok_or_else(|| format!("no srcIcon{n}"))?;
            let x = number(&format!("xIcon{n}"))? as i32;
            let y = number(&format!("yIcon{n}"))? as i32;
            icons.push(MapIcon { picture, file, x, y });
        }
        let mut runs = Vec::new();
        for n in 0.. {
            let Some(v) = s.number(&format!("rle{n}")) else { break };
            let v = v? as i64;
            if v < 0 {
                break;
            }
            runs.push(v);
        }
        Ok(MapKeys {
            name: s.name.clone(),
            north: number("north")?,
            south: number("south")?,
            east: number("east")?,
            west: number("west")?,
            use_fog: flag("useFog"),
            fog_radius: s.number("fogRadius").and_then(Result::ok),
            hold_map_loc: flag("holdMapLoc"),
            is_active: flag("isActive"),
            blank_map: text(s, "blankMap").ok_or("no blankMap")?,
            full_map: text(s, "fullMap").ok_or("no fullMap")?,
            arrow_pba: text(s, "arrowPBA").unwrap_or_else(|| "maparrow.pba".into()),
            map_name: s.get("mapName").and_then(|v| v.trim().strip_prefix('#')).and_then(|v| v.parse().ok()),
            earned_teleport: flag("earnedTeleport"),
            teleport_script: text(s, "TeleportScript"),
            teleport_portal: text(s, "TeleportPortalKQ"),
            icons,
            runs,
        })
    }

    /// Rule 3: the map index (`mapName` − 38), if it is one of 0–15.
    pub fn index(&self) -> Option<usize> {
        self.map_name.and_then(|n| n.checked_sub(FIRST_NAME)).map(|i| i as usize).filter(|i| *i < MAP_COUNT)
    }

    /// The keys written back in the saved form's order of our choosing
    /// (the map's own keys; the usual object keys are not ours to write).
    pub fn to_text(&self) -> String {
        let mut out = format!("[{}]\r\nclassType=KQMap\r\n", self.name);
        let mut put = |k: &str, v: String| out.push_str(&format!("{k}={v}\r\n"));
        for (k, v) in [("north", self.north), ("west", self.west), ("south", self.south), ("east", self.east)] {
            put(k, format!("{v}"));
        }
        put("useFog", u8::from(self.use_fog).to_string());
        if let Some(r) = self.fog_radius {
            put("fogRadius", format!("{r}"));
        }
        put("holdMapLoc", u8::from(self.hold_map_loc).to_string());
        put("isActive", u8::from(self.is_active).to_string());
        put("blankMap", self.blank_map.clone());
        put("fullMap", self.full_map.clone());
        put("arrowPBA", self.arrow_pba.clone());
        put("mapName", self.map_name.map_or(String::new(), |n| format!("#{n}")));
        put("earnedTeleport", u8::from(self.earned_teleport).to_string());
        put("TeleportScript", self.teleport_script.clone().unwrap_or_default());
        put("TeleportPortalKQ", self.teleport_portal.clone().unwrap_or_default());
        put("cMapIcons", self.icons.len().to_string());
        for (n, i) in self.icons.iter().enumerate() {
            put(&format!("mapIcon{n}"), i.picture.clone());
            put(&format!("srcIcon{n}"), i.file.clone());
            put(&format!("xIcon{n}"), i.x.to_string());
            put(&format!("yIcon{n}"), i.y.to_string());
        }
        for (n, r) in self.runs.iter().enumerate() {
            put(&format!("rle{n}"), r.to_string());
        }
        put(&format!("rle{}", self.runs.len()), "-1".into());
        out
    }
}

/// The fog, one yes/no per map pixel, row by row from the top: `true` =
/// out of the fog (revealed).
pub type Revealed = Vec<bool>;

/// What reading a run list found besides the fog.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RunCheck {
    /// Places the runs cover.
    pub places: usize,
    /// Unused places (columns 460–463) a run marked out of the fog
    /// (expected 0; ignored).
    pub unused_revealed: usize,
    /// Places beyond the last row (ignored).
    pub beyond: usize,
}

/// Rule 16: the fog from its run lengths (the −1 already left out). Runs
/// that stop early leave the rest in the fog.
pub fn fog_from_runs(runs: &[i64]) -> (Revealed, RunCheck) {
    let mut fog = vec![false; MAP_SIZE * MAP_SIZE];
    let total = MAP_SIZE * ROW_PLACES;
    let mut check = RunCheck::default();
    let mut place = 0usize;
    for (k, &run) in runs.iter().enumerate() {
        let run = run.max(0) as usize;
        let out = k % 2 == 1;
        let end = place + run;
        if out {
            for p in place..end.min(total) {
                let (row, col) = (p / ROW_PLACES, p % ROW_PLACES);
                if col < MAP_SIZE {
                    fog[row * MAP_SIZE + col] = true;
                } else {
                    check.unused_revealed += 1;
                }
            }
        }
        check.beyond += end.saturating_sub(total.max(place));
        place = end;
    }
    check.places = place;
    (fog, check)
}

/// Rule 16: the run lengths of a fog (without the closing −1): every
/// run up to the last place, the unused places in the fog. Nothing
/// revealed gives no runs at all (written as `rle0=-1`, as the shipped
/// files have).
pub fn runs_from_fog(fog: &[bool]) -> Vec<i64> {
    if !fog.iter().any(|v| *v) {
        return Vec::new();
    }
    let mut runs = Vec::new();
    let mut out = false;
    let mut run = 0i64;
    for row in 0..MAP_SIZE {
        for col in 0..ROW_PLACES {
            let here = col < MAP_SIZE && fog.get(row * MAP_SIZE + col).copied().unwrap_or(false);
            if here != out {
                runs.push(run);
                run = 0;
                out = here;
            }
            run += 1;
        }
    }
    runs.push(run);
    runs
}

/// How many pixels are out of the fog.
pub fn revealed_count(fog: &[bool]) -> usize {
    fog.iter().filter(|v| **v).count()
}

/// A `KQPortal` file's keys (rules 39–40, "Data used").
#[derive(Debug, Clone, PartialEq)]
pub struct PortalKeys {
    pub name: String,
    /// `world`: the destination world's file (`Daventry.kq`).
    pub world: Option<String>,
    pub enter_anim: Option<String>,
    pub exit_anim: Option<String>,
    pub enter_anm_file: Option<String>,
    pub exit_anm_file: Option<String>,
    pub exit_script: Option<String>,
    pub dest_connor_loc: [f64; 3],
    pub dest_connor_dir: [f64; 3],
    pub dest_cam_loc: [f64; 3],
    pub dest_cam_dir: [f64; 3],
    /// `noLoc`; (−1, −1, −1) = none (rule 40 step 2).
    pub no_loc: [f64; 3],
}

impl PortalKeys {
    /// `None` when the section is not a `KQPortal`.
    pub fn from_section(s: &Section) -> Option<Self> {
        if !s.get("classType")?.trim().eq_ignore_ascii_case("KQPortal") {
            return None;
        }
        let three = |k: &str, default: f64| ["X", "Y", "Z"].map(|a| s.number(&format!("{k}{a}")).and_then(Result::ok).unwrap_or(default));
        Some(PortalKeys {
            name: s.name.clone(),
            world: text(s, "world"),
            enter_anim: text(s, "enterAnim"),
            exit_anim: text(s, "exitAnim"),
            enter_anm_file: text(s, "enterAnmFile"),
            exit_anm_file: text(s, "exitAnmFile"),
            exit_script: text(s, "exitScript"),
            dest_connor_loc: three("destConnorLoc", 0.0),
            dest_connor_dir: three("destConnorDir", 0.0),
            dest_cam_loc: three("destCamLoc", 0.0),
            dest_cam_dir: three("destCamDir", 0.0),
            no_loc: three("noLoc", -1.0),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kq_text::parse;

    const DAVMAP: &[u8] = b"[davMap]\r\nclassType=KQMap\r\nnorth=344068\r\nsouth=155670\r\neast=303114\r\nwest=114699\r\nuseFog=1\r\nfogRadius=10000\r\nisActive=0\r\nblankMap=davmap0.pbm\r\nfullMap=davmap.pbm\r\nmapName=#38\r\narrowPBA=maparrow.pba\r\nrle0=-1\r\ncMapIcons=0\r\nTeleportScript=Davtele.cs\r\nTeleportPortalKQ=Davtele.kq\r\n";

    #[test]
    fn a_shipped_map_file() {
        let m = MapKeys::from_section(&parse(DAVMAP)[0]).unwrap().unwrap();
        assert_eq!((m.north, m.south, m.east, m.west), (344068.0, 155670.0, 303114.0, 114699.0));
        assert_eq!((m.use_fog, m.fog_radius, m.is_active, m.earned_teleport), (true, Some(10000.0), false, false));
        assert_eq!((m.map_name, m.index()), (Some(38), Some(0)));
        assert_eq!(m.teleport_portal.as_deref(), Some("Davtele.kq"));
        assert!(m.runs.is_empty() && m.icons.is_empty());
        assert!(MapKeys::from_section(&parse(b"[x]\nclassType=KQPortal\n")[0]).is_none());
    }

    #[test]
    fn icons_and_runs_are_read() {
        let file = b"[davMap]\nclassType=KQMap\nnorth=1\nsouth=0\neast=1\nwest=0\nblankMap=a\nfullMap=b\ncMapIcons=1\nmapIcon0=davmap.pbm\nsrcIcon0=dv_telep.bmp\nxIcon0=250\nyIcon0=98\nrle0=5\nrle1=2\nrle2=-1\nrle3=9\n";
        let m = MapKeys::from_section(&parse(file)[0]).unwrap().unwrap();
        assert_eq!(m.icons, vec![MapIcon { picture: "davmap.pbm".into(), file: "dv_telep.bmp".into(), x: 250, y: 98 }]);
        assert_eq!(m.runs, vec![5, 2], "the -1 ends the list");
        // Written back and read again: the same keys.
        let again = MapKeys::from_section(&parse(m.to_text().as_bytes())[0]).unwrap().unwrap();
        assert_eq!(again, MapKeys { arrow_pba: "maparrow.pba".into(), ..m });
    }

    #[test]
    fn runs_cover_464_places_per_row() {
        // The note's example start: 40 rows of 464 plus 208 in the fog,
        // then 15 out: row 40, columns 208-222.
        let (fog, check) = fog_from_runs(&[18768, 15, 446]);
        assert_eq!(revealed_count(&fog), 15);
        assert!(fog[40 * MAP_SIZE + 208] && fog[40 * MAP_SIZE + 222] && !fog[40 * MAP_SIZE + 223] && !fog[40 * MAP_SIZE + 207]);
        assert_eq!(check, RunCheck { places: 18768 + 15 + 446, unused_revealed: 0, beyond: 0 });
        // A run over the row's end: its 4 unused places are skipped.
        let (fog, check) = fog_from_runs(&[458, 10]);
        assert_eq!((revealed_count(&fog), check.unused_revealed), (6, 4));
        assert!(fog[MAP_SIZE] && fog[MAP_SIZE + 3] && !fog[MAP_SIZE + 4]);
    }

    #[test]
    fn runs_written_back_are_the_same() {
        let runs = vec![18768, 15, 446, 21, MAP_SIZE as i64 * ROW_PLACES as i64 - 18768 - 15 - 446 - 21];
        let (fog, _) = fog_from_runs(&runs);
        assert_eq!(runs_from_fog(&fog), runs);
        assert_eq!(runs_from_fog(&vec![false; MAP_SIZE * MAP_SIZE]), Vec::<i64>::new());
        // The last pixel revealed: the run ends with the 4 unused places.
        let mut fog = vec![false; MAP_SIZE * MAP_SIZE];
        fog[MAP_SIZE * MAP_SIZE - 1] = true;
        let runs = runs_from_fog(&fog);
        assert_eq!(runs, vec![(MAP_SIZE * ROW_PLACES - 5) as i64, 1, 4]);
        assert_eq!(fog_from_runs(&runs).0, fog);
    }

    #[test]
    fn a_portal_file() {
        let file = b"[Teleport]\nclassType=KQPortal\nworld=Daventry.kq\nenterAnim=teleportdown\nexitAnim=teleportup\nnoLocX=-1\nnoLocY=-1\nnoLocZ=-1\ndestConnorLocX=217338\ndestConnorLocY=283698\ndestConnorLocZ=18930\ndestCamDirX=-0.2\n";
        let p = PortalKeys::from_section(&parse(file)[0]).unwrap();
        assert_eq!(p.world.as_deref(), Some("Daventry.kq"));
        assert_eq!((p.enter_anim.as_deref(), p.exit_anim.as_deref()), (Some("teleportdown"), Some("teleportup")));
        assert_eq!(p.dest_connor_loc, [217338.0, 283698.0, 18930.0]);
        assert_eq!((p.dest_cam_dir, p.no_loc), ([-0.2, 0.0, 0.0], [-1.0, -1.0, -1.0]));
    }
}
