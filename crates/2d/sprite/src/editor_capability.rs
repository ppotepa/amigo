use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor,
};

const SPRITE_2D_CAPABILITY_ID: &str = "amigo.2d.sprite.editor";
const SPRITE_2D_COMPONENT_TYPE: &str = "amigo.2d.sprite";

#[derive(Debug, Clone, Copy)]
pub struct Sprite2dEditorCapability;

impl EditorCapability for Sprite2dEditorCapability {
    fn id(&self) -> &'static str {
        SPRITE_2D_CAPABILITY_ID
    }

    fn component_type(&self) -> ComponentTypeId {
        ComponentTypeId::new(SPRITE_2D_COMPONENT_TYPE)
    }

    fn inspector_schema(&self) -> InspectorSchema {
        InspectorSchema::placeholder(self.component_type(), "Sprite2D")
            .with_field(PropertyDescriptor::asset("image", "Texture", "image"))
            .with_field(PropertyDescriptor::vec2("size", "Size"))
            .with_field(PropertyDescriptor::text("render_layer", "Render Layer"))
            .with_field(PropertyDescriptor::number("z_index", "Z Index"))
            .with_field(PropertyDescriptor::read_only_text("animation", "Animation"))
            .with_field(PropertyDescriptor::read_only_text("sheet", "Sheet"))
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Sprite2dEditorCapabilityProvider;

impl EditorCapabilityProvider for Sprite2dEditorCapabilityProvider {
    fn id(&self) -> &'static str {
        "amigo.2d.sprite.editor-provider"
    }

    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
        registry.register_capability(Sprite2dEditorCapability);
        Ok(())
    }
}

pub fn register_sprite2d_editor_capabilities(registry: &EditorCapabilityRegistry) -> AmigoResult<()> {
    Sprite2dEditorCapabilityProvider.register(registry)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sprite_editor_capability_uses_sprite2d_component_type() {
        let capability = Sprite2dEditorCapability;
        assert_eq!(capability.component_type().as_str(), SPRITE_2D_COMPONENT_TYPE);
        assert_eq!(capability.inspector_schema().fields.len(), 6);
    }
}
