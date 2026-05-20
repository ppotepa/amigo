#[cfg(test)]
mod tests;

use super::*;
use amigo_runtime_bundles::amigo_audio_api::AudioStateService;
use amigo_runtime_bundles::amigo_audio_output::AudioOutputBackendService;
use amigo_runtime_bundles::amigo_input_actions::InputActionService;
use amigo_runtime_bundles::amigo_particles_2d_plugin::Particle2dSceneService;
use amigo_session::RuntimeSession;

pub(crate) use amigo_render_api::RenderCompositionDiagnosticsService;
pub(crate) use amigo_render_api::{FrameGraphBuildInfo, build_frame_graph_from_plan};
pub(crate) use amigo_render_api::RenderFrameStats;
pub(crate) use amigo_render_api::RenderFrameStatsService;
#[cfg(test)]
pub(crate) use amigo_render_wgpu::WgpuRenderFramePacket;
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionBuilder;
pub(crate) use amigo_runtime_bundles::{
    WgpuEditorOverlayOutput, extract_game_frame_packet, extract_live_host_overlay_packet,
    render_game_frame_to_cache, update_ui_input_viewport_state,
    update_wgpu_render_composition_diagnostics,
};
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionOptions;
pub(crate) use amigo_runtime_bundles::{
    build_depth_map2d_scene_service_from_packet, build_global_light2d_scene_service_from_packet,
    build_layered_image_scene_service_from_packet, build_light_route2d_scene_service_from_packet,
    build_lightmap2d_scene_service_from_packet, build_render_layer2d_scene_service_from_packet,
    build_sprite_scene_service_from_packet, build_text2d_scene_service_from_packet,
    build_tilemap_scene_service_from_packet, build_vector_scene_service_from_packet,
};

#[cfg(test)]
pub(crate) use amigo_runtime_bundles::{
    build_material_scene_service_from_packet, build_mesh_scene_service_from_packet,
    build_text3d_scene_service_from_packet,
};

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
    update_ui_input_viewport_state(
        runtime,
        surface_size.width as f32,
        surface_size.height as f32,
    );

    session.begin_render_frame_extract();
    let mut render_packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(runtime);
    amigo_editor_ingame::append_editor_overlay(runtime, &mut WgpuEditorOverlayOutput(&mut render_packet));
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
            FrameGraphBuildInfo {
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

    update_wgpu_render_composition_diagnostics(
        runtime,
        assets.as_ref(),
        &render_packet,
        &composition_plan,
        &frame_graph,
    );
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
        required::<amigo_runtime_bundles::amigo_composite_plugin::PostFx2dService>(runtime)
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
            light_sources: render_packet.world_2d_light_sources(),
            camera_optical_candidates: render_packet.camera_optical_candidates_2d(),
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
            .unwrap_or_else(amigo_render_api::CameraDebugView2d::final_output),
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

