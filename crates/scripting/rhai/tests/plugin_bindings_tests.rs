use amigo_scripting_rhai::RhaiPluginBindingProviderDescriptor;

#[test]
fn plugin_binding_provider_descriptor_requires_owner_namespace_and_bindings() {
    let descriptor = RhaiPluginBindingProviderDescriptor::new("amigo.camera.camera-core", "camera")
        .with_binding("set_focus_target");

    assert!(descriptor.is_valid());
}

#[test]
fn plugin_binding_provider_descriptor_rejects_empty_bindings() {
    let descriptor = RhaiPluginBindingProviderDescriptor::new("amigo.camera.camera-core", "camera");

    assert!(!descriptor.is_valid());
}
