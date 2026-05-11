use std::path::{Path, PathBuf};

use amigo_2d_composition::Composition2dPlugin;
use amigo_2d_layered_image::LayeredImagePlugin;
use amigo_2d_lighting::Lighting2dPlugin;
use amigo_2d_motion::MOTION_2D_PLUGIN;
use amigo_2d_particles::Particle2dPlugin;
use amigo_2d_physics::Physics2dPlugin;
use amigo_2d_post_fx::PostFx2dPlugin;
use amigo_2d_sprite::SpritePlugin;
use amigo_2d_text::Text2dPlugin;
use amigo_2d_tilemap::TileMap2dPlugin;
use amigo_2d_vector::Vector2dPlugin;
use amigo_3d_material::MaterialPlugin;
use amigo_3d_mesh::MeshPlugin;
use amigo_3d_text::Text3dPlugin;
use amigo_app_host_winit::WinitAppHost;
use amigo_assets::AssetsPlugin;
use amigo_audio_api::AudioApiPlugin;
use amigo_audio_generated::GeneratedAudioPlugin;
use amigo_audio_mixer::AudioMixerPlugin;
use amigo_audio_output::AudioOutputPlugin;
use amigo_behavior::BehaviorPlugin;
use amigo_core::{AmigoResult, LaunchSelection};
use amigo_event_pipeline::EventPipelinePlugin;
use amigo_file_watch_notify::NotifyFileWatchPlugin;
use amigo_hot_reload::HotReloadPlugin;
use amigo_input_actions::InputActionPlugin;
use amigo_input_winit::WinitInputPlugin;
use amigo_modding::ModdingPlugin;
use amigo_render_wgpu::WgpuRenderPlugin;
use amigo_runtime::{PluginBundle, Runtime, RuntimeBuilder};
use amigo_session::{
    RenderSessionService, RuntimeSession, RuntimeSessionBootstrap, RuntimeSessionProfile,
    SceneSessionService, SchedulerSessionService, ScriptSessionService,
};
use amigo_scene::{SceneKey, ScenePlugin, SceneService};
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_state::StatePlugin;
use amigo_ui::UiPlugin;
use amigo_window_winit::WinitWindowPlugin;

use crate::dev_console::DevConsoleRuntimePlugin;
use crate::launch_selection::{build_launch_selection, validate_launch_selection};
use crate::orchestration::stabilize_runtime_for_session;
use crate::particle_presets::load_particle_preset_catalog;
use crate::runtime_context::required;
use crate::scene_runtime::{
    SceneCommandRuntimePlugin,
    current_loaded_scene_document_summary as current_loaded_scene_document_summary_runtime,
    load_scene_document_for_session, queue_scene_document_hydration_for_session,
};
use crate::script_runtime::ScriptCommandRuntimePlugin;
use crate::scripting_runtime::execute_mod_scripts;
use crate::summary::summarize;
use crate::systems::{
    AudioRuntimeSystemPlugin, RuntimeSystemServicesPlugin, SceneTransitionRuntimeSystemPlugin,
    ScriptUpdateRuntimeSystemPlugin, UiInputRuntimeSystemPlugin, World2dRuntimeSystemsPlugin,
};
use crate::{
    BootstrapOptions, BootstrapSummary, InteractiveRuntimeHostHandler, LaunchSelectionPlugin,
    LoadedSceneDocument, RuntimeDiagnosticsPlugin, SummaryHostHandler,
};

// Internal migration seam. New host/session code should use the
// session-aware bootstrap variants so lifecycle state remains visible through
// `RuntimeSession`.
pub(crate) fn bootstrap_default(
    mods_root: impl Into<PathBuf>,
) -> AmigoResult<(Runtime, BootstrapSummary)> {
    bootstrap_with_options(BootstrapOptions::new(mods_root))
}

pub fn bootstrap_session_default(
    mods_root: impl Into<PathBuf>,
) -> AmigoResult<RuntimeSessionBootstrap<BootstrapSummary>> {
    bootstrap_session_with_options(BootstrapOptions::new(mods_root))
}

pub(crate) fn bootstrap_with_options(
    options: BootstrapOptions,
) -> AmigoResult<(Runtime, BootstrapSummary)> {
    // NOTE:
    // This function still contains the legacy app-owned bootstrap implementation.
    // New host/editor-facing code should prefer `bootstrap_session_with_options`.
    let modding_plugin = match options.active_mods.clone() {
        Some(active_mods) => ModdingPlugin::with_selected_mods(&options.mods_root, active_mods),
        None => ModdingPlugin::new(&options.mods_root),
    };
    let launch_selection = build_launch_selection(&options);
    let scene_session_service = SceneSessionService::new();
    let render_session_service = RenderSessionService::new();
    let scheduler_session_service = SchedulerSessionService::new();
    let script_session_service = ScriptSessionService::new();

    let runtime = RuntimeBuilder::default()
        .with_service(scene_session_service)?
        .with_service(render_session_service)?
        .with_service(scheduler_session_service)?
        .with_service(script_session_service)?
        .with_bundle(CoreRuntimeBundle)?
        .with_bundle(PlatformRuntimeBundle {
            launch_selection: launch_selection.clone(),
        })?
        .with_bundle(TwoDBundle)?
        .with_bundle(AudioBundle)?
        .with_bundle(ThreeDBundle)?
        .with_bundle(ModdingAndScriptingBundle { modding_plugin })?
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
    Ok((session.into_runtime(), summary))
}

pub fn bootstrap_session_with_options(
    options: BootstrapOptions,
) -> AmigoResult<RuntimeSessionBootstrap<BootstrapSummary>> {
    let (runtime, summary) = bootstrap_with_options(options)?;
    let mut session = RuntimeSession::from_runtime(runtime, RuntimeSessionProfile::Game);

    crate::dev_console::register_app_dev_console_command_provider(&mut session);
    crate::diagnostics::register_host_diagnostics_provider(&mut session);
    crate::script_runtime::register_host_script_command_provider(&mut session);
    amigo_2d_text::register_text2d_runtime_contributions(&mut session);
    amigo_2d_sprite::register_sprite2d_runtime_contributions(&mut session);
    amigo_2d_tilemap::register_tilemap2d_runtime_contributions(&mut session);
    amigo_2d_layered_image::register_layered_image_runtime_contributions(&mut session);
    amigo_2d_composition::register_composition2d_runtime_contributions(&mut session);
    amigo_2d_lighting::register_lighting2d_runtime_contributions(&mut session);
    amigo_2d_post_fx::register_post_fx_runtime_contributions(&mut session);
    amigo_2d_particles::register_particles2d_runtime_contributions(&mut session);
    amigo_2d_motion::register_motion2d_runtime_contributions(&mut session);
    amigo_2d_physics::register_physics2d_runtime_contributions(&mut session);
    amigo_2d_vector::register_vector2d_runtime_contributions(&mut session);
    amigo_3d_mesh::register_mesh3d_runtime_contributions(&mut session);
    amigo_3d_material::register_material3d_runtime_contributions(&mut session);
    amigo_3d_text::register_text3d_runtime_contributions(&mut session);
    amigo_assets::register_assets_runtime_contributions(&mut session);
    amigo_scene::register_scene_runtime_contributions(&mut session);
    amigo_input_actions::register_input_actions_runtime_contributions(&mut session);
    amigo_behavior::register_behavior_runtime_contributions(&mut session);
    amigo_event_pipeline::register_event_pipeline_runtime_contributions(&mut session);
    amigo_render_api::register_render_runtime_contributions(&mut session);
    amigo_session::register_session_runtime_contributions(&mut session);
    amigo_audio_api::register_audio_runtime_contributions(&mut session);
    amigo_audio_mixer::register_audio_mixer_runtime_contributions(&mut session);
    amigo_ui::register_ui_runtime_contributions(&mut session);
    amigo_scripting_rhai::register_rhai_runtime_contributions(&mut session);
    crate::scene_runtime::register_legacy_scene_command_provider(&mut session);
    crate::systems::register_legacy_systems_provider(&mut session);
    crate::render_runtime::register_host_render_extractor_provider(&mut session);

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

// Internal migration seam. New host/session code should use the
// session-aware bootstrap variants so lifecycle state remains visible through
// `RuntimeSession`.
pub(crate) fn run_default(mods_root: impl AsRef<Path>) -> AmigoResult<BootstrapSummary> {
    let (_runtime, summary) = bootstrap_default(mods_root.as_ref().to_path_buf())?;
    Ok(summary)
}

// Internal migration seam. New host/session code should use the
// session-aware bootstrap variants so lifecycle state remains visible through
// `RuntimeSession`.
pub(crate) fn run_with_options(options: BootstrapOptions) -> AmigoResult<BootstrapSummary> {
    let (_runtime, summary) = bootstrap_with_options(options)?;
    Ok(summary)
}

pub(crate) fn run_hosted_once(mods_root: impl AsRef<Path>) -> AmigoResult<()> {
    run_hosted_with_options(BootstrapOptions::new(mods_root.as_ref().to_path_buf()))
}

pub fn run_hosted_with_options(options: BootstrapOptions) -> AmigoResult<()> {
    let interactive = should_use_interactive_host(&options);
    let (session, summary) = bootstrap_session_with_options(options)?.into_parts();

    if interactive {
        let handler = InteractiveRuntimeHostHandler::new(session, summary)?;
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

struct CoreRuntimeBundle;

impl PluginBundle for CoreRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-app-core-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(AssetsPlugin)?
            .with_plugin(HotReloadPlugin)?
            .with_plugin(NotifyFileWatchPlugin)?
            .with_plugin(ScenePlugin)?
            .with_plugin(StatePlugin)?
            .with_plugin(BehaviorPlugin)?
            .with_plugin(EventPipelinePlugin)
    }
}

struct PlatformRuntimeBundle {
    launch_selection: LaunchSelection,
}

impl PluginBundle for PlatformRuntimeBundle {
    fn name(&self) -> &'static str {
        "amigo-app-platform-runtime-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(WinitWindowPlugin::default())?
            .with_plugin(WinitInputPlugin)?
            .with_plugin(InputActionPlugin)?
            .with_plugin(WgpuRenderPlugin)?
            .with_plugin(LaunchSelectionPlugin::new(self.launch_selection))?
            .with_plugin(RuntimeSystemServicesPlugin)?
            .with_plugin(DevConsoleRuntimePlugin)?
            .with_plugin(UiInputRuntimeSystemPlugin)?
            .with_plugin(ScriptUpdateRuntimeSystemPlugin)?
            .with_plugin(World2dRuntimeSystemsPlugin)?
            .with_plugin(SceneTransitionRuntimeSystemPlugin)?
            .with_plugin(AudioRuntimeSystemPlugin)?
            .with_plugin(SceneCommandRuntimePlugin)?
            .with_plugin(ScriptCommandRuntimePlugin)
    }
}

struct TwoDBundle;

impl PluginBundle for TwoDBundle {
    fn name(&self) -> &'static str {
        "amigo-app-2d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(SpritePlugin)?
            .with_plugin(LayeredImagePlugin)?
            .with_plugin(Lighting2dPlugin)?
            .with_plugin(Composition2dPlugin)?
            .with_plugin(PostFx2dPlugin)?
            .with_plugin(Text2dPlugin)?
            .with_plugin(Vector2dPlugin)?
            .with_plugin(Particle2dPlugin)?
            .with_plugin(UiPlugin)?
            .with_plugin(Physics2dPlugin)?
            .with_plugin(TileMap2dPlugin)?
            .with_plugin(MOTION_2D_PLUGIN)
    }
}

struct AudioBundle;

impl PluginBundle for AudioBundle {
    fn name(&self) -> &'static str {
        "amigo-app-audio-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(AudioApiPlugin)?
            .with_plugin(GeneratedAudioPlugin)?
            .with_plugin(AudioMixerPlugin)?
            .with_plugin(AudioOutputPlugin)
    }
}

struct ThreeDBundle;

impl PluginBundle for ThreeDBundle {
    fn name(&self) -> &'static str {
        "amigo-app-3d-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(MeshPlugin)?
            .with_plugin(Text3dPlugin)?
            .with_plugin(MaterialPlugin)
    }
}

struct ModdingAndScriptingBundle {
    modding_plugin: ModdingPlugin,
}

impl PluginBundle for ModdingAndScriptingBundle {
    fn name(&self) -> &'static str {
        "amigo-app-modding-and-scripting-bundle"
    }

    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder> {
        builder
            .with_plugin(self.modding_plugin)?
            .with_plugin(RuntimeDiagnosticsPlugin::phase1())?
            .with_plugin(RhaiScriptingPlugin)
    }
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
