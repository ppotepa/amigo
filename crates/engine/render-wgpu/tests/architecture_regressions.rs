use std::fs;
use std::path::{Path, PathBuf};

fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path).expect("file should be readable")
}

fn cargo_toml() -> PathBuf {
    crate_root().join("Cargo.toml")
}

fn walk_rs_files(root: &Path, out: &mut Vec<PathBuf>) {
    for entry in fs::read_dir(root).expect("directory should be readable") {
        let entry = entry.expect("dir entry should be readable");
        let path = entry.path();
        if path.is_dir() {
            walk_rs_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}

fn render_wgpu_rs_files() -> Vec<PathBuf> {
    let mut files = Vec::new();
    walk_rs_files(&crate_root().join("src"), &mut files);
    files
}

#[test]
fn world_rs_does_not_branch_on_render_primitive_variants() {
    let world_rs =
        crate_root().join("src/renderer/service/render/world.rs");
    assert!(
        !read(world_rs).contains("RenderPrimitive2d::"),
        "world.rs should not branch on RenderPrimitive2d variants",
    );
}

#[test]
fn render_wgpu_does_not_reintroduce_renderable_payload() {
    for path in render_wgpu_rs_files() {
        let content = read(&path);
        assert!(
            !content.contains("Renderable2dPayload")
                && !content.contains("Renderable2dPayloadKind"),
            "render-wgpu should not reference Renderable2dPayload in {}",
            path.display()
        );
    }
}

#[test]
fn post_fx_registry_does_not_use_central_executor_match() {
    let registry_rs = crate_root().join("src/renderer/service/post_fx/registry.rs");
    assert!(
        !read(registry_rs).contains("match descriptor.executor_id"),
        "post-fx registry should dispatch through registry, not central match",
    );
}

#[test]
fn render_wgpu_live_path_does_not_import_2d_plugin_render_models() {
    let forbidden = [
        "amigo_composite_plugin",
        "amigo_layered_image_2d_plugin",
        "amigo_particles_2d_plugin",
        "amigo_sprite_2d_plugin",
        "amigo_camera_optics_plugin",
    ];

    for path in render_wgpu_rs_files() {
        let file_name = path.file_name().and_then(|name| name.to_str()).unwrap_or("");
        if file_name.ends_with("tests.rs") {
            continue;
        }
        let content = read(&path);
        for needle in forbidden {
            assert!(
                !content.contains(needle),
                "render-wgpu live path should not import {needle} in {}",
                path.display()
            );
        }
    }
}

#[test]
fn render_wgpu_source_tree_does_not_import_plugin_crates() {
    for path in render_wgpu_rs_files() {
        let content = read(&path);
        assert!(
            !content.contains("_plugin"),
            "render-wgpu source tree should not import plugin crates in {}",
            path.display()
        );
    }
}

#[test]
fn render_wgpu_cargo_toml_does_not_depend_on_plugin_crates() {
    let cargo = read(cargo_toml());
    assert!(
        !cargo.contains("-plugin"),
        "render-wgpu Cargo.toml should not depend on plugin crates",
    );
}
