use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::super::*;

pub(crate) struct ScenePlatformer2dCommandHandler;

impl SceneCommandHandler for ScenePlatformer2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-motion-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(
            command,
            SceneCommand::QueueMotionController2d { .. }
                | SceneCommand::QueuePlatformerController2d { .. }
        )
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueMotionController2d { command }
            | SceneCommand::QueuePlatformerController2d { command } => {
                let entity = ctx
                    .scene_service
                    .find_or_spawn_named_entity(command.entity_name.clone());
                ctx.platformer_scene_service
                    .queue(PlatformerController2dCommand {
                        entity_id: entity,
                        entity_name: command.entity_name.clone(),
                        controller: PlatformerController2d {
                            params: PlatformerControllerParams {
                                max_speed: command.max_speed,
                                acceleration: command.acceleration,
                                deceleration: command.deceleration,
                                air_acceleration: command.air_acceleration,
                                gravity: command.gravity,
                                jump_velocity: command.jump_velocity,
                                terminal_velocity: command.terminal_velocity,
                            },
                        },
                    });
                ctx.scene_event_queue
                    .publish(SceneEvent::MotionControllerQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d motion controller `{}` from mod `{}`",
                    command.entity_name, command.source_mod
                ));
                Ok(())
            }
            _ => Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            ))),
        }
    }
}
