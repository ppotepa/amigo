#[test]
fn bloom_plugin_owns_scene_descriptor() {
    let descriptor = amigo_bloom_plugin::scene::bloom_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.postfx.bloom.Bloom");
}
