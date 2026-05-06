use crate::editor_mode::dto::{
    EditorCursorDto, EditorCursorIconDto, EditorGizmoHandleKindDto, EditorToolDto,
};

pub fn default_editor_cursor() -> EditorCursorDto {
    EditorCursorDto {
        icon: EditorCursorIconDto::Default,
        visible: true,
        label: None,
    }
}

pub fn editor_cursor(icon: EditorCursorIconDto) -> EditorCursorDto {
    EditorCursorDto {
        icon,
        visible: true,
        label: None,
    }
}

pub fn cursor_for_tool(tool: EditorToolDto) -> EditorCursorDto {
    match tool {
        EditorToolDto::Select => editor_cursor(EditorCursorIconDto::Select),
        EditorToolDto::Move => editor_cursor(EditorCursorIconDto::Move),
        EditorToolDto::Rotate => editor_cursor(EditorCursorIconDto::Rotate),
        EditorToolDto::Scale => editor_cursor(EditorCursorIconDto::Scale),
        EditorToolDto::Rect => editor_cursor(EditorCursorIconDto::Rect),
        EditorToolDto::Pan => editor_cursor(EditorCursorIconDto::Pan),
    }
}

pub fn cursor_for_handle_kind(kind: EditorGizmoHandleKindDto, active: bool) -> EditorCursorDto {
    if active {
        return editor_cursor(EditorCursorIconDto::Grabbing);
    }

    match kind {
        EditorGizmoHandleKindDto::AxisX => editor_cursor(EditorCursorIconDto::MoveX),
        EditorGizmoHandleKindDto::AxisY => editor_cursor(EditorCursorIconDto::MoveY),
        EditorGizmoHandleKindDto::PlaneXY | EditorGizmoHandleKindDto::Body => {
            editor_cursor(EditorCursorIconDto::Move)
        }
        EditorGizmoHandleKindDto::RotationRing => editor_cursor(EditorCursorIconDto::Rotate),
        EditorGizmoHandleKindDto::ScaleCornerNW
        | EditorGizmoHandleKindDto::ScaleCornerNE
        | EditorGizmoHandleKindDto::ScaleCornerSW
        | EditorGizmoHandleKindDto::ScaleCornerSE => editor_cursor(EditorCursorIconDto::Scale),
        EditorGizmoHandleKindDto::ScaleEdgeN | EditorGizmoHandleKindDto::ScaleEdgeS => {
            editor_cursor(EditorCursorIconDto::ScaleY)
        }
        EditorGizmoHandleKindDto::ScaleEdgeE | EditorGizmoHandleKindDto::ScaleEdgeW => {
            editor_cursor(EditorCursorIconDto::ScaleX)
        }
    }
}
