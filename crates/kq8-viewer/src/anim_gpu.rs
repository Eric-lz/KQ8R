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
//! Every animated object, door, chest, character and Connor is drawn this
//! way (4b-4f; the old finished frames were removed in 4g).

use bevy::asset::RenderAssetUsages;
use bevy::mesh::{Indices, MeshVertexAttribute, PrimitiveTopology, VertexAttributeValues};
use bevy::prelude::*;
use bevy::render::render_resource::VertexFormat;
use bevy::render::storage::ShaderBuffer;
use kq8_data::anim_pack::{Pack, Part};


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
    /// `--check-pack` was given (weapon sets added later are checked too).
    pub check: bool,
}

/// The current world's animations on the graphics card.
#[derive(Resource, Default)]
pub struct AnimGpu {
    pub topologies: Vec<Handle<Mesh>>,
    pub objects: Vec<Option<Vec<Vec<Part>>>>,
    pub doors: Vec<Option<Vec<Vec<Part>>>>,
    /// Connor's: his start-up animations, then the weapon sets loaded
    /// while playing (`add_connor`).
    pub connor: Vec<Option<Vec<Vec<Part>>>>,
    /// Kept for bounds and shadows: the node-frames (model space) and the
    /// topologies' corners.
    pub pack: Pack,
    /// `--check-pack`.
    check: bool,
}

impl AnimGpu {
    /// Sends `world` to the graphics card: the buffers' contents and one
    /// mesh per topology. Returns it and a line for the log.
    pub fn install(world: WorldPack, buffers: &AnimBuffers, assets: &mut Assets<ShaderBuffer>, meshes: &mut Assets<Mesh>) -> (AnimGpu, String) {
        let mut gpu = AnimGpu { topologies: Vec::new(), objects: world.objects, doors: world.doors, connor: world.connor, pack: world.pack, check: world.check };
        gpu.add_topologies(meshes);
        let line = gpu.upload(buffers, assets);
        (gpu, line)
    }

    /// A mesh for every topology that has none yet.
    fn add_topologies(&mut self, meshes: &mut Assets<Mesh>) {
        for t in &self.pack.topologies[self.topologies.len()..] {
            let corners: Vec<[u32; 2]> = t.corners.iter().map(|c| [c.point, c.normal]).collect();
            let uvs: Vec<[f32; 2]> = t.corners.iter().map(|c| c.uv).collect();
            let mesh = Mesh::new(PrimitiveTopology::TriangleList, RenderAssetUsages::RENDER_WORLD)
                .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
                .with_inserted_attribute(ATTRIBUTE_ANIM_CORNER, VertexAttributeValues::Uint32x2(corners))
                .with_inserted_indices(Indices::U32(t.indices.clone()));
            self.topologies.push(meshes.add(mesh));
        }
    }

    /// Weapon sets loaded while playing (M6 6c-2): their animations packed
    /// and the buffers sent again. Returns each animation's index in
    /// `connor` (`None`: not packed) and a line for the log.
    pub fn add_connor(&mut self, animations: &[crate::load::Animation], buffers: &AnimBuffers, assets: &mut Assets<ShaderBuffer>, meshes: &mut Assets<Mesh>) -> (Vec<Option<usize>>, String) {
        let started = std::time::Instant::now();
        let mut check = kq8_data::anim_pack::Check::default();
        let indices = animations
            .iter()
            .map(|a| {
                let source = a.source.as_ref()?;
                let flat = |m: i32| a.looks.iter().any(|(i, _, s)| *i == m && s.lighting == kq8_data::Lighting::Flat);
                let id = std::sync::Arc::as_ptr(&source.shape) as usize;
                let (packed, c) = self.pack.add(&source.shape, id, source.first, a.frame_count, &flat, self.check);
                check.add(&c);
                self.connor.push(Some(packed));
                Some(self.connor.len() - 1)
            })
            .collect();
        self.add_topologies(meshes);
        let line = self.upload(buffers, assets);
        let line = format!(
            "{line}; {} animations added while playing, check: position {:.4}, normal {:.6}; {} ms",
            animations.len(),
            check.position,
            check.normal,
            started.elapsed().as_millis()
        );
        (indices, line)
    }

    /// Sends the pack's buffers to the graphics card again.
    fn upload(&self, buffers: &AnimBuffers, assets: &mut Assets<ShaderBuffer>) -> String {
        let p = &self.pack;
        let points: Vec<u32> = p.points.iter().map(|v| u32::from_le_bytes(*v)).collect();
        let normals: Vec<u32> = p.normals.iter().flat_map(|n| [n[0] as u16 as u32 | (n[1] as u16 as u32) << 16, n[2] as u16 as u32 | (n[3] as u16 as u32) << 16]).collect();
        // Five vec4s per node-frame: the matrix's rows (w: where its points
        // and its normals start, and its match in the next frame for
        // smooth animation, S-2), the move, the ball; game space.
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
                    f32::from_bits(n.next),
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
        let [_, _, _, topology_bytes] = p.sizes();
        let mb = |b: usize| b as f64 / (1 << 20) as f64;
        format!(
            "animations on the graphics card (step 4): points {:.1} MB, normals {:.1} MB, node-frames {:.1} MB, {} topology meshes {:.1} MB",
            mb(sizes.0),
            mb(sizes.1),
            mb(sizes.2),
            self.topologies.len(),
            mb(topology_bytes)
        )
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


/// Which list of `AnimGpu` an object's animations index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum List {
    Objects,
    Doors,
    Connor,
}

impl AnimGpu {
    /// The packed frames of animation `anim` of `list`.
    pub fn frames(&self, list: List, anim: usize) -> Option<&Vec<Vec<Part>>> {
        match list {
            List::Objects => self.objects.get(anim),
            List::Doors => self.doors.get(anim),
            List::Connor => self.connor.get(anim),
        }
        .and_then(Option::as_ref)
    }
}

/// Animated copies made so far, by the material they copy: the same
/// material gets one copy (Connor's: his ~470 identical face materials,
/// one per animation, became one; the user's check, 2026-10-23: changing
/// hundreds of materials at each mouth shape stuttered).
pub type Copies = std::collections::HashMap<AssetId<crate::object_light::ObjectMaterial>, Handle<crate::object_light::ObjectMaterial>>;

/// Each key's animated copy, made once per material (`cache`).
fn copies_of(
    materials: impl IntoIterator<Item = ((usize, i32), Handle<crate::object_light::ObjectMaterial>)>,
    assets: &mut Assets<crate::object_light::ObjectMaterial>,
    cache: &mut Copies,
) -> Vec<((usize, i32), Handle<crate::object_light::ObjectMaterial>)> {
    materials
        .into_iter()
        .filter_map(|(k, m)| {
            if let Some(copy) = cache.get(&m.id()) {
                return Some((k, copy.clone()));
            }
            let copy = assets.add(assets.get(&m).map(crate::object_light::animated)?);
            cache.insert(m.id(), copy.clone());
            Some((k, copy))
        })
        .collect()
}

/// An object drawn from the animation buffers: its parts ("slots", one
/// per part of its largest frame) under `new_root`.
pub struct Slots {
    new_root: Entity,
    slots: Vec<Entity>,
    list: List,
    /// Per movement: its animation's index in `list`, model and ball.
    movements: Vec<(usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall)>,
    /// Animated material per (animation, material index).
    pub materials: std::collections::HashMap<(usize, i32), Handle<crate::object_light::ObjectMaterial>>,
    /// (movement, frame) the slots show.
    shown: Option<(usize, usize)>,
    /// The object's box over every frame (Bevy units, its own space).
    aabb: Option<(Vec3, Vec3)>,
}

impl Slots {
    /// The roots under `body` and the slots, if every movement is packed;
    /// `materials`: (animation, material index) -> the object's material
    /// (an animated copy is made of each).
    #[allow(clippy::too_many_arguments)]
    pub fn spawn(
        commands: &mut Commands,
        body: Entity,
        gpu: &AnimGpu,
        list: List,
        movements: Vec<(usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall)>,
        materials: impl IntoIterator<Item = ((usize, i32), Handle<crate::object_light::ObjectMaterial>)>,
        assets: &mut Assets<crate::object_light::ObjectMaterial>,
        cache: &mut Copies,
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
            (a.min(b), a.max(b))
        });
        let new_root = commands.spawn((Transform::default(), Visibility::Inherited, movements[0].2)).id();
        commands.entity(body).add_child(new_root);
        let slots = (0..count)
            .map(|_| {
                let mut e = commands.spawn((Transform::default(), Visibility::Hidden, bevy::mesh::MeshTag::new(0)));
                if let Some((lo, hi)) = aabb {
                    e.insert(bevy::camera::primitives::Aabb::from_min_max(lo, hi));
                }
                let e = e.id();
                commands.entity(new_root).add_child(e);
                e
            })
            .collect();
        let materials = copies_of(materials, assets, cache).into_iter().collect();
        Some(Slots { new_root, slots, list, movements, materials, shown: None, aabb })
    }

    /// Part entities.
    pub fn len(&self) -> usize {
        self.slots.len()
    }

    /// More movements (Connor's weapon sets loaded while playing): more
    /// slots if their frames have more parts, the box grown, animated
    /// copies of their materials. Returns the new copies by key.
    pub fn extend(
        &mut self,
        commands: &mut Commands,
        gpu: &AnimGpu,
        movements: Vec<(usize, Option<crate::load::FrameShape>, crate::shadow_view::ModelBall)>,
        materials: impl IntoIterator<Item = ((usize, i32), Handle<crate::object_light::ObjectMaterial>)>,
        assets: &mut Assets<crate::object_light::ObjectMaterial>,
        cache: &mut Copies,
    ) -> Vec<((usize, i32), Handle<crate::object_light::ObjectMaterial>)> {
        let frames: Vec<&Vec<Vec<Part>>> = movements.iter().filter_map(|(a, _, _)| gpu.frames(self.list, *a)).collect();
        let count = frames.iter().flat_map(|f| f.iter()).map(Vec::len).max().unwrap_or(0);
        if let Some((lo, hi)) = gpu.bounds(frames.iter().copied()) {
            let (a, b) = (Vec3::from(crate::to_bevy(lo, [0.0; 3])), Vec3::from(crate::to_bevy(hi, [0.0; 3])));
            self.aabb = Some(match self.aabb {
                Some((l, h)) => (l.min(a.min(b)), h.max(a.max(b))),
                None => (a.min(b), a.max(b)),
            });
        }
        while self.slots.len() < count {
            let e = commands.spawn((Transform::default(), Visibility::Hidden, bevy::mesh::MeshTag::new(0))).id();
            commands.entity(self.new_root).add_child(e);
            self.slots.push(e);
        }
        self.movements.extend(movements);
        let added = copies_of(materials, assets, cache);
        self.materials.extend(added.iter().cloned());
        self.shown = None;
        added
    }

    /// Nothing shown (the rope model while he isn't on the rope).
    pub fn hide(&mut self, visibility: &mut Query<&mut Visibility>) {
        if self.shown.is_none() {
            return;
        }
        for &slot in &self.slots {
            if let Ok(mut v) = visibility.get_mut(slot) {
                *v = Visibility::Hidden;
            }
        }
        self.shown = None;
    }

    /// The new drawing's place in its body (the rope model's offset).
    pub fn set_offset(&self, transforms: &mut Query<&mut Transform, (Without<crate::connor_view::ConnorBody>, Without<Camera3d>)>, at: Vec3) {
        if let Ok(mut t) = transforms.get_mut(self.new_root) {
            t.translation = at;
        }
    }

    /// The entity whose children are the parts shown (for shadows: its
    /// children carry `AnimSlot`, its ball the movement's).
    pub fn root(&self) -> Entity {
        self.new_root
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
                    // At its part's ball centre: Bevy sorts see-through
                    // parts by their mesh's centre, and a topology mesh has
                    // none of its own (the user's look: the ghost knight's
                    // staff flickered); the shader takes the points
                    // relative to it, the box moves the other way.
                    let ball = gpu.pack.node_frames[part.node_frame as usize].ball;
                    let offset = Vec3::from(crate::to_bevy([ball[0], ball[1], ball[2]], [0.0; 3]));
                    let mut e = commands.entity(slot);
                    e.insert((
                        Mesh3d(gpu.topologies[part.topology as usize].clone()),
                        bevy::mesh::MeshTag::new(part.node_frame),
                        Transform::from_translation(offset),
                        crate::shadow_view::AnimSlot { source: source.clone(), sequence, node: part.node as usize, material: part.material, offset },
                    ));
                    if let Some((lo, hi)) = self.aabb {
                        e.insert(bevy::camera::primitives::Aabb::from_min_max(lo - offset, hi - offset));
                    }
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
                    // Not part of this frame: no shadow from it.
                    commands.entity(slot).remove::<crate::shadow_view::AnimSlot>();
                }
            }
        }
        commands.entity(self.new_root).insert(ball);
        self.shown = Some((movement, frame));
    }
}
