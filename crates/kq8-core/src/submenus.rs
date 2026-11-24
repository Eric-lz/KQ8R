//! Main menu submenus (`docs/behaviour/submenus.md`, M4 step 4g).
//!
//! Pure-Rust rules for:
//! - Screen switching (`MenuScreen`, `submenus.md` rule 1)
//! - `Options.cs` persistence (`UserOptions`, `submenus.md` rule 16)
//! - Options menu (`Options.gui`, `submenus.md` rules 12–16)
//! - Graphics menu (`Graphics.gui`, `submenus.md` rules 17–19)
//! - Remap Move Keys (`movekey.gui`, `movekeys.kq`, `submenus.md` rules 20–22)
//! - Credits roll (`credits.gui`, `400.MSG`, `submenus.md` rules 23–25, `Q206`)

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use kq8_data::msg::MessageFile;
use kq8_data::palette::Rgb;
use kq8_data::pvol::Pvol;

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
            if self.captions { "true" } else { "false" },
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
    pub load_title: String,
    pub save_title: String,
    pub load_btn: String,
    pub save_btn: String,
    pub delete_btn: String,
    pub cancel_btn: String,
    pub ok_btn: String,
    pub delete_prompt: String,
    pub overwrite_prompt: String,
    pub max_saves_prompt: String,
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
            load_title: "Load Game".into(),
            save_title: "Save Game".into(),
            load_btn: "Load".into(),
            save_btn: "Save".into(),
            delete_btn: "Delete".into(),
            cancel_btn: "Cancel".into(),
            ok_btn: "OK".into(),
            delete_prompt: "Are you sure you want to delete the save game:".into(),
            overwrite_prompt: "Are you sure you want to overwrite the save game:".into(),
            max_saves_prompt: "No more space for saved games. Delete an old game or overwrite an existing game.".into(),
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
        if let Some(s) = get(36) { out.load_title = s; }
        if let Some(s) = get(37) { out.save_title = s; }
        if let Some(s) = get(10) { out.load_btn = s; }
        if let Some(s) = get(24) { out.save_btn = s; }
        if let Some(s) = get(86) { out.delete_btn = s; }
        if let Some(s) = get(35) { out.cancel_btn = s; }
        if let Some(s) = get(148) { out.ok_btn = s; }
        if let Some(s) = get(119) { out.delete_prompt = s; }
        if let Some(s) = get(125) { out.overwrite_prompt = s; }
        if let Some(s) = get(166) { out.max_saves_prompt = s; }
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

/// Actions emitted by submenu state machines (`OptionsState`, `GraphicsState`,
/// `MoveKeysState`, `SaveLoadState`, and `CreditsState`).
#[derive(Debug, Clone, PartialEq)]
pub enum SubmenuAction {
    PlaySound(&'static str),
    UpdateVolumes { music: f32, sfx: f32, speech: f32 },
    ApplyGraphics { dynamic_lighting: bool, shadow_mask: u32, brightness: i32, mode_800: bool },
    SwitchScreen(MenuScreen),
    SaveOptionsAndReturnToMain,
    SaveMoveKeysAndReturnToOptions,
    DeleteSave { slot: u32, path: PathBuf, read_only: bool },
    PerformSave { slot: u32, name: String },
    PerformLoad { slot: u32, path: PathBuf, name: String },
    LoadThumbnail { path: PathBuf },
    ReturnToMainMenu,
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

/// Shadow mask bits (`submenus.md` rule 17; `characters.md` rule 36: default
/// `KQMonster` = `2` incl. Wizard, explicit NPC `shadowMask=4`, plain objects = `8`).
pub const GRA_SHADOW_BITS: [(usize, u32); 4] = [
    (6, 0x01), // Connor
    (7, 0x02), // Monster
    (8, 0x04), // Character
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

/// Layout constants and assets for `Loadgame.gui` / `Savegame.gui` (`submenus.md` rules 3–10, `gui-verify.log`).
pub const SL_BG_PICTURE: &str = "saveload.pbm";
pub const SL_HILITE_PICTURE: &str = "sl_hilite.pba";
pub const SL_BTN_PICTURE: &str = "sl_buton.pba";
pub const SL_HIBTN_PICTURE: &str = "sl_hibtn.pba";
pub const SL_UP_PICTURE: &str = "sl_scton.pba";
pub const SL_DOWN_PICTURE: &str = "sl_scbon.pba";
pub const SL_TAB_PICTURE: &str = "sl_scbox.pba";
pub const SL_NOTMB_PICTURE: &str = "sl_notmb.pbm";
pub const SL_POPUP_BG: &str = "sl_win2.pbm";
pub const SL_POPUP_BTN: &str = "s_ybtn.pba";
pub const SL_POPUP_HIBTN: &str = "s_yhibtn.pba";
pub const SL_SAVE_BAR: &str = "savebar.pba";
pub const SL_SAVE_RED: &str = "savered.pbm";
pub const SL_TITLE_FONT: &str = "45sl.pft";
pub const SL_LIST_FONT: &str = "20sl.pft";
pub const SL_POPUP_FONT: &str = "consoles.pft";

pub const SL_TITLE_RECT: [i32; 4] = [0, 0, 640, 138];
pub const LOAD_NAME_RECT: [i32; 4] = [78, 150, 578, 175];
pub const SAVE_NAME_RECT: [i32; 4] = [78, 150, 586, 175];
pub const SL_LIST_ROWS: usize = 10;
pub const SL_SCROLL_UP_RECT: [i32; 4] = [68, 185, 88, 205];
pub const SL_SCROLL_TRACK_RECT: [i32; 4] = [68, 205, 88, 415];
pub const SL_SCROLL_DOWN_RECT: [i32; 4] = [68, 415, 88, 435];
pub const SL_THUMB_TOP: i32 = 205;
pub const SL_THUMB_SPAN: i32 = 190;
pub const SL_THUMB_SIZE: i32 = 20;
pub const SL_BUTTON_RECTS: [[i32; 4]; 3] = [
    [434, 192, 582, 229],
    [434, 234, 582, 271],
    [434, 276, 582, 313],
];
pub const SL_THUMB_RECT: [i32; 4] = [428, 316, 588, 436];
pub const SL_SAVE_BAR_RECT: [i32; 4] = [63, 442, 598, 458];
pub const SL_SAVE_RED_RECT: [i32; 4] = [69, 447, 593, 453];
pub const SL_MAX_SAVES: usize = 50;
pub const SL_THUMB_DEBOUNCE_MS: f64 = 1000.0;
pub const SL_CARET_BLINK_MS: f64 = 300.0;
pub const SL_BACKSPACE_DELAY_MS: f64 = 500.0;
pub const SL_BACKSPACE_REPEAT_MS: f64 = 200.0;

pub const LOAD_POPUP_BG_RECT: [i32; 4] = [140, 130, 500, 250];
pub const LOAD_POPUP_TWO_BTNS: [[i32; 4]; 2] = [[195, 200, 283, 236], [357, 200, 445, 236]];
pub const LOAD_POPUP_OK_BTN: [i32; 4] = [276, 200, 364, 236];
pub const SAVE_POPUP_BG_RECT: [i32; 4] = [140, 120, 500, 240];
pub const SAVE_POPUP_TWO_BTNS: [[i32; 4]; 2] = [[190, 185, 278, 221], [360, 185, 448, 221]];
pub const SAVE_POPUP_OK_BTN: [i32; 4] = [276, 185, 364, 221];

pub fn sl_list_row_rect(row: usize) -> [i32; 4] {
    let top = 185 + row as i32 * 25;
    [95, top, 420, top + 25]
}

pub fn sl_hilite_rect(row: usize) -> [i32; 4] {
    let top = 185 + row as i32 * 25;
    [95, top, 419, top + 25]
}

/// Parses `<N>` from a filename matching `save<N>.vol` case-insensitively (`submenus.md` rule 3).
pub fn parse_save_slot(filename: &str) -> Option<u32> {
    let lower = filename.to_ascii_lowercase();
    let mid = lower.strip_prefix("save")?.strip_suffix(".vol")?;
    if mid.is_empty() {
        return None;
    }
    mid.parse::<u32>().ok()
}

/// One valid saved-game entry discovered from `save<N>.vol` (`submenus.md` rule 3).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SaveEntry {
    pub slot: u32,
    pub filename: String,
    pub name: String,
    pub path: PathBuf,
    pub read_only: bool,
}

fn read_saved_game_entry(path: &Path, filename: &str, read_only: bool) -> Option<SaveEntry> {
    let slot = parse_save_slot(filename)?;
    let bytes = std::fs::read(path).ok()?;
    let pvol = Pvol::parse(bytes).ok()?;
    let chunk = pvol.find("savedGameName")?;
    let raw = pvol.data(chunk).ok()?;
    let end = raw.iter().position(|&b| b == 0).unwrap_or(raw.len());
    let name = String::from_utf8_lossy(&raw[..end]).trim_end_matches('\0').to_string();
    Some(SaveEntry {
        slot,
        filename: filename.to_string(),
        name,
        path: path.to_path_buf(),
        read_only,
    })
}

/// Scans `user_save_dir` (read-write) and `game_save_dir` (read-only `MASK/save`),
/// returning valid saves sorted in descending slot order (`submenus.md` rule 3).
/// Entries in `user_save_dir` shadow matching slots in `game_save_dir`.
pub fn scan_save_entries(
    user_save_dir: &Path,
    game_save_dir: &Path,
    hidden_read_only: &BTreeSet<String>,
) -> Vec<SaveEntry> {
    let mut by_slot: BTreeMap<u32, SaveEntry> = BTreeMap::new();
    for (dir, read_only) in [(game_save_dir, true), (user_save_dir, false)] {
        let Ok(rd) = std::fs::read_dir(dir) else { continue };
        for entry in rd.flatten() {
            let path = entry.path();
            let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
            if read_only && hidden_read_only.contains(&name.to_ascii_lowercase()) {
                continue;
            }
            if let Some(save) = read_saved_game_entry(&path, name, read_only) {
                by_slot.insert(save.slot, save);
            }
        }
    }
    by_slot.into_values().rev().collect()
}

/// Decodes a 19,204-byte `current.bin` (`u32` LE palette ID + 160 × 120 8-bit pixels)
/// into 160 × 120 RGBA pixels (`submenus.md` rule 7).
pub fn decode_thumbnail_bin(
    bytes: &[u8],
    menu_palette: &[Rgb; 256],
    world_palette: Option<&[Rgb; 256]>,
) -> Option<Vec<u8>> {
    let count = 160 * 120;
    if bytes.len() < 4 + count {
        return None;
    }
    let pal_id = u32::from_le_bytes(bytes[0..4].try_into().ok()?);
    let pal = if pal_id == 999 {
        menu_palette
    } else {
        world_palette.unwrap_or(menu_palette)
    };
    let mut rgba = Vec::with_capacity(count * 4);
    for &idx in &bytes[4..4 + count] {
        let [r, g, b] = pal[idx as usize];
        rgba.extend_from_slice(&[r, g, b, 255]);
    }
    Some(rgba)
}

/// Weighted width in tenths of a pixel for `IDCTL_SAVE_NAME` (`submenus.md` rule 8).
pub fn save_name_weighted_width(s: &str) -> u32 {
    s.chars()
        .map(|c| match c {
            ' ' => 50,
            'I' | 'i' | 'j' | 'l' => 56,
            'M' | 'm' => 200,
            'W' => 222,
            _ => 125,
        })
        .sum()
}

/// Returns true if `ch` can be appended to `current` under rule 8
/// (`<= 29` chars before insertion, printable ASCII `0x20..=0x7E`, weighted width `<= 2599`).
pub fn can_append_save_char(current: &str, ch: char) -> bool {
    if !(('\x20'..='\x7E').contains(&ch)) || current.chars().count() >= 30 {
        return false;
    }
    let next_w = save_name_weighted_width(current) + save_name_weighted_width(&ch.to_string());
    next_w <= 2599
}

fn split_sl_popup_lines(text: &str, max_lines: usize) -> Vec<String> {
    let normalized: String = text
        .replace("\r\n", " ")
        .replace('\n', " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    let max_chars = 38usize;
    if trimmed.chars().count() <= max_chars || max_lines <= 1 {
        return vec![trimmed.to_string()];
    }
    if max_lines == 2 {
        return crate::main_menu::split_popup_text(trimmed);
    }
    let mut lines = Vec::new();
    let mut rest = trimmed;
    while !rest.is_empty() && lines.len() + 1 < max_lines && rest.chars().count() > max_chars {
        let mut cut = None;
        for (idx, ch) in rest.char_indices() {
            if rest[..idx].chars().count() > max_chars {
                break;
            }
            if ch == ' ' {
                cut = Some(idx);
            }
        }
        if let Some(idx) = cut {
            lines.push(rest[..idx].trim_end().to_string());
            rest = rest[idx + 1..].trim_start();
        } else {
            break;
        }
    }
    if !rest.is_empty() {
        lines.push(rest.to_string());
    }
    lines
}

/// Confirmation or error popup on `Loadgame.gui` / `Savegame.gui` (`submenus.md` rule 9).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SaveLoadPopup {
    DeleteConfirm {
        entry_index: usize,
        lines: Vec<String>,
        highlight: usize,
    },
    OverwriteConfirm {
        slot: u32,
        name: String,
        lines: Vec<String>,
        highlight: usize,
    },
    MaxSavesError {
        lines: Vec<String>,
    },
}

impl SaveLoadPopup {
    pub fn lines(&self) -> &[String] {
        match self {
            SaveLoadPopup::DeleteConfirm { lines, .. }
            | SaveLoadPopup::OverwriteConfirm { lines, .. }
            | SaveLoadPopup::MaxSavesError { lines } => lines,
        }
    }

    pub fn is_two_button(&self) -> bool {
        !matches!(self, SaveLoadPopup::MaxSavesError { .. })
    }

    pub fn highlight(&self) -> usize {
        match self {
            SaveLoadPopup::DeleteConfirm { highlight, .. }
            | SaveLoadPopup::OverwriteConfirm { highlight, .. } => *highlight,
            SaveLoadPopup::MaxSavesError { .. } => 0,
        }
    }

    pub fn button_rects(&self, is_save: bool) -> Vec<[i32; 4]> {
        if self.is_two_button() {
            if is_save { SAVE_POPUP_TWO_BTNS.to_vec() } else { LOAD_POPUP_TWO_BTNS.to_vec() }
        } else if is_save {
            vec![SAVE_POPUP_OK_BTN]
        } else {
            vec![LOAD_POPUP_OK_BTN]
        }
    }

    pub fn bg_rect(is_save: bool) -> [i32; 4] {
        if is_save { SAVE_POPUP_BG_RECT } else { LOAD_POPUP_BG_RECT }
    }

    pub fn text_rects(&self, is_save: bool) -> Vec<[i32; 4]> {
        let count = self.lines().len();
        let top0 = if is_save {
            if count >= 3 { 128 } else { 135 }
        } else if count >= 3 {
            138
        } else {
            145
        };
        let step = if count >= 3 { 18 } else { 20 };
        (0..count)
            .map(|i| {
                let y = top0 + i as i32 * step;
                [140, y, 500, y + step]
            })
            .collect()
    }
}

/// Shared state machine for `Loadgame.gui` (`is_save = false`) and `Savegame.gui` (`is_save = true`)
/// (`submenus.md` rules 3–9).
#[derive(Debug, Clone)]
pub struct SaveLoadState {
    pub is_save: bool,
    pub entries: Vec<SaveEntry>,
    pub top_index: usize,
    pub selected_row: usize,
    pub active_button: usize,
    pub popup: Option<SaveLoadPopup>,
    pub typed_name: String,
    pub caret_visible: bool,
    pub caret_timer_ms: f64,
    pub backspace_held_ms: Option<f64>,
    pub thumb_debounce_ms: Option<f64>,
    pub scroll_drag: bool,
    pub save_progress: Option<f32>,
}

impl SaveLoadState {
    pub fn new(is_save: bool, entries: Vec<SaveEntry>) -> Self {
        let typed_name = if is_save {
            entries.first().map(|e| e.name.clone()).unwrap_or_default()
        } else {
            String::new()
        };
        SaveLoadState {
            is_save,
            entries,
            top_index: 0,
            selected_row: 0,
            active_button: 0,
            popup: None,
            typed_name,
            caret_visible: true,
            caret_timer_ms: 0.0,
            backspace_held_ms: None,
            thumb_debounce_ms: None,
            scroll_drag: false,
            save_progress: None,
        }
    }

    pub fn selected_index(&self) -> Option<usize> {
        let idx = self.top_index + self.selected_row;
        (idx < self.entries.len()).then_some(idx)
    }

    pub fn selected_entry(&self) -> Option<&SaveEntry> {
        self.selected_index().and_then(|i| self.entries.get(i))
    }

    pub fn thumb_y(&self) -> i32 {
        if self.entries.len() <= SL_LIST_ROWS {
            SL_THUMB_TOP
        } else {
            let max_top = (self.entries.len() - SL_LIST_ROWS) as i32;
            SL_THUMB_TOP + (self.top_index as i32 * SL_THUMB_SPAN) / max_top
        }
    }

    pub fn thumb_rect(&self) -> [i32; 4] {
        let y = self.thumb_y();
        [68, y, 88, y + SL_THUMB_SIZE]
    }

    /// Rebuilds the list after a save or delete while clamping `top_index` and `selected_row`.
    pub fn set_entries(&mut self, entries: Vec<SaveEntry>) {
        self.entries = entries;
        let max_top = self.entries.len().saturating_sub(SL_LIST_ROWS);
        self.top_index = self.top_index.min(max_top);
        let vis = self.entries.len().saturating_sub(self.top_index).min(SL_LIST_ROWS);
        self.selected_row = if vis == 0 { 0 } else { self.selected_row.min(vis - 1) };
        if self.is_save {
            self.typed_name = self.selected_entry().map(|e| e.name.clone()).unwrap_or_default();
        }
        self.thumb_debounce_ms = None;
    }

    fn on_selection_changed(&mut self) {
        if self.is_save {
            if let Some(entry) = self.selected_entry() {
                self.typed_name = entry.name.clone();
            }
        } else {
            self.thumb_debounce_ms = Some(SL_THUMB_DEBOUNCE_MS);
        }
    }

    pub fn move_list(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if self.popup.is_some() || delta == 0 || self.entries.is_empty() {
            return Vec::new();
        }
        let prev = (self.top_index, self.selected_row);
        if delta < 0 {
            if self.selected_row > 0 {
                self.selected_row -= 1;
            } else if self.top_index > 0 {
                self.top_index -= 1;
            }
        } else {
            let visible_max = self
                .entries
                .len()
                .saturating_sub(self.top_index)
                .min(SL_LIST_ROWS)
                .saturating_sub(1);
            if self.selected_row < visible_max {
                self.selected_row += 1;
            } else if self.top_index + SL_LIST_ROWS < self.entries.len() {
                self.top_index += 1;
            }
        }
        if (self.top_index, self.selected_row) == prev {
            return Vec::new();
        }
        self.on_selection_changed();
        vec![SubmenuAction::PlaySound("click.aud")]
    }

    pub fn scroll_step(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if self.popup.is_some() || delta == 0 {
            return Vec::new();
        }
        let prev_idx = self.selected_index();
        if self.entries.len() > SL_LIST_ROWS {
            if delta < 0 && self.top_index > 0 {
                self.top_index -= 1;
                self.selected_row = (self.selected_row + 1).min(SL_LIST_ROWS - 1);
            } else if delta > 0 && self.top_index + SL_LIST_ROWS < self.entries.len() {
                self.top_index += 1;
                self.selected_row = self.selected_row.saturating_sub(1);
            }
        }
        if self.selected_index() != prev_idx {
            self.on_selection_changed();
        }
        vec![SubmenuAction::PlaySound("button1.aud")]
    }

    pub fn scroll_page(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if self.popup.is_some() || delta == 0 {
            return Vec::new();
        }
        let prev_top = self.top_index;
        let max_top = self.entries.len().saturating_sub(SL_LIST_ROWS);
        if delta < 0 {
            self.top_index = self.top_index.saturating_sub(SL_LIST_ROWS);
        } else {
            self.top_index = (self.top_index + SL_LIST_ROWS).min(max_top);
        }
        if self.top_index != prev_top {
            self.on_selection_changed();
        }
        vec![SubmenuAction::PlaySound("button1.aud")]
    }

    pub fn drag_thumb(&mut self, y: f32) -> Vec<SubmenuAction> {
        if self.popup.is_some() || self.entries.len() <= SL_LIST_ROWS {
            return Vec::new();
        }
        let max_top = self.entries.len() - SL_LIST_ROWS;
        let frac = ((y - SL_THUMB_TOP as f32) / SL_THUMB_SPAN as f32).clamp(0.0, 1.0);
        let new_top = (frac * max_top as f32).round() as usize;
        if new_top != self.top_index {
            self.top_index = new_top;
            self.on_selection_changed();
        }
        Vec::new()
    }

    pub fn move_button(&mut self, delta: i32) -> Vec<SubmenuAction> {
        if delta == 0 {
            return Vec::new();
        }
        if let Some(popup) = self.popup.as_mut() {
            match popup {
                SaveLoadPopup::DeleteConfirm { highlight, .. }
                | SaveLoadPopup::OverwriteConfirm { highlight, .. } => {
                    let next = if delta < 0 { 0 } else { 1 };
                    if *highlight != next {
                        *highlight = next;
                        return vec![SubmenuAction::PlaySound("button1.aud")];
                    }
                }
                SaveLoadPopup::MaxSavesError { .. } => {}
            }
            return Vec::new();
        }
        let next = if delta < 0 {
            self.active_button.saturating_sub(1)
        } else {
            (self.active_button + 1).min(2)
        };
        if next != self.active_button {
            self.active_button = next;
            vec![SubmenuAction::PlaySound("button1.aud")]
        } else {
            Vec::new()
        }
    }

    pub fn lowest_unused_slot(&self) -> u32 {
        let mut slot = 1u32;
        while self.entries.iter().any(|e| e.slot == slot) {
            slot += 1;
        }
        slot
    }

    pub fn activate_button(&mut self, texts: &SubmenuTexts) -> Vec<SubmenuAction> {
        if let Some(popup) = self.popup.take() {
            let mut acts = vec![SubmenuAction::PlaySound("button1.aud")];
            match popup {
                SaveLoadPopup::DeleteConfirm { entry_index, highlight, .. } => {
                    if highlight == 0 {
                        if let Some(entry) = self.entries.get(entry_index).cloned() {
                            acts.push(SubmenuAction::DeleteSave {
                                slot: entry.slot,
                                path: entry.path,
                                read_only: entry.read_only,
                            });
                        }
                    }
                }
                SaveLoadPopup::OverwriteConfirm { slot, name, highlight, .. } => {
                    if highlight == 0 {
                        acts.push(SubmenuAction::PerformSave { slot, name });
                    }
                }
                SaveLoadPopup::MaxSavesError { .. } => {}
            }
            return acts;
        }

        match self.active_button {
            0 => {
                if self.is_save {
                    let trimmed = self.typed_name.trim_start().to_string();
                    if trimmed.is_empty() {
                        return vec![SubmenuAction::Note("Save skipped: save name is empty".into())];
                    }
                    if let Some(existing) = self
                        .entries
                        .iter()
                        .find(|e| e.name.eq_ignore_ascii_case(&trimmed))
                    {
                        let lines = split_sl_popup_lines(
                            &format!("{} {}", texts.overwrite_prompt, trimmed),
                            2,
                        );
                        self.popup = Some(SaveLoadPopup::OverwriteConfirm {
                            slot: existing.slot,
                            name: trimmed,
                            lines,
                            highlight: 1,
                        });
                        return vec![SubmenuAction::PlaySound("button1.aud")];
                    }
                    if self.entries.len() >= SL_MAX_SAVES {
                        self.popup = Some(SaveLoadPopup::MaxSavesError {
                            lines: split_sl_popup_lines(&texts.max_saves_prompt, 3),
                        });
                        return vec![SubmenuAction::PlaySound("button1.aud")];
                    }
                    let slot = self.lowest_unused_slot();
                    vec![
                        SubmenuAction::PlaySound("button1.aud"),
                        SubmenuAction::PerformSave { slot, name: trimmed },
                    ]
                } else {
                    let Some(entry) = self.selected_entry().cloned() else {
                        return Vec::new();
                    };
                    vec![
                        SubmenuAction::PlaySound("button1.aud"),
                        SubmenuAction::PerformLoad {
                            slot: entry.slot,
                            path: entry.path,
                            name: entry.name,
                        },
                    ]
                }
            }
            1 => {
                let Some(idx) = self.selected_index() else {
                    return Vec::new();
                };
                let entry_name = self.entries[idx].name.clone();
                let lines = split_sl_popup_lines(
                    &format!("{} {}", texts.delete_prompt, entry_name),
                    2,
                );
                self.popup = Some(SaveLoadPopup::DeleteConfirm {
                    entry_index: idx,
                    lines,
                    highlight: 1,
                });
                vec![SubmenuAction::PlaySound("button1.aud")]
            }
            _ => {
                self.active_button = 0;
                vec![
                    SubmenuAction::PlaySound("button1.aud"),
                    SubmenuAction::ReturnToMainMenu,
                ]
            }
        }
    }

    pub fn press_escape(&mut self) -> Vec<SubmenuAction> {
        if self.popup.take().is_some() {
            return vec![SubmenuAction::Note("closed Save/Load popup with Escape".into())];
        }
        self.active_button = 0;
        vec![SubmenuAction::ReturnToMainMenu]
    }

    pub fn type_char(&mut self, ch: char) -> bool {
        if !self.is_save || self.popup.is_some() {
            return false;
        }
        if can_append_save_char(&self.typed_name, ch) {
            self.typed_name.push(ch);
            true
        } else {
            false
        }
    }

    pub fn press_backspace(&mut self) -> bool {
        if !self.is_save || self.popup.is_some() || self.typed_name.is_empty() {
            return false;
        }
        self.typed_name.pop();
        true
    }

    pub fn click_at(&mut self, pos: [f32; 2], texts: &SubmenuTexts) -> Vec<SubmenuAction> {
        if let Some(popup) = self.popup.as_mut() {
            let rects = popup.button_rects(self.is_save);
            for (idx, r) in rects.into_iter().enumerate() {
                if point_in_rect(pos, r) {
                    match popup {
                        SaveLoadPopup::DeleteConfirm { highlight, .. }
                        | SaveLoadPopup::OverwriteConfirm { highlight, .. } => {
                            *highlight = idx;
                        }
                        SaveLoadPopup::MaxSavesError { .. } => {}
                    }
                    return self.activate_button(texts);
                }
            }
            return Vec::new();
        }

        for (idx, &rect) in SL_BUTTON_RECTS.iter().enumerate() {
            if point_in_rect(pos, rect) {
                self.active_button = idx;
                return self.activate_button(texts);
            }
        }

        for row in 0..SL_LIST_ROWS {
            if point_in_rect(pos, sl_list_row_rect(row)) {
                if self.top_index + row < self.entries.len() {
                    if self.selected_row != row {
                        self.selected_row = row;
                        self.on_selection_changed();
                    } else if self.is_save {
                        self.on_selection_changed();
                    }
                    return vec![SubmenuAction::PlaySound("click.aud")];
                }
                return Vec::new();
            }
        }

        if point_in_rect(pos, SL_SCROLL_UP_RECT) {
            return self.scroll_step(-1);
        }
        if point_in_rect(pos, SL_SCROLL_DOWN_RECT) {
            return self.scroll_step(1);
        }
        let thumb_r = self.thumb_rect();
        if point_in_rect(pos, thumb_r) {
            self.scroll_drag = true;
            return Vec::new();
        }
        if point_in_rect(pos, SL_SCROLL_TRACK_RECT) {
            let dir = if pos[1] < thumb_r[1] as f32 { -1 } else { 1 };
            return self.scroll_page(dir);
        }
        Vec::new()
    }

    pub fn tick(&mut self, dt_ms: f64, backspace_down: bool) -> Vec<SubmenuAction> {
        let mut acts = Vec::new();
        if self.is_save {
            self.caret_timer_ms += dt_ms;
            if self.caret_timer_ms >= SL_CARET_BLINK_MS {
                self.caret_timer_ms %= SL_CARET_BLINK_MS;
                self.caret_visible = !self.caret_visible;
            }
            if backspace_down && self.popup.is_none() {
                match self.backspace_held_ms {
                    None => {
                        self.backspace_held_ms = Some(SL_BACKSPACE_DELAY_MS);
                    }
                    Some(mut rem) => {
                        rem -= dt_ms;
                        while rem <= 0.0 {
                            self.press_backspace();
                            rem += SL_BACKSPACE_REPEAT_MS;
                        }
                        self.backspace_held_ms = Some(rem);
                    }
                }
            } else {
                self.backspace_held_ms = None;
            }
        } else if let Some(rem) = self.thumb_debounce_ms.as_mut() {
            *rem -= dt_ms;
            if *rem <= 0.0 {
                self.thumb_debounce_ms = None;
                if let Some(entry) = self.selected_entry() {
                    acts.push(SubmenuAction::LoadThumbnail { path: entry.path.clone() });
                }
            }
        }
        acts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_entries(count: u32) -> Vec<SaveEntry> {
        (1..=count)
            .rev()
            .map(|slot| SaveEntry {
                slot,
                filename: format!("save{slot}.vol"),
                name: format!("Slot {slot}"),
                path: PathBuf::from(format!("save{slot}.vol")),
                read_only: false,
            })
            .collect()
    }

    #[test]
    fn save_list_sorts_descending_and_scrolls_with_10_row_window() {
        assert_eq!(parse_save_slot("SAVE12.VOL"), Some(12));
        assert_eq!(parse_save_slot("not_a_save.vol"), None);

        let mut st = SaveLoadState::new(false, test_entries(15));
        assert_eq!(st.entries[0].slot, 15);
        assert_eq!(st.entries[14].slot, 1);
        assert_eq!(st.thumb_y(), SL_THUMB_TOP);

        // Move down 9 rows to bottom visible row (slot 6), then 1 more to scroll window.
        for _ in 0..9 {
            st.move_list(1);
        }
        assert_eq!((st.top_index, st.selected_row), (0, 9));
        assert_eq!(st.selected_entry().unwrap().slot, 6);

        st.move_list(1);
        assert_eq!((st.top_index, st.selected_row), (1, 9));
        assert_eq!(st.selected_entry().unwrap().slot, 5);
        assert_eq!(st.thumb_y(), SL_THUMB_TOP + SL_THUMB_SPAN / 5);

        // Moving back to row 0 then Up scrolls top_index back to 0.
        for _ in 0..9 {
            st.move_list(-1);
        }
        assert_eq!((st.top_index, st.selected_row), (1, 0));
        st.move_list(-1);
        assert_eq!((st.top_index, st.selected_row), (0, 0));
    }

    #[test]
    fn save_name_typing_enforces_30_char_and_2599_width_limits() {
        let mut st = SaveLoadState::new(true, Vec::new());
        // Narrow 'i' (56 tenths) reaches 30-char limit first (30 * 56 = 1680 <= 2599).
        for _ in 0..35 {
            st.type_char('i');
        }
        assert_eq!(st.typed_name.len(), 30);

        // Wide 'W' (222 tenths) hits 2599-width limit at 11 chars (11 * 222 = 2442, 12th = 2664 > 2599).
        st.typed_name.clear();
        for _ in 0..20 {
            st.type_char('W');
        }
        assert_eq!(st.typed_name.len(), 11);
        assert!(save_name_weighted_width(&st.typed_name) <= 2599);

        assert!(st.press_backspace());
        assert_eq!(st.typed_name.len(), 10);
    }

    #[test]
    fn save_load_popups_delete_overwrite_and_max_50_limit() {
        let texts = SubmenuTexts::default();
        let mut st = SaveLoadState::new(true, test_entries(2));
        // Existing save name triggers OverwriteConfirm with Cancel (1) highlighted by default.
        st.typed_name = "  slot 2".into();
        st.activate_button(&texts);
        assert!(matches!(
            st.popup,
            Some(SaveLoadPopup::OverwriteConfirm { slot: 2, highlight: 1, .. })
        ));
        // Left selects Yes (0) then Enter emits PerformSave.
        st.move_button(-1);
        let acts = st.activate_button(&texts);
        assert!(acts.contains(&SubmenuAction::PerformSave {
            slot: 2,
            name: "slot 2".into(),
        }));

        // Delete opens DeleteConfirm with Cancel (1) highlighted.
        st.active_button = 1;
        st.activate_button(&texts);
        assert!(matches!(
            st.popup,
            Some(SaveLoadPopup::DeleteConfirm { entry_index: 0, highlight: 1, .. })
        ));
        // Escape closes popup without deleting.
        st.press_escape();
        assert!(st.popup.is_none());

        // 50 saves triggers MaxSavesError (#166) on a new save name.
        let mut full = SaveLoadState::new(true, test_entries(SL_MAX_SAVES as u32));
        full.typed_name = "Brand New Save".into();
        full.activate_button(&texts);
        assert!(matches!(full.popup, Some(SaveLoadPopup::MaxSavesError { .. })));
    }

    #[test]
    fn load_thumbnail_debounce_fires_after_1000_ms() {
        let mut st = SaveLoadState::new(false, test_entries(3));
        assert!(st.tick(2000.0, false).is_empty());

        st.move_list(1);
        assert!(st.thumb_debounce_ms.is_some());
        assert!(st.tick(999.0, false).is_empty());
        let acts = st.tick(1.0, false);
        assert_eq!(
            acts,
            vec![SubmenuAction::LoadThumbnail { path: PathBuf::from("save2.vol") }]
        );
        assert!(st.thumb_debounce_ms.is_none());
    }

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
        // Toggle Monster Shadows (row 7 -> bit 0x02): 15 -> 13.
        gra.selected = 7;
        gra.activate(&mut opts);
        assert_eq!(opts.shadow_mask, 13);
        // Toggle Character Shadows (row 8 -> bit 0x04): 13 -> 9.
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
