#[test]
fn camera_optics_plugin_owns_scene_descriptor() {
    let descriptor = amigo_camera_optics_plugin::scene::camera_optics_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.camera.camera-optics.CameraOptics2D"
    );
}
