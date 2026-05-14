use amigo_scene::{LayeredImage2dSceneCommand, SceneEntityId, SceneService};

use crate::{
    LayeredImageDrawCommand, LayeredImageInstance, LayeredImageLayerOverride,
    LayeredImageSceneService,
};

pub fn queue_layered_image_scene_command(
    scene_service: &SceneService,
    layered_image_scene_service: &LayeredImageSceneService,
    command: &LayeredImage2dSceneCommand,
) -> SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());

    layered_image_scene_service.queue(LayeredImageDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        render_layer: command.render_layer.clone(),
        image: LayeredImageInstance {
            asset: command.asset.clone(),
            size: command.size,
            base_opacity: command.base_opacity.clamp(0.0, 1.0),
            viewport_fit: command.viewport_fit.into(),
            layer_overrides: command
                .layer_overrides
                .iter()
                .map(|item| LayeredImageLayerOverride {
                    id: item.id.clone(),
                    opacity: item.opacity,
                    enabled: item.enabled,
                    blend_mode: item.blend_mode.map(Into::into),
                })
                .collect(),
        },
        z_index: command.z_index,
        transform: command.transform,
    });

    entity
}
