use amigo_2d_physics::Physics2dDomainInfo;
use amigo_2d_platformer::{Motion2dDomainInfo, PlatformerDomainInfo};
use amigo_2d_sprite::{SpriteDomainInfo, SpriteSceneService};
use amigo_2d_text::{Text2dDomainInfo, Text2dSceneService};
use amigo_2d_tilemap::TileMap2dDomainInfo;
use amigo_3d_material::{MaterialDomainInfo, MaterialSceneService};
use amigo_3d_mesh::{MeshDomainInfo, MeshSceneService};
use amigo_3d_text::{Text3dDomainInfo, Text3dSceneService};
use amigo_assets::AssetCatalog;
use amigo_audio_api::{AudioDomainInfo, AudioSceneService, AudioStateService};
use amigo_audio_generated::GeneratedAudioDomainInfo;
use amigo_audio_mixer::{AudioMixerDomainInfo, AudioMixerService};
use amigo_audio_output::{AudioOutputBackendService, AudioOutputDomainInfo};
use amigo_core::{AmigoResult, LaunchSelection};
use amigo_file_watch_api::FileWatchBackendInfo;
use amigo_hot_reload::HotReloadService;
use amigo_input_api::InputServiceInfo;
use amigo_modding::ModCatalog;
use amigo_render_api::RenderBackendInfo;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;
use amigo_scripting_api::{DevConsoleState, ScriptRuntimeInfo};
use amigo_ui::{UiDomainInfo, UiSceneService};
use amigo_window_api::WindowServiceInfo;

use crate::orchestration::stabilize_runtime;
use crate::runtime_context::required;
use crate::scene_runtime::current_loaded_scene_document_summary;
use crate::scripting_runtime::current_executed_scripts;
use crate::{BootstrapSummary, LoadedSceneDocumentSummary, PlaceholderBridgeSummary};

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
    let audio_scene = required::<AudioSceneService>(runtime)?;
    let audio_state = required::<AudioStateService>(runtime)?;
    let audio_mixer = required::<AudioMixerService>(runtime)?;
    let audio_output = required::<AudioOutputBackendService>(runtime)?;
    let sprite_scene = required::<SpriteSceneService>(runtime)?;
    let text_scene = required::<Text2dSceneService>(runtime)?;
    let mesh_scene = required::<MeshSceneService>(runtime)?;
    let text3d_scene = required::<Text3dSceneService>(runtime)?;
    let material_scene = required::<MaterialSceneService>(runtime)?;
    let ui_scene = required::<UiSceneService>(runtime)?;
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

    let mut capabilities = Vec::new();
    capabilities.push(required::<SpriteDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<Text2dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<UiDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<Physics2dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<TileMap2dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<PlatformerDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<Motion2dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<AudioDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<GeneratedAudioDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<AudioMixerDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<AudioOutputDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<MeshDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<Text3dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<MaterialDomainInfo>(runtime)?.capability.to_owned());
    capabilities.sort();

    let loaded_mods = runtime
        .resolve::<ModCatalog>()
        .map(|catalog| {
            catalog
                .mod_ids()
                .into_iter()
                .map(str::to_owned)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let report = runtime.report();
    let audio_output_snapshot = audio_output.snapshot();

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
        active_scene: scene.selected_scene().map(|scene| scene.as_str().to_owned()),
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
        sprite_entities_2d: sprite_scene.entity_names(),
        text_entities_2d: text_scene.entity_names(),
        mesh_entities_3d: mesh_scene.entity_names(),
        material_entities_3d: material_scene.entity_names(),
        text_entities_3d: text3d_scene.entity_names(),
        ui_entities: ui_scene.entity_names(),
        audio_clips: audio_scene
            .clips()
            .into_iter()
            .map(|clip| format!("{} ({:?})", clip.key.as_str(), clip.mode))
            .collect(),
        audio_sources: audio_state
            .playing_sources()
            .into_iter()
            .map(|(source, clip)| format!("{source} -> {}", clip.as_str()))
            .collect(),
        pending_audio_runtime_commands: audio_state
            .pending_runtime_commands()
            .into_iter()
            .map(|command| crate::app_helpers::format_audio_command(&command))
            .collect(),
        audio_master_volume: audio_state.master_volume(),
        mixed_audio_frame_count: audio_mixer.frames().len(),
        active_realtime_audio_sources: audio_mixer.active_realtime_sources(),
        audio_output_started: audio_output_snapshot.started,
        audio_output_device: audio_output_snapshot.device_name,
        audio_output_buffered_samples: audio_output_snapshot.buffered_samples,
        audio_output_last_error: audio_output_snapshot.last_error,
        processed_script_commands: placeholder_bridge.processed_script_commands,
        processed_audio_commands: placeholder_bridge.processed_audio_commands,
        processed_scene_commands: placeholder_bridge.processed_scene_commands,
        processed_script_events: placeholder_bridge.processed_script_events,
        console_commands: dev_console_state.command_history(),
        console_output: dev_console_state.output_lines(),
        capabilities,
        plugins: report.plugin_names.into_iter().map(str::to_owned).collect(),
        services: report
            .service_names
            .into_iter()
            .map(str::to_owned)
            .collect(),
    })
}
