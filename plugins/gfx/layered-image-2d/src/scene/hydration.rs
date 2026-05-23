use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, LayeredImage2dSceneCommand,
    LayeredImageBlendMode2dDocument, LayeredImageBlendMode2dSceneCommand,
    LayeredImageLayerOverrideDocument, LayeredImageLayerOverrideSceneCommand,
    LayeredImageViewportFit2dDocument, LayeredImageViewportFit2dSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator, SceneComponentDocument,
    SceneDocumentError, SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVec2Document, VisualMaps2dDocument, VisualMaps2dSceneCommand,
};

use super::LayeredImage2dDocument;

pub struct LayeredImage2dComponentHydrator;
pub struct LayeredImage2dPluginComponentHydrator;

impl ComponentHydrator for LayeredImage2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.layered-image-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, SceneComponentDocument::LayeredImage2d { .. })
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            SceneComponentDocument::LayeredImage2d { .. } => {
                let Some(document) = LayeredImage2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            _ => return Ok(()),
        };

        push_layered_image_command(
            &document,
            ctx.source_mod,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
        );

        Ok(())
    }
}

impl PluginComponentHydrator for LayeredImage2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.layered-image-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<LayeredImage2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "LayeredImage2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_layered_image_command(
            document,
            ctx.source_mod,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
        );

        Ok(())
    }
}

fn push_layered_image_command(
    document: &LayeredImage2dDocument,
    source_mod: &str,
    entity: &amigo_scene::SceneEntityDocument,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
) {
    commands.push(amigo_scene::SceneCommand::QueueLayeredImage2d {
            command: LayeredImage2dSceneCommand {
                source_mod: source_mod.to_owned(),
                entity_name: entity_name.to_owned(),
                render_layer: document.render_layer.clone(),
                asset: AssetKey::new(document.asset.clone()),
                size: vec2_from_document(document.size),
                base_opacity: document.base_opacity,
                viewport_fit: viewport_fit_from_document(document.viewport_fit),
                visual_maps: document.visual_maps.as_ref().map(visual_maps_from_document),
                z_index: document.z_index,
                transform: transform2_for_entity(entity),
                layer_overrides: document
                    .layer_overrides
                    .iter()
                    .map(layer_override_from_document)
                    .collect(),
            },
        });
}

fn transform2_for_entity(entity: &amigo_scene::SceneEntityDocument) -> Transform2 {
    entity
        .transform2
        .map(transform2_from_document)
        .or_else(|| entity.transform3.map(transform2_from_transform3_document))
        .unwrap_or_default()
}

fn transform2_from_document(document: SceneTransform2Document) -> Transform2 {
    Transform2 {
        translation: vec2_from_document(document.translation),
        rotation_radians: document.rotation_radians,
        scale: vec2_from_document(document.scale),
    }
}

fn transform2_from_transform3_document(document: SceneTransform3Document) -> Transform2 {
    Transform2 {
        translation: Vec2::new(document.translation.x, document.translation.y),
        rotation_radians: document.rotation_euler.z,
        scale: Vec2::new(document.scale.x, document.scale.y),
    }
}

fn vec2_from_document(value: SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

fn visual_maps_from_document(maps: &VisualMaps2dDocument) -> VisualMaps2dSceneCommand {
    VisualMaps2dSceneCommand {
        normal: maps.normal.clone().map(AssetKey::new),
        wetness: maps.wetness.clone().map(AssetKey::new),
        emissive: maps.emissive.clone().map(AssetKey::new),
        highlight: maps.highlight.clone().map(AssetKey::new),
        roughness: maps.roughness,
    }
}

fn viewport_fit_from_document(
    fit: LayeredImageViewportFit2dDocument,
) -> LayeredImageViewportFit2dSceneCommand {
    match fit {
        LayeredImageViewportFit2dDocument::Fixed => LayeredImageViewportFit2dSceneCommand::Fixed,
        LayeredImageViewportFit2dDocument::Stretch => {
            LayeredImageViewportFit2dSceneCommand::Stretch
        }
        LayeredImageViewportFit2dDocument::Contain => {
            LayeredImageViewportFit2dSceneCommand::Contain
        }
        LayeredImageViewportFit2dDocument::Cover => LayeredImageViewportFit2dSceneCommand::Cover,
    }
}

fn layer_override_from_document(
    value: &LayeredImageLayerOverrideDocument,
) -> LayeredImageLayerOverrideSceneCommand {
    LayeredImageLayerOverrideSceneCommand {
        id: value.id.clone(),
        opacity: value.opacity,
        enabled: value.enabled,
        blend_mode: value.blend.map(blend_mode_from_document),
        visual_maps: value.visual_maps.as_ref().map(visual_maps_from_document),
    }
}

fn blend_mode_from_document(
    value: LayeredImageBlendMode2dDocument,
) -> LayeredImageBlendMode2dSceneCommand {
    match value {
        LayeredImageBlendMode2dDocument::Alpha => LayeredImageBlendMode2dSceneCommand::Alpha,
        LayeredImageBlendMode2dDocument::Additive => {
            LayeredImageBlendMode2dSceneCommand::Additive
        }
        LayeredImageBlendMode2dDocument::Screen => LayeredImageBlendMode2dSceneCommand::Screen,
        LayeredImageBlendMode2dDocument::Multiply => {
            LayeredImageBlendMode2dSceneCommand::Multiply
        }
        LayeredImageBlendMode2dDocument::Lighten => LayeredImageBlendMode2dSceneCommand::Lighten,
    }
}
