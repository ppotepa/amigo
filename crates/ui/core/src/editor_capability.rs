use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.ui.editor";
const COMPONENT_TYPE: &str = "amigo.ui";

#[derive(Debug, Clone, Copy)]
pub struct UiEditorCapability;

impl EditorCapability for UiEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "UI")
            .with_field(PropertyDescriptor::asset("document", "Document", "ui"))
            .with_field(PropertyDescriptor::asset("theme", "Theme", "theme"))
            .with_field(PropertyDescriptor::bool("visible", "Visible"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UiEditorCapabilityProvider;

impl EditorCapabilityProvider for UiEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.ui.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(UiEditorCapability);
        Ok(())
    }
}

pub fn register_ui_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    UiEditorCapabilityProvider.register(registry)
}
