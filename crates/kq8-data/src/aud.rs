//! Sound files (`.aud`, and the speech files in the `English` folders).
//!
//! Rules from the behaviour note `docs/behaviour/sound.md` (session A);
//! rule numbers below are that note's. The step table's numbers are from
//! the community note `kq8aud.txt` (libkq8fpc, facts only; rule 7 says the
//! game's own table is the same). See docs/provenance/README.md.
//!
//! - A file is taken as SOL when bytes 0-1 are `0D 0C`, bytes 2-5 `SOL\0`,
//!   the format byte has no bits outside 0x01/0x04/0x08/0x10 and has 0x04
//!   or 0x08 (rule 12). Otherwise it is tried as a plain PCM WAV file.
//! - SOL header (rule 2): u16 samples per second at byte 6, format byte at
//!   8, u32 data length at 9. Byte 1 is ignored: data always starts at
//!   byte 14 (rule 10).
//! - Format 0x01 = compressed, 0x10 = stereo (rule 3). Compressed: one byte
//!   per sample and channel, stereo alternates left, right; the low 7 bits
//!   pick a step, bit 0x80 subtracts it, else it is added; every channel
//!   starts at 0; the sum wraps round at 16 bits (rules 7-8).
//!   Uncompressed (no game file): signed 16-bit samples (rule 9).
//! - Output is always signed 16-bit (rule 10).

use std::path::PathBuf;

use crate::GameDir;

/// A world's loose folders in search order (`models-and-textures.md`
/// "Where texture names are looked up": the folder loaded last wins).
pub const WORLD_FOLDERS: [&str; 7] = ["patch", "8gui", "resource", "English", "8bit", "kq", "sound"];

/// `GAME`'s folders after the world's: `patch` first (the note's likely
/// reading), the rest in an order of ours (not traced; only `SOUND` holds
/// `.aud` files, so it makes no difference).
pub const GAME_FOLDERS: [&str; 7] = ["patch", "8Gui", "Resource", "ENGLISH", "Common", "Kq", "SOUND"];

/// Rule 13: the names tried for a sound name, in order: as given, then
/// with the extension replaced by `.AUD` (added if there is none), then
/// `.WAV`.
pub fn name_variants(name: &str) -> Vec<String> {
    let file_part = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem_len = name.len() - file_part.len() + file_part.rfind('.').unwrap_or(file_part.len());
    let stem = &name[..stem_len];
    let mut out = vec![name.to_string()];
    for ext in [".AUD", ".WAV"] {
        let v = format!("{stem}{ext}");
        if !out.iter().any(|o| o.eq_ignore_ascii_case(&v)) {
            out.push(v);
        }
    }
    out
}

/// Finds a sound file for `world` (rule 13). A name with a folder in it
/// (`daventry\sound\c_drwndi.aud`) is looked up from the game folder;
/// otherwise each name of [`name_variants`] is tried in the world's
/// folders, then `GAME`'s. `None`: not found anywhere.
pub fn find(game: &GameDir, world: &str, name: &str) -> Option<PathBuf> {
    let has_folder = name.contains(['/', '\\']);
    for variant in name_variants(name) {
        if has_folder {
            if let Some(p) = game.find(&variant).filter(|p| p.is_file()) {
                return Some(p);
            }
            continue;
        }
        let folders = WORLD_FOLDERS.iter().map(|f| format!("{world}/{f}")).chain(GAME_FOLDERS.iter().map(|f| format!("GAME/{f}")));
        for folder in folders {
            if let Some(p) = game.find(&format!("{folder}/{variant}")).filter(|p| p.is_file()) {
                return Some(p);
            }
        }
    }
    None
}

/// Bytes before the data of a SOL file (rule 10).
pub const SOL_HEADER: usize = 14;

const SOL_COMPRESSED: u8 = 0x01;
const SOL_16BIT: u8 = 0x04;
const SOL_SIGNED: u8 = 0x08;
const SOL_STEREO: u8 = 0x10;
const SOL_KNOWN_BITS: u8 = SOL_COMPRESSED | SOL_16BIT | SOL_SIGNED | SOL_STEREO;

/// What kind of file a sound came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Sierra SOL; `format` is header byte 8, `header_byte_1` the length
    /// byte the game ignores (rule 10), kept only for the log.
    Sol { format: u8, header_byte_1: u8 },
    /// Plain PCM WAV; `bits` is 8 or 16 as stored.
    Wav { bits: u16 },
}

/// A decoded sound.
#[derive(Debug, Clone)]
pub struct Sound {
    pub kind: Kind,
    /// Samples per second (per channel).
    pub rate: u32,
    /// 1 or 2.
    pub channels: u16,
    /// Signed 16-bit samples; for stereo left, right, left, right...
    pub samples: Vec<i16>,
    /// Data length the header gives (SOL) or the `data` chunk size (WAV).
    pub declared_len: u64,
    /// Data bytes actually present in the file (less than `declared_len`
    /// only for a cut-off file; then what is there is decoded, our choice).
    pub present_len: u64,
    /// Times a compressed sum left the 16-bit range and wrapped round
    /// (rule 8: 0 in every game file).
    pub wraps: u64,
    /// Largest sample size, |sample| (32,768 for -32,768).
    pub peak: u32,
}

impl Sound {
    /// Samples per channel.
    pub fn frames(&self) -> usize {
        self.samples.len() / self.channels.max(1) as usize
    }

    pub fn seconds(&self) -> f64 {
        if self.rate == 0 {
            0.0
        } else {
            self.frames() as f64 / self.rate as f64
        }
    }
}

/// Step `i` (0..128) of the compressed format (rule 7). The numbers are the
/// community note's; we build them from their pattern: 0, 8, then steps of
/// 16 up to 0x200, 8 up to 0x400, 0x40 up to 0x800, 0x100 up to 0x1000,
/// 0x400 up to 0x2000, 0x1000 up to 0x4000.
pub fn step(i: u8) -> i32 {
    let i = i32::from(i & 0x7F);
    match i {
        0 => 0,
        1 => 8,
        2..=33 => 16 * (i - 1),
        34..=97 => 0x200 + 8 * (i - 33),
        98..=113 => 0x400 + 0x40 * (i - 97),
        114..=121 => 0x800 + 0x100 * (i - 113),
        122..=125 => 0x1000 + 0x400 * (i - 121),
        _ => 0x2000 + 0x1000 * (i - 125),
    }
}

/// Is this a SOL file the game accepts (rule 12)?
pub fn is_sol(bytes: &[u8]) -> bool {
    bytes.len() >= SOL_HEADER
        && bytes[0] == 0x0D
        && bytes[1] == 0x0C
        && &bytes[2..6] == b"SOL\0"
        && bytes[8] & !SOL_KNOWN_BITS == 0
        && bytes[8] & (SOL_16BIT | SOL_SIGNED) != 0
}

/// Decodes a sound file: SOL first, then WAV (rule 12). The error is the
/// reason the file is refused.
pub fn parse(bytes: &[u8]) -> Result<Sound, String> {
    if is_sol(bytes) {
        Ok(parse_sol(bytes))
    } else if bytes.len() >= 12 && &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
        parse_wav(bytes)
    } else {
        Err("format is unrecognized (not SOL, not WAV)".into())
    }
}

fn u16_at(b: &[u8], at: usize) -> u16 {
    u16::from_le_bytes([b[at], b[at + 1]])
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes([b[at], b[at + 1], b[at + 2], b[at + 3]])
}

fn parse_sol(bytes: &[u8]) -> Sound {
    let format = bytes[8];
    let channels: u16 = if format & SOL_STEREO != 0 { 2 } else { 1 };
    let declared_len = u64::from(u32_at(bytes, 9));
    let data = &bytes[SOL_HEADER..];
    let data = &data[..data.len().min(declared_len as usize)];
    let mut sound = Sound {
        kind: Kind::Sol { format, header_byte_1: bytes[1] },
        rate: u32::from(u16_at(bytes, 6)),
        channels,
        samples: Vec::new(),
        declared_len,
        present_len: data.len() as u64,
        wraps: 0,
        peak: 0,
    };
    let ch = channels as usize;
    if format & SOL_COMPRESSED != 0 {
        // Rule 9: samples = data length / channels; a last odd byte of a
        // stereo file has no partner and is dropped.
        let usable = data.len() / ch * ch;
        let mut current = [0i16; 2];
        sound.samples.reserve(usable);
        for (n, &code) in data[..usable].iter().enumerate() {
            let c = n % ch;
            let d = step(code);
            let sum = i32::from(current[c]) + if code & 0x80 != 0 { -d } else { d };
            if sum < i32::from(i16::MIN) || sum > i32::from(i16::MAX) {
                sound.wraps += 1;
            }
            current[c] = sum as i16; // wraps round (rule 8)
            sound.samples.push(current[c]);
        }
    } else {
        // Rule 9: length / (2 x channels) samples per channel.
        let usable = data.len() / (2 * ch) * (2 * ch);
        sound.samples = data[..usable].as_chunks::<2>().0.iter().map(|&p| i16::from_le_bytes(p)).collect();
    }
    sound.peak = peak(&sound.samples);
    sound
}

fn peak(samples: &[i16]) -> u32 {
    samples.iter().map(|&s| i32::from(s).unsigned_abs()).max().unwrap_or(0)
}

/// Rule 12's WAV test: a format chunk of at least 14 bytes, tag 1 (plain
/// PCM), 1 or 2 channels, 8-bit (unsigned) or 16-bit samples, and a
/// `data` chunk. Only the fields needed are read ("Do not reproduce" 6).
fn parse_wav(bytes: &[u8]) -> Result<Sound, String> {
    let mut at = 12;
    let mut fmt: Option<&[u8]> = None;
    let mut data: Option<(&[u8], u64)> = None;
    while at + 8 <= bytes.len() {
        let id = &bytes[at..at + 4];
        let size = u32_at(bytes, at + 4) as usize;
        let body = &bytes[at + 8..bytes.len().min(at + 8 + size)];
        match id {
            b"fmt " if fmt.is_none() => fmt = Some(body),
            b"data" if data.is_none() => data = Some((body, size as u64)),
            _ => {}
        }
        at += 8 + size + (size & 1); // chunks are padded to an even size
    }
    let fmt = fmt.ok_or("WAV without a format chunk")?;
    if fmt.len() < 14 {
        return Err(format!("WAV format chunk of {} bytes (needs 14)", fmt.len()));
    }
    let (tag, channels, rate, block_align) = (u16_at(fmt, 0), u16_at(fmt, 2), u32_at(fmt, 4), u16_at(fmt, 12));
    if tag != 1 {
        return Err(format!("WAV format tag {tag} (only 1, plain PCM)"));
    }
    if !(1..=2).contains(&channels) {
        return Err(format!("WAV with {channels} channels"));
    }
    // A 14-byte chunk has no bits field; then it follows from the block
    // size (our choice; no game file has such a chunk).
    let bits = if fmt.len() >= 16 { u16_at(fmt, 14) } else { block_align * 8 / channels };
    if bits != 8 && bits != 16 {
        return Err(format!("WAV with {bits}-bit samples (only 8 or 16)"));
    }
    let (data, declared_len) = data.ok_or("WAV without a data chunk")?;
    let samples: Vec<i16> = if bits == 8 {
        data.iter().map(|&b| (i16::from(b) - 0x80) << 8).collect()
    } else {
        data.as_chunks::<2>().0.iter().map(|&p| i16::from_le_bytes(p)).collect()
    };
    let samples = samples[..samples.len() / channels as usize * channels as usize].to_vec();
    Ok(Sound {
        kind: Kind::Wav { bits },
        rate,
        channels,
        peak: peak(&samples),
        samples,
        declared_len,
        present_len: data.len() as u64,
        wraps: 0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sol(rate: u16, format: u8, data: &[u8]) -> Vec<u8> {
        let mut b = vec![0x0D, 0x0C, b'S', b'O', b'L', 0];
        b.extend_from_slice(&rate.to_le_bytes());
        b.push(format);
        b.extend_from_slice(&(data.len() as u32).to_le_bytes());
        b.push(0);
        b.extend_from_slice(data);
        b
    }

    #[test]
    fn step_table_matches_the_community_note() {
        // Spot checks against kq8aud.txt, one or two per stretch.
        let want = [
            (0, 0), (1, 8), (2, 0x10), (3, 0x20), (32, 0x1F0), (33, 0x200), (34, 0x208),
            (96, 0x3F8), (97, 0x400), (98, 0x440), (112, 0x7C0), (113, 0x800), (114, 0x900),
            (120, 0xF00), (121, 0x1000), (122, 0x1400), (125, 0x2000), (126, 0x3000), (127, 0x4000),
        ];
        for (i, v) in want {
            assert_eq!(step(i), v, "step {i}");
        }
        assert!((1..128).all(|i| step(i) > step(i - 1)), "the table only grows");
        assert_eq!(step(0x80 | 5), step(5), "bit 0x80 is the sign, not part of the index");
    }

    #[test]
    fn compressed_mono_adds_and_subtracts() {
        // +8, +16, -0x10 (0x82), +0x4000 => 8, 24, 8, 16392.
        let s = parse(&sol(22050, 0x0D, &[0x01, 0x02, 0x82, 0x7F])).unwrap();
        assert_eq!((s.rate, s.channels), (22050, 1));
        assert_eq!(s.samples, [8, 24, 8, 16392]);
        assert_eq!((s.frames(), s.wraps, s.peak), (4, 0, 16392));
    }

    #[test]
    fn stereo_channels_start_at_zero_each() {
        // left +8, right -8, left +8, right -8.
        let s = parse(&sol(44100, 0x1D, &[0x01, 0x81, 0x01, 0x81])).unwrap();
        assert_eq!(s.channels, 2);
        assert_eq!(s.samples, [8, -8, 16, -16]);
        assert_eq!(s.frames(), 2);
    }

    #[test]
    fn the_sum_wraps_round_and_is_counted() {
        // 0x4000 twice = 0x8000 = 32768: wraps to -32768.
        let s = parse(&sol(22050, 0x0D, &[0x7F, 0x7F])).unwrap();
        assert_eq!(s.samples, [0x4000, i16::MIN]);
        assert_eq!((s.wraps, s.peak), (1, 32768));
    }

    #[test]
    fn header_byte_1_is_ignored_only_after_the_acceptance_test() {
        // Rule 12 needs 0x0C; rule 10: once accepted, data is at byte 14.
        let mut b = sol(22050, 0x0D, &[0x01]);
        b[1] = 0x0E;
        assert!(parse(&b).is_err());
    }

    #[test]
    fn acceptance_rules() {
        assert!(parse(&sol(22050, 0x01, &[0])).is_err(), "needs 16-bit or signed");
        assert!(parse(&sol(22050, 0x2D, &[0])).is_err(), "unknown bit 0x20");
        assert!(parse(&sol(22050, 0x05, &[1])).is_ok(), "16-bit alone is enough");
        assert!(parse(b"junk").is_err());
    }

    #[test]
    fn uncompressed_sol_reads_signed_16_bit() {
        let s = parse(&sol(22050, 0x0C, &[0x00, 0x80, 0xFF, 0x7F])).unwrap();
        assert_eq!(s.samples, [i16::MIN, i16::MAX]);
    }

    #[test]
    fn a_cut_off_file_decodes_what_is_there() {
        let mut b = sol(22050, 0x0D, &[0x01, 0x01, 0x01]);
        b.truncate(b.len() - 1);
        let s = parse(&b).unwrap();
        assert_eq!((s.declared_len, s.present_len, s.samples.len()), (3, 2, 2));
    }

    fn wav(bits: u16, channels: u16, data: &[u8]) -> Vec<u8> {
        let mut f = Vec::new();
        f.extend_from_slice(&1u16.to_le_bytes());
        f.extend_from_slice(&channels.to_le_bytes());
        f.extend_from_slice(&22050u32.to_le_bytes());
        f.extend_from_slice(&(22050 * u32::from(channels * bits / 8)).to_le_bytes());
        f.extend_from_slice(&(channels * bits / 8).to_le_bytes());
        f.extend_from_slice(&bits.to_le_bytes());
        let mut b = b"RIFF\0\0\0\0WAVE".to_vec();
        b.extend_from_slice(b"fmt ");
        b.extend_from_slice(&(f.len() as u32).to_le_bytes());
        b.extend_from_slice(&f);
        b.extend_from_slice(b"data");
        b.extend_from_slice(&(data.len() as u32).to_le_bytes());
        b.extend_from_slice(data);
        b
    }

    #[test]
    fn name_variants_follow_rule_13() {
        assert_eq!(name_variants("raven.snd"), ["raven.snd", "raven.AUD", "raven.WAV"]);
        assert_eq!(name_variants("dirt1"), ["dirt1", "dirt1.AUD", "dirt1.WAV"]);
        assert_eq!(name_variants("door4.aud"), ["door4.aud", "door4.WAV"]);
        assert_eq!(name_variants("a.b\\c_drwndi.aud"), ["a.b\\c_drwndi.aud", "a.b\\c_drwndi.WAV"]);
    }

    #[test]
    fn wav_8_and_16_bit() {
        let s = parse(&wav(16, 1, &[0x34, 0x12, 0xFF, 0xFF])).unwrap();
        assert_eq!((s.kind, s.samples.as_slice()), (Kind::Wav { bits: 16 }, &[0x1234, -1][..]));
        let s = parse(&wav(8, 2, &[0x80, 0xFF, 0x00, 0x80])).unwrap();
        assert_eq!(s.samples, [0, 0x7F00, i16::MIN, 0]);
        assert_eq!(s.frames(), 2);
        assert!(parse(&wav(24, 1, &[0, 0, 0])).is_err());
    }
}
