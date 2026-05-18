mod graph;
mod services;

#[cfg(test)]
mod tests;

use super::*;
use amigo_session::RuntimeSession;

pub(crate) use amigo_render_api::RenderCompositionDiagnosticsService;
pub(crate) use amigo_render_api::RenderFrameStats;
pub(crate) use amigo_render_api::RenderFrameStatsService;
#[cfg(test)]
pub(crate) use amigo_render_wgpu::WgpuRenderFramePacket;
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionBuilder;
#[cfg(test)]
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionOptions;
pub(crate) use graph::{AppFrameGraphBuildInfo, build_frame_graph_from_plan};
pub(crate) use services::{
    build_depth_map2d_scene_service_from_packet, build_global_light2d_scene_service_from_packet,
    build_layered_image_scene_service_from_packet, build_light_route2d_scene_service_from_packet,
    build_lightmap2d_scene_service_from_packet, build_render_layer2d_scene_service_from_packet,
    build_sprite_scene_service_from_packet, build_text2d_scene_service_from_packet,
    build_tilemap_scene_service_from_packet, build_vector_scene_service_from_packet,
};

#[cfg(test)]
pub(crate) use services::{
    build_material_scene_service_from_packet, build_mesh_scene_service_from_packet,
    build_text3d_scene_service_from_packet,
};

#[derive(Debug, Clone, Copy)]
struct CameraFocusPlanInfo {
    base_focus_distance_m: Option<f32>,
    effective_focus_distance_m: Option<f32>,
    camera_z_m: f32,
    focus_residual_m: f32,
    dolly_signal: f32,
    computed_focus_z_depth: Option<f32>,
    focus_width: f32,
    f_stop: f32,
    max_blur_px: f32,
}

#[cfg(test)]
#[allow(dead_code)]
pub(crate) fn build_render_frame_for_session(
    session: &RuntimeSession,
    surface: &mut WgpuSurfaceState,
    renderer: &mut WgpuSceneRenderer,
) -> AmigoResult<()> {
    let runtime = session.runtime();
    let scene = required::<SceneService>(runtime)?;
    let assets = required::<AssetCatalog>(runtime)?;
    let particles = required::<Particle2dSceneService>(runtime)?;
    let debug_overlay_service = required::<crate::debug_overlay::DebugOverlayService>(runtime)?;

    let surface_size = surface.size();
    if let Ok(ui_viewport_state) =
        required::<amigo_runtime_bundles::amigo_ui::UiInputViewportState>(runtime)
    {
        ui_viewport_state.set(Some(amigo_render_wgpu::UiViewportSize::new(
            surface_size.width as f32,
            surface_size.height as f32,
        )));
    }

    session.begin_render_frame_extract();
    let mut render_packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(runtime);
    amigo_editor_ingame::append_editor_overlay(runtime, &mut render_packet);
    session.complete_render_frame_extract();

    let editor_game_viewport =
        editor_game_viewport_placement(runtime, surface_size.width, surface_size.height);
    let editor_surface_overlay = editor_game_viewport.is_some();

    session.begin_render_composition();
    let composition_plan = if editor_surface_overlay {
        WgpuFrameCompositionBuilder::build_with_options(
            &render_packet,
            WgpuFrameCompositionOptions {
                debug_overlay_after_present: true,
            },
        )
    } else {
        WgpuFrameCompositionBuilder::build(&render_packet)
    };
    session.complete_render_composition();
    let frame_graph = {
        let graph = build_frame_graph_from_plan(
            &composition_plan,
            AppFrameGraphBuildInfo {
                width: editor_game_viewport
                    .map(|placement| placement.logical_width)
                    .unwrap_or(surface_size.width),
                height: editor_game_viewport
                    .map(|placement| placement.logical_height)
                    .unwrap_or(surface_size.height),
            },
        );
        session.complete_render_graph_build();
        graph
    };

    if let Ok(render_diagnostics) = required::<RenderCompositionDiagnosticsService>(runtime) {
        render_diagnostics.set_with_camera_capture_focus_and_contributions(
            &composition_plan,
            &frame_graph,
            render_packet.camera_capture_input_2d().map(|input| {
                render_camera_capture_summary(input, render_packet.visual_source_flags_2d())
            }),
            render_packet.camera_capture_input_2d().map(|input| {
                render_camera_focus_plan_summary(
                    input,
                    camera_focus_for_input(runtime, assets.as_ref(), input),
                )
            }),
            render_camera_contributions_summary(
                runtime,
                assets.as_ref(),
                render_packet.camera_capture_input_2d(),
                amigo_runtime_bundles::wgpu_render_extractors::render_contribution_decisions_summary(
                    render_packet.world_2d_beacons(),
                ),
            ),
            None,
            Some(render_visual_items_summary(
                render_packet.renderables_2d(),
                render_packet.world_2d_render_layers(),
                render_packet
                    .camera_capture_input_2d()
                    .map(|input| input.layers.as_slice())
                    .unwrap_or(&[]),
            )),
        );
    }
    if let Ok(stats_service) = required::<RenderFrameStatsService>(runtime) {
        let previous = stats_service.snapshot();
        let stats = RenderFrameStats {
            frame_index: previous.frame_index + 1,
            window_width: surface_size.width,
            window_height: surface_size.height,
            world_2d_tilemaps: render_packet.world_2d_tilemaps().len(),
            world_2d_sprites: render_packet.world_2d_sprites().len(),
            world_2d_layered_images: render_packet.world_2d_layered_images().len(),
            world_2d_render_layers: render_packet.world_2d_render_layers().len(),
            world_2d_light_routes: render_packet.world_2d_light_routes().len(),
            world_2d_global_lights: render_packet.world_2d_global_lights().len(),
            world_2d_lightmaps: render_packet.world_2d_lightmaps().len(),
            world_2d_light_groups: render_packet.world_2d_light_groups().len(),
            world_2d_vectors: render_packet.world_2d_vectors().len(),
            world_2d_beacons: render_packet.world_2d_beacons().len(),
            world_2d_text: render_packet.world_2d_text().len(),
            world_2d_particles: render_packet.world_2d_particles().len(),
            world_3d_meshes: render_packet.world_3d_meshes().len(),
            world_3d_materials: render_packet.world_3d_materials().len(),
            world_3d_text: render_packet.world_3d_text().len(),
            game_ui_overlays: render_packet.game_ui_overlay().len(),
            debug_overlays: render_packet.debug_overlay().len(),
            ui_overlays: render_packet.all_overlay_count(),
            render_graph_nodes: frame_graph.nodes.len(),
            post_fx_effects: render_packet
                .post_fx_stacks()
                .iter()
                .map(|stack| stack.effects.len())
                .sum(),
        };
        stats_service.set(stats.clone());
        debug_overlay_service.record_render_frame(stats);
    }
    if let Ok(scheduling) = required::<amigo_session::RuntimeSchedulingService>(runtime) {
        debug_overlay_service.record_scheduling_stats(scheduling.stats());
    }
    if let Ok(audio_output) = required::<AudioOutputBackendService>(runtime) {
        let audio_snapshot = audio_output.snapshot();
        let (master_volume, active_sources, pending_commands, bus_count) =
            if let Ok(audio_state) = required::<AudioStateService>(runtime) {
                (
                    audio_state.master_volume(),
                    audio_state.playing_sources().len(),
                    audio_state.pending_runtime_commands().len(),
                    audio_state.bus_volumes().len(),
                )
            } else {
                (1.0, 0, 0, 0)
            };
        debug_overlay_service.record_audio_snapshot(
            audio_snapshot,
            master_volume,
            active_sources,
            pending_commands,
            bus_count,
        );
    }
    if let Ok(input_state) = required::<InputState>(runtime) {
        let pressed_keys = input_state
            .pressed_keys()
            .into_iter()
            .map(|key| format!("{key:?}"))
            .collect::<Vec<_>>();
        let backend_name = runtime
            .resolve::<InputServiceInfo>()
            .map(|info| info.backend_name.to_owned());
        let (active_map, active_actions) =
            if let Ok(actions) = required::<InputActionService>(runtime) {
                let active_map = actions.active_map_id();
                let active_actions = active_map
                    .as_deref()
                    .and_then(|map_id| actions.map(map_id))
                    .map(|map| {
                        let mut names = map
                            .actions
                            .keys()
                            .filter_map(|action| {
                                let name = action.as_str();
                                actions
                                    .down(input_state.as_ref(), name)
                                    .then(|| name.to_owned())
                            })
                            .collect::<Vec<_>>();
                        names.sort();
                        names
                    })
                    .unwrap_or_default();
                (active_map, active_actions)
            } else {
                (None, Vec::new())
            };
        debug_overlay_service.record_input_snapshot(
            backend_name,
            pressed_keys,
            active_map,
            active_actions,
        );
    }
    debug_overlay_service.record_particle_snapshot(
        particles.emitters().len(),
        particles
            .emitters()
            .iter()
            .filter(|emitter| particles.is_active(&emitter.entity_name))
            .count(),
    );

    let extracted_tilemaps = build_tilemap_scene_service_from_packet(&render_packet);
    let extracted_sprites = build_sprite_scene_service_from_packet(&render_packet);
    let extracted_layered_images = build_layered_image_scene_service_from_packet(&render_packet);
    let extracted_depth_maps = build_depth_map2d_scene_service_from_packet(&render_packet);
    let extracted_render_layers = build_render_layer2d_scene_service_from_packet(&render_packet);
    let extracted_light_routes = build_light_route2d_scene_service_from_packet(&render_packet);
    let extracted_global_lights = build_global_light2d_scene_service_from_packet(&render_packet);
    let extracted_lightmaps = build_lightmap2d_scene_service_from_packet(&render_packet);
    let extracted_text2d = build_text2d_scene_service_from_packet(&render_packet);
    let extracted_vectors = build_vector_scene_service_from_packet(&render_packet);

    if let Ok(post_fx_service) =
        required::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>(runtime)
    {
        let has_post_fx = !render_packet.post_fx_stacks().is_empty();
        let renderer_mode = if has_post_fx {
            "frame_graph_postfx"
        } else {
            "frame_graph"
        };
        post_fx_service.set_renderer_mode(renderer_mode);
    }

    let extracted_render_layer_commands = extracted_render_layers.commands();
    let extracted_light_route_commands = extracted_light_routes.commands();
    let emergency_overlay = emergency_overlay_lines(runtime);
    let render_request = amigo_render_wgpu::WgpuFrameRenderRequest {
        target: amigo_render_wgpu::WgpuFrameRenderTarget::Surface(surface),
        scene: scene.as_ref(),
        assets: assets.as_ref(),
        world_2d: amigo_render_wgpu::WgpuWorld2dRenderInput {
            renderables: render_packet.renderables_2d(),
            tilemaps: &extracted_tilemaps,
            sprites: &extracted_sprites,
            layered_images: &extracted_layered_images,
            depth_maps: &extracted_depth_maps,
            global_lights: &extracted_global_lights,
            lightmaps: &extracted_lightmaps,
            text2d: &extracted_text2d,
            vectors: &extracted_vectors,
            beacons: render_packet.world_2d_beacons(),
            render_layers: extracted_render_layer_commands.as_slice(),
            light_routes: extracted_light_route_commands.as_slice(),
            light_groups: render_packet.world_2d_light_groups(),
            particles: render_packet.world_2d_particles(),
        },
        world_3d: amigo_render_wgpu::WgpuWorld3dRenderInput {
            meshes: render_packet.world_3d_meshes(),
            materials: render_packet.world_3d_materials(),
            text3d: Some(render_packet.world_3d_text()),
        },
        game_ui: render_packet.game_ui_overlay(),
        debug_ui: render_packet.debug_overlay(),
        post_fx_stacks: render_packet.post_fx_stacks(),
        active_camera_2d_entity: render_packet.active_camera_2d_entity(),
        camera_capture_input_2d: render_packet.camera_capture_input_2d(),
        visual_source_flags_2d: Some(render_packet.visual_source_flags_2d()),
        camera_debug_view: render_packet
            .camera_debug_view_2d()
            .unwrap_or(amigo_render_api::CameraDebugView2d::FinalOutput),
        emergency_overlay: emergency_overlay.as_slice(),
        composition_plan: &composition_plan,
        frame_graph: &frame_graph,
        game_viewport: editor_game_viewport,
    };
    renderer.render_frame_request(render_request)?;
    if let Ok(render_diagnostics) = required::<RenderCompositionDiagnosticsService>(runtime) {
        render_diagnostics
            .set_plate_relight_summary(renderer.plate_relight_last_summary().to_owned());
        render_diagnostics
            .set_render_materials_summary(renderer.render_materials_last_summary().to_owned());
    }
    session.complete_render_submit();
    Ok(())
}

pub(crate) fn extract_game_frame_packet(
    session: &RuntimeSession,
    include_game_ui: bool,
) -> AmigoResult<amigo_render_wgpu::WgpuRenderFramePacket> {
    session.begin_render_frame_extract();
    let mut render_packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry()
        .extract_all(session.runtime());
    render_packet.clear_debug_overlay();
    if !include_game_ui {
        render_packet.clear_game_ui_overlay();
    }
    session.complete_render_frame_extract();
    Ok(render_packet)
}

pub(crate) fn extract_live_host_overlay_packet(
    session: &RuntimeSession,
) -> AmigoResult<amigo_render_wgpu::WgpuRenderFramePacket> {
    let mut render_packet = amigo_runtime_bundles::default_wgpu_render_extractor_registry()
        .extract_all(session.runtime());
    render_packet.clear_world_content();
    render_packet.clear_game_ui_overlay();
    amigo_editor_ingame::append_editor_overlay(session.runtime(), &mut render_packet);
    Ok(render_packet)
}

pub(crate) fn render_game_frame_to_cache(
    session: &RuntimeSession,
    target: &mut amigo_render_wgpu::WgpuOffscreenTarget,
    renderer: &mut WgpuSceneRenderer,
    include_game_ui: bool,
) -> AmigoResult<()> {
    let runtime = session.runtime();
    let scene = required::<SceneService>(runtime)?;
    let assets = required::<AssetCatalog>(runtime)?;
    let particles = required::<Particle2dSceneService>(runtime)?;
    let debug_overlay_service = required::<crate::debug_overlay::DebugOverlayService>(runtime)?;

    let render_packet = extract_game_frame_packet(session, include_game_ui)?;

    session.begin_render_composition();
    let composition_plan = WgpuFrameCompositionBuilder::build_for_target(
        &render_packet,
        amigo_render_api::RenderTargetPlan::Offscreen {
            width: target.width,
            height: target.height,
        },
    );
    session.complete_render_composition();

    let frame_graph = {
        let graph = build_frame_graph_from_plan(
            &composition_plan,
            AppFrameGraphBuildInfo {
                width: target.width,
                height: target.height,
            },
        );
        session.complete_render_graph_build();
        graph
    };

    if let Ok(render_diagnostics) = required::<RenderCompositionDiagnosticsService>(runtime) {
        render_diagnostics.set_with_camera_capture_focus_and_contributions(
            &composition_plan,
            &frame_graph,
            render_packet.camera_capture_input_2d().map(|input| {
                render_camera_capture_summary(input, render_packet.visual_source_flags_2d())
            }),
            render_packet.camera_capture_input_2d().map(|input| {
                render_camera_focus_plan_summary(
                    input,
                    camera_focus_for_input(runtime, assets.as_ref(), input),
                )
            }),
            render_camera_contributions_summary(
                runtime,
                assets.as_ref(),
                render_packet.camera_capture_input_2d(),
                amigo_runtime_bundles::wgpu_render_extractors::render_contribution_decisions_summary(
                    render_packet.world_2d_beacons(),
                ),
            ),
            None,
            Some(render_visual_items_summary(
                render_packet.renderables_2d(),
                render_packet.world_2d_render_layers(),
                render_packet
                    .camera_capture_input_2d()
                    .map(|input| input.layers.as_slice())
                    .unwrap_or(&[]),
            )),
        );
    }
    if let Ok(stats_service) = required::<RenderFrameStatsService>(runtime) {
        let previous = stats_service.snapshot();
        let stats = RenderFrameStats {
            frame_index: previous.frame_index + 1,
            window_width: target.width,
            window_height: target.height,
            world_2d_tilemaps: render_packet.world_2d_tilemaps().len(),
            world_2d_sprites: render_packet.world_2d_sprites().len(),
            world_2d_layered_images: render_packet.world_2d_layered_images().len(),
            world_2d_render_layers: render_packet.world_2d_render_layers().len(),
            world_2d_light_routes: render_packet.world_2d_light_routes().len(),
            world_2d_global_lights: render_packet.world_2d_global_lights().len(),
            world_2d_lightmaps: render_packet.world_2d_lightmaps().len(),
            world_2d_light_groups: render_packet.world_2d_light_groups().len(),
            world_2d_vectors: render_packet.world_2d_vectors().len(),
            world_2d_beacons: render_packet.world_2d_beacons().len(),
            world_2d_text: render_packet.world_2d_text().len(),
            world_2d_particles: render_packet.world_2d_particles().len(),
            world_3d_meshes: render_packet.world_3d_meshes().len(),
            world_3d_materials: render_packet.world_3d_materials().len(),
            world_3d_text: render_packet.world_3d_text().len(),
            game_ui_overlays: render_packet.game_ui_overlay().len(),
            debug_overlays: render_packet.debug_overlay().len(),
            ui_overlays: render_packet.all_overlay_count(),
            render_graph_nodes: frame_graph.nodes.len(),
            post_fx_effects: render_packet
                .post_fx_stacks()
                .iter()
                .map(|stack| stack.effects.len())
                .sum(),
        };
        stats_service.set(stats.clone());
        debug_overlay_service.record_render_frame(stats);
    }
    if let Ok(scheduling) = required::<amigo_session::RuntimeSchedulingService>(runtime) {
        debug_overlay_service.record_scheduling_stats(scheduling.stats());
    }
    if let Ok(clock) = required::<amigo_session::RuntimeFrameClockService>(runtime) {
        debug_overlay_service.record_frame_clock_snapshot(clock.snapshot());
    }
    debug_overlay_service.record_particle_snapshot(
        particles.emitters().len(),
        particles
            .emitters()
            .iter()
            .filter(|emitter| particles.is_active(&emitter.entity_name))
            .count(),
    );

    let extracted_tilemaps = build_tilemap_scene_service_from_packet(&render_packet);
    let extracted_sprites = build_sprite_scene_service_from_packet(&render_packet);
    let extracted_layered_images = build_layered_image_scene_service_from_packet(&render_packet);
    let extracted_depth_maps = build_depth_map2d_scene_service_from_packet(&render_packet);
    let extracted_render_layers = build_render_layer2d_scene_service_from_packet(&render_packet);
    let extracted_light_routes = build_light_route2d_scene_service_from_packet(&render_packet);
    let extracted_global_lights = build_global_light2d_scene_service_from_packet(&render_packet);
    let extracted_lightmaps = build_lightmap2d_scene_service_from_packet(&render_packet);
    let extracted_text2d = build_text2d_scene_service_from_packet(&render_packet);
    let extracted_vectors = build_vector_scene_service_from_packet(&render_packet);

    if let Ok(post_fx_service) =
        required::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>(runtime)
    {
        let has_post_fx = !render_packet.post_fx_stacks().is_empty();
        let renderer_mode = if has_post_fx {
            "frame_graph_postfx"
        } else {
            "frame_graph"
        };
        post_fx_service.set_renderer_mode(renderer_mode);
    }

    let extracted_render_layer_commands = extracted_render_layers.commands();
    let extracted_light_route_commands = extracted_light_routes.commands();
    let render_request = amigo_render_wgpu::WgpuFrameRenderRequest {
        target: amigo_render_wgpu::WgpuFrameRenderTarget::Offscreen(target),
        scene: scene.as_ref(),
        assets: assets.as_ref(),
        world_2d: amigo_render_wgpu::WgpuWorld2dRenderInput {
            renderables: render_packet.renderables_2d(),
            tilemaps: &extracted_tilemaps,
            sprites: &extracted_sprites,
            layered_images: &extracted_layered_images,
            depth_maps: &extracted_depth_maps,
            global_lights: &extracted_global_lights,
            lightmaps: &extracted_lightmaps,
            text2d: &extracted_text2d,
            vectors: &extracted_vectors,
            beacons: render_packet.world_2d_beacons(),
            render_layers: extracted_render_layer_commands.as_slice(),
            light_routes: extracted_light_route_commands.as_slice(),
            light_groups: render_packet.world_2d_light_groups(),
            particles: render_packet.world_2d_particles(),
        },
        world_3d: amigo_render_wgpu::WgpuWorld3dRenderInput {
            meshes: render_packet.world_3d_meshes(),
            materials: render_packet.world_3d_materials(),
            text3d: Some(render_packet.world_3d_text()),
        },
        game_ui: render_packet.game_ui_overlay(),
        debug_ui: &[],
        post_fx_stacks: render_packet.post_fx_stacks(),
        active_camera_2d_entity: render_packet.active_camera_2d_entity(),
        camera_capture_input_2d: render_packet.camera_capture_input_2d(),
        visual_source_flags_2d: Some(render_packet.visual_source_flags_2d()),
        camera_debug_view: render_packet
            .camera_debug_view_2d()
            .unwrap_or(amigo_render_api::CameraDebugView2d::FinalOutput),
        emergency_overlay: &[],
        composition_plan: &composition_plan,
        frame_graph: &frame_graph,
        game_viewport: None,
    };
    renderer.render_frame_request(render_request)?;
    if let Ok(render_diagnostics) = required::<RenderCompositionDiagnosticsService>(runtime) {
        render_diagnostics
            .set_plate_relight_summary(renderer.plate_relight_last_summary().to_owned());
        render_diagnostics
            .set_render_materials_summary(renderer.render_materials_last_summary().to_owned());
    }
    session.complete_render_submit();
    Ok(())
}

pub(crate) fn editor_game_viewport_placement(
    runtime: &Runtime,
    surface_width: u32,
    surface_height: u32,
) -> Option<amigo_render_wgpu::WgpuGameViewportPlacement> {
    let state = runtime.resolve::<amigo_editor_ingame::IngameEditorState>()?;
    if !state.is_open() {
        return None;
    }

    let viewport =
        amigo_render_wgpu::UiViewportSize::new(surface_width as f32, surface_height as f32);
    let layout = amigo_editor_ingame::layout::EditorLayout::new(viewport);
    let game = layout.game_viewport_layout();
    let snapshot = state.snapshot();

    Some(amigo_render_wgpu::WgpuGameViewportPlacement {
        surface_rect: amigo_render_wgpu::WgpuSurfaceRect::new(
            game.rect.x,
            game.rect.y,
            game.rect.width,
            game.rect.height,
        ),
        logical_width: game.logical_width.round().max(1.0) as u32,
        logical_height: game.logical_height.round().max(1.0) as u32,
        pan_x: snapshot.viewport_pan_x,
        pan_y: snapshot.viewport_pan_y,
        zoom: snapshot.viewport_zoom,
    })
}

pub(crate) fn emergency_overlay_lines(
    runtime: &Runtime,
) -> Vec<amigo_render_wgpu::WgpuEmergencyOverlayLine> {
    runtime
        .resolve::<amigo_devtools::EmergencyNoticeService>()
        .map(|service| {
            service
                .snapshot()
                .into_iter()
                .map(|notice| amigo_render_wgpu::WgpuEmergencyOverlayLine {
                    level: match notice.level {
                        amigo_devtools::EmergencyNoticeLevel::Warning => {
                            amigo_render_wgpu::WgpuEmergencyOverlayLevel::Warning
                        }
                        amigo_devtools::EmergencyNoticeLevel::Error => {
                            amigo_render_wgpu::WgpuEmergencyOverlayLevel::Error
                        }
                    },
                    message: notice.message,
                })
                .collect()
        })
        .unwrap_or_default()
}

fn render_camera_capture_summary(
    input: &amigo_render_api::CameraCaptureInput2d,
    flags: &amigo_render_wgpu::WgpuVisualSourceFlags2d,
) -> String {
    format!(
        "{}\nGeneratedFlags:\n  layer_mask={}\n  layer_roles={}\n  scene_normal={}\n  scene_wetness={}\n  scene_highlight={}\n  scene_emissive={}\n  scene_motion={}",
        input.debug_summary(),
        flags.layer_mask_generated,
        flags.layer_roles_generated,
        flags.scene_normal_generated,
        flags.scene_wetness_generated,
        flags.scene_highlight_generated,
        flags.scene_emissive_generated,
        flags.scene_motion_generated,
    )
}

fn render_visual_items_summary(
    renderables: &[amigo_render_wgpu::Renderable2dItem],
    render_layers: &[amigo_runtime_bundles::amigo_2d_composition::RenderLayer2dCommand],
    layers: &[amigo_render_api::ResolvedLayerOptics2d],
) -> String {
    let mut lines = Vec::new();
    lines.push("render.visual.items:".to_owned());

    if renderables.is_empty() {
        lines.push("none".to_owned());
        return lines.join("\n");
    }

    let layer_lookup = layers
        .iter()
        .map(|layer| (layer.layer_id.as_str(), layer))
        .collect::<std::collections::BTreeMap<_, _>>();
    let order_lookup = render_layers
        .iter()
        .map(|layer| (layer.id.as_str(), layer.order))
        .collect::<std::collections::BTreeMap<_, _>>();

    for item in renderables.iter().take(128) {
        lines.push(String::new());
        lines.push(format!(
            "entity={} component={}",
            item.owner_entity(),
            item.component_kind()
        ));

        let space = match item.render_space() {
            amigo_render_wgpu::RenderSpace2d::World => "World",
            amigo_render_wgpu::RenderSpace2d::ScreenOverlay => "ScreenOverlay",
            amigo_render_wgpu::RenderSpace2d::DebugOverlay => "DebugOverlay",
        };

        if let Some(layer) = layer_lookup.get(item.render_layer()) {
            lines.push(format!(
                "space={} layer={} order={} z_index={:.2}",
                space,
                item.render_layer(),
                order_lookup
                    .get(item.render_layer())
                    .map(|order| format!("{order:.2}"))
                    .unwrap_or_else(|| "?".to_owned()),
                item.z_index()
            ));
            lines.push(format!(
                "payload={} camera_pipeline={} base_z_depth={:.3} effective_z_depth={:.3} effective_distance_m={} z_depth={:.3} blur_scale={:.2} camera_motion_scale={:.2}",
                item.payload_kind(),
                item.uses_camera_pipeline(),
                layer.base_z_depth,
                layer.effective_z_depth,
                layer
                    .effective_distance_m
                    .map(|meters| format!("{meters:.2}"))
                    .unwrap_or_else(|| "?".to_owned()),
                layer.z_depth,
                layer.blur_scale,
                layer.camera_motion_scale
            ));
        } else {
            lines.push(format!(
                "space={} layer={} order={} z_index={:.2}",
                space,
                item.render_layer(),
                order_lookup
                    .get(item.render_layer())
                    .map(|order| format!("{order:.2}"))
                    .unwrap_or_else(|| "?".to_owned()),
                item.z_index()
            ));
            lines.push(format!(
                "payload={} camera_pipeline={} z_depth=? blur_scale=? camera_motion_scale=?",
                item.payload_kind(),
                item.uses_camera_pipeline()
            ));
        }
    }

    lines.join("\n")
}

fn camera_focus_for_input(
    runtime: &amigo_runtime::Runtime,
    assets: &AssetCatalog,
    input: &amigo_render_api::CameraCaptureInput2d,
) -> Option<CameraFocusPlanInfo> {
    let camera_service =
        required::<amigo_runtime_bundles::amigo_camera::CameraService>(runtime).ok()?;
    let rig = camera_service.main_resolved_camera_rig_2d(Some(assets), input.depth_space)?;
    let motion = camera_service.main_camera_depth_motion_2d().unwrap_or_default();
    Some(CameraFocusPlanInfo {
        base_focus_distance_m: rig.aperture.base_focus_distance_m,
        effective_focus_distance_m: rig.aperture.effective_focus_distance_m,
        camera_z_m: rig.aperture.camera_z_m,
        focus_residual_m: rig.aperture.focus_residual_m,
        dolly_signal: motion.dolly_signal,
        computed_focus_z_depth: rig.aperture.computed_focus_z_depth,
        focus_width: rig.aperture.depth_of_field.focus_width,
        f_stop: rig.aperture.state.f_stop,
        max_blur_px: rig.aperture.depth_of_field.max_blur_px,
    })
}

fn render_camera_contributions_summary(
    runtime: &amigo_runtime::Runtime,
    assets: &AssetCatalog,
    input: Option<&amigo_render_api::CameraCaptureInput2d>,
    beacon_contributions_summary: Option<String>,
) -> Option<String> {
    let camera_service =
        required::<amigo_runtime_bundles::amigo_camera::CameraService>(runtime).ok()?;
    let depth_space = input.map(|input| input.depth_space).unwrap_or_default();
    let mut summary = camera_service.camera_render_contributions_summary_for_depth_space(
        Some(assets),
        depth_space,
    );
    if let Some(beacon_contributions_summary) = beacon_contributions_summary {
        summary.push('\n');
        summary.push_str(&beacon_contributions_summary);
    }
    Some(summary)
}

fn render_camera_focus_plan_summary(
    input: &amigo_render_api::CameraCaptureInput2d,
    focus: Option<CameraFocusPlanInfo>,
) -> String {
    let mut lines = Vec::new();
    lines.push("FocusBlur plan:".to_owned());
    lines.push(format!(
        "depth_space: near={} far={} curve={:?}",
        input.depth_space.near_m, input.depth_space.far_m, input.depth_space.curve
    ));

    match focus {
        Some(info) => {
            lines.push(format!(
                "base_focus_distance_m: {}",
                info.base_focus_distance_m
                    .map(|value| format!("{value:.2}"))
                    .unwrap_or_else(|| "unavailable".to_owned())
            ));
            lines.push(format!(
                "effective_focus_distance_m: {}",
                info.effective_focus_distance_m
                    .map(|value| format!("{value:.2}"))
                    .unwrap_or_else(|| "unavailable".to_owned())
            ));
            lines.push(format!("camera_z_m: {:.3}", info.camera_z_m));
            lines.push(format!("focus_residual_m: {:.3}", info.focus_residual_m));
            lines.push(format!("dolly_signal: {:.3}", info.dolly_signal));
            lines.push(format!(
                "computed_focus_z_depth: {}",
                info.computed_focus_z_depth
                    .map(|value| format!("{value:.3}"))
                    .unwrap_or_else(|| "unavailable".to_owned())
            ));
            lines.push(format!("focus_width: {:.3}", info.focus_width));
            lines.push(format!("f_stop: {:.2}", info.f_stop));
            lines.push(format!("max_blur_px: {:.2}", info.max_blur_px));
        }
        None => {
            lines.push("base_focus_distance_m: unavailable".to_owned());
            lines.push("effective_focus_distance_m: unavailable".to_owned());
            lines.push("camera_z_m: unavailable".to_owned());
            lines.push("focus_residual_m: unavailable".to_owned());
            lines.push("dolly_signal: unavailable".to_owned());
            lines.push("computed_focus_z_depth: unavailable".to_owned());
        }
    }

    lines.push(String::new());
    lines.push("layers:".to_owned());
    if input.layers.is_empty() {
        lines.push("none".to_owned());
    } else {
        for layer in &input.layers {
            let distance = layer
                .distance_m
                .map(|meters| format!("{meters:.2}"))
                .unwrap_or_else(|| "-".to_owned());
            let effective_distance = layer
                .effective_distance_m
                .map(|meters| format!("{meters:.2}"))
                .unwrap_or_else(|| "-".to_owned());
            let focus_delta = focus
                .and_then(|info| info.computed_focus_z_depth)
                .map(|focus_z| format!("{:.3}", (layer.z_depth - focus_z).abs() * layer.blur_scale))
                .unwrap_or_else(|| "-".to_owned());
            lines.push(format!(
                "{} mode={} role={:?} distance_m={} effective_distance_m={} base_z_depth={:.3} effective_z_depth={:.3} z_depth={:.3} blur_scale={:.2} camera_motion_scale={:.2} focus_delta={}",
                layer.layer_id,
                layer.depth_mode,
                layer.role,
                distance,
                effective_distance,
                layer.base_z_depth,
                layer.effective_z_depth,
                layer.z_depth,
                layer.blur_scale,
                layer.camera_motion_scale,
                focus_delta
            ));
        }
    }

    lines.join("\n")
}
