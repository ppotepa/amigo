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
                "distance" => RenderDepthMode2d::Distance,
                "z_depth" => RenderDepthMode2d::ZDepth,
                "infinity" => RenderDepthMode2d::Infinity,
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
        ("set_distance_m", [id, value]) => match value.parse::<f32>() {
            Ok(distance_m) => {
                if !ctx
                    .render_layer2d_scene_service
                    .set_distance_m_with_default_space(id, distance_m)
                {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` distance_m"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer distance_m `{value}`: {error}"
            )),
        },
        ("set_z_depth", [id, value]) => match value.parse::<f32>() {
            Ok(z_depth) => {
                if !ctx.render_layer2d_scene_service.set_z_depth(id, z_depth) {
                    return Composition2dScriptCommandOutcome::Updated(format!(
                        "2d render layer `{id}` not found"
                    ));
                }
                Composition2dScriptCommandOutcome::Updated(format!(
                    "updated 2d render layer `{id}` z_depth"
                ))
            }
            Err(error) => Composition2dScriptCommandOutcome::ParseError(format!(
                "invalid 2d render layer z_depth `{value}`: {error}"
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
        ("set_optical_role", [id, role]) => {
            let optical_role = match role.as_str() {
                "world_surface" => amigo_2d_spatial::OpticalLayerRole2d::WorldSurface,
                "scene_medium" => amigo_2d_spatial::OpticalLayerRole2d::SceneMedium,
                "foreground_medium" => amigo_2d_spatial::OpticalLayerRole2d::ForegroundMedium,
                "lens_surface" => amigo_2d_spatial::OpticalLayerRole2d::LensSurface,
                "overlay" => amigo_2d_spatial::OpticalLayerRole2d::Overlay,
                "debug" => amigo_2d_spatial::OpticalLayerRole2d::Debug,
                _ => {
                    return Composition2dScriptCommandOutcome::ParseError(format!(
                        "invalid 2d render layer optical role `{role}`"
                    ));
                }
            };
            if !ctx
                .render_layer2d_scene_service
                .set_optical_role(id, optical_role)
            {
                return Composition2dScriptCommandOutcome::Updated(format!(
                    "2d render layer `{id}` not found"
                ));
            }
            Composition2dScriptCommandOutcome::Updated(format!(
                "updated 2d render layer `{id}` optical role"
            ))
        }
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
