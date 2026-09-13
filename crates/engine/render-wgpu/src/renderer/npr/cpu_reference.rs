use std::collections::BTreeMap;

use amigo_math::{Transform3, Vec3};

use crate::renderer::{
    CachedMeshGeometry3d, ColorVertex, NprDebugOverlay3d, NprEntityPathHistory3d,
    NprPathBuildResult3d, NprPathBuildStats3d, NprStrokeFrameStats3d, NprStrokeSegmentVertex,
    Viewport, append_npr_debug_path_vertices, append_npr_rejected_technical_vertices,
    append_npr_styled_path_vertices, build_npr_face_visibility_buffer,
    build_npr_stroke_paths_for_settings, collect_npr_edge_fragments_for_mesh, cross, dot,
    normalize, project_point_with_camera, rotate_x, rotate_y, rotate_z, sub, transform_point_3d,
    triangle_center,
};

use super::types::NprRejectedTechnicalCandidate;

#[cfg(test)]
use crate::renderer::NprStrokePath;

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

#[cfg(test)]
pub(crate) fn append_mesh_npr_line_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) {
    let paths = build_npr_stroke_paths_for_mesh(
        viewport,
        camera,
        camera_settings,
        geometry,
        transform,
        settings,
    )
    .paths;
    append_npr_paths_as_vertices(vertices, viewport, &paths, settings);
}

pub(crate) fn append_mesh_npr_line_vertices_with_history_and_stats(
    history: &mut BTreeMap<String, NprEntityPathHistory3d>,
    frame_index: u64,
    entity_name: &str,
    vertices: &mut Vec<ColorVertex>,
    npr_stroke_segments: Option<&mut Vec<NprStrokeSegmentVertex>>,
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeFrameStats3d {
    let path_build_start = std::time::Instant::now();
    let path_build_result = build_npr_stroke_paths_for_mesh(
        viewport,
        camera,
        camera_settings,
        geometry,
        transform,
        settings,
    );
    let paths = path_build_result.paths;
    let path_build_us = path_build_start.elapsed().as_secs_f64() * 1_000_000.0;
    let stabilize_start = std::time::Instant::now();
    let stabilized = crate::renderer::stabilize_npr_paths_for_entity(
        history,
        frame_index,
        entity_name,
        settings,
        paths,
    );
    let stabilize_us = stabilize_start.elapsed().as_secs_f64() * 1_000_000.0;
    let mut stats = NprStrokeFrameStats3d {
        paths: stabilized.len(),
        path_build_us,
        path_project_us: path_build_result.stats.project_us,
        path_visibility_us: path_build_result.stats.visibility_us,
        path_edge_sample_us: path_build_result.stats.edge_sample_us,
        path_stitch_us: path_build_result.stats.stitch_us,
        path_visible_edges: path_build_result.stats.visible_edges,
        path_fragments: path_build_result.stats.fragments,
        stabilize_us,
        ..NprStrokeFrameStats3d::default()
    };
    let stroke_vertices_start = std::time::Instant::now();
    let entity_history = history
        .get_mut(entity_name)
        .expect("stabilized NPR paths should initialize entity history");
    let mut npr_stroke_segments = npr_stroke_segments;
    for path in &stabilized {
        stats.record_path_kind(path.kind);
        let cached_plan = entity_history
            .paths
            .get(&path.path_id)
            .and_then(|state| state.cached_plan.as_ref());
        let used_plan = append_npr_styled_path_vertices(
            vertices,
            npr_stroke_segments.as_deref_mut(),
            viewport,
            path,
            settings,
            cached_plan,
            &mut stats,
        );
        if let Some(state) = entity_history.paths.get_mut(&path.path_id) {
            state.cached_plan = Some(used_plan);
        }
    }
    stats.stroke_vertices_us = stroke_vertices_start.elapsed().as_secs_f64() * 1_000_000.0;
    stats.meshes = 1;
    stats
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn append_mesh_npr_debug_overlay_vertices_with_history(
    history: &BTreeMap<String, NprEntityPathHistory3d>,
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
    let mut paths = history
        .get(entity_name)
        .map(|entity| {
            entity
                .paths
                .values()
                .map(|state| state.path.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let mut rejected_technical = Vec::<NprRejectedTechnicalCandidate>::new();

    if overlay == NprDebugOverlay3d::TechnicalSelection {
        let build_result = build_npr_stroke_paths_for_mesh(
            viewport,
            camera,
            camera_settings,
            geometry,
            transform,
            settings,
        );
        rejected_technical = build_result.rejected_technical;
        if paths.is_empty() {
            paths = build_result.paths;
        }
    }

    if paths.is_empty() {
        paths = build_npr_stroke_paths_for_mesh(
            viewport,
            camera,
            camera_settings,
            geometry,
            transform,
            settings,
        )
        .paths;
    }

    if overlay == NprDebugOverlay3d::TechnicalSelection {
        append_npr_rejected_technical_vertices(vertices, viewport, &rejected_technical);
    }

    for path in &paths {
        append_npr_debug_path_vertices(vertices, viewport, path, settings, overlay);
    }
}

fn build_npr_stroke_paths_for_mesh(
    viewport: &Viewport,
    camera: Transform3,
    camera_settings: amigo_render_api::Camera3dRenderSettings,
    geometry: &CachedMeshGeometry3d,
    transform: Transform3,
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprPathBuildResult3d {
    if settings.passes == 0 || settings.width_px <= 0.0 {
        return NprPathBuildResult3d {
            paths: Vec::new(),
            stats: NprPathBuildStats3d::default(),
            rejected_technical: Vec::new(),
        };
    }

    let project_start = std::time::Instant::now();
    let world_vertices = geometry
        .vertices()
        .iter()
        .map(|vertex| transform_point_3d(*vertex, transform))
        .collect::<Vec<_>>();
    let projected_vertices = world_vertices
        .iter()
        .map(|vertex| {
            project_point_with_camera(
                *vertex,
                camera,
                *viewport,
                camera_settings.fov_y_degrees,
                camera_settings.near_clip,
                camera_settings.far_clip,
            )
        })
        .collect::<Vec<_>>();
    let project_us = project_start.elapsed().as_secs_f64() * 1_000_000.0;
    let visibility_start = std::time::Instant::now();
    let mut face_front = Vec::with_capacity(geometry.triangle_count());
    let mut face_view_alignment = Vec::with_capacity(geometry.triangle_count());
    for triangle in geometry.triangles() {
        let world = triangle.indices.map(|index| world_vertices[index]);
        let normal = normalize(cross(sub(world[1], world[0]), sub(world[2], world[0])));
        let center = triangle_center(world);
        let to_camera = normalize(sub(camera.translation, center));
        let view_dot = dot(normal, to_camera);
        face_front.push(view_dot > 0.0);
        face_view_alignment.push(view_dot.abs());
    }
    let visibility = build_npr_face_visibility_buffer(
        geometry,
        &projected_vertices,
        viewport,
        &face_front,
        settings.visibility_max_dimension_px,
    );
    let face_visible = face_front
        .iter()
        .enumerate()
        .map(|(index, front)| {
            *front && visibility.face_visible.get(index).copied().unwrap_or(false)
        })
        .collect::<Vec<_>>();
    let world_normals = geometry
        .triangles()
        .iter()
        .map(|triangle| transform_direction_3d(triangle.normal, transform))
        .collect::<Vec<_>>();
    let visibility_us = visibility_start.elapsed().as_secs_f64() * 1_000_000.0;

    let edge_sample_start = std::time::Instant::now();
    let edge_sample_result = collect_npr_edge_fragments_for_mesh(
        geometry,
        viewport,
        settings,
        &visibility,
        &world_vertices,
        &projected_vertices,
        &face_visible,
        &face_front,
        &face_view_alignment,
        &world_normals,
    );
    let fragments = edge_sample_result.fragments;
    let visible_edges = edge_sample_result.visible_edges;
    let rejected_technical = edge_sample_result.rejected_technical;
    let edge_sample_us = edge_sample_start.elapsed().as_secs_f64() * 1_000_000.0;

    let stitch_start = std::time::Instant::now();
    let fragment_count = fragments.len();
    let paths = build_npr_stroke_paths_for_settings(&fragments, viewport, settings);
    let stitch_us = stitch_start.elapsed().as_secs_f64() * 1_000_000.0;
    NprPathBuildResult3d {
        paths,
        stats: NprPathBuildStats3d {
            project_us,
            visibility_us,
            edge_sample_us,
            stitch_us,
            visible_edges,
            fragments: fragment_count,
        },
        rejected_technical,
    }
}

#[cfg(test)]
fn append_npr_paths_as_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    paths: &[NprStrokePath],
    settings: &amigo_render_api::NprLineSettings3d,
) -> NprStrokeFrameStats3d {
    let mut stats = NprStrokeFrameStats3d {
        paths: paths.len(),
        ..NprStrokeFrameStats3d::default()
    };
    for path in paths {
        stats.record_path_kind(path.kind);
        let _ = append_npr_styled_path_vertices(
            vertices, None, viewport, path, settings, None, &mut stats,
        );
    }
    stats
}

fn transform_direction_3d(point: Vec3, transform: Transform3) -> Vec3 {
    let scaled = Vec3::new(
        point.x * transform.scale.x.signum(),
        point.y * transform.scale.y.signum(),
        point.z * transform.scale.z.signum(),
    );
    let rotated_x = rotate_x(scaled, transform.rotation_euler.x);
    let rotated_y = rotate_y(rotated_x, transform.rotation_euler.y);
    normalize(rotate_z(rotated_y, transform.rotation_euler.z))
}
