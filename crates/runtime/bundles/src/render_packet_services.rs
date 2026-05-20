use amigo_2d_composition::{
    LightRoute2dSceneService, RenderLayer2dSceneService,
};
use amigo_focus_depth_plugin::DepthMap2dSceneService;
use amigo_layered_image_2d_plugin::LayeredImageSceneService;
use amigo_light_2d_plugin::{GlobalLight2dSceneService, LightMap2dSceneService};
pub use amigo_sprite_2d_plugin::{SpriteSceneService, SpriteSheet};
use amigo_text_2d_plugin::Text2dSceneService;
pub use amigo_tilemap_2d_plugin::TileMap2dSceneService;
use amigo_vector_2d_plugin::VectorSceneService;

use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_scene::Sprite2dSceneCommand;

pub fn infer_sprite_sheet_from_prepared_asset(
    prepared: &amigo_assets::PreparedAsset,
) -> Option<SpriteSheet> {
    amigo_sprite_2d_plugin::infer_sprite_sheet_from_prepared_asset(prepared)
}

pub fn infer_tile_ruleset_from_prepared_asset(
    prepared: &amigo_assets::PreparedAsset,
) -> Option<amigo_tilemap_2d_plugin::TileRuleSet2d> {
    amigo_tilemap_2d_plugin::infer_tile_ruleset_from_prepared_asset(prepared)
}

pub fn resolve_sprite_sheet_for_command(
    asset_catalog: &amigo_assets::AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    amigo_sprite_2d_plugin::resolve_sprite_sheet_for_command(asset_catalog, command)
}

pub fn build_sprite_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> SpriteSceneService {
    let service = SpriteSceneService::default();
    for command in packet.world_2d_sprites() {
        service.queue(command.clone());
    }
    service
}

pub fn build_layered_image_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LayeredImageSceneService {
    let service = LayeredImageSceneService::default();
    for command in packet.world_2d_layered_images() {
        service.queue(command.clone());
    }
    service
}

pub fn build_depth_map2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> DepthMap2dSceneService {
    let service = DepthMap2dSceneService::default();
    for command in packet.world_2d_depth_maps() {
        service.queue(command.clone());
    }
    for command in packet.world_2d_depth_aux_maps() {
        service.queue_aux(command.clone());
    }
    service
}

pub fn build_render_layer2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> RenderLayer2dSceneService {
    let service = RenderLayer2dSceneService::default();
    for command in packet.world_2d_render_layers() {
        service.queue(command.clone());
    }
    service
}

pub fn build_light_route2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LightRoute2dSceneService {
    let service = LightRoute2dSceneService::default();
    for command in packet.world_2d_light_routes() {
        service.queue(command.clone());
    }
    service
}

pub fn build_global_light2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> GlobalLight2dSceneService {
    let service = GlobalLight2dSceneService::default();
    for command in packet.world_2d_global_lights() {
        service.queue(command.clone());
    }
    service
}

pub fn build_lightmap2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> LightMap2dSceneService {
    let service = LightMap2dSceneService::default();
    for command in packet.world_2d_lightmaps() {
        service.queue(command.clone());
    }
    service
}

pub fn build_tilemap_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> TileMap2dSceneService {
    let service = TileMap2dSceneService::default();
    for command in packet.world_2d_tilemaps() {
        service.queue(command.clone());
    }
    service
}

pub fn build_vector_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> VectorSceneService {
    let service = VectorSceneService::default();
    for command in packet.world_2d_vectors() {
        service.queue(command.clone());
    }
    service
}

pub fn build_text2d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> Text2dSceneService {
    let service = Text2dSceneService::default();
    for command in packet.world_2d_text() {
        service.queue(command.clone());
    }
    service
}

pub fn build_text3d_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_3d_text::Text3dSceneService {
    let service = amigo_3d_text::Text3dSceneService::default();
    for command in packet.world_3d_text() {
        service.queue(command.clone());
    }
    service
}

pub fn build_mesh_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_3d_mesh::MeshSceneService {
    let service = amigo_3d_mesh::MeshSceneService::default();
    for command in packet.world_3d_meshes() {
        service.queue(command.clone());
    }
    service
}

pub fn build_material_scene_service_from_packet(
    packet: &WgpuRenderFramePacket,
) -> amigo_3d_material::MaterialSceneService {
    let service = amigo_3d_material::MaterialSceneService::default();
    for command in packet.world_3d_materials() {
        service.queue(command.clone());
    }
    service
}
