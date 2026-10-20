//! Objects a script loads while playing as new copies (M5 5d; plan in
//! `docs/DESIGN.md` "5d"; `objects.md` "Where a world's objects come
//! from" rules 2, 6). Core makes the object at the `loadKQ` line
//! (`Active::load_copy`) and asks for it to be drawn
//! (`Request::Spawn`); this module draws it from the template's model,
//! kept from the start, and gives it its collider.
//!
//! 5d-3: still models (the barrels' wreck `barelded`); 5d-4: animated
//! ones (the burst `barelupx`), drawn by `anim_view`. **Ours**: a copy
//! has no draw-distance test (always drawn when in view) and no lights
//! or see-through settings of its own; it doesn't block the camera.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::door_view::DoorSim;
use crate::item_view::Parts;
use crate::terrain_view::{bevy_transform_from, ColliderOrder, ObjectColliders, ObjectName};
use crate::Shared;

/// A still template: its model and what its collider needs.
struct Still {
    parts: Parts,
    bounds: Option<([f32; 3], [f32; 3])>,
    collider_type: u32,
    collider_mask: u32,
}

/// The templates' models, by lower-case file name; the animated ones'
/// object names (their recipes are in `AnimSim`, 5d-4).
#[derive(Resource)]
pub struct TemplateDraw {
    stills: BTreeMap<String, Still>,
    anims: BTreeMap<String, String>,
}

/// After the models are built (`terrain_view::setup_objects`): keeps the
/// still templates' models. `built`: each model slot's parts.
pub fn keep_templates(commands: &mut Commands, templates: &[crate::load::TemplateAssets], placed: &[(kq8_data::objects::Placement, usize, bool)], built: &[Parts]) {
    let stills = templates
        .iter()
        .filter(|t| !t.animated)
        .filter_map(|t| {
            let parts = built.get(placed.get(t.placed)?.1)?.clone();
            Some((t.file.clone(), Still { parts, bounds: t.bounds, collider_type: t.collider_type, collider_mask: t.collider_mask }))
        })
        .collect();
    let anims = templates.iter().filter(|t| t.animated).map(|t| (t.file.clone(), t.name.clone())).collect();
    commands.insert_resource(TemplateDraw { stills, anims });
}

/// Each update, after the scripts: the copies asked for are drawn; a
/// still one whose box and type collide with Connor (the pair rule) gets
/// a collider, tested after all the others (it was loaded last).
pub fn spawn_copies(
    mut commands: Commands,
    sim: Option<ResMut<DoorSim>>,
    draw: Option<Res<TemplateDraw>>,
    mut colliders: Option<ResMut<ObjectColliders>>,
    mut order: Option<ResMut<ColliderOrder>>,
    mut anim: Option<ResMut<crate::anim_view::AnimSim>>,
    shared: Res<Shared>,
) {
    use kq8_core::collide::{pair_collides, BoxCollider, ColliderKind, CONNOR_MASK, CONNOR_TYPE};
    let Some(mut sim) = sim else { return };
    let spawns = std::mem::take(&mut sim.spawns);
    for (file, name, placement) in spawns {
        let p = placement.position;
        let line = match draw.as_ref().and_then(|d| d.stills.get(&file)) {
            Some(t) => {
                let transform = bevy_transform_from(placement.rotation, placement.position);
                for (mesh, material) in &t.parts {
                    commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()), transform, ObjectName(name.clone())));
                }
                let collides = pair_collides(CONNOR_TYPE, CONNOR_MASK, t.collider_type, t.collider_mask);
                let collider = match (t.bounds, collides, colliders.as_mut(), order.as_mut()) {
                    (Some((min, max)), true, Some(c), Some(o)) => {
                        let kind = ColliderKind { collision_type: t.collider_type, collision_mask: t.collider_mask, ball_stack: false, keep_on_ground: true };
                        c.0.push(BoxCollider { name: name.clone(), min, max, placement, kind });
                        o.0.push(c.0.len() - 1);
                        format!("collider ({:?}..{:?}, type 0x{:X})", min, max, t.collider_type)
                    }
                    (None, _, _, _) => "no box: no collider".to_string(),
                    _ => format!("type 0x{:X} mask 0x{:X}: doesn't collide with Connor", t.collider_type, t.collider_mask),
                };
                format!("copy {name} of {file} drawn at ({:.0}, {:.0}, {:.0}), {} part(s); {collider} (M5 5d-3)", p[0], p[1], p[2], t.parts.len())
            }
            None => match (draw.as_ref().and_then(|d| d.anims.get(&file)), anim.as_mut(), sim.active.world.anim_index(&name)) {
                // 5d-4: an animated copy, played by core's object.
                (Some(template), Some(a), Some(index)) => {
                    let line = a.spawn_copy(&mut commands, template, &name, placement, index);
                    format!("{line} at ({:.0}, {:.0}, {:.0})", p[0], p[1], p[2])
                }
                _ => format!("copy {name} of {file} at ({:.0}, {:.0}, {:.0}): no template drawn here; not drawn", p[0], p[1], p[2]),
            },
        };
        shared.log.lock().unwrap().summary(&line);
        sim.log.detail(line);
    }
}
