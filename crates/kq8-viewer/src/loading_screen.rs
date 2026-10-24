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
//! Ours: the 640 × 480 layout is drawn in the largest 4:3 box that fits
//! the window, centred, black around it (the user, 2026-10-22: not
//! stretched to 16:9).
//! `4red.pbm` is a 6 × 6 picture (rows of entries 222–224), stretched over
//! the meter's place (the original's stretch or tiling is not traced; the
//! two look alike at 8 pixels high).
//!
//! **Not drawn: the full-width black bar** `4black.pbm` at (0, 462)–(640,
//! 480). Rule 23 (Q152, the user's check in the original): the picture's
//! painted frame stays; the meter's filled part is its picture stretched
//! to (630 × percent / 100, whole pixels, at least 1 if not 0) × 8 and the
//! unfilled rest of the slot is `4black.pbm`, stretched.

use std::sync::{Arc, Mutex};

use bevy::prelude::*;
use kq8_data::GameDir;

use crate::log::Log;
use crate::world_build::{Persistent, WorldChange};

/// Everything the loading screen spawned.
#[derive(Component)]
pub struct LoadingScreen;

/// The 4:3 box that holds the layout.
#[derive(Component)]
pub struct LoadingBox;

/// The meter's node.
#[derive(Component)]
pub struct LoadingMeter;

/// The black over the meter slot's unfilled part (rule 23, Q152).
#[derive(Component)]
pub struct LoadingBlack;

/// Rule 23 (Q152): the filled part of the 630-pixel meter at `percent`:
/// 630 × percent / 100 cut to whole pixels, at least 1 if not 0.
pub fn filled_pixels(percent: u32) -> u32 {
    let p = percent.min(100);
    let w = (METER[2] as u32 * p) / 100;
    if p > 0 { w.max(1) } else { 0 }
}

/// Layout in 640 × 480 pixels (rule 23).
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
    let (mut image, mut red, mut black) = (None, None, None);
    match palette {
        Err(e) => notes.push(format!("palette {pal_file}: {e}; no pictures")),
        Ok(palette) => {
            match picture(&game, &pic_file, &palette) {
                Ok(i) => image = Some(i),
                Err(e) => notes.push(format!("{pic_file}: {e}")),
            }
            match picture(&game, "GAME/8Gui/4red.pbm", &palette) {
                Ok(i) => {
                    notes.push(format!("meter 4red.pbm {}x{} stretched to 630x8", i.width(), i.height()));
                    red = Some(i);
                }
                Err(e) => notes.push(format!("4red.pbm: {e}; a plain red meter")),
            }
            match picture(&game, "GAME/8Gui/4black.pbm", &palette) {
                Ok(i) => black = Some(i),
                Err(e) => notes.push(format!("4black.pbm: {e}; plain black")),
            }
        }
    }
    let mut assets = world.resource_mut::<Assets<Image>>();
    let (handle, red, black) = (image.map(|i| assets.add(i)), red.map(|i| assets.add(i)), black.map(|i| assets.add(i)));
    notes.push("black only over the slot's unfilled part, no full-width bar (Q152)".into());
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
    // The whole window black; in its middle a 4:3 box that holds the
    // 640 x 480 layout.
    let root = world
        .spawn((
            Node { width: Val::Percent(100.0), height: Val::Percent(100.0), justify_content: JustifyContent::Center, align_items: AlignItems::Center, ..default() },
            BackgroundColor(Color::BLACK),
            Persistent,
            LoadingScreen,
        ))
        .id();
    let four_three = Node { height: Val::Percent(100.0), max_width: Val::Percent(100.0), aspect_ratio: Some(4.0 / 3.0), ..default() };
    let mut screen = world.spawn((four_three, Persistent, LoadingScreen, LoadingBox, ChildOf(root)));
    screen.with_children(|p| {
        // `Stretch`: the picture fills its place whatever its own size
        // (the default keeps the picture's size: the meter showed as a
        // small square, the user's first look).
        let stretched = |h: &Handle<Image>| ImageNode { image: h.clone(), image_mode: NodeImageMode::Stretch, ..default() };
        if let Some(h) = &handle {
            p.spawn((stretched(h), at([0.0, 0.0, 640.0, 480.0]), Persistent, LoadingScreen));
        }
        let mut meter = at(METER);
        meter.width = Val::Percent(0.0);
        match &red {
            Some(h) => p.spawn((stretched(h), meter, Persistent, LoadingScreen, LoadingMeter)),
            None => p.spawn((BackgroundColor(Color::srgb(0.8, 0.0, 0.0)), meter, Persistent, LoadingScreen, LoadingMeter)),
        };
        // At 0 % the whole slot is black.
        let rest = at(METER);
        match &black {
            Some(h) => p.spawn((stretched(h), rest, Persistent, LoadingScreen, LoadingBlack)),
            None => p.spawn((BackgroundColor(Color::BLACK), rest, Persistent, LoadingScreen, LoadingBlack)),
        };
    });
    let mut log = log.lock().unwrap();
    log.summary(format!("loading screen: {pic_file} in {pal_file}, the red meter (world-change.md rules 22-24); {}", notes.join("; ")));
}

/// The meter at `percent` of its 630 pixels (rule 24), the black over the
/// rest of the slot (rule 23, Q152).
pub fn set_meter(world: &mut World, percent: u32) {
    let filled = filled_pixels(percent) as f32;
    let mut q = world.query_filtered::<&mut Node, With<LoadingMeter>>();
    for mut node in q.iter_mut(world) {
        let want = Val::Percent(filled / 6.4);
        if node.width != want {
            node.width = want;
        }
    }
    let mut q = world.query_filtered::<&mut Node, With<LoadingBlack>>();
    for mut node in q.iter_mut(world) {
        let (left, width) = (Val::Percent((METER[0] + filled) / 6.4), Val::Percent((METER[2] - filled) / 6.4));
        if node.left != left || node.width != width {
            node.left = left;
            node.width = width;
        }
    }
}

/// Closes the loading screen (the world is built); returns the 4:3 box's
/// and the window's sizes in pixels, for the log.
pub fn close(world: &mut World) -> String {
    let boxed = world.query_filtered::<&bevy::ui::ComputedNode, With<LoadingBox>>().iter(world).next().map(|c| c.size);
    let window = world.query::<&Window>().iter(world).next().map(|w| (w.physical_width(), w.physical_height()));
    let sizes = format!("layout box {boxed:?} in window {window:?}");
    let all: Vec<Entity> = world.query_filtered::<Entity, With<LoadingScreen>>().iter(world).collect();
    for e in all {
        let _ = world.try_despawn(e);
    }
    sizes
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_filled_part_is_cut_to_whole_pixels_at_least_1() {
        // world-change.md rule 23 (Q152).
        assert_eq!((filled_pixels(0), filled_pixels(1), filled_pixels(50), filled_pixels(100), filled_pixels(150)), (0, 6, 315, 630, 630));
        assert_eq!(filled_pixels(99), 623, "623.7 cut down");
    }

    #[test]
    fn file_names_follow_rule_22() {
        assert_eq!(files("Daventry"), ("GAME/8Gui/Daventry.pbm".into(), "GAME/8Gui/lDaventr.ppl".into()));
        assert_eq!(files("CastleD").1, "GAME/8Gui/lCastleD.ppl");
        assert_eq!(files("IceWorld").1, "GAME/8Gui/lIceWorl.ppl");
        assert_eq!(files("temple1").1, "GAME/8Gui/ltemple1.ppl");
    }
}
