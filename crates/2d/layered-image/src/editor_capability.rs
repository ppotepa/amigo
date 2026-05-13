use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.layeredimage2d.editor-capability";
const COMPONENT_TYPE: &str = "LayeredImage2D";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Layered Image 2D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "image".to_string(),
                    label: "image".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "image".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "layers".to_string(),
                    label: "layers".to_string(),
                    editor: PropertyEditorKind::Text,
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
pub struct LayeredImage2dEditorCapabilityProvider;

impl EditorCapabilityProvider for LayeredImage2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.layeredimage2d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(LayeredImage2dEditorCapability);
        Ok(())
    }
}

pub fn register_layered_image2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    LayeredImage2dEditorCapabilityProvider.register(registry)
}
