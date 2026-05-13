use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.2d.text.editor";
const COMPONENT_TYPE: &str = "amigo.2d.text";

#[derive(Debug, Clone, Copy)]
pub struct Text2dEditorCapability;

impl EditorCapability for Text2dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Text2D")
            .with_field(PropertyDescriptor::text("content", "Content"))
            .with_field(PropertyDescriptor::asset("font", "Font", "font"))
            .with_field(PropertyDescriptor::vec2("bounds", "Bounds"))
            .with_field(PropertyDescriptor::text("render_layer", "Render Layer"))
            .with_field(PropertyDescriptor::number("z_index", "Z Index"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dEditorCapabilityProvider;

impl EditorCapabilityProvider for Text2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.2d.text.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Text2dEditorCapability);
        Ok(())
    }
}

pub fn register_text2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Text2dEditorCapabilityProvider.register(registry)
}