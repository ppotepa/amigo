use amigo_plugin_api::{camera_artifact_layer, scene_emissive, scene_highlight};
use amigo_render_wgpu::WgpuPluginPassDescriptor;

#[test]
fn plugin_pass_descriptor_declares_owner_and_targets() {
    let descriptor =
        WgpuPluginPassDescriptor::new("camera-optics.compose", "amigo.camera.camera-optics")
            .reads(scene_highlight())
            .reads(scene_emissive())
            .writes(camera_artifact_layer());

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.owner.0, "amigo.camera.camera-optics");
    assert_eq!(descriptor.reads.len(), 2);
    assert_eq!(descriptor.writes[0].0, "CameraArtifactLayer");
}
