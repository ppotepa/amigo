#[test]
fn scopes_plugin_owns_scene_descriptor() {
    let descriptor = amigo_scopes_plugin::scene::scopes_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.postfx.scopes.Scopes");
}
