use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const CAPABILITY_ID: &str = "amigo.audio.editor";
const COMPONENT_TYPE: &str = "amigo.audio.emitter";

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
        InspectorSchema::placeholder(self.component_type(), "Audio")
            .with_field(PropertyDescriptor::asset("cue", "Cue", "audio"))
            .with_field(PropertyDescriptor::number("volume", "Volume"))
            .with_field(PropertyDescriptor::bool("loop", "Loop"))
            .with_field(PropertyDescriptor::text("bus", "Bus"))
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