//! Main menu rules (`docs/behaviour/main-menu.md`, M4 step 4f-1).
//!
//! No drawing or Bevy code here: this module tracks the menu state, which
//! items are enabled, keyboard/mouse list navigation, the four popups
//! (`NewConfirm`, `CorruptSave`, `QuitConfirm`, `Difficulty`), save-folder
//! scanning, title parsing, and the combat difficulty levels.

use std::collections::BTreeSet;
use std::path::Path;

use kq8_data::pvol::Pvol;

/// Screen and resource names from `GAME/Kq/Mainmenu.kq` (`main-menu.md` rule 1).
pub const GUI_FILE: &str = "Mainmenu.gui";
pub const PALETTE_FILE: &str = "Menus.ppl";
pub const POINTER_TAG: &str = "IDBMA_CURSGUI";
pub const POINTER_FILE: &str = "Cursgui.pba";
pub const THEME_MUSIC: &str = "theme.aud";
pub const MOVE_SOUND: &str = "click.aud";
pub const SELECT_SOUND: &str = "intrface.aud";
pub const BG_PICTURE: &str = "main18.pbm";
pub const LINE_PICTURE: &str = "45-line.pbm";
pub const CURSOR_PICTURE: &str = "Mskcur.pba";
pub const POPUP_BG_PICTURE: &str = "gm_win2.pbm";
pub const POPUP_BTN_PICTURE: &str = "g_ybtn.pba";
pub const POPUP_HI_BTN_PICTURE: &str = "g_yhibtn.pba";
pub const ITEM_FONT: &str = "27.pft";
pub const TITLE_FONT: &str = "45.pft";
pub const POPUP_FONT: &str = "consoleg.pft";

/// Input is ignored for 0.2 s of real time whenever the menu opens or the
/// intro film ends (`main-menu.md` rules 8, 12).
pub const INPUT_BLOCK_SECONDS: f64 = 0.2;

/// Menu theme fade-out rate when returning to gameplay (`main-menu.md` rule 9).
pub const THEME_FADE_RATE: f32 = 1.0;

/// Real-time pause between Connor entering `Mode::Dead` and the menu appearing
/// (`main-menu.md` rule 25, user's check in original: "a couple of seconds").
pub const DEATH_HANDS_OFF_SECONDS: f64 = 2.0;

/// Combat levels (`main-menu.md` rules 10, 24; `fighting.md` rule 3).
pub const COMBAT_LEVELS: [(i64, f32, &str); 3] = [(1, 0.66, "Easy"), (2, 1.0, "Normal"), (3, 1.5, "Hard")];

/// 640 × 480 layout rectangles from `Mainmenu.gui` (`main-menu.md` rules 2, 4, 5, 14).
pub const TITLE_RECTS: [[i32; 4]; 2] = [[195, 20, 445, 50], [195, 66, 445, 96]];
pub const TITLE_LINE_RECT: [i32; 4] = [203, 53, 425, 57];
pub const ITEM_LEFT: i32 = 64;
pub const ITEM_RIGHT: i32 = 624;
pub const ITEM_HEIGHT: i32 = 40;
pub const CURSOR_LEFT: i32 = 16;
pub const CURSOR_SIZE: i32 = 40;
pub const POPUP_BG_RECT: [i32; 4] = [140, 180, 500, 300];
pub const POPUP_TEXT_LEFT: i32 = 170;
pub const POPUP_TEXT_RIGHT: i32 = 470;
pub const POPUP_TEXT_HEIGHT: i32 = 20;
pub const POPUP_BTN_WIDTH: i32 = 88;
pub const POPUP_BTN_TOP: i32 = 250;
pub const POPUP_BTN_HEIGHT: i32 = 36;
pub const POPUP_TWO_BTN_X: [i32; 2] = [188, 361];
pub const POPUP_THREE_BTN_X: [i32; 3] = [158, 274, 392];
pub const POPUP_WRAP_CHARS: usize = 38;

/// Saved-game thumbnail size (`main-menu.md` rules 7.3, 22).
pub const THUMBNAIL_WIDTH: u32 = 160;
pub const THUMBNAIL_HEIGHT: u32 = 120;

/// Seven selectable menu rows (`main-menu.md` rule 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MenuItem {
    New = 0,
    Load = 1,
    Save = 2,
    Options = 3,
    ReplayIntro = 4,
    Quit = 5,
    BackToGame = 6,
}

impl MenuItem {
    pub const ALL: [MenuItem; 7] = [
        MenuItem::New,
        MenuItem::Load,
        MenuItem::Save,
        MenuItem::Options,
        MenuItem::ReplayIntro,
        MenuItem::Quit,
        MenuItem::BackToGame,
    ];

    pub fn index(self) -> usize {
        self as usize
    }

    pub fn from_index(i: usize) -> Option<Self> {
        Self::ALL.get(i).copied()
    }

    /// Control ID in `Mainmenu.gui` / `kqGuiTag.TTAG`.
    pub fn control_id(self) -> u32 {
        match self {
            MenuItem::New => 9,
            MenuItem::Load => 10,
            MenuItem::Save => 11,
            MenuItem::Options => 14,
            MenuItem::ReplayIntro => 500,
            MenuItem::Quit => 12,
            MenuItem::BackToGame => 502,
        }
    }

    /// Case number in `GAME/ENGLISH/500.MSG` (noun 0, verb 0).
    pub fn msg_case(self) -> u32 {
        match self {
            MenuItem::New => 13,
            MenuItem::Load => 10,
            MenuItem::Save => 24,
            MenuItem::Options => 19,
            MenuItem::ReplayIntro => 22,
            MenuItem::Quit => 20,
            MenuItem::BackToGame => 2,
        }
    }

    pub fn default_label(self) -> &'static str {
        match self {
            MenuItem::New => "New",
            MenuItem::Load => "Load",
            MenuItem::Save => "Save",
            MenuItem::Options => "Options",
            MenuItem::ReplayIntro => "Replay Intro",
            MenuItem::Quit => "Quit",
            MenuItem::BackToGame => "Back to Game",
        }
    }

    /// Top Y coordinate in the 640 × 480 layout (`main-menu.md` rule 2).
    pub fn row_top(self) -> i32 {
        match self {
            MenuItem::New => 125,
            MenuItem::Load => 165,
            MenuItem::Save => 205,
            MenuItem::Options => 245,
            MenuItem::ReplayIntro => 285,
            MenuItem::Quit => 345,
            MenuItem::BackToGame => 405,
        }
    }

    pub fn rect(self) -> [i32; 4] {
        let top = self.row_top();
        [ITEM_LEFT, top, ITEM_RIGHT, top + ITEM_HEIGHT]
    }

    /// Mask cursor rectangle centred vertically on the item (`main-menu.md` rule 5).
    pub fn cursor_rect(self) -> [i32; 4] {
        let top = self.row_top() + (ITEM_HEIGHT - CURSOR_SIZE) / 2;
        [CURSOR_LEFT, top, CURSOR_LEFT + CURSOR_SIZE, top + CURSOR_SIZE]
    }
}

/// First two lines of `starttxt.txt`, turning line breaks into spaces
/// (`main-menu.md` rule 4).
pub fn parse_title_lines(bytes: &[u8]) -> [String; 2] {
    let text = String::from_utf8_lossy(bytes);
    let mut lines = text
        .split('\n')
        .map(|l| l.replace(['\r', '\n'], " ").trim().to_string())
        .filter(|l| !l.is_empty());
    let l1 = lines.next().unwrap_or_else(|| "KING'S QUEST".to_string());
    let l2 = lines.next().unwrap_or_else(|| "MASK of ETERNITY".to_string());
    [l1, l2]
}

/// Splits popup text longer than 38 characters into two lines at the space
/// nearest the middle (`main-menu.md` rule 14).
pub fn split_popup_text(text: &str) -> Vec<String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return vec![String::new()];
    }
    let len = trimmed.chars().count();
    if len <= POPUP_WRAP_CHARS {
        return vec![trimmed.to_string()];
    }
    let mid = trimmed.len() / 2;
    let best_space = trimmed
        .match_indices(' ')
        .map(|(idx, _)| (idx.abs_diff(mid), idx))
        .min_by_key(|&(dist, idx)| (dist, idx))
        .map(|(_, idx)| idx);
    if let Some(idx) = best_space {
        let a = trimmed[..idx].trim_end();
        let b = trimmed[idx + 1..].trim_start();
        if !a.is_empty() && !b.is_empty() {
            return vec![a.to_string(), b.to_string()];
        }
    }
    let cut = trimmed.char_indices().nth(len / 2).map_or(trimmed.len(), |(i, _)| i);
    vec![trimmed[..cut].to_string(), trimmed[cut..].to_string()]
}

/// Result of scanning `<game>/save/*.vol` (`main-menu.md` rule 23).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SaveScan {
    pub total_files: usize,
    pub valid_saves: usize,
    pub first_corrupt: Option<String>,
}

/// Inspects raw `(filename, bytes)` pairs of `*.vol` save files (`main-menu.md` rule 23).
pub fn scan_save_entries<'a>(entries: impl IntoIterator<Item = (&'a str, &'a [u8])>, ignored: &BTreeSet<String>) -> SaveScan {
    let mut items: Vec<(&str, &[u8])> = entries
        .into_iter()
        .filter(|(name, _)| name.to_ascii_lowercase().ends_with(".vol"))
        .collect();
    items.sort_by(|a, b| a.0.to_ascii_lowercase().cmp(&b.0.to_ascii_lowercase()));
    let mut scan = SaveScan::default();
    for (name, bytes) in items {
        scan.total_files += 1;
        let ok = Pvol::parse(bytes.to_vec())
            .ok()
            .and_then(|p| p.find("savedGameName").and_then(|e| p.data(e).ok()).map(|_| ()))
            .is_some();
        if ok {
            scan.valid_saves += 1;
        } else if scan.first_corrupt.is_none() && !ignored.contains(&name.to_ascii_lowercase()) {
            scan.first_corrupt = Some(name.to_string());
        }
    }
    scan
}

/// Read-only scan of `<game>/save/*.vol` (`main-menu.md` rule 23, AGENTS.md:
/// never writes to or deletes from the game install).
pub fn scan_save_folder(save_dir: &Path, ignored: &BTreeSet<String>) -> SaveScan {
    let Ok(rd) = std::fs::read_dir(save_dir) else {
        return SaveScan::default();
    };
    let mut files: Vec<(String, Vec<u8>)> = Vec::new();
    for entry in rd.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else { continue };
        if !name.to_ascii_lowercase().ends_with(".vol") {
            continue;
        }
        let bytes = std::fs::read(&path).unwrap_or_default();
        files.push((name.to_string(), bytes));
    }
    scan_save_entries(files.iter().map(|(n, b)| (n.as_str(), b.as_slice())), ignored)
}

/// Strings loaded from `GAME/ENGLISH/500.MSG` (noun 0, verb 0, case = number)
/// and `starttxt.txt` (`main-menu.md` "Data used").
#[derive(Debug, Clone)]
pub struct MenuTexts {
    pub title: [String; 2],
    pub items: [String; 7],
    pub yes: String,
    pub no: String,
    pub difficulty_prompt: String,
    pub easy: String,
    pub normal: String,
    pub hard: String,
    pub corrupt_save: String,
    pub new_game_question: String,
    pub quit_question: String,
}

impl Default for MenuTexts {
    fn default() -> Self {
        MenuTexts {
            title: ["KING'S QUEST".into(), "MASK of ETERNITY".into()],
            items: MenuItem::ALL.map(|i| i.default_label().to_string()),
            yes: "Yes".into(),
            no: "No".into(),
            difficulty_prompt: "Combat difficulty".into(),
            easy: "Easy".into(),
            normal: "Normal".into(),
            hard: "Hard".into(),
            corrupt_save: "A corrupt saved game was detected. Delete it?".into(),
            new_game_question: "You already have a game running. Are you sure you want to start a NEW one?".into(),
            quit_question: "Are you sure you want to exit the game?".into(),
        }
    }
}

impl MenuTexts {
    pub fn from_msg_and_title(msg: Option<&kq8_data::msg::MessageFile>, starttxt: Option<&[u8]>) -> Self {
        let mut out = Self::default();
        if let Some(b) = starttxt {
            out.title = parse_title_lines(b);
        }
        if let Some(f) = msg {
            let get = |case: u32| {
                f.lines
                    .iter()
                    .find(|l| l.tuple.noun == 0 && l.tuple.verb == 0 && l.tuple.case as u32 == case)
                    .map(|l| l.raw.trim().to_string())
                    .filter(|s| !s.is_empty())
            };
            for item in MenuItem::ALL {
                if let Some(s) = get(item.msg_case()) {
                    out.items[item.index()] = s;
                }
            }
            if let Some(s) = get(29) { out.yes = s; }
            if let Some(s) = get(14) { out.no = s; }
            if let Some(s) = get(120) { out.difficulty_prompt = s; }
            if let Some(s) = get(121) { out.easy = s; }
            if let Some(s) = get(122) { out.normal = s; }
            if let Some(s) = get(123) { out.hard = s; }
            if let Some(s) = get(157) { out.corrupt_save = s; }
            if let Some(s) = get(158) { out.new_game_question = s; }
            if let Some(s) = get(159) { out.quit_question = s; }
        }
        out
    }
}

/// Which modal popup is open on top of the menu (`main-menu.md` rules 10, 13–17, 23).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MenuPopupKind {
    NewConfirm,
    CorruptSave { file: String },
    QuitConfirm,
    Difficulty,
}

#[derive(Debug, Clone, PartialEq)]
pub struct MenuPopup {
    pub kind: MenuPopupKind,
    pub lines: Vec<String>,
    /// Highlighted button index (`0..button_count`).
    pub highlight: usize,
    /// Button currently held down by Enter (`main-menu.md` rule 16).
    pub pressed: Option<usize>,
}

impl MenuPopup {
    fn open(kind: MenuPopupKind, texts: &MenuTexts) -> Self {
        let (prompt, highlight) = match &kind {
            MenuPopupKind::NewConfirm => (texts.new_game_question.as_str(), 1),
            MenuPopupKind::CorruptSave { .. } => (texts.corrupt_save.as_str(), 1),
            MenuPopupKind::QuitConfirm => (texts.quit_question.as_str(), 1),
            MenuPopupKind::Difficulty => (texts.difficulty_prompt.as_str(), 1),
        };
        MenuPopup {
            kind,
            lines: split_popup_text(prompt),
            highlight,
            pressed: None,
        }
    }

    pub fn button_count(&self) -> usize {
        match self.kind {
            MenuPopupKind::Difficulty => 3,
            _ => 2,
        }
    }

    pub fn button_labels<'a>(&self, texts: &'a MenuTexts) -> Vec<&'a str> {
        match self.kind {
            MenuPopupKind::Difficulty => vec![&texts.easy, &texts.normal, &texts.hard],
            _ => vec![&texts.yes, &texts.no],
        }
    }

    pub fn button_rects(&self) -> Vec<[i32; 4]> {
        match self.kind {
            MenuPopupKind::Difficulty => POPUP_THREE_BTN_X
                .iter()
                .map(|&x| [x, POPUP_BTN_TOP, x + POPUP_BTN_WIDTH, POPUP_BTN_TOP + POPUP_BTN_HEIGHT])
                .collect(),
            _ => POPUP_TWO_BTN_X
                .iter()
                .map(|&x| [x, POPUP_BTN_TOP, x + POPUP_BTN_WIDTH, POPUP_BTN_TOP + POPUP_BTN_HEIGHT])
                .collect(),
        }
    }

    /// Text line rectangles in 640 × 480 layout coordinates (`main-menu.md` rule 14).
    pub fn text_rects(&self) -> Vec<[i32; 4]> {
        let ys: &[i32] = match (&self.kind, self.lines.len()) {
            (MenuPopupKind::NewConfirm | MenuPopupKind::CorruptSave { .. }, _) | (_, 2..) => &[200, 220],
            (MenuPopupKind::QuitConfirm | MenuPopupKind::Difficulty, _) => &[215],
        };
        self.lines
            .iter()
            .enumerate()
            .map(|(k, _)| {
                let top = ys.get(k).copied().unwrap_or(200 + k as i32 * 20);
                [POPUP_TEXT_LEFT, top, POPUP_TEXT_RIGHT, top + POPUP_TEXT_HEIGHT]
            })
            .collect()
    }
}

/// Actions emitted by [`MainMenuState`] for the viewer (`menu_view.rs`).
#[derive(Debug, Clone, PartialEq)]
pub enum MenuAction {
    PlayMenuSound { file: &'static str, why: String },
    StartTheme,
    FadeTheme,
    ResumeGame,
    StartNewGame { combat_level_number: i64, factor: f32 },
    OpenSubScreen(MenuItem),
    ReplayIntro,
    Quit,
    DeleteCorruptSave { file: String },
    Note(String),
}

/// Core state machine for the Main Menu (`docs/behaviour/main-menu.md`).
#[derive(Debug, Clone)]
pub struct MainMenuState {
    pub open: bool,
    pub has_started_or_loaded: bool,
    pub connor_died: bool,
    pub has_saves: bool,
    pub corrupt_saves_ignored: BTreeSet<String>,
    pub selected: MenuItem,
    pub popup: Option<MenuPopup>,
    pub input_block_remaining: f64,
}

impl MainMenuState {
    /// Program startup on the Main Menu (`main-menu.md` rules 6, 8).
    pub fn new_at_startup(scan: SaveScan, texts: &MenuTexts) -> (Self, Vec<MenuAction>) {
        let mut s = MainMenuState {
            open: true,
            has_started_or_loaded: false,
            connor_died: false,
            has_saves: false,
            corrupt_saves_ignored: BTreeSet::new(),
            selected: MenuItem::New,
            popup: None,
            input_block_remaining: INPUT_BLOCK_SECONDS,
        };
        let mut actions = s.apply_show(scan, texts, "startup");
        actions.insert(0, MenuAction::Note("main menu opens at program start (rule 6): no game running, New chosen, Save & Back to Game off".into()));
        (s, actions)
    }

    /// Viewer launched straight into a world (`--skip-menu` or test flag).
    pub fn new_skipped_at_startup() -> Self {
        MainMenuState {
            open: false,
            has_started_or_loaded: true,
            connor_died: false,
            has_saves: false,
            corrupt_saves_ignored: BTreeSet::new(),
            selected: MenuItem::BackToGame,
            popup: None,
            input_block_remaining: 0.0,
        }
    }

    /// Opening the menu from gameplay via Escape or the mask button
    /// (`main-menu.md` rules 7, 8).
    pub fn open_from_gameplay(&mut self, scan: SaveScan, texts: &MenuTexts, how: &str) -> Vec<MenuAction> {
        self.open = true;
        self.connor_died = false;
        let mut actions = vec![MenuAction::Note(format!(
            "main menu opens via {how} (rules 7-8): panels hidden, game paused, 160x120 thumbnail recorded, death mark cleared"
        ))];
        actions.extend(self.apply_show(scan, texts, how));
        actions
    }

    /// Opening the menu after Connor dies (`main-menu.md` rule 25).
    pub fn open_on_death(&mut self, scan: SaveScan, texts: &MenuTexts) -> Vec<MenuAction> {
        self.open = true;
        self.connor_died = true;
        let mut actions = vec![MenuAction::Note(
            "main menu opens after Connor's death (rule 25): death mark set, Save & Back to Game off, New chosen".into(),
        )];
        actions.extend(self.apply_show(scan, texts, "Connor's death"));
        actions
    }

    fn apply_show(&mut self, scan: SaveScan, texts: &MenuTexts, why: &str) -> Vec<MenuAction> {
        let mut actions = Vec::new();
        self.has_saves = scan.total_files > 0;
        self.selected = if self.game_running() { MenuItem::BackToGame } else { MenuItem::New };
        self.input_block_remaining = INPUT_BLOCK_SECONDS;
        if let Some(file) = scan.first_corrupt {
            actions.push(MenuAction::Note(format!("save scan ({why}): corrupt save {file} found; opening question (rule 23)")));
            self.popup = Some(MenuPopup::open(MenuPopupKind::CorruptSave { file }, texts));
        } else {
            self.popup = None;
        }
        actions.push(MenuAction::Note(format!(
            "save scan ({why}): {} save file(s) ({} valid), Load={}, BackToGame={}",
            scan.total_files,
            scan.valid_saves,
            self.is_enabled(MenuItem::Load),
            self.is_enabled(MenuItem::BackToGame)
        )));
        actions.push(MenuAction::StartTheme);
        actions
    }

    /// Real-time clock advance for the 0.2 s input block (`main-menu.md` rules 8, 12).
    pub fn tick(&mut self, dt_secs: f64) {
        if self.input_block_remaining > 0.0 {
            self.input_block_remaining = (self.input_block_remaining - dt_secs.max(0.0)).max(0.0);
        }
    }

    pub fn input_blocked(&self) -> bool {
        self.input_block_remaining > 0.0
    }

    /// Rule 3: a game is running if a New game started or a saved game loaded
    /// since startup, and Connor has not died since.
    pub fn game_running(&self) -> bool {
        self.has_started_or_loaded && !self.connor_died
    }

    /// Rule 3: whether `item` can currently be chosen.
    pub fn is_enabled(&self, item: MenuItem) -> bool {
        match item {
            MenuItem::New | MenuItem::Options | MenuItem::ReplayIntro | MenuItem::Quit => true,
            MenuItem::Load => {
                self.has_saves && !matches!(self.popup.as_ref().map(|p| &p.kind), Some(MenuPopupKind::CorruptSave { .. }))
            }
            MenuItem::Save | MenuItem::BackToGame => self.game_running(),
        }
    }

    /// Up (`delta = -1`) or Down (`delta = 1`) on the list (`main-menu.md` rule 18).
    /// Blocked while a popup is open ("Do not reproduce" 5).
    pub fn move_selection(&mut self, delta: i32) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() || self.popup.is_some() || delta == 0 {
            return Vec::new();
        }
        let step = if delta > 0 { 1isize } else { -1isize };
        let count = MenuItem::ALL.len() as isize;
        let mut idx = self.selected.index() as isize;
        for _ in 0..count {
            idx = (idx + step).rem_euclid(count);
            if let Some(candidate) = MenuItem::from_index(idx as usize) {
                if self.is_enabled(candidate) {
                    self.selected = candidate;
                    return vec![
                        MenuAction::PlayMenuSound { file: MOVE_SOUND, why: format!("menu move to {}", candidate.default_label()) },
                        MenuAction::Note(format!("cursor -> {} (rule 18)", candidate.default_label())),
                    ];
                }
            }
        }
        Vec::new()
    }

    /// Left (`delta = -1`) or Right (`delta = 1`) in an open popup (`main-menu.md` rule 15).
    pub fn move_popup_highlight(&mut self, delta: i32) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() || delta == 0 {
            return Vec::new();
        }
        let Some(popup) = self.popup.as_mut() else {
            return Vec::new();
        };
        let max_idx = popup.button_count().saturating_sub(1);
        let next = if delta < 0 {
            popup.highlight.saturating_sub(1)
        } else {
            (popup.highlight + 1).min(max_idx)
        };
        let moved = next != popup.highlight;
        popup.highlight = next;
        // Rule 15: a click sound plays even when it can't move.
        vec![
            MenuAction::PlayMenuSound { file: MOVE_SOUND, why: "popup Left/Right".into() },
            MenuAction::Note(format!(
                "popup highlight -> button {}{} (rule 15)",
                next,
                if moved { "" } else { " (at edge)" }
            )),
        ]
    }

    /// Enter key going down (`main-menu.md` rules 16, 19; user check 2026-11-23:
    /// pressing Enter on a popup presses the highlighted button without moving
    /// the mouse pointer).
    pub fn press_enter(&mut self, texts: &MenuTexts) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() {
            return Vec::new();
        }
        if let Some(popup) = self.popup.as_mut() {
            popup.pressed = Some(popup.highlight);
            return vec![MenuAction::Note(format!(
                "popup Enter pressed on button {} (rule 16)",
                popup.highlight
            ))];
        }
        let item = self.selected;
        if !self.is_enabled(item) {
            return Vec::new();
        }
        self.activate_item(item, true, texts)
    }

    /// Enter key let go (`main-menu.md` rule 16).
    pub fn release_enter(&mut self, texts: &MenuTexts) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() {
            return Vec::new();
        }
        let Some(popup) = self.popup.as_mut() else {
            return Vec::new();
        };
        let Some(pressed) = popup.pressed.take() else {
            return Vec::new();
        };
        self.pick_popup_button(pressed, "Enter released", texts)
    }

    /// Escape key pressed on the menu (`main-menu.md` rules 9, 16, Edge cases).
    pub fn press_escape(&mut self) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() {
            return Vec::new();
        }
        if let Some(popup) = self.popup.as_ref() {
            match popup.kind {
                MenuPopupKind::Difficulty => {
                    self.popup = None;
                    return vec![MenuAction::Note("Escape on difficulty popup: closed back to menu (rule 16)".into())];
                }
                MenuPopupKind::NewConfirm | MenuPopupKind::CorruptSave { .. } | MenuPopupKind::QuitConfirm => {
                    if self.game_running() {
                        self.popup = None;
                        return self.close_back_to_game("Escape on popup");
                    }
                    return vec![MenuAction::Note("Escape on popup: no game running, nothing (rule 16)".into())];
                }
            }
        }
        if self.game_running() {
            self.close_back_to_game("Escape")
        } else {
            vec![MenuAction::Note("Escape on menu: no game running, nothing (Edge cases)".into())]
        }
    }

    /// Left mouse click at 640 × 480 layout position (`main-menu.md` rules 16, 21,
    /// "Do not reproduce" 1).
    pub fn click_at(&mut self, pos: [f32; 2], texts: &MenuTexts) -> Vec<MenuAction> {
        if !self.open || self.input_blocked() {
            return Vec::new();
        }
        if let Some(popup) = self.popup.as_ref() {
            for (idx, r) in popup.button_rects().into_iter().enumerate() {
                if point_in_rect(pos, r) {
                    return self.pick_popup_button(idx, "mouse click", texts);
                }
            }
            return Vec::new();
        }
        for item in MenuItem::ALL {
            if point_in_rect(pos, item.rect()) {
                if !self.is_enabled(item) {
                    return Vec::new();
                }
                self.selected = item;
                return self.activate_item(item, false, texts);
            }
        }
        Vec::new()
    }

    /// Completes `Replay Intro` (`main-menu.md` rule 12).
    pub fn finish_replay_intro(&mut self) -> Vec<MenuAction> {
        self.input_block_remaining = INPUT_BLOCK_SECONDS;
        vec![MenuAction::Note("Replay Intro finished: input blocked for 0.2 s (rule 12)".into())]
    }

    fn activate_item(&mut self, item: MenuItem, via_enter: bool, texts: &MenuTexts) -> Vec<MenuAction> {
        let how = if via_enter { "Enter" } else { "click" };
        let mut actions = vec![
            MenuAction::PlayMenuSound { file: SELECT_SOUND, why: format!("{} selected ({how})", item.default_label()) },
            MenuAction::Note(format!("item picked via {how}: {} (rules 19/21)", item.default_label())),
        ];
        match item {
            MenuItem::New => {
                if self.game_running() {
                    self.popup = Some(MenuPopup::open(MenuPopupKind::NewConfirm, texts));
                    actions.push(MenuAction::Note("popup opened: NewConfirm (No highlighted; rule 10)".into()));
                } else {
                    self.popup = Some(MenuPopup::open(MenuPopupKind::Difficulty, texts));
                    actions.push(MenuAction::Note("popup opened: Difficulty (Normal highlighted; rule 10)".into()));
                }
            }
            MenuItem::Load | MenuItem::Save | MenuItem::Options => {
                actions.push(MenuAction::OpenSubScreen(item));
            }
            MenuItem::ReplayIntro => {
                actions.push(MenuAction::ReplayIntro);
            }
            MenuItem::Quit => {
                self.popup = Some(MenuPopup::open(MenuPopupKind::QuitConfirm, texts));
                actions.push(MenuAction::Note("popup opened: QuitConfirm (No highlighted; rule 13)".into()));
            }
            MenuItem::BackToGame => {
                actions.extend(self.close_back_to_game(how));
            }
        }
        actions
    }

    fn pick_popup_button(&mut self, button: usize, how: &str, texts: &MenuTexts) -> Vec<MenuAction> {
        let Some(popup) = self.popup.take() else {
            return Vec::new();
        };
        let mut actions = vec![MenuAction::PlayMenuSound {
            file: SELECT_SOUND,
            why: format!("popup button {button} ({how})"),
        }];
        match popup.kind {
            MenuPopupKind::NewConfirm => {
                if button == 0 {
                    self.popup = Some(MenuPopup::open(MenuPopupKind::Difficulty, texts));
                    actions.push(MenuAction::Note(format!("NewConfirm Yes ({how}) -> Difficulty popup (Normal highlighted)")));
                } else {
                    actions.push(MenuAction::Note(format!("NewConfirm No ({how}) -> popup closed")));
                }
            }
            MenuPopupKind::CorruptSave { file } => {
                self.corrupt_saves_ignored.insert(file.to_ascii_lowercase());
                if button == 0 {
                    // "Do not reproduce" 1: click on Yes acts the same as Enter.
                    actions.push(MenuAction::Note(format!("CorruptSave Yes ({how}) -> delete {file} (rule 23, Do not reproduce 1)")));
                    actions.push(MenuAction::DeleteCorruptSave { file });
                } else {
                    actions.push(MenuAction::Note(format!("CorruptSave No ({how}) -> kept {file}")));
                }
            }
            MenuPopupKind::QuitConfirm => {
                if button == 0 {
                    actions.push(MenuAction::Note(format!("QuitConfirm Yes ({how}) -> quit program (rule 13)")));
                    actions.push(MenuAction::Quit);
                } else {
                    actions.push(MenuAction::Note(format!("QuitConfirm No ({how}) -> popup closed")));
                }
            }
            MenuPopupKind::Difficulty => {
                let idx = button.min(2);
                let (level_num, factor, label) = COMBAT_LEVELS[idx];
                self.open = false;
                self.has_started_or_loaded = true;
                self.connor_died = false;
                actions.push(MenuAction::Note(format!(
                    "Difficulty {label} ({how}): combat level {level_num} ({factor}); fading theme & starting Daventry (rules 10, 24)"
                )));
                actions.push(MenuAction::FadeTheme);
                actions.push(MenuAction::StartNewGame { combat_level_number: level_num, factor });
            }
        }
        actions
    }

    fn close_back_to_game(&mut self, how: &str) -> Vec<MenuAction> {
        if !self.game_running() {
            return Vec::new();
        }
        self.open = false;
        self.popup = None;
        vec![
            MenuAction::FadeTheme,
            MenuAction::ResumeGame,
            MenuAction::Note(format!("Back to Game via {how} (rule 9): theme fading out over 1 s, game resumed")),
        ]
    }
}

fn point_in_rect(pos: [f32; 2], rect: [i32; 4]) -> bool {
    pos[0] >= rect[0] as f32 && pos[0] < rect[2] as f32 && pos[1] >= rect[1] as f32 && pos[1] < rect[3] as f32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_pvol(has_saved_name: bool) -> Vec<u8> {
        let name = if has_saved_name { b"savedGameName\0".as_slice() } else { b"otherFile\0".as_slice() };
        let mut b = b"PVOL\0\0\0\0".to_vec();
        b.extend_from_slice(b"VBLK\x04\0\0\x80test");
        let names_at = b.len() as u32;
        b[4..8].copy_from_slice(&names_at.to_le_bytes());
        b.extend_from_slice(b"vols");
        b.extend_from_slice(&(name.len() as u32).to_le_bytes());
        b.extend_from_slice(name);
        while b.len() % 2 != 0 {
            b.push(0);
        }
        b.extend_from_slice(b"voli\x11\0\0\0");
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&0u32.to_le_bytes());
        b.extend_from_slice(&8u32.to_le_bytes());
        b.extend_from_slice(&4u32.to_le_bytes());
        b.push(0);
        b
    }

    #[test]
    fn title_and_popup_text_splitting() {
        let title = parse_title_lines(b"KING'S QUEST\r\nMASK of ETERNITY\r\nInstall\r\n");
        assert_eq!(title, ["KING'S QUEST", "MASK of ETERNITY"]);
        assert_eq!(split_popup_text("Combat difficulty"), vec!["Combat difficulty"]);
        let lines = split_popup_text("Are you sure you want to exit the game?");
        assert_eq!(lines.len(), 2);
        assert_eq!(lines.join(" "), "Are you sure you want to exit the game?");
        let new_lines = split_popup_text("You already have a game running. Are you sure you want to start a NEW one?");
        assert_eq!(new_lines.len(), 2);
        assert!(new_lines[0].chars().count() <= 40 && new_lines[1].chars().count() <= 40);
    }

    #[test]
    fn startup_list_wrapping_skips_disabled_items() {
        let texts = MenuTexts::default();
        let (mut m, actions) = MainMenuState::new_at_startup(SaveScan::default(), &texts);
        assert!(m.open && m.selected == MenuItem::New);
        assert!(actions.contains(&MenuAction::StartTheme));
        assert!(!m.is_enabled(MenuItem::Load));
        assert!(!m.is_enabled(MenuItem::Save));
        assert!(!m.is_enabled(MenuItem::BackToGame));

        // Ignored during 0.2 s block.
        assert!(m.move_selection(1).is_empty());
        m.tick(0.2);

        // Down visits New -> Options -> Replay Intro -> Quit -> New (skipping Load, Save, Back).
        let mut visited = Vec::new();
        for _ in 0..4 {
            let act = m.move_selection(1);
            assert!(act.contains(&MenuAction::PlayMenuSound { file: MOVE_SOUND, why: format!("menu move to {}", m.selected.default_label()) }));
            visited.push(m.selected);
        }
        assert_eq!(visited, [MenuItem::Options, MenuItem::ReplayIntro, MenuItem::Quit, MenuItem::New]);

        // Up from New wraps to Quit when no game is running.
        m.move_selection(-1);
        assert_eq!(m.selected, MenuItem::Quit);
        assert_eq!(MenuItem::Quit.cursor_rect(), [16, 345, 56, 385]);
    }

    #[test]
    fn new_game_difficulty_popup_and_gameplay_resume() {
        let texts = MenuTexts::default();
        let (mut m, _) = MainMenuState::new_at_startup(SaveScan::default(), &texts);
        m.tick(0.2);

        // Escape does nothing when no game is running.
        let esc = m.press_escape();
        assert!(!esc.iter().any(|a| matches!(a, MenuAction::ResumeGame)));

        // Enter on New opens Difficulty with Normal (1) highlighted.
        let act = m.press_enter(&texts);
        assert!(act.iter().any(|a| matches!(a, MenuAction::PlayMenuSound { file: SELECT_SOUND, .. })));
        let p = m.popup.as_ref().unwrap();
        assert_eq!((&p.kind, p.highlight), (&MenuPopupKind::Difficulty, 1));
        // Releasing that opening Enter must NOT pick Normal immediately.
        assert!(m.release_enter(&texts).is_empty());
        assert!(m.popup.is_some());

        // Up/Down blocked while popup open ("Do not reproduce" 5).
        assert!(m.move_selection(1).is_empty());

        // Left -> Easy, Left again -> clamped at 0 and still plays click.aud.
        let l1 = m.move_popup_highlight(-1);
        assert_eq!(m.popup.as_ref().unwrap().highlight, 0);
        assert!(l1.iter().any(|a| matches!(a, MenuAction::PlayMenuSound { file: MOVE_SOUND, .. })));
        let l2 = m.move_popup_highlight(-1);
        assert_eq!(m.popup.as_ref().unwrap().highlight, 0);
        assert!(l2.iter().any(|a| matches!(a, MenuAction::PlayMenuSound { file: MOVE_SOUND, .. })));

        // Right twice -> Hard (2).
        m.move_popup_highlight(1);
        m.move_popup_highlight(1);
        assert_eq!(m.popup.as_ref().unwrap().highlight, 2);

        // Enter presses Hard without moving the pointer and release starts Daventry.
        let press = m.press_enter(&texts);
        assert_eq!(m.popup.as_ref().unwrap().pressed, Some(2));
        assert!(!press.iter().any(|a| matches!(a, MenuAction::StartNewGame { .. })));
        let rel = m.release_enter(&texts);
        assert!(rel.contains(&MenuAction::FadeTheme));
        assert!(rel.contains(&MenuAction::StartNewGame { combat_level_number: 3, factor: 1.5 }));
        assert!(!m.open && m.game_running());

        // Re-open during play: Back to Game chosen, Save enabled.
        let valid_vol = sample_pvol(true);
        let scan = scan_save_entries([("save1.vol", valid_vol.as_slice())], &m.corrupt_saves_ignored);
        let open_act = m.open_from_gameplay(scan, &texts, "Escape");
        assert!(open_act.contains(&MenuAction::StartTheme));
        assert_eq!(m.selected, MenuItem::BackToGame);
        assert!(m.is_enabled(MenuItem::Save) && m.is_enabled(MenuItem::Load));
        m.tick(0.2);

        // New while game running asks Yes/No (No highlighted), Escape drops back to game.
        m.click_at([100.0, 140.0], &texts);
        assert_eq!(m.popup.as_ref().unwrap().kind, MenuPopupKind::NewConfirm);
        assert_eq!(m.popup.as_ref().unwrap().highlight, 1);
        let esc_popup = m.press_escape();
        assert!(esc_popup.contains(&MenuAction::ResumeGame) && !m.open);
    }

    #[test]
    fn corrupt_save_scan_and_death_state() {
        let texts = MenuTexts::default();
        let good = sample_pvol(true);
        let bad_missing = sample_pvol(false);
        let ignored = BTreeSet::new();
        let scan = scan_save_entries([("save1.vol", good.as_slice()), ("save2.vol", bad_missing.as_slice()), ("save3.vol", b"junk".as_slice())], &ignored);
        assert_eq!(scan.total_files, 3);
        assert_eq!(scan.valid_saves, 1);
        assert_eq!(scan.first_corrupt.as_deref(), Some("save2.vol"));

        let (mut m, _) = MainMenuState::new_at_startup(scan, &texts);
        assert!(!m.is_enabled(MenuItem::Load), "Load disabled while corrupt save question is waiting");
        m.tick(0.2);
        // Click Yes on corrupt save ("Do not reproduce" 1).
        let yes_rect = m.popup.as_ref().unwrap().button_rects()[0];
        let click = m.click_at([(yes_rect[0] + 10) as f32, (yes_rect[1] + 10) as f32], &texts);
        assert!(click.contains(&MenuAction::DeleteCorruptSave { file: "save2.vol".into() }));
        assert!(m.popup.is_none() && m.is_enabled(MenuItem::Load));

        // Start a game, then die -> Save and BackToGame off, New chosen.
        m.has_started_or_loaded = true;
        m.open = false;
        m.open_on_death(SaveScan::default(), &texts);
        assert!(m.open && m.connor_died && !m.game_running());
        assert_eq!(m.selected, MenuItem::New);
        assert!(!m.is_enabled(MenuItem::Save) && !m.is_enabled(MenuItem::BackToGame));
    }
}
