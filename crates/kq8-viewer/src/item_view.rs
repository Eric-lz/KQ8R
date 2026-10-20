//! Items lying in the world (M3 step 3e-3, docs/DESIGN.md; rules:
//! docs/behaviour/items.md).
//!
//! The item objects live in `kq8_core::active` (`DoorSim`), which picks
//! them up when clicked. Here each item lying in the world (and not
//! hidden) is drawn with its type's model, turned and placed as made
//! (rule 5), see-through by its type's translucency (as objects,
//! `objects.md` Q14). An item made while playing gets its entity when it
//! first appears; one picked up, deleted or hidden is hidden.

use std::collections::BTreeMap;

use bevy::prelude::*;

use crate::door_view::DoorSim;
use crate::terrain_view::bevy_transform_from;
use crate::draw_distance::{DistanceTest, ShapeBalls};
use crate::object_light::{see_through, ObjectMaterial};

/// One drawn model: (mesh, material) per material group.
type Parts = Vec<(Handle<Mesh>, Handle<ObjectMaterial>)>;

#[derive(Resource)]
pub struct ItemDraw {
    /// Each item type (`IDName`) that has a model here.
    parts: BTreeMap<String, Parts>,
    /// Each item object drawn so far, by name.
    drawn: BTreeMap<String, Entity>,
    /// Each item type's draw distance test (M1 6h).
    tests: BTreeMap<String, DistanceTest>,
}

/// `looks`: (`IDName`, index into `built`, opacity) per type.
pub fn spawn_items(commands: &mut Commands, looks: Vec<(String, usize, f32, String)>, built: &[Parts], materials: &mut Assets<ObjectMaterial>, balls: &ShapeBalls) {
    let mut parts = BTreeMap::new();
    let mut tests = BTreeMap::new();
    for (kind, slot, opacity, shape) in looks {
        let Some(model) = built.get(slot) else { continue };
        tests.insert(kind.clone(), balls.item(&kind, &shape));
        let p: Parts = model
            .iter()
            .map(|(mesh, material)| {
                let material = see_through(materials, material, opacity);
                (mesh.clone(), material)
            })
            .collect();
        parts.insert(kind, p);
    }
    commands.insert_resource(ItemDraw { parts, drawn: BTreeMap::new(), tests });
}

/// Shows the items lying in the world, hides the others.
pub fn item_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<ItemDraw>>, mut visibility: Query<&mut Visibility>, mut transforms: Query<&mut Transform>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    for o in &sim.active.world.items.objects {
        let at = o.placement().filter(|_| !o.hidden);
        match (draw.drawn.get(&o.name), at) {
            (Some(&e), at) => {
                // M5 5c-8: a thrown item moves while it flies.
                if let Some(p) = at.filter(|_| sim.active.world.flights.iter().any(|f| f.item == o.name)) {
                    if let Ok(mut t) = transforms.get_mut(e) {
                        *t = bevy_transform_from(p.rotation, p.position);
                    }
                }
                let want = if at.is_some() { Visibility::Inherited } else { Visibility::Hidden };
                if let Ok(mut v) = visibility.get_mut(e) {
                    if *v != want {
                        *v = want;
                    }
                }
            }
            (None, Some(p)) => {
                let Some(parts) = draw.parts.get(&o.kind) else { continue };
                let mut body = commands.spawn((bevy_transform_from(p.rotation, p.position), Visibility::Inherited));
                if let Some(test) = draw.tests.get(&o.kind) {
                    // Its own name: the collider switch and clicks go by it.
                    let mut test = test.another();
                    test.name = o.name.clone();
                    body.insert(test);
                }
                let body = body.id();
                for (mesh, material) in parts {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(body).add_child(part);
                }
                draw.drawn.insert(o.name.clone(), body);
            }
            (None, None) => {}
        }
    }
}

/// The corpses' models (`fighting.md` rule 32; M5 5c-7), and the corpses
/// drawn so far.
#[derive(Resource)]
pub struct CorpseDraw {
    /// Lower-case shape name -> its model.
    parts: BTreeMap<String, Parts>,
    /// How many of the world's corpses have their entity.
    drawn: usize,
    /// The bolts in the air drawn now (M5 5c-9), by name.
    bolts: BTreeMap<String, Entity>,
}

/// `shapes`: lower-case shape name -> index into `built`.
pub fn spawn_corpses(commands: &mut Commands, shapes: BTreeMap<String, usize>, built: &[Parts]) {
    let parts = shapes.into_iter().filter_map(|(k, slot)| built.get(slot).map(|p| (k, p.clone()))).collect();
    commands.insert_resource(CorpseDraw { parts, drawn: 0, bolts: BTreeMap::new() });
}

/// Each new corpse: its model where the character stood, turned the same
/// way (rule 32). **Ours**: no shadow (not a caster) and no draw-distance
/// test (always drawn when in view); the skin swap is not applied (only
/// `bossprig` has one in Daventry).
pub fn corpse_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<CorpseDraw>>, shared: Res<crate::Shared>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let corpses = &sim.active.world.corpses;
    while draw.drawn < corpses.len() {
        let c = &corpses[draw.drawn];
        draw.drawn += 1;
        let Some(parts) = draw.parts.get(&c.shape.to_ascii_lowercase()) else {
            shared.log.lock().unwrap().summary(format!("corpse of {}: shape {} not loaded; not drawn", c.name, c.shape));
            continue;
        };
        let body = commands.spawn((bevy_transform_from(c.rotation, c.position), Visibility::Inherited, crate::terrain_view::ObjectName(format!("{}_corpse", c.name)))).id();
        for (mesh, material) in parts {
            let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
            commands.entity(body).add_child(part);
        }
        shared.log.lock().unwrap().summary(format!("corpse of {}: {} drawn at ({:.0}, {:.0}, {:.0})", c.name, c.shape, c.position[0], c.position[1], c.position[2]));
    }
}

/// The bolts in the air (`fighting.md` rule 36; M5 5c-9): each drawn with
/// its shape where it is, turned to its heading (**stand-in**: no pitch;
/// the model's own direction is taken as the game's forward, −y);
/// removed when it is gone.
pub fn bolt_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<CorpseDraw>>, mut transforms: Query<&mut Transform>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    let bolts = &sim.active.world.bolts;
    draw.bolts.retain(|name, e| {
        let alive = bolts.iter().any(|b| &b.name == name);
        if !alive {
            commands.entity(*e).despawn();
        }
        alive
    });
    for b in bolts {
        let heading = b.velocity[0].atan2(-b.velocity[1]);
        let rotation = kq8_data::objects::rotation([0.0, 0.0, heading]);
        let t = bevy_transform_from(rotation, b.position);
        match draw.bolts.get(&b.name) {
            Some(&e) => {
                if let Ok(mut tr) = transforms.get_mut(e) {
                    *tr = t;
                }
            }
            None => {
                let Some(parts) = b.shape.as_ref().and_then(|s| draw.parts.get(&s.to_ascii_lowercase())) else { continue };
                let body = commands.spawn((t, Visibility::Inherited)).id();
                for (mesh, material) in parts {
                    let part = commands.spawn((Mesh3d(mesh.clone()), MeshMaterial3d(material.clone()))).id();
                    commands.entity(body).add_child(part);
                }
                draw.bolts.insert(b.name.clone(), body);
            }
        }
    }
}
