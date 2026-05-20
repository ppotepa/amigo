use amigo_assets::AssetCatalog;
use amigo_render_api::{
    FrameCompositionPlan, FrameGraph, RenderCompositionDiagnosticsService,
    RenderCompositionDiagnosticsUpdate,
};
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use crate::{
    camera_focus_for_input, render_camera_capture_summary, render_camera_contributions_summary,
    render_camera_focus_plan_summary, render_visual_items_summary,
};

pub fn update_wgpu_render_composition_diagnostics(
    runtime: &Runtime,
    assets: &AssetCatalog,
    render_packet: &WgpuRenderFramePacket,
    composition_plan: &FrameCompositionPlan,
    frame_graph: &FrameGraph,
) {
    let Ok(render_diagnostics) = runtime.required::<RenderCompositionDiagnosticsService>() else {
        return;
    };

    render_diagnostics.set_with_update(
        composition_plan,
        frame_graph,
        RenderCompositionDiagnosticsUpdate {
            camera_capture_summary: render_packet.camera_capture_input_2d().map(|input| {
                render_camera_capture_summary(input, render_packet.visual_source_flags_2d())
            }),
            camera_focus_plan_summary: render_packet.camera_capture_input_2d().map(|input| {
                render_camera_focus_plan_summary(
                    input,
                    camera_focus_for_input(runtime, assets, input),
                )
            }),
            light_sources_summary: Some(crate::render_extractor_bridges::format_light_sources_2d(
                render_packet.world_2d_light_sources(),
            )),
            camera_optical_candidates_summary: Some(
                amigo_camera_optics_plugin::diagnostics::format_camera_optical_candidates_2d(
                    render_packet.camera_optical_candidates_2d(),
                ),
            ),
            render_contributions_summary: render_camera_contributions_summary(
                runtime,
                assets,
                render_packet.camera_capture_input_2d(),
                crate::render_extractor_bridges::render_contribution_decisions_summary(
                    render_packet.renderables_2d(),
                    render_packet.world_2d_light_sources(),
                ),
            ),
            render_materials_summary: None,
            visual_items_summary: Some(render_visual_items_summary(
                render_packet.renderables_2d(),
                render_packet.world_2d_render_layers(),
                render_packet
                    .camera_capture_input_2d()
                    .map(|input| input.layers.as_slice())
                    .unwrap_or(&[]),
            )),
        },
    );
}

pub fn update_wgpu_postfx_renderer_mode(runtime: &Runtime, render_packet: &WgpuRenderFramePacket) {
    let Ok(post_fx_service) = runtime.required::<amigo_composite_plugin::PostFx2dService>() else {
        return;
    };

    let renderer_mode = if render_packet.post_fx_stacks().is_empty() {
        "frame_graph"
    } else {
        "frame_graph_postfx"
    };
    post_fx_service.set_renderer_mode(renderer_mode);
}

pub fn particle_debug_snapshot(runtime: &Runtime) -> Option<(usize, usize)> {
    let particles = runtime.resolve::<amigo_particles_2d_plugin::Particle2dSceneService>()?;
    let emitters = particles.emitters();
    let active_emitters = emitters
        .iter()
        .filter(|emitter| particles.is_active(&emitter.entity_name))
        .count();
    Some((emitters.len(), active_emitters))
}
