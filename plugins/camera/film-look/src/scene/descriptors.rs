use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn film_look_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new("amigo.camera.film-look.FilmLook", "camera", "FilmLook")
}
