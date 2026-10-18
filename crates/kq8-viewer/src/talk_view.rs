//! Remarks and speech in the viewer (M4 step 4a, docs/DESIGN.md; rules:
//! docs/behaviour/talking.md).
//!
//! 4a-2: [`load_talk`] reads the message files of the world's `English`
//! folder, then `GAME/ENGLISH` (rule 1: the world's first, our reading of
//! the data), `TALKERS.SH`, and gives the world's rules a way to find a
//! line's voice and lip-sync files next to its message file (rule 4).
//! The queues themselves run in `kq8_core::talk`, inside the door
//! simulation; their lines go to `kq8-artifacts/logs/talk.log`.
//!
//! 4a-3: [`talk_step`] turns the talk events into voices for
//! `sound_view` (group 3; Connor's plain, others placed at the speaker,
//! 2,000 / 30,000) and the ducking switch (groups 0, 2, 5 x 0.2 while a
//! line without `nofade` plays; `sound.md` rules 53, 58).
//!
//! 4a-4: the caption panel ([`setup_caption`], [`caption_draw`]): the
//! line's text (stage directions taken out) at the bottom of the screen
//! while its queue is talking, if `Options::Captions` is true (rule 10).
//! STAND-IN: how the original draws the panel (font, place, wrapping) is
//! not studied (`talking.md` open question 4, M4 GUI): Bevy's own font,
//! white on a dark band, wrapped to 80% of the window.

use bevy::prelude::*;
use kq8_core::talk::{Book, Talk, TalkEvent, Voice, CONNOR};
use kq8_data::msg::{parse_lips, MessageFile, Talkers, Tuple};
use kq8_data::{aud, GameDir};

use crate::door_view::DoorSim;
use crate::log::Log;

/// The message files, talkers and voice lookup for `world`, and the
/// `set` lines of `MASK/Options.cs` (the console variables it sets, e.g.
/// `Options::Captions`, rule 10).
pub fn load_talk(game: &GameDir, world: &str, log: &mut Log) -> (Talk, Vec<(String, String)>) {
    let mut book = Book::default();
    for folder in [format!("{world}/English"), "GAME/ENGLISH".to_string()] {
        let Some(dir) = game.find(&folder) else {
            log.summary(format!("talk: no folder {folder}"));
            continue;
        };
        let rel = dir.strip_prefix(game.root()).unwrap_or(&dir).to_string_lossy().replace('\\', "/");
        let mut names: Vec<_> = std::fs::read_dir(&dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
        names.sort();
        for path in names {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let is_msg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("msg"));
            let Some(number) = stem.parse::<u16>().ok().filter(|_| is_msg) else { continue };
            match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| MessageFile::parse(number, &b)) {
                Ok(m) => {
                    let lines = m.lines.len();
                    let added = book.add(&rel, m, number);
                    log.summary(format!("talk: {rel}/{stem}.MSG: {lines} lines{}", if added { "" } else { " (number already read from another folder: not used)" }));
                }
                Err(e) => log.summary(format!("talk: {rel}/{stem}.MSG: {e}; not used")),
            }
        }
    }
    match game.find("GAME/ENGLISH/TALKERS.SH").map(std::fs::read) {
        Some(Ok(b)) => book.talkers = Talkers::parse(&b),
        _ => log.summary("talk: GAME/ENGLISH/TALKERS.SH not read: every line is spoken by its issuer"),
    }
    log.summary(format!("talk: {} message files, {} talkers", book.files.len(), book.talkers.names.len()));
    let mut talk = Talk::default();
    talk.book = book;
    let g = game.clone();
    talk.voices = Some(Box::new(move |folder: &str, t: Tuple| voice(&g, folder, t)));
    let sets = match game.find("Options.cs").map(std::fs::read) {
        Some(Ok(b)) => options_sets(&b),
        _ => Vec::new(),
    };
    for (k, v) in &sets {
        log.summary(format!("talk: Options.cs sets {k} = '{v}'"));
    }
    (talk, sets)
}

/// A line's voice file and its lip-sync file (rules 4, 17), both next to
/// its message file; `None` without a readable voice file.
fn voice(game: &GameDir, folder: &str, t: Tuple) -> Option<Voice> {
    let path = game.find(&format!("{folder}/{}", t.voice_name()))?;
    let sound = std::fs::read(&path).ok().and_then(|b| aud::parse(&b).ok())?;
    let rel = path.strip_prefix(game.root()).unwrap_or(&path).to_string_lossy().replace('\\', "/");
    let mouths = game.find(&format!("{folder}/{}", t.lip_name())).and_then(|p| std::fs::read(p).ok()).and_then(|b| parse_lips(&b).ok()).unwrap_or_default();
    Some(Voice { path: rel, seconds: sound.seconds(), mouths })
}

/// The `set <name> <value…>` lines of a console script such as
/// `Options.cs` (the value is the rest of the line, words joined by one
/// space, as the script runner's `set`).
pub fn options_sets(bytes: &[u8]) -> Vec<(String, String)> {
    let text: String = bytes.iter().map(|&b| b as char).collect();
    text.lines()
        .filter_map(|line| {
            let mut w = line.split('#').next().unwrap_or("").split_whitespace();
            (w.next()? == "set").then_some(())?;
            let name = w.next()?.to_string();
            Some((name, w.collect::<Vec<_>>().join(" ")))
        })
        .collect()
}

/// What the talk events ask of the sound, the captions and the mouths
/// (M4 4a-3 on), filled each update by [`talk_step`].
#[derive(Resource, Default)]
pub struct Speech {
    /// Voices to start or stop, for `sound_view`.
    pub voices: Vec<VoiceOrder>,
    /// A line that turns other sounds down is playing (rules 10-11).
    pub ducking: bool,
    /// The caption shown (4a-4): the issuer whose queue showed it, and
    /// the text; `None`: the panel is hidden (rule 11).
    pub caption: Option<(String, String)>,
}

/// A voice for the sound system.
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceOrder {
    /// Line `line`'s voice file `path`: plain (`at` `None`, Connor's) or
    /// placed at the speaker (rule 10).
    Start { line: u64, path: String, at: Option<[f32; 3]>, why: String },
    /// Cut short (`clearSpeak`, rule 15).
    Stop { line: u64 },
}

/// After the scripts and the traps of an update: the talk events become
/// orders for the sound (4a-3). Logged to `talk.log` through the door
/// simulation's log lines.
pub fn talk_step(sim: Option<ResMut<DoorSim>>, mut speech: ResMut<Speech>, shared: Res<crate::Shared>) {
    let Some(mut sim) = sim else { return };
    let events = std::mem::take(&mut sim.active.world.talk.events);
    // The update these events belong to (the counter has moved on).
    let now = sim.active.world.updates.saturating_sub(1) as f64 / kq8_core::UPDATES_PER_SECOND;
    for e in events {
        match e.clone() {
            TalkEvent::Start { id, speaker, voice: Some(path), tuple, .. } => {
                // Rule 10: Connor's plain, anyone else's placed where the
                // speaker is when the line starts.
                let at = if speaker.eq_ignore_ascii_case(CONNOR) { None } else { sim.active.world.object_point(&speaker) };
                let line = match (speaker.eq_ignore_ascii_case(CONNOR), at) {
                    (true, _) => format!("line #{id}: voice {path} plain (Connor's)"),
                    (false, Some(p)) => format!("line #{id}: voice {path} placed at {speaker} ({:.0}, {:.0}, {:.0})", p[0], p[1], p[2]),
                    (false, None) => format!("line #{id}: voice {path}: {speaker} has no place: plain (our choice)"),
                };
                sim.active.world.talk.log.push(format!("t {now:8.3}: {line}"));
                speech.voices.push(VoiceOrder::Start { line: id, path, at, why: format!("speech {tuple} line #{id} by {speaker}") });
            }
            TalkEvent::End { id, cut: true, .. } => speech.voices.push(VoiceOrder::Stop { line: id }),
            _ => {}
        }
        // 4a-4, rules 10-11: a line with a caption shows it; the queue
        // that showed it running empty hides it.
        match e {
            TalkEvent::Start { issuer, caption: Some(text), .. } => speech.caption = Some((issuer, text)),
            TalkEvent::Empty { issuer } if speech.caption.as_ref().is_some_and(|(i, _)| *i == issuer) => speech.caption = None,
            _ => {}
        }
    }
    speech.ducking = sim.active.world.talk.ducking();
    sim.flush_talk(&shared);
}

/// The caption panel (4a-4).
#[derive(Component)]
pub struct CaptionPanel;

/// Its text.
#[derive(Component)]
pub struct CaptionText;

/// Spawns the hidden caption panel at the bottom of the window.
pub fn setup_caption(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(10.0),
                width: Val::Percent(80.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Visibility::Hidden,
            CaptionPanel,
        ))
        .with_children(|p| {
            p.spawn((Text::new(""), TextFont { font_size: FontSize::Px(20.0), ..default() }, TextColor(Color::WHITE), CaptionText));
        });
}

/// Shows or hides the caption when it changed; each change in
/// `viewer.log`.
pub fn caption_draw(
    speech: Res<Speech>,
    mut panel: Query<&mut Visibility, With<CaptionPanel>>,
    mut text: Query<&mut Text, With<CaptionText>>,
    shared: Res<crate::Shared>,
) {
    if !speech.is_changed() {
        return;
    }
    let shown = speech.caption.as_ref().map(|(_, t)| t.as_str());
    let (Ok(mut vis), Ok(mut t)) = (panel.single_mut(), text.single_mut()) else { return };
    let was = (*vis != Visibility::Hidden).then(|| t.0.clone());
    if was.as_deref() == shown {
        return;
    }
    match shown {
        Some(s) => {
            t.0 = s.to_string();
            *vis = Visibility::Inherited;
            shared.log.lock().unwrap().detail(format!("caption shown: {s:?}"));
        }
        None => {
            *vis = Visibility::Hidden;
            shared.log.lock().unwrap().detail("caption hidden");
        }
    }
}
