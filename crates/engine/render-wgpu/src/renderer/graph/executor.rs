use amigo_core::AmigoResult;
use amigo_render_api::{FrameGraph, FrameGraphNode, FrameGraphNodeKind, FrameResourceKind};

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
        mut request: WgpuFrameRenderRequest<'_>,
    ) -> AmigoResult<()> {
        self.prepare_transient_resources(request.frame_graph, &request)?;

        let node_count = request.frame_graph.nodes.len();
        for index in 0..node_count {
            let node = request.frame_graph.nodes[index].clone();
            self.execute_node(renderer, &mut request, &node)?;
        }

        Ok(())
    }

    fn execute_node(
        &mut self,
        renderer: &mut WgpuSceneRenderer,
        request: &mut WgpuFrameRenderRequest<'_>,
        node: &FrameGraphNode,
    ) -> AmigoResult<()> {
        match &node.kind {
            FrameGraphNodeKind::World2D => {
                renderer.execute_world_2d_graph_node(request, node, &mut self.resources)
            }
            FrameGraphNodeKind::World3D => {
                renderer.execute_world_3d_graph_node(request, node, &mut self.resources)
            }
            FrameGraphNodeKind::PostFx {
                feature_id,
                effect_index,
            } => {
                renderer.execute_post_fx_graph_node(
                    request,
                    node,
                    feature_id.clone(),
                    *effect_index,
                    &mut self.resources,
                )
            }
            FrameGraphNodeKind::GameUi => {
                renderer.execute_game_ui_graph_node(request, node, &mut self.resources)
            }
            FrameGraphNodeKind::DebugOverlay => {
                renderer.execute_debug_overlay_graph_node(request, node, &mut self.resources)
            }
            FrameGraphNodeKind::Present => {
                renderer.execute_present_graph_node(request, node, &mut self.resources)
            }
        }
    }

    pub(crate) fn prepare_transient_resources(
        &mut self,
        graph: &FrameGraph,
        request: &WgpuFrameRenderRequest<'_>,
    ) -> AmigoResult<()> {
        self.resources.clear();

        for resource in &graph.resources {
            if let FrameResourceKind::TextureColor {
                width,
                height,
                transient: true,
            } = resource.kind
            {
                self.resources.create_color_texture(
                    request.target.device(),
                    request.target.queue(),
                    resource.id,
                    &format!("amigo-framegraph-{}", resource.label),
                    width,
                    height,
                    request.target.format(),
                )?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use amigo_render_api::{FrameGraph, FrameGraphNodeKind, FrameResourceKind};

    #[test]
    fn executor_graph_model_can_represent_world_ui_debug_present() {
        let mut graph = FrameGraph::new();
        let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);

        graph.add_node("world_2d", FrameGraphNodeKind::World2D, vec![], vec![surface]);
        graph.add_node("game_ui", FrameGraphNodeKind::GameUi, vec![surface], vec![surface]);
        graph.add_node(
            "debug_overlay",
            FrameGraphNodeKind::DebugOverlay,
            vec![surface],
            vec![surface],
        );
        graph.add_node("present", FrameGraphNodeKind::Present, vec![surface], vec![surface]);

        assert_eq!(
            graph.node_labels(),
            vec!["world_2d", "game_ui", "debug_overlay", "present"]
        );
    }
}
