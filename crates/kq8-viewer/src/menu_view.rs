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

#[derive(Component)]
pub struct MenuRoot;

#[derive(Component)]
pub struct MenuCanvasNode;

#[derive(Component)]
pub struct MenuCamera;

/// In-memory 160 × 120 thumbnail saved when Escape / the mask button opens
/// the menu from gameplay (`main-menu.md` rules 7.3, 22, "Do not reproduce" 3).
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
    pub texts: MenuTexts,
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
    selected: MenuItem,
    enabled_mask: u8,
    cursor_frame: usize,
    popup: Option<MenuPopup>,
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

        let msg = find_bytes(game, &["GAME/ENGLISH/500.MSG"]).and_then(|(_, b)| MessageFile::parse(500, &b).ok());
        let starttxt = find_bytes(game, &["starttxt.txt"]).map(|(_, b)| b);
        let texts = MenuTexts::from_msg_and_title(msg.as_ref(), starttxt.as_deref());

        let mut files = HashMap::new();
        for name in [
            BG_PICTURE,
            LINE_PICTURE,
            CURSOR_PICTURE,
            POPUP_BG_PICTURE,
            POPUP_BTN_PICTURE,
            POPUP_HI_BTN_PICTURE,
            POINTER_FILE,
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
        for name in [ITEM_FONT, TITLE_FONT, POPUP_FONT] {
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
            texts,
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

    fn draw_key(&self) -> MenuDrawKey {
        let mut mask = 0u8;
        for item in MenuItem::ALL {
            if self.state.is_enabled(item) {
                mask |= 1 << item.index();
            }
        }
        MenuDrawKey {
            selected: self.state.selected,
            enabled_mask: mask,
            cursor_frame: self.cursor_frame(),
            popup: self.state.popup.clone(),
        }
    }

    /// Renders the 640 × 480 menu screen into a [`gui_draw::Canvas`].
    pub fn render_canvas(&self) -> gui_draw::Canvas {
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
#[allow(clippy::too_many_arguments)]
pub fn menu_step(
    menu: Option<ResMut<MenuView>>,
    mut sound: Option<ResMut<SoundSim>>,
    mut sim: Option<ResMut<crate::door_view::DoorSim>>,
    connor: Option<Res<crate::connor_view::ConnorSim>>,
    mut change: ResMut<WorldChange>,
    time: Res<Time<Real>>,
    mut keys: ResMut<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    mut window: Query<&mut Window, With<bevy::window::PrimaryWindow>>,
    frame: Res<ScreenFrame>,
    mut key_at: Option<ResMut<KeyAt>>,
    mut root_vis: Query<&mut Visibility, With<MenuRoot>>,
    canvas_node: Query<&ImageNode, With<MenuCanvasNode>>,
    mut images: ResMut<Assets<Image>>,
    mut commands: Commands,
    shared: Res<Shared>,
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
        let rem = menu.death_timer.get_or_insert_with(|| {
            if let Some(s) = sim.as_mut() {
                s.active.world.connor_hands_off = true;
            }
            menu.note("Connor entered Mode::Dead: waiting 2.0 s in handsOff before opening main menu (main-menu.md rule 25)");
            main_menu::DEATH_HANDS_OFF_SECONDS
        });
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
    let to_window = |p: [f32; 2]| -> Vec2 {
        Vec2::new(
            fl + p[0] * (fw / 640.0),
            ft + p[1] * (fh / 480.0),
        )
    };
    let cursor_layout = window.single().ok().and_then(|w| w.cursor_position()).map(to_layout);

    let mut actions = std::mem::take(&mut menu.pending_actions);

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
            let texts = menu.texts.clone();
            match sub.as_str() {
                "up" => actions.extend(menu.state.move_selection(-1)),
                "down" => actions.extend(menu.state.move_selection(1)),
                "left" => actions.extend(menu.state.move_popup_highlight(-1)),
                "right" => actions.extend(menu.state.move_popup_highlight(1)),
                "escape" | "close" => actions.extend(menu.state.press_escape()),
                "enter_down" => actions.extend(menu.state.press_enter(cursor_layout.or(Some([320.0, 240.0])), &texts)),
                "enter_up" => actions.extend(menu.state.release_enter(&texts)),
                "enter" => {
                    if menu.state.popup.is_some() {
                        actions.extend(menu.state.press_enter(cursor_layout.or(Some([320.0, 240.0])), &texts));
                        actions.extend(menu.state.release_enter(&texts));
                    } else {
                        actions.extend(menu.state.press_enter(cursor_layout, &texts));
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

        // Physical keyboard and mouse input.
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
            actions.extend(menu.state.press_enter(cursor_layout, &texts));
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
            MenuAction::MovePointer(target) => {
                let win_pos = to_window(target);
                menu.note(format!("pointer moved onto popup button at layout ({:.1}, {:.1}) -> window ({:.1}, {:.1}) (rule 16)", target[0], target[1], win_pos.x, win_pos.y));
                if let Ok(mut w) = window.single_mut() {
                    w.set_cursor_position(Some(win_pos));
                }
            }
            MenuAction::RestorePointer(orig) => {
                let win_pos = to_window(orig);
                menu.note(format!("pointer restored to layout ({:.1}, {:.1}) -> window ({:.1}, {:.1}) (rule 16)", orig[0], orig[1], win_pos.x, win_pos.y));
                if let Ok(mut w) = window.single_mut() {
                    w.set_cursor_position(Some(win_pos));
                }
            }
            MenuAction::Note(line) => {
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
            if let Some(slot) = images.get_mut(&node.image) {
                *slot = picture;
            }
        }
        menu.last_drawn_key = Some(key);
    }
}
