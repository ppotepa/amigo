use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{
    LayeredImage2dSceneCommand, LayeredImageBlendMode2dDocument,
    LayeredImageBlendMode2dSceneCommand, LayeredImageLayerOverrideDocument,
    LayeredImageLayerOverrideSceneCommand, LayeredImageViewportFit2dDocument,
    LayeredImageViewportFit2dSceneCommand, PluginComponentHydrationContext,
    PluginComponentHydrator, SceneDocumentError, SceneDocumentResult, SceneTransform2Document,
    SceneTransform3Document, SceneVec2Document, VisualMaps2dDocument, VisualMaps2dSceneCommand,
};

use super::LayeredImage2dDocument;

#[derive(Default)]
pub struct LayeredImage2dPluginComponentHydrator;

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
        let Some(document) = ctx
            .payload
            .as_any()
            .downcast_ref::<LayeredImage2dDocument>()
        else {
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
    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::layered_image_2d_plugin_scene_command(LayeredImage2dSceneCommand {
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
        }),
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
        LayeredImageBlendMode2dDocument::Additive => LayeredImageBlendMode2dSceneCommand::Additive,
        LayeredImageBlendMode2dDocument::Screen => LayeredImageBlendMode2dSceneCommand::Screen,
        LayeredImageBlendMode2dDocument::Multiply => LayeredImageBlendMode2dSceneCommand::Multiply,
        LayeredImageBlendMode2dDocument::Lighten => LayeredImageBlendMode2dSceneCommand::Lighten,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_scene::{
        PluginComponentHydrationContext, SceneCommand, SceneDocument, SceneEntityDocument,
        SceneMetadataDocument, SceneVisual2dDocument,
    };
    use std::collections::BTreeMap;

    #[test]
    fn layered_image_hydrator_carries_asset_opacity_and_overrides() {
        let hydrator = LayeredImage2dPluginComponentHydrator;
        let payload = LayeredImage2dDocument {
            entity_name: String::new(),
            layers: Vec::new(),
            render_layer: "background".to_owned(),
            asset: "test/layered/alley".to_owned(),
            size: SceneVec2Document {
                x: 1280.0,
                y: 720.0,
            },
            base_opacity: 0.25,
            viewport_fit: LayeredImageViewportFit2dDocument::Cover,
            visual_maps: Some(VisualMaps2dDocument {
                normal: Some("test/layered/alley/normal".to_owned()),
                wetness: None,
                emissive: Some("test/layered/alley/emissive".to_owned()),
                highlight: None,
                roughness: Some(0.65),
            }),
            z_index: -100.0,
            layer_overrides: vec![LayeredImageLayerOverrideDocument {
                id: "accent_light".to_owned(),
                opacity: Some(0.5),
                enabled: Some(true),
                blend: Some(LayeredImageBlendMode2dDocument::Screen),
                visual_maps: None,
                post_fx: Vec::new(),
            }],
        };
        let entity = test_entity("bg");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "main-menu-background",
                component_index: 0,
                component_type: "amigo.gfx.layered-image-2d.LayeredImage2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("layered image hydrator should accept plugin payload");

        let command = plugin_payload::<LayeredImage2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "main-menu-background");
        assert_eq!(command.render_layer, "background");
        assert_eq!(command.asset, AssetKey::new("test/layered/alley"));
        assert_eq!(command.size, Vec2::new(1280.0, 720.0));
        assert_eq!(command.base_opacity, 0.25);
        assert_eq!(
            command.viewport_fit,
            LayeredImageViewportFit2dSceneCommand::Cover
        );
        assert_eq!(command.z_index, -100.0);
        let maps = command
            .visual_maps
            .as_ref()
            .expect("visual maps should be preserved");
        assert_eq!(
            maps.normal,
            Some(AssetKey::new("test/layered/alley/normal"))
        );
        assert_eq!(
            maps.emissive,
            Some(AssetKey::new("test/layered/alley/emissive"))
        );
        assert_eq!(maps.roughness, Some(0.65));
        assert_eq!(command.layer_overrides.len(), 1);
        assert_eq!(command.layer_overrides[0].id, "accent_light");
        assert_eq!(command.layer_overrides[0].opacity, Some(0.5));
        assert_eq!(
            command.layer_overrides[0].blend_mode,
            Some(LayeredImageBlendMode2dSceneCommand::Screen)
        );
    }

    fn plugin_payload<T: 'static>(commands: &[SceneCommand]) -> &T {
        assert_eq!(commands.len(), 1);
        match &commands[0] {
            SceneCommand::Plugin { command } => command
                .payload_as::<T>()
                .expect("plugin scene payload should downcast"),
            other => panic!("expected plugin scene command, got {other:?}"),
        }
    }

    fn test_entity(name: &str) -> SceneEntityDocument {
        SceneEntityDocument {
            id: name.to_owned(),
            name: name.to_owned(),
            tags: Vec::new(),
            groups: Vec::new(),
            visible: true,
            simulation_enabled: true,
            collision_enabled: true,
            properties: BTreeMap::new(),
            transform2: None,
            transform3: None,
            post_fx: Vec::new(),
            prefab: None,
            prefab_overrides: Vec::new(),
            components: Vec::new(),
        }
    }

    fn test_document(entity: SceneEntityDocument) -> SceneDocument {
        SceneDocument {
            panels: Vec::new(),
            version: 1,
            scene: SceneMetadataDocument {
                id: "test-scene".to_owned(),
                label: String::new(),
                description: None,
            },
            transitions: Vec::new(),
            collision_events: Vec::new(),
            audio_cues: Vec::new(),
            activation_sets: Vec::new(),
            visual2d: SceneVisual2dDocument::default(),
            state: BTreeMap::new(),
            entities: vec![entity],
        }
    }
}
