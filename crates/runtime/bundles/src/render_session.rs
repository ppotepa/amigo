use std::any::type_name;
use std::sync::Arc;

use amigo_assets::AssetCatalog;
use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;
use amigo_session::RuntimeSession;
use amigo_render_api::{
    FrameGraphBuildInfo, RenderCompositionDiagnosticsService, RenderContribution2d, RenderFrameStats,
    RenderFrameStatsService, build_frame_graph_from_plan,
};
use amigo_render_wgpu::WgpuSceneRenderer;
use amigo_scene::SceneService;

use crate::{
    LightRoute2dSceneService, RenderLayer2dSceneService, WgpuFrameCompositionBuilder,
    update_wgpu_render_composition_diagnostics,
};
use crate::amigo_particles_2d_plugin::Particle2dSceneService;
use crate::amigo_composite_plugin::PostFx2dService;
use crate::amigo_2d_composition::RenderLayer2dCommand;

fn required<T>(runtime: &Runtime) -> AmigoResult<Arc<T>>
where
    T: Send + Sync + 'static,
{
    runtime
        .resolve::<T>()
        .ok_or(AmigoError::MissingService(type_name::<T>()))
}

#[derive(Debug, Clone, Copy)]
pub struct CameraFocusPlanInfo {
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

pub struct WgpuEditorOverlayOutput<'a>(pub &'a mut amigo_render_wgpu::WgpuRenderFramePacket);

impl amigo_editor_ingame::EditorOverlayRenderOutput for WgpuEditorOverlayOutput<'_> {
    fn push_editor_overlay_document(&mut self, document: amigo_overlay_api::UiOverlayDocument) {
        self.0.push_debug_overlay(document);
    }
}

pub fn extract_game_frame_packet(
    session: &RuntimeSession,
    include_game_ui: bool,
) -> AmigoResult<amigo_render_wgpu::WgpuRenderFramePacket> {
    session.begin_render_frame_extract();
    let mut render_packet = crate::default_wgpu_render_extractor_registry_for_runtime(
        session.runtime(),
    )
    .extract_all(session.runtime());
    render_packet.clear_debug_overlay();
    if !include_game_ui {
        render_packet.clear_game_ui_overlay();
    }
    session.complete_render_frame_extract();
    Ok(render_packet)
}

pub fn extract_live_host_overlay_packet(
    session: &RuntimeSession,
) -> AmigoResult<amigo_render_wgpu::WgpuRenderFramePacket> {
    let mut render_packet = crate::default_wgpu_render_extractor_registry_for_runtime(
        session.runtime(),
    )
    .extract_all(session.runtime());
    render_packet.clear_world_content();
    render_packet.clear_game_ui_overlay();
    amigo_editor_ingame::append_editor_overlay(
        session.runtime(),
        &mut WgpuEditorOverlayOutput(&mut render_packet),
    );
    Ok(render_packet)
}

pub fn render_game_frame_to_cache(
    session: &RuntimeSession,
    target: &mut amigo_render_wgpu::WgpuOffscreenTarget,
    renderer: &mut WgpuSceneRenderer,
    include_game_ui: bool,
) -> AmigoResult<()> {
    let runtime = session.runtime();
    let scene = required::<SceneService>(runtime)?;
    let assets = required::<AssetCatalog>(runtime)?;
    let render_layers = required::<RenderLayer2dSceneService>(runtime)?;
    let light_routes = required::<LightRoute2dSceneService>(runtime)?;
    let particles = required::<Particle2dSceneService>(runtime)?;
    let debug_overlay_service = required::<amigo_devtools::DebugOverlayService>(runtime)?;

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
            FrameGraphBuildInfo {
                width: target.width,
                height: target.height,
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
            window_width: target.width,
            window_height: target.height,
            world_2d_tilemaps: render_packet.renderable_2d_count_by_component_kind("TileMap2D"),
            world_2d_sprites: render_packet.renderable_2d_count_by_component_kind("Sprite2D"),
            world_2d_layered_images: render_packet
                .renderable_2d_count_by_component_kind("LayeredImage2D"),
            world_2d_render_layers: render_packet.world_2d_render_layers().len(),
            world_2d_light_routes: render_packet.world_2d_light_routes().len(),
            world_2d_global_lights: render_packet.light_source_2d_contribution_count(),
            world_2d_lightmaps: render_packet.lightmap_2d_contribution_count(),
            world_2d_light_groups: render_packet.light_group_2d_contribution_count(),
            world_2d_vectors: render_packet.renderable_2d_count_by_component_kind("VectorShape2D"),
            world_2d_beacons: render_packet.renderable_2d_count_by_component_kind("BeaconLight2D"),
            world_2d_text: render_packet.renderable_2d_count_by_component_kind("Text2D"),
            world_2d_particles: render_packet
                .renderable_2d_count_by_component_kind("ParticleEmitter2D"),
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

    if let Ok(post_fx_service) =
        required::<PostFx2dService>(runtime)
    {
        let has_post_fx = !render_packet.post_fx_stacks().is_empty();
        let renderer_mode = if has_post_fx {
            "frame_graph_postfx"
        } else {
            "frame_graph"
        };
        post_fx_service.set_renderer_mode(renderer_mode);
    }

    let extracted_render_layer_commands = render_layers.commands();
    let extracted_light_route_commands = light_routes.commands();
    let render_lightmaps_2d = render_packet
        .render_contributions_2d()
        .iter()
        .filter_map(RenderContribution2d::as_lightmap_2d)
        .cloned()
        .collect::<Vec<_>>();
    let render_depth_maps_2d = render_packet
        .render_contributions_2d()
        .iter()
        .filter_map(RenderContribution2d::as_depth_map_2d)
        .cloned()
        .collect::<Vec<_>>();
    let render_depth_aux_maps_2d = render_packet
        .render_contributions_2d()
        .iter()
        .filter_map(RenderContribution2d::as_depth_aux_map_2d)
        .cloned()
        .collect::<Vec<_>>();
    let camera_optical_candidates =
        crate::render_extractor_bridges::collect_camera_optical_candidates_from_light_sources_2d(
            render_packet.world_2d_light_sources(),
        );
    let render_request = amigo_render_wgpu::WgpuFrameRenderRequest {
        target: amigo_render_wgpu::WgpuFrameRenderTarget::Offscreen(target),
        scene: scene.as_ref(),
        assets: assets.as_ref(),
        world_2d: amigo_render_wgpu::WgpuWorld2dRenderInput {
            renderables: render_packet.renderables_2d(),
            depth_maps: render_depth_maps_2d.as_slice(),
            depth_aux_maps: render_depth_aux_maps_2d.as_slice(),
            lightmaps: render_lightmaps_2d.as_slice(),
            light_sources: render_packet.world_2d_light_sources(),
            camera_optical_candidates: camera_optical_candidates.as_slice(),
            render_layers: extracted_render_layer_commands.as_slice(),
            light_routes: extracted_light_route_commands.as_slice(),
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
            .unwrap_or_else(amigo_render_api::CameraDebugView2d::final_output),
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

pub fn render_camera_capture_summary(
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

pub fn render_visual_items_summary(
    renderables: &[amigo_render_wgpu::Renderable2dItem],
    render_layers: &[RenderLayer2dCommand],
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
                "primitive={} camera_pipeline={} base_z_depth={:.3} effective_z_depth={:.3} effective_distance_m={} z_depth={:.3} blur_scale={:.2} camera_motion_scale={:.2}",
                item.primitive_kind().as_str(),
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
                "primitive={} camera_pipeline={} z_depth=? blur_scale=? camera_motion_scale=?",
                item.primitive_kind().as_str(),
                item.uses_camera_pipeline()
            ));
        }
    }

    lines.join("\n")
}

pub fn camera_focus_for_input(
    runtime: &amigo_runtime::Runtime,
    assets: &AssetCatalog,
    input: &amigo_render_api::CameraCaptureInput2d,
) -> Option<CameraFocusPlanInfo> {
    let camera_service =
        required::<amigo_camera_core_plugin::CameraService>(runtime).ok()?;
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

pub fn render_camera_contributions_summary(
    runtime: &amigo_runtime::Runtime,
    assets: &AssetCatalog,
    input: Option<&amigo_render_api::CameraCaptureInput2d>,
    beacon_contributions_summary: Option<String>,
) -> Option<String> {
    let camera_service =
        required::<amigo_camera_core_plugin::CameraService>(runtime).ok()?;
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

pub fn render_camera_focus_plan_summary(
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
