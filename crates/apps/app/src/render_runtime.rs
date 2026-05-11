mod composition;
mod context;
mod diagnostics;
mod extractors;
mod graph;
mod services;
mod stats;

#[cfg(test)]
mod tests;

pub(crate) use composition::AppFrameCompositionBuilder;
pub(crate) use context::AppRenderExtractContext;
#[cfg(test)]
pub(crate) use context::AppRenderFramePacket;
pub(crate) use diagnostics::RenderCompositionDiagnosticsService;
pub(crate) use extractors::default_app_render_extractor_registry;
pub(crate) use graph::{AppFrameGraphBuildInfo, build_frame_graph_from_plan};
pub(crate) use services::{
    build_global_light2d_scene_service_from_packet, build_layered_image_scene_service_from_packet,
    build_light_route2d_scene_service_from_packet, build_lightmap2d_scene_service_from_packet,
    build_render_layer2d_scene_service_from_packet, build_sprite_scene_service_from_packet,
    build_text2d_scene_service_from_packet, build_tilemap_scene_service_from_packet,
    build_vector_scene_service_from_packet,
};
pub(crate) use stats::{RenderFrameStats, RenderFrameStatsService};

#[cfg(test)]
pub(crate) use services::{
    build_material_scene_service_from_packet, build_mesh_scene_service_from_packet,
    build_text3d_scene_service_from_packet,
};
