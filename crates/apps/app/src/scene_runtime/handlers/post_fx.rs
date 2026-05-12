use super::super::super::*;
use super::super::context::AppSceneCommandContext;
use amigo_session::SceneCommandHandler;

pub(crate) struct ScenePostFxCommandHandler;

impl<'a> SceneCommandHandler<AppSceneCommandContext<'a>, SceneCommand, AmigoResult<()>>
    for ScenePostFxCommandHandler
{
    fn name(&self) -> &'static str {
        "scene-post-fx"
    }

    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::SetPostFx2dStack { .. })
    }

    fn handle(&self, ctx: &AppSceneCommandContext<'a>, command: SceneCommand) -> AmigoResult<()> {
        let post_fx =
            crate::runtime_context::required::<amigo_2d_post_fx::PostFx2dService>(ctx.runtime)?;
        let SceneCommand::SetPostFx2dStack {
            stack,
            lens_certification_reports,
        } = command
        else {
            return Err(AmigoError::Message(format!(
                "{} cannot handle command {}",
                self.name(),
                amigo_scene::format_scene_command(&command)
            )));
        };
        let outcome = amigo_2d_post_fx::handle_post_fx_scene_stack(
            amigo_2d_post_fx::PostFxSceneCommandContext {
                post_fx2d_service: &post_fx,
            },
            stack,
            lens_certification_reports,
        )?;

        ctx.dev_console_state.write_line(format!(
            "queued 2d post-fx stack with {} effects",
            outcome.effect_count
        ));

        Ok(())
    }
}


