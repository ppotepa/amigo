use crate::{PostFx2dService, ScopedPostFx2dStack};

#[derive(Clone, Copy)]
pub struct PostFx2dRenderExtractionContext<'a> {
    pub post_fx_service: &'a PostFx2dService,
    pub viewport_width: f32,
    pub viewport_height: f32,
}

pub trait PostFx2dRenderOutput {
    fn set_post_fx2d_stacks(&mut self, stacks: Vec<ScopedPostFx2dStack>);
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
        extract_post_fx2d_render_stacks(ctx, output);
    }
}

pub fn extract_post_fx2d_render_stacks(
    ctx: PostFx2dRenderExtractionContext<'_>,
    output: &mut dyn PostFx2dRenderOutput,
) {
    output.set_post_fx2d_stacks(ctx.post_fx_service.scoped_stacks());
}
