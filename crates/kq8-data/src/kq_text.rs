//! The game's plain-text description files (`.kq`): sections in square
//! brackets, then `key=value` lines. Lines starting with `;` are comments.
//!
//! Observed in the data files, e.g. `data/Daventry/KQ/Terrain/TERRAIN.KQ`:
//!
//! ```text
//! [Terrain]
//! classType=KQTerrain
//! groundScale=12
//! ;smoothing=1
//! ```
//!
//! Keys are compared without regard to case (the files mix `locX`,
//! `LocX`, ...). Values are kept as text; callers convert them.

#[derive(Debug, Clone, Default)]
pub struct Section {
    /// Text between the brackets, as written.
    pub name: String,
    /// (key, value) in file order, both trimmed.
    pub entries: Vec<(String, String)>,
}

impl Section {
    /// Value of the first entry called `key` (any case).
    pub fn get(&self, key: &str) -> Option<&str> {
        self.entries.iter().find(|(k, _)| k.eq_ignore_ascii_case(key)).map(|(_, v)| v.as_str())
    }

    /// `key` as a number. `None` if missing; `Some(Err)` if not a number.
    pub fn number(&self, key: &str) -> Option<Result<f64, String>> {
        self.get(key).map(|v| v.parse::<f64>().map_err(|_| format!("{key}={v} is not a number")))
    }
}

/// Splits a file into sections. Lines before the first section go into a
/// section with an empty name. Bytes that are not UTF-8 are kept as
/// Latin-1 (the files are 1998 Windows text).
pub fn parse(bytes: &[u8]) -> Vec<Section> {
    let text: String = match std::str::from_utf8(bytes) {
        Ok(s) => s.to_string(),
        Err(_) => bytes.iter().map(|&b| b as char).collect(),
    };
    let mut sections = Vec::new();
    let mut current = Section::default();
    for raw in text.lines() {
        let line = raw.trim();
        if line.is_empty() || line.starts_with(';') {
            continue;
        }
        if let Some(name) = line.strip_prefix('[').and_then(|l| l.strip_suffix(']')) {
            if !current.name.is_empty() || !current.entries.is_empty() {
                sections.push(std::mem::take(&mut current));
            }
            current.name = name.trim().to_string();
        } else if let Some((k, v)) = line.split_once('=') {
            current.entries.push((k.trim().to_string(), v.trim().to_string()));
        }
    }
    if !current.name.is_empty() || !current.entries.is_empty() {
        sections.push(current);
    }
    sections
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sections_keys_comments() {
        let s = parse(b"[Terrain]\r\nclassType=KQTerrain\r\ngroundScale=12\r\n;smoothing=1\r\n\r\n[b]\nLocX = 5\n");
        assert_eq!(s.len(), 2);
        assert_eq!(s[0].get("groundscale"), Some("12"));
        assert_eq!(s[0].get("smoothing"), None);
        assert_eq!(s[1].number("locx").unwrap().unwrap(), 5.0);
    }
}
