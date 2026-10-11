//! 3D models: `.kqs` files, class `KQ8::Shape`, holding `TS::CelAnimMesh`
//! meshes and a `TS::MaterialList`.
//!
//! Layout from libkq8fpc `kq8kqs.txt` (facts only), checked against the
//! real files with `kq8 shape-verify`; see docs/provenance/README.md.
//!
//! How a model is put together:
//! - A *mesh* is a set of triangles. Its vertices are stored as bytes
//!   (0..=254 per axis) and become positions through a per-frame scale and
//!   origin. A mesh can hold several *frames* (vertex animation).
//! - A *mesh node* places one frame of one mesh with a transform.
//! - A *sub-sequence* is a run of mesh nodes that are visible together.
//! - A *sequence* is one animation frame: a transform plus a sub-sequence.
//! - A *loop* is one animation: a run of sequences.
//!
//! The game's own normal table (256 directions that each vertex points
//! into) is not copied here; we compute normals from the triangles.

use crate::material::MaterialList;
use crate::pers::{read_block, ReadError, Reader};

pub const CLASS: &str = "KQ8::Shape";
pub const MESH_CLASS: &str = "TS::CelAnimMesh";

/// A placement: 3x3 matrix plus translation.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    /// bit 0: has rotation, bit 1: has translation, bit 2: has scale.
    pub flags: u32,
    pub m: [[f32; 3]; 3],
    pub p: [f32; 3],
}

impl Transform {
    fn read(r: &mut Reader) -> Result<Self, ReadError> {
        let flags = r.u32()?;
        let m = [r.vec3()?, r.vec3()?, r.vec3()?];
        let p = r.vec3()?;
        Ok(Transform { flags, m, p })
    }

    /// Applies the transform to a point.
    ///
    /// Assumption, not verified: the point is a row vector multiplied by
    /// the matrix (`x * m[0] + y * m[1] + z * m[2] + p`). Gives the same
    /// result as the other convention whenever the matrix only scales,
    /// which `kq8 shape-verify` counts. See docs/behaviour/QUESTIONS.md.
    pub fn apply(&self, v: [f32; 3]) -> [f32; 3] {
        let m = &self.m;
        std::array::from_fn(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j] + self.p[j])
    }

    /// True if any value off the matrix diagonal is non-zero.
    pub fn rotates(&self) -> bool {
        (0..3).any(|i| (0..3).any(|j| i != j && self.m[i][j] != 0.0))
    }
}

#[derive(Debug, Clone)]
pub struct MeshNode {
    pub transform: Transform,
    pub mesh: i16,
    pub vertex_frame: i16,
    /// -1 = no texture coordinates.
    pub texture_frame: i16,
    pub name: i16,
}

#[derive(Debug, Clone, Copy)]
pub struct SubSequence {
    pub first_node: i16,
    pub node_count: i16,
}

#[derive(Debug, Clone)]
pub struct Sequence {
    pub transform: Transform,
    /// -1 = nothing visible.
    pub mesh_subsequence: i16,
    /// -1 = none (always -1 in version 1 files).
    pub light_subsequence: i16,
}

#[derive(Debug, Clone)]
pub struct Loop {
    pub duration: f32,
    pub name: i16,
    pub first_keyframe: i16,
    pub keyframe_count: i16,
    pub first_sequence: i16,
    pub sequence_count: i16,
}

#[derive(Debug, Clone)]
pub struct LightNode {
    pub name: i16,
    pub flags: u8,
    pub color: [u8; 3],
    pub position: [f32; 3],
    pub attenuation_end: f32,
    pub attenuation_begin: f32,
    pub intensity_level: f32,
    pub intensity_scale: f32,
}

#[derive(Debug, Clone, Copy)]
pub struct Face {
    /// (vertex index, texture vertex index) per corner. Vertex indices are
    /// relative to the frame's first vertex.
    pub corners: [(i32, i32); 3],
    pub material: i32,
}

#[derive(Debug, Clone, Copy)]
pub struct MeshFrame {
    pub first_vertex: i32,
    pub scale: [f32; 3],
    pub origin: [f32; 3],
}

#[derive(Debug, Clone)]
pub struct Mesh {
    pub version: i32,
    pub vertices_per_frame: usize,
    /// 0 = the mesh has no texture coordinates.
    pub texture_vertices_per_frame: usize,
    pub radius: f32,
    /// x, y, z (0..=254) and the index of a normal direction.
    pub vertices: Vec<[u8; 4]>,
    pub texture_vertices: Vec<[f32; 2]>,
    pub faces: Vec<Face>,
    pub frames: Vec<MeshFrame>,
    /// True if `frames` holds one frame made up from the mesh-wide scale
    /// and origin, because the file had none (versions below 3).
    pub frame_was_implied: bool,
}

impl Mesh {
    fn read(bytes: &[u8], at: usize) -> Result<(Self, usize), ReadError> {
        let block = read_block(bytes, at)?;
        if block.class != MESH_CLASS {
            return Err(ReadError {
                offset: at,
                message: format!("expected class {MESH_CLASS}, found '{}'", block.class),
            });
        }
        let v = block.version;
        if !(1..=3).contains(&v) {
            return Err(ReadError {
                offset: at,
                message: format!("{MESH_CLASS} version {v} not supported (know 1 to 3)"),
            });
        }
        let mut r = Reader::new(bytes, block.data_offset);
        let vertex_count = r.count("vertex count")?;
        let vertices_per_frame = r.count("vertices per frame")?;
        let texture_vertex_count = r.count("texture vertex count")?;
        let face_count = r.count("face count")?;
        let frame_count = r.count("frame count")?;
        let texture_vertices_per_frame =
            if v >= 2 { r.count("texture vertices per frame")? } else { texture_vertex_count };
        let (mesh_scale, mesh_origin) = if v < 3 { (r.vec3()?, r.vec3()?) } else { ([1.0; 3], [0.0; 3]) };
        let radius = r.f32()?;
        let vertices = (0..vertex_count).map(|_| r.array::<4>()).collect::<Result<Vec<_>, _>>()?;
        let texture_vertices = (0..texture_vertex_count)
            .map(|_| Ok([r.f32()?, r.f32()?]))
            .collect::<Result<Vec<_>, ReadError>>()?;
        let mut faces = Vec::with_capacity(face_count);
        for _ in 0..face_count {
            let mut corners = [(0, 0); 3];
            for c in &mut corners {
                *c = (r.i32()?, r.i32()?);
            }
            faces.push(Face { corners, material: r.i32()? });
        }
        let mut frames = Vec::with_capacity(frame_count.max(1));
        for _ in 0..frame_count {
            let first_vertex = r.i32()?;
            let (scale, origin) = if v >= 3 { (r.vec3()?, r.vec3()?) } else { (mesh_scale, mesh_origin) };
            frames.push(MeshFrame { first_vertex, scale, origin });
        }
        let frame_was_implied = frame_count == 0 && v < 3;
        if frame_was_implied {
            frames.push(MeshFrame { first_vertex: 0, scale: mesh_scale, origin: mesh_origin });
        }
        let data_end = block.data_offset + block.data.len();
        if r.pos != data_end {
            return Err(r.error(format!(
                "mesh data ends at 0x{:X} but its block ends at 0x{data_end:X}",
                r.pos
            )));
        }
        Ok((
            Mesh {
                version: v,
                vertices_per_frame,
                texture_vertices_per_frame,
                radius,
                vertices,
                texture_vertices,
                faces,
                frames,
                frame_was_implied,
            },
            block.end,
        ))
    }

    /// Position of vertex `index` (relative to the frame) in mesh space.
    pub fn position(&self, frame: &MeshFrame, index: i32) -> Option<[f32; 3]> {
        let at = usize::try_from(frame.first_vertex.checked_add(index)?).ok()?;
        let v = self.vertices.get(at)?;
        Some(std::array::from_fn(|i| v[i] as f32 * frame.scale[i] + frame.origin[i]))
    }
}

#[derive(Debug, Clone)]
pub struct Shape {
    pub version: i32,
    pub radius: f32,
    pub center: [f32; 3],
    pub nodes: Vec<MeshNode>,
    pub subsequences: Vec<SubSequence>,
    pub sequences: Vec<Sequence>,
    pub keyframes: Vec<f32>,
    pub loops: Vec<Loop>,
    pub names: Vec<String>,
    pub lights: Vec<LightNode>,
    pub meshes: Vec<Mesh>,
    pub materials: Option<MaterialList>,
    /// Bytes inside the shape's block that were not read (expected 0).
    pub unread_bytes: usize,
    /// Bytes in the file after the shape's PERS block (expected 0).
    pub trailing_bytes: usize,
}

/// One triangle ready to draw, in model space.
#[derive(Debug, Clone, Copy)]
pub struct Triangle {
    pub positions: [[f32; 3]; 3],
    pub uvs: [[f32; 2]; 3],
    /// Index into the shape's material list.
    pub material: i32,
    /// Index of the mesh node it came from.
    pub node: usize,
}

/// Triangles of one sequence, plus anything that could not be built.
#[derive(Debug, Clone, Default)]
pub struct Geometry {
    pub triangles: Vec<Triangle>,
    pub problems: Vec<String>,
}

impl Shape {
    pub fn parse(bytes: &[u8]) -> Result<Self, ReadError> {
        let block = read_block(bytes, 0)?;
        if block.class != CLASS {
            return Err(ReadError {
                offset: 0,
                message: format!("expected class {CLASS}, found '{}'", block.class),
            });
        }
        let v = block.version;
        if !(1..=4).contains(&v) {
            return Err(ReadError {
                offset: 0,
                message: format!("{CLASS} version {v} not supported (know 1 to 4)"),
            });
        }
        let mut r = Reader::new(bytes, block.data_offset);
        let radius = r.f32()?;
        let center = r.vec3()?;
        let node_count = r.count("mesh node count")?;
        let subsequence_count = r.count("sub-sequence count")?;
        let sequence_count = r.count("sequence count")?;
        let keyframe_count = r.count("keyframe count")?;
        let loop_count = r.count("loop count")?;
        let name_count = r.count("name count")?;
        let mesh_count = r.count("mesh count")?;
        let light_count = if v >= 2 { r.count("light count")? } else { 0 };

        let mut nodes = Vec::with_capacity(node_count);
        for _ in 0..node_count {
            let transform = Transform::read(&mut r)?;
            nodes.push(MeshNode {
                transform,
                mesh: r.i16()?,
                vertex_frame: r.i16()?,
                texture_frame: r.i16()?,
                name: r.i16()?,
            });
        }
        let subsequences = (0..subsequence_count)
            .map(|_| Ok(SubSequence { first_node: r.i16()?, node_count: r.i16()? }))
            .collect::<Result<Vec<_>, ReadError>>()?;
        let mut sequences = Vec::with_capacity(sequence_count);
        for _ in 0..sequence_count {
            let transform = Transform::read(&mut r)?;
            let mesh_subsequence = r.i16()?;
            // Version 1 has 2 unused bytes here (random leftovers).
            let light_subsequence = if v >= 2 { r.i16()? } else { r.bytes(2).map(|_| -1)? };
            sequences.push(Sequence { transform, mesh_subsequence, light_subsequence });
        }
        let keyframes = (0..keyframe_count).map(|_| r.f32()).collect::<Result<Vec<_>, _>>()?;
        let mut loops = Vec::with_capacity(loop_count);
        for _ in 0..loop_count {
            let l = Loop {
                duration: r.f32()?,
                name: r.i16()?,
                first_keyframe: r.i16()?,
                keyframe_count: r.i16()?,
                first_sequence: r.i16()?,
                sequence_count: r.i16()?,
            };
            r.bytes(2)?; // unused
            loops.push(l);
        }
        let names = (0..name_count).map(|_| r.fixed_str(24)).collect::<Result<Vec<_>, _>>()?;
        let mut lights = Vec::with_capacity(light_count);
        for _ in 0..light_count {
            let name = r.i16()?;
            let flags = r.u8()?;
            let c = r.array::<3>()?;
            r.bytes(2)?; // unused
            let position = r.vec3()?;
            let attenuation_end = r.f32()?;
            let attenuation_begin = if v >= 3 { r.f32()? } else { 0.0 };
            let (intensity_level, intensity_scale) = if v >= 4 { (r.f32()?, r.f32()?) } else { (1.0, 1.0) };
            lights.push(LightNode {
                name,
                flags,
                color: c,
                position,
                attenuation_end,
                attenuation_begin,
                intensity_level,
                intensity_scale,
            });
        }
        let mut meshes = Vec::with_capacity(mesh_count);
        for _ in 0..mesh_count {
            let (mesh, end) = Mesh::read(bytes, r.pos)?;
            meshes.push(mesh);
            r.pos = end;
        }
        let has_materials = r.i32()?;
        let materials = if has_materials != 0 {
            let list = MaterialList::read_at(bytes, r.pos)?;
            r.pos = list.end;
            Some(list)
        } else {
            None
        };
        let data_end = block.data_offset + block.data.len();
        if r.pos > data_end {
            return Err(r.error(format!("shape contents run past its block end 0x{data_end:X}")));
        }
        Ok(Shape {
            version: v,
            radius,
            center,
            nodes,
            subsequences,
            sequences,
            keyframes,
            loops,
            names,
            lights,
            meshes,
            materials,
            unread_bytes: data_end - r.pos,
            trailing_bytes: bytes.len() - block.end,
        })
    }

    /// The sequence the model shows when it is not animating: the first
    /// sequence of the first loop (or sequence 0 if there are no loops).
    pub fn rest_sequence(&self) -> usize {
        self.loops
            .first()
            .map(|l| l.first_sequence.max(0) as usize)
            .filter(|&s| s < self.sequences.len())
            .unwrap_or(0)
    }

    /// Builds the triangles visible in sequence `index`, in model space:
    /// mesh vertex -> node transform -> sequence transform.
    pub fn geometry(&self, index: usize) -> Geometry {
        let mut g = Geometry::default();
        let Some(seq) = self.sequences.get(index) else {
            g.problems.push(format!("sequence {index} does not exist ({} sequences)", self.sequences.len()));
            return g;
        };
        if seq.mesh_subsequence < 0 {
            return g;
        }
        let Some(sub) = self.subsequences.get(seq.mesh_subsequence as usize) else {
            g.problems.push(format!("sequence {index}: sub-sequence {} missing", seq.mesh_subsequence));
            return g;
        };
        let first = sub.first_node.max(0) as usize;
        for node_index in first..first + sub.node_count.max(0) as usize {
            let Some(node) = self.nodes.get(node_index) else {
                g.problems.push(format!("mesh node {node_index} missing"));
                continue;
            };
            let Some(mesh) = usize::try_from(node.mesh).ok().and_then(|m| self.meshes.get(m)) else {
                g.problems.push(format!("node {node_index}: mesh {} missing", node.mesh));
                continue;
            };
            let Some(frame) = usize::try_from(node.vertex_frame).ok().and_then(|f| mesh.frames.get(f)) else {
                g.problems.push(format!("node {node_index}: vertex frame {} missing", node.vertex_frame));
                continue;
            };
            // Texture coordinates for this node start here; none if the
            // node or the mesh has no texture frame.
            let uv_base = (node.texture_frame >= 0 && mesh.texture_vertices_per_frame > 0)
                .then(|| node.texture_frame as usize * mesh.texture_vertices_per_frame);
            for (face_index, face) in mesh.faces.iter().enumerate() {
                let mut positions = [[0.0; 3]; 3];
                let mut uvs = [[0.0; 2]; 3];
                let mut ok = true;
                for (k, &(vi, ti)) in face.corners.iter().enumerate() {
                    match mesh.position(frame, vi) {
                        Some(p) => positions[k] = seq.transform.apply(node.transform.apply(p)),
                        None => ok = false,
                    }
                    if let Some(base) = uv_base {
                        match usize::try_from(ti).ok().and_then(|t| mesh.texture_vertices.get(base + t)) {
                            Some(uv) => uvs[k] = *uv,
                            None => ok = false,
                        }
                    }
                }
                if !ok {
                    g.problems.push(format!("node {node_index} face {face_index}: index out of range"));
                    continue;
                }
                g.triangles.push(Triangle { positions, uvs, material: face.material, node: node_index });
            }
        }
        g
    }
}

/// Smallest and largest corner of a set of triangles.
pub fn bounds(triangles: &[Triangle]) -> Option<([f32; 3], [f32; 3])> {
    let mut it = triangles.iter().flat_map(|t| t.positions);
    let first = it.next()?;
    Some(it.fold((first, first), |(lo, hi), p| {
        (std::array::from_fn(|i| lo[i].min(p[i])), std::array::from_fn(|i| hi[i].max(p[i])))
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pers::tests::block;

    fn i32s(v: &[i32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }
    fn f32s(v: &[f32]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }
    fn i16s(v: &[i16]) -> Vec<u8> {
        v.iter().flat_map(|x| x.to_le_bytes()).collect()
    }
    fn transform(scale: f32, p: [f32; 3]) -> Vec<u8> {
        let mut t = 6i32.to_le_bytes().to_vec();
        t.extend(f32s(&[scale, 0.0, 0.0, 0.0, scale, 0.0, 0.0, 0.0, scale]));
        t.extend(f32s(&p));
        t
    }

    /// One version-3 mesh: a single triangle, 1 frame of 5 vertices
    /// (2 bounding-box vertices + 3 real ones).
    fn mesh() -> Vec<u8> {
        let mut d = i32s(&[5, 5, 3, 1, 1, 3]);
        d.extend(f32s(&[10.0])); // radius
        for v in [[0u8, 0, 0, 0], [2, 2, 0, 0], [0, 0, 0, 1], [2, 0, 0, 1], [0, 2, 0, 1]] {
            d.extend(v);
        }
        d.extend(f32s(&[0.0, 0.0, 1.0, 0.0, 0.0, 1.0]));
        d.extend(i32s(&[2, 0, 3, 1, 4, 2, 0])); // face, material 0
        d.extend(i32s(&[0]));
        d.extend(f32s(&[0.5, 0.5, 0.5, 1.0, 1.0, 1.0])); // scale, origin
        block(MESH_CLASS, 3, &d)
    }

    fn shape() -> Vec<u8> {
        let mut d = f32s(&[1.0, 0.0, 0.0, 0.0]);
        d.extend(i32s(&[1, 1, 2, 1, 1, 1, 1, 0])); // counts, 0 lights
        d.extend(transform(2.0, [0.0, 0.0, 100.0]));
        d.extend(i16s(&[0, 0, 0, 0]));
        d.extend(i16s(&[0, 1]));
        for _ in 0..2 {
            d.extend(transform(1.0, [0.0; 3]));
            d.extend(i16s(&[0, -1]));
        }
        d.extend(f32s(&[0.0]));
        d.extend(f32s(&[0.0]));
        d.extend(i16s(&[0, 0, 1, 0, 2, 0]));
        let mut name = b"body\0".to_vec();
        name.resize(24, 0xCC);
        d.extend(name);
        d.extend(mesh());
        d.extend(i32s(&[1]));
        let mut ml = i32s(&[1, 1, 0x103, 0, 0, 0]);
        ml.extend([0u8; 16]);
        d.extend(block(crate::material::CLASS, 1, &ml));
        block(CLASS, 3, &d)
    }

    #[test]
    fn parses_and_builds_a_triangle() {
        let s = Shape::parse(&shape()).unwrap();
        assert_eq!(s.names, vec!["body"]);
        assert_eq!(s.meshes[0].faces.len(), 1);
        assert_eq!(s.materials.as_ref().unwrap().materials.len(), 1);
        assert_eq!(s.trailing_bytes, 0);
        let g = s.geometry(s.rest_sequence());
        assert!(g.problems.is_empty(), "{:?}", g.problems);
        let t = &g.triangles[0];
        // vertex (2,0,0) -> *0.5 + 1 = (2,1,1) -> node *2 + (0,0,100)
        assert_eq!(t.positions[1], [4.0, 2.0, 102.0]);
        assert_eq!(t.uvs[2], [0.0, 1.0]);
    }

    #[test]
    fn bad_face_index_is_reported_not_fatal() {
        let mut s = Shape::parse(&shape()).unwrap();
        s.meshes[0].faces[0].corners[0].0 = 99;
        let g = s.geometry(0);
        assert!(g.triangles.is_empty());
        assert_eq!(g.problems.len(), 1);
    }
}
