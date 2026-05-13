use crate::{LightRoute2dCommand, RenderLayer2dCommand};

impl From<amigo_scene::RenderLayer2dSceneCommand> for RenderLayer2dCommand {
    fn from(value: amigo_scene::RenderLayer2dSceneCommand) -> Self {
        Self {
            source_mod: value.source_mod,
            id: value.id,
            label: value.label,
            order: value.order,
            visible: value.visible,
            opacity: value.opacity.clamp(0.0, 1.0),
        }
    }
}

impl From<amigo_scene::LightRoute2dSceneCommand> for LightRoute2dCommand {
    fn from(value: amigo_scene::LightRoute2dSceneCommand) -> Self {
        Self {
            source_mod: value.source_mod,
            receiver_layer: value.receiver_layer,
            groups: value.groups,
        }
    }
}

