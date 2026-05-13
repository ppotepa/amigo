use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.text2d.editor-capability";
const COMPONENT_TYPE: &str = "Text2D";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Text 2D".to_string(),
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
                    id: "bounds".to_string(),
                    label: "bounds".to_string(),
                    editor: PropertyEditorKind::Vec2,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "render_layer".to_string(),
                    label: "render layer".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "z_index".to_string(),
                    label: "z index".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Text2dEditorCapabilityProvider;

impl EditorCapabilityProvider for Text2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.text2d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Text2dEditorCapability);
        Ok(())
    }
}

pub fn register_text2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Text2dEditorCapabilityProvider.register(registry)
}
