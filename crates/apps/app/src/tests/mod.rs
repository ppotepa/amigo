use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::{TileMap2dSceneService, TileVariantKind2d};
use amigo_app_host_api::{HostHandler, HostLifecycleEvent};
use amigo_assets::{AssetCatalog, AssetKey, AssetManifest, AssetSourceKind};
use amigo_audio_api::{AudioCommand, AudioCommandQueue, AudioSceneService, AudioStateService};
use amigo_audio_mixer::AudioMixerService;
use amigo_core::{AmigoError, RuntimeDiagnostics};
use amigo_input_api::{InputEvent, KeyCode};
use amigo_render_wgpu::{UiOverlayNodeKind, UiViewportSize, build_ui_layout_tree};
use amigo_scene::{
    EntityPoolSceneService, HydratedSceneState, SceneCommand, SceneCommandQueue, SceneKey,
    SceneService,
};
use amigo_scripting_api::{
    DevConsoleCommand, DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptEvent,
    ScriptEventQueue,
};
use amigo_ui::{UiInputService, UiSceneService, UiStateService, UiThemeService};

use super::{
    BootstrapOptions, InteractiveRuntimeHostHandler, OverlayUiLayoutNode, bootstrap_with_options,
    next_scene_id, refresh_runtime_summary, scene_ids_for_launch_selection,
};
use crate::orchestration::{process_audio_command, process_placeholder_bridges};
use crate::script_runtime;
use amigo_core::LaunchSelection;
use amigo_modding::ModCatalog;

mod bootstrap_tests;
mod hot_reload_tests;
mod interactive_runtime_tests;
mod launch_selection_tests;
mod particles_tests;
mod render_runtime_tests;
mod runtime_summary_tests;
mod scene_loading_tests;
mod ui_runtime_tests;

fn mods_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|path| path.parent())
        .and_then(|path| path.parent())
        .expect("workspace root should exist")
        .join("mods")
}

fn copied_mods_root(label: &str, mod_ids: &[&str]) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!("amigo-app-{label}-{unique}"));
    fs::create_dir_all(&root).expect("temp mods root should exist");

    for mod_id in mod_ids {
        copy_dir_recursive(&mods_root().join(mod_id), &root.join(mod_id));
    }

    root
}

fn write_lifecycle_probe(temp_mods: &Path, source: &str) {
    fs::write(
        temp_mods
            .join("playground-2d")
            .join("scripts")
            .join("components")
            .join("lifecycle_probe.rhai"),
        source,
    )
    .expect("lifecycle probe script should be writable");
}

fn assert_script_component_diagnostic(error: &AmigoError, phase: &str, cause: &str) {
    let message = error.to_string();
    assert!(message.contains(&format!("phase `{phase}`")), "{message}");
    assert!(
        message.contains("entity `playground-2d-demo-square`"),
        "{message}"
    );
    assert!(
        message.contains("script path `") && message.contains("lifecycle_probe.rhai"),
        "{message}"
    );
    assert!(
        message.contains("source name `component:playground-2d:playground-2d-demo-square:"),
        "{message}"
    );
    assert!(message.contains(cause), "{message}");
}

fn first_resolved_tile_id_for_variant(
    runtime: &amigo_runtime::Runtime,
    variant: TileVariantKind2d,
) -> Option<u32> {
    runtime
        .resolve::<TileMap2dSceneService>()?
        .commands()
        .into_iter()
        .find(|command| command.entity_name == "playground-sidescroller-tilemap")
        .and_then(|command| command.tilemap.resolved)
        .and_then(|resolved| {
            for row in resolved.rows {
                for tile in row {
                    if tile.variant == Some(variant) {
                        return tile.tile_id;
                    }
                }
            }

            None
        })
}

fn copy_dir_recursive(source: &Path, target: &Path) {
    fs::create_dir_all(target).expect("target directory should be created");

    for entry in fs::read_dir(source).expect("source directory should be readable") {
        let entry = entry.expect("directory entry should be readable");
        let entry_path = entry.path();
        let target_path = target.join(entry.file_name());
        let file_type = entry.file_type().expect("file type should be readable");

        if file_type.is_dir() {
            copy_dir_recursive(&entry_path, &target_path);
        } else {
            fs::copy(&entry_path, &target_path).expect("file should be copied");
        }
    }
}
