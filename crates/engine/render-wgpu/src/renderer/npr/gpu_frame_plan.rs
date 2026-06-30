use crate::renderer::{GpuNprFrameUniforms3d, NprStrokeSegmentVertex, Viewport};

use super::{
    NprGpuMeshJob3d, npr_gpu_endpoint_head_count, npr_gpu_pass_count,
    npr_gpu_path_segment_capacity_units, npr_gpu_stroke_segment_capacity_units,
    scaled_face_id_dimensions,
};

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct NprGpuFramePlan3d {
    pub target_width: u32,
    pub target_height: u32,
    pub total_vertices: usize,
    pub total_edges: usize,
    pub total_triangles: usize,
    pub projected_capacity: u64,
    pub visible_segments_capacity: u64,
    pub endpoint_heads_capacity: u64,
    pub endpoint_entries_capacity: u64,
    pub path_links_capacity: u64,
    pub path_segments_capacity: u64,
    pub path_states_capacity: u64,
    pub aggregated_paths_capacity: u64,
    pub stroke_segments_capacity: u64,
    pub uniform_size: usize,
}

impl NprGpuFramePlan3d {
    pub(crate) fn built_stroke_capacity_units(self) -> usize {
        (self.stroke_segments_capacity / std::mem::size_of::<NprStrokeSegmentVertex>() as u64)
            as usize
    }

    pub(crate) fn allocated_projected_capacity(self) -> u64 {
        self.projected_capacity.max(64)
    }

    pub(crate) fn allocated_visible_segments_capacity(self) -> u64 {
        self.visible_segments_capacity.max(64)
    }

    pub(crate) fn allocated_endpoint_heads_capacity(self) -> u64 {
        self.endpoint_heads_capacity.max(64)
    }

    pub(crate) fn allocated_endpoint_entries_capacity(self) -> u64 {
        self.endpoint_entries_capacity.max(64)
    }

    pub(crate) fn allocated_path_links_capacity(self) -> u64 {
        self.path_links_capacity.max(64)
    }

    pub(crate) fn allocated_path_segments_capacity(self) -> u64 {
        self.path_segments_capacity.max(64)
    }

    pub(crate) fn allocated_path_states_capacity(self) -> u64 {
        self.path_states_capacity.max(64)
    }

    pub(crate) fn allocated_aggregated_paths_capacity(self) -> u64 {
        self.aggregated_paths_capacity.max(64)
    }

    pub(crate) fn allocated_stroke_segments_capacity(self) -> u64 {
        self.stroke_segments_capacity.max(64)
    }
}

pub(crate) fn build_npr_gpu_frame_plan(
    frame_jobs: &[NprGpuMeshJob3d],
    viewport: &Viewport,
) -> NprGpuFramePlan3d {
    let total_vertices = frame_jobs
        .iter()
        .map(|job| job.geometry.vertex_count())
        .sum::<usize>();
    let total_edges = frame_jobs
        .iter()
        .map(|job| job.geometry.edge_count())
        .sum::<usize>();
    let total_triangles = frame_jobs
        .iter()
        .map(|job| job.geometry.triangle_count())
        .sum::<usize>();
    let max_visibility_dimension_px = frame_jobs
        .iter()
        .map(|job| job.settings.visibility_max_dimension_px.max(1.0))
        .fold(1.0, f32::max);
    let (target_width, target_height) =
        scaled_face_id_dimensions(viewport, max_visibility_dimension_px);
    let projected_capacity = (total_vertices * std::mem::size_of::<super::GpuNprProjectedVertex3d>())
        as u64;
    let visible_segments_capacity =
        (total_edges * std::mem::size_of::<super::GpuNprVisibleSegment3d>()) as u64;
    let endpoint_head_count = npr_gpu_endpoint_head_count(total_edges.max(1));
    let endpoint_heads_capacity =
        endpoint_head_count as u64 * std::mem::size_of::<u32>() as u64;
    let endpoint_entries_capacity = (frame_jobs
        .iter()
        .map(|job| job.geometry.edge_count() * 2)
        .sum::<usize>()
        * std::mem::size_of::<super::GpuNprEndpointEntry3d>()) as u64;
    let path_links_capacity =
        (total_edges * std::mem::size_of::<super::GpuNprPathLink3d>()) as u64;
    let job_count = frame_jobs.len().max(1);
    let path_segments_capacity = (frame_jobs
        .iter()
        .map(|job| npr_gpu_path_segment_capacity_units(job.geometry.edge_count(), job_count))
        .sum::<usize>()
        * std::mem::size_of::<super::GpuNprPathSegment3d>()) as u64;
    let path_states_capacity =
        (total_edges * std::mem::size_of::<super::GpuNprPathState3d>()) as u64;
    let aggregated_paths_capacity =
        (total_edges * std::mem::size_of::<super::GpuNprAggregatedPath3d>()) as u64;
    let stroke_segments_capacity = (frame_jobs
        .iter()
        .map(|job| {
            npr_gpu_stroke_segment_capacity_units(
                job.geometry.edge_count(),
                npr_gpu_pass_count(&job.settings),
                job.settings.pipeline.budget_strategy,
            )
        })
        .sum::<usize>()
        * std::mem::size_of::<NprStrokeSegmentVertex>()) as u64;

    NprGpuFramePlan3d {
        target_width,
        target_height,
        total_vertices,
        total_edges,
        total_triangles,
        projected_capacity,
        visible_segments_capacity,
        endpoint_heads_capacity,
        endpoint_entries_capacity,
        path_links_capacity,
        path_segments_capacity,
        path_states_capacity,
        aggregated_paths_capacity,
        stroke_segments_capacity,
        uniform_size: std::mem::size_of::<GpuNprFrameUniforms3d>(),
    }
}
