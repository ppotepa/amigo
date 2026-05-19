use amigo_codemap_api::{
    validate_codemap_graph, CodeMapEdgeKind, CodeMapNodeId,
};
use amigo_plugin_api::{
    CapabilityRef, DiagnosticChannelId, DiagnosticChannelRef, PluginKind,
    PluginManifest, RenderParticipation, SlotId, TargetId,
};
use amigo_plugin_index::{
    build_codemap_graph_from_index, validate_plugin_index, PluginIndex,
};

fn camera_optics_manifest() -> PluginManifest {
    let mut manifest = PluginManifest::new(
        "amigo.camera.camera-optics",
        "camera",
        PluginKind::TargetConsumer,
        false,
        RenderParticipation::TargetConsumer,
    );

    manifest
        .capabilities
        .provides
        .push(CapabilityRef::new("camera.optics.2d", 1));

    manifest
        .capabilities
        .requires
        .push(CapabilityRef::new("camera.frame_context.2d", 1));

    manifest
        .slots
        .implements
        .push(SlotId("camera.optics.consumer.2d".to_string()));

    manifest
        .targets
        .reads
        .push(TargetId("SceneHighlight".to_string()));

    manifest
        .targets
        .reads
        .push(TargetId("SceneEmissive".to_string()));

    manifest
        .targets
        .writes
        .push(TargetId("CameraArtifactLayer".to_string()));

    manifest.diagnostics.channels.push(DiagnosticChannelRef {
        id: DiagnosticChannelId("camera.optical.candidates".to_string()),
        owner: manifest.id.clone(),
    });

    manifest.docs.pipeline = Some("docs/pipeline.md".to_string());
    manifest.docs.contributions = Some("docs/contributions.md".to_string());
    manifest.docs.diagnostics = Some("docs/diagnostics.md".to_string());

    manifest.tests.waterfall = Some("tests/waterfall_tests.rs".to_string());
    manifest.tests.diagnostics = Some("tests/diagnostics_tests.rs".to_string());

    manifest
}

#[test]
fn plugin_index_accepts_valid_manifest() {
    let index = PluginIndex::from_manifests([camera_optics_manifest()]);

    assert_eq!(index.len(), 1);
    assert_eq!(validate_plugin_index(&index), Ok(()));
}

#[test]
fn plugin_index_builds_valid_codemap_graph() {
    let index = PluginIndex::from_manifests([camera_optics_manifest()]);
    let graph = build_codemap_graph_from_index(&index);

    assert_eq!(validate_codemap_graph(&graph), Ok(()));
}

#[test]
fn plugin_graph_contains_plugin_and_targets() {
    let index = PluginIndex::from_manifests([camera_optics_manifest()]);
    let graph = build_codemap_graph_from_index(&index);

    let plugin = CodeMapNodeId::Plugin(amigo_plugin_api::PluginId(
        "amigo.camera.camera-optics".to_string(),
    ));
    let scene_highlight =
        CodeMapNodeId::Target(TargetId("SceneHighlight".to_string()));

    assert!(graph.contains_node(&plugin));
    assert!(graph.contains_node(&scene_highlight));

    let target_edges = graph.outgoing(&plugin);
    assert!(target_edges
        .iter()
        .any(|edge| edge.kind == CodeMapEdgeKind::ReadsTarget));
    assert!(target_edges
        .iter()
        .any(|edge| edge.kind == CodeMapEdgeKind::WritesTarget));
}

#[test]
fn invalid_manifest_is_reported_by_index_validation() {
    let mut manifest = camera_optics_manifest();
    manifest.docs.pipeline = None;

    let index = PluginIndex::from_manifests([manifest]);
    let errors = validate_plugin_index(&index).unwrap_err();

    assert!(format!("{errors:?}").contains("MissingPipelineDocs"));
}
