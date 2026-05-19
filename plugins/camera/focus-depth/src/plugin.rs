use amigo_plugin_api::{
    capabilities, slots, CapabilityRef, PluginKind, PluginManifest,
    RenderParticipation,
};

pub fn plugin_manifest() -> PluginManifest {
    let mut manifest = PluginManifest::new(
        "amigo.camera.focus-depth",
        "camera",
        PluginKind::TargetConsumer,
        false,
        RenderParticipation::TargetConsumer,
    );

    manifest.capabilities.provides.push(CapabilityRef::new(
        capabilities::camera_focus_depth_2d().0,
        1,
    ));
    manifest.capabilities.requires.push(CapabilityRef::new(
        capabilities::camera_frame_context_2d().0,
        1,
    ));
    manifest
        .slots
        .implements
        .push(slots::camera_focus_model_2d());

    manifest.docs.pipeline = Some("docs/pipeline.md".to_string());
    manifest.tests.waterfall = Some("tests/waterfall_tests.rs".to_string());

    manifest
}
