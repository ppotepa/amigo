use amigo_math::{ColorRgba, Vec2};

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

#[derive(Debug, Clone, PartialEq)]
pub enum VectorShapeKind2d {
    Polyline {
        points: Vec<Vec2>,
        closed: bool,
    },
    Polygon {
        points: Vec<Vec2>,
    },
    Circle {
        radius: f32,
        segments: u32,
    },
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
    pub entity_id: amigo_scene::SceneEntityId,
    pub entity_name: String,
    pub shape: VectorShape2d,
    pub z_index: f32,
    pub transform: amigo_math::Transform2,
}

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
