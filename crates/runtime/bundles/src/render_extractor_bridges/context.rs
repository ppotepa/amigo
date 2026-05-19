use amigo_render_api::RenderFrameExtractorRegistry;
use amigo_render_wgpu::WgpuRenderFramePacket;

pub type WgpuRenderExtractorRegistry =
    RenderFrameExtractorRegistry<amigo_runtime::Runtime, WgpuRenderFramePacket>;
