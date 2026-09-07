use std::sync::Arc;

use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::context::WgpuRenderExtractorRegistry;

pub fn register_world_3d_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(WgpuMesh3dRenderExtractorBridge);
    registry.register(WgpuMaterial3dRenderExtractorBridge);
    registry.register(WgpuText3dRenderExtractorBridge);
    registry.register(WgpuNprRenderExtractorBridge);
}

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> Arc<T> {
    runtime
        .required::<T>()
        .expect("render extractor required service should be registered")
}

pub struct WgpuMesh3dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuMesh3dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_3d_mesh::Mesh3dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let mesh_scene_service = required::<amigo_3d_mesh::MeshSceneService>(runtime);
        amigo_3d_mesh::Mesh3dRenderExtractor.extract(
            amigo_3d_mesh::Mesh3dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                mesh_scene_service: mesh_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuMaterial3dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuMaterial3dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_3d_material::Material3dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let material_scene_service = required::<amigo_3d_material::MaterialSceneService>(runtime);
        amigo_3d_material::Material3dRenderExtractor.extract(
            amigo_3d_material::Material3dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                material_scene_service: material_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuText3dRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuText3dRenderExtractorBridge {
    fn name(&self) -> &'static str {
        amigo_3d_text::Text3dRenderExtractor.name()
    }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let scene_service = required::<SceneService>(runtime);
        let text3d_scene_service = required::<amigo_3d_text::Text3dSceneService>(runtime);
        amigo_3d_text::Text3dRenderExtractor.extract(
            amigo_3d_text::Text3dRenderExtractionContext {
                scene_service: scene_service.as_ref(),
                text3d_scene_service: text3d_scene_service.as_ref(),
            },
            packet,
        );
    }
}

pub struct WgpuNprRenderExtractorBridge;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for WgpuNprRenderExtractorBridge {
    fn name(&self) -> &'static str { amigo_npr_playground_plugin::render::NPR_PLAYGROUND_EXTRACTOR_ID }

    fn extract(&self, runtime: &Runtime, packet: &mut WgpuRenderFramePacket) {
        let Some(service) = runtime.resolve::<amigo_npr_playground_plugin::NprPlaygroundRenderService>() else { return; };
        if let Some(command) = service.snapshot() {
            packet.push_npr_draw_command(command);
        }
        if let Some(background) = service.background() {
            packet.set_npr_background(background);
        }
    }
}
