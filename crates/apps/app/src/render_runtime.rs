mod composition;
mod context;
mod diagnostics;
mod extractors;
mod graph;
mod services;
mod stats;

#[cfg(test)]
mod tests;

use super::*;
use amigo_session::RuntimeSession;

pub(crate) use composition::AppFrameCompositionBuilder;
pub(crate) use context::AppRenderExtractContext;
#[cfg(test)]
pub(crate) use context::AppRenderFramePacket;
pub(crate) use diagnostics::RenderCompositionDiagnosticsService;
pub(crate) use extractors::{
    default_app_render_extractor_registry, register_host_render_extractor_provider,
};
pub(crate) use graph::{AppFrameGraphBuildInfo, build_frame_graph_from_plan};
pub(crate) use services::{
    build_global_light2d_scene_service_from_packet, build_layered_image_scene_service_from_packet,
    build_light_route2d_scene_service_from_packet, build_lightmap2d_scene_service_from_packet,
    build_render_layer2d_scene_service_from_packet, build_sprite_scene_service_from_packet,
    build_text2d_scene_service_from_packet, build_tilemap_scene_service_from_packet,
    build_vector_scene_service_from_packet,
};
pub(crate) use stats::{RenderFrameStats, RenderFrameStatsService};

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
    let tilemaps = required::<TileMap2dSceneService>(runtime)?;
    let sprites = required::<SpriteSceneService>(runtime)?;
    let layered_images = required::<amigo_2d_layered_image::LayeredImageSceneService>(runtime)?;
    let render_layers = required::<amigo_2d_composition::RenderLayer2dSceneService>(runtime)?;
    let light_routes = required::<amigo_2d_composition::LightRoute2dSceneService>(runtime)?;
    let global_lights = required::<amigo_2d_lighting::GlobalLight2dSceneService>(runtime)?;
    let lightmaps = required::<amigo_2d_lighting::LightMap2dSceneService>(runtime)?;
    let light_groups = required::<amigo_2d_lighting::LightGroup2dSceneService>(runtime)?;
    let text2d = required::<Text2dSceneService>(runtime)?;
    let vectors = required::<VectorSceneService>(runtime)?;
    let particles = required::<Particle2dSceneService>(runtime)?;
    let meshes = required::<MeshSceneService>(runtime)?;
    let text3d = required::<Text3dSceneService>(runtime)?;
    let materials = required::<MaterialSceneService>(runtime)?;
    let ui_scene = required::<UiSceneService>(runtime)?;
    let ui_state = required::<UiStateService>(runtime)?;
    let ui_theme = required::<UiThemeService>(runtime)?;
    let post_fx_service = required::<amigo_2d_post_fx::PostFx2dService>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let dev_console_completion = required::<crate::dev_console::completion::ConsoleCompletionState>(runtime)?;
    let debug_overlay_service = required::<crate::debug_overlay::DebugOverlayService>(runtime)?;
    let ui_viewport_state = required::<systems::UiInputViewportState>(runtime)?;

    session.begin_render_frame_extract();
    let render_packet = default_app_render_extractor_registry().extract_all(&AppRenderExtractContext {
        scene_service: scene.as_ref(),
        tilemap_scene_service: tilemaps.as_ref(),
        sprite_scene_service: sprites.as_ref(),
        layered_image_scene_service: layered_images.as_ref(),
        render_layer2d_scene_service: render_layers.as_ref(),
        light_route2d_scene_service: light_routes.as_ref(),
        global_light2d_scene_service: global_lights.as_ref(),
        lightmap2d_scene_service: lightmaps.as_ref(),
        light_group2d_scene_service: light_groups.as_ref(),
        text2d_scene_service: text2d.as_ref(),
        vector_scene_service: vectors.as_ref(),
        particle2d_scene_service: particles.as_ref(),
        mesh_scene_service: meshes.as_ref(),
        material_scene_service: materials.as_ref(),
        text3d_scene_service: text3d.as_ref(),
        ui_scene_service: ui_scene.as_ref(),
        ui_state_service: ui_state.as_ref(),
        ui_theme_service: ui_theme.as_ref(),
        post_fx_service: post_fx_service.as_ref(),
        dev_console_state: dev_console_state.as_ref(),
        dev_console_completion: dev_console_completion.as_ref(),
        debug_overlay_service: debug_overlay_service.as_ref(),
        ui_viewport_state: ui_viewport_state.as_ref(),
    });
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
    if let Ok(scheduling) = required::<crate::scheduling::AppSchedulingService>(runtime) {
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

    if let Ok(post_fx_service) = required::<amigo_2d_post_fx::PostFx2dService>(runtime) {
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
