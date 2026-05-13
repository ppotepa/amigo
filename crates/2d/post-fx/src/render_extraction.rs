use crate::{PostFx2dService, PostFx2dStack};

pub struct PostFx2dRenderExtractionContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
}

pub trait PostFx2dRenderOutput {
    fn set_post_fx2d_stack(&mut self, stack: PostFx2dStack);
}

pub struct PostFx2dRenderExtractor;

impl PostFx2dRenderExtractor {
    pub fn name(&self) -> &'static str {
        "post_fx_2d"
    }

    pub fn extract(
        &self,
        ctx: PostFx2dRenderExtractionContext<'_>,
        output: &mut impl PostFx2dRenderOutput,
    ) {
        if let Some(stack) = extract_post_fx2d_render_stack(ctx) {
            output.set_post_fx2d_stack(stack);
        }
    }
}

pub fn extract_post_fx2d_render_stack(
    ctx: PostFx2dRenderExtractionContext<'_>,
) -> Option<PostFx2dStack> {
    let stack = ctx.post_fx_service.scene_stack().normalized();
    (!stack.is_empty()).then_some(stack)
}

