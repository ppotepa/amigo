use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.3d.mesh.editor";
const COMPONENT_TYPE: &str = "amigo.3d.mesh";

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
        InspectorSchema::placeholder(self.component_type(), "Mesh3D")
            .with_field(PropertyDescriptor::asset("mesh", "Mesh", "mesh"))
            .with_field(PropertyDescriptor::asset("material", "Material", "material"))
            .with_field(PropertyDescriptor::vec3("transform", "Transform"))
            .with_field(PropertyDescriptor::bool("visible", "Visible"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Mesh3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Mesh3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.3d.mesh.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Mesh3dEditorCapability);
        Ok(())
    }
}

pub fn register_mesh3d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Mesh3dEditorCapabilityProvider.register(registry)
}