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
                Ok(None)
            } else {
                npr_line_settings_3d_from_settings_document(
                    settings,
                    scene_id,
                    entity_id,
                    component_kind,
                )
                .map(Some)
            }
        }
    }
}

fn npr_line_settings_3d_from_settings_document(
    settings: &crate::document::NprLine3dSettingsDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprLineSettings3d> {
    let mut resolved = npr_style_preset_3d_from_document(
        settings.style_preset.as_deref(),
        scene_id,
        entity_id,
        component_kind,
    )
    .map(amigo_render_api::NprLineSettings3d::from_preset)
    .unwrap_or_default();

    apply_grouped_npr_line_settings_3d(settings, &mut resolved, scene_id, entity_id, component_kind)?;

    if let Some(strategy) = settings.strategy.as_deref() {
        resolved.render_strategy =
            npr_render_strategy_3d_from_document(strategy, scene_id, entity_id, component_kind)?;
    }
    if let Some(fill_mode) = settings.fill_mode.as_deref() {
        resolved.fill_mode =
            npr_fill_mode_3d_from_document(fill_mode, scene_id, entity_id, component_kind)?;
    }
    if let Some(stroke_tool) = settings.stroke_tool.as_deref() {
        resolved.stroke_tool =
            npr_stroke_tool_3d_from_document(stroke_tool, scene_id, entity_id, component_kind)?;
    }
    if let Some(boundary) = settings.boundary {
        resolved.boundary = boundary;
    }
    if let Some(silhouette) = settings.silhouette {
        resolved.silhouette = silhouette;
    }
    if let Some(feature) = settings.feature {
        resolved.feature = feature;
    }
    if let Some(suggestive) = settings.suggestive {
        resolved.suggestive = suggestive;
    }
    if let Some(contact) = settings.contact {
        resolved.contact = contact;
    }
    if let Some(contact_ground_y) = settings.contact_ground_y {
        resolved.contact_ground_y = contact_ground_y;
    }
    if let Some(contact_threshold) = settings.contact_threshold {
        resolved.contact_threshold = contact_threshold;
    }
    if let Some(feature_angle_degrees) = settings.feature_angle_degrees {
        resolved.feature_angle_degrees = feature_angle_degrees;
    }
    if let Some(min_screen_length_px) = settings.min_screen_length_px {
        resolved.min_screen_length_px = min_screen_length_px;
    }
    if let Some(ink_color) = settings.ink_color.as_deref() {
        resolved.ink_color = parse_color_rgba_hex(ink_color, scene_id, entity_id, component_kind)?;
    }
    if let Some(humanization) = settings.humanization {
        resolved.humanization = humanization;
    }
    if let Some(line_confidence) = settings.line_confidence {
        resolved.line_confidence = line_confidence;
    }
    if let Some(temporal_stability) = settings.temporal_stability {
        resolved.temporal_stability = temporal_stability;
    }
    if let Some(temporal_path_smoothing) = settings.temporal_path_smoothing {
        resolved.temporal_path_smoothing = temporal_path_smoothing;
    }
    if let Some(visibility_hysteresis_frames) = settings.visibility_hysteresis_frames {
        resolved.visibility_hysteresis_frames = visibility_hysteresis_frames;
    }
    if let Some(visibility_max_dimension_px) = settings.visibility_max_dimension_px {
        resolved.visibility_max_dimension_px = visibility_max_dimension_px;
    }
    if let Some(width_px) = settings.width_px {
        resolved.width_px = width_px;
    }
    if let Some(tool_width_multiplier) = settings.tool_width_multiplier {
        resolved.tool_width_multiplier = tool_width_multiplier;
    }
    if let Some(tool_alpha_multiplier) = settings.tool_alpha_multiplier {
        resolved.tool_alpha_multiplier = tool_alpha_multiplier;
    }
    if let Some(tool_wobble_multiplier) = settings.tool_wobble_multiplier {
        resolved.tool_wobble_multiplier = tool_wobble_multiplier;
    }
    if let Some(tool_pressure_jitter_multiplier) = settings.tool_pressure_jitter_multiplier {
        resolved.tool_pressure_jitter_multiplier = tool_pressure_jitter_multiplier;
    }
    if let Some(tool_dropout_multiplier) = settings.tool_dropout_multiplier {
        resolved.tool_dropout_multiplier = tool_dropout_multiplier;
    }
    if let Some(tool_search_multiplier) = settings.tool_search_multiplier {
        resolved.tool_search_multiplier = tool_search_multiplier;
    }
    if let Some(silhouette_width_multiplier) = settings.silhouette_width_multiplier {
        resolved.silhouette_width_multiplier = silhouette_width_multiplier;
    }
    if let Some(boundary_width_multiplier) = settings.boundary_width_multiplier {
        resolved.boundary_width_multiplier = boundary_width_multiplier;
    }
    if let Some(feature_width_multiplier) = settings.feature_width_multiplier {
        resolved.feature_width_multiplier = feature_width_multiplier;
    }
    if let Some(distance_width_falloff) = settings.distance_width_falloff {
        resolved.distance_width_falloff = distance_width_falloff;
    }
    if let Some(depth_pressure) = settings.depth_pressure {
        resolved.depth_pressure = depth_pressure;
    }
    if let Some(depth_alpha) = settings.depth_alpha {
        resolved.depth_alpha = depth_alpha;
    }
    if let Some(width_pressure_curve) = settings.width_pressure_curve {
        resolved.width_pressure_curve = width_pressure_curve;
    }
    if let Some(alpha_pressure_curve) = settings.alpha_pressure_curve {
        resolved.alpha_pressure_curve = alpha_pressure_curve;
    }
    if let Some(endpoint_snap_px) = settings.endpoint_snap_px {
        resolved.endpoint_snap_px = endpoint_snap_px;
    }
    if let Some(endpoint_lock_start_px) = settings.endpoint_lock_start_px {
        resolved.endpoint_lock_start_px = endpoint_lock_start_px;
    }
    if let Some(endpoint_lock_end_px) = settings.endpoint_lock_end_px {
        resolved.endpoint_lock_end_px = endpoint_lock_end_px;
    }
    if let Some(path_simplify_px) = settings.path_simplify_px {
        resolved.path_simplify_px = path_simplify_px;
    }
    if let Some(straightness) = settings.straightness {
        resolved.straightness = straightness;
    }
    if let Some(taper) = settings.taper {
        resolved.taper = taper;
    }
    if let Some(stroke_wobble_px) = settings.stroke_wobble_px {
        resolved.stroke_wobble_px = stroke_wobble_px;
    }
    if let Some(stroke_wobble_frequency) = settings.stroke_wobble_frequency {
        resolved.stroke_wobble_frequency = stroke_wobble_frequency;
    }
    if let Some(micro_wobble_px) = settings.micro_wobble_px {
        resolved.micro_wobble_px = micro_wobble_px;
    }
    if let Some(micro_wobble_frequency) = settings.micro_wobble_frequency {
        resolved.micro_wobble_frequency = micro_wobble_frequency;
    }
    if let Some(pressure_jitter) = settings.pressure_jitter {
        resolved.pressure_jitter = pressure_jitter;
    }
    if let Some(local_angular_drift_degrees) = settings.local_angular_drift_degrees {
        resolved.local_angular_drift_degrees = local_angular_drift_degrees;
    }
    if let Some(overshoot_px) = settings.overshoot_px {
        resolved.overshoot_px = overshoot_px;
    }
    if let Some(undershoot_px) = settings.undershoot_px {
        resolved.undershoot_px = undershoot_px;
    }
    if let Some(pass_offset_px) = settings.pass_offset_px {
        resolved.pass_offset_px = pass_offset_px;
    }
    if let Some(dropout) = settings.dropout {
        resolved.dropout = dropout;
    }
    if let Some(dropout_segment_min_px) = settings.dropout_segment_min_px {
        resolved.dropout_segment_min_px = dropout_segment_min_px;
    }
    if let Some(passes) = settings.passes.as_ref() {
        apply_npr_passes_field_3d(passes, &mut resolved);
    }
    if let Some(search_line_count) = settings.search_line_count {
        resolved.search_line_count = search_line_count;
    }
    if let Some(search_line_alpha) = settings.search_line_alpha {
        resolved.search_line_alpha = search_line_alpha;
    }
    if let Some(seed) = settings.seed {
        resolved.seed = seed;
    }
    if let Some(gpu_realtime_tuning) = settings.gpu_realtime_tuning.as_ref() {
        apply_npr_gpu_realtime_tuning_3d(gpu_realtime_tuning, &mut resolved);
    }

    if let Some(silhouette_override) = settings.silhouette_override.as_ref() {
        resolved.silhouette_override =
            Some(npr_line_kind_override_3d_from_document(silhouette_override));
    }
    if let Some(boundary_override) = settings.boundary_override.as_ref() {
        resolved.boundary_override = Some(npr_line_kind_override_3d_from_document(boundary_override));
    }
    if let Some(feature_override) = settings.feature_override.as_ref() {
        resolved.feature_override = Some(npr_line_kind_override_3d_from_document(feature_override));
    }

    Ok(resolved)
}

fn apply_grouped_npr_line_settings_3d(
    settings: &crate::document::NprLine3dSettingsDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<()> {
    if let Some(tool) = settings.tool.as_ref() {
        if let Some(kind) = tool.kind.as_deref() {
            resolved.stroke_tool =
                npr_stroke_tool_3d_from_document(kind, scene_id, entity_id, component_kind)?;
        }
        if let Some(base_width_px) = tool.base_width_px {
            resolved.width_px = base_width_px;
        }
        if let Some(base_alpha) = tool.base_alpha {
            resolved.ink_color.a = base_alpha;
        }
        if let Some(width_multiplier) = tool.width_multiplier {
            resolved.tool_width_multiplier = width_multiplier;
        }
        if let Some(alpha_multiplier) = tool.alpha_multiplier {
            resolved.tool_alpha_multiplier = alpha_multiplier;
        }
        if let Some(wobble_multiplier) = tool.wobble_multiplier {
            resolved.tool_wobble_multiplier = wobble_multiplier;
        }
        if let Some(pressure_jitter_multiplier) = tool.pressure_jitter_multiplier {
            resolved.tool_pressure_jitter_multiplier = pressure_jitter_multiplier;
        }
        if let Some(dropout_multiplier) = tool.dropout_multiplier {
            resolved.tool_dropout_multiplier = dropout_multiplier;
        }
        if let Some(search_multiplier) = tool.search_multiplier {
            resolved.tool_search_multiplier = search_multiplier;
        }
    }
    if let Some(trajectory) = settings.trajectory.as_ref() {
        if let Some(path_adherence) = trajectory.path_adherence {
            resolved.straightness = path_adherence;
        }
        if let Some(straightness) = trajectory.straightness {
            resolved.straightness = straightness;
        }
        if let Some(humanization) = trajectory.humanization {
            resolved.humanization = humanization;
        }
        if let Some(gesture_offset_px) = trajectory.gesture_offset_px {
            resolved.stroke_wobble_px = gesture_offset_px;
        }
        if let Some(gesture_frequency_per_100px) = trajectory.gesture_frequency_per_100px {
            resolved.stroke_wobble_frequency = gesture_frequency_per_100px;
        }
        if let Some(micro_offset_px) = trajectory.micro_offset_px {
            resolved.micro_wobble_px = micro_offset_px;
        }
        if let Some(micro_frequency_per_100px) = trajectory.micro_frequency_per_100px {
            resolved.micro_wobble_frequency = micro_frequency_per_100px;
        }
        if let Some(angular_drift_degrees) = trajectory.angular_drift_degrees {
            resolved.local_angular_drift_degrees = angular_drift_degrees;
        }
        if let Some(endpoint_snap_px) = trajectory.endpoint_snap_px {
            resolved.endpoint_snap_px = endpoint_snap_px;
        }
        if let Some(path_simplify_px) = trajectory.path_simplify_px {
            resolved.path_simplify_px = path_simplify_px;
        }
    }
    if let Some(pressure) = settings.pressure.as_ref() {
        if let Some(width_curve) = pressure.width_curve {
            resolved.width_pressure_curve = width_curve;
        }
        if let Some(jitter) = pressure.jitter {
            resolved.pressure_jitter = jitter;
        }
    }
    if let Some(opacity) = settings.opacity.as_ref() {
        if let Some(alpha_curve) = opacity.alpha_curve {
            resolved.alpha_pressure_curve = alpha_curve;
        }
        if let Some(base_alpha) = opacity.base_alpha {
            resolved.ink_color.a = base_alpha;
        }
    }
    if let Some(endpoints) = settings.endpoints.as_ref() {
        if let Some(taper) = endpoints.taper {
            resolved.taper = taper;
        }
        if let Some(lock_start_px) = endpoints.lock_start_px {
            resolved.endpoint_lock_start_px = lock_start_px;
        }
        if let Some(lock_end_px) = endpoints.lock_end_px {
            resolved.endpoint_lock_end_px = lock_end_px;
        }
        if let Some(overshoot_px) = endpoints.overshoot_px.or(endpoints.overshoot_end_px) {
            resolved.overshoot_px = overshoot_px;
        }
        if let Some(undershoot_px) = endpoints.undershoot_px.or(endpoints.undershoot_end_px) {
            resolved.undershoot_px = undershoot_px;
        }
    }
    if let Some(breakup) = settings.breakup.as_ref() {
        if let Some(amount) = breakup.amount.or(breakup.dropout) {
            resolved.dropout = amount;
        }
        if let Some(min_gap_px) = breakup.min_gap_px.or(breakup.min_visible_segment_px) {
            resolved.dropout_segment_min_px = min_gap_px;
        }
    }
    if let Some(depth) = settings.depth.as_ref() {
        if let Some(width_influence) = depth.width_influence {
            resolved.depth_pressure = width_influence;
        }
        if let Some(alpha_influence) = depth.alpha_influence {
            resolved.depth_alpha = alpha_influence;
        }
    }
    if let Some(confidence) = settings.confidence.as_ref() {
        if let Some(line_confidence) = confidence.line_confidence {
            resolved.line_confidence = line_confidence;
        }
    }
    if let Some(class_overrides) = settings.class_overrides.as_ref() {
        if let Some(silhouette) = class_overrides.silhouette.as_ref() {
            resolved.silhouette_override = Some(npr_line_kind_override_3d_from_document(silhouette));
        }
        if let Some(boundary) = class_overrides.boundary.as_ref() {
            resolved.boundary_override = Some(npr_line_kind_override_3d_from_document(boundary));
        }
        if let Some(feature) = class_overrides.feature.as_ref() {
            resolved.feature_override = Some(npr_line_kind_override_3d_from_document(feature));
        }
    }
    if let Some(performance) = settings.performance.as_ref() {
        if let Some(visibility_max_dimension_px) = performance.visibility_max_dimension_px {
            resolved.visibility_max_dimension_px = visibility_max_dimension_px;
        }
    }
    Ok(())
}

fn apply_npr_passes_field_3d(
    passes: &crate::document::NprLine3dPassesFieldDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
) {
    match passes {
        crate::document::NprLine3dPassesFieldDocument::Count(count) => {
            resolved.passes = *count;
        }
        crate::document::NprLine3dPassesFieldDocument::Plan(plan) => {
            if let Some(primary_count) = plan.primary_count {
                resolved.passes = primary_count;
            }
            if let Some(search_count) = plan.search_count {
                resolved.search_line_count = search_count;
            }
            if let Some(search_alpha) = plan.search_alpha {
                resolved.search_line_alpha = search_alpha;
            }
            if let Some(search_offset_px) = plan.search_offset_px {
                resolved.pass_offset_px = search_offset_px;
            }
        }
    }
}

fn apply_npr_gpu_realtime_tuning_3d(
    document: &crate::document::NprGpuRealtimeTuningDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
) {
    let mut tuning = resolved.gpu_realtime_tuning;
    if let Some(value) = document.max_render_length_px {
        tuning.max_render_length_px = value;
    }
    if let Some(value) = document.max_segment_length_px {
        tuning.max_segment_length_px = value;
    }
    if let Some(value) = document.max_terminal_walk_edges {
        tuning.max_terminal_walk_edges = value;
    }
    if let Some(value) = document.max_chained_walk_edges {
        tuning.max_chained_walk_edges = value;
    }
    if let Some(value) = document.max_chain_angle_degrees {
        tuning.max_chain_angle_degrees = value;
    }
    if let Some(value) = document.search_enabled {
        tuning.search_enabled = value;
    }
    if let Some(value) = document.search_max_render_length_px {
        tuning.search_max_render_length_px = value;
    }
    if let Some(value) = document.search_alpha_multiplier {
        tuning.search_alpha_multiplier = value;
    }
    if let Some(value) = document.feature_min_length_multiplier {
        tuning.feature_min_length_multiplier = value;
    }
    if let Some(value) = document.feature_alpha_multiplier {
        tuning.feature_alpha_multiplier = value;
    }
    if let Some(value) = document.silhouette_min_length_multiplier {
        tuning.silhouette_min_length_multiplier = value;
    }
    resolved.gpu_realtime_tuning = tuning.normalized();
}

fn npr_stroke_tool_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprStrokeTool3d> {
    match value.trim() {
        "ink_pen" => Ok(amigo_render_api::NprStrokeTool3d::InkPen),
        "pencil" => Ok(amigo_render_api::NprStrokeTool3d::Pencil),
        "brush" => Ok(amigo_render_api::NprStrokeTool3d::Brush),
        "marker" => Ok(amigo_render_api::NprStrokeTool3d::Marker),
        "technical_pen" => Ok(amigo_render_api::NprStrokeTool3d::TechnicalPen),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "expected NPR stroke_tool to be ink_pen, pencil, brush, marker, or technical_pen, got `{other}`"
            ),
        }),
    }
}

fn npr_render_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprRenderStrategy3d> {
    match value.trim() {
        "gpu_realtime" => Ok(amigo_render_api::NprRenderStrategy3d::GpuRealtime),
        "cpu_reference" => Ok(amigo_render_api::NprRenderStrategy3d::CpuReference),
        "hybrid" | "auto" => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "unsupported Mesh3D.npr.strategy `{value}`; hybrid/auto is intentionally disabled; use `gpu_realtime` or `cpu_reference`"
            ),
        }),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.strategy `{other}`; expected `gpu_realtime` or `cpu_reference`"
            ),
        }),
    }
}

fn npr_fill_mode_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprFillMode3d> {
    match value.trim() {
        "shaded" => Ok(amigo_render_api::NprFillMode3d::Shaded),
        "none" => Ok(amigo_render_api::NprFillMode3d::None),
        "depth_only" => Ok(amigo_render_api::NprFillMode3d::DepthOnly),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.fill_mode `{other}`; expected `shaded`, `none`, or `depth_only`"
            ),
        }),
    }
}

fn npr_style_preset_3d_from_document(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprStylePreset3d> {
    let Some(value) = value.map(str::trim).filter(|value| !value.is_empty()) else {
        return Ok(amigo_render_api::NprStylePreset3d::default());
    };

    match value {
        "gpu_stable_comic" | "default_gpu_comic" => {
            Ok(amigo_render_api::NprStylePreset3d::GpuStableComic)
        }
        "rough_comic_ink" => Ok(amigo_render_api::NprStylePreset3d::RoughComicInk),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "expected Mesh3D.npr.style_preset to be `gpu_stable_comic`, `default_gpu_comic`, or `rough_comic_ink`, got `{other}`"
            ),
        }),
    }
}

fn npr_line_kind_override_3d_from_document(
    document: &crate::document::NprLine3dKindOverrideDocument,
) -> amigo_render_api::NprLineKindOverride3d {
    amigo_render_api::NprLineKindOverride3d {
        width_multiplier: document.width_multiplier,
        wobble_px: document.wobble_px,
        dropout: document.dropout,
        taper: document.taper,
        overshoot_px: document.overshoot_px,
        alpha_multiplier: document.alpha_multiplier,
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
