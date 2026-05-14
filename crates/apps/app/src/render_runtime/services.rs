use amigo_runtime_bundles::amigo_2d_composition::{
    LightRoute2dSceneService, RenderLayer2dSceneService,
};
use amigo_runtime_bundles::amigo_2d_layered_image::LayeredImageSceneService;
use amigo_runtime_bundles::amigo_2d_lighting::{GlobalLight2dSceneService, LightMap2dSceneService};
use amigo_runtime_bundles::amigo_2d_sprite::SpriteSceneService;
use amigo_runtime_bundles::amigo_2d_text::Text2dSceneService;
use amigo_runtime_bundles::amigo_2d_tilemap::TileMap2dSceneService;
use amigo_runtime_bundles::amigo_2d_vector::VectorSceneService;

use amigo_render_wgpu::WgpuRenderFramePacket;

pub(crate) fn build_sprite_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> SpriteSceneService {
    let service = SpriteSceneService::default();
    for command in packet.world_2d_sprites() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_layered_image_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LayeredImageSceneService {
    let service = LayeredImageSceneService::default();
    for command in packet.world_2d_layered_images() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_render_layer2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> RenderLayer2dSceneService {
    let service = RenderLayer2dSceneService::default();
    for command in packet.world_2d_render_layers() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_light_route2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LightRoute2dSceneService {
    let service = LightRoute2dSceneService::default();
    for command in packet.world_2d_light_routes() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_global_light2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> GlobalLight2dSceneService {
    let service = GlobalLight2dSceneService::default();
    for command in packet.world_2d_global_lights() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_lightmap2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LightMap2dSceneService {
    let service = LightMap2dSceneService::default();
    for command in packet.world_2d_lightmaps() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_tilemap_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> TileMap2dSceneService {
    let service = TileMap2dSceneService::default();
    for command in packet.world_2d_tilemaps() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_vector_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> VectorSceneService {
    let service = VectorSceneService::default();
    for command in packet.world_2d_vectors() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_text2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> Text2dSceneService {
    let service = Text2dSceneService::default();
    for command in packet.world_2d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_text3d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_runtime_bundles::amigo_3d_text::Text3dSceneService {
    let service = amigo_runtime_bundles::amigo_3d_text::Text3dSceneService::default();
    for command in packet.world_3d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_mesh_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_runtime_bundles::amigo_3d_mesh::MeshSceneService {
    let service = amigo_runtime_bundles::amigo_3d_mesh::MeshSceneService::default();
    for command in packet.world_3d_meshes() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_material_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_runtime_bundles::amigo_3d_material::MaterialSceneService {
    let service = amigo_runtime_bundles::amigo_3d_material::MaterialSceneService::default();
    for command in packet.world_3d_materials() {
        service.queue(command.clone());
    }
    service
}
