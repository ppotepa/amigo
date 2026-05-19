use std::fs;

use amigo_plugin_loader::load_plugin_manifests_from_plugins_dir;

#[test]
fn loads_two_level_plugin_manifest() {
    let root =
        std::env::temp_dir().join(format!("amigo-plugin-loader-test-{}", std::process::id()));
    let plugin_dir = root.join("camera").join("camera-optics");
    fs::create_dir_all(&plugin_dir).unwrap();

    fs::write(
        plugin_dir.join("plugin.toml"),
        r#"
id = "amigo.camera.camera-optics"
family = "camera"
kind = "target-consumer"
renderable = false
render_participation = "target-consumer"

[capabilities]
provides = ["camera.optics.2d@1"]
requires = []

[slots]
implements = ["camera.optics.consumer.2d"]
requires = []
replaces = []

[targets]
reads = ["SceneHighlight"]
writes = ["CameraArtifactLayer"]
contributes = []

[contributions]
emits = []
consumes = []

[diagnostics]
channels = ["camera.optical.candidates"]

[docs]
pipeline = "docs/pipeline.md"

[tests]
waterfall = "tests/waterfall_tests.rs"
"#,
    )
    .unwrap();

    let manifests = load_plugin_manifests_from_plugins_dir(&root).unwrap();

    assert_eq!(manifests.len(), 1);
    assert_eq!(manifests[0].id.0, "amigo.camera.camera-optics");

    let _ = fs::remove_dir_all(root);
}
