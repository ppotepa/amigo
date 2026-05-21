use amigo_assets::AssetKey;
use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, Material2dDocument,
    Material2dLightingSceneCommand, Material2dOpticalModeDocument,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    RenderContributions2dSceneCommand, SceneComponentDocument, SceneDocumentError,
    SceneDocumentResult, SceneTransform2Document, SceneTransform3Document, SceneVec2Document,
    Text2dAlignDocument, Text2dAlignSceneCommand, Text2dBlendModeDocument,
    Text2dBlendModeSceneCommand, Text2dGlowSceneCommand, Text2dOutlineSceneCommand,
    Text2dSceneCommand, Text2dShadowSceneCommand, Text2dStyleDocument,
    Text2dStyleSceneCommand,
};

use super::{Text2dDocument, parse_text_2d_plugin_payload};

pub struct Text2dComponentHydrator;

impl ComponentHydrator for Text2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.text-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, SceneComponentDocument::Text2d { .. })
            || matches!(
                component,
                SceneComponentDocument::Plugin { component_type, .. }
                    if component_type == "amigo.gfx.text-2d.Text2D"
                        || component_type == "Text2D"
            )
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            SceneComponentDocument::Text2d { .. } => {
                let Some(document) = Text2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.text-2d.Text2D"
                || component_type == "Text2D" =>
            {
                parse_text_2d_plugin_payload(payload)?
            }
            _ => return Ok(()),
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
                    color: parse_color_rgba_hex(
                        &glow.color,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?,
                    radius: glow.radius.max(0.0),
                    intensity: glow.intensity.max(0.0),
                    passes: glow.passes.max(1),
                })
            })
            .transpose()?,
    })
}

fn material2d_scene_command(material: Option<Material2dDocument>) -> Option<Material2dSceneCommand> {
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
