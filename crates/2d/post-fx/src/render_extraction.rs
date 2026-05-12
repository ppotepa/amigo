use crate::{PostFx2dService, PostFx2dStack};

pub struct PostFx2dRenderExtractionContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub fn extract_post_fx2d_render_stack(
    ctx: PostFx2dRenderExtractionContext<'_>,
) -> Option<PostFx2dStack> {
    let stack = ctx.post_fx_service.scene_stack().normalized();
    (!stack.is_empty()).then_some(stack)
}
