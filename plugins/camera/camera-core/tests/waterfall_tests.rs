#[test]
fn camera_core_plugin_owns_scene_descriptor() {
    let descriptor = amigo_camera_core_plugin::scene::camera_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.camera.camera-core.Camera2D");
}
