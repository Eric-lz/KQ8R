//! What the potions show on Connor (M5 5g; `docs/behaviour/potions.md`).
//!
//! 5g-4, the shield's orbit (rule 20): while `INVITEM_Invulnerable` runs,
//! the `orbit1.kq` shape (`invulnfx.kqs`, see-through 165 of 256, no
//! shadow) circles Connor: its centre is his model part `body` lowered by
//! 820 and raised by `orbitZoffset`, at `orbitRadius`, turned by
//! `orbitStepAngZ` every update from `orbitStartAngZ`; it is removed when
//! the shield ends. Its place each update goes to `items.log`.
//! **Ours** (not in the note): the shape turns with its angle (a rigid
//! turn about the centre), and without a `body` part in the frame his
//! feet are the centre.
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

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::item_view::Parts;
use crate::object_light::{see_through, ObjectMaterial};
use crate::terrain_view::bevy_transform_from;

/// What the potions change on Connor's model (5g-5).
pub struct ConnorLooks {
    /// Every material of his model: its texture file (lower case) and its
    /// own picture.
    materials: Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>,
    body_skin: Option<String>,
    iron: Option<(String, Handle<Image>)>,
    faces: kq8_data::characters::SkinList,
    face_sets: BTreeMap<String, Vec<Option<Handle<Image>>>>,
    /// Shown now: the Strength skin, the face set.
    iron_on: bool,
    face: Option<String>,
}

impl ConnorLooks {
    pub fn new(l: crate::load::PotionLooks, materials: Vec<(Handle<ObjectMaterial>, String, Option<Handle<Image>>)>, images: &mut Assets<Image>) -> Self {
        let mut image = |look: crate::load::Look| match look {
            crate::load::Look::Texture { width, height, rgba, .. } => Some(images.add(crate::object_light::texture_image(width, height, rgba, bevy::image::ImageAddressMode::Repeat))),
            _ => None,
        };
        let iron = l.iron.and_then(|(n, look)| Some((n, image(look)?)));
        let face_sets = l.face_sets.into_iter().map(|(n, set)| (n, set.into_iter().map(|s| s.and_then(&mut image)).collect())).collect();
        ConnorLooks { materials, body_skin: l.body_skin.map(|s| s.to_ascii_lowercase()), iron, faces: l.faces, face_sets, iron_on: false, face: None }
    }
}

#[derive(Resource)]
pub struct OrbitDraw {
    keys: Option<kq8_data::characters::OrbitKeys>,
    parts: Parts,
    /// The orbit drawn now and its angle.
    drawn: Option<(Entity, f32)>,
}

/// At load: the orbit's keys and its model (see-through by its file).
pub fn spawn_orbit(commands: &mut Commands, orbit: Option<(kq8_data::characters::OrbitKeys, Option<usize>)>, built: &[Parts], materials: &mut Assets<ObjectMaterial>) {
    let (keys, parts) = match orbit {
        Some((k, slot)) => {
            let opacity = if k.translucency >= 256.0 { 1.0 } else { (k.translucency / 255.0).clamp(0.0, 1.0) };
            let parts = slot.and_then(|s| built.get(s)).map(|p| p.iter().map(|(m, mat)| (m.clone(), see_through(materials, mat, opacity))).collect()).unwrap_or_default();
            (Some(k), parts)
        }
        None => (None, Vec::new()),
    };
    commands.insert_resource(OrbitDraw { keys, parts, drawn: None });
}

/// Every update: the orbit appears with the shield, moves, and goes.
pub fn orbit_step(mut commands: Commands, sim: Option<ResMut<DoorSim>>, connor: Option<Res<ConnorSim>>, draw: Option<ResMut<OrbitDraw>>, mut transforms: Query<&mut Transform>) {
    let (Some(mut sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    let w = &mut sim.active.world;
    let shield = w.potions.iter().any(|r| r.effect == kq8_core::potions::Effect::Shield);
    let updates = w.updates;
    if !shield {
        if let Some((e, _)) = draw.drawn.take() {
            commands.entity(e).despawn();
            w.item_log.push(format!("update {updates}: the shield's orbit is removed (potions.md rule 20)"));
        }
        return;
    }
    let Some(k) = draw.keys.clone() else { return };
    let Some(c) = connor else { return };
    let (body, how) = match c.part_now("body") {
        Some(p) => (p, "his part 'body'"),
        None => (c.mover.position, "no part 'body' in this frame: his feet"),
    };
    let angle = match draw.drawn {
        Some((_, a)) => a + k.step_angle[2],
        None => k.start_angle[2],
    };
    let at = kq8_core::potions::orbit_place(body, k.radius, k.z_offset, angle);
    let t = bevy_transform_from(kq8_data::objects::rotation([0.0, 0.0, angle]), at);
    let e = match draw.drawn {
        Some((e, _)) => {
            if let Ok(mut tr) = transforms.get_mut(e) {
                *tr = t;
            }
            e
        }
        None => {
            let e = commands.spawn((t, Visibility::Inherited, crate::terrain_view::ObjectName("orbit1".into()))).id();
            for (mesh, material) in &draw.parts {
                let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                commands.entity(e).add_child(part);
            }
            w.item_log.push(format!("update {updates}: the shield's orbit appears: {} ({} model parts) (potions.md rule 20)", k.shape.as_deref().unwrap_or("no shape"), draw.parts.len()));
            e
        }
    };
    draw.drawn = Some((e, angle));
    w.item_log.push(format!(
        "update {updates}: orbit at ({:.0}, {:.0}, {:.0}), angle {angle:.2} rad; centre {how} at ({:.0}, {:.0}, {:.0}) - 820 + {} (potions.md rule 20)",
        at[0], at[1], at[2], body[0], body[1], body[2], k.z_offset
    ));
}

/// Every update (5g-5): Connor's body skin and face set follow the
/// potions; each change in `connor.log` and `items.log`.
pub fn look_step(sim: Option<ResMut<DoorSim>>, connor: Option<ResMut<ConnorSim>>, mut materials: ResMut<Assets<ObjectMaterial>>) {
    let (Some(mut sim), Some(mut c)) = (sim, connor) else { return };
    let c = &mut *c;
    let w = &mut sim.active.world;
    let updates = w.updates;
    let strength = w.potions.iter().any(|r| r.effect == kq8_core::potions::Effect::Strength);
    if strength != c.looks.iron_on {
        c.looks.iron_on = strength;
        let l = &c.looks;
        let line = match (&l.body_skin, &l.iron) {
            (Some(skin), Some((iron, iron_image))) => {
                let mut n = 0;
                for (h, texture, own) in &l.materials {
                    if texture == skin {
                        if let Some(mut m) = materials.get_mut(h) {
                            m.texture = if strength { Some(iron_image.clone()) } else { own.clone() };
                            n += 1;
                        }
                    }
                }
                if strength {
                    format!("strength: his skin {skin} drawn as {iron} on {n} materials (potions.md rule 25)")
                } else {
                    format!("strength over: his skin {skin} back on {n} materials (potions.md rule 25)")
                }
            }
            _ => format!("strength {}: no body skin or Strength skin known; his look unchanged", if strength { "on" } else { "off" }),
        };
        c.log.detail(format!("connor update {}: {line}", c.updates));
        w.item_log.push(format!("update {updates}: {line}"));
    }
    let bits = kq8_core::potions::list_bits(w, &c.looks.faces.items);
    let wanted = c.looks.faces.best(bits).map(str::to_ascii_lowercase);
    if wanted != c.looks.face {
        let line = match wanted.as_ref().and_then(|f| c.looks.face_sets.get(f).filter(|s| s.iter().any(Option::is_some)).map(|s| (f, s.clone()))) {
            Some((f, set)) => {
                c.lip_images = set;
                let mouth = if c.mouth > 7 { 0 } else { c.mouth as usize };
                if let Some(img) = c.lip_images.get(mouth).cloned().flatten() {
                    for h in &c.face {
                        if let Some(mut m) = materials.get_mut(h) {
                            m.texture = Some(img.clone());
                        }
                    }
                }
                format!("face list bits {bits}: face set {f} (mouth {mouth} shown; potions.md rule 30)")
            }
            None => format!("face list bits {bits}: face set {wanted:?} has no pictures in this world; the face stays (potions.md rule 30)"),
        };
        c.looks.face = wanted;
        c.log.detail(format!("connor update {}: {line}", c.updates));
        w.item_log.push(format!("update {updates}: {line}"));
    }
}
