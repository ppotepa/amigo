use amigo_plugin_api::{
    validate_plugin_manifest, CapabilityRef, DiagnosticChannelId, DiagnosticChannelRef, PluginKind,
    PluginManifest, RenderParticipation, SlotId, TargetId,
};

fn valid_manifest() -> PluginManifest {
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
        .slots
        .implements
        .push(SlotId("camera.optics.consumer.2d".to_string()));

    manifest
        .targets
        .reads
        .push(TargetId("SceneHighlight".to_string()));

    manifest
        .targets
        .writes
        .push(TargetId("CameraArtifactLayer".to_string()));

    manifest.diagnostics.channels.push(DiagnosticChannelRef {
        id: DiagnosticChannelId("camera.optical.candidates".to_string()),
        owner: manifest.id.clone(),
    });

    manifest.docs.pipeline = Some("docs/pipeline.md".to_string());
    manifest.tests.waterfall = Some("tests/waterfall_tests.rs".to_string());

    manifest
}

#[test]
fn valid_manifest_passes_validation() {
    let manifest = valid_manifest();

    assert_eq!(validate_plugin_manifest(&manifest), Ok(()));
}

#[test]
fn empty_plugin_id_fails_validation() {
    let mut manifest = valid_manifest();
    manifest.id.0.clear();

    let errors = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(format!("{errors:?}").contains("EmptyPluginId"));
}

#[test]
fn duplicate_capability_fails_validation() {
    let mut manifest = valid_manifest();

    manifest
        .capabilities
        .provides
        .push(CapabilityRef::new("camera.optics.2d", 1));

    let errors = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(format!("{errors:?}").contains("DuplicateProvidedCapability"));
}

#[test]
fn missing_pipeline_docs_fails_validation() {
    let mut manifest = valid_manifest();
    manifest.docs.pipeline = None;

    let errors = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(format!("{errors:?}").contains("MissingPipelineDocs"));
}

#[test]
fn missing_waterfall_test_fails_validation() {
    let mut manifest = valid_manifest();
    manifest.tests.waterfall = None;

    let errors = validate_plugin_manifest(&manifest).unwrap_err();

    assert!(format!("{errors:?}").contains("MissingWaterfallTest"));
}
