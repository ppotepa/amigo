use crate::model::{
    ProceduralVectorError, RadialJitterPolygon, VectorShape2d,
    VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d,
    radial_jitter_polygon_points,
};
use crate::VectorSceneService;
use crate::plugin::Vector2dPlugin;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_runtime::RuntimeBuilder;
use amigo_scene::{
    SceneService, VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand,
    VectorStyle2dSceneCommand,
};

#[test]
fn stores_vector_draw_commands() {
    let service = VectorSceneService::default();
    service.queue(VectorShape2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(1),
        entity_name: "test-shape".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polyline {
                points: vec![Vec2::new(0.0, 12.0), Vec2::new(-8.0, -8.0)],
                closed: true,
            },
            style: VectorStyle2d {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 2.0,
                fill_color: None,
            },
        },
        z_index: 1.0,
        transform: Transform2::default(),
    });

    assert_eq!(service.commands().len(), 1);
    assert_eq!(service.entity_names(), vec!["test-shape".to_owned()]);

    service.clear();
    assert!(service.commands().is_empty());
}

#[test]
fn updates_vector_polygon_points_by_entity_name() {
    let service = VectorSceneService::default();
    service.queue(VectorShape2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(1),
        entity_name: "asteroid".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polygon {
                points: vec![
                    Vec2::new(-8.0, 0.0),
                    Vec2::new(0.0, 8.0),
                    Vec2::new(8.0, 0.0),
                ],
            },
            style: VectorStyle2d::default(),
        },
        z_index: 1.0,
        transform: Transform2::default(),
    });

    assert!(service.set_polygon_points(
        "asteroid",
        vec![
            Vec2::new(-10.0, -2.0),
            Vec2::new(-2.0, 9.0),
            Vec2::new(8.0, 7.0),
            Vec2::new(10.0, -4.0),
        ],
    ));

    let commands = service.commands();
    assert_eq!(commands.len(), 1);
    match &commands[0].shape.kind {
        VectorShapeKind2d::Polygon { points } => {
            assert_eq!(points.len(), 4);
            assert_eq!(points[0], Vec2::new(-10.0, -2.0));
        }
        other => panic!("expected polygon, got {other:?}"),
    }
}

#[test]
fn generates_radial_jitter_polygon_deterministically() {
    let config = RadialJitterPolygon::new(8, 12.0, 0.35, 42);

    let first = radial_jitter_polygon_points(config).expect("valid polygon config");
    let second = radial_jitter_polygon_points(config).expect("valid polygon config");
    let different = radial_jitter_polygon_points(RadialJitterPolygon::new(8, 12.0, 0.35, 43))
        .expect("valid polygon config");

    assert_eq!(first, second);
    assert_ne!(first, different);
    assert_eq!(first.len(), 8);
}

#[test]
fn validates_radial_jitter_polygon_config() {
    assert_eq!(
        radial_jitter_polygon_points(RadialJitterPolygon::new(2, 12.0, 0.0, 1)),
        Err(ProceduralVectorError::TooFewVertices)
    );
    assert_eq!(
        radial_jitter_polygon_points(RadialJitterPolygon::new(3, -1.0, 0.0, 1)),
        Err(ProceduralVectorError::InvalidRadius)
    );
}

#[test]
fn clamps_radial_jitter_polygon_jitter() {
    let no_jitter =
        radial_jitter_polygon_points(RadialJitterPolygon::new(5, 10.0, f32::NAN, 7))
            .expect("nan jitter should be handled defensively");
    let negative_jitter =
        radial_jitter_polygon_points(RadialJitterPolygon::new(5, 10.0, -1.0, 7))
            .expect("negative jitter should be clamped");

    assert_eq!(no_jitter, negative_jitter);
    for point in radial_jitter_polygon_points(RadialJitterPolygon::new(12, 10.0, 10.0, 7))
        .expect("large jitter should be clamped")
    {
        let distance = (point.x * point.x + point.y * point.y).sqrt();
        assert!((0.0..=20.0).contains(&distance));
    }
}

#[test]
fn applies_radial_jitter_polygon_to_existing_entity() {
    let service = VectorSceneService::default();
    service.queue(VectorShape2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(1),
        entity_name: "rock".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polygon {
                points: vec![
                    Vec2::new(-8.0, 0.0),
                    Vec2::new(0.0, 8.0),
                    Vec2::new(8.0, 0.0),
                ],
            },
            style: VectorStyle2d::default(),
        },
        z_index: 1.0,
        transform: Transform2::default(),
    });

    assert!(
        service.set_radial_jitter_polygon("rock", RadialJitterPolygon::new(6, 9.0, 0.25, 99),)
    );
    assert!(
        !service.set_radial_jitter_polygon(
            "missing",
            RadialJitterPolygon::new(6, 9.0, 0.25, 99),
        )
    );
    assert!(
        !service.set_radial_jitter_polygon("rock", RadialJitterPolygon::new(2, 9.0, 0.25, 99),)
    );

    let commands = service.commands();
    match &commands[0].shape.kind {
        VectorShapeKind2d::Polygon { points } => assert_eq!(points.len(), 6),
        other => panic!("expected polygon, got {other:?}"),
    }
}

#[test]
fn updates_vector_polyline_points_by_entity_name() {
    let service = VectorSceneService::default();
    service.queue(VectorShape2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(2),
        entity_name: "trail".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polyline {
                points: vec![Vec2::new(0.0, 0.0), Vec2::new(8.0, 4.0)],
                closed: false,
            },
            style: VectorStyle2d::default(),
        },
        z_index: 1.0,
        transform: Transform2::default(),
    });

    assert!(service.set_polyline_points(
        "trail",
        vec![
            Vec2::new(-6.0, 1.0),
            Vec2::new(0.0, 5.0),
            Vec2::new(6.0, 2.0),
        ],
        true,
    ));

    let commands = service.commands();
    match &commands[0].shape.kind {
        VectorShapeKind2d::Polyline { points, closed } => {
            assert_eq!(points.len(), 3);
            assert!(*closed);
        }
        other => panic!("expected polyline, got {other:?}"),
    }
}

#[test]
fn queues_vector_shape_scene_command() {
    let scene = SceneService::default();
    let service = VectorSceneService::default();
    let command = VectorShape2dSceneCommand {
        source_mod: "test-mod".to_owned(),
        entity_name: "test-shape".to_owned(),
        kind: VectorShapeKind2dSceneCommand::Polyline {
            points: vec![
                Vec2::new(0.0, 12.0),
                Vec2::new(-8.0, -8.0),
                Vec2::new(8.0, -8.0),
            ],
            closed: true,
        },
        style: VectorStyle2dSceneCommand {
            stroke_color: ColorRgba::WHITE,
            stroke_width: 2.0,
            fill_color: None,
        },
        z_index: 2.0,
        transform: Transform2::default(),
    };

    let entity = crate::scene_bridge::queue_vector_shape_scene_command(&scene, &service, &command);
    assert_eq!(entity.raw(), 0);
    assert_eq!(service.commands().len(), 1);
    assert_eq!(scene.entity_names(), vec!["test-shape".to_owned()]);
}

#[test]
fn registers_vector_runtime_plugin() {
    let runtime = RuntimeBuilder::default()
        .with_plugin(Vector2dPlugin)
        .expect("vector plugin should register")
        .build();
    assert!(runtime.resolve::<crate::service::VectorSceneService>().is_some());
}
