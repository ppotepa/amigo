pub(crate) mod audio;
pub(crate) mod camera_follow_2d;
pub(crate) mod collision_events_2d;
pub(crate) mod lifetime;
pub(crate) mod motion_2d;
pub(crate) mod parallax_2d;
pub(crate) mod particles_2d;
pub(crate) mod scene_transition;
pub(crate) mod script_update;
pub(crate) mod ui_bindings;
pub(crate) mod ui_input;

use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_render_wgpu::UiViewportSize;
use amigo_runtime::{RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry};

use crate::runtime_context::required_from_registry;

pub(crate) const HOST_DELTA_SECONDS: f32 = 1.0 / 60.0;

#[derive(Debug, Default)]
pub(crate) struct UiInputViewportState {
    viewport: Mutex<Option<UiViewportSize>>,
}

impl UiInputViewportState {
    pub(crate) fn set(&self, viewport: Option<UiViewportSize>) {
        *self
            .viewport
            .lock()
            .expect("ui viewport mutex should not be poisoned") = viewport;
    }

    pub(crate) fn get(&self) -> Option<UiViewportSize> {
        *self
            .viewport
            .lock()
            .expect("ui viewport mutex should not be poisoned")
    }
}

fn register_system<F>(
    registry: &ServiceRegistry,
    phase: SystemPhase,
    name: &'static str,
    run: F,
) -> AmigoResult<()>
where
    F: Fn(&amigo_runtime::Runtime) -> AmigoResult<()> + Send + Sync + 'static,
{
    required_from_registry::<SystemRegistry>(registry)?.register_fn(phase, name, run);
    Ok(())
}

pub(crate) struct RuntimeSystemServicesPlugin;

impl RuntimePlugin for RuntimeSystemServicesPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-runtime-system-services"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(UiInputViewportState::default())?;
        registry.register(SystemRegistry::default())
    }
}

pub(crate) struct UiInputRuntimeSystemPlugin;

impl RuntimePlugin for UiInputRuntimeSystemPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-ui-input-runtime-system"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_system(
            registry,
            SystemPhase::PreUpdate,
            "ui_input",
            move |runtime| ui_input::process_ui_input(runtime),
        )
    }
}

pub(crate) struct ScriptUpdateRuntimeSystemPlugin;

impl RuntimePlugin for ScriptUpdateRuntimeSystemPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-script-update-runtime-system"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_system(
            registry,
            SystemPhase::Update,
            "ui_bindings",
            move |runtime| ui_bindings::tick_ui_bindings(runtime),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "script_update",
            move |runtime| script_update::tick_active_scripts(runtime, HOST_DELTA_SECONDS),
        )
    }
}

pub(crate) struct World2dRuntimeSystemsPlugin;

impl RuntimePlugin for World2dRuntimeSystemsPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-world-2d-runtime-systems"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_system(registry, SystemPhase::Update, "motion_2d", move |runtime| {
            motion_2d::tick_motion_2d_world(runtime, HOST_DELTA_SECONDS)
        })?;
        register_system(
            registry,
            SystemPhase::Update,
            "particles_2d",
            move |runtime| particles_2d::tick_particles_2d_world(runtime, HOST_DELTA_SECONDS),
        )?;
        register_system(registry, SystemPhase::Update, "lifetime", move |runtime| {
            lifetime::tick_lifetimes(runtime, HOST_DELTA_SECONDS)
        })?;
        register_system(
            registry,
            SystemPhase::Update,
            "collision_events_2d",
            move |runtime| collision_events_2d::tick_collision_events_2d(runtime),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "camera_follow_2d",
            move |runtime| camera_follow_2d::tick_camera_follow_world(runtime),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "parallax_2d",
            move |runtime| parallax_2d::tick_parallax_world(runtime),
        )
    }
}

pub(crate) struct SceneTransitionRuntimeSystemPlugin;

impl RuntimePlugin for SceneTransitionRuntimeSystemPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-scene-transition-runtime-system"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_system(
            registry,
            SystemPhase::Update,
            "scene_transition",
            move |runtime| scene_transition::tick_scene_transitions(runtime, HOST_DELTA_SECONDS),
        )
    }
}

pub(crate) struct AudioRuntimeSystemPlugin;

impl RuntimePlugin for AudioRuntimeSystemPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-audio-runtime-system"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        register_system(
            registry,
            SystemPhase::PostUpdate,
            "audio_runtime",
            move |runtime| audio::tick_audio_runtime(runtime, HOST_DELTA_SECONDS),
        )
    }
}
