use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.vector2d.editor-capability";
const COMPONENT_TYPE: &str = "Vector2D";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Vector 2D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "shape".to_string(),
                    label: "shape".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "color".to_string(),
                    label: "color".to_string(),
                    editor: PropertyEditorKind::Color,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "stroke_width".to_string(),
                    label: "stroke width".to_string(),
                    editor: PropertyEditorKind::Number,
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
pub struct Vector2dEditorCapabilityProvider;

impl EditorCapabilityProvider for Vector2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.vector2d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Vector2dEditorCapability);
        Ok(())
    }
}

pub fn register_vector2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Vector2dEditorCapabilityProvider.register(registry)
}
