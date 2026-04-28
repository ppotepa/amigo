use std::any::type_name;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use amigo_2d_physics::{
    AabbCollider2d, AabbCollider2dCommand, CollisionLayer, CollisionMask, KinematicBody2d,
    KinematicBody2dCommand, Physics2dDomainInfo, Physics2dPlugin, Physics2dSceneService,
    StaticCollider2d, StaticCollider2dCommand, Trigger2d, Trigger2dCommand, move_and_collide,
    overlaps_trigger,
};
use amigo_2d_platformer::{
    PlatformerAnimationState, PlatformerController2d, PlatformerController2dCommand,
    PlatformerControllerParams, PlatformerControllerState, PlatformerDomainInfo, PlatformerFacing,
    PlatformerPlugin, PlatformerSceneService, animation_state_for, drive_controller,
};
use amigo_2d_sprite::{
    Sprite, SpriteDomainInfo, SpriteDrawCommand, SpritePlugin, SpriteSceneService, SpriteSheet,
};
use amigo_2d_text::{
    Text2d, Text2dDomainInfo, Text2dDrawCommand, Text2dPlugin, Text2dSceneService,
};
use amigo_2d_tilemap::{
    TileCollisionKind2d, TileMap2d, TileMap2dDomainInfo, TileMap2dDrawCommand, TileMap2dPlugin,
    TileMap2dSceneService, TileRuleSet2d, TileTerrainRule2d, TileVariantSet2d, marker_cells,
    resolve_tilemap, solid_cells,
};
use amigo_3d_material::{
    Material3d, MaterialDomainInfo, MaterialDrawCommand, MaterialPlugin, MaterialSceneService,
};
use amigo_3d_mesh::{Mesh3d, MeshDomainInfo, MeshDrawCommand, MeshPlugin, MeshSceneService};
use amigo_3d_text::{
    Text3d, Text3dDomainInfo, Text3dDrawCommand, Text3dPlugin, Text3dSceneService,
};
use amigo_app_host_api::{
    HostConfig, HostControl, HostExitStrategy, HostHandler, HostLifecycleEvent,
};
use amigo_app_host_winit::WinitAppHost;
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    AssetsPlugin, prepare_asset_from_contents,
};
use amigo_audio_api::{
    AudioApiPlugin, AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioDomainInfo,
    AudioPlaybackMode, AudioSceneService, AudioSourceId, AudioStateService,
};
use amigo_audio_generated::{GeneratedAudioDomainInfo, GeneratedAudioPlugin};
use amigo_audio_mixer::{AudioMixerDomainInfo, AudioMixerPlugin, AudioMixerService};
use amigo_audio_output::{
    AudioOutputBackendService, AudioOutputDomainInfo, AudioOutputPlugin, AudioOutputStartStatus,
};
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_file_watch_api::{FileWatchBackendInfo, FileWatchService};
use amigo_file_watch_notify::NotifyFileWatchPlugin;
use amigo_hot_reload::{
    AssetWatch, HotReloadPlugin, HotReloadService, HotReloadWatchKind, SceneDocumentWatch,
};
use amigo_input_api::{InputEvent, InputServiceInfo, InputState, KeyCode};
use amigo_input_winit::WinitInputPlugin;
use amigo_math::Vec2;
use amigo_modding::{ModCatalog, ModScriptMode, ModdingPlugin};
use amigo_render_api::RenderBackendInfo;
use amigo_render_wgpu::{
    UiLayoutNode as OverlayUiLayoutNode, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
    UiOverlayNodeKind, UiOverlayStyle, UiViewportSize, WgpuRenderBackend, WgpuRenderPlugin,
    WgpuSceneRenderer, WgpuSurfaceState, build_ui_layout_tree,
};
use amigo_runtime::{Runtime, RuntimeBuilder, RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    CameraFollow2dSceneCommand, CameraFollow2dSceneService, HydratedSceneSnapshot,
    HydratedSceneState, Material3dSceneCommand, Mesh3dSceneCommand, Parallax2dSceneCommand,
    Parallax2dSceneService, SceneCommand, SceneCommandQueue, SceneEvent, SceneEventQueue,
    SceneHydrationPlan, SceneKey, ScenePlugin, SceneService, SceneTransitionPlan,
    SceneTransitionService, SceneUiDocument, SceneUiEventBinding, SceneUiLayer, SceneUiNode,
    SceneUiNodeKind, SceneUiStyle, SceneUiTarget, Sprite2dSceneCommand, Text2dSceneCommand,
    Text3dSceneCommand, build_scene_hydration_plan, build_scene_transition_plan,
    load_scene_document_from_path,
};
use amigo_scripting_api::{
    DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptCommandQueue, ScriptEvent,
    ScriptEventQueue, ScriptLifecycleState, ScriptRuntimeInfo, ScriptRuntimeService,
};
use amigo_scripting_rhai::RhaiScriptingPlugin;
use amigo_ui::{
    UiDocument as RuntimeUiDocument, UiDomainInfo, UiDrawCommand, UiEventBinding, UiInputService,
    UiLayer as RuntimeUiLayer, UiNode as RuntimeUiNode, UiNodeKind as RuntimeUiNodeKind, UiPlugin,
    UiSceneService, UiStateService, UiStateSnapshot, UiStyle as RuntimeUiStyle,
    UiTarget as RuntimeUiTarget,
};
use amigo_window_api::{WindowDescriptor, WindowEvent, WindowServiceInfo, WindowSurfaceHandles};
use amigo_window_winit::WinitWindowPlugin;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptExecutionRole {
    ModBootstrap,
    ModPersistent,
    Scene,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutedScript {
    pub source_name: String,
    pub mod_id: String,
    pub scene_id: Option<String>,
    pub relative_script_path: PathBuf,
    pub role: ScriptExecutionRole,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedSceneDocumentSummary {
    pub source_mod: String,
    pub scene_id: String,
    pub relative_path: PathBuf,
    pub entity_names: Vec<String>,
    pub component_kinds: Vec<String>,
    pub transition_ids: Vec<String>,
}

#[derive(Debug, Clone)]
struct LoadedSceneDocument {
    summary: LoadedSceneDocumentSummary,
    hydration_plan: SceneHydrationPlan,
    transition_plan: Option<SceneTransitionPlan>,
}

#[derive(Debug, Clone)]
pub struct BootstrapSummary {
    pub window_backend: String,
    pub input_backend: String,
    pub render_backend: String,
    pub script_backend: String,
    pub file_watch_backend: String,
    pub loaded_mods: Vec<String>,
    pub executed_scripts: Vec<ExecutedScript>,
    pub startup_mod: Option<String>,
    pub startup_scene: Option<String>,
    pub active_scene: Option<String>,
    pub loaded_scene_document: Option<LoadedSceneDocumentSummary>,
    pub scene_entities: Vec<String>,
    pub registered_assets: Vec<String>,
    pub loaded_assets: Vec<String>,
    pub prepared_assets: Vec<String>,
    pub failed_assets: Vec<String>,
    pub pending_asset_loads: Vec<String>,
    pub watched_reload_targets: Vec<String>,
    pub sprite_entities_2d: Vec<String>,
    pub text_entities_2d: Vec<String>,
    pub mesh_entities_3d: Vec<String>,
    pub material_entities_3d: Vec<String>,
    pub text_entities_3d: Vec<String>,
    pub ui_entities: Vec<String>,
    pub audio_clips: Vec<String>,
    pub audio_sources: Vec<String>,
    pub processed_script_commands: Vec<String>,
    pub processed_audio_commands: Vec<String>,
    pub processed_scene_commands: Vec<String>,
    pub processed_script_events: Vec<String>,
    pub console_commands: Vec<String>,
    pub console_output: Vec<String>,
    pub capabilities: Vec<String>,
    pub plugins: Vec<String>,
    pub services: Vec<String>,
}

#[derive(Debug, Clone, Default)]
struct PlaceholderBridgeSummary {
    processed_script_commands: Vec<String>,
    processed_audio_commands: Vec<String>,
    processed_scene_commands: Vec<String>,
    processed_script_events: Vec<String>,
    console_commands: Vec<String>,
    console_output: Vec<String>,
}

#[derive(Debug, Clone)]
struct ResolvedUiOverlayDocument {
    overlay: UiOverlayDocument,
    click_bindings: BTreeMap<String, UiEventBinding>,
}

const MAX_PLACEHOLDER_BRIDGE_PASSES: usize = 16;
const MAX_RUNTIME_STABILIZATION_PASSES: usize = 16;

impl Display for BootstrapSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amigo bootstrap")?;
        writeln!(f, "window backend: {}", self.window_backend)?;
        writeln!(f, "input backend: {}", self.input_backend)?;
        writeln!(f, "render backend: {}", self.render_backend)?;
        writeln!(f, "script backend: {}", self.script_backend)?;
        writeln!(f, "file watch backend: {}", self.file_watch_backend)?;
        writeln!(f, "mods: {}", display_string_list(&self.loaded_mods))?;
        writeln!(
            f,
            "scripts: {}",
            display_executed_scripts(&self.executed_scripts)
        )?;
        writeln!(
            f,
            "root mod: {}",
            self.startup_mod.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "startup scene: {}",
            self.startup_scene.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "active scene: {}",
            self.active_scene.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "scene document: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| format!(
                    "{}:{}",
                    document.source_mod,
                    document.relative_path.display()
                ))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document entities: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| display_string_list(&document.entity_names))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document components: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| display_string_list(&document.component_kinds))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document transitions: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| display_string_list(&document.transition_ids))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene entities: {}",
            display_string_list(&self.scene_entities)
        )?;
        writeln!(
            f,
            "registered assets: {}",
            display_string_list(&self.registered_assets)
        )?;
        writeln!(
            f,
            "loaded assets: {}",
            display_string_list(&self.loaded_assets)
        )?;
        writeln!(
            f,
            "prepared assets: {}",
            display_string_list(&self.prepared_assets)
        )?;
        writeln!(
            f,
            "failed assets: {}",
            display_string_list(&self.failed_assets)
        )?;
        writeln!(
            f,
            "pending asset loads: {}",
            display_string_list(&self.pending_asset_loads)
        )?;
        writeln!(
            f,
            "watched reload targets: {}",
            display_string_list(&self.watched_reload_targets)
        )?;
        writeln!(
            f,
            "2d sprite entities: {}",
            display_string_list(&self.sprite_entities_2d)
        )?;
        writeln!(
            f,
            "2d text entities: {}",
            display_string_list(&self.text_entities_2d)
        )?;
        writeln!(
            f,
            "3d mesh entities: {}",
            display_string_list(&self.mesh_entities_3d)
        )?;
        writeln!(
            f,
            "3d material entities: {}",
            display_string_list(&self.material_entities_3d)
        )?;
        writeln!(
            f,
            "3d text entities: {}",
            display_string_list(&self.text_entities_3d)
        )?;
        writeln!(f, "ui entities: {}", display_string_list(&self.ui_entities))?;
        writeln!(f, "audio clips: {}", display_string_list(&self.audio_clips))?;
        writeln!(
            f,
            "audio sources: {}",
            display_string_list(&self.audio_sources)
        )?;
        writeln!(
            f,
            "script commands: {}",
            display_string_list(&self.processed_script_commands)
        )?;
        writeln!(
            f,
            "audio commands: {}",
            display_string_list(&self.processed_audio_commands)
        )?;
        writeln!(
            f,
            "scene commands: {}",
            display_string_list(&self.processed_scene_commands)
        )?;
        writeln!(
            f,
            "script events: {}",
            display_string_list(&self.processed_script_events)
        )?;
        writeln!(
            f,
            "console commands: {}",
            display_string_list(&self.console_commands)
        )?;
        writeln!(
            f,
            "console output: {}",
            display_string_list(&self.console_output)
        )?;
        writeln!(
            f,
            "capabilities: {}",
            display_string_list(&self.capabilities)
        )?;
        writeln!(f, "plugins: {}", display_string_list(&self.plugins))?;
        write!(f, "services: {}", display_string_list(&self.services))
    }
}

#[derive(Debug, Clone)]
pub struct BootstrapOptions {
    pub mods_root: PathBuf,
    pub active_mods: Option<Vec<String>>,
    pub startup_mod: Option<String>,
    pub startup_scene: Option<String>,
    pub dev_mode: bool,
}

impl Default for BootstrapOptions {
    fn default() -> Self {
        Self {
            mods_root: PathBuf::from("mods"),
            active_mods: None,
            startup_mod: None,
            startup_scene: None,
            dev_mode: false,
        }
    }
}

impl BootstrapOptions {
    pub fn new(mods_root: impl Into<PathBuf>) -> Self {
        Self {
            mods_root: mods_root.into(),
            ..Self::default()
        }
    }

    pub fn with_active_mods(mut self, active_mods: impl Into<Vec<String>>) -> Self {
        self.active_mods = Some(active_mods.into());
        self
    }

    pub fn with_startup_mod(mut self, startup_mod: impl Into<String>) -> Self {
        self.startup_mod = Some(startup_mod.into());
        self
    }

    pub fn with_startup_scene(mut self, startup_scene: impl Into<String>) -> Self {
        self.startup_scene = Some(startup_scene.into());
        self
    }

    pub fn with_dev_mode(mut self, dev_mode: bool) -> Self {
        self.dev_mode = dev_mode;
        self
    }
}

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

fn should_use_interactive_host(options: &BootstrapOptions) -> bool {
    options.dev_mode
        || options
            .startup_mod
            .as_deref()
            .is_some_and(|mod_id| mod_id != "core")
}

fn start_audio_output(runtime: &Runtime) -> AmigoResult<()> {
    let audio_backend = required::<AudioOutputBackendService>(runtime)?;
    match audio_backend.start_if_available() {
        Ok(AudioOutputStartStatus::Started) => {
            let snapshot = audio_backend.snapshot();
            println!(
                "audio init: backend={} device={} sample_rate={} channels={}",
                snapshot.backend_name,
                snapshot.device_name.as_deref().unwrap_or("unknown"),
                snapshot.sample_rate.unwrap_or_default(),
                snapshot.channels.unwrap_or_default()
            );
        }
        Ok(AudioOutputStartStatus::AlreadyStarted) => {}
        Ok(AudioOutputStartStatus::Unavailable) => {
            let snapshot = audio_backend.snapshot();
            println!(
                "audio init: backend={} unavailable ({})",
                snapshot.backend_name,
                snapshot
                    .last_error
                    .as_deref()
                    .unwrap_or("no audio output device")
            );
        }
        Err(error) => {
            println!("audio init failed: {error}");
        }
    }

    Ok(())
}

fn summarize(
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

fn refresh_runtime_summary(runtime: &Runtime) -> AmigoResult<BootstrapSummary> {
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
    capabilities.push(
        required::<Physics2dDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(
        required::<TileMap2dDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(
        required::<PlatformerDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(required::<AudioDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(
        required::<GeneratedAudioDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(
        required::<AudioMixerDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(
        required::<AudioOutputDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
    capabilities.push(required::<MeshDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(required::<Text3dDomainInfo>(runtime)?.capability.to_owned());
    capabilities.push(
        required::<MaterialDomainInfo>(runtime)?
            .capability
            .to_owned(),
    );
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

fn build_launch_selection(options: &BootstrapOptions) -> LaunchSelection {
    LaunchSelection::new(
        options.startup_mod.clone(),
        options.startup_scene.clone(),
        options.active_mods.clone().unwrap_or_default(),
        options.dev_mode,
    )
}

fn validate_launch_selection(
    runtime: &Runtime,
    launch_selection: &LaunchSelection,
) -> AmigoResult<()> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(());
    };
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let discovered_mod = mod_catalog.mod_by_id(startup_mod).ok_or_else(|| {
        AmigoError::Message(format!(
            "root mod `{startup_mod}` was not loaded by the current bootstrap selection"
        ))
    })?;

    if let Some(startup_scene) = launch_selection.startup_scene.as_deref() {
        if discovered_mod.scene_by_id(startup_scene).is_none() {
            return Err(AmigoError::Message(format!(
                "startup scene `{startup_scene}` was not declared by root mod `{startup_mod}`"
            )));
        }
    }

    Ok(())
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

fn load_scene_document_for_mod(
    runtime: &Runtime,
    root_mod: &str,
    scene_id: &str,
) -> AmigoResult<Option<LoadedSceneDocument>> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let discovered_mod = mod_catalog.mod_by_id(root_mod).ok_or_else(|| {
        AmigoError::Message(format!(
            "root mod `{root_mod}` was not loaded by the current bootstrap selection"
        ))
    })?;
    let scene_manifest = discovered_mod.scene_by_id(scene_id).ok_or_else(|| {
        AmigoError::Message(format!(
            "scene `{scene_id}` was not declared by root mod `{root_mod}`"
        ))
    })?;
    let document_path = discovered_mod
        .scene_document_path(scene_id)
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` has no resolved document path"
            ))
        })?;
    if !document_path.is_file() {
        return if scene_manifest.document.is_some() {
            Err(AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` declares document `{}` but the file does not exist",
                document_path.display()
            )))
        } else {
            Err(AmigoError::Message(format!(
                "scene `{scene_id}` for mod `{root_mod}` is missing default document `{}`",
                document_path.display()
            )))
        };
    }
    let relative_document_path =
        relative_path_within_root(&discovered_mod.root_path, &document_path)?;
    let document = load_scene_document_from_path(&document_path)
        .map_err(|error| AmigoError::Message(error.to_string()))?;

    if document.scene.id != scene_id {
        return Err(AmigoError::Message(format!(
            "scene document `{}` declares id `{}` but manifest selected `{scene_id}`",
            document_path.display(),
            document.scene.id
        )));
    }

    let hydration_plan = build_scene_hydration_plan(root_mod, &document)
        .map_err(|error| AmigoError::Message(error.to_string()))?;
    let transition_plan = build_scene_transition_plan(root_mod, &document)
        .map_err(|error| AmigoError::Message(error.to_string()))?;

    let component_kinds = document
        .component_kind_counts()
        .into_iter()
        .map(|(kind, count)| format!("{kind} x{count}"))
        .collect::<Vec<_>>();
    let transition_ids = transition_plan
        .as_ref()
        .map(|plan| {
            plan.transitions
                .iter()
                .map(|transition| transition.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    Ok(Some(LoadedSceneDocument {
        summary: LoadedSceneDocumentSummary {
            source_mod: root_mod.to_owned(),
            scene_id: scene_id.to_owned(),
            relative_path: relative_document_path,
            entity_names: document.entity_names(),
            component_kinds,
            transition_ids,
        },
        hydration_plan,
        transition_plan,
    }))
}

fn queue_loaded_scene_document_hydration(
    runtime: &Runtime,
    loaded_scene_document: Option<&LoadedSceneDocument>,
) -> AmigoResult<()> {
    let Some(loaded_scene_document) = loaded_scene_document else {
        return Ok(());
    };

    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
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

fn queue_scene_document_hydration(
    scene_command_queue: &SceneCommandQueue,
    dev_console_state: &DevConsoleState,
    hydrated_scene_state: &HydratedSceneState,
    scene_transition_service: &SceneTransitionService,
    loaded_scene_document: &LoadedSceneDocument,
) {
    hydrated_scene_state.replace(HydratedSceneSnapshot {
        source_mod: Some(loaded_scene_document.summary.source_mod.clone()),
        scene_id: Some(loaded_scene_document.summary.scene_id.clone()),
        relative_document_path: Some(loaded_scene_document.summary.relative_path.clone()),
        entity_names: loaded_scene_document.summary.entity_names.clone(),
        component_kinds: loaded_scene_document.summary.component_kinds.clone(),
    });
    scene_transition_service.activate(loaded_scene_document.transition_plan.clone());

    for command in &loaded_scene_document.hydration_plan.commands {
        scene_command_queue.submit(command.clone());
    }

    dev_console_state.write_line(format!(
        "queued scene document hydration for `{}` with {} commands",
        loaded_scene_document.summary.scene_id,
        loaded_scene_document.hydration_plan.commands.len()
    ));
}

fn current_loaded_scene_document_summary(
    runtime: &Runtime,
) -> AmigoResult<Option<LoadedSceneDocumentSummary>> {
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let snapshot = hydrated_scene_state.snapshot();
    let transition_snapshot = scene_transition_service.snapshot();
    let (Some(source_mod), Some(scene_id), Some(relative_path)) = (
        snapshot.source_mod,
        snapshot.scene_id,
        snapshot.relative_document_path,
    ) else {
        return Ok(None);
    };

    Ok(Some(LoadedSceneDocumentSummary {
        source_mod,
        scene_id,
        relative_path,
        entity_names: snapshot.entity_names,
        component_kinds: snapshot.component_kinds,
        transition_ids: transition_snapshot.transition_ids,
    }))
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

#[derive(Debug, Clone)]
struct PreparedScriptSource {
    executed: ExecutedScript,
    source: String,
}

fn mod_script_source_name(mod_id: &str) -> String {
    format!("mod:{mod_id}")
}

fn scene_script_source_name(mod_id: &str, scene_id: &str) -> String {
    format!("scene:{mod_id}:{scene_id}")
}

fn relative_path_within_root(root_path: &Path, absolute_path: &Path) -> AmigoResult<PathBuf> {
    let relative_path = absolute_path.strip_prefix(root_path).map_err(|_| {
        AmigoError::Message(format!(
            "script path `{}` must stay within mod root `{}`",
            absolute_path.display(),
            root_path.display()
        ))
    })?;

    if relative_path.components().any(|component| {
        matches!(
            component,
            Component::ParentDir | Component::RootDir | Component::Prefix(_)
        )
    }) {
        return Err(AmigoError::Message(format!(
            "script path `{}` resolved to an invalid relative path `{}`",
            absolute_path.display(),
            relative_path.display()
        )));
    }

    Ok(relative_path.to_path_buf())
}

fn validate_script_path(
    script_runtime: &ScriptRuntimeService,
    script_path: &Path,
    owner_label: &str,
) -> AmigoResult<()> {
    let extension = script_path
        .extension()
        .and_then(|extension| extension.to_str())
        .ok_or_else(|| {
            AmigoError::Message(format!(
                "{owner_label} `{}` has no file extension",
                script_path.display()
            ))
        })?;

    if !script_runtime.supports_extension(extension) {
        return Err(AmigoError::Message(format!(
            "{owner_label} `{}` is not supported by `{}`",
            script_path.display(),
            script_runtime.backend_name()
        )));
    }

    Ok(())
}

fn build_script_descriptor(
    root_path: &Path,
    absolute_script_path: &Path,
    source_name: String,
    mod_id: &str,
    scene_id: Option<&str>,
    role: ScriptExecutionRole,
) -> AmigoResult<ExecutedScript> {
    let relative_script_path = relative_path_within_root(root_path, absolute_script_path)?;

    Ok(ExecutedScript {
        source_name,
        mod_id: mod_id.to_owned(),
        scene_id: scene_id.map(str::to_owned),
        relative_script_path,
        role,
    })
}

fn prepare_mod_script_source(
    script_runtime: &ScriptRuntimeService,
    discovered_mod: &amigo_modding::DiscoveredMod,
) -> AmigoResult<Option<PreparedScriptSource>> {
    let Some(scripting) = discovered_mod.manifest.scripting.as_ref() else {
        return Ok(None);
    };

    let role = match scripting.mod_script_mode {
        ModScriptMode::Disabled => return Ok(None),
        ModScriptMode::Bootstrap => ScriptExecutionRole::ModBootstrap,
        ModScriptMode::Persistent => ScriptExecutionRole::ModPersistent,
    };

    let script_path = discovered_mod.mod_script_path().ok_or_else(|| {
        AmigoError::Message(format!(
            "mod `{}` enables scripting but has no configured mod script path",
            discovered_mod.manifest.id
        ))
    })?;
    let descriptor = build_script_descriptor(
        &discovered_mod.root_path,
        &script_path,
        mod_script_source_name(&discovered_mod.manifest.id),
        &discovered_mod.manifest.id,
        None,
        role,
    )?;
    validate_script_path(
        script_runtime,
        &descriptor.relative_script_path,
        &format!("mod script for mod `{}`", discovered_mod.manifest.id),
    )?;

    let source = fs::read_to_string(&script_path).map_err(|error| {
        AmigoError::Message(format!(
            "failed to read mod script for mod `{}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;
    script_runtime.validate_source(&source).map_err(|error| {
        AmigoError::Message(format!(
            "failed to validate mod script for mod `{}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;

    Ok(Some(PreparedScriptSource {
        executed: descriptor,
        source,
    }))
}

fn scene_script_descriptor_for_scene(
    discovered_mod: &amigo_modding::DiscoveredMod,
    scene_id: &str,
) -> AmigoResult<Option<ExecutedScript>> {
    let Some(scene_manifest) = discovered_mod.scene_by_id(scene_id) else {
        return Ok(None);
    };
    let Some(script_path) = discovered_mod.scene_script_path(scene_id) else {
        return Ok(None);
    };

    if !script_path.is_file() {
        return if scene_manifest.script.is_some() {
            Err(AmigoError::Message(format!(
                "scene `{}` for mod `{}` declares script `{}` but the file does not exist",
                scene_id,
                discovered_mod.manifest.id,
                script_path.display()
            )))
        } else {
            Ok(None)
        };
    }

    build_script_descriptor(
        &discovered_mod.root_path,
        &script_path,
        scene_script_source_name(&discovered_mod.manifest.id, scene_id),
        &discovered_mod.manifest.id,
        Some(scene_id),
        ScriptExecutionRole::Scene,
    )
    .map(Some)
}

fn prepare_scene_script_source(
    script_runtime: &ScriptRuntimeService,
    discovered_mod: &amigo_modding::DiscoveredMod,
    scene_id: &str,
) -> AmigoResult<Option<PreparedScriptSource>> {
    let Some(descriptor) = scene_script_descriptor_for_scene(discovered_mod, scene_id)? else {
        return Ok(None);
    };
    let script_path = discovered_mod.scene_script_path(scene_id).ok_or_else(|| {
        AmigoError::Message(format!(
            "scene `{scene_id}` for mod `{}` has no resolved script path",
            discovered_mod.manifest.id
        ))
    })?;
    validate_script_path(
        script_runtime,
        &descriptor.relative_script_path,
        &format!(
            "scene script for `{}` scene `{scene_id}`",
            discovered_mod.manifest.id
        ),
    )?;

    let source = fs::read_to_string(&script_path).map_err(|error| {
        AmigoError::Message(format!(
            "failed to read scene script for `{}` scene `{scene_id}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;
    script_runtime.validate_source(&source).map_err(|error| {
        AmigoError::Message(format!(
            "failed to validate scene script for `{}` scene `{scene_id}` at `{}`: {error}",
            discovered_mod.manifest.id,
            script_path.display()
        ))
    })?;

    Ok(Some(PreparedScriptSource {
        executed: descriptor,
        source,
    }))
}

fn execute_mod_scripts(runtime: &Runtime) -> AmigoResult<()> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;

    for discovered_mod in mod_catalog.mods() {
        let Some(prepared) = prepare_mod_script_source(script_runtime.as_ref(), discovered_mod)?
        else {
            continue;
        };
        script_runtime.execute_source(&prepared.executed.source_name, &prepared.source)?;
        if prepared.executed.role == ScriptExecutionRole::ModBootstrap {
            script_runtime.unload_source(&prepared.executed.source_name)?;
        }
    }

    Ok(())
}

fn persistent_mod_script_descriptors(mod_catalog: &ModCatalog) -> AmigoResult<Vec<ExecutedScript>> {
    let mut scripts = Vec::new();

    for discovered_mod in mod_catalog.mods() {
        let Some(scripting) = discovered_mod.manifest.scripting.as_ref() else {
            continue;
        };
        if scripting.mod_script_mode != ModScriptMode::Persistent {
            continue;
        }
        let Some(script_path) = discovered_mod.mod_script_path() else {
            continue;
        };
        if !script_path.is_file() {
            return Err(AmigoError::Message(format!(
                "persistent mod script for `{}` does not exist at `{}`",
                discovered_mod.manifest.id,
                script_path.display()
            )));
        }

        scripts.push(build_script_descriptor(
            &discovered_mod.root_path,
            &script_path,
            mod_script_source_name(&discovered_mod.manifest.id),
            &discovered_mod.manifest.id,
            None,
            ScriptExecutionRole::ModPersistent,
        )?);
    }

    Ok(scripts)
}

fn active_scene_script_descriptor(
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
    active_scene: Option<&str>,
) -> AmigoResult<Option<ExecutedScript>> {
    let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
        return Ok(None);
    };
    let Some(active_scene) = active_scene else {
        return Ok(None);
    };

    mod_catalog
        .mod_by_id(startup_mod)
        .map(|discovered_mod| scene_script_descriptor_for_scene(discovered_mod, active_scene))
        .transpose()
        .map(|descriptor| descriptor.flatten())
}

fn current_executed_scripts(runtime: &Runtime) -> AmigoResult<Vec<ExecutedScript>> {
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let launch_selection = required::<LaunchSelection>(runtime)?;
    let scene_service = required::<SceneService>(runtime)?;

    let mut scripts = persistent_mod_script_descriptors(mod_catalog.as_ref())?;
    if let Some(scene_script) = active_scene_script_descriptor(
        mod_catalog.as_ref(),
        launch_selection.as_ref(),
        scene_service
            .selected_scene()
            .as_ref()
            .map(SceneKey::as_str),
    )? {
        scripts.push(scene_script);
    }

    Ok(scripts)
}

fn dispatch_script_event_to_active_scripts(
    script_runtime: &ScriptRuntimeService,
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
    scene_service: &SceneService,
    event: &ScriptEvent,
) -> AmigoResult<()> {
    for script in persistent_mod_script_descriptors(mod_catalog)? {
        script_runtime.call_on_event(&script.source_name, &event.topic, &event.payload)?;
    }
    if let Some(scene_script) = active_scene_script_descriptor(
        mod_catalog,
        launch_selection,
        scene_service
            .selected_scene()
            .as_ref()
            .map(SceneKey::as_str),
    )? {
        script_runtime.call_on_event(&scene_script.source_name, &event.topic, &event.payload)?;
    }

    Ok(())
}

fn sync_active_scene_script_lifecycle(
    scene_service: &SceneService,
    script_lifecycle_state: &ScriptLifecycleState,
    script_runtime: &ScriptRuntimeService,
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
) -> AmigoResult<bool> {
    let current_scene = scene_service
        .selected_scene()
        .map(|scene| scene.as_str().to_owned());
    let previous_scene = script_lifecycle_state.active_scene();

    if current_scene == previous_scene {
        return Ok(false);
    }

    let previous_scene_script =
        active_scene_script_descriptor(mod_catalog, launch_selection, previous_scene.as_deref())?;
    let current_scene_script = if let Some(current_scene_id) = current_scene.as_deref() {
        let Some(startup_mod) = launch_selection.startup_mod.as_deref() else {
            script_lifecycle_state.set_active_scene(current_scene.clone());
            return Ok(false);
        };
        let Some(discovered_mod) = mod_catalog.mod_by_id(startup_mod) else {
            script_lifecycle_state.set_active_scene(current_scene.clone());
            return Ok(false);
        };
        prepare_scene_script_source(script_runtime, discovered_mod, current_scene_id)?
    } else {
        None
    };

    if let Some(previous_script) = &previous_scene_script {
        script_runtime.call_on_exit(&previous_script.source_name)?;
        script_runtime.unload_source(&previous_script.source_name)?;
    }

    script_lifecycle_state.set_active_scene(current_scene.clone());

    if let Some(current_script) = current_scene_script {
        script_runtime
            .execute_source(&current_script.executed.source_name, &current_script.source)?;
        script_runtime.call_on_enter(&current_script.executed.source_name)?;
        return Ok(true);
    }

    Ok(previous_scene_script.is_some())
}

fn stabilize_runtime(runtime: &Runtime) -> AmigoResult<PlaceholderBridgeSummary> {
    let mut summary = PlaceholderBridgeSummary::default();

    for _ in 0..MAX_RUNTIME_STABILIZATION_PASSES {
        merge_placeholder_bridge_summary(&mut summary, process_placeholder_bridges(runtime)?);
        process_pending_asset_loads(runtime)?;
        sync_hot_reload_watches(runtime)?;

        if queue_hot_reload_changes(runtime)? == 0 {
            let dev_console_state = required::<DevConsoleState>(runtime)?;
            summary.console_commands = dev_console_state.command_history();
            summary.console_output = dev_console_state.output_lines();
            return Ok(summary);
        }
    }

    Err(AmigoError::Message(format!(
        "runtime stabilization exceeded the maximum of {MAX_RUNTIME_STABILIZATION_PASSES} passes"
    )))
}

fn merge_placeholder_bridge_summary(
    target: &mut PlaceholderBridgeSummary,
    update: PlaceholderBridgeSummary,
) {
    target
        .processed_script_commands
        .extend(update.processed_script_commands);
    target
        .processed_audio_commands
        .extend(update.processed_audio_commands);
    target
        .processed_scene_commands
        .extend(update.processed_scene_commands);
    target
        .processed_script_events
        .extend(update.processed_script_events);
    target.console_commands = update.console_commands;
    target.console_output = update.console_output;
}

fn process_placeholder_bridges(runtime: &Runtime) -> AmigoResult<PlaceholderBridgeSummary> {
    let script_command_queue = required::<ScriptCommandQueue>(runtime)?;
    let script_event_queue = required::<ScriptEventQueue>(runtime)?;
    let script_lifecycle_state = required::<ScriptLifecycleState>(runtime)?;
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;
    let dev_console_queue = required::<DevConsoleQueue>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let scene_event_queue = required::<SceneEventQueue>(runtime)?;
    let scene_service = required::<SceneService>(runtime)?;
    let hydrated_scene_state = required::<HydratedSceneState>(runtime)?;
    let scene_transition_service = required::<SceneTransitionService>(runtime)?;
    let camera_follow_scene_service = required::<CameraFollow2dSceneService>(runtime)?;
    let parallax_scene_service = required::<Parallax2dSceneService>(runtime)?;
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let audio_command_queue = required::<AudioCommandQueue>(runtime)?;
    let audio_scene_service = required::<AudioSceneService>(runtime)?;
    let audio_state_service = required::<AudioStateService>(runtime)?;
    let audio_mixer_service = required::<AudioMixerService>(runtime)?;
    let audio_output_service = required::<AudioOutputBackendService>(runtime)?;
    let sprite_scene_service = required::<SpriteSceneService>(runtime)?;
    let text_scene_service = required::<Text2dSceneService>(runtime)?;
    let physics_scene_service = required::<Physics2dSceneService>(runtime)?;
    let tilemap_scene_service = required::<TileMap2dSceneService>(runtime)?;
    let platformer_scene_service = required::<PlatformerSceneService>(runtime)?;
    let mesh_scene_service = required::<MeshSceneService>(runtime)?;
    let text3d_scene_service = required::<Text3dSceneService>(runtime)?;
    let material_scene_service = required::<MaterialSceneService>(runtime)?;
    let ui_scene_service = required::<UiSceneService>(runtime)?;
    let ui_state_service = required::<UiStateService>(runtime)?;
    let diagnostics = required::<RuntimeDiagnostics>(runtime)?;
    let launch_selection = required::<LaunchSelection>(runtime)?;
    let mod_catalog = required::<ModCatalog>(runtime)?;

    let mut summary = PlaceholderBridgeSummary::default();

    for _ in 0..MAX_PLACEHOLDER_BRIDGE_PASSES {
        let mut made_progress = false;

        let script_commands = script_command_queue.drain();
        if !script_commands.is_empty() {
            made_progress = true;
        }
        for command in script_commands {
            summary
                .processed_script_commands
                .push(format_script_command(&command));
            handle_script_command(
                command,
                scene_command_queue.as_ref(),
                script_event_queue.as_ref(),
                dev_console_state.as_ref(),
                asset_catalog.as_ref(),
                ui_state_service.as_ref(),
                audio_command_queue.as_ref(),
                audio_scene_service.as_ref(),
                diagnostics.as_ref(),
                launch_selection.as_ref(),
            );
        }

        let console_commands = dev_console_queue.drain();
        if !console_commands.is_empty() {
            made_progress = true;
        }
        for command in console_commands {
            handle_console_command(
                command,
                scene_command_queue.as_ref(),
                script_event_queue.as_ref(),
                dev_console_state.as_ref(),
                diagnostics.as_ref(),
                asset_catalog.as_ref(),
            );
        }

        let script_events = script_event_queue.drain();
        if !script_events.is_empty() {
            made_progress = true;
        }
        for event in script_events {
            summary
                .processed_script_events
                .push(format_script_event(&event));
            for command in
                scene_transition_service.observe_script_event(&event.topic, &event.payload)
            {
                scene_command_queue.submit(command);
            }
            dispatch_script_event_to_active_scripts(
                script_runtime.as_ref(),
                mod_catalog.as_ref(),
                launch_selection.as_ref(),
                scene_service.as_ref(),
                &event,
            )?;
        }

        let audio_commands = audio_command_queue.drain();
        if !audio_commands.is_empty() {
            made_progress = true;
        }
        for command in audio_commands {
            summary
                .processed_audio_commands
                .push(format_audio_command(&command));
            process_audio_command(
                command,
                audio_state_service.as_ref(),
                dev_console_state.as_ref(),
            );
        }

        let scene_commands = scene_command_queue.drain();
        if !scene_commands.is_empty() {
            made_progress = true;
        }
        for command in scene_commands {
            summary
                .processed_scene_commands
                .push(format_scene_command(&command));
            apply_scene_command(
                runtime,
                command,
                scene_command_queue.as_ref(),
                launch_selection.as_ref(),
                hydrated_scene_state.as_ref(),
                scene_transition_service.as_ref(),
                scene_service.as_ref(),
                scene_event_queue.as_ref(),
                dev_console_state.as_ref(),
                asset_catalog.as_ref(),
                sprite_scene_service.as_ref(),
                text_scene_service.as_ref(),
                physics_scene_service.as_ref(),
                tilemap_scene_service.as_ref(),
                platformer_scene_service.as_ref(),
                camera_follow_scene_service.as_ref(),
                parallax_scene_service.as_ref(),
                mesh_scene_service.as_ref(),
                text3d_scene_service.as_ref(),
                material_scene_service.as_ref(),
                ui_scene_service.as_ref(),
                ui_state_service.as_ref(),
                audio_scene_service.as_ref(),
                audio_state_service.as_ref(),
                audio_mixer_service.as_ref(),
                audio_output_service.as_ref(),
            )?;
        }

        if scene_command_queue.pending().is_empty()
            && sync_active_scene_script_lifecycle(
                scene_service.as_ref(),
                script_lifecycle_state.as_ref(),
                script_runtime.as_ref(),
                mod_catalog.as_ref(),
                launch_selection.as_ref(),
            )?
        {
            made_progress = true;
        }

        if !made_progress {
            break;
        }
    }

    if !script_command_queue.pending().is_empty()
        || !script_event_queue.pending().is_empty()
        || !dev_console_queue.pending().is_empty()
        || !audio_command_queue.snapshot().is_empty()
        || !scene_command_queue.pending().is_empty()
    {
        return Err(AmigoError::Message(format!(
            "placeholder bridge exceeded the maximum of {MAX_PLACEHOLDER_BRIDGE_PASSES} orchestration passes"
        )));
    }

    summary.console_commands = dev_console_state.command_history();
    summary.console_output = dev_console_state.output_lines();

    Ok(summary)
}

fn handle_script_command(
    command: ScriptCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    ui_state_service: &UiStateService,
    audio_command_queue: &AudioCommandQueue,
    audio_scene_service: &AudioSceneService,
    diagnostics: &RuntimeDiagnostics,
    launch_selection: &LaunchSelection,
) {
    match (
        command.namespace.as_str(),
        command.name.as_str(),
        command.arguments.as_slice(),
    ) {
        ("scene", "select", [scene_id]) => {
            scene_command_queue.submit(SceneCommand::SelectScene {
                scene: SceneKey::new(scene_id.clone()),
            });
        }
        ("scene", "reload", []) => {
            scene_command_queue.submit(SceneCommand::ReloadActiveScene);
        }
        ("scene", "spawn", [entity_name]) => {
            scene_command_queue.submit(SceneCommand::SpawnNamedEntity {
                name: entity_name.clone(),
                transform: None,
            });
        }
        ("scene", "clear", []) => {
            scene_command_queue.submit(SceneCommand::ClearEntities);
        }
        ("2d.sprite", "spawn", [source_mod, entity_name, texture_key, width, height]) => {
            match parse_scene_vec2(width, height, "2d sprite size") {
                Ok(size) => scene_command_queue.submit(SceneCommand::QueueSprite2d {
                    command: Sprite2dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        AssetKey::new(texture_key.clone()),
                        size,
                    ),
                }),
                Err(message) => dev_console_state.write_line(message),
            }
        }
        ("2d.sprite", "spawn", [entity_name, texture_key, width, height]) => {
            match parse_scene_vec2(width, height, "2d sprite size") {
                Ok(size) => scene_command_queue.submit(SceneCommand::QueueSprite2d {
                    command: Sprite2dSceneCommand::new(
                        launch_selection.selected_mod(),
                        entity_name.clone(),
                        AssetKey::new(texture_key.clone()),
                        size,
                    ),
                }),
                Err(message) => dev_console_state.write_line(message),
            }
        }
        ("2d.text", "spawn", [source_mod, entity_name, content, font_key, width, height]) => {
            match parse_scene_vec2(width, height, "2d text bounds") {
                Ok(bounds) => scene_command_queue.submit(SceneCommand::QueueText2d {
                    command: Text2dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        content.clone(),
                        AssetKey::new(font_key.clone()),
                        bounds,
                    ),
                }),
                Err(message) => dev_console_state.write_line(message),
            }
        }
        ("2d.text", "spawn", [entity_name, content, font_key, width, height]) => {
            match parse_scene_vec2(width, height, "2d text bounds") {
                Ok(bounds) => scene_command_queue.submit(SceneCommand::QueueText2d {
                    command: Text2dSceneCommand::new(
                        launch_selection.selected_mod(),
                        entity_name.clone(),
                        content.clone(),
                        AssetKey::new(font_key.clone()),
                        bounds,
                    ),
                }),
                Err(message) => dev_console_state.write_line(message),
            }
        }
        ("3d.mesh", "spawn", [source_mod, entity_name, mesh_key]) => {
            scene_command_queue.submit(SceneCommand::QueueMesh3d {
                command: Mesh3dSceneCommand::new(
                    source_mod.clone(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                ),
            });
        }
        ("3d.mesh", "spawn", [entity_name, mesh_key]) => {
            scene_command_queue.submit(SceneCommand::QueueMesh3d {
                command: Mesh3dSceneCommand::new(
                    launch_selection.selected_mod(),
                    entity_name.clone(),
                    AssetKey::new(mesh_key.clone()),
                ),
            });
        }
        ("3d.material", "bind", [source_mod, entity_name, label, material_key]) => {
            scene_command_queue.submit(SceneCommand::QueueMaterial3d {
                command: Material3dSceneCommand::new(
                    source_mod.clone(),
                    entity_name.clone(),
                    label.clone(),
                    Some(AssetKey::new(material_key.clone())),
                ),
            });
        }
        ("3d.material", "bind", [entity_name, label, material_key]) => {
            scene_command_queue.submit(SceneCommand::QueueMaterial3d {
                command: Material3dSceneCommand::new(
                    launch_selection.selected_mod(),
                    entity_name.clone(),
                    label.clone(),
                    Some(AssetKey::new(material_key.clone())),
                ),
            });
        }
        ("3d.text", "spawn", [source_mod, entity_name, content, font_key, size]) => {
            match size.parse::<f32>() {
                Ok(size) => scene_command_queue.submit(SceneCommand::QueueText3d {
                    command: Text3dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        content.clone(),
                        AssetKey::new(font_key.clone()),
                        size,
                    ),
                }),
                Err(error) => dev_console_state.write_line(format!(
                    "failed to parse 3d text size `{size}` as f32: {error}"
                )),
            }
        }
        ("3d.text", "spawn", [entity_name, content, font_key, size]) => match size.parse::<f32>() {
            Ok(size) => scene_command_queue.submit(SceneCommand::QueueText3d {
                command: Text3dSceneCommand::new(
                    launch_selection.selected_mod(),
                    entity_name.clone(),
                    content.clone(),
                    AssetKey::new(font_key.clone()),
                    size,
                ),
            }),
            Err(error) => dev_console_state.write_line(format!(
                "failed to parse 3d text size `{size}` as f32: {error}"
            )),
        },
        ("asset", "reload", [asset_key]) => {
            request_asset_reload(
                asset_catalog,
                asset_key,
                AssetLoadPriority::Immediate,
                dev_console_state,
            );
            script_event_queue.publish(ScriptEvent::new(
                "asset.reload-requested",
                vec![asset_key.clone()],
            ));
        }
        ("audio", "play", [clip_name]) => {
            let asset_key = resolve_mod_audio_asset_key(launch_selection, clip_name);
            register_audio_clip_reference(
                asset_catalog,
                audio_scene_service,
                &asset_key,
                AudioPlaybackMode::OneShot,
            );
            audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            dev_console_state.write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
        }
        ("audio", "play-asset", [asset_key]) => {
            let asset_key = AssetKey::new(asset_key.clone());
            register_audio_clip_reference(
                asset_catalog,
                audio_scene_service,
                &asset_key,
                AudioPlaybackMode::OneShot,
            );
            audio_command_queue.push(AudioCommand::PlayOnce {
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            dev_console_state.write_line(format!("queued audio one-shot `{}`", asset_key.as_str()));
        }
        ("audio", "start-realtime", [source]) => {
            let asset_key = resolve_mod_audio_asset_key(launch_selection, source);
            register_audio_clip_reference(
                asset_catalog,
                audio_scene_service,
                &asset_key,
                AudioPlaybackMode::Looping,
            );
            audio_command_queue.push(AudioCommand::StartSource {
                source: AudioSourceId::new(source.clone()),
                clip: AudioClipKey::new(asset_key.as_str().to_owned()),
            });
            dev_console_state.write_line(format!(
                "queued realtime audio source `{}` using `{}`",
                source,
                asset_key.as_str()
            ));
        }
        ("audio", "stop", [source]) => {
            audio_command_queue.push(AudioCommand::StopSource {
                source: AudioSourceId::new(source.clone()),
            });
            dev_console_state.write_line(format!("queued stop for audio source `{source}`"));
        }
        ("audio", "set-param", [source, param, value]) => match value.parse::<f32>() {
            Ok(value) => {
                audio_command_queue.push(AudioCommand::SetParam {
                    source: AudioSourceId::new(source.clone()),
                    param: param.clone(),
                    value,
                });
            }
            Err(error) => dev_console_state.write_line(format!(
                "failed to parse audio param value `{value}` as f32: {error}"
            )),
        },
        ("audio", "set-volume", [bus, value]) => match value.parse::<f32>() {
            Ok(value) if bus == "master" => {
                audio_command_queue.push(AudioCommand::SetMasterVolume { value });
                dev_console_state.write_line(format!(
                    "queued master audio volume = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
            Ok(value) => {
                audio_command_queue.push(AudioCommand::SetVolume {
                    bus: bus.clone(),
                    value,
                });
                dev_console_state.write_line(format!(
                    "queued audio bus volume `{bus}` = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
            Err(error) => dev_console_state.write_line(format!(
                "failed to parse audio volume `{value}` as f32: {error}"
            )),
        },
        ("ui", "set-text", [path, value]) => {
            if ui_state_service.set_text(path.clone(), value.clone()) {
                dev_console_state.write_line(format!("updated ui text override `{path}`"));
            }
        }
        ("ui", "set-value", [path, value]) => match value.parse::<f32>() {
            Ok(value) => {
                if ui_state_service.set_value(path.clone(), value) {
                    dev_console_state.write_line(format!(
                        "updated ui value override `{path}` to {}",
                        value.clamp(0.0, 1.0)
                    ));
                }
            }
            Err(error) => dev_console_state.write_line(format!(
                "failed to parse ui value `{value}` as f32: {error}"
            )),
        },
        ("ui", "show", [path]) => {
            if ui_state_service.show(path.clone()) {
                dev_console_state.write_line(format!("showed ui path `{path}`"));
            }
        }
        ("ui", "hide", [path]) => {
            if ui_state_service.hide(path.clone()) {
                dev_console_state.write_line(format!("hid ui path `{path}`"));
            }
        }
        ("ui", "enable", [path]) => {
            if ui_state_service.enable(path.clone()) {
                dev_console_state.write_line(format!("enabled ui path `{path}`"));
            }
        }
        ("ui", "disable", [path]) => {
            if ui_state_service.disable(path.clone()) {
                dev_console_state.write_line(format!("disabled ui path `{path}`"));
            }
        }
        ("debug", "log", [line]) => {
            dev_console_state.write_line(format!("script: {line}"));
        }
        ("debug", "warn", [line]) => {
            dev_console_state.write_line(format!("script warning: {line}"));
        }
        ("dev-shell", "refresh-diagnostics", [target_mod]) => {
            dev_console_state.write_line(format!(
                "diagnostics refreshed for mod={} scene={} window={} input={} render={} script={}",
                target_mod,
                launch_selection.selected_scene(),
                diagnostics.window_backend,
                diagnostics.input_backend,
                diagnostics.render_backend,
                diagnostics.script_backend
            ));
            script_event_queue.publish(ScriptEvent::new(
                "dev-shell.diagnostics-refreshed",
                vec![target_mod.clone()],
            ));
        }
        _ => {
            dev_console_state.write_line(format!(
                "unhandled placeholder script command: {}",
                format_script_command(&command)
            ));
        }
    }
}

fn parse_scene_vec2(width: &str, height: &str, label: &str) -> Result<Vec2, String> {
    let width = width
        .parse::<f32>()
        .map_err(|error| format!("failed to parse {label} width `{width}` as f32: {error}"))?;
    let height = height
        .parse::<f32>()
        .map_err(|error| format!("failed to parse {label} height `{height}` as f32: {error}"))?;

    Ok(Vec2::new(width, height))
}

fn register_mod_asset_reference(
    asset_catalog: &AssetCatalog,
    source_mod: &str,
    asset_key: &AssetKey,
    domain_scope: &str,
    domain_tag: &str,
) {
    asset_catalog.register_manifest(AssetManifest {
        key: asset_key.clone(),
        source: AssetSourceKind::Mod(source_mod.to_owned()),
        tags: vec![
            "phase3".to_owned(),
            domain_scope.to_owned(),
            domain_tag.to_owned(),
        ],
    });
    asset_catalog.request_load(AssetLoadRequest::new(
        asset_key.clone(),
        AssetLoadPriority::Interactive,
    ));
}

fn register_audio_clip_reference(
    asset_catalog: &AssetCatalog,
    audio_scene_service: &AudioSceneService,
    asset_key: &AssetKey,
    mode: AudioPlaybackMode,
) {
    let source_mod = asset_key
        .as_str()
        .split('/')
        .next()
        .unwrap_or_default()
        .to_owned();
    if source_mod.is_empty() {
        return;
    }

    register_mod_asset_reference(asset_catalog, &source_mod, asset_key, "audio", "generated");
    audio_scene_service.register_clip(AudioClip {
        key: AudioClipKey::new(asset_key.as_str().to_owned()),
        mode,
    });
}

fn resolve_mod_audio_asset_key(launch_selection: &LaunchSelection, clip_name: &str) -> AssetKey {
    if clip_name.contains('/') {
        AssetKey::new(clip_name.to_owned())
    } else {
        AssetKey::new(format!(
            "{}/audio/{}",
            launch_selection.selected_mod(),
            clip_name
        ))
    }
}

fn process_audio_command(
    command: AudioCommand,
    audio_state_service: &AudioStateService,
    dev_console_state: &DevConsoleState,
) {
    audio_state_service.record_processed_command(command.clone());
    audio_state_service.queue_runtime_command(command.clone());

    match command {
        AudioCommand::PlayOnce { clip } => {
            dev_console_state.write_line(format!("audio play once `{}`", clip.as_str()));
        }
        AudioCommand::StartSource { source, clip } => {
            audio_state_service.start_source(source.clone(), clip.clone());
            dev_console_state.write_line(format!(
                "audio start source `{}` -> `{}`",
                source.as_str(),
                clip.as_str()
            ));
        }
        AudioCommand::StopSource { source } => {
            let _ = audio_state_service.stop_source(source.as_str());
            dev_console_state.write_line(format!("audio stop source `{}`", source.as_str()));
        }
        AudioCommand::SetParam {
            source,
            param,
            value,
        } => {
            if audio_state_service.set_param(source.as_str(), param.clone(), value) {
                dev_console_state.write_line(format!(
                    "audio set param `{}` for `{}` = {}",
                    param,
                    source.as_str(),
                    value
                ));
            }
        }
        AudioCommand::SetVolume { bus, value } => {
            if audio_state_service.set_volume(&bus, value) {
                dev_console_state.write_line(format!(
                    "audio set bus volume `{bus}` = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
        }
        AudioCommand::SetMasterVolume { value } => {
            if audio_state_service.set_master_volume(value) {
                dev_console_state.write_line(format!(
                    "audio set master volume = {}",
                    value.clamp(0.0, 1.0)
                ));
            }
        }
    }
}

fn register_ui_font_asset_references(
    asset_catalog: &AssetCatalog,
    source_mod: &str,
    node: &SceneUiNode,
) {
    match &node.kind {
        SceneUiNodeKind::Text { font, .. } | SceneUiNodeKind::Button { font, .. } => {
            if let Some(font) = font.as_ref() {
                register_mod_asset_reference(asset_catalog, source_mod, font, "ui", "font");
            }
        }
        SceneUiNodeKind::Panel
        | SceneUiNodeKind::Row
        | SceneUiNodeKind::Column
        | SceneUiNodeKind::Stack
        | SceneUiNodeKind::ProgressBar { .. }
        | SceneUiNodeKind::Spacer => {}
    }

    for child in &node.children {
        register_ui_font_asset_references(asset_catalog, source_mod, child);
    }
}

fn convert_scene_ui_document(document: &SceneUiDocument) -> RuntimeUiDocument {
    RuntimeUiDocument {
        target: convert_scene_ui_target(&document.target),
        root: convert_scene_ui_node(&document.root),
    }
}

fn convert_scene_ui_target(target: &SceneUiTarget) -> RuntimeUiTarget {
    match target {
        SceneUiTarget::ScreenSpace { layer } => RuntimeUiTarget::ScreenSpace {
            layer: convert_scene_ui_layer(*layer),
        },
    }
}

fn convert_scene_ui_layer(layer: SceneUiLayer) -> RuntimeUiLayer {
    match layer {
        SceneUiLayer::Background => RuntimeUiLayer::Background,
        SceneUiLayer::Hud => RuntimeUiLayer::Hud,
        SceneUiLayer::Menu => RuntimeUiLayer::Menu,
        SceneUiLayer::Debug => RuntimeUiLayer::Debug,
    }
}

fn convert_scene_ui_node(node: &SceneUiNode) -> RuntimeUiNode {
    RuntimeUiNode {
        id: node.id.clone(),
        kind: convert_scene_ui_node_kind(&node.kind),
        style: convert_scene_ui_style(&node.style),
        events: amigo_ui::UiEvents {
            on_click: node.on_click.as_ref().map(convert_scene_ui_event_binding),
        },
        children: node.children.iter().map(convert_scene_ui_node).collect(),
    }
}

fn convert_scene_ui_node_kind(kind: &SceneUiNodeKind) -> RuntimeUiNodeKind {
    match kind {
        SceneUiNodeKind::Panel => RuntimeUiNodeKind::Panel,
        SceneUiNodeKind::Row => RuntimeUiNodeKind::Row,
        SceneUiNodeKind::Column => RuntimeUiNodeKind::Column,
        SceneUiNodeKind::Stack => RuntimeUiNodeKind::Stack,
        SceneUiNodeKind::Text { content, font } => RuntimeUiNodeKind::Text {
            content: content.clone(),
            font: font.clone(),
        },
        SceneUiNodeKind::Button { text, font } => RuntimeUiNodeKind::Button {
            text: text.clone(),
            font: font.clone(),
        },
        SceneUiNodeKind::ProgressBar { value } => RuntimeUiNodeKind::ProgressBar { value: *value },
        SceneUiNodeKind::Spacer => RuntimeUiNodeKind::Spacer,
    }
}

fn convert_scene_ui_style(style: &SceneUiStyle) -> RuntimeUiStyle {
    RuntimeUiStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: style.background,
        color: style.color,
        border_color: style.border_color,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
    }
}

fn convert_scene_ui_event_binding(binding: &SceneUiEventBinding) -> UiEventBinding {
    UiEventBinding::new(binding.event.clone(), binding.payload.clone())
}

fn resolve_ui_overlay_documents(
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
) -> Vec<ResolvedUiOverlayDocument> {
    let snapshot = ui_state_service.snapshot();
    let mut documents = ui_scene_service
        .commands()
        .into_iter()
        .filter_map(|command| {
            resolve_ui_overlay_document(&command.entity_name, &command.document, &snapshot)
        })
        .collect::<Vec<_>>();
    documents.sort_by_key(|document| document.overlay.layer);
    documents
}

fn resolve_ui_overlay_document(
    entity_name: &str,
    document: &RuntimeUiDocument,
    snapshot: &UiStateSnapshot,
) -> Option<ResolvedUiOverlayDocument> {
    let root_segment = document
        .root
        .id
        .clone()
        .unwrap_or_else(|| "root".to_owned());
    let root_path = format!("{entity_name}.{root_segment}");
    let mut click_bindings = BTreeMap::new();
    let root = resolve_ui_overlay_node(&document.root, &root_path, snapshot, &mut click_bindings)?;

    Some(ResolvedUiOverlayDocument {
        overlay: UiOverlayDocument {
            entity_name: entity_name.to_owned(),
            layer: resolve_ui_overlay_layer(&document.target),
            root,
        },
        click_bindings,
    })
}

fn resolve_ui_overlay_layer(target: &RuntimeUiTarget) -> UiOverlayLayer {
    match target.layer() {
        RuntimeUiLayer::Background => UiOverlayLayer::Background,
        RuntimeUiLayer::Hud => UiOverlayLayer::Hud,
        RuntimeUiLayer::Menu => UiOverlayLayer::Menu,
        RuntimeUiLayer::Debug => UiOverlayLayer::Debug,
    }
}

fn resolve_ui_overlay_node(
    node: &RuntimeUiNode,
    path: &str,
    snapshot: &UiStateSnapshot,
    click_bindings: &mut BTreeMap<String, UiEventBinding>,
) -> Option<UiOverlayNode> {
    if snapshot
        .visibility_overrides
        .get(path)
        .copied()
        .unwrap_or(true)
        == false
    {
        return None;
    }

    let kind = match &node.kind {
        RuntimeUiNodeKind::Panel => UiOverlayNodeKind::Panel,
        RuntimeUiNodeKind::Row => UiOverlayNodeKind::Row,
        RuntimeUiNodeKind::Column => UiOverlayNodeKind::Column,
        RuntimeUiNodeKind::Stack => UiOverlayNodeKind::Stack,
        RuntimeUiNodeKind::Text { content, font } => UiOverlayNodeKind::Text {
            content: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| content.clone()),
            font: font.clone(),
        },
        RuntimeUiNodeKind::Button { text, font } => UiOverlayNodeKind::Button {
            text: snapshot
                .text_overrides
                .get(path)
                .cloned()
                .unwrap_or_else(|| text.clone()),
            font: font.clone(),
        },
        RuntimeUiNodeKind::ProgressBar { value } => UiOverlayNodeKind::ProgressBar {
            value: snapshot
                .value_overrides
                .get(path)
                .copied()
                .unwrap_or(*value)
                .clamp(0.0, 1.0),
        },
        RuntimeUiNodeKind::Spacer => UiOverlayNodeKind::Spacer,
    };

    let mut children = Vec::new();
    for (index, child) in node.children.iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", runtime_ui_node_kind_slug(&child.kind)));
        let child_path = format!("{path}.{segment}");
        if let Some(child) = resolve_ui_overlay_node(child, &child_path, snapshot, click_bindings) {
            children.push(child);
        }
    }

    if let Some(binding) = node.events.on_click.as_ref() {
        click_bindings.insert(path.to_owned(), binding.clone());
    }

    Some(UiOverlayNode {
        id: node.id.clone(),
        kind,
        style: resolve_ui_overlay_style(&node.style),
        children,
    })
}

fn runtime_ui_node_kind_slug(kind: &RuntimeUiNodeKind) -> &'static str {
    match kind {
        RuntimeUiNodeKind::Panel => "panel",
        RuntimeUiNodeKind::Row => "row",
        RuntimeUiNodeKind::Column => "column",
        RuntimeUiNodeKind::Stack => "stack",
        RuntimeUiNodeKind::Text { .. } => "text",
        RuntimeUiNodeKind::Button { .. } => "button",
        RuntimeUiNodeKind::ProgressBar { .. } => "progress-bar",
        RuntimeUiNodeKind::Spacer => "spacer",
    }
}

fn resolve_ui_overlay_style(style: &RuntimeUiStyle) -> UiOverlayStyle {
    UiOverlayStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: style.background,
        color: style.color,
        border_color: style.border_color,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
    }
}

fn hit_test_ui_layout(node: &OverlayUiLayoutNode, x: f32, y: f32) -> Option<String> {
    if x < node.rect.x
        || y < node.rect.y
        || x > node.rect.x + node.rect.width
        || y > node.rect.y + node.rect.height
    {
        return None;
    }

    for child in node.children.iter().rev() {
        if let Some(path) = hit_test_ui_layout(child, x, y) {
            return Some(path);
        }
    }

    Some(node.path.clone())
}

fn tick_platformer_world(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let physics_scene_service = required::<Physics2dSceneService>(runtime)?;
    let platformer_scene_service = required::<PlatformerSceneService>(runtime)?;
    let script_event_queue = required::<ScriptEventQueue>(runtime)?;

    let static_colliders = physics_scene_service.static_colliders();
    let triggers = physics_scene_service.triggers();

    for body_command in physics_scene_service.kinematic_bodies() {
        let entity_name = body_command.entity_name.clone();
        let Some(mut transform) = scene_service.transform_of(&entity_name) else {
            continue;
        };
        let Some(collider_command) = physics_scene_service.aabb_collider(&entity_name) else {
            continue;
        };

        let mut body_state = physics_scene_service
            .body_state(&entity_name)
            .unwrap_or_default();
        let controller_command = platformer_scene_service.controller(&entity_name);
        let previous_controller_state = platformer_scene_service
            .state(&entity_name)
            .unwrap_or_else(|| PlatformerControllerState {
                grounded: body_state.grounded.grounded,
                facing: PlatformerFacing::Right,
                animation: PlatformerAnimationState::Idle,
                velocity: body_state.velocity,
            });

        let mut facing = previous_controller_state.facing;
        if let Some(controller_command) = controller_command.as_ref() {
            let motor = platformer_scene_service
                .motor(&entity_name)
                .unwrap_or_default();
            let drive = drive_controller(
                &controller_command.controller.params,
                &body_state,
                &motor,
                facing,
                delta_seconds,
            );
            body_state.velocity = drive.velocity;
            facing = drive.facing;

            if drive.jumped {
                script_event_queue
                    .publish(ScriptEvent::new("player.jump", vec![entity_name.clone()]));
            }
        } else {
            body_state.velocity.y += -980.0 * body_command.body.gravity_scale * delta_seconds;
            if body_command.body.terminal_velocity > 0.0 {
                body_state.velocity.y = body_state
                    .velocity
                    .y
                    .max(-body_command.body.terminal_velocity.abs());
            }
        }

        let translation = Vec2::new(transform.translation.x, transform.translation.y);
        let step = move_and_collide(
            translation,
            &collider_command.collider,
            body_state.velocity,
            delta_seconds,
            &static_colliders,
        );

        body_state.velocity = step.velocity;
        body_state.grounded = step.grounded.clone();
        transform.translation.x = step.translation.x;
        transform.translation.y = step.translation.y;
        let _ = scene_service.set_transform(&entity_name, transform);
        let _ = physics_scene_service.sync_body_state(&entity_name, body_state.clone());

        if controller_command.is_some() {
            let _ = platformer_scene_service.sync_state(
                &entity_name,
                PlatformerControllerState {
                    grounded: step.grounded.grounded,
                    facing,
                    animation: animation_state_for(step.velocity, step.grounded.grounded),
                    velocity: step.velocity,
                },
            );
            platformer_scene_service.clear_motor(&entity_name);
        }

        for trigger in &triggers {
            let overlapping =
                overlaps_trigger(step.translation, &collider_command.collider, trigger);
            let was_active =
                physics_scene_service.is_trigger_overlap_active(&trigger.entity_name, &entity_name);

            if overlapping && !was_active {
                physics_scene_service.set_trigger_overlap_active(
                    &trigger.entity_name,
                    &entity_name,
                    true,
                );
                if let Some(topic) = trigger.trigger.topic.as_ref() {
                    script_event_queue.publish(ScriptEvent::new(
                        topic.clone(),
                        vec![trigger.entity_name.clone(), entity_name.clone()],
                    ));
                }
            } else if !overlapping && was_active {
                physics_scene_service.set_trigger_overlap_active(
                    &trigger.entity_name,
                    &entity_name,
                    false,
                );
            }
        }
    }

    Ok(())
}

fn tick_camera_follow_world(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let camera_follow_scene_service = required::<CameraFollow2dSceneService>(runtime)?;

    for follow in camera_follow_scene_service.commands() {
        let Some(target_transform) = scene_service.transform_of(&follow.target) else {
            continue;
        };
        let Some(mut camera_transform) = scene_service.transform_of(&follow.entity_name) else {
            continue;
        };

        let desired_x = target_transform.translation.x + follow.offset.x;
        let desired_y = target_transform.translation.y + follow.offset.y;
        let alpha = follow.lerp.clamp(0.0, 1.0);

        if alpha >= 1.0 {
            camera_transform.translation.x = desired_x;
            camera_transform.translation.y = desired_y;
        } else {
            camera_transform.translation.x += (desired_x - camera_transform.translation.x) * alpha;
            camera_transform.translation.y += (desired_y - camera_transform.translation.y) * alpha;
        }

        let _ = scene_service.set_transform(&follow.entity_name, camera_transform);
    }

    Ok(())
}

fn tick_parallax_world(runtime: &Runtime) -> AmigoResult<()> {
    let scene_service = required::<SceneService>(runtime)?;
    let parallax_scene_service = required::<Parallax2dSceneService>(runtime)?;

    for parallax in parallax_scene_service.commands() {
        let Some(camera_transform) = scene_service.transform_of(&parallax.camera) else {
            continue;
        };
        let Some(mut entity_transform) = scene_service.transform_of(&parallax.entity_name) else {
            continue;
        };

        let camera_translation = Vec2::new(
            camera_transform.translation.x,
            camera_transform.translation.y,
        );
        let camera_origin = parallax.camera_origin.unwrap_or(camera_translation);
        if parallax.camera_origin.is_none() {
            let _ =
                parallax_scene_service.set_camera_origin(&parallax.entity_name, camera_translation);
        }

        let factor_x = parallax.factor.x.clamp(0.0, 1.0);
        let factor_y = parallax.factor.y.clamp(0.0, 1.0);
        entity_transform.translation.x =
            parallax.anchor.x + (camera_translation.x - camera_origin.x) * (1.0 - factor_x);
        entity_transform.translation.y =
            parallax.anchor.y + (camera_translation.y - camera_origin.y) * (1.0 - factor_y);

        let _ = scene_service.set_transform(&parallax.entity_name, entity_transform);
    }

    Ok(())
}

fn tick_audio_runtime(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let audio_state_service = required::<AudioStateService>(runtime)?;
    let audio_mixer_service = required::<AudioMixerService>(runtime)?;
    let audio_output_service = required::<AudioOutputBackendService>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;

    let prepared_assets = asset_catalog
        .prepared_assets()
        .into_iter()
        .map(|asset| (asset.key.as_str().to_owned(), asset))
        .collect::<BTreeMap<_, _>>();
    let playing_sources = audio_state_service.playing_sources();
    let source_params = audio_state_service.source_params();
    let frame_sample_count = ((44_100.0 * delta_seconds.max(0.0)).round() as usize).max(1);

    for command in audio_state_service.drain_runtime_commands() {
        if let AudioCommand::PlayOnce { clip } = command {
            if let Some(prepared_asset) = prepared_assets.get(clip.as_str()) {
                if let Err(error) = audio_mixer_service
                    .queue_generated_one_shot(clip.as_str().to_owned(), prepared_asset)
                {
                    dev_console_state.write_line(format!(
                        "audio runtime queue failed for `{}`: {error}",
                        clip.as_str()
                    ));
                }
            }
        }
    }

    if let Some(frame) = audio_mixer_service.tick_generated_audio(
        &prepared_assets,
        &playing_sources,
        &source_params,
        audio_state_service.master_volume(),
        frame_sample_count,
    ) {
        audio_output_service.enqueue_mix_frame(&frame);
    }

    Ok(())
}

fn process_pending_asset_loads(runtime: &Runtime) -> AmigoResult<()> {
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let sprite_scene_service = required::<SpriteSceneService>(runtime)?;
    let tilemap_scene_service = required::<TileMap2dSceneService>(runtime)?;

    for request in asset_catalog.drain_pending_loads() {
        let Some(manifest) = asset_catalog.manifest(&request.key) else {
            asset_catalog.mark_failed(
                request.key.clone(),
                "asset manifest missing for pending load request",
            );
            continue;
        };

        match resolve_asset_request_path(mod_catalog.as_ref(), &manifest.source, &request.key) {
            Ok(resolved_path) => match fs::metadata(&resolved_path) {
                Ok(metadata) if metadata.is_file() => {
                    let loaded_asset = amigo_assets::LoadedAsset {
                        key: request.key.clone(),
                        source: manifest.source.clone(),
                        resolved_path: resolved_path.clone(),
                        byte_len: metadata.len(),
                    };
                    asset_catalog.mark_loaded(loaded_asset.clone());
                    dev_console_state.write_line(format!(
                        "resolved asset `{}` to `{}` ({} bytes)",
                        request.key.as_str(),
                        resolved_path.display(),
                        metadata.len()
                    ));
                    if let Err(reason) = prepare_loaded_asset(
                        asset_catalog.as_ref(),
                        &loaded_asset,
                        dev_console_state.as_ref(),
                    ) {
                        asset_catalog.mark_failed(request.key.clone(), reason.clone());
                        dev_console_state.write_line(format!(
                            "asset prepare failed for `{}`: {reason}",
                            request.key.as_str()
                        ));
                    } else {
                        sync_sprite_sheet_metadata(
                            asset_catalog.as_ref(),
                            sprite_scene_service.as_ref(),
                            &loaded_asset.key,
                        );
                        sync_tile_ruleset_metadata(
                            asset_catalog.as_ref(),
                            tilemap_scene_service.as_ref(),
                            &loaded_asset.key,
                        );
                    }
                }
                Ok(_) => {
                    let reason = format!(
                        "resolved asset path `{}` is not a file",
                        resolved_path.display()
                    );
                    asset_catalog.mark_failed(request.key.clone(), reason.clone());
                    dev_console_state.write_line(format!(
                        "asset load failed for `{}`: {reason}",
                        request.key.as_str()
                    ));
                }
                Err(error) => {
                    let reason = format!(
                        "failed to access resolved asset path `{}`: {error}",
                        resolved_path.display()
                    );
                    asset_catalog.mark_failed(request.key.clone(), reason.clone());
                    dev_console_state.write_line(format!(
                        "asset load failed for `{}`: {reason}",
                        request.key.as_str()
                    ));
                }
            },
            Err(reason) => {
                asset_catalog.mark_failed(request.key.clone(), reason.clone());
                dev_console_state.write_line(format!(
                    "asset load failed for `{}`: {reason}",
                    request.key.as_str()
                ));
            }
        }
    }

    Ok(())
}

fn sync_sprite_sheet_metadata(
    asset_catalog: &AssetCatalog,
    sprite_scene_service: &SpriteSceneService,
    asset_key: &AssetKey,
) {
    let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
        return;
    };
    let Some(sheet) = infer_sprite_sheet_from_prepared_asset(&prepared) else {
        return;
    };
    sprite_scene_service.sync_sheet_for_texture(asset_key, sheet);
}

fn sync_tile_ruleset_metadata(
    asset_catalog: &AssetCatalog,
    tilemap_scene_service: &TileMap2dSceneService,
    asset_key: &AssetKey,
) {
    let Some(prepared) = asset_catalog.prepared_asset(asset_key) else {
        return;
    };
    let Some(ruleset) = infer_tile_ruleset_from_prepared_asset(&prepared) else {
        return;
    };
    tilemap_scene_service.sync_ruleset_for_asset(asset_key, &ruleset);
}

fn resolve_sprite_sheet_for_command(
    asset_catalog: &AssetCatalog,
    command: &Sprite2dSceneCommand,
) -> Option<SpriteSheet> {
    let explicit_sheet = command.sheet.as_ref().map(|sheet| SpriteSheet {
        columns: sheet.columns,
        rows: sheet.rows,
        frame_count: sheet.frame_count,
        frame_size: sheet.frame_size,
        fps: sheet.fps,
        looping: sheet.looping,
    });

    let base_sheet = explicit_sheet.or_else(|| {
        asset_catalog
            .prepared_asset(&command.texture)
            .and_then(|prepared| infer_sprite_sheet_from_prepared_asset(&prepared))
    })?;

    Some(apply_sprite_animation_override(
        base_sheet,
        command.animation.as_ref(),
    ))
}

fn apply_sprite_animation_override(
    mut sheet: SpriteSheet,
    animation: Option<&amigo_scene::SpriteAnimation2dSceneOverride>,
) -> SpriteSheet {
    let Some(animation) = animation else {
        return sheet;
    };

    if let Some(fps) = animation.fps {
        sheet.fps = fps.max(0.0);
    }
    if let Some(looping) = animation.looping {
        sheet.looping = looping;
    }
    sheet
}

fn sync_hot_reload_watches(runtime: &Runtime) -> AmigoResult<()> {
    let hot_reload = required::<HotReloadService>(runtime)?;
    let mod_catalog = required::<ModCatalog>(runtime)?;
    let asset_catalog = required::<AssetCatalog>(runtime)?;

    let scene_watch = current_loaded_scene_document_summary(runtime)?.and_then(|document| {
        mod_catalog
            .mod_by_id(&document.source_mod)
            .map(|discovered_mod| SceneDocumentWatch {
                source_mod: document.source_mod,
                scene_id: document.scene_id,
                path: discovered_mod.root_path.join(document.relative_path),
            })
    });
    hot_reload.sync_scene_document(scene_watch);

    let asset_watches = asset_catalog
        .manifests()
        .into_iter()
        .filter_map(|manifest| {
            resolve_asset_request_path(mod_catalog.as_ref(), &manifest.source, &manifest.key)
                .ok()
                .map(|path| AssetWatch {
                    asset_key: manifest.key.as_str().to_owned(),
                    path,
                })
        })
        .collect::<Vec<_>>();
    hot_reload.sync_assets(asset_watches);

    if let Some(file_watch_service) = runtime.resolve::<FileWatchService>() {
        let watched_paths = hot_reload
            .watched_targets()
            .into_iter()
            .map(|watch| watch.path)
            .collect::<Vec<_>>();
        file_watch_service.sync_paths(&watched_paths)?;
    }

    Ok(())
}

fn queue_hot_reload_changes(runtime: &Runtime) -> AmigoResult<usize> {
    let hot_reload = required::<HotReloadService>(runtime)?;
    let scene_command_queue = required::<SceneCommandQueue>(runtime)?;
    let script_event_queue = required::<ScriptEventQueue>(runtime)?;
    let dev_console_state = required::<DevConsoleState>(runtime)?;
    let asset_catalog = required::<AssetCatalog>(runtime)?;
    let native_changes = runtime
        .resolve::<FileWatchService>()
        .map(|file_watch_service| {
            let changed_paths = file_watch_service
                .drain_events()
                .into_iter()
                .map(|event| event.path)
                .collect::<Vec<_>>();
            hot_reload.changes_for_paths(&changed_paths)
        })
        .unwrap_or_default();
    let changes = if native_changes.is_empty() {
        hot_reload.poll_changes()
    } else {
        native_changes
    };
    for change in &changes {
        match &change.watch.kind {
            HotReloadWatchKind::SceneDocument {
                source_mod,
                scene_id,
            } => {
                dev_console_state.write_line(format!(
                    "detected scene document change for `{source_mod}:{scene_id}` at `{}`",
                    change.watch.path.display()
                ));
                script_event_queue.publish(ScriptEvent::new(
                    "hot-reload.scene-document-changed",
                    vec![
                        source_mod.clone(),
                        scene_id.clone(),
                        change.watch.path.display().to_string(),
                    ],
                ));
                scene_command_queue.submit(SceneCommand::ReloadActiveScene);
            }
            HotReloadWatchKind::Asset { asset_key } => {
                dev_console_state.write_line(format!(
                    "detected asset change for `{asset_key}` at `{}`",
                    change.watch.path.display()
                ));
                script_event_queue.publish(ScriptEvent::new(
                    "hot-reload.asset-changed",
                    vec![asset_key.clone(), change.watch.path.display().to_string()],
                ));
                request_asset_reload(
                    asset_catalog.as_ref(),
                    asset_key,
                    AssetLoadPriority::Immediate,
                    dev_console_state.as_ref(),
                );
            }
        }
    }

    Ok(changes.len())
}

fn prepare_loaded_asset(
    asset_catalog: &AssetCatalog,
    loaded_asset: &amigo_assets::LoadedAsset,
    dev_console_state: &DevConsoleState,
) -> Result<(), String> {
    let contents = fs::read_to_string(&loaded_asset.resolved_path).map_err(|error| {
        format!(
            "failed to read loaded asset path `{}`: {error}",
            loaded_asset.resolved_path.display()
        )
    })?;
    let prepared = prepare_asset_from_contents(loaded_asset, &contents).map_err(|error| {
        format!(
            "failed to prepare asset `{}` from `{}`: {error}",
            loaded_asset.key.as_str(),
            loaded_asset.resolved_path.display()
        )
    })?;
    let kind = prepared.kind.as_str().to_owned();
    asset_catalog.mark_prepared(prepared);
    dev_console_state.write_line(format!(
        "prepared asset `{}` as `{kind}`",
        loaded_asset.key.as_str()
    ));

    Ok(())
}

fn infer_sprite_sheet_from_prepared_asset(
    prepared: &amigo_assets::PreparedAsset,
) -> Option<SpriteSheet> {
    if !matches!(
        prepared.kind,
        amigo_assets::PreparedAssetKind::SpriteSheet2d
    ) {
        return None;
    }

    let columns = prepared
        .metadata
        .get("columns")?
        .parse::<u32>()
        .ok()?
        .max(1);
    let rows = prepared.metadata.get("rows")?.parse::<u32>().ok()?.max(1);
    let frame_width = prepared.metadata.get("frame_size.x")?.parse::<f32>().ok()?;
    let frame_height = prepared.metadata.get("frame_size.y")?.parse::<f32>().ok()?;
    let fps = prepared
        .metadata
        .get("fps")
        .and_then(|value| value.parse::<f32>().ok())
        .or_else(|| first_animation_f32(prepared, "fps"))
        .unwrap_or(0.0);
    let looping = prepared
        .metadata
        .get("looping")
        .and_then(|value| value.parse::<bool>().ok())
        .or_else(|| first_animation_bool(prepared, "looping"))
        .unwrap_or(true);

    Some(SpriteSheet {
        columns,
        rows,
        frame_count: prepared
            .metadata
            .get("frame_count")
            .and_then(|value| value.parse::<u32>().ok())
            .unwrap_or(columns.saturating_mul(rows))
            .max(1),
        frame_size: Vec2::new(frame_width, frame_height),
        fps,
        looping,
    })
}

fn infer_tile_ruleset_from_prepared_asset(
    prepared: &amigo_assets::PreparedAsset,
) -> Option<TileRuleSet2d> {
    if !matches!(
        prepared.kind,
        amigo_assets::PreparedAssetKind::TileRuleSet2d
    ) {
        return None;
    }

    let mut terrains = BTreeMap::<String, TileTerrainRule2d>::new();

    for key in prepared.metadata.keys() {
        let Some(terrain_name) = key.strip_prefix("terrains.") else {
            continue;
        };
        let Some((terrain_name, field_path)) = terrain_name.split_once('.') else {
            continue;
        };

        let terrain =
            terrains
                .entry(terrain_name.to_owned())
                .or_insert_with(|| TileTerrainRule2d {
                    name: terrain_name.to_owned(),
                    symbol: '\0',
                    collision: TileCollisionKind2d::None,
                    variants: TileVariantSet2d::default(),
                });

        match field_path {
            "symbol" => {
                if let Some(symbol) = prepared
                    .metadata
                    .get(key)
                    .and_then(|value| value.chars().next())
                {
                    terrain.symbol = symbol;
                }
            }
            "collision" => {
                terrain.collision = match prepared.metadata.get(key).map(String::as_str) {
                    Some("solid") => TileCollisionKind2d::Solid,
                    Some("trigger") => TileCollisionKind2d::Trigger,
                    _ => TileCollisionKind2d::None,
                };
            }
            "variants.single" => terrain.variants.single = metadata_u32(prepared, key),
            "variants.left_cap" => terrain.variants.left_cap = metadata_u32(prepared, key),
            "variants.middle" => terrain.variants.middle = metadata_u32(prepared, key),
            "variants.right_cap" => terrain.variants.right_cap = metadata_u32(prepared, key),
            "variants.top_cap" => terrain.variants.top_cap = metadata_u32(prepared, key),
            "variants.bottom_cap" => terrain.variants.bottom_cap = metadata_u32(prepared, key),
            "variants.vertical_middle" => {
                terrain.variants.vertical_middle = metadata_u32(prepared, key)
            }
            "variants.inner_corner_top_left" => {
                terrain.variants.inner_corner_top_left = metadata_u32(prepared, key)
            }
            "variants.inner_corner_top_right" => {
                terrain.variants.inner_corner_top_right = metadata_u32(prepared, key)
            }
            "variants.inner_corner_bottom_left" => {
                terrain.variants.inner_corner_bottom_left = metadata_u32(prepared, key)
            }
            "variants.inner_corner_bottom_right" => {
                terrain.variants.inner_corner_bottom_right = metadata_u32(prepared, key)
            }
            "variants.outer_corner_top_left" => {
                terrain.variants.outer_corner_top_left = metadata_u32(prepared, key)
            }
            "variants.outer_corner_top_right" => {
                terrain.variants.outer_corner_top_right = metadata_u32(prepared, key)
            }
            "variants.outer_corner_bottom_left" => {
                terrain.variants.outer_corner_bottom_left = metadata_u32(prepared, key)
            }
            "variants.outer_corner_bottom_right" => {
                terrain.variants.outer_corner_bottom_right = metadata_u32(prepared, key)
            }
            _ => {}
        }
    }

    let terrains = terrains
        .into_values()
        .filter(|terrain| terrain.symbol != '\0')
        .collect::<Vec<_>>();
    if terrains.is_empty() {
        return None;
    }

    Some(TileRuleSet2d { terrains })
}

fn metadata_u32(prepared: &amigo_assets::PreparedAsset, key: &str) -> Option<u32> {
    prepared.metadata.get(key)?.parse::<u32>().ok()
}

fn first_animation_f32(prepared: &amigo_assets::PreparedAsset, field: &str) -> Option<f32> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<f32>().ok())
            .flatten()
    })
}

fn first_animation_bool(prepared: &amigo_assets::PreparedAsset, field: &str) -> Option<bool> {
    let suffix = format!(".{field}");
    prepared.metadata.iter().find_map(|(key, value)| {
        (key.starts_with("animations.") && key.ends_with(&suffix))
            .then(|| value.parse::<bool>().ok())
            .flatten()
    })
}

fn resolve_asset_request_path(
    mod_catalog: &ModCatalog,
    source: &AssetSourceKind,
    asset_key: &AssetKey,
) -> Result<PathBuf, String> {
    match source {
        AssetSourceKind::Mod(mod_id) => resolve_mod_asset_path(mod_catalog, mod_id, asset_key),
        AssetSourceKind::Engine => {
            let relative = safe_relative_asset_path(asset_key.as_str())?;
            resolve_existing_asset_path(PathBuf::from("assets").join(relative), asset_key.as_str())
        }
        AssetSourceKind::FileSystemRoot(root) => {
            let relative = safe_relative_asset_path(asset_key.as_str())?;
            resolve_existing_asset_path(PathBuf::from(root).join(relative), asset_key.as_str())
        }
        AssetSourceKind::Generated => Err(format!(
            "generated asset `{}` cannot be resolved from filesystem",
            asset_key.as_str()
        )),
    }
}

fn resolve_mod_asset_path(
    mod_catalog: &ModCatalog,
    mod_id: &str,
    asset_key: &AssetKey,
) -> Result<PathBuf, String> {
    let discovered_mod = mod_catalog.mod_by_id(mod_id).ok_or_else(|| {
        format!(
            "mod `{mod_id}` not found while resolving asset `{}`",
            asset_key.as_str()
        )
    })?;
    let Some(relative_key) = asset_key.as_str().strip_prefix(&format!("{mod_id}/")) else {
        return Err(format!(
            "asset key `{}` does not match mod source `{mod_id}`",
            asset_key.as_str()
        ));
    };
    let relative = safe_relative_asset_path(relative_key)?;

    resolve_existing_asset_path(discovered_mod.root_path.join(relative), asset_key.as_str())
}

fn resolve_existing_asset_path(base_path: PathBuf, asset_key: &str) -> Result<PathBuf, String> {
    if base_path.is_file() {
        return Ok(base_path);
    }

    for extension in ["yml", "yaml", "toml"] {
        let candidate = base_path.with_extension(extension);
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "resolved asset path for `{asset_key}` does not exist as a file or known metadata candidate"
    ))
}

fn safe_relative_asset_path(value: &str) -> Result<PathBuf, String> {
    let path = PathBuf::from(value);

    if path.as_os_str().is_empty() {
        return Err("asset path must not be empty".to_owned());
    }

    if path.is_absolute()
        || path.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "asset path `{value}` must stay relative and inside its source root"
        ));
    }

    Ok(path)
}

fn handle_console_command(
    command: amigo_scripting_api::DevConsoleCommand,
    scene_command_queue: &SceneCommandQueue,
    script_event_queue: &ScriptEventQueue,
    dev_console_state: &DevConsoleState,
    runtime_diagnostics: &RuntimeDiagnostics,
    asset_catalog: &AssetCatalog,
) {
    dev_console_state.record_command(command.line.clone());

    let parts = command
        .line
        .split_whitespace()
        .map(str::to_owned)
        .collect::<Vec<_>>();

    match parts.as_slice() {
        [help] if help == "help" => {
            dev_console_state.write_line(
                "available placeholder commands: help, diagnostics, assets, scene select <scene-id>, scene reload, asset reload <asset-key>",
            );
        }
        [diagnostics] if diagnostics == "diagnostics" => {
            dev_console_state.write_line(format!(
                "window={} input={} render={} script={} loaded_mods={} assets_loaded={} assets_prepared={} assets_failed={} assets_pending={}",
                runtime_diagnostics.window_backend,
                runtime_diagnostics.input_backend,
                runtime_diagnostics.render_backend,
                runtime_diagnostics.script_backend,
                runtime_diagnostics.loaded_mods.len(),
                asset_catalog.loaded_assets().len(),
                asset_catalog.prepared_assets().len(),
                asset_catalog.failed_assets().len(),
                asset_catalog.pending_loads().len()
            ));
        }
        [assets] if assets == "assets" => {
            let loaded = asset_catalog
                .loaded_assets()
                .into_iter()
                .map(|asset| asset.key.as_str().to_owned())
                .collect::<Vec<_>>();
            let prepared = asset_catalog
                .prepared_assets()
                .into_iter()
                .map(|asset| format!("{} ({})", asset.key.as_str(), asset.kind.as_str()))
                .collect::<Vec<_>>();
            let failed = asset_catalog
                .failed_assets()
                .into_iter()
                .map(|asset| format!("{}: {}", asset.key.as_str(), asset.reason))
                .collect::<Vec<_>>();
            let pending = asset_catalog
                .pending_loads()
                .into_iter()
                .map(|request| request.key.as_str().to_owned())
                .collect::<Vec<_>>();

            dev_console_state.write_line(format!(
                "assets loaded={} prepared={} failed={} pending={}",
                display_string_list(&loaded),
                display_string_list(&prepared),
                display_string_list(&failed),
                display_string_list(&pending)
            ));
        }
        [scene, select, scene_id] if scene == "scene" && select == "select" => {
            scene_command_queue.submit(SceneCommand::SelectScene {
                scene: SceneKey::new(scene_id.clone()),
            });
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.scene-select-requested",
                vec![scene_id.clone()],
            ));
            dev_console_state.write_line(format!(
                "queued placeholder scene selection for `{scene_id}`"
            ));
        }
        [scene, reload] if scene == "scene" && reload == "reload" => {
            scene_command_queue.submit(SceneCommand::ReloadActiveScene);
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.scene-reload-requested",
                Vec::<String>::new(),
            ));
            dev_console_state.write_line("queued placeholder reload for the active scene");
        }
        [asset, reload, asset_key] if asset == "asset" && reload == "reload" => {
            request_asset_reload(
                asset_catalog,
                asset_key,
                AssetLoadPriority::Immediate,
                dev_console_state,
            );
            script_event_queue.publish(ScriptEvent::new(
                "dev-console.asset-reload-requested",
                vec![asset_key.clone()],
            ));
        }
        _ => {
            dev_console_state.write_line(format!(
                "unknown placeholder console command: {}",
                command.line
            ));
        }
    }
}

fn request_asset_reload(
    asset_catalog: &AssetCatalog,
    asset_key: &str,
    priority: AssetLoadPriority,
    dev_console_state: &DevConsoleState,
) {
    let asset_key = AssetKey::new(asset_key);
    if asset_catalog.manifest(&asset_key).is_none() {
        dev_console_state.write_line(format!(
            "cannot reload unknown asset `{}`",
            asset_key.as_str()
        ));
        return;
    }

    asset_catalog.request_reload(AssetLoadRequest::new(asset_key.clone(), priority));
    dev_console_state.write_line(format!("queued asset reload for `{}`", asset_key.as_str()));
}

fn apply_scene_command(
    runtime: &Runtime,
    command: SceneCommand,
    scene_command_queue: &SceneCommandQueue,
    launch_selection: &LaunchSelection,
    hydrated_scene_state: &HydratedSceneState,
    scene_transition_service: &SceneTransitionService,
    scene_service: &SceneService,
    scene_event_queue: &SceneEventQueue,
    dev_console_state: &DevConsoleState,
    asset_catalog: &AssetCatalog,
    sprite_scene_service: &SpriteSceneService,
    text_scene_service: &Text2dSceneService,
    physics_scene_service: &Physics2dSceneService,
    tilemap_scene_service: &TileMap2dSceneService,
    platformer_scene_service: &PlatformerSceneService,
    camera_follow_scene_service: &CameraFollow2dSceneService,
    parallax_scene_service: &Parallax2dSceneService,
    mesh_scene_service: &MeshSceneService,
    text3d_scene_service: &Text3dSceneService,
    material_scene_service: &MaterialSceneService,
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    audio_scene_service: &AudioSceneService,
    audio_state_service: &AudioStateService,
    audio_mixer_service: &AudioMixerService,
    audio_output_service: &AudioOutputBackendService,
) -> AmigoResult<()> {
    match command {
        SceneCommand::SpawnNamedEntity { name, transform } => {
            let entity = transform
                .map(|transform| scene_service.spawn_with_transform(name.clone(), transform))
                .unwrap_or_else(|| scene_service.spawn(name.clone()));
            scene_event_queue.publish(SceneEvent::EntitySpawned {
                entity_id: entity.raw(),
                name,
            });
            Ok(())
        }
        SceneCommand::SelectScene { scene } => {
            let scene_id = scene.as_str().to_owned();
            let loaded_scene_document =
                if let Some(root_mod) = launch_selection.startup_mod.as_deref() {
                    match load_scene_document_for_mod(runtime, root_mod, &scene_id) {
                        Ok(document) => document,
                        Err(error) => {
                            dev_console_state.write_line(error.to_string());
                            return Ok(());
                        }
                    }
                } else {
                    None
                };

            clear_runtime_scene_content(
                hydrated_scene_state,
                scene_service,
                dev_console_state,
                sprite_scene_service,
                text_scene_service,
                physics_scene_service,
                tilemap_scene_service,
                platformer_scene_service,
                camera_follow_scene_service,
                parallax_scene_service,
                mesh_scene_service,
                text3d_scene_service,
                material_scene_service,
                ui_scene_service,
                ui_state_service,
                audio_scene_service,
                audio_state_service,
                audio_mixer_service,
                audio_output_service,
            );
            scene_service.select_scene(scene.clone());
            scene_event_queue.publish(SceneEvent::SceneSelected {
                scene: scene.clone(),
            });
            if let Some(loaded_scene_document) = loaded_scene_document {
                queue_scene_document_hydration(
                    scene_command_queue,
                    dev_console_state,
                    hydrated_scene_state,
                    scene_transition_service,
                    &loaded_scene_document,
                );
            } else {
                scene_transition_service.clear();
                dev_console_state.write_line(format!(
                    "active placeholder scene set to `{}` without scene document hydration",
                    scene.as_str()
                ));
            }
            Ok(())
        }
        SceneCommand::ReloadActiveScene => {
            let Some(active_scene) = scene_service.selected_scene() else {
                dev_console_state
                    .write_line("cannot reload scene because no active scene is selected");
                return Ok(());
            };
            scene_event_queue.publish(SceneEvent::SceneReloadRequested {
                scene: active_scene.clone(),
            });
            dev_console_state.write_line(format!(
                "reloading active scene `{}` through queue-driven scene selection",
                active_scene.as_str()
            ));
            scene_command_queue.submit(SceneCommand::SelectScene {
                scene: active_scene,
            });
            Ok(())
        }
        SceneCommand::ClearEntities => {
            scene_service.clear_entities();
            scene_event_queue.publish(SceneEvent::EntitiesCleared);
            dev_console_state.write_line("cleared placeholder scene entities");
            Ok(())
        }
        SceneCommand::QueueSprite2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.texture,
                "2d",
                "sprite",
            );
            sprite_scene_service.queue(SpriteDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                sprite: Sprite {
                    texture: command.texture.clone(),
                    size: command.size,
                    sheet: resolve_sprite_sheet_for_command(asset_catalog, &command),
                    sheet_is_explicit: command.sheet.is_some(),
                    animation_override: command.animation.as_ref().map(|animation| {
                        amigo_2d_sprite::SpriteAnimationOverride {
                            fps: animation.fps,
                            looping: animation.looping,
                            start_frame: animation.start_frame,
                        }
                    }),
                    frame_index: command
                        .animation
                        .as_ref()
                        .and_then(|animation| animation.start_frame)
                        .unwrap_or(0),
                    frame_elapsed: 0.0,
                },
                z_index: command.z_index,
                transform: command.transform,
            });
            scene_event_queue.publish(SceneEvent::SpriteQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                texture: command.texture.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d sprite entity `{}` from mod `{}` with asset `{}`",
                command.entity_name,
                command.source_mod,
                command.texture.as_str()
            ));
            Ok(())
        }
        SceneCommand::QueueText2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.font,
                "2d",
                "text",
            );
            text_scene_service.queue(Text2dDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                text: Text2d {
                    content: command.content.clone(),
                    font: command.font.clone(),
                    bounds: command.bounds,
                    transform: command.transform,
                },
            });
            scene_event_queue.publish(SceneEvent::TextQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                font: command.font.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d text entity `{}` from mod `{}` with font `{}`",
                command.entity_name,
                command.source_mod,
                command.font.as_str()
            ));
            Ok(())
        }
        SceneCommand::QueueTileMap2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.tileset,
                "2d",
                "tilemap",
            );
            if let Some(ruleset) = command.ruleset.as_ref() {
                register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    ruleset,
                    "2d",
                    "tile-ruleset",
                );
            }
            let mut tilemap = TileMap2d {
                tileset: command.tileset.clone(),
                ruleset: command.ruleset.clone(),
                tile_size: command.tile_size,
                grid: command.grid.clone(),
                resolved: None,
            };
            if let Some(ruleset_key) = command.ruleset.as_ref() {
                if let Some(prepared) = asset_catalog.prepared_asset(ruleset_key) {
                    if let Some(ruleset) = infer_tile_ruleset_from_prepared_asset(&prepared) {
                        tilemap.resolved = Some(resolve_tilemap(&tilemap, &ruleset));
                    }
                }
            }
            tilemap_scene_service.queue(TileMap2dDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                tilemap: tilemap.clone(),
                z_index: command.z_index,
            });
            for cell in solid_cells(&tilemap) {
                physics_scene_service.queue_static_collider(StaticCollider2dCommand {
                    entity_id: entity,
                    entity_name: format!(
                        "{}.solid.{}.{}",
                        command.entity_name, cell.row_from_bottom, cell.column
                    ),
                    collider: StaticCollider2d {
                        size: command.tile_size,
                        offset: Vec2::new(
                            cell.origin.x + command.tile_size.x * 0.5,
                            cell.origin.y + command.tile_size.y * 0.5,
                        ),
                        layer: CollisionLayer::new("world"),
                    },
                });
            }
            scene_event_queue.publish(SceneEvent::TileMapQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                tileset: command.tileset.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d tilemap entity `{}` from mod `{}` with tileset `{}`",
                command.entity_name,
                command.source_mod,
                command.tileset.as_str()
            ));
            Ok(())
        }
        SceneCommand::QueueKinematicBody2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            physics_scene_service.queue_body(KinematicBody2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                body: KinematicBody2d {
                    velocity: command.velocity,
                    gravity_scale: command.gravity_scale,
                    terminal_velocity: command.terminal_velocity,
                },
            });
            scene_event_queue.publish(SceneEvent::KinematicBodyQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d kinematic body `{}` from mod `{}`",
                command.entity_name, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueAabbCollider2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            physics_scene_service.queue_aabb_collider(AabbCollider2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                collider: AabbCollider2d {
                    size: command.size,
                    offset: command.offset,
                    layer: CollisionLayer::new(command.layer.clone()),
                    mask: CollisionMask::new(
                        command
                            .mask
                            .iter()
                            .cloned()
                            .map(CollisionLayer::new)
                            .collect::<Vec<_>>(),
                    ),
                },
            });
            scene_event_queue.publish(SceneEvent::AabbColliderQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d aabb collider `{}` from mod `{}`",
                command.entity_name, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueTrigger2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            let entity_transform = scene_service
                .transform_of(&command.entity_name)
                .unwrap_or_default();
            physics_scene_service.queue_trigger(Trigger2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                trigger: Trigger2d {
                    size: command.size,
                    offset: Vec2::new(
                        entity_transform.translation.x + command.offset.x,
                        entity_transform.translation.y + command.offset.y,
                    ),
                    layer: CollisionLayer::new(command.layer.clone()),
                    mask: CollisionMask::new(
                        command
                            .mask
                            .iter()
                            .cloned()
                            .map(CollisionLayer::new)
                            .collect::<Vec<_>>(),
                    ),
                    topic: command.event.clone(),
                },
            });
            scene_event_queue.publish(SceneEvent::TriggerQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                topic: command.event.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d trigger `{}` from mod `{}`",
                command.entity_name, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueuePlatformerController2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            platformer_scene_service.queue(PlatformerController2dCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                controller: PlatformerController2d {
                    params: PlatformerControllerParams {
                        max_speed: command.max_speed,
                        acceleration: command.acceleration,
                        deceleration: command.deceleration,
                        air_acceleration: command.air_acceleration,
                        gravity: command.gravity,
                        jump_velocity: command.jump_velocity,
                        terminal_velocity: command.terminal_velocity,
                    },
                },
            });
            scene_event_queue.publish(SceneEvent::PlatformerControllerQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d platformer controller `{}` from mod `{}`",
                command.entity_name, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueCameraFollow2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            camera_follow_scene_service.queue(CameraFollow2dSceneCommand {
                source_mod: command.source_mod.clone(),
                entity_name: command.entity_name.clone(),
                target: command.target.clone(),
                offset: command.offset,
                lerp: command.lerp,
            });
            scene_event_queue.publish(SceneEvent::CameraFollowQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                target: command.target.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d camera follow `{}` -> `{}` from mod `{}`",
                command.entity_name, command.target, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueParallax2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            parallax_scene_service.queue(Parallax2dSceneCommand {
                source_mod: command.source_mod.clone(),
                entity_name: command.entity_name.clone(),
                camera: command.camera.clone(),
                factor: command.factor,
                anchor: command.anchor,
                camera_origin: None,
            });
            scene_event_queue.publish(SceneEvent::ParallaxQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                camera: command.camera.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 2d parallax `{}` -> `{}` from mod `{}`",
                command.entity_name, command.camera, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueTileMapMarker2d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            let symbol = command.symbol.chars().next().unwrap_or_default();
            let tilemap = command
                .tilemap_entity
                .as_deref()
                .and_then(|tilemap_entity| {
                    tilemap_scene_service
                        .commands()
                        .into_iter()
                        .find(|queued| queued.entity_name == tilemap_entity)
                })
                .or_else(|| tilemap_scene_service.commands().into_iter().next());

            let Some(tilemap) = tilemap else {
                dev_console_state.write_line(format!(
                    "cannot resolve tilemap marker `{}` for `{}` because no tilemap has been queued yet",
                    command.symbol, command.entity_name
                ));
                return Ok(());
            };

            let markers = marker_cells(&tilemap.tilemap, symbol);
            let Some(marker) = markers.get(command.index) else {
                dev_console_state.write_line(format!(
                    "cannot resolve tilemap marker `{}`[{}] for `{}` in tilemap `{}`",
                    command.symbol, command.index, command.entity_name, tilemap.entity_name
                ));
                return Ok(());
            };

            let mut transform = scene_service
                .transform_of(&command.entity_name)
                .unwrap_or_default();
            transform.translation.x =
                marker.origin.x + tilemap.tilemap.tile_size.x * 0.5 + command.offset.x;
            transform.translation.y =
                marker.origin.y + tilemap.tilemap.tile_size.y * 0.5 + command.offset.y;
            let _ = scene_service.set_transform(&command.entity_name, transform);

            scene_event_queue.publish(SceneEvent::TileMapMarkerQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                symbol: command.symbol.clone(),
            });
            dev_console_state.write_line(format!(
                "anchored entity `{}` to tilemap marker `{}`[{}] in `{}`",
                command.entity_name, command.symbol, command.index, tilemap.entity_name
            ));
            Ok(())
        }
        SceneCommand::QueueMesh3d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.mesh_asset,
                "3d",
                "mesh",
            );
            mesh_scene_service.queue(MeshDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                mesh: Mesh3d {
                    mesh_asset: command.mesh_asset.clone(),
                    transform: command.transform,
                },
            });
            scene_event_queue.publish(SceneEvent::MeshQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                mesh_asset: command.mesh_asset.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 3d mesh entity `{}` from mod `{}` with mesh `{}`",
                command.entity_name,
                command.source_mod,
                command.mesh_asset.as_str()
            ));
            Ok(())
        }
        SceneCommand::QueueMaterial3d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);

            if let Some(source) = command.source.as_ref() {
                register_mod_asset_reference(
                    asset_catalog,
                    &command.source_mod,
                    source,
                    "3d",
                    "material",
                );
            }

            material_scene_service.queue(MaterialDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                material: Material3d {
                    label: command.label.clone(),
                    albedo: command.albedo,
                    source: command.source.clone(),
                },
            });
            scene_event_queue.publish(SceneEvent::MaterialQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                material_label: command.label.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 3d material `{}` for entity `{}` from mod `{}`",
                command.label, command.entity_name, command.source_mod
            ));
            Ok(())
        }
        SceneCommand::QueueText3d { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_mod_asset_reference(
                asset_catalog,
                &command.source_mod,
                &command.font,
                "3d",
                "text",
            );
            text3d_scene_service.queue(Text3dDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                text: Text3d {
                    content: command.content.clone(),
                    font: command.font.clone(),
                    size: command.size,
                    transform: command.transform,
                },
            });
            scene_event_queue.publish(SceneEvent::Text3dQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
                font: command.font.clone(),
            });
            dev_console_state.write_line(format!(
                "queued 3d text entity `{}` from mod `{}` with font `{}`",
                command.entity_name,
                command.source_mod,
                command.font.as_str()
            ));
            Ok(())
        }
        SceneCommand::QueueUi { command } => {
            let entity = find_or_spawn_scene_entity(scene_service, &command.entity_name);
            register_ui_font_asset_references(
                asset_catalog,
                &command.source_mod,
                &command.document.root,
            );
            ui_scene_service.queue(UiDrawCommand {
                entity_id: entity,
                entity_name: command.entity_name.clone(),
                document: convert_scene_ui_document(&command.document),
            });
            scene_event_queue.publish(SceneEvent::UiQueued {
                entity_id: entity.raw(),
                entity_name: command.entity_name.clone(),
            });
            dev_console_state.write_line(format!(
                "queued ui document entity `{}` from mod `{}`",
                command.entity_name, command.source_mod
            ));
            Ok(())
        }
    }
}

fn clear_runtime_scene_content(
    hydrated_scene_state: &HydratedSceneState,
    scene_service: &SceneService,
    dev_console_state: &DevConsoleState,
    sprite_scene_service: &SpriteSceneService,
    text_scene_service: &Text2dSceneService,
    physics_scene_service: &Physics2dSceneService,
    tilemap_scene_service: &TileMap2dSceneService,
    platformer_scene_service: &PlatformerSceneService,
    camera_follow_scene_service: &CameraFollow2dSceneService,
    parallax_scene_service: &Parallax2dSceneService,
    mesh_scene_service: &MeshSceneService,
    text3d_scene_service: &Text3dSceneService,
    material_scene_service: &MaterialSceneService,
    ui_scene_service: &UiSceneService,
    ui_state_service: &UiStateService,
    audio_scene_service: &AudioSceneService,
    audio_state_service: &AudioStateService,
    audio_mixer_service: &AudioMixerService,
    audio_output_service: &AudioOutputBackendService,
) {
    let previous = hydrated_scene_state.clear();

    if !previous.entity_names.is_empty() {
        let removed = scene_service.remove_entities_by_name(&previous.entity_names);
        dev_console_state.write_line(format!(
            "removed {removed} hydrated scene entities from `{}`",
            previous.scene_id.as_deref().unwrap_or("unknown")
        ));
    }

    sprite_scene_service.clear();
    text_scene_service.clear();
    physics_scene_service.clear();
    tilemap_scene_service.clear();
    platformer_scene_service.clear();
    camera_follow_scene_service.clear();
    parallax_scene_service.clear();
    mesh_scene_service.clear();
    text3d_scene_service.clear();
    material_scene_service.clear();
    ui_scene_service.clear();
    ui_state_service.clear();
    audio_scene_service.clear();
    audio_state_service.clear();
    audio_mixer_service.clear();
    audio_output_service.clear_buffer();
}

fn find_or_spawn_scene_entity(
    scene_service: &SceneService,
    entity_name: &str,
) -> amigo_scene::SceneEntityId {
    scene_service
        .entity_by_name(entity_name)
        .map(|entity| entity.id)
        .unwrap_or_else(|| scene_service.spawn(entity_name.to_owned()))
}

fn scene_ids_for_launch_selection(
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
) -> Vec<String> {
    launch_selection
        .startup_mod
        .as_deref()
        .and_then(|root_mod| mod_catalog.mod_by_id(root_mod))
        .map(|discovered_mod| {
            discovered_mod
                .manifest
                .scenes
                .iter()
                .map(|scene| scene.id.clone())
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn next_scene_id(scene_ids: &[String], active_scene: Option<&str>, step: isize) -> Option<String> {
    if scene_ids.is_empty() {
        return None;
    }

    let current_index = active_scene
        .and_then(|active_scene| {
            scene_ids
                .iter()
                .position(|scene_id| scene_id == active_scene)
        })
        .unwrap_or(0);
    let len = scene_ids.len() as isize;
    let next_index = (current_index as isize + step).rem_euclid(len) as usize;

    scene_ids.get(next_index).cloned()
}

fn format_script_command(command: &ScriptCommand) -> String {
    if command.arguments.is_empty() {
        return format!("{}.{}", command.namespace, command.name);
    }

    format!(
        "{}.{}({})",
        command.namespace,
        command.name,
        command.arguments.join(", ")
    )
}

fn format_scene_command(command: &SceneCommand) -> String {
    match command {
        SceneCommand::SpawnNamedEntity { name, .. } => format!("scene.spawn({name})"),
        SceneCommand::SelectScene { scene } => format!("scene.select({})", scene.as_str()),
        SceneCommand::ReloadActiveScene => "scene.reload_active".to_owned(),
        SceneCommand::ClearEntities => "scene.clear".to_owned(),
        SceneCommand::QueueSprite2d { command } => format!(
            "scene.2d.sprite({}, {}, {}x{})",
            command.entity_name,
            command.texture.as_str(),
            command.size.x,
            command.size.y
        ),
        SceneCommand::QueueTileMap2d { command } => format!(
            "scene.2d.tilemap({}, {}, {} rows)",
            command.entity_name,
            command.tileset.as_str(),
            command.grid.len()
        ),
        SceneCommand::QueueText2d { command } => format!(
            "scene.2d.text({}, {}, {}x{})",
            command.entity_name,
            command.font.as_str(),
            command.bounds.x,
            command.bounds.y
        ),
        SceneCommand::QueueKinematicBody2d { command } => format!(
            "scene.2d.physics.body({}, {}, {}, {})",
            command.entity_name, command.velocity.x, command.velocity.y, command.gravity_scale
        ),
        SceneCommand::QueueAabbCollider2d { command } => format!(
            "scene.2d.physics.collider({}, {}x{}, {})",
            command.entity_name, command.size.x, command.size.y, command.layer
        ),
        SceneCommand::QueueTrigger2d { command } => format!(
            "scene.2d.physics.trigger({}, {}x{}, {})",
            command.entity_name,
            command.size.x,
            command.size.y,
            command.event.as_deref().unwrap_or("none")
        ),
        SceneCommand::QueuePlatformerController2d { command } => format!(
            "scene.2d.platformer({}, max_speed={}, jump_velocity={})",
            command.entity_name, command.max_speed, command.jump_velocity
        ),
        SceneCommand::QueueCameraFollow2d { command } => format!(
            "scene.2d.camera_follow({}, {}, {}, {})",
            command.entity_name, command.target, command.offset.x, command.offset.y
        ),
        SceneCommand::QueueParallax2d { command } => format!(
            "scene.2d.parallax({}, {}, {}, {})",
            command.entity_name, command.camera, command.factor.x, command.factor.y
        ),
        SceneCommand::QueueTileMapMarker2d { command } => format!(
            "scene.2d.tilemap_marker({}, {}, #{})",
            command.entity_name, command.symbol, command.index
        ),
        SceneCommand::QueueMesh3d { command } => format!(
            "scene.3d.mesh({}, {})",
            command.entity_name,
            command.mesh_asset.as_str()
        ),
        SceneCommand::QueueMaterial3d { command } => format!(
            "scene.3d.material({}, {}, {})",
            command.entity_name,
            command.label,
            command
                .source
                .as_ref()
                .map(|asset| asset.as_str().to_owned())
                .unwrap_or_else(|| "generated".to_owned())
        ),
        SceneCommand::QueueText3d { command } => format!(
            "scene.3d.text({}, {}, {})",
            command.entity_name,
            command.font.as_str(),
            command.size
        ),
        SceneCommand::QueueUi { command } => {
            format!("scene.ui({}, screen-space)", command.entity_name)
        }
    }
}

fn format_audio_command(command: &AudioCommand) -> String {
    match command {
        AudioCommand::PlayOnce { clip } => format!("audio.play({})", clip.as_str()),
        AudioCommand::StartSource { source, clip } => {
            format!("audio.start({}, {})", source.as_str(), clip.as_str())
        }
        AudioCommand::StopSource { source } => format!("audio.stop({})", source.as_str()),
        AudioCommand::SetParam {
            source,
            param,
            value,
        } => format!("audio.set_param({}, {}, {})", source.as_str(), param, value),
        AudioCommand::SetVolume { bus, value } => {
            format!("audio.set_volume({}, {})", bus, value)
        }
        AudioCommand::SetMasterVolume { value } => format!("audio.set_master_volume({value})"),
    }
}

fn format_script_event(event: &ScriptEvent) -> String {
    if event.payload.is_empty() {
        return event.topic.clone();
    }

    format!("{}({})", event.topic, event.payload.join(", "))
}

fn display_string_list(values: &[String]) -> String {
    if values.is_empty() {
        return "none".to_owned();
    }

    values.join(", ")
}

fn display_executed_scripts(scripts: &[ExecutedScript]) -> String {
    if scripts.is_empty() {
        return "none".to_owned();
    }

    scripts
        .iter()
        .map(|script| script.source_name.clone())
        .collect::<Vec<_>>()
        .join(", ")
}

fn required<T>(runtime: &Runtime) -> AmigoResult<Arc<T>>
where
    T: Send + Sync + 'static,
{
    runtime
        .resolve::<T>()
        .ok_or(AmigoError::MissingService(type_name::<T>()))
}

fn required_from_registry<T>(registry: &ServiceRegistry) -> AmigoResult<Arc<T>>
where
    T: Send + Sync + 'static,
{
    registry
        .resolve::<T>()
        .ok_or(AmigoError::MissingService(type_name::<T>()))
}

struct SummaryHostHandler {
    summary: BootstrapSummary,
    surface: Option<WgpuSurfaceState>,
    printed: bool,
}

impl SummaryHostHandler {
    fn new(summary: BootstrapSummary) -> Self {
        Self {
            summary,
            surface: None,
            printed: false,
        }
    }
}

struct InteractiveRuntimeHostHandler {
    runtime: Runtime,
    summary: BootstrapSummary,
    surface: Option<WgpuSurfaceState>,
    renderer: Option<WgpuSceneRenderer>,
    scene_ids: Vec<String>,
    printed_console_lines: usize,
    printed: bool,
}

impl InteractiveRuntimeHostHandler {
    fn new(runtime: Runtime, summary: BootstrapSummary) -> AmigoResult<Self> {
        let launch_selection = required::<LaunchSelection>(&runtime)?;
        let mod_catalog = required::<ModCatalog>(&runtime)?;
        let scene_ids =
            scene_ids_for_launch_selection(mod_catalog.as_ref(), launch_selection.as_ref());

        Ok(Self {
            runtime,
            printed_console_lines: summary.console_output.len(),
            summary,
            surface: None,
            renderer: None,
            scene_ids,
            printed: false,
        })
    }

    fn queue_scene_switch(&mut self, step: isize) -> AmigoResult<()> {
        let scene_service = required::<SceneService>(&self.runtime)?;
        let active_scene = scene_service.selected_scene();
        let Some(next_scene_id) = next_scene_id(
            &self.scene_ids,
            active_scene.as_ref().map(SceneKey::as_str),
            step,
        ) else {
            return Ok(());
        };

        required::<SceneCommandQueue>(&self.runtime)?.submit(SceneCommand::SelectScene {
            scene: SceneKey::new(next_scene_id.clone()),
        });

        Ok(())
    }

    fn queue_console_command(&mut self, line: &str) -> AmigoResult<()> {
        required::<DevConsoleQueue>(&self.runtime)?
            .submit(amigo_scripting_api::DevConsoleCommand::new(line));
        Ok(())
    }

    fn tick_active_scripts(&mut self, delta_seconds: f32) -> AmigoResult<()> {
        let script_runtime = required::<ScriptRuntimeService>(&self.runtime)?;
        for script in current_executed_scripts(&self.runtime)? {
            match script.role {
                ScriptExecutionRole::ModPersistent | ScriptExecutionRole::Scene => {
                    script_runtime.call_update(&script.source_name, delta_seconds)?;
                }
                ScriptExecutionRole::ModBootstrap => {}
            }
        }

        Ok(())
    }

    fn tick_scene_transitions(&mut self, delta_seconds: f32) -> AmigoResult<()> {
        let scene_transition_service = required::<SceneTransitionService>(&self.runtime)?;
        let scene_command_queue = required::<SceneCommandQueue>(&self.runtime)?;

        for command in scene_transition_service.tick(delta_seconds) {
            scene_command_queue.submit(command);
        }

        Ok(())
    }

    fn host_scene_switch_enabled(&self) -> bool {
        self.summary.startup_mod.as_deref() == Some("core-game") && self.scene_ids.len() > 1
    }

    fn pump_runtime(&mut self) -> AmigoResult<()> {
        let previous_scene = self.summary.active_scene.clone();
        let previous_document = self.summary.loaded_scene_document.clone();
        let previous_entities = self.summary.scene_entities.clone();
        let updated = refresh_runtime_summary(&self.runtime)?;

        if updated.active_scene != previous_scene {
            println!(
                "active scene: {}",
                updated.active_scene.as_deref().unwrap_or("none")
            );
        }

        if updated.loaded_scene_document != previous_document {
            println!(
                "scene document: {}",
                updated
                    .loaded_scene_document
                    .as_ref()
                    .map(|document| format!(
                        "{}:{}",
                        document.source_mod,
                        document.relative_path.display()
                    ))
                    .unwrap_or_else(|| "none".to_owned())
            );
        }

        if updated.scene_entities != previous_entities {
            println!(
                "scene entities: {}",
                display_string_list(&updated.scene_entities)
            );
        }

        for line in updated
            .console_output
            .iter()
            .skip(self.printed_console_lines)
        {
            println!("console: {line}");
        }

        self.printed_console_lines = updated.console_output.len();
        self.summary = updated;

        Ok(())
    }

    fn process_ui_input(&mut self) -> AmigoResult<()> {
        let Some(surface) = self.surface.as_ref() else {
            return Ok(());
        };

        let ui_input = required::<UiInputService>(&self.runtime)?;
        let snapshot = ui_input.snapshot();
        if !snapshot.mouse_left_released {
            return Ok(());
        }

        let Some(mouse_position) = snapshot.mouse_position else {
            return Ok(());
        };

        let ui_scene = required::<UiSceneService>(&self.runtime)?;
        let ui_state = required::<UiStateService>(&self.runtime)?;
        let script_event_queue = required::<ScriptEventQueue>(&self.runtime)?;
        let resolved = resolve_ui_overlay_documents(ui_scene.as_ref(), ui_state.as_ref());
        let size = surface.size();
        let viewport = UiViewportSize::new(size.width as f32, size.height as f32);

        for document in resolved.iter().rev() {
            let layout = build_ui_layout_tree(viewport, &document.overlay);
            let Some(path) = hit_test_ui_layout(&layout, mouse_position.x, mouse_position.y) else {
                continue;
            };

            if !ui_state.is_enabled(&path) {
                continue;
            }

            if let Some(binding) = document.click_bindings.get(&path) {
                script_event_queue.publish(ScriptEvent::new(
                    binding.event.clone(),
                    binding.payload.clone(),
                ));
                break;
            }
        }

        Ok(())
    }
}

struct LaunchSelectionPlugin {
    selection: LaunchSelection,
}

impl LaunchSelectionPlugin {
    fn new(selection: LaunchSelection) -> Self {
        Self { selection }
    }
}

impl RuntimePlugin for LaunchSelectionPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-launch-selection"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(self.selection.clone())
    }
}

struct RuntimeDiagnosticsPlugin {
    script_backend: String,
    plugin_names: Vec<String>,
}

impl RuntimeDiagnosticsPlugin {
    fn phase1() -> Self {
        Self {
            script_backend: "rhai".to_owned(),
            plugin_names: vec![
                "amigo-assets".to_owned(),
                "amigo-scene".to_owned(),
                "amigo-window-winit".to_owned(),
                "amigo-input-winit".to_owned(),
                "amigo-render-wgpu".to_owned(),
                "amigo-app-launch-selection".to_owned(),
                "amigo-2d-sprite".to_owned(),
                "amigo-2d-text".to_owned(),
                "amigo-ui".to_owned(),
                "amigo-2d-physics".to_owned(),
                "amigo-2d-tilemap".to_owned(),
                "amigo-2d-platformer".to_owned(),
                "amigo-audio-api".to_owned(),
                "amigo-audio-generated".to_owned(),
                "amigo-audio-mixer".to_owned(),
                "amigo-audio-output".to_owned(),
                "amigo-3d-mesh".to_owned(),
                "amigo-3d-text".to_owned(),
                "amigo-3d-material".to_owned(),
                "amigo-modding".to_owned(),
                "amigo-app-runtime-diagnostics".to_owned(),
                "amigo-scripting-rhai".to_owned(),
            ],
        }
    }
}

impl RuntimePlugin for RuntimeDiagnosticsPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-runtime-diagnostics"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        let window = required_from_registry::<WindowServiceInfo>(registry)?;
        let input = required_from_registry::<InputServiceInfo>(registry)?;
        let render = required_from_registry::<RenderBackendInfo>(registry)?;

        let mut capabilities = Vec::new();
        capabilities.push(
            required_from_registry::<SpriteDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Text2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<UiDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Physics2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<TileMap2dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<PlatformerDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<GeneratedAudioDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioMixerDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<AudioOutputDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<MeshDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<Text3dDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.push(
            required_from_registry::<MaterialDomainInfo>(registry)?
                .capability
                .to_owned(),
        );
        capabilities.sort();

        let loaded_mods = registry
            .resolve::<ModCatalog>()
            .map(|catalog| {
                catalog
                    .mod_ids()
                    .into_iter()
                    .map(str::to_owned)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut service_names = registry
            .registered_names()
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>();
        service_names.push(type_name::<ScriptCommandQueue>().to_owned());
        service_names.push(type_name::<ScriptEventQueue>().to_owned());
        service_names.push(type_name::<DevConsoleQueue>().to_owned());
        service_names.push(type_name::<DevConsoleState>().to_owned());
        service_names.push(type_name::<ScriptRuntimeInfo>().to_owned());
        service_names.push(type_name::<ScriptRuntimeService>().to_owned());
        service_names.push(type_name::<RuntimeDiagnostics>().to_owned());
        service_names.sort();
        service_names.dedup();

        registry.register(RuntimeDiagnostics::new(
            window.backend_name,
            input.backend_name,
            render.backend_name,
            self.script_backend.clone(),
            loaded_mods,
            capabilities,
            self.plugin_names.clone(),
            service_names,
        ))
    }
}

impl HostHandler for SummaryHostHandler {
    fn config(&self) -> HostConfig {
        HostConfig {
            window: WindowDescriptor {
                title: "Amigo Hosted".to_owned(),
                ..WindowDescriptor::default()
            },
            exit_strategy: HostExitStrategy::AfterFirstRedraw,
        }
    }

    fn on_lifecycle(&mut self, event: HostLifecycleEvent) -> AmigoResult<HostControl> {
        if matches!(event, HostLifecycleEvent::WindowCreated) && !self.printed {
            println!("{}", self.summary);
            self.printed = true;
        }

        Ok(HostControl::Continue)
    }

    fn on_window_event(&mut self, event: WindowEvent) -> AmigoResult<HostControl> {
        if let WindowEvent::Resized(size) = event {
            if let Some(surface) = &mut self.surface {
                surface.resize(size);
            }
        }

        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(HostControl::Exit);
        }

        Ok(HostControl::Continue)
    }

    fn on_window_ready(&mut self, handles: WindowSurfaceHandles) -> AmigoResult<HostControl> {
        let backend = WgpuRenderBackend::default();
        let surface = backend.initialize_for_window(handles)?;

        println!(
            "render init: backend={} adapter={} adapter_backend={} device_type={} queue_ready={}",
            surface.report.backend_name,
            surface.report.adapter_name,
            surface.report.adapter_backend,
            surface.report.device_type,
            surface.report.queue_ready
        );

        self.surface = Some(surface);

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        if let Some(surface) = &mut self.surface {
            surface.render_default_frame()?;
        }

        Ok(HostControl::Continue)
    }
}

impl HostHandler for InteractiveRuntimeHostHandler {
    fn config(&self) -> HostConfig {
        HostConfig {
            window: WindowDescriptor {
                title: "Amigo Hosted Dev".to_owned(),
                ..WindowDescriptor::default()
            },
            exit_strategy: HostExitStrategy::Manual,
        }
    }

    fn on_lifecycle(&mut self, event: HostLifecycleEvent) -> AmigoResult<HostControl> {
        if matches!(event, HostLifecycleEvent::WindowCreated) && !self.printed {
            println!("{}", self.summary);
            if self.host_scene_switch_enabled() {
                println!(
                    "host controls: Left/Right switch scenes, Enter help, Space diagnostics, Escape exits"
                );
            } else {
                println!(
                    "host controls: arrow keys flow into InputState, Enter help, Space diagnostics, Escape exits"
                );
            }
            self.printed = true;
        }

        if matches!(event, HostLifecycleEvent::AboutToWait) {
            self.process_ui_input()?;
            self.tick_active_scripts(1.0 / 60.0)?;
            tick_platformer_world(&self.runtime, 1.0 / 60.0)?;
            tick_camera_follow_world(&self.runtime)?;
            tick_parallax_world(&self.runtime)?;
            self.tick_scene_transitions(1.0 / 60.0)?;
            self.pump_runtime()?;
            tick_audio_runtime(&self.runtime, 1.0 / 60.0)?;
            if let Some(input_state) = self.runtime.resolve::<InputState>() {
                input_state.clear_frame_transients();
            }
            if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                ui_input.clear_frame_transients();
            }
        }

        Ok(HostControl::Continue)
    }

    fn on_input_event(&mut self, event: InputEvent) -> AmigoResult<HostControl> {
        match event {
            InputEvent::CursorMoved { x, y } => {
                if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                    ui_input.set_mouse_position(x as f32, y as f32);
                }
            }
            InputEvent::MouseButton {
                button: amigo_input_api::MouseButton::Left,
                pressed,
            } => {
                if let Some(ui_input) = self.runtime.resolve::<UiInputService>() {
                    ui_input.set_left_button(pressed);
                }
            }
            _ => {}
        }

        if let InputEvent::Key { key, pressed } = event {
            if let Some(input_state) = self.runtime.resolve::<InputState>() {
                input_state.set_key(key, pressed);
            }

            if key == KeyCode::Escape && pressed {
                return Ok(HostControl::Exit);
            }

            if !pressed {
                return Ok(HostControl::Continue);
            }

            if self.host_scene_switch_enabled() {
                match key {
                    KeyCode::Right | KeyCode::Down => self.queue_scene_switch(1)?,
                    KeyCode::Left | KeyCode::Up => self.queue_scene_switch(-1)?,
                    KeyCode::Enter => self.queue_console_command("help")?,
                    KeyCode::Space => self.queue_console_command("diagnostics")?,
                    _ => {}
                }

                return Ok(HostControl::Continue);
            }
        }

        match event {
            InputEvent::Key {
                key: KeyCode::Enter,
                pressed: true,
            } => self.queue_console_command("help")?,
            InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            } => self.queue_console_command("diagnostics")?,
            _ => {}
        }

        Ok(HostControl::Continue)
    }

    fn on_window_event(&mut self, event: WindowEvent) -> AmigoResult<HostControl> {
        if let WindowEvent::Resized(size) = event {
            if let Some(surface) = &mut self.surface {
                surface.resize(size);
            }
        }

        if matches!(event, WindowEvent::CloseRequested) {
            return Ok(HostControl::Exit);
        }

        Ok(HostControl::Continue)
    }

    fn on_window_ready(&mut self, handles: WindowSurfaceHandles) -> AmigoResult<HostControl> {
        let backend = WgpuRenderBackend::default();
        let surface = backend.initialize_for_window(handles)?;
        let renderer = WgpuSceneRenderer::new(&surface);

        println!(
            "render init: backend={} adapter={} adapter_backend={} device_type={} queue_ready={}",
            surface.report.backend_name,
            surface.report.adapter_name,
            surface.report.adapter_backend,
            surface.report.device_type,
            surface.report.queue_ready
        );

        self.surface = Some(surface);
        self.renderer = Some(renderer);
        start_audio_output(&self.runtime)?;

        Ok(HostControl::Continue)
    }

    fn on_redraw_requested(&mut self) -> AmigoResult<HostControl> {
        if let Some(surface) = &mut self.surface {
            if let Some(renderer) = &mut self.renderer {
                let scene = required::<SceneService>(&self.runtime)?;
                let assets = required::<AssetCatalog>(&self.runtime)?;
                let tilemaps = required::<TileMap2dSceneService>(&self.runtime)?;
                let sprites = required::<SpriteSceneService>(&self.runtime)?;
                let text2d = required::<Text2dSceneService>(&self.runtime)?;
                let meshes = required::<MeshSceneService>(&self.runtime)?;
                let text3d = required::<Text3dSceneService>(&self.runtime)?;
                let materials = required::<MaterialSceneService>(&self.runtime)?;
                let ui_scene = required::<UiSceneService>(&self.runtime)?;
                let ui_state = required::<UiStateService>(&self.runtime)?;
                let ui_documents =
                    resolve_ui_overlay_documents(ui_scene.as_ref(), ui_state.as_ref());
                let ui_overlays = ui_documents
                    .iter()
                    .map(|document| document.overlay.clone())
                    .collect::<Vec<_>>();

                renderer.render_scene_with_ui_documents(
                    surface,
                    scene.as_ref(),
                    assets.as_ref(),
                    tilemaps.as_ref(),
                    sprites.as_ref(),
                    text2d.as_ref(),
                    meshes.as_ref(),
                    materials.as_ref(),
                    Some(text3d.as_ref()),
                    &ui_overlays,
                )?;
            } else {
                surface.render_default_frame()?;
            }
        }

        Ok(HostControl::Continue)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    use amigo_2d_sprite::SpriteSceneService;
    use amigo_2d_text::Text2dSceneService;
    use amigo_2d_tilemap::{TileMap2dSceneService, TileVariantKind2d};
    use amigo_app_host_api::{HostHandler, HostLifecycleEvent};
    use amigo_assets::{AssetCatalog, AssetKey};
    use amigo_audio_api::{AudioCommand, AudioCommandQueue, AudioSceneService, AudioStateService};
    use amigo_audio_mixer::AudioMixerService;
    use amigo_core::RuntimeDiagnostics;
    use amigo_input_api::{InputEvent, KeyCode};
    use amigo_scene::{HydratedSceneState, SceneCommandQueue, SceneService};
    use amigo_scripting_api::{
        DevConsoleCommand, DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptEventQueue,
    };
    use amigo_ui::{UiSceneService, UiStateService};

    use super::{
        BootstrapOptions, InteractiveRuntimeHostHandler, bootstrap_with_options,
        handle_script_command, next_scene_id, process_audio_command, refresh_runtime_summary,
        scene_ids_for_launch_selection,
    };
    use amigo_core::LaunchSelection;
    use amigo_modding::ModCatalog;

    #[test]
    fn core_game_console_scene_processes_placeholder_queues() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
                .with_startup_mod("core-game")
                .with_startup_scene("console")
                .with_dev_mode(true),
        )
        .expect("console bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("console"));
        assert!(
            summary
                .console_commands
                .iter()
                .any(|command| command == "help")
        );
        assert!(
            summary
                .console_output
                .iter()
                .any(|line| line.contains("available placeholder commands"))
        );
        assert!(
            summary
                .processed_script_events
                .iter()
                .any(|event| event == "core-game.bootstrapped(console)")
        );
    }

    #[test]
    fn core_game_diagnostics_scene_writes_refresh_output() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
                .with_startup_mod("core-game")
                .with_startup_scene("diagnostics")
                .with_dev_mode(true),
        )
        .expect("diagnostics bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("diagnostics"));
        assert!(
            summary
                .processed_script_commands
                .iter()
                .any(|command| command == "dev-shell.refresh-diagnostics(core-game)")
        );
        assert!(
            summary
                .console_output
                .iter()
                .any(|line| line.contains("diagnostics refreshed for mod=core-game"))
        );
        assert!(
            summary
                .processed_script_events
                .iter()
                .any(|event| event == "dev-shell.diagnostics-refreshed(core-game)")
        );
    }

    #[test]
    fn playground_2d_sprite_scene_populates_2d_domain_and_assets() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("2d sprite playground bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("sprite-lab"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/sprite-lab/scene.yml")
        );
        assert!(
            summary
                .processed_scene_commands
                .iter()
                .any(|command| command.starts_with("scene.2d.sprite("))
        );
        assert!(
            summary
                .registered_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/sprite-lab")
        );
        assert!(
            summary
                .loaded_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/sprite-lab")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/sprite-lab (sprite-2d)")
        );
        assert!(summary.failed_assets.is_empty());
        assert!(summary.pending_asset_loads.is_empty());
        assert!(
            summary
                .sprite_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-sprite")
        );
        assert!(summary.text_entities_2d.is_empty());
    }

    #[test]
    fn playground_2d_text_scene_populates_2d_text_domain_and_assets() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("text-lab")
                .with_dev_mode(true),
        )
        .expect("2d text playground bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("text-lab"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/text-lab/scene.yml")
        );
        assert!(
            summary
                .processed_scene_commands
                .iter()
                .any(|command| command.starts_with("scene.2d.text("))
        );
        assert!(
            summary
                .registered_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui")
        );
        assert!(
            summary
                .loaded_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
        );
        assert!(summary.failed_assets.is_empty());
        assert!(summary.pending_asset_loads.is_empty());
        assert!(
            summary
                .text_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-label")
        );
        assert!(summary.sprite_entities_2d.is_empty());
    }

    #[test]
    fn playground_2d_scene_selection_rehydrates_document_content() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("2d sprite playground bootstrap should succeed");

        runtime
            .resolve::<DevConsoleQueue>()
            .expect("dev console queue should exist")
            .submit(amigo_scripting_api::DevConsoleCommand::new(
                "scene select text-lab",
            ));

        let bridge = super::process_placeholder_bridges(&runtime)
            .expect("scene selection bridge should succeed");
        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let hydrated = runtime
            .resolve::<HydratedSceneState>()
            .expect("hydrated scene state should exist");
        let sprite = runtime
            .resolve::<SpriteSceneService>()
            .expect("sprite scene service should exist");
        let text = runtime
            .resolve::<Text2dSceneService>()
            .expect("text scene service should exist");

        assert_eq!(
            scene.selected_scene().as_ref().map(|scene| scene.as_str()),
            Some("text-lab")
        );
        assert!(scene.entity_by_name("playground-2d-sprite").is_none());
        assert!(scene.entity_by_name("playground-2d-label").is_some());
        assert!(sprite.entity_names().is_empty());
        assert_eq!(text.entity_names(), vec!["playground-2d-label".to_owned()]);
        assert_eq!(hydrated.snapshot().scene_id.as_deref(), Some("text-lab"));
        assert!(
            bridge
                .processed_scene_commands
                .iter()
                .any(|command| command == "scene.select(text-lab)")
        );
        assert!(
            bridge
                .processed_scene_commands
                .iter()
                .any(|command| command.starts_with("scene.2d.text("))
        );
    }

    #[test]
    fn scene_helpers_resolve_scene_ids_and_wrap_indices() {
        let mod_catalog = ModCatalog::from_discovered_mods(vec![amigo_modding::DiscoveredMod {
            manifest: amigo_modding::ModManifest {
                id: "playground-2d".to_owned(),
                name: "Playground 2D".to_owned(),
                version: "0.1.0".to_owned(),
                description: None,
                authors: Vec::new(),
                dependencies: vec!["core".to_owned()],
                capabilities: vec!["rendering_2d".to_owned()],
                scripting: None,
                scenes: vec![
                    amigo_modding::ModSceneManifest {
                        id: "sprite-lab".to_owned(),
                        label: "Sprite Lab".to_owned(),
                        description: None,
                        path: "scenes/sprite-lab".to_owned(),
                        document: None,
                        script: None,
                        launcher_visible: true,
                    },
                    amigo_modding::ModSceneManifest {
                        id: "text-lab".to_owned(),
                        label: "Text Lab".to_owned(),
                        description: None,
                        path: "scenes/text-lab".to_owned(),
                        document: None,
                        script: None,
                        launcher_visible: true,
                    },
                ],
            },
            root_path: mods_root().join("playground-2d"),
        }]);
        let launch_selection = LaunchSelection::new(
            Some("playground-2d".to_owned()),
            Some("sprite-lab".to_owned()),
            vec!["core".to_owned(), "playground-2d".to_owned()],
            true,
        );

        let scene_ids = scene_ids_for_launch_selection(&mod_catalog, &launch_selection);

        assert_eq!(
            scene_ids,
            vec!["sprite-lab".to_owned(), "text-lab".to_owned()]
        );
        assert_eq!(
            next_scene_id(&scene_ids, Some("sprite-lab"), 1).as_deref(),
            Some("text-lab")
        );
        assert_eq!(
            next_scene_id(&scene_ids, Some("sprite-lab"), -1).as_deref(),
            Some("text-lab")
        );
    }

    #[test]
    fn runtime_can_process_console_commands_after_bootstrap() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "core-game".to_owned()])
                .with_startup_mod("core-game")
                .with_startup_scene("console")
                .with_dev_mode(true),
        )
        .expect("console bootstrap should succeed");

        runtime
            .resolve::<DevConsoleQueue>()
            .expect("dev console queue should exist")
            .submit(DevConsoleCommand::new("diagnostics"));

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should process queued console command");

        assert!(
            updated
                .console_commands
                .iter()
                .any(|command| command == "diagnostics")
        );
        assert!(
            updated
                .console_output
                .iter()
                .any(|line| line.contains("window=winit input=winit render=wgpu script=rhai"))
        );
    }

    #[test]
    fn runtime_can_reload_active_scene_after_bootstrap() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("sprite playground bootstrap should succeed");

        runtime
            .resolve::<DevConsoleQueue>()
            .expect("dev console queue should exist")
            .submit(DevConsoleCommand::new("scene reload"));

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should process scene reload command");

        assert_eq!(updated.active_scene.as_deref(), Some("sprite-lab"));
        assert!(
            updated
                .console_commands
                .iter()
                .any(|command| command == "scene reload")
        );
        assert!(
            updated
                .processed_scene_commands
                .iter()
                .any(|command| command == "scene.reload_active")
        );
        assert!(
            updated
                .processed_scene_commands
                .iter()
                .any(|command| command == "scene.select(sprite-lab)")
        );
        assert!(
            updated
                .console_output
                .iter()
                .any(|line| line.contains("reloading active scene `sprite-lab`"))
        );
    }

    #[test]
    fn runtime_can_reload_asset_after_bootstrap() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("sprite playground bootstrap should succeed");

        runtime
            .resolve::<DevConsoleQueue>()
            .expect("dev console queue should exist")
            .submit(DevConsoleCommand::new(
                "asset reload playground-2d/textures/sprite-lab",
            ));

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should process asset reload command");

        assert!(
            updated
                .console_commands
                .iter()
                .any(|command| command == "asset reload playground-2d/textures/sprite-lab")
        );
        assert!(
            updated
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/sprite-lab (sprite-2d)")
        );
        assert!(updated.console_output.iter().any(|line| {
            line.contains("queued asset reload for `playground-2d/textures/sprite-lab`")
        }));
        assert!(updated.console_output.iter().any(|line| {
            line.contains("prepared asset `playground-2d/textures/sprite-lab` as `sprite-2d`")
        }));
    }

    #[test]
    fn runtime_detects_asset_file_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root("asset-hot-reload", &["core", "playground-2d"]);
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods.clone())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("sprite playground bootstrap should succeed");

        fs::write(
            temp_mods
                .join("playground-2d")
                .join("textures")
                .join("sprite-lab"),
            "kind = \"sprite-2d\"\nlabel = \"Reloaded Sprite\"\nformat = \"debug-placeholder\"\n",
        )
        .expect("asset file should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect asset file changes");

        assert!(updated.console_output.iter().any(|line| {
            line.contains("detected asset change for `playground-2d/textures/sprite-lab`")
        }));
        assert!(
            updated
                .processed_script_events
                .iter()
                .any(|event| event.starts_with("hot-reload.asset-changed("))
        );
        assert!(
            updated
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/sprite-lab (sprite-2d)")
        );
    }

    #[test]
    fn bootstrap_reports_file_watch_backend() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("sprite playground bootstrap should succeed");

        assert!(summary.file_watch_backend.starts_with("notify"));
    }

    #[test]
    fn playground_2d_main_scene_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("hello-world-spritesheet")
                .with_dev_mode(true),
        )
        .expect("2d main playground bootstrap should succeed");

        assert_eq!(
            summary.active_scene.as_deref(),
            Some("hello-world-spritesheet")
        );
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/hello-world-spritesheet/scene.yml")
        );
        assert!(
            summary
                .sprite_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-spritesheet")
        );
        assert!(
            summary
                .text_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-hello")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/hello-world-spritesheet (sprite-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn playground_2d_basic_scripting_demo_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("basic-scripting-demo")
                .with_dev_mode(true),
        )
        .expect("2d scripting demo bootstrap should succeed");

        assert_eq!(
            summary.active_scene.as_deref(),
            Some("basic-scripting-demo")
        );
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/basic-scripting-demo/scene.yml")
        );
        assert!(
            summary
                .sprite_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-demo-square")
        );
        assert!(
            summary
                .sprite_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-demo-spritesheet")
        );
        assert!(
            summary
                .text_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-demo-title")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/square (sprite-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/textures/hello-world-spritesheet (sprite-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
        );
        assert!(
            summary
                .processed_script_events
                .iter()
                .any(|event| event == "playground-2d.demo.entered(basic-scripting-demo)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn runtime_detects_scene_document_file_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root("scene-hot-reload", &["core", "playground-2d"]);
        let scene_path = temp_mods
            .join("playground-2d")
            .join("scenes")
            .join("sprite-lab")
            .join("scene.yml");
        let original_scene =
            fs::read_to_string(&scene_path).expect("scene document should be readable");

        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("sprite-lab")
                .with_dev_mode(true),
        )
        .expect("sprite playground bootstrap should succeed");

        fs::write(
            &scene_path,
            original_scene.replace("playground-2d-sprite", "playground-2d-sprite-live"),
        )
        .expect("scene document should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect scene document changes");

        assert_eq!(updated.active_scene.as_deref(), Some("sprite-lab"));
        assert!(
            updated
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-2d-sprite-live")
        );
        assert!(
            updated
                .scene_entities
                .iter()
                .all(|entity| entity != "playground-2d-sprite")
        );
        assert!(updated.console_output.iter().any(|line| {
            line.contains("detected scene document change for `playground-2d:sprite-lab`")
        }));
        assert!(
            updated
                .processed_scene_commands
                .iter()
                .any(|command| command == "scene.reload_active")
        );
    }

    #[test]
    fn runtime_detects_sidescroller_scene_document_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root(
            "sidescroller-scene-hot-reload",
            &["core", "playground-sidescroller"],
        );
        let scene_path = temp_mods
            .join("playground-sidescroller")
            .join("scenes")
            .join("vertical-slice")
            .join("scene.yml");
        let original_scene =
            fs::read_to_string(&scene_path).expect("scene document should be readable");

        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        fs::write(
            &scene_path,
            original_scene.replace("PLAYGROUND SIDESCROLLER", "PLAYGROUND SIDESCROLLER LIVE"),
        )
        .expect("scene document should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect sidescroller scene changes");

        assert_eq!(updated.active_scene.as_deref(), Some("vertical-slice"));
        assert!(updated.console_output.iter().any(|line| {
            line.contains(
                "detected scene document change for `playground-sidescroller:vertical-slice`",
            )
        }));
        assert!(
            updated
                .processed_scene_commands
                .iter()
                .any(|command| command == "scene.reload_active")
        );

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let title = ui_scene
            .commands()
            .into_iter()
            .find(|command| command.entity_name == "playground-sidescroller-hud")
            .and_then(|command| {
                command.document.root.children.into_iter().find_map(|node| {
                    match (node.id.as_deref(), node.kind) {
                        (Some("title"), amigo_ui::UiNodeKind::Text { content, .. }) => {
                            Some(content)
                        }
                        _ => None,
                    }
                })
            });
        assert_eq!(title.as_deref(), Some("PLAYGROUND SIDESCROLLER LIVE"));
    }

    #[test]
    fn runtime_detects_sidescroller_visual_asset_metadata_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root(
            "sidescroller-player-hot-reload",
            &["core", "playground-sidescroller"],
        );
        let asset_path = temp_mods
            .join("playground-sidescroller")
            .join("textures")
            .join("player.yml");
        let original_asset =
            fs::read_to_string(&asset_path).expect("player metadata should be readable");

        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        fs::write(
            &asset_path,
            original_asset.replace(
                "label: Sidescroller Player (Kenney)",
                "label: Sidescroller Player Reloaded",
            ),
        )
        .expect("player metadata should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect player metadata changes");

        assert!(updated.console_output.iter().any(|line| {
            line.contains("detected asset change for `playground-sidescroller/textures/player`")
        }));
        assert!(
            updated
                .processed_script_events
                .iter()
                .any(|event| event.starts_with("hot-reload.asset-changed("))
        );
        assert!(
            updated
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/textures/player (sprite-sheet-2d)")
        );

        let assets = runtime
            .resolve::<AssetCatalog>()
            .expect("asset catalog should exist");
        let prepared = assets
            .prepared_asset(&AssetKey::new("playground-sidescroller/textures/player"))
            .expect("player prepared asset should exist after reload");
        assert_eq!(
            prepared.label.as_deref(),
            Some("Sidescroller Player Reloaded")
        );
    }

    #[test]
    fn runtime_detects_sidescroller_tile_ruleset_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root(
            "sidescroller-ruleset-hot-reload",
            &["core", "playground-sidescroller"],
        );
        let asset_path = temp_mods
            .join("playground-sidescroller")
            .join("tilesets")
            .join("platformer-rules.yml");
        let original_asset =
            fs::read_to_string(&asset_path).expect("ruleset metadata should be readable");

        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let initial_left_cap =
            first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::LeftCap)
                .expect("initial left cap tile id should exist");
        assert_eq!(initial_left_cap, 1);

        fs::write(
            &asset_path,
            original_asset.replace("left_cap: 1", "left_cap: 0"),
        )
        .expect("ruleset metadata should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect ruleset metadata changes");

        assert!(updated.console_output.iter().any(|line| {
            line.contains(
                "detected asset change for `playground-sidescroller/tilesets/platformer-rules`",
            )
        }));
        assert!(updated.prepared_assets.iter().any(|asset| {
            asset == "playground-sidescroller/tilesets/platformer-rules (tile-ruleset-2d)"
        }));

        let updated_left_cap =
            first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::LeftCap)
                .expect("updated left cap tile id should exist");
        assert_eq!(updated_left_cap, 0);
    }

    #[test]
    fn runtime_detects_sidescroller_generated_audio_metadata_changes_through_hot_reload_service() {
        let temp_mods = copied_mods_root(
            "sidescroller-audio-hot-reload",
            &["core", "playground-sidescroller"],
        );
        let asset_path = temp_mods
            .join("playground-sidescroller")
            .join("audio")
            .join("proximity-beep.yml");
        let original_asset =
            fs::read_to_string(&asset_path).expect("audio metadata should be readable");

        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        fs::write(
            &asset_path,
            original_asset.replace(
                "label: Sidescroller Proximity Beep",
                "label: Sidescroller Proximity Beep Reloaded",
            ),
        )
        .expect("audio metadata should be updated");

        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should detect audio metadata changes");

        assert!(updated.console_output.iter().any(|line| {
            line.contains(
                "detected asset change for `playground-sidescroller/audio/proximity-beep`",
            )
        }));
        assert!(
            updated
                .prepared_assets
                .iter()
                .any(|asset| asset
                    == "playground-sidescroller/audio/proximity-beep (generated-audio)")
        );

        let assets = runtime
            .resolve::<AssetCatalog>()
            .expect("asset catalog should exist");
        let prepared = assets
            .prepared_asset(&AssetKey::new(
                "playground-sidescroller/audio/proximity-beep",
            ))
            .expect("audio prepared asset should exist after reload");
        assert_eq!(
            prepared.label.as_deref(),
            Some("Sidescroller Proximity Beep Reloaded")
        );
    }

    #[test]
    fn playground_3d_main_scene_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
                .with_startup_mod("playground-3d")
                .with_startup_scene("hello-world-cube")
                .with_dev_mode(true),
        )
        .expect("3d main playground bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("hello-world-cube"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/hello-world-cube/scene.yml")
        );
        assert!(
            summary
                .mesh_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-cube")
        );
        assert!(
            summary
                .material_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-cube")
        );
        assert!(
            summary
                .text_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-hello")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/cube (mesh-3d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/materials/cube-material (material-3d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/fonts/debug-3d (font-3d)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn playground_2d_screen_space_preview_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("screen-space-preview")
                .with_dev_mode(true),
        )
        .expect("screen-space preview bootstrap should succeed");

        assert_eq!(
            summary.active_scene.as_deref(),
            Some("screen-space-preview")
        );
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/screen-space-preview/scene.yml")
        );
        assert!(
            summary
                .loaded_scene_document
                .as_ref()
                .expect("loaded scene document should exist")
                .component_kinds
                .iter()
                .any(|kind| kind == "UiDocument x1")
        );
        assert!(
            summary
                .ui_entities
                .iter()
                .any(|entity| entity == "playground-2d-ui-preview")
        );
        assert!(
            summary
                .sprite_entities_2d
                .iter()
                .any(|entity| entity == "playground-2d-ui-preview-square")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d/fonts/debug-ui (font-2d)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn playground_sidescroller_vertical_slice_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller vertical slice bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("vertical-slice"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/vertical-slice/scene.yml")
        );
        let component_kinds = &summary
            .loaded_scene_document
            .as_ref()
            .expect("loaded scene document should exist")
            .component_kinds;
        assert!(component_kinds.iter().any(|kind| kind == "TileMap2D x1"));
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "KinematicBody2D x1")
        );
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "AabbCollider2D x1")
        );
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "PlatformerController2D x1")
        );
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "CameraFollow2D x1")
        );
        assert!(component_kinds.iter().any(|kind| kind == "Parallax2D x2"));
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "TileMapMarker2D x3")
        );
        assert!(component_kinds.iter().any(|kind| kind == "Trigger2D x2"));
        assert!(component_kinds.iter().any(|kind| kind == "UiDocument x1"));

        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-far")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-near")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-player")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-tilemap")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-hud")
        );
        let player_transform = _runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-player")
            .expect("player transform should exist after tilemap marker anchoring");
        assert!(
            player_transform.translation.x > 0.0 && player_transform.translation.y > 0.0,
            "player should be anchored to a non-zero tilemap marker position"
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/backgrounds/far (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/backgrounds/near (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/textures/player (sprite-sheet-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/textures/coin (sprite-sheet-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/textures/finish (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/tilesets/platformer (tileset-2d)")
        );
        assert!(summary.prepared_assets.iter().any(|asset| {
            asset == "playground-sidescroller/tilesets/platformer-rules (tile-ruleset-2d)"
        }));
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/fonts/debug-ui (font-2d)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn playground_sidescroller_tilemap_bootstraps_without_ruleset() {
        let temp_mods = copied_mods_root(
            "sidescroller-no-ruleset",
            &["core", "playground-sidescroller"],
        );
        let scene_path = temp_mods
            .join("playground-sidescroller")
            .join("scenes")
            .join("vertical-slice")
            .join("scene.yml");
        let original_scene =
            fs::read_to_string(&scene_path).expect("sidescroller scene should be readable");
        let updated_scene = original_scene
            .lines()
            .filter(|line| {
                !line.contains("ruleset: playground-sidescroller/tilesets/platformer-rules")
            })
            .collect::<Vec<_>>()
            .join("\n");
        fs::write(&scene_path, updated_scene).expect("scene without ruleset should be writable");

        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(temp_mods)
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap without ruleset should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("vertical-slice"));
        assert!(summary.failed_assets.is_empty());

        let tilemap_command = runtime
            .resolve::<TileMap2dSceneService>()
            .expect("tilemap scene service should exist")
            .commands()
            .into_iter()
            .find(|command| command.entity_name == "playground-sidescroller-tilemap")
            .expect("tilemap command should exist");
        assert!(tilemap_command.tilemap.ruleset.is_none());
        assert!(tilemap_command.tilemap.resolved.is_none());
    }

    #[test]
    fn bootstrap_reports_task_003_scaffold_plugins_and_capabilities() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned()])
                .with_startup_mod("core")
                .with_startup_scene("bootstrap")
                .with_dev_mode(true),
        )
        .expect("core bootstrap should succeed");

        for capability in [
            "physics_2d",
            "tilemap_2d",
            "platformer_2d",
            "audio_api",
            "generated_audio",
            "audio_mix",
            "audio_output",
        ] {
            assert!(
                summary.capabilities.iter().any(|entry| entry == capability),
                "missing capability `{capability}` in bootstrap summary"
            );
        }

        for plugin in [
            "amigo-2d-physics",
            "amigo-2d-tilemap",
            "amigo-2d-platformer",
            "amigo-audio-api",
            "amigo-audio-generated",
            "amigo-audio-mixer",
            "amigo-audio-output",
        ] {
            assert!(
                summary.plugins.iter().any(|entry| entry == plugin),
                "missing plugin `{plugin}` in bootstrap summary"
            );
        }
    }

    #[test]
    fn handle_script_command_updates_ui_state() {
        let scene_command_queue = SceneCommandQueue::default();
        let script_event_queue = ScriptEventQueue::default();
        let dev_console_state = DevConsoleState::default();
        let asset_catalog = AssetCatalog::default();
        let ui_state = UiStateService::default();
        let audio_command_queue = AudioCommandQueue::default();
        let audio_scene_service = AudioSceneService::default();
        let diagnostics = RuntimeDiagnostics::default();
        let launch_selection = LaunchSelection::new(
            Some("playground-2d".to_owned()),
            Some("screen-space-preview".to_owned()),
            Vec::new(),
            true,
        );

        handle_script_command(
            ScriptCommand::ui_set_text("playground-2d-ui-preview.subtitle", "Updated from Rhai"),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::ui_set_value("playground-2d-ui-preview.hp-bar", 0.5),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::ui_hide("playground-2d-ui-preview.root"),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::ui_disable(
                "playground-2d-ui-preview.root.control-card.button-row.repair-button",
            ),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::ui_enable(
                "playground-2d-ui-preview.root.control-card.button-row.repair-button",
            ),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );

        assert_eq!(
            ui_state
                .text_override("playground-2d-ui-preview.subtitle")
                .as_deref(),
            Some("Updated from Rhai")
        );
        assert_eq!(
            ui_state.value_override("playground-2d-ui-preview.hp-bar"),
            Some(0.5)
        );
        assert!(!ui_state.is_visible("playground-2d-ui-preview.root"));
        assert!(
            ui_state
                .is_enabled("playground-2d-ui-preview.root.control-card.button-row.repair-button")
        );
    }

    #[test]
    fn handle_script_command_queues_and_processes_audio_state() {
        let scene_command_queue = SceneCommandQueue::default();
        let script_event_queue = ScriptEventQueue::default();
        let dev_console_state = DevConsoleState::default();
        let asset_catalog = AssetCatalog::default();
        let ui_state = UiStateService::default();
        let audio_command_queue = AudioCommandQueue::default();
        let audio_scene_service = AudioSceneService::default();
        let audio_state = AudioStateService::default();
        let diagnostics = RuntimeDiagnostics::default();
        let launch_selection = LaunchSelection::new(
            Some("playground-sidescroller".to_owned()),
            Some("vertical-slice".to_owned()),
            Vec::new(),
            true,
        );

        handle_script_command(
            ScriptCommand::audio_play("jump"),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::audio_start_realtime("proximity-beep"),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );
        handle_script_command(
            ScriptCommand::audio_set_param("proximity-beep", "distance", 128.0),
            &scene_command_queue,
            &script_event_queue,
            &dev_console_state,
            &asset_catalog,
            &ui_state,
            &audio_command_queue,
            &audio_scene_service,
            &diagnostics,
            &launch_selection,
        );

        let commands = audio_command_queue.drain();
        assert_eq!(commands.len(), 3);
        assert_eq!(audio_scene_service.clips().len(), 2);

        for command in commands {
            process_audio_command(command, &audio_state, &dev_console_state);
        }

        assert!(audio_state.playing_sources().contains_key("proximity-beep"));
        assert_eq!(audio_state.drain_runtime_commands().len(), 3);
        assert_eq!(
            audio_state
                .source_params()
                .get("proximity-beep")
                .and_then(|params| params.get("distance"))
                .copied(),
            Some(128.0)
        );
    }

    #[test]
    fn interactive_host_handler_applies_arrow_input_to_playground_3d_cube() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
                .with_startup_mod("playground-3d")
                .with_startup_scene("hello-world-cube")
                .with_dev_mode(true),
        )
        .expect("3d main playground bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let initial = scene
            .transform_of("playground-3d-cube")
            .expect("playground 3d cube should exist");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            })
            .expect("input event should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");

        let updated = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-3d-cube")
            .expect("playground 3d cube should exist after update");

        assert!(
            updated.rotation_euler.y > initial.rotation_euler.y,
            "Right arrow should rotate the 3D cube around the Y axis"
        );
    }

    #[test]
    fn interactive_host_handler_moves_sidescroller_player_right() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let initial = scene
            .transform_of("playground-sidescroller-player")
            .expect("sidescroller player should exist");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            })
            .expect("input event should be accepted");

        for _ in 0..8 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime tick should succeed");
        }

        let updated = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-player")
            .expect("sidescroller player should exist after update");

        assert!(
            updated.translation.x > initial.translation.x,
            "Right arrow should move the sidescroller player to the right"
        );
    }

    #[test]
    fn interactive_host_handler_moves_sidescroller_camera_with_player() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let initial = scene
            .transform_of("playground-sidescroller-camera")
            .expect("sidescroller camera should exist");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            })
            .expect("input event should be accepted");

        for _ in 0..8 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime tick should succeed");
        }

        let updated = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-camera")
            .expect("sidescroller camera should exist after update");

        assert!(
            updated.translation.x > initial.translation.x,
            "camera follow should move the sidescroller camera to the right with the player"
        );
    }

    #[test]
    fn interactive_host_handler_applies_sidescroller_parallax() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let initial_camera = scene
            .transform_of("playground-sidescroller-camera")
            .expect("sidescroller camera should exist");
        let initial_far = scene
            .transform_of("playground-sidescroller-background-far")
            .expect("far background should exist");
        let initial_near = scene
            .transform_of("playground-sidescroller-background-near")
            .expect("near background should exist");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            })
            .expect("input event should be accepted");

        for _ in 0..12 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime tick should succeed");
        }

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let updated_camera = scene
            .transform_of("playground-sidescroller-camera")
            .expect("sidescroller camera should exist after update");
        let updated_far = scene
            .transform_of("playground-sidescroller-background-far")
            .expect("far background should exist after update");
        let updated_near = scene
            .transform_of("playground-sidescroller-background-near")
            .expect("near background should exist after update");

        let far_screen_delta = (updated_far.translation.x - updated_camera.translation.x)
            - (initial_far.translation.x - initial_camera.translation.x);
        let near_screen_delta = (updated_near.translation.x - updated_camera.translation.x)
            - (initial_near.translation.x - initial_camera.translation.x);

        assert!(
            far_screen_delta.abs() > 0.0,
            "far background should visibly shift on screen"
        );
        assert!(
            near_screen_delta.abs() > far_screen_delta.abs(),
            "near background should move more on screen than the far layer"
        );
    }

    #[test]
    fn interactive_host_handler_advances_sidescroller_sprite_frames() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        let sprites = handler
            .runtime
            .resolve::<SpriteSceneService>()
            .expect("sprite scene service should exist");
        assert_eq!(sprites.frame_of("playground-sidescroller-coin"), Some(0));
        assert_eq!(sprites.frame_of("playground-sidescroller-player"), Some(0));

        for _ in 0..12 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime tick should succeed");
        }

        let sprites = handler
            .runtime
            .resolve::<SpriteSceneService>()
            .expect("sprite scene service should exist");
        assert_ne!(
            sprites.frame_of("playground-sidescroller-coin"),
            Some(0),
            "coin should advance its spritesheet frame over time"
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Right,
                pressed: true,
            })
            .expect("input event should be accepted");

        for _ in 0..2 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime tick should succeed");
        }

        let sprites = handler
            .runtime
            .resolve::<SpriteSceneService>()
            .expect("sprite scene service should exist");
        assert!(
            matches!(
                sprites.frame_of("playground-sidescroller-player"),
                Some(1 | 2)
            ),
            "player should switch into run frames while moving right"
        );
    }

    #[test]
    fn interactive_host_handler_collects_sidescroller_coin_and_updates_hud() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let coin = scene
            .transform_of("playground-sidescroller-coin")
            .expect("coin should exist");
        assert!(
            scene.set_transform("playground-sidescroller-player", coin),
            "player transform should be repositioned onto the coin"
        );

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");

        let ui_state = handler
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.score")
                .as_deref(),
            Some("Coins: 1")
        );
        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.message")
                .as_deref(),
            Some("COIN COLLECTED")
        );

        let audio_state = handler
            .runtime
            .resolve::<AudioStateService>()
            .expect("audio state service should exist");
        assert!(
            audio_state
                .processed_commands()
                .iter()
                .any(|command| matches!(
                    command,
                    AudioCommand::PlayOnce { clip }
                        if clip.as_str() == "playground-sidescroller/audio/coin"
                ))
        );
        let audio_mixer = handler
            .runtime
            .resolve::<AudioMixerService>()
            .expect("audio mixer service should exist");
        assert!(audio_mixer.frames().iter().any(|frame| {
            frame
                .sources
                .iter()
                .any(|source| source == "playground-sidescroller/audio/coin")
        }));
    }

    #[test]
    fn interactive_host_handler_reaching_finish_updates_message_and_audio_state() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let scene = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let finish = scene
            .transform_of("playground-sidescroller-finish")
            .expect("finish should exist");
        assert!(
            scene.set_transform("playground-sidescroller-player", finish),
            "player transform should be repositioned onto the finish trigger"
        );

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");

        let ui_state = handler
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.message")
                .as_deref(),
            Some("LEVEL COMPLETE")
        );

        let audio_state = handler
            .runtime
            .resolve::<AudioStateService>()
            .expect("audio state service should exist");
        assert!(
            audio_state
                .processed_commands()
                .iter()
                .any(|command| matches!(
                    command,
                    AudioCommand::PlayOnce { clip }
                        if clip.as_str() == "playground-sidescroller/audio/level-complete"
                ))
        );
        assert!(
            audio_state
                .processed_commands()
                .iter()
                .any(|command| matches!(
                    command,
                    AudioCommand::StopSource { source } if source.as_str() == "proximity-beep"
                ))
        );
        assert!(
            audio_state
                .playing_sources()
                .iter()
                .all(|(source_id, _)| source_id != "proximity-beep"),
            "finish event should stop the realtime proximity source"
        );
        let audio_mixer = handler
            .runtime
            .resolve::<AudioMixerService>()
            .expect("audio mixer service should exist");
        assert!(audio_mixer.frames().iter().any(|frame| {
            frame
                .sources
                .iter()
                .any(|source| source == "playground-sidescroller/audio/level-complete")
        }));
        assert!(
            audio_mixer
                .active_realtime_sources()
                .iter()
                .all(|source| source != "proximity-beep")
        );
    }

    #[test]
    fn interactive_host_handler_player_jump_updates_hud_and_audio() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-sidescroller".to_owned(),
                ])
                .with_startup_mod("playground-sidescroller")
                .with_startup_scene("vertical-slice")
                .with_dev_mode(true),
        )
        .expect("sidescroller bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        for _ in 0..24 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime settle tick should succeed");
        }

        let before = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-player")
            .expect("player should exist");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("jump input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime jump tick should succeed");

        let after = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-player")
            .expect("player should exist after jump");
        assert!(
            after.translation.y > before.translation.y,
            "jump should move the player upward"
        );

        let ui_state = handler
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.message")
                .as_deref(),
            Some("JUMP")
        );

        let audio_state = handler
            .runtime
            .resolve::<AudioStateService>()
            .expect("audio state service should exist");
        assert!(
            audio_state
                .processed_commands()
                .iter()
                .any(|command| matches!(
                    command,
                    AudioCommand::PlayOnce { clip }
                        if clip.as_str() == "playground-sidescroller/audio/jump"
                ))
        );
        let audio_mixer = handler
            .runtime
            .resolve::<AudioMixerService>()
            .expect("audio mixer service should exist");
        assert!(audio_mixer.frames().iter().any(|frame| {
            frame
                .sources
                .iter()
                .any(|source| source == "playground-sidescroller/audio/jump")
        }));
    }

    #[test]
    fn interactive_host_handler_can_switch_playground_2d_scenes_through_script_input() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("hello-world-square")
                .with_dev_mode(true),
        )
        .expect("2d square playground bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            })
            .expect("input event should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");

        let updated_scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .selected_scene()
            .map(|scene| scene.as_str().to_owned());

        assert_eq!(
            updated_scene.as_deref(),
            Some("hello-world-spritesheet"),
            "ArrowUp on the square scene should switch to the spritesheet scene through Rhai"
        );
    }

    #[test]
    fn interactive_host_handler_can_return_from_spritesheet_through_yaml_transition() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("hello-world-spritesheet")
                .with_dev_mode(true),
        )
        .expect("2d spritesheet playground bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Down,
                pressed: true,
            })
            .expect("input event should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick should succeed");

        let updated_scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .selected_scene()
            .map(|scene| scene.as_str().to_owned());

        assert_eq!(
            updated_scene.as_deref(),
            Some("hello-world-square"),
            "ArrowDown on the spritesheet scene should emit a script event that triggers the YAML transition"
        );
    }

    #[test]
    fn hosted_playground_mods_use_interactive_handler_even_without_dev_flag() {
        let core_options = BootstrapOptions::new(mods_root())
            .with_startup_mod("core")
            .with_startup_scene("bootstrap");
        let playground_options = BootstrapOptions::new(mods_root())
            .with_startup_mod("playground-3d")
            .with_startup_scene("hello-world-cube");

        assert!(!super::should_use_interactive_host(&core_options));
        assert!(super::should_use_interactive_host(&playground_options));
    }

    #[test]
    fn playground_3d_mesh_scene_populates_3d_domain_and_assets() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
                .with_startup_mod("playground-3d")
                .with_startup_scene("mesh-lab")
                .with_dev_mode(true),
        )
        .expect("3d mesh playground bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("mesh-lab"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/mesh-lab/scene.yml")
        );
        assert!(
            summary
                .processed_scene_commands
                .iter()
                .any(|command| command.starts_with("scene.3d.mesh("))
        );
        assert!(
            summary
                .registered_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/probe")
        );
        assert!(
            summary
                .loaded_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/probe")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/probe (mesh-3d)")
        );
        assert!(summary.failed_assets.is_empty());
        assert!(summary.pending_asset_loads.is_empty());
        assert!(
            summary
                .mesh_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-probe")
        );
        assert!(summary.material_entities_3d.is_empty());
    }

    #[test]
    fn playground_3d_material_scene_populates_3d_material_domain_and_assets() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-3d".to_owned()])
                .with_startup_mod("playground-3d")
                .with_startup_scene("material-lab")
                .with_dev_mode(true),
        )
        .expect("3d material playground bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("material-lab"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/material-lab/scene.yml")
        );
        assert!(
            summary
                .processed_scene_commands
                .iter()
                .any(|command| command.starts_with("scene.3d.material("))
        );
        assert!(
            summary
                .registered_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/material-probe")
        );
        assert!(
            summary
                .registered_assets
                .iter()
                .any(|asset| asset == "playground-3d/materials/debug-surface")
        );
        assert!(
            summary
                .loaded_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/material-probe")
        );
        assert!(
            summary
                .loaded_assets
                .iter()
                .any(|asset| asset == "playground-3d/materials/debug-surface")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/meshes/material-probe (mesh-3d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-3d/materials/debug-surface (material-3d)")
        );
        assert!(summary.failed_assets.is_empty());
        assert!(summary.pending_asset_loads.is_empty());
        assert!(
            summary
                .mesh_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-material-probe")
        );
        assert!(
            summary
                .material_entities_3d
                .iter()
                .any(|entity| entity == "playground-3d-material-probe")
        );
    }

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

    #[test]
    fn resolve_existing_asset_path_prefers_metadata_candidates() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("time should be after epoch")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("amigo-asset-path-{unique}"));
        fs::create_dir_all(root.join("textures")).expect("temp textures dir should exist");

        let metadata_path = root.join("textures").join("player.yml");
        fs::write(&metadata_path, "kind: sprite-sheet-2d\nimage: player.png\n")
            .expect("metadata file should be created");

        let resolved =
            super::resolve_existing_asset_path(root.join("textures").join("player"), "test/player")
                .expect("metadata candidate should resolve");

        assert_eq!(resolved, metadata_path);
    }
}
