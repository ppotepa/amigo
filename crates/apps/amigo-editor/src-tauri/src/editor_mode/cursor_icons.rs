use crate::editor_mode::dto::EditorCursorIconDto;

pub fn editor_cursor_svg(icon: EditorCursorIconDto, x: f32, y: f32) -> String {
    match icon {
        EditorCursorIconDto::Default | EditorCursorIconDto::Select => pointer_cursor(x, y),
        EditorCursorIconDto::Move => move_cursor(x, y),
        EditorCursorIconDto::MoveX => move_x_cursor(x, y),
        EditorCursorIconDto::MoveY => move_y_cursor(x, y),
        EditorCursorIconDto::Rotate => rotate_cursor(x, y),
        EditorCursorIconDto::Scale | EditorCursorIconDto::ScaleX | EditorCursorIconDto::ScaleY => {
            scale_cursor(x, y)
        }
        EditorCursorIconDto::Rect => rect_cursor(x, y),
        EditorCursorIconDto::Pan | EditorCursorIconDto::Grab => hand_cursor(x, y, false),
        EditorCursorIconDto::Grabbing => hand_cursor(x, y, true),
        EditorCursorIconDto::NotAllowed => not_allowed_cursor(x, y),
    }
}

fn pointer_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="#f8fafc" stroke="#020617" stroke-width="1.5">
<path d="M0 0 L0 22 L6 16 L10 25 L14 23 L10 14 L18 14 Z"/>
</g>"##,
        x, y
    )
}

fn move_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
<path d="M0 -12 L0 12 M-12 0 L12 0"/>
<path d="M0 -12 L-4 -8 M0 -12 L4 -8"/>
<path d="M0 12 L-4 8 M0 12 L4 8"/>
<path d="M-12 0 L-8 -4 M-12 0 L-8 4"/>
<path d="M12 0 L8 -4 M12 0 L8 4"/>
</g>"##,
        x, y
    )
}

fn move_x_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
<path d="M-14 0 L14 0"/>
<path d="M-14 0 L-9 -5 M-14 0 L-9 5"/>
<path d="M14 0 L9 -5 M14 0 L9 5"/>
</g>"##,
        x, y
    )
}

fn move_y_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
<path d="M0 -14 L0 14"/>
<path d="M0 -14 L-5 -9 M0 -14 L5 -9"/>
<path d="M0 14 L-5 9 M0 14 L5 9"/>
</g>"##,
        x, y
    )
}

fn rotate_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
<path d="M10 -7 A13 13 0 1 0 12 7"/>
<path d="M10 -7 L10 -15 L18 -10"/>
</g>"##,
        x, y
    )
}

fn scale_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
<path d="M-10 10 L10 -10"/>
<path d="M10 -10 L2 -10 M10 -10 L10 -2"/>
<path d="M-10 10 L-2 10 M-10 10 L-10 2"/>
</g>"##,
        x, y
    )
}

fn rect_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#facc15" stroke-width="2">
<rect x="-10" y="-7" width="20" height="14"/>
<rect x="-13" y="-10" width="6" height="6" fill="#facc15"/>
<rect x="7" y="-10" width="6" height="6" fill="#facc15"/>
<rect x="-13" y="4" width="6" height="6" fill="#facc15"/>
<rect x="7" y="4" width="6" height="6" fill="#facc15"/>
</g>"##,
        x, y
    )
}

fn hand_cursor(x: f32, y: f32, active: bool) -> String {
    let fill = if active { "#facc15" } else { "#f8fafc" };
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="{}" stroke="#020617" stroke-width="1.5" stroke-linejoin="round">
<path d="M-5 2 L-5 -10 Q-5 -13 -2 -13 Q1 -13 1 -10 L1 -2 L3 -4 Q5 -6 8 -4 Q10 -2 8 1 L3 11 Q1 15 -4 15 Q-9 15 -11 10 L-14 2 Q-15 -1 -12 -2 Q-9 -3 -8 0 Z"/>
</g>"##,
        x, y, fill
    )
}

fn not_allowed_cursor(x: f32, y: f32) -> String {
    format!(
        r##"<g transform="translate({:.2} {:.2})" fill="none" stroke="#ef4444" stroke-width="2">
<circle cx="0" cy="0" r="10"/>
<path d="M-7 -7 L7 7"/>
</g>"##,
        x, y
    )
}
