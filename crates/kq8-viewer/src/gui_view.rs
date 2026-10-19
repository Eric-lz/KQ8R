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

/// The pictures drawn (4c-2; the top bar since 4c-3).
const DRAWN: [Drawn; 6] = [Drawn::Panel(Panel::Top), Drawn::Panel(Panel::BottomLeft), Drawn::Panel(Panel::Slots), Drawn::Panel(Panel::Meters), Drawn::Panel(Panel::BottomRight), Drawn::Status];

/// The armour, hand and ranged slot pictures, their numbers and their
/// flashes, by layout name (rules 33, 34, 36), in [`Slot::ALL`] order.
const SLOT_PICTURES: [&str; 3] = ["IDCTL_INVBAR_ARMOR", "IDCTL_INVBAR_SHORT", "IDCTL_INVBAR_LONG"];
const SLOT_NUMBERS: [&str; 3] = ["IDCTL_INVBAR_ARMORTEXT", "IDCTL_INVBAR_SHORTTEXT", "IDCTL_INVBAR_LONGTEXT"];
const SLOT_FLASHES: [&str; 3] = ["IDCTL_INVBAR_ARMOR_FX", "IDCTL_INVBAR_SHORT_FX", "IDCTL_INVBAR_LONG_FX"];

/// One thing to draw, before its pictures and fonts are looked up.
#[derive(Debug, Clone, PartialEq)]
enum Spec {
    Pic { rect: [i32; 4], file: String, frame: usize, cut: Option<u32> },
    Text { rect: [i32; 4], font: String, justification: u32, text: String },
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
    /// The bottom-bar key (`movekeys.kq` row 12, rule 10).
    space_key: Option<KeyCode>,
    /// Connor's stats events already sent (rule 24).
    stats_events: Option<u64>,
    hands_off: bool,
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
    let pal_names = [format!("{world}/resource/{world}.ppl"), format!("GAME/Resource/{world}.ppl")];
    let Some((pal_path, pal_bytes)) = first_found(game, &pal_names) else { return fail(main, format!("no palette among {pal_names:?}")) };
    let palette = match kq8_data::Palette::parse(&pal_bytes) {
        Ok(p) => p.shown_colors(),
        Err(e) => return fail(main, format!("{pal_path}: {e}")),
    };
    let wide = crate::talk_view::set800mode(game);
    let screen = kq8_core::caption::screen(wide);
    let scale = kq8_core::caption::scale(screen);
    // Rule 10: the bottom-bar key is row 12 of movekeys.kq.
    let (space_key, key_line) = match first_found(game, &["movekeys.kq".into()]).map(|(_, b)| kq8_data::kq_text::parse(&b)) {
        Some(file) => {
            let s = file.first();
            let char12 = s.and_then(|s| s.number("KeyChar12")).and_then(Result::ok).map(|v| v as i64);
            let word12 = s.and_then(|s| s.get("KeyWord12")).unwrap_or("?").to_string();
            let key = char12.and_then(virtual_key);
            (key, format!("movekeys.kq row 12: KeyChar12={char12:?} KeyWord12={word12} -> {key:?}"))
        }
        None => (Some(KeyCode::Space), "no movekeys.kq: Space (the manual's default)".to_string()),
    };
    let mut log = match Log::create("gui") {
        Ok(l) => Some(l),
        Err(e) => {
            main.summary(format!("gui: cannot create gui.log: {e}; panels still drawn, not logged"));
            None
        }
    };
    let head = format!(
        "gui: Gplayscr.gui read; screen {}x{} (set800mode {wide}, layout x {scale}; stand-in Q108); palette {pal_path}; the bottom-bar key: {key_line} (interface.md rule 10)",
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
        space_key,
        stats_events: None,
        hands_off: false,
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
            Some((_, Ok(v))) => Some(v),
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
                    if let Some(file) = file {
                        out.push(Spec::Pic { rect, file, frame: 0, cut });
                    }
                }
                Kind::Text { text, pictures: None } => {
                    // Rules 23, 36, 39, 43, 45: the numbers we set; the
                    // layout's own placeholder texts ("0", "3Dfx") are not
                    // drawn, nor the red alternative numbers (rule 32, M5).
                    // Counts of 0 are not written (stand-in, Q109).
                    let count = |c: Option<i64>| c.filter(|n| *n > 0).map(|n| n.to_string());
                    let value = match (slot(&SLOT_NUMBERS), name.as_str()) {
                        (Some(k), _) => self.meters.numbers.map(|n| n[k].to_string()),
                        (None, "IDCTL_METERS_LEVEL") => Some(self.meters.level.to_string()),
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
                    } else if name == "IDCTL_TIMER_7" && self.panels.mask_icon {
                        // Rule 15: the spinning mask, looping.
                        out.push(Spec::Pic { rect, file, frame: (now / frame_ms) as usize, cut: None });
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
    fn draw(&mut self, size: [i32; 2], specs: &[Spec]) -> gui_draw::Canvas {
        for s in specs {
            match s {
                Spec::Pic { file, .. } => self.load_file(file),
                Spec::Text { font, .. } => self.load_font(font),
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
                Spec::Text { rect, font, justification, text } => {
                    let Some(Some(f)) = self.fonts.get(&font.to_ascii_lowercase()) else { continue };
                    pieces.push(Piece::Text { rect: *rect, font: f, justification: *justification, text: text.clone() });
                }
            }
        }
        gui_draw::draw(size, self.scale, &pieces, &self.palette)
    }

    /// How many frames a `.pba` has (0 if not loaded).
    fn frames(&mut self, file: &str) -> usize {
        self.load_file(file);
        self.files.get(&file.to_ascii_lowercase()).and_then(|f| f.as_ref()).map_or(0, Vec::len)
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
            ImageNode::new(image),
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
            k if k.len() == 1 && ("1"..="8").contains(&k) => actions.0.push(BarAction::Key { slot: k.parse::<usize>().unwrap_or(1) - 1, how }),
            // A click on a control of the bars: left:NAME / right:NAME.
            k if k.starts_with("left:") || k.starts_with("right:") => {
                let (button, control) = name.split_once(':').unwrap_or_default();
                let control = control.to_ascii_uppercase();
                let left = button.eq_ignore_ascii_case("left");
                match (control.as_str(), gui.item_at(&control)) {
                    ("IDCTL_INVBAR_QUESTL", _) if left => actions.0.push(BarAction::Scroll { by: -1, how }),
                    ("IDCTL_INVBAR_QUESTR", _) if left => actions.0.push(BarAction::Scroll { by: 1, how }),
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
    let dead = false;
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
                Some(name) if !dead => {
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
    if hands_off != gui.hands_off {
        gui.hands_off = hands_off;
        gui.panels.hands_off(hands_off);
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
        }
    }
    // Rules 34-35: the slot pictures follow what Connor wears.
    let (types, why) = kq8_core::inventory::slot_types(&sim.active.world.items, "Connor");
    // 4c-4: the item in hand and the pointer (inventory.md rules 7, 9).
    let hand = kq8_core::connor_inventory::hand(&mut sim.active.world);
    let pointer = hand
        .as_ref()
        .and_then(|(_, kind)| sim.active.world.items.kind(kind).and_then(|t| t.cursor.clone()))
        .unwrap_or_else(|| PLAY_POINTER.to_string());
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
    let sent = gui.meters.send(&stats, false);
    let width = |name: &str| {
        gui.control(name).map_or(0, |c| (gui_draw::scaled(c.rect[2], gui.scale) - gui_draw::scaled(c.rect[0], gui.scale)).max(0) as u32)
    };
    let (hw, ew) = (width("IDCTL_METERS_HEALTH"), width("IDCTL_METERS_EXP"));
    let exp = match sent.experience_percent {
        Some(p) => format!("{p} % (bar {} of {ew} px)", meters::bar_width(ew, p)),
        None => "not sent (unchanged, rule 22)".into(),
    };
    let line = format!(
        "update {updates}: {}: Connor {:?} level {}: sent armour {}, hand {}, ranged {}, health {} % (bar {} of {hw} px), experience {exp}, level {}; flashes {:?} (interface.md rules 20-24, 33)",
        if first { "the screen opens".to_string() } else { format!("stats event #{events}") },
        stats.stats.0,
        stats.level,
        sent.numbers[0],
        sent.numbers[1],
        sent.numbers[2],
        sent.health_percent,
        meters::bar_width(hw, sent.health_percent),
        sent.level,
        sent.flashes.iter().map(|s| s.name()).collect::<Vec<_>>()
    );
    gui.note(&line);
    shared.log.lock().unwrap().detail(format!("gui: {line}"));
    let now = gui.panels.now_ms;
    for s in &sent.flashes {
        let k = Slot::ALL.iter().position(|x| x == s).expect("listed");
        gui.flashes[k] = Some(now);
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
) {
    let Some(mut gui) = gui else { return };
    let gui = &mut *gui;
    if gui.space_key.is_some_and(|k| keys.just_pressed(k)) {
        gui.panels.toggle_bottom("Space key");
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
    let mut bottom = sh;
    for (picture, mut node, image, mut vis) in &mut nodes {
        let d = DRAWN[picture.0];
        let Some(rect) = gui.rect(d) else { continue };
        let (away, panel) = match d {
            Drawn::Panel(p) => (gui.panels.away(p), Some(p)),
            Drawn::Status => (if gui.panels.mask_icon { 0.0 } else { 1.0 }, None),
        };
        let (left, top) = match panel {
            Some(p) => kq8_core::panels::place(p, rect, [sw, sh], away),
            None => (rect[0], rect[1]),
        };
        let (w, h) = (rect[2] - rect[0], rect[3] - rect[1]);
        if matches!(panel, Some(Panel::BottomLeft | Panel::Slots | Panel::Meters)) {
            bottom = bottom.min(top);
        }
        if away >= 1.0 {
            *vis = Visibility::Hidden;
            continue;
        }
        *vis = Visibility::Inherited;
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
        let canvas = gui.draw(size, &specs);
        let mut picture = Image::new(Extent3d { width: canvas.width, height: canvas.height, depth_or_array_layers: 1 }, TextureDimension::D2, canvas.rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
        picture.sampler = ImageSampler::nearest();
        if let Some(mut slot) = images.get_mut(&image.image) {
            *slot = picture;
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
