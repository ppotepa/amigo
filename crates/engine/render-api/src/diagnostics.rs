use crate::composition::FrameCompositionPlan;
use crate::frame_graph::{FrameGraph, FrameGraphNodeKind, FrameResourceKind};
use std::collections::BTreeSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderFrameDiagnostic {
    pub code: String,
    pub message: String,
}

impl RenderFrameDiagnostic {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderCompositionDiagnostics {
    pub composition_summary: String,
    pub graph_summary: String,
    pub camera_capture_summary: String,
    pub camera_focus_plan_summary: String,
    pub light_sources_summary: String,
    pub camera_optical_candidates_summary: String,
    pub plate_relight_summary: String,
    pub render_contributions_summary: String,
    pub render_materials_summary: String,
    pub visual_items_summary: String,
    pub frame_diagnostics: Vec<RenderFrameDiagnostic>,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct RenderCompositionDiagnosticsUpdate {
    pub camera_capture_summary: Option<String>,
    pub camera_focus_plan_summary: Option<String>,
    pub light_sources_summary: Option<String>,
    pub camera_optical_candidates_summary: Option<String>,
    pub render_contributions_summary: Option<String>,
    pub render_materials_summary: Option<String>,
    pub visual_items_summary: Option<String>,
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
            light_sources_summary: String::new(),
            camera_optical_candidates_summary: String::new(),
            plate_relight_summary: String::new(),
            render_contributions_summary: String::new(),
            render_materials_summary: String::new(),
            visual_items_summary: String::new(),
            frame_diagnostics: Vec::new(),
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
        self.set_with_update(
            plan,
            graph,
            RenderCompositionDiagnosticsUpdate {
                camera_capture_summary,
                camera_focus_plan_summary,
                ..Default::default()
            },
        );
    }

    pub fn set_with_update(
        &self,
        plan: &FrameCompositionPlan,
        graph: &FrameGraph,
        update: RenderCompositionDiagnosticsUpdate,
    ) {
        let mut diagnostics = RenderCompositionDiagnostics::from_plan_and_graph(plan, graph);
        if let Some(summary) = update.camera_capture_summary {
            diagnostics.camera_capture_summary = summary;
        }
        if let Some(summary) = update.camera_focus_plan_summary {
            diagnostics.camera_focus_plan_summary = summary;
        }
        if let Some(summary) = update.light_sources_summary {
            diagnostics.light_sources_summary = summary;
        }
        if let Some(summary) = update.camera_optical_candidates_summary {
            diagnostics.camera_optical_candidates_summary = summary;
        }
        if let Some(summary) = update.render_contributions_summary {
            diagnostics.render_contributions_summary = summary;
        }
        if let Some(summary) = update.render_materials_summary {
            diagnostics.render_materials_summary = summary;
        }
        if let Some(summary) = update.visual_items_summary {
            diagnostics.visual_items_summary = summary;
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

    pub fn set_plate_relight_summary(&self, summary: impl Into<String>) {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .plate_relight_summary = summary.into();
    }

    pub fn set_light_sources_summary(&self, summary: impl Into<String>) {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .light_sources_summary = summary.into();
    }

    pub fn set_render_materials_summary(&self, summary: impl Into<String>) {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .render_materials_summary = summary.into();
    }

    pub fn set_visual_items_summary(&self, summary: impl Into<String>) {
        self.inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned")
            .visual_items_summary = summary.into();
    }

    pub fn set_frame_diagnostics(&self, diagnostics: Vec<RenderFrameDiagnostic>) {
        let mut inner = self
            .inner
            .lock()
            .expect("render composition diagnostics mutex should not be poisoned");
        inner.warnings.extend(
            diagnostics
                .iter()
                .map(|diagnostic| format!("{}: {}", diagnostic.code, diagnostic.message)),
        );
        inner.frame_diagnostics = diagnostics;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn render_diagnostics_service_stores_plate_relight_summary() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_plate_relight_summary("abc");
        assert_eq!(service.snapshot().plate_relight_summary, "abc");
    }

    #[test]
    fn render_diagnostics_service_stores_render_contributions_summary() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_with_update(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &FrameGraph::default(),
            RenderCompositionDiagnosticsUpdate {
                render_contributions_summary: Some("render.contributions ok".to_owned()),
                ..Default::default()
            },
        );

        assert_eq!(
            service.snapshot().render_contributions_summary,
            "render.contributions ok"
        );
    }

    #[test]
    fn render_diagnostics_service_stores_render_materials_summary() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_with_update(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &FrameGraph::default(),
            RenderCompositionDiagnosticsUpdate {
                render_materials_summary: Some("render.materials ok".to_owned()),
                ..Default::default()
            },
        );

        assert_eq!(
            service.snapshot().render_materials_summary,
            "render.materials ok"
        );
    }

    #[test]
    fn render_diagnostics_service_stores_visual_items_summary() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_visual_items_summary("render.visual.items ok");

        assert_eq!(
            service.snapshot().visual_items_summary,
            "render.visual.items ok"
        );
    }

    #[test]
    fn render_diagnostics_service_stores_light_sources_summary() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_light_sources_summary("render.light.sources ok");

        assert_eq!(
            service.snapshot().light_sources_summary,
            "render.light.sources ok"
        );
    }

    #[test]
    fn render_diagnostics_service_exposes_frame_diagnostics_as_warnings() {
        let service = RenderCompositionDiagnosticsService::default();
        service.set_with_update(
            &FrameCompositionPlan::single_main_view(Vec::new()),
            &FrameGraph::default(),
            RenderCompositionDiagnosticsUpdate::default(),
        );

        service.set_frame_diagnostics(vec![RenderFrameDiagnostic::new(
            "postfx.focus_blur.depth_map_missing",
            "Depth map 'main-depth' was requested but not rendered.",
        )]);

        let snapshot = service.snapshot();
        assert_eq!(
            snapshot.frame_diagnostics,
            vec![RenderFrameDiagnostic::new(
                "postfx.focus_blur.depth_map_missing",
                "Depth map 'main-depth' was requested but not rendered."
            )]
        );
        assert!(snapshot.warnings.iter().any(|warning| {
            warning.contains("postfx.focus_blur.depth_map_missing")
                && warning.contains("main-depth")
        }));
    }
}
