//! Renderer-neutral artistic value planning.
//!
//! This is intentionally separate from a medium.  It answers how much value a
//! surface should carry; graphite, ink and paint decide how to deposit it.

use crate::{NprGeometry, TopologyEdge, face_normal};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprValueSample {
    pub direct: f32,
    pub ambient_occlusion: f32,
    pub contact: f32,
    pub material_luma: f32,
    pub focal_weight: f32,
}

impl NprValueSample {
    /// A deliberately explicit first composition. Scene preparation can fill
    /// AO/contact later without changing any brush or value-grouping code.
    pub fn artistic_light(self) -> f32 {
        (self.direct * (1.0 - self.ambient_occlusion) * (1.0 - self.contact)
            + self.material_luma * 0.15)
            .clamp(0.0, 1.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprValueField {
    pub samples: Vec<NprValueSample>,
    pub values: Vec<f32>,
}

impl NprValueField {
    pub fn directional(geometry: &NprGeometry, light_direction: Vec3) -> Self {
        let light = light_direction.normalize_or_zero();
        let samples = geometry
            .triangles
            .iter()
            .enumerate()
            .map(|(face, _)| NprValueSample {
                direct: face_normal(geometry, face as u32).dot(light).max(0.0),
                ambient_occlusion: 0.0,
                contact: 0.0,
                material_luma: 0.0,
                focal_weight: 1.0,
            })
            .collect::<Vec<_>>();
        let values = samples.iter().map(|sample| sample.artistic_light()).collect();
        Self { samples, values }
    }

    /// Edge-aware graph smoothing. Values never cross a sharp normal crease;
    /// that keeps a building wall a single mass instead of polygon confetti.
    pub fn group_by_surface(
        &mut self,
        geometry: &NprGeometry,
        topology: &[TopologyEdge],
        iterations: u8,
        normal_similarity: f32,
    ) {
        let normals = geometry
            .triangles
            .iter()
            .enumerate()
            .map(|(face, _)| face_normal(geometry, face as u32))
            .collect::<Vec<_>>();
        let mut neighbours = vec![Vec::new(); geometry.triangles.len()];
        for edge in topology {
            if edge.faces[1] != u32::MAX {
                neighbours[edge.faces[0] as usize].push(edge.faces[1] as usize);
                neighbours[edge.faces[1] as usize].push(edge.faces[0] as usize);
            }
        }
        for _ in 0..iterations {
            let mut next = self.values.clone();
            for (face, adjacent) in neighbours.iter().enumerate() {
                let compatible = adjacent
                    .iter()
                    .copied()
                    .filter(|other| normals[face].dot(normals[*other]) >= normal_similarity)
                    .collect::<Vec<_>>();
                if !compatible.is_empty() {
                    let average = compatible.iter().map(|other| self.values[*other]).sum::<f32>()
                        / compatible.len() as f32;
                    next[face] = (self.values[face] * 2.0 + average) / 3.0;
                }
            }
            self.values = next;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn more_occlusion_never_increases_artistic_light() {
        let lit = NprValueSample { direct: 0.8, ambient_occlusion: 0.0, contact: 0.0, material_luma: 0.0, focal_weight: 1.0 };
        let occluded = NprValueSample { ambient_occlusion: 0.7, ..lit };
        assert!(occluded.artistic_light() <= lit.artistic_light());
    }
}
