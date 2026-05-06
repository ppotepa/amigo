mod debug;
mod primitives;
mod svg;

#[cfg(test)]
mod tests;

use super::coordinates::EditorCoordinateMapper;
use super::dto::EditorSceneSnapshotDto;
use super::session::EditorModeSession;

pub fn compose_editor_overlay_image_url(
    image_url: Option<String>,
    width: u32,
    height: u32,
    snapshot: &EditorSceneSnapshotDto,
    session: &EditorModeSession,
    mapper: EditorCoordinateMapper,
) -> Option<String> {
    let image_url = image_url?;
    let width = width.max(1);
    let height = height.max(1);
    let mut svg = String::new();
    svg::append_svg_header(&mut svg, width, height);
    svg::append_background_image(&mut svg, &image_url, width, height);
    svg.push_str(r#"<g fill="none" stroke-linecap="round" stroke-linejoin="round">"#);
    debug::render_debug_origin(&mut svg, mapper);
    debug::render_debug_pointer(&mut svg, session, mapper);
    debug::render_draft_object_proxy(&mut svg, session, mapper);

    for gizmo in &snapshot.gizmos {
        for primitive in &gizmo.primitives {
            primitives::append_primitive(&mut svg, primitive, mapper);
        }
    }

    svg.push_str("</g></svg>");
    Some(format!(
        "data:image/svg+xml;base64,{}",
        svg::encode_base64(svg.as_bytes())
    ))
}
