use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

#[test]
fn runtime_bundles_do_not_publicly_reexport_plugin_crates() {
    let crate_root = crate_root();
    let lib_rs = read(crate_root.join("src/lib.rs"));
    assert!(
        !lib_rs.contains("pub mod plugin_crates"),
        "runtime-bundles should not expose a public plugin_crates facade",
    );
    assert!(
        !lib_rs.contains("pub use plugin_crates"),
        "runtime-bundles should not glob-reexport plugin_crates",
    );
    assert!(
        !crate_root.join("src/plugin_crates.rs").exists(),
        "plugin_crates.rs should not remain after public plugin crate reexports are removed",
    );

    for entry in fs::read_dir(crate_root.join("src")).expect("src dir should be readable") {
        let entry = entry.expect("src entry should be readable");
        let path = entry.path();
        if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
            continue;
        }

        let content = read(&path);
        for line in content.lines() {
            let trimmed = line.trim();
            assert!(
                !(trimmed.starts_with("pub use amigo_")
                    && !trimmed.contains("::")
                    && trimmed.ends_with(';')),
                "{} must not publicly reexport a whole plugin crate: {trimmed}",
                path.display(),
            );
        }
    }
}

#[test]
fn runtime_bundle_runtime_exports_do_not_reexport_domain_types() {
    let crate_root = crate_root();
    for relative_path in ["src/render_packet_services.rs", "src/two_d.rs"] {
        let path = crate_root.join(relative_path);
        let content = read(&path);
        assert!(
            !content.contains("pub use amigo_"),
            "{relative_path} must not publicly reexport plugin/domain crates",
        );
    }
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
