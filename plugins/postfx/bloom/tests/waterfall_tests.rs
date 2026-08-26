#[test]
fn bloom_waterfall_declares_input_output_and_diagnostics_contracts() {
    let descriptor = amigo_bloom_plugin::scene::bloom_scene_descriptor();
    assert!(descriptor.is_valid());
    assert_eq!(descriptor.id.as_str(), "amigo.postfx.bloom.Bloom");

    let manifest = amigo_plugin_manifest::parse_plugin_manifest_str(include_str!("../plugin.toml"))
        .expect("bloom plugin manifest should parse");
    assert!(manifest.targets.reads.iter().any(|target| target.0 == "SceneEmissive"));
    assert!(manifest.targets.reads.iter().any(|target| target.0 == "SceneColor"));
    assert!(manifest.targets.writes.iter().any(|target| target.0 == "SceneColor"));
    assert!(manifest
        .diagnostics
        .channels
        .iter()
        .any(|channel| channel.id.0 == "postfx.bloom"));
    assert!(manifest.tests.waterfall.as_deref() == Some("tests/waterfall_tests.rs"));
}
