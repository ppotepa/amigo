#[test]
fn trails_2d_plugin_owns_scene_descriptor() {
    let descriptor = amigo_trails_2d_plugin::scene::trail_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.vfx.trails-2d.Trail2D");
}
