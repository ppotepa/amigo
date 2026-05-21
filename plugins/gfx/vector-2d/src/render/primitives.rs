use amigo_material_api::MaterialCoverageKind2d;
use amigo_render_api::{
    RenderMaterialBinding2d, RenderPrimitive2d, Renderable2dCommon, Renderable2dItem,
    Renderable2dKind, VectorShape2dKindPrimitive, VectorShape2dPrimitive,
    VectorShape2dStylePrimitive, VectorShape2dViewportFit,
};

use crate::vector::{
    VectorShape2dDrawCommand, VectorShapeKind2d, VectorViewportFit2d,
};

fn viewport_fit(mode: VectorViewportFit2d) -> VectorShape2dViewportFit {
    match mode {
        VectorViewportFit2d::Fixed => VectorShape2dViewportFit::Fixed,
        VectorViewportFit2d::Stretch => VectorShape2dViewportFit::Stretch,
        VectorViewportFit2d::Contain => VectorShape2dViewportFit::Contain,
        VectorViewportFit2d::Cover => VectorShape2dViewportFit::Cover,
    }
}

fn shape_kind(kind: &VectorShapeKind2d) -> VectorShape2dKindPrimitive {
    match kind {
        VectorShapeKind2d::Polyline { points, closed } => VectorShape2dKindPrimitive::Polyline {
            points: points.clone(),
            closed: *closed,
        },
        VectorShapeKind2d::Polygon { points } => VectorShape2dKindPrimitive::Polygon {
            points: points.clone(),
        },
        VectorShapeKind2d::Circle { radius, segments } => VectorShape2dKindPrimitive::Circle {
            radius: *radius,
            segments: *segments,
        },
    }
}

pub fn vector_draw_command_to_render_primitive(
    command: &VectorShape2dDrawCommand,
) -> RenderPrimitive2d {
    RenderPrimitive2d::VectorShape(VectorShape2dPrimitive {
        shape: shape_kind(&command.shape.kind),
        style: VectorShape2dStylePrimitive {
            stroke_color: command.shape.style.stroke_color,
            stroke_width: command.shape.style.stroke_width,
            fill_color: command.shape.style.fill_color,
        },
        transform: command.transform,
        viewport_fit: viewport_fit(command.viewport_fit),
        viewport_canvas_size: command.viewport_canvas_size,
        material: RenderMaterialBinding2d::new(
            command.material,
            command.render_contributions.clone(),
            MaterialCoverageKind2d::VectorCoverage,
        ),
    })
}

pub fn vector_draw_command_to_renderable_2d(
    command: &VectorShape2dDrawCommand,
) -> Renderable2dItem {
    Renderable2dItem::new(
        Renderable2dCommon::world(
            command.entity_name.clone(),
            "VectorShape2D",
            command.render_layer.clone(),
            command.z_index,
            Renderable2dKind::Vector,
        ),
        vector_draw_command_to_render_primitive(command),
    )
}
