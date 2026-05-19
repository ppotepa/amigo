#[test]
fn color_grading_plugin_owns_scene_descriptor() {
    let descriptor =
        amigo_color_grading_plugin::scene::color_grading_scene_descriptor();

    assert!(descriptor.is_valid());
    assert_eq!(
        descriptor.id.as_str(),
        "amigo.postfx.color-grading.ColorGrading"
    );
}

