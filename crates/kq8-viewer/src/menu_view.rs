//! Main menu view (`docs/behaviour/main-menu.md`, M4 step 4f-2).
//!
//! Loads `GAME/Resource/Mainmenu.gui`, `GAME/8Gui/Menus.ppl`, the menu
//! pictures (`main18.pbm`, `45-line.pbm`, `Mskcur.pba`, `gm_win2.pbm`,
//! `g_ybtn.pba`, `g_yhibtn.pba`, `Cursgui.pba`), fonts (`27.pft`, `45.pft`,
//! `consoleg.pft`), `GAME/ENGLISH/500.MSG`, and `starttxt.txt`.
//! Draws the 640 × 480 menu in the centred 4:3 screen frame, drives the
//! core [`MainMenuState`] rules, coordinates sound pause and `theme.aud`
//! with [`SoundSim`], and logs every menu event to `kq8-artifacts/logs/menu.log`.

use std::collections::HashMap;
use std::path::PathBuf;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use kq8_core::gui_draw::{self, Pic, Piece};
use kq8_core::main_menu::{
    self, MainMenuState, MenuAction, MenuItem, MenuPopup, MenuTexts, SaveScan, BG_PICTURE,
    CURSOR_PICTURE, GUI_FILE, ITEM_FONT, LINE_PICTURE, PALETTE_FILE, POINTER_FILE,
    POPUP_BG_PICTURE, POPUP_BG_RECT, POPUP_BTN_PICTURE, POPUP_FONT, POPUP_HI_BTN_PICTURE,
    THUMBNAIL_HEIGHT, THUMBNAIL_WIDTH, TITLE_FONT, TITLE_LINE_RECT, TITLE_RECTS,
};
use kq8_core::submenus::{
    brightness_factor, move_label_rect, move_value_rect, CreditsState, GraphicsState, MenuScreen,
    MoveKeysFile, MoveKeysState, OptionsState, SubmenuAction, SubmenuTexts, UserOptions,
    CREDITS_PHOTO_FILE, CREDITS_PHOTO_RECT, GRA_BRIGHTNESS_SLIDER_RECT, GRA_LABEL_RECTS,
    GRA_MODE_LABEL_RECT, GRA_RES_LABEL_RECT, GRA_ROW_COUNT, GRA_SHADOW_BITS, GRA_TITLE_RECT,
    GRA_VALUE_RECTS, MOVE_KEY_COUNT, MOVE_TITLE_RECT, OPT_LABEL_RECTS,
    OPT_ROW_COUNT, OPT_ROW_RECTS, OPT_SLIDER_RECTS, OPT_TITLE_RECT, OPT_VALUE_RECTS,
    SLIDER_KNOB_SIZE, SLIDER_LEFT,
};
use kq8_data::font::Font;
use kq8_data::msg::MessageFile;
use kq8_data::palette::Rgb;
use kq8_data::{Bitmap, GameDir};

use crate::gui_view::{KeyAt, ScreenFrame};
use crate::loading_screen::LoadingScreen;
use crate::log::Log;
use crate::sound_view::SoundSim;
use crate::world_build::{Persistent, WorldChange};
use crate::Shared;

/// Mask cursor animation speed (`interface.md` rule 26a / Q108 d: 10 fps real time).
const CURSOR_FPS: f64 = 10.0;

const SUBMENU_BG_PICTURE: &str = "blkbkgd.pbm";
const BLACK_PICTURE: &str = "4black.pbm";
const SUBMENU_CURSOR_PICTURE: &str = "mskicng.pba";
const SLIDER_PICTURE: &str = "slider.pba";
const OPT_FONT: &str = "20.pft";
const CREDITS_SMALL_FONT: &str = "20sl.pft";
const CREDITS_LARGE_FONT: &str = "27sl.pft";
const SL_TITLE_FONT: &str = "45sl.pft";
const SL_POPUP_FONT: &str = "consoles.pft";

/// Path for user-written config files (`Options.cs`, `movekeys.kq`) under
/// `kq8-artifacts/user/` so `MASK/` remains strictly read-only (`AGENTS.md` rule 4).
pub fn user_config_path(filename: &str) -> PathBuf {
    PathBuf::from("kq8-artifacts").join("user").join(filename)
}

/// Reads `<filename>` from `kq8-artifacts/user/` first, falling back to `MASK/` read-only.
pub fn read_user_or_game(game: &GameDir, filename: &str) -> Option<Vec<u8>> {
    let user_path = user_config_path(filename);
    if let Ok(bytes) = std::fs::read(&user_path) {
        return Some(bytes);
    }
    game.find(filename).and_then(|p| std::fs::read(p).ok())
}

/// Writes `<filename>` into `kq8-artifacts/user/<filename>`, creating parent folders as needed.
pub fn write_user_config(filename: &str, contents: &str) -> std::io::Result<PathBuf> {
    let path = user_config_path(filename);
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(&path, contents.as_bytes())?;
    Ok(path)
}

/// Maps a Windows virtual-key code (`movekeys.kq` `KeyChar<n>`) to a Bevy `KeyCode`.
pub fn vk_to_keycode(vk: u32) -> Option<KeyCode> {
    Some(match vk {
        0x08 => KeyCode::Backspace,
        0x09 => KeyCode::Tab,
        0x0D => KeyCode::Enter,
        0x10 => KeyCode::ShiftLeft,
        0x11 => KeyCode::ControlLeft,
        0x12 => KeyCode::AltLeft,
        0x1B => KeyCode::Escape,
        0x20 => KeyCode::Space,
        0x21 => KeyCode::PageUp,
        0x22 => KeyCode::PageDown,
        0x23 => KeyCode::End,
        0x24 => KeyCode::Home,
        0x25 => KeyCode::ArrowLeft,
        0x26 => KeyCode::ArrowUp,
        0x27 => KeyCode::ArrowRight,
        0x28 => KeyCode::ArrowDown,
        0x2D => KeyCode::Insert,
        0x2E => KeyCode::Delete,
        0x30 => KeyCode::Digit0,
        0x31 => KeyCode::Digit1,
        0x32 => KeyCode::Digit2,
        0x33 => KeyCode::Digit3,
        0x34 => KeyCode::Digit4,
        0x35 => KeyCode::Digit5,
        0x36 => KeyCode::Digit6,
        0x37 => KeyCode::Digit7,
        0x38 => KeyCode::Digit8,
        0x39 => KeyCode::Digit9,
        0x41 => KeyCode::KeyA,
        0x42 => KeyCode::KeyB,
        0x43 => KeyCode::KeyC,
        0x44 => KeyCode::KeyD,
        0x45 => KeyCode::KeyE,
        0x46 => KeyCode::KeyF,
        0x47 => KeyCode::KeyG,
        0x48 => KeyCode::KeyH,
        0x49 => KeyCode::KeyI,
        0x4A => KeyCode::KeyJ,
        0x4B => KeyCode::KeyK,
        0x4C => KeyCode::KeyL,
        0x4D => KeyCode::KeyM,
        0x4E => KeyCode::KeyN,
        0x4F => KeyCode::KeyO,
        0x50 => KeyCode::KeyP,
        0x51 => KeyCode::KeyQ,
        0x52 => KeyCode::KeyR,
        0x53 => KeyCode::KeyS,
        0x54 => KeyCode::KeyT,
        0x55 => KeyCode::KeyU,
        0x56 => KeyCode::KeyV,
        0x57 => KeyCode::KeyW,
        0x58 => KeyCode::KeyX,
        0x59 => KeyCode::KeyY,
        0x5A => KeyCode::KeyZ,
        0x60 => KeyCode::Numpad0,
        0x61 => KeyCode::Numpad1,
        0x62 => KeyCode::Numpad2,
        0x63 => KeyCode::Numpad3,
        0x64 => KeyCode::Numpad4,
        0x65 => KeyCode::Numpad5,
        0x66 => KeyCode::Numpad6,
        0x67 => KeyCode::Numpad7,
        0x68 => KeyCode::Numpad8,
        0x69 => KeyCode::Numpad9,
        0x6A => KeyCode::NumpadMultiply,
        0x6B => KeyCode::NumpadAdd,
        0x6D => KeyCode::NumpadSubtract,
        0x6E => KeyCode::NumpadDecimal,
        0x6F => KeyCode::NumpadDivide,
        0x70 => KeyCode::F1,
        0x71 => KeyCode::F2,
        0x72 => KeyCode::F3,
        0x73 => KeyCode::F4,
        0x74 => KeyCode::F5,
        0x75 => KeyCode::F6,
        0x76 => KeyCode::F7,
        0x77 => KeyCode::F8,
        0x78 => KeyCode::F9,
        0x79 => KeyCode::F10,
        0x7A => KeyCode::F11,
        0x7B => KeyCode::F12,
        0xBA => KeyCode::Semicolon,
        0xBB => KeyCode::Equal,
        0xBC => KeyCode::Comma,
        0xBD => KeyCode::Minus,
        0xBE => KeyCode::Period,
        0xBF => KeyCode::Slash,
        0xC0 => KeyCode::Backquote,
        0xDB => KeyCode::BracketLeft,
        0xDC => KeyCode::Backslash,
        0xDD => KeyCode::BracketRight,
        0xDE => KeyCode::Quote,
        _ => return None,
    })
}

/// Maps a Bevy `KeyCode` back to a Windows virtual-key code for Remap Move Keys.
pub fn keycode_to_vk(key: KeyCode) -> Option<u32> {
    Some(match key {
        KeyCode::Backspace => 0x08,
        KeyCode::Tab => 0x09,
        KeyCode::Enter | KeyCode::NumpadEnter => 0x0D,
        KeyCode::ShiftLeft | KeyCode::ShiftRight => 0x10,
        KeyCode::ControlLeft | KeyCode::ControlRight => 0x11,
        KeyCode::AltLeft | KeyCode::AltRight => 0x12,
        KeyCode::Escape => 0x1B,
        KeyCode::Space => 0x20,
        KeyCode::PageUp => 0x21,
        KeyCode::PageDown => 0x22,
        KeyCode::End => 0x23,
        KeyCode::Home => 0x24,
        KeyCode::ArrowLeft => 0x25,
        KeyCode::ArrowUp => 0x26,
        KeyCode::ArrowRight => 0x27,
        KeyCode::ArrowDown => 0x28,
        KeyCode::Insert => 0x2D,
        KeyCode::Delete => 0x2E,
        KeyCode::Digit0 => 0x30,
        KeyCode::Digit1 => 0x31,
        KeyCode::Digit2 => 0x32,
        KeyCode::Digit3 => 0x33,
        KeyCode::Digit4 => 0x34,
        KeyCode::Digit5 => 0x35,
        KeyCode::Digit6 => 0x36,
        KeyCode::Digit7 => 0x37,
        KeyCode::Digit8 => 0x38,
        KeyCode::Digit9 => 0x39,
        KeyCode::KeyA => 0x41,
        KeyCode::KeyB => 0x42,
        KeyCode::KeyC => 0x43,
        KeyCode::KeyD => 0x44,
        KeyCode::KeyE => 0x45,
        KeyCode::KeyF => 0x46,
        KeyCode::KeyG => 0x47,
        KeyCode::KeyH => 0x48,
        KeyCode::KeyI => 0x49,
        KeyCode::KeyJ => 0x4A,
        KeyCode::KeyK => 0x4B,
        KeyCode::KeyL => 0x4C,
        KeyCode::KeyM => 0x4D,
        KeyCode::KeyN => 0x4E,
        KeyCode::KeyO => 0x4F,
        KeyCode::KeyP => 0x50,
        KeyCode::KeyQ => 0x51,
        KeyCode::KeyR => 0x52,
        KeyCode::KeyS => 0x53,
        KeyCode::KeyT => 0x54,
        KeyCode::KeyU => 0x55,
        KeyCode::KeyV => 0x56,
        KeyCode::KeyW => 0x57,
        KeyCode::KeyX => 0x58,
        KeyCode::KeyY => 0x59,
        KeyCode::KeyZ => 0x5A,
        KeyCode::Numpad0 => 0x60,
        KeyCode::Numpad1 => 0x61,
        KeyCode::Numpad2 => 0x62,
        KeyCode::Numpad3 => 0x63,
        KeyCode::Numpad4 => 0x64,
        KeyCode::Numpad5 => 0x65,
        KeyCode::Numpad6 => 0x66,
        KeyCode::Numpad7 => 0x67,
        KeyCode::Numpad8 => 0x68,
        KeyCode::Numpad9 => 0x69,
        KeyCode::NumpadMultiply => 0x6A,
        KeyCode::NumpadAdd => 0x6B,
        KeyCode::NumpadSubtract => 0x6D,
        KeyCode::NumpadDecimal => 0x6E,
        KeyCode::NumpadDivide => 0x6F,
        KeyCode::F1 => 0x70,
        KeyCode::F2 => 0x71,
        KeyCode::F3 => 0x72,
        KeyCode::F4 => 0x73,
        KeyCode::F5 => 0x74,
        KeyCode::F6 => 0x75,
        KeyCode::F7 => 0x76,
        KeyCode::F8 => 0x77,
        KeyCode::F9 => 0x78,
        KeyCode::F10 => 0x79,
        KeyCode::F11 => 0x7A,
        KeyCode::F12 => 0x7B,
        KeyCode::Semicolon => 0xBA,
        KeyCode::Equal => 0xBB,
        KeyCode::Comma => 0xBC,
        KeyCode::Minus => 0xBD,
        KeyCode::Period => 0xBE,
        KeyCode::Slash => 0xBF,
        KeyCode::Backquote => 0xC0,
        KeyCode::BracketLeft => 0xDB,
        KeyCode::Backslash => 0xDC,
        KeyCode::BracketRight => 0xDD,
        KeyCode::Quote => 0xDE,
        _ => return None,
    })
}

fn just_pressed_vk(keys: &ButtonInput<KeyCode>) -> Option<u32> {
    keys.get_just_pressed().find_map(|&k| keycode_to_vk(k))
}

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuCanvasNode;

#[derive(Component)]
pub struct MenuCamera;

/// In-memory 160 × 120 thumbnail saved when Escape / the mask button opens
/// the menu from gameplay (`main-menu.md` rules 7.3, 22, "Do not reproduce" 3).
#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct SavedThumbnail {
    pub width: u32,
    pub height: u32,
    pub world: String,
    pub pixels: Vec<u8>,
}

#[derive(Resource)]
pub struct MenuView {
    pub game: GameDir,
    pub state: MainMenuState,
    pub screen: MenuScreen,
    pub texts: MenuTexts,
    pub submenu_texts: SubmenuTexts,
    pub options: UserOptions,
    pub options_state: OptionsState,
    pub graphics_state: GraphicsState,
    pub movekeys_state: MoveKeysState,
    pub credits_state: Option<CreditsState>,
    pub slider_drag: Option<usize>,
    pub gra_brightness_drag: bool,
    pub msg400: Option<MessageFile>,
    #[allow(dead_code)]
    pub msg500: Option<MessageFile>,
    pub palette: [Rgb; 256],
    files: HashMap<String, Vec<Bitmap>>,
    fonts: HashMap<String, Font>,
    pub now_ms: f64,
    pub total_frames: u64,
    pub pending_actions: Vec<MenuAction>,
    pub thumbnail: Option<SavedThumbnail>,
    /// Countdown (seconds) after Connor enters `Mode::Dead` before the menu
    /// opens (`main-menu.md` rule 25: "in handsOff for a couple of seconds").
    pub death_timer: Option<f64>,
    pub death_handled: bool,
    last_drawn_key: Option<MenuDrawKey>,
    pub log: Option<Log>,
}

#[derive(Debug, Clone, PartialEq)]
struct MenuDrawKey {
    screen: MenuScreen,
    selected: MenuItem,
    enabled_mask: u8,
    cursor_frame: usize,
    popup: Option<MenuPopup>,
    options_selected: usize,
    options_sliders: [i32; 3],
    options_flags: (bool, u8, bool, u32, i32, bool),
    graphics_selected: usize,
    movekeys_state: (usize, Option<usize>, Option<usize>, Vec<(u32, String)>),
    credits_page: Option<usize>,
}

fn find_bytes(game: &GameDir, candidates: &[&str]) -> Option<(String, Vec<u8>)> {
    candidates.iter().find_map(|&rel| {
        let p = game.find(rel)?;
        Some((rel.to_string(), std::fs::read(p).ok()?))
    })
}

impl MenuView {
    pub fn load(game: &GameDir, skip_at_startup: bool, main_log: &mut Log) -> Self {
        let mut log = match Log::create("menu") {
            Ok(l) => Some(l),
            Err(e) => {
                main_log.summary(format!("menu: cannot create menu.log: {e}"));
                None
            }
        };
        if let Some(l) = &mut log {
            l.detail("menu.log: main menu events, item selection, popups, save scan, combat level, pause and theme music (docs/behaviour/main-menu.md)");
        }

        let palette = match find_bytes(game, &[&format!("GAME/8Gui/{PALETTE_FILE}"), &format!("GAME/Resource/{PALETTE_FILE}")])
            .and_then(|(_, b)| kq8_data::Palette::parse(&b).ok())
        {
            Some(p) => kq8_data::palette::as_shown(p.colors),
            None => {
                main_log.summary(format!("menu: {PALETTE_FILE} not found; using greyscale fallback"));
                std::array::from_fn(|i| [i as u8, i as u8, i as u8])
            }
        };

        let gui_ok = find_bytes(game, &[&format!("GAME/Resource/{GUI_FILE}"), &format!("GAME/8Gui/{GUI_FILE}")])
            .and_then(|(_, b)| kq8_data::gui::parse(&b).ok())
            .map(|c| c.controls.len())
            .unwrap_or(0);

        let msg500 = find_bytes(game, &["GAME/ENGLISH/500.MSG"]).and_then(|(_, b)| MessageFile::parse(500, &b).ok());
        let msg400 = find_bytes(game, &["GAME/ENGLISH/400.MSG"]).and_then(|(_, b)| MessageFile::parse(400, &b).ok());
        let starttxt = find_bytes(game, &["starttxt.txt"]).map(|(_, b)| b);
        let texts = MenuTexts::from_msg_and_title(msg500.as_ref(), starttxt.as_deref());
        let submenu_texts = SubmenuTexts::from_msg(msg500.as_ref());
        let options = read_user_or_game(game, "Options.cs")
            .map(|b| UserOptions::parse(&b))
            .unwrap_or_default();
        let options_state = OptionsState::from_options(&options);
        let movekeys_file = read_user_or_game(game, "movekeys.kq")
            .map(|b| MoveKeysFile::parse(&b))
            .unwrap_or_default();
        let movekeys_state = MoveKeysState::new(movekeys_file);

        let mut files = HashMap::new();
        for name in [
            BG_PICTURE,
            LINE_PICTURE,
            CURSOR_PICTURE,
            POPUP_BG_PICTURE,
            POPUP_BTN_PICTURE,
            POPUP_HI_BTN_PICTURE,
            POINTER_FILE,
            SUBMENU_BG_PICTURE,
            BLACK_PICTURE,
            SUBMENU_CURSOR_PICTURE,
            SLIDER_PICTURE,
            CREDITS_PHOTO_FILE,
        ] {
            let rel1 = format!("GAME/8Gui/{name}");
            let rel2 = format!("GAME/Resource/{name}");
            if let Some((_, bytes)) = find_bytes(game, &[&rel1, &rel2]) {
                let parsed = if name.to_ascii_lowercase().ends_with(".pba") {
                    kq8_data::font::bitmap_array(&bytes)
                } else {
                    Bitmap::parse(&bytes).map(|b| vec![b])
                };
                if let Ok(bmps) = parsed {
                    files.insert(name.to_ascii_lowercase(), bmps);
                }
            }
        }

        let mut fonts = HashMap::new();
        for name in [
            ITEM_FONT,
            TITLE_FONT,
            POPUP_FONT,
            OPT_FONT,
            CREDITS_SMALL_FONT,
            CREDITS_LARGE_FONT,
            SL_TITLE_FONT,
            SL_POPUP_FONT,
        ] {
            let rel1 = format!("GAME/8Gui/{name}");
            let rel2 = format!("GAME/Resource/{name}");
            if let Some((_, bytes)) = find_bytes(game, &[&rel1, &rel2]) {
                if let Ok(f) = Font::parse(&bytes) {
                    fonts.insert(name.to_ascii_lowercase(), f);
                }
            }
        }

        let (state, pending_actions) = if skip_at_startup {
            (MainMenuState::new_skipped_at_startup(), Vec::new())
        } else {
            let scan = Self::scan_saves(game, &Default::default());
            MainMenuState::new_at_startup(scan, &texts)
        };

        let head = format!(
            "menu: {GUI_FILE} ({gui_ok} controls) in {PALETTE_FILE}; title {:?} / {:?}; {} pictures, {} fonts loaded; startup {}",
            texts.title[0],
            texts.title[1],
            files.len(),
            fonts.len(),
            if skip_at_startup { "skipped (--skip-menu / direct world launch)" } else { "shown (New selected)" }
        );
        main_log.summary(&head);
        if let Some(l) = &mut log {
            l.summary(&head);
        }

        MenuView {
            game: game.clone(),
            state,
            screen: MenuScreen::MainMenu,
            texts,
            submenu_texts,
            options,
            options_state,
            graphics_state: GraphicsState::default(),
            movekeys_state,
            credits_state: None,
            slider_drag: None,
            gra_brightness_drag: false,
            msg400,
            msg500,
            palette,
            files,
            fonts,
            now_ms: 0.0,
            total_frames: 0,
            pending_actions,
            thumbnail: None,
            death_timer: None,
            death_handled: false,
            last_drawn_key: None,
            log,
        }
    }

    pub fn scan_saves(game: &GameDir, ignored: &std::collections::BTreeSet<String>) -> SaveScan {
        match game.find("save") {
            Some(dir) => main_menu::scan_save_folder(&dir, ignored),
            None => SaveScan::default(),
        }
    }

    pub fn note(&mut self, line: impl AsRef<str>) {
        if let Some(l) = &mut self.log {
            l.detail(format!("t {:8.3} s (frame {}): {}", self.now_ms / 1000.0, self.total_frames, line.as_ref()));
        }
    }

    pub fn summary(&mut self, main_log: &Shared, line: impl AsRef<str>) {
        let s = line.as_ref();
        self.note(s);
        main_log.log.lock().unwrap().summary(format!("menu: {s}"));
    }

    /// Opens the main menu from gameplay (`main-menu.md` rules 7, 8).
    pub fn open_from_gameplay(&mut self, world_name: &str, how: &str, sound: Option<&mut SoundSim>, shared: &Shared) {
        self.screen = MenuScreen::MainMenu;
        self.credits_state = None;
        self.slider_drag = None;
        // Rule 7.3 & 22 ("Do not reproduce" 3): 160 x 120 thumbnail kept in memory.
        self.thumbnail = Some(SavedThumbnail {
            width: THUMBNAIL_WIDTH,
            height: THUMBNAIL_HEIGHT,
            world: world_name.to_string(),
            pixels: vec![0; (THUMBNAIL_WIDTH * THUMBNAIL_HEIGHT) as usize],
        });
        if let Some(snd) = sound {
            snd.set_game_paused(true);
        }
        let scan = Self::scan_saves(&self.game, &self.state.corrupt_saves_ignored);
        let texts = self.texts.clone();
        let actions = self.state.open_from_gameplay(scan, &texts, how);
        self.last_drawn_key = None;
        self.summary(
            shared,
            format!(
                "opened from gameplay ({how}) in {world_name}: 160x120 thumbnail saved in memory (rule 22), game & world sounds paused (rule 7.2), chosen={:?}",
                self.state.selected
            ),
        );
        self.pending_actions.extend(actions);
    }

    /// Opens the main menu after Connor's death (`main-menu.md` rule 25).
    pub fn open_on_death(&mut self, world_name: &str, sound: Option<&mut SoundSim>, shared: &Shared) {
        self.screen = MenuScreen::MainMenu;
        self.credits_state = None;
        self.slider_drag = None;
        if let Some(snd) = sound {
            snd.set_game_paused(true);
        }
        let scan = Self::scan_saves(&self.game, &self.state.corrupt_saves_ignored);
        let texts = self.texts.clone();
        let actions = self.state.open_on_death(scan, &texts);
        self.last_drawn_key = None;
        self.summary(
            shared,
            format!("opened after Connor's death in {world_name} (rule 25): Save & Back to Game off, New chosen"),
        );
        self.pending_actions.extend(actions);
    }

    fn cursor_frame_count(&self) -> usize {
        self.files.get(&CURSOR_PICTURE.to_ascii_lowercase()).map_or(1, |v| v.len().max(1))
    }

    fn cursor_frame(&self) -> usize {
        let n = self.cursor_frame_count();
        ((self.now_ms / (1000.0 / CURSOR_FPS)) as usize) % n
    }

    fn submenu_cursor_frame(&self) -> usize {
        let n = self.files.get(&SUBMENU_CURSOR_PICTURE.to_ascii_lowercase()).map_or(1, |v| v.len().max(1));
        ((self.now_ms / (1000.0 / CURSOR_FPS)) as usize) % n
    }

    fn draw_key(&self) -> MenuDrawKey {
        let mut mask = 0u8;
        for item in MenuItem::ALL {
            if self.state.is_enabled(item) {
                mask |= 1 << item.index();
            }
        }
        let cursor_frame = if self.screen == MenuScreen::MainMenu {
            self.cursor_frame()
        } else {
            self.submenu_cursor_frame()
        };
        MenuDrawKey {
            screen: self.screen,
            selected: self.state.selected,
            enabled_mask: mask,
            cursor_frame,
            popup: self.state.popup.clone(),
            options_selected: self.options_state.selected,
            options_sliders: [
                self.options_state.music_slider,
                self.options_state.sfx_slider,
                self.options_state.speech_slider,
            ],
            options_flags: (
                self.options.captions,
                self.options.not_starting_help,
                self.options.dynamic_lighting,
                self.options.shadow_mask,
                self.options.brightness,
                self.options.mode_800,
            ),
            graphics_selected: self.graphics_state.selected,
            movekeys_state: (
                self.movekeys_state.selected,
                self.movekeys_state.capturing,
                self.movekeys_state.invalid_key_row,
                self.movekeys_state
                    .file
                    .bindings
                    .iter()
                    .map(|b| (b.vk, b.name_tag.clone()))
                    .collect(),
            ),
            credits_page: self.credits_state.as_ref().map(|c| c.page),
        }
    }

    fn push_slider<'a>(&'a self, pieces: &mut Vec<Piece<'a>>, rect: [i32; 4], val: i32) {
        let Some(frames) = self.files.get(&SLIDER_PICTURE.to_ascii_lowercase()) else {
            return;
        };
        if let Some(track) = frames.first().and_then(Pic::of) {
            let mut x = rect[0];
            while x < rect[2] {
                let r = (x + SLIDER_KNOB_SIZE).min(rect[2]);
                pieces.push(Piece::Picture {
                    rect: [x, rect[1], r, rect[3]],
                    pic: track,
                    stretch: true,
                });
                x += SLIDER_KNOB_SIZE;
            }
        }
        if let Some(knob) = frames.get(1).or_else(|| frames.first()).and_then(Pic::of) {
            let span = (rect[2] - rect[0] - SLIDER_KNOB_SIZE).max(0);
            let kx = SLIDER_LEFT + (val.clamp(0, 100) * span) / 100;
            pieces.push(Piece::Picture {
                rect: [kx, rect[1], kx + SLIDER_KNOB_SIZE, rect[3]],
                pic: knob,
                stretch: true,
            });
        }
    }

    fn push_submenu_cursors<'a>(&'a self, pieces: &mut Vec<Piece<'a>>, rects: [[i32; 4]; 2]) {
        if let Some(frames) = self.files.get(&SUBMENU_CURSOR_PICTURE.to_ascii_lowercase()) {
            let f = self.submenu_cursor_frame().min(frames.len().saturating_sub(1));
            if let Some(pic) = frames.get(f).and_then(Pic::of) {
                for rect in rects {
                    pieces.push(Piece::Picture { rect, pic, stretch: true });
                }
            }
        }
    }

    fn push_submenu_bg<'a>(&'a self, pieces: &mut Vec<Piece<'a>>, file: &str) {
        if let Some(bmp) = self.files.get(&file.to_ascii_lowercase()).and_then(|v| v.first()) {
            if let Some(mut pic) = Pic::of(bmp) {
                pic.see_through = false;
                pieces.push(Piece::Picture { rect: [0, 0, 640, 480], pic, stretch: true });
            }
        }
    }

    /// Renders the active menu screen (640 × 480) into a [`gui_draw::Canvas`].
    pub fn render_canvas(&self) -> gui_draw::Canvas {
        match self.screen {
            MenuScreen::Options => return self.render_options(),
            MenuScreen::Graphics => return self.render_graphics(),
            MenuScreen::MoveKeys => return self.render_movekeys(),
            MenuScreen::Credits => return self.render_credits(),
            _ => {}
        }
        // `gui_draw::draw` draws `pieces` in reverse order (first element on top),
        // so we build `pieces` from bottom layer to top layer and reverse at the end.
        let mut pieces: Vec<Piece> = Vec::new();

        // 1. Background `main18.pbm` (640 x 480, opaque colour 0).
        if let Some(bmp) = self.files.get(&BG_PICTURE.to_ascii_lowercase()).and_then(|v| v.first()) {
            if let Some(mut pic) = Pic::of(bmp) {
                pic.see_through = false;
                pieces.push(Piece::Picture { rect: [0, 0, 640, 480], pic, stretch: true });
            }
        }

        // 2. Title lines (`45.pft`) and separator (`45-line.pbm`).
        if let Some(font) = self.fonts.get(&TITLE_FONT.to_ascii_lowercase()) {
            pieces.push(Piece::Text {
                rect: TITLE_RECTS[0],
                font,
                justification: 0xC,
                text: self.texts.title[0].clone(),
            });
            pieces.push(Piece::Text {
                rect: TITLE_RECTS[1],
                font,
                justification: 0xC,
                text: self.texts.title[1].clone(),
            });
        }
        if let Some(bmp) = self.files.get(&LINE_PICTURE.to_ascii_lowercase()).and_then(|v| v.first()) {
            if let Some(pic) = Pic::of(bmp) {
                pieces.push(Piece::Picture { rect: TITLE_LINE_RECT, pic, stretch: true });
            }
        }

        // 3. The 7 menu items (`27.pft`), normal when enabled, dithered when disabled (rule 3).
        if let Some(font) = self.fonts.get(&ITEM_FONT.to_ascii_lowercase()) {
            for item in MenuItem::ALL {
                let text = self.texts.items[item.index()].clone();
                let rect = item.rect();
                if self.state.is_enabled(item) {
                    pieces.push(Piece::Text { rect, font, justification: 0x11, text });
                } else {
                    pieces.push(Piece::TextDithered { rect, font, justification: 0x11, text });
                }
            }
        }

        // 4. Mask cursor (`Mskcur.pba`, 40 x 40 at x 16, level with chosen item, rule 5).
        if let Some(frames) = self.files.get(&CURSOR_PICTURE.to_ascii_lowercase()) {
            let f = self.cursor_frame().min(frames.len().saturating_sub(1));
            if let Some(pic) = frames.get(f).and_then(Pic::of) {
                pieces.push(Piece::Picture {
                    rect: self.state.selected.cursor_rect(),
                    pic,
                    stretch: true,
                });
            }
        }

        // 5. Modal popup (`gm_win2.pbm`, `consoleg.pft`, `g_ybtn.pba` / `g_yhibtn.pba`, rules 14-16).
        if let Some(popup) = &self.state.popup {
            if let Some(bmp) = self.files.get(&POPUP_BG_PICTURE.to_ascii_lowercase()).and_then(|v| v.first()) {
                if let Some(mut pic) = Pic::of(bmp) {
                    pic.see_through = false;
                    pieces.push(Piece::Picture { rect: POPUP_BG_RECT, pic, stretch: false });
                }
            }
            if let Some(font) = self.fonts.get(&POPUP_FONT.to_ascii_lowercase()) {
                for (line, rect) in popup.lines.iter().zip(popup.text_rects()) {
                    pieces.push(Piece::Text {
                        rect,
                        font,
                        justification: 0xC,
                        text: line.clone(),
                    });
                }
            }
            let labels = popup.button_labels(&self.texts);
            for (idx, (rect, label)) in popup.button_rects().into_iter().zip(labels).enumerate() {
                let btn_file = if idx == popup.highlight { POPUP_HI_BTN_PICTURE } else { POPUP_BTN_PICTURE };
                let pressed = popup.pressed == Some(idx);
                if let Some(frames) = self.files.get(&btn_file.to_ascii_lowercase()) {
                    let f = usize::from(pressed).min(frames.len().saturating_sub(1));
                    if let Some(mut pic) = frames.get(f).and_then(Pic::of) {
                        pic.see_through = false;
                        pieces.push(Piece::Picture { rect, pic, stretch: true });
                    }
                }
                if let Some(font) = self.fonts.get(&POPUP_FONT.to_ascii_lowercase()) {
                    let shift = if pressed { 1 } else { 0 };
                    let text_rect = [rect[0] + shift, rect[1] + shift, rect[2] + shift, rect[3] + shift];
                    pieces.push(Piece::Text {
                        rect: text_rect,
                        font,
                        justification: 0xC,
                        text: label.to_string(),
                    });
                }
            }
        }

        pieces.reverse();
        gui_draw::draw([0, 0], [640, 480], 1.0, &pieces, &self.palette)
    }

    fn render_options(&self) -> gui_draw::Canvas {
        let mut pieces: Vec<Piece> = Vec::new();
        self.push_submenu_bg(&mut pieces, SUBMENU_BG_PICTURE);

        if let Some(font) = self.fonts.get(&TITLE_FONT.to_ascii_lowercase()) {
            pieces.push(Piece::Text {
                rect: OPT_TITLE_RECT,
                font,
                justification: 0xC,
                text: self.submenu_texts.options_title.clone(),
            });
        }

        let font20 = self.fonts.get(&OPT_FONT.to_ascii_lowercase());
        let font27 = self.fonts.get(&ITEM_FONT.to_ascii_lowercase());
        for row in 0..OPT_ROW_COUNT {
            let enabled = OptionsState::is_row_enabled(row, &self.options);
            let font = if row == 8 { font27 } else { font20 };
            if let Some(font) = font {
                let just = if matches!(row, 0 | 1 | 7 | 8) { 0xC } else { 0xA };
                let rect = OPT_LABEL_RECTS[row];
                let text = self.submenu_texts.opt_rows[row].clone();
                if enabled {
                    pieces.push(Piece::Text { rect, font, justification: just, text });
                } else {
                    pieces.push(Piece::TextDithered { rect, font, justification: just, text });
                }
            }
        }

        let slider_vals = [
            self.options_state.music_slider,
            self.options_state.sfx_slider,
            self.options_state.speech_slider,
        ];
        for (idx, &(_, rect)) in OPT_SLIDER_RECTS.iter().enumerate() {
            self.push_slider(&mut pieces, rect, slider_vals[idx]);
        }

        if let Some(font) = font20 {
            for &(row, rect) in &OPT_VALUE_RECTS {
                let is_yes = if row == 5 {
                    self.options.captions
                } else {
                    self.options.not_starting_help == 0
                };
                let text = if is_yes {
                    self.submenu_texts.yes.clone()
                } else {
                    self.submenu_texts.no.clone()
                };
                if OptionsState::is_row_enabled(row, &self.options) {
                    pieces.push(Piece::Text { rect, font, justification: 0x9, text });
                } else {
                    pieces.push(Piece::TextDithered { rect, font, justification: 0x9, text });
                }
            }
        }

        self.push_submenu_cursors(&mut pieces, self.options_state.cursor_rects());

        pieces.reverse();
        gui_draw::draw([0, 0], [640, 480], 1.0, &pieces, &self.palette)
    }

    fn render_credits(&self) -> gui_draw::Canvas {
        let mut pieces: Vec<Piece> = Vec::new();
        self.push_submenu_bg(&mut pieces, BLACK_PICTURE);

        if let Some(credits) = &self.credits_state {
            if let Some(page) = credits.current() {
                if page.show_photo {
                    if let Some(bmp) = self.files.get(&CREDITS_PHOTO_FILE.to_ascii_lowercase()).and_then(|v| v.first()) {
                        if let Some(mut pic) = Pic::of(bmp) {
                            pic.see_through = false;
                            pieces.push(Piece::Picture {
                                rect: CREDITS_PHOTO_RECT,
                                pic,
                                stretch: true,
                            });
                        }
                    }
                }
                let small_font = self.fonts.get(&CREDITS_SMALL_FONT.to_ascii_lowercase());
                let large_font = self.fonts.get(&CREDITS_LARGE_FONT.to_ascii_lowercase());
                for (rect, line) in page.line_rects() {
                    if line.text.trim().is_empty() {
                        continue;
                    }
                    let font = if line.small { small_font } else { large_font };
                    if let Some(font) = font {
                        pieces.push(Piece::Text {
                            rect,
                            font,
                            justification: 0xC,
                            text: line.text.clone(),
                        });
                    }
                }
            }
        }

        pieces.reverse();
        gui_draw::draw([0, 0], [640, 480], 1.0, &pieces, &self.palette)
    }

    fn render_graphics(&self) -> gui_draw::Canvas {
        let mut pieces: Vec<Piece> = Vec::new();
        self.push_submenu_bg(&mut pieces, SUBMENU_BG_PICTURE);

        if let Some(font) = self.fonts.get(&TITLE_FONT.to_ascii_lowercase()) {
            pieces.push(Piece::Text {
                rect: GRA_TITLE_RECT,
                font,
                justification: 0xC,
                text: self.submenu_texts.graphics_title.clone(),
            });
        }

        let font20 = self.fonts.get(&OPT_FONT.to_ascii_lowercase());
        let font27 = self.fonts.get(&ITEM_FONT.to_ascii_lowercase());
        if let Some(font) = font20 {
            pieces.push(Piece::Text {
                rect: GRA_MODE_LABEL_RECT,
                font,
                justification: 0xA,
                text: self.submenu_texts.gra_mode_label.clone(),
            });
            pieces.push(Piece::Text {
                rect: GRA_RES_LABEL_RECT,
                font,
                justification: 0xA,
                text: self.submenu_texts.gra_res_label.clone(),
            });
        }

        for row in 0..GRA_ROW_COUNT {
            let font = if row == 11 { font27 } else { font20 };
            let Some(font) = font else { continue };
            let just = if row <= 4 {
                0x9
            } else if row == 11 {
                0xC
            } else {
                0xA
            };
            let raw = &self.submenu_texts.gra_rows[row];
            let text = match row {
                1 => format!("> {raw} <"),
                3 if !self.options.mode_800 => format!("> {raw} <"),
                4 if self.options.mode_800 => format!("> {raw} <"),
                _ => raw.clone(),
            };
            let rect = GRA_LABEL_RECTS[row];
            if GraphicsState::is_row_enabled(row) {
                pieces.push(Piece::Text { rect, font, justification: just, text });
            } else {
                pieces.push(Piece::TextDithered { rect, font, justification: just, text });
            }
        }

        if let Some(font) = font20 {
            for &(row, rect) in &GRA_VALUE_RECTS {
                let on = match row {
                    5 => self.options.dynamic_lighting,
                    _ => GRA_SHADOW_BITS
                        .iter()
                        .find(|&&(r, _)| r == row)
                        .is_some_and(|&(_, bit)| self.options.shadow_mask & bit != 0),
                };
                let text = if on {
                    self.submenu_texts.on.clone()
                } else {
                    self.submenu_texts.off.clone()
                };
                pieces.push(Piece::Text { rect, font, justification: 0x9, text });
            }
        }

        self.push_slider(&mut pieces, GRA_BRIGHTNESS_SLIDER_RECT, self.options.brightness);
        self.push_submenu_cursors(&mut pieces, self.graphics_state.cursor_rects());

        pieces.reverse();
        gui_draw::draw([0, 0], [640, 480], 1.0, &pieces, &self.palette)
    }

    fn render_movekeys(&self) -> gui_draw::Canvas {
        let mut pieces: Vec<Piece> = Vec::new();
        self.push_submenu_bg(&mut pieces, SUBMENU_BG_PICTURE);

        if let Some(font) = self.fonts.get(&TITLE_FONT.to_ascii_lowercase()) {
            pieces.push(Piece::Text {
                rect: MOVE_TITLE_RECT,
                font,
                justification: 0xC,
                text: self.submenu_texts.movekeys_title.clone(),
            });
        }

        let font20 = self.fonts.get(&OPT_FONT.to_ascii_lowercase());
        let font27 = self.fonts.get(&ITEM_FONT.to_ascii_lowercase());
        if let Some(font) = font20 {
            for row in 0..MOVE_KEY_COUNT {
                pieces.push(Piece::Text {
                    rect: move_label_rect(row),
                    font,
                    justification: 0xA,
                    text: self.submenu_texts.move_rows[row].clone(),
                });
                let val_text = if self.movekeys_state.invalid_key_row == Some(row) {
                    self.submenu_texts.invalid_key.clone()
                } else if self.movekeys_state.capturing == Some(row) {
                    self.submenu_texts.press_desired_key.clone()
                } else if let Some(b) = self.movekeys_state.file.bindings.get(row) {
                    self.submenu_texts.resolve_tag(&b.name_tag).to_string()
                } else {
                    String::new()
                };
                pieces.push(Piece::Text {
                    rect: move_value_rect(row),
                    font,
                    justification: 0x9,
                    text: val_text,
                });
            }
        }
        if let Some(font) = font27 {
            let locked = self.movekeys_state.capturing.is_some();
            let rect = move_label_rect(MOVE_KEY_COUNT);
            let text = self.submenu_texts.back_to_options.clone();
            if locked {
                pieces.push(Piece::TextDithered { rect, font, justification: 0xC, text });
            } else {
                pieces.push(Piece::Text { rect, font, justification: 0xC, text });
            }
        }

        self.push_submenu_cursors(&mut pieces, self.movekeys_state.cursor_rects());

        pieces.reverse();
        gui_draw::draw([0, 0], [640, 480], 1.0, &pieces, &self.palette)
    }
}

/// Spawns the persistent Main Menu UI nodes at startup.
pub fn setup_menu(mut commands: Commands, mut images: ResMut<Assets<Image>>) {
    let image = images.add(Image::new_fill(
        Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
        TextureDimension::D2,
        &[0, 0, 0, 255],
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    ));
    let root = commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                ..default()
            },
            BackgroundColor(Color::BLACK),
            Visibility::Hidden,
            ZIndex(100),
            Persistent,
            MenuRoot,
        ))
        .id();
    commands.spawn((
        Node {
            height: Val::Percent(100.0),
            max_width: Val::Percent(100.0),
            aspect_ratio: Some(4.0 / 3.0),
            ..default()
        },
        ImageNode {
            image_mode: bevy::ui::widget::NodeImageMode::Stretch,
            ..ImageNode::new(image)
        },
        Persistent,
        MenuCanvasNode,
        ChildOf(root),
    ));
}

/// Manages the fallback 2D camera when the Main Menu is open and no 3D world
/// camera or loading screen camera exists yet.
pub fn menu_camera_step(
    mut commands: Commands,
    menu: Option<Res<MenuView>>,
    cam3d: Query<Entity, With<Camera3d>>,
    loading_cam: Query<Entity, (With<Camera2d>, With<LoadingScreen>)>,
    menu_cam: Query<Entity, With<MenuCamera>>,
) {
    let want_cam = menu.is_some_and(|m| m.state.open) && cam3d.is_empty() && loading_cam.is_empty();
    if want_cam && menu_cam.is_empty() {
        commands.spawn((Camera2d, bevy::ui_render::UiAntiAlias::Off, Persistent, MenuCamera));
    } else if !want_cam {
        for e in &menu_cam {
            commands.entity(e).despawn();
        }
    }
}

/// Updates the Main Menu pointer (`Cursgui.pba`) while the menu is open.
pub fn menu_pointer_draw(
    menu: Option<Res<MenuView>>,
    window: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    frame: Res<ScreenFrame>,
    mut shown: Local<Option<u32>>,
) {
    use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};
    let Some(menu) = menu else { return };
    if !menu.state.open {
        *shown = None;
        return;
    }
    let Ok((entity, _)) = window.single() else { return };
    let step = ((frame.0[3] / 480.0).round() as u32).max(1);
    if *shown == Some(step) {
        return;
    }
    let Some(frames) = menu.files.get(&POINTER_FILE.to_ascii_lowercase()) else {
        *shown = Some(step);
        return;
    };
    let Some(pic) = frames.first().and_then(Pic::of) else {
        *shown = Some(step);
        return;
    };
    let (pw, ph) = (pic.width * step, pic.height * step);
    let mut rgba = vec![0u8; (pw * ph * 4) as usize];
    for y in 0..ph {
        for x in 0..pw {
            let index = pic.pixels[((y / step) * pic.width + x / step) as usize];
            if pic.see_through && index == 0 {
                continue;
            }
            let [r, g, b] = menu.palette[index as usize];
            let at = ((y * pw + x) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    let image = Image::new(
        Extent3d { width: pw, height: ph, depth_or_array_layers: 1 },
        TextureDimension::D2,
        rgba,
        TextureFormat::Rgba8UnormSrgb,
        RenderAssetUsages::default(),
    );
    let handle = images.add(image);
    commands
        .entity(entity)
        .insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage { handle, hotspot: (0, 0), ..default() })));
    *shown = Some(step);
}

/// Every frame: handles Main Menu input, death timer, opening via Escape,
/// and redraws the menu canvas when needed.
#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn menu_step(
    menu: Option<ResMut<MenuView>>,
    mut sound: Option<ResMut<SoundSim>>,
    mut sim: Option<ResMut<crate::door_view::DoorSim>>,
    mut connor: Option<ResMut<crate::connor_view::ConnorSim>>,
    mut change: ResMut<WorldChange>,
    (time, mut keys, buttons, window, frame, mut key_at): (
        Res<Time<Real>>,
        ResMut<ButtonInput<KeyCode>>,
        Res<ButtonInput<MouseButton>>,
        Query<&Window, With<bevy::window::PrimaryWindow>>,
        Res<ScreenFrame>,
        Option<ResMut<KeyAt>>,
    ),
    mut root_vis: Query<&mut Visibility, With<MenuRoot>>,
    canvas_node: Query<&ImageNode, With<MenuCanvasNode>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    shared: Res<Shared>,
    (mut gui, mut ground_light, mut ground_materials, mut light_label, mut shadow_world, mut game_cam): (
        Option<ResMut<crate::gui_view::Gui>>,
        Option<ResMut<crate::light_view::GroundLight>>,
        ResMut<Assets<crate::ground_haze::GroundMaterial>>,
        Query<&mut Text, With<crate::light_view::LightLabel>>,
        Option<ResMut<crate::shadow_view::ShadowWorld>>,
        Option<ResMut<crate::game_camera::GameCam>>,
    ),
) {
    let Some(mut menu) = menu else { return };
    let dt = time.delta_secs_f64();
    menu.now_ms += dt * 1000.0;
    menu.total_frames += 1;

    // 1. Check Connor's death -> 2.0 s handsOff timer -> Main Menu (`main-menu.md` rule 25).
    let connor_dead = connor.as_ref().is_some_and(|c| c.walker.mode == kq8_core::connor::Mode::Dead);
    if !connor_dead {
        menu.death_timer = None;
        menu.death_handled = false;
    } else if !menu.state.open && !menu.death_handled {
        if menu.death_timer.is_none() {
            if let Some(s) = sim.as_mut() {
                s.active.world.connor_hands_off = true;
            }
            menu.note("Connor entered Mode::Dead: waiting 2.0 s in handsOff before opening main menu (main-menu.md rule 25)");
            menu.death_timer = Some(main_menu::DEATH_HANDS_OFF_SECONDS);
        }
        let rem = menu.death_timer.as_mut().unwrap();
        *rem -= dt;
        if *rem <= 0.0 {
            menu.death_timer = None;
            menu.death_handled = true;
            let world_name = change.current.clone();
            menu.open_on_death(&world_name, sound.as_deref_mut(), &shared);
        }
    }

    // 2. If the menu is closed, check for Escape or `--key menu:open` during play (`main-menu.md` rule 7).
    if !menu.state.open {
        for mut v in &mut root_vis {
            if *v != Visibility::Hidden {
                *v = Visibility::Hidden;
            }
        }
        let playing_now = change.is_playing();
        if !playing_now {
            return;
        }
        let portal_or_help = sim.as_ref().is_some_and(|s| s.active.world.portal_asking() || s.active.world.help_open());
        let hands_off = connor.as_ref().is_some_and(|c| c.hands_off)
            || sim.as_ref().is_some_and(|s| s.active.world.connor_hands_off);
        let updates = sim.as_ref().map_or(menu.total_frames, |s| s.active.world.updates);

        let mut open_how: Option<String> = None;
        if let Some(ka) = key_at.as_mut() {
            if let Some(pos) = ka.0.iter().position(|(name, n)| {
                let lower = name.to_ascii_lowercase();
                (lower == "menu:open" || lower == "menu:escape" || lower == "escape") && updates >= *n
            }) {
                let (name, _) = ka.0.remove(pos);
                open_how = Some(format!("test --key {name}"));
            }
        }
        if open_how.is_none() && !portal_or_help && keys.just_pressed(KeyCode::Escape) {
            keys.clear_just_pressed(KeyCode::Escape);
            open_how = Some("Escape key".into());
        }
        if let Some(how) = open_how {
            if hands_off {
                menu.note(format!("{how} ignored during handsOff (main-menu.md rule 7)"));
            } else {
                let world_name = change.current.clone();
                menu.open_from_gameplay(&world_name, &how, sound.as_deref_mut(), &shared);
            }
        }
        if !menu.state.open {
            return;
        }
    }

    // 3. The menu is open: show `MenuRoot`, tick the 0.2 s input block, and process input.
    for mut v in &mut root_vis {
        if *v != Visibility::Inherited {
            *v = Visibility::Inherited;
        }
    }
    menu.state.tick(dt.max(1.0 / 60.0));

    let [fl, ft, fw, fh] = frame.0;
    let to_layout = |p: Vec2| -> [f32; 2] {
        [
            (p.x - fl) * (640.0 / fw.max(1.0)),
            (p.y - ft) * (480.0 / fh.max(1.0)),
        ]
    };
    let cursor_layout = window.single().ok().and_then(|w| w.cursor_position()).map(to_layout);

    let mut actions = std::mem::take(&mut menu.pending_actions);
    let mut sub_actions: Vec<SubmenuAction> = Vec::new();

    if menu.screen == MenuScreen::Credits {
        let mut finish_credits = false;
        if let Some(credits) = menu.credits_state.as_mut() {
            if credits.tick(dt * 1000.0) {
                finish_credits = true;
            }
        }
        if finish_credits {
            sub_actions.push(SubmenuAction::Note("credits finished final page timer (submenus.md rule 25)".into()));
            sub_actions.push(SubmenuAction::ExitCreditsToMain);
        }
    }

    if !menu.state.input_blocked() {
        // Test keys (`--key menu:...`, processed one per frame once the 0.2 s block expires).
        let mut test_cmd: Option<String> = None;
        if let Some(ka) = key_at.as_mut() {
            let updates = sim.as_ref().map_or(menu.total_frames, |s| s.active.world.updates);
            if let Some(pos) = ka.0.iter().position(|(name, n)| {
                name.to_ascii_lowercase().starts_with("menu:") && (menu.total_frames >= *n || updates >= *n)
            }) {
                test_cmd = Some(ka.0.remove(pos).0);
            }
        }
        if let Some(cmd) = test_cmd {
            let sub = cmd[5..].to_ascii_lowercase();
            menu.note(format!("test --key {cmd}"));
            match menu.screen {
                MenuScreen::MainMenu => {
                    let texts = menu.texts.clone();
                    match sub.as_str() {
                        "up" => actions.extend(menu.state.move_selection(-1)),
                        "down" => actions.extend(menu.state.move_selection(1)),
                        "left" => actions.extend(menu.state.move_popup_highlight(-1)),
                        "right" => actions.extend(menu.state.move_popup_highlight(1)),
                        "escape" | "close" => actions.extend(menu.state.press_escape()),
                        "enter_down" => actions.extend(menu.state.press_enter(&texts)),
                        "enter_up" => actions.extend(menu.state.release_enter(&texts)),
                        "enter" => {
                            if menu.state.popup.is_some() {
                                actions.extend(menu.state.press_enter(&texts));
                                actions.extend(menu.state.release_enter(&texts));
                            } else {
                                actions.extend(menu.state.press_enter(&texts));
                            }
                        }
                        click if click.starts_with("click:") => {
                            let target = &click[6..];
                            let pos = match target {
                                "new" => Some(MenuItem::New.rect()),
                                "load" => Some(MenuItem::Load.rect()),
                                "save" => Some(MenuItem::Save.rect()),
                                "options" => Some(MenuItem::Options.rect()),
                                "intro" | "replay" => Some(MenuItem::ReplayIntro.rect()),
                                "quit" => Some(MenuItem::Quit.rect()),
                                "back" => Some(MenuItem::BackToGame.rect()),
                                "btn0" | "yes" | "easy" => menu.state.popup.as_ref().and_then(|p| p.button_rects().first().copied()),
                                "btn1" | "no" | "normal" => menu.state.popup.as_ref().and_then(|p| p.button_rects().get(1).copied()),
                                "btn2" | "hard" => menu.state.popup.as_ref().and_then(|p| p.button_rects().get(2).copied()),
                                _ => None,
                            };
                            if let Some(r) = pos {
                                let mid = [0.5 * (r[0] + r[2]) as f32, 0.5 * (r[1] + r[3]) as f32];
                                actions.extend(menu.state.click_at(mid, &texts));
                            }
                        }
                        _ => {}
                    }
                }
                MenuScreen::Options => {
                    let mv = &mut *menu;
                    match sub.as_str() {
                        "up" | "opt:up" => sub_actions.extend(mv.options_state.move_selection(-1, &mv.options)),
                        "down" | "opt:down" => sub_actions.extend(mv.options_state.move_selection(1, &mv.options)),
                        "left" | "opt:left" => sub_actions.extend(mv.options_state.adjust_left_right(-1, &mut mv.options)),
                        "right" | "opt:right" => sub_actions.extend(mv.options_state.adjust_left_right(1, &mut mv.options)),
                        "enter" | "opt:enter" => sub_actions.extend(mv.options_state.activate(&mut mv.options)),
                        "escape" | "close" | "opt:escape" => sub_actions.extend(mv.options_state.press_escape()),
                        click if click.starts_with("click:") => {
                            let target = &click[6..];
                            let row_idx = match target {
                                "graphics" | "row0" => Some(0),
                                "movekeys" | "keys" | "row1" => Some(1),
                                "music" | "row2" => Some(2),
                                "sfx" | "row3" => Some(3),
                                "speech" | "row4" => Some(4),
                                "subtitles" | "row5" => Some(5),
                                "help" | "row6" => Some(6),
                                "credits" | "row7" => Some(7),
                                "back" | "row8" => Some(8),
                                _ => None,
                            };
                            if let Some(row) = row_idx {
                                let r = OPT_ROW_RECTS[row];
                                let mid = [0.5 * (r[0] + r[2]) as f32, 0.5 * (r[1] + r[3]) as f32];
                                let (acts, _) = mv.options_state.click_at(mid, &mut mv.options);
                                sub_actions.extend(acts);
                            }
                        }
                        _ => {}
                    }
                }
                MenuScreen::Graphics => {
                    let mv = &mut *menu;
                    match sub.as_str() {
                        "up" | "gra:up" => sub_actions.extend(mv.graphics_state.move_selection(-1)),
                        "down" | "gra:down" => sub_actions.extend(mv.graphics_state.move_selection(1)),
                        "left" | "gra:left" => sub_actions.extend(mv.graphics_state.adjust_left_right(-1, &mut mv.options)),
                        "right" | "gra:right" => sub_actions.extend(mv.graphics_state.adjust_left_right(1, &mut mv.options)),
                        "enter" | "gra:enter" => sub_actions.extend(mv.graphics_state.activate(&mut mv.options)),
                        "escape" | "close" | "gra:escape" => sub_actions.extend(mv.graphics_state.press_escape()),
                        _ => {}
                    }
                }
                MenuScreen::MoveKeys => {
                    let mv = &mut *menu;
                    match sub.as_str() {
                        "up" | "mk:up" => sub_actions.extend(mv.movekeys_state.move_selection(-1)),
                        "down" | "mk:down" => sub_actions.extend(mv.movekeys_state.move_selection(1)),
                        "enter" | "mk:enter" => sub_actions.extend(mv.movekeys_state.activate()),
                        "escape" | "close" | "mk:escape" => sub_actions.extend(mv.movekeys_state.press_escape()),
                        vk if vk.starts_with("vk:") => {
                            if let Ok(code) = vk[3..].parse::<u32>() {
                                sub_actions.extend(mv.movekeys_state.press_vk(code));
                            }
                        }
                        _ => {}
                    }
                }
                MenuScreen::Credits => {
                    if matches!(sub.as_str(), "escape" | "close" | "credits:escape") {
                        sub_actions.push(SubmenuAction::ExitCreditsToMain);
                    }
                }
                _ => {}
            }
        }

        // Physical keyboard and mouse input.
        match menu.screen {
            MenuScreen::MainMenu => {
                let texts = menu.texts.clone();
                if keys.just_pressed(KeyCode::ArrowUp) {
                    actions.extend(menu.state.move_selection(-1));
                }
                if keys.just_pressed(KeyCode::ArrowDown) {
                    actions.extend(menu.state.move_selection(1));
                }
                if keys.just_pressed(KeyCode::ArrowLeft) {
                    actions.extend(menu.state.move_popup_highlight(-1));
                }
                if keys.just_pressed(KeyCode::ArrowRight) {
                    actions.extend(menu.state.move_popup_highlight(1));
                }
                let enter = [KeyCode::Enter, KeyCode::NumpadEnter];
                if keys.any_just_pressed(enter) {
                    actions.extend(menu.state.press_enter(&texts));
                }
                if keys.any_just_released(enter) {
                    actions.extend(menu.state.release_enter(&texts));
                }
                if keys.just_pressed(KeyCode::Escape) {
                    keys.clear_just_pressed(KeyCode::Escape);
                    actions.extend(menu.state.press_escape());
                }
                if buttons.just_pressed(MouseButton::Left) {
                    if let Some(pos) = cursor_layout {
                        actions.extend(menu.state.click_at(pos, &texts));
                    }
                }
            }
            MenuScreen::Options => {
                let mv = &mut *menu;
                if keys.just_pressed(KeyCode::ArrowUp) {
                    sub_actions.extend(mv.options_state.move_selection(-1, &mv.options));
                }
                if keys.just_pressed(KeyCode::ArrowDown) {
                    sub_actions.extend(mv.options_state.move_selection(1, &mv.options));
                }
                if keys.just_pressed(KeyCode::ArrowLeft) {
                    sub_actions.extend(mv.options_state.adjust_left_right(-1, &mut mv.options));
                }
                if keys.just_pressed(KeyCode::ArrowRight) {
                    sub_actions.extend(mv.options_state.adjust_left_right(1, &mut mv.options));
                }
                if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
                    sub_actions.extend(mv.options_state.activate(&mut mv.options));
                }
                if keys.just_pressed(KeyCode::Escape) {
                    keys.clear_just_pressed(KeyCode::Escape);
                    sub_actions.extend(mv.options_state.press_escape());
                }
                if buttons.just_pressed(MouseButton::Left) {
                    if let Some(pos) = cursor_layout {
                        let (acts, drag) = mv.options_state.click_at(pos, &mut mv.options);
                        mv.slider_drag = drag;
                        sub_actions.extend(acts);
                    }
                } else if buttons.pressed(MouseButton::Left) {
                    if let (Some(row), Some(pos)) = (mv.slider_drag, cursor_layout) {
                        sub_actions.extend(mv.options_state.set_slider_from_x(row, pos[0], &mut mv.options));
                    }
                } else {
                    mv.slider_drag = None;
                }
                if buttons.just_released(MouseButton::Left) {
                    mv.slider_drag = None;
                }
            }
            MenuScreen::Graphics => {
                let mv = &mut *menu;
                if keys.just_pressed(KeyCode::ArrowUp) {
                    sub_actions.extend(mv.graphics_state.move_selection(-1));
                }
                if keys.just_pressed(KeyCode::ArrowDown) {
                    sub_actions.extend(mv.graphics_state.move_selection(1));
                }
                if keys.just_pressed(KeyCode::ArrowLeft) {
                    sub_actions.extend(mv.graphics_state.adjust_left_right(-1, &mut mv.options));
                }
                if keys.just_pressed(KeyCode::ArrowRight) {
                    sub_actions.extend(mv.graphics_state.adjust_left_right(1, &mut mv.options));
                }
                if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
                    sub_actions.extend(mv.graphics_state.activate(&mut mv.options));
                }
                if keys.just_pressed(KeyCode::Escape) {
                    keys.clear_just_pressed(KeyCode::Escape);
                    sub_actions.extend(mv.graphics_state.press_escape());
                }
                if buttons.just_pressed(MouseButton::Left) {
                    if let Some(pos) = cursor_layout {
                        let (acts, drag) = mv.graphics_state.click_at(pos, &mut mv.options);
                        mv.gra_brightness_drag = drag;
                        sub_actions.extend(acts);
                    }
                } else if buttons.pressed(MouseButton::Left) && mv.gra_brightness_drag {
                    if let Some(pos) = cursor_layout {
                        sub_actions.extend(mv.graphics_state.set_brightness_from_x(pos[0], &mut mv.options));
                    }
                } else {
                    mv.gra_brightness_drag = false;
                }
                if buttons.just_released(MouseButton::Left) {
                    mv.gra_brightness_drag = false;
                }
            }
            MenuScreen::MoveKeys => {
                let mv = &mut *menu;
                if mv.movekeys_state.capturing.is_some() {
                    if let Some(vk) = just_pressed_vk(&keys) {
                        keys.clear_just_pressed(KeyCode::Escape);
                        sub_actions.extend(mv.movekeys_state.press_vk(vk));
                    }
                } else {
                    if keys.just_pressed(KeyCode::ArrowUp) {
                        sub_actions.extend(mv.movekeys_state.move_selection(-1));
                    }
                    if keys.just_pressed(KeyCode::ArrowDown) {
                        sub_actions.extend(mv.movekeys_state.move_selection(1));
                    }
                    if keys.any_just_pressed([KeyCode::Enter, KeyCode::NumpadEnter]) {
                        sub_actions.extend(mv.movekeys_state.activate());
                    }
                    if keys.just_pressed(KeyCode::Escape) {
                        keys.clear_just_pressed(KeyCode::Escape);
                        sub_actions.extend(mv.movekeys_state.press_escape());
                    }
                    if buttons.just_pressed(MouseButton::Left) {
                        if let Some(pos) = cursor_layout {
                            sub_actions.extend(mv.movekeys_state.click_at(pos));
                        }
                    }
                }
            }
            MenuScreen::Credits => {
                if keys.just_pressed(KeyCode::Escape) {
                    keys.clear_just_pressed(KeyCode::Escape);
                    sub_actions.push(SubmenuAction::ExitCreditsToMain);
                }
            }
            _ => {}
        }
    }

    // 4. Execute emitted MenuActions.
    for action in actions {
        match action {
            MenuAction::PlayMenuSound { file, why } => {
                menu.note(format!("sound: {file} ({why})"));
                if let Some(snd) = sound.as_mut() {
                    snd.play_menu_sound(file, why);
                }
            }
            MenuAction::StartTheme => {
                menu.note("theme: start theme.aud in group 4, full loudness, no fade-in (rule 8)");
                if let Some(snd) = sound.as_mut() {
                    snd.start_menu_theme();
                }
            }
            MenuAction::FadeTheme => {
                menu.note("theme: fade theme.aud 1.0 -> 0.0 at 1.0/s over 1 s (rule 9)");
                if let Some(snd) = sound.as_mut() {
                    snd.fade_menu_theme();
                }
            }
            MenuAction::ResumeGame => {
                if let Some(snd) = sound.as_mut() {
                    snd.set_game_paused(false);
                }
                menu.summary(&shared, "resumed gameplay (rule 9): theme fading out over 1 s, world clock & sounds resumed");
            }
            MenuAction::StartNewGame { combat_level_number, factor } => {
                menu.summary(
                    &shared,
                    format!("starting New game in Daventry: combat level {combat_level_number} ({factor}), kept worlds cleared, conStart.cs skipped, BeginNow queued (rules 10, 24)"),
                );
                change.start_new_game(combat_level_number);
            }
            MenuAction::OpenSubScreen(MenuItem::Options) => {
                if let Some(s) = sim.as_ref() {
                    menu.options.not_starting_help = s.active.world.help.not_starting_help;
                    if let Some(v) = s.active.scripts.vars.get("Options::Captions") {
                        menu.options.captions = !matches!(v.to_ascii_lowercase().as_str(), "0" | "false");
                    }
                }
                let opts_snapshot = menu.options.clone();
                menu.options_state.reset_on_open(&opts_snapshot);
                menu.slider_drag = None;
                menu.screen = MenuScreen::Options;
                menu.last_drawn_key = None;
                let msg = format!(
                    "opened Options screen (submenus.md rules 12-16): music={}%, sfx={}%, speech={}%, captions={}, notStartingHelp={}",
                    menu.options_state.music_slider,
                    menu.options_state.sfx_slider,
                    menu.options_state.speech_slider,
                    menu.options.captions,
                    menu.options.not_starting_help
                );
                menu.summary(&shared, msg);
            }
            MenuAction::OpenSubScreen(item) => {
                menu.summary(
                    &shared,
                    format!(
                        "{} screen chosen (rule 11): sub-screen not built yet; staying on main menu with theme.aud playing unchanged",
                        item.default_label()
                    ),
                );
            }
            MenuAction::ReplayIntro => {
                menu.summary(
                    &shared,
                    "Replay Intro chosen (rule 12): W32opn_1.avi playback not built yet; returning to menu with 0.2 s input block",
                );
                let more = menu.state.finish_replay_intro();
                for a in more {
                    if let MenuAction::Note(n) = a {
                        menu.note(n);
                    }
                }
            }
            MenuAction::Quit => {
                menu.summary(&shared, "Quit confirmed (rule 13): exiting program");
                commands.write_message(AppExit::Success);
            }
            MenuAction::DeleteCorruptSave { file } => {
                // AGENTS.md Hard Rule 4 & main-menu.md "Do not reproduce" 3: never write/delete in MASK/.
                menu.summary(
                    &shared,
                    format!("corrupt save {file}: user chose Yes; file in read-only game install MASK/ left untouched on disk and skipped for this session (AGENTS.md rule 4)"),
                );
            }
            MenuAction::Note(line) => {
                menu.note(line);
            }
        }
    }

    // 4b. Execute emitted SubmenuActions (Options & Credits).
    for action in sub_actions {
        match action {
            SubmenuAction::PlaySound(file) => {
                menu.note(format!("sound: {file} (submenu)"));
                if let Some(snd) = sound.as_mut() {
                    snd.play_menu_sound(file, format!("submenu {:?}", menu.screen));
                }
            }
            SubmenuAction::UpdateVolumes { music, sfx, speech } => {
                if let Some(snd) = sound.as_mut() {
                    snd.set_group_volumes(music, sfx, speech);
                }
            }
            SubmenuAction::ApplyGraphics { dynamic_lighting, shadow_mask, brightness, mode_800 } => {
                if let Some(sw) = shadow_world.as_deref_mut() {
                    sw.mask = shadow_mask;
                }
                if let Some(gl) = ground_light.as_deref_mut() {
                    crate::light_view::set_dynamic_lighting(
                        dynamic_lighting,
                        "Graphics menu",
                        &shared,
                        gl,
                        &mut images,
                        &mut ground_materials,
                        &mut light_label,
                    );
                }
                menu.note(format!(
                    "applied Graphics: dynamic_lighting={dynamic_lighting}, shadow_mask={shadow_mask} ({shadow_mask:#x}), brightness={brightness} ({:.2}), mode_800={mode_800}",
                    brightness_factor(brightness)
                ));
            }
            SubmenuAction::SwitchScreen(MenuScreen::Options) => {
                menu.gra_brightness_drag = false;
                menu.screen = MenuScreen::Options;
                menu.last_drawn_key = None;
                menu.summary(&shared, "returned to Options screen");
            }
            SubmenuAction::SwitchScreen(MenuScreen::Graphics) => {
                if let Some(gl) = ground_light.as_ref() {
                    menu.options.dynamic_lighting = gl.dynamic;
                }
                if let Some(sw) = shadow_world.as_ref() {
                    menu.options.shadow_mask = sw.mask;
                }
                menu.graphics_state.reset_on_open();
                menu.gra_brightness_drag = false;
                menu.screen = MenuScreen::Graphics;
                menu.last_drawn_key = None;
                let msg = format!(
                    "opened Graphics screen (submenus.md rules 17-19): dynamic={}, shadow_mask={}, brightness={} ({:.2})",
                    menu.options.dynamic_lighting,
                    menu.options.shadow_mask,
                    menu.options.brightness,
                    brightness_factor(menu.options.brightness)
                );
                menu.summary(&shared, msg);
            }
            SubmenuAction::SwitchScreen(MenuScreen::MoveKeys) => {
                menu.movekeys_state.reset_on_open();
                menu.screen = MenuScreen::MoveKeys;
                menu.last_drawn_key = None;
                menu.summary(&shared, "opened Remap Move Keys screen (submenus.md rules 20-22)");
            }
            SubmenuAction::SwitchScreen(MenuScreen::Credits) => {
                let score = sim
                    .as_ref()
                    .and_then(|s| s.active.world.connor_stats.map(|c| c.stats.0[4]))
                    .unwrap_or(0);
                let cs = CreditsState::from_msg(menu.msg400.as_ref(), score);
                let count = cs.pages.len();
                menu.credits_state = Some(cs);
                menu.screen = MenuScreen::Credits;
                menu.last_drawn_key = None;
                menu.summary(
                    &shared,
                    format!("opened Credits screen ({count} pages from 400.MSG, score={score}; submenus.md rules 23-25, Q206)"),
                );
            }
            SubmenuAction::SwitchScreen(target) => {
                menu.summary(
                    &shared,
                    format!("{target:?} screen selected: coming in next step"),
                );
            }
            SubmenuAction::SaveMoveKeysAndReturnToOptions => {
                let text = menu.movekeys_state.file.format();
                match write_user_config("movekeys.kq", &text) {
                    Ok(path) => menu.summary(
                        &shared,
                        format!("saved {} and returned to Options", path.display()),
                    ),
                    Err(e) => menu.summary(&shared, format!("failed to write movekeys.kq: {e}")),
                }
                if let Some(g) = gui.as_deref_mut() {
                    g.reload_movekeys();
                }
                if let Some(c) = connor.as_deref_mut() {
                    c.controls.apply_movekeys(text.as_bytes());
                }
                if let Some(cam) = game_cam.as_deref_mut() {
                    cam.apply_movekeys(text.as_bytes());
                }
                menu.screen = MenuScreen::Options;
                menu.last_drawn_key = None;
            }
            SubmenuAction::SaveOptionsAndReturnToMain => {
                let text = menu.options.format();
                match write_user_config("Options.cs", &text) {
                    Ok(path) => {
                        let msg = format!(
                            "saved {} (music={:.2}, sfx={:.2}, speech={:.2}, captions={}, notStartingHelp={}) and returned to Main Menu (theme.aud unchanged)",
                            path.display(),
                            menu.options.music_vol,
                            menu.options.sfx_vol,
                            menu.options.speech_vol,
                            menu.options.captions,
                            menu.options.not_starting_help
                        );
                        menu.summary(&shared, msg);
                    }
                    Err(e) => menu.summary(&shared, format!("failed to write Options.cs: {e}")),
                }
                if let Some(s) = sim.as_deref_mut() {
                    s.active.world.help.set_not_starting_help(menu.options.not_starting_help);
                    s.active.scripts.vars.set(
                        "Options::Captions",
                        if menu.options.captions { "true".into() } else { "false".into() },
                    );
                }
                menu.slider_drag = None;
                menu.screen = MenuScreen::MainMenu;
                menu.last_drawn_key = None;
            }
            SubmenuAction::ExitCreditsToMain => {
                menu.credits_state = None;
                menu.screen = MenuScreen::MainMenu;
                menu.last_drawn_key = None;
                if let Some(snd) = sound.as_mut() {
                    snd.fade_menu_theme();
                    snd.start_menu_theme();
                }
                menu.summary(
                    &shared,
                    "exited Credits -> Main Menu (submenus.md rule 25: theme.aud faded out and restarted)",
                );
            }
            SubmenuAction::Note(line) => {
                menu.note(line);
            }
        }
    }

    // 5. Redraw the 640 x 480 menu image if state or cursor animation frame changed.
    if !menu.state.open {
        for mut v in &mut root_vis {
            *v = Visibility::Hidden;
        }
        return;
    }
    let key = menu.draw_key();
    if menu.last_drawn_key.as_ref() != Some(&key) {
        let canvas = menu.render_canvas();
        let mut picture = Image::new(
            Extent3d { width: canvas.width, height: canvas.height, depth_or_array_layers: 1 },
            TextureDimension::D2,
            canvas.rgba,
            TextureFormat::Rgba8UnormSrgb,
            RenderAssetUsages::default(),
        );
        picture.sampler = ImageSampler::nearest();
        if let Ok(node) = canvas_node.single() {
            if let Some(mut slot) = images.get_mut(&node.image) {
                *slot = picture;
            }
        }
        menu.last_drawn_key = Some(key);
    }
}
