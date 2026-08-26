use crate::{
    FrameCompositionPlan, FrameGraph, FrameGraphNodeKind, FrameResourceId, FrameResourceKind,
    RenderPassPlan, RenderTargetPlan, resource_for_input, resource_for_output,
};

#[derive(Debug, Clone, Copy)]
pub struct FrameGraphBuildInfo {
    pub width: u32,
    pub height: u32,
}

pub fn build_frame_graph_from_plan(
    plan: &FrameCompositionPlan,
    info: FrameGraphBuildInfo,
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
        let target = view_target_resource(&mut graph, view.id.as_str(), view.target, surface);
        for pass in &view.passes {
            match pass {
                RenderPassPlan::World(pass) => {
                    let output = resource_for_output(pass.output, target, world, post_fx);
                    debug_assert_ne!(output, target, "only Present may write the view target");
                    graph.add_node("world", FrameGraphNodeKind::World, vec![], vec![output]);
                }
                RenderPassPlan::PostFx(pass) => {
                    let input = resource_for_input(pass.input, target, world, post_fx);
                    let output = resource_for_output(pass.output, target, world, post_fx);
                    debug_assert_ne!(output, target, "only Present may write the view target");
                    graph.add_node(
                        format!(
                            "post_fx:{}:{}:{}",
                            pass.host_id.as_str(),
                            pass.effect_id.as_str(),
                            pass.feature_id
                        ),
                        FrameGraphNodeKind::PostFx {
                            host_id: pass.host_id.clone(),
                            effect_id: pass.effect_id.clone(),
                            scope: pass.scope.clone(),
                            pipeline: pass.pipeline,
                            feature_id: pass.feature_id.clone(),
                        },
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::GameUi(pass) => {
                    let input = resource_for_input(pass.input, target, world, post_fx);
                    let output = resource_for_output(pass.output, target, world, post_fx);
                    debug_assert_ne!(output, target, "only Present may write the view target");
                    graph.add_node(
                        "game_ui",
                        FrameGraphNodeKind::GameUi,
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::DebugOverlay(pass) => {
                    let input = resource_for_input(pass.input, target, world, post_fx);
                    let output = resource_for_output(pass.output, target, world, post_fx);
                    debug_assert_ne!(output, target, "only Present may write the view target");
                    graph.add_node(
                        "debug_overlay",
                        FrameGraphNodeKind::DebugOverlay,
                        input.into_iter().collect(),
                        vec![output],
                    );
                }
                RenderPassPlan::Present(pass) => {
                    let input = resource_for_input(pass.input, target, world, post_fx);
                    graph.add_node(
                        "present",
                        FrameGraphNodeKind::Present,
                        input.into_iter().collect(),
                        vec![target],
                    );
                }
            }
        }
    }

    graph
}

fn view_target_resource(
    graph: &mut FrameGraph,
    view_id: &str,
    target: RenderTargetPlan,
    surface: FrameResourceId,
) -> FrameResourceId {
    match target {
        RenderTargetPlan::Surface => surface,
        RenderTargetPlan::Offscreen { width, height } => graph.add_resource(
            format!("view:{view_id}:target"),
            FrameResourceKind::TextureColor {
                width,
                height,
                transient: false,
            },
        ),
    }
}
