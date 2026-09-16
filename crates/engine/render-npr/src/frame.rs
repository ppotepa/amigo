use crate::{ComicInk, NprCamera, NprDebugView, NprGeometry, NprMediumDefinition, NprPaperDefinition, NprTemporalState, PerspectiveCamera, TessellatedStroke};
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
    pub hatching_marks: usize,
    pub strokes: usize,
    pub stroke_vertices: usize,
    pub stroke_indices: usize,
    pub viewport: [u32; 2],
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NprMedium {
    Ink,
    Graphite,
}
#[derive(Debug, Clone, PartialEq)]
pub struct NprRenderPacket {
    pub fills: Vec<NprFillTriangle>,
    pub strokes: Vec<TessellatedStroke>,
    pub background: Vec4,
    pub paper: NprPaperDefinition,
    pub medium: NprMediumDefinition,
    /// Changes only backend material deposition; never rerolls planned paths.
    pub material_epoch: u64,
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
    build_packet_with_camera(geometry, camera.into(), viewport, style, seed, debug_view)
}

pub fn build_packet_with_camera(
    geometry: &NprGeometry,
    camera: NprCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    use crate::{MinimalInkPipeline, NprPipeline, NprPipelineInput};
    MinimalInkPipeline::default().build(NprPipelineInput {
        geometry,
        camera,
        viewport,
        style,
        seed,
        debug_view,
        temporal: NprTemporalState::default(),
    })
}

pub fn build_pencil_animation_packet_with_camera(
    geometry: &NprGeometry,
    camera: NprCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
) -> NprRenderPacket {
    build_pencil_animation_packet_with_camera_and_temporal(
        geometry,
        camera,
        viewport,
        style,
        seed,
        debug_view,
        NprTemporalState::default(),
    )
}

pub fn build_pencil_animation_packet_with_camera_and_temporal(
    geometry: &NprGeometry,
    camera: NprCamera,
    viewport: [u32; 2],
    style: ComicInk,
    seed: u64,
    debug_view: NprDebugView,
    temporal: NprTemporalState,
) -> NprRenderPacket {
    use crate::{NprPipeline, NprPipelineInput, PencilAnimationPipeline};
    PencilAnimationPipeline::default().build(NprPipelineInput {
        geometry,
        camera,
        viewport,
        style,
        seed,
        debug_view,
        temporal,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{build_topology, classify_features};
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
    fn pencil_pipeline_selects_graphite_without_changing_packet_contract() {
        let geometry = NprGeometry::canonical_cube();
        let packet = build_pencil_animation_packet_with_camera(
            &geometry,
            PerspectiveCamera::cube_default(1.0).into(),
            [512, 512],
            ComicInk::default(),
            7,
            NprDebugView::Final,
        );
        assert_eq!(packet.medium.kind(), NprMedium::Graphite);
        assert!(!packet.strokes.is_empty());
        assert!(!packet.strokes.iter().any(|stroke| stroke.class == crate::FeatureClass::Hatching));
        assert!(packet.strokes.iter().any(|stroke| stroke.vertices.len() > 4));
        assert!(packet.stats.hatching_marks == 0);
        let silhouette = packet.strokes.iter().find(|stroke| stroke.class == crate::FeatureClass::Silhouette).unwrap();
        assert!(silhouette.medium.is_some());
        assert!(packet.fills.is_empty());
    }

    #[test]
    fn material_boil_does_not_change_planned_paths_but_path_epoch_does() {
        let geometry = NprGeometry::canonical_cube();
        let camera = PerspectiveCamera::cube_default(1.0).into();
        let base = build_pencil_animation_packet_with_camera_and_temporal(
            &geometry, camera, [512, 512], ComicInk::default(), 7, NprDebugView::Final,
            NprTemporalState { path_epoch: 0, material_epoch: 0 },
        );
        let boil = build_pencil_animation_packet_with_camera_and_temporal(
            &geometry, camera, [512, 512], ComicInk::default(), 7, NprDebugView::Final,
            NprTemporalState { path_epoch: 0, material_epoch: 3 },
        );
        let redraw = build_pencil_animation_packet_with_camera_and_temporal(
            &geometry, camera, [512, 512], ComicInk::default(), 7, NprDebugView::Final,
            NprTemporalState { path_epoch: 3, material_epoch: 3 },
        );
        assert_eq!(base.strokes, boil.strokes);
        assert_ne!(boil.strokes, redraw.strokes);
        assert_ne!(base.material_epoch, boil.material_epoch);
    }
}
