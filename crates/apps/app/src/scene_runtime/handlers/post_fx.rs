use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use super::super::dispatcher::SceneCommandHandler;

pub(crate) struct ScenePostFxCommandHandler;

impl SceneCommandHandler for ScenePostFxCommandHandler {
    fn name(&self) -> &'static str {
        "scene-post-fx"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::SetPostFx2dStack { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'_>, command: SceneCommand) -> AmigoResult<()> {
        match command {
            SceneCommand::SetPostFx2dStack {
                stack,
                lens_certification_reports,
            } => {
                let post_fx = crate::runtime_context::required::<amigo_2d_post_fx::PostFx2dService>(
                    ctx.runtime,
                )?;
                post_fx.set_scene_stack(stack.clone());
                post_fx.set_lens_certification_reports(lens_certification_reports);
                ctx.dev_console_state.write_line(format!(
                    "queued 2d post-fx stack with {} effects",
                    stack.effects.len()
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
