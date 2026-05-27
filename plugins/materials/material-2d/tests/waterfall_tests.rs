#[test]
fn material_2d_plugin_owns_scene_descriptor() {
    let descriptor = amigo_material_2d_plugin::scene::material_2d_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.materials.material-2d.Material2D"
    );
}
