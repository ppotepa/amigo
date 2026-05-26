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

fn strip_rust_comments(content: &str) -> String {
    let mut stripped = String::with_capacity(content.len());
    let mut chars = content.chars().peekable();
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if in_block_comment {
            if ch == '*' && chars.peek().is_some_and(|next| *next == '/') {
                chars.next();
                stripped.push(' ');
                stripped.push(' ');
                in_block_comment = false;
            } else if ch == '\n' {
                stripped.push('\n');
            } else {
                stripped.push(' ');
            }
            continue;
        }

        if ch == '/' && chars.peek().is_some_and(|next| *next == '/') {
            chars.next();
            stripped.push(' ');
            stripped.push(' ');
            for comment_ch in chars.by_ref() {
                if comment_ch == '\n' {
                    stripped.push('\n');
                    break;
                }
                stripped.push(' ');
            }
        } else if ch == '/' && chars.peek().is_some_and(|next| *next == '*') {
            chars.next();
            stripped.push(' ');
            stripped.push(' ');
            in_block_comment = true;
        } else {
            stripped.push(ch);
        }
    }

    stripped
}

fn line_number_for_offset(content: &str, offset: usize) -> usize {
    content[..offset]
        .bytes()
        .filter(|byte| *byte == b'\n')
        .count()
        + 1
}

fn first_needle_line(content: &str, needle: &str) -> Option<usize> {
    content
        .find(needle)
        .map(|offset| line_number_for_offset(content, offset))
}

fn skip_ascii_whitespace(bytes: &[u8], mut offset: usize) -> usize {
    while bytes
        .get(offset)
        .is_some_and(|byte| byte.is_ascii_whitespace())
    {
        offset += 1;
    }
    offset
}

fn find_ascii(haystack: &[u8], needle: &[u8], start: usize) -> Option<usize> {
    haystack
        .get(start..)?
        .windows(needle.len())
        .position(|window| window == needle)
        .map(|offset| start + offset)
}

fn owner_entity_equality_line(content: &str) -> Option<usize> {
    let bytes = content.as_bytes();
    let needle = b"owner_entity";
    let mut start = 0;

    while let Some(offset) = find_ascii(bytes, needle, start) {
        let mut cursor = offset + needle.len();
        cursor = skip_ascii_whitespace(bytes, cursor);
        if bytes.get(cursor) != Some(&b'(') {
            start = offset + 1;
            continue;
        }
        cursor = skip_ascii_whitespace(bytes, cursor + 1);
        if bytes.get(cursor) != Some(&b')') {
            start = offset + 1;
            continue;
        }
        cursor = skip_ascii_whitespace(bytes, cursor + 1);
        if bytes.get(cursor..cursor + 2) == Some(b"==") {
            return Some(line_number_for_offset(content, offset));
        }
        start = offset + 1;
    }

    None
}

#[test]
fn world_rs_does_not_branch_on_render_primitive_variants() {
    let world_rs = crate_root().join("src/renderer/service/render/world.rs");
    assert!(
        !read(world_rs).contains("RenderPrimitive2d::"),
        "world.rs should not branch on RenderPrimitive2d variants",
    );
}

#[test]
fn architecture_procedural_material_pass_has_no_renderer_side_guessing() {
    let path = crate_root()
        .join("src/renderer/service/render/visual_source_buffer_pass/procedural_material.rs");
    let content = strip_rust_comments(&read(&path));
    if let Some(line) = first_needle_line(&content, "RenderPrimitive2d::") {
        panic!(
            "procedural material visual-source path should not branch on RenderPrimitive2d variants in {}:{}",
            path.display(),
            line,
        );
    }
    if let Some(line) = owner_entity_equality_line(&content) {
        panic!(
            "procedural material visual-source path should not compare owner_entity() with domain state in {}:{}",
            path.display(),
            line,
        );
    }
}

#[test]
fn architecture_plate_relight_has_no_renderer_side_beacon_matching() {
    let path = crate_root().join("src/renderer/service/render/plate_relight.rs");
    let content = strip_rust_comments(&read(&path));
    if let Some(line) = owner_entity_equality_line(&content) {
        panic!(
            "plate relight should not compare owner_entity() with light source state in {}:{}",
            path.display(),
            line,
        );
    }
    if let Some(line) = first_needle_line(&content, "source.component_kind == \"BeaconLight2D\"") {
        panic!(
            "plate relight should not match beacon payloads through component-kind strings in {}:{}",
            path.display(),
            line,
        );
    }
}

#[test]
fn architecture_lightmap_uses_render_source_id_not_owner_entity_guessing() {
    let path = crate_root().join("src/renderer/lightmap2d.rs");
    let content = strip_rust_comments(&read(&path));
    if let Some(line) = owner_entity_equality_line(&content) {
        panic!(
            "lightmap2d should resolve authored sources through RenderSourceId, not owner_entity equality in {}:{}",
            path.display(),
            line,
        );
    }
    assert!(
        content.contains("source_id()") && content.contains("source.source.source_id"),
        "lightmap2d should bind sources through explicit RenderSourceId"
    );
}

#[test]
fn architecture_motion_buffer_uses_render_source_id_for_temporal_keys() {
    let path = crate_root().join("src/renderer/service/render/visual_source_buffer_pass/motion.rs");
    let content = strip_rust_comments(&read(&path));
    if let Some(line) = first_needle_line(&content, "item.owner_entity()") {
        panic!(
            "motion buffer temporal keys should use RenderSourceId, not owner_entity() in {}:{}",
            path.display(),
            line,
        );
    }
    if let Some(line) = first_needle_line(&content, "item.component_kind()") {
        panic!(
            "motion buffer temporal keys should use RenderSourceId, not component_kind() in {}:{}",
            path.display(),
            line,
        );
    }
    assert!(
        content.contains("item.source_id().as_str()"),
        "motion buffer should build temporal keys from explicit RenderSourceId"
    );
}

#[test]
fn architecture_special_render_passes_use_renderable_adapters_for_primitive_behavior() {
    for relative in [
        "src/renderer/service/post_fx/focus_blur.rs",
        "src/renderer/service/render/visual_source_buffer_pass/material_maps.rs",
        "src/renderer/service/render/refractive_material.rs",
    ] {
        let path = crate_root().join(relative);
        let content = strip_rust_comments(&read(&path));
        if let Some(line) = first_needle_line(&content, "RenderPrimitive2d::") {
            panic!(
                "special render pass should use renderable adapter contracts instead of branching on RenderPrimitive2d variants in {}:{}",
                path.display(),
                line,
            );
        }
    }
}

#[test]
fn architecture_world_selection_uses_render_object_id_not_owner_entity_filtering() {
    for relative in [
        "src/renderer/service/render/world.rs",
        "src/renderer/service/render/layered_image_parts_pass.rs",
    ] {
        let path = crate_root().join(relative);
        let content = strip_rust_comments(&read(&path));
        if let Some(line) = first_needle_line(&content, "item.owner_entity()") {
            panic!(
                "world selection and layered-image part targeting should use RenderObjectId, not owner_entity() in {}:{}",
                path.display(),
                line,
            );
        }
        assert!(
            content.contains("item.object_id()"),
            "{} should filter renderables through explicit object_id()",
            path.display(),
        );
    }
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
fn init_uses_centralized_post_fx_pipeline_bootstrap() {
    let init_rs = crate_root().join("src/renderer/service/init.rs");
    let content = read(init_rs);
    assert!(
        content.contains("build_default_post_fx_pipelines("),
        "init.rs should build post-fx pipelines through centralized bootstrap helper",
    );
    assert!(
        !content.contains("post_fx_pipeline_registry.register("),
        "init.rs should not manually register post-fx pipelines one by one",
    );
    for shader in [
        "CAMERA_EXPOSURE_SHADER",
        "CAMERA_OPTICS_SHADER",
        "FOCUS_BLUR_SHADER",
        "FILM_EMULSION_SHADER",
        "FILM_NOISE_SHADER",
        "SCAN_OUTPUT_SHADER",
    ] {
        assert!(
            !content.contains(shader),
            "init.rs should not embed pilot post-fx shader source {shader}",
        );
    }
}

#[test]
fn world_rs_does_not_embed_layered_image_parts_pass_logic() {
    let world_rs = crate_root().join("src/renderer/service/render/world.rs");
    let content = read(world_rs);
    assert!(
        !content.contains("execute_layered_image_parts_to_offscreen"),
        "world.rs should delegate layered image parts pass out of the world renderer",
    );
    assert!(
        !content.contains(".layered_textured_quads("),
        "world.rs should not inspect layered image part primitives directly",
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
        let file_name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("");
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
            !content.contains("_plugin") || content.contains("amigo_plugin_api"),
            "render-wgpu source tree should not import plugin crates in {}",
            path.display()
        );
    }
}

#[test]
fn render_wgpu_cargo_toml_does_not_depend_on_plugin_crates() {
    let cargo = read(cargo_toml());
    assert!(
        !cargo.contains("-plugin") || cargo.contains("amigo-plugin-api"),
        "render-wgpu Cargo.toml should not depend on plugin crates",
    );
}

#[test]
fn renderer_rs_does_not_embed_core_shader_sources() {
    let renderer_rs = crate_root().join("src/renderer.rs");
    let content = read(renderer_rs);
    assert!(
        !content.contains("const COLOR_SHADER"),
        "renderer.rs should not embed COLOR_SHADER",
    );
    assert!(
        !content.contains("const TEXTURE_SHADER"),
        "renderer.rs should not embed TEXTURE_SHADER",
    );
}

#[test]
fn model_rs_does_not_keep_dedicated_core_pipeline_fields() {
    let model_rs = crate_root().join("src/renderer/service/model.rs");
    let content = read(model_rs);
    for field in [
        "color_alpha_pipeline",
        "color_additive_pipeline",
        "color_multiply_pipeline",
        "color_screen_pipeline",
        "texture_alpha_pipeline",
        "texture_opaque_pipeline",
        "texture_additive_pipeline",
        "texture_multiply_pipeline",
        "texture_screen_pipeline",
        "texture_lighten_pipeline",
    ] {
        assert!(
            !content.contains(field),
            "service/model.rs should not keep dedicated core pipeline field {field}",
        );
    }
}

#[test]
fn init_rs_does_not_manually_create_core_pipelines() {
    let init_rs = crate_root().join("src/renderer/service/init.rs");
    let content = read(init_rs);
    assert!(
        content.contains("build_default_core_pipelines("),
        "init.rs should build core pipelines through centralized bootstrap helper",
    );
    assert!(
        !content.contains("create_color_pipeline"),
        "init.rs should not manually create core pipelines",
    );
    assert!(
        !content.contains("COLOR_SHADER"),
        "init.rs should not embed COLOR_SHADER usage",
    );
    assert!(
        !content.contains("TEXTURE_SHADER"),
        "init.rs should not embed TEXTURE_SHADER usage",
    );
}
