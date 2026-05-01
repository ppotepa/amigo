use amigo_2d_motion::{FreeflightMotionIntent2d, Motion2dSceneService, projectile_launch_2d};
use amigo_2d_particles::{
    Particle2dSceneService, ParticleShape2d, ParticleSpawnArea2d, ParticleVelocityMode2d,
    WeightedParticleShape2d,
};
use amigo_2d_physics::Physics2dSceneService;
use amigo_audio_api::{AudioClipKey, AudioCommand, AudioCommandQueue};
use amigo_behavior::{
    BehaviorKind, BehaviorSceneService, ParticleProfileCurve4, ParticleProfilePhase,
    ParticleProfileScalar, ParticleProfileVelocityMode,
};
use amigo_core::AmigoResult;
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;
use amigo_runtime::Runtime;
use amigo_scene::{
    CameraFollow2dSceneCommand, CameraFollow2dSceneService, EntityPoolSceneService,
    LifetimeSceneService, SceneCommand, SceneCommandQueue, SceneKey, SceneService,
};
use amigo_scripting_api::{ScriptEvent, ScriptEventQueue};
use amigo_state::SceneStateService;
use amigo_state::SceneTimerService;
use amigo_ui::UiThemeService;

use crate::runtime_context::required;

include!("behavior/tick.rs");
include!("behavior/menu.rs");
include!("behavior/particle_profile.rs");
include!("behavior/actions.rs");

#[cfg(test)]
include!("behavior/tests.rs");
