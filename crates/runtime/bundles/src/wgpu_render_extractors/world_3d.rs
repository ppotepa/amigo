use std::sync::Arc;

use amigo_render_api::RenderFrameExtractor;
use amigo_render_wgpu::WgpuRenderFramePacket;
use amigo_runtime::Runtime;
use amigo_scene::SceneService;

use super::context::WgpuRenderExtractorRegistry;

pub fn register_world_3d_render_extractors(registry: &mut WgpuRenderExtractorRegistry) {
    registry.register(AppMesh3dRenderExtractor);
    registry.register(AppMaterial3dRenderExtractor);
    registry.register(AppText3dRenderExtractor);
}

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> Arc<T> {
    runtime
        .required::<T>()
        .expect("render extractor required service should be registered")
}

pub struct AppMesh3dRenderExtractor;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for AppMesh3dRenderExtractor {
    fn name(&self) -> &'static str { amigo_3d_mesh::Mesh3dRenderExtractor.name() }

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

pub struct AppMaterial3dRenderExtractor;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for AppMaterial3dRenderExtractor {
    fn name(&self) -> &'static str { amigo_3d_material::Material3dRenderExtractor.name() }

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

pub struct AppText3dRenderExtractor;

impl RenderFrameExtractor<Runtime, WgpuRenderFramePacket> for AppText3dRenderExtractor {
    fn name(&self) -> &'static str { amigo_3d_text::Text3dRenderExtractor.name() }

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


