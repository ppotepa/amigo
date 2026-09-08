//! Surface-direction fields used to guide form hatching.
//!
//! This first field smooths only across sufficiently smooth topology edges.
//! It deliberately preserves authored creases and stays independent of model
//! names, renderer state and frame number.

use crate::{NprGeometry, PerspectiveCamera, TopologyEdge, face_normal};
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceDirectionField {
    face_normals: Vec<Vec3>,
    smoothed_normals: Vec<Vec3>,
    form_axis: Vec3,
    face_tangents: Vec<Vec3>,
    face_confidence: Vec<f32>,
    face_curvature: Vec<f32>,
}

impl SurfaceDirectionField {
    /// Builds a one-ring normal field. `smooth_dot_threshold` is an explicit
    /// drawing decision: pairs below it are hard form boundaries, never a
    /// convenient route for hatching to leak through.
    pub fn build(
        geometry: &NprGeometry,
        topology: &[TopologyEdge],
        smooth_dot_threshold: f32,
    ) -> Self {
        let face_normals: Vec<_> = (0..geometry.triangles.len())
            .map(|face| face_normal(geometry, face as u32))
            .collect();
        let mut sums = face_normals.clone();
        for edge in topology {
            let [a, b] = edge.faces;
            if a == u32::MAX || b == u32::MAX {
                continue;
            }
            let a_index = a as usize;
            let b_index = b as usize;
            if face_normals[a_index].dot(face_normals[b_index]) >= smooth_dot_threshold {
                sums[a_index] += face_normals[b_index];
                sums[b_index] += face_normals[a_index];
            }
        }
        let smoothed_normals: Vec<Vec3> = sums
            .into_iter()
            .zip(&face_normals)
            .map(|(sum, fallback)| {
                let normal = sum.normalize_or_zero();
                if normal.length_squared() > 1e-8 {
                    normal
                } else {
                    *fallback
                }
            })
            .collect();
        let form_axis = principal_extent_axis(geometry);
        // Conservative one-ring normal-turn estimate. Hard edges remain zero:
        // a gesture stops there rather than treating a crease as a smooth form.
        let mut face_curvature = vec![0.0f32; face_normals.len()];
        for edge in topology {
            let [a, b] = edge.faces;
            if a == u32::MAX || b == u32::MAX {
                continue;
            }
            let a = a as usize;
            let b = b as usize;
            let normal_dot = face_normals[a].dot(face_normals[b]);
            if normal_dot >= smooth_dot_threshold {
                let turn = (1.0 - normal_dot).clamp(0.0, 1.0);
                face_curvature[a] = face_curvature[a].max(turn);
                face_curvature[b] = face_curvature[b].max(turn);
            }
        }
        let mut face_confidence = Vec::with_capacity(face_normals.len());
        let mut tangent_sums = Vec::with_capacity(face_normals.len());
        for normal in &smoothed_normals {
            let projected = form_axis - *normal * form_axis.dot(*normal);
            let confidence = projected.length().clamp(0.0, 1.0);
            face_confidence.push(confidence);
            tangent_sums.push(projected.normalize_or_zero());
        }
        // Transport local tangent direction across smooth edges. Signs are
        // aligned before accumulation, so no arbitrary cancellation appears at
        // a shared triangulation edge.
        for edge in topology {
            let [a, b] = edge.faces;
            if a == u32::MAX || b == u32::MAX {
                continue;
            }
            let a = a as usize;
            let b = b as usize;
            if face_normals[a].dot(face_normals[b]) < smooth_dot_threshold
                || face_confidence[a] <= 1e-5
                || face_confidence[b] <= 1e-5
            {
                continue;
            }
            let sign = if tangent_sums[a].dot(tangent_sums[b]) < 0.0 {
                -1.0
            } else {
                1.0
            };
            let tangent_a = tangent_sums[a];
            let tangent_b = tangent_sums[b];
            tangent_sums[a] += tangent_b * sign;
            tangent_sums[b] += tangent_a * sign;
        }
        let face_tangents = tangent_sums
            .into_iter()
            .zip(&smoothed_normals)
            .map(|(tangent, normal)| {
                let tangent = tangent.normalize_or_zero();
                if tangent.length_squared() > 1e-8 {
                    tangent
                } else {
                    (Vec3::X - *normal * Vec3::X.dot(*normal)).normalize_or_zero()
                }
            })
            .collect();
        Self {
            face_normals,
            smoothed_normals,
            form_axis,
            face_tangents,
            face_confidence,
            face_curvature,
        }
    }

    pub fn face_normal(&self, face: usize) -> Vec3 {
        self.face_normals[face]
    }

    pub fn smoothed_normal(&self, face: usize) -> Vec3 {
        self.smoothed_normals[face]
    }

    /// Dominant geometric extent. It is a stable, geometry-derived fallback
    /// direction for tonal paths on elongated forms such as a cylinder or a
    /// limb; symmetric forms retain the deterministic world-up seed.
    pub fn form_axis(&self) -> Vec3 {
        self.form_axis
    }

    /// Locally transported tangent aligned to the inferred form axis.
    pub fn face_tangent(&self, face: usize) -> Vec3 {
        self.face_tangents[face]
    }

    /// Zero means the form axis is locally normal to the surface, so a tangent
    /// direction is intrinsically ambiguous and should contribute less tone.
    pub fn face_confidence(&self, face: usize) -> f32 {
        self.face_confidence[face]
    }

    /// Smooth local normal turn used to shorten a gesture near rapid form
    /// changes. It never replaces the geometric direction field.
    pub fn face_curvature(&self, face: usize) -> f32 {
        self.face_curvature[face]
    }

    /// Returns a screen-space tangent for hatch lines. The direction comes
    /// from the smoothed surface field, then is projected back to the actual
    /// triangle tangent plane so the local clipping operation remains valid.
    pub fn projected_direction(
        &self,
        face: usize,
        camera: PerspectiveCamera,
        anchor: Vec3,
        viewport: Vec2,
    ) -> Vec2 {
        let normal = self.face_normals[face];
        let smoothed = self.smoothed_normals[face];
        let candidate = camera.forward.cross(smoothed);
        let tangent = (candidate - normal * candidate.dot(normal)).normalize_or_zero();
        let tangent = if tangent.length_squared() > 1e-8 {
            tangent
        } else {
            // A front-facing region has no preferred view contour direction.
            // Object up is stable under camera zoom and triangle ordering.
            let up = Vec3::Y - normal * Vec3::Y.dot(normal);
            let up = up.normalize_or_zero();
            if up.length_squared() > 1e-8 {
                up
            } else {
                (Vec3::X - normal * Vec3::X.dot(normal)).normalize_or_zero()
            }
        };
        let fallback = Vec2::new(1.0, 0.0);
        let Some(start) = camera.project(anchor, viewport) else {
            return fallback;
        };
        let Some(end) = camera.project(anchor + tangent * 0.25, viewport) else {
            return fallback;
        };
        let direction = (end.screen - start.screen).normalize_or_zero();
        if direction.length_squared() > 1e-8 {
            direction
        } else {
            fallback
        }
    }
}

fn principal_extent_axis(geometry: &NprGeometry) -> Vec3 {
    if geometry.vertices.len() < 2 {
        return Vec3::Y;
    }
    let center = geometry
        .vertices
        .iter()
        .map(|vertex| vertex.position)
        .sum::<Vec3>()
        / geometry.vertices.len() as f32;
    let mut covariance = [[0.0f32; 3]; 3];
    for vertex in &geometry.vertices {
        let delta = vertex.position - center;
        let values = [delta.x, delta.y, delta.z];
        for row in 0..3 {
            for column in 0..3 {
                covariance[row][column] += values[row] * values[column];
            }
        }
    }
    // Power iteration for the principal eigenvector of a symmetric covariance
    // matrix. The fixed seed makes spheres/cubes deterministic rather than
    // introducing an arbitrary triangle-order choice.
    let mut axis = Vec3::new(0.37, 0.71, 0.59).normalize();
    for _ in 0..32 {
        let next = Vec3::new(
            covariance[0][0] * axis.x + covariance[0][1] * axis.y + covariance[0][2] * axis.z,
            covariance[1][0] * axis.x + covariance[1][1] * axis.y + covariance[1][2] * axis.z,
            covariance[2][0] * axis.x + covariance[2][1] * axis.y + covariance[2][2] * axis.z,
        );
        if next.length_squared() <= 1e-10 {
            break;
        }
        axis = next.normalize();
    }
    if axis.length_squared() <= 1e-8 {
        Vec3::Y
    } else {
        let energy = Vec3::new(
            covariance[0][0] * axis.x + covariance[0][1] * axis.y + covariance[0][2] * axis.z,
            covariance[1][0] * axis.x + covariance[1][1] * axis.y + covariance[1][2] * axis.z,
            covariance[2][0] * axis.x + covariance[2][1] * axis.y + covariance[2][2] * axis.z,
        )
        .dot(axis);
        let average_energy = (covariance[0][0] + covariance[1][1] + covariance[2][2]) / 3.0;
        // Symmetric primitives have no meaningful extent axis. Keeping world
        // up makes their output familiar and avoids a numerical pseudo-axis.
        if energy <= average_energy * 1.03 {
            Vec3::Y
        } else {
            axis
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NprGeometry, build_topology};

    #[test]
    fn cube_field_does_not_blur_across_authored_corners() {
        let geometry = NprGeometry::canonical_cube();
        let field = SurfaceDirectionField::build(&geometry, &build_topology(&geometry), 0.85);
        for face in 0..geometry.triangles.len() {
            assert!(field.face_normal(face).dot(field.smoothed_normal(face)) > 0.9999);
        }
    }

    #[test]
    fn smooth_neighbors_receive_a_more_continuous_normal() {
        let geometry = NprGeometry::cylinder(12);
        let topology = build_topology(&geometry);
        let field = SurfaceDirectionField::build(&geometry, &topology, 0.85);
        let edge = topology
            .iter()
            .find(|edge| {
                edge.faces[1] != u32::MAX
                    && field
                        .face_normal(edge.faces[0] as usize)
                        .dot(field.face_normal(edge.faces[1] as usize))
                        >= 0.85
                    && field
                        .face_normal(edge.faces[0] as usize)
                        .dot(field.face_normal(edge.faces[1] as usize))
                        < 0.9999
            })
            .unwrap();
        let raw = field
            .face_normal(edge.faces[0] as usize)
            .dot(field.face_normal(edge.faces[1] as usize));
        let smoothed = field
            .smoothed_normal(edge.faces[0] as usize)
            .dot(field.smoothed_normal(edge.faces[1] as usize));
        assert!(smoothed > raw);
    }

    #[test]
    fn form_axis_follows_a_rotated_elongated_model() {
        let geometry = NprGeometry::cylinder(24);
        let axis =
            SurfaceDirectionField::build(&geometry, &build_topology(&geometry), 0.85).form_axis();
        assert!(axis.dot(Vec3::Y).abs() > 0.9);
        let rotated =
            geometry.transformed(glam::Mat4::from_rotation_z(std::f32::consts::FRAC_PI_2));
        let axis =
            SurfaceDirectionField::build(&rotated, &build_topology(&rotated), 0.85).form_axis();
        assert!(axis.dot(Vec3::X).abs() > 0.9);
    }

    #[test]
    fn cylinder_side_has_a_confident_tangent_while_cap_is_ambiguous() {
        let geometry = NprGeometry::cylinder(12);
        let field = SurfaceDirectionField::build(&geometry, &build_topology(&geometry), 0.85);
        let side = (0..geometry.triangles.len())
            .find(|face| field.face_normal(*face).dot(Vec3::Y).abs() < 0.1)
            .unwrap();
        let cap = (0..geometry.triangles.len())
            .find(|face| field.face_normal(*face).dot(Vec3::Y).abs() > 0.9)
            .unwrap();
        assert!(field.face_tangent(side).dot(field.face_normal(side)).abs() < 1e-4);
        assert!(field.face_confidence(side) > 0.9);
        assert!(field.face_confidence(cap) < 0.1);
    }

    #[test]
    fn curvature_tracks_smooth_cylinder_turns_without_rounding_cube_creases() {
        let cylinder = NprGeometry::cylinder(24);
        let cylinder_field =
            SurfaceDirectionField::build(&cylinder, &build_topology(&cylinder), 0.85);
        assert!(
            (0..cylinder.triangles.len()).any(|face| cylinder_field.face_curvature(face) > 0.01)
        );
        let cube = NprGeometry::canonical_cube();
        let cube_field = SurfaceDirectionField::build(&cube, &build_topology(&cube), 0.85);
        assert!((0..cube.triangles.len()).all(|face| cube_field.face_curvature(face) < 1e-5));
    }
}
