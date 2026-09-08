use crate::{
    NprDebugView,
    camera::PerspectiveCamera,
    feature::FeatureClass,
    geometry::NprGeometry,
    style::ComicInk,
    tessellation::{TessellatedStroke, tessellate_polyline, tessellate_polyline_variants},
    topology::{build_topology, face_normal},
};
use glam::{Vec2, Vec4};

#[derive(Debug, Clone, PartialEq)]
pub struct NprFillTriangle {
    pub positions: [Vec2; 3],
    pub color: Vec4,
    pub depths: [f32; 3],
}
#[derive(Debug, Clone, PartialEq, Default)]
pub struct NprRenderStats {
    pub geometry: usize,
    pub topology_edges: usize,
    pub feature_segments: usize,
    pub silhouettes: usize,
    pub creases: usize,
    pub strokes: usize,
    pub stroke_vertices: usize,
    pub stroke_indices: usize,
    pub viewport: [u32; 2],
}
#[derive(Debug, Clone, PartialEq)]
pub struct NprRenderPacket {
    pub fills: Vec<NprFillTriangle>,
    pub strokes: Vec<TessellatedStroke>,
    pub background: Vec4,
    pub debug_view: NprDebugView,
    pub ink: Vec4,
    pub stats: NprRenderStats,
}

impl NprRenderPacket {
    /// Explicit selection annotation, independent of ink styling and feature extraction.
    /// Two windings let culling backends draw one face without a special pipeline.
    pub fn mark_selection(
        &mut self,
        geometry: &NprGeometry,
        camera: PerspectiveCamera,
        color: Vec4,
    ) {
        let viewport = Vec2::new(self.stats.viewport[0] as f32, self.stats.viewport[1] as f32);
        let mut min = Vec2::splat(f32::INFINITY);
        let mut max = Vec2::splat(f32::NEG_INFINITY);
        for point in geometry
            .vertices
            .iter()
            .filter_map(|v| camera.project(v.position, viewport))
        {
            min = min.min(point.screen);
            max = max.max(point.screen);
        }
        let center = Vec2::new((min.x + max.x) * 0.5, min.y - 10.0);
        if !center.is_finite()
            || center.x < 7.0
            || center.x > viewport.x - 7.0
            || center.y < 7.0
            || center.y > viewport.y - 7.0
        {
            return;
        }
        let positions = [
            center + Vec2::new(-6.0, -4.0),
            center + Vec2::new(6.0, -4.0),
            center + Vec2::new(0.0, 4.0),
        ];
        for positions in [positions, [positions[2], positions[1], positions[0]]] {
            self.fills.push(NprFillTriangle {
                positions,
                color,
                depths: [0.0; 3],
            });
        }
    }
    pub fn stroke_color(&self, stroke: &TessellatedStroke) -> [f32; 4] {
        match self.debug_view {
            NprDebugView::Final => self.ink.to_array(),
            NprDebugView::FeatureClasses => match stroke.class {
                FeatureClass::Boundary => [0.9, 0.15, 0.1, 1.0],
                FeatureClass::Silhouette => [0.1, 0.75, 0.2, 1.0],
                FeatureClass::Crease => [0.15, 0.3, 0.95, 1.0],
            },
            NprDebugView::StrokeIds => {
                let hue = (stroke.id.wrapping_mul(97) % 255) as f32 / 255.0;
                [hue, 1.0 - hue, 0.8, 1.0]
            }
        }
    }
}

pub fn build_packet(
    geometry: &NprGeometry,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    let topology = build_topology(geometry);
    build_packet_with_topology(
        geometry, &topology, camera, viewport, style, seed, debug_view,
    )
}

pub fn build_packet_with_topology(
    geometry: &NprGeometry,
    topology: &[crate::TopologyEdge],
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    let vp = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let features = crate::feature::classify_perspective_features(
        geometry,
        &topology,
        camera.position,
        style.crease_angle,
    );
    let mut fills = Vec::new();
    let mut tone_strokes = Vec::new();
    let light_direction = style.light_direction.normalize_or_zero();
    for (face_index, tri) in geometry.triangles.iter().enumerate() {
        for clipped in camera.clip_triangle(tri.map(|i| geometry.vertices[i as usize].position)) {
            if let (Some(a), Some(b), Some(c)) = (
                camera.project(clipped[0], vp),
                camera.project(clipped[1], vp),
                camera.project(clipped[2], vp),
            ) {
                let shade = face_normal(geometry, face_index as u32).dot(light_direction);
                let color = if shade < -0.1 {
                    style.shadow
                } else if shade < 0.45 {
                    style.mid
                } else {
                    style.light
                };
                let positions = [a.screen, b.screen, c.screen];
                let depths = [a.depth, b.depth, c.depth].map(|d| camera.normalized_depth(d));
                fills.push(NprFillTriangle {
                    positions,
                    color,
                    depths,
                });
                append_hatching(
                    &mut tone_strokes,
                    positions,
                    depths,
                    shade,
                    style,
                    seed.wrapping_add(face_index as u64 * 31),
                );
            }
        }
    }
    let mut strokes = Vec::new();
    for chain in crate::stroke::chain_features(&features) {
        let mut points = Vec::new();
        let flush = |points: &mut Vec<(Vec2, f32)>, strokes: &mut Vec<TessellatedStroke>| {
            if points.len() > 1 {
                let closed = points.first() == points.last();
                strokes.extend(tessellate_polyline_variants(
                    chain.id,
                    chain.class,
                    points,
                    closed,
                    style,
                    seed,
                ));
            }
            points.clear();
        };
        for edge in chain.vertices.windows(2) {
            if let Some((a, b)) = camera.project_segment(
                geometry.vertices[edge[0] as usize].position,
                geometry.vertices[edge[1] as usize].position,
                vp,
            ) {
                let a = (a.screen, camera.normalized_depth(a.depth));
                let b = (b.screen, camera.normalized_depth(b.depth));
                if points.last().is_some_and(|last| *last != a) {
                    flush(&mut points, &mut strokes);
                }
                if points.is_empty() {
                    points.push(a);
                }
                points.push(b);
            } else {
                flush(&mut points, &mut strokes);
            }
        }
        flush(&mut points, &mut strokes);
    }
    strokes.extend(tone_strokes);
    let silhouettes = features
        .iter()
        .filter(|f| f.class == FeatureClass::Silhouette)
        .count();
    let creases = features
        .iter()
        .filter(|f| f.class == FeatureClass::Crease)
        .count();
    let stats = NprRenderStats {
        geometry: 1,
        topology_edges: topology.len(),
        feature_segments: features.len(),
        silhouettes,
        creases,
        strokes: strokes.len(),
        stroke_vertices: strokes.iter().map(|s| s.vertices.len()).sum(),
        stroke_indices: strokes.iter().map(|s| s.indices.len()).sum(),
        viewport,
    };
    NprRenderPacket {
        fills,
        strokes,
        background: style.paper,
        debug_view,
        ink: style.ink,
        stats,
    }
}

fn append_hatching(
    output: &mut Vec<TessellatedStroke>,
    positions: [Vec2; 3],
    depths: [f32; 3],
    shade: f32,
    style: ComicInk,
    seed: u64,
) {
    let density = style.tone_density.clamp(0.0, 1.0) * ((0.5 - shade) / 1.25).clamp(0.0, 1.0);
    if density <= 0.001 {
        return;
    }
    let spacing = (style.hatching_spacing / (0.35 + density * 0.95)).clamp(1.0, 40.0);
    let angle = style.hatching_angle.to_radians();
    let direction = Vec2::new(angle.cos(), angle.sin());
    let normal = Vec2::new(-direction.y, direction.x);
    let mut append_direction =
        |normal: Vec2, direction: Vec2, id_start: u32, coverage_scale: f32, stroke_seed: u64| {
            let coordinates = positions.map(|p| p.dot(normal));
            let min = coordinates.iter().copied().fold(f32::INFINITY, f32::min);
            let max = coordinates
                .iter()
                .copied()
                .fold(f32::NEG_INFINITY, f32::max);
            let mut offset = (min / spacing).ceil() * spacing;
            let mut line_index = 0u32;
            while offset <= max {
                let intersections = triangle_line_intersections(positions, depths, normal, offset);
                if intersections.len() >= 2 {
                    let mut line = intersections;
                    line.sort_by(|a, b| a.0.dot(direction).total_cmp(&b.0.dot(direction)));
                    let mut stroke = tessellate_polyline(
                        id_start.wrapping_add(line_index),
                        crate::feature::FeatureClass::Crease,
                        &line[..2],
                        false,
                        style,
                        stroke_seed,
                    );
                    if coverage_scale < 1.0 {
                        for vertex in &mut stroke.vertices {
                            vertex.coverage *= coverage_scale;
                        }
                    }
                    output.push(stroke);
                }
                line_index = line_index.wrapping_add(1);
                offset += spacing;
            }
        };
    let stroke_id = (seed as u32).wrapping_mul(2654435761);
    append_direction(normal, direction, stroke_id, 1.0, seed);
    if style.hatching_cross > 0.001 {
        let cross_angle = angle + std::f32::consts::FRAC_PI_2;
        let cross_direction = Vec2::new(cross_angle.cos(), cross_angle.sin());
        let cross_normal = Vec2::new(-cross_direction.y, cross_direction.x);
        append_direction(
            cross_normal,
            cross_direction,
            stroke_id.wrapping_add(0x2000_0000),
            style.hatching_cross.clamp(0.0, 1.0),
            seed ^ 0x51ed_270b,
        );
    }
}

fn triangle_line_intersections(
    positions: [Vec2; 3],
    depths: [f32; 3],
    normal: Vec2,
    offset: f32,
) -> Vec<(Vec2, f32)> {
    let mut intersections = Vec::with_capacity(3);
    for i in 0..3 {
        let j = (i + 1) % 3;
        let a = positions[i];
        let b = positions[j];
        let da = a.dot(normal) - offset;
        let db = b.dot(normal) - offset;
        if da.abs() <= 1e-4 {
            intersections.push((a, depths[i]));
        }
        if da * db < 0.0 {
            let t = da / (da - db);
            intersections.push((a.lerp(b, t), depths[i] + (depths[j] - depths[i]) * t));
        }
    }
    intersections.sort_by(|a, b| {
        a.0.x
            .total_cmp(&b.0.x)
            .then_with(|| a.0.y.total_cmp(&b.0.y))
    });
    intersections.dedup_by(|a, b| a.0.distance_squared(b.0) < 1e-4);
    intersections
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify_features;
    #[test]
    fn cube_has_18_edges() {
        assert_eq!(build_topology(&NprGeometry::canonical_cube()).len(), 18);
    }
    #[test]
    fn packet_is_deterministic() {
        let g = NprGeometry::canonical_cube();
        let c = PerspectiveCamera::cube_default(1.0);
        let s = ComicInk {
            wobble: 0.4,
            ..Default::default()
        };
        assert_eq!(
            build_packet(&g, c, [512, 512], s, 7, NprDebugView::Final),
            build_packet(&g, c, [512, 512], s, 7, NprDebugView::Final)
        );
    }
    #[test]
    fn near_clipping_keeps_crossing_segment() {
        let c = PerspectiveCamera::cube_default(1.0);
        assert!(
            c.project_segment(
                c.position + c.forward * 0.01,
                c.position + c.forward * 2.0,
                Vec2::splat(512.0)
            )
            .is_some()
        );
    }

    #[test]
    fn triangulation_diagonals_do_not_become_feature_strokes() {
        let geometry = NprGeometry::canonical_cube();
        let topology = build_topology(&geometry);
        let features = classify_features(&geometry, &topology, glam::Vec3::Z, 0.35);
        assert!(features.len() <= 12);
    }

    #[test]
    fn cube_packet_is_stable_for_thirty_six_rotations() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera::cube_default(1.0);
        for step in 0..36 {
            let angle = step as f32 * std::f32::consts::TAU / 36.0;
            let mut rotated = geometry.clone();
            let rotation = glam::Mat3::from_rotation_y(angle);
            for vertex in &mut rotated.vertices {
                vertex.position = rotation * vertex.position;
            }
            let first = build_packet(
                &rotated,
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            );
            let second = build_packet(
                &rotated,
                camera,
                [512, 512],
                ComicInk::default(),
                42,
                NprDebugView::Final,
            );
            assert_eq!(first, second);
        }
    }

    #[test]
    fn tone_hatching_is_opt_in_and_stays_inside_face_bounds() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera::cube_default(1.0);
        let plain = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk::default(),
            42,
            NprDebugView::Final,
        );
        let hatched = build_packet(
            &geometry,
            camera,
            [512, 512],
            ComicInk {
                tone_density: 0.8,
                hatching_spacing: 5.0,
                ..Default::default()
            },
            42,
            NprDebugView::Final,
        );
        assert!(hatched.strokes.len() > plain.strokes.len());
        assert!(hatched.strokes.iter().all(|stroke| {
            stroke
                .vertices
                .iter()
                .all(|vertex| vertex.position.is_finite())
        }));
    }
}
