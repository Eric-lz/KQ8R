//! Sound effects: which sounds play, how loud and from which side (M3
//! step 3h-2). No sound card here: [`SoundSystem`] turns sound starts and
//! the listener's position into plain [`Order`]s for a mixer, and writes
//! every start and stop to its log.
//!
//! Rules from the behaviour note `docs/behaviour/sound.md` (session A);
//! rule numbers below are that note's. See docs/provenance/README.md.
//!
//! - Loudness = own × group, at most 1 (rule 21). Group = slider × duck
//!   (no ducking yet: speech is not played). Sliders from `Options.cs`
//!   (rules 50-52), 1 before it runs.
//! - Placed sounds: distance from Connor in 3D (rule 28), the original
//!   curve that forgets min (rule 29, kept: "Do not reproduce" 1), balance
//!   from the camera's own axes (rule 31), redone every frame (rule 32).
//!   Beyond max a one-shot stops, a loop pauses, unless it started out of
//!   range and was never heard (rule 30).
//! - Pan law (rule 23): p > 0 turns the left channel down to 1 - p, p < 0
//!   the right one to 1 + p.
//! - Mix limit 8 (rule 26): when a sound starts and more than 8 are
//!   playing, the oldest non-looping one of the lowest group stops first.
//! - Finished one-shots leave the playing list on the 200 ms housekeeping
//!   step (rule 20).
//! - A sound whose file is missing or broken is skipped, logged once per
//!   name ("Do not reproduce" 2).

use std::collections::BTreeSet;

/// Number of volume groups (rule 50).
pub const GROUPS: usize = 6;
/// World sound effects (rule 50).
pub const GROUP_EFFECTS: usize = 0;
/// Min and max of every placed one-shot: cues, hurt sounds, item landing
/// (rules 35, 41, 42).
pub const ONE_SHOT_MIN: f32 = 5_000.0;
pub const ONE_SHOT_MAX: f32 = 25_000.0;
/// Balance of a sound exactly beside the camera (rule 31).
pub const PAN_SCALE: f32 = 0.82;
/// The mix limit (rule 26).
pub const MIX_LIMIT: usize = 8;
/// The housekeeping step, real seconds (rule 20).
pub const HOUSEKEEPING: f64 = 0.2;
/// A playing placed sound gets a log line when its loudness, or (while it
/// can be heard) its balance, moved this much since its last line (our
/// choice, to keep the log short).
pub const LOG_CHANGE: f32 = 0.05;

/// The slider value and duck factor of each group (rules 50-53).
#[derive(Debug, Clone, PartialEq)]
pub struct Groups {
    pub slider: [f32; GROUPS],
    pub duck: [f32; GROUPS],
}

impl Default for Groups {
    /// Before `Options.cs` runs every group is at 1 (rule 52).
    fn default() -> Self {
        Groups { slider: [1.0; GROUPS], duck: [1.0; GROUPS] }
    }
}

impl Groups {
    /// Applies `KQSound::setSpeechVol`, `setSfxVol`, `setMusicVol` lines of
    /// a console script such as `MASK/Options.cs` (rule 51): effects set
    /// groups 0, 1, 2; speech 3; music 4, 5. A value that is not a number
    /// is skipped (our choice). Returns the lines used, for the log.
    pub fn apply_script(&mut self, script: &[u8]) -> Vec<String> {
        let text: String = script.iter().map(|&b| b as char).collect();
        let mut used = Vec::new();
        for line in text.lines() {
            let line = line.split("//").next().unwrap_or("").trim();
            let mut words = line.split_whitespace();
            let groups: &[usize] = match words.next().map(str::to_ascii_lowercase).as_deref() {
                Some("kqsound::setsfxvol") => &[0, 1, 2],
                Some("kqsound::setspeechvol") => &[3],
                Some("kqsound::setmusicvol") => &[4, 5],
                _ => continue,
            };
            if let Some(v) = words.next().and_then(|v| v.parse::<f32>().ok()) {
                for &g in groups {
                    self.slider[g] = v;
                }
                used.push(line.to_string());
            }
        }
        used
    }

    pub fn loudness(&self, group: usize) -> f32 {
        self.slider[group] * self.duck[group]
    }
}

/// Rule 29: own loudness of a placed sound at distance `d`. The original
/// divides `d` (not `d - min`) by `max - min`.
pub fn distance_factor(d: f32, min: f32, max: f32) -> f32 {
    if d < min {
        1.0
    } else if d > max {
        0.0
    } else {
        (1.0 - d / (max - min)).max(0.0)
    }
}

/// The camera's position and its own axes, in game space (Z up).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct View {
    pub position: [f32; 3],
    /// Towards the screen's right.
    pub right: [f32; 3],
    /// Where the camera looks.
    pub forward: [f32; 3],
}

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn sub(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn length(a: [f32; 3]) -> f32 {
    dot(a, a).sqrt()
}

/// Rule 31: balance from -1 (left) to +1 (right) of a sound at `at`,
/// 0.82 × sideways ÷ √(sideways² + forward²), the camera's up axis
/// ignored. Straight above or below the camera: 0 ("Do not reproduce" 4).
pub fn balance(view: &View, at: [f32; 3]) -> f32 {
    let d = sub(at, view.position);
    let (s, f) = (dot(d, view.right), dot(d, view.forward));
    let h = (s * s + f * f).sqrt();
    if h <= f32::EPSILON {
        0.0
    } else {
        PAN_SCALE * s / h
    }
}

/// Rule 23: left and right loudness for a loudness and a balance.
pub fn channel_gains(loudness: f32, pan: f32) -> [f32; 2] {
    if pan > 0.0 {
        [loudness * (1.0 - pan), loudness]
    } else {
        [loudness, loudness * (1.0 + pan)]
    }
}

/// Where a placed sound is and its two distances.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Placed {
    pub position: [f32; 3],
    pub min: f32,
    pub max: f32,
}

/// A sound to start.
#[derive(Debug, Clone, PartialEq)]
pub struct Start {
    /// The name as the data gives it (`door4.aud`, `chstopen`).
    pub name: String,
    /// `None`: a plain sound.
    pub placed: Option<Placed>,
    pub looping: bool,
    pub group: usize,
    /// Who asked, for the log (`door Door1 cue`, `KQSound::play`).
    pub why: String,
}

impl Start {
    /// A placed one-shot in group 0 (rules 35, 42).
    pub fn placed_one_shot(name: &str, position: [f32; 3], why: String) -> Self {
        Start {
            name: name.to_string(),
            placed: Some(Placed { position, min: ONE_SHOT_MIN, max: ONE_SHOT_MAX }),
            looping: false,
            group: GROUP_EFFECTS,
            why,
        }
    }

    /// A plain one-shot in group 0 (rules 40, 42).
    pub fn plain_one_shot(name: &str, why: String) -> Self {
        Start { name: name.to_string(), placed: None, looping: false, group: GROUP_EFFECTS, why }
    }
}

/// The file found for a start, or why there is none.
#[derive(Debug, Clone, PartialEq)]
pub struct Found {
    /// Where it was found, for the log.
    pub path: String,
    /// Its length in seconds.
    pub seconds: f64,
}

/// What the mixer is told to do. `id` names one playing sound.
#[derive(Debug, Clone, PartialEq)]
pub enum Order {
    /// Start `name` (as in [`Start::name`]) from its beginning.
    Start { id: u64, name: String, looping: bool, gains: [f32; 2] },
    Gains { id: u64, gains: [f32; 2] },
    /// Pause, keeping its place.
    Pause { id: u64 },
    Resume { id: u64 },
    Stop { id: u64 },
}

/// Where Connor and the camera are this frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ears {
    /// Connor's position (rule 28).
    pub listener: [f32; 3],
    pub view: View,
}

#[derive(Debug, Clone)]
struct Playing {
    id: u64,
    name: String,
    group: usize,
    looping: bool,
    placed: Option<Placed>,
    seconds: f64,
    /// Seconds played (not counting pauses).
    played: f64,
    paused: bool,
    /// A loop that started beyond max and was not yet heard (rule 30).
    never_heard: bool,
    gains: [f32; 2],
    /// Loudness and balance of its last log line.
    logged: (f32, f32),
}

/// The playing sounds and the rules that drive them.
#[derive(Debug, Clone)]
pub struct SoundSystem {
    pub groups: Groups,
    pub mix_limit: usize,
    playing: Vec<Playing>,
    next_id: u64,
    /// Real seconds since the start.
    clock: f64,
    since_housekeeping: f64,
    failed: BTreeSet<String>,
    /// Lines for `sound.log`, taken by the caller.
    pub log: Vec<String>,
    /// Orders for the mixer, taken by the caller.
    pub orders: Vec<Order>,
}

impl Default for SoundSystem {
    fn default() -> Self {
        SoundSystem::new(Groups::default())
    }
}

/// Loudness and balance of a sound for these ears; `None` for a plain one.
fn hearing(p: &Placed, ears: &Ears) -> (f32, f32, f32) {
    let d = length(sub(p.position, ears.listener));
    (d, distance_factor(d, p.min, p.max), balance(&ears.view, p.position))
}

fn fmt3(p: [f32; 3]) -> String {
    format!("({:.0}, {:.0}, {:.0})", p[0], p[1], p[2])
}

impl SoundSystem {
    pub fn new(groups: Groups) -> Self {
        SoundSystem {
            groups,
            mix_limit: MIX_LIMIT,
            playing: Vec::new(),
            next_id: 0,
            clock: 0.0,
            since_housekeeping: 0.0,
            failed: BTreeSet::new(),
            log: Vec::new(),
            orders: Vec::new(),
        }
    }

    fn note(&mut self, line: impl AsRef<str>) {
        self.log.push(format!("t {:8.3}: {}", self.clock, line.as_ref()));
    }

    /// Sounds in the playing list (rule 27: not paused).
    pub fn playing_count(&self) -> usize {
        self.playing.iter().filter(|p| !p.paused).count()
    }

    /// Ids of the sounds in the playing list, oldest first.
    pub fn playing_ids(&self) -> Vec<u64> {
        self.playing.iter().filter(|p| !p.paused).map(|p| p.id).collect()
    }

    /// Final loudness and balance of a sound: own × group, at most 1
    /// (rule 21), and its distance (0 for a plain sound).
    fn mix_values(&self, group: usize, placed: Option<&Placed>, ears: &Ears) -> (f32, f32, f32, f32) {
        let (d, own, pan) = placed.map_or((0.0, 1.0, 0.0), |p| hearing(p, ears));
        let loud = (own * self.groups.loudness(group)).min(1.0);
        (d, own, loud, pan)
    }

    /// Starts a sound. `file` is what the caller found for its name (rule
    /// 13) or why it found nothing. Returns its id, `None` if skipped.
    pub fn start(&mut self, s: &Start, file: Result<Found, String>, ears: &Ears) -> Option<u64> {
        let found = match file {
            Ok(f) => f,
            Err(why) => {
                // "Do not reproduce" 2: skipped, not put in the list.
                if self.failed.insert(s.name.to_ascii_lowercase()) {
                    self.note(format!("NOT PLAYED {} ({}): {why} (logged once per name)", s.name, s.why));
                }
                return None;
            }
        };
        // Rule 26: more than the limit already playing: one goes first.
        if self.playing_count() > self.mix_limit {
            let victim = self
                .playing
                .iter()
                .enumerate()
                .filter(|(_, p)| !p.looping && !p.paused)
                .min_by_key(|(n, p)| (p.group, *n))
                .map(|(n, _)| n);
            if let Some(n) = victim {
                let p = self.playing.remove(n);
                self.orders.push(Order::Stop { id: p.id });
                let id = self.next_id;
                self.note(format!("stop #{} {}: mix limit ({} playing, for #{id})", p.id, p.name, self.playing_count() + 1));
            }
        }
        let id = self.next_id;
        self.next_id += 1;
        let (d, own, loud, pan) = self.mix_values(s.group, s.placed.as_ref(), ears);
        let gains = channel_gains(loud, pan);
        let beyond = s.placed.is_some_and(|p| d > p.max);
        let where_ = match &s.placed {
            Some(p) => format!(
                "placed at {} min {:.0} max {:.0}: distance {d:.0}, own {own:.3}",
                fmt3(p.position),
                p.min,
                p.max
            ),
            None => "plain".into(),
        };
        self.note(format!(
            "start #{id} {} ({}, {:.3} s{}) group {} {where_}, loudness {loud:.3}, balance {pan:+.3} (L {:.3} R {:.3}) [{}]",
            s.name,
            found.path,
            found.seconds,
            if s.looping { ", loops" } else { "" },
            s.group,
            gains[0],
            gains[1],
            s.why
        ));
        self.orders.push(Order::Start { id, name: s.name.clone(), looping: s.looping, gains });
        self.playing.push(Playing {
            id,
            name: s.name.clone(),
            group: s.group,
            looping: s.looping,
            placed: s.placed,
            seconds: found.seconds,
            played: 0.0,
            paused: false,
            never_heard: s.looping && beyond,
            gains,
            logged: (loud, pan),
        });
        if beyond {
            if s.looping {
                self.note(format!("#{id} {}: started beyond max: plays on silently until Connor comes in range (rule 30)", s.name));
            } else {
                self.stop_at(self.playing.len() - 1, "beyond max at its start");
            }
        }
        Some(id)
    }

    fn stop_at(&mut self, n: usize, why: &str) {
        let p = self.playing.remove(n);
        self.orders.push(Order::Stop { id: p.id });
        self.note(format!("stop #{} {}: {why} (played {:.3} of {:.3} s)", p.id, p.name, p.played, p.seconds));
    }

    /// Stops a sound by id (a deleted or switched-off owner, 3h-3).
    pub fn stop(&mut self, id: u64, why: &str) {
        if let Some(n) = self.playing.iter().position(|p| p.id == id) {
            self.stop_at(n, why);
        }
    }

    /// One frame: `dt` real seconds passed. Moves the clock, does the
    /// housekeeping steps due, and redoes every placed sound (rule 32).
    pub fn update(&mut self, dt: f64, ears: &Ears) {
        self.clock += dt;
        for p in self.playing.iter_mut().filter(|p| !p.paused) {
            p.played += dt;
        }
        self.since_housekeeping += dt;
        while self.since_housekeeping >= HOUSEKEEPING {
            self.since_housekeeping -= HOUSEKEEPING;
            // Rule 20: finished one-shots leave the list.
            while let Some(n) = self.playing.iter().position(|p| !p.looping && p.played >= p.seconds) {
                self.stop_at(n, "finished");
            }
        }
        let mut n = 0;
        while n < self.playing.len() {
            let p = self.playing[n].clone();
            let (d, _own, loud, pan) = self.mix_values(p.group, p.placed.as_ref(), ears);
            let beyond = p.placed.is_some_and(|pl| d > pl.max);
            if beyond && !p.looping {
                self.stop_at(n, &format!("beyond max (distance {d:.0})"));
                continue;
            }
            if beyond && !p.paused && !p.never_heard {
                self.playing[n].paused = true;
                self.orders.push(Order::Pause { id: p.id });
                self.note(format!("pause #{} {}: beyond max (distance {d:.0}), keeps its place", p.id, p.name));
            } else if !beyond && p.paused {
                self.playing[n].paused = false;
                self.orders.push(Order::Resume { id: p.id });
                self.note(format!("resume #{} {}: back in range (distance {d:.0})", p.id, p.name));
            }
            if !beyond {
                self.playing[n].never_heard = false;
            }
            let gains = channel_gains(loud, pan);
            if gains != p.gains {
                self.playing[n].gains = gains;
                self.orders.push(Order::Gains { id: p.id, gains });
            }
            let (l0, p0) = p.logged;
            if p.placed.is_some() && ((loud - l0).abs() >= LOG_CHANGE || (loud > 0.0 && (pan - p0).abs() >= LOG_CHANGE)) {
                self.playing[n].logged = (loud, pan);
                self.note(format!(
                    "#{} {}: distance {d:.0}, loudness {loud:.3}, balance {pan:+.3} (L {:.3} R {:.3})",
                    p.id, p.name, gains[0], gains[1]
                ));
            }
            n += 1;
        }
    }
}

/// Footsteps (M3 step 3h-4, rules 36-39): the sound set for the ground
/// flags under a character, the first that matches in this order; `None`
/// = no step (e.g. only 0x10, or only 0x80 lava). `weirda` is never picked
/// and 0x10 is silent ("Do not reproduce" 5, kept).
pub fn footstep_set(flags: u32) -> Option<&'static str> {
    if flags & 0x1 != 0 {
        Some("dirt")
    } else if flags & 0x2 != 0 {
        Some("grass")
    } else if flags & 0x4 != 0 {
        Some("rock")
    } else if flags & (0x8 | 0x100 | 0x40) != 0 {
        Some("water")
    } else if flags & 0x20 != 0 {
        Some("weirdb")
    } else {
        None
    }
}

/// Rule 39: the flags of a building's floor from its `footsoundType`: 2ⁿ
/// for n from 0 to 8, else 0x1 (dirt). A building without the key: 0x1
/// too (our stand-in, Q81 a: not traced).
pub fn building_floor_flags(footsound_type: Option<i64>) -> u32 {
    match footsound_type {
        Some(n) if (0..=8).contains(&n) => 1 << n,
        _ => 0x1,
    }
}

/// Rule 38: which of the set's three files a roll of 0-3 picks: 25 % file
/// 1, 50 % file 2, 25 % file 3. No extension (found by rule 13).
pub fn footstep_file(set: &str, roll: u32) -> String {
    let n = match roll % 4 {
        0 => 1,
        3 => 3,
        _ => 2,
    };
    format!("{set}{n}")
}

/// A small random number source for the footstep pick (our own: a
/// 64-bit xorshift with a fixed start, so test runs repeat).
#[derive(Debug, Clone)]
pub struct Dice(u64);

impl Default for Dice {
    fn default() -> Self {
        Dice(0x9E37_79B9_7F4A_7C15)
    }
}

impl Dice {
    /// 0, 1, 2 or 3, each a quarter of the time.
    pub fn roll4(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        (x >> 62) as u32
    }
}

/// Radius of a sound object's ball in the near/far/off test: it has no
/// shape (rule 47, `object-lighting.md` rule 35).
pub const AUDIBLE_RADIUS: f32 = kq8_data::objects::NO_SHAPE_RADIUS;

/// A world sound object, `classType=KQAudibleObject` (M3 step 3h-3, rules
/// 43-48): an invisible looping sound in group 0, placed when `isSound3D`
/// is 1.
#[derive(Debug, Clone, PartialEq)]
pub struct Audible {
    pub name: String,
    /// `soundFileName`.
    pub file: String,
    /// `isSound3D=1`.
    pub placed: bool,
    pub min: f32,
    pub max: f32,
    pub position: [f32; 3],
    /// The playing sound's id while switched on.
    playing: Option<u64>,
    /// Its file could not be played: not tried again.
    failed: bool,
    /// `deleteObject` stopped it for good (rule 48).
    deleted: bool,
}

impl Audible {
    pub fn new(name: &str, file: &str, placed: bool, min: f32, max: f32, position: [f32; 3]) -> Self {
        Audible { name: name.into(), file: file.into(), placed, min, max, position, playing: None, failed: false, deleted: false }
    }

    pub fn is_playing(&self) -> bool {
        self.playing.is_some()
    }

    /// Rules 46-47: on while near or far, off from `visible` (the world's
    /// `visibleDistance`) on, measured from `reference` (Connor) to its
    /// ball of radius 100.
    pub fn is_active(&self, reference: [f32; 3], visible: f32) -> bool {
        let d = length(sub(self.position, reference)) - AUDIBLE_RADIUS;
        kq8_data::objects::Activity::of(d, visible, 1.0) != kq8_data::objects::Activity::Off
    }

    /// One frame (before [`SoundSystem::update`]): switched on, it starts
    /// from the beginning (rule 46); switched off or deleted, it stops
    /// (rules 46, 48). `file` finds its file (rule 13).
    pub fn update(
        &mut self,
        system: &mut SoundSystem,
        ears: &Ears,
        visible: f32,
        deleted: bool,
        file: &mut dyn FnMut(&str) -> Result<Found, String>,
    ) {
        if self.deleted {
            return;
        }
        let d = length(sub(self.position, ears.listener));
        if deleted {
            self.deleted = true;
            if let Some(id) = self.playing.take() {
                system.stop(id, &format!("{} deleted (deleteObject)", self.name));
            }
            return;
        }
        let active = self.is_active(ears.listener, visible);
        match (active, self.playing) {
            (true, None) if !self.failed => {
                let start = Start {
                    name: self.file.clone(),
                    placed: self.placed.then_some(Placed { position: self.position, min: self.min, max: self.max }),
                    looping: true,
                    group: GROUP_EFFECTS,
                    why: format!("{} switched on: distance {d:.0} - {AUDIBLE_RADIUS:.0} < {visible:.0}", self.name),
                };
                self.playing = system.start(&start, file(&self.file), ears);
                self.failed = self.playing.is_none();
            }
            (false, Some(id)) => {
                self.playing = None;
                system.stop(id, &format!("{} switched off: distance {d:.0} - {AUDIBLE_RADIUS:.0} >= {visible:.0}", self.name));
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Camera at the origin looking along +y, right is +x; Connor at the
    /// origin too.
    fn ears() -> Ears {
        Ears { listener: [0.0; 3], view: View { position: [0.0; 3], right: [1.0, 0.0, 0.0], forward: [0.0, 1.0, 0.0] } }
    }

    fn found(seconds: f64) -> Result<Found, String> {
        Ok(Found { path: "test".into(), seconds })
    }

    fn close(a: f32, b: f32) -> bool {
        (a - b).abs() < 1e-4
    }

    #[test]
    fn the_original_distance_curve() {
        // sound.md "How to check": a playsound cue.
        let f = |d| distance_factor(d, ONE_SHOT_MIN, ONE_SHOT_MAX);
        assert_eq!(f(4_999.0), 1.0);
        assert!(close(f(5_000.0), 0.75));
        assert!(close(f(10_000.0), 0.5));
        assert_eq!(f(20_000.0), 0.0);
        assert_eq!(f(24_000.0), 0.0);
        assert_eq!(f(25_001.0), 0.0);
        // The fountain: 0.919 at 1,500, 0.5 at 9,250, 0 from 18,500.
        let g = |d| distance_factor(d, 1_500.0, 20_000.0);
        assert!((g(1_500.0) - 0.919).abs() < 0.001);
        assert!(close(g(9_250.0), 0.5));
        assert_eq!(g(18_500.0), 0.0);
    }

    #[test]
    fn balance_follows_the_camera() {
        let v = ears().view;
        assert!(close(balance(&v, [100.0, 0.0, 0.0]), 0.82), "beside, on the right");
        assert!(close(balance(&v, [-100.0, 0.0, 50.0]), -0.82), "beside, on the left; height ignored");
        assert!(close(balance(&v, [0.0, 500.0, 0.0]), 0.0), "straight ahead");
        assert!(close(balance(&v, [0.0, -500.0, 0.0]), 0.0), "straight behind");
        assert!(close(balance(&v, [100.0, 100.0, 0.0]), 0.82 / 2f32.sqrt()));
        assert!(close(balance(&v, [1000.0, 1000.0, 0.0]), 0.82 / 2f32.sqrt()), "distance doesn't change it");
        assert_eq!(balance(&v, [0.0, 0.0, 300.0]), 0.0, "straight above: centre");
    }

    #[test]
    fn pan_law() {
        assert_eq!(channel_gains(0.5, 0.0), [0.5, 0.5]);
        assert!(close(channel_gains(1.0, 0.82)[0], 0.18) && channel_gains(1.0, 0.82)[1] == 1.0);
        assert!(channel_gains(1.0, -0.82)[0] == 1.0 && close(channel_gains(1.0, -0.82)[1], 0.18));
    }

    #[test]
    fn shipped_options_give_014_for_effects() {
        let mut g = Groups::default();
        let used = g.apply_script(b"KQSound::setSpeechVol 0.34\r\nKQSound::setSfxVol    0.14\r\nKQSound::setMusicVol  0.14\r\nsetShadowMask 15\r\n");
        assert_eq!(used.len(), 3);
        assert_eq!(g.slider, [0.14, 0.14, 0.14, 0.34, 0.14, 0.14]);
        // A door sound next to Connor = 0.14 ("How to check").
        let mut s = SoundSystem::new(g);
        s.start(&Start::placed_one_shot("door4.aud", [100.0, 0.0, 0.0], "test".into()), found(1.0), &ears());
        let Order::Start { gains, .. } = &s.orders[0] else { panic!() };
        assert!(close(gains[1], 0.14) && close(gains[0], 0.14 * 0.18), "{gains:?}");
    }

    #[test]
    fn mix_limit_stops_the_oldest_one_shots() {
        // "How to check": 12 one-shots at once; the first three stop as the
        // 10th, 11th and 12th start; 9 play at the end.
        let mut s = SoundSystem::default();
        for n in 0..12 {
            s.start(&Start::plain_one_shot(&format!("s{n}"), "test".into()), found(5.0), &ears());
        }
        assert_eq!(s.playing_ids(), (3..12).collect::<Vec<u64>>());
        let stops: Vec<u64> = s.orders.iter().filter_map(|o| if let Order::Stop { id } = o { Some(*id) } else { None }).collect();
        assert_eq!(stops, [0, 1, 2]);
    }

    #[test]
    fn mix_limit_spares_loops_and_takes_the_lowest_group_first() {
        let mut s = SoundSystem::default();
        let mut start = |name: &str, group: usize, looping: bool| {
            let st = Start { name: name.into(), placed: None, looping, group, why: "test".into() };
            s.start(&st, found(5.0), &ears());
        };
        start("loop", 0, true);
        start("menu", 1, false);
        for n in 0..7 {
            start(&format!("fx{n}"), 0, false);
        }
        start("new", 1, false); // 9 playing: fx0 (group 0, oldest one-shot) goes
        assert!(!s.playing.iter().any(|p| p.name == "fx0"));
        assert!(s.playing.iter().any(|p| p.name == "loop") && s.playing.iter().any(|p| p.name == "menu"));
        assert_eq!(s.playing_count(), 9);
    }

    #[test]
    fn only_loops_playing_means_nothing_stops() {
        let mut s = SoundSystem::default();
        for n in 0..12 {
            s.start(&Start { name: format!("l{n}"), placed: None, looping: true, group: 0, why: "t".into() }, found(1.0), &ears());
        }
        assert_eq!(s.playing_count(), 12);
    }

    #[test]
    fn one_shots_finish_on_the_housekeeping_step() {
        let mut s = SoundSystem::default();
        s.start(&Start::plain_one_shot("a", "t".into()), found(0.25), &ears());
        s.update(0.1, &ears());
        s.update(0.1, &ears()); // step at 0.2: not done yet
        assert_eq!(s.playing_count(), 1);
        s.update(0.1, &ears()); // 0.3 played, but the next step is at 0.4
        assert_eq!(s.playing_count(), 1);
        s.update(0.1, &ears());
        assert_eq!(s.playing_count(), 0);
        assert!(s.log.iter().any(|l| l.contains("stop #0 a: finished")));
    }

    #[test]
    fn a_one_shot_stops_beyond_max() {
        let mut s = SoundSystem::default();
        let mut e = ears();
        s.start(&Start::placed_one_shot("door4.aud", [0.0, 10_000.0, 0.0], "t".into()), found(9.0), &e);
        e.listener = [0.0, 10_000.0 - 25_001.0, 0.0];
        s.update(0.01, &e);
        assert_eq!(s.playing_count(), 0);
        assert!(s.log.iter().any(|l| l.contains("beyond max (distance 25001)")), "{:?}", s.log);
        // Started beyond max: stopped at once.
        s.start(&Start::placed_one_shot("far", [0.0, 30_000.0, 0.0], "t".into()), found(9.0), &ears());
        assert_eq!(s.playing_count(), 0);
    }

    #[test]
    fn a_loop_pauses_beyond_max_and_resumes() {
        let looping = |at: [f32; 3]| Start {
            name: "fount.aud".into(),
            placed: Some(Placed { position: at, min: 1_500.0, max: 20_000.0 }),
            looping: true,
            group: 0,
            why: "t".into(),
        };
        let mut s = SoundSystem::default();
        let mut e = ears();
        s.start(&looping([0.0, 1_000.0, 0.0]), found(3.0), &e);
        e.listener = [0.0, -20_000.0, 0.0];
        s.update(0.01, &e);
        assert_eq!(s.playing_count(), 0, "paused: out of the list");
        e.listener = [0.0; 3];
        s.update(0.01, &e);
        assert_eq!(s.playing_count(), 1);
        assert!(s.orders.contains(&Order::Pause { id: 0 }) && s.orders.contains(&Order::Resume { id: 0 }));
        // Started beyond max: not paused, plays silently and counts.
        let mut s = SoundSystem::default();
        s.start(&looping([0.0, 50_000.0, 0.0]), found(3.0), &ears());
        s.update(0.01, &ears());
        assert_eq!(s.playing_count(), 1);
        let Order::Start { gains, .. } = &s.orders[0] else { panic!() };
        assert_eq!(*gains, [0.0, 0.0]);
        // Heard once, then beyond max again: paused.
        let mut e = ears();
        e.listener = [0.0, 45_000.0, 0.0];
        s.update(0.01, &e);
        s.update(0.01, &ears());
        assert_eq!(s.playing_count(), 0);
    }

    #[test]
    fn a_missing_file_is_skipped_and_logged_once() {
        let mut s = SoundSystem::default();
        for _ in 0..3 {
            assert_eq!(s.start(&Start::plain_one_shot("poffup.aud", "t".into()), Err("not found".into()), &ears()), None);
        }
        assert_eq!(s.playing_count(), 0);
        assert_eq!(s.log.iter().filter(|l| l.contains("NOT PLAYED poffup.aud")).count(), 1);
        assert!(s.orders.is_empty());
    }

    #[test]
    fn a_sound_object_switches_off_beyond_the_draw_distance_and_restarts() {
        // The waterfall: max 35,000, but off from 27,000 + 100 (rule 47).
        let mut s = SoundSystem::default();
        let mut w = Audible::new("WaterfallSound", "watrfall.aud", true, 1_500.0, 35_000.0, [0.0; 3]);
        let mut file = |_: &str| found(10.0);
        let at = |y: f32| Ears { listener: [0.0, y, 0.0], view: ears().view };
        w.update(&mut s, &at(27_099.0), 27_000.0, false, &mut file);
        assert!(w.is_playing());
        // Still heard just inside: about 0.19 (rule 47), 1 - 27,099 / 33,500.
        let Order::Start { gains, .. } = &s.orders[0] else { panic!("{:?}", s.orders) };
        assert!((gains[0] - 0.191).abs() < 0.001, "{gains:?}");
        w.update(&mut s, &at(27_100.0), 27_000.0, false, &mut file);
        assert!(!w.is_playing() && s.playing_count() == 0);
        assert!(s.log.iter().any(|l| l.contains("WaterfallSound switched off: distance 27100 - 100 >= 27000")), "{:?}", s.log);
        // Back in: a new start from the beginning.
        w.update(&mut s, &at(1_000.0), 27_000.0, false, &mut file);
        assert!(matches!(s.orders.last(), Some(Order::Start { id: 1, looping: true, .. })));
        // Deleted: stopped, never again.
        w.update(&mut s, &at(1_000.0), 27_000.0, true, &mut file);
        w.update(&mut s, &at(1_000.0), 27_000.0, false, &mut file);
        assert!(!w.is_playing() && s.playing_count() == 0);
        assert!(s.log.iter().any(|l| l.contains("WaterfallSound deleted")));
    }

    #[test]
    fn a_plain_sound_object_and_a_missing_file() {
        let mut s = SoundSystem::default();
        let mut w = Audible::new("Hum", "hum.aud", false, 0.0, 0.0, [0.0, 5_000.0, 0.0]);
        w.update(&mut s, &ears(), 27_000.0, false, &mut |_| found(1.0));
        let Order::Start { gains, looping, .. } = &s.orders[0] else { panic!() };
        assert!(*looping && *gains == [1.0, 1.0], "isSound3D=0: plain, full loudness");
        let mut m = Audible::new("Gone", "nothere.aud", true, 1.0, 2.0, [0.0; 3]);
        let mut tries = 0;
        for _ in 0..3 {
            m.update(&mut s, &ears(), 27_000.0, false, &mut |_| {
                tries += 1;
                Err("not found".into())
            });
        }
        assert_eq!(tries, 1, "a failed file is not looked for again");
    }

    #[test]
    fn footstep_sets_and_files() {
        assert_eq!(footstep_set(0x1 | 0x2), Some("dirt"), "first match wins");
        assert_eq!(footstep_set(0x2), Some("grass"));
        assert_eq!(footstep_set(0x4), Some("rock"));
        assert_eq!(footstep_set(0x8), Some("water"));
        assert_eq!(footstep_set(0x100), Some("water"));
        assert_eq!(footstep_set(0x40), Some("water"));
        assert_eq!(footstep_set(0x20), Some("weirdb"));
        assert_eq!(footstep_set(0x10), None, "weird1 / cav2.kq: silent");
        assert_eq!(footstep_set(0x80), None, "lava alone: silent");
        assert_eq!(footstep_set(0), None);
        // Rule 39: bigbrig2.kq 2 -> rock, cav2.kq 4 -> silent.
        assert_eq!(footstep_set(building_floor_flags(Some(2))), Some("rock"));
        assert_eq!(footstep_set(building_floor_flags(Some(4))), None);
        assert_eq!(building_floor_flags(Some(9)), 0x1);
        assert_eq!(building_floor_flags(None), 0x1, "stand-in, Q81 a");
        assert_eq!([0, 1, 2, 3].map(|r| footstep_file("grass", r)), ["grass1", "grass2", "grass2", "grass3"]);
    }

    #[test]
    fn the_dice_give_25_50_25() {
        let mut d = Dice::default();
        let mut count = [0u32; 3];
        for _ in 0..40_000 {
            let f = footstep_file("x", d.roll4());
            count[(f.as_bytes()[1] - b'1') as usize] += 1;
        }
        for (got, want) in count.iter().zip([10_000.0, 20_000.0, 10_000.0]) {
            assert!((*got as f64 - want).abs() < want * 0.05, "{count:?}");
        }
    }

    #[test]
    fn loudness_is_capped_at_1() {
        let mut g = Groups::default();
        g.slider[0] = 3.0;
        let mut s = SoundSystem::new(g);
        s.start(&Start::plain_one_shot("a", "t".into()), found(1.0), &ears());
        assert_eq!(s.orders[0], Order::Start { id: 0, name: "a".into(), looping: false, gains: [1.0, 1.0] });
    }
}
