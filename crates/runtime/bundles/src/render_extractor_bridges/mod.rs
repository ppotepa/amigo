mod composition;
pub mod context;
mod host_overlay;
pub mod light_sources_2d;
pub mod visual_2d_items;
mod world_2d;
mod world_3d;

// WGPU render extractors are backend bridges only.
// Domain extraction logic must remain in domain crates.
// This module adapts domain extractors into WgpuRenderFramePacket.

pub use host_overlay::{
    register_host_overlay_render_extractors, register_host_render_extractor_provider,
    register_surface_overlay_render_extractors,
};

pub use composition::{WgpuFrameCompositionBuilder, WgpuFrameCompositionOptions};
pub use context::WgpuRenderExtractorRegistry;
pub use light_sources_2d::{
    collect_camera_optical_candidates_from_light_sources_2d, collect_light_sources_2d,
    format_light_sources_2d,
};
pub use visual_2d_items::{
    RenderSpace2d, Renderable2dCommon, Renderable2dItem, Renderable2dKind,
    render_contribution_decisions_summary,
};
pub(crate) use world_2d::{
    available_world_2d_plugin_bridge_installers, register_world_2d_builtin_render_extractors,
};
pub use world_3d::register_world_3d_render_extractors;

pub fn default_wgpu_render_extractor_registry() -> WgpuRenderExtractorRegistry {
    let mut registry = WgpuRenderExtractorRegistry::new();
    world_2d::register_world_2d_render_extractors(&mut registry);
    world_3d::register_world_3d_render_extractors(&mut registry);
    host_overlay::register_host_overlay_render_extractors(&mut registry);
    registry
}
