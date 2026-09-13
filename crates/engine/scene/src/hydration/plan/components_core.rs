use crate::document::SceneComponentDocument as ComponentDocument;

fn hydrate_component_core(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &String,
    component_index: usize,
    component: &SceneComponentDocument,
    hydrators: Option<&crate::ComponentHydratorRegistry>,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    if let Some(hydrators) = hydrators {
        if hydrators.hydrate_first(crate::ComponentHydrationContext {
            source_mod,
            document,
            entity,
            entity_name,
            component_index,
            component,
            commands,
        })? {
            return Ok(true);
        }
    }
    match component {
        ComponentDocument::Camera3d { .. } | ComponentDocument::Light3d { .. } => {}
        ComponentDocument::LightMap2dSource {
            id,
            source,
            channels,
        } => {
            commands.push(SceneCommand::Plugin {
                command: lightmap_2d_source_plugin_scene_command(LightMap2dSourceSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    id: id.clone(),
                    source: lightmap_source_ref_from_document(document, source),
                    channels: channels
                        .iter()
                        .map(lightmap_channel_from_document)
                        .collect(),
                }),
            });
        }
        ComponentDocument::EntityPool { pool, members } => {
            commands.push(SceneCommand::Plugin {
                command: entity_pool_plugin_scene_command(EntityPoolSceneCommand::new(
                    source_mod.to_owned(),
                    pool.clone().unwrap_or_else(|| entity_name.clone()),
                    members.clone(),
                )),
            });
        }
        ComponentDocument::Lifetime {
            seconds,
            outcome,
            pool,
        } => {
            commands.push(SceneCommand::Plugin {
                command: lifetime_plugin_scene_command(LifetimeSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    *seconds,
                    lifetime_outcome_from_document(*outcome, pool.clone()),
                )),
            });
        }
        ComponentDocument::ProjectileEmitter2d {
            pool,
            speed,
            spawn_offset,
            inherit_velocity_scale,
        } => {
            commands.push(SceneCommand::Plugin {
                command: projectile_emitter_2d_plugin_scene_command(
                    ProjectileEmitter2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        pool.clone(),
                        *speed,
                        vec2_from_document(*spawn_offset),
                        *inherit_velocity_scale,
                    ),
                ),
            });
        }
        ComponentDocument::InputActionMap {
            id,
            active,
            actions,
        } => {
            commands.push(SceneCommand::Plugin {
                command: input_action_map_plugin_scene_command(InputActionMapSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    id: id.clone(),
                    active: *active,
                    actions: actions
                        .iter()
                        .map(|(action, binding)| {
                            (action.clone(), input_action_binding_from_document(binding))
                        })
                        .collect(),
                }),
            });
        }
        ComponentDocument::Behavior {
            enabled_when,
            behavior,
        } => {
            commands.push(SceneCommand::Plugin {
                command: behavior_plugin_scene_command(BehaviorSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    condition: enabled_when.as_ref().map(|condition| {
                        BehaviorConditionSceneCommand {
                            state_key: condition.state.clone(),
                            equals: condition.equals.clone(),
                            not_equals: condition.not_equals.clone(),
                            greater_than: condition.greater_than,
                            greater_or_equal: condition.greater_or_equal,
                            less_than: condition.less_than,
                            less_or_equal: condition.less_or_equal,
                            is_true: condition.is_true,
                            is_false: condition.is_false,
                        }
                    }),
                    behavior: behavior_from_document(
                        behavior,
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?,
                }),
            });
        }
        ComponentDocument::EventPipeline { id, topic, steps } => {
            commands.push(SceneCommand::Plugin {
                command: event_pipeline_plugin_scene_command(EventPipelineSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    id: id.clone(),
                    topic: topic.clone(),
                    steps: steps
                        .iter()
                        .map(event_pipeline_step_from_document)
                        .collect(),
                }),
            });
        }
        ComponentDocument::UiModelBindings { bindings } => {
            commands.push(SceneCommand::Plugin {
                command: ui_model_bindings_plugin_scene_command(UiModelBindingsSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    bindings: bindings
                        .iter()
                        .map(ui_model_binding_from_document)
                        .collect(),
                }),
            });
        }
        ComponentDocument::ScriptComponent { script, params } => {
            commands.push(SceneCommand::Plugin {
                command: script_component_plugin_scene_command(ScriptComponentSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    script: script.into(),
                    params: params
                        .iter()
                        .map(|(key, value)| {
                            (key.clone(), script_component_param_from_document(value))
                        })
                        .collect(),
                }),
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn lightmap_source_ref_from_document(
    document: &SceneDocument,
    source: &LightMap2dSourceRefDocument,
) -> LightMap2dSourceRefSceneCommand {
    match source {
        LightMap2dSourceRefDocument::LayeredImage2d { entity } => LightMap2dSourceRefSceneCommand {
            kind: LightMap2dSourceKindSceneCommand::LayeredImage2d,
            entity_name: resolve_entity_display_name(document, entity),
        },
    }
}

fn resolve_entity_display_name(document: &SceneDocument, entity_ref: &str) -> String {
    document
        .entities
        .iter()
        .find(|entity| entity.id == entity_ref || entity.display_name() == entity_ref)
        .map(crate::SceneEntityDocument::display_name)
        .unwrap_or_else(|| entity_ref.to_owned())
}

fn lightmap_channel_from_document(
    channel: &LightMap2dChannelDocument,
) -> LightMap2dChannelSceneCommand {
    LightMap2dChannelSceneCommand {
        id: channel.id.clone(),
        layers: channel.layers.clone(),
    }
}
