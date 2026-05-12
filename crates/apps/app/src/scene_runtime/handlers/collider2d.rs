use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct SceneCollider2dCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for SceneCollider2dCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-collider-2d"
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

        match outcome {
            amigo_2d_physics::PhysicsSceneCommandOutcome::AabbCollider {
                entity_name,
                source_mod,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued 2d aabb collider `{}` from mod `{}`",
                    entity_name, source_mod
                ));
            }
            amigo_2d_physics::PhysicsSceneCommandOutcome::StaticCollider {
                entity_name,
                source_mod,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued 2d static collider `{}` from mod `{}`",
                    entity_name, source_mod
                ));
            }
            amigo_2d_physics::PhysicsSceneCommandOutcome::CircleCollider {
                entity_name,
                source_mod,
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued 2d circle collider `{}` from mod `{}`",
                    entity_name, source_mod
                ));
            }
            amigo_2d_physics::PhysicsSceneCommandOutcome::CollisionEventRule {
                id,
                source_mod,
                ..
            } => {
                ctx.dev_console_state.write_line(format!(
                    "queued 2d collision event rule `{}` from mod `{}`",
                    id, source_mod
                ));
            }
            _ => {
                return Err(AmigoError::Message(format!(
                    "{} received wrong physics outcome",
                    self.name()
                )));
            }
        }

        Ok(())
    }
}


