#[test]
fn beacon_light_2d_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_beacon_light_2d_plugin::scene::beacon_light_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.lighting.beacon-light-2d.BeaconLight2D"
    );
}
