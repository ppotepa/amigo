use crate::composition::FrameCompositionPlan;
use crate::frame_graph::{FrameGraph, FrameGraphNodeKind, FrameResourceKind};
use std::collections::BTreeSet;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderCompositionDiagnostics {
    pub composition_summary: String,
    pub graph_summary: String,
    pub camera_capture_summary: String,
    pub camera_focus_plan_summary: String,
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
            camera_capture_summary: String::new(),
            camera_focus_plan_summary: String::new(),
            warnings: collect_graph_warnings(graph),
        }
    }
}

fn collect_graph_warnings(graph: &FrameGraph) -> Vec<String> {
    let mut warnings = Vec::new();

    let surface_ids: BTreeSet<_> = graph
        .resources
        .iter()
        .filter(|resource| matches!(resource.kind, FrameResourceKind::SurfaceColor))
        .map(|resource| resource.id)
        .collect();

    let has_post_fx = graph
        .nodes
        .iter()
        .any(|node| matches!(node.kind, FrameGraphNodeKind::PostFx { .. }));

    for node in &graph.nodes {
        let writes_surface = node
            .writes
            .iter()
            .any(|resource_id| surface_ids.contains(resource_id));

        if !matches!(node.kind, FrameGraphNodeKind::Present) && writes_surface {
            warnings.push(format!(
                "non-present node '{}' writes surface resource",
                node.label
            ));
        }

        match &node.kind {
            FrameGraphNodeKind::World => {
                if node.writes.is_empty() {
                    warnings.push(format!("world node '{}' has no writes", node.label));
                }
            }
            FrameGraphNodeKind::PostFx { .. } => {
                if node.reads.is_empty() {
                    warnings.push(format!("post-fx node '{}' has no reads", node.label));
                }
                if node.writes.is_empty() {
                    warnings.push(format!("post-fx node '{}' has no writes", node.label));
                }
            }
            FrameGraphNodeKind::GameUi => {
                if node.reads.is_empty() {
                    warnings.push(format!("game-ui node '{}' has no reads", node.label));
                }
                if node.writes.is_empty() {
                    warnings.push(format!("game-ui node '{}' has no writes", node.label));
                }
            }
            FrameGraphNodeKind::DebugOverlay => {
                if node.reads.is_empty() {
                    warnings.push(format!("debug-overlay node '{}' has no reads", node.label));
                }
                if node.writes.is_empty() {
                    warnings.push(format!("debug-overlay node '{}' has no writes", node.label));
                }
            }
            FrameGraphNodeKind::Present => {
                if node.reads.is_empty() {
                    warnings.push(format!("present node '{}' has no reads", node.label));
                }
            }
        }
    }

    let mut saw_post_fx = false;
    let mut saw_debug_overlay = false;
    for node in &graph.nodes {
        match &node.kind {
            FrameGraphNodeKind::DebugOverlay if has_post_fx && !saw_post_fx => {
                warnings.push(format!(
                    "debug-overlay node '{}' appears before post-fx",
                    node.label
                ));
            }
            FrameGraphNodeKind::PostFx { .. } if saw_debug_overlay => warnings.push(format!(
                "post-fx node '{}' appears after debug-overlay",
                node.label
            )),
            _ => {}
        }

        if matches!(node.kind, FrameGraphNodeKind::PostFx { .. }) {
            saw_post_fx = true;
        }

        if matches!(node.kind, FrameGraphNodeKind::DebugOverlay) {
            saw_debug_overlay = true;
        }
    }

    warnings
}

use std::sync::Mutex;

#[derive(Debug, Default)]
pub struct RenderCompositionDiagnosticsService {
    inner: Mutex<RenderCompositionDiagnostics>,
}

impl RenderCompositionDiagnosticsService {
    pub fn set(&self, plan: &FrameCompositionPlan, graph: &FrameGraph) {
        self.set_with_camera_capture(plan, graph, None);
    }

    pub fn set_with_camera_capture(
        &self,
        plan: &FrameCompositionPlan,
        graph: &FrameGraph,
        camera_capture_summary: Option<String>,
    ) {
        self.set_with_camera_capture_and_focus_plan(plan, graph, camera_capture_summary, None);
    }

    pub fn set_with_camera_capture_and_focus_plan(
        &self,
        plan: &FrameCompositionPlan,
        graph: &FrameGraph,
        camera_capture_summary: Option<String>,
        camera_focus_plan_summary: Option<String>,
    ) {
        let mut diagnostics = RenderCompositionDiagnostics::from_plan_and_graph(plan, graph);
        if let Some(summary) = camera_capture_summary {
            diagnostics.camera_capture_summary = summary;
        }
        if let Some(summary) = camera_focus_plan_summary {
            diagnostics.camera_focus_plan_summary = summary;
        }
        *self
            .inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned") = diagnostics;
    }

    pub fn snapshot(&self) -> RenderCompositionDiagnostics {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .clone()
    }
}
