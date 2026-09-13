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
    pub world_3d_materials: usize,
    pub world_3d_text: usize,
    pub game_ui_overlays: usize,
    pub debug_overlays: usize,
    pub ui_overlays: usize,
    pub render_graph_nodes: usize,
    pub post_fx_effects: usize,
    pub npr_geometry: usize,
    pub npr_surface_source_triangles: usize,
    pub npr_surface_proxy_triangles: usize,
    pub npr_topology_edges: usize,
    pub npr_feature_segments: usize,
    pub npr_silhouettes: usize,
    pub npr_creases: usize,
    pub npr_strokes: usize,
    pub npr_stroke_vertices: usize,
    pub npr_stroke_indices: usize,
    pub npr_stroke_budget_rejected: usize,
    pub npr_stroke_budget_exhausted: usize,
    pub npr_viewport: [u32; 2],
    pub npr_preset: Option<&'static str>,
    pub npr_debug_view: Option<amigo_render_npr::NprDebugView>,
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
