use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;

use crate::render_extractor_bridges::context::WgpuRenderExtractorRegistry;

use super::camera_capture::update_camera_2d_capture;
use super::common::optional;

pub fn register(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuPostFx2dRenderExtractorBridge);
}

pub struct WgpuPostFx2dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuPostFx2dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_composite_plugin::PostFx2dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        extract_post_fx_2d(runtime, packet);

        if let Some(camera_service) = runtime.resolve::<amigo_camera_core_plugin::CameraService>() {
            update_camera_2d_capture(runtime, camera_service.as_ref(), packet);
        }
    }
}

fn extract_post_fx_2d(runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
    let Some(post_fx_service) = optional::<amigo_composite_plugin::PostFx2dService>(runtime) else {
        return;
    };
    let viewport = runtime
        .resolve::<amigo_ui::UiInputViewportState>()
        .and_then(|state| state.get())
        .unwrap_or_else(|| amigo_render_wgpu::UiViewportSize::new(1280.0, 720.0));
    amigo_composite_plugin::PostFx2dRenderExtractor.extract(
        amigo_composite_plugin::PostFx2dRenderExtractionContext {
            post_fx_service: post_fx_service.as_ref(),
            viewport_width: viewport.width,
            viewport_height: viewport.height,
        },
        packet,
    );
}
