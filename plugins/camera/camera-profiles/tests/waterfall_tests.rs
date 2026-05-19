#[test]
fn camera_profiles_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_camera_profiles_plugin::scene::camera_profile_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.camera.camera-profiles.CameraProfile"
    );
}
