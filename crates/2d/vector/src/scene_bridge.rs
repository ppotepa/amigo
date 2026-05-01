use amigo_scene::{
    SceneService, VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand, VectorStyle2dSceneCommand,
};
use crate::model::{VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d};
use crate::service::VectorSceneService;

pub fn queue_vector_shape_scene_command(
    scene_service: &SceneService,
    vector_scene_service: &VectorSceneService,
    command: &VectorShape2dSceneCommand,
) -> amigo_scene::SceneEntityId {
    let entity = scene_service.find_or_spawn_named_entity(command.entity_name.clone());
    vector_scene_service.queue(VectorShape2dDrawCommand {
        entity_id: entity,
        entity_name: command.entity_name.clone(),
        shape: VectorShape2d {
            kind: map_shape_kind(&command.kind),
            style: map_style(&command.style),
        },
        z_index: command.z_index,
        transform: command.transform,
    });
    entity
}

fn map_shape_kind(kind: &VectorShapeKind2dSceneCommand) -> VectorShapeKind2d {
    match kind {
        VectorShapeKind2dSceneCommand::Polyline { points, closed } => VectorShapeKind2d::Polyline {
            points: points.clone(),
            closed: *closed,
        },
        VectorShapeKind2dSceneCommand::Polygon { points } => VectorShapeKind2d::Polygon {
            points: points.clone(),
        },
        VectorShapeKind2dSceneCommand::Circle { radius, segments } => VectorShapeKind2d::Circle {
            radius: *radius,
            segments: (*segments).max(3),
        },
    }
}

fn map_style(style: &VectorStyle2dSceneCommand) -> VectorStyle2d {
    VectorStyle2d {
        stroke_color: style.stroke_color,
        stroke_width: style.stroke_width.max(0.0),
        fill_color: style.fill_color,
    }
}
