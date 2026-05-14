use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.2d.layered_image.editor";
const COMPONENT_TYPE: &str = "amigo.2d.layered_image";

#[derive(Debug, Clone, Copy)]
pub struct LayeredImage2dEditorCapability;

impl EditorCapability for LayeredImage2dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "LayeredImage2D")
            .with_field(PropertyDescriptor::asset("image", "Image", "image"))
            .with_field(PropertyDescriptor::text("layers", "Layers"))
            .with_field(PropertyDescriptor::text("render_layer", "Render Layer"))
            .with_field(PropertyDescriptor::number("z_index", "Z Index"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LayeredImage2dEditorCapabilityProvider;

impl EditorCapabilityProvider for LayeredImage2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.2d.layered_image.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(LayeredImage2dEditorCapability);
        Ok(())
    }
}

pub fn register_layered_image2d_editor_capabilities(
    registry: &EditorCapabilityRegistry,
) -> AmigoResult<()> {
    LayeredImage2dEditorCapabilityProvider.register(registry)
}
