//! World music, `KQMusic` (M3 step 3h-5). The world's `music.kq` names a
//! list of tunes; scripts push, pop and play tunes on it. Each tune is a
//! looping plain sound in group 5 run by [`SoundSystem`]; only the top one
//! is meant to be heard, and changes cross-fade.
//!
//! Rules from `docs/behaviour/sound.md` "World music", rules 57-57l (and
//! 33 for fades); rule numbers below are that note's. See
//! docs/provenance/README.md.
//!
//! - The list from `music.kq`: `music0` at the bottom, the last one on top
//!   (rule 57). Names are compared ignoring capitals (57a).
//! - World start: the top starts at full loudness, no fade (57b).
//! - `push`: nothing if the name is anywhere on the list; else the old top
//!   fades out (stays on the list, stopped at 0), the new one starts at
//!   once at 0 and fades in; full loudness if the list was empty (57c).
//! - `pop`: not on the list, nothing; the top fades out and leaves, the
//!   one below fades in from its beginning (or, still fading out, from
//!   where it is); the only one leaves silence; a lower one leaves at once,
//!   its fade cut off (57d).
//! - `play`: the top fades out and leaves, the new one fades in; no name
//!   check (57e).
//!
//! When script requests act (0.2 s later for push and pop) is up to the
//! caller (`crate::active`, rule 57f).

use crate::sound::{Ears, Found, SoundSystem, Start, GROUP_WORLD_MUSIC, MUSIC_FADE_RATE};

/// A request from a script (rule 57f).
#[derive(Debug, Clone, PartialEq)]
pub enum MusicRequest {
    Push(String),
    Pop(String),
    Play(String),
}

impl MusicRequest {
    pub fn describe(&self) -> String {
        match self {
            MusicRequest::Push(n) => format!("KQMusic::push {n}"),
            MusicRequest::Pop(n) => format!("KQMusic::pop {n}"),
            MusicRequest::Play(n) => format!("KQMusic::play {n}"),
        }
    }
}

/// One tune on the list.
#[derive(Debug, Clone, PartialEq)]
pub struct Tune {
    pub name: String,
    /// Its sound while it plays (or fades out); `None` when stopped.
    id: Option<u64>,
}

/// The music object and its list of tunes, bottom first.
#[derive(Debug, Clone, PartialEq)]
pub struct Music {
    /// The object's name (`Music` in every world).
    pub object: String,
    pub tunes: Vec<Tune>,
    started: bool,
}

/// Finds and checks a tune's file (rule 13), as for any sound.
pub type FileFinder<'a> = &'a mut dyn FnMut(&str) -> Result<Found, String>;

impl Music {
    /// The tunes of `music.kq` in their order (`music0` first, rule 57).
    pub fn new(object: &str, names: &[String]) -> Self {
        Music { object: object.into(), tunes: names.iter().map(|n| Tune { name: n.clone(), id: None }).collect(), started: false }
    }

    /// The tune names, bottom first, for the log.
    pub fn names(&self) -> Vec<&str> {
        self.tunes.iter().map(|t| t.name.as_str()).collect()
    }

    fn find(&self, name: &str) -> Option<usize> {
        self.tunes.iter().position(|t| t.name.eq_ignore_ascii_case(name))
    }

    /// Whether the tune's sound is still in the sound system's list.
    fn sounding(&self, i: usize, system: &SoundSystem) -> Option<u64> {
        self.tunes[i].id.filter(|&id| system.is_playing(id))
    }

    fn begin(&mut self, i: usize, level: f32, why: &str, system: &mut SoundSystem, ears: &Ears, file: FileFinder) {
        let name = self.tunes[i].name.clone();
        let start = Start { name: name.clone(), placed: None, looping: true, group: GROUP_WORLD_MUSIC, why: why.into() };
        let id = system.start_at_level(&start, file(&name), ears, level);
        if let (Some(id), true) = (id, level < 1.0) {
            system.fade(id, 1.0, MUSIC_FADE_RATE);
        }
        self.tunes[i].id = id;
    }

    /// Rule 57b: the music object enters the world; its top tune starts
    /// from the beginning at full loudness. Once only.
    pub fn enter(&mut self, system: &mut SoundSystem, ears: &Ears, file: FileFinder) {
        if std::mem::replace(&mut self.started, true) {
            return;
        }
        if let Some(top) = self.tunes.len().checked_sub(1) {
            let why = format!("{}: the world's tune, full loudness, no fade (rule 57b)", self.object);
            self.begin(top, 1.0, &why, system, ears, file);
        }
    }

    /// A request, now due. Returns a line for the log.
    pub fn apply(&mut self, req: &MusicRequest, system: &mut SoundSystem, ears: &Ears, file: FileFinder) -> String {
        let what = req.describe();
        let line = match req {
            MusicRequest::Push(name) => self.push(name, system, ears, file),
            MusicRequest::Pop(name) => self.pop(name, system, ears, file),
            MusicRequest::Play(name) => self.play(name, system, ears, file),
        };
        format!("{what}: {line}; list now [{}]", self.names().join(", "))
    }

    /// Fades the top out, keeping it on the list (rule 57c 2).
    fn cover_top(&mut self, system: &mut SoundSystem) -> bool {
        let Some(top) = self.tunes.len().checked_sub(1) else { return false };
        if let Some(id) = self.sounding(top, system) {
            system.fade(id, 0.0, MUSIC_FADE_RATE);
        }
        true
    }

    fn push(&mut self, name: &str, system: &mut SoundSystem, ears: &Ears, file: FileFinder) -> String {
        if let Some(i) = self.find(name) {
            return format!("already on the list ({}); nothing (rule 57c 1)", self.tunes[i].name);
        }
        let had_top = self.cover_top(system);
        self.tunes.push(Tune { name: name.into(), id: None });
        let top = self.tunes.len() - 1;
        if had_top {
            self.begin(top, 0.0, "KQMusic::push: fades in (rule 57c 3)", system, ears, file);
            "the old top fades out and stays on the list, the new one fades in (rule 57c 2-3)".into()
        } else {
            self.begin(top, 1.0, "KQMusic::push on an empty list: full loudness (rule 57c 4)", system, ears, file);
            "the list was empty: full loudness at once (rule 57c 4)".into()
        }
    }

    fn pop(&mut self, name: &str, system: &mut SoundSystem, ears: &Ears, file: FileFinder) -> String {
        let Some(i) = self.find(name) else {
            return "not on the list; nothing (rule 57d 1)".into();
        };
        let sounding = self.sounding(i, system);
        let gone = self.tunes.remove(i);
        if i < self.tunes.len() {
            // Rule 57d 4: a lower one leaves silently, its fade cut off.
            if let Some(id) = sounding {
                system.stop(id, &format!("{}: popped while lower down; its fade is cut off (rule 57d 4)", gone.name));
            }
            return "lower down: taken off silently, the top plays on (rule 57d 4)".into();
        }
        if let Some(id) = sounding {
            system.fade(id, 0.0, MUSIC_FADE_RATE);
        }
        let Some(below) = self.tunes.len().checked_sub(1) else {
            return "the only tune: it fades out, then silence (rule 57d 3)".into();
        };
        if let Some(id) = self.sounding(below, system) {
            // Covered less than 1.4 s ago: still fading out.
            system.set_level(id, 0.0);
            system.fade(id, 1.0, MUSIC_FADE_RATE);
            format!("the top fades out; {} was still fading out: from 0, from where it is in the music (rule 57d 2)", self.tunes[below].name)
        } else {
            self.begin(below, 0.0, "KQMusic::pop: the tune below fades in from its beginning (rule 57d 2)", system, ears, file);
            format!("the top fades out; {} fades in from its beginning (rule 57d 2)", self.tunes[below].name)
        }
    }

    fn play(&mut self, name: &str, system: &mut SoundSystem, ears: &Ears, file: FileFinder) -> String {
        let had_top = !self.tunes.is_empty();
        if let Some(top) = self.tunes.len().checked_sub(1) {
            if let Some(id) = self.sounding(top, system) {
                system.fade(id, 0.0, MUSIC_FADE_RATE);
            }
            self.tunes.pop();
        }
        self.tunes.push(Tune { name: name.into(), id: None });
        let top = self.tunes.len() - 1;
        let level = if had_top { 0.0 } else { 1.0 };
        self.begin(top, level, "KQMusic::play (rule 57e)", system, ears, file);
        if had_top {
            "the top fades out and leaves, the new one fades in (rule 57e)".into()
        } else {
            "the list was empty: full loudness at once (rule 57e)".into()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sound::{Order, View, HOUSEKEEPING};

    fn ears() -> Ears {
        Ears { listener: [0.0; 3], view: View { position: [0.0; 3], right: [1.0, 0.0, 0.0], forward: [0.0, 1.0, 0.0] } }
    }

    fn file(_: &str) -> Result<Found, String> {
        Ok(Found { path: "test".into(), seconds: 100.0 })
    }

    /// Daventry: `forest.aud`, started.
    fn daventry() -> (Music, SoundSystem) {
        let mut s = SoundSystem::default();
        let mut m = Music::new("Music", &["forest.aud".into()]);
        m.enter(&mut s, &ears(), &mut file);
        (m, s)
    }

    fn steps(s: &mut SoundSystem, n: usize) {
        for _ in 0..n {
            s.update(HOUSEKEEPING, &ears());
        }
    }

    fn id_of(m: &Music, name: &str) -> Option<u64> {
        m.tunes.iter().find(|t| t.name == name).and_then(|t| t.id)
    }

    #[test]
    fn the_world_tune_starts_at_full_loudness_and_loops() {
        let (m, s) = daventry();
        assert_eq!(s.orders, [Order::Start { id: 0, name: "forest.aud".into(), looping: true, gains: [1.0, 1.0] }]);
        assert_eq!(s.level(0), Some(1.0));
        assert_eq!(m.names(), ["forest.aud"]);
    }

    #[test]
    fn push_cross_fades_in_7_steps_and_keeps_the_old_one() {
        let (mut m, mut s) = daventry();
        let line = m.apply(&MusicRequest::Push("mchurch.aud".into()), &mut s, &ears(), &mut file);
        assert!(line.contains("rule 57c 2-3"), "{line}");
        let new = id_of(&m, "mchurch.aud").unwrap();
        assert_eq!(s.level(new), Some(0.0), "starts at once, at 0");
        steps(&mut s, 1);
        assert!((s.level(new).unwrap() - 0.15).abs() < 1e-6 && (s.level(0).unwrap() - 0.85).abs() < 1e-6);
        steps(&mut s, 5);
        assert!(s.is_playing(0), "both heard for a while");
        steps(&mut s, 1);
        assert_eq!(s.level(new), Some(1.0), "0 -> 1 in 7 steps (1.4 s)");
        assert!(!s.is_playing(0), "the old top stopped at 0");
        assert_eq!(m.names(), ["forest.aud", "mchurch.aud"], "but stays on the list");
    }

    #[test]
    fn push_of_a_name_already_on_the_list_does_nothing() {
        let (mut m, mut s) = daventry();
        m.apply(&MusicRequest::Push("mchurch.aud".into()), &mut s, &ears(), &mut file);
        let before = s.orders.len();
        for name in ["MCHURCH.AUD", "forest.aud"] {
            let line = m.apply(&MusicRequest::Push(name.into()), &mut s, &ears(), &mut file);
            assert!(line.contains("rule 57c 1"), "{line}");
        }
        assert_eq!(s.orders.len(), before);
        assert_eq!(m.names(), ["forest.aud", "mchurch.aud"]);
    }

    #[test]
    fn pop_of_the_top_brings_the_one_below_back_from_its_beginning() {
        let (mut m, mut s) = daventry();
        m.apply(&MusicRequest::Push("mwizard.aud".into()), &mut s, &ears(), &mut file);
        steps(&mut s, 10);
        let wiz = id_of(&m, "mwizard.aud").unwrap();
        m.apply(&MusicRequest::Pop("mwizard.aud".into()), &mut s, &ears(), &mut file);
        assert_eq!(m.names(), ["forest.aud"]);
        let forest = id_of(&m, "forest.aud").unwrap();
        assert_ne!(forest, 0, "a new start: from the beginning");
        assert!(matches!(s.orders.last(), Some(Order::Start { looping: true, .. })));
        assert_eq!(s.level(forest), Some(0.0));
        steps(&mut s, 7);
        assert_eq!(s.level(forest), Some(1.0));
        assert!(!s.is_playing(wiz), "the popped one is thrown away at 0");
    }

    #[test]
    fn pop_soon_after_a_push_fades_the_old_one_back_from_where_it_is() {
        let (mut m, mut s) = daventry();
        m.apply(&MusicRequest::Push("municorn.aud".into()), &mut s, &ears(), &mut file);
        steps(&mut s, 3); // forest at 0.55, still fading out
        m.apply(&MusicRequest::Pop("municorn.aud".into()), &mut s, &ears(), &mut file);
        assert_eq!(id_of(&m, "forest.aud"), Some(0), "the same sound: not restarted");
        assert_eq!(s.level(0), Some(0.0), "drops to 0 at once");
        steps(&mut s, 7);
        assert_eq!(s.level(0), Some(1.0));
    }

    #[test]
    fn pop_of_a_lower_one_is_silent_and_cuts_its_fade() {
        let (mut m, mut s) = daventry();
        m.apply(&MusicRequest::Push("mchurch.aud".into()), &mut s, &ears(), &mut file);
        steps(&mut s, 2);
        m.apply(&MusicRequest::Push("mbattle.aud".into()), &mut s, &ears(), &mut file);
        let church = id_of(&m, "mchurch.aud").unwrap();
        assert!(s.is_playing(church), "still fading out");
        let battle = id_of(&m, "mbattle.aud").unwrap();
        let line = m.apply(&MusicRequest::Pop("mchurch.aud".into()), &mut s, &ears(), &mut file);
        assert!(line.contains("rule 57d 4"), "{line}");
        assert!(!s.is_playing(church), "cut off at once");
        assert_eq!(m.names(), ["forest.aud", "mbattle.aud"]);
        assert!(s.is_playing(battle));
    }

    #[test]
    fn pop_of_an_unknown_or_the_only_tune() {
        let (mut m, mut s) = daventry();
        let line = m.apply(&MusicRequest::Pop("mbattle.aud".into()), &mut s, &ears(), &mut file);
        assert!(line.contains("rule 57d 1"), "{line}");
        let line = m.apply(&MusicRequest::Pop("FOREST.AUD".into()), &mut s, &ears(), &mut file);
        assert!(line.contains("rule 57d 3"), "{line}");
        assert!(m.tunes.is_empty());
        steps(&mut s, 7);
        assert_eq!(s.playing_count(), 0, "silence");
        // Pushed onto an empty list: full loudness at once.
        let line = m.apply(&MusicRequest::Push("mbattle.aud".into()), &mut s, &ears(), &mut file);
        assert!(line.contains("rule 57c 4"), "{line}");
        assert_eq!(s.level(id_of(&m, "mbattle.aud").unwrap()), Some(1.0));
    }

    #[test]
    fn play_replaces_the_top_without_a_name_check() {
        let (mut m, mut s) = daventry();
        m.apply(&MusicRequest::Play("forest.aud".into()), &mut s, &ears(), &mut file);
        assert_eq!(m.names(), ["forest.aud"], "the old top left, the same name came back");
        let new = id_of(&m, "forest.aud").unwrap();
        assert_ne!(new, 0);
        assert_eq!(s.level(new), Some(0.0));
        steps(&mut s, 7);
        assert!(!s.is_playing(0) && s.level(new) == Some(1.0));
    }
}
