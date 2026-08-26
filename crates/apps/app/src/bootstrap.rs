use std::path::PathBuf;

use amigo_app_host_winit::WinitAppHost;
use amigo_core::{AmigoResult, LaunchSelection};
use amigo_modding::ModdingPlugin;
use amigo_runtime::{Runtime, RuntimeBuilder};
use amigo_scene::{SceneKey, SceneService};
use amigo_session::{RuntimeSession, RuntimeSessionBootstrap, RuntimeSessionProfile};

use crate::launch_selection::{build_launch_selection, validate_launch_selection};
use crate::orchestration::stabilize_runtime_for_session;
use crate::particle_presets::load_particle_preset_catalog;
use crate::runtime_context::required;
use crate::runtime_control::RuntimeControlPlugin;
use crate::scene_runtime::{
    SceneCommandRuntimePlugin,
    current_loaded_scene_document_summary as current_loaded_scene_document_summary_runtime,
    load_scene_document_for_session, queue_scene_document_hydration_for_session,
};
use crate::script_runtime::ScriptCommandRuntimePlugin;
use crate::scripting_runtime::execute_mod_scripts;
use crate::summary::summarize;
use crate::systems::RuntimeSystemServicesPlugin;
use crate::{
    BootstrapOptions, BootstrapSummary, InteractiveRuntimeHostHandler, LaunchSelectionPlugin,
    LoadedSceneDocument, RuntimeDiagnosticsPlugin, SummaryHostHandler,
};

pub fn bootstrap_session_default(
    mods_root: impl Into<PathBuf>,
) -> AmigoResult<RuntimeSessionBootstrap<BootstrapSummary>> {
    bootstrap_session_with_options(BootstrapOptions::new(mods_root))
}

#[allow(dead_code)]
pub(crate) fn bootstrap_with_options(
    options: BootstrapOptions,
) -> AmigoResult<(Runtime, BootstrapSummary)> {
    let (session, summary) = bootstrap_runtime_session_with_options(options)?;
    Ok((session.into_runtime(), summary))
}

fn bootstrap_runtime_session_with_options(
    options: BootstrapOptions,
) -> AmigoResult<(RuntimeSession, BootstrapSummary)> {
    let modding_plugin = match options.active_mods.clone() {
        Some(active_mods) => ModdingPlugin::with_selected_mods(&options.mods_root, active_mods),
        None => ModdingPlugin::new(&options.mods_root),
    };
    let launch_selection = build_launch_selection(&options);

    let runtime = amigo_runtime_bundles::compose_game_runtime(
        launch_selection.clone(),
        register_app_host_platform_plugins,
        modding_plugin,
        true,
    )?
    .with_plugin(amigo_editor_authoring::EditorAuthoringPlugin::new(
        options.dev_mode || options.editor_mode,
    ))?
    .with_plugin(amigo_editor_ingame::IngameEditorPlugin::new(
        options.editor_mode,
    ))?
    .with_plugin(RuntimeDiagnosticsPlugin::phase1())?
    .build();

    let mut session = RuntimeSession::from_runtime(runtime, RuntimeSessionProfile::Game);

    validate_launch_selection(session.runtime(), &launch_selection)?;
    preload_runtime_font_assets(session.runtime())?;
    load_particle_preset_catalog(session.runtime())?;
    let loaded_scene_document = load_selected_scene_document(&mut session, &launch_selection)?;
    apply_initial_scene_selection(session.runtime(), &launch_selection)?;
    queue_loaded_scene_document_hydration(&mut session, loaded_scene_document.as_ref())?;
    execute_mod_scripts(session.runtime())?;
    let placeholder_bridge = stabilize_runtime_for_session(&session)?;
    let loaded_scene_document = current_loaded_scene_document_summary(session.runtime())?;
    let summary = summarize(
        session.runtime(),
        launch_selection,
        placeholder_bridge,
        loaded_scene_document,
    )?;
    Ok((session, summary))
}

pub fn bootstrap_session_with_options(
    options: BootstrapOptions,
) -> AmigoResult<RuntimeSessionBootstrap<BootstrapSummary>> {
    let (mut session, summary) = bootstrap_runtime_session_with_options(options)?;
    amigo_runtime_bundles::register_full_runtime_capabilities(&mut session);
    Ok(RuntimeSessionBootstrap::new(session, summary))
}

fn preload_runtime_font_assets(runtime: &Runtime) -> AmigoResult<()> {
    let asset_catalog = required::<amigo_assets::AssetCatalog>(runtime)?;
    let mod_catalog = required::<amigo_modding::ModCatalog>(runtime)?;
    if mod_catalog.mod_by_id("core").is_none() {
        return Ok(());
    }

    crate::app_helpers::register_mod_asset_reference(
        asset_catalog.as_ref(),
        "core",
        &amigo_assets::AssetKey::new("core/fonts/console-mono"),
        "font",
        "runtime-default",
    );
    Ok(())
}

pub fn run_hosted_with_options(options: BootstrapOptions) -> AmigoResult<()> {
    let interactive = should_use_interactive_host(&options);
    let editor_mode = options.editor_mode;
    let (session, summary) = bootstrap_session_with_options(options)?.into_parts();

    if interactive {
        let handler = InteractiveRuntimeHostHandler::new_with_editor_mode(
            session,
            summary,
            editor_mode,
        )?;
        WinitAppHost::run(handler)
    } else {
        let handler = SummaryHostHandler::new(summary);
        WinitAppHost::run(handler)
    }
}

pub(crate) fn should_use_interactive_host(options: &BootstrapOptions) -> bool {
    options.editor_mode
        || options.dev_mode
        || options
            .startup_mod
            .as_deref()
            .is_some_and(|mod_id| mod_id != "core")
}

fn load_selected_scene_document(
    session: &mut RuntimeSession,
    launch_selection: &LaunchSelection,
) -> AmigoResult<Option<LoadedSceneDocument>> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(None);
    };
    let Some(startup_scene) = launch_selection.startup_scene.as_deref() else {
        return Ok(None);
    };
    load_scene_document_for_session(session, startup_mod, startup_scene)
}

fn queue_loaded_scene_document_hydration(
    session: &mut RuntimeSession,
    loaded_scene_document: Option<&LoadedSceneDocument>,
) -> AmigoResult<()> {
    let Some(loaded_scene_document) = loaded_scene_document else {
        return Ok(());
    };
    queue_scene_document_hydration_for_session(session, loaded_scene_document)
}

fn register_app_host_platform_plugins(
    builder: RuntimeBuilder,
    launch_selection: LaunchSelection,
) -> AmigoResult<RuntimeBuilder> {
    builder
        .with_plugin(LaunchSelectionPlugin::new(launch_selection))?
        .with_plugin(RuntimeSystemServicesPlugin)?
        .with_plugin(RuntimeControlPlugin)?
        .with_plugin(SceneCommandRuntimePlugin)?
        .with_plugin(ScriptCommandRuntimePlugin)
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
