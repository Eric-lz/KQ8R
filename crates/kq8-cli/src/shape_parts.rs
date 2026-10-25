//! `kq8 shape-parts <file>`: prints a `.kqs` model's named parts, with
//! each part's full placement (matrix rows and translation), in the
//! first and last frame of every loop. Used to check where a dropped
//! weapon or an `Inv` item is laid (`fighting.md` rule 31a,
//! `anim-scripts.md` rule 16): the part named like the item, and the
//! item model's own part `body`.

use std::process::ExitCode;

use kq8_data::shape::Shape;
use kq8_data::GameDir;

pub fn run(game: &GameDir, file: &str) -> ExitCode {
    let bytes = match crate::source::load(game, file) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::FAILURE;
        }
    };
    let shape = match Shape::parse(&bytes) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: {file}: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("{file}: {} loops, {} sequences, rest sequence {}", shape.loops.len(), shape.sequences.len(), shape.rest_sequence());
    for l in &shape.loops {
        let name = usize::try_from(l.name).ok().and_then(|n| shape.names.get(n)).map_or("?", |n| n.as_str());
        let (first, count) = (l.first_sequence.max(0) as usize, l.sequence_count.max(0) as usize);
        println!("loop '{name}': sequences {first}..{}, {:.4} s", first + count, l.duration);
        let mut frames = vec![first];
        if count > 1 {
            frames.push(first + count - 1);
        }
        for seq in frames {
            println!("  sequence {seq}:");
            for (part, t, size) in parts(&shape, seq) {
                println!(
                    "    {part:<20} m [{:.3} {:.3} {:.3}] [{:.3} {:.3} {:.3}] [{:.3} {:.3} {:.3}] p ({:.1}, {:.1}, {:.1}); drawn size {:.0} x {:.0} x {:.0}",
                    t.m[0][0], t.m[0][1], t.m[0][2], t.m[1][0], t.m[1][1], t.m[1][2], t.m[2][0], t.m[2][1], t.m[2][2], t.p[0], t.p[1], t.p[2], size[0], size[1], size[2]
                );
            }
        }
    }
    ExitCode::SUCCESS
}

/// Each named part of `seq` with its placement and the size of its mesh
/// as drawn in the model (the box of its vertices after the placement's
/// matrix, along the model's axes).
fn parts(shape: &Shape, seq: usize) -> Vec<(String, kq8_data::shape::Transform, [f32; 3])> {
    let Some(sub) = shape.sequences.get(seq).and_then(|s| usize::try_from(s.mesh_subsequence).ok()).and_then(|k| shape.subsequences.get(k)) else {
        return Vec::new();
    };
    let nodes = shape.nodes.iter().skip(sub.first_node.max(0) as usize).take(sub.node_count.max(0) as usize);
    nodes
        .filter_map(|n| {
            let name = shape.names.get(usize::try_from(n.name).ok()?)?.clone();
            let mesh = shape.meshes.get(usize::try_from(n.mesh).ok()?)?;
            let frame = mesh.frames.get(n.vertex_frame.max(0) as usize).or(mesh.frames.first())?;
            let (mut lo, mut hi) = ([f32::MAX; 3], [f32::MIN; 3]);
            for i in 0..mesh.vertices_per_frame as i32 {
                let v = mesh.position(frame, i)?;
                let m = n.transform.m;
                let w: [f32; 3] = std::array::from_fn(|j| v[0] * m[0][j] + v[1] * m[1][j] + v[2] * m[2][j]);
                for k in 0..3 {
                    lo[k] = lo[k].min(w[k]);
                    hi[k] = hi[k].max(w[k]);
                }
            }
            Some((name, n.transform, std::array::from_fn(|k| hi[k] - lo[k])))
        })
        .filter(|(name, _, _)| !name.is_empty())
        .collect()
}
