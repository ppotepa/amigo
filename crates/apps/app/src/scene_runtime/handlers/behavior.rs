use amigo_behavior::{
    BehaviorCommand, BehaviorCondition, BehaviorKind, FreeflightInputControllerBehavior,
    MenuNavigationControllerBehavior, ParticleIntensityControllerBehavior,
    ProjectileFireControllerBehavior, SceneTransitionControllerBehavior, UiThemeSwitcherBehavior,
};
use amigo_scene::BehaviorKindSceneCommand;

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
            thruster_emitter,
        } => BehaviorKind::FreeflightInputController(FreeflightInputControllerBehavior {
            target_entity,
            thrust_action,
            turn_action,
            strafe_action,
            thruster_emitter,
        }),
        BehaviorKindSceneCommand::ParticleIntensityController { emitter, action } => {
            BehaviorKind::ParticleIntensityController(ParticleIntensityControllerBehavior {
                emitter,
                action,
            })
        }
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
        BehaviorKindSceneCommand::UiThemeSwitcher {
            bindings,
            cycle_action,
        } => BehaviorKind::UiThemeSwitcher(UiThemeSwitcherBehavior {
            bindings,
            cycle_action,
        }),
    }
}
