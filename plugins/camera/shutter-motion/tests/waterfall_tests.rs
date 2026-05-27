#[test]
fn shutter_motion_plugin_owns_scene_descriptor() {
    let descriptor = amigo_shutter_motion_plugin::scene::shutter_motion_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.camera.shutter-motion.ShutterMotion2D"
    );
}
