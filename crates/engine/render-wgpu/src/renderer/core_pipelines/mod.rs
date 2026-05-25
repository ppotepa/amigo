use std::collections::BTreeMap;

mod color;
mod texture;

use color::{
    ColorAdditivePipelineProvider, ColorAlphaPipelineProvider, ColorMultiplyPipelineProvider,
    ColorScreenPipelineProvider,
};
use texture::{
    TextureAdditivePipelineProvider, TextureAlphaPipelineProvider, TextureLightenPipelineProvider,
    TextureMultiplyPipelineProvider, TextureOpaquePipelineProvider, TextureScreenPipelineProvider,
};

pub(crate) const CORE_COLOR_ALPHA_PIPELINE: &str = "core.color.alpha";
pub(crate) const CORE_COLOR_ADDITIVE_PIPELINE: &str = "core.color.additive";
pub(crate) const CORE_COLOR_MULTIPLY_PIPELINE: &str = "core.color.multiply";
pub(crate) const CORE_COLOR_SCREEN_PIPELINE: &str = "core.color.screen";

pub(crate) const CORE_TEXTURE_ALPHA_PIPELINE: &str = "core.texture.alpha";
pub(crate) const CORE_TEXTURE_OPAQUE_PIPELINE: &str = "core.texture.opaque";
pub(crate) const CORE_TEXTURE_ADDITIVE_PIPELINE: &str = "core.texture.additive";
pub(crate) const CORE_TEXTURE_MULTIPLY_PIPELINE: &str = "core.texture.multiply";
pub(crate) const CORE_TEXTURE_SCREEN_PIPELINE: &str = "core.texture.screen";
pub(crate) const CORE_TEXTURE_LIGHTEN_PIPELINE: &str = "core.texture.lighten";

pub(crate) struct WgpuCorePipelineCreateContext<'a> {
    pub(crate) device: &'a wgpu::Device,
    pub(crate) surface_format: wgpu::TextureFormat,
    pub(crate) texture_bind_group_layout: &'a wgpu::BindGroupLayout,
}

pub(crate) trait WgpuCorePipelineProvider {
    fn pipeline_id(&self) -> &'static str;

    fn create_pipeline(&self, ctx: &WgpuCorePipelineCreateContext<'_>) -> wgpu::RenderPipeline;
}

pub(crate) fn build_default_core_pipelines(
    ctx: &WgpuCorePipelineCreateContext<'_>,
) -> BTreeMap<&'static str, wgpu::RenderPipeline> {
    let providers: Vec<Box<dyn WgpuCorePipelineProvider>> = vec![
        Box::new(ColorAlphaPipelineProvider),
        Box::new(ColorAdditivePipelineProvider),
        Box::new(ColorMultiplyPipelineProvider),
        Box::new(ColorScreenPipelineProvider),
        Box::new(TextureAlphaPipelineProvider),
        Box::new(TextureOpaquePipelineProvider),
        Box::new(TextureAdditivePipelineProvider),
        Box::new(TextureMultiplyPipelineProvider),
        Box::new(TextureScreenPipelineProvider),
        Box::new(TextureLightenPipelineProvider),
    ];

    providers
        .into_iter()
        .map(|provider| {
            let id = provider.pipeline_id();
            let pipeline = provider.create_pipeline(ctx);
            (id, pipeline)
        })
        .collect()
}
