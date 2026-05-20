use amigo_core::{AmigoError, AmigoResult};
use amigo_scene::{format_scene_command, RuntimeSceneCommandHandler, SceneCommand};

use crate::{LensDroplets2dCertificationReport, PostFx2dService, ScopedPostFx2dStack};

pub struct PostFxSceneCommandContext<'a> {
    pub post_fx2d_service: &'a PostFx2dService,
}

#[derive(Debug, Clone)]
pub struct PostFxSceneCommandOutcome {
    pub effect_count: usize,
}

pub fn handle_post_fx_scoped_stacks(
    ctx: PostFxSceneCommandContext<'_>,
    stacks: Vec<ScopedPostFx2dStack>,
    lens_certification_reports: Vec<LensDroplets2dCertificationReport>,
) -> AmigoResult<PostFxSceneCommandOutcome> {
    let effect_count = stacks.iter().map(|stack| stack.effects.len()).sum();
    ctx.post_fx2d_service.set_scoped_stacks(stacks);
    ctx.post_fx2d_service
        .set_lens_certification_reports(lens_certification_reports);
    Ok(PostFxSceneCommandOutcome { effect_count })
}

pub struct CompositePostFx2dRuntimeSceneCommandHandler;

impl RuntimeSceneCommandHandler for CompositePostFx2dRuntimeSceneCommandHandler {
    fn can_handle(&self, command: &SceneCommand) -> bool {
        matches!(command, SceneCommand::SetPostFx2dStacks { .. })
    }

    fn handle(&self, runtime: &amigo_runtime::Runtime, command: SceneCommand) -> AmigoResult<()> {
        let post_fx = runtime.required::<PostFx2dService>()?;
        let SceneCommand::SetPostFx2dStacks {
            stacks,
            lens_certification_reports,
        } = command
        else {
            return Err(AmigoError::Message(format!(
                "composite-post-fx-2d runtime handler cannot handle command {}",
                format_scene_command(&command)
            )));
        };

        handle_post_fx_scoped_stacks(
            PostFxSceneCommandContext {
                post_fx2d_service: post_fx.as_ref(),
            },
            stacks,
            lens_certification_reports,
        )?;
        Ok(())
    }
}
