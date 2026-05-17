use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{RuntimeScriptCommandHandler, ScriptCommand};

use crate::{RenderDepthMode2d, RenderLayer2dSceneService};

pub struct Composition2dScriptCommandContext<'a> {
    pub render_layer2d_scene_service: &'a RenderLayer2dSceneService,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Composition2dScriptCommandOutcome {
    Updated(String),
    ParseError(String),
    Unhandled,
}

pub fn can_handle_composition2d_script_command(command: &ScriptCommand) -> bool {
    command.namespace == "2d.render_layer"
}

pub fn handle_composition2d_script_command(
    ctx: Composition2dScriptCommandContext<'_>,
    command: ScriptCommand,
) -> Composition2dScriptCommandOutcome {
    match (command.name.as_str(), command.arguments.as_slice()) {
        ("set_opacity", [id, opacity]) => match opacity.parse::<f32>() {
            Ok(opacity) => {
                if !ctx.render_layer2d_scene_service.set_opacity(id, opacity) {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` opacity"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer opacity `{opacity}`: {error}"
            )),
        },
        ("set_visible", [id, visible]) => match visible.parse::<bool>() {
            Ok(visible) => {
                if !ctx.render_layer2d_scene_service.set_visible(id, visible) {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` visibility"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer visibility `{visible}`: {error}"
            )),
        },
        ("set_depth_mode", [id, mode]) => {
            let mode = match mode.as_str() {
                "depth_map" => RenderDepthMode2d::DepthMap,
                "plane" => RenderDepthMode2d::Plane,
                "overlay" => RenderDepthMode2d::Overlay,
                _ => {
                    return Composition2dScriptCommandOutcome::ParseError(format!(
                        "invalid 2d render layer depth mode `{mode}`"
                    ));
                }
            };
            if !ctx.render_layer2d_scene_service.set_depth_mode(id, mode) {
                return Composition2dScriptCommandOutcome::Updated(format!(
                    "2d render layer `{id}` not found"
                ));
            }
            Composition2dScriptCommandOutcome::Updated(format!(
                "updated 2d render layer `{id}` depth mode"
            ))
        }
        ("set_depth_value", [id, value]) => match value.parse::<f32>() {
            Ok(value) => {
                if !ctx
                    .render_layer2d_scene_service
                    .set_depth_plane_value(id, value)
                {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` depth value"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer depth value `{value}`: {error}"
            )),
        },
        ("set_depth_blur_scale", [id, blur_scale]) => match blur_scale.parse::<f32>() {
            Ok(blur_scale) => {
                if !ctx
                    .render_layer2d_scene_service
                    .set_depth_blur_scale(id, blur_scale)
                {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` depth blur scale"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer depth blur scale `{blur_scale}`: {error}"
            )),
        },
        _ => Composition2dScriptCommandOutcome::Unhandled,
    }
}

pub struct RenderLayer2dScriptCommandHandler;

impl RuntimeScriptCommandHandler for RenderLayer2dScriptCommandHandler {
    fn name(&self) -> &'static str {
        "2d.render_layer"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        can_handle_composition2d_script_command(command)
    }

    fn handle(&self, runtime: &Runtime, command: ScriptCommand) -> AmigoResult<()> {
        let render_layer2d_scene_service = runtime.required::<RenderLayer2dSceneService>()?;
        let _ = handle_composition2d_script_command(
            Composition2dScriptCommandContext {
                render_layer2d_scene_service: render_layer2d_scene_service.as_ref(),
            },
            command,
        );
        Ok(())
    }
}
