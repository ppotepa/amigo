use amigo_core::AmigoResult;

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
