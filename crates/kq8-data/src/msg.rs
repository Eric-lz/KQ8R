//! Message files (`<n>.MSG`: the lines characters say), the talker list
//! (`TALKERS.SH`), the voice and lip-sync file names of a line, and the
//! lip-sync files themselves (M4 step 4a-1).
//!
//! Rules from the behaviour note `docs/behaviour/talking.md` (session A);
//! rule numbers below are that note's. The byte layouts are also in the
//! community notes `kq8msg.txt` and `kq8lip.txt` (libkq8fpc, facts only).
//! See docs/provenance/README.md.
//!
//! - A message file (rule 2): 2-byte header (type 0x0F or 0x8F, header
//!   size 0), then a 4-byte version (5010), a 2-byte data size, a 2-byte
//!   "last id" and a 2-byte count; then `count` records of 11 bytes
//!   (noun, verb, case, sequence, talker: 1 byte each; the text's offset,
//!   2 bytes, counted from the start of the version field; 4 unused
//!   bytes); the texts end in a zero byte. Little-endian.
//! - Stage directions (rule 3): everything inside `( … )` is taken out of
//!   the shown text; a `#` inside sets the line's "no stop" mark; `[n]`
//!   inside is a conversation camera number.
//! - Voice file (rule 4): `A` + file (3) + noun (2) + verb (2) + `.` +
//!   case (2) + sequence (1), each in base 36, padded with zeros; the
//!   lip-sync file has `S` instead of `A`. Both sit next to the message
//!   file.
//! - Talkers (rule 5): `(define NAME n)`; the object name is the name in
//!   lower case with a capital first letter; ten "generic" talkers are
//!   spoken by the character that issued the line instead.
//! - Lip-sync file (rule 17, `kq8lip.txt`): 2-byte header (0x8E or 0x0E,
//!   0), then records of two u16: time in 1/60 s from the line's start,
//!   mouth shape (0-7, 0 = rest); a time of 0xFFFF ends the list (what
//!   follows, the 0x8E files' letter section, is not used).

use std::collections::BTreeMap;

/// The version every message file has (rule 2, data).
pub const VERSION: u32 = 5010;

/// Bytes of one line record (rule 2).
pub const RECORD: usize = 11;

/// Bytes before the version field (the header the offsets don't count).
const HEAD: usize = 2;

/// The five numbers that name a line (the note's "tuple").
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tuple {
    pub file: u16,
    pub noun: u8,
    pub verb: u8,
    pub case: u8,
    pub seq: u8,
}

impl Tuple {
    pub fn new(file: u16, noun: u8, verb: u8, case: u8, seq: u8) -> Self {
        Tuple { file, noun, verb, case, seq }
    }

    /// The same line with another sequence number.
    pub fn with_seq(self, seq: u8) -> Self {
        Tuple { seq, ..self }
    }

    /// The voice file's name (rule 4), e.g. 1000/55/0/0/1 → `A0RS1J00.001`.
    pub fn voice_name(&self) -> String {
        self.file_name('A')
    }

    /// The lip-sync file's name (rule 4): `S` instead of `A`.
    pub fn lip_name(&self) -> String {
        self.file_name('S')
    }

    fn file_name(&self, first: char) -> String {
        format!(
            "{first}{}{}{}.{}{}",
            base36(self.file as u32, 3),
            base36(self.noun as u32, 2),
            base36(self.verb as u32, 2),
            base36(self.case as u32, 2),
            base36(self.seq as u32, 1)
        )
    }
}

impl std::fmt::Display for Tuple {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} {} {} {} {}", self.file, self.noun, self.verb, self.case, self.seq)
    }
}

/// `n` in base 36 (digits 0-9 then A-Z), padded with zeros on the left to
/// at least `width` characters (rule 4).
pub fn base36(mut n: u32, width: usize) -> String {
    const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
    let mut out = Vec::new();
    loop {
        out.push(DIGITS[(n % 36) as usize]);
        n /= 36;
        if n == 0 {
            break;
        }
    }
    while out.len() < width {
        out.push(b'0');
    }
    out.reverse();
    String::from_utf8(out).unwrap_or_default()
}

/// What the stage directions of a text leave (rule 3).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Shown {
    /// The text without the bracketed parts (spaces around them kept).
    pub text: String,
    /// A `#` inside the brackets: the player can't cut the line short.
    pub no_stop: bool,
    /// A `[n]` inside the brackets: a conversation camera number.
    pub camera: Option<u32>,
    /// A `(` without its `)`: the rest of the text was taken as a stage
    /// direction (our choice; logged by callers).
    pub unclosed: bool,
}

/// Takes the stage directions out of a line's text (rule 3).
pub fn stage_directions(raw: &str) -> Shown {
    let mut shown = Shown::default();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c != '(' {
            shown.text.push(c);
            continue;
        }
        let mut inside = String::new();
        let mut closed = false;
        for c in chars.by_ref() {
            if c == ')' {
                closed = true;
                break;
            }
            inside.push(c);
        }
        shown.unclosed |= !closed;
        shown.no_stop |= inside.contains('#');
        let mut rest = inside.as_str();
        while let Some(open) = rest.find('[') {
            let after = &rest[open + 1..];
            let Some(close) = after.find(']') else { break };
            if let Ok(n) = after[..close].trim().parse::<u32>() {
                shown.camera.get_or_insert(n);
            }
            rest = &after[close + 1..];
        }
    }
    shown
}

/// One line of a message file.
#[derive(Debug, Clone, PartialEq)]
pub struct Line {
    pub tuple: Tuple,
    /// The talker's number (`TALKERS.SH`).
    pub talker: u8,
    /// The text as stored, stage directions included.
    pub raw: String,
}

impl Line {
    pub fn shown(&self) -> Shown {
        stage_directions(&self.raw)
    }
}

/// A parsed message file.
#[derive(Debug, Clone, PartialEq)]
pub struct MessageFile {
    /// The header's type byte (0x0F or 0x8F).
    pub kind: u8,
    pub version: u32,
    pub last_id: u16,
    pub lines: Vec<Line>,
}

fn u16_at(b: &[u8], at: usize) -> Option<u16> {
    Some(u16::from_le_bytes([*b.get(at)?, *b.get(at + 1)?]))
}

impl MessageFile {
    /// Parses a message file whose number (its file name) is `file`.
    pub fn parse(file: u16, bytes: &[u8]) -> Result<MessageFile, String> {
        let kind = *bytes.first().ok_or("empty file")?;
        if kind != 0x0F && kind != 0x8F {
            return Err(format!("header type 0x{kind:02X}, not 0x0F or 0x8F"));
        }
        let version = bytes.get(HEAD..HEAD + 4).map(|v| u32::from_le_bytes([v[0], v[1], v[2], v[3]])).ok_or("too short for the version")?;
        let last_id = u16_at(bytes, HEAD + 6).ok_or("too short for the last id")?;
        let count = u16_at(bytes, HEAD + 8).ok_or("too short for the count")? as usize;
        let first = HEAD + 10;
        let mut lines = Vec::with_capacity(count);
        for n in 0..count {
            let r = bytes.get(first + n * RECORD..first + (n + 1) * RECORD).ok_or(format!("record {n} of {count} is past the end"))?;
            let offset = u16::from_le_bytes([r[5], r[6]]) as usize;
            let start = HEAD + offset;
            let text = bytes.get(start..).ok_or(format!("record {n}: text offset {offset} is past the end"))?;
            let end = text.iter().position(|&b| b == 0).ok_or(format!("record {n}: text has no end"))?;
            lines.push(Line {
                tuple: Tuple::new(file, r[0], r[1], r[2], r[3]),
                talker: r[4],
                // The texts are 8-bit Windows text; bytes as Latin-1.
                raw: text[..end].iter().map(|&b| b as char).collect(),
            });
        }
        Ok(MessageFile { kind, version, last_id, lines })
    }

    pub fn find(&self, t: Tuple) -> Option<&Line> {
        self.lines.iter().find(|l| l.tuple == t)
    }

    /// Rule 6, `<last>` 0: the highest sequence of the unbroken run
    /// `first`, `first + 1`, … of this noun, verb and case; `None` if
    /// `first` itself is missing.
    pub fn last_of_run(&self, t: Tuple) -> Option<u8> {
        self.find(t)?;
        let mut last = t.seq;
        while last < u8::MAX && self.find(t.with_seq(last + 1)).is_some() {
            last += 1;
        }
        Some(last)
    }
}

/// The ten talkers spoken by the issuer instead of an object by their
/// name (rule 5), as the game writes object names.
pub const GENERIC_TALKERS: [&str; 10] =
    ["Me", "Iceorc", "Gen_gryph", "Firedwarf", "Ghenchman", "Gen_hillmn", "Gen_gnome1", "Gen_gnome2", "Gen_gnome3", "Gen_nymph"];

/// `TALKERS.SH` (rule 5): talker number → object name.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Talkers {
    pub names: BTreeMap<u8, String>,
}

/// A talker's name as an object name (rule 5): lower case with a capital
/// first letter (`GEN_GNOME1` → `Gen_gnome1`).
pub fn object_name(talker: &str) -> String {
    let lower = talker.to_ascii_lowercase();
    let mut c = lower.chars();
    match c.next() {
        Some(f) => f.to_ascii_uppercase().to_string() + c.as_str(),
        None => String::new(),
    }
}

impl Talkers {
    /// Reads the `(define NAME n)` lines; `;` starts a comment.
    pub fn parse(bytes: &[u8]) -> Talkers {
        let text: String = bytes.iter().map(|&b| b as char).collect();
        let mut names = BTreeMap::new();
        for line in text.lines() {
            let line = line.split(';').next().unwrap_or("").trim();
            let Some(inner) = line.strip_prefix('(').and_then(|l| l.strip_suffix(')')) else { continue };
            let words: Vec<&str> = inner.split_whitespace().collect();
            if let ["define", name, n] = words[..] {
                if let Ok(n) = n.parse::<u8>() {
                    names.insert(n, object_name(name));
                }
            }
        }
        Talkers { names }
    }

    /// The object name of talker `n`.
    pub fn name(&self, n: u8) -> Option<&str> {
        self.names.get(&n).map(String::as_str)
    }

    /// Whether talker `n` is one of the ten generic ones (rule 5).
    pub fn is_generic(&self, n: u8) -> bool {
        self.name(n).is_some_and(|name| GENERIC_TALKERS.iter().any(|g| g.eq_ignore_ascii_case(name)))
    }
}

/// One mouth change of a lip-sync file (rule 17).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Mouth {
    /// 1/60 s from the start of the line.
    pub time: u16,
    /// 0-7 (0 = rest); above 7: handled differently (not traced).
    pub shape: u16,
}

/// Time unit of a lip-sync file, seconds (rule 17).
pub const LIP_TICK: f64 = 1.0 / 60.0;

/// Reads a lip-sync file (rule 17, `kq8lip.txt`).
pub fn parse_lips(bytes: &[u8]) -> Result<Vec<Mouth>, String> {
    let kind = *bytes.first().ok_or("empty file")?;
    if kind != 0x8E && kind != 0x0E {
        return Err(format!("header type 0x{kind:02X}, not 0x8E or 0x0E"));
    }
    let mut out = Vec::new();
    let mut at = 2;
    loop {
        let time = u16_at(bytes, at).ok_or("no end mark (time 0xFFFF)")?;
        if time == 0xFFFF {
            return Ok(out);
        }
        let shape = u16_at(bytes, at + 2).ok_or("a record cut short")?;
        out.push(Mouth { time, shape });
        at += 4;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A made-up message file (never a copy of the game's): two lines.
    fn sample() -> Vec<u8> {
        let texts: [&[u8]; 2] = [b"([#]CALM) Hello there.\0", b"Second.\0"];
        let first_text = 10 + 2 * RECORD; // from the version field
        let mut b = vec![0x0F, 0x00];
        b.extend(VERSION.to_le_bytes());
        b.extend(0u16.to_le_bytes()); // data size (not used)
        b.extend(7u16.to_le_bytes());
        b.extend(2u16.to_le_bytes());
        let mut offset = first_text;
        for (n, t) in texts.iter().enumerate() {
            b.extend([55, 0, 0, n as u8 + 1, 4]);
            b.extend((offset as u16).to_le_bytes());
            b.extend([0; 4]);
            offset += t.len();
        }
        for t in texts {
            b.extend(t);
        }
        b
    }

    #[test]
    fn a_message_file_gives_its_lines() {
        let m = MessageFile::parse(1000, &sample()).unwrap();
        assert_eq!((m.version, m.last_id, m.lines.len()), (VERSION, 7, 2));
        let l = &m.lines[0];
        assert_eq!((l.tuple, l.talker), (Tuple::new(1000, 55, 0, 0, 1), 4));
        let s = l.shown();
        assert_eq!(s.text, " Hello there.", "the space after the bracket stays");
        assert!(s.no_stop);
        assert_eq!(m.lines[1].shown(), Shown { text: "Second.".into(), ..Default::default() });
        assert_eq!(m.last_of_run(Tuple::new(1000, 55, 0, 0, 1)), Some(2));
        assert_eq!(m.last_of_run(Tuple::new(1000, 55, 0, 0, 2)), Some(2));
        assert_eq!(m.last_of_run(Tuple::new(1000, 55, 0, 0, 3)), None);
        assert!(MessageFile::parse(1000, &[0x10, 0]).is_err());
    }

    #[test]
    fn stage_directions_cameras_and_no_stop() {
        let s = stage_directions("([2]SADLY) I see.");
        assert_eq!((s.text.as_str(), s.no_stop, s.camera), (" I see.", false, Some(2)));
        let s = stage_directions("A (#) b (c");
        assert_eq!((s.text.as_str(), s.no_stop, s.unclosed), ("A  b ", true, true));
    }

    #[test]
    fn voice_names_are_base_36() {
        // The note's example (rule 4).
        assert_eq!(Tuple::new(1000, 55, 0, 0, 1).voice_name(), "A0RS1J00.001");
        assert_eq!(Tuple::new(1000, 55, 0, 0, 1).lip_name(), "S0RS1J00.001");
        assert_eq!(Tuple::new(0, 0, 0, 1, 1).voice_name(), "A0000000.011");
        assert_eq!(base36(35, 1), "Z");
        assert_eq!(base36(36, 1), "10");
    }

    #[test]
    fn talkers_become_object_names() {
        let t = Talkers::parse(b"; comment\n(define CONNOR       4)\n(define GEN_GNOME1 40)\n(define LAKELADY 9)\n");
        assert_eq!(t.name(4), Some("Connor"));
        assert_eq!(t.name(9), Some("Lakelady"));
        assert_eq!(t.name(40), Some("Gen_gnome1"));
        assert!(t.is_generic(40) && !t.is_generic(4));
    }

    #[test]
    fn lip_files_end_at_ffff() {
        let mut b = vec![0x8E, 0];
        for (t, s) in [(2u16, 7u16), (65, 0)] {
            b.extend(t.to_le_bytes());
            b.extend(s.to_le_bytes());
        }
        b.extend([0xFF, 0xFF, 0x7B, 0x7B]);
        assert_eq!(parse_lips(&b).unwrap(), vec![Mouth { time: 2, shape: 7 }, Mouth { time: 65, shape: 0 }]);
        assert!(parse_lips(&[0x8E, 0, 1, 0]).is_err());
    }
}
