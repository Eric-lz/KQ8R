//! Animations drawn from compact data on the graphics card ("Animation
//! memory" step 4, docs/DESIGN.md; not from the original game: how we
//! keep its models).
//!
//! The world's animations are packed when it is read
//! (`kq8_data::anim_pack`, 4a); here the pack goes to the graphics card:
//! three storage buffers every `ObjectMaterial` binds (points, normals,
//! node-frames; [`AnimBuffers`], their contents replaced per world) and
//! one small mesh per topology (each corner's point number, its normal's
//! point number, its texture place). A drawn part is an entity with a
//! topology mesh, an animated copy of its material and a `MeshTag`: the
//! node-frame to show. The vertex shader (`object_light.wesl`, ANIMATED)
//! places and lights it.
//!
//! 4b: still animated objects (anim_view's objects that aren't
//! characters) are drawn both ways; F5 (or `--anim-draw old|new`) picks
//! which is shown, to compare by eye. The other kinds keep the old way.

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use bevy::render::storage::ShaderBuffer;
use kq8_data::anim_pack::{Pack, Part};

use crate::Shared;

/// Each corner's (point, normal's point) in its vertex frame.
pub const ATTRIBUTE_ANIM_CORNER: MeshVertexAttribute = MeshVertexAttribute::new("Anim_Corner", 0x4B51_3809, VertexFormat::Uint32x2);

/// The three buffers every object material binds (kept across worlds;
/// their contents are the current world's).
#[derive(Resource, Clone)]
pub struct AnimBuffers {
    pub points: Handle<ShaderBuffer>,
    pub normals: Handle<ShaderBuffer>,
    pub nodes: Handle<ShaderBuffer>,
}

/// A buffer for before any world (a storage buffer can't be empty).
fn placeholder() -> ShaderBuffer {
    ShaderBuffer::new(vec![0u32; 20], RenderAssetUsages::RENDER_WORLD)
}

impl AnimBuffers {
    pub fn new(buffers: &mut Assets<ShaderBuffer>) -> Self {
        AnimBuffers { points: buffers.add(placeholder()), normals: buffers.add(placeholder()), nodes: buffers.add(placeholder()) }
    }
}

/// A world's pack as read, with what each animation's frames show
/// (index = the animation's index in its list; `None`: not packed).
#[derive(Default)]
pub struct WorldPack {
    pub pack: Pack,
    pub objects: Vec<Option<Vec<Vec<Part>>>>,
    pub doors: Vec<Option<Vec<Vec<Part>>>>,
    pub connor: Vec<Option<Vec<Vec<Part>>>>,
}

/// The current world's animations on the graphics card.
#[derive(Resource, Default)]
pub struct AnimGpu {
    pub topologies: Vec<Handle<Mesh>>,
    pub objects: Vec<Option<Vec<Vec<Part>>>>,
    /// Doors' and Connor's, for 4c and 4f.
    #[allow(dead_code)]
    pub doors: Vec<Option<Vec<Vec<Part>>>>,
    #[allow(dead_code)]
    pub connor: Vec<Option<Vec<Vec<Part>>>>,
    /// Kept for bounds and shadows: the node-frames (model space) and the
    /// topologies' corners.
    pub pack: Pack,
}

impl AnimGpu {
    /// Sends `world` to the graphics card: the buffers' contents and one
    /// mesh per topology. Returns it and a line for the log.
    pub fn install(world: WorldPack, buffers: &AnimBuffers, assets: &mut Assets<ShaderBuffer>, meshes: &mut Assets<Mesh>) -> (AnimGpu, String) {
        let p = &world.pack;
        let points: Vec<u32> = p.points.iter().map(|v| u32::from_le_bytes(*v)).collect();
        let normals: Vec<u32> = p.normals.iter().flat_map(|n| [n[0] as u16 as u32 | (n[1] as u16 as u32) << 16, n[2] as u16 as u32 | (n[3] as u16 as u32) << 16]).collect();
        // Five vec4s per node-frame: the matrix's rows (w: where its points
        // and its normals start), the move, the ball; game space.
        let nodes: Vec<f32> = p
            .node_frames
            .iter()
            .flat_map(|n| {
                let m = n.matrix;
                [
                    m[0][0],
                    m[0][1],
                    m[0][2],
                    f32::from_bits(n.points),
                    m[1][0],
                    m[1][1],
                    m[1][2],
                    f32::from_bits(n.normals),
                    m[2][0],
                    m[2][1],
                    m[2][2],
                    0.0,
                    n.offset[0],
                    n.offset[1],
                    n.offset[2],
                    0.0,
                    n.ball[0],
                    n.ball[1],
                    n.ball[2],
                    n.ball[3],
                ]
            })
            .collect();
        let sizes = (points.len() * 4, normals.len() * 4, nodes.len() * 4);
        let pad = |mut v: Vec<u32>| {
            if v.is_empty() {
                v.push(0);
            }
            v
        };
        let _ = assets.insert(&buffers.points, ShaderBuffer::new(pad(points), RenderAssetUsages::RENDER_WORLD));
        let _ = assets.insert(&buffers.normals, ShaderBuffer::new(pad(normals), RenderAssetUsages::RENDER_WORLD));
        let nodes = if nodes.is_empty() { vec![0.0f32; 20] } else { nodes };
        let _ = assets.insert(&buffers.nodes, ShaderBuffer::new(nodes, RenderAssetUsages::RENDER_WORLD));
        let topologies: Vec<Handle<Mesh>> = p
            .topologies
            .iter()
            .map(|t| {
                let corners: Vec<[u32; 2]> = t.corners.iter().map(|c| [c.point, c.normal]).collect();
                let uvs: Vec<[f32; 2]> = t.corners.iter().map(|c| c.uv).collect();
                let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
                    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                    .with_inserted_attribute(ATTRIBUTE_ANIM_CORNER, VertexAttributeValues::Uint32x2(corners))
                    .with_inserted_indices(Indices::U32(t.indices.clone()));
                meshes.add(mesh)
            })
            .collect();
        let [_, _, _, topology_bytes] = p.sizes();
        let mb = |b: usize| b as f64 / (1 << 20) as f64;
        let line = format!(
            "animations on the graphics card (step 4): points {:.1} MB, normals {:.1} MB, node-frames {:.1} MB, {} topology meshes {:.1} MB",
            mb(sizes.0),
            mb(sizes.1),
            mb(sizes.2),
            topologies.len(),
            mb(topology_bytes)
        );
        let gpu = AnimGpu { topologies, objects: world.objects, doors: world.doors, connor: world.connor, pack: world.pack };
        (gpu, line)
    }

    /// The bounds (model space, game units) of every part of `frames`.
    pub fn bounds<'a>(&self, frames: impl IntoIterator<Item = &'a Vec<Vec<Part>>>) -> Option<([f32; 3], [f32; 3])> {
        let mut out: Option<([f32; 3], [f32; 3])> = None;
        for part in frames.into_iter().flatten().flatten() {
            let nf = &self.pack.node_frames[part.node_frame as usize];
            for c in &self.pack.topologies[part.topology as usize].corners {
                let q = kq8_data::anim_pack::place(nf, self.pack.points[(nf.points + c.point) as usize]);
                out = Some(match out {
                    None => (q, q),
                    Some((lo, hi)) => (std::array::from_fn(|i| lo[i].min(q[i])), std::array::from_fn(|i| hi[i].max(q[i]))),
                });
            }
        }
        out
    }
}

/// Which drawing still animated objects show (4b-4f): new (the default)
/// or the old frame entities, to compare. F5 switches.
#[derive(Resource, Clone, Copy, PartialEq, Eq, Debug)]
pub enum DrawMode {
    New,
    Old,
}

/// F5: switch between the old and the new drawing (logged).
pub fn toggle_draw(keys: Res<ButtonInput<KeyCode>>, mut mode: ResMut<DrawMode>, shared: Res<Shared>) {
    if !keys.just_pressed(KeyCode::F5) {
        return;
    }
    *mode = if *mode == DrawMode::New { DrawMode::Old } else { DrawMode::New };
    shared.log.lock().unwrap().summary(format!("key F5: animated objects drawn the {} way (Animation memory step 4)", if *mode == DrawMode::New { "new" } else { "old" }));
}

/// Which list of `AnimGpu` an object's animations index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum List {
    Objects,
    Doors,
}

impl AnimGpu {
    /// The packed frames of animation `anim` of `list`.
    pub fn frames(&self, list: List, anim: usize) -> Option<&Vec<Vec<Part>>> {
        match list {
            List::Objects => self.objects.get(anim),
            List::Doors => self.doors.get(anim),
        }
        .and_then(Option::as_ref)
    }
}

/// An object drawn from the animation buffers: its parts ("slots", one
/// per part of its largest frame) under `new_root`, its old frames under
/// `old_root`; the draw mode shows one of the two (4b-4f).
pub struct Slots {
    pub old_root: Entity,
    new_root: Entity,
    slots: Vec<Entity>,
    list: List,
    /// Per movement: its animation's index in `list`, model and ball.
    movements: Vec<(usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall)>,
    /// Animated material per (animation, material index).
    pub materials: std::collections::HashMap<(usize, i32), Handle<crate::object_light::ObjectMaterial>>,
    /// (movement, frame) the slots show.
    shown: Option<(usize, usize)>,
    /// The draw mode shown.
    mode: Option<DrawMode>,
}

impl Slots {
    /// The roots under `body` and the slots, if every movement is packed;
    /// `materials`: (animation, material index) -> the object's material
    /// (an animated copy is made of each).
    pub fn spawn(
        commands: &mut Commands,
        body: Entity,
        gpu: &AnimGpu,
        list: List,
        movements: Vec<(usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall)>,
        materials: impl IntoIterator<Item = ((usize, i32), Handle<crate::object_light::ObjectMaterial>)>,
        assets: &mut Assets<crate::object_light::ObjectMaterial>,
    ) -> Option<Slots> {
        if movements.is_empty() || movements.iter().any(|(a, _, _)| gpu.frames(list, *a).is_none()) {
            return None;
        }
        let frames: Vec<&Vec<Vec<Part>>> = movements.iter().filter_map(|(a, _, _)| gpu.frames(list, *a)).collect();
        let count = frames.iter().flat_map(|f| f.iter()).map(Vec::len).max().unwrap_or(0);
        // One box over every frame of every movement (the topology meshes
        // have no points of their own to measure).
        let aabb = gpu.bounds(frames.iter().copied()).map(|(lo, hi)| {
            let (a, b) = (Vec3::from(crate::to_bevy(lo, [0.0; 3])), Vec3::from(crate::to_bevy(hi, [0.0; 3])));
            bevy::camera::primitives::Aabb::from_min_max(a.min(b), a.max(b))
        });
        let old_root = commands.spawn((Transform::default(), Visibility::Inherited)).id();
        let new_root = commands.spawn((Transform::default(), Visibility::Hidden, movements[0].2)).id();
        commands.entity(body).add_children(&[old_root, new_root]);
        let slots = (0..count)
            .map(|_| {
                let mut e = commands.spawn((Transform::default(), Visibility::Hidden, bevy::mesh::MeshTag::new(0)));
                if let Some(aabb) = aabb {
                    e.insert(aabb);
                }
                let e = e.id();
                commands.entity(new_root).add_child(e);
                e
            })
            .collect();
        let materials = materials
            .into_iter()
            .filter_map(|(k, m)| assets.get(&m).map(crate::object_light::animated).map(|copy| (k, assets.add(copy))))
            .collect();
        Some(Slots { old_root, new_root, slots, list, movements, materials, shown: None, mode: None })
    }

    /// Part entities.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// Shows the old or the new drawing.
    pub fn set_mode(&mut self, mode: DrawMode, visibility: &mut Query<&mut Visibility>) {
        if self.mode == Some(mode) {
            return;
        }
        let new = mode == DrawMode::New;
        if let Ok(mut v) = visibility.get_mut(self.new_root) {
            *v = if new { Visibility::Inherited } else { Visibility::Hidden };
        }
        if let Ok(mut v) = visibility.get_mut(self.old_root) {
            *v = if new { Visibility::Hidden } else { Visibility::Inherited };
        }
        self.mode = Some(mode);
    }

    /// Whether the slots must be set again (never set, or forgotten).
    pub fn stale(&self, movement: usize, frame: usize) -> bool {
        self.shown != Some((movement, frame))
    }

    /// The materials changed (see-through copies): set the slots again.
    pub fn forget_shown(&mut self) {
        self.shown = None;
    }

    /// The slots show `frame` of `movement`: each part's topology mesh,
    /// its animated material and its node-frame tag; slots beyond the
    /// frame's parts are hidden.
    pub fn show(&mut self, commands: &mut Commands, gpu: &AnimGpu, movement: usize, frame: usize, visibility: &mut Query<&mut Visibility>) {
        let Some((anim, source, ball)) = self.movements.get(movement).cloned() else { return };
        let parts = gpu.frames(self.list, anim).and_then(|f| f.get(frame)).map_or(&[][..], Vec::as_slice);
        let sequence = source.as_ref().map_or(0, |s| s.first + frame);
        for (k, &slot) in self.slots.iter().enumerate() {
            match parts.get(k) {
                Some(part) => {
                    let mut e = commands.entity(slot);
                    e.insert((
                        Mesh3d(gpu.topologies[part.topology as usize].clone()),
                        bevy::mesh::MeshTag::new(part.node_frame),
                        crate::shadow_view::AnimSlot { source: source.clone(), sequence, node: part.node as usize, material: part.material },
                    ));
                    if let Some(m) = self.materials.get(&(anim, part.material)) {
                        e.insert(MeshMaterial3d(m.clone()));
                    }
                    if let Ok(mut v) = visibility.get_mut(slot) {
                        *v = Visibility::Inherited;
                    }
                }
                None => {
                    if let Ok(mut v) = visibility.get_mut(slot) {
                        *v = Visibility::Hidden;
                    }
                }
            }
        }
        commands.entity(self.new_root).insert(ball);
        self.shown = Some((movement, frame));
    }
}
