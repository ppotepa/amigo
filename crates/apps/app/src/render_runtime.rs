mod composition;
mod graph;
mod services;

#[cfg(test)]
mod tests;

use super::*;
use amigo_session::RuntimeSession;

pub(crate) use composition::AppFrameCompositionBuilder;
#[cfg(test)]
pub(crate) use amigo_render_wgpu::WgpuRenderFramePacket;
pub(crate) use amigo_render_api::RenderCompositionDiagnosticsService;
pub(crate) use amigo_runtime_bundles::{
    default_wgpu_render_extractor_registry, register_host_render_extractor_provider,
};
pub(crate) use graph::{AppFrameGraphBuildInfo, build_frame_graph_from_plan};
pub(crate) use services::{
    build_global_light2d_scene_service_from_packet, build_layered_image_scene_service_from_packet,
    build_light_route2d_scene_service_from_packet, build_lightmap2d_scene_service_from_packet,
    build_render_layer2d_scene_service_from_packet, build_sprite_scene_service_from_packet,
    build_text2d_scene_service_from_packet, build_tilemap_scene_service_from_packet,
    build_vector_scene_service_from_packet,
};
pub(crate) use amigo_render_api::RenderFrameStats;
pub(crate) use amigo_render_api::RenderFrameStatsService;

#[cfg(test)]
pub(crate) use services::{
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

    session.begin_render_frame_extract();
    let render_packet =
        amigo_runtime_bundles::default_wgpu_render_extractor_registry().extract_all(runtime);
    session.complete_render_frame_extract();

    let surface_size = surface.size();
    session.begin_render_composition();
    let composition_plan = AppFrameCompositionBuilder::build(&render_packet);
    session.complete_render_composition();
    let frame_graph = {
        let graph = build_frame_graph_from_plan(
            &composition_plan,
            AppFrameGraphBuildInfo {
                width: surface_size.width,
                height: surface_size.height,
            },
        );
        session.complete_render_graph_build();
        graph
    };

    if let Ok(render_diagnostics) = required::<RenderCompositionDiagnosticsService>(runtime) {
        render_diagnostics.set(&composition_plan, &frame_graph);
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
            world_2d_text: render_packet.world_2d_text().len(),
            world_2d_particles: render_packet.world_2d_particles().len(),
            world_3d_meshes: render_packet.world_3d_meshes().len(),
            world_3d_materials: render_packet.world_3d_materials().len(),
            world_3d_text: render_packet.world_3d_text().len(),
            game_ui_overlays: render_packet.game_ui_overlay().len(),
            debug_overlays: render_packet.debug_overlay().len(),
            ui_overlays: render_packet.all_overlay_count(),
            render_graph_nodes: frame_graph.nodes.len(),
            post_fx_effects: render_packet.post_fx_stack().map(|stack| stack.effects.len()).unwrap_or(0),
        };
        stats_service.set(stats.clone());
        debug_overlay_service.record_render_frame(stats);
    }
    if let Ok(scheduling) = required::<amigo_session::AppSchedulingService>(runtime) {
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
        let (active_map, active_actions) = if let Ok(actions) = required::<InputActionService>(runtime) {
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
                            actions.down(input_state.as_ref(), name).then(|| name.to_owned())
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
    let extracted_layered_images =
        build_layered_image_scene_service_from_packet(&render_packet);
    let extracted_render_layers = build_render_layer2d_scene_service_from_packet(&render_packet);
    let extracted_light_routes = build_light_route2d_scene_service_from_packet(&render_packet);
    let extracted_global_lights = build_global_light2d_scene_service_from_packet(&render_packet);
    let extracted_lightmaps = build_lightmap2d_scene_service_from_packet(&render_packet);
    let extracted_text2d = build_text2d_scene_service_from_packet(&render_packet);
    let extracted_vectors = build_vector_scene_service_from_packet(&render_packet);

    if let Ok(post_fx_service) = required::<amigo_runtime_bundles::amigo_2d_post_fx::PostFx2dService>(runtime) {
        let has_post_fx = render_packet.post_fx_stack().is_some_and(|stack| !stack.is_empty());
        let renderer_mode = if has_post_fx { "frame_graph_postfx" } else { "frame_graph" };
        post_fx_service.set_renderer_mode(renderer_mode);
    }

    let extracted_render_layer_commands = extracted_render_layers.commands();
    let extracted_light_route_commands = extracted_light_routes.commands();
    let render_request = amigo_render_wgpu::WgpuFrameRenderRequest {
        target: amigo_render_wgpu::WgpuFrameRenderTarget::Surface(surface),
        scene: scene.as_ref(),
        assets: assets.as_ref(),
        world_2d: amigo_render_wgpu::WgpuWorld2dRenderInput {
            tilemaps: &extracted_tilemaps,
            sprites: &extracted_sprites,
            layered_images: &extracted_layered_images,
            global_lights: &extracted_global_lights,
            lightmaps: &extracted_lightmaps,
            text2d: &extracted_text2d,
            vectors: &extracted_vectors,
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
        post_fx_stack: render_packet.post_fx_stack(),
        composition_plan: &composition_plan,
        frame_graph: &frame_graph,
    };
    renderer.render_frame_request(render_request)?;
    session.complete_render_submit();
    Ok(())
}



