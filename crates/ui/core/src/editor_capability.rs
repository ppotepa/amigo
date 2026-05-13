use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.ui.editor-capability";
const COMPONENT_TYPE: &str = "UI";

#[derive(Debug, Clone, Copy)]
pub struct UiEditorCapability;

impl EditorCapability for UiEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema {
            component_type: self.component_type(),
            title: "UI".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "document".to_string(),
                    label: "document".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "ui".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "theme".to_string(),
                    label: "theme".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "theme".to_string() },
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
pub struct UiEditorCapabilityProvider;

impl EditorCapabilityProvider for UiEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.ui.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(UiEditorCapability);
        Ok(())
    }
}

pub fn register_ui_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    UiEditorCapabilityProvider.register(registry)
}
