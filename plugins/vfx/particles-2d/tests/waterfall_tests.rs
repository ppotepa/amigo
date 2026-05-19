#[test]
fn particles_2d_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_particles_2d_plugin::scene::particle_emitter_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.vfx.particles-2d.ParticleEmitter2D"
    );
}
