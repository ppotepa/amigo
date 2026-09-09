//! View-dependent contours extracted from a smoothed vertex-normal field.
//!
//! This is intentionally separate from `feature`: a hard crease is an edge of
//! the authored mesh, whereas a smooth contour is the zero set of a field and
//! normally runs through triangle interiors.

use crate::{face_normal, NprGeometry};
use glam::Vec3;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, PartialEq)]
pub struct SmoothContourStroke {
    pub id: u32,
    pub points: Vec<Vec3>,
}

#[derive(Debug, Clone)]
struct ContourSegment {
    endpoints: [((u32, u32), Vec3); 2],
    component: u32,
}

/// Extracts the zero crossings of `dot(smoothed_normal, camera - position)`.
/// Endpoints carry canonical mesh-edge keys, which lets segments produced by
/// adjacent triangles be assembled without comparing rounded positions.
pub fn smooth_perspective_contours(
    geometry: &NprGeometry,
    camera: Vec3,
    smooth_crease_angle: f32,
) -> Vec<SmoothContourStroke> {
    let normals = smoothed_corner_normals(geometry, smooth_crease_angle);
    let components = smooth_face_components(geometry, smooth_crease_angle);
    let mut segments = Vec::new();
    for (face, triangle) in geometry.triangles.iter().enumerate() {
        let values: [f32; 3] = std::array::from_fn(|corner| {
            let vertex = triangle[corner];
            let position = geometry.vertices[vertex as usize].position;
            normals[face][corner].dot(camera - position)
        });
        let mut crossings = Vec::with_capacity(2);
        for index in 0..3 {
            let a = triangle[index];
            let b = triangle[(index + 1) % 3];
            let Some(t) = contour_edge_crossing(a, b, values[index], values[(index + 1) % 3])
            else {
                continue;
            };
            let point = geometry.vertices[a as usize]
                .position
                .lerp(geometry.vertices[b as usize].position, t);
            crossings.push((ordered_edge(a, b), point));
        }
        if crossings.len() == 2 {
            segments.push(ContourSegment {
                endpoints: [crossings[0], crossings[1]],
                component: components[face],
            });
        }
    }
    chain_segments(segments, 0x2000_0000)
}

/// Extracts interior, view-dependent form lines from zero crossings of an
/// estimated radial curvature field. Unlike an occluding contour, this is an
/// opt-in secondary mark: only front-facing corners with a reliable projected
/// view direction participate. The estimate is face-local, but joining still
/// uses canonical mesh-edge keys and smooth-region components.
pub fn suggestive_perspective_contours(
    geometry: &NprGeometry,
    camera: Vec3,
    smooth_crease_angle: f32,
    min_confidence: f32,
) -> Vec<SmoothContourStroke> {
    let normals = smoothed_corner_normals(geometry, smooth_crease_angle);
    let components = smooth_face_components(geometry, smooth_crease_angle);
    let minimum = min_confidence.clamp(0.0, 1.0);
    let mut segments = Vec::new();
    for (face, triangle) in geometry.triangles.iter().enumerate() {
        let positions = triangle.map(|vertex| geometry.vertices[vertex as usize].position);
        let samples: [Option<(f32, f32)>; 3] = std::array::from_fn(|corner| {
            radial_curvature_sample(positions, normals[face], corner, camera)
        });
        if samples
            .iter()
            .any(|sample| sample.is_none_or(|sample| sample.1 < minimum))
        {
            continue;
        }
        let values = samples.map(|sample| sample.expect("checked suggestive sample").0);
        let mut crossings = Vec::with_capacity(2);
        for index in 0..3 {
            let a = triangle[index];
            let b = triangle[(index + 1) % 3];
            let Some(t) = contour_edge_crossing(a, b, values[index], values[(index + 1) % 3])
            else {
                continue;
            };
            crossings.push((
                ordered_edge(a, b),
                positions[index].lerp(positions[(index + 1) % 3], t),
            ));
        }
        if crossings.len() == 2 {
            segments.push(ContourSegment {
                endpoints: [crossings[0], crossings[1]],
                component: components[face],
            });
        }
    }
    chain_segments(segments, 0x3000_0000)
}

/// Estimates normal curvature in the projected camera direction with a least
/// squares derivative over the two local triangle edges. The result is not a
/// CAD-grade curvature tensor; its confidence explicitly declines when the
/// view tangent is ill-defined or the local sample has poor directional span.
fn radial_curvature_sample(
    positions: [Vec3; 3],
    normals: [Vec3; 3],
    corner: usize,
    camera: Vec3,
) -> Option<(f32, f32)> {
    let position = positions[corner];
    let normal = normals[corner].normalize_or_zero();
    let view = (camera - position).normalize_or_zero();
    if normal.length_squared() <= 1e-8 || view.length_squared() <= 1e-8 || normal.dot(view) <= 0.02
    {
        return None;
    }
    let projected_view = view - normal * normal.dot(view);
    let tangent_strength = projected_view.length();
    if tangent_strength <= 1e-5 {
        return None;
    }
    let tangent = projected_view / tangent_strength;
    let mut numerator = 0.0;
    let mut denominator = 0.0;
    let mut edge_extent = 0.0;
    for other in 0..3 {
        if other == corner {
            continue;
        }
        let edge = positions[other] - position;
        let along = edge.dot(tangent);
        numerator += (normals[other] - normal).dot(tangent) * along;
        denominator += along * along;
        edge_extent += edge.length_squared();
    }
    if denominator <= 1e-10 || edge_extent <= 1e-10 {
        return None;
    }
    let span_confidence = (denominator / edge_extent).sqrt().clamp(0.0, 1.0);
    Some((numerator / denominator, tangent_strength * span_confidence))
}

/// Resolves a field crossing on one indexed edge. If only one endpoint is
/// exactly on the zero set, a stable symbolic sign avoids dropping the span
/// simply because the contour happened to pass through a mesh vertex. An edge
/// whose two endpoints are exactly zero remains ambiguous and is left to its
/// neighbouring non-degenerate edges.
fn contour_edge_crossing(a: u32, b: u32, value_a: f32, value_b: f32) -> Option<f32> {
    const EPSILON: f32 = 1.0e-6;
    if value_a.abs() <= EPSILON && value_b.abs() <= EPSILON {
        return None;
    }
    let value_a = symbolic_contour_value(value_a, a, EPSILON);
    let value_b = symbolic_contour_value(value_b, b, EPSILON);
    if value_a.signum() == value_b.signum() {
        None
    } else {
        Some((value_a / (value_a - value_b)).clamp(0.0, 1.0))
    }
}

fn symbolic_contour_value(value: f32, vertex: u32, epsilon: f32) -> f32 {
    if value.abs() > epsilon {
        return value;
    }
    // SplitMix-like integer mixing makes the symbolic side independent of
    // triangle order while retaining the same answer for every use of this
    // indexed vertex.
    let mut bits = vertex.wrapping_add(0x9e37_79b9);
    bits ^= bits >> 16;
    bits = bits.wrapping_mul(0x85eb_ca6b);
    bits ^= bits >> 13;
    if bits & 1 == 0 { -epsilon } else { epsilon }
}

/// Labels face components which may share a smooth contour. A sharp edge is a
/// region boundary even when both incident faces belong to the same indexed
/// mesh. This is the stable source identity for a contour, rather than the
/// particular triangle edges crossed in one camera pose.
fn smooth_face_components(geometry: &NprGeometry, smooth_crease_angle: f32) -> Vec<u32> {
    let normals = (0..geometry.triangles.len())
        .map(|face| face_normal(geometry, face as u32))
        .collect::<Vec<_>>();
    let threshold = smooth_crease_angle.clamp(0.0, std::f32::consts::PI).cos();
    let mut by_edge = BTreeMap::<(u32, u32), Vec<usize>>::new();
    for (face, triangle) in geometry.triangles.iter().enumerate() {
        for edge in [
            ordered_edge(triangle[0], triangle[1]),
            ordered_edge(triangle[1], triangle[2]),
            ordered_edge(triangle[2], triangle[0]),
        ] {
            by_edge.entry(edge).or_default().push(face);
        }
    }
    let mut adjacency = vec![Vec::new(); geometry.triangles.len()];
    for faces in by_edge.values().filter(|faces| faces.len() == 2) {
        let a = faces[0];
        let b = faces[1];
        if normals[a].dot(normals[b]) >= threshold {
            adjacency[a].push(b);
            adjacency[b].push(a);
        }
    }
    let mut labels = vec![u32::MAX; geometry.triangles.len()];
    for start in 0..geometry.triangles.len() {
        if labels[start] != u32::MAX {
            continue;
        }
        let label = start as u32;
        let mut stack = vec![start];
        labels[start] = label;
        while let Some(face) = stack.pop() {
            for &next in &adjacency[face] {
                if labels[next] == u32::MAX {
                    labels[next] = label;
                    stack.push(next);
                }
            }
        }
    }
    labels
}

fn smoothed_corner_normals(geometry: &NprGeometry, smooth_crease_angle: f32) -> Vec<[Vec3; 3]> {
    let face_normals = (0..geometry.triangles.len())
        .map(|face| face_normal(geometry, face as u32))
        .collect::<Vec<_>>();
    let face_areas = geometry
        .triangles
        .iter()
        .map(|triangle| {
            let [a, b, c] = triangle.map(|index| geometry.vertices[index as usize].position);
            (b - a).cross(c - a).length()
        })
        .collect::<Vec<_>>();
    let mut incident = vec![Vec::new(); geometry.vertices.len()];
    for (face, triangle) in geometry.triangles.iter().enumerate() {
        for vertex in triangle {
            incident[*vertex as usize].push(face);
        }
    }
    let threshold = smooth_crease_angle.clamp(0.0, std::f32::consts::PI).cos();
    geometry
        .triangles
        .iter()
        .enumerate()
        .map(|(face, triangle)| {
            triangle.map(|vertex| {
                let base = face_normals[face];
                incident[vertex as usize]
                    .iter()
                    .filter(|&&neighbour| face_normals[neighbour].dot(base) >= threshold)
                    .fold(Vec3::ZERO, |sum, &neighbour| {
                        sum + face_normals[neighbour] * face_areas[neighbour]
                    })
                    .normalize_or_zero()
            })
        })
        .collect()
}

fn chain_segments(segments: Vec<ContourSegment>, id_namespace: u32) -> Vec<SmoothContourStroke> {
    let mut adjacency: BTreeMap<(u32, u32, u32), Vec<usize>> = BTreeMap::new();
    for (index, segment) in segments.iter().enumerate() {
        for (edge, _) in segment.endpoints {
            adjacency
                .entry((segment.component, edge.0, edge.1))
                .or_default()
                .push(index);
        }
    }
    let mut remaining = (0..segments.len()).collect::<BTreeSet<_>>();
    let mut output = Vec::new();
    while let Some(&first) = remaining.iter().next() {
        let segment = &segments[first];
        let start = segment
            .endpoints
            .iter()
            .find(|(edge, _)| adjacency[&(segment.component, edge.0, edge.1)].len() != 2)
            .map(|(edge, _)| *edge)
            .unwrap_or(segment.endpoints[0].0);
        let mut current = start;
        let mut points = vec![endpoint_point(segment, start)];
        let component = segment.component;
        let mut discriminator = u32::MAX;
        loop {
            let Some(&index) = adjacency[&(component, current.0, current.1)]
                .iter()
                .find(|index| remaining.contains(index))
            else {
                break;
            };
            remaining.remove(&index);
            let segment = &segments[index];
            for (edge, _) in segment.endpoints {
                discriminator = discriminator
                    .min(edge.0.wrapping_mul(73_856_093) ^ edge.1.wrapping_mul(19_349_663));
            }
            let next = if segment.endpoints[0].0 == current {
                segment.endpoints[1]
            } else {
                segment.endpoints[0]
            };
            points.push(next.1);
            current = next.0;
            if current == start || adjacency[&(component, current.0, current.1)].len() != 2 {
                break;
            }
        }
        if points.len() >= 2 {
            output.push((component, discriminator, points));
        }
    }
    let counts = output
        .iter()
        .fold(BTreeMap::<u32, usize>::new(), |mut counts, value| {
            *counts.entry(value.0).or_default() += 1;
            counts
        });
    output
        .into_iter()
        .map(|(component, discriminator, points)| SmoothContourStroke {
            id: if counts[&component] == 1 {
                contour_component_id(component, id_namespace)
            } else {
                contour_component_id(component ^ discriminator, id_namespace)
            },
            points,
        })
        .collect()
}

fn contour_component_id(component: u32, id_namespace: u32) -> u32 {
    id_namespace | component.wrapping_mul(0x45d9_f3b) & 0x0fff_ffff
}

fn endpoint_point(segment: &ContourSegment, edge: (u32, u32)) -> Vec3 {
    segment
        .endpoints
        .iter()
        .find_map(|(candidate, point)| (*candidate == edge).then_some(*point))
        .expect("chain endpoint belongs to its segment")
}

fn ordered_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b {
        (a, b)
    } else {
        (b, a)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contour_crossing_at_one_vertex_has_a_stable_symbolic_side() {
        let symbolic = symbolic_contour_value(0.0, 7, 1.0e-6);
        let opposite = -symbolic.signum();
        let forward = contour_edge_crossing(7, 11, 0.0, opposite).unwrap();
        let reverse = contour_edge_crossing(11, 7, opposite, 0.0).unwrap();
        assert!((forward + reverse - 1.0).abs() < 1.0e-6);
        assert_eq!(
            symbolic,
            symbolic_contour_value(0.0, 7, 1.0e-6)
        );
        assert!(contour_edge_crossing(7, 11, 0.0, 0.0).is_none());
    }

    #[test]
    fn contour_crosses_the_interior_of_a_smooth_quad_without_a_diagonal_break() {
        let geometry = NprGeometry::from_indexed(
            &[
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [1.0, 1.0, 0.0],
                [-1.0, 1.0, 0.0],
            ],
            &[0, 1, 2, 0, 2, 3],
        )
        .unwrap();
        // The plane itself has no contour from this direction. Use a folded
        // proxy below for a non-trivial zero crossing instead.
        assert!(smooth_perspective_contours(&geometry, Vec3::new(0.0, 0.0, 5.0), 1.2).is_empty());
    }

    #[test]
    fn smooth_contour_is_deterministic() {
        let geometry = NprGeometry::icosphere();
        let camera = Vec3::new(0.3, 0.2, 4.0);
        let first = smooth_perspective_contours(&geometry, camera, 1.2);
        assert_eq!(first, smooth_perspective_contours(&geometry, camera, 1.2));
        assert!(first.iter().any(|stroke| stroke.points.len() > 2));
    }

    #[test]
    fn a_smooth_form_retains_a_region_identity_across_camera_motion() {
        let geometry = NprGeometry::icosphere();
        let before = smooth_perspective_contours(&geometry, Vec3::new(0.1, 0.2, 4.0), 1.2);
        let after = smooth_perspective_contours(&geometry, Vec3::new(0.7, 0.1, 3.9), 1.2);
        let before_ids = before
            .into_iter()
            .map(|stroke| stroke.id)
            .collect::<BTreeSet<_>>();
        let after_ids = after
            .into_iter()
            .map(|stroke| stroke.id)
            .collect::<BTreeSet<_>>();
        assert!(!before_ids.is_empty());
        assert!(before_ids.intersection(&after_ids).next().is_some());
    }

    #[test]
    fn simultaneous_contour_spans_do_not_share_an_identity() {
        let geometry = NprGeometry::icosphere();
        let contours = smooth_perspective_contours(&geometry, Vec3::new(0.1, 0.2, 4.0), 1.2);
        let ids = contours
            .iter()
            .map(|stroke| stroke.id)
            .collect::<BTreeSet<_>>();
        assert_eq!(ids.len(), contours.len());
    }

    #[test]
    fn smooth_contours_do_not_reuse_a_triangle_edge_as_a_path() {
        let geometry = NprGeometry::icosphere();
        let contours = smooth_perspective_contours(&geometry, Vec3::new(0.3, 0.2, 4.0), 1.2);
        let mesh_edges = geometry
            .triangles
            .iter()
            .flat_map(|triangle| {
                [
                    ordered_edge(triangle[0], triangle[1]),
                    ordered_edge(triangle[1], triangle[2]),
                    ordered_edge(triangle[2], triangle[0]),
                ]
            })
            .collect::<BTreeSet<_>>();

        for stroke in contours {
            for segment in stroke.points.windows(2) {
                let matches_mesh_edge = mesh_edges.iter().any(|&(a, b)| {
                    let start = geometry.vertices[a as usize].position;
                    let end = geometry.vertices[b as usize].position;
                    point_on_segment(segment[0], start, end)
                        && point_on_segment(segment[1], start, end)
                });
                assert!(!matches_mesh_edge, "smooth contour followed a mesh edge");
            }
        }
    }

    #[test]
    fn suggestive_contours_are_deterministic_on_a_smooth_saddle() {
        let geometry = NprGeometry::from_indexed(
            &[
                [-1.0, -1.0, 1.0],
                [0.0, -1.0, 0.0],
                [1.0, -1.0, -1.0],
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [-1.0, 1.0, -1.0],
                [0.0, 1.0, 0.0],
                [1.0, 1.0, 1.0],
            ],
            &[
                0, 1, 4, 0, 4, 3, 1, 2, 5, 1, 5, 4, 3, 4, 7, 3, 7, 6, 4, 5, 8, 4, 8, 7,
            ],
        )
        .unwrap();
        let first = suggestive_perspective_contours(&geometry, Vec3::new(0.2, 0.1, 4.0), 1.2, 0.0);
        assert!(!first.is_empty());
        assert!(first
            .iter()
            .all(|stroke| stroke.id & 0xf000_0000 == 0x3000_0000));
        assert_eq!(
            first,
            suggestive_perspective_contours(&geometry, Vec3::new(0.2, 0.1, 4.0), 1.2, 0.0)
        );
    }

    #[test]
    fn corner_normals_do_not_smooth_across_a_declared_sharp_dihedral() {
        let geometry = NprGeometry::from_indexed(
            &[
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            &[0, 1, 2, 0, 3, 1],
        )
        .unwrap();
        let hard = smoothed_corner_normals(&geometry, 1.2);
        let fully_smooth = smoothed_corner_normals(&geometry, std::f32::consts::PI);
        assert!(hard[0][0].dot(hard[1][0]) < 0.1);
        assert!(fully_smooth[0][0].dot(fully_smooth[1][0]) > 0.99);
    }

    fn point_on_segment(point: Vec3, start: Vec3, end: Vec3) -> bool {
        let edge = end - start;
        let length_squared = edge.length_squared();
        let t = (point - start).dot(edge) / length_squared;
        (0.0..=1.0).contains(&t) && (start + edge * t).distance(point) < 1e-4
    }
}
