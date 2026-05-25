use crate::document::SceneComponentDocument as DomainComponentDocument;

fn hydrate_component_domains(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &String,
    component: &SceneComponentDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    match component {
        DomainComponentDocument::ParticleEmitter2d {
            render_layer,
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
            size_jitter,
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
            post_fx: _,
        } => {
            commands.push(SceneCommand::Plugin {
                command: particle_emitter_2d_plugin_scene_command(ParticleEmitter2dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    render_layer: render_layer.clone(),
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
                    simulation_space: particle_simulation_space_from_document(*simulation_space),
                    initial_size: *initial_size,
                    final_size: *final_size,
                    size_jitter: size_jitter.max(0.0),
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
                            shutter_seconds: motion_stretch.shutter_seconds.max(0.0),
                            tail_alpha: motion_stretch.tail_alpha.clamp(0.0, 1.0),
                            head_alpha: motion_stretch.head_alpha.clamp(0.0, 1.0),
                        }
                    }),
                    material: material
                        .as_ref()
                        .map(|material| crate::ParticleMaterial2dSceneCommand {
                            lighting_mode: particle_lighting_mode_from_document(
                                material.lighting_mode,
                                material.receives_light,
                                material.light_receiver.as_ref(),
                            ),
                            receives_light: material.receives_light,
                            light_response: material.light_response.max(0.0),
                            light_receiver: material
                                .light_receiver
                                .as_ref()
                                .map(light_receiver_binding_from_document),
                        })
                        .unwrap_or(crate::ParticleMaterial2dSceneCommand {
                            lighting_mode: Material2dLightingModeSceneCommand::Unlit,
                            receives_light: false,
                            light_response: 1.0,
                            light_receiver: None,
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
                }),
            });
        }
        DomainComponentDocument::Velocity2d { velocity } => {
            commands.push(SceneCommand::Plugin {
                command: velocity_2d_plugin_scene_command(Velocity2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec2_from_document(*velocity),
                )),
            });
        }
        DomainComponentDocument::Bounds2d {
            min,
            max,
            behavior,
            restitution,
        } => {
            commands.push(SceneCommand::Plugin {
                command: bounds_2d_plugin_scene_command(Bounds2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec2_from_document(*min),
                    vec2_from_document(*max),
                    bounds_behavior_from_document(*behavior, *restitution),
                )),
            });
        }
        DomainComponentDocument::FreeflightMotion2d {
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
            commands.push(SceneCommand::Plugin {
                command: freeflight_motion_2d_plugin_scene_command(
                    FreeflightMotion2dSceneCommand::new(
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
                ),
            });
        }
        DomainComponentDocument::KinematicBody2d {
            velocity,
            gravity_scale,
            terminal_velocity,
        } => {
            commands.push(SceneCommand::Plugin {
                command: kinematic_body_2d_plugin_scene_command(KinematicBody2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec2_from_document(*velocity),
                    *gravity_scale,
                    *terminal_velocity,
                )),
            });
        }
        DomainComponentDocument::AabbCollider2d {
            size,
            offset,
            layer,
            mask,
        } => {
            commands.push(SceneCommand::Plugin {
                command: aabb_collider_2d_plugin_scene_command(AabbCollider2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec2_from_document(*size),
                    vec2_from_document(*offset),
                    layer.clone(),
                    mask.clone(),
                )),
            });
        }
        DomainComponentDocument::StaticCollider2d {
            size,
            offset,
            layer,
        } => {
            let transform = transform2_for_entity(entity);
            let offset = vec2_from_document(*offset);
            commands.push(SceneCommand::Plugin {
                command: static_collider_2d_plugin_scene_command(
                    StaticCollider2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        vec2_from_document(*size),
                        amigo_math::Vec2::new(
                            transform.translation.x + offset.x,
                            transform.translation.y + offset.y,
                        ),
                        layer.clone(),
                    ),
                ),
            });
        }
        DomainComponentDocument::CircleCollider2d { radius, offset } => {
            commands.push(SceneCommand::Plugin {
                command: circle_collider_2d_plugin_scene_command(
                    CircleCollider2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        (*radius).max(0.0),
                        vec2_from_document(*offset),
                    ),
                ),
            });
        }
        DomainComponentDocument::Trigger2d {
            size,
            offset,
            layer,
            mask,
            event,
        } => {
            commands.push(SceneCommand::Plugin {
                command: trigger_2d_plugin_scene_command(Trigger2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec2_from_document(*size),
                    vec2_from_document(*offset),
                    layer.clone(),
                    mask.clone(),
                    event.clone(),
                )),
            });
        }
        DomainComponentDocument::MotionController2d {
            max_speed,
            acceleration,
            deceleration,
            air_acceleration,
            gravity,
            jump_velocity,
            terminal_velocity,
        } => {
            commands.push(SceneCommand::Plugin {
                command: motion_controller_2d_plugin_scene_command(
                    MotionController2dSceneCommand::new(
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
                ),
            });
        }
        DomainComponentDocument::CameraFollow2d {
            target,
            offset,
            lerp,
            lookahead_velocity_scale,
            lookahead_max_distance,
            sway_amount,
            sway_frequency,
        } => {
            commands.push(SceneCommand::Plugin {
                command: camera_follow_2d_plugin_scene_command(
                    CameraFollow2dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        target.clone(),
                        vec2_from_document(*offset),
                        *lerp,
                    )
                    .with_lookahead(*lookahead_velocity_scale, *lookahead_max_distance)
                    .with_sway(*sway_amount, *sway_frequency),
                ),
            });
        }
        DomainComponentDocument::Parallax2d { camera, factor } => {
            commands.push(SceneCommand::Plugin {
                command: parallax_2d_plugin_scene_command(Parallax2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    camera.clone(),
                    vec2_from_document(*factor),
                    transform2_for_entity(entity).translation,
                )),
            });
        }
        DomainComponentDocument::TileMapMarker2d {
            symbol,
            tilemap_entity,
            index,
            offset,
        } => {
            commands.push(SceneCommand::Plugin {
                command: tilemap_marker_2d_plugin_scene_command(TileMapMarker2dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    tilemap_entity.clone(),
                    symbol.clone(),
                    *index,
                    vec2_from_document(*offset),
                )),
            });
        }
        DomainComponentDocument::Mesh3d { mesh } => {
            commands.push(SceneCommand::Plugin {
                command: mesh_3d_plugin_scene_command(Mesh3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    mesh_asset: AssetKey::new(mesh.clone()),
                    transform: transform3_for_entity(entity),
                }),
            });
        }
        DomainComponentDocument::Material3d {
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
                command.albedo =
                    parse_color_rgba_hex(albedo, &document.scene.id, &entity.id, component.kind())?;
            }

            commands.push(SceneCommand::Plugin {
                command: material_3d_plugin_scene_command(command),
            });
        }
        DomainComponentDocument::Text3d {
            content,
            font,
            size,
        } => {
            commands.push(SceneCommand::Plugin {
                command: text_3d_plugin_scene_command(Text3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    content: content.clone(),
                    font: AssetKey::new(font.clone()),
                    size: *size,
                    transform: transform3_for_entity(entity),
                }),
            });
        }
        DomainComponentDocument::UiDocument { target, root } => {
            commands.push(SceneCommand::Plugin {
                command: ui_document_plugin_scene_command(UiSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    document: ui_document_from_component(
                        target,
                        root,
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?,
                }),
            });
        }
        DomainComponentDocument::UiThemeSet { active, themes } => {
            commands.push(SceneCommand::Plugin {
                command: ui_theme_set_plugin_scene_command(UiThemeSetSceneCommand {
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
                }),
            });
        }
        _ => return Ok(false),
    }
    Ok(true)
}

fn light_receiver_binding_from_document(
    binding: &LightReceiver2dBindingSceneDocument,
) -> LightReceiver2dBindingSceneCommand {
    LightReceiver2dBindingSceneCommand {
        groups: binding.groups.clone(),
        source: binding.source.clone(),
        channel: binding.channel.clone(),
        sample_strategy: light_sample_strategy_from_document(binding.sample_strategy),
        sample_points: binding.sample_points.clamp(1, 9),
        radius_px: binding.radius_px.max(0.0),
        exposure: binding.exposure.max(0.0),
        dark_policy: light_receiver_dark_policy_from_document(binding.dark_policy),
        global_lights: binding
            .global_lights
            .iter()
            .map(light_receiver_global_light_from_document)
            .collect(),
    }
}

fn particle_lighting_mode_from_document(
    explicit: Option<Material2dLightingModeSceneDocument>,
    receives_light: bool,
    receiver: Option<&LightReceiver2dBindingSceneDocument>,
) -> Material2dLightingModeSceneCommand {
    if let Some(mode) = explicit {
        return match mode {
            Material2dLightingModeSceneDocument::Unlit => Material2dLightingModeSceneCommand::Unlit,
            Material2dLightingModeSceneDocument::DynamicLights => {
                Material2dLightingModeSceneCommand::DynamicLights
            }
            Material2dLightingModeSceneDocument::LightmapSampled => {
                Material2dLightingModeSceneCommand::LightMapSampled
            }
            Material2dLightingModeSceneDocument::LightGroupSampled => {
                Material2dLightingModeSceneCommand::LightGroupSampled
            }
        };
    }

    let Some(receiver) = receiver else {
        return if receives_light {
            Material2dLightingModeSceneCommand::DynamicLights
        } else {
            Material2dLightingModeSceneCommand::Unlit
        };
    };

    if !receiver.groups.is_empty() {
        Material2dLightingModeSceneCommand::LightGroupSampled
    } else if !receiver.source.is_empty() && !receiver.channel.is_empty() {
        Material2dLightingModeSceneCommand::LightMapSampled
    } else if receives_light {
        Material2dLightingModeSceneCommand::DynamicLights
    } else {
        Material2dLightingModeSceneCommand::Unlit
    }
}

fn light_sample_strategy_from_document(
    strategy: LightSampleStrategy2dSceneDocument,
) -> LightSampleStrategy2dSceneCommand {
    match strategy {
        LightSampleStrategy2dSceneDocument::Point => LightSampleStrategy2dSceneCommand::Point,
        LightSampleStrategy2dSceneDocument::Line => LightSampleStrategy2dSceneCommand::Line,
    }
}

fn light_receiver_dark_policy_from_document(
    policy: LightReceiverDarkPolicy2dSceneDocument,
) -> LightReceiverDarkPolicy2dSceneCommand {
    match policy {
        LightReceiverDarkPolicy2dSceneDocument::Transparent => {
            LightReceiverDarkPolicy2dSceneCommand::Transparent
        }
        LightReceiverDarkPolicy2dSceneDocument::BaseColor => {
            LightReceiverDarkPolicy2dSceneCommand::BaseColor
        }
        LightReceiverDarkPolicy2dSceneDocument::ShadowTint => {
            LightReceiverDarkPolicy2dSceneCommand::ShadowTint
        }
    }
}

fn light_receiver_global_light_from_document(
    global_light: &LightReceiverGlobalLight2dSceneDocument,
) -> LightReceiverGlobalLight2dSceneCommand {
    LightReceiverGlobalLight2dSceneCommand {
        id: global_light.id.clone(),
        response: global_light.response.max(0.0),
    }
}
