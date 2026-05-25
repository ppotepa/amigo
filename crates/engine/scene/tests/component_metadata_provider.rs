use std::sync::Arc;

use amigo_scene::{
    ComponentMetadataProvider, ComponentRegistry, ComponentSchemaRegistry,
    RuntimeSceneCommandHandler, SceneCommand, SceneComponentSchemaProvider,
    ScenePluginCommandHandlerRegistry, camera_2d_descriptor,
};
use serde_yaml::{Mapping, Value};

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

    assert!(registry.descriptor("Camera2D").is_some());
    assert_eq!(registry.iter().count(), 1);
}

#[test]
fn duplicate_component_metadata_registration_is_explicit_error() {
    let mut registry = ComponentRegistry::new([camera_2d_descriptor()]);

    let error = registry.try_insert(camera_2d_descriptor()).unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate component metadata provider")
    );
    assert!(registry.descriptor("Camera2D").is_some());
    assert_eq!(registry.iter().count(), 1);
}

struct TestSchemaProvider;

impl SceneComponentSchemaProvider for TestSchemaProvider {
    fn component_type(&self) -> &'static str {
        "test.sprite"
    }

    fn aliases(&self) -> &'static [&'static str] {
        &["TestSprite"]
    }

    fn parse_yaml(&self, payload: Mapping) -> Result<Value, serde_yaml::Error> {
        Ok(Value::Mapping(payload))
    }
}

#[test]
fn duplicate_schema_provider_registration_is_explicit_error() {
    let registry = ComponentSchemaRegistry::default();
    registry
        .try_register_schema_provider(TestSchemaProvider)
        .unwrap();

    let error = registry
        .try_register_schema_provider(TestSchemaProvider)
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate component schema provider")
    );
    assert_eq!(
        registry
            .parse_plugin_payload_with_canonical_type("TestSprite", Mapping::new())
            .unwrap()
            .unwrap()
            .0,
        "test.sprite"
    );
}

struct TestCommandHandler;

impl RuntimeSceneCommandHandler for TestCommandHandler {
    fn can_handle(&self, _command: &SceneCommand) -> bool {
        false
    }

    fn handle(
        &self,
        _runtime: &amigo_runtime::Runtime,
        _command: SceneCommand,
    ) -> amigo_core::AmigoResult<()> {
        Ok(())
    }
}

#[test]
fn duplicate_scene_command_handler_registration_is_explicit_error() {
    let registry = ScenePluginCommandHandlerRegistry::default();
    registry
        .try_register("test.command", Arc::new(TestCommandHandler))
        .unwrap();

    let error = registry
        .try_register("test.command", Arc::new(TestCommandHandler))
        .unwrap_err();

    assert!(
        error
            .to_string()
            .contains("duplicate plugin scene command handler")
    );
    assert!(registry.handler_for("test.command").is_some());
}
