use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

#[test]
fn render_session_extracts_through_runtime_registry() {
    let render_session_rs = crate_root().join("src/render_session.rs");
    let content = read(render_session_rs);
    assert!(
        content.contains("default_wgpu_render_extractor_registry_for_runtime("),
        "render_session should build render extraction through runtime registry",
    );
    assert!(
        content.contains(".extract_all(session.runtime())"),
        "render_session should execute render extraction through registry.extract_all(...)",
    );
}

#[test]
fn runtime_bundles_do_not_reintroduce_manual_world_2d_plugin_calls_in_render_session() {
    let render_session_rs = crate_root().join("src/render_session.rs");
    let content = read(render_session_rs);
    assert!(
        !content.contains("register_world_2d_plugin_"),
        "render_session should not manually wire world_2d plugin extractor installers",
    );
}
