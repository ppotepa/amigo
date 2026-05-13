use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.camera.editor";
const COMPONENT_TYPE: &str = "amigo.camera";

#[derive(Debug, Clone, Copy)]
pub struct CameraEditorCapability;

impl EditorCapability for CameraEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Camera")
            .with_field(PropertyDescriptor::text("projection", "Projection"))
            .with_field(PropertyDescriptor::text("viewport", "Viewport"))
            .with_field(PropertyDescriptor::number("near", "Near"))
            .with_field(PropertyDescriptor::number("far", "Far"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CameraEditorCapabilityProvider;

impl EditorCapabilityProvider for CameraEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.camera.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(CameraEditorCapability);
        Ok(())
    }
}

pub fn register_camera_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    CameraEditorCapabilityProvider.register(registry)
}