use amigo_assets::AssetKey;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, DepthAuxMap2dChannelsDocument,
    DepthAuxMap2dChannelsSceneCommand, DepthAuxMap2dSceneCommand, DepthMap2dSceneCommand,
    DepthMapViewportFit2dSceneCommand, LayeredImageViewportFit2dDocument, SceneComponentDocument,
    SceneDocumentResult, SceneTransform2Document, SceneTransform3Document, SceneVec2Document,
};

use crate::api::FocusDepthResponse2d;

use super::{
    parse_depth_aux_map_2d_plugin_payload, parse_depth_map_2d_plugin_payload,
    DepthAuxMap2dDocument, DepthMap2dDocument, FocusDepthResponse2dDocument,
};

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

pub struct DepthMap2dComponentHydrator;

impl ComponentHydrator for DepthMap2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.camera.focus-depth"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(
            component,
            SceneComponentDocument::DepthMap2d { .. }
                | SceneComponentDocument::DepthAuxMap2d { .. }
        ) || matches!(
            component,
            SceneComponentDocument::Plugin { component_type, .. }
                if component_type == "amigo.camera.focus-depth.DepthMap2D"
                    || component_type == "DepthMap2D"
                    || component_type == "amigo.camera.focus-depth.DepthAuxMap2D"
                    || component_type == "DepthAuxMap2D"
        )
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        match ctx.component {
            SceneComponentDocument::DepthMap2d { .. } => {
                let Some(document) = DepthMap2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                push_depth_map_command(ctx, document);
            }
            SceneComponentDocument::DepthAuxMap2d { .. } => {
                let Some(document) = DepthAuxMap2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                push_depth_aux_map_command(ctx, document);
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.camera.focus-depth.DepthMap2D"
                || component_type == "DepthMap2D" =>
            {
                push_depth_map_command(ctx, parse_depth_map_2d_plugin_payload(payload)?);
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.camera.focus-depth.DepthAuxMap2D"
                || component_type == "DepthAuxMap2D" =>
            {
                push_depth_aux_map_command(ctx, parse_depth_aux_map_2d_plugin_payload(payload)?);
            }
            _ => {}
        }

        Ok(())
    }
}

fn push_depth_map_command(ctx: ComponentHydrationContext<'_>, document: DepthMap2dDocument) {
    ctx.commands.push(amigo_scene::SceneCommand::QueueDepthMap2d {
        command: DepthMap2dSceneCommand {
            source_mod: ctx.source_mod.to_owned(),
            entity_name: ctx.entity_name.to_owned(),
            id: document.id,
            asset: AssetKey::new(document.asset),
            size: vec2_from_document(document.size),
            viewport_fit: depth_map_viewport_fit_from_document(document.viewport_fit),
            white_is_near: document.white_is_near,
            z_index: document.z_index,
            transform: transform2_for_entity(ctx.entity),
        },
    });
}

fn push_depth_aux_map_command(
    ctx: ComponentHydrationContext<'_>,
    document: DepthAuxMap2dDocument,
) {
    ctx.commands.push(amigo_scene::SceneCommand::QueueDepthAuxMap2d {
        command: DepthAuxMap2dSceneCommand {
            source_mod: ctx.source_mod.to_owned(),
            entity_name: ctx.entity_name.to_owned(),
            id: document.id,
            asset: AssetKey::new(document.asset),
            surface_asset: document.surface_asset.map(AssetKey::new),
            size: vec2_from_document(document.size),
            viewport_fit: depth_map_viewport_fit_from_document(document.viewport_fit),
            channels: depth_aux_channels_from_document(&document.channels),
            z_index: document.z_index,
            transform: transform2_for_entity(ctx.entity),
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
