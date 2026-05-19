use amigo_plugin_api::{
    capabilities, slots, CapabilityRef, PluginKind, PluginManifest,
    RenderParticipation,
};

pub fn plugin_manifest() -> PluginManifest {
    let mut manifest = PluginManifest::new(
        "amigo.camera.camera-core",
        "camera",
        PluginKind::SemanticSource,
        false,
        RenderParticipation::None,
    );

    manifest.capabilities.provides.push(CapabilityRef::new(
        capabilities::camera_frame_context_2d().0,
        1,
    ));

    manifest
        .slots
        .implements
        .push(slots::camera_frame_provider_2d());

    manifest.docs.pipeline = Some("docs/pipeline.md".to_string());
    manifest.tests.waterfall = Some("tests/waterfall_tests.rs".to_string());

    manifest
}
