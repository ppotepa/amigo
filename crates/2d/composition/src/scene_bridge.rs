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
                    amigo_scene::RenderDepthMode2dSceneCommand::Distance => {
                        RenderDepthMode2d::Distance
                    }
                    amigo_scene::RenderDepthMode2dSceneCommand::ZDepth => RenderDepthMode2d::ZDepth,
                    amigo_scene::RenderDepthMode2dSceneCommand::Infinity => {
                        RenderDepthMode2d::Infinity
                    }
                    amigo_scene::RenderDepthMode2dSceneCommand::Overlay => {
                        RenderDepthMode2d::Overlay
                    }
                },
                distance_m: value.depth.distance_m,
                z_depth: value.depth.z_depth,
                blur_scale: value.depth.blur_scale,
            }
            .normalized(),
            optical_role: value.optical_role.to_runtime(),
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
