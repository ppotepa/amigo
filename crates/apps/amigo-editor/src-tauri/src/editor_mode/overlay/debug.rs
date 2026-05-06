use super::super::coordinates::{EditorCoordinateMapper, EditorScenePoint, EditorSceneRect};
use super::super::session::{EditorActiveInteractionKind, EditorModeSession};

pub fn render_draft_object_proxy(
    svg: &mut String,
    session: &EditorModeSession,
    mapper: EditorCoordinateMapper,
) {
    let Some(interaction) = &session.active_interaction else {
        return;
    };
    if !matches!(
        interaction.kind,
        EditorActiveInteractionKind::MoveEntity2D
            | EditorActiveInteractionKind::MoveAxisX
            | EditorActiveInteractionKind::MoveAxisY
    ) {
        return;
    }
    let Some(entity_id) = interaction.entity_id.as_deref() else {
        return;
    };
    let Some(object) = session
        .snapshot
        .objects
        .iter()
        .find(|object| object.entity_id == entity_id)
    else {
        return;
    };
    let Some(bounds) = object
        .render_bounds_2
        .as_ref()
        .or(object.selection_bounds_2.as_ref())
    else {
        return;
    };
    let rect = mapper.scene_to_frame_rect(EditorSceneRect {
        x: bounds.x,
        y: bounds.y,
        width: bounds.width,
        height: bounds.height,
    });
    svg.push_str(&format!(
        r##"<rect x="{:.2}" y="{:.2}" width="{:.2}" height="{:.2}" fill="#00ffff" fill-opacity="0.12" stroke="#00ffff" stroke-width="1" stroke-dasharray="4 4"/>"##,
        rect.x, rect.y, rect.width, rect.height,
    ));
}

pub fn render_debug_origin(svg: &mut String, mapper: EditorCoordinateMapper) {
    let origin = mapper.scene_to_frame_point(EditorScenePoint { x: 0.0, y: 0.0 });
    svg.push_str(&format!(
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#ff00ff" stroke-width="1"/>"##,
        origin.x - 16.0,
        origin.y,
        origin.x + 16.0,
        origin.y,
    ));
    svg.push_str(&format!(
        r##"<line x1="{:.2}" y1="{:.2}" x2="{:.2}" y2="{:.2}" stroke="#ff00ff" stroke-width="1"/>"##,
        origin.x,
        origin.y - 16.0,
        origin.x,
        origin.y + 16.0,
    ));
    svg.push_str(&format!(
        r##"<text x="{:.2}" y="{:.2}" fill="#ff00ff" font-size="11">0,0</text>"##,
        origin.x + 6.0,
        origin.y - 6.0,
    ));
}

pub fn render_debug_pointer(
    svg: &mut String,
    session: &EditorModeSession,
    mapper: EditorCoordinateMapper,
) {
    if let (Some(frame_x), Some(frame_y)) =
        (session.last_pointer_frame_x, session.last_pointer_frame_y)
    {
        svg.push_str(&format!(
            r##"<circle cx="{:.2}" cy="{:.2}" r="5" fill="none" stroke="#00ffff" stroke-width="2"/>"##,
            frame_x, frame_y,
        ));
        svg.push_str(&format!(
            r##"<text x="{:.2}" y="{:.2}" fill="#00ffff" font-size="11">frame</text>"##,
            frame_x + 7.0,
            frame_y - 7.0,
        ));
    }

    if let (Some(scene_x), Some(scene_y)) =
        (session.last_pointer_scene_x, session.last_pointer_scene_y)
    {
        let point = mapper.scene_to_frame_point(EditorScenePoint {
            x: scene_x,
            y: scene_y,
        });
        svg.push_str(&format!(
            r##"<circle cx="{:.2}" cy="{:.2}" r="8" fill="none" stroke="#ff00ff" stroke-width="1.5" stroke-dasharray="3 3"/>"##,
            point.x, point.y,
        ));
        svg.push_str(&format!(
            r##"<text x="{:.2}" y="{:.2}" fill="#ff00ff" font-size="11">scene</text>"##,
            point.x + 8.0,
            point.y + 14.0,
        ));
    }
}
