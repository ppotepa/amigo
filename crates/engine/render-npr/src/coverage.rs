//! Object-local mask inputs, prepared by the domain extractor rather than
//! guessed from screen depth or ink colour by a presentation backend.
use crate::{NprGeometry, PerspectiveCamera, contour::ContourField};
use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NprCoverageSample {
    pub position: Vec3,
    pub normal: Vec3,
    /// Normalized object-local Y, relative to this surface's bounds.
    pub height: f32,
    /// Continuous surface illumination, independent of the selected ink colour.
    pub tone: f32,
}
impl NprCoverageSample {
    /// Perspective-correct interpolation of the attributes carried by a packet
    /// triangle. Normalization happens when a direction mask is evaluated.
    pub fn interpolate(
        samples: [Option<Self>; 3],
        depths: [f32; 3],
        weights: [f32; 3],
    ) -> Option<Self> {
        let [Some(a), Some(b), Some(c)] = samples else {
            return None;
        };
        let mut q = std::array::from_fn::<_, 3, _>(|i| weights[i] * (1.0 - depths[i]));
        let sum: f32 = q.iter().sum();
        if sum <= 1e-8 {
            return None;
        }
        for weight in &mut q {
            *weight /= sum;
        }
        Some(Self {
            position: a.position * q[0] + b.position * q[1] + c.position * q[2],
            normal: a.normal * q[0] + b.normal * q[1] + c.normal * q[2],
            height: a.height * q[0] + b.height * q[1] + c.height * q[2],
            tone: a.tone * q[0] + b.tone * q[1] + c.tone * q[2],
        })
    }
}

/// Smooth seeded lattice noise, stable under camera motion and subpixel jitter.
pub(crate) fn coverage_noise(position: Vec3, seed: u64) -> f32 {
    let point = position * 8.0;
    let cell = point.floor();
    let fraction = point - cell;
    let smooth = fraction * fraction * (Vec3::splat(3.0) - fraction * 2.0);
    let mut value = 0.0;
    for z in 0..2 {
        for y in 0..2 {
            for x in 0..2 {
                let mut hash = seed as u32 ^ ((seed >> 32) as u32).rotate_left(16) ^ 0x9e3779b9;
                for coordinate in [
                    (cell.x as i32).wrapping_add(x),
                    (cell.y as i32).wrapping_add(y),
                    (cell.z as i32).wrapping_add(z),
                ] {
                    hash ^= coordinate as u32;
                    hash = (hash ^ (hash >> 16)).wrapping_mul(0x7feb352d);
                    hash = (hash ^ (hash >> 15)).wrapping_mul(0x846ca68b);
                    hash ^= hash >> 16;
                }
                let weight = (if x == 0 { 1.0 - smooth.x } else { smooth.x })
                    * (if y == 0 { 1.0 - smooth.y } else { smooth.y })
                    * (if z == 0 { 1.0 - smooth.z } else { smooth.z });
                value += (hash >> 8) as f32 / 16777215.0 * weight;
            }
        }
    }
    value.clamp(0.0, 1.0)
}

#[derive(Debug)]
struct Node {
    min: Vec3,
    max: Vec3,
    faces: std::ops::Range<usize>,
    children: Option<[usize; 2]>,
}
impl Node {
    fn distance(&self, point: Vec3) -> f32 {
        (point - point.clamp(self.min, self.max)).length_squared()
    }
}

/// Revision-local acceleration for sampling tessellated gestures. The nearest
/// point is on the actual mesh; no invented normal is used outside a silhouette.
#[derive(Debug, Default)]
pub(crate) struct CoverageIndex {
    faces: Vec<usize>,
    nodes: Vec<Node>,
}
impl CoverageIndex {
    pub fn build(geometry: &NprGeometry) -> Self {
        let mut result = Self {
            faces: (0..geometry.triangles.len()).collect(),
            nodes: Vec::new(),
        };
        if !result.faces.is_empty() {
            result.branch(geometry, 0..result.faces.len());
        }
        result
    }
    fn branch(&mut self, geometry: &NprGeometry, range: std::ops::Range<usize>) -> usize {
        let mut min = Vec3::splat(f32::INFINITY);
        let mut max = Vec3::splat(f32::NEG_INFINITY);
        for face in &self.faces[range.clone()] {
            for point in triangle(geometry, *face) {
                min = min.min(point);
                max = max.max(point);
            }
        }
        let id = self.nodes.len();
        self.nodes.push(Node {
            min,
            max,
            faces: range.clone(),
            children: None,
        });
        if range.len() > 8 {
            let extent = max - min;
            let axis = if extent.x >= extent.y && extent.x >= extent.z {
                0
            } else if extent.y >= extent.z {
                1
            } else {
                2
            };
            let middle = range.len() / 2;
            self.faces[range.clone()].select_nth_unstable_by(middle, |a, b| {
                let center = |face| {
                    triangle(geometry, face)
                        .iter()
                        .map(|v| v[axis])
                        .sum::<f32>()
                };
                center(*a).total_cmp(&center(*b)).then(a.cmp(b))
            });
            let split = range.start + middle;
            let left = self.branch(geometry, range.start..split);
            let right = self.branch(geometry, split..range.end);
            self.nodes[id].children = Some([left, right]);
        }
        id
    }
    fn nearest(&self, geometry: &NprGeometry, point: Vec3) -> Option<(usize, Vec3)> {
        if self.nodes.is_empty() || !point.is_finite() {
            return None;
        }
        let mut pending = vec![0];
        let mut best = (f32::INFINITY, usize::MAX, Vec3::ZERO);
        while let Some(id) = pending.pop() {
            let node = &self.nodes[id];
            if node.distance(point) > best.0 {
                continue;
            }
            if let Some([a, b]) = node.children {
                if self.nodes[a].distance(point) < self.nodes[b].distance(point) {
                    pending.extend([b, a]);
                } else {
                    pending.extend([a, b]);
                }
            } else {
                for face in &self.faces[node.faces.clone()] {
                    let points = triangle(geometry, *face);
                    let weights = closest_weights(points, point);
                    let surface =
                        points[0] * weights.x + points[1] * weights.y + points[2] * weights.z;
                    let distance = surface.distance_squared(point);
                    if distance < best.0 || (distance == best.0 && *face < best.1) {
                        best = (distance, *face, weights);
                    }
                }
            }
        }
        (best.1 != usize::MAX).then_some((best.1, best.2))
    }
    pub fn sample_face(
        &self,
        geometry: &NprGeometry,
        normals: Option<&ContourField>,
        face: usize,
        point: Vec3,
        light: Vec3,
    ) -> NprCoverageSample {
        self.sample_weights(
            geometry,
            normals,
            face,
            closest_weights(triangle(geometry, face), point),
            light,
        )
    }
    fn sample_weights(
        &self,
        geometry: &NprGeometry,
        normals: Option<&ContourField>,
        face: usize,
        weights: Vec3,
        light: Vec3,
    ) -> NprCoverageSample {
        let [a, b, c] = triangle(geometry, face);
        let position = a * weights.x + b * weights.y + c * weights.z;
        let normal = if let Some(field) = normals {
            let [a, b, c] = field.normals(face);
            (a * weights.x + b * weights.y + c * weights.z).normalize_or_zero()
        } else {
            (b - a).cross(c - a).normalize_or_zero()
        };
        let bounds = &self.nodes[0];
        NprCoverageSample {
            position,
            normal,
            height: if bounds.max.y - bounds.min.y > 1e-6 {
                ((position.y - bounds.min.y) / (bounds.max.y - bounds.min.y)).clamp(0.0, 1.0)
            } else {
                0.5
            },
            tone: (normal.dot(light.normalize_or_zero()) * 0.5 + 0.5).clamp(0.0, 1.0),
        }
    }
    pub fn sample_screen(
        &self,
        geometry: &NprGeometry,
        normals: Option<&ContourField>,
        camera: PerspectiveCamera,
        viewport: Vec2,
        screen: Vec2,
        depth: f32,
        light: Vec3,
    ) -> Option<NprCoverageSample> {
        let point = camera.unproject(screen, depth, viewport)?;
        let (face, weights) = self.nearest(geometry, point)?;
        Some(self.sample_weights(geometry, normals, face, weights, light))
    }
}
fn triangle(geometry: &NprGeometry, face: usize) -> [Vec3; 3] {
    geometry.triangles[face].map(|i| geometry.vertices[i as usize].position)
}

/// Closest point on a triangle, including its boundary and degenerate edges.
fn closest_weights([a, b, c]: [Vec3; 3], point: Vec3) -> Vec3 {
    let ab = b - a;
    let ac = c - a;
    let ap = point - a;
    let d00 = ab.dot(ab);
    let d01 = ab.dot(ac);
    let d11 = ac.dot(ac);
    let denominator = d00 * d11 - d01 * d01;
    if denominator > 1e-12 {
        let v = (d11 * ap.dot(ab) - d01 * ap.dot(ac)) / denominator;
        let w = (d00 * ap.dot(ac) - d01 * ap.dot(ab)) / denominator;
        if v >= 0.0 && w >= 0.0 && v + w <= 1.0 {
            return Vec3::new(1.0 - v - w, v, w);
        }
    }
    let mut best = (f32::INFINITY, Vec3::X);
    for (start, end, weights_start, weights_end) in [
        (a, b, Vec3::X, Vec3::Y),
        (b, c, Vec3::Y, Vec3::Z),
        (c, a, Vec3::Z, Vec3::X),
    ] {
        let edge = end - start;
        let t = ((point - start).dot(edge) / edge.length_squared().max(1e-20)).clamp(0.0, 1.0);
        let distance = (start + edge * t).distance_squared(point);
        if distance < best.0 {
            best = (distance, weights_start.lerp(weights_end, t));
        }
    }
    best.1
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn camera_motion_does_not_change_local_coverage_inputs() {
        let geometry = NprGeometry::canonical_cube();
        let index = CoverageIndex::build(&geometry);
        let point = Vec3::new(0.2, 0.5, 1.0);
        let mut reference: Option<NprCoverageSample> = None;
        for (position, viewport) in [
            (Vec3::new(0.0, 0.0, 5.0), Vec2::new(96.0, 96.0)),
            (Vec3::new(3.0, 2.0, 7.0), Vec2::new(640.0, 360.0)),
        ] {
            let camera = PerspectiveCamera {
                position,
                forward: -position.normalize(),
                ..PerspectiveCamera::cube_default(viewport.x / viewport.y)
            };
            let projected = camera.project(point, viewport).unwrap();
            let sample = index
                .sample_screen(
                    &geometry,
                    None,
                    camera,
                    viewport,
                    projected.screen,
                    camera.normalized_depth(projected.depth),
                    Vec3::Z,
                )
                .unwrap();
            assert!(sample.position.distance(point) < 0.0001);
            if let Some(previous) = reference {
                assert!(sample.normal.distance(previous.normal) < 0.0001);
                assert!((sample.height - previous.height).abs() < 0.0001);
                assert!(
                    (coverage_noise(sample.position, 14) - coverage_noise(previous.position, 14))
                        .abs()
                        < 0.0001
                );
            }
            reference = Some(sample);
        }
    }
    #[test]
    fn smooth_extraction_populates_surface_inputs_and_fingerprints_them() {
        let surface = crate::NprPreparedSurface::new(NprGeometry::cylinder(24));
        let style = crate::ComicInk {
            surface_mode: crate::NprSurfaceMode::Smooth,
            tone_mode: crate::NprToneMode::ThreeBand,
            ..Default::default()
        };
        let mut packet = crate::build_packet_for_surface(
            &surface,
            PerspectiveCamera::cube_default(1.0),
            [96, 96],
            style,
            4,
            crate::NprDebugView::Final,
        );
        assert!(!packet.strokes.is_empty());
        assert!(!packet.fills.is_empty());
        for sample in packet
            .strokes
            .iter()
            .flat_map(|s| s.vertices.iter().map(|v| v.surface))
            .chain(packet.fills.iter().flat_map(|t| t.surface))
            .chain(packet.underpainting.iter().flat_map(|t| t.surface))
        {
            let sample = sample.expect("surface-derived geometry must carry mask inputs");
            assert!((sample.normal.length() - 1.0).abs() < 0.0001);
            assert!((0.0..=1.0).contains(&sample.height));
        }
        let before = packet.fingerprint();
        packet.strokes[0].vertices[0]
            .surface
            .as_mut()
            .unwrap()
            .height = 0.12345;
        assert_ne!(packet.fingerprint(), before);
    }
    #[test]
    fn interior_ranges_are_not_rejected_by_corner_only_diagnostics() {
        let sample = |height| {
            Some(NprCoverageSample {
                position: Vec3::Y * height,
                normal: Vec3::Z,
                height,
                tone: 0.5,
            })
        };
        let samples = [sample(0.0), sample(0.0), sample(1.0)];
        let mask = crate::CoverageMask::Height {
            min: 0.4,
            max: 0.6,
            invert: false,
        };
        assert!(samples.iter().all(|s| mask.evaluate(*s) == 0.0));
        assert!(mask.triangle_estimate(samples, [0.5; 3]) > 0.0);
        let narrow = crate::CoverageMask::Height {
            min: 0.49999,
            max: 0.50001,
            invert: false,
        };
        assert!(narrow.triangle_may_cover(samples));
        let noise = |seed| {
            crate::CoverageMask::Noise {
                amount: 0.75,
                seed,
                invert: false,
            }
            .evaluate(sample(0.3))
        };
        assert_ne!(noise(3), noise(4));
        assert_ne!(noise(3), noise(3 + (1 << 32)));
    }
    #[test]
    fn cube_samples_use_real_faces_and_local_height() {
        let geometry = NprGeometry::canonical_cube();
        let index = CoverageIndex::build(&geometry);
        for (point, expected) in [
            (Vec3::new(2.0, 0.7, 0.2), Vec3::X),
            (Vec3::new(0.2, -2.0, 0.1), Vec3::NEG_Y),
        ] {
            let (face, weights) = index.nearest(&geometry, point).unwrap();
            let sample = index.sample_weights(&geometry, None, face, weights, Vec3::X);
            assert!(sample.normal.dot(expected) > 0.999);
            assert!((sample.height - (sample.position.y + 1.0) * 0.5).abs() < 1e-5);
            assert!((sample.tone - (expected.dot(Vec3::X) * 0.5 + 0.5)).abs() < 1e-5);
        }
    }
    #[test]
    fn bvh_matches_exhaustive_surface_sampling() {
        let geometry = NprGeometry::cylinder(32);
        let index = CoverageIndex::build(&geometry);
        for i in 0..80 {
            let point = Vec3::new(
                (i as f32 * 0.21).cos() * 1.4,
                (i as f32 * 0.31).sin() * 1.5,
                0.45,
            );
            let (face, weights) = index.nearest(&geometry, point).unwrap();
            let nearest = index
                .sample_weights(&geometry, None, face, weights, Vec3::Y)
                .position
                .distance_squared(point);
            let brute = (0..geometry.triangles.len())
                .map(|face| {
                    index
                        .sample_face(&geometry, None, face, point, Vec3::Y)
                        .position
                        .distance_squared(point)
                })
                .fold(f32::INFINITY, f32::min);
            assert!((nearest - brute).abs() < 1e-5);
        }
    }
}
