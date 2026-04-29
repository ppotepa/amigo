use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;
use super::super::super::*;

pub(crate) struct SceneCollider2dCommandHandler;

impl SceneCommandHandler for SceneCollider2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-collider-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueAabbCollider2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::QueueAabbCollider2d { command } => {
                let entity = amigo_2d_physics::queue_aabb_collider_scene_command(
                    ctx.scene_service,
                    ctx.physics_scene_service,
                    &command,
                );
                ctx.scene_event_queue.publish(SceneEvent::AabbColliderQueued {
                    entity_id: entity.raw(),
                    entity_name: command.entity_name.clone(),
                });
                ctx.dev_console_state.write_line(format!(
                    "queued 2d aabb collider `{}` from mod `{}`",
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
