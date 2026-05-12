//! Frame systems executed by the main application runtime.
//! They advance gameplay, UI, scripting, audio, and scene transitions after bootstrap.

pub(crate) mod audio;
pub(crate) mod scene_transition;
pub(crate) mod script_update;
pub(crate) mod ui_input;

use std::sync::Mutex;

use amigo_core::AmigoResult;
use amigo_render_wgpu::UiViewportSize;
use amigo_runtime::{
    EngineTaskSystem, RuntimePlugin, ServiceRegistry, SystemPhase, SystemRegistry,
};
use amigo_session::{
    runtime_capabilities::{
        RuntimeCapabilityDescriptor, RuntimeCapabilityKind, RuntimeCapability,
        RuntimeDomainId, SystemContribution, SystemDescriptor, SystemProvider,
    },
    RuntimeSession,
};

use crate::runtime_context::{required, required_from_registry};

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

pub(crate) fn run_app_system_phase_for_session(
    session: &RuntimeSession,
    phase: SystemPhase,
) -> AmigoResult<()> {
    let phase_name = match phase {
        SystemPhase::PreUpdate => "pre_update",
        SystemPhase::Update => "update",
        SystemPhase::PostUpdate => "post_update",
        _ => "other",
    };
    let _session_systems = session
        .runtime_capabilities()
        .descriptors_by_kind(RuntimeCapabilityKind::SystemPhaseHandler)
        .filter(|descriptor| descriptor.kind == RuntimeCapabilityKind::SystemPhaseHandler)
        .filter(|descriptor| descriptor.id.ends_with(phase_name))
        .map(|descriptor| (&descriptor.id, &descriptor.label))
        .collect::<Vec<_>>();

    let systems = required::<SystemRegistry>(session.runtime())?;
    session.begin_system_phase(phase);

    if let Err(error) = systems.run_phase(phase, session.runtime()) {
        session.mark_scheduler_error(phase, format!("system phase {phase:?} failed: {error}"));
        return Err(error);
    }

    session.complete_system_phase(phase);
    Ok(())
}

pub(crate) struct AppSystemsProvider;

impl SystemProvider for AppSystemsProvider {
    fn register_system_phase_contributions(&self, descriptors: &mut Vec<SystemDescriptor>) {
        let _ = descriptors;
    }
}

pub(crate) fn register_app_systems_provider(
    session: &mut RuntimeSession,
) -> Vec<SystemContribution> {
    let mut descriptors = Vec::new();
    AppSystemsProvider.register_system_phase_contributions(&mut descriptors);
    let contributions = descriptors
        .into_iter()
        .map(|descriptor| SystemContribution {
            descriptor: descriptor.clone(),
        })
        .collect::<Vec<_>>();

    for contribution in &contributions {
        session
            .runtime_capabilities_mut()
            .register(RuntimeCapability {
                descriptor: RuntimeCapabilityDescriptor {
                    domain_id: RuntimeDomainId::new("app.host"),
                    kind: RuntimeCapabilityKind::SystemPhaseHandler,
                    id: format!("{}.{}", contribution.descriptor.system_id, contribution.descriptor.phase),
                    label: format!("System {}", contribution.descriptor.system_id),
                    description: "app legacy system phase handler".to_string(),
                    capabilities: contribution.descriptor.capabilities.clone(),
                    tags: contribution.descriptor.tags.clone(),
                    migration_seam: contribution.descriptor.migration_seam,
                },
            });
    }

    contributions
}

pub(crate) struct RuntimeSystemServicesPlugin;

impl RuntimePlugin for RuntimeSystemServicesPlugin {
    fn name(&self) -> &'static str {
        "amigo-app-runtime-system-services"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(UiInputViewportState::default())?;
        registry.register(crate::render_runtime::RenderFrameStatsService::default())?;
        registry.register(crate::render_runtime::RenderCompositionDiagnosticsService::default())?;
        registry.register(crate::debug_overlay::DebugOverlayService::default())?;
        registry.register(amigo_session::AppSchedulingService::default())?;
        registry.register(EngineTaskSystem::default())?;
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
        register_system(registry, SystemPhase::Update, "behavior", move |runtime| {
            amigo_behavior::tick_behaviors(runtime, HOST_DELTA_SECONDS)
        })?;
        register_system(
            registry,
            SystemPhase::Update,
            "script_components",
            move |runtime| amigo_scripting_rhai::tick_script_components(runtime, HOST_DELTA_SECONDS),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "script_update",
            move |runtime| script_update::tick_active_scripts(runtime, HOST_DELTA_SECONDS),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "ui_bindings",
            move |runtime| amigo_ui::tick_ui_bindings(runtime),
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
            amigo_2d_motion::tick_motion_2d_world(runtime, HOST_DELTA_SECONDS)
        })?;
        register_system(
            registry,
            SystemPhase::Update,
            "camera_follow_2d",
            move |runtime| amigo_scene::tick_camera_follow_world(runtime, HOST_DELTA_SECONDS),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "particles_2d",
            move |runtime| amigo_2d_particles::tick_particles_2d_world(runtime, HOST_DELTA_SECONDS),
        )?;
        register_system(registry, SystemPhase::Update, "lifetime", move |runtime| {
            amigo_scene::tick_lifetimes(runtime, HOST_DELTA_SECONDS)
        })?;
        register_system(
            registry,
            SystemPhase::Update,
            "collision_events_2d",
            move |runtime| amigo_2d_physics::tick_collision_events_2d(runtime),
        )?;
        register_system(
            registry,
            SystemPhase::Update,
            "parallax_2d",
            move |runtime| amigo_scene::tick_parallax_world(runtime),
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
