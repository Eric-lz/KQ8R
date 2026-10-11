//! Key bindings ("action maps", `.sae` files such as `MASK/Connor.sae`).
//!
//! Format observed in the data (docs/behaviour/connor-movement.md,
//! "Controls"): a text file starting with `[ACTIONMAP]` and ending with
//! `[END]`. Each binding is three lines:
//!
//! ```text
//! [KEY] make up          (or "break up"; [MOU] for mouse buttons)
//! [ACTION] MoveY 1.000000 0
//! [ACTIONFLAG] ActionFlagValue
//! ```
//!
//! `make` = key pressed, `break` = key released. The action has a name, a
//! value and a second number (always 0 in `Connor.sae`).

#[derive(Debug, Clone, PartialEq)]
pub struct Binding {
    /// `KEY` or `MOU`.
    pub device: String,
    /// True for `make` (pressed), false for `break` (released).
    pub press: bool,
    /// Key name as written (`up`, `end`, `f2`...); for the mouse, the rest
    /// of the line (`R 0 0`).
    pub key: String,
    pub action: String,
    pub value: f32,
    pub flag: String,
}

/// Reads an action map. Lines that do not fit are returned as problems.
pub fn parse(bytes: &[u8]) -> (Vec<Binding>, Vec<String>) {
    let text: String = bytes.iter().map(|&b| b as char).collect();
    let mut bindings = Vec::new();
    let mut problems = Vec::new();
    let mut current: Option<Binding> = None;
    for (n, raw) in text.lines().enumerate() {
        let line = raw.trim();
        let Some(rest) = line.strip_prefix('[') else {
            if !line.is_empty() {
                problems.push(format!("line {}: '{line}'", n + 1));
            }
            continue;
        };
        let Some((tag, rest)) = rest.split_once(']') else {
            problems.push(format!("line {}: no ']' in '{line}'", n + 1));
            continue;
        };
        let words: Vec<&str> = rest.split_whitespace().collect();
        match tag {
            "ACTIONMAP" | "END" => {}
            "KEY" | "MOU" => {
                if let Some(b) = current.take() {
                    problems.push(format!("binding for '{}' has no action", b.key));
                }
                let press = match words.first() {
                    Some(&"make") => true,
                    Some(&"break") => false,
                    _ => {
                        problems.push(format!("line {}: expected make/break: '{line}'", n + 1));
                        continue;
                    }
                };
                current = Some(Binding {
                    device: tag.to_string(),
                    press,
                    key: words[1..].join(" ").to_ascii_lowercase(),
                    action: String::new(),
                    value: 0.0,
                    flag: String::new(),
                });
            }
            "ACTION" => match (current.as_mut(), words.first(), words.get(1).map(|v| v.parse::<f32>())) {
                (Some(b), Some(name), Some(Ok(value))) => {
                    b.action = name.to_string();
                    b.value = value;
                }
                _ => problems.push(format!("line {}: action without a key or value: '{line}'", n + 1)),
            },
            "ACTIONFLAG" => match current.take() {
                Some(mut b) if !b.action.is_empty() => {
                    b.flag = words.first().unwrap_or(&"").to_string();
                    bindings.push(b);
                }
                _ => problems.push(format!("line {}: flag without an action", n + 1)),
            },
            _ => problems.push(format!("line {}: unknown tag [{tag}]", n + 1)),
        }
    }
    if let Some(b) = current {
        problems.push(format!("binding for '{}' is not finished", b.key));
    }
    (bindings, problems)
}

/// The key (pressed) that sends `action` with `value`, if any.
pub fn key_for<'a>(bindings: &'a [Binding], action: &str, value: f32) -> Option<&'a str> {
    bindings
        .iter()
        .find(|b| b.press && b.device == "KEY" && b.action.eq_ignore_ascii_case(action) && b.value == value)
        .map(|b| b.key.as_str())
}

#[cfg(test)]
mod tests {
    use super::*;

    const SAMPLE: &str = "[ACTIONMAP]\n[KEY] make end\n[ACTION] Run 0.000000 0\n[ACTIONFLAG] ActionFlagNone\n[KEY] make up\n[ACTION] MoveY 1.000000 0\n[ACTIONFLAG] ActionFlagValue\n[KEY] break up\n[ACTION] MoveY 0.500000 0\n[ACTIONFLAG] ActionFlagValue\n[MOU] make R 0 0\n[ACTION] MoveCamera 1.000000 0\n[ACTIONFLAG] ActionFlagValue\n[END]\n";

    #[test]
    fn reads_bindings() {
        let (b, problems) = parse(SAMPLE.as_bytes());
        assert!(problems.is_empty(), "{problems:?}");
        assert_eq!(b.len(), 4);
        assert_eq!((b[1].press, b[1].key.as_str(), b[1].action.as_str(), b[1].value), (true, "up", "MoveY", 1.0));
        assert!(!b[2].press);
        assert_eq!((b[3].device.as_str(), b[3].key.as_str()), ("MOU", "r 0 0"));
        assert_eq!(key_for(&b, "MoveY", 1.0), Some("up"));
        assert_eq!(key_for(&b, "Run", 0.0), Some("end"));
        assert_eq!(key_for(&b, "MoveY", -1.0), None);
    }
}
