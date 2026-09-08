//! Surface-space hatching paths.
//!
//! Parallel planes intersect a mesh into paths. Joining happens through shared
//! topology edges before projection, so a smooth surface does not acquire a
//! cap or a new gesture merely because its author used another triangle.

use crate::{NprGeometry, SurfaceDirectionField, TopologyEdge, face_normal};
use glam::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct SurfaceHatchPath {
    pub points: Vec<Vec3>,
    pub faces: Vec<u32>,
}

#[derive(Debug, Clone)]
struct SurfaceSegment {
    face: u32,
    points: [Vec3; 2],
    edges: [(u32, u32); 2],
}

/// Intersects selected faces with planes `dot(plane_normal, point) = offset`.
/// A connection is allowed only across a shared edge whose normal agreement is
/// above `smooth_dot_threshold`. Degenerate vertex hits are intentionally
/// skipped rather than creating ambiguous branches at a mesh vertex.
pub fn trace_parallel_surface_lines(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    plane_normal: Vec3,
    offsets: impl IntoIterator<Item = f32>,
    selected_faces: &[bool],
    smooth_dot_threshold: f32,
    max_paths: usize,
) -> Vec<SurfaceHatchPath> {
    if plane_normal.length_squared() <= 1e-8 || selected_faces.len() != geometry.triangles.len() {
        return vec![];
    }
    let normal = plane_normal.normalize();
    let normals: Vec<_> = (0..geometry.triangles.len())
        .map(|face| face_normal(geometry, face as u32))
        .collect();
    let mut output = Vec::new();
    for offset in offsets {
        if output.len() >= max_paths {
            break;
        }
        let mut segments = Vec::new();
        for (face, triangle) in geometry.triangles.iter().enumerate() {
            if !selected_faces[face] {
                continue;
            }
            if let Some(segment) =
                intersect_triangle(geometry, *triangle, face as u32, normal, offset)
            {
                segments.push(segment);
            }
        }
        let mut used = vec![false; segments.len()];
        for start in 0..segments.len() {
            if used[start] || output.len() >= max_paths {
                continue;
            }
            used[start] = true;
            let first = &segments[start];
            let mut path = SurfaceHatchPath {
                points: vec![first.points[0], first.points[1]],
                faces: vec![first.face],
            };
            extend_path(
                &mut path,
                false,
                &segments,
                &mut used,
                topology,
                &normals,
                smooth_dot_threshold,
            );
            extend_path(
                &mut path,
                true,
                &segments,
                &mut used,
                topology,
                &normals,
                smooth_dot_threshold,
            );
            if path.points.len() >= 2 {
                output.push(path);
            }
        }
    }
    output
}

fn intersect_triangle(
    geometry: &NprGeometry,
    triangle: [u32; 3],
    face: u32,
    normal: Vec3,
    offset: f32,
) -> Option<SurfaceSegment> {
    const EPSILON: f32 = 1e-6;
    let edges = [
        (triangle[0], triangle[1]),
        (triangle[1], triangle[2]),
        (triangle[2], triangle[0]),
    ];
    let mut hits = Vec::with_capacity(2);
    for (a, b) in edges {
        let point_a = geometry.vertices[a as usize].position;
        let point_b = geometry.vertices[b as usize].position;
        let da = normal.dot(point_a) - offset;
        let db = normal.dot(point_b) - offset;
        // Exact hits generate unstable, multi-way vertex choices. The next
        // nearby plane supplies the visual line without such an ambiguity.
        if da.abs() <= EPSILON || db.abs() <= EPSILON || da.signum() == db.signum() {
            continue;
        }
        let t = da / (da - db);
        hits.push((point_a.lerp(point_b, t), ordered_edge(a, b)));
    }
    (hits.len() == 2).then(|| SurfaceSegment {
        face,
        points: [hits[0].0, hits[1].0],
        edges: [hits[0].1, hits[1].1],
    })
}

fn extend_path(
    path: &mut SurfaceHatchPath,
    prepend: bool,
    segments: &[SurfaceSegment],
    used: &mut [bool],
    topology: &[TopologyEdge],
    normals: &[Vec3],
    smooth_dot_threshold: f32,
) {
    loop {
        let current_face = if prepend {
            *path.faces.first().unwrap()
        } else {
            *path.faces.last().unwrap()
        };
        let connection = if prepend {
            path.points[0]
        } else {
            *path.points.last().unwrap()
        };
        let edge = find_segment_edge(segments, current_face, connection);
        let Some(edge) = edge else { break };
        let next = segments.iter().enumerate().find_map(|(index, segment)| {
            (!used[index]
                && segment.edges.contains(&edge)
                && smooth_connection(
                    topology,
                    current_face,
                    segment.face,
                    edge,
                    normals,
                    smooth_dot_threshold,
                ))
            .then_some(index)
        });
        let Some(index) = next else { break };
        used[index] = true;
        let segment = &segments[index];
        let endpoint = if segment.edges[0] == edge {
            segment.points[1]
        } else {
            segment.points[0]
        };
        if prepend {
            path.points.insert(0, endpoint);
            path.faces.insert(0, segment.face);
        } else {
            path.points.push(endpoint);
            path.faces.push(segment.face);
        }
    }
}

fn find_segment_edge(segments: &[SurfaceSegment], face: u32, point: Vec3) -> Option<(u32, u32)> {
    segments
        .iter()
        .find(|segment| {
            segment.face == face && (segment.points[0] == point || segment.points[1] == point)
        })
        .and_then(|segment| {
            (segment.points[0] == point)
                .then_some(segment.edges[0])
                .or_else(|| (segment.points[1] == point).then_some(segment.edges[1]))
        })
}

fn smooth_connection(
    topology: &[TopologyEdge],
    from: u32,
    to: u32,
    edge: (u32, u32),
    normals: &[Vec3],
    threshold: f32,
) -> bool {
    from != to
        && topology.iter().any(|topology_edge| {
            ordered_edge(topology_edge.a, topology_edge.b) == edge
                && topology_edge.faces.contains(&from)
                && topology_edge.faces.contains(&to)
                && normals[from as usize].dot(normals[to as usize]) >= threshold
        })
}

fn ordered_edge(a: u32, b: u32) -> (u32, u32) {
    if a < b { (a, b) } else { (b, a) }
}

/// Integrates a local tangent field over the actual triangle surface. A path
/// may cross only a selected, smooth neighbor; clipping and tessellation are
/// deliberately left to later stages.
pub fn trace_surface_streamline(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    field: &SurfaceDirectionField,
    selected_faces: &[bool],
    start_face: u32,
    start: Vec3,
    step_length: f32,
    max_steps: usize,
    smooth_dot_threshold: f32,
) -> SurfaceHatchPath {
    if step_length <= 1e-6
        || start_face as usize >= geometry.triangles.len()
        || selected_faces.len() != geometry.triangles.len()
        || !selected_faces[start_face as usize]
    {
        return SurfaceHatchPath {
            points: vec![],
            faces: vec![],
        };
    }
    let start_face =
        containing_selected_face(geometry, selected_faces, start).unwrap_or(start_face);
    let forward = trace_streamline_direction(
        geometry,
        topology,
        field,
        selected_faces,
        start_face,
        start,
        step_length,
        max_steps,
        smooth_dot_threshold,
        1.0,
    );
    let mut backward = trace_streamline_direction(
        geometry,
        topology,
        field,
        selected_faces,
        start_face,
        start,
        step_length,
        max_steps,
        smooth_dot_threshold,
        -1.0,
    );
    backward.points.reverse();
    backward.faces.reverse();
    if !backward.points.is_empty() {
        backward.points.pop();
    }
    if !backward.faces.is_empty() {
        backward.faces.pop();
    }
    backward.points.extend(forward.points);
    backward.faces.extend(forward.faces);
    SurfaceHatchPath {
        points: backward.points,
        faces: backward.faces,
    }
}

fn containing_selected_face(
    geometry: &NprGeometry,
    selected_faces: &[bool],
    point: Vec3,
) -> Option<u32> {
    geometry.triangles.iter().enumerate().find_map(|(face, _)| {
        (selected_faces[face] && point_in_face(geometry, face as u32, point, 1e-4))
            .then_some(face as u32)
    })
}

#[allow(clippy::too_many_arguments)]
fn trace_streamline_direction(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    field: &SurfaceDirectionField,
    selected_faces: &[bool],
    start_face: u32,
    start: Vec3,
    step_length: f32,
    max_steps: usize,
    smooth_dot_threshold: f32,
    sign: f32,
) -> SurfaceHatchPath {
    const EPSILON: f32 = 1e-5;
    let mut face = start_face;
    let mut point = start;
    let mut previous_direction = Vec3::ZERO;
    let mut points = vec![point];
    let mut faces = vec![face];
    for _ in 0..max_steps {
        let face_index = face as usize;
        if field.face_confidence(face_index) <= 0.05 {
            break;
        }
        let mut direction = field.face_tangent(face_index) * sign;
        if previous_direction.length_squared() > 1e-8 && direction.dot(previous_direction) < 0.0 {
            direction = -direction;
        }
        // The field is based on smoothed normals, while each integration step
        // must stay on the actual current triangle. Projecting here avoids
        // gradually drifting off a face and later accepting a different,
        // nearby face merely because its barycentric projection happened to fit.
        let geometric_normal = face_normal(geometry, face);
        direction -= geometric_normal * direction.dot(geometric_normal);
        direction = direction.normalize_or_zero();
        if direction.length_squared() <= 1e-8 {
            break;
        }
        let candidate = point + direction * step_length;
        let candidate_barycentric = barycentric(geometry, face, candidate);
        if point_in_face(geometry, face, candidate, EPSILON) {
            point = candidate;
            points.push(point);
            previous_direction = direction;
            continue;
        }
        let current_barycentric = barycentric(geometry, face, point);
        let mut edge_time = 1.0f32;
        let mut exit = None;
        for index in 0..3 {
            if candidate_barycentric[index] < -EPSILON {
                let denominator = current_barycentric[index] - candidate_barycentric[index];
                if denominator > EPSILON {
                    let time = (current_barycentric[index] / denominator).clamp(0.0, 1.0);
                    if time < edge_time {
                        edge_time = time;
                        exit = Some(index);
                    }
                }
            }
        }
        let Some(exit) = exit else { break };
        let edge_point = point.lerp(candidate, edge_time);
        points.push(edge_point);
        let triangle = geometry.triangles[face_index];
        let edge = ordered_edge(triangle[(exit + 1) % 3], triangle[(exit + 2) % 3]);
        let Some(next_face) = smooth_neighbor(
            geometry,
            topology,
            selected_faces,
            face,
            edge,
            smooth_dot_threshold,
        ) else {
            break;
        };
        let mut next_direction = field.face_tangent(next_face as usize) * sign;
        if next_direction.dot(direction) < 0.0 {
            next_direction = -next_direction;
        }
        let next_normal = face_normal(geometry, next_face);
        next_direction -= next_normal * next_direction.dot(next_normal);
        let inner = edge_point + next_direction.normalize_or_zero() * EPSILON * 4.0;
        if !point_in_face(geometry, next_face, inner, EPSILON) {
            break;
        }
        point = inner;
        face = next_face;
        faces.push(face);
        previous_direction = next_direction;
    }
    SurfaceHatchPath { points, faces }
}

fn barycentric(geometry: &NprGeometry, face: u32, point: Vec3) -> Vec3 {
    let triangle = geometry.triangles[face as usize];
    let a = geometry.vertices[triangle[0] as usize].position;
    let b = geometry.vertices[triangle[1] as usize].position;
    let c = geometry.vertices[triangle[2] as usize].position;
    let v0 = b - a;
    let v1 = c - a;
    let v2 = point - a;
    let d00 = v0.dot(v0);
    let d01 = v0.dot(v1);
    let d11 = v1.dot(v1);
    let denominator = d00 * d11 - d01 * d01;
    if denominator.abs() <= 1e-10 {
        return Vec3::splat(-1.0);
    }
    let d20 = v2.dot(v0);
    let d21 = v2.dot(v1);
    let v = (d11 * d20 - d01 * d21) / denominator;
    let w = (d00 * d21 - d01 * d20) / denominator;
    Vec3::new(1.0 - v - w, v, w)
}

fn point_in_face(geometry: &NprGeometry, face: u32, point: Vec3, epsilon: f32) -> bool {
    let triangle = geometry.triangles[face as usize];
    let a = geometry.vertices[triangle[0] as usize].position;
    let b = geometry.vertices[triangle[1] as usize].position;
    let c = geometry.vertices[triangle[2] as usize].position;
    let normal = (b - a).cross(c - a);
    let normal_length = normal.length();
    normal_length > 1e-10
        && ((point - a).dot(normal) / normal_length).abs() <= epsilon
        && barycentric(geometry, face, point).min_element() >= -epsilon
}

fn smooth_neighbor(
    geometry: &NprGeometry,
    topology: &[TopologyEdge],
    selected_faces: &[bool],
    face: u32,
    edge: (u32, u32),
    threshold: f32,
) -> Option<u32> {
    topology.iter().find_map(|topology_edge| {
        (ordered_edge(topology_edge.a, topology_edge.b) == edge
            && topology_edge.faces.contains(&face))
        .then(|| {
            topology_edge.faces.iter().copied().find(|neighbor| {
                *neighbor != face
                    && *neighbor != u32::MAX
                    && selected_faces[*neighbor as usize]
                    && face_normal(geometry, face).dot(face_normal(geometry, *neighbor))
                        >= threshold
            })
        })
        .flatten()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::build_topology;

    #[test]
    fn path_crosses_a_smooth_triangulation_diagonal() {
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
        let paths = trace_parallel_surface_lines(
            &geometry,
            &build_topology(&geometry),
            Vec3::X,
            [0.0],
            &[true, true],
            0.99,
            16,
        );
        assert_eq!(paths.len(), 1);
        assert_eq!(paths[0].points.len(), 3);
        assert_eq!(paths[0].faces.len(), 2);
    }

    #[test]
    fn path_stops_at_a_crease() {
        let geometry = NprGeometry::canonical_cube();
        let paths = trace_parallel_surface_lines(
            &geometry,
            &build_topology(&geometry),
            Vec3::X,
            [0.2],
            &vec![true; geometry.triangles.len()],
            0.99,
            64,
        );
        assert!(paths.iter().all(|path| path.faces.len() <= 2));
    }

    #[test]
    fn streamline_crosses_a_smooth_diagonal_using_the_local_tangent() {
        let geometry = NprGeometry::from_indexed(
            &[
                [-2.0, -1.0, 0.0],
                [2.0, -1.0, 0.0],
                [2.0, 1.0, 0.0],
                [-2.0, 1.0, 0.0],
            ],
            &[0, 1, 2, 0, 2, 3],
        )
        .unwrap();
        let topology = build_topology(&geometry);
        let field = SurfaceDirectionField::build(&geometry, &topology, 0.99);
        let path = trace_surface_streamline(
            &geometry,
            &topology,
            &field,
            &[true, true],
            0,
            Vec3::new(-1.0, 0.0, 0.0),
            0.2,
            32,
            0.99,
        );
        assert!(path.points.len() > 3);
        assert!(path.faces.contains(&1));
    }

    #[test]
    fn streamline_does_not_jump_to_a_nearby_parallel_face() {
        let geometry = NprGeometry::from_indexed(
            &[
                [-1.0, -1.0, 0.0],
                [1.0, -1.0, 0.0],
                [0.0, 1.0, 0.0],
                [-1.0, -1.0, 0.1],
                [1.0, -1.0, 0.1],
                [0.0, 1.0, 0.1],
            ],
            &[0, 1, 2, 3, 4, 5],
        )
        .unwrap();
        let topology = build_topology(&geometry);
        let field = SurfaceDirectionField::build(&geometry, &topology, 0.99);
        let path = trace_surface_streamline(
            &geometry,
            &topology,
            &field,
            &[true, true],
            1,
            Vec3::new(0.0, 0.0, 0.1),
            0.1,
            8,
            0.99,
        );
        assert!(path.faces.iter().all(|face| *face == 1));
        assert!(path.points.iter().all(|point| (point.z - 0.1).abs() < 1e-5));
    }
}
