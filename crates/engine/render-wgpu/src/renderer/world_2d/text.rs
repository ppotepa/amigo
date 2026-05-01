use crate::renderer::*;

pub(crate) fn append_text_2d_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    content: &str,
    transform: Transform2,
    bounds: Vec2,
    color: ColorRgba,
) {
    let pixel_size = (bounds.y / 7.0).clamp(4.0, 18.0);
    let advance = 6.0 * pixel_size;
    let text_width = content.chars().count() as f32 * advance;
    let start_x = -text_width * 0.5;
    let start_y = -3.5 * pixel_size;

    for (index, ch) in content.chars().enumerate() {
        let rows = glyph_rows(ch);
        let glyph_origin_x = start_x + index as f32 * advance;
        for (row_index, row_bits) in rows.iter().enumerate() {
            for column in 0..5 {
                if row_bits & (1 << (4 - column)) == 0 {
                    continue;
                }

                let min = Vec2::new(
                    glyph_origin_x + column as f32 * pixel_size,
                    start_y + (6 - row_index) as f32 * pixel_size,
                );
                let max = Vec2::new(min.x + pixel_size, min.y + pixel_size);
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
                    color,
                );
            }
        }
    }
}

