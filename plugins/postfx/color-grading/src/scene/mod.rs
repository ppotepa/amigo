use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn color_grading_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.postfx.color-grading.ColorGrading",
        "postfx",
        "ColorGrading",
    )
}
