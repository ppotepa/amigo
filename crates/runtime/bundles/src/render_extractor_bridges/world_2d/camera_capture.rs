use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

pub(super) fn update_camera_2d_capture(
    runtime: &Runtime,
    camera_service: &amigo_camera_core_plugin::CameraService,
    packet: &mut WgpuRenderFramePacket,
) {
    let depth_space = runtime
        .resolve::<amigo_2d_composition::RenderLayer2dSceneService>()
        .map(|service| service.depth_space())
        .unwrap_or_default();

    let (quality_settings, camera_motion) = if let Some(camera) = camera_service.main_camera2d() {
        let settings = camera_service.quality_profile_2d(&camera.id).settings();
        let camera_motion = camera_service.camera_depth_motion_2d(&camera.id);
        packet.set_active_camera_2d_entity(Some(camera.entity_name));
        packet.set_camera_debug_view_2d(camera_service.debug_view_2d(&camera.id));
        (settings, camera_motion)
    } else {
        (
            amigo_camera_core_plugin::CameraQualityProfile2d::default().settings(),
            amigo_camera_core_plugin::api::CameraDepthMotion2d::default(),
        )
    };

    let assets = runtime.resolve::<amigo_assets::AssetCatalog>();
    let camera_stacks =
        camera_service.frame_post_fx_stacks_for_depth_space(assets.as_deref(), depth_space);
    if !camera_stacks.is_empty() {
        let mut stacks = camera_stacks;
        stacks.extend(packet.post_fx_stacks().iter().cloned());
        packet.set_post_fx_stacks(stacks);
    }
    packet.set_light_sources_2d(super::super::light_sources_2d::collect_light_sources_2d(
        packet.renderables_2d(),
        packet.render_contributions_2d(),
        None,
    ));
    let camera_optical_candidates =
        super::super::light_sources_2d::collect_camera_optical_candidates_from_light_sources_2d(
            packet.world_2d_light_sources(),
        );
    packet.set_camera_capture_input_2d(build_camera_capture_input(
        runtime,
        packet,
        camera_optical_candidates.as_slice(),
        depth_space,
        camera_motion,
    ));
    packet.set_visual_source_flags_2d(build_visual_source_flags_2d(packet, quality_settings));
}

fn build_visual_source_flags_2d(
    packet: &WgpuRenderFramePacket,
    quality_settings: amigo_camera_core_plugin::CameraQualitySettings2d,
) -> amigo_render_wgpu::WgpuVisualSourceFlags2d {
    let capture = packet.camera_capture_input_2d();
    let generate_visual = quality_settings.debug_buffers
        || quality_settings
            .visual_source_buffer_quality
            .should_generate()
        || quality_settings.generate_visual_source_buffers;
    let generate_motion = quality_settings.debug_buffers
        || quality_settings.motion_source_quality.should_generate()
        || quality_settings.generate_motion_debug_source;
    let generate_layer_mask = quality_settings.debug_buffers
        || quality_settings.layer_mask_quality.should_generate()
        || quality_settings.generate_layer_mask_debug_source;
    amigo_render_wgpu::WgpuVisualSourceFlags2d {
        layer_mask_generated: generate_layer_mask && !packet.world_2d_render_layers().is_empty(),
        layer_roles_generated: generate_layer_mask && !packet.world_2d_render_layers().is_empty(),
        scene_normal_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneNormal,
        ) && generate_visual,
        scene_wetness_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneWetness,
        ) && generate_visual,
        scene_highlight_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneHighlight,
        ) && generate_visual,
        scene_emissive_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneEmissive,
        ) && generate_visual,
        scene_motion_generated: is_produced(
            capture,
            amigo_render_api::VisualSourceKind2d::SceneMotion,
        ) && generate_motion,
    }
}

fn is_produced(
    input: Option<&amigo_render_api::CameraCaptureInput2d>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> bool {
    input
        .and_then(|input| input.source(kind))
        .is_some_and(|source| {
            source.availability == amigo_render_api::VisualSourceAvailability2d::Produced
        })
}

pub(super) fn build_camera_capture_input(
    runtime: &Runtime,
    packet: &WgpuRenderFramePacket,
    camera_optical_candidates: &[amigo_camera_optics_plugin::api::CameraOpticalCandidate2d],
    depth_space: amigo_2d_spatial::DepthSpace2d,
    camera_motion: amigo_camera_core_plugin::api::CameraDepthMotion2d,
) -> amigo_render_api::CameraCaptureInput2d {
    let depth_space = depth_space.normalized();
    let camera_motion = camera_motion.normalized();
    let layers = packet
        .world_2d_render_layers()
        .iter()
        .map(|layer| {
            let base_z_depth = match layer.depth.mode {
                amigo_2d_composition::RenderDepthMode2d::Distance => layer
                    .depth
                    .distance_m
                    .map(|distance_m| {
                        amigo_2d_spatial::distance_to_z_depth(distance_m, depth_space)
                    })
                    .unwrap_or(layer.depth.z_depth)
                    .clamp(0.0, 1.0),
                amigo_2d_composition::RenderDepthMode2d::Infinity => 0.0,
                amigo_2d_composition::RenderDepthMode2d::Overlay => 1.0,
                _ => layer.depth.z_depth.clamp(0.0, 1.0),
            };
            let effective_distance_m = match layer.depth.mode {
                amigo_2d_composition::RenderDepthMode2d::Distance => {
                    layer.depth.distance_m.and_then(|distance_m| {
                        amigo_focus_depth_plugin::runtime::effective_layer_distance_m(
                            Some(distance_m),
                            &camera_motion,
                        )
                    })
                }
                _ => None,
            };
            let effective_z_depth = match layer.depth.mode {
                amigo_2d_composition::RenderDepthMode2d::Distance => effective_distance_m
                    .map(|distance_m| {
                        amigo_2d_spatial::distance_to_z_depth(distance_m, depth_space)
                    })
                    .unwrap_or(base_z_depth)
                    .clamp(0.0, 1.0),
                amigo_2d_composition::RenderDepthMode2d::Infinity => 0.0,
                amigo_2d_composition::RenderDepthMode2d::Overlay => base_z_depth,
                _ => base_z_depth,
            };
            let z_depth = effective_z_depth;
            amigo_render_api::ResolvedLayerOptics2d {
                layer_id: layer.id.clone(),
                role: layer.optical_role,
                depth_mode: depth_mode_label(layer.depth.mode).to_owned(),
                distance_m: layer.depth.distance_m,
                z_depth,
                base_z_depth,
                effective_z_depth,
                effective_distance_m,
                blur_scale: layer.depth.blur_scale,
                camera_motion_scale: amigo_2d_spatial::z_depth_to_camera_motion_scale(z_depth),
            }
        })
        .collect();
    let mut builder = amigo_render_api::CameraCaptureInput2dBuilder::new(depth_space, layers)
        .with_depth("world.depth");
    if !packet.world_2d_render_layers().is_empty() {
        builder = builder.with_layer_mask("world.layer_mask");
    }
    if targets_scene_highlight_buffer(runtime, camera_optical_candidates) {
        // Produced by authored visual maps or active CameraOpticalCandidate2d targets.
        // CameraOptics consumes this semantic buffer before camera post-fx.
        builder = builder.with_highlight_produced("world.highlight");
    }
    if targets_scene_emissive_buffer(runtime, camera_optical_candidates) {
        // Produced by authored visual maps or active CameraOpticalCandidate2d targets.
        // CameraOptics consumes this semantic buffer before camera post-fx.
        builder = builder.with_emissive_produced("world.emissive");
    }
    if should_produce_scene_normal(runtime, packet) {
        // Current limitation: produced by authored visual maps and wet-reflection asset placeholder.
        // Final target: dedicated material pass writes this buffer before camera post-fx.
        builder = builder.with_normal_produced("world.normal");
    } else if let Some(normal) = wetness_normal_source(packet.post_fx_stacks()) {
        builder = builder.with_normal_asset(normal);
    }
    if should_produce_scene_wetness(runtime, packet) {
        // Current limitation: produced by authored visual maps and wet-reflection asset placeholder.
        // Final target: dedicated material pass writes this buffer before camera post-fx.
        builder = builder.with_wetness_produced("world.wetness");
    } else if let Some(mask) = wetness_mask_source(packet.post_fx_stacks()) {
        builder = builder.with_wetness_asset(mask);
    }
    if motion_source(packet.post_fx_stacks()).is_some() {
        // V1 limitation: produced from previous per-draw transform positions and shutter active state.
        // Final target: typed motion-vector source from motion/runtime systems.
        builder = builder.with_motion_produced("world.motion");
    }
    builder.build()
}

fn targets_scene_highlight_buffer(
    runtime: &Runtime,
    camera_optical_candidates: &[amigo_camera_optics_plugin::api::CameraOpticalCandidate2d],
) -> bool {
    amigo_camera_optics_plugin::render::targets_scene_highlight_buffer(
        has_visual_map(
            runtime,
            amigo_render_api::VisualSourceKind2d::SceneHighlight,
        ),
        camera_optical_candidates,
    )
}

fn targets_scene_emissive_buffer(
    runtime: &Runtime,
    camera_optical_candidates: &[amigo_camera_optics_plugin::api::CameraOpticalCandidate2d],
) -> bool {
    amigo_camera_optics_plugin::render::targets_scene_emissive_buffer(
        has_visual_map(runtime, amigo_render_api::VisualSourceKind2d::SceneEmissive),
        camera_optical_candidates,
    )
}

fn should_produce_scene_normal(runtime: &Runtime, packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(runtime, amigo_render_api::VisualSourceKind2d::SceneNormal)
        || wetness_normal_source(packet.post_fx_stacks()).is_some()
}

fn should_produce_scene_wetness(runtime: &Runtime, packet: &WgpuRenderFramePacket) -> bool {
    has_visual_map(runtime, amigo_render_api::VisualSourceKind2d::SceneWetness)
        || wetness_mask_source(packet.post_fx_stacks()).is_some()
}

fn has_visual_map(runtime: &Runtime, kind: amigo_render_api::VisualSourceKind2d) -> bool {
    runtime
        .resolve::<amigo_sprite_2d_plugin::SpriteSceneService>()
        .map(|service| service.commands())
        .unwrap_or_default()
        .iter()
        .filter_map(|command| visual_map_for_kind(command.sprite.visual_maps.as_ref(), kind))
        .chain(
            runtime
                .resolve::<amigo_layered_image_2d_plugin::LayeredImageSceneService>()
                .map(|service| service.commands())
                .unwrap_or_default()
                .iter()
                .filter_map(|command| {
                    visual_map_for_kind(command.image.visual_maps.as_ref(), kind).or_else(|| {
                        command
                            .image
                            .layer_overrides
                            .iter()
                            .filter_map(|override_| {
                                visual_map_for_kind(override_.visual_maps.as_ref(), kind)
                            })
                            .next()
                    })
                }),
        )
        .next()
        .is_some()
}

fn visual_map_for_kind(
    maps: Option<&amigo_scene::VisualMaps2dSceneCommand>,
    kind: amigo_render_api::VisualSourceKind2d,
) -> Option<&amigo_assets::AssetKey> {
    let maps = maps?;
    match kind {
        amigo_render_api::VisualSourceKind2d::SceneNormal => maps.normal.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneWetness => maps.wetness.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneEmissive => maps.emissive.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneHighlight => maps.highlight.as_ref(),
        amigo_render_api::VisualSourceKind2d::SceneColor
        | amigo_render_api::VisualSourceKind2d::SceneDepth
        | amigo_render_api::VisualSourceKind2d::LayerMask
        | amigo_render_api::VisualSourceKind2d::SceneMotion
        | amigo_render_api::VisualSourceKind2d::Debug => None,
    }
}

fn wetness_normal_source(stacks: &[amigo_composite_plugin::ScopedPostFx2dStack]) -> Option<&str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| {
            let wet = effect.effect.as_wet_reflections()?;
            (wet.is_active()
                && wet
                    .noise_normal
                    .as_deref()
                    .is_some_and(|path| !path.trim().is_empty()))
            .then(|| wet.noise_normal.as_deref())
            .flatten()
        })
}

fn wetness_mask_source(stacks: &[amigo_composite_plugin::ScopedPostFx2dStack]) -> Option<&str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| {
            let wet = effect.effect.as_wet_reflections()?;
            (wet.is_active() && !wet.reflection_mask.trim().is_empty())
                .then_some(wet.reflection_mask.as_str())
        })
}

fn motion_source(stacks: &[amigo_composite_plugin::ScopedPostFx2dStack]) -> Option<&'static str> {
    stacks
        .iter()
        .flat_map(|stack| stack.effects.iter())
        .find_map(|effect| {
            effect
                .effect
                .as_shutter_blur()
                .filter(|shutter| shutter.is_active())
                .map(|_| "camera.shutter_history.motion")
        })
}

fn depth_mode_label(mode: amigo_2d_composition::RenderDepthMode2d) -> &'static str {
    match mode {
        amigo_2d_composition::RenderDepthMode2d::DepthMap => "depth_map",
        amigo_2d_composition::RenderDepthMode2d::Distance => "distance",
        amigo_2d_composition::RenderDepthMode2d::ZDepth => "z_depth",
        amigo_2d_composition::RenderDepthMode2d::Infinity => "infinity",
        amigo_2d_composition::RenderDepthMode2d::Overlay => "overlay",
    }
}
