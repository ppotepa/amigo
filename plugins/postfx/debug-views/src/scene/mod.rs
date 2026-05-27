use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn debug_views_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.postfx.debug-views.DebugViews",
        "postfx",
        "DebugViews",
    )
}
