use crate::{
    geometry::NprGeometry,
    topology::{TopologyEdge, face_normal},
};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FeatureClass {
    Boundary,
    Silhouette,
    Crease,
}

impl Default for FeatureClass {
    fn default() -> Self {
        Self::Boundary
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FeatureSegment {
    pub edge: TopologyEdge,
    pub class: FeatureClass,
    pub midpoint: Vec3,
}

pub fn classify_features(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    view_direction: Vec3,
    crease_angle: f32,
) -> Vec<FeatureSegment> {
    let view = view_direction.normalize_or_zero();
    topology
        .iter()
        .filter_map(|edge| {
            let first = face_normal(geometry, edge.faces[0]);
            let midpoint = (geometry.vertices[edge.a as usize].position
                + geometry.vertices[edge.b as usize].position)
                * 0.5;
            let class = if edge.faces[1] == u32::MAX {
                FeatureClass::Boundary
            } else {
                let second = face_normal(geometry, edge.faces[1]);
                let facing_a = first.dot(view) >= 0.0;
                let facing_b = second.dot(view) >= 0.0;
                if facing_a != facing_b {
                    FeatureClass::Silhouette
                } else if first.dot(second) < crease_angle.cos() {
                    FeatureClass::Crease
                } else {
                    return None;
                }
            };
            Some(FeatureSegment {
                edge: *edge,
                class,
                midpoint,
            })
        })
        .collect()
}
