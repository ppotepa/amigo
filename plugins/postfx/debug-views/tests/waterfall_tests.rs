#[test]
fn debug_views_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_debug_views_plugin::scene::debug_views_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.postfx.debug-views.DebugViews"
    );
}

