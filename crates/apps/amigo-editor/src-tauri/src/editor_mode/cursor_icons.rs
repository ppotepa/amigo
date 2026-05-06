use crate::editor_mode::dto::EditorCursorIconDto;

#[derive(Debug, Clone, Copy)]
struct CursorAsset {
    svg: &'static str,
    hotspot_x: f32,
    hotspot_y: f32,
    scale: f32,
}

pub fn editor_cursor_svg(icon: EditorCursorIconDto, x: f32, y: f32) -> String {
    let asset = cursor_asset(icon);
    let fill = fill_for_icon(icon);
    let outline = "#020617";
    let accent = accent_for_icon(icon);

    let body = asset
        .svg
        .replace("currentColor", fill)
        .replace("__FILL__", fill)
        .replace("__OUTLINE__", outline)
        .replace("__ACCENT__", accent);

    format!(
        r#"<g transform="translate({:.2} {:.2}) scale({:.3})">{}</g>"#,
        x - asset.hotspot_x * asset.scale,
        y - asset.hotspot_y * asset.scale,
        asset.scale,
        strip_svg_shell(&body),
    )
}

fn cursor_asset(icon: EditorCursorIconDto) -> CursorAsset {
    match icon {
        EditorCursorIconDto::Default | EditorCursorIconDto::Select => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/pointer.svg"),
            hotspot_x: 2.0,
            hotspot_y: 2.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Move => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/move.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::MoveX => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/move-x.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::MoveY => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/move-y.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Rotate => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/rotate.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Scale => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/scale.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::ScaleX => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/scale-x.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::ScaleY => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/scale-y.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Rect => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/rect.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Pan | EditorCursorIconDto::Grab => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/pan.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::Grabbing => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/grabbing.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
        EditorCursorIconDto::NotAllowed => CursorAsset {
            svg: include_str!("../../resources/editor-cursors/amigo-pro/not-allowed.svg"),
            hotspot_x: 12.0,
            hotspot_y: 12.0,
            scale: 1.0,
        },
    }
}

fn fill_for_icon(icon: EditorCursorIconDto) -> &'static str {
    match icon {
        EditorCursorIconDto::Default | EditorCursorIconDto::Select => "#f8fafc",
        EditorCursorIconDto::Grabbing => "#facc15",
        EditorCursorIconDto::NotAllowed => "#ef4444",
        _ => "#facc15",
    }
}

fn accent_for_icon(icon: EditorCursorIconDto) -> &'static str {
    match icon {
        EditorCursorIconDto::MoveX => "#ef4444",
        EditorCursorIconDto::MoveY => "#22c55e",
        EditorCursorIconDto::Rotate => "#a855f7",
        EditorCursorIconDto::NotAllowed => "#ef4444",
        _ => "#facc15",
    }
}

fn strip_svg_shell(svg: &str) -> String {
    let Some(start) = svg.find('>') else {
        return svg.to_owned();
    };
    let Some(end) = svg.rfind("</svg>") else {
        return svg.to_owned();
    };
    svg[start + 1..end].trim().to_owned()
}
