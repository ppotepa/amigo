use super::super::super::*;
use super::super::AppScriptCommandContext;
use amigo_session::ScriptCommandHandler;

pub(super) struct RenderScriptCommandHandler;

impl<'a> ScriptCommandHandler<AppScriptCommandContext<'a>, ScriptCommand, ()>
    for RenderScriptCommandHandler
{
    fn name(&self) -> &'static str {
        "render"
    }

    fn can_handle(&self, command: &ScriptCommand) -> bool {
        matches!(
            command.namespace.as_str(),
            "2d.sprite"
                | "2d.text"
                | "2d.layered_image"
                | "2d.light"
                | "2d.light_group"
                | "2d.render_layer"
                | "3d.mesh"
                | "3d.material"
                | "3d.text"
        )
    }

    fn handle(&self, ctx: &AppScriptCommandContext<'a>, command: ScriptCommand) {
        match (
            command.namespace.as_str(),
            command.name.as_str(),
            command.arguments.as_slice(),
        ) {
            ("2d.sprite", "spawn", [source_mod, entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.sprite", "spawn", [entity_name, texture_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d sprite size") {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueSprite2d {
                        command: Sprite2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            AssetKey::new(texture_key.clone()),
                            size,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.text", "spawn", [source_mod, entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.text", "spawn", [entity_name, content, font_key, width, height]) => {
                match crate::app_helpers::parse_scene_vec2(width, height, "2d text bounds") {
                    Ok(bounds) => ctx.scene_command_queue.submit(SceneCommand::QueueText2d {
                        command: Text2dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            bounds,
                        ),
                    }),
                    Err(message) => ctx.dev_console_state.write_line(message),
                }
            }
            ("2d.layered_image", "set_base_opacity", [entity_name, opacity]) => {
                match opacity.parse::<f32>() {
                    Ok(opacity) => {
                        if !ctx
                            .layered_image_scene_service
                            .set_base_opacity(entity_name, opacity)
                        {
                            ctx.dev_console_state
                                .write_line(format!("layered image `{entity_name}` not found"));
                        }
                    }
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "invalid layered image base opacity `{opacity}`: {error}"
                    )),
                }
            }
            ("2d.layered_image", "set_opacity", [entity_name, layer_id, opacity]) => {
                match opacity.parse::<f32>() {
                    Ok(opacity) => {
                        if !ctx.layered_image_scene_service.set_layer_opacity(
                            entity_name,
                            layer_id,
                            opacity,
                        ) {
                            ctx.dev_console_state.write_line(format!(
                                "layered image `{entity_name}` layer `{layer_id}` not found"
                            ));
                        }
                    }
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "invalid layered image opacity `{opacity}`: {error}"
                    )),
                }
            }
            ("2d.layered_image", "set_enabled", [entity_name, layer_id, enabled]) => {
                match enabled.parse::<bool>() {
                    Ok(enabled) => {
                        if !ctx.layered_image_scene_service.set_layer_enabled(
                            entity_name,
                            layer_id,
                            enabled,
                        ) {
                            ctx.dev_console_state.write_line(format!(
                                "layered image `{entity_name}` layer `{layer_id}` not found"
                            ));
                        }
                    }
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "invalid layered image enabled `{enabled}`: {error}"
                    )),
                }
            }
            ("2d.layered_image", "set_blend", [entity_name, layer_id, blend]) => {
                match amigo_2d_layered_image::LayeredImageBlendMode2d::parse_strict(blend) {
                    Some(blend_mode) => {
                        if !ctx.layered_image_scene_service.set_layer_blend_mode(
                            entity_name,
                            layer_id,
                            blend_mode,
                        ) {
                            ctx.dev_console_state.write_line(format!(
                                "layered image `{entity_name}` layer `{layer_id}` not found"
                            ));
                        }
                    }
                    None => ctx
                        .dev_console_state
                        .write_line(format!("invalid layered image blend mode `{blend}`")),
                }
            }
            ("2d.light", "set_intensity", [id, intensity]) => match intensity.parse::<f32>() {
                Ok(intensity) => {
                    if !ctx
                        .global_light2d_scene_service
                        .set_intensity(id, intensity)
                    {
                        ctx.dev_console_state
                            .write_line(format!("global 2d light `{id}` not found"));
                    }
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "invalid global 2d light intensity `{intensity}`: {error}"
                )),
            },
            ("2d.light", "set_color", [id, color]) => match parse_color_rgba_hex(color) {
                Some(color) => {
                    if !ctx.global_light2d_scene_service.set_color(id, color) {
                        ctx.dev_console_state
                            .write_line(format!("global 2d light `{id}` not found"));
                    }
                }
                None => ctx
                    .dev_console_state
                    .write_line(format!("invalid global 2d light color `{color}`")),
            },
            ("2d.light_group", "set_intensity", [id, intensity]) => {
                match intensity.parse::<f32>() {
                    Ok(intensity) => {
                        if !ctx.light_group2d_scene_service.set_intensity(id, intensity) {
                            ctx.dev_console_state
                                .write_line(format!("2d light group `{id}` not found"));
                        }
                    }
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "invalid 2d light group intensity `{intensity}`: {error}"
                    )),
                }
            }
            ("2d.light_group", "set_color", [id, color]) => match parse_color_rgba_hex(color) {
                Some(color) => {
                    if !ctx.light_group2d_scene_service.set_color(id, color) {
                        ctx.dev_console_state
                            .write_line(format!("2d light group `{id}` not found"));
                    }
                }
                None => ctx
                    .dev_console_state
                    .write_line(format!("invalid 2d light group color `{color}`")),
            },
            ("2d.render_layer", "set_opacity", [id, opacity]) => match opacity.parse::<f32>() {
                Ok(opacity) => {
                    if !ctx.render_layer2d_scene_service.set_opacity(id, opacity) {
                        ctx.dev_console_state
                            .write_line(format!("2d render layer `{id}` not found"));
                    }
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "invalid 2d render layer opacity `{opacity}`: {error}"
                )),
            },
            ("2d.render_layer", "set_visible", [id, visible]) => match visible.parse::<bool>() {
                Ok(visible) => {
                    if !ctx.render_layer2d_scene_service.set_visible(id, visible) {
                        ctx.dev_console_state
                            .write_line(format!("2d render layer `{id}` not found"));
                    }
                }
                Err(error) => ctx.dev_console_state.write_line(format!(
                    "invalid 2d render layer visibility `{visible}`: {error}"
                )),
            },
            ("3d.mesh", "spawn", [source_mod, entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        source_mod.clone(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.mesh", "spawn", [entity_name, mesh_key]) => {
                ctx.scene_command_queue.submit(SceneCommand::QueueMesh3d {
                    command: Mesh3dSceneCommand::new(
                        ctx.launch_selection.selected_mod(),
                        entity_name.clone(),
                        AssetKey::new(mesh_key.clone()),
                    ),
                });
            }
            ("3d.material", "bind", [source_mod, entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.material", "bind", [entity_name, label, material_key]) => {
                ctx.scene_command_queue
                    .submit(SceneCommand::QueueMaterial3d {
                        command: Material3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            label.clone(),
                            Some(AssetKey::new(material_key.clone())),
                        ),
                    });
            }
            ("3d.text", "spawn", [source_mod, entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            source_mod.clone(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            ("3d.text", "spawn", [entity_name, content, font_key, size]) => {
                match size.parse::<f32>() {
                    Ok(size) => ctx.scene_command_queue.submit(SceneCommand::QueueText3d {
                        command: Text3dSceneCommand::new(
                            ctx.launch_selection.selected_mod(),
                            entity_name.clone(),
                            content.clone(),
                            AssetKey::new(font_key.clone()),
                            size,
                        ),
                    }),
                    Err(error) => ctx.dev_console_state.write_line(format!(
                        "failed to parse 3d text size `{size}` as f32: {error}"
                    )),
                }
            }
            _ => ctx.dev_console_state.write_line(format!(
                "{} could not handle command: {}",
                self.name(),
                crate::app_helpers::format_script_command(&command)
            )),
        }
    }
}

fn parse_color_rgba_hex(value: &str) -> Option<amigo_math::ColorRgba> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2])?,
            parse_hex_channel(&hex[2..4])?,
            parse_hex_channel(&hex[4..6])?,
            parse_hex_channel(&hex[6..8])?,
        ),
        _ => return None,
    };
    Some(amigo_math::ColorRgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn parse_hex_channel(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}



