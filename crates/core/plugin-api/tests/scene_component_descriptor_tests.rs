use amigo_plugin_api::{
    PluginSceneComponentDescriptor, PluginSceneComponentId,
};

#[test]
fn plugin_scene_component_descriptor_validates_required_fields() {
    let descriptor = PluginSceneComponentDescriptor::new(
        "amigo.gfx.sprite-2d.Sprite2D",
        "gfx",
        "Sprite2D",
    );

    assert!(descriptor.is_valid());
}

#[test]
fn plugin_scene_component_descriptor_rejects_empty_id() {
    let descriptor =
        PluginSceneComponentDescriptor::new("", "gfx", "Sprite2D");

    assert!(!descriptor.is_valid());
}

#[test]
fn plugin_scene_component_id_exposes_stable_string() {
    let id = PluginSceneComponentId::new("amigo.camera.camera-core.Camera2D");

    assert_eq!(id.as_str(), "amigo.camera.camera-core.Camera2D");
    assert_eq!(id.to_string(), "amigo.camera.camera-core.Camera2D");
}

