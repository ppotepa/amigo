use crate::LayeredImageSceneService;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayeredImageDevConsoleCommandOutcome {
    Handled(String),
    Error(String),
    Unhandled,
}

pub struct LayeredImageDevConsoleCommandContext<'a> {
    pub layered_image_scene_service: &'a LayeredImageSceneService,
}

pub fn handle_layered_image_dev_console_command(
    ctx: LayeredImageDevConsoleCommandContext<'_>,
    name: &str,
    args: &[String],
) -> LayeredImageDevConsoleCommandOutcome {
    match name {
        "layered.list" => {
            let names = ctx.layered_image_scene_service.entity_names();
            LayeredImageDevConsoleCommandOutcome::Handled(if names.is_empty() {
                "layered images: none".to_owned()
            } else {
                format!("layered images:\n{}", names.join("\n"))
            })
        }
        "layered.info" => {
            let Some(entity) = args.first() else {
                return LayeredImageDevConsoleCommandOutcome::Error(
                    "usage: layered.info <entity>".to_owned(),
                );
            };
            let Some(command) = ctx
                .layered_image_scene_service
                .commands()
                .into_iter()
                .find(|command| command.entity_name == *entity)
            else {
                return LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "unknown layered image `{entity}`"
                ));
            };
            let overrides = command
                .image
                .layer_overrides
                .iter()
                .map(|override_| {
                    format!(
                        "{} opacity={:?} enabled={:?} blend={:?}",
                        override_.id, override_.opacity, override_.enabled, override_.blend_mode
                    )
                })
                .collect::<Vec<_>>();
            LayeredImageDevConsoleCommandOutcome::Handled(format!(
                "{} asset={} base_opacity={} render_layer={} z={} overrides={}",
                command.entity_name,
                command.image.asset.as_str(),
                command.image.base_opacity,
                command.render_layer,
                command.z_index,
                if overrides.is_empty() {
                    "none".to_owned()
                } else {
                    overrides.join("; ")
                }
            ))
        }
        "layered.base" => {
            let [entity, value] = args else {
                return LayeredImageDevConsoleCommandOutcome::Error(
                    "usage: layered.base <entity> <opacity>".to_owned(),
                );
            };
            let Ok(opacity) = value.parse::<f32>() else {
                return LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "invalid opacity `{value}`"
                ));
            };
            if ctx
                .layered_image_scene_service
                .set_base_opacity(entity, opacity)
            {
                LayeredImageDevConsoleCommandOutcome::Handled(format!(
                    "layered image `{entity}` base opacity={opacity}"
                ))
            } else {
                LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "unknown layered image `{entity}`"
                ))
            }
        }
        "layered.opacity" => {
            let [entity, layer, value] = args else {
                return LayeredImageDevConsoleCommandOutcome::Error(
                    "usage: layered.opacity <entity> <layer-id> <value>".to_owned(),
                );
            };
            let Ok(opacity) = value.parse::<f32>() else {
                return LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "invalid opacity `{value}`"
                ));
            };
            if ctx
                .layered_image_scene_service
                .set_layer_opacity(entity, layer, opacity)
            {
                LayeredImageDevConsoleCommandOutcome::Handled(format!(
                    "layered image `{entity}` layer `{layer}` opacity={opacity}"
                ))
            } else {
                LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "layered image `{entity}` layer `{layer}` not found"
                ))
            }
        }
        "layered.enable" => {
            let [entity, layer, value] = args else {
                return LayeredImageDevConsoleCommandOutcome::Error(
                    "usage: layered.enable <entity> <layer-id> true|false".to_owned(),
                );
            };
            let Ok(enabled) = value.parse::<bool>() else {
                return LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "invalid enabled value `{value}`"
                ));
            };
            if ctx
                .layered_image_scene_service
                .set_layer_enabled(entity, layer, enabled)
            {
                LayeredImageDevConsoleCommandOutcome::Handled(format!(
                    "layered image `{entity}` layer `{layer}` enabled={enabled}"
                ))
            } else {
                LayeredImageDevConsoleCommandOutcome::Error(format!(
                    "layered image `{entity}` layer `{layer}` not found"
                ))
            }
        }
        _ => LayeredImageDevConsoleCommandOutcome::Unhandled,
    }
}
