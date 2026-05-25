use amigo_2d_spatial::{resolve_depth_source, DepthSource2d};
use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    BeaconLight2dSceneCommand, ComponentHydrationContext, ComponentHydrator,
    LayeredImageViewportFit2dDocument, LayeredImageViewportFit2dSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator,
    RenderContributions2dSceneCommand, RenderDepth2dDocument, RenderDepth2dSceneCommand,
    RenderDepthMode2dDocument, RenderDepthMode2dSceneCommand, SceneComponentDocument,
    SceneDocumentError, SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVec2Document,
};
use amigo_scene::SceneComponentDocument as ComponentDocument;

use super::BeaconLight2dDocument;

pub struct BeaconLight2dComponentHydrator;
pub struct BeaconLight2dPluginComponentHydrator;

impl ComponentHydrator for BeaconLight2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, ComponentDocument::BeaconLight2d { .. })
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            ComponentDocument::BeaconLight2d { .. } => {
                let Some(document) = BeaconLight2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            _ => return Ok(()),
        };

        push_beacon_light_command(
            ctx.source_mod,
            ctx.document,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
            &document,
        )
    }
}

impl PluginComponentHydrator for BeaconLight2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.lighting.beacon-light-2d.BeaconLight2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<BeaconLight2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "BeaconLight2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_beacon_light_command(
            ctx.source_mod,
            ctx.document,
            ctx.entity,
            ctx.entity_name,
            ctx.commands,
            document,
        )
    }
}

fn push_beacon_light_command(
    source_mod: &str,
    document_root: &amigo_scene::SceneDocument,
    entity: &amigo_scene::SceneEntityDocument,
    entity_name: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
    document: &BeaconLight2dDocument,
) -> SceneDocumentResult<()> {
    let depth_space = document_root.visual2d.spatial.depth_space.to_runtime();
    let resolved_depth = document
        .depth
        .as_ref()
        .map(|depth| render_depth_from_document(depth, depth_space));
    let z_depth = resolved_depth
        .as_ref()
        .map(|depth| depth.z_depth)
        .or_else(|| document.z_depth.map(|value| value.clamp(0.0, 1.0)));

    commands.push(amigo_scene::SceneCommand::Plugin {
        command: amigo_scene::beacon_light_2d_plugin_scene_command(BeaconLight2dSceneCommand {
            source_mod: source_mod.to_owned(),
            entity_name: entity_name.to_owned(),
            id: document.id.clone(),
            render_layer: document.render_layer.clone(),
            color: parse_color_rgba_hex(
                &document.color,
                &document_root.scene.id,
                &entity.id,
                "BeaconLight2D",
            )?,
            base_intensity: document.base_intensity.max(0.0),
            frequency_hz: document.frequency_hz.max(0.0),
            duty_cycle: document.duty_cycle.clamp(0.01, 0.99),
            rise_seconds: document.rise_seconds.max(0.0),
            fall_seconds: document.fall_seconds.max(0.0),
            phase_offset: document.phase_offset,
            sync_group: document.sync_group.clone(),
            jitter_amount: document.jitter_amount.max(0.0),
            jitter_hz: document.jitter_hz.max(0.0),
            core_radius_px: document.core_radius_px.max(0.25),
            halo_radius_px: document.halo_radius_px.max(0.25),
            glow_strength: document.glow_strength.max(0.0),
            beam_enabled: document.beam_enabled,
            beam_length_px: document.beam_length_px.max(0.0),
            beam_width_degrees: document.beam_width_degrees.clamp(1.0, 179.0),
            beam_strength: document.beam_strength.max(0.0),
            aberration_px: document.aberration_px.max(0.0),
            bloom: document.bloom.max(0.0),
            camera_response: camera_optical_response_from_document(document.camera_response),
            depth: resolved_depth,
            z_depth,
            z_index: document.z_index,
            render_contributions: RenderContributions2dSceneCommand {
                roles: document
                    .render_contributions
                    .clone()
                    .with_defaults(beacon_render_contribution_defaults())
                    .into_roles(),
            },
            enabled: document.enabled,
            transform: transform2_for_entity(entity),
            viewport_fit: viewport_fit_from_document(document.viewport_fit),
            viewport_canvas_size: document.viewport_canvas_size.map(vec2_from_document),
        }),
    });

    Ok(())
}

fn beacon_render_contribution_defaults() -> [(&'static str, bool); 4] {
    [
        ("overlay.visible", true),
        ("relight.plate", true),
        ("bloom.source", true),
        ("camera.fx_source", true),
    ]
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

fn render_depth_from_document(
    depth: &RenderDepth2dDocument,
    depth_space: amigo_2d_spatial::DepthSpace2d,
) -> RenderDepth2dSceneCommand {
    let source = match depth.mode {
        RenderDepthMode2dDocument::DepthMap => DepthSource2d::DepthMap,
        RenderDepthMode2dDocument::Distance => DepthSource2d::Distance {
            meters: depth.distance_m.unwrap_or(depth_space.near_m),
        },
        RenderDepthMode2dDocument::ZDepth => DepthSource2d::ZDepth {
            value: depth.z_depth,
        },
        RenderDepthMode2dDocument::Infinity => DepthSource2d::Infinity,
        RenderDepthMode2dDocument::Overlay => DepthSource2d::Overlay,
    };
    let resolved = resolve_depth_source(source, depth_space);

    RenderDepth2dSceneCommand {
        mode: match depth.mode {
            RenderDepthMode2dDocument::DepthMap => RenderDepthMode2dSceneCommand::DepthMap,
            RenderDepthMode2dDocument::Distance => RenderDepthMode2dSceneCommand::Distance,
            RenderDepthMode2dDocument::ZDepth => RenderDepthMode2dSceneCommand::ZDepth,
            RenderDepthMode2dDocument::Infinity => RenderDepthMode2dSceneCommand::Infinity,
            RenderDepthMode2dDocument::Overlay => RenderDepthMode2dSceneCommand::Overlay,
        },
        distance_m: resolved.distance_m,
        z_depth: resolved.z_depth.clamp(0.0, 1.0),
        blur_scale: depth.blur_scale.clamp(0.0, 4.0),
    }
}

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!("expected color `{value}` to use #RRGGBB or #RRGGBBAA syntax"),
            });
        }
    };

    Ok(ColorRgba::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ))
}

fn parse_hex_channel(
    channel: &str,
    original: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|source| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("invalid color `{original}`: {source}"),
    })
}
