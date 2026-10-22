//! The game-play screen's panels (M4 step 4c, docs/DESIGN.md; rules:
//! docs/behaviour/interface.md and inventory.md).
//!
//! 4c-2, the always-visible bottom bar: bottom left (armour, hand and
//! ranged weapon slots with their numbers), the eight slots (empty until
//! 4c-3), the meters (health, experience, level) and bottom right (rope,
//! rocks, map icon), each drawn from its block of `Gplayscr.gui` with the
//! world's `8gui` pictures and fonts in the world's palette
//! (`kq8_core::gui_draw`), as one picture per panel, moved by
//! `kq8_core::panels` (sliding, Space, handsOff, the meters' pop-up, the
//! script commands). The status icons show the spinning mask during
//! handsOff. Every change goes to `kq8-artifacts/logs/gui.log`.
//!
//! The screen is 800 × 600 (`set800mode 1`) (STAND-IN, Q108), drawn on
//! the largest 4:3 rectangle centred in the window ([`ScreenFrame`], 4c-6,
//! the user's choice); the panels' timers count real time.

use std::collections::HashMap;

use bevy::asset::RenderAssetUsages;
use bevy::image::ImageSampler;
use bevy::prelude::*;
use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
use kq8_core::gui_draw::{self, Pic, Piece};
use kq8_core::meters::{self, Meters, Slot};
use kq8_core::panels::{Panel, Panels};
use kq8_data::font::Font;
use kq8_data::gui::{Control, Kind, Tags};
use kq8_data::palette::Rgb;
use kq8_data::{Bitmap, GameDir};

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::log::Log;
use crate::Shared;

/// STAND-IN (Q108 d): the panels' little animations play at this many
/// frames per second.
const ANIMATION_FPS: f64 = 15.0;

/// The play pointer (`Gplayscr.kq` `cursor=`, `interface.md` rule 1).
const PLAY_POINTER: &str = "IDBMA_CURS04";
/// The sword pointer while a hand weapon is ready (`fighting.md` rule 39,
/// manual: "the pointer is a sword"): `curs03.pba`, the only sword among
/// the attack cursors of `interface.md` rule 12 (seen in its picture; M5
/// 5c fix, Q138).
const SWORD_POINTER: &str = "IDBMA_CURS03";
/// The ranged weapon's pointer (`ranged-weapons.md` rule 11; the name from
/// the pointers' order, open question 4).
const BOW_POINTER: &str = "IDBMA_CURS05";

/// Rule 44: a blink changes the picture every 0.25 s, 5 times.
const BLINK_MS: f64 = 250.0;
const BLINKS: u32 = 5;

/// What Connor carries, as the bars show it (4c-3): item names, counts
/// and picture files.
#[derive(Debug, Clone, Default, PartialEq)]
struct InvView {
    /// The quest items in the order they were added (`inventory.md`
    /// rule 5): name, picture file.
    quest: Vec<(String, Option<String>)>,
    /// The mask piece (rule 41).
    mask: Option<(String, Option<String>)>,
    /// The coins: name, count, picture file (rule 43).
    gold: Option<(String, i64, Option<String>)>,
    /// The eight slots (rule 38): name, count, picture file.
    slots: [Option<(String, i64, Option<String>)>; 8],
    /// The rope's item (rule 45).
    rope: Option<String>,
    /// The rocks: name, count, picture file (rule 45).
    rocks: Option<(String, i64, Option<String>)>,
    /// Every carried item's count and category (for blinking and news).
    counts: std::collections::BTreeMap<String, (i64, String)>,
}

/// A click on the bars or a slot key, waiting for the next update.
#[derive(Debug, Clone, PartialEq)]
pub enum BarAction {
    /// A left (`true`) or right click on a control showing item `item`.
    Item { item: String, left: bool, how: String },
    /// The quest strip's arrows (`inventory.md` rule 6).
    Scroll { by: i32, how: String },
    /// Keys 1-8 (`interface.md` rule 40), slot 0-7.
    Key { slot: usize, how: String },
}

/// The clicks and keys of the panels since the last update (4c-3).
#[derive(Resource, Default)]
pub struct BarActions(pub Vec<BarAction>);

/// What is drawn as its own picture: a sliding panel, or the status icons
/// (they don't slide; rule 7).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Drawn {
    Panel(Panel),
    Status,
}

impl Drawn {
    fn layout_id(self) -> &'static str {
        match self {
            Drawn::Panel(p) => p.layout_id(),
            Drawn::Status => "IDCTL_TIMER",
        }
    }
}

/// The pictures drawn (4c-2; the top bar since 4c-3; the map window since
/// M6 6d-3).
const DRAWN: [Drawn; 7] = [Drawn::Panel(Panel::Top), Drawn::Panel(Panel::BottomLeft), Drawn::Panel(Panel::Slots), Drawn::Panel(Panel::Meters), Drawn::Panel(Panel::BottomRight), Drawn::Panel(Panel::Map), Drawn::Status];

/// The map window's parts (`magic-map.md` rule 5).
const MAP_AREA: &str = "IDCTL_MAPWIN_MAP";
/// Rule 8: one line, or two.
const MAP_NAME_ONE: &str = "IDCTL_MAPWIN_NAME_TEXT1";
const MAP_NAME_TWO: [&str; 2] = ["IDCTL_MAPWIN_NAME_TEXT2", "IDCTL_MAPWIN_NAME_TEXT3"];
/// The page arrows and the "Go" button (unlit; rules 32, 38).
const MAP_BUTTONS: [&str; 3] = ["IDCTL_MAPWIN_LEFT", "IDCTL_MAPWIN_RIGHT", "IDCTL_MAPWIN_TELEPORT"];
/// The bottom-right map slot (`interface.md` rule 45).
const MINIMAP: &str = "IDCTL_INVBAR_MINIMAP";

/// The magic map as the panels show it (M6 6d-3), copied from the rules
/// each update (the picture only when it changed).
#[derive(Debug, Clone, Default)]
struct MapShown {
    /// `magic-map.md` rule 2.
    active: bool,
    version: Option<u64>,
    pixels: Vec<u8>,
    /// Rule 22 (layout pixels from the area's centre).
    view: [f64; 4],
    /// Rule 11: Connor's last map pixel; rule 17: the arrow frame.
    pixel: Option<(i32, i32)>,
    frame: usize,
    arrow_file: String,
    /// Rule 8: the name (`500.MSG` 0 0 38 + index), its lines.
    name: Vec<String>,
}

/// The armour, hand and ranged slot pictures, their numbers and their
/// flashes, by layout name (rules 33, 34, 36), in [`Slot::ALL`] order.
const SLOT_PICTURES: [&str; 3] = ["IDCTL_INVBAR_ARMOR", "IDCTL_INVBAR_SHORT", "IDCTL_INVBAR_LONG"];
const SLOT_NUMBERS: [&str; 3] = ["IDCTL_INVBAR_ARMORTEXT", "IDCTL_INVBAR_SHORTTEXT", "IDCTL_INVBAR_LONGTEXT"];
const SLOT_FLASHES: [&str; 3] = ["IDCTL_INVBAR_ARMOR_FX", "IDCTL_INVBAR_SHORT_FX", "IDCTL_INVBAR_LONG_FX"];
/// Their red numbers (`interface.md` rule 32, `potions.md` rule 37).
const SLOT_RED_NUMBERS: [&str; 3] = ["IDCTL_INVBAR_ARMOR_ALTTXT", "IDCTL_INVBAR_SHORT_ALTTXT", "IDCTL_INVBAR_LONG_ALTTXT"];
/// The level-up flame over the level number (`interface.md` rule 26).
const LEVEL_FLAME: &str = "IDCTL_METERS_LEVELUP";

/// The spinning mask's speed (`potions.md` rule 35: an icon with several
/// frames and no duration plays at 10 frames a second).
const MASK_FPS: f64 = 10.0;

/// The status icons' picture (M5 5g-3): the layout's 336 wide, and tall
/// enough for the spinning mask's own 57 rows (4c-8); every icon stands
/// on its bottom.
const STATUS_SIZE: [i32; 2] = [336, 57];

/// What a status icon place shows (M5 5g-3, `potions.md` rule 35).
#[derive(Debug, Clone, PartialEq)]
enum StatusShown {
    Mask,
    /// A potion: its picture file (`extraData`) and the percent left.
    Potion { item: String, file: Option<String>, percent: i64 },
    /// A hook icon (`ropeandhook.md` rule 7; M6 6a-1): its picture file
    /// (the hook item type's `extraData`).
    Hook { name: &'static str, file: Option<String> },
}

/// One thing to draw, before its pictures and fonts are looked up.
#[derive(Debug, Clone, PartialEq)]
enum Spec {
    Pic { rect: [i32; 4], file: String, frame: usize, cut: Option<u32> },
    /// Only the lowest `rows` rows of a picture (a potion's part left).
    Lower { rect: [i32; 4], file: String, frame: usize, rows: u32 },
    Text { rect: [i32; 4], font: String, justification: u32, text: String },
    Fill { rect: [i32; 4], rgb: [u8; 3] },
    /// The magic map (M6 6d-3): the picture's version, the view, the
    /// arrow (file, frame, top-left from the area's centre).
    Map { rect: [i32; 4], version: u64, view: [f64; 4], arrow: Option<(String, usize, [f64; 2])> },
}

/// The panels and what they show.
#[derive(Resource)]
pub struct Gui {
    game: GameDir,
    world: String,
    tags: Tags,
    root: Control,
    screen: [u32; 2],
    scale: f32,
    palette: [Rgb; 256],
    /// Pictures by lower-case file name (a `.pbm` is one, a `.pba` many);
    /// `None`: not found or not readable (logged once).
    files: HashMap<String, Option<Vec<Bitmap>>>,
    fonts: HashMap<String, Option<Font>>,
    pub panels: Panels,
    meters: Meters,
    /// The armour, hand and ranged slots' item types (rule 34).
    slot_types: [Option<String>; 3],
    /// Their picture files (their types' `GUIBitmap`).
    slot_files: [Option<String>; 3],
    /// The item objects worn there.
    slot_items: [Option<String>; 3],
    /// What Connor carries (4c-3); `None` before the first update.
    inv: InvView,
    first_inv: bool,
    /// The quest strip's first-shown index (`inventory.md` rule 6).
    quest_first: usize,
    /// Items blinking (rule 44): when it started, real ms.
    blinks: HashMap<String, f64>,
    /// The description voice playing (`talking.md` rule 20).
    description: Option<u64>,
    /// The item in hand (4c-4) and the pointer's tag: its type's `cursor`,
    /// or the play pointer `IDBMA_CURS04` (`Gplayscr.kq`).
    hand: Option<String>,
    pointer: String,
    /// Each slot number's flash: when it started (real ms; rule 33).
    flashes: [Option<f64>; 3],
    /// The level-up flame: when it started (real ms; DESIGN L-2, stand-in
    /// Q149 b).
    level_flame: Option<f64>,
    /// The bottom-bar key (`movekeys.kq` row 12, rule 10).
    space_key: Option<KeyCode>,
    /// The map key (`movekeys.kq` row 8, rule 10; `magic-map.md` rule 30).
    map_key: Option<KeyCode>,
    /// The magic map (M6 6d-3); `None` while the world has none.
    map: Option<MapShown>,
    /// 4c-11 (not in the original, the user's idea): the bottom bar's
    /// left and right panels at the window's edges instead of the 4:3
    /// screen's; `--corners on|off`, F6.
    pub corners: bool,
    /// Connor's stats events already sent (rule 24).
    stats_events: Option<u64>,
    hands_off: bool,
    /// The status icon places (M5 5g-3), `IDCTL_TIMER_1` first.
    status: [Option<StatusShown>; kq8_core::potions::ICON_PLACES],
    /// What each drawn picture was last made from.
    last: HashMap<&'static str, Vec<Spec>>,
    log: Option<Log>,
}

/// The screen rectangles of the shown panels, as shares of the window
/// (left, top, right, bottom): a click there is not a world click.
#[derive(Resource, Default)]
pub struct GuiCover(pub Vec<(&'static str, [f32; 4])>);

/// 4c-6: the game's 4:3 screen in the window (left, top, width, height,
/// window pixels), from `kq8_core::panels::screen_frame`.
#[derive(Resource, Default, Clone, Copy)]
pub struct ScreenFrame(pub [f32; 4]);

/// Every frame, first: fits the 4:3 screen into the window; each new
/// frame in `viewer.log`.
pub fn fit_screen(window: Query<&Window, With<bevy::window::PrimaryWindow>>, mut frame: ResMut<ScreenFrame>, shared: Res<crate::Shared>) {
    let Ok(w) = window.single() else { return };
    let new = kq8_core::panels::screen_frame([w.width(), w.height()]);
    if new != frame.0 {
        frame.0 = new;
        let [l, t, fw, fh] = new;
        shared.log.lock().unwrap().detail(format!(
            "screen: window {}x{}: the game's 4:3 screen at left {l:.1} top {t:.1}, {fw:.1}x{fh:.1} (the panels, the caption and the pointer on it; the world fills the window; 4c-6)",
            w.width(),
            w.height()
        ));
    }
}

/// Where the caption box goes (`interface.md` rule 51), screen pixels:
/// the top of the highest bottom panel; the map's right edge.
#[derive(Resource, Default, Clone, Copy)]
pub struct CaptionPlace {
    pub bottom: Option<f32>,
    pub map_right: Option<f32>,
}

/// `--key NAME,N` (4c-2): a key pressed before update N + 1 (`space`,
/// `1`-`8`), or a click on a control of the bars (`left:NAME`,
/// `right:NAME`, NAME as in `Gplayscr.gui`, 4c-3).
#[derive(Resource, Default)]
pub struct KeyAt(pub Vec<(String, u64)>);

/// A Windows virtual-key number (`movekeys.kq` `KeyChar<n>`) as a Bevy
/// key; only the ones the panels use.
fn virtual_key(n: i64) -> Option<KeyCode> {
    Some(match n {
        9 => KeyCode::Tab,
        13 => KeyCode::Enter,
        27 => KeyCode::Escape,
        32 => KeyCode::Space,
        _ => return None,
    })
}

/// The first of `names` (paths under the install) that exists.
fn first_found(game: &GameDir, names: &[String]) -> Option<(String, Vec<u8>)> {
    names.iter().find_map(|n| Some((n.clone(), std::fs::read(game.find(n)?).ok()?)))
}

/// Reads the layout, the tag file, the world's palette and the keys
/// (`movekeys.kq`). `None` (logged) if the layout can't be read.
pub fn load_gui(game: &GameDir, world: &str, main: &mut Log) -> Option<Gui> {
    let fail = |main: &mut Log, why: String| {
        main.summary(format!("gui: {why}; no panels drawn"));
        None
    };
    let Some((_, tag_bytes)) = first_found(game, &["GAME/Resource/kqGuiTag.TTAG".into()]) else { return fail(main, "no kqGuiTag.TTAG".into()) };
    let (tags, _) = Tags::parse(&String::from_utf8_lossy(&tag_bytes));
    let Some((_, gui_bytes)) = first_found(game, &["GAME/Resource/Gplayscr.gui".into()]) else { return fail(main, "no Gplayscr.gui".into()) };
    let root = match kq8_data::gui::parse(&gui_bytes) {
        Ok(l) => l,
        Err(e) => return fail(main, format!("Gplayscr.gui: {e}")),
    };
    // interface.md rules 1c-1f (Q118): the world's objectPalette.
    let (pal_names, pal_note) = crate::load::screen_palette_paths(game, world);
    let Some((pal_path, pal_bytes)) = first_found(game, &pal_names) else { return fail(main, format!("no palette found ({pal_note}; tried {pal_names:?})")) };
    let palette = match kq8_data::Palette::parse(&pal_bytes) {
        Ok(p) => p.shown_colors(),
        Err(e) => return fail(main, format!("{pal_path}: {e}")),
    };
    let wide = crate::talk_view::set800mode(game);
    let screen = kq8_core::caption::screen(wide);
    let scale = kq8_core::caption::scale(screen);
    // Rule 10: the bottom-bar key is row 12 of movekeys.kq, the map key
    // row 8 (M6 6d-3).
    let keys_file = first_found(game, &["movekeys.kq".into()]).map(|(_, b)| kq8_data::kq_text::parse(&b));
    let row = |n: u32, default: KeyCode, manual: &str| match &keys_file {
        Some(file) => {
            let s = file.first();
            let char_n = s.and_then(|s| s.number(&format!("KeyChar{n}"))).and_then(Result::ok).map(|v| v as i64);
            let word_n = s.and_then(|s| s.get(&format!("KeyWord{n}"))).unwrap_or("?").to_string();
            let key = char_n.and_then(virtual_key);
            (key, format!("movekeys.kq row {n}: KeyChar{n}={char_n:?} KeyWord{n}={word_n} -> {key:?}"))
        }
        None => (Some(default), format!("no movekeys.kq: {manual} (the manual's default)")),
    };
    let (space_key, key_line) = row(12, KeyCode::Space, "Space");
    let (map_key, map_key_line) = row(8, KeyCode::Tab, "Tab");
    let mut log = match Log::create("gui") {
        Ok(l) => Some(l),
        Err(e) => {
            main.summary(format!("gui: cannot create gui.log: {e}; panels still drawn, not logged"));
            None
        }
    };
    let head = format!(
        "gui: Gplayscr.gui read; screen {}x{} (set800mode {wide}, layout x {scale}; stand-in Q108); palette {pal_path} (world.cs: {pal_note}; interface.md rule 1c); the bottom-bar key: {key_line}; the map key: {map_key_line} (interface.md rule 10)",
        screen[0], screen[1]
    );
    main.summary(&head);
    if let Some(l) = &mut log {
        l.detail("gui.log: panels shown and hidden with the reason and real time; per stats event the values sent and the bar widths (docs/behaviour/interface.md)");
        l.detail(&head);
    }
    // The screen opens with the bottom bar shown (the user's check 3 in
    // the original; rule 19).
    let panels = Panels::new(&[Panel::BottomLeft, Panel::Slots, Panel::Meters, Panel::BottomRight]);
    Some(Gui {
        game: game.clone(),
        world: world.to_string(),
        tags,
        root,
        screen,
        scale,
        palette,
        files: HashMap::new(),
        fonts: HashMap::new(),
        panels,
        meters: Meters::default(),
        slot_types: [None, None, None],
        slot_files: [None, None, None],
        slot_items: [None, None, None],
        inv: InvView::default(),
        first_inv: true,
        quest_first: 0,
        blinks: HashMap::new(),
        description: None,
        hand: None,
        pointer: PLAY_POINTER.into(),
        flashes: [None; 3],
        level_flame: None,
        space_key,
        map_key,
        map: None,
        corners: false,
        stats_events: None,
        hands_off: false,
        status: Default::default(),
        last: HashMap::new(),
        log,
    })
}

impl Gui {
    fn note(&mut self, line: impl AsRef<str>) {
        if let Some(l) = &mut self.log {
            l.detail(line);
        }
    }

    fn paths(&self, name: &str) -> Vec<String> {
        vec![format!("{}/8gui/{name}", self.world), format!("GAME/8Gui/{name}"), format!("GAME/Resource/{name}")]
    }

    /// Loads a picture file once (`.pbm` one bitmap, `.pba` an array).
    fn load_file(&mut self, name: &str) {
        let key = name.to_ascii_lowercase();
        if self.files.contains_key(&key) {
            return;
        }
        let read = first_found(&self.game, &self.paths(name)).map(|(path, b)| {
            let r = if key.ends_with(".pba") { kq8_data::font::bitmap_array(&b) } else { Bitmap::parse(&b).map(|b| vec![b]) };
            (path, r)
        });
        let value = match read {
            Some((path, Ok(v))) => {
                let sizes: Vec<String> = v.iter().take(3).map(|b| format!("{}x{}", b.width, b.height)).collect();
                self.note(format!("picture {path}: {} frame(s), {} (flags 0x{:x})", v.len(), sizes.join(", "), v.first().map_or(0, |b| b.flags)));
                Some(v)
            }
            Some((path, Err(e))) => {
                self.note(format!("picture {path}: {e}; not drawn"));
                None
            }
            None => {
                self.note(format!("picture {name}: not found in {:?}; not drawn", self.paths(name)));
                None
            }
        };
        self.files.insert(key, value);
    }

    fn load_font(&mut self, name: &str) {
        let key = name.to_ascii_lowercase();
        if self.fonts.contains_key(&key) {
            return;
        }
        let font = first_found(&self.game, &self.paths(name)).and_then(|(path, b)| match Font::parse(&b) {
            Ok(f) => Some(f),
            Err(e) => {
                self.note(format!("font {path}: {e}; its texts not drawn"));
                None
            }
        });
        self.fonts.insert(key, font);
    }

    /// A tag name's file (`IDBMP_2SORD1` → `2sord1.pbm`).
    fn file_of(&self, tag_name: &str) -> Option<String> {
        self.tags.id_of(tag_name).and_then(|id| self.tags.string(id)).map(str::to_string)
    }

    /// The control named `name` in the layout.
    fn control(&self, name: &str) -> Option<&Control> {
        self.tags.id_of(name).and_then(|id| self.root.find(id as u32))
    }

    fn name_of(&self, c: &Control) -> String {
        self.tags.name(c.id as i32).unwrap_or("?").to_string()
    }

    /// Every control of `dialog` with its rectangle relative to it; a
    /// child dialog's controls are moved by its corner (rule 5).
    fn controls<'a>(&'a self, dialog: &'a Control) -> Vec<(&'a Control, [i32; 4])> {
        let mut out: Vec<(&Control, [i32; 4])> = dialog.controls.iter().map(|c| (c, c.rect)).collect();
        for child in &dialog.dialogs {
            let [dx, dy, ..] = child.rect;
            out.extend(self.controls(child).into_iter().map(|(c, [l, t, r, b])| (c, [l + dx, t + dy, r + dx, b + dy])));
        }
        out
    }

    /// Whether item `name` shows its empty picture now (rule 44: a new
    /// item or a higher count switches between its picture and the empty
    /// one every 0.25 s, 5 times).
    fn blank(&self, name: &str) -> bool {
        self.blinks.get(name).is_some_and(|start| {
            let k = ((self.panels.now_ms - start) / BLINK_MS) as u32;
            k < BLINKS && k % 2 == 1
        })
    }

    /// The picture file an item shows in its slot, or `None` while it
    /// blinks (then the layout's empty picture).
    fn shown_file(&self, item: &Option<(String, Option<String>)>) -> Option<String> {
        let (name, file) = item.as_ref()?;
        if self.blank(name) {
            None
        } else {
            file.clone()
        }
    }

    /// The quest strip's items now: the held one left out, the others
    /// closing the gap (`inventory.md` rule 7).
    fn strip(&self) -> Vec<(String, Option<String>)> {
        self.inv.quest.iter().filter(|(n, _)| self.hand.as_deref() != Some(n.as_str())).cloned().collect()
    }

    /// What item a control shows (for clicks): its name.
    fn item_at(&self, control: &str) -> Option<String> {
        let strip = self.strip();
        let slot_no = |prefix: &str| control.strip_prefix(prefix).and_then(|k| k.parse::<usize>().ok());
        if let Some(k) = slot_no("IDCTL_INVBAR_QUEST") {
            // Rule 12: the empty place a held quest item left puts it back.
            let held_quest = self.hand.clone().filter(|h| self.inv.quest.iter().any(|(n, _)| n == h));
            return strip.get(self.quest_first + k - 1).map(|(n, _)| n.clone()).or(held_quest);
        }
        if let Some(k) = slot_no("IDCTL_HEALTHBAR_") {
            return self.inv.slots.get(k.wrapping_sub(1))?.as_ref().filter(|(_, c, _)| *c > 0).map(|(n, ..)| n.clone());
        }
        if let Some(k) = SLOT_PICTURES.iter().position(|n| *n == control) {
            return self.slot_items[k].clone();
        }
        match control {
            "IDCTL_INVBAR_MASK" => self.inv.mask.as_ref().map(|(n, _)| n.clone()),
            "IDCTL_INVBAR_GOLD" => self.inv.gold.as_ref().map(|(n, ..)| n.clone()),
            "IDCTL_INVBAR_ROPE" => self.inv.rope.clone(),
            "IDCTL_INVBAR_ROCKS" => self.inv.rocks.as_ref().map(|(n, ..)| n.clone()),
            _ => None,
        }
    }

    /// What drawn picture `d` is made of now (its layout size and the
    /// pieces), or `None` if its block is missing.
    fn specs(&self, d: Drawn) -> Option<([i32; 2], Vec<Spec>)> {
        if d == Drawn::Status {
            return self.status_specs();
        }
        let dialog = self.control(d.layout_id())?;
        let [l, t, r, b] = dialog.rect;
        let now = self.panels.now_ms;
        let frame_ms = 1000.0 / ANIMATION_FPS;
        let strip = self.strip();
        let mut out = Vec::new();
        for (c, rect) in self.controls(dialog) {
            let name = self.name_of(c);
            let slot = |list: &[&str; 3]| list.iter().position(|n| *n == name);
            let numbered = |prefix: &str| name.strip_prefix(prefix).and_then(|k| k.parse::<usize>().ok());
            match &c.kind {
                Kind::Bitmap(p) => {
                    let default = self.tags.string(p.bitmap).map(str::to_string);
                    let file = if let Some(k) = slot(&SLOT_PICTURES) {
                        // Rule 34: the worn item's picture, else the empty one.
                        self.slot_files[k].clone()
                    } else if let Some(k) = numbered("IDCTL_INVBAR_QUEST") {
                        // inventory.md rules 5-6: from the first-shown index.
                        self.shown_file(&strip.get(self.quest_first + k - 1).cloned())
                    } else if let Some(k) = numbered("IDCTL_HEALTHBAR_") {
                        // Rules 38-39: a stack at 0 shows the empty picture.
                        let item = self.inv.slots.get(k - 1).cloned().flatten().filter(|(_, n, _)| *n > 0).map(|(n, _, f)| (n, f));
                        self.shown_file(&item)
                    } else {
                        match name.as_str() {
                            // Rule 41: the mask piece carried.
                            "IDCTL_INVBAR_MASK" => self.shown_file(&self.inv.mask),
                            // Rule 43: the coins by count.
                            "IDCTL_INVBAR_GOLD" => self.inv.gold.as_ref().and_then(|(n, count, _)| {
                                let step = match *count {
                                    c if c <= 0 => 0,
                                    1..=24 => 1,
                                    25..=49 => 2,
                                    _ => 3,
                                };
                                if self.blank(n) {
                                    None
                                } else {
                                    self.file_of(&format!("IDBMP_2GOLD{step}"))
                                }
                            }),
                            // Rule 45: the rope carried.
                            "IDCTL_INVBAR_ROPE" => self.inv.rope.as_ref().filter(|n| !self.blank(n)).and_then(|_| self.file_of("IDBMP_2ROPE1")),
                            // Rule 45; the levels are not traced (stand-in,
                            // Q109: the rock type's own picture when he has
                            // any).
                            "IDCTL_INVBAR_ROCKS" => self.inv.rocks.as_ref().filter(|(n, c, _)| *c > 0 && !self.blank(n)).and_then(|(_, _, f)| f.clone()),
                            _ => None,
                        }
                    }
                    .or(default);
                    // Rule 20: the meters' bars cut to a width.
                    let full = (gui_draw::scaled(rect[2], self.scale) - gui_draw::scaled(rect[0], self.scale)).max(0) as u32;
                    let cut = match name.as_str() {
                        "IDCTL_METERS_HEALTH" => Some(meters::bar_width(full, self.meters.health_percent)),
                        "IDCTL_METERS_EXP" => Some(meters::bar_width(full, self.meters.experience_percent)),
                        _ => None,
                    };
                    // M6 6d-3 (magic-map.md rules 6, 18): the map over the
                    // black area, the arrow when Connor's pixel is known.
                    if name == MAP_AREA {
                        if let Some(m) = self.map.as_ref().filter(|m| m.version.is_some()) {
                            let view = kq8_core::magic_map::View { left: m.view[0], top: m.view[1], right: m.view[2], bottom: m.view[3] };
                            let arrow = m.pixel.map(|p| {
                                let (x, y) = view.arrow_at(p);
                                (m.arrow_file.clone(), m.frame, [x, y])
                            });
                            out.push(Spec::Map { rect, version: m.version.unwrap_or(0), view: m.view, arrow });
                        }
                    }
                    if let Some(file) = file {
                        out.push(Spec::Pic { rect, file, frame: 0, cut });
                    }
                    // 4c-8 (the user's check in the original, Q111 d):
                    // black behind the bars, where they are empty.
                    if cut.is_some() {
                        out.push(Spec::Fill { rect, rgb: [0, 0, 0] });
                    }
                }
                Kind::Text { text, pictures: None } => {
                    // Rules 23, 36, 39, 43, 45: the numbers we set; the
                    // layout's own placeholder texts ("0", "3Dfx") are not
                    // drawn, nor the red alternative numbers (rule 32, M5).
                    // Counts of 0 are not written (stand-in, Q109).
                    let count = |c: Option<i64>| c.filter(|n| *n > 0).map(|n| n.to_string());
                    let value = match (slot(&SLOT_NUMBERS), name.as_str()) {
                        // potions.md rule 37: the red number replaces the
                        // white one while it is on.
                        (Some(k), _) if self.meters.red[k] => None,
                        (Some(k), _) => self.meters.numbers.map(|n| n[k].to_string()),
                        (None, n) if slot(&SLOT_RED_NUMBERS).is_some() => {
                            let k = SLOT_RED_NUMBERS.iter().position(|x| *x == n).expect("listed");
                            self.meters.numbers.filter(|_| self.meters.red[k]).map(|n| n[k].to_string())
                        }
                        (None, "IDCTL_METERS_LEVEL") => Some(self.meters.level.to_string()),
                        // magic-map.md rule 8: one line in TEXT1, two in
                        // TEXT2 and TEXT3.
                        (None, MAP_NAME_ONE) => self.map.as_ref().filter(|m| m.name.len() == 1).map(|m| m.name[0].clone()),
                        (None, n) if MAP_NAME_TWO.contains(&n) => {
                            let k = MAP_NAME_TWO.iter().position(|x| *x == n).expect("listed");
                            self.map.as_ref().filter(|m| m.name.len() >= 2).map(|m| m.name[k].clone())
                        }
                        (None, "IDCTL_INVBAR_GOLDTEXT") => count(self.inv.gold.as_ref().map(|(_, c, _)| *c)),
                        (None, "IDCTL_INVBAR_ROCKSTEXT") => count(self.inv.rocks.as_ref().map(|(_, c, _)| *c)),
                        (None, n) => match n.strip_prefix("IDCTL_HEALTHBAR_").and_then(|k| k.strip_suffix("TEXT")).and_then(|k| k.parse::<usize>().ok()) {
                            Some(k) => count(self.inv.slots.get(k - 1).cloned().flatten().map(|(_, c, _)| c)),
                            None => None,
                        },
                    };
                    if let (Some(value), Some(font)) = (value, self.tags.string(text.font)) {
                        out.push(Spec::Text { rect, font: font.to_string(), justification: text.justification, text: value });
                    }
                }
                Kind::Text { pictures: Some(p), .. } => {
                    let Some(file) = self.tags.string(p.bitmap).map(str::to_string) else { continue };
                    // Rule 33: a slot's flash plays once over its slot.
                    if let Some(k) = slot(&SLOT_FLASHES) {
                        if let (Some(start), Some(at)) = (self.flashes[k], self.control(SLOT_PICTURES[k])) {
                            out.push(Spec::Pic { rect: at.rect, file, frame: ((now - start) / frame_ms) as usize, cut: None });
                        }
                    } else if MAP_BUTTONS.contains(&name.as_str()) {
                        // magic-map.md rule 5: the page arrows and the unlit
                        // "Go" button, their first frames.
                        out.push(Spec::Pic { rect, file, frame: 0, cut: None });
                    } else if name == LEVEL_FLAME {
                        // DESIGN L-2 (stand-in Q149 b): over the level
                        // number, in its own layout place.
                        if let Some(start) = self.level_flame {
                            out.push(Spec::Pic { rect, file, frame: ((now - start) / frame_ms) as usize, cut: None });
                        }
                    }
                }
                _ => {}
            }
        }
        Some(([r - l, b - t], out))
    }

    /// The control under screen point (`x`, `y`) of drawn picture `d`
    /// standing at (`left`, `top`): the first listed is on top; texts and
    /// the flashes parked off screen don't count.
    fn control_at(&self, d: Drawn, left: f32, top: f32, x: f32, y: f32) -> Option<String> {
        let dialog = self.control(d.layout_id())?;
        let (px, py) = (x - left, y - top);
        self.controls(dialog)
            .into_iter()
            .filter(|(c, r)| r[2] > r[0] && r[3] > r[1] && r[1] < 480 && !matches!(c.kind, Kind::Text { pictures: None, .. }))
            .find(|(_, r)| {
                let [l, t, r, b] = r.map(|v| gui_draw::scaled(v, self.scale) as f32);
                px >= l && px < r && py >= t && py < b
            })
            .map(|(c, _)| self.name_of(c))
    }

    /// Draws a picture `size` layout pixels big from `specs`, loading the
    /// pictures and fonts it needs.
    fn draw(&mut self, origin: [i32; 2], size: [i32; 2], specs: &[Spec], scale: f32) -> gui_draw::Canvas {
        for s in specs {
            match s {
                Spec::Pic { file, .. } | Spec::Lower { file, .. } | Spec::Map { arrow: Some((file, ..)), .. } => self.load_file(file),
                Spec::Text { font, .. } => self.load_font(font),
                Spec::Fill { .. } | Spec::Map { .. } => {}
            }
        }
        let mut pieces = Vec::new();
        for s in specs {
            match s {
                Spec::Pic { rect, file, frame, cut } => {
                    let Some(Some(frames)) = self.files.get(&file.to_ascii_lowercase()) else { continue };
                    let Some(pic) = frames.get(*frame % frames.len().max(1)).and_then(Pic::of) else { continue };
                    pieces.push(Piece::Picture { rect: *rect, pic, cut: *cut });
                }
                Spec::Lower { rect, file, frame, rows } => {
                    let Some(Some(frames)) = self.files.get(&file.to_ascii_lowercase()) else { continue };
                    let Some(pic) = frames.get(*frame % frames.len().max(1)).and_then(Pic::of) else { continue };
                    pieces.push(Piece::Lower { rect: *rect, pic, rows: *rows });
                }
                Spec::Text { rect, font, justification, text } => {
                    let Some(Some(f)) = self.fonts.get(&font.to_ascii_lowercase()) else { continue };
                    pieces.push(Piece::Text { rect: *rect, font: f, justification: *justification, text: text.clone() });
                }
                Spec::Fill { rect, rgb } => pieces.push(Piece::Fill { rect: *rect, rgb: *rgb }),
                Spec::Map { rect, view, arrow, .. } => {
                    let Some(m) = &self.map else { continue };
                    let arrow = arrow.as_ref().and_then(|(file, frame, at)| {
                        let frames = self.files.get(&file.to_ascii_lowercase())?.as_ref()?;
                        frames.get(*frame % frames.len().max(1)).and_then(Pic::of).map(|p| (p, *at))
                    });
                    pieces.push(Piece::Map { rect: *rect, pixels: &m.pixels, view: *view, arrow });
                }
            }
        }
        gui_draw::draw(origin, size, scale, &pieces, &self.palette)
    }

    /// M5 5g-3 (`potions.md` rule 35): the status icons, each at its
    /// layout place standing on the picture's bottom. A potion: the lower
    /// part (the time left) from its picture's frame 1, the part used
    /// from frame 0 (each potion `.pba` has two frames: 0 is a black
    /// square with the gold outline, 1 the picture; the user saw a black
    /// background and a gold outline where it is used). The spinning mask
    /// at its own size (4c-8), its bottom-right at its place's, at 10
    /// frames a second.
    fn status_specs(&self) -> Option<([i32; 2], Vec<Spec>)> {
        let [w, h] = STATUS_SIZE;
        let mut out = Vec::new();
        for (k, shown) in self.status.iter().enumerate() {
            let Some(shown) = shown else { continue };
            let Some(c) = self.control(&format!("IDCTL_TIMER_{}", k + 1)) else { continue };
            let [l, t, r, b] = c.rect;
            let tall = b - t;
            match shown {
                StatusShown::Mask => {
                    let Some(file) = self.mask_file() else { continue };
                    let [mw, mh] = self.mask_size().unwrap_or([r - l, tall]);
                    let frame = (self.panels.now_ms / (1000.0 / MASK_FPS)) as usize;
                    out.push(Spec::Pic { rect: [r - mw, h - mh, r, h], file, frame, cut: None });
                }
                StatusShown::Potion { file: Some(file), percent, .. } => {
                    let rect = [l, h - tall, r, h];
                    let rows = (tall as i64 * percent / 100).clamp(0, tall as i64) as u32;
                    out.push(Spec::Lower { rect, file: file.clone(), frame: 1, rows });
                    out.push(Spec::Pic { rect, file: file.clone(), frame: 0, cut: None });
                }
                StatusShown::Potion { file: None, .. } => {}
                // M6 6a-1: drawn whole at its place; a picture of several
                // frames animates at 10 a second like the mask (potions.md
                // rule 35, last point: no duration).
                StatusShown::Hook { file: Some(file), .. } => {
                    let frame = (self.panels.now_ms / (1000.0 / MASK_FPS)) as usize;
                    out.push(Spec::Pic { rect: [l, h - tall, r, h], file: file.clone(), frame, cut: None });
                }
                StatusShown::Hook { file: None, .. } => {}
            }
        }
        Some(([w, h], out))
    }

    /// The spinning mask's file (`IDCTL_TIMER_7`).
    fn mask_file(&self) -> Option<String> {
        match &self.control("IDCTL_TIMER_7")?.kind {
            Kind::Text { pictures: Some(p), .. } => self.tags.string(p.bitmap).map(str::to_string),
            _ => None,
        }
    }

    /// The spinning mask's own picture size, once loaded.
    fn mask_size(&self) -> Option<[i32; 2]> {
        let frames = self.files.get(&self.mask_file()?.to_ascii_lowercase())?.as_ref()?;
        frames.first().map(|b| [b.width as i32, b.height as i32])
    }

    /// How many frames a `.pba` has (0 if not loaded).
    fn frames(&mut self, file: &str) -> usize {
        self.load_file(file);
        self.files.get(&file.to_ascii_lowercase()).and_then(|f| f.as_ref()).map_or(0, Vec::len)
    }

    /// The level-up flame's picture file (`flame14.pba`, from the layout).
    fn flame_file(&self) -> Option<String> {
        self.control(LEVEL_FLAME).and_then(|c| match &c.kind {
            Kind::Text { pictures: Some(p), .. } => self.tags.string(p.bitmap).map(str::to_string),
            _ => None,
        })
    }

    /// `magic-map.md` rule 30 (M6 6d-3): the map key or a left click on
    /// the mini-map toggles the window, only if the map is active; nothing
    /// else is checked (not handsOff, not a conversation).
    fn toggle_map(&mut self, how: &str) {
        if self.map.as_ref().is_some_and(|m| m.active) {
            self.panels.toggle_map(&format!("{how} (magic-map.md rule 30)"));
        } else {
            let line = format!("{how}: the map is not active (or this world has none); nothing (magic-map.md rule 30)");
            self.note(line);
        }
    }

    /// The shown rectangle of a drawn picture on the screen, pixels.
    fn rect(&self, d: Drawn) -> Option<[f32; 4]> {
        self.control(d.layout_id()).map(|c| c.rect.map(|v| gui_draw::scaled(v, self.scale) as f32))
    }
}

/// What Connor carries, for the bars (4c-3).
fn inventory_view(gui: &Gui, items: &kq8_core::item::Items) -> InvView {
    let mut v = InvView::default();
    let mut carried: Vec<&kq8_core::item::ItemObject> = items.objects.iter().filter(|o| o.carried_by("Connor")).collect();
    carried.sort_by_key(|o| o.added);
    for o in carried {
        let Some(t) = items.kind(&o.kind) else { continue };
        let file = gui.file_of(&t.picture);
        v.counts.insert(o.name.clone(), (o.quantity, t.category.clone()));
        match t.category.as_str() {
            "QuestItem" => v.quest.push((o.name.clone(), file)),
            "MaskPiece" => v.mask = Some((o.name.clone(), file)),
            "Money" => v.gold = Some((o.name.clone(), o.quantity, file)),
            "Rope" => v.rope = Some(o.name.clone()),
            "Rock" => v.rocks = Some((o.name.clone(), o.quantity, file)),
            "HealthItem" => {
                if let Some(k) = kq8_core::connor_inventory::SLOT_TYPES.iter().position(|s| s.eq_ignore_ascii_case(&o.kind)) {
                    v.slots[k] = Some((o.name.clone(), o.quantity, file));
                }
            }
            _ => {}
        }
    }
    v
}

/// One drawn picture's node.
#[derive(Component)]
pub struct GuiPicture(usize);

/// Spawns one hidden node per drawn picture.
pub fn setup_gui(mut commands: Commands, gui: Option<Res<Gui>>, mut images: ResMut<Assets<Image>>) {
    if gui.is_none() {
        return;
    }
    for k in 0..DRAWN.len() {
        let image = images.add(Image::new_fill(Extent3d { width: 1, height: 1, depth_or_array_layers: 1 }, TextureDimension::D2, &[0, 0, 0, 0], TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default()));
        commands.spawn((
            Node { position_type: PositionType::Absolute, left: Val::Percent(0.0), top: Val::Percent(100.0), width: Val::Percent(0.0), height: Val::Percent(0.0), ..default() },
            // 4c-10: the picture fills its box exactly (Bevy's default
            // keeps its proportions and centres it, which left the world
            // showing at the meters' ends, the user's check).
            ImageNode { image_mode: bevy::ui::widget::NodeImageMode::Stretch, ..ImageNode::new(image) },
            Visibility::Hidden,
            GuiPicture(k),
        ));
    }
}

/// One game update (15 per second), after the scripts and Connor: the
/// stats events, handsOff, the script commands, `--key`, and what the
/// slots show.
#[allow(clippy::too_many_arguments)]
pub fn gui_step(
    gui: Option<ResMut<Gui>>,
    sim: Option<ResMut<DoorSim>>,
    connor: Option<Res<ConnorSim>>,
    mut key_at: ResMut<KeyAt>,
    mut actions: ResMut<BarActions>,
    mut speech: ResMut<crate::talk_view::Speech>,
    shared: Res<Shared>,
    keys: Res<ButtonInput<KeyCode>>,
) {
    let (Some(mut gui), Some(mut sim)) = (gui, sim) else { return };
    let gui = &mut *gui;
    let updates = sim.active.world.updates;
    // Test keys (`--key NAME,N`).
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut key_at.0).into_iter().partition(|(_, n)| updates >= *n);
    key_at.0 = later;
    for (name, _) in due {
        let how = format!("test --key {name} before update {updates}");
        match name.to_ascii_lowercase().as_str() {
            "space" => gui.panels.toggle_bottom(&how),
            "tab" => gui.toggle_map(&how),
            "f4" => crate::door_view::flip_ai(&mut sim, &how),
            "f3" => crate::door_view::flip_god(&mut sim, &how),
            // ranged-weapons.md rule 1 (M5 5f-3): Delete (Draw), Alt (Away).
            "delete" => sim.active.world.ask_weapon(kq8_core::weapons::Ask::Draw, &format!("update {updates}: {how} (Draw)")),
            "alt" => sim.active.world.ask_weapon(kq8_core::weapons::Ask::Away, &format!("update {updates}: {how} (Away)")),
            k if k.len() == 1 && ("1"..="8").contains(&k) => actions.0.push(BarAction::Key { slot: k.parse::<usize>().unwrap_or(1) - 1, how }),
            // A click on a control of the bars: left:NAME / right:NAME.
            k if k.starts_with("left:") || k.starts_with("right:") => {
                let (button, control) = name.split_once(':').unwrap_or_default();
                let control = control.to_ascii_uppercase();
                let left = button.eq_ignore_ascii_case("left");
                match (control.as_str(), gui.item_at(&control)) {
                    ("IDCTL_INVBAR_QUESTL", _) if left => actions.0.push(BarAction::Scroll { by: -1, how }),
                    ("IDCTL_INVBAR_QUESTR", _) if left => actions.0.push(BarAction::Scroll { by: 1, how }),
                    (MINIMAP, _) if left => gui.toggle_map(&how),
                    (_, Some(item)) => actions.0.push(BarAction::Item { item, left, how }),
                    _ => {
                        let line = format!("update {updates}: {how}: nothing there; nothing (inventory.md rule 10)");
                        gui.note(line);
                    }
                }
            }
            other => {
                let line = format!("update {updates}: test --key {other}: not a panel key; nothing");
                gui.note(line);
            }
        }
    }
    // inventory.md rules 6, 9-11, interface.md rule 40: the bars' clicks
    // and keys since the last update.
    for action in std::mem::take(&mut actions.0) {
        let w = &mut sim.active.world;
        match action {
            BarAction::Scroll { by, how } => {
                // Rule 6: +-1, then kept between 0 and count - 7; the held
                // item still counts (rule 7).
                let count = gui.inv.quest.len();
                let before = gui.quest_first;
                gui.quest_first = (before as i64 + by as i64).clamp(0, count.saturating_sub(7) as i64) as usize;
                let line = format!("update {updates}: quest strip arrow {by:+} ({how}): first shown {before} -> {} of {count} (inventory.md rule 6)", gui.quest_first);
                gui.note(line);
            }
            BarAction::Key { slot, how } => match kq8_core::connor_inventory::slot_item(w, slot) {
                Some(name) => {
                    kq8_core::connor_inventory::bar_click(w, &name, true, gui.hands_off, &how);
                }
                _ => {
                    let line = format!("update {updates}: {how}: slot {} is empty; nothing (interface.md rule 40)", slot + 1);
                    gui.note(line);
                }
            },
            BarAction::Item { item, left, how } => {
                kq8_core::connor_inventory::bar_click(w, &item, left, gui.hands_off, &how);
            }
        }
    }
    // talking.md rule 20: a description is a plain voice; a new one stops
    // the last one.
    for v in std::mem::take(&mut sim.active.world.item_voices) {
        if let Some(id) = gui.description.take() {
            speech.voices.push(crate::talk_view::VoiceOrder::Stop { line: id });
        }
        if let Some(path) = v.path {
            static NEXT: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(1 << 62);
            let id = NEXT.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            gui.description = Some(id);
            speech.voices.push(crate::talk_view::VoiceOrder::Start { line: id, path, at: None, why: v.why });
        }
    }
    // Rule 15: handsOff going on or off.
    let hands_off = connor.as_ref().is_some_and(|c| c.hands_off);
    gui.hands_off = hands_off;
    // azriel-hammer.md rule 3 (M6 6b-3): the hammer throw's handsOff
    // leaves the panels as they are (STAND-IN, Q148 e); clicks on them are
    // still ignored (`gui.hands_off`).
    gui.panels.hands_off(connor.as_ref().is_some_and(|c| c.hands_off && c.throw.is_none()));
    // M5 5g-3 (potions.md rule 35): the spinning mask takes a status icon
    // place like a potion (STAND-IN, Q143: first free).
    {
        use kq8_core::potions::{free_place, take_place, StatusIcon};
        let places = &mut sim.active.world.status_icons;
        let has = places.contains(&Some(StatusIcon::Mask));
        let change = match (gui.panels.mask_icon, has) {
            (true, false) => take_place(places, StatusIcon::Mask).map(|k| format!("the spinning mask takes IDCTL_TIMER_{}", k + 1)),
            (false, true) => free_place(places, &StatusIcon::Mask).map(|k| format!("the spinning mask leaves IDCTL_TIMER_{}", k + 1)),
            _ => None,
        };
        if let Some(line) = change {
            gui.note(format!("update {updates}: {line} (potions.md rule 35; first free: stand-in, Q143)"));
        }
    }
    let status: [Option<StatusShown>; kq8_core::potions::ICON_PLACES] = sim.active.world.status_icons.clone().map(|p| {
        p.map(|icon| match icon {
            kq8_core::potions::StatusIcon::Mask => StatusShown::Mask,
            kq8_core::potions::StatusIcon::Hook(hook) => {
                let file = sim.active.world.items.kind(hook.item_type()).and_then(|t| t.extra.clone()).and_then(|tag| gui.file_of(&tag));
                StatusShown::Hook { name: hook.name(), file }
            }
            kq8_core::potions::StatusIcon::Potion(item) => {
                let w = &sim.active.world;
                let running = w.potions.iter().find(|r| r.item == item);
                let file = running.and_then(|r| w.items.kind(&r.kind)).and_then(|t| t.extra.clone()).and_then(|tag| gui.file_of(&tag));
                StatusShown::Potion { item, file, percent: running.map_or(0, |r| r.percent()) }
            }
        })
    });
    if status != gui.status {
        for (k, (old, new)) in gui.status.clone().iter().zip(&status).enumerate() {
            if old != new {
                let what = match new {
                    None => "empty".to_string(),
                    Some(StatusShown::Mask) => "the spinning mask".to_string(),
                    Some(StatusShown::Hook { name, file }) => format!("the hook icon {name}, {} (ropeandhook.md rule 7)", file.as_deref().unwrap_or("no picture")),
                    Some(StatusShown::Potion { item, file, percent }) => format!("{item} {} at {percent} % (its lowest {} of 48 rows from frame 1, the rest frame 0)", file.as_deref().unwrap_or("no picture"), 48 * percent / 100),
                };
                gui.note(format!("update {updates}: IDCTL_TIMER_{}: {what} (potions.md rule 35)", k + 1));
            }
        }
        gui.status = status;
    }
    // Rule 15: the script commands.
    for order in std::mem::take(&mut sim.active.world.gui_orders) {
        match order {
            kq8_core::active::GuiOrder::Set { group, on, at } => {
                gui.panels.set(group, on, &at);
            }
            kq8_core::active::GuiOrder::Status { at } => {
                let line = format!("update {updates}: {at}: getGuiStatus: {}", gui.panels.status());
                gui.note(line);
            }
            // M4 4e-3: interface.md rule 16 (built in 4c-2).
            kq8_core::active::GuiOrder::Conversation { start, at } => {
                gui.note(format!("update {updates}: {at}"));
                gui.panels.conversation(start);
            }
        }
    }
    // M6 6d-3: the magic map as the panels show it.
    if let Some(m) = &sim.active.world.map {
        let shown = gui.map.get_or_insert_with(MapShown::default);
        if shown.version != Some(m.version) {
            shown.pixels.clone_from(&m.picture);
            shown.version = Some(m.version);
        }
        let was_active = shown.active;
        shown.active = m.active;
        shown.view = m.view.as_array();
        shown.pixel = m.last_pixel;
        shown.frame = m.frame;
        shown.arrow_file.clone_from(&m.keys.arrow_pba);
        if shown.name.is_empty() {
            // Rule 8: 500.MSG noun 0 verb 0 case n; a line break gives two lines.
            let n = m.keys.map_name.unwrap_or(0);
            let text = sim.active.world.talk.book.files.get(&500).and_then(|(_, f)| f.lines.iter().find(|l| l.tuple.noun == 0 && l.tuple.verb == 0 && l.tuple.case as u32 == n)).map(|l| l.raw.clone());
            shown.name = text.as_deref().unwrap_or("").split('\n').map(|l| l.trim_end_matches('\r').to_string()).filter(|l| !l.is_empty()).take(2).collect();
            let line = format!("update {updates}: the map's name plate: 500.MSG 0 0 {n}: {:?} ({} line(s); magic-map.md rule 8)", shown.name, shown.name.len());
            gui.note(line);
        }
        if was_active != m.active {
            let line = format!("update {updates}: the map is {} (magic-map.md rule 2): the map key and the mini-map slot {}", if m.active { "active" } else { "not active" }, if m.active { "work" } else { "do nothing" });
            gui.note(line);
        }
    }
    // Rules 34-35: the slot pictures follow what Connor wears.
    let (types, why) = kq8_core::inventory::slot_types(&sim.active.world.items, "Connor");
    // 4c-4: the item in hand and the pointer (inventory.md rules 7, 9).
    let hand = kq8_core::connor_inventory::hand(&mut sim.active.world);
    // fighting.md rule 39 (manual): with a hand weapon ready the pointer
    // is the sword; Ctrl held gives the play pointer.
    let ctrl = keys.pressed(KeyCode::ControlLeft) || keys.pressed(KeyCode::ControlRight);
    // ranged-weapons.md rules 11, 12 (M5 5f-3): by the mode at the list
    // switch: hand the sword, ranged the bow, none the play pointer; Ctrl
    // with a weapon ready: the play pointer.
    let mode = connor.as_ref().map_or(sim.active.world.weapon_mode, |c| c.set_mode);
    let pointer = hand.as_ref().and_then(|(_, kind)| sim.active.world.items.kind(kind).and_then(|t| t.cursor.clone())).unwrap_or_else(|| {
        match mode {
            _ if ctrl => PLAY_POINTER,
            kq8_core::weapons::WeaponMode::Hand => SWORD_POINTER,
            kq8_core::weapons::WeaponMode::Ranged => BOW_POINTER,
            kq8_core::weapons::WeaponMode::None => PLAY_POINTER,
        }
        .to_string()
    });
    let hand = hand.map(|(n, _)| n);
    if hand != gui.hand || pointer != gui.pointer {
        let line = format!("update {updates}: the item in hand {:?} -> {hand:?}; the pointer {pointer} (inventory.md rules 7, 9, 12)", gui.hand);
        gui.note(line);
        gui.hand = hand;
        gui.pointer = pointer;
    }
    let worn: [Option<String>; 3] = types.clone().map(|t| {
        t.and_then(|t| sim.active.world.items.objects.iter().find(|o| o.carried_by("Connor") && o.kind == t).map(|o| o.name.clone()))
    });
    gui.slot_items = worn;
    // 4c-3: what the bars show; new items and higher counts blink (rule
    // 44); a new quest item or mask piece pops the top bar (rule 13).
    let inv = inventory_view(gui, &sim.active.world.items);
    if inv != gui.inv {
        let now = gui.panels.now_ms;
        let mut news = Vec::new();
        if !gui.first_inv {
            for (name, (count, category)) in &inv.counts {
                let before = gui.inv.counts.get(name).map(|(c, _)| *c);
                if before.is_none_or(|b| *count > b) {
                    gui.blinks.insert(name.clone(), now);
                    if before.is_none() && (category == "QuestItem" || category == "MaskPiece") {
                        news.push(name.clone());
                    }
                }
            }
        }
        let line = format!(
            "update {updates}: the bars show: quest strip {:?} (first shown {}), mask {:?}, gold {:?}, slots {:?}, rope {:?}, rocks {:?}; blinking {:?}",
            inv.quest.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>(),
            gui.quest_first,
            inv.mask.as_ref().map(|(n, _)| n),
            inv.gold.as_ref().map(|(n, c, _)| (n, c)),
            inv.slots.iter().map(|s| s.as_ref().map(|(_, c, _)| *c).unwrap_or(0)).collect::<Vec<_>>(),
            inv.rope,
            inv.rocks.as_ref().map(|(n, c, _)| (n, c)),
            gui.blinks.keys().collect::<Vec<_>>()
        );
        gui.note(line);
        gui.inv = inv;
        gui.first_inv = false;
        let count = gui.inv.quest.len();
        gui.quest_first = gui.quest_first.min(count.saturating_sub(7));
        if !news.is_empty() {
            gui.panels.pop_top(&format!("new in the inventory: {news:?} (rule 13)"));
        }
    }
    if types != gui.slot_types {
        let items = &sim.active.world.items;
        let files = types.clone().map(|t| t.and_then(|t| items.kind(&t).map(|k| k.picture.clone())).and_then(|tag| gui.file_of(&tag)));
        let line = format!(
            "update {updates}: slot pictures armour {:?} ({why}), hand {:?}, ranged {:?}: files {files:?} (interface.md rules 34-35)",
            types[0], types[1], types[2]
        );
        gui.note(line);
        gui.slot_types = types;
        gui.slot_files = files;
    }
    // Rule 24: after every stats event on Connor, all values are sent.
    let events = sim.active.world.connor_stats_events;
    let Some(stats) = sim.active.world.connor_stats else { return };
    if gui.stats_events == Some(events) {
        return;
    }
    let first = gui.stats_events.is_none();
    gui.stats_events = Some(events);
    // Dying states are M5: never dead yet.
    let red = kq8_core::potions::red_numbers(&sim.active.world.potions);
    let old_level = gui.meters.level;
    let sent = gui.meters.send(&stats, false, red);
    let width = |name: &str| {
        gui.control(name).map_or(0, |c| (gui_draw::scaled(c.rect[2], gui.scale) - gui_draw::scaled(c.rect[0], gui.scale)).max(0) as u32)
    };
    let (hw, ew) = (width("IDCTL_METERS_HEALTH"), width("IDCTL_METERS_EXP"));
    let exp = match sent.experience_percent {
        Some(p) => format!("{p} % (bar {} of {ew} px)", meters::bar_width(ew, p)),
        None => "not sent (unchanged, rule 22)".into(),
    };
    let line = format!(
        "update {updates}: {}: Connor {:?} level {}: sent armour {}, hand {}, ranged {}, health {} % (bar {} of {hw} px), experience {exp}, level {}; flashes {:?}; red armour {}, hand {}, ranged {} (interface.md rules 20-24, 32-33, potions.md rule 37)",
        if first { "the screen opens".to_string() } else { format!("stats event #{events}") },
        stats.stats.0,
        stats.level,
        sent.numbers[0],
        sent.numbers[1],
        sent.numbers[2],
        sent.health_percent,
        meters::bar_width(hw, sent.health_percent),
        sent.level,
        sent.flashes.iter().map(|s| s.name()).collect::<Vec<_>>(),
        sent.red[0],
        sent.red[1],
        sent.red[2]
    );
    gui.note(&line);
    shared.log.lock().unwrap().detail(format!("gui: {line}"));
    let now = gui.panels.now_ms;
    for s in &sent.flashes {
        let k = Slot::ALL.iter().position(|x| x == s).expect("listed");
        gui.flashes[k] = Some(now);
    }
    // DESIGN L-2 (stand-in Q149 b): a higher level starts the flame.
    if !first && sent.level > old_level {
        gui.level_flame = Some(now);
        let frames = gui.flame_file().map_or(0, |f| gui.frames(&f));
        let line = format!("update {updates}: level {old_level} -> {}: the level-up flame starts ({frames} frames at {ANIMATION_FPS} per second, once; stand-in Q149 b)", sent.level);
        gui.note(&line);
        shared.log.lock().unwrap().detail(format!("gui: {line}"));
    }
    if !first {
        gui.panels.meters_sent(&format!("stats event #{events}"));
    }
}

/// Every frame: real time moves the panels; Space; each picture redrawn
/// when what it shows changed, and placed; the panels' sounds; the
/// caption's place; where clicks are not world clicks.
#[allow(clippy::too_many_arguments)]
pub fn gui_draw(
    gui: Option<ResMut<Gui>>,
    sim: Option<ResMut<DoorSim>>,
    time: Res<Time<Real>>,
    keys: Res<ButtonInput<KeyCode>>,
    buttons: Res<ButtonInput<MouseButton>>,
    window: Query<&Window, With<bevy::window::PrimaryWindow>>,
    mut actions: ResMut<BarActions>,
    mut nodes: Query<(&GuiPicture, &mut Node, &ImageNode, &mut Visibility)>,
    mut images: ResMut<Assets<Image>>,
    mut cover: ResMut<GuiCover>,
    mut caption: ResMut<CaptionPlace>,
    frame: Res<ScreenFrame>,
    mut status_at: Local<Option<[i32; 4]>>,
) {
    let Some(mut gui) = gui else { return };
    let gui = &mut *gui;
    if gui.space_key.is_some_and(|k| keys.just_pressed(k)) {
        gui.panels.toggle_bottom("Space key");
    }
    // magic-map.md rule 30 (M6 6d-3): the map key, also in handsOff.
    if gui.map_key.is_some_and(|k| keys.just_pressed(k)) {
        gui.toggle_map("the map key");
    }
    // 4c-11: F6, the corner panels on or off.
    if keys.just_pressed(KeyCode::F6) {
        gui.corners = !gui.corners;
        let line = format!("F6: corner panels {} (4c-11, not in the original)", if gui.corners { "on: bottom left and right at the window's edges" } else { "off: on the 4:3 screen" });
        gui.note(line);
    }
    // interface.md rule 40: keys 1-8 and the keypad's.
    const DIGITS: [(KeyCode, KeyCode); 8] = [
        (KeyCode::Digit1, KeyCode::Numpad1),
        (KeyCode::Digit2, KeyCode::Numpad2),
        (KeyCode::Digit3, KeyCode::Numpad3),
        (KeyCode::Digit4, KeyCode::Numpad4),
        (KeyCode::Digit5, KeyCode::Numpad5),
        (KeyCode::Digit6, KeyCode::Numpad6),
        (KeyCode::Digit7, KeyCode::Numpad7),
        (KeyCode::Digit8, KeyCode::Numpad8),
    ];
    for (k, (a, b)) in DIGITS.iter().enumerate() {
        if keys.just_pressed(*a) || keys.just_pressed(*b) {
            actions.0.push(BarAction::Key { slot: k, how: format!("key {}", k + 1) });
        }
    }
    gui.panels.update(time.delta_secs_f64() * 1000.0);
    // Flashes that have played out (rule 33: once).
    let fx = gui.control(SLOT_FLASHES[0]).and_then(|c| match &c.kind {
        Kind::Text { pictures: Some(p), .. } => gui.tags.string(p.bitmap).map(str::to_string),
        _ => None,
    });
    let frames = fx.map_or(0, |f| gui.frames(&f)) as f64;
    // Blinks that have played out (rule 44).
    let now = gui.panels.now_ms;
    gui.blinks.retain(|_, start| now - *start < BLINK_MS * BLINKS as f64);
    let flame_frames = gui.flame_file().map_or(0, |f| gui.frames(&f)) as f64;
    if let Some(start) = gui.level_flame.filter(|start| (now - start) / (1000.0 / ANIMATION_FPS) >= flame_frames) {
        gui.level_flame = None;
        gui.note(format!("the level-up flame played out after {:.0} ms ({flame_frames} frames at {ANIMATION_FPS} per second, stand-in Q149 b)", now - start));
    }
    for k in 0..3 {
        if gui.flashes[k].is_some_and(|start| (gui.panels.now_ms - start) / (1000.0 / ANIMATION_FPS) >= frames) {
            gui.flashes[k] = None;
            let line = format!("{} number's flash played out ({frames} frames at {ANIMATION_FPS} per second, stand-in Q108)", Slot::ALL[k].name());
            gui.note(line);
        }
    }
    let [sw, sh] = gui.screen.map(|v| v as f32);
    // 4c-6: screen pixels to window pixels.
    let [fl, ft, fw, _] = frame.0;
    let k = fw / sw;
    let (ww, wh) = window.single().map_or((fw, frame.0[3]), |w| (w.width(), w.height()));
    let mut covered = Vec::new();
    // Each visible panel: what, left, top, width, height (screen pixels).
    let mut places: Vec<(Drawn, f32, f32, f32, f32)> = Vec::new();
    // The top of the highest bottom panel as they slide (the screen's
    // bottom when they are away): the caption stands on it (rule 51).
    // 4c-9, 4c-10 (the user's checks): the spinning mask stands on the
    // bottom bar, but not on the meters when they pop up alone.
    let mut bottom = sh;
    let mut bar_top = sh;
    for p in [Panel::BottomLeft, Panel::Slots, Panel::Meters] {
        if let Some(rect) = gui.rect(Drawn::Panel(p)) {
            let top = kq8_core::panels::place(p, rect, [sw, sh], gui.panels.away(p)).1;
            bottom = bottom.min(top);
            if p != Panel::Meters {
                bar_top = bar_top.min(top);
            }
        }
    }
    for (picture, mut node, image, mut vis) in &mut nodes {
        let d = DRAWN[picture.0];
        let Some(rect) = gui.rect(d) else { continue };
        let (away, panel) = match d {
            Drawn::Panel(p) => (gui.panels.away(p), Some(p)),
            Drawn::Status => (if gui.status.iter().any(Option::is_some) { 0.0 } else { 1.0 }, None),
        };
        // 4c-7 (the user's check in the original, Q111): the status
        // icons at their own size (not scaled with the screen), the
        // spinning mask in the screen's bottom right corner.
        // 4c-8: only the mask is drawn, at its own size (57 x 57).
        if panel.is_none() {
            if let Some(f) = gui.mask_file() {
                gui.frames(&f);
            }
        }
        let ((mut left, top), (w, h)) = match panel {
            Some(p) => (kq8_core::panels::place(p, rect, [sw, sh], away), (rect[2] - rect[0], rect[3] - rect[1])),
            // 4c-9 (the user's check): it rises and falls with the
            // bottom bar, on top of it. 5g-3: the status icons' picture,
            // its right edge at the screen's.
            None => {
                let [w, h] = STATUS_SIZE.map(|v| v as f32);
                ((sw - w, bar_top - h), (w, h))
            }
        };
        // 4c-11: with the corner panels on, the bottom bar's left panel at
        // the window's left edge, its right panel and the spinning mask
        // at the window's right edge (screen units; off the 4:3 screen on
        // a wide window). Clicks follow, as they use these places.
        if gui.corners {
            let (window_left, window_right) = (-fl / k, (ww - fl) / k);
            match panel {
                Some(Panel::BottomLeft) => left = window_left,
                Some(Panel::BottomRight) | None => left = window_right - w,
                _ => {}
            }
        }
        if away >= 1.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
        if panel.is_none() {
            let at = [left, top, left + w, top + h].map(|v| v.round() as i32);
            if *status_at != Some(at) {
                *status_at = Some(at);
                let line = format!("status icons at {at:?} of {sw}x{sh} (own size; at the right, on the bottom bar (not on the meters alone) or the screen's bottom: the user's checks, 4c-7 to 4c-10, Q111; 5g-3)");
                gui.note(line);
            }
        }
        node.left = Val::Px(fl + left * k);
        node.top = Val::Px(ft + top * k);
        node.width = Val::Px(w * k);
        node.height = Val::Px(h * k);
        if panel.is_some() {
            covered.push((d.layout_id(), [(fl + left * k) / ww, (ft + top * k) / wh, (fl + (left + w) * k) / ww, (ft + (top + h) * k) / wh]));
            places.push((d, left, top, w, h));
        }
        let Some((size, specs)) = gui.specs(d) else { continue };
        if gui.last.get(d.layout_id()) == Some(&specs) {
            continue;
        }
        let scale = if panel.is_some() { gui.scale } else { 1.0 };
        // 4c-9: drawn where it stands, so its rows match its place.
        let origin = match panel {
            Some(_) => gui.control(d.layout_id()).map_or([0, 0], |c| [c.rect[0], c.rect[1]]),
            None => [0, 0],
        };
        let canvas = gui.draw(origin, size, &specs, scale);
        // Where the drawn picture is see-through (the world shows), by
        // column and row ranges, for the log.
        let clear: Vec<(u32, u32)> = (0..canvas.height).flat_map(|y| (0..canvas.width).map(move |x| (x, y))).filter(|&(x, y)| canvas.at(x, y)[3] == 0).collect();
        if !clear.is_empty() && panel.is_some() {
            let xs = (clear.iter().map(|p| p.0).min().unwrap(), clear.iter().map(|p| p.0).max().unwrap());
            let ys = (clear.iter().map(|p| p.1).min().unwrap(), clear.iter().map(|p| p.1).max().unwrap());
            let line = format!("{} drawn {}x{}: {} see-through pixels, columns {}-{}, rows {}-{}", d.layout_id(), canvas.width, canvas.height, clear.len(), xs.0, xs.1, ys.0, ys.1);
            gui.note(line);
        }
        let mut picture = Image::new(Extent3d { width: canvas.width, height: canvas.height, depth_or_array_layers: 1 }, TextureDimension::D2, canvas.rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
        picture.sampler = ImageSampler::nearest();
        if let Some(mut slot) = images.get_mut(&image.image) {
            *slot = picture;
        }
        // M6 6d-3: each redraw of the map with its view and arrow.
        for s in &specs {
            if let Spec::Map { version, view, arrow, .. } = s {
                let arrow = arrow.as_ref().map_or("no arrow (Connor's pixel not known yet)".to_string(), |(_, frame, at)| format!("arrow frame {frame}, top-left ({:.1}, {:.1}) from the area's centre", at[0], at[1]));
                let line = format!("{} drawn: picture version {version}, view ({:.2}, {:.2})-({:.2}, {:.2}) from the area's centre, shown {:.1} x {:.1} layout px; {arrow} (magic-map.md rules 6, 18, 22)", d.layout_id(), view[0], view[1], view[2], view[3], view[2] - view[0], view[3] - view[1]);
                gui.note(line);
            }
        }
        gui.last.insert(d.layout_id(), specs);
    }
    cover.0 = covered;
    // The pointer: on the screen's 800 x 600 (or 640 x 480).
    let pointer = window.single().ok().and_then(|w| w.cursor_position());
    if let Some(c) = pointer {
        let (x, y) = ((c.x - fl) / k, (c.y - ft) / k);
        // interface.md rule 12: the top bar follows the pointer: down at
        // the very top row, up below y 60 of 480, not while the right
        // button is held or it is moving, not up in its popped time.
        let moving = gui.panels.moving(Panel::Top);
        if c.y < ft + 1.0 && !gui.panels.shown(Panel::Top) && !moving && !buttons.pressed(MouseButton::Right) {
            gui.panels.set(kq8_core::panels::Group::Top, true, "the pointer at the top row (rule 12)");
        } else if y > 60.0 / 480.0 * sh && gui.panels.shown(Panel::Top) && !moving && !gui.panels.top_popped() {
            gui.panels.set(kq8_core::panels::Group::Top, false, "the pointer below y 60 (rule 12)");
        }
        // inventory.md rules 6, 9, 10: clicks on the bars.
        for (button, left) in [(MouseButton::Left, true), (MouseButton::Right, false)] {
            if !buttons.just_pressed(button) {
                continue;
            }
            let hit = places.iter().find(|(_, l, t, w, h)| x >= *l && x < l + w && y >= *t && y < t + h).copied();
            let Some((d, l, t, ..)) = hit else { continue };
            let Some(control) = gui.control_at(d, l, t, x, y) else { continue };
            let how = format!("{} click on {control}", if left { "left" } else { "right" });
            match (control.as_str(), gui.item_at(&control)) {
                ("IDCTL_INVBAR_QUESTL", _) if left => actions.0.push(BarAction::Scroll { by: -1, how }),
                ("IDCTL_INVBAR_QUESTR", _) if left => actions.0.push(BarAction::Scroll { by: 1, how }),
                (MINIMAP, _) if left => gui.toggle_map(&how),
                (_, Some(item)) => actions.0.push(BarAction::Item { item, left, how }),
                _ => {
                    let line = format!("{how}: nothing there; nothing (inventory.md rule 10)");
                    gui.note(line);
                }
            }
        }
    }
    *caption = CaptionPlace { bottom: Some(bottom), map_right: Some(kq8_core::caption::map_away_right(sw)) };
    // The panels' sounds (rule 9): plain one-shots in group 0.
    let sounds = std::mem::take(&mut gui.panels.sounds);
    if let Some(mut sim) = sim {
        for s in sounds {
            sim.active.world.sounds.push(kq8_core::sound::Start::plain_one_shot(&s, "a panel starts to slide (interface.md rule 9)".into()));
        }
    }
    let lines = std::mem::take(&mut gui.panels.log);
    for line in lines {
        gui.note(line);
    }
}

/// 4c-4: the mouse pointer is the game's: the play pointer `curs04.pba`,
/// or the held item's cursor picture (`inventory.md` rule 9), as a
/// cursor image scaled like the screen (whole steps, nearest pixel).
/// STAND-IN (Q110): the click point is the picture's top-left corner.
pub fn pointer_draw(
    gui: Option<ResMut<Gui>>,
    window: Query<(Entity, &Window), With<bevy::window::PrimaryWindow>>,
    mut commands: Commands,
    mut images: ResMut<Assets<Image>>,
    mut shown: Local<Option<(String, u32)>>,
    frame: Res<ScreenFrame>,
) {
    use bevy::window::{CursorIcon, CustomCursor, CustomCursorImage};
    let Some(mut gui) = gui else { return };
    let Ok((entity, _)) = window.single() else { return };
    let step = ((frame.0[3] / gui.screen[1] as f32).round() as u32).max(1);
    let wanted = (gui.pointer.clone(), step);
    if shown.as_ref() == Some(&wanted) {
        return;
    }
    let Some(file) = gui.file_of(&gui.pointer.clone()) else {
        let line = format!("pointer {}: no file in the tag file; the system pointer stays", gui.pointer);
        gui.note(line);
        *shown = Some(wanted);
        return;
    };
    gui.load_file(&file);
    let Some(Some(frames)) = gui.files.get(&file.to_ascii_lowercase()) else {
        *shown = Some(wanted);
        return;
    };
    let Some(pic) = frames.first().and_then(Pic::of) else {
        *shown = Some(wanted);
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
            let [r, g, b] = gui.palette[index as usize];
            let at = ((y * pw + x) * 4) as usize;
            rgba[at..at + 4].copy_from_slice(&[r, g, b, 255]);
        }
    }
    let image = Image::new(Extent3d { width: pw, height: ph, depth_or_array_layers: 1 }, TextureDimension::D2, rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    let handle = images.add(image);
    commands.entity(entity).insert(CursorIcon::Custom(CustomCursor::Image(CustomCursorImage { handle, hotspot: (0, 0), ..default() })));
    let line = format!("pointer: {} ({file}, {}x{} drawn x{step}; click point at its top-left, stand-in Q110)", gui.pointer, pic.width, pic.height);
    gui.note(line);
    *shown = Some(wanted);
}

/// 4c-9: where the engine actually put the bottom panels, in physical
/// pixels after its rounding (left, top, right, bottom) and the size of
/// the picture shown in each; each change in `gui.log`.
pub fn gui_layout_log(
    gui: Option<ResMut<Gui>>,
    nodes: Query<(&GuiPicture, &bevy::ui::ComputedNode, &bevy::ui::UiGlobalTransform, &ImageNode, &Visibility)>,
    images: Res<Assets<Image>>,
    mut last: Local<HashMap<&'static str, [i32; 6]>>,
) {
    let Some(mut gui) = gui else { return };
    for (picture, computed, at, image, vis) in &nodes {
        let d = DRAWN[picture.0];
        if !matches!(d, Drawn::Panel(Panel::BottomLeft | Panel::Slots | Panel::Meters | Panel::BottomRight)) || *vis == Visibility::Hidden {
            continue;
        }
        let size = computed.size();
        let centre = at.affine().translation;
        let edges = [centre.x - size.x / 2.0, centre.y - size.y / 2.0, centre.x + size.x / 2.0, centre.y + size.y / 2.0];
        let texture = images.get(&image.image).map_or([0, 0], |i| [i.width(), i.height()]);
        let key = [edges[0].round() as i32, edges[1].round() as i32, edges[2].round() as i32, edges[3].round() as i32, texture[0] as i32, texture[1] as i32];
        if last.get(d.layout_id()) == Some(&key) {
            continue;
        }
        last.insert(d.layout_id(), key);
        let line = format!(
            "engine: {} at left {:.2} top {:.2} right {:.2} bottom {:.2} (physical px, {:.2}x{:.2}, unrounded {:.2}x{:.2}); picture {}x{}",
            d.layout_id(),
            edges[0],
            edges[1],
            edges[2],
            edges[3],
            size.x,
            size.y,
            computed.unrounded_size.x,
            computed.unrounded_size.y,
            texture[0],
            texture[1]
        );
        gui.note(line);
    }
}
