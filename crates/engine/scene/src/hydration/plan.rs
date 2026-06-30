use super::style::{parse_color_rgba_hex, ui_theme_from_component};
use super::*;
use amigo_assets::AssetKey;
use amigo_camera::camera_optical_response_from_document;
use amigo_render_api::PostFxScope2d;
use std::collections::BTreeMap;

use super::post_fx::{
    build_scoped_post_fx_stack, component_post_fx_host_id, draw_layer_post_fx_host_id,
    frame_post_fx_host_id, image_part_post_fx_host_id, scene_object_post_fx_host_id,
};

use crate::{
    AabbCollider2dSceneCommand, ActivationEntrySceneCommand, ActivationSetSceneCommand,
    AudioCueSceneCommand, BehaviorConditionSceneCommand, BehaviorSceneCommand,
    Bounds2dSceneCommand, BoxCollider3dSceneCommand, CameraController3dModeSceneCommand,
    CameraController3dSceneCommand, CameraFollow2dSceneCommand, CircleCollider2dSceneCommand,
    CollisionEventRule2dSceneCommand, DepthCurve2dSceneCommand, DepthSpace2dSceneCommand,
    EntityPoolSceneCommand, EventPipelineSceneCommand, FreeflightMotion2dSceneCommand,
    InputActionMapSceneCommand, KinematicBody2dSceneCommand, LifetimeSceneCommand,
    LightGroup2dSceneCommand, LightGroup2dSourceDocument, LightGroup2dSourceKindSceneCommand,
    LightGroup2dSourceSceneCommand, LightMap2dChannelDocument, LightMap2dChannelSceneCommand,
    LightMap2dSourceKindSceneCommand, LightMap2dSourceRefDocument, LightMap2dSourceRefSceneCommand,
    LightMap2dSourceSceneCommand, LightRoute2dSceneCommand, Material3dSceneCommand,
    Mesh3dSceneCommand, MotionController2dSceneCommand, NprPreset3dSceneCommand,
    OpticalLayerRole2dDocument, OpticalLayerRole2dSceneCommand, Parallax2dSceneCommand,
    PhysicsSpawner3dSceneCommand, PhysicsWorld3dSceneCommand, PostFx2dDocument,
    ProjectileEmitter2dSceneCommand, RenderContributions2dSceneCommand,
    RenderContributionsDocument, RenderDepth2dDocument, RenderDepth2dSceneCommand,
    RenderDepthMode2dDocument, RenderDepthMode2dSceneCommand, RenderLayer2dSceneCommand,
    RigidBody3dSceneCommand, SceneCommand, SceneComponentDocument, SceneDocument,
    SceneDocumentResult, SceneEntityLifecycleOverride, ScenePropertyValue,
    ScriptComponentSceneCommand, StaticBoxCollider3dSceneCommand, StaticCollider2dSceneCommand,
    Text3dSceneCommand, TileMapMarker2dSceneCommand, Trigger2dSceneCommand,
    UiModelBindingsSceneCommand, UiSceneCommand, UiThemeSetSceneCommand, Velocity2dSceneCommand,
    aabb_collider_2d_plugin_scene_command, audio_cue_plugin_scene_command,
    behavior_plugin_scene_command, bounds_2d_plugin_scene_command,
    box_collider_3d_plugin_scene_command, camera_controller_3d_plugin_scene_command,
    camera_follow_2d_plugin_scene_command, circle_collider_2d_plugin_scene_command,
    collision_event_rule_2d_plugin_scene_command, entity_pool_plugin_scene_command,
    event_pipeline_plugin_scene_command, freeflight_motion_2d_plugin_scene_command,
    input_action_map_plugin_scene_command, kinematic_body_2d_plugin_scene_command,
    lifetime_plugin_scene_command, light_group_2d_plugin_scene_command,
    light_route_2d_plugin_scene_command, lightmap_2d_source_plugin_scene_command,
    material_3d_plugin_scene_command, mesh_3d_plugin_scene_command,
    motion_controller_2d_plugin_scene_command, npr_preset_3d_plugin_scene_command,
    parallax_2d_plugin_scene_command, physics_spawner_3d_plugin_scene_command,
    physics_world_3d_plugin_scene_command, projectile_emitter_2d_plugin_scene_command,
    render_layer_2d_plugin_scene_command, rigid_body_3d_plugin_scene_command,
    script_component_plugin_scene_command, static_box_collider_3d_plugin_scene_command,
    static_collider_2d_plugin_scene_command, text_3d_plugin_scene_command,
    tilemap_marker_2d_plugin_scene_command, trigger_2d_plugin_scene_command,
    ui_document_plugin_scene_command, ui_model_bindings_plugin_scene_command,
    ui_theme_set_plugin_scene_command, velocity_2d_plugin_scene_command,
    visual2d_spatial_plugin_scene_command,
};

#[derive(Debug, Clone, PartialEq)]
pub struct SceneHydrationPlan {
    pub commands: Vec<SceneCommand>,
}

pub fn build_scene_hydration_plan(
    source_mod: &str,
    document: &SceneDocument,
) -> SceneDocumentResult<SceneHydrationPlan> {
    build_scene_hydration_plan_with_component_hydrators(source_mod, document, None, None)
}

pub fn build_scene_hydration_plan_for_runtime(
    runtime: &amigo_runtime::Runtime,
    source_mod: &str,
    document: &SceneDocument,
) -> SceneDocumentResult<SceneHydrationPlan> {
    let hydrators = runtime.resolve::<crate::ComponentHydratorRegistry>();
    let schemas = runtime.resolve::<crate::ComponentSchemaRegistry>();
    build_scene_hydration_plan_with_component_hydrators(
        source_mod,
        document,
        hydrators.as_deref(),
        schemas.as_deref(),
    )
}

pub fn build_scene_hydration_plan_with_component_hydrators(
    source_mod: &str,
    document: &SceneDocument,
    hydrators: Option<&crate::ComponentHydratorRegistry>,
    schemas: Option<&crate::ComponentSchemaRegistry>,
) -> SceneDocumentResult<SceneHydrationPlan> {
    // Architectural stop-point:
    // Hydration still consumes SceneDocument directly until typed graph hydration lands.
    // New object/reference semantics live in graph::SemanticSceneGraph.
    // Do not add more ad-hoc string resolution here.
    // New features that need object/layer/effect references should first add
    // typed reference extraction in graph::build_semantic_scene_graph.
    let mut commands = Vec::new();

    hydrate_visual2d(source_mod, document, &mut commands)?;
    hydrate_npr_presets(source_mod, document, &mut commands)?;

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
            properties: runtime_properties_for_entity(entity),
        });

        for (component_index, component) in entity.components.iter().enumerate() {
            if hydrate_plugin_component_payload(
                source_mod,
                document,
                entity,
                &entity_name,
                component_index,
                component,
                hydrators,
                schemas,
                &mut commands,
            )? {
                continue;
            }
            if hydrate_component_core(
                source_mod,
                document,
                entity,
                &entity_name,
                component_index,
                component,
                hydrators,
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
        commands.push(SceneCommand::Plugin {
            command: collision_event_rule_2d_plugin_scene_command(
                CollisionEventRule2dSceneCommand::new(
                    source_mod.to_owned(),
                    rule.id.clone(),
                    entity_selector_from_document(&rule.source),
                    entity_selector_from_document(&rule.target),
                    rule.event.clone(),
                    rule.once_per_overlap,
                ),
            ),
        });
    }

    for cue in &document.audio_cues {
        commands.push(SceneCommand::Plugin {
            command: audio_cue_plugin_scene_command(AudioCueSceneCommand {
                source_mod: source_mod.to_owned(),
                name: cue.name.clone(),
                clip: AssetKey::new(resolve_scene_audio_clip(source_mod, &cue.clip)),
                min_interval: cue
                    .min_interval
                    .filter(|value| value.is_finite())
                    .map(|value| value.max(0.0)),
            }),
        });
    }

    for set in &document.activation_sets {
        commands.push(SceneCommand::ConfigureActivationSet {
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

fn hydrate_npr_presets(
    source_mod: &str,
    document: &SceneDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<()> {
    for preset in &document.npr_presets {
        let crate::document::NprPreset3dDocument::Definition(preset) = preset else {
            continue;
        };
        let settings = npr_line_settings_3d_from_settings_document(
            &preset.settings,
            &document.scene.id,
            &preset.id,
            "NprPreset3D",
        )?;
        commands.push(SceneCommand::Plugin {
            command: npr_preset_3d_plugin_scene_command(NprPreset3dSceneCommand {
                source_mod: source_mod.to_owned(),
                id: preset.id.clone(),
                label: preset.label.clone(),
                settings,
            }),
        });
    }
    Ok(())
}

fn hydrate_plugin_component_payload(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &str,
    component_index: usize,
    component: &SceneComponentDocument,
    hydrators: Option<&crate::ComponentHydratorRegistry>,
    schemas: Option<&crate::ComponentSchemaRegistry>,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    let (Some(hydrators), Some(schemas)) = (hydrators, schemas) else {
        return Ok(false);
    };
    let Some((component_type, payload)) = component.plugin_payload() else {
        return Ok(false);
    };
    let Some(payload) = schemas.parse_typed_plugin_payload(component_type, payload) else {
        return Ok(false);
    };
    let payload = payload?;
    hydrators.hydrate_plugin_payload(
        payload.component_type(),
        crate::PluginComponentHydrationContext {
            source_mod,
            document,
            entity,
            entity_name,
            component_index,
            component_type: payload.component_type(),
            payload: payload.as_ref(),
            commands,
        },
    )
}

include!("plan/components_core.rs");
include!("plan/components_domains.rs");

fn hydrate_visual2d(
    source_mod: &str,
    document: &SceneDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<()> {
    let depth_space = document.visual2d.spatial.depth_space.to_runtime();
    commands.push(SceneCommand::Plugin {
        command: visual2d_spatial_plugin_scene_command(DepthSpace2dSceneCommand {
            near_m: depth_space.near_m,
            far_m: depth_space.far_m,
            curve: match depth_space.curve {
                amigo_2d_spatial::DepthCurve2d::Linear => DepthCurve2dSceneCommand::Linear,
                amigo_2d_spatial::DepthCurve2d::Logarithmic => {
                    DepthCurve2dSceneCommand::Logarithmic
                }
            },
        }),
    });
    for layer in &document.visual2d.render_layers {
        commands.push(SceneCommand::Plugin {
            command: render_layer_2d_plugin_scene_command(RenderLayer2dSceneCommand {
                source_mod: source_mod.to_owned(),
                id: layer.id.clone(),
                label: layer.label.clone(),
                order: layer.order,
                visible: layer.visible,
                opacity: layer.opacity.clamp(0.0, 1.0),
                depth: render_depth_from_document(&layer.depth, depth_space),
                optical_role: optical_layer_role_from_document(layer.optical_role),
            }),
        });
    }

    for route in &document.visual2d.light_routes {
        commands.push(SceneCommand::Plugin {
            command: light_route_2d_plugin_scene_command(LightRoute2dSceneCommand {
                source_mod: source_mod.to_owned(),
                receiver_layer: route.receiver_layer.clone(),
                groups: route.groups.clone(),
            }),
        });
    }

    for group in &document.visual2d.light_groups {
        commands.push(SceneCommand::Plugin {
            command: light_group_2d_plugin_scene_command(LightGroup2dSceneCommand {
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
                render_contributions: RenderContributions2dSceneCommand {
                    roles: light_group_render_contribution_defaults(&group.render_contributions)
                        .into_roles(),
                },
                camera_response: camera_optical_response_from_document(group.camera_response),
                sources: group
                    .sources
                    .iter()
                    .map(light_group_source_from_document)
                    .collect(),
            }),
        });
    }

    let mut stacks = Vec::new();
    let mut lens_reports = Vec::new();

    let (stack, reports) = build_scoped_post_fx_stack(
        frame_post_fx_host_id(&document.scene.id),
        PostFxScope2d::Frame,
        &document.visual2d.post_fx,
        &document.scene.id,
        "visual2d",
        "PostFx2d",
    )?;
    if let Some(stack) = stack {
        stacks.push(stack);
    }
    lens_reports.extend(reports);

    for layer in &document.visual2d.render_layers {
        let (stack, reports) = build_scoped_post_fx_stack(
            draw_layer_post_fx_host_id(&layer.id),
            PostFxScope2d::DrawLayer {
                draw_layer_id: layer.id.clone(),
            },
            &layer.post_fx,
            &document.scene.id,
            "visual2d",
            "PostFx2d",
        )?;
        if let Some(stack) = stack {
            stacks.push(stack);
        }
        lens_reports.extend(reports);
    }

    for entity in &document.entities {
        let (stack, reports) = build_scoped_post_fx_stack(
            scene_object_post_fx_host_id(&entity.id),
            PostFxScope2d::SceneObjectPixels {
                scene_object_id: entity.id.clone(),
            },
            &entity.post_fx,
            &document.scene.id,
            &entity.id,
            "SceneObject2D",
        )?;
        if let Some(stack) = stack {
            stacks.push(stack);
        }
        lens_reports.extend(reports);

        for (component_index, component) in entity.components.iter().enumerate() {
            if let Some(component_docs) = component_post_fx_documents(component) {
                let (stack, reports) = build_scoped_post_fx_stack(
                    component_post_fx_host_id(&entity.id, component_index, component.kind()),
                    PostFxScope2d::SceneObjectPixels {
                        scene_object_id: entity.id.clone(),
                    },
                    component_docs,
                    &document.scene.id,
                    &entity.id,
                    component.kind(),
                )?;
                if let Some(stack) = stack {
                    stacks.push(stack);
                }
                lens_reports.extend(reports);
            }

            if let Some(layer_override_docs) = layered_image_part_post_fx_documents(component) {
                for (part_id, docs) in layer_override_docs {
                    let (stack, reports) = build_scoped_post_fx_stack(
                        image_part_post_fx_host_id(&entity.id, component_index, part_id),
                        PostFxScope2d::ImagePart {
                            owner_scene_object_id: entity.id.clone(),
                            component_id: Some(format!(
                                "{}:{}:{}",
                                entity.id,
                                component_index,
                                component.kind()
                            )),
                            part_id: part_id.to_string(),
                        },
                        docs,
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?;
                    if let Some(stack) = stack {
                        stacks.push(stack);
                    }
                    lens_reports.extend(reports);
                }
            }
        }
    }

    if !stacks.is_empty() || !lens_reports.is_empty() {
        commands.push(SceneCommand::SetPostFx2dStacks {
            stacks,
            lens_certification_reports: lens_reports,
        });
    }

    Ok(())
}

fn light_group_render_contribution_defaults(
    contributions: &RenderContributionsDocument,
) -> RenderContributionsDocument {
    contributions.clone().with_defaults([
        ("lighting.emit", true),
        ("bloom.source", false),
        ("camera.fx_source", false),
    ])
}

fn optical_layer_role_from_document(
    role: OpticalLayerRole2dDocument,
) -> OpticalLayerRole2dSceneCommand {
    match role {
        OpticalLayerRole2dDocument::WorldSurface => OpticalLayerRole2dSceneCommand::WorldSurface,
        OpticalLayerRole2dDocument::SceneMedium => OpticalLayerRole2dSceneCommand::SceneMedium,
        OpticalLayerRole2dDocument::ForegroundMedium => {
            OpticalLayerRole2dSceneCommand::ForegroundMedium
        }
        OpticalLayerRole2dDocument::LensSurface => OpticalLayerRole2dSceneCommand::LensSurface,
        OpticalLayerRole2dDocument::Overlay => OpticalLayerRole2dSceneCommand::Overlay,
        OpticalLayerRole2dDocument::Debug => OpticalLayerRole2dSceneCommand::Debug,
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

fn component_post_fx_documents(component: &SceneComponentDocument) -> Option<&[PostFx2dDocument]> {
    component.post_fx_documents()
}

fn runtime_properties_for_entity(
    entity: &crate::SceneEntityDocument,
) -> BTreeMap<String, ScenePropertyValue> {
    let mut properties: BTreeMap<String, ScenePropertyValue> = entity
        .properties
        .iter()
        .map(|(key, value)| (key.clone(), property_value_from_document(value)))
        .collect();

    for component in &entity.components {
        match component {
            SceneComponentDocument::Camera3d {
                fov_y_degrees,
                near_clip,
                far_clip,
                background_color,
            } => {
                properties.insert(
                    "camera3d.fov_y_degrees".to_owned(),
                    ScenePropertyValue::Float(f64::from(*fov_y_degrees)),
                );
                properties.insert(
                    "camera3d.near_clip".to_owned(),
                    ScenePropertyValue::Float(f64::from(*near_clip)),
                );
                properties.insert(
                    "camera3d.far_clip".to_owned(),
                    ScenePropertyValue::Float(f64::from(*far_clip)),
                );
                if let Some(background_color) = background_color {
                    properties.insert(
                        "camera3d.background_color".to_owned(),
                        ScenePropertyValue::String(background_color.clone()),
                    );
                }
            }
            SceneComponentDocument::Light3d {
                direction,
                color,
                intensity,
                ambient,
                ..
            } => {
                properties.insert(
                    "light3d.direction.x".to_owned(),
                    ScenePropertyValue::Float(f64::from(direction.x)),
                );
                properties.insert(
                    "light3d.direction.y".to_owned(),
                    ScenePropertyValue::Float(f64::from(direction.y)),
                );
                properties.insert(
                    "light3d.direction.z".to_owned(),
                    ScenePropertyValue::Float(f64::from(direction.z)),
                );
                properties.insert(
                    "light3d.intensity".to_owned(),
                    ScenePropertyValue::Float(f64::from(*intensity)),
                );
                properties.insert(
                    "light3d.ambient".to_owned(),
                    ScenePropertyValue::Float(f64::from(*ambient)),
                );
                if let Some(color) = color {
                    properties.insert(
                        "light3d.color".to_owned(),
                        ScenePropertyValue::String(color.clone()),
                    );
                }
            }
            _ => {}
        }
    }

    properties
}

fn layered_image_part_post_fx_documents(
    component: &SceneComponentDocument,
) -> Option<Vec<(&str, &[PostFx2dDocument])>> {
    component.layered_image_part_post_fx_documents()
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
use amigo_2d_spatial::{DepthSource2d, resolve_depth_source};
