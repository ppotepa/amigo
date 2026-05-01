use std::any::type_name;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

use amigo_2d_motion::Motion2dSceneService;
use amigo_2d_particles::{Particle2dDomainInfo, Particle2dSceneService};
use amigo_2d_physics::{
    Physics2dDomainInfo, Physics2dSceneService, move_and_collide, overlaps_trigger_with_translation,
};
use amigo_2d_sprite::{SpriteDomainInfo, SpriteSceneService, SpriteSheet};
use amigo_2d_text::{Text2dDomainInfo, Text2dSceneService};
use amigo_2d_tilemap::{TileMap2dDomainInfo, TileMap2dSceneService, marker_cells};
use amigo_2d_vector::{VectorDomainInfo, VectorSceneService};
use amigo_3d_material::{MaterialDomainInfo, MaterialSceneService};
use amigo_3d_mesh::{MeshDomainInfo, MeshSceneService};
use amigo_3d_text::{Text3dDomainInfo, Text3dSceneService};
use amigo_app_host_api::{
    HostConfig, HostControl, HostExitStrategy, HostHandler, HostLifecycleEvent,
};
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    prepare_asset_from_contents,
};
use amigo_audio_api::{
    AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioDomainInfo, AudioPlaybackMode,
    AudioSceneService, AudioSourceId, AudioStateService,
};
use amigo_audio_generated::GeneratedAudioDomainInfo;
use amigo_audio_mixer::{AudioMixerDomainInfo, AudioMixerService};
use amigo_audio_output::{
    AudioOutputBackendService, AudioOutputDomainInfo, AudioOutputStartStatus,
};
use amigo_behavior::BehaviorSceneService;
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_event_pipeline::EventPipelineService;
use amigo_file_watch_api::FileWatchService;
use amigo_hot_reload::{AssetWatch, HotReloadService, HotReloadWatchKind, SceneDocumentWatch};
use amigo_input_actions::InputActionService;
use amigo_input_api::{InputEvent, InputServiceInfo, InputState, KeyCode};
use amigo_math::Vec2;
use amigo_modding::{ModCatalog, ModScriptMode};
use amigo_render_api::RenderBackendInfo;
use amigo_render_wgpu::{
    UiLayoutNode as OverlayUiLayoutNode, UiOverlayDocument, UiOverlayLayer, UiOverlayNode,
    UiOverlayNodeKind, UiOverlayStyle, UiOverlayViewport, UiOverlayViewportScaling, UiTextAnchor,
    UiViewportSize, WgpuRenderBackend, WgpuSceneRenderer, WgpuSurfaceState, build_ui_layout_tree,
};
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    CameraFollow2dSceneCommand, CameraFollow2dSceneService, EntityPoolSceneService,
    HydratedSceneState, LifetimeSceneService, Material3dSceneCommand, Mesh3dSceneCommand,
    Parallax2dSceneCommand, Parallax2dSceneService, SceneCommand, SceneCommandQueue, SceneEvent,
    SceneEventQueue, SceneHydrationPlan, SceneKey, SceneService, SceneTransitionPlan,
    SceneTransitionService, Sprite2dSceneCommand, Text2dSceneCommand, Text3dSceneCommand,
};
use amigo_scripting_api::{
    DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptCommandQueue, ScriptComponentDefinition,
    ScriptComponentService, ScriptEvent, ScriptEventQueue, ScriptLifecycleState, ScriptParams,
    ScriptRuntimeInfo, ScriptRuntimeService, ScriptTraceService, ScriptValue,
};
use amigo_ui::{
    UiDocument as RuntimeUiDocument, UiDomainInfo, UiDrawCommand, UiEventBinding, UiInputService,
    UiLayer as RuntimeUiLayer, UiModelBinding, UiModelBindingKind, UiModelBindingService,
    UiNode as RuntimeUiNode, UiNodeKind as RuntimeUiNodeKind, UiSceneService, UiStateService,
    UiStateSnapshot, UiStyle as RuntimeUiStyle, UiTab as RuntimeUiTab, UiTarget as RuntimeUiTarget,
    UiTextAlign as RuntimeUiTextAlign, UiTheme, UiThemePalette, UiThemeService,
    UiViewportScaling as RuntimeUiViewportScaling,
};
use amigo_window_api::{WindowDescriptor, WindowEvent, WindowServiceInfo, WindowSurfaceHandles};

mod app_helpers;
mod assets;
mod bootstrap;
mod diagnostics;
mod event_pipeline;
mod host_runtime;
mod launch_selection;
mod orchestration;
mod particle_presets;
mod render_runtime;
mod runtime_context;
mod scene_runtime;
mod script_runtime;
mod scripting_runtime;
mod summary;
mod systems;
mod ui_runtime;

pub use bootstrap::{
    bootstrap_default, bootstrap_with_options, run_default, run_hosted_once,
    run_hosted_with_options, run_with_options,
};
pub(crate) use diagnostics::RuntimeDiagnosticsPlugin;
pub(crate) use host_runtime::{InteractiveRuntimeHostHandler, SummaryHostHandler};
pub(crate) use launch_selection::LaunchSelectionPlugin;
use runtime_context::{required, required_from_registry};
use summary::refresh_runtime_summary;

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
    pub vector_entities_2d: Vec<String>,
    pub mesh_entities_3d: Vec<String>,
    pub material_entities_3d: Vec<String>,
    pub text_entities_3d: Vec<String>,
    pub ui_entities: Vec<String>,
    pub audio_clips: Vec<String>,
    pub audio_sources: Vec<String>,
    pub pending_audio_runtime_commands: Vec<String>,
    pub audio_master_volume: f32,
    pub mixed_audio_frame_count: usize,
    pub active_realtime_audio_sources: Vec<String>,
    pub audio_output_started: bool,
    pub audio_output_device: Option<String>,
    pub audio_output_buffered_samples: usize,
    pub audio_output_last_error: Option<String>,
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

impl Display for BootstrapSummary {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        writeln!(f, "Amigo bootstrap")?;
        writeln!(f, "window backend: {}", self.window_backend)?;
        writeln!(f, "input backend: {}", self.input_backend)?;
        writeln!(f, "render backend: {}", self.render_backend)?;
        writeln!(f, "script backend: {}", self.script_backend)?;
        writeln!(f, "file watch backend: {}", self.file_watch_backend)?;
        writeln!(
            f,
            "mods: {}",
            app_helpers::display_string_list(&self.loaded_mods)
        )?;
        writeln!(
            f,
            "scripts: {}",
            app_helpers::display_executed_scripts(&self.executed_scripts)
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
                .map(|document| app_helpers::display_string_list(&document.entity_names))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document components: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| app_helpers::display_string_list(&document.component_kinds))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene document transitions: {}",
            self.loaded_scene_document
                .as_ref()
                .map(|document| app_helpers::display_string_list(&document.transition_ids))
                .unwrap_or_else(|| "none".to_owned())
        )?;
        writeln!(
            f,
            "scene entities: {}",
            app_helpers::display_string_list(&self.scene_entities)
        )?;
        writeln!(
            f,
            "registered assets: {}",
            app_helpers::display_string_list(&self.registered_assets)
        )?;
        writeln!(
            f,
            "loaded assets: {}",
            app_helpers::display_string_list(&self.loaded_assets)
        )?;
        writeln!(
            f,
            "prepared assets: {}",
            app_helpers::display_string_list(&self.prepared_assets)
        )?;
        writeln!(
            f,
            "failed assets: {}",
            app_helpers::display_string_list(&self.failed_assets)
        )?;
        writeln!(
            f,
            "pending asset loads: {}",
            app_helpers::display_string_list(&self.pending_asset_loads)
        )?;
        writeln!(
            f,
            "watched reload targets: {}",
            app_helpers::display_string_list(&self.watched_reload_targets)
        )?;
        writeln!(
            f,
            "2d sprite entities: {}",
            app_helpers::display_string_list(&self.sprite_entities_2d)
        )?;
        writeln!(
            f,
            "2d text entities: {}",
            app_helpers::display_string_list(&self.text_entities_2d)
        )?;
        writeln!(
            f,
            "2d vector entities: {}",
            app_helpers::display_string_list(&self.vector_entities_2d)
        )?;
        writeln!(
            f,
            "3d mesh entities: {}",
            app_helpers::display_string_list(&self.mesh_entities_3d)
        )?;
        writeln!(
            f,
            "3d material entities: {}",
            app_helpers::display_string_list(&self.material_entities_3d)
        )?;
        writeln!(
            f,
            "3d text entities: {}",
            app_helpers::display_string_list(&self.text_entities_3d)
        )?;
        writeln!(
            f,
            "ui entities: {}",
            app_helpers::display_string_list(&self.ui_entities)
        )?;
        writeln!(
            f,
            "audio clips: {}",
            app_helpers::display_string_list(&self.audio_clips)
        )?;
        writeln!(
            f,
            "audio sources: {}",
            app_helpers::display_string_list(&self.audio_sources)
        )?;
        writeln!(
            f,
            "audio runtime commands: {}",
            app_helpers::display_string_list(&self.pending_audio_runtime_commands)
        )?;
        writeln!(f, "audio master volume: {}", self.audio_master_volume)?;
        writeln!(f, "audio mix frames: {}", self.mixed_audio_frame_count)?;
        writeln!(
            f,
            "audio realtime sources: {}",
            app_helpers::display_string_list(&self.active_realtime_audio_sources)
        )?;
        writeln!(
            f,
            "audio output: started={} device={} buffered_samples={} last_error={}",
            self.audio_output_started,
            self.audio_output_device.as_deref().unwrap_or("none"),
            self.audio_output_buffered_samples,
            self.audio_output_last_error.as_deref().unwrap_or("none")
        )?;
        writeln!(
            f,
            "script commands: {}",
            app_helpers::display_string_list(&self.processed_script_commands)
        )?;
        writeln!(
            f,
            "audio commands: {}",
            app_helpers::display_string_list(&self.processed_audio_commands)
        )?;
        writeln!(
            f,
            "scene commands: {}",
            app_helpers::display_string_list(&self.processed_scene_commands)
        )?;
        writeln!(
            f,
            "script events: {}",
            app_helpers::display_string_list(&self.processed_script_events)
        )?;
        writeln!(
            f,
            "console commands: {}",
            app_helpers::display_string_list(&self.console_commands)
        )?;
        writeln!(
            f,
            "console output: {}",
            app_helpers::display_string_list(&self.console_output)
        )?;
        writeln!(
            f,
            "capabilities: {}",
            app_helpers::display_string_list(&self.capabilities)
        )?;
        writeln!(
            f,
            "plugins: {}",
            app_helpers::display_string_list(&self.plugins)
        )?;
        write!(
            f,
            "services: {}",
            app_helpers::display_string_list(&self.services)
        )
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

fn scene_ids_for_launch_selection(
    mod_catalog: &ModCatalog,
    launch_selection: &LaunchSelection,
) -> Vec<String> {
    launch_selection::scene_ids_for_launch_selection(mod_catalog, launch_selection)
}

fn next_scene_id(scene_ids: &[String], active_scene: Option<&str>, step: isize) -> Option<String> {
    launch_selection::next_scene_id(scene_ids, active_scene, step)
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
    use amigo_assets::{AssetCatalog, AssetKey, AssetManifest, AssetSourceKind};
    use amigo_audio_api::{AudioCommand, AudioCommandQueue, AudioSceneService, AudioStateService};
    use amigo_audio_mixer::AudioMixerService;
    use amigo_core::RuntimeDiagnostics;
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
        BootstrapOptions, InteractiveRuntimeHostHandler, OverlayUiLayoutNode,
        bootstrap_with_options, next_scene_id, refresh_runtime_summary,
        scene_ids_for_launch_selection,
    };
    use crate::orchestration::{process_audio_command, process_placeholder_bridges};
    use crate::script_runtime;
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
    fn playground_2d_asteroids_main_menu_bootstraps() {
        let (_runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-asteroids".to_owned(),
                ])
                .with_startup_mod("playground-2d-asteroids")
                .with_startup_scene("main-menu")
                .with_dev_mode(true),
        )
        .expect("asteroids main menu bootstrap should succeed");

        assert_eq!(summary.active_scene.as_deref(), Some("main-menu"));
        assert_eq!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document.relative_path.to_string_lossy().replace('\\', "/"))
                .as_deref(),
            Some("scenes/main-menu/scene.yml")
        );
        assert!(
            summary
                .loaded_scene_document
                .as_ref()
                .map(|document| document
                    .component_kinds
                    .iter()
                    .any(|kind| kind.starts_with("UiDocument x")))
                .unwrap_or(false)
        );
        assert!(summary.failed_assets.is_empty());
        assert!(summary.pending_asset_loads.is_empty());
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-2d-asteroids/fonts/debug-ui (font-2d)")
        );
        assert!(
            summary
                .ui_entities
                .iter()
                .any(|entity| entity == "playground-2d-asteroids-main-menu")
        );
    }

    #[test]
    fn playground_hud_ui_showcase_bootstraps() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase bootstrap should succeed");

        let themes = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");

        assert_eq!(summary.active_scene.as_deref(), Some("showcase"));
        assert_eq!(themes.active_theme_id().as_deref(), Some("space_dark"));
        assert!(summary.failed_assets.is_empty());
        assert!(
            summary
                .ui_entities
                .iter()
                .any(|entity| entity == "playground-hud-ui-showcase")
        );
    }

    #[test]
    fn playground_hud_ui_click_switches_theme() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase bootstrap should succeed");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1280.0, 720.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);
        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let button = find_path_ending(&layout, ".theme-clean-dev")
            .expect("clean theme button should be in layout");
        let button_path = button.path.clone();
        assert!(
            showcase.click_bindings.contains_key(&button_path),
            "button path should have click binding: {button_path}; known={:?}",
            showcase.click_bindings.keys().collect::<Vec<_>>()
        );
        let click_x = button.rect.x + button.rect.width * 0.5;
        let click_y = button.rect.y + button.rect.height * 0.5;

        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(click_x, click_y);
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime).expect("ui press should be processed");
        assert!(
            ui_state.background_override(&button_path).is_some(),
            "pressing a button should apply a transient pressed background"
        );

        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("ui release should be processed");
        let bridge = process_placeholder_bridges(&runtime).expect("ui click event should dispatch");
        assert!(
            bridge
                .processed_script_events
                .iter()
                .any(|event| event.contains("playground-hud-ui.theme.clean-dev")),
            "ui click should publish the clean theme script event: {:?}",
            bridge.processed_script_events
        );

        assert_eq!(ui_theme.active_theme_id().as_deref(), Some("clean_dev"));
        assert!(
            ui_state.background_override(&button_path).is_none(),
            "pressed background should clear after release"
        );
    }

    #[test]
    fn playground_hud_ui_slider_drag_updates_without_crashing() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase bootstrap should succeed");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1280.0, 720.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let slider =
            find_path_ending(&layout, ".volume-slider").expect("volume slider should be in layout");
        let drag_x = slider.rect.x + slider.rect.width * 0.25;
        let drag_y = slider.rect.y + slider.rect.height * 0.5;

        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(drag_x, drag_y);
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("ui slider drag should process");
        process_placeholder_bridges(&runtime).expect("slider change event should dispatch");

        let scene_state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert!(
            (scene_state.get_float("volume").unwrap_or_default() - 0.25).abs() < 0.02,
            "slider drag should update Rhai scene volume"
        );
    }

    #[test]
    fn playground_hud_ui_f_keys_and_option_set_work() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase host should bootstrap");
        let mut host = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host should build");

        host.on_input_event(InputEvent::Key {
            key: KeyCode::F2,
            pressed: true,
        })
        .expect("F2 should be accepted");
        host.on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime should tick");

        let themes = host
            .runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");
        assert_eq!(themes.active_theme_id().as_deref(), Some("clean_dev"));

        host.runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1280.0, 720.0)));

        let ui_scene = host
            .runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = host
            .runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            themes.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let option_set = find_path_ending(&layout, ".color-options")
            .expect("color option set should be in layout");
        let click_x = option_set.rect.x + option_set.rect.width * 0.5;
        let click_y = option_set.rect.y + option_set.rect.height * 0.5;
        let ui_input = host
            .runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(click_x, click_y);
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&host.runtime)
            .expect("option set press should be processed");
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&host.runtime)
            .expect("option set release should be processed");
        process_placeholder_bridges(&host.runtime).expect("option set event should dispatch");

        let scene_state = host
            .runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(
            scene_state.get_string("color_preset").as_deref(),
            Some("Orange")
        );
    }

    #[test]
    fn playground_hud_ui_dropdown_changes_swatch_color() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase should bootstrap");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1280.0, 720.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let dropdown = find_path_ending(&layout, ".color-dropdown")
            .expect("color dropdown should be in layout");
        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(
            dropdown.rect.x + dropdown.rect.width * 0.5,
            dropdown.rect.y + dropdown.rect.height * 0.5,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown press should be processed");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown should open on click");
        ui_input.clear_frame_transients();
        assert_eq!(
            ui_state.expanded_override(&dropdown.path),
            Some(true),
            "first click should expand dropdown"
        );

        let expanded = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let expanded_showcase = expanded
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let expanded_layout = build_ui_layout_tree(
            UiViewportSize::new(1280.0, 720.0),
            &expanded_showcase.overlay,
        );
        let expanded_dropdown = find_path_ending(&expanded_layout, ".color-dropdown")
            .expect("expanded dropdown should be in layout");
        ui_input.set_mouse_position(
            expanded_dropdown.rect.x + expanded_dropdown.rect.width * 0.5,
            expanded_dropdown.rect.y + 38.0 * 2.5,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown option press should be processed");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown option should select");
        ui_input.clear_frame_transients();
        process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

        let scene_state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(
            scene_state.get_string("color_preset").as_deref(),
            Some("Orange")
        );
        assert!(
            ui_state
                .background_override(
                    "playground-hud-ui-showcase.root.main.editor.dropdown-row.color-swatch"
                )
                .is_some(),
            "dropdown selection should update color swatch background"
        );
    }

    #[test]
    fn playground_hud_ui_tabs_change_editor_panel() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-hud-ui".to_owned()])
                .with_startup_mod("playground-hud-ui")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("hud ui showcase should bootstrap");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1280.0, 720.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-hud-ui-showcase")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1280.0, 720.0), &showcase.overlay);

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let tabs =
            find_path_ending(&layout, ".tab-options").expect("tab option set should be in layout");
        let click_x = tabs.rect.x + tabs.rect.width * (5.5 / 7.0);
        let click_y = tabs.rect.y + tabs.rect.height * 0.5;
        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(click_x, click_y);
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("tab option press should be processed");
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("tab option release should be processed");
        process_placeholder_bridges(&runtime).expect("tab change event should dispatch");

        let scene_state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(
            scene_state.get_string("selected_tab").as_deref(),
            Some("Forces")
        );
    }

    #[test]
    fn particles_playground_menu_bootstraps() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("menu")
                .with_dev_mode(true),
        )
        .expect("particles menu should bootstrap");

        assert_eq!(summary.active_scene.as_deref(), Some("menu"));
        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        assert!(
            ui_scene
                .entity_names()
                .contains(&"playground-2d-particles-menu-ui".to_owned())
        );
    }

    #[test]
    fn particles_showcase_hydrates_emitters() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("particles showcase should bootstrap");

        assert_eq!(summary.active_scene.as_deref(), Some("showcase"));
        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let emitters = particles
            .emitters()
            .into_iter()
            .map(|command| command.entity_name)
            .collect::<Vec<_>>();
        assert_eq!(
            emitters,
            vec!["playground-2d-particles-preview-emitter".to_owned()],
            "showcase should hydrate only the preview emitter; preset data comes from registry"
        );
        let presets = runtime
            .resolve::<amigo_2d_particles::ParticlePreset2dService>()
            .expect("particle preset service should exist");
        let fire = presets.preset("fire").expect("fire preset should exist");
        assert!(
            fire.emitter.color_ramp.is_some(),
            "fire preset should hydrate a color ramp"
        );
        let preview = particles
            .emitter("playground-2d-particles-preview-emitter")
            .expect("preview emitter should exist");
        assert_eq!(preview.emitter.spawn_rate, fire.emitter.spawn_rate);
        assert_eq!(preview.emitter.shape, fire.emitter.shape);
        process_placeholder_bridges(&runtime).expect("showcase ui sync commands should dispatch");
        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state service should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme service should exist");
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase_ui = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase_ui.overlay);
        fn find_dropdown<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_dropdown(child, suffix) {
                    return Some(found);
                }
            }
            None
        }
        let dropdown =
            find_dropdown(&layout, ".preset-options").expect("preset dropdown should exist");
        match &dropdown.node.kind {
            UiOverlayNodeKind::Dropdown { options, .. } => {
                assert_eq!(
                    options,
                    &presets.ids(),
                    "showcase dropdown should be hydrated from the preset registry"
                );
            }
            other => panic!("preset-options should resolve as dropdown, got {other:?}"),
        }

        super::systems::particles_2d::tick_particles_2d_world(&runtime, 1.0 / 10.0)
            .expect("particle runtime tick should succeed");
        assert!(
            !particles.draw_commands().is_empty(),
            "showcase emitters should produce particle draw commands after a tick"
        );
        let scene_service = runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let tilemap_scene_service = runtime
            .resolve::<TileMap2dSceneService>()
            .expect("tilemap service should exist");
        let sprite_scene_service = runtime
            .resolve::<SpriteSceneService>()
            .expect("sprite service should exist");
        let text2d_scene_service = runtime
            .resolve::<Text2dSceneService>()
            .expect("text2d service should exist");
        let vector_scene_service = runtime
            .resolve::<amigo_2d_vector::VectorSceneService>()
            .expect("vector service should exist");
        let mesh_scene_service = runtime
            .resolve::<amigo_3d_mesh::MeshSceneService>()
            .expect("mesh service should exist");
        let material_scene_service = runtime
            .resolve::<amigo_3d_material::MaterialSceneService>()
            .expect("material service should exist");
        let text3d_scene_service = runtime
            .resolve::<amigo_3d_text::Text3dSceneService>()
            .expect("text3d service should exist");
        let ui_scene_service = runtime
            .resolve::<UiSceneService>()
            .expect("ui service should exist");
        let ui_state_service = runtime
            .resolve::<UiStateService>()
            .expect("ui state should exist");
        let ui_theme_service = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme should exist");
        let context = crate::render_runtime::AppRenderExtractContext {
            scene_service: scene_service.as_ref(),
            tilemap_scene_service: tilemap_scene_service.as_ref(),
            sprite_scene_service: sprite_scene_service.as_ref(),
            text2d_scene_service: text2d_scene_service.as_ref(),
            vector_scene_service: vector_scene_service.as_ref(),
            particle2d_scene_service: particles.as_ref(),
            mesh_scene_service: mesh_scene_service.as_ref(),
            material_scene_service: material_scene_service.as_ref(),
            text3d_scene_service: text3d_scene_service.as_ref(),
            ui_scene_service: ui_scene_service.as_ref(),
            ui_state_service: ui_state_service.as_ref(),
            ui_theme_service: ui_theme_service.as_ref(),
        };
        let packet =
            crate::render_runtime::default_app_render_extractor_registry().extract_all(&context);
        assert!(
            !packet.world_2d_particles().is_empty(),
            "render extraction should include generated particles"
        );
    }

    #[test]
    fn particles_showcase_explosion_burst_work() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("particles showcase should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.showcase.select",
            vec!["explosion".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("select event should dispatch");
        super::systems::particles_2d::tick_particles_2d_world(&runtime, 1.0 / 60.0)
            .expect("particle runtime tick should succeed");

        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        assert!(
            particles.particle_count("playground-2d-particles-preview-emitter") > 0,
            "explosion preset should emit particles through preview burst"
        );
    }

    #[test]
    fn particles_showcase_dropdown_can_wheel_scroll_to_lava_sparks() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("showcase")
                .with_dev_mode(true),
        )
        .expect("particles showcase should bootstrap");
        process_placeholder_bridges(&runtime).expect("showcase ui sync commands should dispatch");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1440.0, 900.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme should exist");
        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input should exist");

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let showcase = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-2d-particles-showcase-ui")
            .expect("showcase ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &showcase.overlay);
        let dropdown = find_path_ending(&layout, ".preset-options")
            .expect("preset dropdown should be in layout");
        let options = match &dropdown.node.kind {
            UiOverlayNodeKind::Dropdown { options, .. } => options.clone(),
            other => panic!("preset-options should resolve as dropdown, got {other:?}"),
        };
        let lava_index = options
            .iter()
            .position(|option| option == "lava_sparks")
            .expect("lava_sparks should be present in the dropdown registry");

        ui_input.set_mouse_position(
            dropdown.rect.x + dropdown.rect.width * 0.5,
            dropdown.rect.y + dropdown.rect.height * 0.5,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown press should process");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown release should expand");
        ui_input.clear_frame_transients();

        ui_input.set_mouse_position(
            dropdown.rect.x + dropdown.rect.width * 0.5,
            dropdown.rect.y + 38.0 * 4.5,
        );
        let target_offset = (lava_index as f32 - 4.0).max(0.0);
        ui_input.add_mouse_wheel(-(target_offset / 0.65));
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown wheel should smooth-scroll");
        ui_input.clear_frame_transients();
        let actual_offset = ui_state.dropdown_scroll_offset(&dropdown.path);
        assert!(
            actual_offset > 0.0,
            "wheel scrolling over an expanded dropdown should update its own scroll offset"
        );

        let lava_row = (lava_index as f32 - actual_offset + 1.5).clamp(1.25, 10.75);
        ui_input.set_mouse_position(
            dropdown.rect.x + dropdown.rect.width * 0.5,
            dropdown.rect.y + 38.0 * lava_row,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("lava_sparks option press should process");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("lava_sparks option release should select");
        ui_input.clear_frame_transients();
        process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

        let state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(
            state.get_string("selected_preset").as_deref(),
            Some("lava_sparks")
        );
    }

    #[test]
    fn particles_editor_mutates_emitter_from_script_event() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        assert_eq!(summary.active_scene.as_deref(), Some("editor"));
        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.spawn-rate",
            vec!["0.7500".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("spawn-rate event should dispatch");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.shape-kind",
            vec!["line".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("shape-kind event should dispatch");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.shape-mode",
            vec!["random_mix".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("shape-mode event should dispatch");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.align-kind",
            vec!["emitter".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("align-kind event should dispatch");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.blend-kind",
            vec!["additive".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("blend-kind event should dispatch");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.z-index",
            vec!["0.7000".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("z-index event should dispatch");

        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let emitter = particles
            .emitter("playground-2d-particles-editor-preview-emitter")
            .expect("editor preview emitter should exist");
        assert!(
            (emitter.emitter.spawn_rate - 150.0).abs() < 0.01,
            "expected spawn_rate to mutate to 150, got {}",
            emitter.emitter.spawn_rate
        );
        assert_eq!(emitter.emitter.shape_choices.len(), 3);
        assert_eq!(
            emitter.emitter.align,
            amigo_2d_particles::ParticleAlignMode2d::Emitter
        );
        assert_eq!(
            emitter.emitter.blend_mode,
            amigo_2d_particles::ParticleBlendMode2d::Additive
        );
        assert!((emitter.emitter.z_index - 50.0).abs() < 0.01);
    }

    #[test]
    fn particles_editor_applies_registry_preset() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.preset",
            vec!["smoke".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("preset event should dispatch");

        let presets = runtime
            .resolve::<amigo_2d_particles::ParticlePreset2dService>()
            .expect("preset service should exist");
        let smoke = presets.preset("smoke").expect("smoke preset should exist");
        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let emitter = particles
            .emitter("playground-2d-particles-editor-preview-emitter")
            .expect("editor preview emitter should exist");
        assert_eq!(emitter.emitter.spawn_rate, smoke.emitter.spawn_rate);
        assert_eq!(emitter.emitter.spawn_area, smoke.emitter.spawn_area);

        let state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(
            state.get_string("selected_preset").as_deref(),
            Some("smoke")
        );
    }

    #[test]
    fn particles_editor_tabs_switch_panels() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.tab",
            vec!["Spawn".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("tab event should dispatch");

        let state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(state.get_string("selected_tab").as_deref(), Some("Spawn"));
    }

    #[test]
    fn particles_editor_dropdown_can_select_deep_color_options() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.tab",
            vec!["Color".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("color tab event should dispatch");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1440.0, 900.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme should exist");
        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input should exist");

        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }

        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let editor = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
            .expect("editor ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &editor.overlay);
        let dropdown = find_path_ending(&layout, ".color-dropdown")
            .expect("color dropdown should be in layout");

        ui_input.set_mouse_position(
            dropdown.rect.x + dropdown.rect.width * 0.5,
            dropdown.rect.y + dropdown.rect.height * 0.5,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown press should be processed");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("dropdown release should expand");
        ui_input.clear_frame_transients();

        let expanded = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let expanded_editor = expanded
            .iter()
            .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
            .expect("expanded editor ui should resolve");
        let expanded_layout =
            build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &expanded_editor.overlay);
        let expanded_dropdown = find_path_ending(&expanded_layout, ".color-dropdown")
            .expect("expanded color dropdown should be in layout");

        ui_input.set_mouse_position(
            expanded_dropdown.rect.x + expanded_dropdown.rect.width * 0.5,
            expanded_dropdown.rect.y + 38.0 * 4.5,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("deep dropdown option press should be processed");
        ui_input.clear_frame_transients();
        ui_input.set_left_button(false);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("deep dropdown option release should select");
        ui_input.clear_frame_transients();
        process_placeholder_bridges(&runtime).expect("dropdown event should dispatch");

        let state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert_eq!(state.get_string("color").as_deref(), Some("Purple"));
    }

    #[test]
    fn particles_editor_rgb_color_picker_updates_emitter_color() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.tab",
            vec!["Color".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("color tab event should dispatch");

        runtime
            .resolve::<super::systems::UiInputViewportState>()
            .expect("ui viewport should exist")
            .set(Some(UiViewportSize::new(1440.0, 900.0)));

        let ui_scene = runtime
            .resolve::<UiSceneService>()
            .expect("ui scene service should exist");
        let ui_state = runtime
            .resolve::<UiStateService>()
            .expect("ui state should exist");
        let ui_theme = runtime
            .resolve::<UiThemeService>()
            .expect("ui theme should exist");
        fn find_path_ending<'a>(
            node: &'a OverlayUiLayoutNode,
            suffix: &str,
        ) -> Option<&'a OverlayUiLayoutNode> {
            if node.path.ends_with(suffix) {
                return Some(node);
            }
            for child in &node.children {
                if let Some(found) = find_path_ending(child, suffix) {
                    return Some(found);
                }
            }
            None
        }
        let resolved = crate::ui_runtime::resolve_ui_overlay_documents(
            ui_scene.as_ref(),
            ui_state.as_ref(),
            ui_theme.as_ref(),
        );
        let editor = resolved
            .iter()
            .find(|document| document.overlay.entity_name == "playground-2d-particles-editor-ui")
            .expect("editor ui should resolve");
        let layout = build_ui_layout_tree(UiViewportSize::new(1440.0, 900.0), &editor.overlay);
        let picker =
            find_path_ending(&layout, ".rgb-picker").expect("rgb picker should be visible");

        let slider_start_x = picker.rect.x + 8.0 + 54.0 + 10.0 + 24.0;
        let slider_width = picker.rect.x + picker.rect.width - 8.0 - slider_start_x;
        let ui_input = runtime
            .resolve::<UiInputService>()
            .expect("ui input service should exist");
        ui_input.set_mouse_position(
            slider_start_x + slider_width * 0.82,
            picker.rect.y + 8.0 + 11.0,
        );
        ui_input.set_left_button(true);
        super::systems::ui_input::process_ui_input(&runtime)
            .expect("rgb picker input should be processed");
        process_placeholder_bridges(&runtime).expect("rgb picker event should dispatch");

        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let emitter = particles
            .emitter("playground-2d-particles-editor-preview-emitter")
            .expect("editor preview emitter should exist");
        assert!(
            (emitter.emitter.color.r - 0.82).abs() < 0.02,
            "expected red channel to update from rgb picker, got {:?}",
            emitter.emitter.color
        );
    }

    #[test]
    fn particles_editor_color_ramp_preset_updates_emitter_ramp() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.ramp-preset",
            vec!["Fire".to_owned()],
        ));
        process_placeholder_bridges(&runtime).expect("ramp preset event should dispatch");

        let particles = runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let emitter = particles
            .emitter("playground-2d-particles-editor-preview-emitter")
            .expect("editor preview emitter should exist");
        let ramp = emitter
            .emitter
            .color_ramp
            .expect("ramp preset should set color_ramp");
        assert_eq!(ramp.stops.len(), 4);
        assert!(ramp.stops[1].color.r > 0.9);
        assert!(ramp.stops[1].color.g > 0.75);
    }

    #[test]
    fn particles_editor_export_logs_preset_yaml() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-particles".to_owned(),
                ])
                .with_startup_mod("playground-2d-particles")
                .with_startup_scene("editor")
                .with_dev_mode(true),
        )
        .expect("particles editor should bootstrap");

        let events = runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist");
        events.publish(ScriptEvent::new(
            "playground-2d-particles.editor.print-yaml",
            Vec::new(),
        ));
        process_placeholder_bridges(&runtime).expect("print yaml event should dispatch");

        let output = runtime
            .resolve::<DevConsoleState>()
            .expect("dev console should exist")
            .output_lines()
            .join("\n");
        assert!(output.contains("kind: particle-preset-2d"));
        assert!(output.contains("id: plasma-edited"));
        assert!(output.contains("label: Plasma Edited"));
        assert!(output.contains("category: energy"));
        assert!(output.contains("tags: [continuous, directional, energy, editor, edited]"));
        assert!(output.contains("emitter:"));
        assert!(output.contains("  type: ParticleEmitter2D"));
        assert!(output.contains("  max_particles: 160"));
        assert!(output.contains("  color_ramp:"));
        assert!(output.contains("  spawn_area:"));
        assert!(output.contains("  forces:"));
        assert!(output.contains("--- particle preset export end ---"));

        let export_path = PathBuf::from("target")
            .join("amigo-dev-exports")
            .join("particle-presets")
            .join("plasma-edited.yml");
        let exported = fs::read_to_string(&export_path).unwrap_or_else(|error| {
            panic!(
                "export `{}` should be readable: {error}",
                export_path.display()
            )
        });
        assert!(exported.contains("id: plasma-edited"));
        assert!(exported.contains("label: Plasma Edited"));
    }

    #[test]
    fn particle_preset_catalog_files_are_valid() {
        fn string_field<'a>(value: &'a serde_yaml::Value, key: &str) -> Option<&'a str> {
            value
                .as_mapping()
                .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
                .and_then(serde_yaml::Value::as_str)
        }

        fn mapping_field<'a>(
            value: &'a serde_yaml::Value,
            key: &str,
        ) -> Option<&'a serde_yaml::Mapping> {
            value
                .as_mapping()
                .and_then(|mapping| mapping.get(serde_yaml::Value::String(key.to_owned())))
                .and_then(serde_yaml::Value::as_mapping)
        }

        let preset_dir = mods_root().join("playground-2d-particles").join("presets");
        let expected_ids = [
            "fire",
            "smoke",
            "sparks",
            "magic",
            "snow",
            "dust",
            "thruster",
            "plasma",
            "portal",
            "rain",
            "explosion",
            "embers",
            "steam",
            "lightning",
            "healing",
            "poison",
            "starfield",
            "fountain",
            "shockwave",
            "aurora",
            "bubbles",
            "fireflies",
            "frost",
            "lava_sparks",
            "muzzle_flash",
            "pollen",
            "sandstorm",
            "spiral",
            "waterfall",
            "welding",
        ];
        let mut seen_ids = Vec::new();

        for entry in fs::read_dir(&preset_dir).expect("preset dir should exist") {
            let path = entry.expect("preset dir entry should be readable").path();
            if path.extension().and_then(|ext| ext.to_str()) != Some("yml") {
                continue;
            }

            let contents = fs::read_to_string(&path).expect("preset file should be readable");
            let document: serde_yaml::Value =
                serde_yaml::from_str(&contents).expect("preset file should parse as YAML");
            assert_eq!(
                string_field(&document, "kind"),
                Some("particle-preset-2d"),
                "preset {:?} should declare kind",
                path
            );
            let id = string_field(&document, "id").expect("preset should declare id");
            assert!(
                string_field(&document, "label").is_some(),
                "preset `{id}` should declare label"
            );
            let emitter = mapping_field(&document, "emitter")
                .unwrap_or_else(|| panic!("preset `{id}` should declare emitter mapping"));
            assert_eq!(
                emitter
                    .get(serde_yaml::Value::String("type".to_owned()))
                    .and_then(serde_yaml::Value::as_str),
                Some("ParticleEmitter2D"),
                "preset `{id}` should declare emitter.type"
            );
            seen_ids.push(id.to_owned());
        }

        for expected in expected_ids {
            assert!(
                seen_ids.iter().any(|id| id == expected),
                "preset catalog should include `{expected}`"
            );
        }
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

        let bridge = super::orchestration::process_placeholder_bridges(&runtime)
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
        assert!(
            summary
                .processed_script_events
                .iter()
                .any(|event| event
                    == "playground-2d.demo.component.attach(playground-2d-demo-square)")
        );
        assert!(summary.failed_assets.is_empty());
    }

    #[test]
    fn playground_2d_script_component_updates_and_detaches() {
        let (runtime, _summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec!["core".to_owned(), "playground-2d".to_owned()])
                .with_startup_mod("playground-2d")
                .with_startup_scene("basic-scripting-demo")
                .with_dev_mode(true),
        )
        .expect("2d scripting demo bootstrap should succeed");

        super::systems::script_components::tick_script_components(&runtime, 0.5)
            .expect("script component update should run");

        let scene_state = runtime
            .resolve::<amigo_state::SceneStateService>()
            .expect("scene state should exist");
        assert!(
            scene_state
                .get_float("playground-2d-demo-square.component.elapsed")
                .is_some_and(|elapsed| elapsed >= 0.5)
        );

        runtime
            .resolve::<SceneCommandQueue>()
            .expect("scene command queue should exist")
            .submit(SceneCommand::SelectScene {
                scene: SceneKey::new("hello-world-square"),
            });
        let updated = refresh_runtime_summary(&runtime)
            .expect("runtime refresh should process scene transition");

        assert!(updated.processed_script_events.iter().any(|event| {
            event == "playground-2d.demo.component.detach(playground-2d-demo-square)"
        }));
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

        let initial_center =
            first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::Center)
                .expect("initial center tile id should exist");
        assert_eq!(initial_center, 6);

        fs::write(
            &asset_path,
            original_asset.replace("center: 6", "center: 0"),
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

        let updated_center =
            first_resolved_tile_id_for_variant(&runtime, TileVariantKind2d::Center)
                .expect("updated center tile id should exist");
        assert_eq!(updated_center, 0);
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
                .any(|kind| kind == "MotionController2D x1")
        );
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "CameraFollow2D x1")
        );
        assert!(component_kinds.iter().any(|kind| kind == "Parallax2D x4"));
        assert!(
            component_kinds
                .iter()
                .any(|kind| kind == "TileMapMarker2D x27")
        );
        assert!(component_kinds.iter().any(|kind| kind == "Trigger2D x26"));
        assert!(component_kinds.iter().any(|kind| kind == "UiDocument x1"));

        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-layer-01")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-layer-02")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-layer-03")
        );
        assert!(
            summary
                .scene_entities
                .iter()
                .any(|entity| entity == "playground-sidescroller-background-layer-04")
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
                .any(|entity| entity == "playground-sidescroller-coin-25")
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
                .any(|asset| asset == "playground-sidescroller/backgrounds/layer-01 (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/backgrounds/layer-02 (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/backgrounds/layer-03 (image-2d)")
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| asset == "playground-sidescroller/backgrounds/layer-04 (image-2d)")
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
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| { asset == "playground-sidescroller/audio/jump (generated-audio)" })
        );
        assert!(
            summary
                .prepared_assets
                .iter()
                .any(|asset| { asset == "playground-sidescroller/audio/coin (generated-audio)" })
        );
        assert!(summary.prepared_assets.iter().any(|asset| {
            asset == "playground-sidescroller/audio/level-complete (generated-audio)"
        }));
        assert!(summary.prepared_assets.iter().any(|asset| {
            asset == "playground-sidescroller/audio/proximity-beep (generated-audio)"
        }));
        assert_eq!(summary.audio_master_volume, 1.0);
        assert!(summary.audio_sources.is_empty());
        assert!(
            summary
                .pending_audio_runtime_commands
                .iter()
                .any(|entry| entry == "audio.play(playground-sidescroller/audio/jump)")
        );
        assert!(!summary.audio_output_started);
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
            "vector_2d",
            "physics_2d",
            "tilemap_2d",
            "motion_2d",
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
            "amigo-2d-vector",
            "amigo-2d-physics",
            "amigo-2d-tilemap",
            amigo_2d_motion::CANONICAL_MOTION_2D_RUNTIME_REPORT_LABEL,
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

        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
    fn handle_script_command_writes_debug_text_export() {
        let scene_command_queue = SceneCommandQueue::default();
        let script_event_queue = ScriptEventQueue::default();
        let dev_console_state = DevConsoleState::default();
        let asset_catalog = AssetCatalog::default();
        let ui_state = UiStateService::default();
        let audio_command_queue = AudioCommandQueue::default();
        let audio_scene_service = AudioSceneService::default();
        let diagnostics = RuntimeDiagnostics::default();
        let launch_selection = LaunchSelection::new(None, None, Vec::new(), true);
        let relative_path = format!(
            "tests/debug-export-{}.txt",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("system time should be after epoch")
                .as_nanos()
        );
        let target_path = PathBuf::from("target")
            .join("amigo-dev-exports")
            .join(&relative_path);
        if target_path.exists() {
            fs::remove_file(&target_path).expect("stale debug export should be removable");
        }

        script_runtime::dispatch_script_command(
            ScriptCommand::new(
                "debug",
                "write-text",
                vec![relative_path.clone(), "hello export".to_owned()],
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
            fs::read_to_string(&target_path).expect("debug export should be written"),
            "hello export"
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

        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
        script_runtime::dispatch_script_command(
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
    fn handle_script_command_queues_scene_commands() {
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

        script_runtime::dispatch_script_command(
            ScriptCommand::new("scene", "select", vec!["sprite-showcase".to_owned()]),
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
        script_runtime::dispatch_script_command(
            ScriptCommand::new("scene", "reload", Vec::new()),
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
        script_runtime::dispatch_script_command(
            ScriptCommand::new("scene", "spawn", vec!["runtime-test-entity".to_owned()]),
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
        script_runtime::dispatch_script_command(
            ScriptCommand::new("scene", "clear", Vec::new()),
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

        let commands = scene_command_queue.pending();
        assert!(matches!(
            commands.first(),
            Some(SceneCommand::SelectScene { scene }) if scene.as_str() == "sprite-showcase"
        ));
        assert!(matches!(
            commands.get(1),
            Some(SceneCommand::ReloadActiveScene)
        ));
        assert!(matches!(
            commands.get(2),
            Some(SceneCommand::SpawnNamedEntity { name, transform }) if name == "runtime-test-entity" && transform.is_none()
        ));
        assert!(matches!(commands.get(3), Some(SceneCommand::ClearEntities)));
    }

    #[test]
    fn handle_script_command_asset_reload_requests_load_and_event() {
        let scene_command_queue = SceneCommandQueue::default();
        let script_event_queue = ScriptEventQueue::default();
        let dev_console_state = DevConsoleState::default();
        let asset_catalog = AssetCatalog::default();
        let ui_state = UiStateService::default();
        let audio_command_queue = AudioCommandQueue::default();
        let audio_scene_service = AudioSceneService::default();
        let diagnostics = RuntimeDiagnostics::default();
        let launch_selection = LaunchSelection::new(
            Some("playground-sidescroller".to_owned()),
            Some("vertical-slice".to_owned()),
            Vec::new(),
            true,
        );
        asset_catalog.register_manifest(AssetManifest {
            key: AssetKey::new("playground-sidescroller/audio/jump"),
            source: AssetSourceKind::Mod("playground-sidescroller".to_owned()),
            tags: vec!["audio".to_owned(), "generated".to_owned()],
        });

        script_runtime::dispatch_script_command(
            ScriptCommand::new(
                "asset",
                "reload",
                vec!["playground-sidescroller/audio/jump".to_owned()],
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

        assert!(
            asset_catalog
                .pending_loads()
                .iter()
                .any(|request| request.key.as_str() == "playground-sidescroller/audio/jump")
        );
        assert!(script_event_queue.pending().iter().any(|event| {
            event.topic == "asset.reload-requested"
                && event.payload == vec!["playground-sidescroller/audio/jump".to_owned()]
        }));
    }

    #[test]
    fn handle_script_command_unknown_command_reports_fallback() {
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

        script_runtime::dispatch_script_command(
            ScriptCommand::new("unknown", "noop", vec!["x".to_owned()]),
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

        assert!(
            dev_console_state
                .output_lines()
                .iter()
                .any(|line| line.contains("unhandled placeholder script command: unknown.noop(x)"))
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
    fn interactive_host_handler_updates_asteroids_ship_and_bullet_loop() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-asteroids".to_owned(),
                ])
                .with_startup_mod("playground-2d-asteroids")
                .with_startup_scene("main-menu")
                .with_dev_mode(true),
        )
        .expect("asteroids bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("initial runtime tick should succeed");

        {
            let scene = handler
                .runtime
                .resolve::<SceneService>()
                .expect("scene service should exist");
            assert!(scene.is_visible("playground-2d-asteroids-main-menu"));
            let ui_state = handler
                .runtime
                .resolve::<UiStateService>()
                .expect("ui state service should exist");
            assert!(ui_state.is_visible("playground-2d-asteroids-main-menu.root"));
        }

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("menu start input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("start game tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: false,
            })
            .expect("menu start release should be accepted");
        for _ in 0..3 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("scene transition tick should succeed");
        }

        let initial_ship = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-2d-asteroids-ship")
            .expect("asteroids ship should exist");

        {
            let scene = handler
                .runtime
                .resolve::<SceneService>()
                .expect("scene service should exist");
            assert_eq!(
                scene.selected_scene().map(|id| id.as_str().to_owned()),
                Some("game".to_owned())
            );
            assert!(scene.is_visible("playground-2d-asteroids-hud"));
            assert!(scene.is_visible("playground-2d-asteroids-ship"));
            assert!(scene.is_visible("playground-2d-asteroids-ship-shield"));
            assert!(scene.is_simulation_enabled("playground-2d-asteroids-ship"));
            let ui_state = handler
                .runtime
                .resolve::<UiStateService>()
                .expect("ui state service should exist");
            assert!(ui_state.is_visible("playground-2d-asteroids-hud.root"));
        }

        let asteroid_to_hit = {
            let pools = handler
                .runtime
                .resolve::<EntityPoolSceneService>()
                .expect("entity pool scene service should exist");
            let active_asteroids = pools.active_members("asteroids");
            assert_eq!(active_asteroids.len(), 4);
            active_asteroids
                .first()
                .cloned()
                .expect("wave should spawn an asteroid")
        };
        handler
            .runtime
            .resolve::<ScriptEventQueue>()
            .expect("script event queue should exist")
            .publish(ScriptEvent::new(
                "asteroids.bullet_hit_asteroid",
                vec![
                    "playground-2d-asteroids-bullet-01".to_owned(),
                    asteroid_to_hit,
                ],
            ));
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("bullet hit event tick should succeed");
        let pools = handler
            .runtime
            .resolve::<EntityPoolSceneService>()
            .expect("entity pool scene service should exist");
        assert!(
            pools.active_count("asteroids") > 4,
            "hitting a wave asteroid should split it into smaller active fragments"
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            })
            .expect("thrust input should be accepted");

        for _ in 0..6 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime thrust tick should succeed");
        }

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let updated_ship = scene
            .transform_of("playground-2d-asteroids-ship")
            .expect("asteroids ship should exist after thrust");
        assert!(
            updated_ship.translation.y > initial_ship.translation.y,
            "holding thrust should move the Asteroids ship forward"
        );
        let particles = handler
            .runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        let early_thruster = particles
            .emitter("playground-2d-asteroids-main-thruster")
            .expect("Asteroids thruster emitter should exist");
        assert!(
            early_thruster.emitter.initial_size >= 3.0 && early_thruster.emitter.final_size >= 7.0,
            "short thrust should configure a thick blue plasma spike"
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("fire input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime fire tick should succeed");

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let active_bullet = (1..=6)
            .map(|index| format!("playground-2d-asteroids-bullet-{index:02}"))
            .any(|entity| scene.is_visible(&entity) && scene.is_simulation_enabled(&entity));
        assert!(
            active_bullet,
            "firing should activate the first Asteroids bullet"
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
                        if clip.as_str() == "playground-2d-asteroids/audio/shot"
                )),
            "firing should queue the Asteroids shot audio clip"
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::R,
                pressed: true,
            })
            .expect("reload input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime reload tick should succeed");
        for _ in 0..4 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime post-reload tick should succeed");
        }

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        assert_eq!(
            scene.selected_scene().map(|id| id.as_str().to_owned()),
            Some("game".to_owned())
        );
    }

    #[test]
    fn interactive_asteroids_sustained_thrust_reaches_ion_pulse() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-asteroids".to_owned(),
                ])
                .with_startup_mod("playground-2d-asteroids")
                .with_startup_scene("game")
                .with_dev_mode(true),
        )
        .expect("asteroids game bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("initial runtime tick should succeed");

        let camera_follow = handler
            .runtime
            .resolve::<amigo_scene::CameraFollow2dSceneService>()
            .expect("camera follow scene service should exist");
        assert!(
            camera_follow
                .follow("playground-2d-asteroids-arena-void")
                .is_some(),
            "endless Asteroids background should follow the camera"
        );
        let particles = handler
            .runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        assert!(
            particles.is_active("playground-2d-asteroids-deep-starfield"),
            "deep starfield should stay active around the camera"
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Up,
                pressed: true,
            })
            .expect("thrust input should be accepted");

        for _ in 0..80 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("runtime sustained thrust tick should succeed");
        }

        let particles = handler
            .runtime
            .resolve::<amigo_2d_particles::Particle2dSceneService>()
            .expect("particle scene service should exist");
        assert!(
            particles.is_active("playground-2d-asteroids-speed-starfield"),
            "max thrust should enable the camera speed-starfield effect"
        );
        let late_thruster = particles
            .emitter("playground-2d-asteroids-main-thruster")
            .expect("Asteroids thruster emitter should exist");
        let late_thruster_line_length = match late_thruster.emitter.shape {
            amigo_2d_particles::ParticleShape2d::Line { length } => length,
            _ => 0.0,
        };
        assert!(
            late_thruster.emitter.particle_lifetime <= 0.16
                && late_thruster.emitter.initial_speed >= 420.0
                && late_thruster_line_length >= 22.0,
            "sustained thrust should enter a long, fast ion burst climax"
        );

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let camera = scene
            .transform_of("playground-2d-asteroids-camera")
            .expect("Asteroids camera should exist");
        assert!(
            camera.translation.y > 0.0,
            "endless Asteroids camera should follow the accelerating ship"
        );
    }

    #[test]
    fn interactive_asteroids_options_low_mode_persists_into_game_scene() {
        let (runtime, summary) = bootstrap_with_options(
            BootstrapOptions::new(mods_root())
                .with_active_mods(vec![
                    "core".to_owned(),
                    "playground-2d-asteroids".to_owned(),
                ])
                .with_startup_mod("playground-2d-asteroids")
                .with_startup_scene("main-menu")
                .with_dev_mode(true),
        )
        .expect("asteroids bootstrap should succeed");

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("initial runtime tick should succeed");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Down,
                pressed: true,
            })
            .expect("menu down input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("menu navigation tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Down,
                pressed: false,
            })
            .expect("menu down release should be accepted");

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("options select input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("options select tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: false,
            })
            .expect("options select release should be accepted");
        for _ in 0..3 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("options transition tick should succeed");
        }

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        assert_eq!(
            scene.selected_scene().map(|id| id.as_str().to_owned()),
            Some("options".to_owned())
        );

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("low toggle input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("low toggle tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: false,
            })
            .expect("low toggle release should be accepted");

        let session = handler
            .runtime
            .resolve::<amigo_state::SessionStateService>()
            .expect("session state service should exist");
        assert_eq!(session.get_bool("asteroids.low_mode"), Some(true));

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Escape,
                pressed: true,
            })
            .expect("options back input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("options back tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Escape,
                pressed: false,
            })
            .expect("options back release should be accepted");
        for _ in 0..3 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("main menu transition tick should succeed");
        }

        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: true,
            })
            .expect("start input should be accepted");
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("start tick should succeed");
        handler
            .on_input_event(InputEvent::Key {
                key: KeyCode::Space,
                pressed: false,
            })
            .expect("start release should be accepted");
        for _ in 0..3 {
            handler
                .on_lifecycle(HostLifecycleEvent::AboutToWait)
                .expect("game transition tick should succeed");
        }

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        assert_eq!(
            scene.selected_scene().map(|id| id.as_str().to_owned()),
            Some("game".to_owned())
        );
        let pools = handler
            .runtime
            .resolve::<EntityPoolSceneService>()
            .expect("entity pool service should exist");
        assert_eq!(pools.active_count("asteroids"), 3);
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

        let mut handler = InteractiveRuntimeHostHandler::new(runtime, summary)
            .expect("interactive host handler should initialize");

        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("initial runtime tick should succeed");

        let initial = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-player")
            .expect("sidescroller player should exist");

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
        let initial_layer_01 = scene
            .transform_of("playground-sidescroller-background-layer-01")
            .expect("background layer 01 should exist");
        let initial_layer_02 = scene
            .transform_of("playground-sidescroller-background-layer-02")
            .expect("background layer 02 should exist");
        let initial_layer_03 = scene
            .transform_of("playground-sidescroller-background-layer-03")
            .expect("background layer 03 should exist");
        let initial_layer_04 = scene
            .transform_of("playground-sidescroller-background-layer-04")
            .expect("background layer 04 should exist");

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
        let updated_layer_01 = scene
            .transform_of("playground-sidescroller-background-layer-01")
            .expect("background layer 01 should exist after update");
        let updated_layer_02 = scene
            .transform_of("playground-sidescroller-background-layer-02")
            .expect("background layer 02 should exist after update");
        let updated_layer_03 = scene
            .transform_of("playground-sidescroller-background-layer-03")
            .expect("background layer 03 should exist after update");
        let updated_layer_04 = scene
            .transform_of("playground-sidescroller-background-layer-04")
            .expect("background layer 04 should exist after update");

        let layer_01_screen_delta = (updated_layer_01.translation.x - updated_camera.translation.x)
            - (initial_layer_01.translation.x - initial_camera.translation.x);
        let layer_02_screen_delta = (updated_layer_02.translation.x - updated_camera.translation.x)
            - (initial_layer_02.translation.x - initial_camera.translation.x);
        let layer_03_screen_delta = (updated_layer_03.translation.x - updated_camera.translation.x)
            - (initial_layer_03.translation.x - initial_camera.translation.x);
        let layer_04_screen_delta = (updated_layer_04.translation.x - updated_camera.translation.x)
            - (initial_layer_04.translation.x - initial_camera.translation.x);

        assert!(
            layer_01_screen_delta.abs() > 0.0,
            "background layer 01 should visibly shift on screen"
        );
        assert!(
            layer_02_screen_delta.abs() > layer_01_screen_delta.abs()
                && layer_03_screen_delta.abs() > layer_02_screen_delta.abs()
                && layer_04_screen_delta.abs() > layer_03_screen_delta.abs(),
            "closer background layers should move more on screen than farther ones"
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
        assert_eq!(sprites.frame_of("playground-sidescroller-coin-01"), Some(0));
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
            sprites.frame_of("playground-sidescroller-coin-01"),
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
            .transform_of("playground-sidescroller-coin-01")
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
                .text_override("playground-sidescroller-hud.root.coins")
                .as_deref(),
            Some("Coins: 1 / 25")
        );
        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.message")
                .as_deref(),
            Some("COIN COLLECTED")
        );
        let moved_coin = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist")
            .transform_of("playground-sidescroller-coin-01")
            .expect("coin should still exist after collection");
        assert!(
            moved_coin.translation.x <= -10_000.0 && moved_coin.translation.y <= -10_000.0,
            "collected coin should be moved out of the playable space"
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

        let scene = handler
            .runtime
            .resolve::<SceneService>()
            .expect("scene service should exist");
        let mut reset_transform = coin;
        reset_transform.translation.x = 0.0;
        reset_transform.translation.y = 0.0;
        assert!(
            scene.set_transform("playground-sidescroller-player", reset_transform,),
            "player should be moved away from the collected coin"
        );
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick after moving away should succeed");
        assert!(
            scene.set_transform("playground-sidescroller-player", coin),
            "player should be moved back to the original coin position"
        );
        handler
            .on_lifecycle(HostLifecycleEvent::AboutToWait)
            .expect("runtime tick after returning should succeed");

        assert_eq!(
            ui_state
                .text_override("playground-sidescroller-hud.root.coins")
                .as_deref(),
            Some("Coins: 1 / 25")
        );
        let coin_play_count = audio_state
            .processed_commands()
            .iter()
            .filter(|command| {
                matches!(
                    command,
                    AudioCommand::PlayOnce { clip }
                        if clip.as_str() == "playground-sidescroller/audio/coin"
                )
            })
            .count();
        assert_eq!(
            coin_play_count, 1,
            "collected coin should not replay when revisiting the original location"
        );
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

        assert!(!super::bootstrap::should_use_interactive_host(
            &core_options
        ));
        assert!(super::bootstrap::should_use_interactive_host(
            &playground_options
        ));
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

        let resolved = super::assets::resolve_existing_asset_path(
            root.join("textures").join("player"),
            "test/player",
        )
        .expect("metadata candidate should resolve");

        assert_eq!(resolved, metadata_path);
    }
}
