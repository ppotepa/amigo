use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.2d.vector.editor";
const COMPONENT_TYPE: &str = "amigo.2d.vector";

#[derive(Debug, Clone, Copy)]
pub struct Vector2dEditorCapability;

impl EditorCapability for Vector2dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Vector2D")
            .with_field(PropertyDescriptor::text("shape", "Shape"))
            .with_field(PropertyDescriptor::color("color", "Color"))
            .with_field(PropertyDescriptor::number("stroke_width", "Stroke Width"))
            .with_field(PropertyDescriptor::text("render_layer", "Render Layer"))
            .with_field(PropertyDescriptor::number("z_index", "Z Index"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Vector2dEditorCapabilityProvider;

impl EditorCapabilityProvider for Vector2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.2d.vector.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Vector2dEditorCapability);
        Ok(())
    }
}

pub fn register_vector2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Vector2dEditorCapabilityProvider.register(registry)
}