use super::super::*;
use crate::runtime_context::RuntimeContext;
use amigo_math::ColorRgba;

pub(crate) fn tick_ui_bindings(runtime: &Runtime) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let ui_scene = ctx.required::<UiSceneService>()?;
    let ui_state = ctx.required::<UiStateService>()?;
    let ui_model_bindings = ctx.required::<UiModelBindingService>()?;
    let scene_state = ctx.required::<amigo_state::SceneStateService>()?;

    for command in ui_scene.commands() {
        let root_segment = command
            .document
            .root
            .id
            .clone()
            .unwrap_or_else(|| "root".to_owned());
        let root_path = format!("{}.{}", command.entity_name, root_segment);
        apply_node_binds(
            &command.document.root,
            &root_path,
            ui_state.as_ref(),
            scene_state.as_ref(),
        );
    }

    for binding in ui_model_bindings.bindings() {
        apply_model_binding(&binding, ui_state.as_ref(), scene_state.as_ref());
    }

    Ok(())
}

fn apply_model_binding(
    binding: &UiModelBinding,
    ui_state: &UiStateService,
    scene_state: &amigo_state::SceneStateService,
) {
    match binding.kind {
        UiModelBindingKind::Text => {
            if let Some(value) = scene_state_value_as_text(scene_state, &binding.state_key) {
                let _ =
                    ui_state.set_text(&binding.path, format_text_value(&value, &binding.format));
            }
        }
        UiModelBindingKind::Value => {
            if let Some(value) = scene_state_value_as_f32(scene_state, &binding.state_key) {
                let _ = ui_state.set_value(&binding.path, value);
            }
        }
        UiModelBindingKind::Visible => {
            if let Some(value) = scene_state.get_bool(&binding.state_key) {
                if value {
                    let _ = ui_state.show(&binding.path);
                } else {
                    let _ = ui_state.hide(&binding.path);
                }
            }
        }
        UiModelBindingKind::Enabled => {
            if let Some(value) = scene_state.get_bool(&binding.state_key) {
                if value {
                    let _ = ui_state.enable(&binding.path);
                } else {
                    let _ = ui_state.disable(&binding.path);
                }
            }
        }
        UiModelBindingKind::Selected => {
            if let Some(value) = scene_state.get_string(&binding.state_key) {
                let _ = ui_state.set_selected(&binding.path, value);
            }
        }
        UiModelBindingKind::Color => {
            if let Some(value) = scene_state.get_string(&binding.state_key) {
                if let Some(color) = parse_color_rgba_hex(&value) {
                    let _ = ui_state.set_color(&binding.path, color);
                }
            }
        }
        UiModelBindingKind::Background => {
            if let Some(value) = scene_state.get_string(&binding.state_key) {
                if let Some(color) = parse_color_rgba_hex(&value) {
                    let _ = ui_state.set_background(&binding.path, color);
                }
            }
        }
    }
}

fn apply_node_binds(
    node: &RuntimeUiNode,
    path: &str,
    ui_state: &UiStateService,
    scene_state: &amigo_state::SceneStateService,
) {
    if let Some(key) = node.binds.text.as_deref() {
        if let Some(value) = scene_state_value_as_text(scene_state, key) {
            let _ = ui_state.set_text(path, value);
        }
    }
    if let Some(key) = node.binds.value.as_deref() {
        if let Some(value) = scene_state_value_as_f32(scene_state, key) {
            let _ = ui_state.set_value(path, value);
        }
    }
    if let Some(key) = node.binds.visible.as_deref() {
        if let Some(value) = scene_state.get_bool(key) {
            if value {
                let _ = ui_state.show(path);
            } else {
                let _ = ui_state.hide(path);
            }
        }
    }
    if let Some(key) = node.binds.enabled.as_deref() {
        if let Some(value) = scene_state.get_bool(key) {
            if value {
                let _ = ui_state.enable(path);
            } else {
                let _ = ui_state.disable(path);
            }
        }
    }

    for (index, child) in node.children.iter().enumerate() {
        let segment = child
            .id
            .clone()
            .unwrap_or_else(|| format!("{}-{index}", child.kind.label()));
        let child_path = format!("{path}.{segment}");
        apply_node_binds(child, &child_path, ui_state, scene_state);
    }
}

fn scene_state_value_as_text(
    scene_state: &amigo_state::SceneStateService,
    key: &str,
) -> Option<String> {
    if let Some(value) = scene_state.get_string(key) {
        return Some(value);
    }
    if let Some(value) = scene_state.get_int(key) {
        return Some(value.to_string());
    }
    if let Some(value) = scene_state.get_float(key) {
        return Some(format_float(value));
    }
    scene_state.get_bool(key).map(|value| value.to_string())
}

fn scene_state_value_as_f32(
    scene_state: &amigo_state::SceneStateService,
    key: &str,
) -> Option<f32> {
    if let Some(value) = scene_state.get_float(key) {
        return Some(value as f32);
    }
    if let Some(value) = scene_state.get_int(key) {
        return Some(value as f32);
    }
    scene_state
        .get_bool(key)
        .map(|value| if value { 1.0 } else { 0.0 })
}

fn format_float(value: f64) -> String {
    let text = value.to_string();
    text.strip_suffix(".0").unwrap_or(&text).to_owned()
}

fn format_text_value(value: &str, format: &Option<String>) -> String {
    format
        .as_ref()
        .map(|format| format.replace("{value}", value))
        .unwrap_or_else(|| value.to_owned())
}

fn parse_color_rgba_hex(value: &str) -> Option<ColorRgba> {
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
    Some(ColorRgba::new(
        r as f32 / 255.0,
        g as f32 / 255.0,
        b as f32 / 255.0,
        a as f32 / 255.0,
    ))
}

fn parse_hex_channel(value: &str) -> Option<u8> {
    u8::from_str_radix(value, 16).ok()
}
