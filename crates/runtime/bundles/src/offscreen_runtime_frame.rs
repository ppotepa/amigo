use amigo_assets::AssetCatalog;
use amigo_core::AmigoResult;
use amigo_input_api::InputState;
use amigo_render_api::{FrameGraphBuildInfo, RenderTargetPlan, build_frame_graph_from_plan};
use amigo_render_wgpu::{WgpuEmergencyOverlayLine, WgpuOffscreenTarget, WgpuSceneRenderer};
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use crate::{
    WgpuFrameCompositionBuilder, WgpuFrameSubmitInput,
    default_wgpu_render_extractor_registry_for_runtime, submit_wgpu_frame_render_request,
};

pub fn set_runtime_ui_viewport_state(
    runtime: &Runtime,
    width: f32,
    height: f32,
) -> AmigoResult<()> {
    let state = runtime.required::<amigo_ui::UiInputViewportState>()?;
    state.set(Some(amigo_overlay_api::UiViewportSize::new(width, height)));
    Ok(())
}

pub fn clear_runtime_frame_transients(runtime: &Runtime) {
    if let Some(input_state) = runtime.resolve::<InputState>() {
        input_state.clear_frame_transients();
    }
    if let Some(ui_input) = runtime.resolve::<amigo_ui::UiInputService>() {
        ui_input.clear_frame_transients();
    }
}

pub struct WgpuOffscreenRuntimeFrameInput<'a> {
    pub runtime: &'a Runtime,
    pub target: &'a mut WgpuOffscreenTarget,
    pub renderer: &'a mut WgpuSceneRenderer,
    pub emergency_overlay: &'a [WgpuEmergencyOverlayLine],
}

pub fn render_wgpu_runtime_frame_to_offscreen(
    input: WgpuOffscreenRuntimeFrameInput<'_>,
) -> AmigoResult<()> {
    let scene = input.runtime.required::<SceneService>()?;
    let assets = input.runtime.required::<AssetCatalog>()?;
    let render_layers = input
        .runtime
        .required::<amigo_2d_composition::RenderLayer2dSceneService>()?;
    let light_routes = input
        .runtime
        .required::<amigo_2d_composition::LightRoute2dSceneService>()?;
    require_wgpu_runtime_frame_services(input.runtime)?;

    let render_packet = default_wgpu_render_extractor_registry_for_runtime(input.runtime)
        .extract_all(input.runtime);
    let composition_plan = WgpuFrameCompositionBuilder::build_for_target(
        &render_packet,
        RenderTargetPlan::Offscreen {
            width: input.target.width,
            height: input.target.height,
        },
    );
    let frame_graph = build_frame_graph_from_plan(
        &composition_plan,
        FrameGraphBuildInfo {
            width: input.target.width,
            height: input.target.height,
        },
    );
    let extracted_render_layer_commands = render_layers.commands();
    let extracted_light_route_commands = light_routes.commands();

    submit_wgpu_frame_render_request(
        input.renderer,
        WgpuFrameSubmitInput {
            target: amigo_render_wgpu::WgpuFrameRenderTarget::Offscreen(input.target),
            scene: scene.as_ref(),
            assets: assets.as_ref(),
            render_packet: &render_packet,
            render_layers: extracted_render_layer_commands.as_slice(),
            light_routes: extracted_light_route_commands.as_slice(),
            debug_ui: render_packet.debug_overlay(),
            emergency_overlay: input.emergency_overlay,
            composition_plan: &composition_plan,
            frame_graph: &frame_graph,
            game_viewport: None,
        },
    )
}

fn require_wgpu_runtime_frame_services(runtime: &Runtime) -> AmigoResult<()> {
    let _ = runtime.required::<amigo_light_2d_plugin::LightMap2dSceneService>()?;
    let _ = runtime.required::<amigo_light_2d_plugin::LightGroup2dSceneService>()?;
    let _ = runtime.required::<amigo_3d_mesh::MeshSceneService>()?;
    let _ = runtime.required::<amigo_3d_text::Text3dSceneService>()?;
    let _ = runtime.required::<amigo_3d_material::MaterialSceneService>()?;
    let _ = runtime.required::<amigo_ui::UiSceneService>()?;
    let _ = runtime.required::<amigo_ui::UiStateService>()?;
    let _ = runtime.required::<amigo_ui::UiThemeService>()?;
    let _ = runtime.required::<amigo_composite_plugin::PostFx2dService>()?;
    let _ = runtime.required::<amigo_scripting_api::DevConsoleState>()?;
    let _ = runtime.required::<amigo_devtools::ConsoleCompletionState>()?;
    let _ = runtime.required::<amigo_devtools::DebugOverlayService>()?;
    let _ = runtime.required::<amigo_ui::UiInputViewportState>()?;
    Ok(())
}
