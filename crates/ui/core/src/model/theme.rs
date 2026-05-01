#[derive(Debug, Clone, PartialEq)]
pub struct UiTheme {
    pub id: String,
    pub palette: UiThemePalette,
    pub classes: BTreeMap<String, UiStyle>,
}

impl UiTheme {
    pub fn from_palette(id: impl Into<String>, palette: UiThemePalette) -> Self {
        Self {
            id: id.into(),
            classes: default_theme_classes(&palette),
            palette,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct UiThemePalette {
    pub background: ColorRgba,
    pub surface: ColorRgba,
    pub surface_alt: ColorRgba,
    pub text: ColorRgba,
    pub text_muted: ColorRgba,
    pub border: ColorRgba,
    pub accent: ColorRgba,
    pub accent_text: ColorRgba,
    pub danger: ColorRgba,
    pub warning: ColorRgba,
    pub success: ColorRgba,
}

fn default_theme_classes(palette: &UiThemePalette) -> BTreeMap<String, UiStyle> {
    let mut classes = BTreeMap::new();
    classes.insert(
        "root".to_owned(),
        UiStyle {
            background: Some(palette.background),
            color: Some(palette.text),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "top_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            padding: 16.0,
            gap: 12.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "bottom_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text_muted),
            border_color: Some(palette.border),
            border_width: 1.0,
            padding: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "tab_bar".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 6.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "tab".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "tab_active".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "tab_disabled".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "panel".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 16.0,
            gap: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "group_box".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_section".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            padding: 10.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_row".to_owned(),
        UiStyle {
            color: Some(palette.text),
            gap: 8.0,
            height: Some(28.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_label".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "inspector_value".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "panel_alt".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 10.0,
            padding: 14.0,
            gap: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "card".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            padding: 12.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "card_selected".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.accent),
            border_width: 2.0,
            border_radius: 8.0,
            padding: 12.0,
            gap: 6.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_title".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 28.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_body".to_owned(),
        UiStyle {
            color: Some(palette.text),
            font_size: 16.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "text_muted".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "button".to_owned(),
        button_style(palette.surface_alt, palette.text, palette.border),
    );
    classes.insert(
        "button_primary".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "button_secondary".to_owned(),
        button_style(palette.surface_alt, palette.text, palette.border),
    );
    classes.insert(
        "button_danger".to_owned(),
        button_style(palette.danger, palette.accent_text, palette.danger),
    );
    classes.insert(
        "button_selected".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "button_disabled".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "progress".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(18.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(24.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_track".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            height: Some(8.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_fill".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_radius: 999.0,
            height: Some(8.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "slider_thumb".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_color: Some(palette.accent_text),
            border_width: 1.0,
            border_radius: 999.0,
            width: Some(14.0),
            height: Some(22.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toggle".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            padding: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toggle_on".to_owned(),
        button_style(palette.success, palette.accent_text, palette.success),
    );
    classes.insert(
        "toggle_off".to_owned(),
        button_style(palette.surface_alt, palette.text_muted, palette.border),
    );
    classes.insert(
        "dropdown".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            height: Some(38.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "option_set".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            color: Some(palette.accent),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 8.0,
            height: Some(38.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "option_selected".to_owned(),
        button_style(palette.accent, palette.accent_text, palette.accent),
    );
    classes.insert(
        "gradient_strip".to_owned(),
        UiStyle {
            background: Some(palette.surface_alt),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 6.0,
            height: Some(24.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "gradient_stop".to_owned(),
        UiStyle {
            background: Some(palette.text_muted),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 999.0,
            width: Some(12.0),
            height: Some(28.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "gradient_stop_selected".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            border_color: Some(palette.accent_text),
            border_width: 2.0,
            border_radius: 999.0,
            width: Some(14.0),
            height: Some(30.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "preview_panel".to_owned(),
        UiStyle {
            background: Some(palette.surface),
            color: Some(palette.text),
            border_color: Some(palette.border),
            border_width: 1.0,
            border_radius: 12.0,
            padding: 16.0,
            gap: 10.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "preview_viewport".to_owned(),
        UiStyle {
            background: Some(palette.background),
            border_color: Some(palette.accent),
            border_width: 1.0,
            border_radius: 8.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "toast".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            color: Some(palette.accent_text),
            border_color: Some(palette.accent),
            border_width: 1.0,
            border_radius: 999.0,
            padding: 8.0,
            font_size: 14.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "badge".to_owned(),
        UiStyle {
            background: Some(palette.accent),
            color: Some(palette.accent_text),
            border_radius: 999.0,
            padding: 6.0,
            font_size: 13.0,
            ..UiStyle::default()
        },
    );
    classes.insert(
        "divider".to_owned(),
        UiStyle {
            background: Some(palette.border),
            height: Some(1.0),
            ..UiStyle::default()
        },
    );
    classes.insert(
        "debug_text".to_owned(),
        UiStyle {
            color: Some(palette.text_muted),
            font_size: 13.0,
            ..UiStyle::default()
        },
    );
    classes
}

fn button_style(background: ColorRgba, color: ColorRgba, border: ColorRgba) -> UiStyle {
    UiStyle {
        background: Some(background),
        color: Some(color),
        border_color: Some(border),
        border_width: 1.0,
        border_radius: 8.0,
        padding: 10.0,
        font_size: 16.0,
        ..UiStyle::default()
    }
}
