use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::{LayeredImageBlendMode2d, LayeredImageSceneService};

pub struct LayeredImageScriptCommandContext<'a> {
    pub layered_image_scene_service: &'a LayeredImageSceneService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayeredImageScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_layered_image_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "2d.layered_image"
}

pub fn handle_layered_image_script_command(
    ctx: LayeredImageScriptCommandContext<'_>,
    command: ScriptCommand,
) -> LayeredImageScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("set_base_opacity", [entity_name, opacity]) => match opacity.parse::<f32>() {
            Ok(opacity) => {
                if !ctx
                    .layered_image_scene_service
                    .set_base_opacity(entity_name, opacity)
                {
                    return LayeredImageScriptCommandOutcome::Updated(format!(
                        "layered image `{entity_name}` not found"
                    ));
                }
                LayeredImageScriptCommandOutcome::Updated(format!(
                    "updated layered image `{entity_name}` base opacity"
                ))
            }
            Err(error) => LayeredImageScriptCommandOutcome::ParseError(format!(
                "invalid layered image base opacity `{opacity}`: {error}"
            )),
        },
        ("set_opacity", [entity_name, layer_id, opacity]) => match opacity.parse::<f32>() {
            Ok(opacity) => {
                if !ctx.layered_image_scene_service.set_layer_opacity(
                    entity_name,
                    layer_id,
                    opacity,
                ) {
                    return LayeredImageScriptCommandOutcome::Updated(format!(
                        "layered image `{entity_name}` layer `{layer_id}` not found"
                    ));
                }
                LayeredImageScriptCommandOutcome::Updated(format!(
                    "updated layered image `{entity_name}` layer `{layer_id}` opacity"
                ))
            }
            Err(error) => LayeredImageScriptCommandOutcome::ParseError(format!(
                "invalid layered image opacity `{opacity}`: {error}"
            )),
        },
        ("set_enabled", [entity_name, layer_id, enabled]) => match enabled.parse::<bool>() {
            Ok(enabled) => {
                if !ctx.layered_image_scene_service.set_layer_enabled(
                    entity_name,
                    layer_id,
                    enabled,
                ) {
                    return LayeredImageScriptCommandOutcome::Updated(format!(
                        "layered image `{entity_name}` layer `{layer_id}` not found"
                    ));
                }
                LayeredImageScriptCommandOutcome::Updated(format!(
                    "updated layered image `{entity_name}` layer `{layer_id}` enabled"
                ))
            }
            Err(error) => LayeredImageScriptCommandOutcome::ParseError(format!(
                "invalid layered image enabled `{enabled}`: {error}"
            )),
        },
        ("set_blend", [entity_name, layer_id, blend]) => {
            match LayeredImageBlendMode2d::parse_strict(blend) {
                Some(blend_mode) => {
                    if !ctx.layered_image_scene_service.set_layer_blend_mode(
                        entity_name,
                        layer_id,
                        blend_mode,
                    ) {
                        return LayeredImageScriptCommandOutcome::Updated(format!(
                            "layered image `{entity_name}` layer `{layer_id}` not found"
                        ));
                    }
                    LayeredImageScriptCommandOutcome::Updated(format!(
                        "updated layered image `{entity_name}` layer `{layer_id}` blend mode"
                    ))
                }
                None => LayeredImageScriptCommandOutcome::ParseError(format!(
                    "invalid layered image blend mode `{blend}`"
                )),
            }
        }
        _ => LayeredImageScriptCommandOutcome::Unhandled,
    }
}

pub struct LayeredImage2dScriptCommandHandler;

impl RuntimeScriptCommandHandler for LayeredImage2dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "2d.layered_image"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_layered_image_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let layered_image_scene_service = runtime.required::<LayeredImageSceneService>()?;
        let _ = handle_layered_image_script_command(
            LayeredImageScriptCommandContext {
                layered_image_scene_service: layered_image_scene_service.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
