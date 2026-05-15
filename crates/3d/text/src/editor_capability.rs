use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.3d.text.editor";
const COMPONENT_TYPE: &str = "amigo.3d.text";

#[derive(Debug, Clone, Copy)]
pub struct Text3dEditorCapability;

impl EditorCapability for Text3dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Text3D")
            .with_field(PropertyDescriptor::text("content", "Content"))
            .with_field(PropertyDescriptor::asset("font", "Font", "font"))
            .with_field(PropertyDescriptor::number("size", "Size"))
            .with_field(PropertyDescriptor::vec3("transform", "Transform"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Text3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.3d.text.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Text3dEditorCapability);
        Ok(())
    }
}

pub fn register_text3d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Text3dEditorCapabilityProvider.register(registry)
}
