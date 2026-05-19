use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn particle_emitter_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.vfx.particles-2d.ParticleEmitter2D",
        "vfx",
        "ParticleEmitter2D",
    )
}

