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
