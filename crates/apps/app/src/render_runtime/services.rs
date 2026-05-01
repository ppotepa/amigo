use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::TileMap2dSceneService;
use amigo_2d_vector::VectorSceneService;

use super::context::AppRenderFramePacket;

pub(crate) fn build_sprite_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> SpriteSceneService {
    let service = SpriteSceneService::default();
    for command in packet.world_2d_sprites() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_tilemap_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> TileMap2dSceneService {
    let service = TileMap2dSceneService::default();
    for command in packet.world_2d_tilemaps() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_vector_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> VectorSceneService {
    let service = VectorSceneService::default();
    for command in packet.world_2d_vectors() {
        service.queue(command.clone());
    }
    service
}

pub(crate) fn build_text2d_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> Text2dSceneService {
    let service = Text2dSceneService::default();
    for command in packet.world_2d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_text3d_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> amigo_3d_text::Text3dSceneService {
    let service = amigo_3d_text::Text3dSceneService::default();
    for command in packet.world_3d_text() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_mesh_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> amigo_3d_mesh::MeshSceneService {
    let service = amigo_3d_mesh::MeshSceneService::default();
    for command in packet.world_3d_meshes() {
        service.queue(command.clone());
    }
    service
}

#[cfg(test)]
pub(crate) fn build_material_scene_service_from_packet(
    packet: &AppRenderFramePacket,
) -> amigo_3d_material::MaterialSceneService {
    let service = amigo_3d_material::MaterialSceneService::default();
    for command in packet.world_3d_materials() {
        service.queue(command.clone());
    }
    service
}
