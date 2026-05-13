use amigo_core::AmigoResult;
use amigo_editor_api::{
    ComponentTypeId, EditorCapability, EditorCapabilityProvider, EditorCapabilityRegistry,
    InspectorSchema, PropertyDescriptor, PropertyEditorKind,
};

const SPRITE_2D_CAPABILITY_ID: &str = "amigo.2d.sprite.editor-capability";
const SPRITE_2D_COMPONENT_TYPE: &str = "Sprite2D";

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
        InspectorSchema {
            component_type: self.component_type(),
            title: "Sprite 2D".to_string(),
            fields: vec![
                PropertyDescriptor {
                    id: "texture".to_string(),
                    label: "Texture".to_string(),
                    editor: PropertyEditorKind::AssetPicker {
                        asset_kind: "image".to_string(),
                    },
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "size".to_string(),
                    label: "Size".to_string(),
                    editor: PropertyEditorKind::Vec2,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "render_layer".to_string(),
                    label: "Render Layer".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "z_index".to_string(),
                    label: "Z Index".to_string(),
                    editor: PropertyEditorKind::Number,
                    read_only: false,
                },
                PropertyDescriptor {
                    id: "animation".to_string(),
                    label: "Animation".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: true,
                },
                PropertyDescriptor {
                    id: "sheet".to_string(),
                    label: "Sheet".to_string(),
                    editor: PropertyEditorKind::Text,
                    read_only: true,
                },
            ],
        }
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

pub fn register_sprite2d_editor_capabilities(
    registry: &EditorCapabilityRegistry,
) -> AmigoResult<()> {
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

