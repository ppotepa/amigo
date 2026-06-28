#[cfg(test)]
mod tests;

use super::*;
use amigo_session::RuntimeSession;

pub(crate) use amigo_render_api::RenderCompositionDiagnosticsService;
pub(crate) use amigo_render_api::RenderFrameStats;
pub(crate) use amigo_render_api::RenderFrameStatsService;
pub(crate) use amigo_render_api::Renderable2dKind;
pub(crate) use amigo_render_api::{FrameGraphBuildInfo, build_frame_graph_from_plan};
#[cfg(test)]
pub(crate) use amigo_render_wgpu::WgpuRenderFramePacket;
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionBuilder;
pub(crate) use amigo_runtime_bundles::WgpuFrameCompositionOptions;
pub(crate) use amigo_runtime_bundles::{
    WgpuEditorOverlayOutput, audio_debug_snapshot, extract_live_host_overlay_packet,
    input_debug_snapshot, particle_debug_snapshot, render_game_frame_to_cache,
    update_ui_input_viewport_state, update_wgpu_postfx_renderer_mode,
    update_wgpu_render_composition_diagnostics,
};
pub(crate) use amigo_runtime_bundles::{WgpuFrameSubmitInput, submit_wgpu_frame_render_request};

#[cfg(test)]
pub(crate) use amigo_runtime_bundles::{
    build_material_scene_service_from_packet, build_mesh_scene_service_from_packet,
    build_text3d_scene_service_from_packet,
};

#[allow(dead_code)]
pub(crate) fn build_render_frame_for_session(
    session: &RuntimeSession,
    surface: &mut WgpuSurfaceState,
    renderer: &mut WgpuSceneRenderer,
) -> AmigoResult<()> {
    let runtime = session.runtime();
    let scene = required::<SceneService>(runtime)?;
    let assets = required::<AssetCatalog>(runtime)?;
    let debug_overlay_service = required::<crate::debug_overlay::DebugOverlayService>(runtime)?;
    let dev_console_open = runtime
        .resolve::<amigo_scripting_api::DevConsoleState>()
        .is_some_and(|console| console.is_open());
    let debug_overlay_enabled = debug_overlay_service.is_enabled();
    let wants_render_diagnostics =
        dev_console_open || debug_overlay_service.wants_render_diagnostics();

    let surface_size = surface.size();
    update_ui_input_viewport_state(
        runtime,
        surface_size.width as f32,
        surface_size.height as f32,
    );

    session.begin_render_frame_extract();
    let mut render_packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry_for_runtime(runtime)
            .extract_all(runtime);
    amigo_editor_ingame::append_editor_overlay(
        runtime,
        &mut WgpuEditorOverlayOutput(&mut render_packet),
    );
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

    if wants_render_diagnostics {
        update_wgpu_render_composition_diagnostics(
            runtime,
            assets.as_ref(),
            &render_packet,
            &composition_plan,
            &frame_graph,
        );
    }
    if let Ok(stats_service) = required::<RenderFrameStatsService>(runtime) {
        let previous = stats_service.snapshot();
        let stats = RenderFrameStats {
            frame_index: previous.frame_index + 1,
            window_width: surface_size.width,
            window_height: surface_size.height,
            world_2d_tilemaps: render_packet.renderable_2d_count_by_kind(Renderable2dKind::TileMap),
            world_2d_sprites: render_packet.renderable_2d_count_by_kind(Renderable2dKind::Sprite),
            world_2d_layered_images: render_packet
                .renderable_2d_count_by_kind(Renderable2dKind::LayeredImage),
            world_2d_render_layers: render_packet.world_2d_render_layers().len(),
            world_2d_light_routes: render_packet.world_2d_light_routes().len(),
            world_2d_global_lights: render_packet.light_source_2d_contribution_count(),
            world_2d_lightmaps: render_packet.lightmap_2d_contribution_count(),
            world_2d_light_groups: render_packet.light_group_2d_contribution_count(),
            world_2d_vectors: render_packet.renderable_2d_count_by_kind(Renderable2dKind::Vector),
            world_2d_beacons: render_packet.renderable_2d_count_by_kind(Renderable2dKind::Beacon),
            world_2d_text: render_packet.renderable_2d_count_by_kind(Renderable2dKind::Text),
            world_2d_particles: render_packet
                .renderable_2d_count_by_kind(Renderable2dKind::Particle),
            world_3d_meshes: render_packet.world_3d_meshes().len(),
            world_3d_npr_meshes: render_packet
                .world_3d_meshes()
                .iter()
                .filter(|command| command.mesh.npr.is_some())
                .count(),
            world_3d_npr_gpu_realtime_meshes: 0,
            world_3d_npr_cpu_reference_meshes: 0,
            world_3d_npr_gpu_realtime_enqueued_edges: 0,
            world_3d_npr_gpu_realtime_enqueued_triangles: 0,
            world_3d_npr_gpu_realtime_topology_uploads: 0,
            world_3d_npr_gpu_realtime_buffer_capacity_bytes: 0,
            world_3d_npr_paths: 0,
            world_3d_npr_boundary_paths: 0,
            world_3d_npr_silhouette_paths: 0,
            world_3d_npr_crease_paths: 0,
            world_3d_npr_seam_paths: 0,
            world_3d_npr_feature_paths: 0,
            world_3d_npr_contact_paths: 0,
            world_3d_npr_brush_samples: 0,
            world_3d_npr_strip_vertices: 0,
            world_3d_npr_primary_passes: 0,
            world_3d_npr_search_passes: 0,
            world_3d_npr_dropout_intervals: 0,
            world_3d_npr_cached_plan_hits: 0,
            world_3d_npr_cached_plan_misses: 0,
            world_3d_npr_path_build_us: 0.0,
            world_3d_npr_stabilize_us: 0.0,
            world_3d_npr_stroke_vertices_us: 0.0,
            world_3d_npr_path_project_us: 0.0,
            world_3d_npr_path_visibility_us: 0.0,
            world_3d_npr_path_edge_sample_us: 0.0,
            world_3d_npr_path_stitch_us: 0.0,
            world_3d_npr_path_visible_edges: 0,
            world_3d_npr_path_fragments: 0,
            offscreen_color_buffer_writes: 0,
            offscreen_color_buffer_reallocs: 0,
            offscreen_color_upload_bytes: 0,
            offscreen_color_buffer_capacity_bytes: 0,
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
        if debug_overlay_enabled || dev_console_open {
            debug_overlay_service.record_render_frame(stats);
        }
    }
    if debug_overlay_enabled || dev_console_open {
        if let Ok(scheduling) = required::<amigo_session::RuntimeSchedulingService>(runtime) {
            debug_overlay_service.record_scheduling_stats(scheduling.stats());
        }
        if let Some(audio) = audio_debug_snapshot(runtime) {
            debug_overlay_service.record_audio_snapshot(
                audio.backend,
                audio.master_volume,
                audio.active_sources,
                audio.pending_commands,
                audio.bus_count,
            );
        }
        if let Some(input) = input_debug_snapshot(runtime) {
            debug_overlay_service.record_input_snapshot(
                input.backend_name,
                input.pressed_keys,
                input.active_map,
                input.active_actions,
            );
        }
        if let Some((emitters, active_emitters)) = particle_debug_snapshot(runtime) {
            debug_overlay_service.record_particle_snapshot(emitters, active_emitters);
        }
    }

    update_wgpu_postfx_renderer_mode(runtime, &render_packet);

    let emergency_overlay = emergency_overlay_lines(runtime);
    submit_wgpu_frame_render_request(
        renderer,
        WgpuFrameSubmitInput {
            target: amigo_render_wgpu::WgpuFrameRenderTarget::Surface(surface),
            scene: scene.as_ref(),
            assets: assets.as_ref(),
            render_packet: &render_packet,
            render_layers: render_packet.world_2d_render_layers(),
            light_routes: render_packet.world_2d_light_routes(),
            debug_ui: render_packet.debug_overlay(),
            emergency_overlay: emergency_overlay.as_slice(),
            composition_plan: &composition_plan,
            frame_graph: &frame_graph,
            game_viewport: editor_game_viewport,
        },
    )?;
    if let Ok(stats_service) = required::<RenderFrameStatsService>(runtime) {
        let npr = renderer.npr_stroke_stats_3d();
        let mut stats = stats_service.snapshot();
        stats.world_3d_npr_gpu_realtime_meshes = npr.gpu_realtime_meshes;
        stats.world_3d_npr_cpu_reference_meshes = npr.cpu_reference_meshes;
        stats.world_3d_npr_gpu_realtime_enqueued_edges = npr.gpu_realtime_enqueued_edges;
        stats.world_3d_npr_gpu_realtime_enqueued_triangles =
            npr.gpu_realtime_enqueued_triangles;
        stats.world_3d_npr_gpu_realtime_topology_uploads = npr.gpu_realtime_topology_uploads;
        stats.world_3d_npr_gpu_realtime_buffer_capacity_bytes =
            npr.gpu_realtime_buffer_capacity_bytes;
        stats.world_3d_npr_paths = npr.paths;
        stats.world_3d_npr_boundary_paths = npr.boundary_paths;
        stats.world_3d_npr_silhouette_paths = npr.silhouette_paths;
        stats.world_3d_npr_crease_paths = npr.crease_paths;
        stats.world_3d_npr_seam_paths = npr.seam_paths;
        stats.world_3d_npr_feature_paths = npr.feature_paths;
        stats.world_3d_npr_contact_paths = npr.contact_paths;
        stats.world_3d_npr_brush_samples = npr.brush_samples;
        stats.world_3d_npr_strip_vertices = npr.strip_vertices;
        stats.world_3d_npr_primary_passes = npr.primary_passes;
        stats.world_3d_npr_search_passes = npr.search_passes;
        stats.world_3d_npr_dropout_intervals = npr.dropout_intervals;
        stats.world_3d_npr_cached_plan_hits = npr.cached_plan_hits;
        stats.world_3d_npr_cached_plan_misses = npr.cached_plan_misses;
        stats.world_3d_npr_path_build_us = npr.path_build_us;
        stats.world_3d_npr_stabilize_us = npr.stabilize_us;
        stats.world_3d_npr_stroke_vertices_us = npr.stroke_vertices_us;
        stats.world_3d_npr_path_project_us = npr.path_project_us;
        stats.world_3d_npr_path_visibility_us = npr.path_visibility_us;
        stats.world_3d_npr_path_edge_sample_us = npr.path_edge_sample_us;
        stats.world_3d_npr_path_stitch_us = npr.path_stitch_us;
        stats.world_3d_npr_path_visible_edges = npr.path_visible_edges;
        stats.world_3d_npr_path_fragments = npr.path_fragments;
        let upload = renderer.offscreen_upload_stats();
        stats.offscreen_color_buffer_writes = upload.color_buffer_writes;
        stats.offscreen_color_buffer_reallocs = upload.color_buffer_reallocs;
        stats.offscreen_color_upload_bytes = upload.color_upload_bytes;
        stats.offscreen_color_buffer_capacity_bytes = upload.color_buffer_capacity_bytes;
        stats_service.set(stats.clone());
        if debug_overlay_enabled || dev_console_open {
            debug_overlay_service.record_render_frame(stats);
        }
    }
    if wants_render_diagnostics {
        let render_diagnostics = required::<RenderCompositionDiagnosticsService>(runtime)?;
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
