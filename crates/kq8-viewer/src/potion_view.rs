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

use bevy::prelude::*;

use crate::connor_view::ConnorSim;
use crate::door_view::DoorSim;
use crate::item_view::Parts;
use crate::object_light::{see_through, ObjectMaterial};
use crate::terrain_view::bevy_transform_from;

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
