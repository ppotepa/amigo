mod context;
mod extractors;
mod services;

#[cfg(test)]
mod tests;

pub(crate) use context::AppRenderExtractContext;
#[cfg(test)]
pub(crate) use context::AppRenderFramePacket;
pub(crate) use extractors::default_app_render_extractor_registry;
pub(crate) use services::{
    build_sprite_scene_service_from_packet, build_text2d_scene_service_from_packet,
    build_tilemap_scene_service_from_packet, build_vector_scene_service_from_packet,
};

#[cfg(test)]
pub(crate) use services::{
    build_material_scene_service_from_packet, build_mesh_scene_service_from_packet,
    build_text3d_scene_service_from_packet,
};
