#[test]
fn composite_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_composite_plugin::scene::composite_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.postfx.composite.Composite");
}

