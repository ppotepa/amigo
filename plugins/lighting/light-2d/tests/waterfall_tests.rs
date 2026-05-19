#[test]
fn light_2d_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_light_2d_plugin::scene::global_light_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.lighting.light-2d.GlobalLight2D"
    );
}
