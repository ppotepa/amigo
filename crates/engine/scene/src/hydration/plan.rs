use super::style::{parse_color_rgba_hex, parse_optional_color_rgba_hex, ui_theme_from_component};
use super::*;
use amigo_assets::AssetKey;
use amigo_math::{ColorRgba, Curve1d};

use crate::{
    AabbCollider2dSceneCommand, ActivationEntrySceneCommand, ActivationSetSceneCommand,
    AudioCueSceneCommand, BehaviorConditionSceneCommand, BehaviorSceneCommand,
    Bounds2dSceneCommand, CameraFollow2dSceneCommand, CircleCollider2dSceneCommand,
    CollisionEventRule2dSceneCommand, EntityPoolSceneCommand, EventPipelineSceneCommand,
    FreeflightMotion2dSceneCommand, GlobalLight2dSceneCommand, InputActionMapSceneCommand,
    KinematicBody2dSceneCommand, LayeredImage2dSceneCommand, LayeredImageBlendMode2dDocument,
    LayeredImageBlendMode2dSceneCommand, LayeredImageLayerOverrideSceneCommand,
    LayeredImageViewportFit2dDocument, LayeredImageViewportFit2dSceneCommand, LifetimeSceneCommand,
    LightGroup2dSceneCommand, LightGroup2dSourceDocument, LightGroup2dSourceKindSceneCommand,
    LightGroup2dSourceSceneCommand, LightMap2dChannelDocument, LightMap2dChannelSceneCommand,
    LightMap2dSourceKindSceneCommand, LightMap2dSourceRefDocument, LightMap2dSourceRefSceneCommand,
    LightMap2dSourceSceneCommand, LightReceiver2dBindingSceneCommand,
    LightReceiver2dBindingSceneDocument, LightReceiverDarkPolicy2dSceneCommand,
    LightReceiverDarkPolicy2dSceneDocument, LightReceiverGlobalLight2dSceneCommand,
    LightReceiverGlobalLight2dSceneDocument, LightRoute2dSceneCommand,
    LightSampleStrategy2dSceneCommand, LightSampleStrategy2dSceneDocument,
    Material2dLightingModeSceneCommand, Material2dLightingModeSceneDocument,
    Material3dSceneCommand, Mesh3dSceneCommand, MotionController2dSceneCommand,
    Parallax2dSceneCommand, ParticleEmitter2dSceneCommand, ParticleMotionStretch2dSceneCommand,
    ParticleShapeChoice2dSceneCommand, ParticleShapeKeyframe2dSceneCommand,
    ProjectileEmitter2dSceneCommand, RenderLayer2dSceneCommand, SceneCommand,
    SceneComponentDocument, SceneDocument, SceneDocumentResult, SceneEntityLifecycleOverride,
    SceneVectorShapeKindComponentDocument, ScriptComponentSceneCommand, Sprite2dSceneCommand,
    StaticCollider2dSceneCommand, Text2dSceneCommand, Text3dSceneCommand, TileMap2dSceneCommand,
    TileMapMarker2dSceneCommand, Trigger2dSceneCommand, UiModelBindingsSceneCommand,
    UiSceneCommand, UiThemeSetSceneCommand, VectorShape2dSceneCommand,
    VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand, Velocity2dSceneCommand,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneHydrationPlan {
    pub commands: Vec<SceneCommand>,
}

pub fn build_scene_hydration_plan(
    source_mod: &str,
    document: &SceneDocument,
) -> SceneDocumentResult<SceneHydrationPlan> {
    let mut commands = Vec::new();

    hydrate_visual2d(source_mod, document, &mut commands)?;

    for entity in &document.entities {
        let entity_name = entity.display_name();
        commands.push(SceneCommand::SpawnNamedEntity {
            name: entity_name.clone(),
            transform: Some(transform3_for_entity(entity)),
        });
        commands.push(SceneCommand::ConfigureEntity {
            entity_name: entity_name.clone(),
            lifecycle: lifecycle_for_entity(entity),
            tags: entity.tags.clone(),
            groups: entity.groups.clone(),
            properties: entity
                .properties
                .iter()
                .map(|(key, value)| (key.clone(), property_value_from_document(value)))
                .collect(),
        });

        for component in &entity.components {
            if hydrate_component_core(
                source_mod,
                document,
                entity,
                &entity_name,
                component,
                &mut commands,
            )? {
                continue;
            }
            if hydrate_component_domains(
                source_mod,
                document,
                entity,
                &entity_name,
                component,
                &mut commands,
            )? {
                continue;
            }
        }
    }

    for rule in &document.collision_events {
        commands.push(SceneCommand::QueueCollisionEventRule2d {
            command: CollisionEventRule2dSceneCommand::new(
                source_mod.to_owned(),
                rule.id.clone(),
                entity_selector_from_document(&rule.source),
                entity_selector_from_document(&rule.target),
                rule.event.clone(),
                rule.once_per_overlap,
            ),
        });
    }

    for cue in &document.audio_cues {
        commands.push(SceneCommand::QueueAudioCue {
            command: AudioCueSceneCommand {
                source_mod: source_mod.to_owned(),
                name: cue.name.clone(),
                clip: AssetKey::new(resolve_scene_audio_clip(source_mod, &cue.clip)),
                min_interval: cue
                    .min_interval
                    .filter(|value| value.is_finite())
                    .map(|value| value.max(0.0)),
            },
        });
    }

    for set in &document.activation_sets {
        commands.push(SceneCommand::QueueActivationSet {
            command: ActivationSetSceneCommand {
                source_mod: source_mod.to_owned(),
                id: set.id.clone(),
                entries: set
                    .entries
                    .iter()
                    .map(|entry| ActivationEntrySceneCommand {
                        target: entity_selector_from_document(&entry.target),
                        lifecycle: SceneEntityLifecycleOverride {
                            visible: entry.visible,
                            simulation_enabled: entry.simulation_enabled,
                            collision_enabled: entry.collision_enabled,
                        },
                        transform: entry
                            .transform3
                            .map(transform3_from_document)
                            .or_else(|| entry.transform2.map(transform3_from_transform2_document)),
                        velocity: entry.velocity.map(vec2_from_document),
                        angular_velocity: entry.angular_velocity,
                        properties: entry
                            .properties
                            .iter()
                            .map(|(key, value)| (key.clone(), property_value_from_document(value)))
                            .collect(),
                    })
                    .collect(),
            },
        });
    }

    Ok(SceneHydrationPlan { commands })
}

include!("plan/components_core.rs");
include!("plan/components_domains.rs");

fn hydrate_visual2d(
    source_mod: &str,
    document: &SceneDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<()> {
    for layer in &document.visual2d.render_layers {
        commands.push(SceneCommand::QueueRenderLayer2d {
            command: RenderLayer2dSceneCommand {
                source_mod: source_mod.to_owned(),
                id: layer.id.clone(),
                label: layer.label.clone(),
                order: layer.order,
                visible: layer.visible,
                opacity: layer.opacity.clamp(0.0, 1.0),
            },
        });
    }

    for route in &document.visual2d.light_routes {
        commands.push(SceneCommand::QueueLightRoute2d {
            command: LightRoute2dSceneCommand {
                source_mod: source_mod.to_owned(),
                receiver_layer: route.receiver_layer.clone(),
                groups: route.groups.clone(),
            },
        });
    }

    for group in &document.visual2d.light_groups {
        commands.push(SceneCommand::QueueLightGroup2d {
            command: LightGroup2dSceneCommand {
                source_mod: source_mod.to_owned(),
                id: group.id.clone(),
                label: group.label.clone(),
                color: parse_color_rgba_hex(
                    &group.color,
                    &document.scene.id,
                    "visual2d",
                    "LightGroup2D",
                )?,
                intensity: group.intensity.max(0.0),
                sources: group
                    .sources
                    .iter()
                    .map(light_group_source_from_document)
                    .collect(),
            },
        });
    }

    Ok(())
}

fn light_group_source_from_document(
    source: &LightGroup2dSourceDocument,
) -> LightGroup2dSourceSceneCommand {
    match source {
        LightGroup2dSourceDocument::LightmapChannel {
            source,
            channel,
            response,
        } => LightGroup2dSourceSceneCommand {
            kind: LightGroup2dSourceKindSceneCommand::LightMapChannel {
                source: source.clone(),
                channel: channel.clone(),
            },
            response: response.max(0.0),
        },
        LightGroup2dSourceDocument::GlobalLight { id, response } => {
            LightGroup2dSourceSceneCommand {
                kind: LightGroup2dSourceKindSceneCommand::GlobalLight { id: id.clone() },
                response: response.max(0.0),
            }
        }
    }
}
