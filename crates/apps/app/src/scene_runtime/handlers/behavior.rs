use amigo_behavior::{
    BehaviorCommand, BehaviorCondition, BehaviorKind, CameraFollowModeControllerBehavior,
    FreeflightInputControllerBehavior, MenuNavigationControllerBehavior,
    ParticleIntensityControllerBehavior, ParticleProfileBurst, ParticleProfileControllerBehavior,
    ParticleProfileCurve4, ParticleProfilePhase, ParticleProfileScalar,
    ParticleProfileVelocityMode, ProjectileFireControllerBehavior,
    SceneAutoTransitionControllerBehavior, SceneTransitionControllerBehavior,
    SetStateOnActionControllerBehavior, ToggleStateControllerBehavior, UiThemeSwitcherBehavior,
};
use amigo_scene::{
    BehaviorKindSceneCommand, ParticleProfileBurstSceneCommand, ParticleProfileCurve4SceneCommand,
    ParticleProfilePhaseSceneCommand, ParticleProfileScalarSceneCommand,
    ParticleProfileVelocityModeSceneCommand,
};

use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::*;

pub(crate) struct SceneBehaviorCommandHandler;

impl SceneCommandHandler for SceneBehaviorCommandHandler {
    fn name(&self) -> &'static str {
        "scene-behavior"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueBehavior { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueBehavior { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.behavior_scene_service.queue(BehaviorCommand {
                    source_mod: command.source_mod.clone(),
                    entity_name: command.entity_name.clone(),
                    condition: command.condition.map(|condition| BehaviorCondition {
                        state_key: condition.state_key,
                        equals: condition.equals,
                        not_equals: condition.not_equals,
                        greater_than: condition.greater_than,
                        greater_or_equal: condition.greater_or_equal,
                        less_than: condition.less_than,
                        less_or_equal: condition.less_or_equal,
                        is_true: condition.is_true,
                        is_false: condition.is_false,
                    }),
                    behavior: behavior_from_scene_command(command.behavior),
                });
                ctx.scene_event_queue.publish(SceneEvent::BehaviorQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued behavior `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            other => Err(AmigoError::Message(format!(
                "{} cannot handle {}",
                self.name(),
                amigo_scene::format_scene_command(&other)
            ))),
        }
    }
}

fn behavior_from_scene_command(command: BehaviorKindSceneCommand) -> BehaviorKind {
    match command {
        BehaviorKindSceneCommand::FreeflightInputController {
            target_entity,
            thrust_action,
            turn_action,
            strafe_action,
        } => BehaviorKind::FreeflightInputController(FreeflightInputControllerBehavior {
            target_entity,
            thrust_action,
            turn_action,
            strafe_action,
        }),
        BehaviorKindSceneCommand::ParticleIntensityController { emitter, action } => {
            BehaviorKind::ParticleIntensityController(ParticleIntensityControllerBehavior {
                emitter,
                action,
            })
        }
        BehaviorKindSceneCommand::ParticleProfileController {
            emitter,
            action,
            max_hold_seconds,
            phases,
        } => BehaviorKind::ParticleProfileController(ParticleProfileControllerBehavior {
            emitter,
            action,
            max_hold_seconds,
            phases: phases
                .into_iter()
                .map(particle_profile_phase_from_scene_command)
                .collect(),
        }),
        BehaviorKindSceneCommand::CameraFollowModeController {
            camera,
            action,
            target,
            lerp,
            lookahead_velocity_scale,
            lookahead_max_distance,
            sway_amount,
            sway_frequency,
        } => BehaviorKind::CameraFollowModeController(CameraFollowModeControllerBehavior {
            camera,
            action,
            target,
            lerp,
            lookahead_velocity_scale,
            lookahead_max_distance,
            sway_amount,
            sway_frequency,
        }),
        BehaviorKindSceneCommand::ProjectileFireController {
            emitter,
            source,
            action,
            cooldown_seconds,
            cooldown_id,
            audio,
        } => BehaviorKind::ProjectileFireController(ProjectileFireControllerBehavior {
            emitter,
            source,
            action,
            cooldown_seconds,
            cooldown_id,
            audio,
        }),
        BehaviorKindSceneCommand::MenuNavigationController {
            index_state,
            item_count,
            item_count_state,
            up_action,
            down_action,
            confirm_action,
            wrap,
            move_audio,
            confirm_audio,
            confirm_events,
            selected_color_prefix,
            selected_color,
            unselected_color,
        } => BehaviorKind::MenuNavigationController(MenuNavigationControllerBehavior {
            index_state,
            item_count,
            item_count_state,
            up_action,
            down_action,
            confirm_action,
            wrap,
            move_audio,
            confirm_audio,
            confirm_events,
            selected_color_prefix,
            selected_color,
            unselected_color,
        }),
        BehaviorKindSceneCommand::SceneTransitionController { action, scene } => {
            BehaviorKind::SceneTransitionController(SceneTransitionControllerBehavior {
                action,
                scene,
            })
        }
        BehaviorKindSceneCommand::SceneAutoTransitionController { scene } => {
            BehaviorKind::SceneAutoTransitionController(SceneAutoTransitionControllerBehavior {
                scene,
            })
        }
        BehaviorKindSceneCommand::SetStateOnActionController {
            action,
            key,
            value,
            audio,
        } => BehaviorKind::SetStateOnActionController(SetStateOnActionControllerBehavior {
            action,
            key,
            value,
            audio,
        }),
        BehaviorKindSceneCommand::ToggleStateController {
            action,
            key,
            default,
            audio,
        } => BehaviorKind::ToggleStateController(ToggleStateControllerBehavior {
            action,
            key,
            default,
            audio,
        }),
        BehaviorKindSceneCommand::UiThemeSwitcher {
            bindings,
            cycle_action,
        } => BehaviorKind::UiThemeSwitcher(UiThemeSwitcherBehavior {
            bindings,
            cycle_action,
        }),
    }
}

fn particle_profile_phase_from_scene_command(
    phase: ParticleProfilePhaseSceneCommand,
) -> ParticleProfilePhase {
    ParticleProfilePhase {
        id: phase.id,
        start_seconds: phase.start_seconds,
        end_seconds: phase.end_seconds,
        velocity_mode: phase
            .velocity_mode
            .map(particle_profile_velocity_mode_from_scene_command),
        color_ramp: phase.color_ramp,
        spawn_rate: phase
            .spawn_rate
            .map(particle_profile_scalar_from_scene_command),
        lifetime: phase
            .lifetime
            .map(particle_profile_scalar_from_scene_command),
        lifetime_jitter: phase
            .lifetime_jitter
            .map(particle_profile_scalar_from_scene_command),
        speed: phase.speed.map(particle_profile_scalar_from_scene_command),
        speed_jitter: phase
            .speed_jitter
            .map(particle_profile_scalar_from_scene_command),
        spread_degrees: phase
            .spread_degrees
            .map(particle_profile_scalar_from_scene_command),
        initial_size: phase
            .initial_size
            .map(particle_profile_scalar_from_scene_command),
        final_size: phase
            .final_size
            .map(particle_profile_scalar_from_scene_command),
        spawn_area_line: phase
            .spawn_area_line
            .map(particle_profile_scalar_from_scene_command),
        shape_line: phase
            .shape_line
            .map(particle_profile_scalar_from_scene_command),
        shape_circle_weight: phase
            .shape_circle_weight
            .map(particle_profile_scalar_from_scene_command),
        shape_line_weight: phase
            .shape_line_weight
            .map(particle_profile_scalar_from_scene_command),
        shape_quad_weight: phase
            .shape_quad_weight
            .map(particle_profile_scalar_from_scene_command),
        size_curve: phase
            .size_curve
            .map(particle_profile_curve4_from_scene_command),
        speed_curve: phase
            .speed_curve
            .map(particle_profile_curve4_from_scene_command),
        alpha_curve: phase
            .alpha_curve
            .map(particle_profile_curve4_from_scene_command),
        burst: phase.burst.map(particle_profile_burst_from_scene_command),
        clear_forces: phase.clear_forces,
    }
}

fn particle_profile_velocity_mode_from_scene_command(
    mode: ParticleProfileVelocityModeSceneCommand,
) -> ParticleProfileVelocityMode {
    match mode {
        ParticleProfileVelocityModeSceneCommand::Free => ParticleProfileVelocityMode::Free,
        ParticleProfileVelocityModeSceneCommand::SourceInertial => {
            ParticleProfileVelocityMode::SourceInertial
        }
    }
}

fn particle_profile_scalar_from_scene_command(
    scalar: ParticleProfileScalarSceneCommand,
) -> ParticleProfileScalar {
    ParticleProfileScalar {
        from: scalar.from,
        to: scalar.to,
        curve: scalar.curve,
        intensity_scale: scalar.intensity_scale,
        noise_scale: scalar.noise_scale,
    }
}

fn particle_profile_curve4_from_scene_command(
    curve: ParticleProfileCurve4SceneCommand,
) -> ParticleProfileCurve4 {
    ParticleProfileCurve4 {
        v0: particle_profile_scalar_from_scene_command(curve.v0),
        v1: particle_profile_scalar_from_scene_command(curve.v1),
        v2: particle_profile_scalar_from_scene_command(curve.v2),
        v3: particle_profile_scalar_from_scene_command(curve.v3),
    }
}

fn particle_profile_burst_from_scene_command(
    burst: ParticleProfileBurstSceneCommand,
) -> ParticleProfileBurst {
    ParticleProfileBurst {
        rate_hz: burst.rate_hz,
        min_count: burst.min_count,
        max_count: burst.max_count,
        threshold: burst.threshold,
    }
}
