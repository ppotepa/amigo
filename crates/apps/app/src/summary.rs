use amigo_assets::AssetCatalog;
use amigo_capabilities::CapabilityRegistry;
use amigo_core::{AmigoResult, LaunchSelection};
use amigo_file_watch_api::FileWatchBackendInfo;
use amigo_hot_reload::HotReloadService;
use amigo_input_api::InputServiceInfo;
use amigo_modding::ModCatalog;
use amigo_render_api::RenderBackendInfo;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;
use amigo_scripting_api::{DevConsoleState, ScriptRuntimeInfo};
use amigo_window_api::WindowServiceInfo;

use crate::orchestration::stabilize_runtime;
use crate::runtime_context::required;
use crate::scene_runtime::current_loaded_scene_document_summary;
use crate::scripting_runtime::current_executed_scripts;
use crate::{BootstrapSummary, LoadedSceneDocumentSummary, PlaceholderBridgeSummary};

fn summary_plugin_label(plugin_name: &str) -> String {
    amigo_runtime_bundles::runtime_bundle_plugin_report_label(plugin_name)
}

pub(crate) fn summarize(
    runtime: &Runtime,
    launch_selection: LaunchSelection,
    placeholder_bridge: PlaceholderBridgeSummary,
    loaded_scene_document: Option<LoadedSceneDocumentSummary>,
) -> AmigoResult<BootstrapSummary> {
    summarize_runtime_state_with_loaded_document(
        runtime,
        launch_selection,
        placeholder_bridge,
        loaded_scene_document,
    )
}

fn summarize_runtime_state(
    runtime: &Runtime,
    launch_selection: LaunchSelection,
    placeholder_bridge: PlaceholderBridgeSummary,
) -> AmigoResult<BootstrapSummary> {
    summarize_runtime_state_with_loaded_document(
        runtime,
        launch_selection,
        placeholder_bridge,
        current_loaded_scene_document_summary(runtime)?,
    )
}

pub(crate) fn refresh_runtime_summary(runtime: &Runtime) -> AmigoResult<BootstrapSummary> {
    let launch_selection = required::<LaunchSelection>(runtime)?.as_ref().clone();
    let placeholder_bridge = stabilize_runtime(runtime)?;

    summarize_runtime_state(runtime, launch_selection, placeholder_bridge)
}

fn summarize_runtime_state_with_loaded_document(
    runtime: &Runtime,
    launch_selection: LaunchSelection,
    placeholder_bridge: PlaceholderBridgeSummary,
    loaded_scene_document: Option<LoadedSceneDocumentSummary>,
) -> AmigoResult<BootstrapSummary> {
    let window = required::<WindowServiceInfo>(runtime)?;
    let input = required::<InputServiceInfo>(runtime)?;
    let render = required::<RenderBackendInfo>(runtime)?;
    let script = required::<ScriptRuntimeInfo>(runtime)?;
    let scene = required::<SceneService>(runtime)?;
    let assets = required::<AssetCatalog>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let hot_reload = required::<HotReloadService>(runtime)?;
    let bundle_summary = amigo_runtime_bundles::runtime_bundle_summary(runtime)?;
    let file_watch_backend = runtime
        .resolve::<FileWatchBackendInfo>()
        .map(|info| {
            if info.automatic_notifications {
                info.backend_name.to_owned()
            } else {
                format!("{} (polling fallback)", info.backend_name)
            }
        })
        .unwrap_or_else(|| "polling".to_owned());

    let mut capabilities = collect_capabilities_from_registry(runtime);
    capabilities.sort();

    let mod_catalog = runtime.resolve::<ModCatalog>();
    let loaded_mods = mod_catalog
        .as_ref()
        .map(|catalog| {
            catalog
                .mod_ids()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();
    let frame_cap_fps = launch_selection
        .startup_mod
        .as_deref()
        .and_then(|mod_id| {
            mod_catalog
                .as_ref()
                .and_then(|catalog| catalog.mod_by_id(mod_id))
        })
        .and_then(|discovered| discovered.manifest.runtime.frame_cap_fps)
        .filter(|fps| fps.is_finite() && *fps > 0.0);

    let report = runtime.report();
    Ok(BootstrapSummary {
        window_backend: window.backend_name.to_owned(),
        input_backend: input.backend_name.to_owned(),
        render_backend: render.backend_name.to_owned(),
        script_backend: script.backend_name.to_owned(),
        file_watch_backend,
        loaded_mods,
        executed_scripts: current_executed_scripts(runtime)?,
        startup_mod: launch_selection.startup_mod,
        startup_scene: launch_selection.startup_scene,
        frame_cap_fps,
        active_scene: scene
            .selected_scene()
            .map(|scene| scene.as_str().to_owned()),
        loaded_scene_document,
        scene_entities: scene.entity_names(),
        registered_assets: assets
            .registered_keys()
            .into_iter()
            .map(|key| key.as_str().to_owned())
            .collect(),
        loaded_assets: assets
            .loaded_assets()
            .into_iter()
            .map(|asset| asset.key.as_str().to_owned())
            .collect(),
        prepared_assets: assets
            .prepared_assets()
            .into_iter()
            .map(|asset| format!("{} ({})", asset.key.as_str(), asset.kind.as_str()))
            .collect(),
        failed_assets: assets
            .failed_assets()
            .into_iter()
            .map(|asset| format!("{}: {}", asset.key.as_str(), asset.reason))
            .collect(),
        pending_asset_loads: assets
            .pending_loads()
            .into_iter()
            .map(|request| request.key.as_str().to_owned())
            .collect(),
        watched_reload_targets: hot_reload
            .watched_targets()
            .into_iter()
            .map(|watch| format!("{} -> {}", watch.id, watch.path.display()))
            .collect(),
        sprite_entities_2d: bundle_summary.sprite_entities_2d,
        text_entities_2d: bundle_summary.text_entities_2d,
        vector_entities_2d: bundle_summary.vector_entities_2d,
        mesh_entities_3d: bundle_summary.mesh_entities_3d,
        material_entities_3d: bundle_summary.material_entities_3d,
        text_entities_3d: bundle_summary.text_entities_3d,
        ui_entities: bundle_summary.ui_entities,
        audio_clips: bundle_summary.audio_clips,
        audio_sources: bundle_summary.audio_sources,
        pending_audio_runtime_commands: bundle_summary.pending_audio_runtime_commands,
        audio_master_volume: bundle_summary.audio_master_volume,
        mixed_audio_frame_count: bundle_summary.mixed_audio_frame_count,
        active_realtime_audio_sources: bundle_summary.active_realtime_audio_sources,
        audio_output_started: bundle_summary.audio_output_started,
        audio_output_device: bundle_summary.audio_output_device,
        audio_output_buffered_samples: bundle_summary.audio_output_buffered_samples,
        audio_output_last_error: bundle_summary.audio_output_last_error,
        processed_script_commands: placeholder_bridge.processed_script_commands,
        processed_audio_commands: placeholder_bridge.processed_audio_commands,
        processed_scene_commands: placeholder_bridge.processed_scene_commands,
        processed_script_events: placeholder_bridge.processed_script_events,
        console_commands: dev_console_state.command_history(),
        console_output: dev_console_state.output_lines(),
        capabilities,
        plugins: report
            .plugin_names
            .into_iter()
            .map(summary_plugin_label)
            .collect(),
        services: report
            .service_names
            .into_iter()
            .map(str::to_owned)
            .collect(),
    })
}

fn collect_capabilities_from_registry(runtime: &Runtime) -> Vec<String> {
    runtime
        .resolve::<CapabilityRegistry>()
        .map(|catalog| catalog.capability_names())
        .unwrap_or_default()
}
