use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneBody2dCommandHandler;

impl SceneCommandHandler for SceneBody2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-body-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueKinematicBody2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueKinematicBody2d { command } => {
                let entity = amigo_2d_physics::queue_kinematic_body_scene_command(
                    ctx.scene_service,
                    ctx.physics_scene_service,
                    &command,
                );
                ctx.scene_event_queue
                    .publish(SceneEvent::KinematicBodyQueued {
                        entity_id: entity.raw(),
                        entity_name: command.entity_name.clone(),
                    });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d kinematic body `{}` from mod `{}`",
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
