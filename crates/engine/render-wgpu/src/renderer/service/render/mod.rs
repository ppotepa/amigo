use crate::renderer::*;

mod emergency_overlay;
mod focus_depth_plan;
mod graph_nodes;
mod layered_image_parts_pass;
mod material_candidates;
mod offscreen_ops;
mod plate_relight;
mod presentation;
mod refractive_material;
mod scoped_post_fx;
#[cfg(test)]
mod tests;
mod ui_pass;
mod util;
mod visual_debug;
mod visual_source_buffer_pass;
mod world;
mod world_filters;
mod world_selection;

use self::emergency_overlay::emergency_overlay_lines;
#[cfg(test)]
use self::focus_depth_plan::build_focus_blur_layer_plan;
use self::focus_depth_plan::{depth_debug_post_fx_for, focus_blur_effect_for, replay_scoped_layers_plan_for_effect};
use self::layered_image_parts_pass::execute_layered_image_parts_to_offscreen;
pub(crate) use self::material_candidates::{collect_material_candidate_2d, WgpuMaterialCandidate2d};
use self::world::WorldRenderContext;
use self::world_selection::{WorldPassLoadExt, WorldRenderSelection, base_world_selection};

impl WgpuSceneRenderer {
    pub fn render_frame_request(&mut self, request: WgpuFrameRenderRequest<'_>) -> AmigoResult<()> {
        let mut executor = std::mem::take(&mut self.frame_graph_executor);
        let result = executor.execute(self, request);
        self.frame_graph_executor = executor;
        result
    }

    pub(crate) fn execute_world_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        graph_nodes::execute_world_graph_node(self, request, node, resources)
    }

    pub(crate) fn execute_post_fx_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        host_id: &amigo_render_api::PostFxHost2dId,
        effect_id: &amigo_render_api::PostFx2dId,
        scope: &amigo_render_api::PostFxScope2d,
        pipeline: amigo_render_api::PostFxPipelineKind,
        feature_id: amigo_render_api::RenderFeatureId,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        let post_fx = scoped_post_fx::PostFxGraphNodeContext {
            host_id,
            effect_id,
            scope,
            pipeline,
            feature_id,
        };
        graph_nodes::execute_post_fx_graph_node(
            self, request, node, post_fx, resources,
        )
    }

    pub(crate) fn execute_game_ui_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        graph_nodes::execute_game_ui_graph_node(self, request, node, resources)
    }

    pub(crate) fn execute_debug_overlay_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        graph_nodes::execute_debug_overlay_graph_node(self, request, node, resources)
    }

    pub(crate) fn execute_present_graph_node(
        &mut self,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &amigo_render_api::FrameGraphNode,
        resources: &mut crate::renderer::graph::WgpuFrameResourceAllocator,
    ) -> AmigoResult<()> {
        graph_nodes::execute_present_graph_node(self, request, node, resources)
    }

    pub fn present_cached_frame_to_surface(
        &mut self,
        surface: &mut WgpuSurfaceState,
        source: &WgpuOffscreenTarget,
        assets: &AssetCatalog,
        live_overlay_ui: &[UiOverlayDocument],
        game_viewport: Option<WgpuGameViewportPlacement>,
        emergency_overlay: &[WgpuEmergencyOverlayLine],
    ) -> AmigoResult<()> {
        self.render_texture_to_surface(
            surface,
            &source.view,
            assets,
            live_overlay_ui,
            game_viewport,
            emergency_overlay,
        )
    }
}
