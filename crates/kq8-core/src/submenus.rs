//! Main menu submenus (`docs/behaviour/submenus.md`, M4 step 4g).
//!
//! Pure-Rust rules for:
//! - Screen switching (`MenuScreen`, `submenus.md` rule 1)
//! - `Options.cs` persistence (`UserOptions`, `submenus.md` rule 16)
//! - Options menu (`Options.gui`, `submenus.md` rules 12–16)
//! - Graphics menu (`Graphics.gui`, `submenus.md` rules 17–19)
//! - Remap Move Keys (`movekey.gui`, `movekeys.kq`, `submenus.md` rules 20–22)
//! - Credits roll (`credits.gui`, `400.MSG`, `submenus.md` rules 23–25, `Q206`)

use std::collections::BTreeMap;

use kq8_data::msg::MessageFile;

/// GUI screens reachable from the Main Menu (`submenus.md` rule 1).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuScreen {
    MainMenu = 0,
    Options = 2,
    Graphics = 4,
    LoadGame = 5,
    SaveGame = 6,
    MoveKeys = 7,
    Credits = 8,
}

/// Persisted user settings matching the 10 lines of `Options.cs`
/// (`submenus.md` rule 16).
#[derive(Debug, Clone, PartialEq)]
pub struct UserOptions {
    pub speech_vol: f32,
    pub sfx_vol: f32,
    pub music_vol: f32,
    pub shadow_mask: u32,
    pub dynamic_lighting: bool,
    pub captions: bool,
    /// `0` = Starting Help on (`Yes`), `1` = off (`No`), `2` = permanently locked off (`No`).
    pub not_starting_help: u8,
    pub gmode_name: String,
    pub brightness: i32,
    pub mode_800: bool,
}

impl Default for UserOptions {
    fn default() -> Self {
        UserOptions {
            speech_vol: 1.0,
            sfx_vol: 1.0,
            music_vol: 1.0,
            shadow_mask: 15,
            dynamic_lighting: true,
            captions: true,
            not_starting_help: 0,
            gmode_name: "Direct3D".to_string(),
            brightness: 50,
            mode_800: true,
        }
    }
}

fn parse_bool_word(w: &str) -> Option<bool> {
    match w.to_ascii_lowercase().as_str() {
        "1" | "true" => Some(true),
        "0" | "false" => Some(false),
        _ => None,
    }
}

impl UserOptions {
    /// Parses `Options.cs` commands (`submenus.md` rule 16).
    pub fn parse(bytes: &[u8]) -> Self {
        let mut opts = Self::default();
        let text = String::from_utf8_lossy(bytes);
        for raw in text.lines() {
            let line = raw.split("//").next().unwrap_or("").split('#').next().unwrap_or("").trim();
            if line.is_empty() {
                continue;
            }
            let mut words = line.split_whitespace();
            let Some(cmd) = words.next() else { continue };
            match cmd.to_ascii_lowercase().as_str() {
                "kqsound::setspeechvol" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<f32>().ok()) {
                        opts.speech_vol = v.clamp(0.0, 1.0);
                    }
                }
                "kqsound::setsfxvol" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<f32>().ok()) {
                        opts.sfx_vol = v.clamp(0.0, 1.0);
                    }
                }
                "kqsound::setmusicvol" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<f32>().ok()) {
                        opts.music_vol = v.clamp(0.0, 1.0);
                    }
                }
                "setshadowmask" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<u32>().ok()) {
                        opts.shadow_mask = v & 0x0F;
                    }
                }
                "usedynamiclighting" => {
                    if let Some(b) = words.next().and_then(parse_bool_word) {
                        opts.dynamic_lighting = b;
                    }
                }
                "set" => {
                    let var = words.next();
                    let val = words.next();
                    if let (Some(name), Some(v)) = (var, val) {
                        if name.eq_ignore_ascii_case("Options::Captions") {
                            if let Some(b) = parse_bool_word(v) {
                                opts.captions = b;
                            }
                        }
                    }
                }
                "notstartinghelp" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<u8>().ok()) {
                        opts.not_starting_help = v.min(2);
                    }
                }
                "assigngmodename" => {
                    if let Some(name) = words.next() {
                        opts.gmode_name = name.to_string();
                    }
                }
                "setbrightness" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<i32>().ok()) {
                        opts.brightness = v.clamp(0, 100);
                    }
                }
                "set800mode" => {
                    if let Some(v) = words.next().and_then(|w| w.parse::<i32>().ok()) {
                        opts.mode_800 = v != 0;
                    }
                }
                _ => {}
            }
        }
        opts
    }

    /// Formats the 10 console commands written back to `Options.cs` (`submenus.md` rule 16).
    pub fn format(&self) -> String {
        format!(
            "KQSound::setSpeechVol {:.2}\r\n\
             KQSound::setSfxVol {:.2}\r\n\
             KQSound::setMusicVol {:.2}\r\n\
             setShadowMask {}\r\n\
             useDynamicLighting {}\r\n\
             set Options::Captions {}\r\n\
             notStartingHelp {}\r\n\
             assignGModeName {}\r\n\
             setBrightness {}\r\n\
             set800mode {}\r\n",
            self.speech_vol.clamp(0.0, 1.0),
            self.sfx_vol.clamp(0.0, 1.0),
            self.music_vol.clamp(0.0, 1.0),
            self.shadow_mask & 0x0F,
            if self.dynamic_lighting { "true" } else { "false" },
            if self.captions { "TRUE" } else { "FALSE" },
            self.not_starting_help.min(2),
            self.gmode_name,
            self.brightness.clamp(0, 100),
            u8::from(self.mode_800),
        )
    }
}

/// Converts a `0..=100` slider position into a `0.0..=1.0` volume with the
/// 10% floor for non-zero values (`submenus.md` rule 14).
pub fn slider_to_volume(val: i32) -> f32 {
    let v = val.clamp(0, 100);
    if v == 0 {
        0.0
    } else {
        v.max(10) as f32 / 100.0
    }
}

/// Converts a `0.0..=1.0` volume into an integer slider position in `0..=100`.
pub fn volume_to_slider(vol: f32) -> i32 {
    (vol.clamp(0.0, 1.0) * 100.0).round() as i32
}

/// Steps an integer `0..=100` slider by `-5` or `+5`, snapped to a multiple of 5
/// (`submenus.md` rule 13).
pub fn step_slider_5(current: i32, delta: i32) -> i32 {
    let cur = current.clamp(0, 100);
    let next = if delta > 0 {
        ((cur / 5) + 1) * 5
    } else if delta < 0 {
        if cur % 5 == 0 {
            cur - 5
        } else {
            (cur / 5) * 5
        }
    } else {
        cur
    };
    next.clamp(0, 100)
}

/// Maps a 640×480 layout X coordinate across a `330..=530` slider track to `0..=100`.
pub fn slider_from_x(x: f32) -> i32 {
    (((x - SLIDER_LEFT as f32) / (SLIDER_RIGHT - SLIDER_LEFT) as f32) * 100.0)
        .round()
        .clamp(0.0, 100.0) as i32
}

/// 640×480 layout constants from `Options.gui` (`gui-verify.log`).
pub const OPT_TITLE_RECT: [i32; 4] = [10, 10, 630, 55];
pub const OPT_CURSOR_SIZE: i32 = 30;
pub const OPT_CURSOR1_X: i32 = 10;
pub const OPT_CURSOR2_X: i32 = 600;
pub const SLIDER_LEFT: i32 = 330;
pub const SLIDER_RIGHT: i32 = 530;
pub const SLIDER_KNOB_SIZE: i32 = 10;

pub const OPT_ROW_COUNT: usize = 9;

/// Layout rectangles (640×480) for the 9 Options rows (`Options.gui`).
pub const OPT_ROW_RECTS: [[i32; 4]; OPT_ROW_COUNT] = [
    [40, 80, 600, 110],
    [40, 130, 600, 160],
    [40, 180, 600, 210],
    [40, 210, 600, 240],
    [40, 240, 600, 270],
    [40, 270, 600, 300],
    [40, 300, 600, 330],
    [40, 347, 600, 377],
    [40, 430, 600, 470],
];

pub const OPT_LABEL_RECTS: [[i32; 4]; OPT_ROW_COUNT] = [
    [40, 80, 600, 110],
    [40, 130, 600, 160],
    [40, 180, 310, 210],
    [40, 210, 310, 240],
    [40, 240, 310, 270],
    [40, 270, 310, 300],
    [40, 300, 310, 330],
    [40, 347, 600, 377],
    [40, 430, 600, 470],
];

/// Slider track rectangles (`Options.gui`, `IDCTL_OPT_*VOLUME_VALUE`).
pub const OPT_SLIDER_RECTS: [(usize, [i32; 4]); 3] = [
    (2, [330, 193, 530, 203]),
    (3, [330, 223, 530, 233]),
    (4, [330, 253, 530, 263]),
];

pub const OPT_VALUE_RECTS: [(usize, [i32; 4]); 2] = [
    (5, [330, 270, 600, 300]),
    (6, [330, 300, 600, 330]),
];

/// Strings from `500.MSG` used by `Options.gui`, `Graphics.gui`, and `movekey.gui`.
#[derive(Debug, Clone)]
pub struct SubmenuTexts {
    pub options_title: String,
    pub opt_rows: [String; OPT_ROW_COUNT],
    pub graphics_title: String,
    pub gra_mode_label: String,
    pub gra_res_label: String,
    pub gra_rows: [String; GRA_ROW_COUNT],
    pub movekeys_title: String,
    pub move_rows: [String; MOVE_KEY_COUNT],
    pub back_to_options: String,
    pub press_desired_key: String,
    pub invalid_key: String,
    pub yes: String,
    pub no: String,
    pub on: String,
    pub off: String,
    pub msg_cases: BTreeMap<u32, String>,
}

impl Default for SubmenuTexts {
    fn default() -> Self {
        SubmenuTexts {
            options_title: "Options".into(),
            opt_rows: [
                "Graphics".into(),
                "Keyboard".into(),
                "Music Volume".into(),
                "Sound Effects Volume".into(),
                "Speech Volume".into(),
                "Subtitles".into(),
                "Help Getting Started".into(),
                "Credits".into(),
                "Back to Main Menu".into(),
            ],
            graphics_title: "Graphic Settings".into(),
            gra_mode_label: "Graphics Mode".into(),
            gra_res_label: "Resolution".into(),
            gra_rows: [
                "DirectDraw".into(),
                "Direct3D".into(),
                "3Dfx".into(),
                "640 x 480".into(),
                "800 x 600".into(),
                "Dynamic Lighting".into(),
                "Connor Shadows".into(),
                "Monster Shadows".into(),
                "Character Shadows".into(),
                "Object Shadows".into(),
                "Brightness".into(),
                "Back to Options Menu".into(),
            ],
            movekeys_title: "Keyboard".into(),
            move_rows: [
                "Forward".into(),
                "Back".into(),
                "Right".into(),
                "Left".into(),
                "Walk/Run".into(),
                "Jump".into(),
                "Switch Weapon".into(),
                "Center Camera".into(),
                "Open/Close Map".into(),
                "Modify Action".into(),
                "Toggle Cursor".into(),
                "Put Away Weapon".into(),
                "Open/Close Interface".into(),
            ],
            back_to_options: "Back to Options Menu".into(),
            press_desired_key: "Press Desired Key".into(),
            invalid_key: "Key in use, TRY AGAIN".into(),
            yes: "Yes".into(),
            no: "No".into(),
            on: "On".into(),
            off: "Off".into(),
            msg_cases: BTreeMap::new(),
        }
    }
}

impl SubmenuTexts {
    pub fn from_msg(msg: Option<&MessageFile>) -> Self {
        let mut out = Self::default();
        let Some(f) = msg else { return out };
        for l in &f.lines {
            if l.tuple.noun == 0 && l.tuple.verb == 0 {
                let s = l.raw.trim().to_string();
                if !s.is_empty() {
                    out.msg_cases.entry(l.tuple.case as u32).or_insert(s);
                }
            }
        }
        let get = |case: u32| -> Option<String> { out.msg_cases.get(&case).cloned() };
        if let Some(s) = get(19) { out.options_title = s; }
        let opt_cases = [33, 54, 12, 25, 26, 27, 152, 168, 3];
        for (i, c) in opt_cases.into_iter().enumerate() {
            if let Some(s) = get(c) {
                out.opt_rows[i] = s;
            }
        }
        if let Some(s) = get(32) { out.graphics_title = s; }
        if let Some(s) = get(9) { out.gra_mode_label = s; }
        if let Some(s) = get(23) { out.gra_res_label = s; }
        let gra_cases = [7, 8, 1, 116, 117, 31, 6, 11, 5, 15, 4, 34];
        for (i, c) in gra_cases.into_iter().enumerate() {
            if let Some(s) = get(c) {
                out.gra_rows[i] = s;
            }
        }
        if let Some(s) = get(54) { out.movekeys_title = s; }
        let move_cases = [55, 56, 57, 58, 59, 60, 61, 68, 69, 64, 65, 113, 66];
        for (i, c) in move_cases.into_iter().enumerate() {
            if let Some(s) = get(c) {
                out.move_rows[i] = s;
            }
        }
        if let Some(s) = get(34) { out.back_to_options = s; }
        if let Some(s) = get(112) { out.press_desired_key = s; }
        if let Some(s) = get(67) { out.invalid_key = s; }
        if let Some(s) = get(29) { out.yes = s; }
        if let Some(s) = get(14) { out.no = s; }
        if let Some(s) = get(17) { out.on = s; }
        if let Some(s) = get(16) { out.off = s; }
        out
    }

    /// Resolves a `KeyName<n>` tag such as `"#84"` from `500.MSG`, or returns literal text.
    pub fn resolve_tag<'a>(&'a self, tag: &'a str) -> &'a str {
        let trimmed = tag.trim();
        if let Some(num_str) = trimmed.strip_prefix('#') {
            if let Ok(num) = num_str.parse::<u32>() {
                if let Some(s) = self.msg_cases.get(&num) {
                    return s.as_str();
                }
            }
        }
        trimmed
    }
}

/// Actions emitted by `OptionsState`, `GraphicsState`, `MoveKeysState`, and `CreditsState`.
#[derive(Debug, Clone, PartialEq)]
pub enum SubmenuAction {
    PlaySound(&'static str),
    UpdateVolumes { music: f32, sfx: f32, speech: f32 },
    ApplyGraphics { dynamic_lighting: bool, shadow_mask: u32, brightness: i32, mode_800: bool },
    SwitchScreen(MenuScreen),
    SaveOptionsAndReturnToMain,
    SaveMoveKeysAndReturnToOptions,
    ExitCreditsToMain,
    Note(String),
}

/// State machine for `Options.gui` (`submenus.md` rules 12–16).
#[derive(Debug, Clone)]
pub struct OptionsState {
    pub selected: usize,
    /// Integer `0..=100` slider positions matching Music, SFX, Speech.
    pub music_slider: i32,
    pub sfx_slider: i32,
    pub speech_slider: i32,
}

impl Default for OptionsState {
    fn default() -> Self {
        Self::from_options(&UserOptions::default())
    }
}

impl OptionsState {
    pub fn from_options(opts: &UserOptions) -> Self {
        OptionsState {
            selected: 0,
            music_slider: volume_to_slider(opts.music_vol),
            sfx_slider: volume_to_slider(opts.sfx_vol),
            speech_slider: volume_to_slider(opts.speech_vol),
        }
    }

    /// Re-syncs sliders on opening Options (`submenus.md` rule 14).
    pub fn reset_on_open(&mut self, opts: &UserOptions) {
        self.selected = 0;
        self.music_slider = volume_to_slider(opts.music_vol);
        self.sfx_slider = volume_to_slider(opts.sfx_vol);
        self.speech_slider = volume_to_slider(opts.speech_vol);
    }

    /// Row 6 (`Starting Help`) is disabled when `not_starting_help == 2` (`submenus.md` rule 15).
    pub fn is_row_enabled(row: usize, opts: &UserOptions) -> bool {
        row < OPT_ROW_COUNT && !(row == 6 && opts.not_starting_help == 2)
    }

    pub fn cursor_rects(&self) -> [[i32; 4]; 2] {
        let r = OPT_ROW_RECTS[self.selected.min(OPT_ROW_COUNT - 1)];
        let top = r[1] + ((r[3] - r[1]) - OPT_CURSOR_SIZE) / 2;
        [
            [OPT_CURSOR1_X, top, OPT_CURSOR1_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
            [OPT_CURSOR2_X, top, OPT_CURSOR2_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
        ]
    }

    pub fn move_selection(&mut self, delta: i32, opts: &UserOptions) -> Vec<SubmenuAction> {
        if delta == 0 {
            return Vec::new();
        }
        let step = if delta > 0 { 1isize } else { -1isize };
        let mut idx = self.selected as isize;
        for _ in 0..OPT_ROW_COUNT {
            idx = (idx + step).rem_euclid(OPT_ROW_COUNT as isize);
            let cand = idx as usize;
            if Self::is_row_enabled(cand, opts) {
                self.selected = cand;
                return vec![
                    SubmenuAction::PlaySound("click.aud"),
                    SubmenuAction::Note(format!("options cursor -> row {cand}")),
                ];
            }
        }
        Vec::new()
    }

    pub fn adjust_left_right(&mut self, delta: i32, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        if delta == 0 {
            return Vec::new();
        }
        match self.selected {
            2..=4 => self.step_slider(self.selected, delta, opts),
            5 => self.toggle_subtitles(opts),
            6 => self.toggle_starting_help(opts),
            _ => Vec::new(),
        }
    }

    fn step_slider(&mut self, row: usize, delta: i32, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        let val = match row {
            2 => {
                self.music_slider = step_slider_5(self.music_slider, delta);
                opts.music_vol = slider_to_volume(self.music_slider);
                self.music_slider
            }
            3 => {
                self.sfx_slider = step_slider_5(self.sfx_slider, delta);
                opts.sfx_vol = slider_to_volume(self.sfx_slider);
                self.sfx_slider
            }
            4 => {
                self.speech_slider = step_slider_5(self.speech_slider, delta);
                opts.speech_vol = slider_to_volume(self.speech_slider);
                self.speech_slider
            }
            _ => return Vec::new(),
        };
        vec![
            SubmenuAction::UpdateVolumes {
                music: opts.music_vol,
                sfx: opts.sfx_vol,
                speech: opts.speech_vol,
            },
            SubmenuAction::Note(format!("options slider row {row} -> {val}% (vols m={:.2} s={:.2} v={:.2})", opts.music_vol, opts.sfx_vol, opts.speech_vol)),
        ]
    }

    pub fn set_slider_from_x(&mut self, row: usize, x: f32, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        let val = slider_from_x(x);
        self.selected = row;
        match row {
            2 => {
                if self.music_slider == val {
                    return Vec::new();
                }
                self.music_slider = val;
                opts.music_vol = slider_to_volume(val);
            }
            3 => {
                if self.sfx_slider == val {
                    return Vec::new();
                }
                self.sfx_slider = val;
                opts.sfx_vol = slider_to_volume(val);
            }
            4 => {
                if self.speech_slider == val {
                    return Vec::new();
                }
                self.speech_slider = val;
                opts.speech_vol = slider_to_volume(val);
            }
            _ => return Vec::new(),
        }
        vec![
            SubmenuAction::UpdateVolumes {
                music: opts.music_vol,
                sfx: opts.sfx_vol,
                speech: opts.speech_vol,
            },
            SubmenuAction::Note(format!("options slider drag row {row} -> {val}%")),
        ]
    }

    fn toggle_subtitles(&self, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        opts.captions = !opts.captions;
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            SubmenuAction::Note(format!("options Subtitles -> {}", if opts.captions { "Yes" } else { "No" })),
        ]
    }

    fn toggle_starting_help(&self, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        if opts.not_starting_help == 2 {
            return Vec::new();
        }
        opts.not_starting_help = if opts.not_starting_help == 0 { 1 } else { 0 };
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            SubmenuAction::Note(format!("options Starting Help -> {} (notStartingHelp={})", if opts.not_starting_help == 0 { "Yes" } else { "No" }, opts.not_starting_help)),
        ]
    }

    pub fn activate(&mut self, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        match self.selected {
            0 => vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SwitchScreen(MenuScreen::Graphics),
            ],
            1 => vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SwitchScreen(MenuScreen::MoveKeys),
            ],
            5 => self.toggle_subtitles(opts),
            6 => self.toggle_starting_help(opts),
            7 => vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SwitchScreen(MenuScreen::Credits),
            ],
            8 => vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SaveOptionsAndReturnToMain,
            ],
            _ => Vec::new(),
        }
    }

    pub fn press_escape(&mut self) -> Vec<SubmenuAction> {
        vec![SubmenuAction::SaveOptionsAndReturnToMain]
    }

    /// Handles a left mouse click at 640×480 layout coords. Returns `Some(slider_row)`
    /// when a slider track was clicked so dragging can continue.
    pub fn click_at(&mut self, pos: [f32; 2], opts: &mut UserOptions) -> (Vec<SubmenuAction>, Option<usize>) {
        for &(row, rect) in &OPT_SLIDER_RECTS {
            if point_in_rect(pos, rect) {
                return (self.set_slider_from_x(row, pos[0], opts), Some(row));
            }
        }
        for (row, &rect) in OPT_ROW_RECTS.iter().enumerate() {
            if point_in_rect(pos, rect) {
                if !Self::is_row_enabled(row, opts) {
                    return (Vec::new(), None);
                }
                self.selected = row;
                return (self.activate(opts), None);
            }
        }
        (Vec::new(), None)
    }
}

/// Credits layout constants (`credits.gui`, `submenus.md` rules 23–24).
pub const CREDITS_PHOTO_FILE: &str = "kqteam08.pbm";
pub const CREDITS_PHOTO_RECT: [i32; 4] = [120, 10, 520, 310];
pub const CREDITS_LARGE_HEIGHT: i32 = 30;
pub const CREDITS_SMALL_HEIGHT: i32 = 17;
pub const CREDITS_MAX_LARGE_LINES: usize = 13;
pub const CREDITS_MAX_SMALL_LINES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditsLine {
    pub text: String,
    pub small: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreditsPage {
    pub lines: Vec<CreditsLine>,
    pub duration_ms: u64,
    pub show_photo: bool,
}

impl CreditsPage {
    pub fn from_text(raw: &str, is_final: bool, score: i64) -> Self {
        let formatted = if is_final {
            raw.replace("%d", &score.to_string())
        } else {
            raw.to_string()
        };
        let mut lines = Vec::new();
        let mut large_count = 0usize;
        let mut small_count = 0usize;
        for part in formatted.split('\n') {
            let line = part.strip_suffix('\r').unwrap_or(part);
            if let Some(rest) = line.strip_prefix('*') {
                if small_count < CREDITS_MAX_SMALL_LINES {
                    small_count += 1;
                    lines.push(CreditsLine { text: rest.to_string(), small: true });
                }
            } else if large_count < CREDITS_MAX_LARGE_LINES {
                large_count += 1;
                lines.push(CreditsLine { text: line.to_string(), small: false });
            }
        }
        let n = lines.len() as u64;
        let duration_ms = if is_final {
            14_000 + 500 * n
        } else {
            2_000 + 500 * n
        };
        CreditsPage {
            lines,
            duration_ms,
            show_photo: is_final,
        }
    }

    pub fn line_rects(&self) -> Vec<([i32; 4], &CreditsLine)> {
        let total_h: i32 = self
            .lines
            .iter()
            .map(|l| if l.small { CREDITS_SMALL_HEIGHT } else { CREDITS_LARGE_HEIGHT })
            .sum();
        let mut y = (480 - total_h) / 2;
        let mut out = Vec::with_capacity(self.lines.len());
        for line in &self.lines {
            let h = if line.small { CREDITS_SMALL_HEIGHT } else { CREDITS_LARGE_HEIGHT };
            out.push(([0, y, 640, y + h], line));
            y += h;
        }
        out
    }
}

/// Timed Credits roll (`400.MSG` cases `1..=25`, `submenus.md` rules 23–25, `Q206`).
#[derive(Debug, Clone)]
pub struct CreditsState {
    pub pages: Vec<CreditsPage>,
    pub page: usize,
    pub elapsed_ms: f64,
}

impl CreditsState {
    /// Builds the credits pages from `400.MSG` (`noun = 0, verb = 0, case = 1..=25, seq = 1`).
    pub fn from_msg(msg: Option<&MessageFile>, score: i64) -> Self {
        let mut entries: Vec<(u8, String)> = Vec::new();
        if let Some(f) = msg {
            for l in &f.lines {
                if l.tuple.noun == 0 && l.tuple.verb == 0 && l.tuple.seq == 1 && l.tuple.case >= 1 {
                    if !entries.iter().any(|(c, _)| *c == l.tuple.case) {
                        entries.push((l.tuple.case, l.raw.clone()));
                    }
                }
            }
            entries.sort_by_key(|(c, _)| *c);
        }
        if entries.is_empty() {
            entries.push((1, "Team Mask of Eternity\r\n*You have scored %d points!".to_string()));
        }
        let last_idx = entries.len() - 1;
        let pages = entries
            .into_iter()
            .enumerate()
            .map(|(i, (_, text))| CreditsPage::from_text(&text, i == last_idx, score))
            .collect();
        CreditsState {
            pages,
            page: 0,
            elapsed_ms: 0.0,
        }
    }

    pub fn current(&self) -> Option<&CreditsPage> {
        self.pages.get(self.page)
    }

    /// Advances the timer by `dt_ms`. Returns `true` when the final page finishes
    /// and Credits should exit to the Main Menu (`submenus.md` rule 25).
    pub fn tick(&mut self, dt_ms: f64) -> bool {
        self.elapsed_ms += dt_ms.max(0.0);
        while let Some(cur) = self.pages.get(self.page) {
            let dur = cur.duration_ms as f64;
            if self.elapsed_ms < dur {
                return false;
            }
            self.elapsed_ms -= dur;
            if self.page + 1 >= self.pages.len() {
                return true;
            }
            self.page += 1;
        }
        true
    }
}

pub(crate) fn point_in_rect(pos: [f32; 2], rect: [i32; 4]) -> bool {
    pos[0] >= rect[0] as f32 && pos[0] < rect[2] as f32 && pos[1] >= rect[1] as f32 && pos[1] < rect[3] as f32
}

/// Graphics menu (`Graphics.gui`, `submenus.md` rules 17–19, "Do not reproduce").
pub const GRA_ROW_COUNT: usize = 12;
pub const GRA_TITLE_RECT: [i32; 4] = [10, 10, 630, 50];
pub const GRA_MODE_LABEL_RECT: [i32; 4] = [40, 50, 310, 80];
pub const GRA_RES_LABEL_RECT: [i32; 4] = [40, 150, 310, 180];
pub const GRA_BRIGHTNESS_SLIDER_RECT: [i32; 4] = [330, 383, 530, 393];

pub const GRA_ROW_RECTS: [[i32; 4]; GRA_ROW_COUNT] = [
    [330, 50, 600, 80],   // 0: DirectDraw (disabled in modern viewer)
    [330, 80, 600, 110],  // 1: Direct3D (informational active)
    [330, 110, 600, 140], // 2: 3Dfx (disabled)
    [330, 150, 600, 180], // 3: 640x480
    [330, 180, 600, 210], // 4: 800x600
    [40, 220, 600, 250],  // 5: Dynamic Lights
    [40, 250, 600, 280],  // 6: Connor Shadows (0x01)
    [40, 280, 600, 310],  // 7: Monster Shadows (0x04)
    [40, 310, 600, 340],  // 8: Character Shadows (0x02)
    [40, 340, 600, 370],  // 9: Object Shadows (0x08)
    [40, 370, 600, 400],  // 10: Brightness
    [40, 430, 600, 470],  // 11: Back to Options Menu
];

pub const GRA_LABEL_RECTS: [[i32; 4]; GRA_ROW_COUNT] = [
    [330, 50, 600, 80],
    [330, 80, 600, 110],
    [330, 110, 600, 140],
    [330, 150, 600, 180],
    [330, 180, 600, 210],
    [40, 220, 310, 250],
    [40, 250, 310, 280],
    [40, 280, 310, 310],
    [40, 310, 310, 340],
    [40, 340, 310, 370],
    [40, 370, 310, 400],
    [40, 430, 600, 470],
];

pub const GRA_VALUE_RECTS: [(usize, [i32; 4]); 5] = [
    (5, [330, 220, 600, 250]),
    (6, [330, 250, 600, 280]),
    (7, [330, 280, 600, 310]),
    (8, [330, 310, 600, 340]),
    (9, [330, 340, 600, 370]),
];

/// Shadow mask bits (`submenus.md` rule 17).
pub const GRA_SHADOW_BITS: [(usize, u32); 4] = [
    (6, 0x01), // Connor
    (7, 0x04), // Monster
    (8, 0x02), // Character
    (9, 0x08), // Object
];

/// Maps brightness slider `0..=100` (default `50`) linearly to `0.3..=1.7` (`submenus.md` rule 19).
pub fn brightness_factor(val: i32) -> f32 {
    0.3 + (val.clamp(0, 100) as f32 / 100.0) * 1.4
}

/// State machine for `Graphics.gui` (`submenus.md` rules 17–19, "Do not reproduce").
#[derive(Debug, Clone)]
pub struct GraphicsState {
    pub selected: usize,
}

impl Default for GraphicsState {
    fn default() -> Self {
        GraphicsState { selected: 3 }
    }
}

impl GraphicsState {
    pub fn reset_on_open(&mut self) {
        self.selected = 3;
    }

    /// Renderer mode rows `0..=2` are informational/disabled in our single-renderer viewer
    /// (`submenus.md` "Do not reproduce"); rows `3..=11` are interactive.
    pub fn is_row_enabled(row: usize) -> bool {
        (3..GRA_ROW_COUNT).contains(&row)
    }

    pub fn cursor_rects(&self) -> [[i32; 4]; 2] {
        let r = GRA_ROW_RECTS[self.selected.min(GRA_ROW_COUNT - 1)];
        let top = r[1] + ((r[3] - r[1]) - OPT_CURSOR_SIZE) / 2;
        [
            [OPT_CURSOR1_X, top, OPT_CURSOR1_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
            [OPT_CURSOR2_X, top, OPT_CURSOR2_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
        ]
    }

    pub fn move_selection(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if delta == 0 {
            return Vec::new();
        }
        let step = if delta > 0 { 1isize } else { -1isize };
        let mut idx = self.selected as isize;
        for _ in 0..GRA_ROW_COUNT {
            idx = (idx + step).rem_euclid(GRA_ROW_COUNT as isize);
            let cand = idx as usize;
            if Self::is_row_enabled(cand) {
                self.selected = cand;
                return vec![
                    SubmenuAction::PlaySound("click.aud"),
                    SubmenuAction::Note(format!("graphics cursor -> row {cand}")),
                ];
            }
        }
        Vec::new()
    }

    pub fn adjust_left_right(&mut self, delta: i32, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        if delta == 0 {
            return Vec::new();
        }
        match self.selected {
            3 => self.set_resolution(false, opts),
            4 => self.set_resolution(true, opts),
            5 => self.toggle_dynamic_lights(opts),
            6..=9 => self.toggle_shadow_row(self.selected, opts),
            10 => {
                opts.brightness = step_slider_5(opts.brightness, delta);
                vec![
                    Self::apply_action(opts),
                    SubmenuAction::Note(format!(
                        "graphics Brightness -> {} ({:.2}; submenus.md rule 19)",
                        opts.brightness,
                        brightness_factor(opts.brightness)
                    )),
                ]
            }
            _ => Vec::new(),
        }
    }

    pub fn set_brightness_from_x(&mut self, x: f32, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        self.selected = 10;
        let val = slider_from_x(x);
        if opts.brightness == val {
            return Vec::new();
        }
        opts.brightness = val;
        vec![
            Self::apply_action(opts),
            SubmenuAction::Note(format!(
                "graphics Brightness drag -> {} ({:.2})",
                val,
                brightness_factor(val)
            )),
        ]
    }

    fn apply_action(opts: &UserOptions) -> SubmenuAction {
        SubmenuAction::ApplyGraphics {
            dynamic_lighting: opts.dynamic_lighting,
            shadow_mask: opts.shadow_mask,
            brightness: opts.brightness,
            mode_800: opts.mode_800,
        }
    }

    fn set_resolution(&self, wide: bool, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        opts.mode_800 = wide;
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            Self::apply_action(opts),
            SubmenuAction::Note(format!("graphics resolution -> {}", if wide { "800x600" } else { "640x480" })),
        ]
    }

    fn toggle_dynamic_lights(&self, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        opts.dynamic_lighting = !opts.dynamic_lighting;
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            Self::apply_action(opts),
            SubmenuAction::Note(format!("graphics Dynamic Lights -> {}", if opts.dynamic_lighting { "On" } else { "Off" })),
        ]
    }

    fn toggle_shadow_row(&self, row: usize, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        let Some(&(_, bit)) = GRA_SHADOW_BITS.iter().find(|&&(r, _)| r == row) else {
            return Vec::new();
        };
        opts.shadow_mask ^= bit;
        let on = opts.shadow_mask & bit != 0;
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            Self::apply_action(opts),
            SubmenuAction::Note(format!("graphics shadow bit {bit:#x} (row {row}) -> {on} (mask={})", opts.shadow_mask)),
        ]
    }

    pub fn activate(&mut self, opts: &mut UserOptions) -> Vec<SubmenuAction> {
        match self.selected {
            3 => self.set_resolution(false, opts),
            4 => self.set_resolution(true, opts),
            5 => self.toggle_dynamic_lights(opts),
            6..=9 => self.toggle_shadow_row(self.selected, opts),
            11 => vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SwitchScreen(MenuScreen::Options),
            ],
            _ => Vec::new(),
        }
    }

    pub fn press_escape(&mut self) -> Vec<SubmenuAction> {
        vec![SubmenuAction::SwitchScreen(MenuScreen::Options)]
    }

    pub fn click_at(&mut self, pos: [f32; 2], opts: &mut UserOptions) -> (Vec<SubmenuAction>, bool) {
        if point_in_rect(pos, GRA_BRIGHTNESS_SLIDER_RECT) {
            return (self.set_brightness_from_x(pos[0], opts), true);
        }
        for (row, &rect) in GRA_ROW_RECTS.iter().enumerate() {
            if point_in_rect(pos, rect) {
                if !Self::is_row_enabled(row) {
                    return (Vec::new(), false);
                }
                self.selected = row;
                return (self.activate(opts), false);
            }
        }
        (Vec::new(), false)
    }
}

/// Remap Move Keys (`movekey.gui`, `movekeys.kq`, `submenus.md` rules 20–22).
pub const MOVE_KEY_COUNT: usize = 13;
pub const MOVE_ROW_COUNT: usize = 14;
pub const MOVE_TITLE_RECT: [i32; 4] = [10, 10, 630, 55];

pub fn move_label_rect(row: usize) -> [i32; 4] {
    if row >= MOVE_KEY_COUNT {
        [40, 430, 600, 470]
    } else {
        let top = 60 + row as i32 * 28;
        [40, top, 310, top + 28]
    }
}

pub fn move_value_rect(row: usize) -> [i32; 4] {
    let top = 60 + row as i32 * 28;
    [330, top, 600, top + 28]
}

pub fn move_row_rect(row: usize) -> [i32; 4] {
    if row >= MOVE_KEY_COUNT {
        [40, 430, 600, 470]
    } else {
        let top = 60 + row as i32 * 28;
        [40, top, 600, top + 28]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveKeyBinding {
    pub vk: u32,
    pub name_tag: String,
    pub keyword: String,
}

/// Default `movekeys.kq` bindings (`MASK/movekeys.kq`, `submenus.md` rule 20).
pub const DEFAULT_MOVE_KEYS: [(u32, &str, &str); MOVE_KEY_COUNT] = [
    (38, "#84", "up"),
    (40, "#85", "down"),
    (39, "#81", "right"),
    (37, "#83", "left"),
    (35, "#80", "end"),
    (45, "#82", "insert"),
    (46, "#86", "delete"),
    (36, "#92", "home"),
    (9, "#93", "tab"),
    (16, "#89", "shift"),
    (17, "#90", "control"),
    (18, "#106", "alt"),
    (32, "#91", "space"),
];

/// Reserved Windows virtual key codes that cannot be bound (`submenus.md` rule 21).
pub fn is_reserved_vk(vk: u32) -> bool {
    matches!(
        vk,
        0x71 // F2
        | 0x7A // F11
        | 0x7B // F12
        | 0x1B // Escape
        | 0x0D // Enter
        | 0x21 // Page Up
        | 0x22 // Page Down
        | 0x31..=0x38 // Top-row 1..8
        | 0x61..=0x69 // Numpad 1..9
    )
}

/// Bindable Windows virtual key table: returns `(KeyName tag/text, KeyWord)`.
pub fn bindable_key_info(vk: u32) -> Option<(String, String)> {
    if is_reserved_vk(vk) {
        return None;
    }
    let pair: (&str, &str) = match vk {
        0x08 => ("#107", "backspace"),
        0x09 => ("#93", "tab"),
        0x10 => ("#89", "shift"),
        0x11 => ("#90", "control"),
        0x12 => ("#106", "alt"),
        0x20 => ("#91", "space"),
        0x23 => ("#80", "end"),
        0x24 => ("#92", "home"),
        0x25 => ("#83", "left"),
        0x26 => ("#84", "up"),
        0x27 => ("#81", "right"),
        0x28 => ("#85", "down"),
        0x2D => ("#82", "insert"),
        0x2E => ("#86", "delete"),
        0x30 => ("0", "0"),
        0x39 => ("9", "9"),
        0x41..=0x5A => {
            let ch = (vk as u8) as char;
            let upper = ch.to_string();
            let lower = ch.to_ascii_lowercase().to_string();
            return Some((upper, lower));
        }
        0x60 => ("Numpad 0", "numpad0"),
        0x6A => ("Numpad *", "multiply"),
        0x6B => ("Numpad +", "add"),
        0x6D => ("Numpad -", "subtract"),
        0x6E => ("Numpad .", "decimal"),
        0x6F => ("Numpad /", "divide"),
        0x70 => ("#94", "f1"),
        0x72 => ("#96", "f3"),
        0x73 => ("#97", "f4"),
        0x74 => ("#98", "f5"),
        0x75 => ("#99", "f6"),
        0x76 => ("#100", "f7"),
        0x77 => ("#101", "f8"),
        0x78 => ("#102", "f9"),
        0x79 => ("#103", "f10"),
        0xBA => (";", ";"),
        0xBB => ("=", "="),
        0xBC => (",", ","),
        0xBD => ("-", "-"),
        0xBE => (".", "."),
        0xBF => ("/", "/"),
        0xC0 => ("#115", "`"),
        0xDB => ("[", "["),
        0xDC => ("\\", "\\"),
        0xDD => ("]", "]"),
        0xDE => ("'", "'"),
        _ => return None,
    };
    Some((pair.0.to_string(), pair.1.to_string()))
}

/// Parsed `movekeys.kq` file preserving header settings (`submenus.md` rule 22).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MoveKeysFile {
    pub header: Vec<(String, String)>,
    pub bindings: Vec<MoveKeyBinding>,
}

impl Default for MoveKeysFile {
    fn default() -> Self {
        MoveKeysFile {
            header: vec![
                ("classType".into(), "KQMoveKeysScreen".into()),
                ("guiFile".into(), "Movekey.gui".into()),
                ("palette".into(), "Menus.ppl".into()),
                ("volume".into(), "game / $KQGame::BitDepth GUI".into()),
                ("bgColor".into(), "0".into()),
                ("useWorldPal".into(), "0".into()),
                ("cursor".into(), "IDBMA_CURSGUI".into()),
                ("moveSound".into(), "click.aud".into()),
                ("selectSound".into(), "intrface.aud".into()),
            ],
            bindings: DEFAULT_MOVE_KEYS
                .iter()
                .map(|&(vk, name_tag, keyword)| MoveKeyBinding {
                    vk,
                    name_tag: name_tag.to_string(),
                    keyword: keyword.to_string(),
                })
                .collect(),
        }
    }
}

impl MoveKeysFile {
    pub fn parse(bytes: &[u8]) -> Self {
        let mut out = Self::default();
        let sections = kq8_data::kq_text::parse(bytes);
        let Some(sec) = sections.first() else {
            return out;
        };
        let mut header = Vec::new();
        for (k, v) in &sec.entries {
            let kl = k.to_ascii_lowercase();
            if !kl.starts_with("keychar") && !kl.starts_with("keyname") && !kl.starts_with("keyword") {
                header.push((k.clone(), v.clone()));
            }
        }
        if !header.is_empty() {
            out.header = header;
        }
        for i in 0..MOVE_KEY_COUNT {
            if let Some(Ok(vk)) = sec.number(&format!("KeyChar{i}")) {
                out.bindings[i].vk = vk as u32;
            }
            if let Some(name) = sec.get(&format!("KeyName{i}")) {
                out.bindings[i].name_tag = name.trim().to_string();
            }
            if let Some(word) = sec.get(&format!("KeyWord{i}")) {
                out.bindings[i].keyword = word.trim().to_string();
            }
        }
        out
    }

    pub fn format(&self) -> String {
        let mut s = String::from("[MovekeysScreen]\r\n");
        for (k, v) in &self.header {
            s.push_str(&format!("{k}={v}\r\n"));
        }
        for (i, b) in self.bindings.iter().enumerate() {
            s.push_str(&format!("KeyChar{i}={}\r\n", b.vk));
            s.push_str(&format!("KeyName{i}={}\r\n", b.name_tag));
            s.push_str(&format!("KeyWord{i}={}\r\n", b.keyword));
        }
        s
    }
}

/// State machine for `movekey.gui` (`submenus.md` rules 20–22).
#[derive(Debug, Clone)]
pub struct MoveKeysState {
    pub selected: usize,
    pub capturing: Option<usize>,
    pub invalid_key_row: Option<usize>,
    pub file: MoveKeysFile,
}

impl MoveKeysState {
    pub fn new(file: MoveKeysFile) -> Self {
        MoveKeysState {
            selected: 0,
            capturing: None,
            invalid_key_row: None,
            file,
        }
    }

    pub fn reset_on_open(&mut self) {
        self.selected = 0;
        self.capturing = None;
        self.invalid_key_row = None;
    }

    pub fn cursor_rects(&self) -> [[i32; 4]; 2] {
        let r = move_row_rect(self.selected.min(MOVE_ROW_COUNT - 1));
        let top = r[1] + ((r[3] - r[1]) - OPT_CURSOR_SIZE) / 2;
        [
            [OPT_CURSOR1_X, top, OPT_CURSOR1_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
            [OPT_CURSOR2_X, top, OPT_CURSOR2_X + OPT_CURSOR_SIZE, top + OPT_CURSOR_SIZE],
        ]
    }

    pub fn move_selection(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if delta == 0 || self.capturing.is_some() {
            return Vec::new();
        }
        let step = if delta > 0 { 1isize } else { -1isize };
        let idx = (self.selected as isize + step).rem_euclid(MOVE_ROW_COUNT as isize) as usize;
        self.selected = idx;
        vec![
            SubmenuAction::PlaySound("click.aud"),
            SubmenuAction::Note(format!("movekeys cursor -> row {idx}")),
        ]
    }

    pub fn activate(&mut self) -> Vec<SubmenuAction> {
        if self.capturing.is_some() {
            return Vec::new();
        }
        if self.selected >= MOVE_KEY_COUNT {
            return vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::SaveMoveKeysAndReturnToOptions,
            ];
        }
        let row = self.selected;
        self.capturing = Some(row);
        self.invalid_key_row = None;
        vec![
            SubmenuAction::PlaySound("intrface.aud"),
            SubmenuAction::Note(format!("movekeys row {row} entered key-capture mode (#112)")),
        ]
    }

    pub fn press_escape(&mut self) -> Vec<SubmenuAction> {
        if self.capturing.is_some() {
            return vec![SubmenuAction::Note("movekeys Escape blocked while capturing key (submenus.md rule 21)".into())];
        }
        vec![SubmenuAction::SaveMoveKeysAndReturnToOptions]
    }

    /// Handles a key press while in key-capture mode (`submenus.md` rule 21).
    pub fn press_vk(&mut self, vk: u32) -> Vec<SubmenuAction> {
        let Some(row) = self.capturing else {
            return Vec::new();
        };
        let Some((name_tag, keyword)) = bindable_key_info(vk) else {
            self.invalid_key_row = Some(row);
            return vec![
                SubmenuAction::PlaySound("click.aud"),
                SubmenuAction::Note(format!("movekeys row {row}: rejected vk={vk:#x} (#67 Invalid Key)")),
            ];
        };
        let conflict = self
            .file
            .bindings
            .iter()
            .enumerate()
            .find(|&(i, b)| i != row && b.vk == vk)
            .map(|(i, _)| i);

        self.file.bindings[row] = MoveKeyBinding {
            vk,
            name_tag: name_tag.clone(),
            keyword: keyword.clone(),
        };
        self.invalid_key_row = None;

        if let Some(other) = conflict {
            self.file.bindings[other] = MoveKeyBinding {
                vk: 99,
                name_tag: "#112".to_string(),
                keyword: "none".to_string(),
            };
            self.selected = other;
            self.capturing = Some(other);
            vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::Note(format!(
                    "movekeys row {row} bound to vk={vk:#x} ({keyword}); displaced row {other} (now capturing row {other})"
                )),
            ]
        } else {
            self.capturing = None;
            vec![
                SubmenuAction::PlaySound("intrface.aud"),
                SubmenuAction::Note(format!("movekeys row {row} bound to vk={vk:#x} ({keyword})")),
            ]
        }
    }

    pub fn click_at(&mut self, pos: [f32; 2]) -> Vec<SubmenuAction> {
        if self.capturing.is_some() {
            return Vec::new();
        }
        for row in 0..MOVE_ROW_COUNT {
            if point_in_rect(pos, move_row_rect(row)) {
                self.selected = row;
                return self.activate();
            }
        }
        Vec::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_volume_floor_and_options_cs_round_trip() {
        assert_eq!(slider_to_volume(0), 0.0);
        assert_eq!(slider_to_volume(1), 0.10);
        assert_eq!(slider_to_volume(9), 0.10);
        assert_eq!(slider_to_volume(10), 0.10);
        assert_eq!(slider_to_volume(55), 0.55);
        assert_eq!(slider_to_volume(100), 1.0);

        let sample = b"KQSound::setSpeechVol 0.34\r\n\
            KQSound::setSfxVol    0.14\r\n\
            KQSound::setMusicVol  0.14\r\n\
            setShadowMask 15\r\n\
            useDynamicLighting 1\r\n\
            set Options::Captions true\r\n\
            notStartingHelp 0\r\n\
            assignGModeName Glide\r\n\
            setBrightness 50\r\n\
            set800mode 1\r\n";
        let opts = UserOptions::parse(sample);
        assert_eq!(opts.speech_vol, 0.34);
        assert_eq!(opts.sfx_vol, 0.14);
        assert_eq!(opts.music_vol, 0.14);
        assert_eq!(opts.shadow_mask, 15);
        assert!(opts.dynamic_lighting);
        assert!(opts.captions);
        assert_eq!(opts.not_starting_help, 0);
        assert_eq!(opts.gmode_name, "Glide");
        assert_eq!(opts.brightness, 50);
        assert!(opts.mode_800);

        let formatted = opts.format();
        let reparsed = UserOptions::parse(formatted.as_bytes());
        assert_eq!(reparsed, opts);
    }

    #[test]
    fn options_navigation_skips_locked_help_and_steps_sliders_by_5() {
        let mut opts = UserOptions {
            music_vol: 0.0,
            not_starting_help: 2,
            ..UserOptions::default()
        };
        let mut st = OptionsState::from_options(&opts);
        // Up from row 0 wraps to row 8.
        st.move_selection(-1, &opts);
        assert_eq!(st.selected, 8);
        // Up twice visits 7, then skips locked row 6 to row 5.
        st.move_selection(-1, &opts);
        assert_eq!(st.selected, 7);
        st.move_selection(-1, &opts);
        assert_eq!(st.selected, 5);
        // Toggling subtitles on row 5.
        let acts = st.adjust_left_right(1, &mut opts);
        assert!(!opts.captions);
        assert!(acts.contains(&SubmenuAction::PlaySound("intrface.aud")));
        // Step music slider on row 2 by +5: 0 -> 5 -> low-volume floor 0.10.
        st.selected = 2;
        let acts = st.adjust_left_right(1, &mut opts);
        assert_eq!(st.music_slider, 5);
        assert_eq!(opts.music_vol, 0.10);
        assert!(matches!(acts[0], SubmenuAction::UpdateVolumes { music, .. } if (music - 0.10).abs() < 1e-6));
        // Escape returns SaveOptionsAndReturnToMain.
        assert_eq!(st.press_escape(), vec![SubmenuAction::SaveOptionsAndReturnToMain]);
    }

    #[test]
    fn credits_timing_score_formatting_and_vertical_centering() {
        let p0 = CreditsPage::from_text("*DESIGNER/WRITER\r\nRoberta Williams", false, 1234);
        assert_eq!(p0.lines.len(), 2);
        assert_eq!(p0.lines[0], CreditsLine { text: "DESIGNER/WRITER".into(), small: true });
        assert_eq!(p0.lines[1], CreditsLine { text: "Roberta Williams".into(), small: false });
        assert_eq!(p0.duration_ms, 2000 + 500 * 2);

        let final_raw = " \r\n \r\n \r\n \r\n \r\n \r\n \r\n \r\nTeam Mask of Eternity\r\n*You have scored %d out of 6631364 points possible!";
        let pf = CreditsPage::from_text(final_raw, true, 4200);
        assert_eq!(pf.duration_ms, 14000 + 500 * 10);
        assert!(pf.show_photo);
        assert_eq!(pf.lines[9].text, "You have scored 4200 out of 6631364 points possible!");
        let rects = pf.line_rects();
        // 8*30 + 30 + 17 = 287 px -> top_y = (480 - 287) / 2 = 96 -> line 8 at 96 + 240 = 336 (below photo y=310).
        assert_eq!(rects[8].0[1], 336);

        let mut credits = CreditsState {
            pages: vec![p0, pf],
            page: 0,
            elapsed_ms: 0.0,
        };
        assert!(!credits.tick(2999.0));
        assert_eq!(credits.page, 0);
        assert!(!credits.tick(1.0));
        assert_eq!(credits.page, 1);
        assert!(!credits.tick(18_999.0));
        assert!(credits.tick(1.0));
    }

    #[test]
    fn graphics_shadow_bits_and_brightness_formula() {
        assert!((brightness_factor(0) - 0.3).abs() < 1e-6);
        assert!((brightness_factor(50) - 1.0).abs() < 1e-6);
        assert!((brightness_factor(100) - 1.7).abs() < 1e-6);

        let mut opts = UserOptions::default();
        let mut gra = GraphicsState::default();
        assert!(!GraphicsState::is_row_enabled(0));
        assert!(!GraphicsState::is_row_enabled(1));
        assert!(!GraphicsState::is_row_enabled(2));
        // Toggle Monster Shadows (row 7 -> bit 0x04): 15 -> 11.
        gra.selected = 7;
        gra.activate(&mut opts);
        assert_eq!(opts.shadow_mask, 11);
        // Toggle Character Shadows (row 8 -> bit 0x02): 11 -> 9.
        gra.selected = 8;
        gra.activate(&mut opts);
        assert_eq!(opts.shadow_mask, 9);
        // Step brightness (row 10) right: 50 -> 55.
        gra.selected = 10;
        gra.adjust_left_right(1, &mut opts);
        assert_eq!(opts.brightness, 55);
        // Up from row 3 wraps over disabled 0..=2 to row 11.
        gra.selected = 3;
        gra.move_selection(-1);
        assert_eq!(gra.selected, 11);
    }

    #[test]
    fn movekeys_reserved_rejection_duplicate_displacement_and_round_trip() {
        let mut mk = MoveKeysState::new(MoveKeysFile::default());
        assert_eq!(mk.file.bindings.len(), MOVE_KEY_COUNT);
        // Enter capture on row 0 (Forward, default Up 38).
        mk.activate();
        assert_eq!(mk.capturing, Some(0));
        // Navigation and Escape are locked while capturing.
        assert!(mk.move_selection(1).is_empty());
        assert!(matches!(mk.press_escape()[0], SubmenuAction::Note(_)));
        // Reserved F2 (0x71) rejected -> invalid_key_row stays 0, capturing stays 0.
        mk.press_vk(0x71);
        assert_eq!(mk.invalid_key_row, Some(0));
        assert_eq!(mk.capturing, Some(0));
        // Binding 'W' (0x57) succeeds cleanly.
        mk.press_vk(0x57);
        assert_eq!(mk.capturing, None);
        assert_eq!(mk.invalid_key_row, None);
        assert_eq!(mk.file.bindings[0].vk, 0x57);
        assert_eq!(mk.file.bindings[0].keyword, "w");

        // Binding row 1 (Back) to 'W' displaces row 0 (vk=99, #112) and jumps capture to row 0.
        mk.selected = 1;
        mk.activate();
        mk.press_vk(0x57);
        assert_eq!(mk.file.bindings[1].vk, 0x57);
        assert_eq!(mk.file.bindings[0].vk, 99);
        assert_eq!(mk.file.bindings[0].name_tag, "#112");
        assert_eq!(mk.selected, 0);
        assert_eq!(mk.capturing, Some(0));

        // Finish capturing displaced row 0 with 'S' (0x53), then Escape exits.
        mk.press_vk(0x53);
        assert_eq!(mk.capturing, None);
        assert_eq!(mk.press_escape(), vec![SubmenuAction::SaveMoveKeysAndReturnToOptions]);

        let text = mk.file.format();
        let reparsed = MoveKeysFile::parse(text.as_bytes());
        assert_eq!(reparsed, mk.file);
    }
}
