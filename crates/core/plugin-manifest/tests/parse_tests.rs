use amigo_plugin_api::validate_plugin_manifest;
use amigo_plugin_manifest::parse_plugin_manifest_str;

#[test]
fn parses_valid_manifest() {
    let manifest = parse_plugin_manifest_str(
        r#"
id = "amigo.camera.camera-optics"
family = "camera"
kind = "target-consumer"
renderable = false
render_participation = "target-consumer"

[capabilities]
provides = ["camera.optics.2d@1"]
requires = ["camera.frame_context.2d@1"]

[slots]
implements = ["camera.optics.consumer.2d"]
requires = []
replaces = []

[targets]
reads = ["SceneHighlight", "SceneEmissive"]
writes = ["CameraArtifactLayer"]
contributes = []

[contributions]
emits = []
consumes = [
  { domain = "camera.optics", type = "CameraOpticsContribution2d", policy = "ExplicitOnly" }
]

[diagnostics]
channels = ["camera.optical.candidates"]

[docs]
pipeline = "docs/pipeline.md"
contributions = "docs/contributions.md"
diagnostics = "docs/diagnostics.md"

[tests]
waterfall = "tests/waterfall_tests.rs"
diagnostics = "tests/diagnostics_tests.rs"
"#,
    )
    .unwrap();

    assert_eq!(manifest.id.0, "amigo.camera.camera-optics");
    assert_eq!(manifest.capabilities.provides.len(), 1);
    assert_eq!(validate_plugin_manifest(&manifest), Ok(()));
}
