use std::sync::Arc;

use amigo_2d_motion::{FreeflightMotionIntent2d, Motion2dSceneService};
use amigo_2d_particles::Particle2dSceneService;
use amigo_input_actions::InputActionService;
use amigo_input_api::InputState;

#[derive(Clone)]
pub struct ArcadeApi {
    pub(crate) actions: Option<Arc<InputActionService>>,
    pub(crate) input_state: Option<Arc<InputState>>,
    pub(crate) motion: Option<Arc<Motion2dSceneService>>,
    pub(crate) particles: Option<Arc<Particle2dSceneService>>,
}

impl ArcadeApi {
    pub fn drive_freeflight(
        &mut self,
        entity_name: &str,
        thrust_action: &str,
        turn_action: &str,
    ) -> bool {
        self.drive_freeflight_internal(entity_name, thrust_action, turn_action, None)
            .0
    }

    pub fn drive_freeflight_with_emitter(
        &mut self,
        entity_name: &str,
        emitter_name: &str,
        thrust_action: &str,
        turn_action: &str,
    ) -> bool {
        let (motion_ok, particles_ok) = self.drive_freeflight_internal(
            entity_name,
            thrust_action,
            turn_action,
            Some(emitter_name),
        );
        motion_ok && particles_ok
    }

    fn drive_freeflight_internal(
        &self,
        entity_name: &str,
        thrust_action: &str,
        turn_action: &str,
        emitter_name: Option<&str>,
    ) -> (bool, bool) {
        let (Some(actions), Some(input_state), Some(motion)) = (
            self.actions.as_ref(),
            self.input_state.as_ref(),
            self.motion.as_ref(),
        ) else {
            return (false, emitter_name.is_none());
        };

        let thrust = actions.axis(input_state, thrust_action);
        let turn = actions.axis(input_state, turn_action);
        let motion_ok = motion.drive_freeflight(
            entity_name,
            FreeflightMotionIntent2d {
                thrust,
                strafe: 0.0,
                turn,
            },
        );

        let Some(emitter_name) = emitter_name else {
            return (motion_ok, true);
        };
        let Some(particles) = self.particles.as_ref() else {
            return (motion_ok, false);
        };

        let intensity = thrust.abs().clamp(0.0, 1.0);
        let active_ok = particles.set_active(emitter_name, intensity > 0.01);
        let intensity_ok = particles.set_intensity(emitter_name, intensity);
        (motion_ok, active_ok && intensity_ok)
    }
}
