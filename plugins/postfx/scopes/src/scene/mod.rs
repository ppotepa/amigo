use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn scopes_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.postfx.scopes.Scopes", "postfx", "Scopes")
}
