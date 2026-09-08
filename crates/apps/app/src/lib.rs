//! Primary Amigo application runtime.
//! It bootstraps engine services, loads content, drives systems, and coordinates rendering, scripting, audio, and scene hydration.

use std::any::type_name;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

use amigo_app_host_api::{
    HostConfig, HostControl, HostExitStrategy, HostHandler, HostLifecycleEvent,
};
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    prepare_asset_from_contents,
};
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_file_watch_api::FileWatchService;
use amigo_hot_reload::{AssetWatch, HotReloadService, HotReloadWatchKind, SceneDocumentWatch};
use amigo_input_api::{InputEvent, InputServiceInfo, InputState, KeyCode};
use amigo_modding::{ModCatalog, ModScriptMode};
use amigo_render_api::RenderBackendInfo;
#[cfg(test)]
use amigo_render_wgpu::UiLayoutNode as OverlayUiLayoutNode;
use amigo_render_wgpu::{WgpuRenderBackend, WgpuSceneRenderer, WgpuSurfaceState};
use amigo_runtime::{Runtime, RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    HydratedSceneState, SceneCommand, SceneCommandQueue, SceneHydrationPlan, SceneKey,
    SceneService, SceneTransitionPlan, SceneTransitionService,
};
use amigo_scripting_api::{
    DevConsoleQueue, DevConsoleState, ScriptCommand, ScriptCommandQueue, ScriptComponentService,
    ScriptEvent, ScriptEventQueue, ScriptLifecycleState, ScriptRuntimeInfo, ScriptRuntimeService,
};
use amigo_window_api::{WindowDescriptor, WindowEvent, WindowServiceInfo, WindowSurfaceHandles};

/// Local helper functions shared across bootstrap and runtime modules.
mod app_helpers;
/// App-specific asset extraction and adaptation helpers.
mod assets;
/// Public bootstrap entry points for hosted and standalone execution.
mod bootstrap;
/// Engine-level runtime debug overlay and live frame metrics.
mod debug_overlay;
/// Runtime developer console and diagnostic command handlers.
/// Runtime diagnostics collection and plugin wiring.
mod diagnostics;
/// App integration for the engine event pipeline domain.
mod event_pipeline;
/// Host handlers used by interactive and summary execution modes.
mod host_runtime;
/// Launch selection parsing and scene/mod resolution helpers.
mod launch_selection;
/// Bridges that connect auxiliary subsystems during startup.
mod orchestration;
/// Built-in particle preset registration for the app runtime.
mod particle_presets;
/// Render backend integration and frame extraction plumbing.
mod render_runtime;
/// Runtime service lookup helpers used across the app crate.
mod runtime_context;
/// Tooling-oriented scene preview host and snapshot seam.
mod scene_preview;
/// Scene document loading and scene command dispatch.
mod scene_runtime;
/// Runtime scheduler config/state resolved from defaults and scene metadata.
mod scheduling;
/// Script command handling and script-event integration.
mod script_runtime;
/// Script runtime bootstrap helpers shared by app entry points.
mod scripting_runtime;
/// Summary reporting shown by non-interactive hosts and tooling.
mod summary;
/// Frame systems that advance gameplay and presentation each tick.
mod systems;
pub use bootstrap::{
    bootstrap_session_default, bootstrap_session_with_options, run_hosted_with_options,
};
pub(crate) use diagnostics::RuntimeDiagnosticsPlugin;
pub(crate) use host_runtime::{InteractiveRuntimeHostHandler, SummaryHostHandler};
pub(crate) use launch_selection::LaunchSelectionPlugin;
use runtime_context::{required, required_from_registry};
pub use scene_preview::{
    ScenePreviewFrame, ScenePreviewHost, ScenePreviewOptions, capture_scene_preview,
};
use summary::refresh_runtime_summary;

include!("app_model/summary.rs");
include!("app_model/display.rs");
include!("app_model/options.rs");

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
mod tests;
