use amigo_render_api::RenderFrameExtractor;
use amigo_scene::SceneService;

use super::context::{AppRenderExtractContext, AppRenderExtractorRegistry, AppRenderFramePacket};

pub(crate) fn default_app_render_extractor_registry<'a>() -> AppRenderExtractorRegistry<'a> {
    let mut registry = AppRenderExtractorRegistry::new();
    registry.register(ResolvedTileMap2dExtractor);
    registry.register(ResolvedSprite2dExtractor);
    registry.register(ResolvedText2dExtractor);
    registry.register(ResolvedVector2dExtractor);
    registry.register(ResolvedParticle2dExtractor);
    registry.register(ResolvedMesh3dExtractor);
    registry.register(ResolvedMaterial3dExtractor);
    registry.register(ResolvedText3dExtractor);
    registry.register(ResolvedUiOverlayExtractor);
    registry
}

pub(crate) struct ResolvedTileMap2dExtractor;

pub(crate) struct ResolvedSprite2dExtractor;

pub(crate) struct ResolvedVector2dExtractor;

pub(crate) struct ResolvedParticle2dExtractor;

pub(crate) struct ResolvedText2dExtractor;

pub(crate) struct ResolvedMesh3dExtractor;

pub(crate) struct ResolvedMaterial3dExtractor;

pub(crate) struct ResolvedText3dExtractor;

pub(crate) struct ResolvedUiOverlayExtractor;

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedTileMap2dExtractor {
    fn name(&self) -> &'static str {
        "resolved_tilemap_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.tilemap_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_tilemap(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedSprite2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_sprite_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.sprite_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_sprite(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedVector2dExtractor {
    fn name(&self) -> &'static str {
        "resolved_vector_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.vector_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_vector(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedText2dExtractor {
    fn name(&self) -> &'static str {
        "resolved_text_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.text2d_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_2d_text(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedMesh3dExtractor {
    fn name(&self) -> &'static str {
        "resolved_mesh_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.mesh_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_mesh(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedMaterial3dExtractor {
    fn name(&self) -> &'static str {
        "resolved_material_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.material_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_material(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedText3dExtractor {
    fn name(&self) -> &'static str {
        "resolved_text_3d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.text3d_scene_service.commands() {
            if is_entity_render_visible(context.scene_service, &command.entity_name) {
                packet.push_world_3d_text(command);
            }
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket>
    for ResolvedParticle2dExtractor
{
    fn name(&self) -> &'static str {
        "resolved_particle_2d"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        for command in context.particle2d_scene_service.draw_commands() {
            packet.push_world_2d_particle(command);
        }
    }
}

impl RenderFrameExtractor<AppRenderExtractContext<'_>, AppRenderFramePacket> for ResolvedUiOverlayExtractor {
    fn name(&self) -> &'static str {
        "resolved_ui_overlay"
    }

    fn extract(&self, context: &AppRenderExtractContext<'_>, packet: &mut AppRenderFramePacket) {
        let overlays = crate::ui_runtime::resolve_ui_overlay_documents(
            context.ui_scene_service,
            context.ui_state_service,
            context.ui_theme_service,
        )
        .into_iter()
        .map(|document| document.overlay);
        packet.extend_overlay(overlays);
    }
}

fn is_entity_render_visible(scene_service: &SceneService, entity_name: &str) -> bool {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.lifecycle.visible)
        .unwrap_or(true)
}
