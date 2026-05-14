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
                    render_layer,
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
                            render_layer: render_layer.clone(),
                            texture: AssetKey::new(texture.clone()),
                            size: vec2_from_document(*size),
                            sheet: sheet.map(sprite_sheet_from_document),
                            animation: animation.map(sprite_animation_from_document),
                            z_index: *z_index,
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::LayeredImage2d {
                    render_layer,
                    asset,
                    size,
                    base_opacity,
                    viewport_fit,
                    z_index,
                    layer_overrides,
                } => {
                    commands.push(SceneCommand::QueueLayeredImage2d {
                        command: LayeredImage2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            render_layer: render_layer.clone(),
                            asset: AssetKey::new(asset.clone()),
                            size: vec2_from_document(*size),
                            base_opacity: base_opacity.clamp(0.0, 1.0),
                            viewport_fit: layered_image_viewport_fit_from_document(*viewport_fit),
                            z_index: *z_index,
                            transform: transform2_for_entity(entity),
                            layer_overrides: layer_overrides
                                .iter()
                                .map(|item| LayeredImageLayerOverrideSceneCommand {
                                    id: item.id.clone(),
                                    opacity: item.opacity,
                                    enabled: item.enabled,
                                    blend_mode: item.blend.map(layered_image_blend_from_document),
                                })
                                .collect(),
                        },
                    });
                }
                SceneComponentDocument::GlobalLight2d {
                    id,
                    color,
                    intensity,
                } => {
                    commands.push(SceneCommand::QueueGlobalLight2d {
                        command: GlobalLight2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            id: id.clone(),
                            color: parse_color_rgba_hex(
                                color,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )?,
                            intensity: intensity.max(0.0),
                        },
                    });
                }
                SceneComponentDocument::LightMap2dSource {
                    id,
                    source,
                    channels,
                } => {
                    commands.push(SceneCommand::QueueLightMap2dSource {
                        command: LightMap2dSourceSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            id: id.clone(),
                            source: lightmap_source_ref_from_document(source),
                            channels: channels
                                .iter()
                                .map(lightmap_channel_from_document)
                                .collect(),
                        },
                    });
                }
                SceneComponentDocument::TileMap2d {
                    render_layer,
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
                    command.render_layer = render_layer.clone();
                    command.depth_fill_rows = *depth_fill_rows;
                    command.z_index = *z_index;
                    commands.push(SceneCommand::QueueTileMap2d { command });
                }
                SceneComponentDocument::Text2d {
                    render_layer,
                    content,
                    font,
                    bounds,
                    z_index,
                } => {
                    commands.push(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            render_layer: render_layer.clone(),
                            content: content.clone(),
                            font: AssetKey::new(font.clone()),
                            bounds: vec2_from_document(*bounds),
                            z_index: *z_index,
                            transform: transform2_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::VectorShape2d {
                    render_layer,
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
                    command.render_layer = render_layer.clone();
                    command.transform = transform2_for_entity(entity);
                    commands.push(SceneCommand::QueueVectorShape2d { command });
                }
                SceneComponentDocument::BeaconLight2d {
                    id,
                    render_layer,
                    color,
                    base_intensity,
                    frequency_hz,
                    duty_cycle,
                    rise_seconds,
                    fall_seconds,
                    phase_offset,
                    sync_group,
                    jitter_amount,
                    jitter_hz,
                    core_radius_px,
                    halo_radius_px,
                    aberration_px,
                    flare_length_px,
                    flare_strength,
                    z_index,
                    enabled,
                    viewport_fit,
                    viewport_canvas_size,
                } => {
                    commands.push(SceneCommand::QueueBeaconLight2d {
                        command: BeaconLight2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            id: id.clone(),
                            render_layer: render_layer.clone(),
                            color: parse_color_rgba_hex(
                                color,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )?,
                            base_intensity: (*base_intensity).max(0.0),
                            frequency_hz: (*frequency_hz).max(0.0),
                            duty_cycle: duty_cycle.clamp(0.01, 0.99),
                            rise_seconds: (*rise_seconds).max(0.0),
                            fall_seconds: (*fall_seconds).max(0.0),
                            phase_offset: *phase_offset,
                            sync_group: sync_group.clone(),
                            jitter_amount: (*jitter_amount).max(0.0),
                            jitter_hz: (*jitter_hz).max(0.0),
                            core_radius_px: (*core_radius_px).max(0.25),
                            halo_radius_px: (*halo_radius_px).max(0.25),
                            aberration_px: (*aberration_px).max(0.0),
                            flare_length_px: (*flare_length_px).max(0.0),
                            flare_strength: (*flare_strength).max(0.0),
                            z_index: *z_index,
                            enabled: *enabled,
                            transform: transform2_for_entity(entity),
                            viewport_fit: layered_image_viewport_fit_from_document(*viewport_fit),
                            viewport_canvas_size: viewport_canvas_size
                                .as_ref()
                                .map(|size| vec2_from_document(*size)),
                        },
                    });
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

fn layered_image_blend_from_document(
    blend: LayeredImageBlendMode2dDocument,
) -> LayeredImageBlendMode2dSceneCommand {
    match blend {
        LayeredImageBlendMode2dDocument::Alpha => LayeredImageBlendMode2dSceneCommand::Alpha,
        LayeredImageBlendMode2dDocument::Additive => LayeredImageBlendMode2dSceneCommand::Additive,
        LayeredImageBlendMode2dDocument::Screen => LayeredImageBlendMode2dSceneCommand::Screen,
        LayeredImageBlendMode2dDocument::Multiply => LayeredImageBlendMode2dSceneCommand::Multiply,
        LayeredImageBlendMode2dDocument::Lighten => LayeredImageBlendMode2dSceneCommand::Lighten,
    }
}

fn layered_image_viewport_fit_from_document(
    fit: LayeredImageViewportFit2dDocument,
) -> LayeredImageViewportFit2dSceneCommand {
    match fit {
        LayeredImageViewportFit2dDocument::Fixed => LayeredImageViewportFit2dSceneCommand::Fixed,
        LayeredImageViewportFit2dDocument::Stretch => {
            LayeredImageViewportFit2dSceneCommand::Stretch
        }
        LayeredImageViewportFit2dDocument::Contain => {
            LayeredImageViewportFit2dSceneCommand::Contain
        }
        LayeredImageViewportFit2dDocument::Cover => LayeredImageViewportFit2dSceneCommand::Cover,
    }
}

fn lightmap_source_ref_from_document(
    source: &LightMap2dSourceRefDocument,
) -> LightMap2dSourceRefSceneCommand {
    match source {
        LightMap2dSourceRefDocument::LayeredImage2d { entity } => {
            LightMap2dSourceRefSceneCommand {
                kind: LightMap2dSourceKindSceneCommand::LayeredImage2d,
                entity_name: entity.clone(),
            }
        }
    }
}

fn lightmap_channel_from_document(
    channel: &LightMap2dChannelDocument,
) -> LightMap2dChannelSceneCommand {
    LightMap2dChannelSceneCommand {
        id: channel.id.clone(),
        layers: channel.layers.clone(),
    }
}

