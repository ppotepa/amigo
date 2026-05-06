use crate::editor_mode::controls::EditorControlBuildContext;
use crate::editor_mode::dto::{
    EditorBounds2Dto, EditorGizmoDto, EditorGizmoHandleDto, EditorGizmoHandleKindDto,
    EditorGizmoHitShapeDto, EditorGizmoKindDto, EditorGizmoPointDto, EditorGizmoPrimitiveDto,
    EditorGizmoRectDto, EditorGizmoToneDto, EditorSceneObjectDto, EditorToolDto,
};

const MOVE_AXIS_LENGTH: f32 = 84.0;
const MOVE_CENTER_SIZE: f32 = 18.0;
const ROTATE_RING_RADIUS: f32 = 52.0;
const ROTATE_RING_WIDTH: f32 = 8.0;
const SCALE_HANDLE_SIZE: f32 = 10.0;

pub fn gizmos_for_selection(
    objects: &[EditorSceneObjectDto],
    selected_entity_ids: &[String],
    active_tool: EditorToolDto,
    control_context: &EditorControlBuildContext,
) -> Vec<EditorGizmoDto> {
    let Some(entity_id) = selected_entity_ids.first() else {
        return Vec::new();
    };
    let Some(object) = objects.iter().find(|object| object.entity_id == *entity_id) else {
        return Vec::new();
    };
    let Some(bounds) = selectable_bounds(object) else {
        return Vec::new();
    };

    let mut gizmos = vec![selection_gizmo(object, bounds)];
    match active_tool {
        EditorToolDto::Select => {}
        EditorToolDto::Move => gizmos.push(move_gizmo(object, bounds, control_context)),
        EditorToolDto::Rotate => gizmos.push(rotate_gizmo(object, bounds, control_context)),
        EditorToolDto::Scale => gizmos.push(scale_gizmo(object, bounds, control_context)),
        EditorToolDto::Rect => gizmos.push(rect_gizmo(object, bounds)),
        EditorToolDto::Pan => {}
    }
    gizmos
}

fn selection_gizmo(object: &EditorSceneObjectDto, bounds: &EditorBounds2Dto) -> EditorGizmoDto {
    EditorGizmoDto {
        id: format!("selection:{}", object.entity_id),
        kind: EditorGizmoKindDto::SelectionBounds2D,
        entity_id: Some(object.entity_id.clone()),
        primitives: vec![EditorGizmoPrimitiveDto::Rect2D {
            rect: rect_from_bounds(bounds),
            tone: EditorGizmoToneDto::Selection,
        }],
        handles: vec![EditorGizmoHandleDto {
            id: "body".to_owned(),
            kind: EditorGizmoHandleKindDto::Body,
            cursor: Some("grab".to_owned()),
            hit_shape: EditorGizmoHitShapeDto::Rect2D {
                rect: rect_from_bounds(bounds),
            },
        }],
    }
}

fn move_gizmo(
    object: &EditorSceneObjectDto,
    bounds: &EditorBounds2Dto,
    control_context: &EditorControlBuildContext,
) -> EditorGizmoDto {
    let gizmo_id = format!("move:{}", object.entity_id);
    let center = center(bounds);
    let axis_x = EditorGizmoPointDto {
        x: center.x + MOVE_AXIS_LENGTH,
        y: center.y,
    };
    let axis_y = EditorGizmoPointDto {
        x: center.x,
        y: center.y + MOVE_AXIS_LENGTH,
    };
    let center_rect = EditorGizmoRectDto {
        x: center.x - MOVE_CENTER_SIZE / 2.0,
        y: center.y - MOVE_CENTER_SIZE / 2.0,
        width: MOVE_CENTER_SIZE,
        height: MOVE_CENTER_SIZE,
    };
    let x_tone = handle_tone(
        control_context,
        &gizmo_id,
        "axis-x",
        EditorGizmoToneDto::X,
        EditorGizmoToneDto::HoverX,
        EditorGizmoToneDto::Active,
    );
    let y_tone = handle_tone(
        control_context,
        &gizmo_id,
        "axis-y",
        EditorGizmoToneDto::Y,
        EditorGizmoToneDto::HoverY,
        EditorGizmoToneDto::Active,
    );
    let center_tone = handle_tone(
        control_context,
        &gizmo_id,
        "plane-xy",
        EditorGizmoToneDto::Center,
        EditorGizmoToneDto::CenterHover,
        EditorGizmoToneDto::CenterActive,
    );

    EditorGizmoDto {
        id: gizmo_id,
        kind: EditorGizmoKindDto::Move2D,
        entity_id: Some(object.entity_id.clone()),
        primitives: vec![
            EditorGizmoPrimitiveDto::Arrow2D {
                from: center.clone(),
                to: axis_x.clone(),
                tone: x_tone,
            },
            EditorGizmoPrimitiveDto::Arrow2D {
                from: center.clone(),
                to: axis_y.clone(),
                tone: y_tone,
            },
            EditorGizmoPrimitiveDto::Rect2D {
                rect: center_rect.clone(),
                tone: center_tone,
            },
        ],
        handles: vec![
            EditorGizmoHandleDto {
                id: "axis-x".to_owned(),
                kind: EditorGizmoHandleKindDto::AxisX,
                cursor: Some("ew-resize".to_owned()),
                hit_shape: EditorGizmoHitShapeDto::Rect2D {
                    rect: EditorGizmoRectDto {
                        x: center.x - 4.0,
                        y: center.y - 12.0,
                        width: MOVE_AXIS_LENGTH + 12.0,
                        height: 24.0,
                    },
                },
            },
            EditorGizmoHandleDto {
                id: "axis-y".to_owned(),
                kind: EditorGizmoHandleKindDto::AxisY,
                cursor: Some("ns-resize".to_owned()),
                hit_shape: EditorGizmoHitShapeDto::Rect2D {
                    rect: EditorGizmoRectDto {
                        x: center.x - 12.0,
                        y: center.y - 4.0,
                        width: 24.0,
                        height: MOVE_AXIS_LENGTH + 12.0,
                    },
                },
            },
            EditorGizmoHandleDto {
                id: "plane-xy".to_owned(),
                kind: EditorGizmoHandleKindDto::PlaneXY,
                cursor: Some("move".to_owned()),
                hit_shape: EditorGizmoHitShapeDto::Rect2D { rect: center_rect },
            },
        ],
    }
}

fn rotate_gizmo(
    object: &EditorSceneObjectDto,
    bounds: &EditorBounds2Dto,
    control_context: &EditorControlBuildContext,
) -> EditorGizmoDto {
    let gizmo_id = format!("rotate:{}", object.entity_id);
    let tone = handle_tone(
        control_context,
        &gizmo_id,
        "rotation-ring",
        EditorGizmoToneDto::Rotation,
        EditorGizmoToneDto::RotationHover,
        EditorGizmoToneDto::RotationActive,
    );
    let center = center(bounds);
    EditorGizmoDto {
        id: gizmo_id,
        kind: EditorGizmoKindDto::Rotate2D,
        entity_id: Some(object.entity_id.clone()),
        primitives: vec![
            EditorGizmoPrimitiveDto::Ring2D {
                center: center.clone(),
                inner_radius: ROTATE_RING_RADIUS - ROTATE_RING_WIDTH / 2.0,
                outer_radius: ROTATE_RING_RADIUS + ROTATE_RING_WIDTH / 2.0,
                tone,
            },
            EditorGizmoPrimitiveDto::Circle2D {
                center: center.clone(),
                radius: 4.0,
                tone: EditorGizmoToneDto::Selection,
            },
        ],
        handles: vec![EditorGizmoHandleDto {
            id: "rotation-ring".to_owned(),
            kind: EditorGizmoHandleKindDto::RotationRing,
            cursor: Some("crosshair".to_owned()),
            hit_shape: EditorGizmoHitShapeDto::Ring2D {
                center,
                inner_radius: ROTATE_RING_RADIUS - 10.0,
                outer_radius: ROTATE_RING_RADIUS + 10.0,
            },
        }],
    }
}

fn scale_gizmo(
    object: &EditorSceneObjectDto,
    bounds: &EditorBounds2Dto,
    control_context: &EditorControlBuildContext,
) -> EditorGizmoDto {
    let gizmo_id = format!("scale:{}", object.entity_id);
    let handles = scale_handles(bounds);
    let mut primitives = vec![EditorGizmoPrimitiveDto::Rect2D {
        rect: rect_from_bounds(bounds),
        tone: EditorGizmoToneDto::Scale,
    }];
    primitives.extend(handles.iter().filter_map(|handle| {
        if let EditorGizmoHitShapeDto::Rect2D { rect } = &handle.hit_shape {
            return Some(EditorGizmoPrimitiveDto::Rect2D {
                rect: rect.clone(),
                tone: handle_tone(
                    control_context,
                    &gizmo_id,
                    &handle.id,
                    EditorGizmoToneDto::Scale,
                    EditorGizmoToneDto::ScaleHover,
                    EditorGizmoToneDto::ScaleActive,
                ),
            });
        }
        None
    }));

    EditorGizmoDto {
        id: gizmo_id,
        kind: EditorGizmoKindDto::Scale2D,
        entity_id: Some(object.entity_id.clone()),
        primitives,
        handles,
    }
}

fn rect_gizmo(object: &EditorSceneObjectDto, bounds: &EditorBounds2Dto) -> EditorGizmoDto {
    EditorGizmoDto {
        id: format!("rect:{}", object.entity_id),
        kind: EditorGizmoKindDto::Rect2D,
        entity_id: Some(object.entity_id.clone()),
        primitives: vec![EditorGizmoPrimitiveDto::Rect2D {
            rect: rect_from_bounds(bounds),
            tone: EditorGizmoToneDto::Warning,
        }],
        handles: scale_handles(bounds),
    }
}

fn handle_tone(
    context: &EditorControlBuildContext,
    control_id: &str,
    handle_id: &str,
    idle: EditorGizmoToneDto,
    hovered: EditorGizmoToneDto,
    active: EditorGizmoToneDto,
) -> EditorGizmoToneDto {
    if context.is_active(control_id, handle_id) {
        active
    } else if context.is_hovered(control_id, handle_id) {
        hovered
    } else {
        idle
    }
}

fn scale_handles(bounds: &EditorBounds2Dto) -> Vec<EditorGizmoHandleDto> {
    let half = SCALE_HANDLE_SIZE / 2.0;
    let left = bounds.x;
    let right = bounds.x + bounds.width;
    let top = bounds.y;
    let bottom = bounds.y + bounds.height;
    let mid_x = bounds.x + bounds.width / 2.0;
    let mid_y = bounds.y + bounds.height / 2.0;

    vec![
        scale_handle(
            "nw",
            EditorGizmoHandleKindDto::ScaleCornerNW,
            left - half,
            top - half,
            "nwse-resize",
        ),
        scale_handle(
            "ne",
            EditorGizmoHandleKindDto::ScaleCornerNE,
            right - half,
            top - half,
            "nesw-resize",
        ),
        scale_handle(
            "sw",
            EditorGizmoHandleKindDto::ScaleCornerSW,
            left - half,
            bottom - half,
            "nesw-resize",
        ),
        scale_handle(
            "se",
            EditorGizmoHandleKindDto::ScaleCornerSE,
            right - half,
            bottom - half,
            "nwse-resize",
        ),
        scale_handle(
            "n",
            EditorGizmoHandleKindDto::ScaleEdgeN,
            mid_x - half,
            top - half,
            "ns-resize",
        ),
        scale_handle(
            "e",
            EditorGizmoHandleKindDto::ScaleEdgeE,
            right - half,
            mid_y - half,
            "ew-resize",
        ),
        scale_handle(
            "s",
            EditorGizmoHandleKindDto::ScaleEdgeS,
            mid_x - half,
            bottom - half,
            "ns-resize",
        ),
        scale_handle(
            "w",
            EditorGizmoHandleKindDto::ScaleEdgeW,
            left - half,
            mid_y - half,
            "ew-resize",
        ),
    ]
}

fn scale_handle(
    id: &str,
    kind: EditorGizmoHandleKindDto,
    x: f32,
    y: f32,
    cursor: &str,
) -> EditorGizmoHandleDto {
    EditorGizmoHandleDto {
        id: id.to_owned(),
        kind,
        cursor: Some(cursor.to_owned()),
        hit_shape: EditorGizmoHitShapeDto::Rect2D {
            rect: EditorGizmoRectDto {
                x,
                y,
                width: SCALE_HANDLE_SIZE,
                height: SCALE_HANDLE_SIZE,
            },
        },
    }
}

pub fn selectable_bounds(object: &EditorSceneObjectDto) -> Option<&EditorBounds2Dto> {
    object
        .selection_bounds_2
        .as_ref()
        .or(object.bounds_2.as_ref())
        .or(object.render_bounds_2.as_ref())
}

pub fn point_in_bounds(x: f32, y: f32, bounds: &EditorBounds2Dto) -> bool {
    x >= bounds.x && x <= bounds.x + bounds.width && y >= bounds.y && y <= bounds.y + bounds.height
}

fn rect_from_bounds(bounds: &EditorBounds2Dto) -> EditorGizmoRectDto {
    EditorGizmoRectDto {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
    }
}

fn center(bounds: &EditorBounds2Dto) -> EditorGizmoPointDto {
    EditorGizmoPointDto {
        x: bounds.x + bounds.width / 2.0,
        y: bounds.y + bounds.height / 2.0,
    }
}
