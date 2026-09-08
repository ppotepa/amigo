//! Crease-aware, fixed-level subdivision used to prepare a smooth drawing proxy.
//!
//! The proxy is prepared per surface revision and policy; it is never selected
//! from camera distance.  The implementation follows Loop's edge/vertex masks
//! on smooth regions while preserving vertices touching a hard or boundary edge.

use crate::{NprGeometry, NprVertex, build_topology, face_normal};
use glam::Vec3;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NprSubdivisionError {
    TriangleBudget { requested: usize, maximum: usize },
}

impl std::fmt::Display for NprSubdivisionError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TriangleBudget { requested, maximum } => write!(
                formatter,
                "NPR smooth proxy requires {requested} triangles, exceeding its {maximum} triangle budget"
            ),
        }
    }
}

/// Generates a fixed-level drawing proxy. `crease_angle` is the same semantic
/// boundary used by smooth normal interpolation: faces separated by a sharper
/// dihedral are never blended.
pub fn subdivide_smooth_proxy(
    source: &NprGeometry,
    levels: u8,
    crease_angle: f32,
    max_triangles: usize,
) -> Result<NprGeometry, NprSubdivisionError> {
    let mut requested = source.triangles.len();
    for _ in 0..levels {
        requested = requested.saturating_mul(4);
    }
    if requested > max_triangles {
        return Err(NprSubdivisionError::TriangleBudget {
            requested,
            maximum: max_triangles,
        });
    }
    let mut current = source.clone();
    let crease_cos = crease_angle.clamp(0.0, std::f32::consts::PI).cos();
    for _ in 0..levels {
        current = subdivide_once(&current, crease_cos);
    }
    Ok(current)
}

fn subdivide_once(source: &NprGeometry, crease_cos: f32) -> NprGeometry {
    let topology = build_topology(source);
    let mut smooth_neighbors = vec![BTreeSet::new(); source.vertices.len()];
    let mut hard_vertex = vec![false; source.vertices.len()];
    let mut edge_midpoints = BTreeMap::new();
    let mut vertices: Vec<NprVertex> = source.vertices.clone();

    for edge in &topology {
        let [left, right] = edge.faces;
        let smooth = right != u32::MAX
            && face_normal(source, left).dot(face_normal(source, right)) >= crease_cos;
        if smooth {
            smooth_neighbors[edge.a as usize].insert(edge.b);
            smooth_neighbors[edge.b as usize].insert(edge.a);
        } else {
            hard_vertex[edge.a as usize] = true;
            hard_vertex[edge.b as usize] = true;
        }
    }

    // Loop's vertex mask is valid only entirely inside one smooth region.
    // Keeping vertices on a hard/boundary edge fixed avoids rounding a cube
    // corner or welding distinct authored patches together.
    for (index, vertex) in source.vertices.iter().enumerate() {
        if hard_vertex[index] {
            continue;
        }
        let neighbors = &smooth_neighbors[index];
        let count = neighbors.len();
        if count < 3 {
            continue;
        }
        let beta = if count == 3 {
            3.0 / 16.0
        } else {
            3.0 / (8.0 * count as f32)
        };
        let sum = neighbors.iter().fold(Vec3::ZERO, |sum, neighbor| {
            sum + source.vertices[*neighbor as usize].position
        });
        vertices[index].position = vertex.position * (1.0 - count as f32 * beta) + sum * beta;
    }

    for edge in &topology {
        let a = source.vertices[edge.a as usize].position;
        let b = source.vertices[edge.b as usize].position;
        let [left, right] = edge.faces;
        let smooth = right != u32::MAX
            && face_normal(source, left).dot(face_normal(source, right)) >= crease_cos;
        let position = if smooth {
            let opposite_left = opposite_vertex(source.triangles[left as usize], edge.a, edge.b);
            let opposite_right = opposite_vertex(source.triangles[right as usize], edge.a, edge.b);
            a * (3.0 / 8.0)
                + b * (3.0 / 8.0)
                + source.vertices[opposite_left as usize].position * (1.0 / 8.0)
                + source.vertices[opposite_right as usize].position * (1.0 / 8.0)
        } else {
            (a + b) * 0.5
        };
        let index = vertices.len() as u32;
        vertices.push(NprVertex { position });
        edge_midpoints.insert((edge.a, edge.b), index);
    }

    let midpoint = |a: u32, b: u32| edge_midpoints[&ordered_edge(a, b)];
    let mut triangles = Vec::with_capacity(source.triangles.len().saturating_mul(4));
    for [a, b, c] in &source.triangles {
        let ab = midpoint(*a, *b);
        let bc = midpoint(*b, *c);
        let ca = midpoint(*c, *a);
        triangles.extend([[*a, ab, ca], [ab, *b, bc], [ca, bc, *c], [ab, bc, ca]]);
    }
    NprGeometry {
        vertices,
        triangles,
    }
}

fn ordered_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

fn opposite_vertex(triangle: [u32; 3], a: u32, b: u32) -> u32 {
    triangle
        .into_iter()
        .find(|index| *index != a && *index != b)
        .expect("topology edge belongs to its recorded triangle")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cube_keeps_hard_corners_and_quadruples_triangle_count() {
        let cube = NprGeometry::canonical_cube();
        let proxy = subdivide_smooth_proxy(&cube, 1, 1.2, 1_000).unwrap();
        assert_eq!(proxy.triangles.len(), cube.triangles.len() * 4);
        assert_eq!(proxy.vertices[0].position, cube.vertices[0].position);
    }

    #[test]
    fn smooth_sphere_moves_an_interior_vertex() {
        let sphere = NprGeometry::icosphere();
        let proxy = subdivide_smooth_proxy(&sphere, 1, 1.2, 1_000).unwrap();
        assert_ne!(proxy.vertices[0].position, sphere.vertices[0].position);
    }

    #[test]
    fn proxy_refuses_budget_before_allocating() {
        let error = subdivide_smooth_proxy(&NprGeometry::icosphere(), 3, 1.2, 100).unwrap_err();
        assert!(matches!(error, NprSubdivisionError::TriangleBudget { .. }));
    }
}
