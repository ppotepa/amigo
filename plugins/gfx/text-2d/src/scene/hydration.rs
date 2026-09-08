use amigo_assets::AssetKey;
use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    Material2dDocument, Material2dLightingSceneCommand, Material2dOpticalModeDocument,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator, RenderContributions2dSceneCommand,
    SceneDocumentError, SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVec2Document, Text2dAlignDocument, Text2dAlignSceneCommand, Text2dBlendModeDocument,
    Text2dBlendModeSceneCommand, Text2dGlowSceneCommand, Text2dOutlineSceneCommand,
    Text2dSceneCommand, Text2dShadowSceneCommand, Text2dStyleDocument, Text2dStyleSceneCommand,
};

use super::Text2dDocument;

#[derive(Default)]
pub struct Text2dPluginComponentHydrator;

impl PluginComponentHydrator for Text2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.text-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.text-2d.Text2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<Text2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "Text2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        ctx.commands.push(amigo_scene::SceneCommand::plugin(
            crate::text_plugin_scene_command(Text2dSceneCommand {
                source_mod: ctx.source_mod.to_owned(),
                entity_name: ctx.entity_name.to_owned(),
                render_layer: document.render_layer.clone(),
                content: document.content.clone(),
                font: AssetKey::new(document.font.clone()),
                bounds: vec2_from_document(document.bounds),
                style: text2d_style_from_document(
                    &document.style,
                    &ctx.document.scene.id,
                    &ctx.entity.id,
                    "Text2D",
                )?,
                render_contributions: RenderContributions2dSceneCommand {
                    roles: document.render_contributions.clone().into_roles(),
                },
                post_fx_host_id: None,
                z_index: document.z_index,
                material: material2d_scene_command(document.material),
                transform: transform2_for_entity(ctx.entity),
            }),
        ));

        Ok(())
    }
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

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    let parse_channel = |slice: &str| -> SceneDocumentResult<u8> {
        u8::from_str_radix(slice, 16).map_err(|source| SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!("invalid color `{value}`: {source}"),
        })
    };

    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_channel(&hex[0..2])?,
            parse_channel(&hex[2..4])?,
            parse_channel(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_channel(&hex[0..2])?,
            parse_channel(&hex[2..4])?,
            parse_channel(&hex[4..6])?,
            parse_channel(&hex[6..8])?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!(
                    "expected albedo color `{value}` to use #RRGGBB or #RRGGBBAA syntax"
                ),
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

fn text2d_style_from_document(
    style: &Text2dStyleDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<Text2dStyleSceneCommand> {
    let base_color = style
        .color
        .as_deref()
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()?
        .unwrap_or_else(|| ColorRgba::new(1.0, 0.96, 0.82, 1.0));
    let opacity = style.opacity.unwrap_or(1.0).clamp(0.0, 1.0);

    Ok(Text2dStyleSceneCommand {
        color: ColorRgba::new(
            base_color.r,
            base_color.g,
            base_color.b,
            base_color.a * opacity,
        ),
        opacity,
        font_size: style
            .font_size
            .filter(|value| value.is_finite())
            .map(|value| value.max(1.0)),
        align: match style.align {
            Text2dAlignDocument::Left => Text2dAlignSceneCommand::Left,
            Text2dAlignDocument::Center => Text2dAlignSceneCommand::Center,
            Text2dAlignDocument::Right => Text2dAlignSceneCommand::Right,
        },
        blend: match style.blend {
            Text2dBlendModeDocument::Alpha => Text2dBlendModeSceneCommand::Alpha,
            Text2dBlendModeDocument::Additive => Text2dBlendModeSceneCommand::Additive,
            Text2dBlendModeDocument::Multiply => Text2dBlendModeSceneCommand::Multiply,
            Text2dBlendModeDocument::Screen => Text2dBlendModeSceneCommand::Screen,
        },
        shadow: style
            .shadow
            .as_ref()
            .map(|shadow| {
                Ok(Text2dShadowSceneCommand {
                    color: parse_color_rgba_hex(
                        &shadow.color,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?,
                    offset: vec2_from_document(shadow.offset),
                })
            })
            .transpose()?,
        outline: style
            .outline
            .as_ref()
            .map(|outline| {
                Ok(Text2dOutlineSceneCommand {
                    color: parse_color_rgba_hex(
                        &outline.color,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?,
                    width: outline.width.max(0.0),
                })
            })
            .transpose()?,
        glow: style
            .glow
            .as_ref()
            .map(|glow| {
                Ok(Text2dGlowSceneCommand {
                    color: parse_color_rgba_hex(&glow.color, scene_id, entity_id, component_kind)?,
                    radius: glow.radius.max(0.0),
                    intensity: glow.intensity.max(0.0),
                    passes: glow.passes.max(1),
                })
            })
            .transpose()?,
    })
}

fn material2d_scene_command(
    material: Option<Material2dDocument>,
) -> Option<Material2dSceneCommand> {
    material.map(|material| Material2dSceneCommand {
        optical: Material2dOpticalSceneCommand {
            mode: match material.optical.mode {
                Material2dOpticalModeDocument::Opaque => Material2dOpticalModeSceneCommand::Opaque,
                Material2dOpticalModeDocument::Transmissive => {
                    Material2dOpticalModeSceneCommand::Transmissive
                }
                Material2dOpticalModeDocument::Refractive => {
                    Material2dOpticalModeSceneCommand::Refractive
                }
                Material2dOpticalModeDocument::Emissive => {
                    Material2dOpticalModeSceneCommand::Emissive
                }
            },
            transmission: material.optical.transmission,
            refraction_px: material.optical.refraction_px,
            distortion: material.optical.distortion,
            dispersion: material.optical.dispersion,
            roughness: material.optical.roughness,
            edge_boost: material.optical.edge_boost,
        },
        lighting: Material2dLightingSceneCommand {
            receives_light: material.lighting.receives_light,
            response: material.lighting.response,
        },
        camera_response: camera_optical_response_from_document(material.camera_response),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_material_api::{Material2dLightingDocument, Material2dOpticalDocument};
    use amigo_scene::{
        PluginComponentHydrationContext, RenderContributionsDocument, SceneCommand, SceneDocument,
        SceneEntityDocument, SceneMetadataDocument, SceneVisual2dDocument,
    };
    use std::collections::BTreeMap;

    #[test]
    fn text_hydrator_carries_material_and_render_contributions() {
        let hydrator = Text2dPluginComponentHydrator;
        let mut render_contributions = RenderContributionsDocument::default();
        render_contributions.set("material.mask", true);
        render_contributions.set("optics.refract", true);
        let payload = Text2dDocument {
            entity_name: String::new(),
            render_layer: "ui.title".to_owned(),
            content: "ROTTEN CLUB".to_owned(),
            font: "rotten-club/fonts/game".to_owned(),
            bounds: SceneVec2Document {
                x: 1180.0,
                y: 240.0,
            },
            style: Text2dStyleDocument::default(),
            render_contributions,
            z_index: 12.0,
            material: Some(Material2dDocument {
                optical: Material2dOpticalDocument {
                    mode: Material2dOpticalModeDocument::Refractive,
                    transmission: 0.58,
                    refraction_px: 4.5,
                    ..Material2dOpticalDocument::default()
                },
                lighting: Material2dLightingDocument {
                    receives_light: true,
                    response: 0.35,
                },
                camera_response: Default::default(),
            }),
        };
        let entity = test_entity("title");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "rotten-club",
                document: &document,
                entity: &entity,
                entity_name: "title",
                component_index: 0,
                component_type: "amigo.gfx.text-2d.Text2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("text hydrator should accept plugin payload");

        let command = plugin_payload::<Text2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "rotten-club");
        assert_eq!(command.entity_name, "title");
        assert_eq!(command.render_layer, "ui.title");
        assert_eq!(command.content, "ROTTEN CLUB");
        assert_eq!(command.font, AssetKey::new("rotten-club/fonts/game"));
        assert_eq!(command.bounds, Vec2::new(1180.0, 240.0));
        assert_eq!(
            command.render_contributions.roles.get("material.mask"),
            Some(&true)
        );
        assert_eq!(
            command.render_contributions.roles.get("optics.refract"),
            Some(&true)
        );
        let material = command
            .material
            .as_ref()
            .expect("material should be preserved");
        assert_eq!(
            material.optical.mode,
            Material2dOpticalModeSceneCommand::Refractive
        );
        assert_eq!(material.optical.transmission, 0.58);
        assert_eq!(material.optical.refraction_px, 4.5);
        assert!(material.lighting.receives_light);
        assert_eq!(material.lighting.response, 0.35);
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
