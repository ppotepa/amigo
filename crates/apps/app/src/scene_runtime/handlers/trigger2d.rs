use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneTrigger2dCommandHandler;

impl SceneCommandHandler for SceneTrigger2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-trigger-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::QueueTrigger2d { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_physics::handle_physics_scene_command(
            amigo_2d_physics::PhysicsSceneCommandContext {
                scene_service: ctx.scene_service,
                physics_scene_service: ctx.physics_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;
        let amigo_2d_physics::PhysicsSceneCommandOutcome::Trigger {
            entity_name,
            source_mod,
            ..
        } = outcome
        else {
            return Err(AmigoError::Message(format!("{} received wrong physics outcome", self.name())));
        };
        ctx.dev_console_state.write_line(format!(
            "queued 2d trigger `{}` from mod `{}`",
            entity_name, source_mod
        ));
        Ok(())
    }
}
