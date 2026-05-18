use crate::VectorSceneService;
use crate::model::{
    ProceduralVectorError, RadialJitterPolygon, VectorShape2d, VectorShape2dDrawCommand,
    VectorShapeKind2d, VectorStyle2d, VectorViewportFit2d, radial_jitter_polygon_points,
};
use crate::plugin::Vector2dPlugin;
use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_render_api::{RenderContributionSet, render_contribution_roles as roles};
use amigo_runtime::RuntimeBuilder;
use amigo_scene::{
    Material2dOpticalModeSceneCommand, Material2dOpticalSceneCommand, Material2dSceneCommand,
    Material2dCameraResponseSceneCommand, Material2dLightingSceneCommand,
    RenderContributions2dSceneCommand, RuntimeSceneCommandHandlerRegistry, SceneCommand, SceneEvent,
    SceneEventQueue, SceneService, VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand,
    VectorStyle2dSceneCommand,
};

#[test]
fn stores_vector_draw_commands() {
    let service = VectorSceneService::default();
    service.queue(VectorShape2dDrawCommand {
        entity_id: amigo_scene::SceneEntityId::new(1),
        entity_name: "test-shape".to_owned(),
        render_layer: "world".to_owned(),
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
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: RenderContributionSet::default(),
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
        render_layer: "world".to_owned(),
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
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: RenderContributionSet::default(),
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
    let no_jitter = radial_jitter_polygon_points(RadialJitterPolygon::new(5, 10.0, f32::NAN, 7))
        .expect("nan jitter should be handled defensively");
    let negative_jitter = radial_jitter_polygon_points(RadialJitterPolygon::new(5, 10.0, -1.0, 7))
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
        render_layer: "world".to_owned(),
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
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: RenderContributionSet::default(),
    });

    assert!(service.set_radial_jitter_polygon("rock", RadialJitterPolygon::new(6, 9.0, 0.25, 99),));
    assert!(
        !service.set_radial_jitter_polygon("missing", RadialJitterPolygon::new(6, 9.0, 0.25, 99),)
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
        render_layer: "world".to_owned(),
        shape: VectorShape2d {
            kind: VectorShapeKind2d::Polyline {
                points: vec![Vec2::new(0.0, 0.0), Vec2::new(8.0, 4.0)],
                closed: false,
            },
            style: VectorStyle2d::default(),
        },
        z_index: 1.0,
        transform: Transform2::default(),
        viewport_fit: VectorViewportFit2d::Fixed,
        viewport_canvas_size: None,
        material: None,
        render_contributions: RenderContributionSet::default(),
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
        render_layer: "world".to_owned(),
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
        render_contributions: RenderContributions2dSceneCommand::default(),
        material: None,
        transform: Transform2::default(),
    };

    let entity = crate::scene_bridge::queue_vector_shape_scene_command(&scene, &service, &command);
    assert_eq!(entity.raw(), 0);
    assert_eq!(service.commands().len(), 1);
    assert_eq!(scene.entity_names(), vec!["test-shape".to_owned()]);
}

#[test]
fn queues_vector_shape_scene_command_with_material_and_render_contributions() {
    let scene = SceneService::default();
    let service = VectorSceneService::default();
    let mut command = VectorShape2dSceneCommand::new(
        "test-mod",
        "glass-vector",
        VectorShapeKind2dSceneCommand::Circle {
            radius: 24.0,
            segments: 16,
        },
        VectorStyle2dSceneCommand {
            stroke_color: ColorRgba::WHITE,
            stroke_width: 2.0,
            fill_color: Some(ColorRgba::WHITE),
        },
    );
    command.render_contributions.roles.insert(roles::MATERIAL_MASK.to_owned(), true);
    command.render_contributions.roles.insert(roles::OPTICS_REFRACT.to_owned(), true);
    command.material = Some(Material2dSceneCommand {
        optical: Material2dOpticalSceneCommand {
            mode: Material2dOpticalModeSceneCommand::Refractive,
            transmission: 0.35,
            refraction_px: 5.0,
            distortion: 0.1,
            dispersion: 0.05,
            roughness: 0.2,
            edge_boost: 0.0,
        },
        lighting: Material2dLightingSceneCommand {
            receives_light: false,
            response: 0.0,
        },
        camera_response: Material2dCameraResponseSceneCommand {
            highlight: 0.0,
            bloom_source: false,
            rain_glass_affects: false,
        },
    });

    crate::scene_bridge::queue_vector_shape_scene_command(&scene, &service, &command);

    let commands = service.commands();
    let draw = commands.first().expect("vector command should be queued");
    let material = draw.material.expect("material should be carried to runtime");
    assert!(material.is_refractive());
    assert!(draw.render_contributions.enabled_or(roles::WORLD_COLOR, false));
    assert!(draw.render_contributions.enabled_or(roles::MATERIAL_MASK, false));
    assert!(draw.render_contributions.enabled_or(roles::OPTICS_REFRACT, false));
    assert!(!draw.render_contributions.enabled_or(roles::TRANSMISSION_SOURCE, true));
}

#[test]
fn can_handle_vector_scene_command() {
    let command = SceneCommand::QueueVectorShape2d {
        command: VectorShape2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "test-shape".to_owned(),
            render_layer: "world".to_owned(),
            kind: VectorShapeKind2dSceneCommand::Polyline {
                points: vec![Vec2::new(0.0, 12.0), Vec2::new(-8.0, -8.0)],
                closed: false,
            },
            style: VectorStyle2dSceneCommand {
                stroke_color: ColorRgba::WHITE,
                stroke_width: 1.0,
                fill_color: None,
            },
            z_index: 1.0,
            render_contributions: RenderContributions2dSceneCommand::default(),
            material: None,
            transform: Transform2::default(),
        },
    };

    assert!(crate::can_handle_vector_scene_command(&command));
}

#[test]
fn handles_vector_scene_command_and_publishes_event() {
    let scene_service = SceneService::default();
    let vector_scene_service = VectorSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let command = SceneCommand::QueueVectorShape2d {
        command: VectorShape2dSceneCommand {
            source_mod: "test-mod".to_owned(),
            entity_name: "test-shape".to_owned(),
            render_layer: "world".to_owned(),
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
            render_contributions: RenderContributions2dSceneCommand::default(),
            material: None,
            transform: Transform2::default(),
        },
    };

    let outcome = crate::handle_vector_scene_command(
        crate::VectorSceneCommandContext {
            scene_service: &scene_service,
            vector_scene_service: &vector_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect("vector scene command should be handled");

    assert_eq!(outcome.entity_name, "test-shape");
    assert_eq!(outcome.source_mod, "test-mod");
    assert_eq!(scene_service.entity_names(), vec!["test-shape".to_owned()]);
    assert_eq!(vector_scene_service.commands().len(), 1);

    let events = scene_event_queue.drain();
    assert_eq!(events.len(), 1);
    match &events[0] {
        SceneEvent::VectorQueued {
            entity_id,
            entity_name,
        } => {
            assert_eq!(*entity_id, 0);
            assert_eq!(entity_name, "test-shape");
        }
        other => panic!("expected vector queued event, got {other:?}"),
    }
}

#[test]
fn rejects_non_vector_scene_command() {
    let scene_service = SceneService::default();
    let vector_scene_service = VectorSceneService::default();
    let scene_event_queue = SceneEventQueue::default();
    let command = SceneCommand::ReloadActiveScene;

    let error = crate::handle_vector_scene_command(
        crate::VectorSceneCommandContext {
            scene_service: &scene_service,
            vector_scene_service: &vector_scene_service,
            scene_event_queue: &scene_event_queue,
        },
        command,
    )
    .expect_err("non-vector command should be rejected");

    match error {
        amigo_core::AmigoError::Message(message) => {
            assert!(message.contains("vector-2d cannot handle command"));
        }
        other => panic!("expected message error, got {other:?}"),
    }
}

#[test]
fn registers_vector_runtime_plugin() {
    let runtime = RuntimeBuilder::default()
        .with_service(RuntimeSceneCommandHandlerRegistry::new())
        .expect("scene command registry should register")
        .with_plugin(Vector2dPlugin)
        .expect("vector plugin should register")
        .build();
    assert!(
        runtime
            .resolve::<crate::service::VectorSceneService>()
            .is_some()
    );
}
