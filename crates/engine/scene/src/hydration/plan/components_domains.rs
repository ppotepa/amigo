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
        DomainComponentDocument::CameraController3d {
            camera,
            mode,
            switch_action,
            orbit_target,
            orbit_distance,
            orbit_min_distance,
            orbit_max_distance,
            orbit_yaw,
            orbit_pitch,
            orbit_sensitivity,
            orbit_zoom_speed,
            freelook_speed,
            freelook_sensitivity,
            move_forward_action,
            move_strafe_action,
            move_lift_action,
        } => {
            commands.push(SceneCommand::Plugin {
                command: camera_controller_3d_plugin_scene_command(CameraController3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    camera: camera.clone(),
                    mode: camera_controller_3d_mode_from_document(
                        mode,
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?,
                    switch_action: switch_action.clone(),
                    orbit_target: orbit_target.clone(),
                    orbit_distance: *orbit_distance,
                    orbit_min_distance: *orbit_min_distance,
                    orbit_max_distance: *orbit_max_distance,
                    orbit_yaw: *orbit_yaw,
                    orbit_pitch: *orbit_pitch,
                    orbit_sensitivity: *orbit_sensitivity,
                    orbit_zoom_speed: *orbit_zoom_speed,
                    freelook_speed: *freelook_speed,
                    freelook_sensitivity: *freelook_sensitivity,
                    move_forward_action: move_forward_action.clone(),
                    move_strafe_action: move_strafe_action.clone(),
                    move_lift_action: move_lift_action.clone(),
                }),
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
        DomainComponentDocument::Mesh3d { mesh, npr } => {
            commands.push(SceneCommand::Plugin {
                command: mesh_3d_plugin_scene_command(Mesh3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    mesh_asset: AssetKey::new(mesh.clone()),
                    transform: transform3_for_entity(entity),
                    npr: npr_line_settings_3d_from_document(
                        npr.as_ref(),
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?,
                }),
            });
        }
        DomainComponentDocument::Material3d {
            label,
            source,
            albedo,
            render_order,
            shading,
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
            command.render_order = *render_order;
            command.shading = material_3d_shading_from_document(
                shading.as_deref(),
                &document.scene.id,
                &entity.id,
                component.kind(),
            )?;

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
        DomainComponentDocument::PhysicsWorld3d {
            gravity,
            substeps,
            solver_iterations,
            ccd_substeps,
        } => {
            commands.push(SceneCommand::Plugin {
                command: physics_world_3d_plugin_scene_command(PhysicsWorld3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    gravity: vec3_from_document(*gravity),
                    substeps: *substeps,
                    solver_iterations: *solver_iterations,
                    ccd_substeps: *ccd_substeps,
                }),
            });
        }
        DomainComponentDocument::RigidBody3d {
            velocity,
            angular_velocity,
            mass,
            linear_damping,
            angular_damping,
            gravity_scale,
            restitution,
            friction,
            ccd,
        } => {
            commands.push(SceneCommand::Plugin {
                command: rigid_body_3d_plugin_scene_command(
                    RigidBody3dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        vec3_from_document(*velocity),
                        *gravity_scale,
                        *restitution,
                    )
                    .with_angular(vec3_from_document(*angular_velocity), *angular_damping)
                    .with_physical_properties(
                        *mass,
                        *linear_damping,
                        *friction,
                        *ccd,
                    ),
                ),
            });
        }
        DomainComponentDocument::BoxCollider3d { size, offset } => {
            commands.push(SceneCommand::Plugin {
                command: box_collider_3d_plugin_scene_command(BoxCollider3dSceneCommand::new(
                    source_mod.to_owned(),
                    entity_name.clone(),
                    vec3_from_document(*size),
                    vec3_from_document(*offset),
                )),
            });
        }
        DomainComponentDocument::StaticBoxCollider3d {
            size,
            offset,
            friction,
            restitution,
        } => {
            let transform = transform3_for_entity(entity);
            let offset = vec3_from_document(*offset);
            commands.push(SceneCommand::Plugin {
                command: static_box_collider_3d_plugin_scene_command(
                    StaticBoxCollider3dSceneCommand::new(
                        source_mod.to_owned(),
                        entity_name.clone(),
                        vec3_from_document(*size),
                        amigo_math::Vec3::new(
                            transform.translation.x + offset.x,
                            transform.translation.y + offset.y,
                            transform.translation.z + offset.z,
                        ),
                    )
                    .with_surface(*friction, *restitution),
                ),
            });
        }
        DomainComponentDocument::PhysicsSpawner3d {
            entity_prefix,
            mesh,
            material,
            material_label,
            interval_seconds,
            origin,
            spawn_scale,
            grid_spacing,
            initial_velocity,
            angular_velocity,
            spawn_position_jitter,
            spawn_rotation_jitter,
            initial_velocity_jitter,
            angular_velocity_jitter,
            mass,
            linear_damping,
            angular_damping,
            gravity_scale,
            restitution,
            friction,
            ccd,
            collider_size,
            max_alive,
            counter_entity,
            counter_prefix,
            counter_font,
            counter_size,
            counter_position,
        } => {
            commands.push(SceneCommand::Plugin {
                command: physics_spawner_3d_plugin_scene_command(PhysicsSpawner3dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    entity_prefix: entity_prefix.clone(),
                    mesh: AssetKey::new(mesh.clone()),
                    material: AssetKey::new(material.clone()),
                    material_label: material_label
                        .clone()
                        .unwrap_or_else(|| "cube-material".to_owned()),
                    interval_seconds: *interval_seconds,
                    origin: vec3_from_document(*origin),
                    spawn_scale: vec3_from_document(*spawn_scale),
                    grid_spacing: vec3_from_document(*grid_spacing),
                    initial_velocity: vec3_from_document(*initial_velocity),
                    angular_velocity: vec3_from_document(*angular_velocity),
                    spawn_position_jitter: vec3_from_document(*spawn_position_jitter),
                    spawn_rotation_jitter: vec3_from_document(*spawn_rotation_jitter),
                    initial_velocity_jitter: vec3_from_document(*initial_velocity_jitter),
                    angular_velocity_jitter: vec3_from_document(*angular_velocity_jitter),
                    mass: *mass,
                    linear_damping: *linear_damping,
                    angular_damping: *angular_damping,
                    gravity_scale: *gravity_scale,
                    restitution: *restitution,
                    friction: *friction,
                    ccd: *ccd,
                    collider_size: vec3_from_document(*collider_size),
                    max_alive: *max_alive,
                    counter_entity: counter_entity.clone(),
                    counter_prefix: counter_prefix.clone(),
                    counter_font: counter_font.as_ref().map(AssetKey::new),
                    counter_size: *counter_size,
                    counter_position: vec3_from_document(*counter_position),
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

fn npr_line_settings_3d_from_document(
    document: Option<&crate::document::NprLine3dDocument>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<Option<amigo_render_api::NprLineSettings3d>> {
    let Some(document) = document else {
        return Ok(None);
    };

    match document {
        crate::document::NprLine3dDocument::Enabled(enabled) => {
            Ok(enabled.then(amigo_render_api::NprLineSettings3d::default))
        }
        crate::document::NprLine3dDocument::Settings(settings) => {
            if !settings.enabled {
                return Ok(None);
            }

            Ok(Some(amigo_render_api::NprLineSettings3d {
                boundary: settings.boundary,
                silhouette: settings.silhouette,
                feature: settings.feature,
                feature_angle_degrees: settings.feature_angle_degrees,
                min_screen_length_px: settings.min_screen_length_px,
                ink_color: parse_color_rgba_hex(
                    &settings.ink_color,
                    scene_id,
                    entity_id,
                    component_kind,
                )?,
                width_px: settings.width_px,
                width_jitter_px: settings.width_jitter_px,
                path_jitter_px: settings.path_jitter_px,
                endpoint_quant_px: settings.endpoint_quant_px,
                path_simplify_px: settings.path_simplify_px,
                taper: settings.taper,
                overshoot_px: settings.overshoot_px,
                dropout: settings.dropout,
                passes: settings.passes,
                seed: settings.seed,
            }))
        }
    }
}

fn material_3d_shading_from_document(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::Material3dShadingMode> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(amigo_render_api::Material3dShadingMode::Lit);
    };

    match value {
        "lit" => Ok(amigo_render_api::Material3dShadingMode::Lit),
        "unlit" => Ok(amigo_render_api::Material3dShadingMode::Unlit),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!("expected Material3D shading to be `lit` or `unlit`, got `{other}`"),
        }),
    }
}

fn camera_controller_3d_mode_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<CameraController3dModeSceneCommand> {
    match value.trim() {
        "orbit" => Ok(CameraController3dModeSceneCommand::Orbit),
        "freelook" => Ok(CameraController3dModeSceneCommand::Freelook),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "expected CameraController3D mode to be `orbit` or `freelook`, got `{other}`"
            ),
        }),
    }
}
