//! What the potions show on Connor (M5 5g; `docs/behaviour/potions.md`).
//!
//! 5g-4, the shield's orbits (rule 20): while `INVITEM_Invulnerable`
//! runs, the shapes of `orbit1.kq`–`orbit3.kq` (each `invulnfx.kqs`,
//! see-through 165 of 256, no shadow; start angles 0, 2.09, −2.09)
//! circle Connor: centre his model part `body` lowered by 820 and raised
//! by `orbitZoffset`, at `orbitRadius`, turned by `orbitStepAngZ` every
//! update from `orbitStartAngZ`; removed when the shield ends. Their
//! places each update go to `items.log`. **The user's check
//! (2026-10-21):** three shields, their flat side facing out (each shape
//! turned its orbit angle + a quarter turn). **Ours:** without a `body`
//! part in the frame his feet are the centre.
//!
//! 5g-5, Connor's look (rules 25, 30): while Strength runs, every material
//! of his model drawn with his body skin (`con0.bmp`, the skin list's
//! entry for no bits) shows the Strength entry (`coniron.bmp`); his
//! normal skin comes back at the end. His face pictures are the face
//! list's set for his bits (helmet 1, temple armour 2, Clarity 4,
//! Strength 8: an item of that type applied); a missing combination
//! takes the entry with most of the wanted bits ("Do not reproduce" 3).
//! Not built: the first-person arms (`armconfx.bmp`, no first person
//! yet), the temple armour's skin (`con65.bmp`, armour skins not built).
//!
//! 5g-7, `KQObject::dither` (rule 31: the keep wall "becomes
//! see-through"): STAND-IN (Q144): a dithered object's parts are drawn
//! half see-through until `dither 0`.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::item_view::Parts;
use crate::object_light::{see_through, ObjectMaterial};
use crate::terrain_view::bevy_transform_from;

/// What the potions change on Connor's model (5g-5, 5g-6).
pub struct ConnorLooks {
    /// Every material of his model: its texture file (lower case) and its
    /// own picture.
    materials: Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>,
    body_skin: Option<String>,
    iron: Option<(String, Handle<Image>)>,
    faces: kq8_data::characters::SkinList,
    face_sets: BTreeMap<String, Vec<Option<Handle<Image>>>>,
    /// The `I_` pictures of this world by his texture's name (5g-6).
    ghosts: BTreeMap<String, Handle<Image>>,
    /// The mesh copies made so far, by the picture they copy (5g-6).
    meshes: std::collections::HashMap<AssetId<Image>, Handle<Image>>,
    /// Shown now: the Strength skin, the face set, the Invisible look.
    iron_on: bool,
    /// The face set asked for last, and the one shown (a set with no
    /// pictures here leaves the last one shown).
    face: Option<String>,
    face_shown: Option<String>,
    invisible: bool,
    /// Materials were added (M6 6c-2): the next look step redoes them all.
    redo: bool,
}

impl ConnorLooks {
    pub fn new(l: crate::load::PotionLooks, materials: Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>, images: &mut Assets<Image>) -> Self {
        let mut image = |look: crate::load::Look| match look {
            crate::load::Look::Texture { width, height, rgba, .. } => Some(images.add(crate::object_light::texture_image(width, height, rgba, bevy::image::ImageAddressMode::Repeat))),
            _ => None,
        };
        let iron = l.iron.and_then(|(n, look)| Some((n.to_ascii_lowercase(), image(look)?)));
        let face_sets = l.face_sets.into_iter().map(|(n, set)| (n, set.into_iter().map(|s| s.and_then(&mut image)).collect())).collect();
        let ghosts = l.ghosts.into_iter().filter_map(|(n, look)| Some((n, image(look)?))).collect();
        ConnorLooks {
            materials,
            body_skin: l.body_skin.map(|s| s.to_ascii_lowercase()),
            iron,
            faces: l.faces,
            face_sets,
            ghosts,
            meshes: Default::default(),
            iron_on: false,
            face: None,
            face_shown: None,
            invisible: false,
            redo: false,
        }
    }

    /// M6 6c-2: the materials of a weapon set loaded while playing, and the
    /// `I_` pictures of its textures; the next look step gives them the
    /// looks shown now (Strength, Invisible, the face set).
    pub fn add(&mut self, materials: Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>, ghosts: Vec<(String, crate::load::Look)>, images: &mut Assets<Image>) {
        self.materials.extend(materials);
        for (n, look) in ghosts {
            if let crate::load::Look::Texture { width, height, rgba, .. } = look {
                self.ghosts.entry(n).or_insert_with(|| images.add(crate::object_light::texture_image(width, height, rgba, bevy::image::ImageAddressMode::Repeat)));
            }
        }
        self.redo = true;
    }

    /// Rule 29: a copy of `pic` with 3 of every 4 pixels see-through, one
    /// kept every 4 along a row, shifted by 2 on each next row (the
    /// pattern of the game's own `I_` pictures); made once per picture.
    fn mesh(&mut self, images: &mut Assets<Image>, pic: &Handle<Image>) -> Option<Handle<Image>> {
        if let Some(m) = self.meshes.get(&pic.id()) {
            return Some(m.clone());
        }
        let src = images.get(pic)?;
        let (w, h) = (src.width(), src.height());
        let mut rgba = src.data.clone()?;
        for y in 0..h {
            for x in 0..w {
                if (x + 2 * y) % 4 != 0 {
                    rgba[((y * w + x) * 4 + 3) as usize] = 0;
                }
            }
        }
        let m = images.add(crate::object_light::texture_image(w, h, rgba, bevy::image::ImageAddressMode::Repeat));
        self.meshes.insert(pic.id(), m.clone());
        Some(m)
    }
}

#[derive(Resource)]
pub struct OrbitDraw {
    /// Each orbit file's keys and its model.
    orbits: Vec<(kq8_data::characters::OrbitKeys, Parts)>,
    /// Each orbit drawn now and its angle (same order).
    drawn: Vec<(Entity, f32)>,
}

/// At load: the orbits' keys and their model (see-through by its file).
pub fn spawn_orbit(commands: &mut Commands, orbits: Vec<(kq8_data::characters::OrbitKeys, Option<usize>)>, built: &[Parts], materials: &mut Assets<ObjectMaterial>) {
    let orbits = orbits
        .into_iter()
        .map(|(k, slot)| {
            let opacity = if k.translucency >= 256.0 { 1.0 } else { (k.translucency / 255.0).clamp(0.0, 1.0) };
            let parts = slot.and_then(|s| built.get(s)).map(|p| p.iter().map(|(m, mat)| (m.clone(), see_through(materials, mat, opacity))).collect()).unwrap_or_default();
            (k, parts)
        })
        .collect();
    commands.insert_resource(OrbitDraw { orbits, drawn: Vec::new() });
}

/// The user's check (2026-10-21): the shield's flat side faces away from
/// Connor. Its model (`invulnfx.kqs`) is wide along x and thin along y,
/// so it is turned a quarter turn more than its orbit angle (its bulging
/// −y side outwards).
const ORBIT_FACING: f32 = std::f32::consts::FRAC_PI_2;

/// Every update: the orbits appear with the shield, move, and go.
pub fn orbit_step(mut commands: Commands, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>, draw: Option<ResMut<OrbitDraw>>, mut transforms: Query<&mut Transform>) {
    let (Some(mut sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    let w = &mut sim.active.world;
    let shield = w.potions.iter().any(|r| r.effect == kq8_core::potions::Effect::Shield);
    let updates = w.updates;
    if !shield {
        if !draw.drawn.is_empty() {
            for (e, _) in draw.drawn.drain(..) {
                commands.entity(e).despawn();
            }
            w.item_log.push(format!("update {updates}: the shield's orbits are removed (potions.md rule 20)"));
        }
        return;
    }
    let Some(c) = connor else { return };
    let (body, how) = match c.part_now("body") {
        Some(p) => (p, "his part 'body'"),
        None => (c.mover.position, "no part 'body' in this frame: his feet"),
    };
    let first = draw.drawn.is_empty();
    let mut places = Vec::new();
    for (n, (k, parts)) in draw.orbits.iter().enumerate() {
        let angle = match draw.drawn.get(n) {
            Some((_, a)) => a + k.step_angle[2],
            None => k.start_angle[2],
        };
        let at = kq8_core::potions::orbit_place(body, k.radius, k.z_offset, angle);
        let t = bevy_transform_from(kq8_data::objects::rotation([0.0, 0.0, angle + ORBIT_FACING]), at);
        match draw.drawn.get_mut(n) {
            Some((e, a)) => {
                if let Ok(mut tr) = transforms.get_mut(*e) {
                    *tr = t;
                }
                *a = angle;
            }
            None => {
                let e = commands.spawn((t, Visibility::Inherited, crate::terrain_view::ObjectName(format!("orbit{}", n + 1)))).id();
                for (mesh, material) in parts {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(e).add_child(part);
                }
                draw.drawn.push((e, angle));
            }
        }
        places.push(format!("orbit{} ({:.0}, {:.0}, {:.0}) angle {angle:.2}", n + 1, at[0], at[1], at[2]));
    }
    if first {
        w.item_log.push(format!("update {updates}: the shield's orbits appear: {} (potions.md rule 20)", draw.orbits.len()));
    }
    w.item_log.push(format!("update {updates}: {}; centre {how} at ({:.0}, {:.0}, {:.0}) - 820 + z offset; each turned a quarter turn more (faces out) (potions.md rule 20)", places.join(", "), body[0], body[1], body[2]));
}

/// Every update (5g-5, 5g-6): Connor's body skin, face set and the
/// Invisible look follow the potions; worked out again whenever one of
/// them changes (rule 29: the textures are converted again when his skin
/// or face changes). Each change in `connor.log` and `items.log`.
pub fn look_step(sim: Option<ResMut<DoorSim>>, connor: Option<ResMut<ConnorSim>>, mut materials: ResMut<Assets<ObjectMaterial>>, mut images: ResMut<Assets<Image>>) {
    let (Some(mut sim), Some(mut c)) = (sim, connor) else { return };
    let c = &mut *c;
    let w = &mut sim.active.world;
    let updates = w.updates;
    let strength = w.potions.iter().any(|r| r.effect == kq8_core::potions::Effect::Strength);
    let invisible = w.connor_invisible();
    let bits = kq8_core::potions::list_bits(w, &c.looks.faces.items);
    let wanted = c.looks.faces.best(bits).map(str::to_ascii_lowercase);
    if strength == c.looks.iron_on && wanted == c.looks.face && invisible == c.looks.invisible && !c.looks.redo {
        return;
    }
    c.looks.redo = false;
    let mut lines = Vec::new();
    if strength != c.looks.iron_on {
        lines.push(match (&c.looks.body_skin, &c.looks.iron) {
            (Some(skin), Some((iron, _))) if strength => format!("strength: his skin {skin} drawn as {iron} (potions.md rule 25)"),
            (Some(skin), Some(_)) => format!("strength over: his skin {skin} back (potions.md rule 25)"),
            _ => "strength: no body skin or Strength skin known; his skin unchanged".to_string(),
        });
    }
    if invisible != c.looks.invisible {
        let what = if invisible { "I_ pictures where this world has them, else mesh copies; the holes cut out" } else { "his own pictures back" };
        lines.push(format!("invisible {}: {what} (potions.md rule 29)", if invisible { "on" } else { "off" }));
    }
    // Rule 29: every material, from its picture as it would be shown.
    let (mut ghosted, mut meshed, mut plain) = (0, 0, 0);
    let list = c.looks.materials.clone();
    for (h, texture, own) in &list {
        if c.face.contains(h) {
            continue;
        }
        let (name, base) = match (&c.looks.body_skin, &c.looks.iron) {
            (Some(skin), Some((iron, img))) if strength && skin == texture => (iron.clone(), Some(img.clone())),
            _ => (texture.clone(), own.clone()),
        };
        let pic = if !invisible {
            plain += 1;
            base
        } else if let Some(g) = c.looks.ghosts.get(&name) {
            ghosted += 1;
            Some(g.clone())
        } else {
            meshed += 1;
            base.and_then(|b| c.looks.mesh(&mut images, &b))
        };
        if let Some(mut m) = materials.get_mut(h) {
            m.texture = pic;
            m.settings.cutout = u32::from(invisible);
        }
    }
    lines.push(if invisible { format!("{ghosted} materials with an I_ picture, {meshed} with a mesh copy") } else { format!("{plain} materials with their plain pictures") });
    // Rule 30: the face set (a mesh of it while Invisible runs).
    let found = wanted.as_ref().and_then(|f| c.looks.face_sets.get(f).filter(|s| s.iter().any(Option::is_some)).cloned());
    match &found {
        Some(_) => lines.push(format!("face list bits {bits}: face set {} (potions.md rule 30)", wanted.as_deref().unwrap_or("?"))),
        None => lines.push(format!("face list bits {bits}: face set {wanted:?} has no pictures in this world; the set stays (potions.md rule 30)")),
    }
    if found.is_some() {
        c.looks.face_shown = wanted.clone();
    }
    c.looks.face = wanted;
    let set = found.or_else(|| c.looks.face_shown.as_ref().and_then(|f| c.looks.face_sets.get(f).cloned()));
    if let Some(set) = set {
        c.lip_images = if invisible { set.iter().map(|p| p.as_ref().and_then(|p| c.looks.mesh(&mut images, p))).collect() } else { set };
    }
    let mouth = if c.mouth > 7 { 0 } else { c.mouth as usize };
    let img = c.lip_images.get(mouth).cloned().flatten();
    for h in &c.face {
        if let Some(mut m) = materials.get_mut(h) {
            if img.is_some() {
                m.texture = img.clone();
            }
            m.settings.cutout = u32::from(invisible);
        }
    }
    c.looks.iron_on = strength;
    c.looks.invisible = invisible;
    let line = format!("{}; mouth {mouth} shown", lines.join("; "));
    c.log.detail(format!("connor update {}: {line}", c.updates));
    w.item_log.push(format!("update {updates}: {line}"));
}

/// The parts drawn dithered now, with their own materials (5g-7).
#[derive(Default)]
pub struct Dithered {
    names: std::collections::BTreeSet<String>,
    parts: BTreeMap<Entity, Handle<ObjectMaterial>>,
}

/// STAND-IN (Q144): how much a dithered object shows.
const DITHER_OPACITY: f32 = 0.5;

/// Every update (5g-7): objects `KQObject::dither`ed are drawn half
/// see-through; `dither 0` brings their own materials back.
pub fn dither_step(
    sim: Option<Res<DoorSim>>,
    mut parts: Query<(Entity, &crate::terrain_view::ObjectName, &mut MeshMaterial3d<ObjectMaterial>)>,
    mut materials: ResMut<Assets<ObjectMaterial>>,
    mut state: Local<Dithered>,
    shared: Res<crate::Shared>,
    (change, mut seen): (Res<crate::world_build::WorldChange>, Local<u64>),
) {
    // A new world's objects have their own materials (M6 6e-2).
    if crate::world_build::new_world(&change, &mut seen) {
        *state = Dithered::default();
    }
    let Some(sim) = sim else { return };
    let wanted = &sim.active.world.dithered;
    if *wanted == state.names {
        return;
    }
    let (mut on, mut off) = (0, 0);
    for (e, name, mut m) in &mut parts {
        let dithered = wanted.contains(&name.0.to_ascii_lowercase());
        match (dithered, state.parts.contains_key(&e)) {
            (true, false) => {
                let own = m.0.clone();
                m.0 = see_through(&mut materials, &own, DITHER_OPACITY);
                state.parts.insert(e, own);
                on += 1;
            }
            (false, true) => {
                if let Some(own) = state.parts.remove(&e) {
                    m.0 = own;
                    off += 1;
                }
            }
            _ => {}
        }
    }
    state.names = wanted.clone();
    shared.log.lock().unwrap().summary(format!("dither: {wanted:?} now; {on} parts drawn half see-through, {off} back to their own (potions.md rule 31; stand-in, Q144)"));
}
