use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{
    DepthAuxMap2dChannelsDocument, DepthAuxMap2dChannelsSceneCommand, DepthAuxMap2dSceneCommand,
    DepthMap2dSceneCommand, DepthMapViewportFit2dSceneCommand, LayeredImageViewportFit2dDocument,
    PluginComponentHydrationContext, PluginComponentHydrator, SceneDocumentError,
    SceneDocumentResult, SceneTransform2Document, SceneTransform3Document, SceneVec2Document,
};

use crate::api::FocusDepthResponse2d;

use super::{DepthAuxMap2dDocument, DepthMap2dDocument, FocusDepthResponse2dDocument};

pub fn focus_depth_response_from_document(
    document: FocusDepthResponse2dDocument,
) -> FocusDepthResponse2d {
    FocusDepthResponse2d {
        enabled: document.enabled,
        strength: document.strength,
        focus_width_m: document.focus_width_m,
        max_blur_px: document.max_blur_px,
    }
    .normalized()
}

#[derive(Default)]
pub struct DepthMap2dPluginComponentHydrator;
#[derive(Default)]
pub struct DepthAuxMap2dPluginComponentHydrator;

impl PluginComponentHydrator for DepthMap2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthMap2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<DepthMap2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "DepthMap2D plugin hydrator received wrong payload".to_owned(),
            });
        };
        push_depth_map_plugin_command(
            ctx.source_mod,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
            document.clone(),
        );
        Ok(())
    }
}

impl PluginComponentHydrator for DepthAuxMap2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth"
    }

    fn component_type(&self) -> &'static str {
        "amigo.camera.focus-depth.DepthAuxMap2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<DepthAuxMap2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "DepthAuxMap2D plugin hydrator received wrong payload".to_owned(),
            });
        };
        push_depth_aux_map_plugin_command(
            ctx.source_mod,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
            document.clone(),
        );
        Ok(())
    }
}

fn push_depth_map_plugin_command(
    source_mod: &str,
    entity: &amigo_scene::SceneEntityDocument,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
    document: DepthMap2dDocument,
) {
    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::depth_map_2d_plugin_scene_command(DepthMap2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            id: document.id,
            asset: AssetKey::new(document.asset),
            size: vec2_from_document(document.size),
            viewport_fit: depth_map_viewport_fit_from_document(document.viewport_fit),
            white_is_near: document.white_is_near,
            z_index: document.z_index,
            transform: transform2_for_entity(entity),
        }),
    });
}

fn push_depth_aux_map_plugin_command(
    source_mod: &str,
    entity: &amigo_scene::SceneEntityDocument,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
    document: DepthAuxMap2dDocument,
) {
    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::depth_aux_map_2d_plugin_scene_command(DepthAuxMap2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            id: document.id,
            asset: AssetKey::new(document.asset),
            surface_asset: document.surface_asset.map(AssetKey::new),
            size: vec2_from_document(document.size),
            viewport_fit: depth_map_viewport_fit_from_document(document.viewport_fit),
            channels: depth_aux_channels_from_document(&document.channels),
            z_index: document.z_index,
            transform: transform2_for_entity(entity),
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

fn depth_map_viewport_fit_from_document(
    fit: LayeredImageViewportFit2dDocument,
) -> DepthMapViewportFit2dSceneCommand {
    match fit {
        LayeredImageViewportFit2dDocument::Fixed => DepthMapViewportFit2dSceneCommand::Fixed,
        LayeredImageViewportFit2dDocument::Stretch => DepthMapViewportFit2dSceneCommand::Stretch,
        LayeredImageViewportFit2dDocument::Contain => DepthMapViewportFit2dSceneCommand::Contain,
        LayeredImageViewportFit2dDocument::Cover => DepthMapViewportFit2dSceneCommand::Cover,
    }
}

fn depth_aux_channels_from_document(
    channels: &DepthAuxMap2dChannelsDocument,
) -> DepthAuxMap2dChannelsSceneCommand {
    DepthAuxMap2dChannelsSceneCommand {
        r: channels.r.clone(),
        g: channels.g.clone(),
        b: channels.b.clone(),
        a: channels.a.clone(),
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
    fn depth_map_hydrator_carries_asset_size_fit_and_depth_flags() {
        let hydrator = DepthMap2dPluginComponentHydrator;
        let payload = DepthMap2dDocument {
            id: "main-depth".to_owned(),
            asset: "scene/depth/main".to_owned(),
            size: SceneVec2Document {
                x: 1280.0,
                y: 720.0,
            },
            viewport_fit: LayeredImageViewportFit2dDocument::Cover,
            white_is_near: false,
            z_index: -100.0,
        };
        let entity = test_entity("depth");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "depth",
                component_index: 0,
                component_type: "amigo.camera.focus-depth.DepthMap2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("depth map hydrator should accept plugin payload");

        let command = plugin_payload::<DepthMap2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "depth");
        assert_eq!(command.id, "main-depth");
        assert_eq!(command.asset, AssetKey::new("scene/depth/main"));
        assert_eq!(command.size, Vec2::new(1280.0, 720.0));
        assert_eq!(
            command.viewport_fit,
            DepthMapViewportFit2dSceneCommand::Cover
        );
        assert!(!command.white_is_near);
        assert_eq!(command.z_index, -100.0);
    }

    #[test]
    fn depth_aux_hydrator_carries_surface_asset_and_channels() {
        let hydrator = DepthAuxMap2dPluginComponentHydrator;
        let payload = DepthAuxMap2dDocument {
            id: "main-depth-aux".to_owned(),
            asset: "scene/depth/aux".to_owned(),
            surface_asset: Some("scene/surface/aux".to_owned()),
            size: SceneVec2Document { x: 640.0, y: 360.0 },
            viewport_fit: LayeredImageViewportFit2dDocument::Contain,
            channels: DepthAuxMap2dChannelsDocument {
                r: "auxiliary_depth".to_owned(),
                g: "local_height".to_owned(),
                b: "occluder_strength".to_owned(),
                a: "valid_mask".to_owned(),
            },
            z_index: -99.5,
        };
        let entity = test_entity("depth-aux");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "depth-aux",
                component_index: 0,
                component_type: "amigo.camera.focus-depth.DepthAuxMap2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("depth aux hydrator should accept plugin payload");

        let command = plugin_payload::<DepthAuxMap2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "depth-aux");
        assert_eq!(command.id, "main-depth-aux");
        assert_eq!(command.asset, AssetKey::new("scene/depth/aux"));
        assert_eq!(
            command.surface_asset,
            Some(AssetKey::new("scene/surface/aux"))
        );
        assert_eq!(command.size, Vec2::new(640.0, 360.0));
        assert_eq!(
            command.viewport_fit,
            DepthMapViewportFit2dSceneCommand::Contain
        );
        assert_eq!(command.channels.r, "auxiliary_depth");
        assert_eq!(command.channels.g, "local_height");
        assert_eq!(command.channels.b, "occluder_strength");
        assert_eq!(command.channels.a, "valid_mask");
        assert_eq!(command.z_index, -99.5);
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
