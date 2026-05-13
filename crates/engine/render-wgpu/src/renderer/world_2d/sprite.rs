use crate::renderer::*;

pub(crate) fn append_sprite_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    sprite: &Sprite,
    color: ColorRgba,
) {
    let asset_key = sprite.texture.as_str();
    let size = sprite.size;
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let points = [
        transform_point_2d(Vec2::new(-half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, -half.y), transform),
        transform_point_2d(Vec2::new(half.x, half.y), transform),
        transform_point_2d(Vec2::new(-half.x, half.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(points[0], camera, viewport),
        ndc_from_world_2d(points[1], camera, viewport),
        ndc_from_world_2d(points[2], camera, viewport),
        ndc_from_world_2d(points[3], camera, viewport),
        if sprite.sheet.is_some() {
            modulate_color(color, 0.18)
        } else {
            color
        },
    );

    if let Some(sheet) = sprite.sheet {
        append_sprite_sheet_overlay(
            vertices,
            viewport,
            camera,
            transform,
            size,
            sheet,
            sprite.frame_index,
            color,
        );
    } else if asset_key.contains("square") || asset_key.contains("sprite") {
        let marker_half = Vec2::new(size.x * 0.12, size.y * 0.12);
        let marker_center = Vec2::new(size.x * 0.18, size.y * 0.18);
        let marker_points = [
            transform_point_2d(
                Vec2::new(
                    marker_center.x - marker_half.x,
                    marker_center.y - marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x + marker_half.x,
                    marker_center.y - marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x + marker_half.x,
                    marker_center.y + marker_half.y,
                ),
                transform,
            ),
            transform_point_2d(
                Vec2::new(
                    marker_center.x - marker_half.x,
                    marker_center.y + marker_half.y,
                ),
                transform,
            ),
        ];

        push_quad(
            vertices,
            ndc_from_world_2d(marker_points[0], camera, viewport),
            ndc_from_world_2d(marker_points[1], camera, viewport),
            ndc_from_world_2d(marker_points[2], camera, viewport),
            ndc_from_world_2d(marker_points[3], camera, viewport),
            ColorRgba::new(0.98, 0.98, 0.98, 1.0),
        );
    }
}

pub(crate) fn append_sprite_sheet_overlay(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    size: Vec2,
    sheet: SpriteSheet,
    frame_index: u32,
    base_color: ColorRgba,
) {
    let columns = sheet.columns.max(1);
    let rows = sheet.rows.max(1);
    let visible_frames = sheet.visible_frame_count();
    let half = Vec2::new(size.x * 0.5, size.y * 0.5);
    let sheet_width = size.x * 0.64;
    let preview_width = size.x * 0.24;
    let gap_width = (size.x - sheet_width - preview_width).max(12.0);
    let sheet_left = -half.x;
    let sheet_right = sheet_left + sheet_width;
    let preview_left = sheet_right + gap_width;
    let preview_right = half.x;
    let cell_size = Vec2::new(sheet_width / columns as f32, size.y / rows as f32);
    let pad = Vec2::new((cell_size.x * 0.08).max(3.0), (cell_size.y * 0.08).max(3.0));

    for frame in 0..visible_frames {
        let column = frame % columns;
        let row = frame / columns;
        let left = sheet_left + column as f32 * cell_size.x;
        let right = left + cell_size.x;
        let top = half.y - row as f32 * cell_size.y;
        let bottom = top - cell_size.y;
        let min = Vec2::new(left + pad.x, bottom + pad.y);
        let max = Vec2::new(right - pad.x, top - pad.y);
        let frame_color = if frame == frame_index.min(visible_frames.saturating_sub(1)) {
            ColorRgba::new(0.99, 0.97, 0.88, 1.0)
        } else {
            modulate_color(
                blend_colors(base_color, spritesheet_frame_color(frame)),
                0.42,
            )
        };
        let quad = [
            transform_point_2d(min, transform),
            transform_point_2d(Vec2::new(max.x, min.y), transform),
            transform_point_2d(max, transform),
            transform_point_2d(Vec2::new(min.x, max.y), transform),
        ];
        push_quad(
            vertices,
            ndc_from_world_2d(quad[0], camera, viewport),
            ndc_from_world_2d(quad[1], camera, viewport),
            ndc_from_world_2d(quad[2], camera, viewport),
            ndc_from_world_2d(quad[3], camera, viewport),
            frame_color,
        );
    }

    let preview_frame = frame_index.min(visible_frames.saturating_sub(1));
    let preview_color = blend_colors(base_color, spritesheet_frame_color(preview_frame));
    let preview_pad = Vec2::new(
        ((preview_right - preview_left) * 0.16).max(4.0),
        (size.y * 0.16).max(4.0),
    );
    let preview_min = Vec2::new(preview_left + preview_pad.x, -half.y + preview_pad.y);
    let preview_max = Vec2::new(preview_right - preview_pad.x, half.y - preview_pad.y);
    let preview_quad = [
        transform_point_2d(preview_min, transform),
        transform_point_2d(Vec2::new(preview_max.x, preview_min.y), transform),
        transform_point_2d(preview_max, transform),
        transform_point_2d(Vec2::new(preview_min.x, preview_max.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(preview_quad[0], camera, viewport),
        ndc_from_world_2d(preview_quad[1], camera, viewport),
        ndc_from_world_2d(preview_quad[2], camera, viewport),
        ndc_from_world_2d(preview_quad[3], camera, viewport),
        preview_color,
    );

    let marker_size = Vec2::new(
        (preview_max.x - preview_min.x) * 0.28,
        (preview_max.y - preview_min.y) * 0.28,
    );
    let marker_center = Vec2::new(
        (preview_min.x + preview_max.x) * 0.5,
        (preview_min.y + preview_max.y) * 0.5,
    );
    let marker_min = Vec2::new(
        marker_center.x - marker_size.x * 0.5,
        marker_center.y - marker_size.y * 0.5,
    );
    let marker_max = Vec2::new(
        marker_center.x + marker_size.x * 0.5,
        marker_center.y + marker_size.y * 0.5,
    );
    let marker_quad = [
        transform_point_2d(marker_min, transform),
        transform_point_2d(Vec2::new(marker_max.x, marker_min.y), transform),
        transform_point_2d(marker_max, transform),
        transform_point_2d(Vec2::new(marker_min.x, marker_max.y), transform),
    ];
    push_quad(
        vertices,
        ndc_from_world_2d(marker_quad[0], camera, viewport),
        ndc_from_world_2d(marker_quad[1], camera, viewport),
        ndc_from_world_2d(marker_quad[2], camera, viewport),
        ndc_from_world_2d(marker_quad[3], camera, viewport),
        ColorRgba::new(0.98, 0.98, 0.98, 1.0),
    );
}

