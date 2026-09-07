use crate::{
    NprDebugView,
    camera::PerspectiveCamera,
    feature::{FeatureClass, classify_features},
    geometry::NprGeometry,
    style::ComicInk,
    tessellation::{TessellatedStroke, tessellate_segment_with_depth},
    topology::{build_topology, face_normal},
};
use glam::{Vec2, Vec4};

#[derive(Debug, Clone, PartialEq)]
pub struct NprFillTriangle {
    pub positions: [Vec2; 3],
    pub color: Vec4,
    pub depth: f32,
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
    pub stats: NprRenderStats,
}

pub fn build_packet(
    geometry: &NprGeometry,
    camera: PerspectiveCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    let vp = Vec2::new(viewport[0] as f32, viewport[1] as f32);
    let topology = build_topology(geometry);
    let features = classify_features(geometry, &topology, -camera.forward, 0.35);
    let mut fills = Vec::new();
    let light_direction = glam::Vec3::new(-0.4, 0.7, 1.0).normalize();
    for (face_index, tri) in geometry.triangles.iter().enumerate() {
        if let (Some(a), Some(b), Some(c)) = (
            camera.project(geometry.vertices[tri[0] as usize].position, vp),
            camera.project(geometry.vertices[tri[1] as usize].position, vp),
            camera.project(geometry.vertices[tri[2] as usize].position, vp),
        ) {
            let shade = face_normal(geometry, face_index as u32).dot(light_direction);
            let color = if shade < -0.1 {
                style.shadow
            } else if shade < 0.45 {
                style.mid
            } else {
                style.light
            };
            fills.push(NprFillTriangle {
                positions: [a.screen, b.screen, c.screen],
                color,
                depth: (a.depth + b.depth + c.depth) / 3.0,
            });
        }
    }
    let strokes: Vec<_> = features
        .iter()
        .enumerate()
        .filter_map(|(id, feature)| {
            camera
                .project_segment(
                    geometry.vertices[feature.edge.a as usize].position,
                    geometry.vertices[feature.edge.b as usize].position,
                    vp,
                )
                .map(|(a, b)| {
                    tessellate_segment_with_depth(
                        id as u32,
                        feature.class,
                        (a.screen, b.screen),
                        (a.depth, b.depth),
                        style,
                        seed,
                    )
                })
        })
        .collect();
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
        stats,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
