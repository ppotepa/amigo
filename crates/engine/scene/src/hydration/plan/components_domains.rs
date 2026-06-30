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
            orbit_pan_sensitivity,
            orbit_zoom_speed,
            freelook_speed,
            freelook_sensitivity,
            freelook_fast_multiplier,
            move_forward_action,
            move_strafe_action,
            move_lift_action,
        } => {
            commands.push(SceneCommand::Plugin {
                command: camera_controller_3d_plugin_scene_command(
                    CameraController3dSceneCommand {
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
                        orbit_pan_sensitivity: *orbit_pan_sensitivity,
                        orbit_zoom_speed: *orbit_zoom_speed,
                        freelook_speed: *freelook_speed,
                        freelook_sensitivity: *freelook_sensitivity,
                        freelook_fast_multiplier: *freelook_fast_multiplier,
                        move_forward_action: move_forward_action.clone(),
                        move_strafe_action: move_strafe_action.clone(),
                        move_lift_action: move_lift_action.clone(),
                    },
                ),
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

    apply_grouped_npr_line_settings_3d(
        settings,
        &mut resolved,
        scene_id,
        entity_id,
        component_kind,
    )?;

    if let Some(strategy) = settings.strategy.as_deref() {
        resolved.render_strategy =
            npr_render_strategy_3d_from_document(strategy, scene_id, entity_id, component_kind)?;
    }
    if let Some(fill_mode) = settings.fill_mode.as_deref() {
        resolved.fill_mode =
            npr_fill_mode_3d_from_document(fill_mode, scene_id, entity_id, component_kind)?;
    }
    if let Some(pipeline) = settings.pipeline.as_ref() {
        apply_npr_pipeline_strategies_3d(
            pipeline,
            &mut resolved,
            scene_id,
            entity_id,
            component_kind,
        )?;
    }
    if let Some(profile) = settings.cpu_strategy_profile.as_ref() {
        resolved.cpu_strategy_profile =
            npr_cpu_strategy_profile_from_document(profile, scene_id, entity_id, component_kind)?;
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
    if let Some(black_mass_material_ids) = settings.black_mass_material_ids.as_ref() {
        resolved.black_mass_material_ids = black_mass_material_ids.clone();
    }
    if let Some(ink_detail_material_ids) = settings.ink_detail_material_ids.as_ref() {
        resolved.ink_detail_material_ids = ink_detail_material_ids.clone();
    }
    if let Some(black_tone_hatching) = settings.black_tone_hatching.as_ref() {
        resolved.black_tone_hatching = npr_black_tone_hatching_from_document(
            black_tone_hatching,
            scene_id,
            entity_id,
            component_kind,
        )?;
    }
    if let Some(brushes) = settings.brushes.as_ref() {
        resolved.brush_profiles =
            npr_brush_profiles_from_document(brushes, scene_id, entity_id, component_kind)?;
    }
    if let Some(families) = settings.families.as_ref() {
        resolved.line_families =
            npr_line_families_from_document(families, scene_id, entity_id, component_kind)?;
    }
    if let Some(feature_angle_degrees) = settings.feature_angle_degrees {
        resolved.feature_angle_degrees = feature_angle_degrees;
    }
    if let Some(min_screen_length_px) = settings.min_screen_length_px {
        resolved.min_screen_length_px = min_screen_length_px;
    }
    if let Some(min_stroke_length_px) = settings.min_stroke_length_px {
        resolved.min_stroke_length_px = min_stroke_length_px;
    }
    if let Some(preferred_stroke_length_px) = settings.preferred_stroke_length_px {
        resolved.preferred_stroke_length_px = preferred_stroke_length_px;
    }
    if let Some(stroke_join_gap_px) = settings.stroke_join_gap_px {
        resolved.stroke_join_gap_px = stroke_join_gap_px;
    }
    if let Some(stroke_join_max_angle_degrees) = settings.stroke_join_max_angle_degrees {
        resolved.stroke_join_max_angle_degrees = stroke_join_max_angle_degrees;
    }
    if let Some(technical_detail_keep) = settings.technical_detail_keep {
        resolved.technical_detail_keep = technical_detail_keep;
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
        apply_npr_gpu_realtime_tuning_3d(
            gpu_realtime_tuning,
            &mut resolved,
            scene_id,
            entity_id,
            component_kind,
        )?;
    }
    if let Some(camera_response) = settings.camera_response.as_ref() {
        apply_npr_camera_response_3d(camera_response, &mut resolved);
    }

    if let Some(silhouette_override) = settings.silhouette_override.as_ref() {
        resolved.silhouette_override =
            Some(npr_line_kind_override_3d_from_document(silhouette_override));
    }
    if let Some(boundary_override) = settings.boundary_override.as_ref() {
        resolved.boundary_override =
            Some(npr_line_kind_override_3d_from_document(boundary_override));
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
            resolved.silhouette_override =
                Some(npr_line_kind_override_3d_from_document(silhouette));
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
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<()> {
    let mut tuning = resolved.gpu_realtime_tuning;
    if let Some(value) = document.debug_mode.as_deref() {
        tuning.debug_mode =
            npr_gpu_debug_mode_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
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
    if let Some(value) = document.artist_selection_amount {
        tuning.artist_selection_amount = value;
    }
    if let Some(value) = document.artist_trim_amount {
        tuning.artist_trim_amount = value;
    }
    if let Some(value) = document.artist_lift_amount {
        tuning.artist_lift_amount = value;
    }
    resolved.gpu_realtime_tuning = tuning.normalized();
    Ok(())
}

fn apply_npr_camera_response_3d(
    document: &crate::document::NprCameraResponseDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
) {
    let mut response = resolved.camera_response;
    if let Some(value) = document.enabled {
        response.enabled = value;
    }
    if let Some(value) = document.auto_focus {
        response.auto_focus = value;
    }
    if let Some(value) = document.near_distance {
        response.near_distance = value;
    }
    if let Some(value) = document.far_distance {
        response.far_distance = value;
    }
    if let Some(value) = document.focus_near_band {
        response.focus_near_band = value;
    }
    if let Some(value) = document.focus_far_band {
        response.focus_far_band = value;
    }
    if let Some(value) = document.near_width_boost {
        response.near_width_boost = value;
    }
    if let Some(value) = document.near_detail_boost {
        response.near_detail_boost = value;
    }
    if let Some(value) = document.near_hatching_boost {
        response.near_hatching_boost = value;
    }
    if let Some(value) = document.far_width_falloff {
        response.far_width_falloff = value;
    }
    if let Some(value) = document.far_alpha_falloff {
        response.far_alpha_falloff = value;
    }
    if let Some(value) = document.far_detail_suppression {
        response.far_detail_suppression = value;
    }
    if let Some(value) = document.rim_silhouette_boost {
        response.rim_silhouette_boost = value;
    }
    if let Some(value) = document.front_feature_suppression {
        response.front_feature_suppression = value;
    }
    resolved.camera_response = response.normalized();
}

fn npr_gpu_debug_mode_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprGpuDebugMode3d> {
    amigo_render_api::NprGpuDebugMode3d::parse(value).ok_or_else(|| crate::SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!(
            "expected Mesh3D.npr.gpu_realtime_tuning.debug_mode to be final, line_kinds, raw_paths, dropout, width_alpha, chain_hops, candidate_importance, technical_selection, stroke_length_bucket, source_edge_count, stroke_roles, or material_roles, got `{value}`"
        ),
    })
}

fn npr_brush_profiles_from_document(
    brushes: &std::collections::BTreeMap<String, crate::document::NprBrushProfileDocument>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<std::collections::BTreeMap<String, amigo_render_api::NprBrushProfile3d>> {
    let mut resolved = std::collections::BTreeMap::new();
    for (id, document) in brushes {
        let tool = match document.tool.as_deref() {
            Some(value) => Some(npr_stroke_tool_3d_from_document(
                value,
                scene_id,
                entity_id,
                component_kind,
            )?),
            None => None,
        };
        let tip = match document.tip.as_deref() {
            Some(value) => Some(npr_brush_tip_3d_from_document(
                value,
                scene_id,
                entity_id,
                component_kind,
            )?),
            None => None,
        };
        let ink_color = document
            .ink_color
            .as_deref()
            .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
            .transpose()?;
        resolved.insert(
            id.clone(),
            amigo_render_api::NprBrushProfile3d {
                tool,
                tip,
                ink_color,
                width_multiplier: document.width_multiplier.unwrap_or(1.0),
                alpha_multiplier: document.alpha_multiplier.unwrap_or(1.0),
                pressure_jitter_multiplier: document.pressure_jitter_multiplier.unwrap_or(1.0),
                dropout_multiplier: document.dropout_multiplier.unwrap_or(1.0),
                search_multiplier: document.search_multiplier.unwrap_or(1.0),
                path_wobble_multiplier: document.path_wobble_multiplier.unwrap_or(1.0),
                micro_wobble_multiplier: document.micro_wobble_multiplier.unwrap_or(1.0),
                hand_arc_multiplier: document.hand_arc_multiplier.unwrap_or(1.0),
                tangent_drift_multiplier: document.tangent_drift_multiplier.unwrap_or(1.0),
                detail_crispness_multiplier: document.detail_crispness_multiplier.unwrap_or(1.0),
                taper_multiplier: document.taper_multiplier.unwrap_or(1.0),
                overshoot_px: document.overshoot_px,
                width_curve: document.width_curve.unwrap_or([1.0, 1.0, 1.0, 1.0]),
                alpha_curve: document.alpha_curve.unwrap_or([1.0, 1.0, 1.0, 1.0]),
                angle_bias_degrees: document.angle_bias_degrees.unwrap_or(0.0),
                angle_influence: document.angle_influence.unwrap_or(0.0),
                nib_width_base_scale: document.nib_width_base_scale.unwrap_or(1.0),
                nib_width_angle_scale: document.nib_width_angle_scale.unwrap_or(1.0),
                path_adherence_multiplier: document.path_adherence_multiplier.unwrap_or(1.0),
            },
        );
    }
    Ok(resolved)
}

fn npr_line_families_from_document(
    families: &[crate::document::NprLineFamilyDocument],
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<Vec<amigo_render_api::NprLineFamily3d>> {
    families
        .iter()
        .map(|family| {
            Ok(amigo_render_api::NprLineFamily3d {
                id: family.id.clone(),
                enabled: family.enabled,
                role: match family.role.as_deref() {
                    Some(value) => Some(npr_line_family_role_3d_from_document(
                        value,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?),
                    None => None,
                },
                priority: family.priority.unwrap_or(0),
                sources: family
                    .sources
                    .iter()
                    .map(|value| {
                        npr_line_source_3d_from_document(value, scene_id, entity_id, component_kind)
                    })
                    .collect::<SceneDocumentResult<Vec<_>>>()?,
                brush: family.brush.clone(),
                preferred_stroke_length_px: family.preferred_stroke_length_px,
                stroke_join_gap_px: family.stroke_join_gap_px,
                stroke_join_max_angle_degrees: family.stroke_join_max_angle_degrees,
                technical_detail_keep: family.technical_detail_keep,
                min_screen_length_px: family.min_screen_length_px,
                min_stroke_length_px: family.min_stroke_length_px,
                technical_detail_preference: family.technical_detail_preference,
                ink_detail_material_preference: family.ink_detail_material_preference,
                material_seam_preference: family.material_seam_preference,
                continuation_bias: family.continuation_bias,
                breakup_bias: family.breakup_bias,
                width_multiplier: family.width_multiplier.unwrap_or(1.0),
                alpha_multiplier: family.alpha_multiplier.unwrap_or(1.0),
                taper_multiplier: family.taper_multiplier.unwrap_or(1.0),
                overshoot_px: family.overshoot_px,
            })
        })
        .collect()
}

fn npr_line_source_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprLineSource3d> {
    match value.trim() {
        "silhouette" => Ok(amigo_render_api::NprLineSource3d::Silhouette),
        "boundary" => Ok(amigo_render_api::NprLineSource3d::Boundary),
        "feature" => Ok(amigo_render_api::NprLineSource3d::Feature),
        "crease" => Ok(amigo_render_api::NprLineSource3d::Crease),
        "seam" => Ok(amigo_render_api::NprLineSource3d::Seam),
        "contact" => Ok(amigo_render_api::NprLineSource3d::Contact),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid NPR line family source `{other}`; expected `silhouette`, `boundary`, `feature`, `crease`, `seam`, or `contact`"
            ),
        }),
    }
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

fn npr_brush_tip_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprBrushTip3d> {
    match value.trim() {
        "round" => Ok(amigo_render_api::NprBrushTip3d::Round),
        "flat" => Ok(amigo_render_api::NprBrushTip3d::Flat),
        "g_pen" => Ok(amigo_render_api::NprBrushTip3d::GPen),
        "maru_pen" => Ok(amigo_render_api::NprBrushTip3d::MaruPen),
        "dry_brush" => Ok(amigo_render_api::NprBrushTip3d::DryBrush),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "expected NPR brush tip to be round, flat, g_pen, maru_pen, or dry_brush, got `{other}`"
            ),
        }),
    }
}

fn npr_line_family_role_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprLineFamilyRole3d> {
    match value.trim() {
        "generic" => Ok(amigo_render_api::NprLineFamilyRole3d::Generic),
        "outer_contour" => Ok(amigo_render_api::NprLineFamilyRole3d::OuterContour),
        "detail_ink" => Ok(amigo_render_api::NprLineFamilyRole3d::DetailInk),
        "cloth_fold" => Ok(amigo_render_api::NprLineFamilyRole3d::ClothFold),
        "material_cut" => Ok(amigo_render_api::NprLineFamilyRole3d::MaterialCut),
        "shadow_hatch" => Ok(amigo_render_api::NprLineFamilyRole3d::ShadowHatch),
        "contact_shadow" => Ok(amigo_render_api::NprLineFamilyRole3d::ContactShadow),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid NPR line family role `{other}`; expected `generic`, `outer_contour`, `detail_ink`, `cloth_fold`, `material_cut`, `shadow_hatch`, or `contact_shadow`"
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

fn npr_black_tone_hatching_from_document(
    document: &crate::document::NprBlackToneHatchingDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprBlackToneHatching3d> {
    let mut resolved = amigo_render_api::NprBlackToneHatching3d::default();
    if let Some(enabled) = document.enabled {
        resolved.enabled = enabled;
    }
    if let Some(source) = document.source.as_deref() {
        resolved.source = npr_black_tone_hatching_source_from_document(
            source,
            scene_id,
            entity_id,
            component_kind,
        )?;
    }
    if let Some(spacing_px) = document.spacing_px {
        resolved.spacing_px = spacing_px;
    }
    if let Some(length_px) = document.length_px {
        resolved.length_px = length_px;
    }
    if let Some(width_px) = document.width_px {
        resolved.width_px = width_px;
    }
    if let Some(alpha) = document.alpha {
        resolved.alpha = alpha;
    }
    if let Some(density) = document.density {
        resolved.density = density;
    }
    if let Some(tone_threshold) = document.tone_threshold {
        resolved.tone_threshold = tone_threshold;
    }
    if let Some(tone_softness) = document.tone_softness {
        resolved.tone_softness = tone_softness;
    }
    if let Some(angle_degrees) = document.angle_degrees {
        resolved.angle_degrees = angle_degrees;
    }
    if let Some(angle_jitter_degrees) = document.angle_jitter_degrees {
        resolved.angle_jitter_degrees = angle_jitter_degrees;
    }
    if let Some(surface_clip_samples) = document.surface_clip_samples {
        resolved.surface_clip_samples = surface_clip_samples;
    }
    if let Some(max_strokes) = document.max_strokes {
        resolved.max_strokes = max_strokes;
    }
    Ok(resolved.normalized())
}

fn npr_black_tone_hatching_source_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprBlackToneHatchingSource3d> {
    match value.trim() {
        "auto" => Ok(amigo_render_api::NprBlackToneHatchingSource3d::Auto),
        "explicit_materials" => {
            Ok(amigo_render_api::NprBlackToneHatchingSource3d::ExplicitMaterials)
        }
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.black_tone_hatching.source `{other}`; expected `auto` or `explicit_materials`"
            ),
        }),
    }
}

fn apply_npr_pipeline_strategies_3d(
    document: &crate::document::NprPipelineStrategiesDocument,
    resolved: &mut amigo_render_api::NprLineSettings3d,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<()> {
    if let Some(value) = document.candidate_strategy.as_deref() {
        resolved.pipeline.candidate_strategy =
            npr_candidate_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.path_strategy.as_deref() {
        resolved.pipeline.path_strategy =
            npr_path_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.stroke_strategy.as_deref() {
        resolved.pipeline.stroke_strategy =
            npr_stroke_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.fill_strategy.as_deref() {
        resolved.pipeline.fill_strategy =
            npr_ink_fill_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.hatching_strategy.as_deref() {
        resolved.pipeline.hatching_strategy =
            npr_hatching_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.budget_strategy.as_deref() {
        resolved.pipeline.budget_strategy =
            npr_budget_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    if let Some(value) = document.temporal_strategy.as_deref() {
        resolved.pipeline.temporal_strategy =
            npr_temporal_strategy_3d_from_document(value, scene_id, entity_id, component_kind)?;
    }
    Ok(())
}

fn npr_cpu_strategy_profile_from_document(
    document: &crate::document::NprCpuStrategyProfileDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprCpuStrategyProfile3d> {
    let mut profile = match document.preset.as_deref().map(str::trim) {
        None | Some("") | Some("default") | Some("neutral") => {
            amigo_render_api::NprCpuStrategyProfile3d::default()
        }
        Some("toriyama_manga_ink") => amigo_render_api::NprCpuStrategyProfile3d::toriyama_manga_ink(),
        Some(other) => {
            return Err(crate::SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!(
                    "invalid Mesh3D.npr.cpu_strategy_profile.preset `{other}`; expected `default`, `neutral`, or `toriyama_manga_ink`"
                ),
            });
        }
    };

    if let Some(line_selection) = document.line_selection.as_ref() {
        apply_npr_line_selection_profile(line_selection, &mut profile.line_selection);
    }
    if let Some(path_joining) = document.path_joining.as_ref() {
        apply_npr_path_joining_profile(path_joining, &mut profile.path_joining);
    }
    if let Some(break_policy) = document.break_policy.as_ref() {
        apply_npr_break_policy_profile(break_policy, &mut profile.break_policy);
    }
    if let Some(stroke_synthesis) = document.stroke_synthesis.as_ref() {
        apply_npr_stroke_synthesis_profile(stroke_synthesis, &mut profile.stroke_synthesis);
    }
    if let Some(tessellation) = document.tessellation.as_ref() {
        apply_npr_tessellation_profile(tessellation, &mut profile.tessellation);
    }

    Ok(profile)
}

fn apply_npr_line_selection_profile(
    document: &crate::document::NprLineSelectionProfileDocument,
    profile: &mut amigo_render_api::NprLineSelectionProfile3d,
) {
    if let Some(value) = document.feature_importance {
        profile.feature_importance = value;
    }
    if let Some(value) = document.crease_importance {
        profile.crease_importance = value;
    }
    if let Some(value) = document.seam_importance {
        profile.seam_importance = value;
    }
    if let Some(value) = document.cloth_fold_importance {
        profile.cloth_fold_importance = value;
    }
    if let Some(value) = document.detail_ink_importance {
        profile.detail_ink_importance = value;
    }
    if let Some(value) = document.material_detail_bonus {
        profile.material_detail_bonus = value;
    }
    if let Some(value) = document.material_seam_penalty {
        profile.material_seam_penalty = value;
    }
    if let Some(value) = document.length_weight {
        profile.length_weight = value;
    }
    if let Some(value) = document.angle_weight {
        profile.angle_weight = value;
    }
    if let Some(value) = document.view_weight {
        profile.view_weight = value;
    }
    if let Some(value) = document.depth_weight {
        profile.depth_weight = value;
    }
    if let Some(value) = document.feature_face_bonus {
        profile.feature_face_bonus = value;
    }
    if let Some(value) = document.feature_torso_bonus {
        profile.feature_torso_bonus = value;
    }
    if let Some(value) = document.feature_hand_bonus {
        profile.feature_hand_bonus = value;
    }
    if let Some(value) = document.crease_face_bonus {
        profile.crease_face_bonus = value;
    }
    if let Some(value) = document.crease_torso_bonus {
        profile.crease_torso_bonus = value;
    }
    if let Some(value) = document.crease_hand_bonus {
        profile.crease_hand_bonus = value;
    }
    if let Some(value) = document.seam_torso_bonus {
        profile.seam_torso_bonus = value;
    }
    if let Some(value) = document.seam_hand_bonus {
        profile.seam_hand_bonus = value;
    }
    if let Some(value) = document.readable_face_start_y {
        profile.readable_face_start_y = value;
    }
    if let Some(value) = document.readable_face_height {
        profile.readable_face_height = value;
    }
    if let Some(value) = document.readable_face_half_width {
        profile.readable_face_half_width = value;
    }
    if let Some(value) = document.readable_torso_center_y {
        profile.readable_torso_center_y = value;
    }
    if let Some(value) = document.readable_torso_half_height {
        profile.readable_torso_half_height = value;
    }
    if let Some(value) = document.readable_torso_half_width {
        profile.readable_torso_half_width = value;
    }
    if let Some(value) = document.readable_hand_start_x {
        profile.readable_hand_start_x = value;
    }
    if let Some(value) = document.readable_hand_width {
        profile.readable_hand_width = value;
    }
    if let Some(value) = document.readable_hand_start_y {
        profile.readable_hand_start_y = value;
    }
    if let Some(value) = document.readable_hand_height {
        profile.readable_hand_height = value;
    }
    if let Some(value) = document.short_feature_penalty {
        profile.short_feature_penalty = value;
    }
    if let Some(value) = document.short_crease_penalty {
        profile.short_crease_penalty = value;
    }
    if let Some(value) = document.short_seam_penalty {
        profile.short_seam_penalty = value;
    }
    if let Some(value) = document.readable_region_penalty_relief {
        profile.readable_region_penalty_relief = value;
    }
    if let Some(value) = document.material_detail_penalty_scale {
        profile.material_detail_penalty_scale = value;
    }
    if let Some(value) = document.material_detail_min_screen_length_multiplier {
        profile.material_detail_min_screen_length_multiplier = value;
    }
    if let Some(value) = document.candidate_length_span_min_screen_multiplier {
        profile.candidate_length_span_min_screen_multiplier = value;
    }
    if let Some(value) = document.candidate_depth_weight {
        profile.candidate_depth_weight = value;
    }
    if let Some(value) = document.candidate_depth_min_score {
        profile.candidate_depth_min_score = value;
    }
    if let Some(value) = document.cloth_fold_length_weight {
        profile.cloth_fold_length_weight = value;
    }
    if let Some(value) = document.detail_ink_material_base {
        profile.detail_ink_material_base = value;
    }
    if let Some(value) = document.detail_ink_length_weight {
        profile.detail_ink_length_weight = value;
    }
    if let Some(value) = document.material_cut_seam_base {
        profile.material_cut_seam_base = value;
    }
    if let Some(value) = document.material_cut_length_weight {
        profile.material_cut_length_weight = value;
    }
    if let Some(value) = document.short_crease_base_penalty {
        profile.short_crease_base_penalty = value;
    }
    if let Some(value) = document.short_seam_base_penalty {
        profile.short_seam_base_penalty = value;
    }
    if let Some(value) = document.short_feature_base_penalty {
        profile.short_feature_base_penalty = value;
    }
    if let Some(value) = document.readable_region_relief_scale {
        profile.readable_region_relief_scale = value;
    }
    if let Some(value) = document.detail_keep_importance_weight {
        profile.detail_keep_importance_weight = value;
    }
    if let Some(value) = document.cloth_fold_keep_floor {
        profile.cloth_fold_keep_floor = value;
    }
    if let Some(value) = document.detail_ink_keep_floor {
        profile.detail_ink_keep_floor = value;
    }
    if let Some(value) = document.material_cut_keep_floor {
        profile.material_cut_keep_floor = value;
    }
    if let Some(value) = document.shadow_hatch_keep_floor {
        profile.shadow_hatch_keep_floor = value;
    }
    if let Some(value) = document.contact_shadow_keep_floor {
        profile.contact_shadow_keep_floor = value;
    }
    if let Some(value) = document.generic_feature_keep_floor {
        profile.generic_feature_keep_floor = value;
    }
    if let Some(value) = document.generic_crease_keep_floor {
        profile.generic_crease_keep_floor = value;
    }
    if let Some(value) = document.material_detail_keep_floor_relief {
        profile.material_detail_keep_floor_relief = value;
    }
    if let Some(value) = document.keep_floor_max {
        profile.keep_floor_max = value;
    }
    if let Some(value) = document.dense_edge_start_per_10k_px {
        profile.dense_edge_start_per_10k_px = value;
    }
    if let Some(value) = document.dense_edge_full_per_10k_px {
        profile.dense_edge_full_per_10k_px = value;
    }
    if let Some(value) = document.dense_material_seam_start_ratio {
        profile.dense_material_seam_start_ratio = value;
    }
    if let Some(value) = document.dense_material_seam_full_ratio {
        profile.dense_material_seam_full_ratio = value;
    }
    if let Some(value) = document.dense_boundary_start_ratio {
        profile.dense_boundary_start_ratio = value;
    }
    if let Some(value) = document.dense_boundary_full_ratio {
        profile.dense_boundary_full_ratio = value;
    }
    if let Some(value) = document.dense_technical_min_length_boost {
        profile.dense_technical_min_length_boost = value;
    }
    if let Some(value) = document.dense_boundary_min_length_boost {
        profile.dense_boundary_min_length_boost = value;
    }
    if let Some(value) = document.dense_technical_keep_scale_drop {
        profile.dense_technical_keep_scale_drop = value;
    }
    if let Some(value) = document.dense_keep_floor_boost {
        profile.dense_keep_floor_boost = value;
    }
    if let Some(value) = document.dense_material_detail_keep_floor_boost_scale {
        profile.dense_material_detail_keep_floor_boost_scale = value;
    }
    if let Some(value) = document.dense_material_detail_keep_scale_retention {
        profile.dense_material_detail_keep_scale_retention = value;
    }
    if let Some(value) = document.dense_boundary_outer_contour_threshold {
        profile.dense_boundary_outer_contour_threshold = value;
    }
    if let Some(value) = document.dense_pressure_outer_contour_threshold {
        profile.dense_pressure_outer_contour_threshold = value;
    }
    if let Some(value) = document.dense_seam_pressure_weight {
        profile.dense_seam_pressure_weight = value;
    }
    if let Some(value) = document.dense_boundary_pressure_weight {
        profile.dense_boundary_pressure_weight = value;
    }
    if let Some(value) = document.dense_material_detail_protection {
        profile.dense_material_detail_protection = value;
    }
    if let Some(value) = document.dense_material_detail_min_length_multiplier {
        profile.dense_material_detail_min_length_multiplier = value;
    }
    if let Some(value) = document.dense_quality_relief_start {
        profile.dense_quality_relief_start = value;
    }
    if let Some(value) = document.dense_quality_relief_span {
        profile.dense_quality_relief_span = value;
    }
    if let Some(value) = document.dense_quality_relief_scale {
        profile.dense_quality_relief_scale = value;
    }
    if let Some(value) = document.dense_quality_relief_penalty_scale {
        profile.dense_quality_relief_penalty_scale = value;
    }
    if let Some(value) = document.dense_seam_quality_relief_scale {
        profile.dense_seam_quality_relief_scale = value;
    }
    if let Some(value) = document.dense_seam_penalty_min {
        profile.dense_seam_penalty_min = value;
    }
    if let Some(value) = document.dense_seam_penalty {
        profile.dense_seam_penalty = value;
    }
    if let Some(value) = document.dense_feature_penalty {
        profile.dense_feature_penalty = value;
    }
    if let Some(value) = document.dense_crease_penalty {
        profile.dense_crease_penalty = value;
    }
}

fn apply_npr_path_joining_profile(
    document: &crate::document::NprPathJoiningProfileDocument,
    profile: &mut amigo_render_api::NprPathJoiningProfile3d,
) {
    if let Some(value) = document.readable_detail_relax_multiplier {
        profile.readable_detail_relax_multiplier = value;
    }
    if let Some(value) = document.readable_detail_importance_relax {
        profile.readable_detail_importance_relax = value;
    }
    if let Some(value) = document.readable_detail_relax_max {
        profile.readable_detail_relax_max = value;
    }
    if let Some(value) = document.continuation_bias_scale {
        profile.continuation_bias_scale = value;
    }
    if let Some(value) = document.readable_continuation_bonus {
        profile.readable_continuation_bonus = value;
    }
    if let Some(value) = document.readable_region_join_bonus {
        profile.readable_region_join_bonus = value;
    }
    if let Some(value) = document.preferred_length_bias_base {
        profile.preferred_length_bias_base = value;
    }
    if let Some(value) = document.gap_weight_base {
        profile.gap_weight_base = value;
    }
    if let Some(value) = document.gap_weight_breakup_scale {
        profile.gap_weight_breakup_scale = value;
    }
    if let Some(value) = document.gap_weight_continuation_scale {
        profile.gap_weight_continuation_scale = value;
    }
    if let Some(value) = document.gap_weight_readable_relax_scale {
        profile.gap_weight_readable_relax_scale = value;
    }
    if let Some(value) = document.gap_weight_min {
        profile.gap_weight_min = value;
    }
    if let Some(value) = document.tangent_weight_base {
        profile.tangent_weight_base = value;
    }
    if let Some(value) = document.tangent_weight_breakup_scale {
        profile.tangent_weight_breakup_scale = value;
    }
    if let Some(value) = document.tangent_weight_continuation_scale {
        profile.tangent_weight_continuation_scale = value;
    }
    if let Some(value) = document.tangent_weight_readable_relax_scale {
        profile.tangent_weight_readable_relax_scale = value;
    }
    if let Some(value) = document.tangent_weight_min {
        profile.tangent_weight_min = value;
    }
    if let Some(value) = document.readability_join_region_scale {
        profile.readability_join_region_scale = value;
    }
    if let Some(value) = document.readability_join_importance_scale {
        profile.readability_join_importance_scale = value;
    }
    if let Some(value) = document.readability_join_continuation_base {
        profile.readability_join_continuation_base = value;
    }
    if let Some(value) = document.readability_join_continuation_scale {
        profile.readability_join_continuation_scale = value;
    }
    if let Some(value) = document.feature_arc_target_degrees {
        profile.feature_arc_target_degrees = value;
    }
    if let Some(value) = document.feature_arc_window_degrees {
        profile.feature_arc_window_degrees = value;
    }
    if let Some(value) = document.feature_arc_bonus {
        profile.feature_arc_bonus = value;
    }
    if let Some(value) = document.crease_arc_target_degrees {
        profile.crease_arc_target_degrees = value;
    }
    if let Some(value) = document.crease_arc_window_degrees {
        profile.crease_arc_window_degrees = value;
    }
    if let Some(value) = document.crease_arc_bonus {
        profile.crease_arc_bonus = value;
    }
    if let Some(value) = document.seam_arc_target_degrees {
        profile.seam_arc_target_degrees = value;
    }
    if let Some(value) = document.seam_arc_window_degrees {
        profile.seam_arc_window_degrees = value;
    }
    if let Some(value) = document.seam_arc_bonus {
        profile.seam_arc_bonus = value;
    }
    if let Some(value) = document.feature_dead_straight_penalty {
        profile.feature_dead_straight_penalty = value;
    }
    if let Some(value) = document.crease_dead_straight_penalty {
        profile.crease_dead_straight_penalty = value;
    }
    if let Some(value) = document.path_importance_chain_bonus_per_edge {
        profile.path_importance_chain_bonus_per_edge = value;
    }
    if let Some(value) = document.path_importance_chain_bonus_max {
        profile.path_importance_chain_bonus_max = value;
    }
    if let Some(value) = document.path_importance_candidate_base {
        profile.path_importance_candidate_base = value;
    }
    if let Some(value) = document.path_importance_candidate_scale {
        profile.path_importance_candidate_scale = value;
    }
    if let Some(value) = document.path_importance_min {
        profile.path_importance_min = value;
    }
    if let Some(value) = document.path_importance_max {
        profile.path_importance_max = value;
    }
    if let Some(value) = document.path_importance_depth_base {
        profile.path_importance_depth_base = value;
    }
    if let Some(value) = document.path_importance_depth_weight {
        profile.path_importance_depth_weight = value;
    }
    if let Some(value) = document.path_importance_depth_min {
        profile.path_importance_depth_min = value;
    }
    if let Some(value) = document.path_importance_depth_max {
        profile.path_importance_depth_max = value;
    }
    if let Some(value) = document.path_importance_silhouette_multiplier {
        profile.path_importance_silhouette_multiplier = value;
    }
    if let Some(value) = document.path_importance_boundary_multiplier {
        profile.path_importance_boundary_multiplier = value;
    }
    if let Some(value) = document.path_importance_crease_multiplier {
        profile.path_importance_crease_multiplier = value;
    }
    if let Some(value) = document.path_importance_seam_multiplier {
        profile.path_importance_seam_multiplier = value;
    }
    if let Some(value) = document.path_importance_feature_multiplier {
        profile.path_importance_feature_multiplier = value;
    }
    if let Some(value) = document.path_importance_contact_multiplier {
        profile.path_importance_contact_multiplier = value;
    }
    if let Some(value) = document.region_feature_face_bonus {
        profile.region_feature_face_bonus = value;
    }
    if let Some(value) = document.region_feature_torso_bonus {
        profile.region_feature_torso_bonus = value;
    }
    if let Some(value) = document.region_feature_hand_bonus {
        profile.region_feature_hand_bonus = value;
    }
    if let Some(value) = document.region_crease_face_bonus {
        profile.region_crease_face_bonus = value;
    }
    if let Some(value) = document.region_crease_torso_bonus {
        profile.region_crease_torso_bonus = value;
    }
    if let Some(value) = document.region_crease_hand_bonus {
        profile.region_crease_hand_bonus = value;
    }
    if let Some(value) = document.region_seam_torso_bonus {
        profile.region_seam_torso_bonus = value;
    }
    if let Some(value) = document.region_seam_hand_bonus {
        profile.region_seam_hand_bonus = value;
    }
    if let Some(value) = document.survival_trait_keep_weight {
        profile.survival_trait_keep_weight = value;
    }
    if let Some(value) = document.survival_base_keep {
        profile.survival_base_keep = value;
    }
    if let Some(value) = document.survival_length_weight {
        profile.survival_length_weight = value;
    }
    if let Some(value) = document.survival_confidence_weight {
        profile.survival_confidence_weight = value;
    }
    if let Some(value) = document.survival_chain_bonus_per_edge {
        profile.survival_chain_bonus_per_edge = value;
    }
    if let Some(value) = document.survival_chain_bonus_max {
        profile.survival_chain_bonus_max = value;
    }
    if let Some(value) = document.survival_cloth_fold_base_bonus {
        profile.survival_cloth_fold_base_bonus = value;
    }
    if let Some(value) = document.survival_cloth_fold_chain_bonus_per_edge {
        profile.survival_cloth_fold_chain_bonus_per_edge = value;
    }
    if let Some(value) = document.survival_cloth_fold_chain_bonus_max {
        profile.survival_cloth_fold_chain_bonus_max = value;
    }
    if let Some(value) = document.survival_detail_material_bonus {
        profile.survival_detail_material_bonus = value;
    }
    if let Some(value) = document.survival_detail_plain_bonus {
        profile.survival_detail_plain_bonus = value;
    }
    if let Some(value) = document.survival_material_cut_seam_bonus {
        profile.survival_material_cut_seam_bonus = value;
    }
    if let Some(value) = document.survival_material_cut_plain_bonus {
        profile.survival_material_cut_plain_bonus = value;
    }
    if let Some(value) = document.survival_long_form_length_weight {
        profile.survival_long_form_length_weight = value;
    }
    if let Some(value) = document.survival_long_form_chain_bonus_per_edge {
        profile.survival_long_form_chain_bonus_per_edge = value;
    }
    if let Some(value) = document.survival_long_form_chain_bonus_max {
        profile.survival_long_form_chain_bonus_max = value;
    }
    if let Some(value) = document.survival_continuation_weight {
        profile.survival_continuation_weight = value;
    }
    if let Some(value) = document.survival_breakup_penalty {
        profile.survival_breakup_penalty = value;
    }
    if let Some(value) = document.isolated_detail_short_ratio {
        profile.isolated_detail_short_ratio = value;
    }
    if let Some(value) = document.isolated_cloth_fold_short_ratio {
        profile.isolated_cloth_fold_short_ratio = value;
    }
    if let Some(value) = document.isolated_material_cut_short_ratio {
        profile.isolated_material_cut_short_ratio = value;
    }
    if let Some(value) = document.min_length_character_readability_multiplier {
        profile.min_length_character_readability_multiplier = value;
    }
    if let Some(value) = document.min_length_silhouette_multiplier {
        profile.min_length_silhouette_multiplier = value;
    }
    if let Some(value) = document.min_length_boundary_multiplier {
        profile.min_length_boundary_multiplier = value;
    }
    if let Some(value) = document.min_length_contact_multiplier {
        profile.min_length_contact_multiplier = value;
    }
    if let Some(value) = document.min_length_crease_multiplier {
        profile.min_length_crease_multiplier = value;
    }
    if let Some(value) = document.min_length_seam_multiplier {
        profile.min_length_seam_multiplier = value;
    }
    if let Some(value) = document.min_length_feature_multiplier {
        profile.min_length_feature_multiplier = value;
    }
}

fn apply_npr_break_policy_profile(
    document: &crate::document::NprBreakPolicyProfileDocument,
    profile: &mut amigo_render_api::NprBreakPolicyProfile3d,
) {
    if let Some(value) = document.allow_seeded_long_feature_breaks {
        profile.allow_seeded_long_feature_breaks = value;
    }
    if let Some(value) = document.important_feature_break_threshold {
        profile.important_feature_break_threshold = value;
    }
    if let Some(value) = document.long_feature_break_min_length_px {
        profile.long_feature_break_min_length_px = value;
    }
    if let Some(value) = document.long_feature_break_min_complexity {
        profile.long_feature_break_min_complexity = value;
    }
    if let Some(value) = document.long_feature_break_chance {
        profile.long_feature_break_chance = value;
    }
    if let Some(value) = document.long_feature_break_center_t {
        profile.long_feature_break_center_t = value;
    }
    if let Some(value) = document.long_feature_break_center_jitter {
        profile.long_feature_break_center_jitter = value;
    }
    if let Some(value) = document.long_feature_break_center_min_t {
        profile.long_feature_break_center_min_t = value;
    }
    if let Some(value) = document.long_feature_break_center_max_t {
        profile.long_feature_break_center_max_t = value;
    }
    if let Some(value) = document.long_feature_break_min_gap_px {
        profile.long_feature_break_min_gap_px = value;
    }
    if let Some(value) = document.long_feature_break_gap_jitter_px {
        profile.long_feature_break_gap_jitter_px = value;
    }
    if let Some(value) = document.long_feature_break_half_t_min {
        profile.long_feature_break_half_t_min = value;
    }
    if let Some(value) = document.long_feature_break_half_t_max {
        profile.long_feature_break_half_t_max = value;
    }
    if let Some(value) = document.long_feature_break_t0_min {
        profile.long_feature_break_t0_min = value;
    }
    if let Some(value) = document.long_feature_break_t0_max {
        profile.long_feature_break_t0_max = value;
    }
    if let Some(value) = document.long_feature_break_t1_min {
        profile.long_feature_break_t1_min = value;
    }
    if let Some(value) = document.long_feature_break_t1_max {
        profile.long_feature_break_t1_max = value;
    }
    if let Some(value) = document.dropout_complexity_edge_limit {
        profile.dropout_complexity_edge_limit = value;
    }
    if let Some(value) = document.dropout_complexity_drop_per_edge {
        profile.dropout_complexity_drop_per_edge = value;
    }
    if let Some(value) = document.dropout_effective_max {
        profile.dropout_effective_max = value;
    }
    if let Some(value) = document.dropout_interval_length_px {
        profile.dropout_interval_length_px = value;
    }
    if let Some(value) = document.dropout_max_intervals {
        profile.dropout_max_intervals = value;
    }
    if let Some(value) = document.dropout_min_gap_t {
        profile.dropout_min_gap_t = value;
    }
    if let Some(value) = document.dropout_max_gap_t {
        profile.dropout_max_gap_t = value;
    }
    if let Some(value) = document.dropout_edge_margin_t {
        profile.dropout_edge_margin_t = value;
    }
}

fn apply_npr_stroke_synthesis_profile(
    document: &crate::document::NprStrokeSynthesisProfileDocument,
    profile: &mut amigo_render_api::NprStrokeSynthesisProfile3d,
) {
    if let Some(value) = document.silhouette_pressure {
        profile.silhouette_pressure = value;
    }
    if let Some(value) = document.boundary_pressure {
        profile.boundary_pressure = value;
    }
    if let Some(value) = document.feature_pressure {
        profile.feature_pressure = value;
    }
    if let Some(value) = document.crease_pressure {
        profile.crease_pressure = value;
    }
    if let Some(value) = document.seam_pressure {
        profile.seam_pressure = value;
    }
    if let Some(value) = document.contact_pressure {
        profile.contact_pressure = value;
    }
    if let Some(value) = document.technical_importance_base {
        profile.technical_importance_base = value;
    }
    if let Some(value) = document.technical_candidate_weight {
        profile.technical_candidate_weight = value;
    }
    if let Some(value) = document.technical_importance_min {
        profile.technical_importance_min = value;
    }
    if let Some(value) = document.technical_importance_max {
        profile.technical_importance_max = value;
    }
    if let Some(value) = document.expressive_importance_min {
        profile.expressive_importance_min = value;
    }
    if let Some(value) = document.expressive_importance_max {
        profile.expressive_importance_max = value;
    }
    if let Some(value) = document.protected_silhouette_importance_threshold {
        profile.protected_silhouette_importance_threshold = value;
    }
    if let Some(value) = document.single_pass_jitter_multiplier {
        profile.single_pass_jitter_multiplier = value;
    }
    if let Some(value) = document.single_pass_width_multiplier {
        profile.single_pass_width_multiplier = value;
    }
    if let Some(value) = document.single_pass_alpha {
        profile.single_pass_alpha = value;
    }
    if let Some(value) = document.dual_primary_jitter_multiplier {
        profile.dual_primary_jitter_multiplier = value;
    }
    if let Some(value) = document.dual_secondary_jitter_multiplier {
        profile.dual_secondary_jitter_multiplier = value;
    }
    if let Some(value) = document.dual_primary_width_multiplier {
        profile.dual_primary_width_multiplier = value;
    }
    if let Some(value) = document.dual_secondary_width_multiplier {
        profile.dual_secondary_width_multiplier = value;
    }
    if let Some(value) = document.dual_primary_alpha {
        profile.dual_primary_alpha = value;
    }
    if let Some(value) = document.dual_secondary_alpha {
        profile.dual_secondary_alpha = value;
    }
    if let Some(value) = document.multi_pass_jitter_base {
        profile.multi_pass_jitter_base = value;
    }
    if let Some(value) = document.multi_pass_jitter_step {
        profile.multi_pass_jitter_step = value;
    }
    if let Some(value) = document.multi_pass_width_multiplier {
        profile.multi_pass_width_multiplier = value;
    }
    if let Some(value) = document.multi_pass_alpha {
        profile.multi_pass_alpha = value;
    }
    if let Some(value) = document.search_wobble_multiplier {
        profile.search_wobble_multiplier = value;
    }
    if let Some(value) = document.search_width_multiplier {
        profile.search_width_multiplier = value;
    }
    if let Some(value) = document.hatch_chance_akira {
        profile.hatch_chance_akira = value;
    }
    if let Some(value) = document.hatch_chance_confident_manga {
        profile.hatch_chance_confident_manga = value;
    }
    if let Some(value) = document.hatch_chance_generic {
        profile.hatch_chance_generic = value;
    }
    if let Some(value) = document.hatch_path_length_min_px {
        profile.hatch_path_length_min_px = value;
    }
    if let Some(value) = document.hatch_path_length_max_px {
        profile.hatch_path_length_max_px = value;
    }
    if let Some(value) = document.hatch_center_t {
        profile.hatch_center_t = value;
    }
    if let Some(value) = document.hatch_center_jitter {
        profile.hatch_center_jitter = value;
    }
    if let Some(value) = document.hatch_length_min_px {
        profile.hatch_length_min_px = value;
    }
    if let Some(value) = document.hatch_length_jitter_px {
        profile.hatch_length_jitter_px = value;
    }
    if let Some(value) = document.hatch_half_t_min {
        profile.hatch_half_t_min = value;
    }
    if let Some(value) = document.hatch_half_t_max {
        profile.hatch_half_t_max = value;
    }
    if let Some(value) = document.hatch_wobble_multiplier {
        profile.hatch_wobble_multiplier = value;
    }
    if let Some(value) = document.hatch_width_multiplier {
        profile.hatch_width_multiplier = value;
    }
    if let Some(value) = document.hatch_alpha_multiplier {
        profile.hatch_alpha_multiplier = value;
    }
    if let Some(value) = document.hatch_alpha_max {
        profile.hatch_alpha_max = value;
    }
    if let Some(value) = document.short_detail_boost {
        profile.short_detail_boost = value;
    }
    if let Some(value) = document.short_detail_threshold_px {
        profile.short_detail_threshold_px = value;
    }
    if let Some(value) = document.medium_detail_boost {
        profile.medium_detail_boost = value;
    }
    if let Some(value) = document.medium_detail_threshold_px {
        profile.medium_detail_threshold_px = value;
    }
}

fn apply_npr_tessellation_profile(
    document: &crate::document::NprTessellationProfileDocument,
    profile: &mut amigo_render_api::NprTessellationProfile3d,
) {
    if let Some(value) = document.rail_tangent_smoothing {
        profile.rail_tangent_smoothing = value;
    }
    if let Some(value) = document.kink_fallback_dot {
        profile.kink_fallback_dot = value;
    }
    if let Some(value) = document.resample_spacing_px {
        profile.resample_spacing_px = value;
    }
    if let Some(value) = document.endpoint_lock_max_t {
        profile.endpoint_lock_max_t = value;
    }
    if let Some(value) = document.taper_endpoint_floor {
        profile.taper_endpoint_floor = value;
    }
    if let Some(value) = document.pass_wobble_max_px {
        profile.pass_wobble_max_px = value;
    }
    if let Some(value) = document.angle_alpha_influence {
        profile.angle_alpha_influence = value;
    }
    if let Some(value) = document.min_sample_width_px {
        profile.min_sample_width_px = value;
    }
    if let Some(value) = document.long_stroke_detail_crispness {
        profile.long_stroke_detail_crispness = value;
    }
    if let Some(value) = document.hand_arc_length_min {
        profile.hand_arc_length_min = value;
    }
    if let Some(value) = document.hand_arc_length_max {
        profile.hand_arc_length_max = value;
    }
    if let Some(value) = document.hand_arc_scale {
        profile.hand_arc_scale = value;
    }
    if let Some(value) = document.preferred_length_floor_px {
        profile.preferred_length_floor_px = value;
    }
    if let Some(value) = document.primary_noise_frequency_scale {
        profile.primary_noise_frequency_scale = value;
    }
    if let Some(value) = document.hand_arc_noise_frequency_scale {
        profile.hand_arc_noise_frequency_scale = value;
    }
    if let Some(value) = document.hand_arc_noise_phase {
        profile.hand_arc_noise_phase = value;
    }
    if let Some(value) = document.tangent_drift_noise_frequency_scale {
        profile.tangent_drift_noise_frequency_scale = value;
    }
    if let Some(value) = document.tangent_drift_noise_phase {
        profile.tangent_drift_noise_phase = value;
    }
    if let Some(value) = document.micro_noise_frequency_scale {
        profile.micro_noise_frequency_scale = value;
    }
    if let Some(value) = document.micro_noise_phase {
        profile.micro_noise_phase = value;
    }
    if let Some(value) = document.width_noise_frequency_scale {
        profile.width_noise_frequency_scale = value;
    }
    if let Some(value) = document.width_noise_phase {
        profile.width_noise_phase = value;
    }
    if let Some(value) = document.bow_min_length_px {
        profile.bow_min_length_px = value;
    }
    if let Some(value) = document.bow_preferred_min_px {
        profile.bow_preferred_min_px = value;
    }
    if let Some(value) = document.bow_length_min {
        profile.bow_length_min = value;
    }
    if let Some(value) = document.bow_length_max {
        profile.bow_length_max = value;
    }
    if let Some(value) = document.bow_wobble_floor_px {
        profile.bow_wobble_floor_px = value;
    }
    if let Some(value) = document.bow_scale {
        profile.bow_scale = value;
    }
    if let Some(value) = document.bow_non_feature_factor {
        profile.bow_non_feature_factor = value;
    }
    if let Some(value) = document.bow_max_px {
        profile.bow_max_px = value;
    }
}

fn npr_candidate_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprCandidateStrategy3d> {
    match value.trim() {
        "geometry_edges" => Ok(amigo_render_api::NprCandidateStrategy3d::GeometryEdges),
        "character_semantic" => Ok(amigo_render_api::NprCandidateStrategy3d::CharacterSemantic),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.candidate_strategy `{other}`; expected `geometry_edges` or `character_semantic`"
            ),
        }),
    }
}

fn npr_path_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprPathStrategy3d> {
    match value.trim() {
        "stable_stroked_paths" => Ok(amigo_render_api::NprPathStrategy3d::StableStrokedPaths),
        "direct_visible_segments" => Ok(amigo_render_api::NprPathStrategy3d::DirectVisibleSegments),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.path_strategy `{other}`; expected `stable_stroked_paths` or `direct_visible_segments`"
            ),
        }),
    }
}

fn npr_stroke_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprStrokeStrategy3d> {
    match value.trim() {
        "comic_ink" => Ok(amigo_render_api::NprStrokeStrategy3d::ComicInk),
        "akira_ink" => Ok(amigo_render_api::NprStrokeStrategy3d::AkiraInk),
        "confident_manga_ink" => Ok(amigo_render_api::NprStrokeStrategy3d::ConfidentMangaInk),
        "technical_ink" => Ok(amigo_render_api::NprStrokeStrategy3d::TechnicalInk),
        "rough_pencil" => Ok(amigo_render_api::NprStrokeStrategy3d::RoughPencil),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.stroke_strategy `{other}`; expected `comic_ink`, `akira_ink`, `confident_manga_ink`, `technical_ink`, or `rough_pencil`"
            ),
        }),
    }
}

fn npr_ink_fill_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprInkFillStrategy3d> {
    match value.trim() {
        "none" => Ok(amigo_render_api::NprInkFillStrategy3d::None),
        "material_black_mass" => Ok(amigo_render_api::NprInkFillStrategy3d::MaterialBlackMass),
        "binary_manga_shadow" => Ok(amigo_render_api::NprInkFillStrategy3d::BinaryMangaShadow),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.fill_strategy `{other}`; expected `none`, `material_black_mass`, or `binary_manga_shadow`"
            ),
        }),
    }
}

fn npr_hatching_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprHatchingStrategy3d> {
    match value.trim() {
        "none" => Ok(amigo_render_api::NprHatchingStrategy3d::None),
        "sparse_character_hatching" => {
            Ok(amigo_render_api::NprHatchingStrategy3d::SparseCharacterHatching)
        }
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.hatching_strategy `{other}`; expected `none` or `sparse_character_hatching`"
            ),
        }),
    }
}

fn npr_budget_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprBudgetStrategy3d> {
    match value.trim() {
        "edge_visibility" => Ok(amigo_render_api::NprBudgetStrategy3d::EdgeVisibility),
        "face_and_silhouette_priority" => {
            Ok(amigo_render_api::NprBudgetStrategy3d::FaceAndSilhouettePriority)
        }
        "character_readability" => Ok(amigo_render_api::NprBudgetStrategy3d::CharacterReadability),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.budget_strategy `{other}`; expected `edge_visibility`, `face_and_silhouette_priority`, or `character_readability`"
            ),
        }),
    }
}

fn npr_temporal_strategy_3d_from_document(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<amigo_render_api::NprTemporalStrategy3d> {
    match value.trim() {
        "path_history" => Ok(amigo_render_api::NprTemporalStrategy3d::PathHistory),
        "stable_arc_length" => Ok(amigo_render_api::NprTemporalStrategy3d::StableArcLength),
        other => Err(crate::SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!(
                "invalid Mesh3D.npr.pipeline.temporal_strategy `{other}`; expected `path_history` or `stable_arc_length`"
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
