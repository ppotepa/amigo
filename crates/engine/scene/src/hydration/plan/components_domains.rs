fn hydrate_component_domains(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &String,
    component: &SceneComponentDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    match component {
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
                    velocity_mode,
                    simulation_space,
                    initial_size,
                    final_size,
                    color,
                    color_ramp,
                    z_index,
                    shape,
                    shape_choices,
                    shape_over_lifetime,
                    line_anchor,
                    align,
                    blend_mode,
                    motion_stretch,
                    material,
                    light,
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
                            velocity_mode: particle_velocity_mode_from_document(*velocity_mode),
                            simulation_space: particle_simulation_space_from_document(
                                *simulation_space,
                            ),
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
                            line_anchor: particle_line_anchor_from_document(*line_anchor),
                            align: particle_align_from_document(*align),
                            blend_mode: particle_blend_from_document(*blend_mode),
                            motion_stretch: motion_stretch.map(|motion_stretch| {
                                ParticleMotionStretch2dSceneCommand {
                                    enabled: motion_stretch.enabled,
                                    velocity_scale: motion_stretch.velocity_scale.max(0.0),
                                    max_length: motion_stretch.max_length.max(0.0),
                                }
                            }),
                            material: material
                                .map(|material| crate::ParticleMaterial2dSceneCommand {
                                    receives_light: material.receives_light,
                                    light_response: material.light_response.max(0.0),
                                })
                                .unwrap_or(crate::ParticleMaterial2dSceneCommand {
                                    receives_light: false,
                                    light_response: 1.0,
                                }),
                            light: light.map(|light| crate::ParticleLight2dSceneCommand {
                                radius: light.radius.max(0.0),
                                intensity: light.intensity.max(0.0),
                                mode: particle_light_mode_from_document(light.mode),
                                glow: light.glow,
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
                SceneComponentDocument::StaticCollider2d {
                    size,
                    offset,
                    layer,
                } => {
                    let transform = transform2_for_entity(entity);
                    let offset = vec2_from_document(*offset);
                    commands.push(SceneCommand::QueueStaticCollider2d {
                        command: StaticCollider2dSceneCommand::new(
                            source_mod.to_owned(),
                            entity_name.clone(),
                            vec2_from_document(*size),
                            amigo_math::Vec2::new(
                                transform.translation.x + offset.x,
                                transform.translation.y + offset.y,
                            ),
                            layer.clone(),
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
        _ => return Ok(false),
    }
    Ok(true)
}
