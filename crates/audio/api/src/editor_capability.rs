use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const CAPABILITY_ID: &str = "amigo.audio.editor-capability";
const COMPONENT_TYPE: &str = "Audio";

#[derive(Debug, Clone, Copy)]
pub struct AudioEditorCapability;

impl EditorCapability for AudioEditorCapability {
    fn id(&self) -> &'static str {
        CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema {
            component_type: self.component_type(),
            title: "Audio".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "cue".to_string(),
                    label: "cue".to_string(),
                    editor: PropertyEditorKind::AssetPicker { asset_kind: "audio".to_string() },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "volume".to_string(),
                    label: "volume".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "loop".to_string(),
                    label: "loop".to_string(),
                    editor: PropertyEditorKind::Bool,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "bus".to_string(),
                    label: "bus".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
            ],
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AudioEditorCapabilityProvider;

impl EditorCapabilityProvider for AudioEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.audio.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(AudioEditorCapability);
        Ok(())
    }
}

pub fn register_audio_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    AudioEditorCapabilityProvider.register(registry)
}
