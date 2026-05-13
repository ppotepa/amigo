use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.mesh3d.editor-capability";
const COMPONENT_TYPE: &str = "Mesh3D";

#[derive(Debug, Clone, Copy)]
pub struct Mesh3dEditorCapability;

impl EditorCapability for Mesh3dEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema {
            component_type: self.component_type(),
            title: "Mesh 3D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "mesh".to_string(),
                    label: "mesh".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "mesh".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "material".to_string(),
                    label: "material".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "material".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "transform".to_string(),
                    label: "transform".to_string(),
                    editor: PropertyEditorKind::Vec3,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "visible".to_string(),
                    label: "visible".to_string(),
                    editor: PropertyEditorKind::Bool,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mesh3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Mesh3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.mesh3d.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Mesh3dEditorCapability);
        Ok(())
    }
}

pub fn register_mesh3d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Mesh3dEditorCapabilityProvider.register(registry)
}
