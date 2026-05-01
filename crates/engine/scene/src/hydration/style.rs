use amigo_math::ColorRgba;

use crate::*;

pub(super) fn required_ui_text(
    node: &SceneUiNodeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    label: &str,
) -> SceneDocumentResult<String> {
    node.text
        .clone()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| SceneDocumentError::Hydration {
            scene_id: scene_id.to_owned(),
            entity_id: entity_id.to_owned(),
            component_kind: component_kind.to_owned(),
            message: format!("expected UI node to define non-empty `{label}` content"),
        })
}

pub(super) fn ui_style_from_component(
    style: &SceneUiStyleComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiStyle> {
    Ok(SceneUiStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: parse_optional_color_rgba_hex(
            style.background.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "background",
        )?,
        color: parse_optional_color_rgba_hex(
            style.color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "color",
        )?,
        border_color: parse_optional_color_rgba_hex(
            style.border_color.as_deref(),
            scene_id,
            entity_id,
            component_kind,
            "border_color",
        )?,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
        align: match style.align {
            Some(SceneUiTextAlignComponentDocument::Center) => SceneUiTextAlign::Center,
            Some(SceneUiTextAlignComponentDocument::Start) | None => SceneUiTextAlign::Start,
        },
    })
}

pub(super) fn ui_event_binding_from_component(
    binding: &SceneUiEventBindingComponentDocument,
) -> SceneUiEventBinding {
    SceneUiEventBinding {
        event: binding.event.clone(),
        payload: binding.payload.clone(),
    }
}

pub(super) fn ui_theme_from_component(
    theme: &SceneUiThemeComponentDocument,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<SceneUiTheme> {
    Ok(SceneUiTheme {
        id: theme.id.clone(),
        palette: SceneUiThemePalette {
            background: parse_color_rgba_hex(
                &theme.palette.background,
                scene_id,
                entity_id,
                component_kind,
            )?,
            surface: parse_color_rgba_hex(
                &theme.palette.surface,
                scene_id,
                entity_id,
                component_kind,
            )?,
            surface_alt: parse_color_rgba_hex(
                &theme.palette.surface_alt,
                scene_id,
                entity_id,
                component_kind,
            )?,
            text: parse_color_rgba_hex(&theme.palette.text, scene_id, entity_id, component_kind)?,
            text_muted: parse_color_rgba_hex(
                &theme.palette.text_muted,
                scene_id,
                entity_id,
                component_kind,
            )?,
            border: parse_color_rgba_hex(
                &theme.palette.border,
                scene_id,
                entity_id,
                component_kind,
            )?,
            accent: parse_color_rgba_hex(
                &theme.palette.accent,
                scene_id,
                entity_id,
                component_kind,
            )?,
            accent_text: parse_color_rgba_hex(
                &theme.palette.accent_text,
                scene_id,
                entity_id,
                component_kind,
            )?,
            danger: parse_color_rgba_hex(
                &theme.palette.danger,
                scene_id,
                entity_id,
                component_kind,
            )?,
            warning: parse_color_rgba_hex(
                &theme.palette.warning,
                scene_id,
                entity_id,
                component_kind,
            )?,
            success: parse_color_rgba_hex(
                &theme.palette.success,
                scene_id,
                entity_id,
                component_kind,
            )?,
        },
    })
}

pub(super) fn parse_optional_color_rgba_hex(
    value: Option<&str>,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
    _field_name: &str,
) -> SceneDocumentResult<Option<ColorRgba>> {
    value
        .map(|value| parse_color_rgba_hex(value, scene_id, entity_id, component_kind))
        .transpose()
}

pub(super) fn parse_color_rgba_hex(
    value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<ColorRgba> {
    let value = value.trim();
    let hex = value.strip_prefix('#').unwrap_or(value);

    let (r, g, b, a) = match hex.len() {
        6 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            255,
        ),
        8 => (
            parse_hex_channel(&hex[0..2], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[2..4], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[4..6], value, scene_id, entity_id, component_kind)?,
            parse_hex_channel(&hex[6..8], value, scene_id, entity_id, component_kind)?,
        ),
        _ => {
            return Err(SceneDocumentError::Hydration {
                scene_id: scene_id.to_owned(),
                entity_id: entity_id.to_owned(),
                component_kind: component_kind.to_owned(),
                message: format!(
                    "expected albedo color `{value}` to use #RRGGBB or #RRGGBBAA syntax"
                ),
            });
        }
    };

    Ok(ColorRgba::new(
        channel_to_f32(r),
        channel_to_f32(g),
        channel_to_f32(b),
        channel_to_f32(a),
    ))
}

pub(super) fn parse_hex_channel(
    channel: &str,
    raw_value: &str,
    scene_id: &str,
    entity_id: &str,
    component_kind: &str,
) -> SceneDocumentResult<u8> {
    u8::from_str_radix(channel, 16).map_err(|_| SceneDocumentError::Hydration {
        scene_id: scene_id.to_owned(),
        entity_id: entity_id.to_owned(),
        component_kind: component_kind.to_owned(),
        message: format!("expected albedo color `{raw_value}` to contain only hex digits"),
    })
}

pub(super) fn channel_to_f32(value: u8) -> f32 {
    f32::from(value) / 255.0
}
