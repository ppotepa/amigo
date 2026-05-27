use amigo_runtime_bundles::default_camera_2d_plugin_composition;

#[test]
fn default_camera_2d_composition_declares_plugins_and_contracts() {
    let composition = default_camera_2d_plugin_composition();

    assert!(
        composition
            .plugins
            .iter()
            .any(|plugin| plugin.0 == "amigo.camera.camera-optics")
    );
    assert!(
        composition
            .required_capabilities
            .iter()
            .any(|capability| capability.0 == "camera.optics.2d")
    );
    assert!(
        composition
            .required_slots
            .iter()
            .any(|slot| slot.0 == "camera.optics.consumer.2d")
    );
}

#[test]
fn two_d_bundle_bridges_motion_velocity_into_particle_sources() {
    use amigo_math::Vec2;
    use amigo_runtime::RuntimeBuilder;
    use amigo_runtime_bundles::{CoreRuntimeBundle, TwoDRuntimeBundle};

    let runtime = RuntimeBuilder::default()
        .with_bundle(CoreRuntimeBundle)
        .expect("core bundle should register")
        .with_bundle(TwoDRuntimeBundle)
        .expect("2d bundle should register")
        .build();

    let motion = runtime
        .required::<amigo_shutter_motion_plugin::Motion2dSceneService>()
        .expect("2d bundle should register motion service");
    motion.set_velocity("ship", Vec2::new(3.0, -2.0));

    let particle_velocity = runtime
        .required::<amigo_particles_2d_plugin::Particle2dSourceVelocityProviderRegistry>()
        .expect("particles plugin should register source velocity providers")
        .source_velocity("ship");

    assert_eq!(particle_velocity, Some(Vec2::new(3.0, -2.0)));
}
