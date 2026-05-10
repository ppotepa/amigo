use std::sync::Mutex;

#[derive(Debug, Clone, Default)]
pub(crate) struct RenderFrameStats {
    pub(crate) frame_index: u64,
    pub(crate) window_width: u32,
    pub(crate) window_height: u32,
    pub(crate) world_2d_tilemaps: usize,
    pub(crate) world_2d_sprites: usize,
    pub(crate) world_2d_layered_images: usize,
    pub(crate) world_2d_render_layers: usize,
    pub(crate) world_2d_light_routes: usize,
    pub(crate) world_2d_global_lights: usize,
    pub(crate) world_2d_lightmaps: usize,
    pub(crate) world_2d_light_groups: usize,
    pub(crate) world_2d_vectors: usize,
    pub(crate) world_2d_text: usize,
    pub(crate) world_2d_particles: usize,
    pub(crate) world_3d_meshes: usize,
    pub(crate) world_3d_materials: usize,
    pub(crate) world_3d_text: usize,
    pub(crate) ui_overlays: usize,
}

#[derive(Debug, Default)]
pub(crate) struct RenderFrameStatsService {
    inner: Mutex<RenderFrameStats>,
}

impl RenderFrameStatsService {
    pub(crate) fn set(&self, stats: RenderFrameStats) {
        *self
            .inner
            .lock()
            .expect("render stats mutex should not be poisoned") = stats;
    }

    pub(crate) fn snapshot(&self) -> RenderFrameStats {
        self.inner
            .lock()
            .expect("render stats mutex should not be poisoned")
            .clone()
    }
}
