//! The loading screen while the next world is read (M6 6e-4a).
//!
//! Rules from `docs/behaviour/world-change.md` (A, 2026-10-21; the user's
//! check in the original 2026-10-22):
//!
//! - Rule 22: the picture is `GAME/8Gui/<world>.pbm` (`daventry.pbm`,
//!   `castleD.pbm`, …) and the palette `GAME/8Gui/L<first 7 letters of
//!   the world>.ppl` (`ldaventr.ppl`, `lcastleD.ppl`, …), not the world's.
//!   "Do not reproduce" 1: the original copies them over `loading.pbm` /
//!   `loading.ppl` in the install; we read them by name and never write.
//! - Rule 23 (`GAME/Resource/Loading.gui`, 640 × 480): the picture full
//!   screen, a black bar (0, 462)–(640, 480) (`4black.pbm`) and the meter
//!   at (5, 468), 630 × 8. In the loading phase the meter is **red**
//!   (`4red.pbm`). The copy-from-CD phase (the overview `<5 letters>2.pbm`
//!   with a green meter) is left out ("Do not reproduce" 2). The text line
//!   (0, 415)–(640, 445) is not drawn (the note does not say what it
//!   shows).
//! - Rule 24: the meter fills from left to right with the scripts'
//!   `setLoadProgress` percent.
//! - Rule 25: keys and clicks do nothing meanwhile (the game's systems
//!   don't run, `world_build::playing`).
//!
//! Ours: the 640 × 480 layout is stretched over the whole window.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use kq8_data::GameDir;

use crate::log::Log;
use crate::world_build::{Persistent, WorldChange};

/// Everything the loading screen spawned.
#[derive(Component)]
pub struct LoadingScreen;

/// The meter's node.
#[derive(Component)]
pub struct LoadingMeter;

/// Layout in 640 × 480 pixels (rule 23).
const BAR: [f32; 4] = [0.0, 462.0, 640.0, 18.0];
const METER: [f32; 4] = [5.0, 468.0, 630.0, 8.0];

/// The picture's and the palette's file names for `world` (rule 22).
pub fn files(world: &str) -> (String, String) {
    let short: String = world.chars().take(7).collect();
    (format!("GAME/8Gui/{world}.pbm"), format!("GAME/8Gui/l{short}.ppl"))
}

/// A picture as an image, in `palette`'s colours.
fn picture(game: &GameDir, file: &str, palette: &[kq8_data::palette::Rgb; 256]) -> Result<Image, String> {
    let path = game.find(file).ok_or("not found")?;
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let bmp = kq8_data::Bitmap::parse(&bytes)?;
    if bmp.bit_count != 8 {
        return Err(format!("{} bits, 8 expected", bmp.bit_count));
    }
    let rgba = bmp.pixels.iter().flat_map(|&i| {
        let c = palette[i as usize];
        [c[0], c[1], c[2], 255]
    }).collect();
    Ok(crate::object_light::texture_image(bmp.width, bmp.height, rgba, bevy::image::ImageAddressMode::ClampToEdge))
}

/// Opens the loading screen for `world` (at the take-down, rule 7.2).
pub fn open(world: &mut World, to: &str, log: &Arc<Mutex<Log>>) {
    let Some(game) = world.resource::<WorldChange>().game().and_then(|g| GameDir::open(g).ok()) else { return };
    let (pic_file, pal_file) = files(to);
    let palette = game
        .find(&pal_file)
        .ok_or("not found".to_string())
        .and_then(|p| std::fs::read(p).map_err(|e| e.to_string()))
        .and_then(|b| kq8_data::Palette::parse(&b))
        .map(|p| kq8_data::palette::as_shown(p.colors));
    let mut notes = Vec::new();
    let mut images = Vec::new();
    match palette {
        Err(e) => notes.push(format!("palette {pal_file}: {e}; no pictures")),
        Ok(palette) => {
            for file in [pic_file.clone(), "GAME/8Gui/4black.pbm".into(), "GAME/8Gui/4red.pbm".into()] {
                match picture(&game, &file, &palette) {
                    Ok(image) => images.push(Some(image)),
                    Err(e) => {
                        notes.push(format!("{file}: {e}"));
                        images.push(None);
                    }
                }
            }
        }
    }
    let mut assets = world.resource_mut::<Assets<Image>>();
    let handles: Vec<Option<Handle<Image>>> = images.into_iter().map(|i| i.map(|i| assets.add(i))).collect();
    let at = |r: [f32; 4]| Node {
        position_type: PositionType::Absolute,
        left: Val::Percent(r[0] / 6.4),
        top: Val::Percent(r[1] / 4.8),
        width: Val::Percent(r[2] / 6.4),
        height: Val::Percent(r[3] / 4.8),
        ..default()
    };
    // The world's camera is gone: the panels need one of their own.
    world.spawn((Camera2d, Persistent, LoadingScreen));
    let mut root = world.spawn((Node { width: Val::Percent(100.0), height: Val::Percent(100.0), ..default() }, BackgroundColor(Color::BLACK), Persistent, LoadingScreen));
    root.with_children(|p| {
        if let Some(Some(h)) = handles.first() {
            p.spawn((ImageNode::new(h.clone()), at([0.0, 0.0, 640.0, 480.0]), Persistent, LoadingScreen));
        }
        match handles.get(1) {
            Some(Some(h)) => p.spawn((ImageNode::new(h.clone()), at(BAR), Persistent, LoadingScreen)),
            _ => p.spawn((BackgroundColor(Color::BLACK), at(BAR), Persistent, LoadingScreen)),
        };
        let mut meter = at(METER);
        meter.width = Val::Percent(0.0);
        match handles.get(2) {
            Some(Some(h)) => p.spawn((ImageNode::new(h.clone()), meter, Persistent, LoadingScreen, LoadingMeter)),
            _ => p.spawn((BackgroundColor(Color::srgb(0.8, 0.0, 0.0)), meter, Persistent, LoadingScreen, LoadingMeter)),
        };
    });
    let mut log = log.lock().unwrap();
    log.summary(format!("loading screen: {pic_file} in {pal_file}, the black bar and the red meter (world-change.md rules 22-24){}", if notes.is_empty() { String::new() } else { format!("; {}", notes.join("; ")) }));
}

/// The meter at `percent` of its 630 pixels (rule 24).
pub fn set_meter(world: &mut World, percent: u32) {
    let mut q = world.query_filtered::<&mut Node, With<LoadingMeter>>();
    for mut node in q.iter_mut(world) {
        let want = Val::Percent(METER[2] / 6.4 * percent.min(100) as f32 / 100.0);
        if node.width != want {
            node.width = want;
        }
    }
}

/// Closes the loading screen (the world is built).
pub fn close(world: &mut World) {
    let all: Vec<Entity> = world.query_filtered::<Entity, With<LoadingScreen>>().iter(world).collect();
    for e in all {
        let _ = world.try_despawn(e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn file_names_follow_rule_22() {
        assert_eq!(files("Daventry"), ("GAME/8Gui/Daventry.pbm".into(), "GAME/8Gui/lDaventr.ppl".into()));
        assert_eq!(files("CastleD").1, "GAME/8Gui/lCastleD.ppl");
        assert_eq!(files("IceWorld").1, "GAME/8Gui/lIceWorl.ppl");
        assert_eq!(files("temple1").1, "GAME/8Gui/ltemple1.ppl");
    }
}
