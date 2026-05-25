use amigo_render_api::{
    VectorShape2dKindPrimitive, VectorShape2dPrimitive, VectorShape2dViewportFit,
};

use crate::renderer::*;

pub(crate) fn append_vector_primitive_vertices(
    vertices: &mut Vec<ColorVertex>,
    viewport: &Viewport,
    camera: Transform2,
    primitive: &VectorShape2dPrimitive,
    override_transform: Option<Transform2>,
    override_fill: Option<ColorRgba>,
    override_stroke: Option<ColorRgba>,
) {
    let transform = override_transform
        .unwrap_or_else(|| vector_primitive_viewport_fit_transform(viewport, primitive));
    let local_points = vector_shape_points(primitive);
    if local_points.is_empty() {
        return;
    }

    let world_points = local_points
        .into_iter()
        .map(|point| transform_point_2d(point, transform))
        .collect::<Vec<_>>();
    let (closed, can_fill) = match &primitive.shape {
        VectorShape2dKindPrimitive::Polyline { closed, .. } => (*closed, *closed),
        VectorShape2dKindPrimitive::Polygon { .. } | VectorShape2dKindPrimitive::Circle { .. } => {
            (true, true)
        }
    };
    let fill_color = override_fill.or(primitive.style.fill_color);
    let stroke_color = override_stroke.unwrap_or(primitive.style.stroke_color);

    if can_fill {
        if let Some(fill_color) = fill_color {
            append_filled_polygon_vertices(vertices, viewport, camera, &world_points, fill_color);
        }
    }

    if primitive.style.stroke_width > 0.0 {
        append_polyline_stroke_vertices(
            vertices,
            viewport,
            camera,
            &world_points,
            closed,
            primitive.style.stroke_width,
            stroke_color,
        );
    }
}

pub(crate) fn vector_primitive_viewport_fit_transform(
    viewport: &Viewport,
    primitive: &VectorShape2dPrimitive,
) -> Transform2 {
    let Some(canvas_size) = primitive.viewport_canvas_size else {
        return primitive.transform;
    };
    if canvas_size.x <= 0.0 || canvas_size.y <= 0.0 {
        return primitive.transform;
    }

    let viewport_size = viewport.size();
    let scale_x = viewport_size.x / canvas_size.x;
    let scale_y = viewport_size.y / canvas_size.y;
    let scale = match primitive.viewport_fit {
        VectorShape2dViewportFit::Fixed => return primitive.transform,
        VectorShape2dViewportFit::Stretch => Vec2::new(scale_x, scale_y),
        VectorShape2dViewportFit::Contain => {
            let scale = scale_x.min(scale_y);
            Vec2::new(scale, scale)
        }
        VectorShape2dViewportFit::Cover => {
            let scale = scale_x.max(scale_y);
            Vec2::new(scale, scale)
        }
    };

    let mut transform = primitive.transform;
    transform.translation.x *= scale.x;
    transform.translation.y *= scale.y;
    transform.scale.x *= scale.x;
    transform.scale.y *= scale.y;
    transform
}

fn vector_shape_points(shape: &VectorShape2dPrimitive) -> Vec<Vec2> {
    match &shape.shape {
        VectorShape2dKindPrimitive::Polyline { points, .. }
        | VectorShape2dKindPrimitive::Polygon { points } => points.clone(),
        VectorShape2dKindPrimitive::Circle { radius, segments } => {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cover_fit_scales_vector_transform_from_reference_canvas() {
        let viewport = Viewport::from_dimensions(1920.0, 1080.0);
        let transform = Transform2 {
            translation: Vec2::new(100.0, -50.0),
            scale: Vec2::new(2.0, 3.0),
            ..Default::default()
        };
        let primitive = VectorShape2dPrimitive {
            shape: VectorShape2dKindPrimitive::Polygon {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.0, 1.0),
                ],
            },
            style: amigo_render_api::VectorShape2dStylePrimitive {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 1.0,
                fill_color: None,
            },
            transform,
            viewport_fit: VectorShape2dViewportFit::Cover,
            viewport_canvas_size: Some(Vec2::new(1280.0, 720.0)),
            material: amigo_render_api::RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::VectorCoverage,
            ),
        };

        let fitted = vector_primitive_viewport_fit_transform(&viewport, &primitive);

        assert_eq!(fitted.translation, Vec2::new(150.0, -75.0));
        assert_eq!(fitted.scale, Vec2::new(3.0, 4.5));
    }

    #[test]
    fn fixed_fit_leaves_vector_transform_unchanged() {
        let viewport = Viewport::from_dimensions(1920.0, 1080.0);
        let transform = Transform2 {
            translation: Vec2::new(100.0, -50.0),
            scale: Vec2::new(2.0, 3.0),
            ..Default::default()
        };
        let primitive = VectorShape2dPrimitive {
            shape: VectorShape2dKindPrimitive::Polygon {
                points: vec![
                    Vec2::new(0.0, 0.0),
                    Vec2::new(1.0, 0.0),
                    Vec2::new(0.0, 1.0),
                ],
            },
            style: amigo_render_api::VectorShape2dStylePrimitive {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 1.0,
                fill_color: None,
            },
            transform,
            viewport_fit: VectorShape2dViewportFit::Fixed,
            viewport_canvas_size: Some(Vec2::new(1280.0, 720.0)),
            material: amigo_render_api::RenderMaterialBinding2d::none(
                amigo_material_api::MaterialCoverageKind2d::VectorCoverage,
            ),
        };

        let fitted = vector_primitive_viewport_fit_transform(&viewport, &primitive);

        assert_eq!(fitted, transform);
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
