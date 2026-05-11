use amigo_core::AmigoResult;
use amigo_render_api::{FrameGraph, FrameGraphNodeKind, FrameResourceKind};

use crate::renderer::graph::WgpuFrameResourceAllocator;
use crate::renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer};

#[derive(Default)]
pub(crate) struct WgpuFrameGraphExecutor {
    resources: WgpuFrameResourceAllocator,
}

impl WgpuFrameGraphExecutor {
    pub(crate) fn execute(
        &mut self,
        renderer: &mut WgpuSceneRenderer,
        request: WgpuFrameRenderRequest<'_>,
    ) -> AmigoResult<()> {
        self.prepare_transient_resources(request.frame_graph, &request);

        let _plan = split_graph_plan(request.frame_graph);
        renderer.render_frame_request_graph(request)
    }

    pub(crate) fn prepare_transient_resources(
        &mut self,
        graph: &FrameGraph,
        request: &WgpuFrameRenderRequest<'_>,
    ) {
        self.resources.clear();

        for resource in &graph.resources {
            if let FrameResourceKind::TextureColor {
                width,
                height,
                transient: true,
            } = resource.kind
            {
                self.resources.create_color_texture(
                    &request.surface.device,
                    resource.id,
                    &format!("amigo-framegraph-{}", resource.label),
                    width,
                    height,
                    request.surface.config.format,
                );
            }
        }
    }

}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
struct WgpuSplitGraphPlan {
    has_world: bool,
    has_post_fx: bool,
    has_game_ui: bool,
    has_debug_overlay: bool,
    has_present: bool,
}

fn split_graph_plan(graph: &FrameGraph) -> Option<WgpuSplitGraphPlan> {
    let mut plan = WgpuSplitGraphPlan::default();
    let mut phase = WgpuSplitGraphPhase::World;

    for node in &graph.nodes {
        match node.kind {
            FrameGraphNodeKind::World2D | FrameGraphNodeKind::World3D => {
                if phase > WgpuSplitGraphPhase::World {
                    return None;
                }
                plan.has_world = true;
            }
            FrameGraphNodeKind::PostFx(_) => {
                if phase > WgpuSplitGraphPhase::PostFx {
                    return None;
                }
                phase = WgpuSplitGraphPhase::PostFx;
                plan.has_post_fx = true;
            }
            FrameGraphNodeKind::GameUi => {
                if phase > WgpuSplitGraphPhase::GameUi {
                    return None;
                }
                phase = WgpuSplitGraphPhase::GameUi;
                plan.has_game_ui = true;
            }
            FrameGraphNodeKind::DebugOverlay => {
                if phase > WgpuSplitGraphPhase::DebugOverlay {
                    return None;
                }
                phase = WgpuSplitGraphPhase::DebugOverlay;
                plan.has_debug_overlay = true;
            }
            FrameGraphNodeKind::Present => {
                if phase > WgpuSplitGraphPhase::Present {
                    return None;
                }
                phase = WgpuSplitGraphPhase::Present;
                plan.has_present = true;
            }
        }
    }

    (plan.has_present && plan.has_world).then_some(plan)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum WgpuSplitGraphPhase {
    World,
    PostFx,
    GameUi,
    DebugOverlay,
    Present,
}

#[cfg(test)]
mod tests {
    use super::*;
    use amigo_render_api::{
        FrameGraphNodeKind, FrameResourceKind, PostFxPassKind,
    };

    #[test]
    fn split_graph_plan_accepts_world_post_fx_ui_debug_present() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        let world = graph.add_resource(
            "world",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node("world_2d", FrameGraphNodeKind::World2D, vec![], vec![world]);
        graph.add_node(
            "post_fx_lens_droplets",
            FrameGraphNodeKind::PostFx(PostFxPassKind::LensDroplets),
            vec![world],
            vec![surface],
        );
        graph.add_node("game_ui", FrameGraphNodeKind::GameUi, vec![surface], vec![surface]);
        graph.add_node(
            "debug_overlay",
            FrameGraphNodeKind::DebugOverlay,
            vec![surface],
            vec![surface],
        );
        graph.add_node("present", FrameGraphNodeKind::Present, vec![surface], vec![surface]);

        let plan = split_graph_plan(&graph).expect("split graph should be supported");
        assert!(plan.has_world);
        assert!(plan.has_post_fx);
        assert!(plan.has_game_ui);
        assert!(plan.has_debug_overlay);
        assert!(plan.has_present);
    }

    #[test]
    fn split_graph_plan_rejects_debug_before_game_ui() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        let world = graph.add_resource(
            "world",
            FrameResourceKind::TextureColor {
                width: 1280,
                height: 720,
                transient: true,
            },
        );

        graph.add_node("world_2d", FrameGraphNodeKind::World2D, vec![], vec![world]);
        graph.add_node(
            "debug_overlay",
            FrameGraphNodeKind::DebugOverlay,
            vec![surface],
            vec![surface],
        );
        graph.add_node("game_ui", FrameGraphNodeKind::GameUi, vec![surface], vec![surface]);
        graph.add_node("present", FrameGraphNodeKind::Present, vec![surface], vec![surface]);

        assert!(split_graph_plan(&graph).is_none());
    }

    #[test]
    fn split_graph_plan_rejects_ui_only_graph_for_now() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
        graph.add_node("game_ui", FrameGraphNodeKind::GameUi, vec![surface], vec![surface]);
        graph.add_node("present", FrameGraphNodeKind::Present, vec![surface], vec![surface]);

        assert!(split_graph_plan(&graph).is_none());
    }
}
