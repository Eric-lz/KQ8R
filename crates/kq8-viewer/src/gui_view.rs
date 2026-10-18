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
//! The screen is 800 × 600 (`set800mode 1`) and stretched to the window
//! (STAND-IN, Q108); the panels' timers count real time.

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

/// The pictures drawn in 4c-2.
const DRAWN: [Drawn; 5] = [Drawn::Panel(Panel::BottomLeft), Drawn::Panel(Panel::Slots), Drawn::Panel(Panel::Meters), Drawn::Panel(Panel::BottomRight), Drawn::Status];

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

/// Where the caption box goes (`interface.md` rule 51), screen pixels:
/// the top of the highest bottom panel; the map's right edge.
#[derive(Resource, Default, Clone, Copy)]
pub struct CaptionPlace {
    pub bottom: Option<f32>,
    pub map_right: Option<f32>,
}

/// `--key NAME,N` (4c-2): a key pressed before update N + 1 (`space`).
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

    /// What drawn picture `d` is made of now (its layout size and the
    /// pieces), or `None` if its block is missing.
    fn specs(&self, d: Drawn) -> Option<([i32; 2], Vec<Spec>)> {
        let dialog = self.control(d.layout_id())?;
        let [l, t, r, b] = dialog.rect;
        let now = self.panels.now_ms;
        let frame_ms = 1000.0 / ANIMATION_FPS;
        let mut out = Vec::new();
        for c in &dialog.controls {
            let name = self.name_of(c);
            let slot = |list: &[&str; 3]| list.iter().position(|n| *n == name);
            match &c.kind {
                Kind::Bitmap(p) => {
                    // Rule 34: the worn item's picture, else the layout's
                    // empty one.
                    let file = match slot(&SLOT_PICTURES) {
                        Some(k) => self.slot_files[k].clone().or_else(|| self.tags.string(p.bitmap).map(str::to_string)),
                        None => self.tags.string(p.bitmap).map(str::to_string),
                    };
                    // Rule 20: the meters' bars cut to a width.
                    let full = (gui_draw::scaled(c.rect[2], self.scale) - gui_draw::scaled(c.rect[0], self.scale)).max(0) as u32;
                    let cut = match name.as_str() {
                        "IDCTL_METERS_HEALTH" => Some(meters::bar_width(full, self.meters.health_percent)),
                        "IDCTL_METERS_EXP" => Some(meters::bar_width(full, self.meters.experience_percent)),
                        _ => None,
                    };
                    if let Some(file) = file {
                        out.push(Spec::Pic { rect: c.rect, file, frame: 0, cut });
                    }
                }
                Kind::Text { text, pictures: None } => {
                    // Rules 23, 36: the numbers we set; the layout's own
                    // placeholder texts ("0", "3Dfx") are not drawn, nor
                    // the red alternative numbers (rule 32, M5).
                    let value = match (slot(&SLOT_NUMBERS), name.as_str()) {
                        (Some(k), _) => self.meters.numbers.map(|n| n[k].to_string()),
                        (None, "IDCTL_METERS_LEVEL") => Some(self.meters.level.to_string()),
                        _ => None,
                    };
                    if let (Some(value), Some(font)) = (value, self.tags.string(text.font)) {
                        out.push(Spec::Text { rect: c.rect, font: font.to_string(), justification: text.justification, text: value });
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
                        out.push(Spec::Pic { rect: c.rect, file, frame: (now / frame_ms) as usize, cut: None });
                    }
                }
                _ => {}
            }
        }
        Some(([r - l, b - t], out))
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
pub fn gui_step(gui: Option<ResMut<Gui>>, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>, mut key_at: ResMut<KeyAt>, shared: Res<Shared>) {
    let (Some(mut gui), Some(mut sim)) = (gui, sim) else { return };
    let gui = &mut *gui;
    let updates = sim.active.world.updates;
    // Test keys (`--key NAME,N`).
    let (due, later): (Vec<_>, Vec<_>) = std::mem::take(&mut key_at.0).into_iter().partition(|(_, n)| updates >= *n);
    key_at.0 = later;
    for (name, _) in due {
        match name.to_ascii_lowercase().as_str() {
            "space" => gui.panels.toggle_bottom(&format!("test --key space before update {updates}")),
            other => {
                let line = format!("update {updates}: test --key {other}: not a panel key in this step; nothing");
                gui.note(line);
            }
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
    mut nodes: Query<(&GuiPicture, &mut Node, &ImageNode, &mut Visibility)>,
    mut images: ResMut<Assets<Image>>,
    mut cover: ResMut<GuiCover>,
    mut caption: ResMut<CaptionPlace>,
) {
    let Some(mut gui) = gui else { return };
    let gui = &mut *gui;
    if gui.space_key.is_some_and(|k| keys.just_pressed(k)) {
        gui.panels.toggle_bottom("Space key");
    }
    gui.panels.update(time.delta_secs_f64() * 1000.0);
    // Flashes that have played out (rule 33: once).
    let fx = gui.control(SLOT_FLASHES[0]).and_then(|c| match &c.kind {
        Kind::Text { pictures: Some(p), .. } => gui.tags.string(p.bitmap).map(str::to_string),
        _ => None,
    });
    let frames = fx.map_or(0, |f| gui.frames(&f)) as f64;
    for k in 0..3 {
        if gui.flashes[k].is_some_and(|start| (gui.panels.now_ms - start) / (1000.0 / ANIMATION_FPS) >= frames) {
            gui.flashes[k] = None;
            let line = format!("{} number's flash played out ({frames} frames at {ANIMATION_FPS} per second, stand-in Q108)", Slot::ALL[k].name());
            gui.note(line);
        }
    }
    let [sw, sh] = gui.screen.map(|v| v as f32);
    let mut covered = Vec::new();
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
        node.left = Val::Percent(100.0 * left / sw);
        node.top = Val::Percent(100.0 * top / sh);
        node.width = Val::Percent(100.0 * w / sw);
        node.height = Val::Percent(100.0 * h / sh);
        if panel.is_some() {
            covered.push((d.layout_id(), [left / sw, top / sh, (left + w) / sw, (top + h) / sh]));
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
