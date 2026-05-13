use amigo_core::AmigoResult;

use crate::{LensDroplets2dCertificationReport, PostFx2dService, PostFx2dStack};

pub struct PostFxSceneCommandContext<'a> {
    pub post_fx2d_service: &'a PostFx2dService,
}

#[derive(Debug, Clone)]
pub struct PostFxSceneCommandOutcome {
    pub effect_count: usize,
}

pub fn handle_post_fx_scene_stack(
    ctx: PostFxSceneCommandContext<'_>,
    stack: PostFx2dStack,
    lens_certification_reports: Vec<LensDroplets2dCertificationReport>,
) -> AmigoResult<PostFxSceneCommandOutcome> {
    let effect_count = stack.effects.len();
    ctx.post_fx2d_service.set_scene_stack(stack);
    ctx.post_fx2d_service
        .set_lens_certification_reports(lens_certification_reports);
    Ok(PostFxSceneCommandOutcome { effect_count })
}

