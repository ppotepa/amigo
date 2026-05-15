use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.3d.material.editor";
const COMPONENT_TYPE: &str = "amigo.3d.material";

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
        InspectorSchema::placeholder(self.component_type(), "Material3D")
            .with_field(PropertyDescriptor::asset(
                "material", "Material", "material",
            ))
            .with_field(PropertyDescriptor::color("base_color", "Base Color"))
            .with_field(PropertyDescriptor::number("metallic", "Metallic"))
            .with_field(PropertyDescriptor::number("roughness", "Roughness"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Material3dEditorCapabilityProvider;

impl EditorCapabilityProvider for Material3dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.3d.material.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Material3dEditorCapability);
        Ok(())
    }
}

pub fn register_material3d_editor_capabilities(
    registry: &EditorCapabilityRegistry,
) -> AmigoResult<()> {
    Material3dEditorCapabilityProvider.register(registry)
}
