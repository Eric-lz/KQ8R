//! Animated objects playing their current movement (M3 step 3c). Rules
//! from `docs/behaviour/objects.md` "Animated objects at load" rules 2–4
//! (traced): the `loop=` line's start position, speed and repeat flag, and
//! `flags` bit 8 (random start and speed). The phase moves as the doors'
//! do (`doors.md` rules 5, 8): speed × (1/15 s) ÷ the loop's stored length
//! per update.
//!
//! Not done: cues, `nextAnimName`, `animPos`, scripts changing the
//! movement.

use crate::door::{chain, identity};
use crate::emitter::Rng;
use kq8_data::shape::Transform;

/// `flags` bit for a random start frame and a random speed 0.9–1.1
/// (objects.md rule 4).
pub const RANDOM_START: u32 = 8;

/// One object's running loop.
#[derive(Debug, Clone)]
pub struct LoopPlayer {
    /// 0..1 through the loop.
    pub phase: f32,
    /// 1 forward, −1 backward, 0 still (times the random factor).
    pub speed: f32,
    pub repeat: bool,
    /// Frames in the loop (at least 1).
    pub frames: usize,
    /// The loop's stored length in seconds.
    pub play_time: f32,
    /// A one-shot that reached its end.
    pub finished: bool,
}

impl LoopPlayer {
    /// A loop starting at `start` (0..1) with `speed`; with `random`
    /// (`flags` bit 8) the start is a random phase and the speed is
    /// multiplied by a random 0.9–1.1.
    pub fn new(start: f32, speed: f32, repeat: bool, frames: usize, play_time: f32, random: Option<&mut Rng>) -> Self {
        let (phase, speed) = match random {
            Some(rng) => (rng.fraction(), speed * rng.between(0.9, 1.1)),
            None => (start, speed),
        };
        let phase = if repeat { phase.rem_euclid(1.0) } else { phase.clamp(0.0, 1.0) };
        LoopPlayer { phase, speed, repeat, frames: frames.max(1), play_time, finished: false }
    }

    /// Plays: speed × 1/15 s ÷ the loop's length per update.
    pub fn plays(&self) -> bool {
        self.speed != 0.0 && self.play_time > 0.0 && !self.finished
    }

    /// One update of `dt` seconds.
    pub fn advance(&mut self, dt: f32) {
        if !self.plays() {
            return;
        }
        let p = self.phase + self.speed * dt / self.play_time;
        if self.repeat {
            self.phase = p.rem_euclid(1.0);
        } else if p >= 1.0 || p <= 0.0 {
            // Played once: stays on its last (or, backwards, first) frame.
            self.phase = p.clamp(0.0, 1.0);
            self.finished = true;
        } else {
            self.phase = p;
        }
    }

    /// The frame shown: floor(phase × frames + 0.001)
    /// ([`kq8_data::shape::frame_at_phase`], Q153).
    pub fn frame(&self) -> usize {
        kq8_data::shape::frame_at_phase(self.phase, self.frames, self.repeat)
    }
}

/// The whole-object pose at `frame`: the steps of frames 1…`frame`
/// chained, as for doors (`door::DoorMovement::turn_at_frame`), never
/// added up across cycles.
pub fn pose_at_frame(steps: &[Transform], frame: usize) -> Transform {
    let mut c = identity();
    for s in steps.iter().take(frame + 1).skip(1) {
        c = chain(&c, s);
    }
    c
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_repeating_loop_wraps_once_per_stored_length() {
        // 16 frames, stored length 15/15 = 1.0 s: 15 updates per cycle, so
        // each update moves 16/15 of a frame and one frame per cycle is
        // never shown (true for any rounding).
        let mut p = LoopPlayer::new(0.0, 1.0, true, 16, 1.0, None);
        let mut shown = [0u32; 16];
        for _ in 0..15 * 4 {
            p.advance(1.0 / 15.0);
            shown[p.frame()] += 1;
        }
        // Back at the start after whole cycles.
        assert!(p.phase.min(1.0 - p.phase) < 1e-4, "phase {}", p.phase);
        assert_eq!(shown.iter().filter(|&&n| n == 4).count(), 15, "{shown:?}");
        assert_eq!(shown.iter().filter(|&&n| n == 0).count(), 1, "{shown:?}");
    }

    #[test]
    fn the_start_frame_is_the_still_models() {
        // telepad1: start 0.3 of 16 frames -> frame 4 (cut down, Q153).
        assert_eq!(LoopPlayer::new(0.3, 1.0, true, 16, 1.0, None).frame(), 4);
        // The King's portrait's `stop2`: 0.99 of 46, still -> frame 45, the
        // pushed pose (not 0, which round() gave).
        assert_eq!(LoopPlayer::new(0.99, 0.0, true, 46, 1.0, None).frame(), 45);
        // A one-shot at 1.0 shows its last frame.
        assert_eq!(LoopPlayer::new(1.0, 0.0, false, 16, 1.0, None).frame(), 15);
    }

    #[test]
    fn a_one_shot_stops_at_its_end_and_backwards_runs_down() {
        let mut p = LoopPlayer::new(0.0, 1.0, false, 8, 0.4667, None);
        for _ in 0..20 {
            p.advance(1.0 / 15.0);
        }
        assert!(p.finished && p.phase == 1.0 && p.frame() == 7);
        let mut b = LoopPlayer::new(0.5, -1.0, true, 10, 1.0, None);
        b.advance(0.1);
        assert!((b.phase - 0.4).abs() < 1e-6);
    }

    #[test]
    fn speed_zero_holds() {
        let mut p = LoopPlayer::new(0.25, 0.0, true, 8, 1.0, None);
        p.advance(1.0);
        assert!(!p.plays() && p.phase == 0.25);
    }

    #[test]
    fn random_start_and_speed_stay_in_range() {
        let mut rng = Rng::new(4);
        for _ in 0..200 {
            let p = LoopPlayer::new(0.0, 1.0, true, 33, 2.0, Some(&mut rng));
            assert!((0.0..1.0).contains(&p.phase));
            assert!((0.9..=1.1).contains(&p.speed));
        }
    }
}
