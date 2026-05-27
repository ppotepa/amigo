use amigo_2d_spatial::{resolve_depth_source, DepthSource2d};
use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    BeaconLight2dSceneCommand, LayeredImageViewportFit2dDocument,
    LayeredImageViewportFit2dSceneCommand, PluginComponentHydrationContext,
    PluginComponentHydrator, RenderContributions2dSceneCommand, RenderDepth2dDocument,
    RenderDepth2dSceneCommand, RenderDepthMode2dDocument, RenderDepthMode2dSceneCommand,
    SceneDocumentError, SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVec2Document,
};

use super::BeaconLight2dDocument;

#[derive(Default)]
pub struct BeaconLight2dPluginComponentHydrator;

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

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_scene::{
        PluginComponentHydrationContext, RenderContributionsDocument, SceneCommand, SceneDocument,
        SceneEntityDocument, SceneMetadataDocument, SceneTransform2Document, SceneVec2Document,
        SceneVisual2dDocument,
    };
    use std::collections::BTreeMap;

    #[test]
    fn beacon_hydrator_carries_render_contributions_and_transform() {
        let hydrator = BeaconLight2dPluginComponentHydrator;
        let mut render_contributions = RenderContributionsDocument::default();
        render_contributions.set("overlay.visible", false);
        render_contributions.set("relight.plate", true);
        render_contributions.set("bloom.source", false);
        let payload = BeaconLight2dDocument {
            id: "alarm".to_owned(),
            render_layer: "fx.beacons".to_owned(),
            color: "#8040FFFF".to_owned(),
            base_intensity: 2.5,
            render_contributions,
            viewport_canvas_size: Some(SceneVec2Document {
                x: 1280.0,
                y: 720.0,
            }),
            ..BeaconLight2dDocument {
                id: "unused".to_owned(),
                render_layer: "default".to_owned(),
                color: "#FFFFFFFF".to_owned(),
                base_intensity: 1.0,
                frequency_hz: 1.0,
                duty_cycle: 0.2,
                rise_seconds: 0.1,
                fall_seconds: 0.2,
                phase_offset: 0.0,
                sync_group: None,
                jitter_amount: 0.06,
                jitter_hz: 9.0,
                core_radius_px: 2.0,
                halo_radius_px: 9.0,
                glow_strength: 1.0,
                beam_enabled: true,
                beam_length_px: 0.0,
                beam_width_degrees: 20.0,
                beam_strength: 0.0,
                aberration_px: 0.8,
                bloom: 1.0,
                camera_response: Default::default(),
                depth: None,
                z_depth: None,
                z_index: 0.0,
                render_contributions: RenderContributionsDocument::default(),
                enabled: true,
                viewport_fit: LayeredImageViewportFit2dDocument::default(),
                viewport_canvas_size: None,
            }
        };
        let mut entity = test_entity("beacon");
        entity.transform2 = Some(SceneTransform2Document {
            translation: SceneVec2Document { x: 12.0, y: -4.0 },
            rotation_radians: 0.5,
            scale: SceneVec2Document { x: 2.0, y: 3.0 },
        });
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "beacon",
                component_index: 0,
                component_type: "amigo.lighting.beacon-light-2d.BeaconLight2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("beacon hydrator should accept plugin payload");

        let command = plugin_payload::<BeaconLight2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "beacon");
        assert_eq!(command.id, "alarm");
        assert_eq!(command.render_layer, "fx.beacons");
        assert_eq!(
            command.color,
            ColorRgba::new(128.0 / 255.0, 64.0 / 255.0, 1.0, 1.0)
        );
        assert_eq!(command.base_intensity, 2.5);
        assert_eq!(
            command.render_contributions.roles.get("overlay.visible"),
            Some(&false)
        );
        assert_eq!(
            command.render_contributions.roles.get("relight.plate"),
            Some(&true)
        );
        assert_eq!(
            command.render_contributions.roles.get("bloom.source"),
            Some(&false)
        );
        assert_eq!(
            command.render_contributions.roles.get("camera.fx_source"),
            Some(&true)
        );
        assert_eq!(command.transform.translation, Vec2::new(12.0, -4.0));
        assert_eq!(command.transform.rotation_radians, 0.5);
        assert_eq!(command.transform.scale, Vec2::new(2.0, 3.0));
        assert_eq!(command.viewport_canvas_size, Some(Vec2::new(1280.0, 720.0)));
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
