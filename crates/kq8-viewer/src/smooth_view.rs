//! Smooth animation (`docs/DESIGN.md` "smooth animation", steps S-1 to
//! S-6): ours, not from the original game. Off by default. Drawing only:
//! the rules, cues and logs keep the original 15 updates a second and the
//! original frames; with the switch on, model parts are drawn part-way
//! between two frames and moving things between two updates.
//!
//! The switch is one flag for the whole program: `--smooth` at the start
//! turns it on, the console word `smooth [on|off]` while playing.

use std::sync::atomic::{AtomicBool, Ordering};

static ON: AtomicBool = AtomicBool::new(false);

/// Whether smooth animation is on.
pub fn on() -> bool {
    ON.load(Ordering::Relaxed)
}

/// Turns it on or off; returns a line for the console and the log.
pub fn set(on: bool) -> String {
    ON.store(on, Ordering::Relaxed);
    format!("smooth animation is {} (ours: drawing only; the game keeps its 15 updates a second)", if on { "on" } else { "off" })
}

/// Seconds since the last game update (the part of an update gone by).
pub fn ahead(fixed: &bevy::prelude::Time<bevy::prelude::Fixed>) -> f32 {
    fixed.overstep_fraction() / kq8_core::UPDATES_PER_SECOND as f32
}

/// How far toward the next frame, as the tag's amount (0-255).
pub fn amount(toward: f32) -> u8 {
    (toward.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// The frame to draw for `player` and the amount toward the next (S-4):
/// with the switch off, its frame as the game shows it and 0.
pub fn frame_of(player: &kq8_core::named_anim::NamedAnimPlayer, ahead: f32) -> (usize, u8) {
    if on() {
        let (frame, toward) = player.drawn_frame(ahead);
        (frame, amount(toward))
    } else {
        (player.frame(), 0)
    }
}

/// [`frame_of`] for Connor's walker and his rope model: his phase already
/// pushed on by the part of an update gone by.
pub fn frame_at(pushed: f32, frames: usize, repeat: bool, shown: usize) -> (usize, u8) {
    if on() {
        let (frame, toward) = kq8_core::named_anim::mixed_frame(pushed, frames, repeat);
        (frame, amount(toward))
    } else {
        (shown, 0)
    }
}

/// Parts given a tag part-way toward the next frame since the last
/// report (for the log: `frame_stats`).
static MIXED: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);

/// `parts` tags were set part-way (amount above 0).
pub fn count_mixed(parts: usize) {
    MIXED.fetch_add(parts as u64, Ordering::Relaxed);
}

/// The 2-second report's words: the switch and the parts set part-way
/// since the last report.
pub fn report() -> String {
    format!("smooth {} ({} part tags set part-way)", if on() { "on" } else { "off" }, MIXED.swap(0, Ordering::Relaxed))
}
