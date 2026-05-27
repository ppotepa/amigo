use amigo_codemap_api::{
    navigate_to_targets, validate_codemap_graph, CodeMapEdge, CodeMapEdgeKind, CodeMapGraph,
    CodeMapNode, CodeMapNodeId, CodeMapNodeKind, WaterfallTrace,
};
use amigo_plugin_api::{PluginId, TargetId};

#[test]
fn graph_with_known_nodes_validates() {
    let plugin = CodeMapNodeId::Plugin(PluginId("amigo.camera.camera-optics".to_string()));
    let target = CodeMapNodeId::Target(TargetId("SceneHighlight".to_string()));

    let mut graph = CodeMapGraph::new();

    graph.add_node(CodeMapNode::new(
        plugin.clone(),
        CodeMapNodeKind::Plugin,
        "camera-optics",
    ));

    graph.add_node(CodeMapNode::new(
        target.clone(),
        CodeMapNodeKind::Target,
        "SceneHighlight",
    ));

    graph.add_edge(CodeMapEdge::new(
        plugin.clone(),
        target.clone(),
        CodeMapEdgeKind::WritesTarget,
    ));

    assert_eq!(validate_codemap_graph(&graph), Ok(()));
}

#[test]
fn graph_with_missing_edge_target_fails_validation() {
    let plugin = CodeMapNodeId::Plugin(PluginId("amigo.camera.camera-optics".to_string()));
    let target = CodeMapNodeId::Target(TargetId("SceneHighlight".to_string()));

    let mut graph = CodeMapGraph::new();

    graph.add_node(CodeMapNode::new(
        plugin.clone(),
        CodeMapNodeKind::Plugin,
        "camera-optics",
    ));

    graph.add_edge(CodeMapEdge::new(
        plugin,
        target,
        CodeMapEdgeKind::WritesTarget,
    ));

    let errors = validate_codemap_graph(&graph).unwrap_err();

    assert!(format!("{errors:?}").contains("EdgeToMissingNode"));
}

#[test]
fn navigate_to_targets_returns_target_edges() {
    let plugin = CodeMapNodeId::Plugin(PluginId("amigo.gfx.sprite-2d".to_string()));
    let target = CodeMapNodeId::Target(TargetId("SceneColor".to_string()));

    let mut graph = CodeMapGraph::new();

    graph.add_node(CodeMapNode::new(
        plugin.clone(),
        CodeMapNodeKind::Plugin,
        "sprite-2d",
    ));

    graph.add_node(CodeMapNode::new(
        target.clone(),
        CodeMapNodeKind::Target,
        "SceneColor",
    ));

    graph.add_edge(CodeMapEdge::new(
        plugin.clone(),
        target,
        CodeMapEdgeKind::WritesTarget,
    ));

    let result = navigate_to_targets(&graph, &plugin);

    assert_eq!(result.edges.len(), 1);
    assert_eq!(result.edges[0].kind, CodeMapEdgeKind::WritesTarget);
}

#[test]
fn waterfall_trace_reports_missing_stages() {
    let trace = WaterfallTrace::default();

    let missing = trace.missing_stages();

    assert!(missing.contains(&"source"));
    assert!(missing.contains(&"contribution"));
    assert!(missing.contains(&"candidate"));
    assert!(missing.contains(&"target"));
    assert!(missing.contains(&"consumer"));
    assert!(missing.contains(&"diagnostic"));
    assert!(missing.contains(&"test"));
    assert!(!trace.is_complete());
}
