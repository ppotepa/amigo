use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.camera.editor-capability";
const COMPONENT_TYPE: &str = "Camera";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Camera".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "projection".to_string(),
                    label: "projection".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "viewport".to_string(),
                    label: "viewport".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "near".to_string(),
                    label: "near".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "far".to_string(),
                    label: "far".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
            ],
        }
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
