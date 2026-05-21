mod beacon;
mod layered_image;
mod particle;
mod text;
mod textured_quad;
mod tilemap;
mod vector;

use std::sync::OnceLock;

use crate::WgpuRenderable2dAdapterRegistry;

pub(crate) fn default_renderable_2d_adapter_registry() -> &'static WgpuRenderable2dAdapterRegistry {
    static REGISTRY: OnceLock<WgpuRenderable2dAdapterRegistry> = OnceLock::new();

    REGISTRY.get_or_init(|| {
        let mut registry = WgpuRenderable2dAdapterRegistry::default();
        registry.register(tilemap::TileBatch2dRenderableAdapter);
        registry.register(layered_image::LayeredTexturedQuads2dRenderableAdapter);
        registry.register(vector::VectorMesh2dRenderableAdapter);
        registry.register(beacon::RadialLightVisual2dRenderableAdapter);
        registry.register(textured_quad::TexturedQuad2dRenderableAdapter);
        registry.register(text::Text2dRenderableAdapter);
        registry.register(particle::ParticleBatch2dRenderableAdapter);
        registry
    })
}
