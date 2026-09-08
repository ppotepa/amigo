use crate::geometry::NprGeometry;
use glam::Vec3;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TopologyEdge {
    pub a: u32,
    pub b: u32,
    pub faces: [u32; 2],
}

pub fn build_topology(geometry: &NprGeometry) -> Vec<TopologyEdge> {
    let mut edges: BTreeMap<(u32, u32), Vec<u32>> = BTreeMap::new();
    for (face, tri) in geometry.triangles.iter().enumerate() {
        for (a, b) in [(tri[0], tri[1]), (tri[1], tri[2]), (tri[2], tri[0])] {
            let key = if a < b { (a, b) } else { (b, a) };
            edges.entry(key).or_default().push(face as u32);
        }
    }
    edges
        .into_iter()
        .map(|((a, b), faces)| TopologyEdge {
            a,
            b,
            faces: [faces[0], *faces.get(1).unwrap_or(&u32::MAX)],
        })
        .collect()
}

pub fn face_normal(geometry: &NprGeometry, face: u32) -> Vec3 {
    let tri = geometry.triangles[face as usize];
    (geometry.vertices[tri[1] as usize].position - geometry.vertices[tri[0] as usize].position)
        .cross(
            geometry.vertices[tri[2] as usize].position
                - geometry.vertices[tri[0] as usize].position,
        )
        .normalize_or_zero()
}

/// Connected components of faces that form one planar drawing region. The
/// region key is deliberately derived from topology and normals, never from a
/// mesh name, so hatching can continue through triangulation diagonals without
/// crossing authored creases.
pub fn coplanar_face_groups(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    normal_dot_threshold: f32,
) -> Vec<u32> {
    let mut adjacency = vec![Vec::new(); geometry.triangles.len()];
    for edge in topology {
        let [a, b] = edge.faces;
        if a == u32::MAX || b == u32::MAX {
            continue;
        }
        if face_normal(geometry, a).dot(face_normal(geometry, b)) >= normal_dot_threshold {
            adjacency[a as usize].push(b as usize);
            adjacency[b as usize].push(a as usize);
        }
    }
    let mut groups = vec![u32::MAX; geometry.triangles.len()];
    let mut next_group = 0u32;
    for start in 0..groups.len() {
        if groups[start] != u32::MAX {
            continue;
        }
        let mut stack = vec![start];
        groups[start] = next_group;
        while let Some(face) = stack.pop() {
            for &neighbor in &adjacency[face] {
                if groups[neighbor] == u32::MAX {
                    groups[neighbor] = next_group;
                    stack.push(neighbor);
                }
            }
        }
        next_group = next_group.wrapping_add(1);
    }
    groups
}
