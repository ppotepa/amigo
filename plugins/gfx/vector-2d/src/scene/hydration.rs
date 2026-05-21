use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, Material2dDocument,
    Material2dLightingSceneCommand, Material2dOpticalModeDocument,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    RenderContributions2dSceneCommand, SceneComponentDocument, SceneDocumentError,
    SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVectorShapeKindComponentDocument, SceneVec2Document, VectorShape2dSceneCommand,
    VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand,
};

use super::{Vector2dDocument, parse_vector_2d_plugin_payload};

pub struct VectorShape2dComponentHydrator;

impl ComponentHydrator for VectorShape2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.vector-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, SceneComponentDocument::VectorShape2d { .. })
            || matches!(
                component,
                SceneComponentDocument::Plugin { component_type, .. }
                    if component_type == "amigo.gfx.vector-2d.VectorShape2D"
                        || component_type == "VectorShape2D"
            )
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            SceneComponentDocument::VectorShape2d { .. } => {
                let Some(document) = Vector2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.vector-2d.VectorShape2D"
                || component_type == "VectorShape2D" =>
            {
                parse_vector_2d_plugin_payload(payload)?
            }
            _ => return Ok(()),
        };

        let stroke_color = document
            .stroke_color
            .as_deref()
            .map(|value| parse_color_rgba_hex(value, &ctx.document.scene.id, &ctx.entity.id, "VectorShape2D"))
            .transpose()?
            .unwrap_or(ColorRgba::WHITE);
        let fill_color = document
            .fill_color
            .as_deref()
            .map(|value| parse_color_rgba_hex(value, &ctx.document.scene.id, &ctx.entity.id, "VectorShape2D"))
            .transpose()?;
        let kind = match &document.kind {
            SceneVectorShapeKindComponentDocument::Polyline => VectorShapeKind2dSceneCommand::Polyline {
                points: document.points.iter().copied().map(vec2_from_document).collect(),
                closed: document.closed,
            },
            SceneVectorShapeKindComponentDocument::Polygon => VectorShapeKind2dSceneCommand::Polygon {
                points: document.points.iter().copied().map(vec2_from_document).collect(),
            },
            SceneVectorShapeKindComponentDocument::Circle => VectorShapeKind2dSceneCommand::Circle {
                radius: document.radius.max(0.0),
                segments: document.segments.max(3),
            },
        };

        let mut command = VectorShape2dSceneCommand::new(
            ctx.source_mod.to_owned(),
            ctx.entity_name.to_owned(),
            kind,
            VectorStyle2dSceneCommand {
                stroke_color,
                stroke_width: document.stroke_width.max(0.0),
                fill_color,
            },
        );
        command.z_index = document.z_index;
        command.render_layer = document.render_layer.clone();
        command.render_contributions = RenderContributions2dSceneCommand {
            roles: document
                .render_contributions
                .clone()
                .with_defaults(vector_render_contribution_defaults())
                .into_roles(),
        };
        command.material = material2d_scene_command(document.material);
        command.transform = transform2_for_entity(ctx.entity);

        ctx.commands
            .push(amigo_scene::SceneCommand::plugin(crate::vector_plugin_scene_command(
                command,
            )));

        Ok(())
    }
}

fn vector_render_contribution_defaults() -> [(&'static str, bool); 6] {
    [
        ("world.color", true),
        ("material.mask", false),
        ("optics.refract", false),
        ("transmission.source", false),
        ("bloom.source", false),
        ("camera.fx_source", false),
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
