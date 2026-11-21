//! Help popups (`doPopup`, `stopHelp`, `notStartingHelp`, F1 help book, built-in beginner tips).
//!
//! Rules from `docs/behaviour/interface.md` rules 53–59 (`500.MSG`, `movekeys.kq`, `Options.cs`).

use std::collections::VecDeque;

/// Delay from `doPopup` (and several built-in tips) until Connor checks whether
/// the popup can open (`interface.md` rule 53), in game seconds.
pub const DELAY_SECS: f64 = 1.5;
/// Retry interval when deferred (`handsOff`, non-Wait/Move state, speaking, <3 s cooldown; rule 54).
pub const RETRY_SECS: f64 = 1.0;
/// Cooldown between popups (and re-queue delay if another popup is already open; rule 54).
pub const COOLDOWN_SECS: f64 = 3.0;
/// `500.MSG` condition numbers below this belong to "Help Getting Started" (`rule 55`).
pub const STARTING_HELP_LIMIT: u32 = 170;
/// Word-wrap limit and maximum line count in `IDCTL_POPUP` (`rule 55`).
pub const WRAP_CHARS: usize = 38;
pub const MAX_LINES: usize = 7;
/// `stopHelp` sets all 7 built-in tip bits (`0x7f`; `rule 58`).
pub const STOP_HELP_MASK: u8 = 0x7f;

pub const TIP_ROPE: u8 = 1 << 0;
pub const TIP_HEALTH_LOW: u8 = 1 << 1;
pub const TIP_HEALTH_ITEM: u8 = 1 << 2;
pub const TIP_QUEST_ITEM: u8 = 1 << 3;
pub const TIP_FIRST_KILL: u8 = 1 << 4;
pub const TIP_LEVEL_TWO: u8 = 1 << 5;
pub const TIP_EMPTY_HAND_HIT: u8 = 1 << 6;

/// Fixed 24-entry page table shown by F1 Help Book mode (`interface.md` rule 56).
pub const HELP_BOOK_ENTRIES: [u32; 24] = [
    127, 128, 129, 130, 131, 132, 133, 134, 135, 136, 137, 138, 139, 140, 141, 142, 143, 144, 145, 146, 149, 150, 151, 153,
];

/// One queued help cue waiting on Connor's timer.
#[derive(Debug, Clone, PartialEq)]
pub struct PendingPopup {
    pub text_number: u32,
    pub due: f64,
    pub from: String,
    pub last_defer: Option<&'static str>,
}

/// Active help dialog shown inside `IDCTL_POPUP`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HelpPopup {
    /// Normal gameplay tip or `>= 170` dialog (rule 55).
    Normal { text_number: u32, show_checkbox: bool },
    /// F1 Help Book browse mode (`HELP_BOOK_ENTRIES[index]`, rule 56).
    Book { index: usize },
}

/// Persistent + runtime help popup controller (`interface.md` rules 53–58).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct HelpState {
    /// `0` = starting help on, `1` = off by player, `2` = permanently disabled (`Options.cs` starts at `1`).
    pub not_starting_help: u8,
    /// 7-bit mask of built-in one-off tips already triggered or muted via `stopHelp` (`0x7f`).
    pub tip_bits: u8,
    pub pending: VecDeque<PendingPopup>,
    pub last_shown_at: Option<f64>,
    pub open: Option<HelpPopup>,
    /// Set once when a help popup opens so the viewer plays `intrface.aud`.
    pub just_opened: bool,
}

impl HelpState {
    pub fn is_open(&self) -> bool {
        self.open.is_some()
    }

    /// Queues a `doPopup <textNumber>` cue `delay` seconds from `now` (`rule 53`).
    pub fn queue(&mut self, text_number: u32, now: f64, delay: f64, from: impl Into<String>) {
        self.insert_pending(PendingPopup {
            text_number,
            due: now + delay.max(0.0),
            from: from.into(),
            last_defer: None,
        });
    }

    /// Fires a built-in one-off beginner tip unless its bit is already set (e.g. via `stopHelp`).
    /// Returns a log line when scheduled (or muted).
    pub fn queue_builtin_tip(&mut self, bit: u8, text_number: u32, now: f64, delay: f64, from: &str) -> Option<String> {
        if self.tip_bits & bit != 0 {
            return Some(format!("help: built-in tip #{text_number} ({from}) suppressed by tip_bits {:#x} (stopHelp / already asked)", self.tip_bits));
        }
        self.tip_bits |= bit;
        self.queue(text_number, now, delay, from);
        Some(format!("help: built-in tip #{text_number} ({from}) queued in {delay:.1} s at t={:.2} (interface.md rule 58)", now + delay))
    }

    pub fn set_not_starting_help(&mut self, mode: u8) {
        self.not_starting_help = mode.min(2);
    }

    /// `stopHelp` sets all 7 built-in tip bits (`0x7f`, `rule 58`).
    pub fn stop_help(&mut self) {
        self.tip_bits = STOP_HELP_MASK;
    }

    /// Checkbox toggle in `IDCTL_POPUP`: `0 <-> 1`; `2` cannot be re-enabled (`rule 55`).
    pub fn toggle_starting_help(&mut self) -> u8 {
        self.not_starting_help = match self.not_starting_help {
            0 => 1,
            1 => 0,
            other => other,
        };
        self.not_starting_help
    }

    /// Opens the F1 Help Book on entry 0 (`#127`) if no help popup is open (`rule 56`).
    pub fn open_book(&mut self) -> bool {
        if self.open.is_some() {
            return false;
        }
        self.open = Some(HelpPopup::Book { index: 0 });
        self.just_opened = true;
        true
    }

    /// Steps left (`-1`) or right (`+1`) in F1 Help Book mode, wrapping `0 <-> 23` (`rule 56`).
    pub fn page_book(&mut self, delta: isize) -> Option<usize> {
        let len = HELP_BOOK_ENTRIES.len() as isize;
        let Some(HelpPopup::Book { index }) = &mut self.open else { return None };
        let next = ((*index as isize + delta).rem_euclid(len)) as usize;
        *index = next;
        Some(next)
    }

    pub fn close(&mut self) -> Option<HelpPopup> {
        self.open.take()
    }

    fn insert_pending(&mut self, item: PendingPopup) {
        // Stable insertion by due time keeps FIFO ordering clean without the original's index-0 bug.
        let pos = self.pending.iter().position(|e| e.due > item.due + 1e-9).unwrap_or(self.pending.len());
        self.pending.insert(pos, item);
    }

    /// Advances queued popups at world time `now` (`rules 54–55`).
    pub fn tick(&mut self, now: f64, hands_off: bool, wait_or_move: bool, connor_speaking: bool, portal_asking: bool) -> Vec<String> {
        let mut logs = Vec::new();
        while let Some(front) = self.pending.front() {
            if front.due > now + 1e-9 {
                break;
            }
            if self.open.is_some() || portal_asking {
                let mut item = self.pending.pop_front().expect("checked");
                item.due = now + COOLDOWN_SECS;
                if item.last_defer != Some("popup already open") {
                    item.last_defer = Some("popup already open");
                    logs.push(format!("help: popup #{} deferred +{COOLDOWN_SECS:.1} s (already open; interface.md rule 54)", item.text_number));
                }
                self.insert_pending(item);
                break;
            }
            let reason = if hands_off {
                Some("handsOff")
            } else if !wait_or_move {
                Some("Connor not in Wait/Move")
            } else if connor_speaking {
                Some("Connor speaking")
            } else if self.last_shown_at.is_some_and(|t| now - t < COOLDOWN_SECS - 1e-9) {
                Some("cooldown < 3.0 s")
            } else {
                None
            };
            if let Some(why) = reason {
                let mut item = self.pending.pop_front().expect("checked");
                item.due = now + RETRY_SECS;
                if item.last_defer != Some(why) {
                    item.last_defer = Some(why);
                    logs.push(format!("help: popup #{} deferred +{RETRY_SECS:.1} s ({why}; interface.md rule 54)", item.text_number));
                }
                self.insert_pending(item);
                break;
            }
            let item = self.pending.pop_front().expect("checked");
            if item.text_number < STARTING_HELP_LIMIT && self.not_starting_help != 0 {
                logs.push(format!(
                    "help: skipped starting-help popup #{} (from {}; notStartingHelp={}; interface.md rule 55)",
                    item.text_number, item.from, self.not_starting_help
                ));
                continue;
            }
            self.last_shown_at = Some(now);
            let show_checkbox = item.text_number < STARTING_HELP_LIMIT;
            self.open = Some(HelpPopup::Normal { text_number: item.text_number, show_checkbox });
            self.just_opened = true;
            logs.push(format!(
                "help: opening #{} (from {}; checkbox={show_checkbox}; notStartingHelp={}; interface.md rule 55)",
                item.text_number, item.from, self.not_starting_help
            ));
            break;
        }
        logs
    }
}

/// Replaces `^<char>` keybinding placeholders (`^4`, `^5`, `^8`, `^9`, `^:` -> indices `4`, `5`, `8`, `9`, `10` in `key_names`; `rule 55`).
pub fn expand_key_placeholders(text: &str, key_names: &[String]) -> String {
    let mut out = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' {
            if let Some(&next) = chars.peek() {
                if (next as u32) >= b'0' as u32 {
                    let idx = (next as u32 - b'0' as u32) as usize;
                    if let Some(name) = key_names.get(idx).filter(|s| !s.is_empty()) {
                        chars.next();
                        out.push_str(name);
                        continue;
                    }
                }
            }
        }
        out.push(c);
    }
    out
}

/// Wraps help popup text on explicit newlines and at `38` chars per line, capped at `7` lines (`rule 55`).
pub fn wrap_help_text(text: &str) -> Vec<String> {
    let mut lines = Vec::new();
    for raw_para in text.split('\n') {
        let para = raw_para.trim_end_matches('\r');
        if para.trim().is_empty() {
            if lines.len() < MAX_LINES {
                lines.push(String::new());
            }
            continue;
        }
        let mut current = String::new();
        for word in para.split_whitespace() {
            let word_len = word.chars().count();
            if current.is_empty() {
                if word_len <= WRAP_CHARS {
                    current.push_str(word);
                } else {
                    push_long_word(&mut lines, word, &mut current);
                }
            } else if current.chars().count() + 1 + word_len <= WRAP_CHARS {
                current.push(' ');
                current.push_str(word);
            } else {
                lines.push(std::mem::take(&mut current));
                if lines.len() >= MAX_LINES {
                    return lines;
                }
                if word_len <= WRAP_CHARS {
                    current.push_str(word);
                } else {
                    push_long_word(&mut lines, word, &mut current);
                }
            }
            if lines.len() >= MAX_LINES {
                return lines;
            }
        }
        if !current.is_empty() && lines.len() < MAX_LINES {
            lines.push(current);
        }
        if lines.len() >= MAX_LINES {
            break;
        }
    }
    lines.truncate(MAX_LINES);
    lines
}

fn push_long_word(lines: &mut Vec<String>, word: &str, tail: &mut String) {
    let mut chunk = String::new();
    for ch in word.chars() {
        chunk.push(ch);
        if chunk.chars().count() == WRAP_CHARS {
            lines.push(std::mem::take(&mut chunk));
            if lines.len() >= MAX_LINES {
                return;
            }
        }
    }
    *tail = chunk;
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_keys() -> Vec<String> {
        vec![
            "Up Arrow".into(),
            "Down Arrow".into(),
            "Right Arrow".into(),
            "Left Arrow".into(),
            "End".into(),
            "Insert".into(),
            "Delete".into(),
            "Home".into(),
            "Tab".into(),
            "Shift".into(),
            "Ctrl".into(),
            "Alt".into(),
            "Space Bar".into(),
        ]
    }

    #[test]
    fn cue_delay_deferrals_cooldown_and_fifo_queue() {
        let mut h = HelpState::default();
        h.queue(133, 0.0, DELAY_SECS, "jumpa.cs");
        assert!(h.tick(1.4, false, true, false, false).is_empty());
        assert!(!h.is_open());

        // Due at 1.5 s, but deferred by each of the 4 reasons (+1.0 s retry each).
        let log1 = h.tick(1.5, true, true, false, false);
        assert!(log1[0].contains("handsOff"));
        assert!(!h.is_open());
        assert!(h.tick(2.4, false, true, false, false).is_empty());

        let log2 = h.tick(2.5, false, false, false, false);
        assert!(log2[0].contains("Wait/Move"));
        let log3 = h.tick(3.5, false, true, true, false);
        assert!(log3[0].contains("speaking"));
        assert!(!h.is_open());

        // Clear at 4.5 s -> opens #133 with checkbox (< 170).
        h.tick(4.5, false, true, false, false);
        assert_eq!(h.open, Some(HelpPopup::Normal { text_number: 133, show_checkbox: true }));
        assert!(std::mem::take(&mut h.just_opened));

        // Second cue arrives while #133 is open -> re-queued +3.0 s.
        h.queue(175, 4.5, 0.0, "high");
        let open_log = h.tick(4.5, false, true, false, false);
        assert!(open_log[0].contains("already open"));
        assert_eq!(h.close(), Some(HelpPopup::Normal { text_number: 133, show_checkbox: true }));

        // At 7.4 s: still < 3.0 s since 4.5 s (note due was bumped to 7.5 anyway), opens cleanly at 7.5 s without checkbox (>= 170).
        h.tick(7.5, false, true, false, false);
        assert_eq!(h.open, Some(HelpPopup::Normal { text_number: 175, show_checkbox: false }));
    }

    #[test]
    fn not_starting_help_and_stop_help_and_book_wrapping() {
        let mut h = HelpState { not_starting_help: 1, ..Default::default() };
        h.queue(133, 0.0, 0.0, "starting");
        h.queue(170, 0.0, 0.0, "always");
        let logs = h.tick(0.0, false, true, false, false);
        assert!(logs.iter().any(|l| l.contains("skipped starting-help popup #133")));
        assert_eq!(h.open, Some(HelpPopup::Normal { text_number: 170, show_checkbox: false }));
        h.close();

        // Checkbox toggles 0 <-> 1, never re-enables 2.
        h.set_not_starting_help(0);
        assert_eq!(h.toggle_starting_help(), 1);
        assert_eq!(h.toggle_starting_help(), 0);
        h.set_not_starting_help(2);
        assert_eq!(h.toggle_starting_help(), 2);

        // stopHelp sets 0x7f, muting all 7 built-in tips.
        h.stop_help();
        assert_eq!(h.tip_bits, STOP_HELP_MASK);
        let msg = h.queue_builtin_tip(TIP_FIRST_KILL, 137, 5.0, 1.5, "kill").unwrap();
        assert!(msg.contains("suppressed"));
        assert!(h.pending.is_empty());

        // F1 Help Book wraps 0 <-> 23 across all 24 entries.
        assert!(h.open_book());
        assert_eq!(h.open, Some(HelpPopup::Book { index: 0 }));
        assert_eq!(h.page_book(-1), Some(23));
        assert_eq!(h.open, Some(HelpPopup::Book { index: 23 }));
        assert_eq!(HELP_BOOK_ENTRIES[23], 153);
        assert_eq!(h.page_book(1), Some(0));
        assert_eq!(h.open, Some(HelpPopup::Book { index: 0 }));
        assert_eq!(HELP_BOOK_ENTRIES[0], 127);
    }

    #[test]
    fn key_placeholders_and_wrapping_follow_rules() {
        let keys = sample_keys();
        assert_eq!(expand_key_placeholders("Run [^4], Jump [^5], Map [^8], Tilt [^9], Play [^:].", &keys), "Run [End], Jump [Insert], Map [Tab], Tilt [Shift], Play [Ctrl].");

        let long = "CAMERA: To move the camera in third-person mode, right-click-hold and move the mouse. Forward/backward movement zooms in or out. Right/left pans around Connor.";
        let wrapped = wrap_help_text(long);
        assert!(wrapped.len() <= MAX_LINES);
        assert!(wrapped.iter().all(|l| l.chars().count() <= WRAP_CHARS), "{wrapped:?}");
        assert_eq!(wrap_help_text("a\nb\nc\nd\ne\nf\ng\nh").len(), 7);
    }
}
