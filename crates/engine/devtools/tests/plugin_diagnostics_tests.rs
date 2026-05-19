use amigo_devtools::PluginDiagnosticProviderDescriptor;

#[test]
fn plugin_diagnostic_provider_descriptor_requires_owner_and_channel() {
    let descriptor = PluginDiagnosticProviderDescriptor::new(
        "amigo.camera.camera-optics",
    )
    .with_channel("camera.optical.candidates");

    assert!(descriptor.is_valid());
}

#[test]
fn plugin_diagnostic_provider_descriptor_rejects_empty_channels() {
    let descriptor =
        PluginDiagnosticProviderDescriptor::new("amigo.camera.camera-optics");

    assert!(!descriptor.is_valid());
}

