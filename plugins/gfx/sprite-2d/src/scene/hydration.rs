use amigo_assets::AssetKey;
use amigo_camera::camera_optical_response_from_document;
use amigo_math::{Transform2, Vec2};
use amigo_scene::{
    ComponentHydrationContext, ComponentHydrator, Material2dDocument,
    Material2dLightingSceneCommand, Material2dOpticalModeDocument,
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    RenderContributions2dSceneCommand, SceneComponentDocument, SceneDocumentResult,
    SceneSpriteAnimationDocument, SceneSpriteSheetDocument, SceneTransform2Document,
    SceneTransform3Document, SceneVec2Document, Sprite2dSceneCommand,
    SpriteAnimation2dSceneOverride, SpriteSheet2dSceneCommand, VisualMaps2dDocument,
    VisualMaps2dSceneCommand,
};

use crate::api::{Sprite2dRenderResponse, Sprite2dRenderableCandidate};

use super::{Sprite2dDocument, parse_sprite_2d_plugin_payload};

pub fn sprite_candidate_from_document(document: &Sprite2dDocument) -> Sprite2dRenderableCandidate {
    Sprite2dRenderableCandidate::active(
        document.entity_name.clone(),
        document.render_layer.clone(),
        Sprite2dRenderResponse {
            visible: document.visible,
            opacity: document.opacity,
        },
    )
}

pub struct Sprite2dComponentHydrator;

impl ComponentHydrator for Sprite2dComponentHydrator {
    fn provider_id(&self) -> &'static str {
        "amigo.gfx.sprite-2d"
    }

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool {
        matches!(component, SceneComponentDocument::Sprite2d { .. })
            || matches!(
                component,
                SceneComponentDocument::Plugin { component_type, .. }
                    if component_type == "amigo.gfx.sprite-2d.Sprite2D"
                        || component_type == "Sprite2D"
            )
    }

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()> {
        let document = match ctx.component {
            SceneComponentDocument::Sprite2d { .. } => {
                let Some(document) = Sprite2dDocument::from_component(ctx.component) else {
                    return Ok(());
                };
                document
            }
            SceneComponentDocument::Plugin {
                component_type,
                payload,
            } if component_type == "amigo.gfx.sprite-2d.Sprite2D"
                || component_type == "Sprite2D" =>
            {
                parse_sprite_2d_plugin_payload(payload)?
            }
            _ => return Ok(()),
        };

        ctx.commands.push(amigo_scene::SceneCommand::plugin(
            crate::sprite_plugin_scene_command(Sprite2dSceneCommand {
                source_mod: ctx.source_mod.to_owned(),
                entity_name: ctx.entity_name.to_owned(),
                render_layer: document.render_layer.clone(),
                texture: AssetKey::new(document.texture.clone()),
                size: vec2_from_document(document.size),
                sheet: document.sheet.map(sprite_sheet_from_document),
                animation: document.animation.map(sprite_animation_from_document),
                visual_maps: document.visual_maps.as_ref().map(visual_maps_from_document),
                render_contributions: RenderContributions2dSceneCommand {
                    roles: document
                        .render_contributions
                        .clone()
                        .with_defaults(sprite_render_contribution_defaults())
                        .into_roles(),
                },
                material: material2d_scene_command(document.material.clone()),
                z_index: document.z_index,
                transform: transform2_for_entity(ctx.entity),
            }),
        ));

        Ok(())
    }
}

fn sprite_render_contribution_defaults() -> [(&'static str, bool); 6] {
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

fn sprite_sheet_from_document(value: SceneSpriteSheetDocument) -> SpriteSheet2dSceneCommand {
    SpriteSheet2dSceneCommand {
        columns: value.columns.max(1),
        rows: value.rows.max(1),
        frame_count: value.frame_count.max(1),
        frame_size: vec2_from_document(value.frame_size),
        fps: value.fps.max(0.0),
        looping: value.looping,
    }
}

fn sprite_animation_from_document(
    value: SceneSpriteAnimationDocument,
) -> SpriteAnimation2dSceneOverride {
    SpriteAnimation2dSceneOverride {
        fps: value.fps.map(|fps| fps.max(0.0)),
        looping: value.looping,
        start_frame: value.start_frame,
    }
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
