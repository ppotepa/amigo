use std::path::Path;

fn app_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn workspace_root() -> &'static Path {
    app_root()
        .parent()
        .and_then(Path::parent)
        .and_then(Path::parent)
        .expect("apps/app should live under crates/apps/app")
}

fn read_app_file(relative: &str) -> String {
    let path = app_root().join(relative);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()))
}

fn assert_app_path_absent(relative: &str, message: &str) {
    let path = app_root().join(relative);
    assert!(!path.exists(), "{message}: {}", path.display());
}

#[test]
fn app_does_not_recreate_domain_runtime_adapter_directories() {
    let root = app_root();

    for relative in ["src/scene_runtime/handlers", "src/script_runtime/handlers"] {
        assert!(
            !root.join(relative).exists(),
            "apps/app must not own domain runtime adapters at {relative}"
        );
    }
}

#[test]
fn app_runtime_modules_do_not_reintroduce_handlers_submodules() {
    for relative in ["src/script_runtime/mod.rs", "src/scene_runtime/mod.rs"] {
        let contents = read_app_file(relative);
        for forbidden in [
            "mod handlers;",
            "pub mod handlers;",
            "use handlers::",
            "handlers::dispatch",
            "handlers::register",
        ] {
            assert!(
                !contents.contains(forbidden),
                "{relative} must not reintroduce nested handlers module via `{forbidden}`"
            );
        }
    }
}

#[test]
fn app_render_runtime_does_not_reintroduce_resolved_domain_extractors() {
    for relative in [
        "src/render_runtime/context.rs",
        "src/render_runtime/extractors.rs",
        "src/render_runtime/extractors_world_2d.rs",
        "src/render_runtime/extractors_world_3d.rs",
        "src/render_runtime/extractors_host_overlay.rs",
    ] {
        assert_app_path_absent(
            relative,
            &format!("apps/app must not own render extraction bridge file {relative}"),
        );
    }

    let render_runtime = read_app_file("src/render_runtime.rs");

    for forbidden in [
        "AppRenderFramePacket",
        "AppRenderExtractContext",
        "ResolvedSprite2dExtractor",
        "ResolvedMesh3dExtractor",
        "default_app_render_extractor_registry",
    ] {
        assert!(
            !render_runtime.contains(forbidden),
            "src/render_runtime.rs must not reintroduce app-owned render extraction bridge via `{forbidden}`"
        );
    }
}

#[test]
fn app_cargo_does_not_depend_directly_on_domain_crates() {
    let cargo = read_app_file("Cargo.toml");

    for forbidden in [
        "amigo-2d-",
        "amigo-3d-",
        "amigo-audio-api",
        "amigo-audio-output",
        "amigo-audio-mixer",
        "amigo-ui",
        "amigo-behavior",
        "amigo-event-pipeline",
        "amigo-input-actions",
    ] {
        assert!(
            !cargo.contains(forbidden),
            "apps/app Cargo.toml must not depend directly on domain crate `{forbidden}`"
        );
    }
}

#[test]
fn app_systems_do_not_reintroduce_domain_system_plugins() {
    let systems_mod = app_root().join("src/systems/mod.rs");
    if !systems_mod.exists() {
        return;
    }

    let contents = read_app_file("src/systems/mod.rs");

    for forbidden in [
        "World2dRuntimeSystemsPlugin",
        "ScriptUpdateRuntimeSystemPlugin",
        "AudioRuntimeSystemPlugin",
        "UiInputRuntimeSystemPlugin",
    ] {
        assert!(
            !contents.contains(forbidden),
            "apps/app systems must not define legacy domain plugin {forbidden}"
        );
    }
}

#[test]
fn moved_services_remain_owned_by_engine_or_devtools() {
    for relative in [
        "src/render_runtime/diagnostics.rs",
        "src/render_runtime/stats.rs",
        "src/debug_overlay/service.rs",
    ] {
        assert_app_path_absent(
            relative,
            &format!("apps/app must not reintroduce app-owned moved service file {relative}"),
        );
    }

    let render_runtime = read_app_file("src/render_runtime.rs");
    for forbidden in [
        "mod diagnostics;",
        "mod stats;",
        "struct RenderCompositionDiagnosticsService",
        "impl RenderCompositionDiagnosticsService",
        "struct RenderFrameStatsService",
        "impl RenderFrameStatsService",
    ] {
        assert!(
            !render_runtime.contains(forbidden),
            "src/render_runtime.rs must not reintroduce app-owned moved render service via `{forbidden}`"
        );
    }
    for required in [
        "pub(crate) use amigo_render_api::RenderCompositionDiagnosticsService;",
        "pub(crate) use amigo_render_api::RenderFrameStatsService;",
    ] {
        assert!(
            render_runtime.contains(required),
            "src/render_runtime.rs must re-export moved render service via `{required}`"
        );
    }

    let debug_overlay = read_app_file("src/debug_overlay/mod.rs");
    assert!(
        debug_overlay.contains("pub(crate) use amigo_devtools::DebugOverlayService;"),
        "src/debug_overlay/mod.rs must re-export DebugOverlayService from amigo_devtools"
    );
    for forbidden in [
        "mod service;",
        "struct DebugOverlayService",
        "impl DebugOverlayService",
    ] {
        assert!(
            !debug_overlay.contains(forbidden),
            "src/debug_overlay/mod.rs must not reintroduce app-owned debug overlay service via `{forbidden}`"
        );
    }
}

#[test]
fn app_dev_console_does_not_reintroduce_engine_owned_parser_or_model() {
    for relative in [
        "src/dev_console",
        "src/dev_console/commands",
        "src/dev_console/dispatcher.rs",
        "src/dev_console/model.rs",
        "src/dev_console/parser.rs",
        "src/dev_console/completion.rs",
        "src/dev_console/registry.rs",
    ] {
        assert_app_path_absent(
            relative,
            &format!("apps/app must not reintroduce engine-owned dev console module {relative}"),
        );
    }
}

#[test]
fn app_does_not_reintroduce_runtime_capability_provider_seams() {
    for relative in ["src/scene_runtime/mod.rs", "src/diagnostics.rs"] {
        let content = read_app_file(relative);
        for forbidden in [
            "register_app_scene_command_provider",
            "register_host_diagnostics_provider",
            "AppSceneCommandProvider",
            "HostAppDiagnosticsProvider",
            "HostAppMetadataProvider",
            "impl SceneCommandProvider",
            "impl DiagnosticsProvider",
            "impl MetadataProvider",
        ] {
            assert!(
                !content.contains(forbidden),
                "{relative} must not reintroduce app-owned capability provider seam `{forbidden}`"
            );
        }
    }
}

#[test]
fn runtime_bundles_render_bridges_do_not_use_app_owned_names() {
    let workspace = workspace_root();

    for relative in [
        "crates/runtime/bundles/src/wgpu_render_extractors/world_2d.rs",
        "crates/runtime/bundles/src/wgpu_render_extractors/world_3d.rs",
        "crates/runtime/bundles/src/wgpu_render_extractors/host_overlay.rs",
    ] {
        let content = std::fs::read_to_string(workspace.join(relative))
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));

        assert!(
            !content.contains("pub struct App"),
            "{relative} should not define App* render extractors"
        );
        assert!(
            !content.contains("HostAppRenderExtractorProvider"),
            "{relative} should not define host app render extractor providers"
        );
    }
}

#[test]
fn editor_api_remains_ui_framework_free() {
    let workspace = workspace_root();
    let cargo = std::fs::read_to_string(workspace.join("crates/engine/editor-api/Cargo.toml"))
        .expect("editor-api Cargo.toml should be readable");

    for forbidden in ["egui", "imgui", "winit", "wgpu", "amigo-app"] {
        assert!(
            !cargo.contains(forbidden),
            "editor-api must not depend on {forbidden}"
        );
    }
}

#[test]
fn editor_session_remains_ui_framework_free() {
    let workspace = workspace_root();
    let cargo = std::fs::read_to_string(workspace.join("crates/engine/editor-session/Cargo.toml"))
        .expect("editor-session Cargo.toml should be readable");

    for forbidden in ["egui", "imgui", "winit", "wgpu", "amigo-app"] {
        assert!(
            !cargo.contains(forbidden),
            "editor-session must not depend on {forbidden}"
        );
    }
}

#[test]
fn render_space_is_integrated_with_composition_plan() {
    let workspace = workspace_root();
    let composition =
        std::fs::read_to_string(workspace.join("crates/engine/render-api/src/composition.rs"))
            .expect("composition.rs should be readable");

    assert!(
        composition.contains("CompositionLayer"),
        "FrameCompositionPlan should expose composition layers"
    );
    assert!(
        composition.contains("RenderSpace"),
        "composition.rs should use RenderSpace"
    );
}

#[test]
fn editor_capabilities_use_placeholder_schema_helpers() {
    let workspace = workspace_root();

    for relative in [
        "crates/2d/sprite/src/editor_capability.rs",
        "crates/2d/text/src/editor_capability.rs",
        "crates/2d/vector/src/editor_capability.rs",
        "crates/2d/tilemap/src/editor_capability.rs",
        "crates/2d/layered-image/src/editor_capability.rs",
        "crates/3d/mesh/src/editor_capability.rs",
        "crates/3d/material/src/editor_capability.rs",
        "crates/3d/text/src/editor_capability.rs",
        "crates/audio/api/src/editor_capability.rs",
        "crates/ui/core/src/editor_capability.rs",
        "crates/engine/camera/src/editor_capability.rs",
        "crates/engine/devtools/src/editor_capability.rs",
    ] {
        let content = std::fs::read_to_string(workspace.join(relative))
            .unwrap_or_else(|error| panic!("failed to read {relative}: {error}"));

        assert!(
            content.contains("InspectorSchema::placeholder"),
            "{relative} should use placeholder inspector schemas"
        );
        assert!(
            !content.contains("fields: vec!["),
            "{relative} should not manually build inspector field vectors"
        );
        assert!(
            !content.contains("PropertyDescriptor {"),
            "{relative} should use PropertyDescriptor constructor helpers"
        );
        assert!(
            !content.contains("editor-capability"),
            "{relative} should use final editor capability ids"
        );
    }
}
