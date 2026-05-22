use amigo_core::AmigoResult;
use amigo_render_api::{PostFx2d, PostFx2dId, PostFxHost2dId, PostFxRenderDescriptor, RenderFeatureId};

use crate::{
    WgpuOffscreenTarget,
    renderer::service::{WgpuFrameRenderRequest, WgpuSceneRenderer},
};

pub(crate) struct WgpuPostFxExecutionContext<'a> {
    pub(crate) request: &'a WgpuFrameRenderRequest<'a>,
    pub(crate) host_id: &'a PostFxHost2dId,
    pub(crate) effect_id: &'a PostFx2dId,
    pub(crate) feature_id: &'a RenderFeatureId,
    pub(crate) descriptor: &'a PostFxRenderDescriptor,
    pub(crate) effect: PostFx2d,
    pub(crate) input_view: &'a wgpu::TextureView,
    pub(crate) output: &'a mut WgpuOffscreenTarget,
}

pub(crate) trait WgpuPostFxExecutor: Send + Sync {
    fn executor_id(&self) -> &'static str;

    fn execute(
        &self,
        renderer: &mut WgpuSceneRenderer,
        ctx: WgpuPostFxExecutionContext<'_>,
    ) -> AmigoResult<()>;
}
