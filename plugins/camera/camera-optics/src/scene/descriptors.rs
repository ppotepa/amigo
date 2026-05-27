use amigo_plugin_api::PluginSceneComponentDescriptor;

pub fn camera_optics_2d_scene_descriptor() -> PluginSceneComponentDescriptor {
    PluginSceneComponentDescriptor::new(
        "amigo.camera.camera-optics.CameraOptics2D",
        "camera",
        "CameraOptics2D",
    )
}
