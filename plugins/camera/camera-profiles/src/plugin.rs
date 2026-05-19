use amigo_plugin_api::{PluginKind, PluginManifest, RenderParticipation};

pub fn plugin_manifest() -> PluginManifest {
    let mut manifest = PluginManifest::new(
        "amigo.camera.camera-profiles",
        "camera",
        PluginKind::Bundle,
        false,
        RenderParticipation::None,
    );

    manifest.docs.pipeline = Some("docs/pipeline.md".to_string());
    manifest.tests.waterfall = Some("tests/waterfall_tests.rs".to_string());

    manifest
}
