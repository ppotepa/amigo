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
