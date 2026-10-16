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
pub fn spawn_items(commands: &mut Commands, looks: Vec<(String, usize, f32)>, built: &[Parts], materials: &mut Assets<ObjectMaterial>, balls: &ShapeBalls) {
    let mut parts = BTreeMap::new();
    let mut tests = BTreeMap::new();
    for (kind, slot, opacity) in looks {
        let Some(model) = built.get(slot) else { continue };
        tests.insert(kind.clone(), DistanceTest::item(&kind, balls.by_index.get(slot).copied()));
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
pub fn item_draw(mut commands: Commands, sim: Option<Res<DoorSim>>, draw: Option<ResMut<ItemDraw>>, mut visibility: Query<&mut Visibility>) {
    let (Some(sim), Some(mut draw)) = (sim, draw) else { return };
    let draw = &mut *draw;
    for o in &sim.active.world.items.objects {
        let at = o.placement().filter(|_| !o.hidden);
        match (draw.drawn.get(&o.name), at) {
            (Some(&e), at) => {
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
                    body.insert(test.another());
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
