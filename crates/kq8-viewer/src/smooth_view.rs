//! Smooth animation (`docs/DESIGN.md` "smooth animation", steps S-1 to
//! S-6): ours, not from the original game. Off by default. Drawing only:
//! the rules, cues and logs keep the original 15 updates a second and the
//! original frames; with the switch on, model parts are drawn part-way
//! between two frames and moving things between two updates.
//!
//! The switch is one flag for the whole program: `--smooth on|off` at the
//! start, the console word `smooth [on|off]` while playing.

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
