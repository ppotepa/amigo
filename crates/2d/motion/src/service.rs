use std::sync::Mutex;

use amigo_2d_physics::Physics2dSceneService;
use amigo_math::Vec2;
use amigo_scene::{EntityPoolSceneService, SceneEntityId, SceneService};

use crate::{
    bounds::{Bounds2d, Bounds2dCommand},
    controller::{MotionController2dCommand, MotionIntent2d, MotionState2d},
    freeflight::{FreeflightMotion2dCommand, FreeflightMotionIntent2d, FreeflightMotionState2d},
    projectile::{ProjectileEmitter2dCommand, projectile_launch_2d},
    registry::MotionStateRegistry,
    velocity::{Velocity2d, Velocity2dCommand},
};

#[derive(Debug, Default)]
pub struct Motion2dSceneService {
    state: Mutex<MotionStateRegistry>,
}

impl Motion2dSceneService {
    pub fn queue_motion_controller(&self, command: MotionController2dCommand) {
        self.queue(command);
    }

    pub fn queue_projectile_emitter(&self, command: ProjectileEmitter2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .projectile_emitters
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_velocity(&self, command: Velocity2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_bounds(&self, command: Bounds2dCommand) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue_freeflight(&self, command: FreeflightMotion2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .freeflight_states
            .insert(command.entity_name.clone(), command.initial_state);
        state
            .freeflight_intents
            .entry(command.entity_name.clone())
            .or_insert_with(FreeflightMotionIntent2d::default);
        state
            .freeflight_commands
            .insert(command.entity_name.clone(), command);
    }

    pub fn queue(&self, command: MotionController2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .states
            .entry(command.entity_name.clone())
            .or_insert_with(MotionState2d::default);
        state
            .motors
            .entry(command.entity_name.clone())
            .or_insert_with(MotionIntent2d::default);
        state.commands.insert(command.entity_name.clone(), command);
    }

    pub fn clear(&self) {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state.commands.clear();
        state.states.clear();
        state.motors.clear();
        state.velocities.clear();
        state.bounds.clear();
        state.freeflight_commands.clear();
        state.freeflight_states.clear();
        state.freeflight_intents.clear();
        state.projectile_emitters.clear();
    }

    pub fn commands(&self) -> Vec<MotionController2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .commands
            .values()
            .cloned()
            .collect()
    }

    pub fn motion_controller_commands(&self) -> Vec<MotionController2dCommand> {
        self.commands()
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }

    pub fn motion_entity_names(&self) -> Vec<String> {
        self.entity_names()
    }

    pub fn projectile_emitter(&self, entity_name: &str) -> Option<ProjectileEmitter2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .projectile_emitters
            .get(entity_name)
            .cloned()
    }

    pub fn velocities(&self) -> Vec<Velocity2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .values()
            .cloned()
            .collect()
    }

    pub fn velocity(&self, entity_name: &str) -> Option<Velocity2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .velocities
            .get(entity_name)
            .map(|command| command.velocity)
    }

    pub fn set_velocity(&self, entity_name: &str, velocity: Vec2) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        state
            .velocities
            .entry(entity_name.to_owned())
            .and_modify(|command| command.velocity = Velocity2d::new(velocity))
            .or_insert_with(|| Velocity2dCommand {
                entity_id: SceneEntityId::new(0),
                entity_name: entity_name.to_owned(),
                velocity: Velocity2d::new(velocity),
            });
        true
    }

    pub fn bounds(&self) -> Vec<Bounds2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .values()
            .cloned()
            .collect()
    }

    pub fn bounds_for(&self, entity_name: &str) -> Option<Bounds2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .bounds
            .get(entity_name)
            .map(|command| command.bounds)
    }

    pub fn freeflight_commands(&self) -> Vec<FreeflightMotion2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_commands
            .values()
            .cloned()
            .collect()
    }

    pub fn freeflight_command(&self, entity_name: &str) -> Option<FreeflightMotion2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_commands
            .get(entity_name)
            .cloned()
    }

    pub fn freeflight_state(&self, entity_name: &str) -> Option<FreeflightMotionState2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_states
            .get(entity_name)
            .copied()
    }

    pub fn sync_freeflight_state(
        &self,
        entity_name: &str,
        state_value: FreeflightMotionState2d,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state
            .freeflight_states
            .insert(entity_name.to_owned(), state_value);
        true
    }

    pub fn reset_freeflight(&self, entity_name: &str, rotation_radians: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state.freeflight_states.insert(
            entity_name.to_owned(),
            FreeflightMotionState2d {
                velocity: Vec2::ZERO,
                angular_velocity: 0.0,
                rotation_radians,
            },
        );
        state
            .freeflight_intents
            .insert(entity_name.to_owned(), FreeflightMotionIntent2d::default());
        true
    }

    pub fn drive_freeflight(&self, entity_name: &str, intent: FreeflightMotionIntent2d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.freeflight_commands.contains_key(entity_name) {
            return false;
        }
        state.freeflight_intents.insert(
            entity_name.to_owned(),
            FreeflightMotionIntent2d {
                thrust: intent.thrust.clamp(-1.0, 1.0),
                strafe: intent.strafe.clamp(-1.0, 1.0),
                turn: intent.turn.clamp(-1.0, 1.0),
            },
        );
        true
    }

    pub fn freeflight_intent(&self, entity_name: &str) -> Option<FreeflightMotionIntent2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_intents
            .get(entity_name)
            .cloned()
    }

    pub fn clear_freeflight_intent(&self, entity_name: &str) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .freeflight_intents
            .insert(entity_name.to_owned(), FreeflightMotionIntent2d::default());
    }

    pub fn current_velocity(&self, entity_name: &str) -> Vec2 {
        self.freeflight_state(entity_name)
            .map(|state| state.velocity)
            .or_else(|| self.velocity(entity_name).map(|velocity| velocity.linear))
            .unwrap_or(Vec2::ZERO)
    }

    pub fn fire_projectile_from_emitter(
        &self,
        scene_service: &SceneService,
        pool_service: &EntityPoolSceneService,
        physics_scene_service: Option<&Physics2dSceneService>,
        emitter_entity_name: &str,
    ) -> Option<String> {
        let command = self.projectile_emitter(emitter_entity_name)?;
        let source_transform = scene_service.transform_of(emitter_entity_name)?;
        let source_velocity = physics_scene_service
            .and_then(|service| service.body_state(emitter_entity_name))
            .map(|state| state.velocity)
            .unwrap_or_else(|| self.current_velocity(emitter_entity_name));
        let projectile_entity = pool_service.acquire(scene_service, &command.emitter.pool)?;
        let launch = projectile_launch_2d(source_transform, source_velocity, &command.emitter);
        let _ = scene_service.set_transform(&projectile_entity, launch.transform);
        if let Some(physics_scene_service) = physics_scene_service {
            if let Some(mut body_state) = physics_scene_service.body_state(&projectile_entity) {
                body_state.velocity = launch.velocity;
                let _ = physics_scene_service.sync_body_state(&projectile_entity, body_state);
            }
        }
        let _ = self.set_velocity(&projectile_entity, launch.velocity);
        Some(projectile_entity)
    }

    pub fn controller(&self, entity_name: &str) -> Option<MotionController2dCommand> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .commands
            .get(entity_name)
            .cloned()
    }

    pub fn motion_controller(&self, entity_name: &str) -> Option<MotionController2dCommand> {
        self.controller(entity_name)
    }

    pub fn drive(
        &self,
        entity_name: &str,
        move_x: f32,
        jump_pressed: bool,
        jump_held: bool,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.motors.insert(
            entity_name.to_owned(),
            MotionIntent2d {
                move_x: move_x.clamp(-1.0, 1.0),
                jump_pressed,
                jump_held,
            },
        );
        true
    }

    pub fn drive_motion(
        &self,
        entity_name: &str,
        move_x: f32,
        jump_pressed: bool,
        jump_held: bool,
    ) -> bool {
        self.drive(entity_name, move_x, jump_pressed, jump_held)
    }

    pub fn motor(&self, entity_name: &str) -> Option<MotionIntent2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .motors
            .get(entity_name)
            .cloned()
    }

    pub fn motion_intent(&self, entity_name: &str) -> Option<MotionIntent2d> {
        self.motor(entity_name)
    }

    pub fn clear_motor(&self, entity_name: &str) {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .motors
            .insert(entity_name.to_owned(), MotionIntent2d::default());
    }

    pub fn clear_motion_intent(&self, entity_name: &str) {
        self.clear_motor(entity_name);
    }

    pub fn state(&self, entity_name: &str) -> Option<MotionState2d> {
        self.state
            .lock()
            .expect("motion scene service mutex should not be poisoned")
            .states
            .get(entity_name)
            .cloned()
    }

    pub fn motion_state(&self, entity_name: &str) -> Option<MotionState2d> {
        self.state(entity_name)
    }

    pub fn sync_state(&self, entity_name: &str, state_value: MotionState2d) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("motion scene service mutex should not be poisoned");
        if !state.commands.contains_key(entity_name) {
            return false;
        }
        state.states.insert(entity_name.to_owned(), state_value);
        true
    }

    pub fn sync_motion_state(&self, entity_name: &str, state_value: MotionState2d) -> bool {
        self.sync_state(entity_name, state_value)
    }
}
