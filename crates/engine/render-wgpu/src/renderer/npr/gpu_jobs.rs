use std::sync::Arc;

use amigo_math::Transform3;

use crate::renderer::{CachedMeshGeometry3d, NprStrokeFrameStats3d};

#[derive(Debug, Clone)]
pub(crate) struct NprGpuMeshJob3d {
    pub entity_name: String,
    pub mesh_key: String,
    pub geometry: Arc<CachedMeshGeometry3d>,
    pub transform: Transform3,
    pub settings: amigo_render_api::NprLineSettings3d,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, Default)]
pub(crate) struct NprGpuFrameStats3d {
    pub meshes: usize,
    pub projected_vertices: usize,
    pub classified_edges: usize,
    pub built_stroke_capacity: usize,
    pub enqueued_triangles: usize,
    pub topology_uploads: usize,
    pub buffer_capacity_bytes: u64,
    pub frame_jobs: usize,
    pub projected_vertices_capacity: usize,
    pub visible_segments_capacity: usize,
    pub endpoint_heads_capacity: usize,
    pub endpoint_entries_capacity: usize,
    pub path_links_capacity: usize,
    pub path_segments_capacity: usize,
    pub path_states_capacity: usize,
    pub stroke_segments_capacity: usize,
    pub debug_mode: &'static str,
}

impl NprStrokeFrameStats3d {
    pub(crate) fn add_gpu_realtime(&mut self, stats: NprGpuFrameStats3d) {
        self.gpu_realtime_enqueued_edges += stats.classified_edges;
        self.gpu_realtime_enqueued_triangles += stats.enqueued_triangles;
        self.gpu_realtime_topology_uploads += stats.topology_uploads;
        self.gpu_realtime_buffer_capacity_bytes += stats.buffer_capacity_bytes;
        self.gpu_realtime_frame_jobs += stats.frame_jobs;
        self.gpu_realtime_projected_vertices_capacity += stats.projected_vertices_capacity;
        self.gpu_realtime_visible_segments_capacity += stats.visible_segments_capacity;
        self.gpu_realtime_endpoint_heads_capacity += stats.endpoint_heads_capacity;
        self.gpu_realtime_endpoint_entries_capacity += stats.endpoint_entries_capacity;
        self.gpu_realtime_path_links_capacity += stats.path_links_capacity;
        self.gpu_realtime_path_states_capacity += stats.path_states_capacity;
        self.gpu_realtime_path_segments_capacity += stats.path_segments_capacity;
        self.gpu_realtime_stroke_segments_capacity += stats.stroke_segments_capacity;
        if self.gpu_realtime_debug_mode.is_empty() {
            self.gpu_realtime_debug_mode = stats.debug_mode.to_owned();
        } else if self.gpu_realtime_debug_mode != stats.debug_mode {
            self.gpu_realtime_debug_mode = "mixed".to_owned();
        }
    }
}
