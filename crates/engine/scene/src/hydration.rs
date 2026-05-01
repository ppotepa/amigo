use amigo_assets::AssetKey;
use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};
use amigo_math::{ColorRgba, Curve1d, CurvePoint1d, Transform2, Transform3, Vec2, Vec3};

use crate::{
    AabbCollider2dSceneCommand, ActivationEntrySceneCommand, ActivationSetSceneCommand,
    AudioCueSceneCommand, Bounds2dSceneCommand, BoundsBehavior2dSceneCommand,
    CameraFollow2dSceneCommand, CircleCollider2dSceneCommand, CollisionEventRule2dSceneCommand,
    ColorInterpolationSceneDocument, ColorRampSceneDocument, Curve1dSceneDocument,
    EntityPoolSceneCommand, EntitySelector, FreeflightMotion2dSceneCommand,
    KinematicBody2dSceneCommand, LifetimeExpirationOutcome, LifetimeSceneCommand,
    Material3dSceneCommand, Mesh3dSceneCommand, MotionController2dSceneCommand,
    Parallax2dSceneCommand, ParticleAlignMode2dSceneCommand, ParticleAlignMode2dSceneDocument,
    ParticleBlendMode2dSceneCommand, ParticleBlendMode2dSceneDocument,
    ParticleEmitter2dSceneCommand, ParticleForce2dSceneCommand, ParticleForce2dSceneDocument,
    ParticleMotionStretch2dSceneCommand, ParticleShape2dSceneCommand, ParticleShape2dSceneDocument,
    ParticleShapeChoice2dSceneCommand, ParticleShapeKeyframe2dSceneCommand,
    ParticleSpawnArea2dSceneCommand, ParticleSpawnArea2dSceneDocument,
    ProjectileEmitter2dSceneCommand, SceneBoundsBehavior2dDocument, SceneCommand,
    SceneComponentDocument, SceneDocument, SceneDocumentError, SceneDocumentResult,
    SceneEntityDocument, SceneEntityLifecycle, SceneEntityLifecycleOverride,
    SceneEntitySelectorDocument, SceneEntitySelectorKindDocument, SceneKey,
    SceneLifetimeExpirationOutcomeDocument, ScenePropertyValue, ScenePropertyValueDocument,
    SceneSpriteSheetDocument, SceneTransform2Document, SceneTransform3Document, SceneUiBinds,
    SceneUiCurvePoint, SceneUiDocument, SceneUiEventBinding, SceneUiEventBindingComponentDocument,
    SceneUiLayer, SceneUiNode, SceneUiNodeComponentDocument, SceneUiNodeKind,
    SceneUiNodeTypeComponentDocument, SceneUiStyle, SceneUiStyleComponentDocument, SceneUiTab,
    SceneUiTarget, SceneUiTargetComponentDocument, SceneUiTargetTypeComponentDocument,
    SceneUiTextAlign, SceneUiTextAlignComponentDocument, SceneUiTheme,
    SceneUiThemeComponentDocument, SceneUiThemePalette, SceneUiViewport, SceneUiViewportScaling,
    SceneVectorShapeKindComponentDocument, Sprite2dSceneCommand, SpriteAnimation2dSceneOverride,
    SpriteSheet2dSceneCommand, Text2dSceneCommand, Text3dSceneCommand, TileMap2dSceneCommand,
    TileMapMarker2dSceneCommand, Trigger2dSceneCommand, UiSceneCommand, UiThemeSetSceneCommand,
    VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand,
    Velocity2dSceneCommand,
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
                SceneComponentDocument::ParticleEmitter2d {
                    attached_to,
                    local_offset,
                    local_direction_degrees,
                    spawn_area,
                    active,
                    spawn_rate,
                    max_particles,
                    particle_lifetime,
                    lifetime_jitter,
                    initial_speed,
                    speed_jitter,
                    spread_degrees,
                    inherit_parent_velocity,
                    initial_size,
                    final_size,
                    color,
                    color_ramp,
                    z_index,
                    shape,
                    shape_choices,
                    shape_over_lifetime,
                    align,
                    blend_mode,
                    motion_stretch,
                    emission_rate_curve,
                    size_curve,
                    alpha_curve,
                    speed_curve,
                    forces,
                } => {
                    commands.push(SceneCommand::QueueParticleEmitter2d {
                        command: ParticleEmitter2dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            attached_to: attached_to.clone(),
                            local_offset: vec2_from_document(*local_offset),
                            local_direction_radians: local_direction_degrees.to_radians(),
                            spawn_area: particle_spawn_area_from_document(spawn_area.as_ref()),
                            active: *active,
                            spawn_rate: *spawn_rate,
                            max_particles: *max_particles,
                            particle_lifetime: *particle_lifetime,
                            lifetime_jitter: *lifetime_jitter,
                            initial_speed: *initial_speed,
                            speed_jitter: *speed_jitter,
                            spread_radians: spread_degrees.to_radians(),
                            inherit_parent_velocity: *inherit_parent_velocity,
                            initial_size: *initial_size,
                            final_size: *final_size,
                            color: parse_optional_color_rgba_hex(
                                color.as_deref(),
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                                "color",
                            )?
                            .unwrap_or(ColorRgba::WHITE),
                            color_ramp: color_ramp
                                .as_ref()
                                .map(|color_ramp| {
                                    color_ramp_from_document(
                                        color_ramp,
                                        &document.scene.id,
                                        &entity.id,
                                        component.kind(),
                                    )
                                })
                                .transpose()?,
                            z_index: *z_index,
                            shape: particle_shape_from_document(shape.as_ref()),
                            shape_choices: shape_choices
                                .iter()
                                .map(|choice| ParticleShapeChoice2dSceneCommand {
                                    shape: particle_shape_from_document(Some(&choice.shape)),
                                    weight: choice.weight.max(0.0),
                                })
                                .collect(),
                            shape_over_lifetime: shape_over_lifetime
                                .iter()
                                .map(|keyframe| ParticleShapeKeyframe2dSceneCommand {
                                    t: keyframe.t.clamp(0.0, 1.0),
                                    shape: particle_shape_from_document(Some(&keyframe.shape)),
                                })
                                .collect(),
                            align: particle_align_from_document(*align),
                            blend_mode: particle_blend_from_document(*blend_mode),
                            motion_stretch: motion_stretch.map(|motion_stretch| {
                                ParticleMotionStretch2dSceneCommand {
                                    enabled: motion_stretch.enabled,
                                    velocity_scale: motion_stretch.velocity_scale.max(0.0),
                                    max_length: motion_stretch.max_length.max(0.0),
                                }
                            }),
                            emission_rate_curve: curve1d_from_optional_document(
                                emission_rate_curve.as_ref(),
                            ),
                            size_curve: curve1d_from_optional_document(size_curve.as_ref()),
                            alpha_curve: alpha_curve
                                .as_ref()
                                .map(curve1d_from_document)
                                .unwrap_or(Curve1d::Constant(1.0)),
                            speed_curve: speed_curve
                                .as_ref()
                                .map(curve1d_from_document)
                                .unwrap_or(Curve1d::Constant(1.0)),
                            forces: forces.iter().map(particle_force_from_document).collect(),
                        },
                    });
                }
                SceneComponentDocument::Velocity2d { velocity } => {
                    commands.push(SceneCommand::QueueVelocity2d {
                        command: Velocity2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*velocity),
                        ),
                    });
                }
                SceneComponentDocument::Bounds2d {
                    min,
                    max,
                    behavior,
                    restitution,
                } => {
                    commands.push(SceneCommand::QueueBounds2d {
                        command: Bounds2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*min),
                            vec2_from_document(*max),
                            bounds_behavior_from_document(*behavior, *restitution),
                        ),
                    });
                }
                SceneComponentDocument::FreeflightMotion2d {
                    thrust_acceleration,
                    reverse_acceleration,
                    strafe_acceleration,
                    turn_acceleration,
                    linear_damping,
                    turn_damping,
                    max_speed,
                    max_angular_speed,
                    initial_velocity,
                    initial_angular_velocity,
                    thrust_response_curve,
                    reverse_response_curve,
                    strafe_response_curve,
                    turn_response_curve,
                } => {
                    commands.push(SceneCommand::QueueFreeflightMotion2d {
                        command: FreeflightMotion2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            *thrust_acceleration,
                            *reverse_acceleration,
                            *strafe_acceleration,
                            *turn_acceleration,
                            *linear_damping,
                            *turn_damping,
                            *max_speed,
                            *max_angular_speed,
                            vec2_from_document(*initial_velocity),
                            *initial_angular_velocity,
                        )
                        .with_response_curves(
                            curve1d_from_optional_document(thrust_response_curve.as_ref()),
                            curve1d_from_optional_document(reverse_response_curve.as_ref()),
                            curve1d_from_optional_document(strafe_response_curve.as_ref()),
                            curve1d_from_optional_document(turn_response_curve.as_ref()),
                        ),
                    });
                }
                SceneComponentDocument::KinematicBody2d {
                    velocity,
                    gravity_scale,
                    terminal_velocity,
                } => {
                    commands.push(SceneCommand::QueueKinematicBody2d {
                        command: KinematicBody2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*velocity),
                            *gravity_scale,
                            *terminal_velocity,
                        ),
                    });
                }
                SceneComponentDocument::AabbCollider2d {
                    size,
                    offset,
                    layer,
                    mask,
                } => {
                    commands.push(SceneCommand::QueueAabbCollider2d {
                        command: AabbCollider2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*size),
                            vec2_from_document(*offset),
                            layer.clone(),
                            mask.clone(),
                        ),
                    });
                }
                SceneComponentDocument::CircleCollider2d { radius, offset } => {
                    commands.push(SceneCommand::QueueCircleCollider2d {
                        command: CircleCollider2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            (*radius).max(0.0),
                            vec2_from_document(*offset),
                        ),
                    });
                }
                SceneComponentDocument::Trigger2d {
                    size,
                    offset,
                    layer,
                    mask,
                    event,
                } => {
                    commands.push(SceneCommand::QueueTrigger2d {
                        command: Trigger2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*size),
                            vec2_from_document(*offset),
                            layer.clone(),
                            mask.clone(),
                            event.clone(),
                        ),
                    });
                }
                SceneComponentDocument::MotionController2d {
                    max_speed,
                    acceleration,
                    deceleration,
                    air_acceleration,
                    gravity,
                    jump_velocity,
                    terminal_velocity,
                } => {
                    commands.push(SceneCommand::QueueMotionController2d {
                        command: MotionController2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            *max_speed,
                            *acceleration,
                            *deceleration,
                            *air_acceleration,
                            *gravity,
                            *jump_velocity,
                            *terminal_velocity,
                        ),
                    });
                }
                SceneComponentDocument::CameraFollow2d {
                    target,
                    offset,
                    lerp,
                    lookahead_velocity_scale,
                    lookahead_max_distance,
                    sway_amount,
                    sway_frequency,
                } => {
                    commands.push(SceneCommand::QueueCameraFollow2d {
                        command: CameraFollow2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            target.clone(),
                            vec2_from_document(*offset),
                            *lerp,
                        )
                        .with_lookahead(*lookahead_velocity_scale, *lookahead_max_distance)
                        .with_sway(*sway_amount, *sway_frequency),
                    });
                }
                SceneComponentDocument::Parallax2d { camera, factor } => {
                    commands.push(SceneCommand::QueueParallax2d {
                        command: Parallax2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            camera.clone(),
                            vec2_from_document(*factor),
                            transform2_for_entity(entity).translation,
                        ),
                    });
                }
                SceneComponentDocument::TileMapMarker2d {
                    symbol,
                    tilemap_entity,
                    index,
                    offset,
                } => {
                    commands.push(SceneCommand::QueueTileMapMarker2d {
                        command: TileMapMarker2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            tilemap_entity.clone(),
                            symbol.clone(),
                            *index,
                            vec2_from_document(*offset),
                        ),
                    });
                }
                SceneComponentDocument::Mesh3d { mesh } => {
                    commands.push(SceneCommand::QueueMesh3d {
                        command: Mesh3dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            mesh_asset: AssetKey::new(mesh.clone()),
                            transform: transform3_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::Material3d {
                    label,
                    source,
                    albedo,
                } => {
                    let mut command = Material3dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        label.clone(),
                        source.as_ref().map(AssetKey::new),
                    );

                    if let Some(albedo) = albedo.as_deref() {
                        command.albedo = parse_color_rgba_hex(
                            albedo,
                            &document.scene.id,
                            &entity.id,
                            component.kind(),
                        )?;
                    }

                    commands.push(SceneCommand::QueueMaterial3d { command });
                }
                SceneComponentDocument::Text3d {
                    content,
                    font,
                    size,
                } => {
                    commands.push(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            content: content.clone(),
                            font: AssetKey::new(font.clone()),
                            size: *size,
                            transform: transform3_for_entity(entity),
                        },
                    });
                }
                SceneComponentDocument::UiDocument { target, root } => {
                    commands.push(SceneCommand::QueueUi {
                        command: UiSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            document: ui_document_from_component(
                                target,
                                root,
                                &document.scene.id,
                                &entity.id,
                                component.kind(),
                            )?,
                        },
                    });
                }
                SceneComponentDocument::UiThemeSet { active, themes } => {
                    commands.push(SceneCommand::QueueUiThemeSet {
                        command: UiThemeSetSceneCommand {
                            source_mod: source_mod.to_owned(),
                            entity_name: entity_name.clone(),
                            active: active.clone(),
                            themes: themes
                                .iter()
                                .map(|theme| {
                                    ui_theme_from_component(
                                        theme,
                                        &document.scene.id,
                                        &entity.id,
                                        component.kind(),
                                    )
                                })
                                .collect::<SceneDocumentResult<Vec<_>>>()?,
                        },
                    });
                }
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

pub fn scene_key_from_document(document: &SceneDocument) -> SceneKey {
    SceneKey::new(document.scene.id.clone())
}

pub fn entity_selector_from_document(selector: &SceneEntitySelectorDocument) -> EntitySelector {
    match selector.kind {
        SceneEntitySelectorKindDocument::Entity => EntitySelector::Entity(selector.value.clone()),
        SceneEntitySelectorKindDocument::Tag => EntitySelector::Tag(selector.value.clone()),
        SceneEntitySelectorKindDocument::Group => EntitySelector::Group(selector.value.clone()),
        SceneEntitySelectorKindDocument::Pool => EntitySelector::Pool(selector.value.clone()),
    }
}

impl From<SceneEntitySelectorDocument> for EntitySelector {
    fn from(selector: SceneEntitySelectorDocument) -> Self {
        match selector.kind {
            SceneEntitySelectorKindDocument::Entity => Self::Entity(selector.value),
            SceneEntitySelectorKindDocument::Tag => Self::Tag(selector.value),
            SceneEntitySelectorKindDocument::Group => Self::Group(selector.value),
            SceneEntitySelectorKindDocument::Pool => Self::Pool(selector.value),
        }
    }
}

fn transform2_for_entity(entity: &SceneEntityDocument) -> Transform2 {
    entity
        .transform2
        .map(transform2_from_document)
        .or_else(|| entity.transform3.map(transform2_from_transform3_document))
        .unwrap_or_default()
}

fn transform3_for_entity(entity: &SceneEntityDocument) -> Transform3 {
    entity
        .transform3
        .map(transform3_from_document)
        .or_else(|| entity.transform2.map(transform3_from_transform2_document))
        .unwrap_or_default()
}

fn lifecycle_for_entity(entity: &SceneEntityDocument) -> SceneEntityLifecycle {
    SceneEntityLifecycle {
        visible: entity.visible,
        simulation_enabled: entity.simulation_enabled,
        collision_enabled: entity.collision_enabled,
    }
}

fn property_value_from_document(value: &ScenePropertyValueDocument) -> ScenePropertyValue {
    match value {
        ScenePropertyValueDocument::Bool(value) => ScenePropertyValue::Bool(*value),
        ScenePropertyValueDocument::Int(value) => ScenePropertyValue::Int(*value),
        ScenePropertyValueDocument::Float(value) => ScenePropertyValue::Float(*value),
        ScenePropertyValueDocument::String(value) => ScenePropertyValue::String(value.clone()),
    }
}

fn resolve_scene_audio_clip(source_mod: &str, clip: &str) -> String {
    if clip.contains('/') {
        clip.to_owned()
    } else {
        format!("{source_mod}/audio/{clip}")
    }
}

fn lifetime_outcome_from_document(
    outcome: SceneLifetimeExpirationOutcomeDocument,
    pool: Option<String>,
) -> LifetimeExpirationOutcome {
    match outcome {
        SceneLifetimeExpirationOutcomeDocument::Hide => LifetimeExpirationOutcome::Hide,
        SceneLifetimeExpirationOutcomeDocument::Disable => LifetimeExpirationOutcome::Disable,
        SceneLifetimeExpirationOutcomeDocument::Despawn => LifetimeExpirationOutcome::Despawn,
        SceneLifetimeExpirationOutcomeDocument::ReturnToPool => {
            LifetimeExpirationOutcome::ReturnToPool {
                pool: pool.unwrap_or_default(),
            }
        }
    }
}

fn bounds_behavior_from_document(
    behavior: SceneBoundsBehavior2dDocument,
    restitution: f32,
) -> BoundsBehavior2dSceneCommand {
    match behavior {
        SceneBoundsBehavior2dDocument::Bounce => BoundsBehavior2dSceneCommand::Bounce {
            restitution: restitution.max(0.0),
        },
        SceneBoundsBehavior2dDocument::Wrap => BoundsBehavior2dSceneCommand::Wrap,
        SceneBoundsBehavior2dDocument::Hide => BoundsBehavior2dSceneCommand::Hide,
        SceneBoundsBehavior2dDocument::Despawn => BoundsBehavior2dSceneCommand::Despawn,
        SceneBoundsBehavior2dDocument::Clamp => BoundsBehavior2dSceneCommand::Clamp,
    }
}

fn transform2_from_document(document: SceneTransform2Document) -> Transform2 {
    Transform2 {
        translation: vec2_from_document(document.translation),
        rotation_radians: document.rotation_radians,
        scale: vec2_from_document(document.scale),
    }
}

fn transform3_from_document(document: SceneTransform3Document) -> Transform3 {
    Transform3 {
        translation: vec3_from_document(document.translation),
        rotation_euler: vec3_from_document(document.rotation_euler),
        scale: vec3_from_document(document.scale),
    }
}

fn transform3_from_transform2_document(document: SceneTransform2Document) -> Transform3 {
    Transform3 {
        translation: Vec3::new(document.translation.x, document.translation.y, 0.0),
        rotation_euler: Vec3::new(0.0, 0.0, document.rotation_radians),
        scale: Vec3::new(document.scale.x, document.scale.y, 1.0),
    }
}

fn transform2_from_transform3_document(document: SceneTransform3Document) -> Transform2 {
    Transform2 {
        translation: Vec2::new(document.translation.x, document.translation.y),
        rotation_radians: document.rotation_euler.z,
        scale: Vec2::new(document.scale.x, document.scale.y),
    }
}

fn vec2_from_document(value: crate::SceneVec2Document) -> Vec2 {
    Vec2::new(value.x, value.y)
}

fn vec3_from_document(value: crate::SceneVec3Document) -> Vec3 {
    Vec3::new(value.x, value.y, value.z)
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
    value: crate::SceneSpriteAnimationDocument,
) -> SpriteAnimation2dSceneOverride {
    SpriteAnimation2dSceneOverride {
        fps: value.fps.map(|fps| fps.max(0.0)),
        looping: value.looping,
        start_frame: value.start_frame,
    }
}

fn ui_document_from_component(
    target: &SceneUiTargetComponentDocument,
    root: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiDocument> {
    Ok(SceneUiDocument {
        target: ui_target_from_component(target),
        root: ui_node_from_component(root, scene_id, entity_id, component_kind)?,
    })
}

fn ui_target_from_component(target: &SceneUiTargetComponentDocument) -> SceneUiTarget {
    match target.kind {
        SceneUiTargetTypeComponentDocument::ScreenSpace => SceneUiTarget::ScreenSpace {
            layer: match target.layer {
                crate::SceneUiLayerComponentDocument::Background => SceneUiLayer::Background,
                crate::SceneUiLayerComponentDocument::Hud => SceneUiLayer::Hud,
                crate::SceneUiLayerComponentDocument::Menu => SceneUiLayer::Menu,
                crate::SceneUiLayerComponentDocument::Debug => SceneUiLayer::Debug,
            },
            viewport: target.viewport.map(|viewport| SceneUiViewport {
                width: viewport.width,
                height: viewport.height,
                scaling: match viewport.scaling {
                    crate::SceneUiViewportScalingComponentDocument::Expand => {
                        SceneUiViewportScaling::Expand
                    }
                    crate::SceneUiViewportScalingComponentDocument::Fixed => {
                        SceneUiViewportScaling::Fixed
                    }
                    crate::SceneUiViewportScalingComponentDocument::Fit => {
                        SceneUiViewportScaling::Fit
                    }
                },
            }),
        },
    }
}

fn ui_node_from_component(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiNode> {
    let kind = match node.kind {
        SceneUiNodeTypeComponentDocument::Panel => SceneUiNodeKind::Panel,
        SceneUiNodeTypeComponentDocument::GroupBox => SceneUiNodeKind::GroupBox {
            label: node.text.clone().unwrap_or_default(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Row => SceneUiNodeKind::Row,
        SceneUiNodeTypeComponentDocument::Column => SceneUiNodeKind::Column,
        SceneUiNodeTypeComponentDocument::Stack => SceneUiNodeKind::Stack,
        SceneUiNodeTypeComponentDocument::Text => SceneUiNodeKind::Text {
            content: required_ui_text(node, scene_id, entity_id, component_kind, "text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Button => SceneUiNodeKind::Button {
            text: required_ui_text(node, scene_id, entity_id, component_kind, "button text")?,
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::ProgressBar => SceneUiNodeKind::ProgressBar {
            value: node.value.unwrap_or(0.0).clamp(0.0, 1.0),
        },
        SceneUiNodeTypeComponentDocument::Slider => SceneUiNodeKind::Slider {
            value: node.value.unwrap_or(0.0),
            min: node.min.unwrap_or(0.0),
            max: node.max.unwrap_or(1.0),
            step: node.step.unwrap_or(0.0).max(0.0),
        },
        SceneUiNodeTypeComponentDocument::Toggle => SceneUiNodeKind::Toggle {
            checked: node.checked.unwrap_or(false),
            text: node.text.clone().unwrap_or_default(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::OptionSet => SceneUiNodeKind::OptionSet {
            selected: node
                .selected
                .clone()
                .or_else(|| node.options.first().cloned())
                .unwrap_or_default(),
            options: node.options.clone(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::Dropdown => SceneUiNodeKind::Dropdown {
            selected: node
                .selected
                .clone()
                .or_else(|| node.options.first().cloned())
                .unwrap_or_default(),
            options: node.options.clone(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::TabView => SceneUiNodeKind::TabView {
            selected: node
                .selected
                .clone()
                .or_else(|| node.tabs.first().map(|tab| tab.id.clone()))
                .unwrap_or_default(),
            tabs: node
                .tabs
                .iter()
                .map(|tab| SceneUiTab {
                    id: tab.id.clone(),
                    label: tab.label.clone(),
                })
                .collect(),
            font: node.font.clone().map(AssetKey::new),
        },
        SceneUiNodeTypeComponentDocument::ColorPickerRgb => SceneUiNodeKind::ColorPickerRgb {
            color: parse_optional_color_rgba_hex(
                node.color.as_deref(),
                scene_id,
                entity_id,
                component_kind,
                "color",
            )?
            .unwrap_or(ColorRgba::WHITE),
        },
        SceneUiNodeTypeComponentDocument::CurveEditor => SceneUiNodeKind::CurveEditor {
            points: ui_curve_points_from_component(node),
        },
        SceneUiNodeTypeComponentDocument::Spacer => SceneUiNodeKind::Spacer,
    };

    Ok(SceneUiNode {
        id: node.id.clone(),
        kind,
        style_class: node.style_class.clone(),
        style: ui_style_from_component(&node.style, scene_id, entity_id, component_kind)?,
        binds: SceneUiBinds {
            text: node.text_bind.clone(),
            visible: node.visible_bind.clone(),
            enabled: node.enabled_bind.clone(),
            value: node.value_bind.clone(),
        },
        on_click: node.on_click.as_ref().map(ui_event_binding_from_component),
        on_change: node.on_change.as_ref().map(ui_event_binding_from_component),
        children: node
            .children
            .iter()
            .map(|child| ui_node_from_component(child, scene_id, entity_id, component_kind))
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}

fn ui_curve_points_from_component(node: &SceneUiNodeComponentDocument) -> Vec<SceneUiCurvePoint> {
    let points = if node.points.is_empty() {
        let denominator = node.values.len().saturating_sub(1).max(1) as f32;
        node.values
            .iter()
            .enumerate()
            .map(|(index, value)| SceneUiCurvePoint {
                t: index as f32 / denominator,
                value: *value,
            })
            .collect::<Vec<_>>()
    } else {
        node.points
            .iter()
            .map(|point| SceneUiCurvePoint {
                t: point.t,
                value: point.value,
            })
            .collect::<Vec<_>>()
    };

    normalize_scene_ui_curve_points(points)
}

fn normalize_scene_ui_curve_points(mut points: Vec<SceneUiCurvePoint>) -> Vec<SceneUiCurvePoint> {
    if points.is_empty() {
        points = vec![
            SceneUiCurvePoint { t: 0.0, value: 0.0 },
            SceneUiCurvePoint {
                t: 1.0 / 3.0,
                value: 1.0 / 3.0,
            },
            SceneUiCurvePoint {
                t: 2.0 / 3.0,
                value: 2.0 / 3.0,
            },
            SceneUiCurvePoint { t: 1.0, value: 1.0 },
        ];
    }

    for point in &mut points {
        point.t = point.t.clamp(0.0, 1.0);
        point.value = point.value.clamp(0.0, 1.0);
    }
    points.sort_by(|a, b| a.t.total_cmp(&b.t));

    while points.len() < 4 {
        let t = (points.len() as f32 / 3.0).clamp(0.0, 1.0);
        points.push(SceneUiCurvePoint { t, value: t });
        points.sort_by(|a, b| a.t.total_cmp(&b.t));
    }

    points
}

fn curve1d_from_optional_document(document: Option<&Curve1dSceneDocument>) -> Curve1d {
    document
        .map(curve1d_from_document)
        .unwrap_or(Curve1d::Linear)
}

fn curve1d_from_document(document: &Curve1dSceneDocument) -> Curve1d {
    match document {
        Curve1dSceneDocument::Constant { value } => Curve1d::Constant(*value),
        Curve1dSceneDocument::Linear => Curve1d::Linear,
        Curve1dSceneDocument::EaseIn => Curve1d::EaseIn,
        Curve1dSceneDocument::EaseOut => Curve1d::EaseOut,
        Curve1dSceneDocument::EaseInOut => Curve1d::EaseInOut,
        Curve1dSceneDocument::SmoothStep => Curve1d::SmoothStep,
        Curve1dSceneDocument::Custom { points } => Curve1d::Custom {
            points: points
                .iter()
                .map(|point| CurvePoint1d {
                    t: point.t,
                    value: point.value,
                })
                .collect(),
        },
    }
}

fn color_ramp_from_document(
    document: &ColorRampSceneDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRamp> {
    Ok(ColorRamp {
        interpolation: match document.interpolation {
            ColorInterpolationSceneDocument::LinearRgb => ColorInterpolation::LinearRgb,
            ColorInterpolationSceneDocument::Step => ColorInterpolation::Step,
        },
        stops: document
            .stops
            .iter()
            .map(|stop| {
                Ok(ColorStop {
                    t: stop.t,
                    color: parse_color_rgba_hex(&stop.color, scene_id, entity_id, component_kind)?,
                })
            })
            .collect::<SceneDocumentResult<Vec<_>>>()?,
    })
}

fn particle_shape_from_document(
    document: Option<&ParticleShape2dSceneDocument>,
) -> ParticleShape2dSceneCommand {
    match document {
        Some(ParticleShape2dSceneDocument::Circle { segments }) => {
            ParticleShape2dSceneCommand::Circle {
                segments: (*segments).max(3),
            }
        }
        Some(ParticleShape2dSceneDocument::Quad) => ParticleShape2dSceneCommand::Quad,
        Some(ParticleShape2dSceneDocument::Line { length }) => {
            ParticleShape2dSceneCommand::Line { length: *length }
        }
        None => ParticleShape2dSceneCommand::Circle { segments: 8 },
    }
}

fn particle_align_from_document(
    document: Option<ParticleAlignMode2dSceneDocument>,
) -> ParticleAlignMode2dSceneCommand {
    match document {
        Some(ParticleAlignMode2dSceneDocument::None) => ParticleAlignMode2dSceneCommand::None,
        Some(ParticleAlignMode2dSceneDocument::Emitter) => ParticleAlignMode2dSceneCommand::Emitter,
        Some(ParticleAlignMode2dSceneDocument::Random) => ParticleAlignMode2dSceneCommand::Random,
        Some(ParticleAlignMode2dSceneDocument::Velocity) | None => {
            ParticleAlignMode2dSceneCommand::Velocity
        }
    }
}

fn particle_blend_from_document(
    document: Option<ParticleBlendMode2dSceneDocument>,
) -> ParticleBlendMode2dSceneCommand {
    match document {
        Some(ParticleBlendMode2dSceneDocument::Additive) => {
            ParticleBlendMode2dSceneCommand::Additive
        }
        Some(ParticleBlendMode2dSceneDocument::Multiply) => {
            ParticleBlendMode2dSceneCommand::Multiply
        }
        Some(ParticleBlendMode2dSceneDocument::Screen) => ParticleBlendMode2dSceneCommand::Screen,
        Some(ParticleBlendMode2dSceneDocument::Alpha) | None => {
            ParticleBlendMode2dSceneCommand::Alpha
        }
    }
}

fn particle_spawn_area_from_document(
    document: Option<&ParticleSpawnArea2dSceneDocument>,
) -> ParticleSpawnArea2dSceneCommand {
    match document {
        Some(ParticleSpawnArea2dSceneDocument::Point) | None => {
            ParticleSpawnArea2dSceneCommand::Point
        }
        Some(ParticleSpawnArea2dSceneDocument::Line { length }) => {
            ParticleSpawnArea2dSceneCommand::Line { length: *length }
        }
        Some(ParticleSpawnArea2dSceneDocument::Rect { size }) => {
            ParticleSpawnArea2dSceneCommand::Rect {
                size: vec2_from_document(*size),
            }
        }
        Some(ParticleSpawnArea2dSceneDocument::Circle { radius }) => {
            ParticleSpawnArea2dSceneCommand::Circle { radius: *radius }
        }
        Some(ParticleSpawnArea2dSceneDocument::Ring {
            inner_radius,
            outer_radius,
        }) => ParticleSpawnArea2dSceneCommand::Ring {
            inner_radius: *inner_radius,
            outer_radius: *outer_radius,
        },
    }
}

fn particle_force_from_document(
    document: &ParticleForce2dSceneDocument,
) -> ParticleForce2dSceneCommand {
    match document {
        ParticleForce2dSceneDocument::Gravity { acceleration } => {
            ParticleForce2dSceneCommand::Gravity {
                acceleration: vec2_from_document(*acceleration),
            }
        }
        ParticleForce2dSceneDocument::ConstantAcceleration { acceleration } => {
            ParticleForce2dSceneCommand::ConstantAcceleration {
                acceleration: vec2_from_document(*acceleration),
            }
        }
        ParticleForce2dSceneDocument::Drag { coefficient } => ParticleForce2dSceneCommand::Drag {
            coefficient: *coefficient,
        },
        ParticleForce2dSceneDocument::Wind { velocity, strength } => {
            ParticleForce2dSceneCommand::Wind {
                velocity: vec2_from_document(*velocity),
                strength: *strength,
            }
        }
    }
}

fn required_ui_text(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    label: &str,
) -> SceneDocumentResult<String> {
    node.text
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!("expected UI node to define non-empty `{label}` content"),
        })
}

fn ui_style_from_component(
    style: &SceneUiStyleComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiStyle> {
    Ok(SceneUiStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: parse_optional_color_rgba_hex(
            style.background.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "background",
        )?,
        color: parse_optional_color_rgba_hex(
            style.color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "color",
        )?,
        border_color: parse_optional_color_rgba_hex(
            style.border_color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "border_color",
        )?,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
        align: match style.align {
            Some(SceneUiTextAlignComponentDocument::Center) => SceneUiTextAlign::Center,
            Some(SceneUiTextAlignComponentDocument::Start) | None => SceneUiTextAlign::Start,
        },
    })
}

fn ui_event_binding_from_component(
    binding: &SceneUiEventBindingComponentDocument,
) -> SceneUiEventBinding {
    SceneUiEventBinding {
        event: binding.event.clone(),
        payload: binding.payload.clone(),
    }
}

fn ui_theme_from_component(
    theme: &SceneUiThemeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiTheme> {
    Ok(SceneUiTheme {
        id: theme.id.clone(),
        palette: SceneUiThemePalette {
            background: parse_color_rgba_hex(
                &theme.palette.background,
                scene_id,
                entity_id,
                component_kind,
            )?,
            surface: parse_color_rgba_hex(
                &theme.palette.surface,
                scene_id,
                entity_id,
                component_kind,
            )?,
            surface_alt: parse_color_rgba_hex(
                &theme.palette.surface_alt,
                scene_id,
                entity_id,
                component_kind,
            )?,
            text: parse_color_rgba_hex(&theme.palette.text, scene_id, entity_id, component_kind)?,
            text_muted: parse_color_rgba_hex(
                &theme.palette.text_muted,
                scene_id,
                entity_id,
                component_kind,
            )?,
            border: parse_color_rgba_hex(
                &theme.palette.border,
                scene_id,
                entity_id,
                component_kind,
            )?,
            accent: parse_color_rgba_hex(
                &theme.palette.accent,
                scene_id,
                entity_id,
                component_kind,
            )?,
            accent_text: parse_color_rgba_hex(
                &theme.palette.accent_text,
                scene_id,
                entity_id,
                component_kind,
            )?,
            danger: parse_color_rgba_hex(
                &theme.palette.danger,
                scene_id,
                entity_id,
                component_kind,
            )?,
            warning: parse_color_rgba_hex(
                &theme.palette.warning,
                scene_id,
                entity_id,
                component_kind,
            )?,
            success: parse_color_rgba_hex(
                &theme.palette.success,
                scene_id,
                entity_id,
                component_kind,
            )?,
        },
    })
}

fn parse_optional_color_rgba_hex(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    _field_name: &str,
) -> SceneDocumentResult<Option<ColorRgba>> {
    value
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()
}

fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
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
        channel_to_f32(r),
        channel_to_f32(g),
        channel_to_f32(b),
        channel_to_f32(a),
    ))
}

fn parse_hex_channel(
    channel: &str,
    raw_value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|_| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("expected albedo color `{raw_value}` to contain only hex digits"),
    })
}

fn channel_to_f32(value: u8) -> f32 {
    f32::from(value) / 255.0
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use amigo_math::{ColorRgba, Curve1d, Transform2, Transform3, Vec2, Vec3};

    use super::{
        build_scene_hydration_plan, entity_selector_from_document, scene_key_from_document,
    };
    use crate::{
        EntitySelector, ParticleAlignMode2dSceneCommand, ParticleBlendMode2dSceneCommand,
        ParticleSpawnArea2dSceneCommand, SceneCommand, SceneEntitySelectorDocument,
        SceneEntitySelectorKindDocument, load_scene_document_from_path,
        load_scene_document_from_str,
    };

    #[test]
    fn builds_hydration_plan_for_2d_scene_document() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: sprite-lab
  label: Sprite Lab
entities:
  - id: camera
    name: playground-2d-camera
    components:
      - type: Camera2D
  - id: sprite
    name: playground-2d-sprite
    transform2:
      translation: { x: 12.0, y: -4.0 }
      rotation_radians: 0.5
      scale: { x: 2.0, y: 3.0 }
    components:
      - type: Sprite2D
        texture: playground-2d/textures/sprite-lab
        size: { x: 128.0, y: 128.0 }
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert_eq!(scene_key_from_document(&document).as_str(), "sprite-lab");
        assert_eq!(plan.commands.len(), 5);
        assert!(matches!(
            &plan.commands[0],
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { .. })
            } if name == "playground-2d-camera"
        ));
        assert!(matches!(
            &plan.commands[4],
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-sprite"
                    && command.size == Vec2::new(128.0, 128.0)
                    && command.transform == Transform2 {
                        translation: Vec2::new(12.0, -4.0),
                        rotation_radians: 0.5,
                        scale: Vec2::new(2.0, 3.0),
                    }
        ));
    }

    #[test]
    fn builds_hydration_plan_for_entity_metadata() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: metadata-preview
entities:
  - id: actor
    tags: [enemy]
    groups: [wave-1]
    visible: false
    collision_enabled: false
    properties:
      score_value: 100
      label: scout
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("metadata-preview", &document).expect("plan should build");

        assert!(matches!(
            &plan.commands[1],
            SceneCommand::ConfigureEntity {
                entity_name,
                lifecycle,
                tags,
                groups,
                properties,
            } if entity_name == "actor"
                && !lifecycle.visible
                && lifecycle.simulation_enabled
                && !lifecycle.collision_enabled
                && tags == &vec!["enemy".to_owned()]
                && groups == &vec!["wave-1".to_owned()]
                && properties.contains_key("score_value")
                && properties.contains_key("label")
        ));
    }

    #[test]
    fn converts_selector_documents_to_runtime_selectors() {
        let cases = [
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Entity,
                    value: "player".to_owned(),
                },
                EntitySelector::Entity("player".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Tag,
                    value: "enemy".to_owned(),
                },
                EntitySelector::Tag("enemy".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Group,
                    value: "wave-1".to_owned(),
                },
                EntitySelector::Group("wave-1".to_owned()),
            ),
            (
                SceneEntitySelectorDocument {
                    kind: SceneEntitySelectorKindDocument::Pool,
                    value: "bullets".to_owned(),
                },
                EntitySelector::Pool("bullets".to_owned()),
            ),
        ];

        for (document, expected) in cases {
            assert_eq!(entity_selector_from_document(&document), expected);
            assert_eq!(EntitySelector::from(document), expected);
        }
    }

    #[test]
    fn builds_hydration_plan_for_collision_event_rules() {
        let document = load_scene_document_from_str(
            r#"
version: 1
scene:
  id: collision-preview
collision_events:
  - id: projectile-hits-target
    source:
      kind: tag
      value: projectile
    target:
      kind: group
      value: targets
    event: collision.hit
    once_per_overlap: true
entities: []
"#,
        )
        .expect("scene document should parse");

        let plan =
            build_scene_hydration_plan("collision-preview", &document).expect("plan should build");

        assert!(matches!(
            &plan.commands[0],
            SceneCommand::QueueCollisionEventRule2d { command }
                if command.id == "projectile-hits-target"
                    && command.source == EntitySelector::Tag("projectile".to_owned())
                    && command.target == EntitySelector::Group("targets".to_owned())
                    && command.event == "collision.hit"
                    && command.once_per_overlap
        ));
    }

    #[test]
    fn builds_hydration_plan_for_material_scene_document() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/material-lab/scene.yml"),
        )
        .expect("material lab scene should parse");

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::SpawnNamedEntity {
                name,
                transform: Some(Transform3 { translation, scale, .. })
            } if name == "playground-3d-material-probe"
                && *translation == Vec3::ZERO
                && *scale == Vec3::ONE
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-material-probe"
                    && command.label == "debug-surface"
                    && command.albedo == ColorRgba::WHITE
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_2d_main_scene() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/hello-world-spritesheet/scene.yml"),
        )
        .expect("playground 2d main scene should parse");

        let plan =
            build_scene_hydration_plan("playground-2d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-spritesheet"
                    && command.sheet.as_ref().map(|sheet| sheet.frame_count) == Some(8)
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText2d { command }
                if command.entity_name == "playground-2d-hello"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_3d_main_scene() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-3d/scenes/hello-world-cube/scene.yml"),
        )
        .expect("playground 3d main scene should parse");

        let plan =
            build_scene_hydration_plan("playground-3d", &document).expect("plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMesh3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMaterial3d { command }
                if command.entity_name == "playground-3d-cube"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueText3d { command }
                if command.entity_name == "playground-3d-hello"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_2d_screen_space_preview() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d/scenes/screen-space-preview/scene.yml"),
        )
        .expect("screen-space preview scene should parse");

        let plan = build_scene_hydration_plan("playground-2d", &document)
            .expect("screen-space preview plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-2d-ui-preview-square"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.entity_name == "playground-2d-ui-preview"
        )));
    }

    #[test]
    fn hydrates_ui_theme_set_and_style_class() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-theme-test
  label: UI Theme Test
entities:
  - id: ui
    name: playground-hud-ui
    components:
      - type: UiThemeSet
        active: space_dark
        themes:
          - id: space_dark
            palette:
              background: "#050812FF"
              surface: "#101827DD"
              surface_alt: "#172033DD"
              text: "#EAF6FFFF"
              text_muted: "#89A2B7FF"
              border: "#2A6F9EFF"
              accent: "#39D7FFFF"
              accent_text: "#001018FF"
              danger: "#FF4D6DFF"
              warning: "#FFB000FF"
              success: "#5CFF9CFF"
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: column
          id: root
          style_class: root
          children:
            - type: text
              id: title
              text: THEMED
              style_class: text_title
"#####,
        )
        .expect("ui theme scene should parse");
        let plan = build_scene_hydration_plan("playground-hud-ui", &document)
            .expect("ui theme plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUiThemeSet { command }
                if command.entity_name == "playground-hud-ui"
                    && command.active.as_deref() == Some("space_dark")
                    && command.themes.len() == 1
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.document.root.style_class.as_deref() == Some("root")
                    && command.document.root.children[0].style_class.as_deref() == Some("text_title")
        )));
    }

    #[test]
    fn hydrates_native_tab_view_and_group_box() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-native-controls
entities:
  - id: ui
    name: ui
    components:
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: tab-view
          id: tabs
          selected: settings
          tabs:
            - id: overview
              label: Overview
            - id: settings
              label: Settings
          on_change:
            event: ui.tab.changed
          children:
            - type: group-box
              id: overview
              text: Overview Group
            - type: group-box
              id: settings
              text: Settings Group
"#####,
        )
        .expect("native ui control scene should parse");
        let plan = build_scene_hydration_plan("test", &document)
            .expect("native ui control scene should hydrate");

        let ui = plan
            .commands
            .iter()
            .find_map(|command| match command {
                SceneCommand::QueueUi { command } => Some(command),
                _ => None,
            })
            .expect("ui command should be queued");
        match &ui.document.root.kind {
            crate::SceneUiNodeKind::TabView { selected, tabs, .. } => {
                assert_eq!(selected, "settings");
                assert_eq!(tabs.len(), 2);
            }
            _ => panic!("expected tab view root"),
        }
        assert!(matches!(
            ui.document.root.children[0].kind,
            crate::SceneUiNodeKind::GroupBox { .. }
        ));
    }

    #[test]
    fn hydrates_curve_editor_points_and_legacy_values() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: ui-curve-editor
entities:
  - id: ui
    name: ui
    components:
      - type: UiDocument
        target:
          type: screen-space
          layer: hud
        root:
          type: column
          id: root
          children:
            - type: curve-editor
              id: curve
              points:
                - { t: -1.0, value: 0.25 }
                - { t: 0.5, value: 2.0 }
              on_change:
                event: ui.curve.changed
"#####,
        )
        .expect("curve editor scene should parse");
        let plan = build_scene_hydration_plan("test", &document)
            .expect("curve editor scene should hydrate");
        let ui = plan
            .commands
            .iter()
            .find_map(|command| match command {
                SceneCommand::QueueUi { command } => Some(command),
                _ => None,
            })
            .expect("ui command should be queued");
        let curve = &ui.document.root.children[0];

        match &curve.kind {
            crate::SceneUiNodeKind::CurveEditor { points } => {
                assert_eq!(points.len(), 4);
                assert_eq!(points[0].t, 0.0);
                assert_eq!(points[1].value, 1.0);
            }
            _ => panic!("expected curve editor"),
        }
        assert_eq!(
            curve
                .on_change
                .as_ref()
                .map(|binding| binding.event.as_str()),
            Some("ui.curve.changed")
        );
    }

    #[test]
    fn builds_hydration_plan_for_hud_ui_showcase() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-hud-ui/scenes/showcase/scene.yml"),
        )
        .expect("hud ui showcase scene should parse");
        let plan = build_scene_hydration_plan("playground-hud-ui", &document)
            .expect("hud ui showcase plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUiThemeSet { command }
                if command.active.as_deref() == Some("space_dark") && command.themes.len() == 2
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueUi { command }
                if command.entity_name == "playground-hud-ui-showcase"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_playground_2d_asteroids_game() {
        let workspace_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .and_then(|path| path.parent())
            .and_then(|path| path.parent())
            .expect("workspace root should exist")
            .to_path_buf();

        let document = load_scene_document_from_path(
            workspace_root.join("mods/playground-2d-asteroids/scenes/game/scene.yml"),
        )
        .expect("asteroids game scene should parse");

        let plan = build_scene_hydration_plan("playground-2d-asteroids", &document)
            .expect("asteroids game plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueVectorShape2d { command }
                if command.entity_name == "playground-2d-asteroids-ship"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueEntityPool { command } if command.pool == "asteroids"
        )));
    }

    #[test]
    fn builds_hydration_plan_for_sidescroller_components() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: vertical-slice
  label: Vertical Slice
entities:
  - id: camera
    name: playground-sidescroller-camera
    components:
      - type: Camera2D
      - type: CameraFollow2D
        target: playground-sidescroller-player
  - id: tilemap
    name: playground-sidescroller-tilemap
    components:
      - type: TileMap2D
        tileset: playground-sidescroller/tilesets/platformer
        ruleset: playground-sidescroller/tilesets/platformer-rules
        tile_size: { x: 16.0, y: 16.0 }
        grid:
          - "...."
          - ".P.."
          - "####"
  - id: player
    name: playground-sidescroller-player
    components:
      - type: TileMapMarker2D
        tilemap_entity: playground-sidescroller-tilemap
        symbol: "P"
        offset: { x: 0.0, y: 8.0 }
      - type: KinematicBody2D
        velocity: { x: 0.0, y: 0.0 }
        gravity_scale: 1.0
        terminal_velocity: 720.0
      - type: AabbCollider2D
        size: { x: 20.0, y: 30.0 }
        offset: { x: 0.0, y: 1.0 }
        layer: player
        mask: [world, trigger]
      - type: MotionController2D
        max_speed: 180.0
        acceleration: 900.0
        deceleration: 1200.0
        air_acceleration: 500.0
        gravity: 900.0
        jump_velocity: -360.0
        terminal_velocity: 720.0
  - id: coin
    name: playground-sidescroller-coin
    components:
      - type: Sprite2D
        texture: playground-sidescroller/textures/coin
        size: { x: 16.0, y: 16.0 }
        animation:
          fps: 10.0
          looping: true
      - type: Trigger2D
        size: { x: 16.0, y: 16.0 }
        layer: trigger
        mask: [player]
        event: coin.collected
"#####,
        )
        .expect("sidescroller scene should parse");

        let plan = build_scene_hydration_plan("playground-sidescroller", &document)
            .expect("sidescroller hydration plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMap2d { command }
                if command.entity_name == "playground-sidescroller-tilemap"
                    && command.ruleset.as_ref().map(|ruleset| ruleset.as_str())
                        == Some("playground-sidescroller/tilesets/platformer-rules")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueKinematicBody2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueAabbCollider2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueMotionController2d { command }
                if command.entity_name == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTileMapMarker2d { command }
                if command.entity_name == "playground-sidescroller-player"
                    && command.symbol == "P"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueCameraFollow2d { command }
                if command.entity_name == "playground-sidescroller-camera"
                    && command.target == "playground-sidescroller-player"
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueTrigger2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.event.as_deref() == Some("coin.collected")
        )));
        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueSprite2d { command }
                if command.entity_name == "playground-sidescroller-coin"
                    && command.animation.as_ref().and_then(|animation| animation.fps) == Some(10.0)
                    && command.animation.as_ref().and_then(|animation| animation.looping) == Some(true)
        )));
    }

    #[test]
    fn hydrates_freeflight_motion_response_curves() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: curve-motion
entities:
  - id: ship
    name: curve-ship
    components:
      - type: FreeflightMotion2D
        thrust_acceleration: 100.0
        reverse_acceleration: 50.0
        strafe_acceleration: 20.0
        turn_acceleration: 8.0
        linear_damping: 2.0
        turn_damping: 3.0
        max_speed: 300.0
        max_angular_speed: 4.0
        thrust_response_curve:
          kind: ease_out
        reverse_response_curve:
          kind: ease_in
        strafe_response_curve:
          kind: constant
          value: 0.5
        turn_response_curve:
          kind: smooth_step
"#####,
        )
        .expect("freeflight curve scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("freeflight curve hydration plan should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueFreeflightMotion2d { command }
                if command.entity_name == "curve-ship"
                    && command.thrust_response_curve == Curve1d::EaseOut
                    && command.reverse_response_curve == Curve1d::EaseIn
                    && command.strafe_response_curve == Curve1d::Constant(0.5)
                    && command.turn_response_curve == Curve1d::SmoothStep
        )));
    }

    #[test]
    fn hydrates_particle_emitter_2d_command() {
        let document = load_scene_document_from_str(
            r#####"
version: 1
scene:
  id: particle-scene
entities:
  - id: thruster
    name: test-thruster
    components:
      - type: ParticleEmitter2D
        attached_to: test-ship
        local_offset: { x: -12.0, y: 1.0 }
        local_direction_degrees: 180.0
        spawn_area:
          kind: rect
          size: { x: 120.0, y: 20.0 }
        active: false
        spawn_rate: 90.0
        max_particles: 64
        particle_lifetime: 0.5
        initial_speed: 120.0
        initial_size: 2.0
        final_size: 8.0
        color: "#FFFFFFFF"
        shape:
          kind: circle
          segments: 8
        shape_choices:
          - weight: 2.0
            shape: { kind: circle, segments: 8 }
          - weight: 1.0
            shape: { kind: line, length: 14.0 }
        shape_over_lifetime:
          - t: 0.0
            shape: { kind: quad }
          - t: 0.75
            shape: { kind: circle, segments: 12 }
        align: emitter
        blend_mode: additive
        motion_stretch:
          enabled: true
          velocity_scale: 2.2
          max_length: 96.0
        emission_rate_curve:
          kind: ease_out
        forces:
          - kind: gravity
            acceleration: { x: 0.0, y: -480.0 }
          - kind: drag
            coefficient: 1.8
"#####,
        )
        .expect("particle scene should parse");

        let plan = build_scene_hydration_plan("test-mod", &document)
            .expect("particle scene hydration should build");

        assert!(plan.commands.iter().any(|command| matches!(
            command,
            SceneCommand::QueueParticleEmitter2d { command }
                if command.entity_name == "test-thruster"
                    && command.attached_to.as_deref() == Some("test-ship")
                    && command.spawn_rate == 90.0
                    && command.max_particles == 64
                    && command.emission_rate_curve == Curve1d::EaseOut
                    && command.shape_choices.len() == 2
                    && command.shape_over_lifetime.len() == 2
                    && command.align == ParticleAlignMode2dSceneCommand::Emitter
                    && command.blend_mode == ParticleBlendMode2dSceneCommand::Additive
                    && command.motion_stretch.is_some_and(|stretch| stretch.enabled && stretch.velocity_scale == 2.2 && stretch.max_length == 96.0)
                    && matches!(command.spawn_area, ParticleSpawnArea2dSceneCommand::Rect { size } if size == Vec2::new(120.0, 20.0))
                    && command.forces.len() == 2
        )));
    }
}
