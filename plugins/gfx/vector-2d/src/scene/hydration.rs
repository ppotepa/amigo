use amigo_camera::camera_optical_response_from_document;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_scene::{
    Material2dDocument, Material2dLightingSceneCommand, Material2dOpticalModeDocument,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    PluginComponentHydrationContext, PluginComponentHydrator, RenderContributions2dSceneCommand,
    SceneDocumentError, SceneDocumentResult, SceneTransform2Document, SceneTransform3Document,
    SceneVec2Document, SceneVectorShapeKindComponentDocument, VectorShape2dSceneCommand,
    VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand,
};

use super::Vector2dDocument;

#[derive(Default)]
pub struct Vector2dPluginComponentHydrator;

impl PluginComponentHydrator for Vector2dPluginComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.vector-2d"
    }

    fn component_type(&self) -> &'static str {
        "amigo.gfx.vector-2d.VectorShape2D"
    }

    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()> {
        let Some(document) = ctx.payload.as_any().downcast_ref::<Vector2dDocument>() else {
            return Err(SceneDocumentError::Hydration {
                scene_id: ctx.document.scene.id.clone(),
                entity_id: ctx.entity.id.clone(),
                component_kind: ctx.component_type.to_owned(),
                message: "VectorShape2D plugin hydrator received wrong payload".to_owned(),
            });
        };

        push_vector_shape_command(
            document,
            ctx.source_mod,
            ctx.document,
            ctx.entity,
            ctx.entity_name,
            ctx.component_type,
            ctx.commands,
        )
    }
}

fn push_vector_shape_command(
    document: &Vector2dDocument,
    source_mod: &str,
    scene_document: &amigo_scene::SceneDocument,
    entity: &amigo_scene::SceneEntityDocument,
    entity_name: &str,
    component_kind: &str,
    commands: &mut Vec<amigo_scene::SceneCommand>,
) -> SceneDocumentResult<()> {
    let stroke_color = document
        .stroke_color
        .as_deref()
        .map(|value| {
            parse_color_rgba_hex(value, &scene_document.scene.id, &entity.id, component_kind)
        })
        .transpose()?
        .unwrap_or(ColorRgba::WHITE);
    let fill_color = document
        .fill_color
        .as_deref()
        .map(|value| {
            parse_color_rgba_hex(value, &scene_document.scene.id, &entity.id, component_kind)
        })
        .transpose()?;
    let kind = match &document.kind {
        SceneVectorShapeKindComponentDocument::Polyline => {
            VectorShapeKind2dSceneCommand::Polyline {
                points: document
                    .points
                    .iter()
                    .copied()
                    .map(vec2_from_document)
                    .collect(),
                closed: document.closed,
            }
        }
        SceneVectorShapeKindComponentDocument::Polygon => VectorShapeKind2dSceneCommand::Polygon {
            points: document
                .points
                .iter()
                .copied()
                .map(vec2_from_document)
                .collect(),
        },
        SceneVectorShapeKindComponentDocument::Circle => VectorShapeKind2dSceneCommand::Circle {
            radius: document.radius.max(0.0),
            segments: document.segments.max(3),
        },
    };

    let mut command = VectorShape2dSceneCommand::new(
        source_mod.to_owned(),
        entity_name.to_owned(),
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
    command.material = material2d_scene_command(document.material.clone());
    command.transform = transform2_for_entity(entity);

    commands.push(amigo_scene::SceneCommand::plugin(
        crate::vector_plugin_scene_command(command),
    ));
    Ok(())
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
    fn vector_hydrator_carries_shape_material_and_contribution_defaults() {
        let hydrator = Vector2dPluginComponentHydrator;
        let mut render_contributions = RenderContributionsDocument::default();
        render_contributions.set("material.mask", true);
        render_contributions.set("optics.refract", true);
        let payload = Vector2dDocument {
            entity_name: String::new(),
            render_layer: "foreground.props".to_owned(),
            kind: SceneVectorShapeKindComponentDocument::Circle,
            points: Vec::new(),
            closed: false,
            radius: 48.0,
            segments: 24,
            stroke_color: Some("#102030FF".to_owned()),
            stroke_width: 3.0,
            fill_color: Some("#FFFFFFFF".to_owned()),
            render_contributions,
            material: Some(Material2dDocument {
                optical: Material2dOpticalDocument {
                    mode: Material2dOpticalModeDocument::Refractive,
                    transmission: 0.35,
                    refraction_px: 5.0,
                    ..Material2dOpticalDocument::default()
                },
                lighting: Material2dLightingDocument {
                    receives_light: true,
                    response: 0.2,
                },
                camera_response: Default::default(),
            }),
            z_index: 8.0,
        };
        let entity = test_entity("vector-glass");
        let document = test_document(entity.clone());
        let mut commands = Vec::new();

        hydrator
            .hydrate_plugin_payload(PluginComponentHydrationContext {
                source_mod: "test-mod",
                document: &document,
                entity: &entity,
                entity_name: "vector-glass",
                component_index: 0,
                component_type: "amigo.gfx.vector-2d.VectorShape2D",
                payload: &payload,
                commands: &mut commands,
            })
            .expect("vector hydrator should accept plugin payload");

        let command = plugin_payload::<VectorShape2dSceneCommand>(&commands);
        assert_eq!(command.source_mod, "test-mod");
        assert_eq!(command.entity_name, "vector-glass");
        assert_eq!(command.render_layer, "foreground.props");
        assert_eq!(
            command.kind,
            VectorShapeKind2dSceneCommand::Circle {
                radius: 48.0,
                segments: 24
            }
        );
        assert_eq!(command.style.stroke_width, 3.0);
        assert_eq!(
            command.style.stroke_color,
            ColorRgba::new(16.0 / 255.0, 32.0 / 255.0, 48.0 / 255.0, 1.0)
        );
        assert_eq!(
            command.render_contributions.roles.get("world.color"),
            Some(&true)
        );
        assert_eq!(
            command.render_contributions.roles.get("material.mask"),
            Some(&true)
        );
        assert_eq!(
            command.render_contributions.roles.get("optics.refract"),
            Some(&true)
        );
        assert_eq!(
            command
                .render_contributions
                .roles
                .get("transmission.source"),
            Some(&false)
        );
        let material = command
            .material
            .as_ref()
            .expect("material should be preserved");
        assert_eq!(
            material.optical.mode,
            Material2dOpticalModeSceneCommand::Refractive
        );
        assert_eq!(material.optical.transmission, 0.35);
        assert_eq!(material.optical.refraction_px, 5.0);
        assert!(material.lighting.receives_light);
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
