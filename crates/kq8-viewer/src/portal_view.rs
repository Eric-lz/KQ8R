//! Arriving through a portal (M6 6e-4b): Connor and the camera at the
//! portal's places, the arrival movement, then the portal's exit script.
//!
//! Rules from `docs/behaviour/world-change.md` (A, 2026-10-21; the user's
//! check in the original 2026-10-22):
//!
//! - Rule 6: the portal hands over `destConnorLoc` + `destConnorDir` (turn
//!   0 0 0 when none) as `KQPortal::ConLoc`, `destCamLoc` + `destCamDir`
//!   as `KQPortal::CamLoc`, and its `world`.
//! - Rule 20.1: Connor's first load in the new world is placed at
//!   `ConLoc`; a z of −1 gets the ground rule; the camera at `CamLoc`.
//! - Rule 20.2–20.3: he arrives in the Portal state, `handsOff` still on
//!   (rule 4), and plays the `exitAnim`, from the `exitAnmFile` list when
//!   the portal names one (`conner.anm` → `conner2.anm` with the full
//!   armour; rule 5.2).
//! - Rule 20.4: when it ends: `stop`, first person allowed again, the
//!   `exitScript` with `$1` = the portal's name and `$2` = `Exit`, the
//!   time step back to 1 s, `handsOff 0`, Wait. The map stays closed
//!   (rule 20.5).
//!
//! Ours: the places are applied as `move Connor` / `move KQCamera` script
//! commands before the first update (the original sets them on the load
//! itself; nothing runs in between). A portal whose list is Connor's own
//! (`conner.anm`) plays the movement from his loaded list; another list
//! (`Elevator.anm`, …) is named to the Special state, which plays it only
//! if this world loaded that list (the portals' own lists are loaded with
//! 6e-6 / 6e-7). We have no first person and no time-step limit: those
//! two are logged only.

use bevy::prelude::*;
use kq8_data::magic_map::PortalKeys;
use kq8_data::GameDir;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::Shared;

/// What a portal gives the world Connor arrives in (rules 6, 20).
#[derive(Debug, Clone)]
pub struct PortalArrival {
    pub portal: String,
    /// `KQPortal::ConLoc`: x, y, z, tilt x, tilt y, heading.
    pub con_loc: [f32; 6],
    /// `KQPortal::CamLoc`.
    pub cam_loc: [f32; 6],
    pub exit_anim: Option<String>,
    pub exit_list: Option<String>,
    pub exit_script: Option<String>,
}

impl PortalArrival {
    pub fn from_keys(p: &PortalKeys) -> PortalArrival {
        let six = |l: [f64; 3], d: [f64; 3]| [l[0], l[1], l[2], d[0], d[1], d[2]].map(|v| v as f32);
        PortalArrival {
            portal: p.name.clone(),
            con_loc: six(p.dest_connor_loc, p.dest_connor_dir),
            cam_loc: six(p.dest_cam_loc, p.dest_cam_dir),
            exit_anim: p.exit_anim.clone(),
            exit_list: p.exit_anm_file.clone(),
            exit_script: p.exit_script.clone(),
        }
    }

    /// Connor on the ground (z −1, `objects.md` ground rule): x, y,
    /// heading for the viewer's start placement.
    pub fn on_ground(&self) -> Option<[f32; 3]> {
        (self.con_loc[2] == -1.0).then_some([self.con_loc[0], self.con_loc[1], self.con_loc[5]])
    }
}

/// Reads portal file `file` (`GAME/Kq` first, as the teleport portals
/// are, then the world `world`'s archives). Returns its keys and the
/// world it leads to (our world name, from its `world` file name).
pub fn read_portal(game: &GameDir, world: &str, file: &str) -> Result<(PortalKeys, String), String> {
    let bytes = match game.find(&format!("GAME/Kq/{file}")) {
        Some(p) => std::fs::read(p).map_err(|e| e.to_string())?,
        None => {
            let w = crate::load::World::open(game, world)?;
            w.load(file).ok_or(format!("{file}: not in GAME/Kq or {world}'s archives"))??.1
        }
    };
    let sections = kq8_data::kq_text::parse(&bytes);
    let keys = sections.iter().find_map(PortalKeys::from_section).ok_or(format!("{file}: no KQPortal section"))?;
    let target = keys.world.clone().ok_or(format!("{file}: no world"))?;
    let stem = target.rsplit(['/', '\\']).next().unwrap_or(&target);
    let stem = stem.strip_suffix(".kq").or_else(|| stem.strip_suffix(".KQ")).unwrap_or(stem);
    let to = kq8_data::game_dir::WORLDS.iter().find(|w| w.eq_ignore_ascii_case(stem)).ok_or(format!("{file}: world {target} is not one we know"))?;
    Ok((keys, to.to_string()))
}

/// The arrival under way in the world just built.
#[derive(Resource)]
pub struct ArrivalNow {
    pub arrival: PortalArrival,
    /// 0: not started; 1: the arrival movement plays.
    step: u8,
}

impl ArrivalNow {
    pub fn new(arrival: PortalArrival) -> Self {
        ArrivalNow { arrival, step: 0 }
    }
}

/// Every update before the doors' (6e-4b): the first one places Connor
/// and the camera and starts the arrival movement; once it is over, the
/// exit script and `handsOff 0`.
pub fn arrival_step(mut commands: Commands, now: Option<ResMut<ArrivalNow>>, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>, shared: Res<Shared>) {
    let (Some(mut now), Some(mut sim), Some(connor)) = (now, sim, connor) else { return };
    let a = now.arrival.clone();
    let updates = sim.active.world.updates;
    let mut lines = Vec::new();
    let run = |sim: &mut DoorSim, lines: &mut Vec<String>, line: String| {
        let r = sim.active.test_command(&line, "portal arrival");
        lines.push(format!("update {updates}: {line}: {r:?}"));
    };
    match now.step {
        0 => {
            // Rule 4: handsOff stays on through the change (ours: the new
            // world starts without it, so it is set again).
            run(&mut sim, &mut lines, "handsOff 1".into());
            let [x, y, z, _, _, h] = a.con_loc;
            if a.on_ground().is_none() {
                run(&mut sim, &mut lines, format!("move Connor 0 {x} {y} {z} 0 0 {h}"));
            }
            let [cx, cy, cz, tx, ty, ch] = a.cam_loc;
            run(&mut sim, &mut lines, format!("move KQCamera 0 {cx} {cy} {cz} {tx} {ty} {ch}"));
            match &a.exit_anim {
                Some(anim) => {
                    let own = a.exit_list.as_deref().is_none_or(|l| l.eq_ignore_ascii_case("conner.anm") || l.eq_ignore_ascii_case("conner2.anm"));
                    let items = if own { anim.clone() } else { format!("AnimList={},AnimListOn,{anim}", a.exit_list.clone().unwrap_or_default()) };
                    run(&mut sim, &mut lines, format!("KQMonster::setState Connor special {items}"));
                }
                None => lines.push(format!("update {updates}: {}: no exitAnim", a.portal)),
            }
            now.step = 1;
        }
        _ => {
            if connor.walker.mode == kq8_core::connor::Mode::Special {
                return;
            }
            // Rule 20.4: the movement is over.
            lines.push(format!("update {updates}: {}: the arrival movement ended; first person allowed again and the time step back to 1 s (logged only: we have neither)", a.portal));
            match &a.exit_script {
                Some(script) => {
                    let ran = sim.active.run_script(script, &[a.portal.clone(), "Exit".into()]);
                    lines.push(format!("update {updates}: exitScript {script} {} Exit: {}", a.portal, if ran { "run" } else { "not in this world" }));
                }
                None => lines.push(format!("update {updates}: {}: no exitScript", a.portal)),
            }
            run(&mut sim, &mut lines, "handsOff 0".into());
            commands.remove_resource::<ArrivalNow>();
        }
    }
    let mut log = shared.log.lock().unwrap();
    for l in lines {
        log.summary(format!("portal arrival: {l}"));
    }
}
