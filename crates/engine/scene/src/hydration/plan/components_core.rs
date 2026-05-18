fn hydrate_component_core(
    source_mod: &str,
    document: &SceneDocument,
    entity: &crate::SceneEntityDocument,
    entity_name: &String,
    component_index: usize,
    component: &SceneComponentDocument,
    commands: &mut Vec<SceneCommand>,
) -> SceneDocumentResult<bool> {
    match component {
        SceneComponentDocument::Camera2d {
            id,
            mode,
            render_contributions,
            exposure,
            shutter,
            lens,
            lens_surface,
            film,
            look,
            aperture,
        } => {
            commands.push(SceneCommand::QueueCamera2d {
                command: Camera2dSceneCommand {
                    source_mod: source_mod.to_string(),
                    entity_name: entity_name.to_string(),
                    camera_id: id.clone(),
                    mode: camera_mode_from_document(*mode),
                    render_contributions: RenderContributions2dSceneCommand {
                        roles: render_contributions
                            .clone()
                            .with_defaults(camera_render_contribution_defaults())
                            .into_roles(),
                    },
                    exposure: CameraExposure2dSceneCommand {
                        iso: exposure.iso,
                        compensation: exposure.compensation,
                        white_balance: exposure.white_balance,
                        nd_stops: exposure.nd_stops,
                        auto: CameraAutoExposure2dSceneCommand {
                            target_luma: exposure.auto.target_luma,
                            adaptation_speed: exposure.auto.adaptation_speed,
                            min_iso: exposure.auto.min_iso,
                            max_iso: exposure.auto.max_iso,
                        },
                    },
                    shutter: CameraShutter2dSceneCommand {
                        enabled: shutter.enabled,
                        fps: shutter.fps,
                        angle: shutter.angle,
                        opacity: shutter.opacity,
                        history_mix: shutter.history_mix,
                        history_mix_2: shutter.history_mix_2,
                        edge_rejection: shutter.edge_rejection,
                        luma_threshold: shutter.luma_threshold,
                        frame_hold: shutter.frame_hold,
                    },
                    lens: CameraLens2dSceneCommand {
                        profile: lens.profile.clone(),
                        intensity: lens.intensity,
                        aberration_px: lens.aberration_px,
                        distortion: lens.distortion,
                        vignette: lens.vignette,
                        edge_softness_px: lens.edge_softness_px,
                        flare_strength: lens.flare_strength,
                        dirt: lens.dirt,
                        focal_length_mm: lens.focal_length_mm,
                        lens_bloom: lens.lens_bloom,
                        flare_ghosts: lens.flare_ghosts,
                        anamorphic_squeeze: lens.anamorphic_squeeze,
                        coma: lens.coma,
                        cat_eye_bokeh: lens.cat_eye_bokeh,
                        focus_breathing: lens.focus_breathing,
                    },
                    lens_surface: CameraLensSurface2dSceneCommand {
                        rain_profile: lens_surface.rain_profile.clone(),
                    },
                    film: CameraFilm2dSceneCommand {
                        profile: film.profile.clone(),
                        intensity: film.intensity,
                        seed: film.seed,
                        color_shift: film.color_shift,
                        contrast: film.contrast,
                        saturation: film.saturation,
                        flicker: film.flicker,
                        vignette: film.vignette,
                        toe: film.toe,
                        shoulder: film.shoulder,
                        black_lift: film.black_lift,
                        print_fade: film.print_fade,
                        dust: film.dust,
                        scratches: film.scratches,
                        push_pull: film.push_pull,
                        gate_weave: film.gate_weave,
                        scan_softness: film.scan_softness,
                    },
                    look: CameraLook2dSceneCommand {
                        profile: look.profile.clone(),
                        intensity: look.intensity,
                    },
                    aperture: CameraAperture2dSceneCommand {
                        enabled: aperture.enabled,
                        f_stop: aperture.f_stop,
                        focus_distance_m: aperture.focus_distance_m,
                        focus: camera_focus_from_document(&aperture.focus),
                        depth_of_field: CameraDepthOfField2dSceneCommand {
                            depth_map: aperture.depth_of_field.depth_map.clone(),
                            affected_layers: aperture.depth_of_field.affected_layers.clone(),
                            max_blur_px: aperture.depth_of_field.max_blur_px,
                            depth_contrast: aperture.depth_of_field.depth_contrast,
                            focus_width: aperture.depth_of_field.focus_width,
                            foreground_blur_boost: aperture.depth_of_field.foreground_blur_boost,
                            background_blur_boost: aperture.depth_of_field.background_blur_boost,
                            edge_aware: aperture.depth_of_field.edge_aware,
                            invert_depth: aperture.depth_of_field.invert_depth,
                            debug_view: aperture.depth_of_field.debug_view.clone(),
                            aperture_blades: aperture.depth_of_field.aperture_blades,
                            aperture_roundness: aperture.depth_of_field.aperture_roundness,
                            aperture_rotation_degrees: aperture
                                .depth_of_field
                                .aperture_rotation_degrees,
                            sample_count: aperture.depth_of_field.sample_count,
                            highlight_threshold: aperture.depth_of_field.highlight_threshold,
                            highlight_knee: aperture.depth_of_field.highlight_knee,
                            highlight_gain: aperture.depth_of_field.highlight_gain,
                            highlight_saturation: aperture.depth_of_field.highlight_saturation,
                        },
                    },
                },
            });
        }
        SceneComponentDocument::Camera3d | SceneComponentDocument::Light3d { .. } => {}
        SceneComponentDocument::Sprite2d {
            render_layer,
            texture,
            size,
            sheet,
            animation,
            visual_maps,
            render_contributions,
            material,
            z_index,
            post_fx: _,
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
                    visual_maps: visual_maps.as_ref().map(visual_maps_from_document),
                    render_contributions: RenderContributions2dSceneCommand {
                        roles: render_contributions
                            .clone()
                            .with_defaults(sprite_render_contribution_defaults())
                            .into_roles(),
                    },
                    material: material2d_scene_command(*material),
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
            visual_maps,
            z_index,
            layer_overrides,
            post_fx: _,
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
                    visual_maps: visual_maps.as_ref().map(visual_maps_from_document),
                    z_index: *z_index,
                    transform: transform2_for_entity(entity),
                    layer_overrides: layer_overrides
                        .iter()
                        .map(|item| LayeredImageLayerOverrideSceneCommand {
                            id: item.id.clone(),
                            opacity: item.opacity,
                            enabled: item.enabled,
                            blend_mode: item.blend.map(layered_image_blend_from_document),
                            visual_maps: item.visual_maps.as_ref().map(visual_maps_from_document),
                        })
                        .collect(),
                },
            });
        }
        SceneComponentDocument::DepthMap2d {
            id,
            asset,
            size,
            viewport_fit,
            white_is_near,
            z_index,
        } => {
            commands.push(SceneCommand::QueueDepthMap2d {
                command: DepthMap2dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    id: id.clone(),
                    asset: AssetKey::new(asset.clone()),
                    size: vec2_from_document(*size),
                    viewport_fit: depth_map_viewport_fit_from_document(*viewport_fit),
                    white_is_near: *white_is_near,
                    z_index: *z_index,
                    transform: transform2_for_entity(entity),
                },
            });
        }
        SceneComponentDocument::DepthAuxMap2d {
            id,
            asset,
            surface_asset,
            size,
            viewport_fit,
            channels,
            z_index,
        } => {
            commands.push(SceneCommand::QueueDepthAuxMap2d {
                command: DepthAuxMap2dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    id: id.clone(),
                    asset: AssetKey::new(asset.clone()),
                    surface_asset: surface_asset.clone().map(AssetKey::new),
                    size: vec2_from_document(*size),
                    viewport_fit: depth_map_viewport_fit_from_document(*viewport_fit),
                    channels: depth_aux_channels_from_document(channels),
                    z_index: *z_index,
                    transform: transform2_for_entity(entity),
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
            post_fx: _,
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
            style,
            render_contributions,
            z_index,
            material,
            post_fx,
        } => {
            commands.push(SceneCommand::QueueText2d {
                command: Text2dSceneCommand {
                    source_mod: source_mod.to_owned(),
                    entity_name: entity_name.clone(),
                    render_layer: render_layer.clone(),
                    content: content.clone(),
                    font: AssetKey::new(font.clone()),
                    bounds: vec2_from_document(*bounds),
                    style: text2d_style_from_document(
                        style,
                        &document.scene.id,
                        &entity.id,
                        component.kind(),
                    )?,
                    render_contributions: RenderContributions2dSceneCommand {
                        roles: render_contributions.clone().into_roles(),
                    },
                    post_fx_host_id: (!post_fx.is_empty()).then(|| {
                        component_post_fx_host_id(&entity.id, component_index, component.kind())
                    }),
                    z_index: *z_index,
                    material: material2d_scene_command(*material),
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
            render_contributions,
            material,
            z_index,
            post_fx: _,
        } => {
            let stroke_color = stroke_color
                .as_deref()
                .map(|value| {
                    parse_color_rgba_hex(value, &document.scene.id, &entity.id, component.kind())
                })
                .transpose()?
                .unwrap_or(ColorRgba::WHITE);
            let fill_color = fill_color
                .as_deref()
                .map(|value| {
                    parse_color_rgba_hex(value, &document.scene.id, &entity.id, component.kind())
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
            command.render_contributions = RenderContributions2dSceneCommand {
                roles: render_contributions
                    .clone()
                    .with_defaults(vector_render_contribution_defaults())
                    .into_roles(),
            };
            command.material = material2d_scene_command(*material);
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
            glow_strength,
            beam_enabled,
            beam_length_px,
            beam_width_degrees,
            beam_strength,
            aberration_px,
            flare_length_px,
            flare_strength,
            bloom,
            lens_influence,
            depth,
            z_depth,
            z_index,
            render_contributions,
            enabled,
            viewport_fit,
            viewport_canvas_size,
            post_fx: _,
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
                    glow_strength: (*glow_strength).max(0.0),
                    beam_enabled: *beam_enabled,
                    beam_length_px: (*beam_length_px).max(0.0),
                    beam_width_degrees: beam_width_degrees.clamp(1.0, 179.0),
                    beam_strength: (*beam_strength).max(0.0),
                    aberration_px: (*aberration_px).max(0.0),
                    flare_length_px: (*flare_length_px).max(0.0),
                    flare_strength: (*flare_strength).max(0.0),
                    bloom: (*bloom).max(0.0),
                    lens_influence: (*lens_influence).max(0.0),
                    depth: depth.as_ref().map(|depth| {
                        render_depth_from_document(
                            depth,
                            document.visual2d.spatial.depth_space.to_runtime(),
                        )
                    }),
                    z_depth: depth
                        .as_ref()
                        .map(|depth| {
                            render_depth_from_document(
                                depth,
                                document.visual2d.spatial.depth_space.to_runtime(),
                            )
                            .z_depth
                        })
                        .or_else(|| z_depth.map(|value| value.clamp(0.0, 1.0))),
                    z_index: *z_index,
                    render_contributions: RenderContributions2dSceneCommand {
                        roles: render_contributions
                            .clone()
                            .with_defaults(beacon_render_contribution_defaults())
                            .into_roles(),
                    },
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

fn camera_render_contribution_defaults() -> [(&'static str, bool); 9] {
    [
        ("camera.projection", true),
        ("camera.exposure", false),
        ("camera.shutter", false),
        ("camera.optics", false),
        ("camera.focus_blur", false),
        ("camera.lens_surface", false),
        ("camera.film", false),
        ("camera.look", false),
        ("camera.scan_output", false),
    ]
}

fn beacon_render_contribution_defaults() -> [(&'static str, bool); 4] {
    [
        ("overlay.visible", true),
        ("relight.plate", true),
        ("bloom.source", true),
        ("camera.fx_source", true),
    ]
}

fn sprite_render_contribution_defaults() -> [(&'static str, bool); 6] {
    [
        ("world.color", true),
        ("material.mask", false),
        ("optics.refract", false),
        ("transmission.source", false),
        ("bloom.source", false),
        ("camera.fx_source", false),
    ]
}

fn vector_render_contribution_defaults() -> [(&'static str, bool); 6] {
    sprite_render_contribution_defaults()
}

fn camera_mode_from_document(mode: Camera2dModeDocument) -> CameraExposureMode2dSceneCommand {
    match mode {
        Camera2dModeDocument::Auto => CameraExposureMode2dSceneCommand::Auto,
        Camera2dModeDocument::Manual => CameraExposureMode2dSceneCommand::Manual,
    }
}

fn camera_focus_from_document(focus: &CameraFocus2dDocument) -> CameraFocus2dSceneCommand {
    match focus {
        CameraFocus2dDocument::None => CameraFocus2dSceneCommand::None,
        CameraFocus2dDocument::RenderLayer { layer } => CameraFocus2dSceneCommand::RenderLayer {
            layer: layer.clone(),
        },
        CameraFocus2dDocument::SceneObject { object } => CameraFocus2dSceneCommand::SceneObject {
            object: object.clone(),
        },
        CameraFocus2dDocument::Distance { distance_m } => CameraFocus2dSceneCommand::Distance {
            distance_m: distance_m.max(0.0),
        },
        CameraFocus2dDocument::Depth { value } => {
            CameraFocus2dSceneCommand::Depth { value: *value }
        }
    }
}

fn text2d_style_from_document(
    style: &Text2dStyleDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<Text2dStyleSceneCommand> {
    let base_color = style
        .color
        .as_deref()
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()?
        .unwrap_or_else(|| ColorRgba::new(1.0, 0.96, 0.82, 1.0));
    let opacity = style.opacity.unwrap_or(1.0).clamp(0.0, 1.0);

    Ok(Text2dStyleSceneCommand {
        color: ColorRgba::new(
            base_color.r,
            base_color.g,
            base_color.b,
            base_color.a * opacity,
        ),
        opacity,
        font_size: style
            .font_size
            .filter(|value| value.is_finite())
            .map(|value| value.max(1.0)),
        align: match style.align {
            Text2dAlignDocument::Left => Text2dAlignSceneCommand::Left,
            Text2dAlignDocument::Center => Text2dAlignSceneCommand::Center,
            Text2dAlignDocument::Right => Text2dAlignSceneCommand::Right,
        },
        blend: match style.blend {
            Text2dBlendModeDocument::Alpha => Text2dBlendModeSceneCommand::Alpha,
            Text2dBlendModeDocument::Additive => Text2dBlendModeSceneCommand::Additive,
            Text2dBlendModeDocument::Multiply => Text2dBlendModeSceneCommand::Multiply,
            Text2dBlendModeDocument::Screen => Text2dBlendModeSceneCommand::Screen,
        },
        shadow: style
            .shadow
            .as_ref()
            .map(|shadow| {
                Ok(Text2dShadowSceneCommand {
                    color: parse_color_rgba_hex(
                        &shadow.color,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?,
                    offset: vec2_from_document(shadow.offset),
                })
            })
            .transpose()?,
        outline: style
            .outline
            .as_ref()
            .map(|outline| {
                Ok(Text2dOutlineSceneCommand {
                    color: parse_color_rgba_hex(
                        &outline.color,
                        scene_id,
                        entity_id,
                        component_kind,
                    )?,
                    width: outline.width.max(0.0),
                })
            })
            .transpose()?,
        glow: style
            .glow
            .as_ref()
            .map(|glow| {
                Ok(Text2dGlowSceneCommand {
                    color: parse_color_rgba_hex(&glow.color, scene_id, entity_id, component_kind)?,
                    radius: glow.radius.max(0.0),
                    intensity: glow.intensity.max(0.0),
                    passes: glow.passes.max(1),
                })
            })
            .transpose()?,
    })
}

fn material2d_scene_command(material: Option<Material2dDocument>) -> Option<Material2dSceneCommand> {
    material.map(|material| Material2dSceneCommand {
        optical: Material2dOpticalSceneCommand {
            mode: match material.optical.mode {
                Material2dOpticalModeDocument::Opaque => Material2dOpticalModeSceneCommand::Opaque,
                Material2dOpticalModeDocument::Transmissive => {
                    Material2dOpticalModeSceneCommand::Transmissive
                }
                Material2dOpticalModeDocument::Refractive => {
                    Material2dOpticalModeSceneCommand::Refractive
                }
                Material2dOpticalModeDocument::Emissive => {
                    Material2dOpticalModeSceneCommand::Emissive
                }
            },
            transmission: material.optical.transmission,
            refraction_px: material.optical.refraction_px,
            distortion: material.optical.distortion,
            dispersion: material.optical.dispersion,
            roughness: material.optical.roughness,
            edge_boost: material.optical.edge_boost,
        },
        lighting: Material2dLightingSceneCommand {
            receives_light: material.lighting.receives_light,
            response: material.lighting.response,
        },
        camera_response: Material2dCameraResponseSceneCommand {
            highlight: material.camera_response.highlight,
            bloom_source: material.camera_response.bloom_source,
            rain_glass_affects: material.camera_response.rain_glass_affects,
        },
    })
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

fn depth_map_viewport_fit_from_document(
    fit: LayeredImageViewportFit2dDocument,
) -> DepthMapViewportFit2dSceneCommand {
    match fit {
        LayeredImageViewportFit2dDocument::Fixed => DepthMapViewportFit2dSceneCommand::Fixed,
        LayeredImageViewportFit2dDocument::Stretch => DepthMapViewportFit2dSceneCommand::Stretch,
        LayeredImageViewportFit2dDocument::Contain => DepthMapViewportFit2dSceneCommand::Contain,
        LayeredImageViewportFit2dDocument::Cover => DepthMapViewportFit2dSceneCommand::Cover,
    }
}

fn depth_aux_channels_from_document(
    channels: &DepthAuxMap2dChannelsDocument,
) -> DepthAuxMap2dChannelsSceneCommand {
    DepthAuxMap2dChannelsSceneCommand {
        r: channels.r.clone(),
        g: channels.g.clone(),
        b: channels.b.clone(),
        a: channels.a.clone(),
    }
}

fn visual_maps_from_document(maps: &VisualMaps2dDocument) -> VisualMaps2dSceneCommand {
    VisualMaps2dSceneCommand {
        normal: maps.normal.clone().map(AssetKey::new),
        wetness: maps.wetness.clone().map(AssetKey::new),
        emissive: maps.emissive.clone().map(AssetKey::new),
        highlight: maps.highlight.clone().map(AssetKey::new),
        roughness: maps.roughness,
    }
}

fn lightmap_source_ref_from_document(
    source: &LightMap2dSourceRefDocument,
) -> LightMap2dSourceRefSceneCommand {
    match source {
        LightMap2dSourceRefDocument::LayeredImage2d { entity } => LightMap2dSourceRefSceneCommand {
            kind: LightMap2dSourceKindSceneCommand::LayeredImage2d,
            entity_name: entity.clone(),
        },
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
