use super::style::{parse_color_rgba_hex, parse_optional_color_rgba_hex, ui_theme_from_component};
use super::*;
use amigo_assets::AssetKey;
use amigo_2d_post_fx::{
    LensDroplets2dStage, PostFx2d, PostFx2dStack, PostFxLensDroplets2d,
    PostFxWetReflections2d, WetReflectionsDebugView,
};
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
    LensDroplets2dDocument, Material2dLightingModeSceneCommand, Material2dLightingModeSceneDocument,
    Material3dSceneCommand, Mesh3dSceneCommand, MotionController2dSceneCommand,
    Parallax2dSceneCommand, ParticleEmitter2dSceneCommand, ParticleMotionStretch2dSceneCommand,
    ParticleShapeChoice2dSceneCommand, ParticleShapeKeyframe2dSceneCommand, PostFx2dDocument,
    ProjectileEmitter2dSceneCommand, RenderLayer2dSceneCommand, SceneCommand,
    SceneComponentDocument, SceneDocument, SceneDocumentError, SceneDocumentResult,
    SceneEntityLifecycleOverride, SceneVectorShapeKindComponentDocument,
    ScriptComponentSceneCommand, Sprite2dSceneCommand,
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

    let mut effects = Vec::new();
    let mut lens_reports = Vec::new();
    for effect in &document.visual2d.post_fx {
        match effect {
            PostFx2dDocument::LensDroplets(lens) => {
                let runtime = lens_droplets_from_document(lens);
                let report = runtime.certify();
                if !report.accepted && lens.certification.strict {
                    return Err(SceneDocumentError::Hydration {
                        scene_id: document.scene.id.clone(),
                        entity_id: "visual2d".to_owned(),
                        component_kind: "LensDroplets2D".to_owned(),
                        message: format!("LensDroplets2D `{}` failed certification", lens.id),
                    });
                }
                effects.push(PostFx2d::LensDroplets(report.normalized));
                lens_reports.push(report);
            }
            PostFx2dDocument::WetReflections(wet) => {
                let reflection_mask = wet.masks.reflection.clone().unwrap_or_default();
                if reflection_mask.trim().is_empty() {
                    eprintln!(
                        "warning: wet_reflections `{}` has no reflection mask and will be inactive",
                        wet.id
                    );
                }
                let effect = PostFxWetReflections2d {
                    enabled: wet.enabled,
                    reflection_mask,
                    reflection_mask_invert: wet.masks.reflection_invert.unwrap_or(true),
                    edge_map: wet.masks.edges.clone(),
                    reflection_color: wet.masks.reflection_color.clone(),
                    noise_normal: wet.masks.noise_normal.clone(),
                    blur_px: wet.surface.blur_px,
                    distortion_px: wet.surface.distortion_px,
                    shimmer_strength: wet.surface.shimmer_strength,
                    ripple_strength: wet.surface.ripple_strength,
                    wet_darken: wet.surface.wet_darken,
                    specular_boost: wet.surface.specular_boost,
                    edge_power: wet
                        .light_response
                        .edge_power
                        .unwrap_or(wet.surface.edge_power),
                    light_reflection_strength: wet
                        .light_response
                        .strength
                        .unwrap_or(wet.surface.light_reflection_strength),
                    foreground_strength: wet.perspective.foreground_strength,
                    background_strength: wet.perspective.background_strength,
                    horizon_y: wet.perspective.horizon_y,
                    noise_scale: wet.animation.noise_scale,
                    noise_speed: wet.animation.noise_speed,
                    ripple_speed: wet.animation.ripple_speed,
                    debug_view: WetReflectionsDebugView::Final,
                }
                .normalized();
                effects.push(PostFx2d::WetReflections(effect));
            }
        }
    }

    if !effects.is_empty() || !lens_reports.is_empty() {
        commands.push(SceneCommand::SetPostFx2dStack {
            stack: PostFx2dStack { effects }.normalized(),
            lens_certification_reports: lens_reports,
        });
    }

    Ok(())
}

fn lens_droplets_from_document(lens: &LensDroplets2dDocument) -> PostFxLensDroplets2d {
    let stage = match lens.stage.as_deref() {
        Some("after_world_before_ui") | None => LensDroplets2dStage::AfterWorldBeforeUi,
        Some(_) => LensDroplets2dStage::AfterWorldBeforeUi,
    };

    PostFxLensDroplets2d {
        enabled: lens.enabled,
        stage,
        max_droplets: lens.droplets.max,
        spawn_rate: lens.droplets.spawn_rate,
        min_radius_px: lens.droplets.radius_range[0],
        max_radius_px: lens.droplets.radius_range[1],
        min_opacity: lens.droplets.opacity_range[0],
        max_opacity: lens.droplets.opacity_range[1],
        min_lifetime: lens.droplets.lifetime_range[0],
        max_lifetime: lens.droplets.lifetime_range[1],
        dirt_opacity: lens.surface.dirt_opacity,
        darken: lens.surface.darken,
        blur_px: lens.surface.blur_px,
        blur_samples: lens.surface.blur_samples,
        distortion: lens.surface.distortion,
        downsample: lens.surface.downsample,
        streaks_enabled: lens.streaks.enabled,
        streak_chance: lens.streaks.chance,
        gravity_px_per_sec: lens.streaks.gravity_px_per_sec,
        max_streak_length: lens.streaks.max_length,
        wobble: lens.streaks.wobble,
        affects_world: lens.affects.world,
        affects_game_ui: lens.affects.game_ui,
        affects_debug_ui: lens.affects.debug_ui,
        strict_certification: lens.certification.strict,
    }
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
