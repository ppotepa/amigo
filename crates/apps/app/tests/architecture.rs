use std::path::Path;

fn app_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
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

    for relative in [
        "src/scene_runtime/handlers",
        "src/script_runtime/handlers",
    ] {
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
        "src/dev_console/commands",
        "src/dev_console/dispatcher.rs",
        "src/dev_console/model.rs",
        "src/dev_console/parser.rs",
    ] {
        assert_app_path_absent(
            relative,
            &format!("apps/app must not reintroduce engine-owned dev console module {relative}"),
        );
    }

    let dev_console_mod = read_app_file("src/dev_console/mod.rs");
    for forbidden in [
        "mod dispatcher;",
        "mod model;",
        "mod parser;",
        "pub(crate) mod dispatcher;",
        "pub(crate) mod model;",
        "pub(crate) mod parser;",
    ] {
        assert!(
            !dev_console_mod.contains(forbidden),
            "src/dev_console/mod.rs must not reintroduce engine-owned dev console module via `{forbidden}`"
        );
    }
}


