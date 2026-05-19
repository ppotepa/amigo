use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn camera_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.camera-core.Camera2D",
        "camera",
        "Camera2D",
    )
}

pub mod commands;

pub use commands::*;
