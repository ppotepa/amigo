use crate::{LightRoute2dCommand, RenderDepth2d, RenderDepthMode2d, RenderLayer2dCommand};

impl From<amigo_scene::RenderLayer2dSceneCommand> for RenderLayer2dCommand {
    fn from(value: amigo_scene::RenderLayer2dSceneCommand) -> Self {
        Self {
            source_mod: value.source_mod,
            id: value.id,
            label: value.label,
            order: value.order,
            visible: value.visible,
            opacity: value.opacity.clamp(0.0, 1.0),
            depth: RenderDepth2d {
                mode: match value.depth.mode {
                    amigo_scene::RenderDepthMode2dSceneCommand::DepthMap => {
                        RenderDepthMode2d::DepthMap
                    }
                    amigo_scene::RenderDepthMode2dSceneCommand::Plane => RenderDepthMode2d::Plane,
                    amigo_scene::RenderDepthMode2dSceneCommand::Overlay => {
                        RenderDepthMode2d::Overlay
                    }
                },
                value: value.depth.value,
                blur_scale: value.depth.blur_scale,
            }
            .normalized(),
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
