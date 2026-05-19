#[test]
fn lightmaps_2d_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_lightmaps_2d_plugin::scene::lightmap_2d_source_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.lighting.lightmaps-2d.LightMap2DSource"
    );
}

