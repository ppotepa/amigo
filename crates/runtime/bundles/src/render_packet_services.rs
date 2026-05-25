use amigo_2d_composition::{LightRoute2dSceneService, RenderLayer2dSceneService};
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_scene::Sprite2dSceneCommand;
use amigo_sprite_2d_plugin::{SpriteSceneService, SpriteSheet};
use amigo_tilemap_2d_plugin::TileMap2dSceneService;

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

pub fn prepare_loaded_asset_domain_metadata(
    asset_catalog: &amigo_assets::AssetCatalog,
    sprite_scene_service: &SpriteSceneService,
    tilemap_scene_service: &TileMap2dSceneService,
    asset_key: &amigo_assets::AssetKey,
) {
    let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
        return;
    };
    if let Some(sheet) = infer_sprite_sheet_from_prepared_asset(&prepared) {
        sprite_scene_service.sync_sheet_for_texture(asset_key, sheet);
    }
    if let Some(ruleset) = infer_tile_ruleset_from_prepared_asset(&prepared) {
        tilemap_scene_service.sync_ruleset_for_asset(asset_key, &ruleset);
    }
}

pub fn resolve_sprite_sheet_for_command(
    asset_catalog: &amigo_assets::AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    amigo_sprite_2d_plugin::resolve_sprite_sheet_for_command(asset_catalog, command)
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
