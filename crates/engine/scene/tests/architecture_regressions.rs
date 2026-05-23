use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

#[test]
fn plugin_scene_command_payload_does_not_revert_to_central_enum() {
    let plugin_command_rs = crate_root().join("src/plugin_command.rs");
    let content = read(plugin_command_rs);
    assert!(
        !content.contains("enum PluginSceneCommandPayload"),
        "plugin scene command payload must remain trait-based, not a central enum",
    );
}

#[test]
fn scene_loader_keeps_plugin_schema_registry_path() {
    let loader_rs = crate_root().join("src/document/loader.rs");
    let content = read(loader_rs);
    assert!(
        content.contains(".parse_plugin_payload("),
        "scene loader should continue routing plugin payloads through ComponentSchemaRegistry",
    );
    assert!(
        content.contains("SceneComponentDocument::Plugin"),
        "scene loader should preserve plugin envelope path for plugin-owned components",
    );
}
