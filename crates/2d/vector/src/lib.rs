use std::sync::Mutex;

use amigo_math::{ColorRgba, Transform2, Vec2};
use amigo_runtime::{RuntimePlugin, ServiceRegistry};
use amigo_scene::{
    SceneEntityId, SceneService, VectorShape2dSceneCommand, VectorShapeKind2dSceneCommand,
    VectorStyle2dSceneCommand,
};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RadialJitterPolygon {
    pub vertices: usize,
    pub radius: f32,
    pub jitter: f32,
    pub seed: u64,
}

impl RadialJitterPolygon {
    pub fn new(vertices: usize, radius: f32, jitter: f32, seed: u64) -> Self {
        Self {
            vertices,
            radius,
            jitter,
            seed,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProceduralVectorError {
    TooFewVertices,
    InvalidRadius,
}

pub fn radial_jitter_polygon_points(
    config: RadialJitterPolygon,
) -> Result<Vec<Vec2>, ProceduralVectorError> {
    if config.vertices < 3 {
        return Err(ProceduralVectorError::TooFewVertices);
    }

    if !config.radius.is_finite() || config.radius < 0.0 {
        return Err(ProceduralVectorError::InvalidRadius);
    }

    let jitter = if config.jitter.is_finite() {
        config.jitter.clamp(0.0, 1.0)
    } else {
        0.0
    };
    let mut rng = DeterministicRng::new(config.seed);
    let angle_step = std::f32::consts::TAU / config.vertices as f32;
    let mut points = Vec::with_capacity(config.vertices);

    for index in 0..config.vertices {
        let angle = index as f32 * angle_step;
        let radius_scale = 1.0 + rng.next_signed_unit() * jitter;
        let radius = config.radius * radius_scale.max(0.0);
        points.push(Vec2::new(angle.cos() * radius, angle.sin() * radius));
    }

    Ok(points)
}

#[derive(Debug, Clone, Copy)]
struct DeterministicRng {
    state: u64,
}

impl DeterministicRng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u32(&mut self) -> u32 {
        self.state = self
            .state
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        (self.state >> 32) as u32
    }

    fn next_signed_unit(&mut self) -> f32 {
        let unit = self.next_u32() as f32 / u32::MAX as f32;
        unit * 2.0 - 1.0
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum VectorShapeKind2d {
    Polyline { points: Vec<Vec2>, closed: bool },
    Polygon { points: Vec<Vec2> },
    Circle { radius: f32, segments: u32 },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VectorStyle2d {
    pub stroke_color: ColorRgba,
    pub stroke_width: f32,
    pub fill_color: Option<ColorRgba>,
}

impl Default for VectorStyle2d {
    fn default() -> Self {
        Self {
            stroke_color: ColorRgba::WHITE,
            stroke_width: 1.0,
            fill_color: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2d {
    pub kind: VectorShapeKind2d,
    pub style: VectorStyle2d,
}

#[derive(Debug, Clone, PartialEq)]
pub struct VectorShape2dDrawCommand {
    pub entity_id: SceneEntityId,
    pub entity_name: String,
    pub shape: VectorShape2d,
    pub z_index: f32,
    pub transform: Transform2,
}

#[derive(Debug, Default)]
pub struct VectorSceneService {
    commands: Mutex<Vec<VectorShape2dDrawCommand>>,
}

impl VectorSceneService {
    pub fn queue(&self, command: VectorShape2dDrawCommand) {
        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        commands.retain(|existing| existing.entity_name != command.entity_name);
        commands.push(command);
    }

    pub fn clear(&self) {
        self.commands
            .lock()
            .expect("vector scene service mutex should not be poisoned")
            .clear();
    }

    pub fn commands(&self) -> Vec<VectorShape2dDrawCommand> {
        self.commands
            .lock()
            .expect("vector scene service mutex should not be poisoned")
            .clone()
    }

    pub fn set_polygon_points(&self, entity_name: &str, points: Vec<Vec2>) -> bool {
        if points.len() < 3 {
            return false;
        }

        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };

        command.shape.kind = VectorShapeKind2d::Polygon { points };
        true
    }

    pub fn set_radial_jitter_polygon(
        &self,
        entity_name: &str,
        config: RadialJitterPolygon,
    ) -> bool {
        let Ok(points) = radial_jitter_polygon_points(config) else {
            return false;
        };

        self.set_polygon_points(entity_name, points)
    }

    pub fn set_polyline_points(&self, entity_name: &str, points: Vec<Vec2>, closed: bool) -> bool {
        if points.len() < 2 {
            return false;
        }

        let mut commands = self
            .commands
            .lock()
            .expect("vector scene service mutex should not be poisoned");
        let Some(command) = commands
            .iter_mut()
            .find(|command| command.entity_name == entity_name)
        else {
            return false;
        };

        command.shape.kind = VectorShapeKind2d::Polyline { points, closed };
        true
    }

    pub fn entity_names(&self) -> Vec<String> {
        self.commands()
            .into_iter()
            .map(|command| command.entity_name)
            .collect()
    }
}

#[derive(Debug, Clone)]
pub struct VectorDomainInfo {
    pub crate_name: &'static str,
    pub capability: &'static str,
}

pub struct Vector2dPlugin;

impl RuntimePlugin for Vector2dPlugin {
    fn name(&self) -> &'static str {
        "amigo-2d-vector"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(VectorSceneService::default())?;
        registry.register(VectorDomainInfo {
            crate_name: "amigo-2d-vector",
            capability: "vector_2d",
        })
    }
}

pub fn queue_vector_shape_scene_command(
    scene_service: &SceneService,
    vector_scene_service: &VectorSceneService,
    command: &VectorShape2dSceneCommand,
) -> SceneEntityId {
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

#[cfg(test)]
mod tests {
    use super::{
        ProceduralVectorError, RadialJitterPolygon, Vector2dPlugin, VectorSceneService,
        VectorShape2d, VectorShape2dDrawCommand, VectorShapeKind2d, VectorStyle2d,
        queue_vector_shape_scene_command, radial_jitter_polygon_points,
    };
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
            !service
                .set_radial_jitter_polygon("missing", RadialJitterPolygon::new(6, 9.0, 0.25, 99),)
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

        let entity = queue_vector_shape_scene_command(&scene, &service, &command);
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
        assert!(runtime.resolve::<VectorSceneService>().is_some());
    }
}
