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
pub fn frame_at(pushed: f32, movement: &kq8_core::connor::MovementLoop) -> (usize, u8) {
    if on() {
        let (frame, toward) = movement.drawn_frame(pushed);
        (frame, amount(toward))
    } else {
        (movement.frame_at(pushed), 0)
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

/// A move between two updates longer than this (game units) is drawn at
/// once, not glided: a script's `move`, a world change (S-5). The fastest
/// things glided go about 550 a update (a bolt at 8,000 a second).
pub const GLIDE_JUMP: f32 = 2000.0;

/// Smooth animation (S-5): something drawn between where it was at the
/// last update but one and where it is now (as the camera and Connor
/// already are), `a` of the way (the part of an update gone by).
#[derive(Debug, Clone, Copy, Default)]
pub struct Glide {
    before: Option<kq8_core::collide::Placement>,
    now: Option<kq8_core::collide::Placement>,
    update: u64,
}

impl Glide {
    /// Where to draw it: `place` is where it is at game update `update`.
    /// With the switch off, `place` itself.
    pub fn drawn(&mut self, update: u64, place: kq8_core::collide::Placement, a: f32) -> kq8_core::collide::Placement {
        if update != self.update || self.now.is_none() {
            self.before = self.now.or(Some(place));
            self.update = update;
        }
        self.now = Some(place);
        let before = self.before.unwrap_or(place);
        let d: f32 = (0..3).map(|k| (place.position[k] - before.position[k]).powi(2)).sum::<f32>().sqrt();
        if !on() || d > GLIDE_JUMP {
            return place;
        }
        let t = |p: &kq8_core::collide::Placement| kq8_data::shape::Transform { flags: 0, m: p.rotation, p: p.position };
        let m = kq8_core::door::mix_transform(&t(&before), &t(&place), a);
        kq8_core::collide::Placement { rotation: m.m, position: m.p }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_core::collide::Placement;

    #[test]
    fn a_glide_goes_from_the_last_update_to_this_one_and_snaps_on_a_jump() {
        set(true);
        let mut g = Glide::default();
        assert_eq!(g.drawn(1, Placement::at([0.0; 3]), 0.5).position, [0.0; 3], "first seen: where it is");
        assert_eq!(g.drawn(2, Placement::at([100.0, 0.0, 0.0]), 0.25).position, [25.0, 0.0, 0.0]);
        assert_eq!(g.drawn(2, Placement::at([100.0, 0.0, 0.0]), 0.75).position, [75.0, 0.0, 0.0], "same update, later");
        assert_eq!(g.drawn(3, Placement::at([5000.0, 0.0, 0.0]), 0.5).position, [5000.0, 0.0, 0.0], "a jump: at once");
        set(false);
        assert_eq!(g.drawn(4, Placement::at([5100.0, 0.0, 0.0]), 0.5).position, [5100.0, 0.0, 0.0], "off: where it is");
    }
}
