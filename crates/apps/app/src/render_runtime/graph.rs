use amigo_render_api::{
    resource_for_input, resource_for_output, FrameCompositionPlan, FrameGraph,
    FrameGraphNodeKind, FrameResourceKind, RenderPassPlan,
};

#[derive(Debug, Clone, Copy)]
pub(crate) struct AppFrameGraphBuildInfo {
    pub(crate) width: u32,
    pub(crate) height: u32,
}

pub(crate) fn build_frame_graph_from_plan(
    plan: &FrameCompositionPlan,
    info: AppFrameGraphBuildInfo,
) -> FrameGraph {
    let mut graph = FrameGraph::new();

    let surface = graph.add_resource("surface", FrameResourceKind::SurfaceColor);
    let world = graph.add_resource(
        "world_color",
        FrameResourceKind::TextureColor {
            width: info.width,
            height: info.height,
            transient: true,
        },
    );
    let post_fx = graph.add_resource(
        "post_fx_color",
        FrameResourceKind::TextureColor {
            width: info.width,
            height: info.height,
            transient: true,
        },
    );

    for view in &plan.views {
        for pass in &view.passes {
            match pass {
                RenderPassPlan::World2D(pass) => {
                    let output = resource_for_output(pass.output, surface, world, post_fx);
                    graph.add_node("world_2d", FrameGraphNodeKind::World2D, vec![], vec![output]);
                }
                RenderPassPlan::World3D(pass) => {
                    let output = resource_for_output(pass.output, surface, world, post_fx);
                    graph.add_node("world_3d", FrameGraphNodeKind::World3D, vec![], vec![output]);
                }
                RenderPassPlan::PostFx(pass) => {
                    let input = resource_for_input(pass.input, surface, world, post_fx);
                    let output = resource_for_output(pass.output, surface, world, post_fx);
                    graph.add_node(
                        pass.kind.label(),
                        FrameGraphNodeKind::PostFx(pass.kind),
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::GameUi(pass) => {
                    let input = resource_for_input(pass.input, surface, world, post_fx);
                    let output = resource_for_output(pass.output, surface, world, post_fx);
                    graph.add_node(
                        "game_ui",
                        FrameGraphNodeKind::GameUi,
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::DebugOverlay(pass) => {
                    let input = resource_for_input(pass.input, surface, world, post_fx);
                    let output = resource_for_output(pass.output, surface, world, post_fx);
                    graph.add_node(
                        "debug_overlay",
                        FrameGraphNodeKind::DebugOverlay,
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::Present(pass) => {
                    let input = resource_for_input(pass.input, surface, world, post_fx);
                    graph.add_node(
                        "present",
                        FrameGraphNodeKind::Present,
                        input.into_iter().collect(),
                        vec![surface],
                    );
                }
            }
        }
    }

    graph
}
