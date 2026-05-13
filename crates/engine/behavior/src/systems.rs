use amigo_2d_motion::{FreeflightMotionIntent2d, Motion2dSceneService, projectile_launch_2d};
use amigo_2d_particles::{
    Particle2dSceneService, ParticleShape2d, ParticleSpawnArea2d, ParticleVelocityMode2d,
    WeightedParticleShape2d,
};
use amigo_2d_physics::Physics2dSceneService;
use amigo_audio_api::{AudioClipKey, AudioCommand, AudioCommandQueue};
use amigo_core::{AmigoError, AmigoResult};
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;
use amigo_runtime::Runtime;
use amigo_camera::CameraFollow2dSceneService;
use amigo_scene::{
    CameraFollow2dSceneCommand, EntityPoolSceneService, LifetimeSceneService, SceneCommand,
    SceneCommandQueue, SceneKey, SceneService,
};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use amigo_state::SceneStateService;
use amigo_state::SceneTimerService;
use amigo_ui::UiThemeService;

use crate::{BehaviorKind, BehaviorSceneService, ParticleProfileCurve4, ParticleProfilePhase, ParticleProfileScalar, ParticleProfileVelocityMode};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

include!("systems/tick.rs");
include!("systems/menu.rs");
include!("systems/particle_profile.rs");
include!("systems/actions.rs");

#[cfg(test)]
include!("systems/tests.rs");

