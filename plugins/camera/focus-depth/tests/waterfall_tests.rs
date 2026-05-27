#[test]
fn focus_depth_plugin_owns_scene_descriptor() {
    let descriptor = amigo_focus_depth_plugin::scene::focus_depth_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.camera.focus-depth.FocusDepth2D"
    );
}
