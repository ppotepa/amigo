use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.material3d.editor-capability";
const COMPONENT_TYPE: &str = "Material3D";

#[derive(Debug, Clone, Copy)]
pub struct Material3dEditorCapability;

impl EditorCapability for Material3dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema {
            component_type: self.component_type(),
            title: "Material 3D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "material".to_string(),
                    label: "material".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "material".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "base_color".to_string(),
                    label: "base color".to_string(),
                    editor: PropertyEditorKind::Color,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "metallic".to_string(),
                    label: "metallic".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "roughness".to_string(),
                    label: "roughness".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Material3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.material3d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Material3dEditorCapability);
        Ok(())
    }
}

pub fn register_material3d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Material3dEditorCapabilityProvider.register(registry)
}
