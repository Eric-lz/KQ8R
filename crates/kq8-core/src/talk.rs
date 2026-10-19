//! Remarks and speech (M4 step 4a-2): each character's queue of lines,
//! when a line starts and ends, and what the caller is told (voice,
//! caption, ducking, mouth). No sound card and no drawing here: the
//! viewer acts on [`TalkEvent`]s. The script commands that fill the
//! queues are in `active.rs`.
//!
//! Rules from the behaviour note `docs/behaviour/talking.md` (session A);
//! rule numbers below are that note's. See docs/provenance/README.md.
//!
//! - One queue per character, made the first time it speaks (rule 9). A
//!   `speak` adds lines `<first>`…`<last>` (`<last>` 0: up to the end of
//!   the unbroken run, rule 6) to the end, unless the queue already holds
//!   one with the same file, noun, verb, case and **first** sequence. If
//!   the queue was empty the first line starts at once.
//! - A line starts (rule 10): looked up (missing: dropped, the next one
//!   tried); its speaker is the talker object (a generic talker: the
//!   issuer, rule 5); its voice; the caption if `Options::Captions` is
//!   true; other sounds turned down unless the `speak` had `nofade`; its
//!   mouth changes scheduled (rule 17). It lasts exactly as long as its
//!   voice file.
//! - A line ends (rule 11): the mouth to rest, the caller told (for
//!   `TalkComplete`, rule 13), the next line starts at once; when the
//!   queue is empty the caption goes and the sounds come back up.
//! - `clearSpeak` (rule 15): the queue emptied at once, the line playing
//!   stopped, no `TalkComplete`.
//! - The issuer's single waiting slot for the 0.5 s after turning (rule 7,
//!   "Edge cases"): a later `speak` replaces the slot's lines; each wait
//!   that runs out queues what the slot holds then.
//!
//! Conversations (M4 step 4e-2, `docs/behaviour/conversation.md` rules
//! 12, 21–23): a partner's second queue (`<name>Conv`), issuer the
//! partner. Its lines play as `speak` lines; a line's camera number `[n]`
//! is reported when it starts (`speak` lines report none, rule 14). The
//! goodbye line goes in front and cuts the line playing, which goes back to
//! the head of the waiting lines; when the goodbye ends the queue pauses
//! (the waiting lines stay) and the caller is told (for `TalkExit`). Told
//! to start again while paused with lines waiting, the welcome-back line
//! goes in front and the new lines after the waiting ones. Neither the
//! goodbye nor the welcome back gives `TalkComplete`.
//!
//! "Do not reproduce": every line that starts gets its own number, never
//! used again (the original's "TalkComplete" freeze came from reusing
//! one); a line that can't be played (no voice file, or its talker
//! missing) ends after a time worked out from its text
//! ([`TEXT_SECONDS_PER_CHAR`], at least [`TEXT_MIN_SECONDS`]) instead of
//! leaving the queue stuck. Line time is counted in game time (15 updates
//! per second; our choice: the voice plays in real time, which is the
//! same while the game keeps up).

use std::collections::{BTreeMap, VecDeque};

use kq8_data::msg::{MessageFile, Mouth, Talkers, Tuple, LIP_TICK};

/// The wait between turning to the listener and queuing the lines
/// (rule 7), seconds.
pub const TURN_WAIT: f64 = 0.5;
/// A line without a voice: seconds per character of its text ("Do not
/// reproduce", the note's example).
pub const TEXT_SECONDS_PER_CHAR: f64 = 0.06;
/// ... and at least this long.
pub const TEXT_MIN_SECONDS: f64 = 1.0;
/// Other sounds while a line plays (rule 10, `sound.md` rule 53).
pub const DUCK: f32 = 0.2;
/// Connor's object name (rule 5: talker 4).
pub const CONNOR: &str = "Connor";
/// Rule 19: "'Tis beyond my reach." (a click out of reach).
pub const TOO_FAR: Tuple = Tuple { file: 0, noun: 0, verb: 0, case: 1, seq: 1 };
/// Rule 19: "This will not work." (an item something doesn't take).
pub const WONT_WORK: Tuple = Tuple { file: 0, noun: 0, verb: 0, case: 4, seq: 1 };

/// A line's voice file, found by the caller.
#[derive(Debug, Clone, PartialEq)]
pub struct Voice {
    /// Where it is, from the game folder (`daventry/English/A0RS1J00.001`):
    /// the name the sound system is asked to play.
    pub path: String,
    pub seconds: f64,
    /// Its lip-sync file's mouth changes; empty without one (rule 17).
    pub mouths: Vec<Mouth>,
}

/// Finds a line's voice next to its message file: (message file's
/// folder from the game folder, the line).
pub type VoiceLookup = Box<dyn Fn(&str, Tuple) -> Option<Voice> + Send + Sync>;

/// The message files and talker names (rules 1, 2, 5).
#[derive(Debug, Clone, Default)]
pub struct Book {
    /// By file number: (folder from the game folder, the file). The first
    /// one added for a number wins (the caller adds the world's folder
    /// first, then `GAME/ENGLISH`: rule 1's reading of the data).
    pub files: BTreeMap<u16, (String, MessageFile)>,
    pub talkers: Talkers,
}

impl Book {
    /// Adds message file `number` read from `folder`; `false` if that
    /// number is already there.
    pub fn add(&mut self, folder: &str, file: MessageFile, number: u16) -> bool {
        if self.files.contains_key(&number) {
            return false;
        }
        self.files.insert(number, (folder.to_string(), file));
        true
    }

    pub fn line(&self, t: Tuple) -> Option<(&str, &kq8_data::msg::Line)> {
        let (folder, f) = self.files.get(&t.file)?;
        Some((folder.as_str(), f.find(t)?))
    }

    /// Rule 6, `<last>` 0.
    pub fn last_of_run(&self, t: Tuple) -> Option<u8> {
        self.files.get(&t.file)?.1.last_of_run(t)
    }
}

/// What a line is for (conversation.md rules 20–22).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineKind {
    /// A line of a `speak` or of a conversation's range.
    Normal,
    /// The partner's goodbye: no `TalkComplete`; its end pauses the queue.
    Goodbye,
    /// The partner's welcome back: no `TalkComplete`.
    WelcomeBack,
}

/// One `speak`'s lines, waiting in a queue.
#[derive(Debug, Clone, PartialEq)]
struct Entry {
    tuple: Tuple,
    /// The `speak`'s first line: the duplicate check (rule 9).
    first: Tuple,
    /// The last line of its `speak` (rule 11).
    last: bool,
    nofade: bool,
    kind: LineKind,
}

/// A line playing now.
#[derive(Debug, Clone, PartialEq)]
pub struct Playing {
    /// Its own number, never used again ("Do not reproduce").
    pub id: u64,
    pub tuple: Tuple,
    pub speaker: String,
    pub started: f64,
    pub ends: f64,
    /// The `#` mark (rule 3): the player can't cut it short (rule 16).
    pub no_stop: bool,
    pub duck: bool,
    first: Tuple,
    last: bool,
    pub kind: LineKind,
    /// Its camera number `[n]`, conversation lines only (rule 14).
    pub camera: Option<u32>,
    /// Mouth changes not reached yet, earliest first.
    mouths: VecDeque<Mouth>,
}

impl Playing {
    /// Back into a waiting line (the goodbye cut it, rule 22): it plays
    /// again from its beginning.
    fn entry(&self) -> Entry {
        Entry { tuple: self.tuple, first: self.first, last: self.last, nofade: !self.duck, kind: self.kind }
    }
}

#[derive(Debug, Clone, PartialEq)]
struct Queue {
    issuer: String,
    lines: VecDeque<Entry>,
    playing: Option<Playing>,
    /// A conversation queue (`<name>Conv`), not the `speak` one.
    conv: bool,
    /// Paused by a goodbye (rule 22): the waiting lines stay, none starts.
    paused: bool,
    /// The partner's goodbye and welcome-back lines (rule 20).
    goodbye: Option<Tuple>,
    welcome_back: Option<Tuple>,
}

impl Queue {
    fn new(issuer: &str, conv: bool) -> Self {
        Queue { issuer: issuer.to_string(), lines: VecDeque::new(), playing: None, conv, paused: false, goodbye: None, welcome_back: None }
    }

    /// The queue's name in the logs (`WizardTalk`, `WizardConv`).
    fn label(&self) -> String {
        format!("{}{}", self.issuer, if self.conv { "Conv" } else { "Talk" })
    }
}

/// What a `speak` asks for (rule 6), already checked.
#[derive(Debug, Clone, PartialEq)]
pub struct Speak {
    pub issuer: String,
    pub first: Tuple,
    /// The last sequence (0 already replaced).
    pub last: u8,
    pub nofade: bool,
}

/// What the caller is told.
#[derive(Debug, Clone, PartialEq)]
pub enum TalkEvent {
    /// A line starts: play `voice` (Connor's plain, others placed at the
    /// speaker, rule 10), show `caption`, turn other sounds down if
    /// `duck`.
    /// `camera`: a conversation line's camera number (rule 14); `conv`: a
    /// line of a conversation queue; `kind`: a goodbye or welcome back.
    Start { id: u64, issuer: String, speaker: String, tuple: Tuple, voice: Option<String>, caption: Option<String>, duck: bool, no_stop: bool, camera: Option<u32>, conv: bool, kind: LineKind },
    /// The speaker's mouth changes to `shape` (rule 17).
    Mouth { id: u64, speaker: String, shape: u16 },
    /// A line ended (its voice stopped if `cut`); the mouth goes to rest.
    End { id: u64, issuer: String, speaker: String, cut: bool },
    /// The issuer's queue is empty (rule 11): its caption goes; the sounds
    /// come back up once no line ducks them.
    /// `conv`: the conversation queue, not the `speak` one.
    Empty { issuer: String, conv: bool },
}

/// A line that finished playing (not cut short): its `TalkComplete`.
#[derive(Debug, Clone, PartialEq)]
pub struct Finished {
    pub issuer: String,
    pub tuple: Tuple,
    pub id: u64,
    /// The last line of its `speak`.
    pub last: bool,
    /// A conversation line (issuer the partner).
    pub conv: bool,
    /// A conversation line after which nothing waits in its queue: the
    /// conversation's last line (rule 15).
    pub conv_end: bool,
}

/// What walking away does on the partner's side (conversation.md rule 22).
#[derive(Debug, Clone, PartialEq)]
pub enum Goodbye {
    /// The goodbye started, cutting the line playing; `TalkExit` comes
    /// when it ends ([`Talk::goodbyes_ended`]).
    Started,
    /// No goodbye (`why`): the line playing plays on to its end.
    None(String),
}

/// Every character's queue.
#[derive(Default)]
pub struct Talk {
    pub book: Book,
    pub voices: Option<VoiceLookup>,
    queues: Vec<Queue>,
    /// Each issuer's waiting slot (rule 7), by lower-case name.
    slots: BTreeMap<String, Speak>,
    /// Waits running out: (game time, lower-case issuer).
    waits: Vec<(f64, String)>,
    next_id: u64,
    /// Partners whose goodbye ended in the last update (rule 22: their
    /// script gets `TalkExit`), taken by the caller.
    pub goodbyes_ended: Vec<String>,
    /// For the caller (the viewer), taken each update.
    pub events: Vec<TalkEvent>,
    /// Lines for `talk.log`, taken by the caller.
    pub log: Vec<String>,
}

fn same(a: &str, b: &str) -> bool {
    a.eq_ignore_ascii_case(b)
}

/// How long a line without a voice lasts ("Do not reproduce").
pub fn text_seconds(text: &str) -> f64 {
    (text.chars().count() as f64 * TEXT_SECONDS_PER_CHAR).max(TEXT_MIN_SECONDS)
}

impl Talk {
    fn note(&mut self, now: f64, line: String) {
        self.log.push(format!("t {now:8.3}: {line}"));
    }

    /// The `speak` queue of `issuer`.
    fn queue_index(&self, issuer: &str) -> Option<usize> {
        self.queues.iter().position(|q| !q.conv && same(&q.issuer, issuer))
    }

    /// The conversation queue of `partner`.
    fn conv_index(&self, partner: &str) -> Option<usize> {
        self.queues.iter().position(|q| q.conv && same(&q.issuer, partner))
    }

    /// Whether `issuer` is saying something or has lines waiting.
    pub fn is_talking(&self, issuer: &str) -> bool {
        self.queue_index(issuer).is_some_and(|i| self.queues[i].playing.is_some() || !self.queues[i].lines.is_empty())
    }

    /// The line `issuer` is saying now.
    pub fn playing(&self, issuer: &str) -> Option<&Playing> {
        self.queue_index(issuer).and_then(|i| self.queues[i].playing.as_ref())
    }

    /// Any line playing that turns the other sounds down (rule 11: on
    /// between the lines of one queue, as the next starts at once).
    pub fn ducking(&self) -> bool {
        self.queues.iter().any(|q| q.playing.as_ref().is_some_and(|p| p.duck))
    }

    /// Rule 9: adds a `speak`'s lines to the issuer's queue, unless it is
    /// a duplicate. Returns a line for the log. The line starts in the
    /// next [`Talk::update`] (same game time: "at once").
    pub fn queue(&mut self, s: &Speak, now: f64) -> String {
        let i = match self.queue_index(&s.issuer) {
            Some(i) => i,
            None => {
                self.queues.push(Queue::new(&s.issuer, false));
                self.note(now, format!("{}: queue made ({}Talk)", s.issuer, s.issuer));
                self.queues.len() - 1
            }
        };
        self.add_lines(i, s, now)
    }

    /// Rule 9's adding, to queue `i`.
    fn add_lines(&mut self, i: usize, s: &Speak, now: f64) -> String {
        let label = self.queues[i].label();
        let q = &self.queues[i];
        let first = s.first;
        // Rule 9 (corrected 2026-10-19, Q114): the check is on the range's
        // last line (`<last>`, 0 already replaced), waiting or playing.
        let last_line = first.with_seq(s.last);
        if q.playing.as_ref().is_some_and(|p| p.tuple == last_line) || q.lines.iter().any(|e| e.tuple == last_line) {
            let line = format!("{label}: {first}…{}: duplicate: already in the queue; not queued (rule 9)", s.last);
            self.note(now, line.clone());
            return line;
        }
        if s.last < first.seq {
            let line = format!("{label}: {first}…{}: last before first; nothing queued", s.last);
            self.note(now, line.clone());
            return line;
        }
        let was_empty = q.playing.is_none() && q.lines.is_empty();
        let q = &mut self.queues[i];
        for seq in first.seq..=s.last {
            q.lines.push_back(Entry { tuple: first.with_seq(seq), first, last: seq == s.last, nofade: s.nofade, kind: LineKind::Normal });
        }
        let waiting = q.lines.len();
        let line = format!(
            "{label}: queued {first}…{} ({} line(s){}); {}",
            s.last,
            s.last - first.seq + 1,
            if s.nofade { ", nofade" } else { "" },
            if was_empty { "the queue was empty: starts at once".to_string() } else { format!("waits ({waiting} waiting)") }
        );
        self.note(now, line.clone());
        line
    }

    /// Rule 7: the lines go to the issuer's single waiting slot and are
    /// queued [`TURN_WAIT`] later.
    pub fn queue_later(&mut self, s: Speak, now: f64) -> String {
        let key = s.issuer.to_ascii_lowercase();
        let replaced = self.slots.insert(key.clone(), s.clone());
        self.waits.push((now + TURN_WAIT, key));
        let line = format!(
            "{}: {}…{} waits {TURN_WAIT} s{}",
            s.issuer,
            s.first,
            s.last,
            replaced.map_or(String::new(), |r| format!(" (replaces {}…{} in the issuer's one waiting slot: both waits queue this)", r.first, r.last))
        );
        self.note(now, line.clone());
        line
    }

    /// Rule 2: the partner's conversation queue (`<name>Conv`), made the
    /// first time, with its goodbye and welcome-back lines (rule 20).
    /// Returns a line for the log.
    pub fn conv_queue(&mut self, partner: &str, goodbye: Option<Tuple>, welcome_back: Option<Tuple>, now: f64) -> String {
        let show = |t: Option<Tuple>| t.map_or("none".to_string(), |t| t.to_string());
        let line = match self.conv_index(partner) {
            Some(i) => {
                let q = &mut self.queues[i];
                q.goodbye = goodbye;
                q.welcome_back = welcome_back;
                format!("{}: conversation queue already made ({} waiting{})", q.label(), q.lines.len(), if q.paused { ", paused by a goodbye" } else { "" })
            }
            None => {
                let mut q = Queue::new(partner, true);
                q.goodbye = goodbye;
                q.welcome_back = welcome_back;
                let line = format!("{}: queue made; goodbye {}, welcome back {} (conversation.md rules 2, 20)", q.label(), show(goodbye), show(welcome_back));
                self.queues.push(q);
                line
            }
        };
        self.note(now, line.clone());
        line
    }

    /// Rule 21: the partner is told to start: the conversation's lines go
    /// into its conversation queue. A goodbye asked for and not played yet
    /// is dropped first; a queue paused by a goodbye with lines still
    /// waiting gets the welcome-back line in front, the new lines after the
    /// waiting ones (with the duplicate check). Returns a line for the log.
    pub fn conv_start(&mut self, s: &Speak, now: f64) -> String {
        let i = match self.conv_index(&s.issuer) {
            Some(i) => i,
            None => {
                self.conv_queue(&s.issuer, None, None, now);
                self.queues.len() - 1
            }
        };
        let label = self.queues[i].label();
        let q = &mut self.queues[i];
        let mut notes = Vec::new();
        let before = q.lines.len();
        q.lines.retain(|e| e.kind != LineKind::Goodbye);
        if q.lines.len() < before {
            notes.push("a goodbye not played yet dropped (rule 21)".to_string());
        }
        if std::mem::take(&mut q.paused) {
            match (q.lines.is_empty(), q.welcome_back) {
                (false, Some(wb)) => {
                    q.lines.push_front(Entry { tuple: wb, first: wb, last: false, nofade: false, kind: LineKind::WelcomeBack });
                    notes.push(format!("paused by a goodbye with {} line(s) waiting: welcome back {wb} in front, the waiting lines from the cut one on, then the new ones (rule 21)", q.lines.len() - 1));
                }
                (false, None) => notes.push(format!("paused by a goodbye with {} line(s) waiting; no welcome-back line: they play on", q.lines.len())),
                (true, _) => notes.push("paused by a goodbye with nothing waiting".to_string()),
            }
        }
        let added = self.add_lines(i, s, now);
        let line = format!("{label}: the partner is told to start (rule 21){}: {added}", if notes.is_empty() { String::new() } else { format!(": {}", notes.join("; ")) });
        self.note(now, line.clone());
        line
    }

    /// Rule 22: Connor walked away. With more than one line left (the one
    /// playing and at least one waiting) and a goodbye line, the goodbye
    /// starts at once, cutting the line playing (back to the head of the
    /// waiting lines).
    pub fn conv_goodbye(&mut self, partner: &str, now: f64, captions: bool, exists: &dyn Fn(&str) -> bool) -> Goodbye {
        let Some(i) = self.conv_index(partner) else {
            return Goodbye::None("no conversation queue".into());
        };
        let q = &mut self.queues[i];
        let label = q.label();
        let left = usize::from(q.playing.is_some()) + q.lines.len();
        let why = if q.paused {
            Some("already paused by a goodbye".to_string())
        } else if left <= 1 {
            Some(format!("{left} line(s) left: the last line plays to its end (rule 22)"))
        } else if q.goodbye.is_none() {
            Some("the partner has no goodbye line (rule 22)".to_string())
        } else {
            None
        };
        if let Some(why) = why {
            self.note(now, format!("{label}: Connor walked away: no goodbye: {why}"));
            return Goodbye::None(why);
        }
        let gb = q.goodbye.expect("checked above");
        let cut = q.playing.take().inspect(|p| q.lines.push_front(p.entry()));
        if let Some(p) = &cut {
            self.events.push(TalkEvent::End { id: p.id, issuer: partner.to_string(), speaker: p.speaker.clone(), cut: true });
        }
        self.note(
            now,
            format!(
                "{label}: Connor walked away with {left} line(s) left: goodbye {gb} in front, starts at once{} (rule 22)",
                cut.map_or(String::new(), |p| format!(", cutting line #{} {} at {:.3} of {:.3} s (back to the head of the waiting lines; no TalkComplete for it)", p.id, p.tuple, now - p.started, p.ends - p.started))
            ),
        );
        self.start(i, Entry { tuple: gb, first: gb, last: false, nofade: false, kind: LineKind::Goodbye }, now, captions, exists);
        if self.queues[i].playing.is_none() {
            // Our choice: a goodbye that can't be played ends at once.
            self.queues[i].paused = true;
            self.goodbyes_ended.push(self.queues[i].issuer.clone());
            self.note(now, format!("{label}: the goodbye can't be played: the queue pauses at once (our choice)"));
        }
        Goodbye::Started
    }

    /// The partner's conversation queue is paused by a goodbye.
    pub fn conv_paused(&self, partner: &str) -> bool {
        self.conv_index(partner).is_some_and(|i| self.queues[i].paused)
    }

    /// Lines of the partner's conversation queue: playing and waiting.
    pub fn conv_left(&self, partner: &str) -> usize {
        self.conv_index(partner).map_or(0, |i| usize::from(self.queues[i].playing.is_some()) + self.queues[i].lines.len())
    }

    /// The line the partner's conversation queue is playing.
    pub fn conv_playing(&self, partner: &str) -> Option<&Playing> {
        self.conv_index(partner).and_then(|i| self.queues[i].playing.as_ref())
    }

    /// `KQMonster::clearSpeak <partner>` on the conversation queue (rule
    /// 23): emptied at once, no `TalkComplete`, the pause forgotten.
    /// `None` when the partner has no conversation queue.
    pub fn clear_conv(&mut self, partner: &str, now: f64) -> Option<String> {
        let i = self.conv_index(partner)?;
        let q = &mut self.queues[i];
        let label = q.label();
        let dropped = q.lines.len();
        q.lines.clear();
        let paused = std::mem::take(&mut q.paused);
        let playing = q.playing.take();
        let issuer = q.issuer.clone();
        let line = match playing {
            Some(p) => {
                self.events.push(TalkEvent::End { id: p.id, issuer: issuer.clone(), speaker: p.speaker.clone(), cut: true });
                self.events.push(TalkEvent::Empty { issuer, conv: true });
                format!("{label}: clearSpeak: line #{} {} stopped, {dropped} waiting dropped; no TalkComplete; caption off, sounds up", p.id, p.tuple)
            }
            None if dropped > 0 => {
                self.events.push(TalkEvent::Empty { issuer, conv: true });
                format!("{label}: clearSpeak: {dropped} waiting dropped{}; no TalkComplete", if paused { " (it was paused by a goodbye)" } else { "" })
            }
            None => format!("{label}: clearSpeak: already empty"),
        };
        self.note(now, line.clone());
        Some(line)
    }

    /// Rule 15: empties `issuer`'s queue at once; no `TalkComplete`.
    pub fn clear(&mut self, issuer: &str, now: f64) -> String {
        let Some(i) = self.queue_index(issuer) else {
            let line = format!("{issuer}: clearSpeak: no queue; nothing");
            self.note(now, line.clone());
            return line;
        };
        let q = &mut self.queues[i];
        let dropped = q.lines.len();
        q.lines.clear();
        let playing = q.playing.take();
        let issuer = q.issuer.clone();
        let line = match playing {
            Some(p) => {
                self.events.push(TalkEvent::End { id: p.id, issuer: issuer.clone(), speaker: p.speaker.clone(), cut: true });
                self.events.push(TalkEvent::Empty { issuer: issuer.clone(), conv: false });
                format!("{issuer}: clearSpeak: line #{} {} stopped at {:.3} of {:.3} s, {dropped} waiting dropped; no TalkComplete; caption off, mouth at rest", p.id, p.tuple, now - p.started, p.ends - p.started)
            }
            None if dropped > 0 => {
                self.events.push(TalkEvent::Empty { issuer: issuer.clone(), conv: false });
                format!("{issuer}: clearSpeak: {dropped} waiting dropped; no TalkComplete")
            }
            None => format!("{issuer}: clearSpeak: queue already empty"),
        };
        let waits = self.waits.iter().filter(|(_, k)| same(k, &issuer)).count();
        let line = if waits > 0 { format!("{line}; {waits} wait(s) after a turn still pending (kept: not in the queue yet, not traced)") } else { line };
        self.note(now, line.clone());
        line
    }

    /// Rule 16: the player pressed a movement key, a turn key or Jump, or
    /// clicked in the world (`why`): Connor's line stops as `clearSpeak
    /// Connor` would, unless it has the "no stop" mark (`#`). Nothing
    /// when he says nothing. Not called during Connor's talk state
    /// (conversation.md rule 12: the caller checks).
    pub fn cut_short(&mut self, why: &str, now: f64) -> Option<String> {
        let p = self.playing(CONNOR)?;
        let line = if p.no_stop {
            format!("Connor: {why}: line #{} {} has '#': not cut short (rule 16)", p.id, p.tuple)
        } else {
            let id = p.id;
            format!("Connor: {why}: line #{id} cut short (rule 16): {}", self.clear(CONNOR, now))
        };
        self.note(now, line.clone());
        Some(line)
    }

    /// One update at game time `now`: waits that ran out queue their slot;
    /// lines whose time is up end and the next start; mouth changes due
    /// are sent. `exists`: whether an object of that name is in the world.
    /// Returns the lines that finished (for `TalkComplete`).
    pub fn update(&mut self, now: f64, captions: bool, exists: &dyn Fn(&str) -> bool) -> Vec<Finished> {
        // Waits after a turn (rule 7).
        let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut self.waits).into_iter().partition(|(t, _)| *t <= now + 1e-9);
        self.waits = later;
        for (_, key) in due {
            if let Some(s) = self.slots.get(&key).cloned() {
                self.note(now, format!("{}: the wait after the turn is over", s.issuer));
                self.queue(&s, now);
            }
            if !self.waits.iter().any(|(_, k)| *k == key) {
                self.slots.remove(&key);
            }
        }
        let mut finished = Vec::new();
        for i in 0..self.queues.len() {
            // A line ends, the next starts at once; a line too short for
            // one update (or a dropped one) lets the next try.
            let mut ended_here = false;
            loop {
                self.mouths_due(i, now);
                if let Some(p) = &self.queues[i].playing {
                    if p.ends > now + 1e-9 {
                        break;
                    }
                    let p = self.queues[i].playing.take().unwrap();
                    let issuer = self.queues[i].issuer.clone();
                    let label = self.queues[i].label();
                    ended_here = true;
                    self.events.push(TalkEvent::End { id: p.id, issuer: issuer.clone(), speaker: p.speaker.clone(), cut: false });
                    match p.kind {
                        LineKind::Goodbye => {
                            // conversation.md rule 22: the queue pauses, the
                            // waiting lines stay; TalkExit, no TalkComplete.
                            self.queues[i].paused = true;
                            self.goodbyes_ended.push(issuer.clone());
                            self.events.push(TalkEvent::Empty { issuer, conv: true });
                            let waiting = self.queues[i].lines.len();
                            self.note(now, format!("{label}: goodbye #{} {} finished: caption off, sounds back up, the queue pauses with {waiting} line(s) waiting; TalkExit to the partner's script; no TalkComplete (rule 22)", p.id, p.tuple));
                            break;
                        }
                        LineKind::WelcomeBack => {
                            self.note(now, format!("{label}: welcome back #{} {} finished: no TalkComplete (rule 22)", p.id, p.tuple));
                        }
                        LineKind::Normal => {
                            let conv = self.queues[i].conv;
                            let conv_end = conv && self.queues[i].lines.is_empty();
                            self.note(
                                now,
                                format!(
                                    "{label}: line #{} {} finished ({:.3} s){}{}; mouth at rest",
                                    p.id,
                                    p.tuple,
                                    p.ends - p.started,
                                    if p.last { ", the last of its range" } else { "" },
                                    if conv_end { ", the conversation's last line" } else { "" }
                                ),
                            );
                            finished.push(Finished { issuer, tuple: p.tuple, id: p.id, last: p.last, conv, conv_end });
                        }
                    }
                }
                if self.queues[i].paused {
                    break;
                }
                let Some(e) = self.queues[i].lines.pop_front() else {
                    if ended_here {
                        let issuer = self.queues[i].issuer.clone();
                        let label = self.queues[i].label();
                        let conv = self.queues[i].conv;
                        self.events.push(TalkEvent::Empty { issuer, conv });
                        self.note(now, format!("{label}: queue empty: caption off, sounds back up{}", if self.ducking() { " once no other line ducks them" } else { "" }));
                    }
                    break;
                };
                self.start(i, e, now, captions, exists);
            }
        }
        finished
    }

    fn mouths_due(&mut self, i: usize, now: f64) {
        let Some(p) = &mut self.queues[i].playing else { return };
        let mut changes = Vec::new();
        while let Some(m) = p.mouths.front().filter(|m| p.started + m.time as f64 * LIP_TICK <= now + 1e-9) {
            changes.push(*m);
            p.mouths.pop_front();
        }
        let (id, speaker) = (p.id, p.speaker.clone());
        for m in changes {
            self.events.push(TalkEvent::Mouth { id, speaker: speaker.clone(), shape: m.shape });
            let odd = match m.shape {
                0..=7 => "",
                8..=16 => " (a gesture code, not a mouth shape: talking.md rule 17, conversation.md rule 18)",
                _ => " (above 16: nothing, talking.md rule 17)",
            };
            self.note(now, format!("{speaker}: line #{id}: mouth shape {}{odd} (due at {:.3} s, {} / 60)", m.shape, m.time as f64 * LIP_TICK, m.time));
        }
    }

    /// Rule 10: line `e` of queue `i` starts.
    fn start(&mut self, i: usize, e: Entry, now: f64, captions: bool, exists: &dyn Fn(&str) -> bool) {
        let issuer = self.queues[i].issuer.clone();
        let (label, conv) = (self.queues[i].label(), self.queues[i].conv);
        let Some((folder, line)) = self.book.line(e.tuple) else {
            self.note(now, format!("{label}: line {} is not in its message file: dropped, the next one tried (rule 10)", e.tuple));
            return;
        };
        let folder = folder.to_string();
        let shown = line.shown();
        let talker = line.talker;
        let (speaker, whose) = match self.book.talkers.name(talker) {
            Some(_) if self.book.talkers.is_generic(talker) => (issuer.clone(), "generic talker: the issuer".to_string()),
            Some(n) => (n.to_string(), format!("talker {talker}")),
            None => (issuer.clone(), format!("talker {talker} not in TALKERS.SH: the issuer (our choice)")),
        };
        let speaker_there = same(&speaker, CONNOR) || exists(&speaker);
        let voice = self.voices.as_ref().and_then(|v| v(&folder, e.tuple));
        let (seconds, how) = match (&voice, speaker_there) {
            (Some(v), true) => (v.seconds, format!("voice {} ({:.3} s)", v.path, v.seconds)),
            (Some(v), false) => (text_seconds(&shown.text), format!("speaker {speaker} not in the world: voice {} not played; timed by its text (Do not reproduce)", v.path)),
            (None, _) => (text_seconds(&shown.text), format!("no voice file {} next to {folder}: timed by its text (Do not reproduce)", e.tuple.voice_name())),
        };
        let id = self.next_id;
        self.next_id += 1;
        let duck = !e.nofade;
        let mouths: VecDeque<Mouth> = voice.as_ref().filter(|_| speaker_there).map(|v| v.mouths.iter().copied().collect()).unwrap_or_default();
        let caption = captions.then(|| shown.text.clone());
        // conversation.md rule 14: camera numbers only in conversations.
        let camera = shown.camera.filter(|_| conv);
        let what = match e.kind {
            LineKind::Normal => "line",
            LineKind::Goodbye => "goodbye",
            LineKind::WelcomeBack => "welcome back",
        };
        let cam = match (shown.camera, conv) {
            (Some(n), true) => format!("; camera {n}"),
            (Some(n), false) => format!("; camera {n} ignored (not a conversation, rule 14)"),
            (None, _) => String::new(),
        };
        self.note(
            now,
            format!(
                "{label}: {what} #{id} {} starts: speaker {speaker} ({whose}), {how}, lasts {seconds:.3} s; {}; {}; '#' {}; {} mouth change(s); caption {:?}{cam}",
                e.tuple,
                if same(&speaker, CONNOR) { "plain" } else { "placed at the speaker, 2000 / 30000" },
                if duck { "other sounds x 0.2" } else { "nofade: other sounds kept" },
                if shown.no_stop { "yes: can't be cut short" } else { "no" },
                mouths.len(),
                caption.as_deref().unwrap_or("(captions off)")
            ),
        );
        self.events.push(TalkEvent::Start {
            id,
            issuer: issuer.clone(),
            speaker: speaker.clone(),
            tuple: e.tuple,
            voice: voice.filter(|_| speaker_there).map(|v| v.path),
            caption,
            duck,
            no_stop: shown.no_stop,
            camera,
            conv,
            kind: e.kind,
        });
        self.queues[i].playing = Some(Playing {
            id,
            tuple: e.tuple,
            speaker,
            started: now,
            ends: now + seconds,
            no_stop: shown.no_stop,
            duck,
            first: e.first,
            last: e.last,
            kind: e.kind,
            camera,
            mouths,
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use kq8_data::msg::{Line, MessageFile};

    fn line(noun: u8, seq: u8, talker: u8, raw: &str) -> Line {
        Line { tuple: Tuple::new(1000, noun, 0, 0, seq), talker, raw: raw.into() }
    }

    /// Made-up lines (never the game's): noun 55 has two lines, noun 7 one
    /// said by the Wizard (talker 9), noun 8 by a generic talker (40).
    fn talk() -> Talk {
        let mut book = Book::default();
        let file = MessageFile {
            kind: 0x0F,
            version: 5010,
            last_id: 0,
            lines: vec![line(55, 1, 4, "([#]) One."), line(55, 2, 4, "Two."), line(7, 1, 9, "Wizard."), line(8, 1, 40, "Gnome.")],
        };
        book.add("daventry/English", file, 1000);
        book.talkers = Talkers::parse(b"(define CONNOR 4)\n(define WIZARD 9)\n(define GEN_GNOME1 40)\n");
        let mut t = Talk { book, ..Default::default() };
        t.voices = Some(Box::new(|folder, tuple| {
            (tuple.noun != 7).then(|| Voice { path: format!("{folder}/{}", tuple.voice_name()), seconds: 0.4, mouths: vec![Mouth { time: 6, shape: 3 }] })
        }));
        t
    }

    fn speak(issuer: &str, noun: u8, first: u8, last: u8) -> Speak {
        Speak { issuer: issuer.into(), first: Tuple::new(1000, noun, 0, 0, first), last, nofade: false }
    }

    fn run(t: &mut Talk, from: u64, to: u64) -> Vec<Finished> {
        (from..to).flat_map(|u| t.update(u as f64 / 15.0, true, &|n| n == "Wizard")).collect()
    }

    #[test]
    fn lines_play_in_turn_for_their_voice_length_and_duplicates_are_dropped() {
        let mut t = talk();
        t.queue(&speak("Connor", 55, 1, 2), 0.0);
        assert!(t.queue(&speak("Connor", 55, 1, 2), 0.0).contains("duplicate"));
        let f = run(&mut t, 0, 7);
        // 0.4 s = 6 updates: line 1 ends at update 6, line 2 starts then.
        assert_eq!(f.iter().map(|f| (f.tuple.seq, f.last)).collect::<Vec<_>>(), [(1, false)]);
        let starts: Vec<_> = t.events.iter().filter_map(|e| if let TalkEvent::Start { tuple, caption, no_stop, .. } = e { Some((tuple.seq, caption.clone(), *no_stop)) } else { None }).collect();
        assert_eq!(starts, [(1, Some(" One.".into()), true), (2, Some("Two.".into()), false)]);
        assert!(t.events.iter().any(|e| matches!(e, TalkEvent::Mouth { shape: 3, .. })));
        assert!(t.ducking());
        // Line 2 (the range's last) is playing: a duplicate (rule 9 as
        // corrected, Q114: the check is on the last line).
        assert!(t.queue(&speak("Connor", 55, 1, 2), 0.5).contains("duplicate"));
        let f = run(&mut t, 7, 13);
        assert_eq!(f.iter().map(|f| (f.tuple.seq, f.last)).collect::<Vec<_>>(), [(2, true)]);
        assert!(matches!(t.events.last(), Some(TalkEvent::Empty { .. })));
        assert!(!t.ducking() && !t.is_talking("connor"));
        // Finished: the same remark is queued again.
        assert!(t.queue(&speak("Connor", 55, 1, 2), 1.0).contains("starts at once"));
    }

    #[test]
    fn the_duplicate_check_is_on_the_range_s_last_line() {
        // Q114: during line 1 of 1…2, line 2 still waits: 1…2 again adds
        // nothing, but 1…1 (its last, line 1, is playing) adds nothing
        // either, while 2…2 (line 2 waiting) is a duplicate too.
        let mut t = talk();
        t.queue(&speak("Connor", 55, 1, 2), 0.0);
        run(&mut t, 0, 1);
        assert!(t.queue(&speak("Connor", 55, 1, 2), 0.1).contains("duplicate"));
        assert!(t.queue(&speak("Connor", 55, 1, 1), 0.1).contains("duplicate"));
        assert!(t.queue(&speak("Connor", 55, 2, 2), 0.1).contains("duplicate"));
    }

    #[test]
    fn talkers_speakers_and_lines_that_cant_be_played() {
        let mut t = talk();
        // The Wizard's line has no voice: timed by its text, at least 1 s.
        t.queue(&speak("Connor", 7, 1, 1), 0.0);
        run(&mut t, 0, 1);
        assert!(matches!(&t.events[0], TalkEvent::Start { speaker, voice: None, .. } if speaker == "Wizard"));
        assert_eq!(run(&mut t, 1, 15).len(), 0);
        assert_eq!(run(&mut t, 15, 16).len(), 1, "ends after 1 s");
        // A generic talker is spoken by the issuer.
        t.events.clear();
        t.queue(&speak("Gnomey", 8, 1, 1), 2.0);
        t.update(2.0, false, &|_| false);
        assert!(matches!(&t.events[0], TalkEvent::Start { speaker, caption: None, voice: None, .. } if speaker == "Gnomey"), "{:?}", t.events);
        // A line missing from the file is dropped, the next tried.
        t.events.clear();
        t.queue(&speak("Connor", 99, 1, 1), 3.0);
        t.queue(&speak("Connor", 55, 2, 2), 3.0);
        t.update(3.0, true, &|_| false);
        assert!(matches!(&t.events[0], TalkEvent::Start { tuple, .. } if tuple.noun == 55));
    }

    #[test]
    fn clear_speak_stops_everything_without_talk_complete() {
        let mut t = talk();
        t.queue(&speak("Connor", 55, 1, 2), 0.0);
        run(&mut t, 0, 2);
        t.clear("connor", 2.0 / 15.0);
        assert!(t.events.iter().any(|e| matches!(e, TalkEvent::End { cut: true, .. })));
        assert!(run(&mut t, 2, 30).is_empty());
        assert!(!t.is_talking("Connor"));
    }

    #[test]
    fn only_lines_without_the_mark_are_cut_short() {
        let mut t = talk();
        // 55/1 has '#', 55/2 not.
        t.queue(&speak("Connor", 55, 1, 2), 0.0);
        run(&mut t, 0, 1);
        assert!(t.cut_short("forward pressed", 0.0).unwrap().contains("not cut short"));
        run(&mut t, 1, 7);
        assert!(t.cut_short("click", 0.4).unwrap().contains("cut short (rule 16)"));
        assert!(t.events.iter().any(|e| matches!(e, TalkEvent::End { cut: true, .. })));
        assert!(run(&mut t, 7, 20).is_empty(), "no TalkComplete");
        assert_eq!(t.cut_short("click", 2.0), None, "nothing to cut");
    }

    /// A conversation (made-up lines): noun 60 has three lines with
    /// cameras 2, none, 5; noun 61 case 7 is the goodbye, case 8 the
    /// welcome back. Every line 0.4 s (6 updates).
    fn conv_talk() -> Talk {
        let mut t = talk();
        let c = |noun: u8, case: u8, seq: u8, raw: &str| Line { tuple: Tuple::new(1000, noun, 0, case, seq), talker: 9, raw: raw.into() };
        let f = &mut t.book.files.get_mut(&1000).unwrap().1;
        f.lines.extend([c(60, 0, 1, "([2]) A."), c(60, 0, 2, "B."), c(60, 0, 3, "([5]) C."), c(61, 7, 1, "Bye."), c(61, 8, 1, "Back.")]);
        t.voices = Some(Box::new(|folder, tuple| Some(Voice { path: format!("{folder}/{}", tuple.voice_name()), seconds: 0.4, mouths: vec![] })));
        t.conv_queue("Wizard", Some(Tuple::new(1000, 61, 0, 7, 1)), Some(Tuple::new(1000, 61, 0, 8, 1)), 0.0);
        t
    }

    fn starts(t: &Talk) -> Vec<(u8, u8, Option<u32>, LineKind)> {
        t.events.iter().filter_map(|e| if let TalkEvent::Start { tuple, camera, kind, .. } = e { Some((tuple.case, tuple.seq, *camera, *kind)) } else { None }).collect()
    }

    #[test]
    fn a_conversation_plays_its_lines_with_cameras_and_reports_its_end() {
        let mut t = conv_talk();
        assert!(t.conv_start(&speak("Wizard", 60, 1, 3), 0.0).contains("starts at once"));
        let f = run(&mut t, 0, 20);
        assert_eq!(starts(&t), [(0, 1, Some(2), LineKind::Normal), (0, 2, None, LineKind::Normal), (0, 3, Some(5), LineKind::Normal)]);
        assert_eq!(f.iter().map(|f| (f.tuple.seq, f.conv, f.conv_end)).collect::<Vec<_>>(), [(1, true, false), (2, true, false), (3, true, true)]);
        assert!(f.iter().all(|f| f.issuer == "Wizard"), "the partner is the issuer");
        // The speak queue is separate: a speak line has no camera.
        t.events.clear();
        t.queue(&speak("Wizard", 60, 1, 1), 2.0);
        t.update(2.0, true, &|_| true);
        assert_eq!(starts(&t), [(0, 1, None, LineKind::Normal)]);
    }

    #[test]
    fn walking_away_plays_the_goodbye_and_coming_back_the_welcome_back() {
        let mut t = conv_talk();
        t.conv_start(&speak("Wizard", 60, 1, 3), 0.0);
        run(&mut t, 0, 8); // line 2 playing since update 6
        let now = 8.0 / 15.0;
        assert_eq!(t.conv_goodbye("Wizard", now, true, &|_| true), Goodbye::Started);
        assert!(t.events.iter().any(|e| matches!(e, TalkEvent::End { cut: true, .. })), "line 2 cut");
        assert_eq!(t.conv_playing("Wizard").map(|p| p.kind), Some(LineKind::Goodbye));
        let f = run(&mut t, 9, 30);
        assert!(f.is_empty(), "no TalkComplete for the cut line or the goodbye");
        assert_eq!(t.goodbyes_ended, ["Wizard"]);
        assert!(t.conv_paused("Wizard") && t.conv_left("Wizard") == 2, "lines 2 and 3 wait");
        // Told to start again (a duplicate range): welcome back, line 2
        // from its start, line 3; not added twice.
        t.events.clear();
        let line = t.conv_start(&speak("Wizard", 60, 1, 3), 3.0);
        assert!(line.contains("welcome back") && line.contains("duplicate"), "{line}");
        let f = run(&mut t, 45, 70);
        assert_eq!(starts(&t), [(8, 1, None, LineKind::WelcomeBack), (0, 2, None, LineKind::Normal), (0, 3, Some(5), LineKind::Normal)]);
        assert_eq!(f.iter().map(|f| (f.tuple.seq, f.conv_end)).collect::<Vec<_>>(), [(2, false), (3, true)]);
    }

    #[test]
    fn no_goodbye_on_the_last_line_and_clear_speak_empties_the_conversation() {
        let mut t = conv_talk();
        t.conv_start(&speak("Wizard", 60, 1, 3), 0.0);
        run(&mut t, 0, 14); // line 3 playing
        assert!(matches!(t.conv_goodbye("Wizard", 14.0 / 15.0, true, &|_| true), Goodbye::None(w) if w.contains("1 line(s) left")));
        let f = run(&mut t, 14, 20);
        assert_eq!(f.iter().map(|f| f.tuple.seq).collect::<Vec<_>>(), [3], "plays to its end with its TalkComplete");
        // clearSpeak on the partner: everything goes, no TalkComplete.
        t.conv_start(&speak("Wizard", 60, 1, 3), 2.0);
        run(&mut t, 30, 32);
        assert!(t.clear_conv("Wizard", 32.0 / 15.0).unwrap().contains("stopped"));
        assert!(run(&mut t, 32, 60).is_empty() && t.conv_left("Wizard") == 0);
        assert!(t.clear_conv("Nobody", 4.0).is_none());
    }

    #[test]
    fn the_wait_after_turning_has_one_slot() {
        let mut t = talk();
        t.queue_later(speak("Connor", 55, 1, 1), 0.0);
        t.queue_later(speak("Connor", 55, 2, 2), 0.1);
        assert!(run(&mut t, 0, 8).is_empty() && !t.events.iter().any(|e| matches!(e, TalkEvent::Start { .. })));
        // 0.5 s: update 8 (0.533 s); the second wait at update 9 (0.6 s).
        run(&mut t, 8, 10);
        // Both waits queue the later lines; the second is a duplicate.
        let starts: Vec<_> = t.events.iter().filter_map(|e| if let TalkEvent::Start { tuple, .. } = e { Some(tuple.seq) } else { None }).collect();
        assert_eq!(starts, [2]);
        assert!(t.log.iter().any(|l| l.contains("duplicate")));
    }
}
