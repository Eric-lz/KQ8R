//! Remarks and speech in the viewer (M4 step 4a, docs/DESIGN.md; rules:
//! docs/behaviour/talking.md).
//!
//! 4a-2: [`load_talk`] reads the message files of the world's `English`
//! folder, then `GAME/ENGLISH` (rule 1: the world's first, our reading of
//! the data), `TALKERS.SH`, and gives the world's rules a way to find a
//! line's voice and lip-sync files next to its message file (rule 4).
//! The queues themselves run in `kq8_core::talk`, inside the door
//! simulation; their lines go to `kq8-artifacts/logs/talk.log`.
//!
//! 4a-3: [`talk_step`] turns the talk events into voices for
//! `sound_view` (group 3; Connor's plain, others placed at the speaker,
//! 2,000 / 30,000) and the ducking switch (groups 0, 2, 5 x 0.2 while a
//! line without `nofade` plays; `sound.md` rules 53, 58).
//!
//! 4a-4: the caption panel ([`setup_caption`], [`caption_draw`]): the
//! line's text (stage directions taken out) at the bottom of the screen
//! while its queue is talking, if `Options::Captions` is true (rule 10).
//! STAND-IN: how the original draws the panel (font, place, wrapping) is
//! not studied (`talking.md` open question 4, M4 GUI): Bevy's own font,
//! white on a dark band, wrapped to 80% of the window.
//!
//! 4b-3: the caption is drawn as the original's (`kq8_core::caption`):
//! the world's `console.pft` font over the `stontran.pbm` stone strip,
//! both named by the layout `IDDLG_CAPTION` in `Gplayscr.gui`, coloured
//! by the world's palette (STAND-IN, Q106), centred, at the bottom of the
//! window, growing upwards one 16-pixel line at a time (the user's check
//! in the original); wrapped at `WRAP_WIDTH` (STAND-IN, Q107 a). The
//! 640x480 picture is stretched to the window. Without those files, the
//! 4a-4 text panel above.
//!
//! 4a-6: the mouth ([`mouth_draw`]): Connor's face texture (`baseLip`)
//! becomes `lip<n>` of his `lips=` list at each mouth change, `lip0` for
//! shapes of 8 or more and at rest (rule 18). Other characters are not
//! drawn yet: their changes are only in `talk.log`.

use bevy::prelude::*;
use kq8_core::talk::{Book, Talk, TalkEvent, Voice, CONNOR};
use kq8_data::msg::{parse_lips, MessageFile, Talkers, Tuple};
use kq8_data::{aud, GameDir};

use crate::door_view::DoorSim;
use crate::log::Log;

/// The message files, talkers and voice lookup for `world`, and the
/// `set` lines of `MASK/Options.cs` (the console variables it sets, e.g.
/// `Options::Captions`, rule 10).
pub fn load_talk(game: &GameDir, world: &str, log: &mut Log) -> (Talk, Vec<(String, String)>) {
    let mut book = Book::default();
    for folder in [format!("{world}/English"), "GAME/ENGLISH".to_string()] {
        let Some(dir) = game.find(&folder) else {
            log.summary(format!("talk: no folder {folder}"));
            continue;
        };
        let rel = dir.strip_prefix(game.root()).unwrap_or(&dir).to_string_lossy().replace('\\', "/");
        let mut names: Vec<_> = std::fs::read_dir(&dir).map(|d| d.filter_map(|e| e.ok()).map(|e| e.path()).collect()).unwrap_or_default();
        names.sort();
        for path in names {
            let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else { continue };
            let is_msg = path.extension().is_some_and(|e| e.eq_ignore_ascii_case("msg"));
            let Some(number) = stem.parse::<u16>().ok().filter(|_| is_msg) else { continue };
            match std::fs::read(&path).map_err(|e| e.to_string()).and_then(|b| MessageFile::parse(number, &b)) {
                Ok(m) => {
                    let lines = m.lines.len();
                    let added = book.add(&rel, m, number);
                    log.summary(format!("talk: {rel}/{stem}.MSG: {lines} lines{}", if added { "" } else { " (number already read from another folder: not used)" }));
                }
                Err(e) => log.summary(format!("talk: {rel}/{stem}.MSG: {e}; not used")),
            }
        }
    }
    match game.find("GAME/ENGLISH/TALKERS.SH").map(std::fs::read) {
        Some(Ok(b)) => book.talkers = Talkers::parse(&b),
        _ => log.summary("talk: GAME/ENGLISH/TALKERS.SH not read: every line is spoken by its issuer"),
    }
    log.summary(format!("talk: {} message files, {} talkers", book.files.len(), book.talkers.names.len()));
    let mut talk = Talk::default();
    talk.book = book;
    let g = game.clone();
    talk.voices = Some(Box::new(move |folder: &str, t: Tuple| voice(&g, folder, t)));
    let sets = match game.find("Options.cs").map(std::fs::read) {
        Some(Ok(b)) => options_sets(&b),
        _ => Vec::new(),
    };
    for (k, v) in &sets {
        log.summary(format!("talk: Options.cs sets {k} = '{v}'"));
    }
    (talk, sets)
}

/// A line's voice file and its lip-sync file (rules 4, 17), both next to
/// its message file; `None` without a readable voice file.
fn voice(game: &GameDir, folder: &str, t: Tuple) -> Option<Voice> {
    let path = game.find(&format!("{folder}/{}", t.voice_name()))?;
    let sound = std::fs::read(&path).ok().and_then(|b| aud::parse(&b).ok())?;
    let rel = path.strip_prefix(game.root()).unwrap_or(&path).to_string_lossy().replace('\\', "/");
    let mouths = game.find(&format!("{folder}/{}", t.lip_name())).and_then(|p| std::fs::read(p).ok()).and_then(|b| parse_lips(&b).ok()).unwrap_or_default();
    Some(Voice { path: rel, seconds: sound.seconds(), mouths })
}

/// The `set <name> <value…>` lines of a console script such as
/// `Options.cs` (the value is the rest of the line, words joined by one
/// space, as the script runner's `set`).
pub fn options_sets(bytes: &[u8]) -> Vec<(String, String)> {
    let text: String = bytes.iter().map(|&b| b as char).collect();
    text.lines()
        .filter_map(|line| {
            let mut w = line.split('#').next().unwrap_or("").split_whitespace();
            (w.next()? == "set").then_some(())?;
            let name = w.next()?.to_string();
            Some((name, w.collect::<Vec<_>>().join(" ")))
        })
        .collect()
}

/// What the talk events ask of the sound, the captions and the mouths
/// (M4 4a-3 on), filled each update by [`talk_step`].
#[derive(Resource, Default)]
pub struct Speech {
    /// Voices to start or stop, for `sound_view`.
    pub voices: Vec<VoiceOrder>,
    /// A line that turns other sounds down is playing (rules 10-11).
    pub ducking: bool,
    /// The caption shown (4a-4): the issuer whose queue showed it, and
    /// the text; `None`: the panel is hidden (rule 11).
    pub caption: Option<(String, String)>,
    /// Mouth changes not drawn yet (4a-6): speaker, shape (0 at a line's
    /// end: rest).
    pub mouths: Vec<(String, u16)>,
}

/// A voice for the sound system.
#[derive(Debug, Clone, PartialEq)]
pub enum VoiceOrder {
    /// Line `line`'s voice file `path`: plain (`at` `None`, Connor's) or
    /// placed at the speaker (rule 10).
    Start { line: u64, path: String, at: Option<[f32; 3]>, why: String },
    /// Cut short (`clearSpeak`, rule 15).
    Stop { line: u64 },
}

/// After the scripts and the traps of an update: the talk events become
/// orders for the sound (4a-3). Logged to `talk.log` through the door
/// simulation's log lines.
pub fn talk_step(sim: Option<ResMut<DoorSim>>, mut speech: ResMut<Speech>, shared: Res<crate::Shared>) {
    let Some(mut sim) = sim else { return };
    let events = std::mem::take(&mut sim.active.world.talk.events);
    // The update these events belong to (the counter has moved on).
    let now = sim.active.world.updates.saturating_sub(1) as f64 / kq8_core::UPDATES_PER_SECOND;
    for e in events {
        match e.clone() {
            TalkEvent::Start { id, speaker, voice: Some(path), tuple, .. } => {
                // Rule 10: Connor's plain, anyone else's placed where the
                // speaker is when the line starts.
                let at = if speaker.eq_ignore_ascii_case(CONNOR) { None } else { sim.active.world.object_point(&speaker) };
                let line = match (speaker.eq_ignore_ascii_case(CONNOR), at) {
                    (true, _) => format!("line #{id}: voice {path} plain (Connor's)"),
                    (false, Some(p)) => format!("line #{id}: voice {path} placed at {speaker} ({:.0}, {:.0}, {:.0})", p[0], p[1], p[2]),
                    (false, None) => format!("line #{id}: voice {path}: {speaker} has no place: plain (our choice)"),
                };
                sim.active.world.talk.log.push(format!("t {now:8.3}: {line}"));
                speech.voices.push(VoiceOrder::Start { line: id, path, at, why: format!("speech {tuple} line #{id} by {speaker}") });
            }
            TalkEvent::End { id, cut: true, .. } => speech.voices.push(VoiceOrder::Stop { line: id }),
            _ => {}
        }
        // 4a-6, rules 11, 17: mouth changes; the end of a line rests it.
        match &e {
            TalkEvent::Mouth { speaker, shape, .. } => speech.mouths.push((speaker.clone(), *shape)),
            TalkEvent::End { speaker, .. } => speech.mouths.push((speaker.clone(), 0)),
            _ => {}
        }
        // 4a-4, rules 10-11: a line with a caption shows it; the queue
        // that showed it running empty hides it.
        match e {
            TalkEvent::Start { issuer, caption: Some(text), .. } => speech.caption = Some((issuer, text)),
            TalkEvent::Empty { issuer } if speech.caption.as_ref().is_some_and(|(i, _)| *i == issuer) => speech.caption = None,
            _ => {}
        }
    }
    speech.ducking = sim.active.world.talk.ducking();
    sim.flush_talk(&shared);
}

/// The caption panel (4a-4).
#[derive(Component)]
pub struct CaptionPanel;

/// Its text.
#[derive(Component)]
pub struct CaptionText;

/// Spawns the hidden caption panel at the bottom of the window.
pub fn setup_caption(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Percent(10.0),
                width: Val::Percent(80.0),
                justify_content: JustifyContent::Center,
                padding: UiRect::all(Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.6)),
            Visibility::Hidden,
            CaptionPanel,
        ))
        .with_children(|p| {
            p.spawn((Text::new(""), TextFont { font_size: FontSize::Px(20.0), ..default() }, TextColor(Color::WHITE), CaptionText));
        });
}

/// What the caption picture is made from (4b-3).
#[derive(Resource)]
pub struct CaptionStyle {
    font: kq8_data::font::Font,
    /// The strip picture: width, height, palette indices.
    strip: Option<(u32, u32, Vec<u8>)>,
    palette: [kq8_data::palette::Rgb; 256],
    caption_box: kq8_core::caption::Box,
}

/// The first of `names` (paths under the install) that exists.
fn first_found(game: &GameDir, names: &[String]) -> Option<(String, Vec<u8>)> {
    names.iter().find_map(|n| Some((n.clone(), std::fs::read(game.find(n)?).ok()?)))
}

/// 4b-3: reads the caption box of `GAME/Resource/Gplayscr.gui`
/// (`IDDLG_CAPTION` by the tag file), the font and strip picture it
/// names (the world's `8gui` folder first, then `GAME/8Gui`), and the
/// world's palette (`<world>/resource/<world>.ppl`, then
/// `GAME/Resource/<world>.ppl`). `None` (logged) if any is missing.
pub fn load_caption(game: &GameDir, world: &str, log: &mut Log) -> Option<CaptionStyle> {
    use kq8_data::gui::{self, Kind, Tags};
    fn fail(log: &mut Log, why: String) -> Option<CaptionStyle> {
        log.summary(format!("caption: {why}; the 4a-4 text panel is used"));
        None
    }
    let Some((_, tag_bytes)) = first_found(game, &["GAME/Resource/kqGuiTag.TTAG".into()]) else { return fail(log, "no kqGuiTag.TTAG".into()) };
    let (tags, _) = Tags::parse(&String::from_utf8_lossy(&tag_bytes));
    let Some((_, gui_bytes)) = first_found(game, &["GAME/Resource/Gplayscr.gui".into()]) else { return fail(log, "no Gplayscr.gui".into()) };
    let layout = match gui::parse(&gui_bytes) {
        Ok(l) => l,
        Err(e) => return fail(log, format!("Gplayscr.gui: {e}")),
    };
    let Some(id) = tags.id_of("IDDLG_CAPTION") else { return fail(log, "no IDDLG_CAPTION tag".into()) };
    let Some(c) = layout.find(id as u32) else { return fail(log, "no IDDLG_CAPTION in Gplayscr.gui".into()) };
    let Kind::Dialog { caption: Some(cap), picture } = &c.kind else { return fail(log, "IDDLG_CAPTION is not a caption dialog".into()) };
    let Some(font_name) = tags.string(cap.text.font) else { return fail(log, format!("font tag {} has no file name", cap.text.font)) };
    let in_world = |name: &str| vec![format!("{world}/8gui/{name}"), format!("GAME/8Gui/{name}"), format!("GAME/Resource/{name}")];
    let Some((font_path, font_bytes)) = first_found(game, &in_world(font_name)) else { return fail(log, format!("font {font_name} not found")) };
    let font = match kq8_data::font::Font::parse(&font_bytes) {
        Ok(f) => f,
        Err(e) => return fail(log, format!("{font_path}: {e}")),
    };
    let strip_name = picture.and_then(|p| tags.string(p.bitmap));
    let strip = strip_name.and_then(|n| first_found(game, &in_world(n))).and_then(|(path, b)| match kq8_data::Bitmap::parse(&b) {
        Ok(bm) if bm.bit_count == 8 => Some((path, (bm.width, bm.height, bm.pixels))),
        Ok(_) => None,
        Err(e) => {
            log.summary(format!("caption: {path}: {e}; no strip"));
            None
        }
    });
    let pal_names = [format!("{world}/resource/{world}.ppl"), format!("GAME/Resource/{world}.ppl")];
    let Some((pal_path, pal_bytes)) = first_found(game, &pal_names) else { return fail(log, format!("no palette among {pal_names:?}")) };
    let palette = match kq8_data::Palette::parse(&pal_bytes) {
        Ok(p) => p.shown_colors(),
        Err(e) => return fail(log, format!("{pal_path}: {e}")),
    };
    let [l, t, r, b] = c.rect;
    let caption_box = kq8_core::caption::Box { line_height: (b - t).max(1) as u32, text_y: cap.y.max(0) as u32 };
    log.summary(format!(
        "caption: IDDLG_CAPTION ({l}, {t})-({r}, {b}), caption y {}, font {font_path} (height {}), strip {}, palette {pal_path} (stand-in, Q106); line {} px, wrap at {} px (stand-in, Q107 a)",
        cap.y,
        font.char_height,
        strip.as_ref().map_or("none".to_string(), |(p, (w, h, _))| format!("{p} {w}x{h}")),
        caption_box.line_height,
        kq8_core::caption::WRAP_WIDTH
    ));
    Some(CaptionStyle { font, strip: strip.map(|(_, s)| s), palette, caption_box })
}

/// The caption picture's node (4b-3).
#[derive(Component)]
pub struct CaptionPicture;

/// Spawns the hidden caption picture at the bottom of the window.
pub fn setup_caption_picture(mut commands: Commands, style: Option<Res<CaptionStyle>>) {
    if style.is_none() {
        return;
    }
    commands.spawn((
        Node { position_type: PositionType::Absolute, bottom: Val::Px(0.0), left: Val::Px(0.0), width: Val::Percent(100.0), height: Val::Percent(0.0), ..default() },
        ImageNode::default(),
        Visibility::Hidden,
        CaptionPicture,
    ));
}

/// 4b-3: draws the caption picture when the caption changed; each change
/// in `viewer.log` with its lines and their widths.
pub fn caption_picture_draw(
    speech: Res<Speech>,
    style: Option<Res<CaptionStyle>>,
    mut node: Query<(&mut Node, &mut ImageNode, &mut Visibility), With<CaptionPicture>>,
    mut images: ResMut<Assets<Image>>,
    mut last: Local<Option<String>>,
    shared: Res<crate::Shared>,
) {
    use bevy::asset::RenderAssetUsages;
    use bevy::image::ImageSampler;
    use bevy::render::render_resource::{Extent3d, TextureDimension, TextureFormat};
    use kq8_core::caption;
    let Some(style) = style else { return };
    // The speech changes every frame (the mouths): only a new caption
    // text makes a new picture.
    let shown = speech.caption.as_ref().map(|(_, t)| t.clone());
    if !speech.is_changed() || *last == shown {
        return;
    }
    *last = shown;
    let Ok((mut n, mut image, mut vis)) = node.single_mut() else { return };
    let Some(text) = speech.caption.as_ref().map(|(_, t)| t.as_str()) else {
        if *vis != Visibility::Hidden {
            *vis = Visibility::Hidden;
            shared.log.lock().unwrap().detail("caption picture hidden");
        }
        return;
    };
    let lines = caption::wrap(&style.font, text, caption::WRAP_WIDTH);
    let strip = style.strip.as_ref().map(|(w, h, p)| caption::Strip { width: *w, height: *h, pixels: p });
    let (w, h, rgba) = caption::picture(&style.font, strip.as_ref(), &style.palette, &lines, style.caption_box);
    if h == 0 {
        *vis = Visibility::Hidden;
        return;
    }
    let mut picture = Image::new(Extent3d { width: w, height: h, depth_or_array_layers: 1 }, TextureDimension::D2, rgba, TextureFormat::Rgba8UnormSrgb, RenderAssetUsages::default());
    picture.sampler = ImageSampler::nearest();
    image.image = images.add(picture);
    n.height = Val::Percent(100.0 * h as f32 / caption::SCREEN[1] as f32);
    *vis = Visibility::Inherited;
    let widths: Vec<u32> = lines.iter().map(|l| caption::text_width(&style.font, l)).collect();
    shared.log.lock().unwrap().detail(format!("caption picture shown: {} line(s) {lines:?}, widths {widths:?} px (wrap {}), {w}x{h} of 640x480", lines.len(), caption::WRAP_WIDTH));
}

/// Shows or hides the caption when it changed; each change in
/// `viewer.log`. With the original's caption picture (4b-3) this text
/// panel stays hidden.
pub fn caption_draw(
    speech: Res<Speech>,
    mut panel: Query<&mut Visibility, With<CaptionPanel>>,
    mut text: Query<&mut Text, With<CaptionText>>,
    style: Option<Res<CaptionStyle>>,
    shared: Res<crate::Shared>,
) {
    if !speech.is_changed() || style.is_some() {
        return;
    }
    let shown = speech.caption.as_ref().map(|(_, t)| t.as_str());
    let (Ok(mut vis), Ok(mut t)) = (panel.single_mut(), text.single_mut()) else { return };
    let was = (*vis != Visibility::Hidden).then(|| t.0.clone());
    if was.as_deref() == shown {
        return;
    }
    match shown {
        Some(s) => {
            t.0 = s.to_string();
            *vis = Visibility::Inherited;
            shared.log.lock().unwrap().detail(format!("caption shown: {s:?}"));
        }
        None => {
            *vis = Visibility::Hidden;
            shared.log.lock().unwrap().detail("caption hidden");
        }
    }
}

/// Applies the mouth changes (4a-6, rule 18) to Connor's face materials;
/// each change in `connor.log`.
pub fn mouth_draw(mut speech: ResMut<Speech>, connor: Option<ResMut<crate::connor_view::ConnorSim>>, mut materials: ResMut<Assets<crate::object_light::ObjectMaterial>>) {
    if speech.mouths.is_empty() {
        return;
    }
    let changes = std::mem::take(&mut speech.mouths);
    let Some(mut c) = connor else { return };
    for (speaker, shape) in changes {
        if !speaker.eq_ignore_ascii_case(CONNOR) || shape == c.mouth {
            continue;
        }
        // Rule 18: a shape of 8 or more uses lip0.
        let n = if shape > 7 { 0 } else { shape as usize };
        let image = c.lip_images.get(n).cloned().flatten();
        let what = match &image {
            Some(img) => {
                for h in &c.face {
                    if let Some(mut m) = materials.get_mut(h) {
                        m.texture = Some(img.clone());
                    }
                }
                format!("lip{n} on {} face material(s)", c.face.len())
            }
            None => format!("lip{n} has no texture: face unchanged"),
        };
        c.mouth = shape;
        let updates = c.updates;
        c.log.detail(format!("connor update {updates}: mouth shape {shape}: {what} (talking.md rule 18)"));
    }
}
