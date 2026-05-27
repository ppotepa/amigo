#[test]
fn layered_image_2d_waterfall_contract_exists() {
    assert_eq!(
        amigo_layered_image_2d_plugin::scene::layered_image_2d_scene_descriptor()
            .id
            .as_str(),
        "amigo.gfx.layered-image-2d.LayeredImage2D"
    );
}
