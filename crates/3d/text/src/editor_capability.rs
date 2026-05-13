use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.text3d.editor-capability";
const COMPONENT_TYPE: &str = "Text3D";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Text 3D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "content".to_string(),
                    label: "content".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "font".to_string(),
                    label: "font".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "font".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "size".to_string(),
                    label: "size".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "transform".to_string(),
                    label: "transform".to_string(),
                    editor: PropertyEditorKind::Vec3,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Text3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.text3d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Text3dEditorCapability);
        Ok(())
    }
}

pub fn register_text3d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Text3dEditorCapabilityProvider.register(registry)
}
