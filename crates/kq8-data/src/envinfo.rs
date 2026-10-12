//! Ground flags per ground material, from a world's `kq/envInfo.kq`.
//!
//! Rules from the behaviour note `docs/behaviour/water.md` "Ground flags
//! from `envInfo.kq`" (session A, Q43). The file is a spreadsheet saved as
//! text: one `textureN=` line per material number N, columns separated by
//! tabs.
//!
//! How a line is read: the text after `=`, with spaces and tabs stripped
//! from both ends; its first character (the `inUse` mark) is skipped; then
//! each tab moves to the next column and each letter or digit turns the
//! current column on. Column k (from 0, `dirt`) is flag `1 << k`; only 16
//! columns fit.

/// `water` column.
pub const WATER: u32 = 0x8;
/// `poison` column.
pub const POISON: u32 = 0x40;
/// `fastpoison` / `lava` column.
pub const FAST_POISON: u32 = 0x80;
/// `swamp` column.
pub const SWAMP: u32 = 0x100;
/// The liquids: water, poison, fastpoison/lava, swamp (0x1C8).
pub const LIQUID: u32 = WATER | POISON | FAST_POISON | SWAMP;

/// Flags of one `textureN=` value, read as the note says.
pub fn line_flags(value: &str) -> u32 {
    let text = value.trim_matches([' ', '\t']);
    let mut chars = text.chars();
    // The inUse mark (whatever it is) is skipped.
    chars.next();
    // Before the first tab we are still in the inUse column.
    let mut column: i32 = -1;
    let mut flags = 0u32;
    for c in chars {
        if c == '\t' {
            column += 1;
        } else if c.is_ascii_alphanumeric() && (0..16).contains(&column) {
            flags |= 1 << column;
        }
    }
    flags
}

/// Flags per material number (index N = `textureN`; materials without a
/// line get 0). Lines starting with `;` are comments.
pub fn parse(bytes: &[u8]) -> Vec<u32> {
    let text = String::from_utf8_lossy(bytes);
    let mut flags: Vec<u32> = Vec::new();
    for line in text.lines() {
        let line = line.trim_end_matches(['\r', '\n']);
        let trimmed = line.trim_start();
        if trimmed.starts_with(';') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else { continue };
        let key = key.trim().to_ascii_lowercase();
        let Some(n) = key.strip_prefix("texture").and_then(|n| n.parse::<usize>().ok()) else { continue };
        if flags.len() <= n {
            flags.resize(n + 1, 0);
        }
        flags[n] = line_flags(value);
    }
    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_columns_as_the_note_says() {
        // inUse x, dirt x: flag 0x1.
        assert_eq!(line_flags("\tx\tx\t\t\t\t\t\t\t\t\t\t\t"), 0x1);
        // inUse, then water (the 4th column after inUse) as the last mark:
        // the trailing x counts once the ends are stripped.
        assert_eq!(line_flags("\tx\t\t\t\tx"), WATER);
        // poison and swamp.
        assert_eq!(line_flags("\tx\t\t\t\t\t\t\tx\t\tx\t"), POISON | SWAMP);
        // No inUse mark: after stripping, the first mark is the one skipped.
        assert_eq!(line_flags("\t\tx\t"), 0);
        // Digits count too; an empty line is 0.
        assert_eq!(line_flags("\t1\t\t1"), 0x2);
        assert_eq!(line_flags(""), 0);
    }

    #[test]
    fn parses_texture_lines_and_skips_comments() {
        let file = b"[EnvInfo]\r\nclassType=KQEnvInfo\r\nnTextures=3\r\n;textureX=\tinUse\tdirt\tgrass\trock\twater\r\ntexture0=\tx\tx\t\t\t\r\ntexture2=\tx\t\t\t\tx\r\n";
        assert_eq!(parse(file), vec![0x1, 0, WATER]);
        assert_eq!(LIQUID, 0x1C8);
    }
}
