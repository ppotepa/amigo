use std::path::{Path, PathBuf};

use amigo_app_host_winit::WinitAppHost;
use amigo_assets::AssetsPlugin;
use amigo_audio_api::AudioApiPlugin;
use amigo_audio_generated::GeneratedAudioPlugin;
use amigo_audio_mixer::AudioMixerPlugin;
use amigo_audio_output::AudioOutputPlugin;
use amigo_core::{AmigoResult, LaunchSelection};
use amigo_file_watch_notify::NotifyFileWatchPlugin;
use amigo_hot_reload::HotReloadPlugin;
use amigo_modding::ModdingPlugin;
use amigo_runtime::{Runtime, RuntimeBuilder};
use amigo_scene::{
    HydratedSceneState, SceneCommandQueue, SceneKey, ScenePlugin, SceneService,
    SceneTransitionService,
};
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_ui::UiPlugin;
use amigo_window_winit::WinitWindowPlugin;
use amigo_input_winit::WinitInputPlugin;
use amigo_render_wgpu::WgpuRenderPlugin;
use amigo_2d_sprite::SpritePlugin;
use amigo_2d_text::Text2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_2d_tilemap::TileMap2dPlugin;
use amigo_2d_platformer::PlatformerPlugin;
use amigo_3d_mesh::MeshPlugin;
use amigo_3d_text::Text3dPlugin;
use amigo_3d_material::MaterialPlugin;

use crate::launch_selection::{build_launch_selection, validate_launch_selection};
use crate::orchestration::stabilize_runtime;
use crate::runtime_context::required;
use crate::scene_runtime::{
    current_loaded_scene_document_summary as current_loaded_scene_document_summary_runtime,
    load_scene_document_for_mod, SceneCommandRuntimePlugin,
    queue_scene_document_hydration,
};
use crate::summary::summarize;
use crate::script_runtime::ScriptCommandRuntimePlugin;
use crate::scripting_runtime::execute_mod_scripts;
use crate::{
    BootstrapOptions, BootstrapSummary, InteractiveRuntimeHostHandler, LaunchSelectionPlugin,
    LoadedSceneDocument, RuntimeDiagnosticsPlugin, SummaryHostHandler,
};

pub fn bootstrap_default(
    mods_root: impl Into<PathBuf>,
) -> AmigoResult<(Runtime, BootstrapSummary)> {
    bootstrap_with_options(BootstrapOptions::new(mods_root))
}

pub fn bootstrap_with_options(
    options: BootstrapOptions,
) -> AmigoResult<(Runtime, BootstrapSummary)> {
    let modding_plugin = match options.active_mods.clone() {
        Some(active_mods) => ModdingPlugin::with_selected_mods(&options.mods_root, active_mods),
        None => ModdingPlugin::new(&options.mods_root),
    };
    let launch_selection = build_launch_selection(&options);

    let runtime = RuntimeBuilder::default()
        .with_plugin(AssetsPlugin)?
        .with_plugin(HotReloadPlugin)?
        .with_plugin(NotifyFileWatchPlugin)?
        .with_plugin(ScenePlugin)?
        .with_plugin(WinitWindowPlugin::default())?
        .with_plugin(WinitInputPlugin)?
        .with_plugin(WgpuRenderPlugin)?
        .with_plugin(LaunchSelectionPlugin::new(launch_selection.clone()))?
        .with_plugin(SceneCommandRuntimePlugin)?
        .with_plugin(ScriptCommandRuntimePlugin)?
        .with_plugin(SpritePlugin)?
        .with_plugin(Text2dPlugin)?
        .with_plugin(UiPlugin)?
        .with_plugin(Physics2dPlugin)?
        .with_plugin(TileMap2dPlugin)?
        .with_plugin(PlatformerPlugin)?
        .with_plugin(AudioApiPlugin)?
        .with_plugin(GeneratedAudioPlugin)?
        .with_plugin(AudioMixerPlugin)?
        .with_plugin(AudioOutputPlugin)?
        .with_plugin(MeshPlugin)?
        .with_plugin(Text3dPlugin)?
        .with_plugin(MaterialPlugin)?
        .with_plugin(modding_plugin)?
        .with_plugin(RuntimeDiagnosticsPlugin::phase1())?
        .with_plugin(RhaiScriptingPlugin)?
        .build();

    validate_launch_selection(&runtime, &launch_selection)?;
    let loaded_scene_document = load_selected_scene_document(&runtime, &launch_selection)?;
    apply_initial_scene_selection(&runtime, &launch_selection)?;
    queue_loaded_scene_document_hydration(&runtime, loaded_scene_document.as_ref())?;
    execute_mod_scripts(&runtime)?;
    let placeholder_bridge = stabilize_runtime(&runtime)?;
    let loaded_scene_document = current_loaded_scene_document_summary(&runtime)?;
    let summary = summarize(
        &runtime,
        launch_selection,
        placeholder_bridge,
        loaded_scene_document,
    )?;
    Ok((runtime, summary))
}

pub fn run_default(mods_root: impl AsRef<Path>) -> AmigoResult<BootstrapSummary> {
    let (_runtime, summary) = bootstrap_default(mods_root.as_ref().to_path_buf())?;
    Ok(summary)
}

pub fn run_with_options(options: BootstrapOptions) -> AmigoResult<BootstrapSummary> {
    let (_runtime, summary) = bootstrap_with_options(options)?;
    Ok(summary)
}

pub fn run_hosted_once(mods_root: impl AsRef<Path>) -> AmigoResult<()> {
    run_hosted_with_options(BootstrapOptions::new(mods_root.as_ref().to_path_buf()))
}

pub fn run_hosted_with_options(options: BootstrapOptions) -> AmigoResult<()> {
    let interactive = should_use_interactive_host(&options);
    let (runtime, summary) = bootstrap_with_options(options)?;

    if interactive {
        let handler = InteractiveRuntimeHostHandler::new(runtime, summary)?;
        WinitAppHost::run(handler)
    } else {
        let handler = SummaryHostHandler::new(summary);
        WinitAppHost::run(handler)
    }
}

pub(crate) fn should_use_interactive_host(options: &BootstrapOptions) -> bool {
    options.dev_mode
        || options
            .startup_mod
            .as_deref()
            .is_some_and(|mod_id| mod_id != "core")
}

fn load_selected_scene_document(
    runtime: &Runtime,
    launch_selection: &LaunchSelection,
) -> AmigoResult<Option<LoadedSceneDocument>> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(None);
    };
    let Some(startup_scene) = launch_selection.startup_scene.as_deref() else {
        return Ok(None);
    };

    load_scene_document_for_mod(runtime, startup_mod, startup_scene)
}

fn queue_loaded_scene_document_hydration(
    runtime: &Runtime,
    loaded_scene_document: Option<&LoadedSceneDocument>,
) -> AmigoResult<()> {
    let Some(loaded_scene_document) = loaded_scene_document else {
        return Ok(());
    };

    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let dev_console_state = required::<amigo_scripting_api::DevConsoleState>(runtime)?;
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;

    queue_scene_document_hydration(
        scene_command_queue.as_ref(),
        dev_console_state.as_ref(),
        hydrated_scene_state.as_ref(),
        scene_transition_service.as_ref(),
        loaded_scene_document,
    );

    Ok(())
}

pub(crate) fn current_loaded_scene_document_summary(
    runtime: &Runtime,
) -> AmigoResult<Option<crate::LoadedSceneDocumentSummary>> {
    current_loaded_scene_document_summary_runtime(runtime)
}

fn apply_initial_scene_selection(
    runtime: &Runtime,
    launch_selection: &LaunchSelection,
) -> AmigoResult<()> {
    let Some(startup_scene) = launch_selection.startup_scene.as_deref() else {
        return Ok(());
    };
    let scene_service = required::<SceneService>(runtime)?;

    if scene_service.selected_scene().is_none() {
        scene_service.select_scene(SceneKey::new(startup_scene));
    }

    Ok(())
}
