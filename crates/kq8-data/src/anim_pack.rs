//! Animations packed for the graphics card ("Animation memory" step 4,
//! docs/DESIGN.md). Not from the original game: how we keep its models.
//!
//! A model file's meshes keep their points as in the file (4 bytes each,
//! every vertex frame); each node of each frame gets one [`NodeFrame`]: its
//! place (the vertex frame's scale and origin folded into the node's
//! matrix), its part ball and where its points and normals start; the
//! corner normals (`object-lighting.md` rule 8) are kept per node-frame,
//! one per point, as 16-bit numbers, because a matrix that stretches
//! unevenly changes them. Each (mesh, texture frame, material) gets one
//! [`Topology`]: its corners (point, normal's point, texture place) and
//! the triangles over them.
//!
//! With `check`, [`Pack::add`] also unpacks what it packed exactly as the
//! shader will and compares it with [`Shape::geometry`]'s triangles
//! ([`Check`]); every world passed it (2026-10-23), so it is a test
//! option now (it costs about 0.4 s of Daventry's read).

use std::collections::HashMap;

use crate::shape::{Shape, Triangle};

/// One node of one frame.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NodeFrame {
    /// Rows applied as `point · matrix + offset` to the file's 0-254
    /// numbers give the point in model space.
    pub matrix: [[f32; 3]; 3],
    pub offset: [f32; 3],
    /// Its part's ball (centre, radius), as `Triangle::part_ball`.
    pub ball: [f32; 4],
    /// Index of its vertex frame's first point in `Pack::points`.
    pub points: u32,
    /// Index of its first normal in `Pack::normals`.
    pub normals: u32,
    /// The same part in the next frame of its animation (the last
    /// frame's: in frame 0), to draw part-way between the two (smooth
    /// animation, DESIGN S-2; ours): the node at the same place in the
    /// next frame's node list if it has the same mesh, else itself (that
    /// part jumps).
    pub next: u32,
    /// Smooth joins (DESIGN J-1, ours; not sent to the graphics card):
    /// its part's name (lower case, hashed) and its mesh's layout (see
    /// [`layout`]). A part of another movement's frame with both equal can
    /// be drawn mixed with it point by point.
    pub name: u64,
    pub layout: u64,
}

/// A corner of a topology: the point's number in its vertex frame, the
/// point whose normal it uses (its own; the triangle's first corner's for
/// flat-lit materials) and its texture place.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Corner {
    pub point: u32,
    pub normal: u32,
    pub uv: [f32; 2],
}

/// The triangles of one (mesh, texture frame, material), corners shared
/// where all three of a corner's values match. Corner order per triangle
/// as `model_mesh` writes it (0, 2, 1).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Topology {
    pub corners: Vec<Corner>,
    pub indices: Vec<u32>,
}

/// What a frame shows: per node and material, a topology placed by a
/// node-frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Part {
    pub topology: u32,
    pub node_frame: u32,
    pub material: i32,
    /// Its node in the model (`Triangle::node`).
    pub node: u32,
}

/// The largest differences between the packed data unpacked and
/// `Shape::geometry`'s triangles.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Check {
    pub corners: usize,
    pub position: f32,
    pub normal: f32,
    pub uv: f32,
    pub ball: f32,
}

impl Check {
    pub fn add(&mut self, o: &Check) {
        self.corners += o.corners;
        self.position = self.position.max(o.position);
        self.normal = self.normal.max(o.normal);
        self.uv = self.uv.max(o.uv);
        self.ball = self.ball.max(o.ball);
    }
}

/// Everything packed for one world.
#[derive(Debug, Default)]
pub struct Pack {
    /// Every packed mesh's points (all its vertex frames), as in the file.
    pub points: Vec<[u8; 4]>,
    /// Per node-frame, one normal per point of its vertex frame.
    pub normals: Vec<[i16; 4]>,
    pub node_frames: Vec<NodeFrame>,
    pub topologies: Vec<Topology>,
    /// Smooth animation (S-2): node-frames with a match in the next frame,
    /// and all of them.
    pub matched: (u32, u32),
    /// (shape, mesh) -> its first point.
    mesh_points: HashMap<(usize, usize), u32>,
    /// (shape, mesh) -> its layout (J-1).
    mesh_layouts: HashMap<(usize, usize), u64>,
    /// (shape, mesh, texture frame, material, flat) -> topology.
    topology_index: HashMap<(usize, usize, i16, i32, bool), u32>,
}

/// A unit normal as 16-bit numbers.
pub fn pack_normal(n: [f32; 3]) -> [i16; 4] {
    let q = |v: f32| (v.clamp(-1.0, 1.0) * 32767.0).round() as i16;
    [q(n[0]), q(n[1]), q(n[2]), 0]
}

pub fn unpack_normal(n: [i16; 4]) -> [f32; 3] {
    [n[0] as f32 / 32767.0, n[1] as f32 / 32767.0, n[2] as f32 / 32767.0]
}

/// The point in model space, as the shader works it out.
pub fn place(nf: &NodeFrame, raw: [u8; 4]) -> [f32; 3] {
    std::array::from_fn(|j| raw[0] as f32 * nf.matrix[0][j] + raw[1] as f32 * nf.matrix[1][j] + raw[2] as f32 * nf.matrix[2][j] + nf.offset[j])
}

/// The node's matrix and move with its vertex frame's scale and origin
/// folded in: `(v · scale + origin) · m + p` = `v · (scale · m) + (origin · m + p)`.
pub fn fold(scale: [f32; 3], origin: [f32; 3], m: &[[f32; 3]; 3], p: [f32; 3]) -> ([[f32; 3]; 3], [f32; 3]) {
    let matrix = std::array::from_fn(|i| std::array::from_fn(|j| scale[i] * m[i][j]));
    let offset = std::array::from_fn(|j| origin[0] * m[0][j] + origin[1] * m[1][j] + origin[2] * m[2][j] + p[j]);
    (matrix, offset)
}

/// FNV-1a over `bytes` (the same number on every run and machine).
fn fnv(hash: u64, bytes: &[u8]) -> u64 {
    bytes.iter().fold(hash, |h, &b| (h ^ b as u64).wrapping_mul(0x100_0000_01b3))
}

const FNV_START: u64 = 0xcbf2_9ce4_8422_2325;

/// A part name as [`NodeFrame::name`] keeps it.
pub fn name_hash(name: &str) -> u64 {
    fnv(FNV_START, name.to_lowercase().as_bytes())
}

/// A mesh's layout (smooth joins, DESIGN J-1, ours): its number of points
/// per frame and its triangles as sets of point numbers, in any order.
/// Movements are separate model files; two meshes with the same layout
/// number their points the same way, so their poses can be mixed point
/// by point. Meshes of the same size numbered differently (the zombies'
/// files, measured 2026-10-25) get different layouts.
pub fn layout(mesh: &crate::shape::Mesh) -> u64 {
    let mut triangles: Vec<[i32; 3]> = mesh
        .faces
        .iter()
        .map(|f| {
            let mut c = f.corners.map(|c| c.0);
            c.sort_unstable();
            c
        })
        .collect();
    triangles.sort_unstable();
    triangles.dedup();
    let mut h = fnv(FNV_START, &(mesh.vertices_per_frame as u64).to_le_bytes());
    for t in triangles {
        for c in t {
            h = fnv(h, &c.to_le_bytes());
        }
    }
    h
}

fn distance(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b).map(|(x, y)| (x - y) * (x - y)).sum::<f32>().sqrt()
}

impl Pack {
    /// Packs the `count` frames from sequence `first` of `shape` (`id`:
    /// the same number for the same parsed file); `flat`: is a material
    /// flat-lit; `check`: compare what was packed (see the module notes).
    /// Returns what each frame shows and the check (empty without).
    pub fn add(&mut self, shape: &Shape, id: usize, first: usize, count: usize, flat: &dyn Fn(i32) -> bool, check_it: bool) -> (Vec<Vec<Part>>, Check) {
        let mut frames = Vec::with_capacity(count);
        let mut check = Check::default();
        // Per frame: (place in its node list, mesh, node-frame), for the
        // next-frame table (S-2).
        let mut placed: Vec<Vec<(usize, usize, u32)>> = Vec::with_capacity(count);
        for seq in first..first + count {
            let mut parts = Vec::new();
            placed.push(Vec::new());
            let sub = shape.sequences.get(seq).and_then(|s| usize::try_from(s.mesh_subsequence).ok()).and_then(|i| shape.subsequences.get(i));
            let Some(sub) = sub else {
                frames.push(parts);
                continue;
            };
            let geometry = shape.geometry(seq);
            let mut by_node: HashMap<usize, Vec<&Triangle>> = HashMap::new();
            for t in &geometry.triangles {
                by_node.entry(t.node).or_default().push(t);
            }
            let start = sub.first_node.max(0) as usize;
            for node_index in start..start + sub.node_count.max(0) as usize {
                let Some(node) = shape.nodes.get(node_index) else { continue };
                let Some(mesh_index) = usize::try_from(node.mesh).ok().filter(|&m| m < shape.meshes.len()) else { continue };
                let mesh = &shape.meshes[mesh_index];
                let Some(frame) = usize::try_from(node.vertex_frame).ok().and_then(|f| mesh.frames.get(f)) else { continue };
                let triangles = by_node.get(&node_index).map(Vec::as_slice).unwrap_or(&[]);
                if triangles.is_empty() {
                    continue;
                }
                let base = *self.mesh_points.entry((id, mesh_index)).or_insert_with(|| {
                    let at = self.points.len() as u32;
                    self.points.extend_from_slice(&mesh.vertices);
                    at
                });
                // Normals: one per point of this vertex frame, from the
                // corners that use it (all corners of a point carry the
                // same sum, rule 8).
                let normals_at = self.normals.len();
                let corner_points = |t: &Triangle| mesh.faces[t.face].corners.map(|c| c.0.max(0) as usize);
                let used = triangles.iter().flat_map(|t| corner_points(t)).max().map_or(0, |m| m + 1);
                self.normals.resize(normals_at + mesh.vertices_per_frame.max(used), [0; 4]);
                for t in triangles {
                    for (k, p) in corner_points(t).into_iter().enumerate() {
                        self.normals[normals_at + p] = pack_normal(t.normals[k]);
                    }
                }
                let (matrix, offset) = fold(frame.scale, frame.origin, &node.transform.m, node.transform.p);
                let layout = *self.mesh_layouts.entry((id, mesh_index)).or_insert_with(|| layout(mesh));
                let name = name_hash(usize::try_from(node.name).ok().and_then(|n| shape.names.get(n)).map_or("", String::as_str));
                let nf = NodeFrame {
                    matrix,
                    offset,
                    ball: triangles[0].part_ball,
                    points: base + frame.first_vertex.max(0) as u32,
                    normals: normals_at as u32,
                    next: self.node_frames.len() as u32,
                    name,
                    layout,
                };
                let nf_index = self.node_frames.len() as u32;
                self.node_frames.push(nf);
                if let Some(p) = placed.last_mut() {
                    p.push((node_index - start, mesh_index, nf_index));
                }
                // One topology per material, in the order they appear.
                let mut materials: Vec<i32> = Vec::new();
                for t in triangles {
                    if !materials.contains(&t.material) {
                        materials.push(t.material);
                    }
                }
                for m in materials {
                    let is_flat = flat(m);
                    let key = (id, mesh_index, node.texture_frame, m, is_flat);
                    let topology = match self.topology_index.get(&key) {
                        Some(&i) => i,
                        None => {
                            let mut top = Topology::default();
                            let mut seen: HashMap<(u32, u32, u32, u32), u32> = HashMap::new();
                            for t in triangles.iter().filter(|t| t.material == m) {
                                let ps = corner_points(t);
                                for k in [0, 2, 1] {
                                    let c = Corner { point: ps[k] as u32, normal: if is_flat { ps[0] } else { ps[k] } as u32, uv: t.uvs[k] };
                                    let at = *seen.entry((c.point, c.normal, c.uv[0].to_bits(), c.uv[1].to_bits())).or_insert_with(|| {
                                        top.corners.push(c);
                                        top.corners.len() as u32 - 1
                                    });
                                    top.indices.push(at);
                                }
                            }
                            self.topologies.push(top);
                            let i = self.topologies.len() as u32 - 1;
                            self.topology_index.insert(key, i);
                            i
                        }
                    };
                    let part = Part { topology, node_frame: nf_index, material: m, node: node_index as u32 };
                    if check_it {
                        check.add(&self.check(&part, triangles, is_flat));
                    }
                    parts.push(part);
                }
            }
            // Material first, then node: the order the old drawing had
            // (one mesh per material), so parts at the same distance (a
            // see-through figure and what it holds) keep the same drawing
            // order every frame (the user's look, 2026-10-23: the ghost
            // knight's staff flickered).
            parts.sort_by_key(|p| (p.material, p.node));
            frames.push(parts);
        }
        self.link_next(&placed);
        (frames, check)
    }

    /// Smooth animation (S-2, ours): each node-frame's match in the next
    /// frame (frame 0 after the last), `placed` as `add` keeps it.
    fn link_next(&mut self, placed: &[Vec<(usize, usize, u32)>]) {
        for (f, frame) in placed.iter().enumerate() {
            let next = &placed[(f + 1) % placed.len()];
            for &(k, mesh, nf) in frame {
                self.matched.1 += 1;
                if let Some(&(_, _, to)) = next.iter().find(|&&(k2, m2, _)| k2 == k && m2 == mesh) {
                    self.node_frames[nf as usize].next = to;
                    self.matched.0 += 1;
                }
            }
        }
    }

    /// Smooth joins (DESIGN J-1, ours): for each part of `new` (a frame of
    /// the movement now shown), the node-frame of its partner in `old` (the
    /// frame shown before the change): an old node with the same name and
    /// layout, each old node taken once, in the order they come (a frame
    /// may hold two swords). `None`: no partner, that part jumps. The
    /// parts of one node (one per material) share their partner.
    pub fn partners(&self, old: &[Part], new: &[Part]) -> Vec<Option<u32>> {
        let key = |nf: u32| {
            let n = &self.node_frames[nf as usize];
            (n.name, n.layout)
        };
        // One entry per old node: its node-frame, still free or not.
        let mut free: Vec<(u32, bool)> = Vec::new();
        for p in old {
            if !free.iter().any(|&(nf, _)| nf == p.node_frame) {
                free.push((p.node_frame, true));
            }
        }
        let mut chosen: HashMap<u32, Option<u32>> = HashMap::new();
        new.iter()
            .map(|p| {
                *chosen.entry(p.node_frame).or_insert_with(|| {
                    let slot = free.iter_mut().find(|(nf, open)| *open && key(*nf) == key(p.node_frame))?;
                    slot.1 = false;
                    Some(slot.0)
                })
            })
            .collect()
    }

    /// Smooth joins (J-1): over every change from one of `movements` (its
    /// last frame) to another (its first frame), the parts of the new frame
    /// with a partner and all of them, and the changes where every part has
    /// one and all changes.
    pub fn join_share(&self, movements: &[&Vec<Vec<Part>>]) -> ([usize; 2], [usize; 2]) {
        let (mut parts, mut joins) = ([0, 0], [0, 0]);
        for (i, a) in movements.iter().enumerate() {
            for (j, b) in movements.iter().enumerate() {
                let (Some(old), Some(new)) = (a.last(), b.first()) else { continue };
                if i == j || new.is_empty() {
                    continue;
                }
                let found = self.partners(old, new).iter().filter(|p| p.is_some()).count();
                parts = [parts[0] + found, parts[1] + new.len()];
                joins = [joins[0] + usize::from(found == new.len()), joins[1] + 1];
            }
        }
        (parts, joins)
    }

    /// Unpacks `part` as the shader will and compares it with the
    /// triangles of its material (in the same order).
    fn check(&self, part: &Part, triangles: &[&Triangle], flat: bool) -> Check {
        let mut c = Check::default();
        let top = &self.topologies[part.topology as usize];
        let nf = &self.node_frames[part.node_frame as usize];
        let expected = triangles.iter().filter(|t| t.material == part.material);
        for (t, corners) in expected.zip(top.indices.chunks(3)) {
            for (slot, &k) in [0, 2, 1].iter().enumerate() {
                let corner = top.corners[corners[slot] as usize];
                let raw = self.points[(nf.points + corner.point) as usize];
                let normal = unpack_normal(self.normals[(nf.normals + corner.normal) as usize]);
                let want_normal = if flat { t.normals[0] } else { t.normals[k] };
                c.corners += 1;
                c.position = c.position.max(distance(&place(nf, raw), &t.positions[k]));
                c.normal = c.normal.max(distance(&normal, &want_normal));
                c.uv = c.uv.max(distance(&corner.uv, &t.uvs[k]));
                c.ball = c.ball.max(distance(&nf.ball, &t.part_ball));
            }
        }
        c
    }

    /// Puts `other` after this pack (packs made side by side, one per
    /// processor core). Returns what to add to `other`'s parts' topology
    /// and node-frame numbers.
    pub fn merge(&mut self, other: Pack) -> (u32, u32) {
        let (points, normals) = (self.points.len() as u32, self.normals.len() as u32);
        let (node_frames, topologies) = (self.node_frames.len() as u32, self.topologies.len() as u32);
        self.points.extend(other.points);
        self.normals.extend(other.normals);
        self.node_frames.extend(other.node_frames.into_iter().map(|n| NodeFrame { points: n.points + points, normals: n.normals + normals, next: n.next + node_frames, ..n }));
        self.matched = (self.matched.0 + other.matched.0, self.matched.1 + other.matched.1);
        self.topologies.extend(other.topologies);
        for (k, v) in other.mesh_points {
            self.mesh_points.entry(k).or_insert(v + points);
        }
        for (k, v) in other.mesh_layouts {
            self.mesh_layouts.entry(k).or_insert(v);
        }
        for (k, v) in other.topology_index {
            self.topology_index.entry(k).or_insert(v + topologies);
        }
        (topologies, node_frames)
    }

    /// Bytes each buffer would take on the graphics card: points, normals,
    /// node-frames (72 bytes each), topologies (16 bytes a corner, 4 an
    /// index).
    pub fn sizes(&self) -> [usize; 4] {
        let topology = self.topologies.iter().map(|t| t.corners.len() * 16 + t.indices.len() * 4).sum();
        [self.points.len() * 4, self.normals.len() * 8, self.node_frames.len() * 72, topology]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn merging_moves_the_second_packs_numbers_along() {
        let nf = |points, normals| NodeFrame { matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], offset: [0.0; 3], ball: [0.0; 4], points, normals, next: 0, name: 0, layout: 0 };
        let mut a = Pack { points: vec![[1, 2, 3, 0]; 4], normals: vec![[0; 4]; 4], node_frames: vec![nf(0, 0)], topologies: vec![Topology::default()], ..Default::default() };
        let b = Pack { points: vec![[9, 9, 9, 0]; 3], normals: vec![[1; 4]; 3], node_frames: vec![nf(0, 0), nf(1, 2)], topologies: vec![Topology::default(); 2], ..Default::default() };
        let (topology_add, node_frame_add) = a.merge(b);
        assert_eq!((topology_add, node_frame_add), (1, 1));
        assert_eq!(a.points.len(), 7);
        assert_eq!(a.points[4], [9, 9, 9, 0]);
        assert_eq!((a.node_frames[1].points, a.node_frames[1].normals), (4, 4));
        assert_eq!((a.node_frames[2].points, a.node_frames[2].normals), (5, 6));
        assert_eq!((a.node_frames[1].next, a.node_frames[2].next), (1, 1), "next moved along too");
        assert_eq!(a.topologies.len(), 3);
    }

    #[test]
    fn each_node_frame_knows_the_same_part_in_the_next_frame() {
        // Frame 0: parts at places 0 (mesh 4) and 1 (mesh 5); frame 1:
        // place 0 has mesh 4, place 1 changed to mesh 6; frame 2: only
        // place 0. The last frame's go to frame 0.
        let nf = NodeFrame { matrix: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]], offset: [0.0; 3], ball: [0.0; 4], points: 0, normals: 0, next: 0, name: 0, layout: 0 };
        let mut p = Pack { node_frames: (0..5).map(|i| NodeFrame { next: i, ..nf }).collect(), ..Default::default() };
        p.link_next(&[vec![(0, 4, 0), (1, 5, 1)], vec![(0, 4, 2), (1, 6, 3)], vec![(0, 4, 4)]]);
        let next: Vec<u32> = p.node_frames.iter().map(|n| n.next).collect();
        assert_eq!(next, [2, 1, 4, 3, 0]);
        assert_eq!(p.matched, (3, 5));
    }

    #[test]
    fn a_new_frames_parts_find_their_partners_by_name_and_layout() {
        // Old frame: sword (node-frame 0), body in two materials (1), a
        // second sword (2), an unnamed part (3). New frame: body in two
        // materials (10), sword (11, its name in other capitals), sword
        // (12), a third sword (13: no partner left), a "body" numbered
        // otherwise (14), the unnamed part (15).
        let (sword, body, none) = (name_hash("Sword"), name_hash("body"), name_hash(""));
        let nf = |name, layout| NodeFrame { matrix: [[0.0; 3]; 3], offset: [0.0; 3], ball: [0.0; 4], points: 0, normals: 0, next: 0, name, layout };
        let mut p = Pack { node_frames: vec![nf(sword, 7), nf(body, 1), nf(sword, 7), nf(none, 3)], ..Default::default() };
        p.node_frames.resize(10, nf(0, 0));
        p.node_frames.extend([nf(body, 1), nf(name_hash("SWORD"), 7), nf(sword, 7), nf(sword, 7), nf(body, 2), nf(none, 3)]);
        let part = |node_frame| Part { topology: 0, node_frame, material: 0, node: 0 };
        let old: Vec<Part> = [0, 1, 1, 2, 3].map(part).to_vec();
        let new: Vec<Part> = [10, 10, 11, 12, 13, 14, 15].map(part).to_vec();
        assert_eq!(p.partners(&old, &new), [Some(1), Some(1), Some(0), Some(2), None, None, Some(3)]);
        assert_eq!(p.partners(&[], &new), [None; 7], "nothing shown before");
        // Two movements of one frame each: old -> new has 5 of 7, new ->
        // old 5 of 5 (the old frame's parts, 1 twice).
        let (a, b) = (vec![old], vec![new]);
        assert_eq!(p.join_share(&[&a, &b]), ([10, 12], [1, 2]));
    }

    #[test]
    fn a_layout_ignores_the_order_of_triangles_but_not_the_point_numbers() {
        use crate::shape::{Face, Mesh};
        let face = |a, b, c| Face { corners: [(a, 0), (b, 0), (c, 0)], material: 0 };
        let mesh = |faces: Vec<Face>| Mesh {
            version: 3,
            vertices_per_frame: 4,
            texture_vertices_per_frame: 0,
            radius: 1.0,
            vertices: Vec::new(),
            texture_vertices: Vec::new(),
            faces,
            frames: Vec::new(),
            frame_was_implied: false,
        };
        let a = layout(&mesh(vec![face(0, 1, 2), face(1, 2, 3)]));
        assert_eq!(a, layout(&mesh(vec![face(3, 2, 1), face(2, 0, 1)])), "order of triangles and corners");
        assert_ne!(a, layout(&mesh(vec![face(0, 1, 3), face(1, 2, 3)])), "other point numbers");
        assert_ne!(a, layout(&Mesh { vertices_per_frame: 5, ..mesh(vec![face(0, 1, 2), face(1, 2, 3)]) }), "another size");
    }

    #[test]
    fn normals_pack_to_within_a_hair() {
        for n in [[1.0, 0.0, 0.0], [0.0, -1.0, 0.0], [0.6, 0.0, -0.8], [0.267, 0.535, 0.802]] {
            let back = unpack_normal(pack_normal(n));
            assert!(distance(&back, &n) < 1e-4, "{n:?} -> {back:?}");
        }
    }

    #[test]
    fn folding_the_scale_and_origin_gives_the_same_point() {
        let (scale, origin) = ([2.5, 0.5, 1.25], [-300.0, 40.0, 7.5]);
        let m = [[0.0, 1.2, 0.0], [-0.8, 0.0, 0.1], [0.0, 0.0, 1.0]];
        let p = [100.0, -20.0, 5.0];
        let (matrix, offset) = fold(scale, origin, &m, p);
        let nf = NodeFrame { matrix, offset, ball: [0.0; 4], points: 0, normals: 0, next: 0, name: 0, layout: 0 };
        for raw in [[0u8, 0, 0, 0], [254, 3, 77, 9], [12, 200, 254, 0]] {
            let decoded: [f32; 3] = std::array::from_fn(|i| raw[i] as f32 * scale[i] + origin[i]);
            let want: [f32; 3] = std::array::from_fn(|j| decoded[0] * m[0][j] + decoded[1] * m[1][j] + decoded[2] * m[2][j] + p[j]);
            assert!(distance(&place(&nf, raw), &want) < 1e-3);
        }
    }
}
