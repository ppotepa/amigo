use crate::renderer::*;

pub(crate) fn append_vector_shape_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    transform: Transform2,
    shape: &VectorShape2d,
) {
    let local_points = vector_shape_points(shape);
    if local_points.is_empty() {
        return;
    }

    let world_points = local_points
        .into_iter()
        .map(|point| transform_point_2d(point, transform))
        .collect::<Vec<_>>();
    let (closed, can_fill) = match &shape.kind {
        VectorShapeKind2d::Polyline { closed, .. } => (*closed, *closed),
        VectorShapeKind2d::Polygon { .. } | VectorShapeKind2d::Circle { .. } => (true, true),
    };

    if can_fill {
        if let Some(fill_color) = shape.style.fill_color {
            append_filled_polygon_vertices(vertices, viewport, camera, &world_points, fill_color);
        }
    }

    if shape.style.stroke_width > 0.0 {
        append_polyline_stroke_vertices(
            vertices,
            viewport,
            camera,
            &world_points,
            closed,
            shape.style.stroke_width,
            shape.style.stroke_color,
        );
    }
}

fn vector_shape_points(shape: &VectorShape2d) -> Vec<Vec2> {
    match &shape.kind {
        VectorShapeKind2d::Polyline { points, .. } | VectorShapeKind2d::Polygon { points } => {
            points.clone()
        }
        VectorShapeKind2d::Circle { radius, segments } => {
            let segment_count = (*segments).max(3) as usize;
            let mut points = Vec::with_capacity(segment_count);
            for index in 0..segment_count {
                let angle = (index as f32 / segment_count as f32) * std::f32::consts::TAU;
                points.push(Vec2::new(angle.cos() * *radius, angle.sin() * *radius));
            }
            points
        }
    }
}

fn append_filled_polygon_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    points: &[Vec2],
    color: ColorRgba,
) {
    if points.len() < 3 {
        return;
    }

    let origin = ndc_from_world_2d(points[0], camera, viewport);
    for index in 1..points.len() - 1 {
        push_triangle(
            vertices,
            [
                origin,
                ndc_from_world_2d(points[index], camera, viewport),
                ndc_from_world_2d(points[index + 1], camera, viewport),
            ],
            color,
        );
    }
}

fn append_polyline_stroke_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    points: &[Vec2],
    closed: bool,
    stroke_width: f32,
    color: ColorRgba,
) {
    if points.len() < 2 {
        return;
    }

    for index in 0..points.len() - 1 {
        append_line_segment_vertices(
            vertices,
            viewport,
            camera,
            points[index],
            points[index + 1],
            stroke_width,
            color,
        );
    }

    if closed {
        append_line_segment_vertices(
            vertices,
            viewport,
            camera,
            *points
                .last()
                .expect("closed vector shape should have a last point"),
            points[0],
            stroke_width,
            color,
        );
    }
}

fn append_line_segment_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    start: Vec2,
    end: Vec2,
    stroke_width: f32,
    color: ColorRgba,
) {
    let dx = end.x - start.x;
    let dy = end.y - start.y;
    let length = (dx * dx + dy * dy).sqrt();
    if length <= f32::EPSILON {
        return;
    }

    let half_width = stroke_width * 0.5;
    let normal = Vec2::new(-dy / length * half_width, dx / length * half_width);
    let a = Vec2::new(start.x + normal.x, start.y + normal.y);
    let b = Vec2::new(end.x + normal.x, end.y + normal.y);
    let c = Vec2::new(end.x - normal.x, end.y - normal.y);
    let d = Vec2::new(start.x - normal.x, start.y - normal.y);
    push_quad(
        vertices,
        ndc_from_world_2d(a, camera, viewport),
        ndc_from_world_2d(b, camera, viewport),
        ndc_from_world_2d(c, camera, viewport),
        ndc_from_world_2d(d, camera, viewport),
        color,
    );
}

