pub mod context;
mod host_overlay;
mod world_2d;
mod world_3d;

pub use host_overlay::register_host_render_extractor_provider;

pub use context::WgpuRenderExtractorRegistry;

pub fn default_wgpu_render_extractor_registry() -> WgpuRenderExtractorRegistry {
    let mut registry = WgpuRenderExtractorRegistry::new();
    world_2d::register_world_2d_render_extractors(&mut registry);
    world_3d::register_world_3d_render_extractors(&mut registry);
    host_overlay::register_host_overlay_render_extractors(&mut registry);
    registry
}



