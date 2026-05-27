#[test]
fn light_groups_2d_plugin_owns_scene_descriptor() {
    let descriptor = amigo_light_groups_2d_plugin::scene::light_group_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.lighting.light-groups-2d.LightGroup2D"
    );
}
