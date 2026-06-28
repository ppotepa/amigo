use std::collections::BTreeMap;

use amigo_math::Transform3;

use crate::renderer::{
    CachedMeshGeometry3d, ColorVertex, NprDebugOverlay3d, NprEntityPathHistory3d,
    NprStrokeFrameStats3d, NprStrokeSegmentVertex, Viewport,
    append_mesh_npr_debug_overlay_vertices_with_history,
    append_mesh_npr_line_vertices_with_history_and_stats,
};

#[derive(Debug, Default)]
pub(crate) struct CpuReferenceNprRenderer3d {
    path_history: BTreeMap<String, NprEntityPathHistory3d>,
}

impl CpuReferenceNprRenderer3d {
    pub(crate) fn append_mesh(
        &mut self,
        frame_counter: u64,
        entity_name: &str,
        vertices: &mut Vec<ColorVertex>,
        stroke_segments: &mut Vec<NprStrokeSegmentVertex>,
        viewport: &Viewport,
        camera: Transform3,
        camera_settings: amigo_render_api::Camera3dRenderSettings,
        geometry: &CachedMeshGeometry3d,
        transform: Transform3,
        settings: &amigo_render_api::NprLineSettings3d,
    ) -> NprStrokeFrameStats3d {
        append_mesh_npr_line_vertices_with_history_and_stats(
            &mut self.path_history,
            frame_counter,
            entity_name,
            vertices,
            Some(stroke_segments),
            viewport,
            camera,
            camera_settings,
            geometry,
            transform,
            settings,
        )
    }

    pub(crate) fn append_debug_overlay(
        &self,
        entity_name: &str,
        vertices: &mut Vec<ColorVertex>,
        viewport: &Viewport,
        camera: Transform3,
        camera_settings: amigo_render_api::Camera3dRenderSettings,
        geometry: &CachedMeshGeometry3d,
        transform: Transform3,
        settings: &amigo_render_api::NprLineSettings3d,
        overlay: NprDebugOverlay3d,
    ) {
        append_mesh_npr_debug_overlay_vertices_with_history(
            &self.path_history,
            entity_name,
            vertices,
            viewport,
            camera,
            camera_settings,
            geometry,
            transform,
            settings,
            overlay,
        );
    }
}
