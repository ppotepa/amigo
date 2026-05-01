use std::any::type_name;
use std::collections::BTreeMap;
use std::fmt::{self, Display, Formatter};
use std::fs;
use std::path::Component;
use std::path::{Path, PathBuf};

use amigo_2d_motion::Motion2dSceneService;
use amigo_2d_particles::Particle2dSceneService;
use amigo_2d_physics::{
    Physics2dSceneService, move_and_collide, overlaps_trigger_with_translation,
};
use amigo_2d_sprite::{SpriteSceneService, SpriteSheet};
use amigo_2d_text::Text2dSceneService;
use amigo_2d_tilemap::{marker_cells, TileMap2dSceneService};
use amigo_2d_vector::VectorSceneService;
use amigo_3d_material::MaterialSceneService;
use amigo_3d_mesh::MeshSceneService;
use amigo_3d_text::Text3dSceneService;
use amigo_app_host_api::{
    HostConfig, HostControl, HostExitStrategy, HostHandler, HostLifecycleEvent,
};
use amigo_assets::{
    AssetCatalog, AssetKey, AssetLoadPriority, AssetLoadRequest, AssetManifest, AssetSourceKind,
    prepare_asset_from_contents,
};
use amigo_audio_api::{
    AudioClip, AudioClipKey, AudioCommand, AudioCommandQueue, AudioPlaybackMode, AudioSceneService,
    AudioSourceId, AudioStateService,
};
use amigo_audio_mixer::AudioMixerService;
use amigo_audio_output::{AudioOutputBackendService, AudioOutputStartStatus};
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
    UiDocument as RuntimeUiDocument, UiDrawCommand, UiEventBinding, UiInputService,
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
