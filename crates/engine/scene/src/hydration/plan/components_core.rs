fn hydrate_component_core(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &String,
    component: &SceneComponentDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    match component {
                SceneComponentDocument::Camera2d
                | SceneComponentDocument::Camera3d
                | SceneComponentDocument::Light3d { .. } => {}
                SceneComponentDocument::Sprite2d {
                    texture,
                    size,
                    sheet,
                    animation,
                    z_index,
                } => {
                    commands.push(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            texture: AssetKey::new(texture.clone()),
                            size: vec2_from_document(*size),
                            sheet: sheet.map(sprite_sheet_from_document),
                            animation: animation.map(sprite_animation_from_document),
                            z_index: *z_index,
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::TileMap2d {
                    tileset,
                    ruleset,
                    tile_size,
                    editor: _,
                    grid,
                    depth_fill_rows,
                    z_index,
                } => {
                    let mut command = TileMap2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        AssetKey::new(tileset.clone()),
                        vec2_from_document(*tile_size),
                        grid.clone(),
                    );
                    command.ruleset = ruleset.clone().map(AssetKey::new);
                    command.depth_fill_rows = *depth_fill_rows;
                    command.z_index = *z_index;
                    commands.push(SceneCommand::QueueTileMap2d { command });
                }
                SceneComponentDocument::Text2d {
                    content,
                    font,
                    bounds,
                } => {
                    commands.push(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            content: content.clone(),
                            font: AssetKey::new(font.clone()),
                            bounds: vec2_from_document(*bounds),
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::VectorShape2d {
                    kind,
                    points,
                    closed,
                    radius,
                    segments,
                    stroke_color,
                    stroke_width,
                    fill_color,
                    z_index,
                } => {
                    let stroke_color = stroke_color
                        .as_deref()
                        .map(|value| {
                            parse_color_rgba_hex(
                                value,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )
                        })
                        .transpose()?
                        .unwrap_or(ColorRgba::WHITE);
                    let fill_color = fill_color
                        .as_deref()
                        .map(|value| {
                            parse_color_rgba_hex(
                                value,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )
                        })
                        .transpose()?;
                    let kind = match kind {
                        SceneVectorShapeKindComponentDocument::Polyline => {
                            VectorShapeKind2dSceneCommand::Polyline {
                                points: points.iter().copied().map(vec2_from_document).collect(),
                                closed: *closed,
                            }
                        }
                        SceneVectorShapeKindComponentDocument::Polygon => {
                            VectorShapeKind2dSceneCommand::Polygon {
                                points: points.iter().copied().map(vec2_from_document).collect(),
                            }
                        }
                        SceneVectorShapeKindComponentDocument::Circle => {
                            VectorShapeKind2dSceneCommand::Circle {
                                radius: (*radius).max(0.0),
                                segments: (*segments).max(3),
                            }
                        }
                    };
                    let mut command = VectorShape2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        kind,
                        VectorStyle2dSceneCommand {
                            stroke_color,
                            stroke_width: (*stroke_width).max(0.0),
                            fill_color,
                        },
                    );
                    command.z_index = *z_index;
                    command.transform = transform2_for_entity(entity);
                    commands.push(SceneCommand::QueueVectorShape2d { command });
                }
                SceneComponentDocument::EntityPool { pool, members } => {
                    commands.push(SceneCommand::QueueEntityPool {
                        command: EntityPoolSceneCommand::new(
                            source_mod.to_owned(),
                            pool.clone().unwrap_or_else(|| entity_name.clone()),
                            members.clone(),
                        ),
                    });
                }
                SceneComponentDocument::Lifetime {
                    seconds,
                    outcome,
                    pool,
                } => {
                    commands.push(SceneCommand::QueueLifetime {
                        command: LifetimeSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            *seconds,
                            lifetime_outcome_from_document(*outcome, pool.clone()),
                        ),
                    });
                }
                SceneComponentDocument::ProjectileEmitter2d {
                    pool,
                    speed,
                    spawn_offset,
                    inherit_velocity_scale,
                } => {
                    commands.push(SceneCommand::QueueProjectileEmitter2d {
                        command: ProjectileEmitter2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            pool.clone(),
                            *speed,
                            vec2_from_document(*spawn_offset),
                            *inherit_velocity_scale,
                        ),
                    });
                }
                SceneComponentDocument::InputActionMap {
                    id,
                    active,
                    actions,
                } => {
                    commands.push(SceneCommand::QueueInputActionMap {
                        command: InputActionMapSceneCommand {
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
                        },
                    });
                }
                SceneComponentDocument::Behavior {
                    enabled_when,
                    behavior,
                } => {
                    commands.push(SceneCommand::QueueBehavior {
                        command: BehaviorSceneCommand {
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
                        },
                    });
                }
                SceneComponentDocument::EventPipeline { id, topic, steps } => {
                    commands.push(SceneCommand::QueueEventPipeline {
                        command: EventPipelineSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            id: id.clone(),
                            topic: topic.clone(),
                            steps: steps
                                .iter()
                                .map(event_pipeline_step_from_document)
                                .collect(),
                        },
                    });
                }
                SceneComponentDocument::UiModelBindings { bindings } => {
                    commands.push(SceneCommand::QueueUiModelBindings {
                        command: UiModelBindingsSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            bindings: bindings
                                .iter()
                                .map(ui_model_binding_from_document)
                                .collect(),
                        },
                    });
                }
                SceneComponentDocument::ScriptComponent { script, params } => {
                    commands.push(SceneCommand::QueueScriptComponent {
                        command: ScriptComponentSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            script: script.into(),
                            params: params
                                .iter()
                                .map(|(key, value)| {
                                    (key.clone(), script_component_param_from_document(value))
                                })
                                .collect(),
                        },
                    });
                }
        _ => return Ok(false),
    }
    Ok(true)
}
