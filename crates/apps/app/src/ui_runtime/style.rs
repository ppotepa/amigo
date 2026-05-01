fn resolve_ui_overlay_style(style: &RuntimeUiStyle) -> UiOverlayStyle {
    UiOverlayStyle {
        left: style.left,
        top: style.top,
        right: style.right,
        bottom: style.bottom,
        width: style.width,
        height: style.height,
        padding: style.padding,
        gap: style.gap,
        background: style.background,
        color: style.color,
        border_color: style.border_color,
        border_width: style.border_width,
        border_radius: style.border_radius,
        font_size: style.font_size,
        word_wrap: style.word_wrap,
        fit_to_width: style.fit_to_width,
        text_anchor: match style.align {
            RuntimeUiTextAlign::Start => UiTextAnchor::TopLeft,
            RuntimeUiTextAlign::Center => UiTextAnchor::Center,
        },
    }
}

fn resolve_ui_overlay_style_with_overrides(
    active_theme: Option<&UiTheme>,
    style_class: Option<&str>,
    style: &RuntimeUiStyle,
    path: &str,
    snapshot: &UiStateSnapshot,
) -> UiOverlayStyle {
    let mut merged = active_theme
        .and_then(|theme| style_class.and_then(|style_class| theme.classes.get(style_class)))
        .cloned()
        .unwrap_or_default();
    merge_ui_style(&mut merged, style);
    let mut style = resolve_ui_overlay_style(&merged);
    if let Some(color) = snapshot.color_overrides.get(path).copied() {
        style.color = Some(color);
    }
    if let Some(background) = snapshot.background_overrides.get(path).copied() {
        style.background = Some(background);
    }
    style
}

fn merge_ui_style(base: &mut RuntimeUiStyle, overlay: &RuntimeUiStyle) {
    base.left = overlay.left.or(base.left);
    base.top = overlay.top.or(base.top);
    base.right = overlay.right.or(base.right);
    base.bottom = overlay.bottom.or(base.bottom);
    base.width = overlay.width.or(base.width);
    base.height = overlay.height.or(base.height);
    if overlay.padding != 0.0 {
        base.padding = overlay.padding;
    }
    if overlay.gap != 0.0 {
        base.gap = overlay.gap;
    }
    base.background = overlay.background.or(base.background);
    base.color = overlay.color.or(base.color);
    base.border_color = overlay.border_color.or(base.border_color);
    if overlay.border_width != 0.0 {
        base.border_width = overlay.border_width;
    }
    if overlay.border_radius != 0.0 {
        base.border_radius = overlay.border_radius;
    }
    if overlay.font_size != RuntimeUiStyle::default().font_size {
        base.font_size = overlay.font_size;
    }
    base.word_wrap = overlay.word_wrap || base.word_wrap;
    base.fit_to_width = overlay.fit_to_width || base.fit_to_width;
    if overlay.align != RuntimeUiTextAlign::Start {
        base.align = overlay.align;
    }
}

