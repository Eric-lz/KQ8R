//! Smooth animation (`docs/DESIGN.md` "smooth animation", steps S-1 to
//! S-6): ours, not from the original game. Off by default. Drawing only:
//! the rules, cues and logs keep the original 15 updates a second and the
//! original frames; with the switch on, model parts are drawn part-way
//! between two frames and moving things between two updates.
//!
//! The switch is one flag for the whole program: `--smooth` at the start
//! turns it on, the console word `smooth [on|off]` while playing.
//!
//! Smooth joins (DESIGN "smooth joins", J-1 to J-4): with the switch on, a
//! change of movement is drawn mixed from the old pose to the new over a
//! short time (`smooth join <seconds>`); the join records the drawn parts'
//! tags name are kept here and written to the graphics card every frame
//! ([`write_joins`]).

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Mutex;
use std::time::Instant;

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

/// Smooth joins (J-3): how long a change of movement is drawn mixed, in
/// milliseconds (0: joins off). First guess 0.15 s (about 2 updates).
static JOIN_MS: AtomicU32 = AtomicU32::new(150);

/// The join length in seconds.
pub fn join_length() -> f32 {
    JOIN_MS.load(Ordering::Relaxed) as f32 / 1000.0
}

/// Sets the join length (seconds, 0 to 2); returns a line for the console
/// and the log.
pub fn set_join(seconds: f32) -> String {
    let ms = (seconds.clamp(0.0, 2.0) * 1000.0).round() as u32;
    JOIN_MS.store(ms, Ordering::Relaxed);
    if ms == 0 {
        "smooth joins off (a change of movement jumps)".to_string()
    } else {
        format!("smooth joins: a change of movement drawn mixed over {:.2} s (ours; with smooth animation on)", ms as f32 / 1000.0)
    }
}

/// A join record in use: the old part's node-frame (its pose at the
/// change), when the join started and how long it lasts.
#[derive(Clone, Copy)]
struct Record {
    from: u32,
    started: Instant,
    length: f32,
}

/// The records a tag can name (1-255; index 0 here is record 1).
static RECORDS: Mutex<Vec<Option<Record>>> = Mutex::new(Vec::new());

/// Joins started, parts drawn mixed in them and all their parts, since the
/// last report.
static JOINS: Mutex<[u64; 3]> = Mutex::new([0; 3]);

/// A free record for a join from node-frame `from`: its number (1-255),
/// or `None` if all are in use (that part then jumps).
pub fn take_record(from: u32, started: Instant, length: f32) -> Option<u8> {
    let mut records = RECORDS.lock().unwrap();
    if records.is_empty() {
        records.resize(crate::anim_gpu::JOIN_RECORDS - 1, None);
    }
    let k = records.iter().position(Option::is_none)?;
    records[k] = Some(Record { from, started, length });
    Some(k as u8 + 1)
}

/// Gives record `record` (1-255) back.
pub fn free_record(record: u8) {
    if let Some(r) = RECORDS.lock().unwrap().get_mut((record as usize).wrapping_sub(1)) {
        *r = None;
    }
}

/// Records in use.
pub fn records_in_use() -> usize {
    RECORDS.lock().unwrap().iter().flatten().count()
}

/// A join started: `mixed` of its new frame's `parts` drawn mixed.
pub fn count_join(mixed: usize, parts: usize) {
    let mut j = JOINS.lock().unwrap();
    *j = [j[0] + 1, j[1] + mixed as u64, j[2] + parts as u64];
}

/// How far a join has gone from the old pose to the new (0-1), `t` the
/// part of its length gone by: starting and ending gently.
pub fn eased(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Each record's node-frame and how far its join has gone at `now` (a
/// free record: done, so it can't move anything).
fn records_at(now: Instant) -> Vec<(u32, f32)> {
    RECORDS
        .lock()
        .unwrap()
        .iter()
        .map(|r| match r {
            Some(r) if r.length > 0.0 => (r.from, eased(now.duration_since(r.started).as_secs_f32() / r.length)),
            _ => (0, 1.0),
        })
        .collect()
}

/// Writes the join records to the graphics card (every frame while any is
/// in use, and once more after the last ends), as the scene's lights are.
pub fn write_joins(
    mut buffers: bevy::prelude::ResMut<bevy::prelude::Assets<bevy::render::storage::ShaderBuffer>>,
    anim: Option<bevy::prelude::Res<crate::anim_gpu::AnimBuffers>>,
    mut was_active: bevy::prelude::Local<bool>,
) {
    let active = records_in_use() > 0;
    if !active && !*was_active {
        return;
    }
    *was_active = active;
    let Some(anim) = anim else { return };
    if let Some(mut b) = buffers.get_mut(&anim.joins) {
        *b = bevy::render::storage::ShaderBuffer::new(crate::anim_gpu::join_data(&records_at(Instant::now())), bevy::asset::RenderAssetUsages::default());
    }
}

/// `parts` tags were set part-way (amount above 0).
pub fn count_mixed(parts: usize) {
    MIXED.fetch_add(parts as u64, Ordering::Relaxed);
}

/// The 2-second report's words: the switch and the parts set part-way
/// since the last report; the joins started (J-3) and their parts.
pub fn report() -> String {
    let j = std::mem::take(&mut *JOINS.lock().unwrap());
    format!(
        "smooth {} ({} part tags set part-way; {} joins, {} of {} parts mixed, {} records in use)",
        if on() { "on" } else { "off" },
        MIXED.swap(0, Ordering::Relaxed),
        j[0],
        j[1],
        j[2],
        records_in_use()
    )
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
    fn a_join_starts_and_ends_gently() {
        assert_eq!((eased(-1.0), eased(0.0), eased(0.5), eased(1.0), eased(2.0)), (0.0, 0.0, 0.5, 1.0, 1.0));
        assert!(eased(0.1) < 0.1 && eased(0.9) > 0.9);
    }

    #[test]
    fn join_records_are_numbered_from_1_and_given_back() {
        let now = Instant::now();
        let a = take_record(7, now, 0.15).unwrap();
        let b = take_record(8, now, 0.15).unwrap();
        assert!(a >= 1 && b >= 1 && a != b);
        free_record(a);
        assert_eq!(take_record(9, now, 0.15), Some(a), "a free record is used again");
        free_record(a);
        free_record(b);
    }

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
