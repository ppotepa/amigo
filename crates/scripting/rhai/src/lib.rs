//! Rhai scripting backend for gameplay and tooling scripts.
//! It binds engine services into script APIs, owns source loading, and drives lifecycle callbacks.

/// Rhai world bindings that expose engine services to scripts.
mod bindings;
/// Script-facing wrappers around entities, assets, and other references.
mod handles;
/// Rhai package construction and source registration helpers.
mod package;
/// Plugin-owned Rhai binding provider contracts.
mod plugin_bindings;
/// Runtime contribution descriptors for scripting-owned scene handlers and systems.
mod runtime_capabilities;
/// Scene command execution owned by the Rhai scripting domain.
mod scene_command;
mod systems;

use std::collections::BTreeMap;
use std::sync::{Arc, Mutex};

use amigo_shutter_motion_plugin::Motion2dSceneService;
use amigo_2d_particles::{Particle2dSceneService, ParticlePreset2dService};
use amigo_2d_physics::Physics2dSceneService;
use amigo_2d_post_fx::PostFx2dService;
use amigo_2d_sprite::SpriteSceneService;
use amigo_2d_vector::VectorSceneService;
use amigo_assets::AssetCatalog;
use amigo_camera_core_plugin::{CameraFocusTarget2dService, CameraService};
use amigo_core::{AmigoError, AmigoResult, LaunchSelection, RuntimeDiagnostics};
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;
use amigo_modding::ModCatalog;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{EntityPoolSceneService, LifetimeSceneService, SceneService};
use amigo_scripting_api::{
    DevConsoleEvalResult, DevConsoleQueue, DevConsoleScriptContext, DevConsoleState, RunLogService,
    ScriptCommandQueue, ScriptComponentService, ScriptEventQueue, ScriptLifecycleState,
    ScriptParams, ScriptRuntime, ScriptRuntimeInfo, ScriptRuntimeService, ScriptSourceContext,
    ScriptTraceService, ScriptValue,
};
use amigo_state::{SceneStateService, SceneTimerService, SessionStateService};
use amigo_ui::UiThemeService;
use bindings::{ScriptTimeState, WorldApi, register_world_api};
use package::PackageModuleResolver;
use rhai::CallFnOptions;

pub use runtime_capabilities::*;
pub use plugin_bindings::*;
pub use scene_command::*;
pub use systems::*;

include!("runtime/script_runtime.rs");
include!("runtime/plugin.rs");

#[cfg(test)]
mod tests;
