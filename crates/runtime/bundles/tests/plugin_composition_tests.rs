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
