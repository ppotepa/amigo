use crate::composition::FrameCompositionPlan;
use crate::frame_graph::FrameGraph;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderCompositionDiagnostics {
    pub composition_summary: String,
    pub graph_summary: String,
    pub warnings: Vec<String>,
}

impl RenderCompositionDiagnostics {
    pub fn from_plan_and_graph(plan: &FrameCompositionPlan, graph: &FrameGraph) -> Self {
        let composition_summary = plan
            .views
            .iter()
            .map(|view| {
                let passes = view
                    .passes
                    .iter()
                    .map(|pass| pass.label())
                    .collect::<Vec<_>>()
                    .join(" -> ");
                format!("view={} passes=[{}]", view.id, passes)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let graph_summary = graph
            .nodes
            .iter()
            .enumerate()
            .map(|(index, node)| {
                format!(
                    "{:02} {} reads={:?} writes={:?}",
                    index + 1,
                    node.label,
                    node.reads,
                    node.writes
                )
            })
            .collect::<Vec<_>>()
            .join("\n");

        Self {
            composition_summary,
            graph_summary,
            warnings: Vec::new(),
        }
    }
}
