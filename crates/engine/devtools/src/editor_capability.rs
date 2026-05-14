use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.devtools.editor";
const COMPONENT_TYPE: &str = "amigo.devtools";

#[derive(Debug, Clone, Copy)]
pub struct DevtoolsEditorCapability;

impl EditorCapability for DevtoolsEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Devtools")
            .with_field(PropertyDescriptor::bool("enabled", "Enabled"))
            .with_field(PropertyDescriptor::number("overlay_scale", "Overlay Scale"))
            .with_field(PropertyDescriptor::text("overlay_corner", "Overlay Corner"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DevtoolsEditorCapabilityProvider;

impl EditorCapabilityProvider for DevtoolsEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.devtools.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(DevtoolsEditorCapability);
        Ok(())
    }
}

pub fn register_devtools_editor_capabilities(
    registry: &EditorCapabilityRegistry,
) -> AmigoResult<()> {
    DevtoolsEditorCapabilityProvider.register(registry)
}
