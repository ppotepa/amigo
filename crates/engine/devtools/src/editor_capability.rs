use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.devtools.editor-capability";
const COMPONENT_TYPE: &str = "Devtools";

#[derive(Debug, Clone, Copy)]
pub struct DevtoolsEditorCapability;

impl EditorCapability for DevtoolsEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema {
            component_type: self.component_type(),
            title: "Devtools".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "enabled".to_string(),
                    label: "enabled".to_string(),
                    editor: PropertyEditorKind::Bool,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "overlay_scale".to_string(),
                    label: "overlay scale".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "overlay_corner".to_string(),
                    label: "overlay corner".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct DevtoolsEditorCapabilityProvider;

impl EditorCapabilityProvider for DevtoolsEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.devtools.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(DevtoolsEditorCapability);
        Ok(())
    }
}

pub fn register_devtools_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    DevtoolsEditorCapabilityProvider.register(registry)
}
