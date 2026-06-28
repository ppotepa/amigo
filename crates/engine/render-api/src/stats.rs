#[derive(Debug, Clone, Default)]
pub struct RenderFrameStats {
    pub frame_index: u64,
    pub window_width: u32,
    pub window_height: u32,
    pub world_2d_tilemaps: usize,
    pub world_2d_sprites: usize,
    pub world_2d_layered_images: usize,
    pub world_2d_render_layers: usize,
    pub world_2d_light_routes: usize,
    pub world_2d_global_lights: usize,
    pub world_2d_lightmaps: usize,
    pub world_2d_light_groups: usize,
    pub world_2d_vectors: usize,
    pub world_2d_beacons: usize,
    pub world_2d_text: usize,
    pub world_2d_particles: usize,
    pub world_3d_meshes: usize,
    pub world_3d_npr_meshes: usize,
    pub world_3d_npr_gpu_realtime_meshes: usize,
    pub world_3d_npr_cpu_reference_meshes: usize,
    pub world_3d_npr_gpu_realtime_enqueued_edges: usize,
    pub world_3d_npr_gpu_realtime_enqueued_triangles: usize,
    pub world_3d_npr_gpu_realtime_topology_uploads: usize,
    pub world_3d_npr_gpu_realtime_buffer_capacity_bytes: u64,
    pub world_3d_npr_gpu_realtime_frame_jobs: usize,
    pub world_3d_npr_gpu_realtime_projected_vertices_capacity: usize,
    pub world_3d_npr_gpu_realtime_visible_segments_capacity: usize,
    pub world_3d_npr_gpu_realtime_endpoint_heads_capacity: usize,
    pub world_3d_npr_gpu_realtime_endpoint_entries_capacity: usize,
    pub world_3d_npr_gpu_realtime_path_links_capacity: usize,
    pub world_3d_npr_gpu_realtime_path_segments_capacity: usize,
    pub world_3d_npr_gpu_realtime_stroke_segments_capacity: usize,
    pub world_3d_npr_gpu_realtime_debug_mode: String,
    pub world_3d_npr_paths: usize,
    pub world_3d_npr_boundary_paths: usize,
    pub world_3d_npr_silhouette_paths: usize,
    pub world_3d_npr_crease_paths: usize,
    pub world_3d_npr_seam_paths: usize,
    pub world_3d_npr_feature_paths: usize,
    pub world_3d_npr_contact_paths: usize,
    pub world_3d_npr_brush_samples: usize,
    pub world_3d_npr_strip_vertices: usize,
    pub world_3d_npr_primary_passes: usize,
    pub world_3d_npr_search_passes: usize,
    pub world_3d_npr_dropout_intervals: usize,
    pub world_3d_npr_cached_plan_hits: usize,
    pub world_3d_npr_cached_plan_misses: usize,
    pub world_3d_npr_path_build_us: f64,
    pub world_3d_npr_stabilize_us: f64,
    pub world_3d_npr_stroke_vertices_us: f64,
    pub world_3d_npr_path_project_us: f64,
    pub world_3d_npr_path_visibility_us: f64,
    pub world_3d_npr_path_edge_sample_us: f64,
    pub world_3d_npr_path_stitch_us: f64,
    pub world_3d_npr_path_visible_edges: usize,
    pub world_3d_npr_path_fragments: usize,
    pub offscreen_color_buffer_writes: usize,
    pub offscreen_color_buffer_reallocs: usize,
    pub offscreen_color_upload_bytes: u64,
    pub offscreen_color_buffer_capacity_bytes: u64,
    pub world_3d_materials: usize,
    pub world_3d_text: usize,
    pub game_ui_overlays: usize,
    pub debug_overlays: usize,
    pub ui_overlays: usize,
    pub render_graph_nodes: usize,
    pub post_fx_effects: usize,
}

use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct RenderFrameStatsService {
    inner: Mutex<RenderFrameStats>,
}

impl RenderFrameStatsService {
    pub fn set(&self, stats: RenderFrameStats) {
        *self
            .inner
            .lock()
            .expect("render stats mutex should not be poisoned") = stats;
    }

    pub fn snapshot(&self) -> RenderFrameStats {
        self.inner
            .lock()
            .expect("render stats mutex should not be poisoned")
            .clone()
    }
}
