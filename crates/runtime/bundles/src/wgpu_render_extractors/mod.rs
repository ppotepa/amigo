mod composition;
pub mod context;
mod host_overlay;
pub mod visual_2d_items;
mod world_2d;
mod world_3d;

// WGPU render extractors are backend bridges only.
// Domain extraction logic must remain in domain crates.
// This module adapts domain extractors into WgpuRenderFramePacket.

pub use host_overlay::register_host_render_extractor_provider;

pub use composition::{WgpuFrameCompositionBuilder, WgpuFrameCompositionOptions};
pub use context::WgpuRenderExtractorRegistry;
pub use visual_2d_items::{
    RenderSpace2d, Renderable2dCommon, Renderable2dItem, Renderable2dKind,
    Renderable2dPayload, render_contribution_decisions_summary,
    supported_renderable_2d_component_kinds,
};

pub fn default_wgpu_render_extractor_registry() -> WgpuRenderExtractorRegistry {
    let mut registry = WgpuRenderExtractorRegistry::new();
    world_2d::register_world_2d_render_extractors(&mut registry);
    world_3d::register_world_3d_render_extractors(&mut registry);
    host_overlay::register_host_overlay_render_extractors(&mut registry);
    registry
}
