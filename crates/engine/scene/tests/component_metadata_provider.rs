use amigo_scene::{
    camera_2d_descriptor, ComponentMetadataProvider, ComponentRegistry, ComponentKind,
};

struct TestProvider;

impl ComponentMetadataProvider for TestProvider {
    fn provider_id(&self) -> &'static str {
        "test.component-metadata"
    }

    fn register_component_metadata(&self, registry: &mut ComponentRegistry) {
        registry.insert(camera_2d_descriptor());
    }
}

#[test]
fn provider_can_register_component_metadata() {
    let mut registry = ComponentRegistry::new([]);
    TestProvider.register_component_metadata(&mut registry);

    assert!(registry.descriptor(ComponentKind::Camera2D).is_some());
    assert_eq!(registry.iter().count(), 1);
}
