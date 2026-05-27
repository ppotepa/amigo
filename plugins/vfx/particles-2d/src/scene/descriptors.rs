use amigo_plugin_api::PluginSceneComponentDescriptor;
use amigo_scene::{ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry};

pub fn particle_emitter_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.vfx.particles-2d.ParticleEmitter2D",
        "vfx",
        "ParticleEmitter2D",
    )
}

#[derive(Default)]
pub struct ParticleEmitter2dSceneDescriptorProvider;

impl ScenePluginDescriptorProvider for ParticleEmitter2dSceneDescriptorProvider {
    fn register_scene_descriptors(&self, registry: &mut ScenePluginDescriptorRegistry) {
        registry.insert(particle_emitter_2d_scene_descriptor());
    }
}
