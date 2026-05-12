use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneBody2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneBody2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-body-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_physics::can_handle_physics_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_physics::handle_physics_scene_command(
            amigo_2d_physics::PhysicsSceneCommandContext {
                scene_service: ctx.scene_service,
                physics_scene_service: ctx.physics_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;
        let amigo_2d_physics::PhysicsSceneCommandOutcome::KinematicBody {
            entity_name,
            source_mod,
        } = outcome
        else {
            return Err(AmigoError::Message(format!("{} received wrong physics outcome", self.name())));
        };
        ctx.dev_console_state.write_line(format!(
            "queued 2d kinematic body `{}` from mod `{}`",
            entity_name, source_mod
        ));
        Ok(())
    }
}


