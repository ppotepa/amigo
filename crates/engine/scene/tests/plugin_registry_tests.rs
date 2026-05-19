use amigo_scene::{
    ScenePluginComponentDescriptor, ScenePluginComponentId,
    ScenePluginDescriptorProvider, ScenePluginDescriptorRegistry,
};

#[test]
fn plugin_component_descriptor_validates_required_fields() {
    let descriptor = ScenePluginComponentDescriptor::new(
        "amigo.gfx.sprite-2d.Sprite2D",
        "gfx",
        "Sprite2D",
    );

    assert!(descriptor.is_valid());
}

#[test]
fn plugin_component_descriptor_rejects_empty_fields() {
    let descriptor = ScenePluginComponentDescriptor::new("", "gfx", "Sprite2D");

    assert!(!descriptor.is_valid());
}

#[test]
fn plugin_descriptor_registry_indexes_by_component_id() {
    let mut registry = ScenePluginDescriptorRegistry::new();
    let id = ScenePluginComponentId::new("amigo.gfx.sprite-2d.Sprite2D");

    registry.insert(ScenePluginComponentDescriptor::new(
        id.as_str(),
        "gfx",
        "Sprite2D",
    ));

    assert_eq!(registry.len(), 1);
    assert_eq!(registry.get(&id).unwrap().label, "Sprite2D");
    assert!(registry.invalid_descriptors().is_empty());
}

struct TestDescriptorProvider;

impl ScenePluginDescriptorProvider for TestDescriptorProvider {
    fn register_scene_descriptors(
        &self,
        registry: &mut ScenePluginDescriptorRegistry,
    ) {
        registry.insert(ScenePluginComponentDescriptor::new(
            "amigo.gfx.text-2d.Text2D",
            "gfx",
            "Text2D",
        ));
    }
}

#[test]
fn plugin_descriptor_registry_accepts_provider_registration() {
    let mut registry = ScenePluginDescriptorRegistry::new();

    registry.register_provider(&TestDescriptorProvider);

    assert!(registry
        .get(&ScenePluginComponentId::new("amigo.gfx.text-2d.Text2D"))
        .is_some());
}
