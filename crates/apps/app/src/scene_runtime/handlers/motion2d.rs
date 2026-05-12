use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct SceneMotion2dCommandHandler;

impl SceneCommandHandler for SceneMotion2dCommandHandler {
    fn name(&self) -> &'static str {
        "scene-motion-2d"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        amigo_2d_motion::can_handle_motion_scene_command(command)
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        let outcome = amigo_2d_motion::handle_motion_scene_command(
            amigo_2d_motion::MotionSceneCommandContext {
                scene_service: ctx.scene_service,
                motion_scene_service: ctx.motion_scene_service,
                entity_pool_scene_service: ctx.entity_pool_scene_service,
                lifetime_scene_service: ctx.lifetime_scene_service,
                scene_event_queue: ctx.scene_event_queue,
            },
            command,
        )?;

        match outcome {
            amigo_2d_motion::MotionSceneCommandOutcome::MotionController {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued 2d motion controller `{}` from mod `{}`",
                entity_name, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::EntityPool {
                pool,
                source_mod,
                member_count,
            } => ctx.dev_console_state.write_line(format!(
                "queued entity pool `{}` with {} members from mod `{}`",
                pool, member_count, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::Lifetime {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued lifetime `{}` from mod `{}`",
                entity_name, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::ProjectileEmitter {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued 2d projectile emitter `{}` from mod `{}`",
                entity_name, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::Velocity {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued 2d velocity `{}` from mod `{}`",
                entity_name, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::Bounds {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued 2d bounds `{}` from mod `{}`",
                entity_name, source_mod
            )),
            amigo_2d_motion::MotionSceneCommandOutcome::Freeflight {
                entity_name,
                source_mod,
            } => ctx.dev_console_state.write_line(format!(
                "queued 2d freeflight motion `{}` from mod `{}`",
                entity_name, source_mod
            )),
        }

        Ok(())
    }
}
