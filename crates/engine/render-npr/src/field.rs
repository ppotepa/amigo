//! Stable surface-space directions used by form-following mark planners.

use crate::{NprGeometry, face_normal};
use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprSurfaceDirection {
    pub tangent: Vec3,
    pub confidence: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct NprSurfaceDirectionField {
    directions: Vec<NprSurfaceDirection>,
}

impl NprSurfaceDirectionField {
    pub fn from_geometry(geometry: &NprGeometry) -> Self {
        let directions = geometry
            .triangles
            .iter()
            .enumerate()
            .map(|(face, _)| {
                let normal = face_normal(geometry, face as u32);
                // Choose the least parallel world reference, yielding a
                // deterministic tangent even for assets without UVs.
                let reference = if normal.y.abs() < 0.85 { Vec3::Y } else { Vec3::X };
                let tangent = reference.cross(normal).normalize_or_zero();
                NprSurfaceDirection { tangent, confidence: tangent.length() }
            })
            .collect();
        Self { directions }
    }

    pub fn direction(&self, face: usize) -> Option<NprSurfaceDirection> {
        self.directions.get(face).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generated_tangents_are_perpendicular_to_face_normals() {
        let geometry = NprGeometry::canonical_cube();
        let field = NprSurfaceDirectionField::from_geometry(&geometry);
        for (face, _) in geometry.triangles.iter().enumerate() {
            assert!(field.direction(face).unwrap().tangent.dot(face_normal(&geometry, face as u32)).abs() < 0.001);
        }
    }
}
