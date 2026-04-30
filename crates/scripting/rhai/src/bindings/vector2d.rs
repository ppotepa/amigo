use std::sync::Arc;

use amigo_2d_vector::{RadialJitterPolygon, VectorSceneService};
use amigo_math::Vec2;
use rhai::{Array, Dynamic, FLOAT, INT, Map};

#[derive(Clone)]
pub struct Vector2dApi {
    pub(crate) vector_scene: Option<Arc<VectorSceneService>>,
}

impl Vector2dApi {
    pub fn set_polygon(&mut self, entity_name: &str, points: Array) -> bool {
        set_polygon(self.vector_scene.as_ref(), entity_name, points)
    }

    pub fn set_polyline(&mut self, entity_name: &str, points: Array, closed: bool) -> bool {
        set_polyline(self.vector_scene.as_ref(), entity_name, points, closed)
    }

    pub fn set_radial_jitter_polygon(
        &mut self,
        entity_name: &str,
        vertices: INT,
        radius: FLOAT,
        jitter: FLOAT,
        seed: INT,
    ) -> bool {
        set_radial_jitter_polygon(
            self.vector_scene.as_ref(),
            entity_name,
            vertices,
            radius,
            jitter,
            seed,
        )
    }
}

pub fn set_polygon(
    vector_scene: Option<&Arc<VectorSceneService>>,
    entity_name: &str,
    points: Array,
) -> bool {
    if entity_name.trim().is_empty() {
        return false;
    }

    let Some(parsed) = parse_points(points) else {
        return false;
    };

    vector_scene
        .map(|scene| scene.set_polygon_points(entity_name, parsed))
        .unwrap_or(false)
}

pub fn set_polyline(
    vector_scene: Option<&Arc<VectorSceneService>>,
    entity_name: &str,
    points: Array,
    closed: bool,
) -> bool {
    if entity_name.trim().is_empty() {
        return false;
    }

    let Some(parsed) = parse_points(points) else {
        return false;
    };

    vector_scene
        .map(|scene| scene.set_polyline_points(entity_name, parsed, closed))
        .unwrap_or(false)
}

pub fn set_radial_jitter_polygon(
    vector_scene: Option<&Arc<VectorSceneService>>,
    entity_name: &str,
    vertices: INT,
    radius: FLOAT,
    jitter: FLOAT,
    seed: INT,
) -> bool {
    if entity_name.trim().is_empty() || vertices < 0 || seed < 0 {
        return false;
    }

    vector_scene
        .map(|scene| {
            scene.set_radial_jitter_polygon(
                entity_name,
                RadialJitterPolygon::new(
                    vertices as usize,
                    radius as f32,
                    jitter as f32,
                    seed as u64,
                ),
            )
        })
        .unwrap_or(false)
}

fn parse_points(values: Array) -> Option<Vec<Vec2>> {
    if let Some(flat) = parse_flat_points(&values) {
        return Some(flat);
    }

    let mut points = Vec::with_capacity(values.len());
    for value in values {
        points.push(parse_point(value)?);
    }

    if points.is_empty() {
        return None;
    }

    Some(points)
}

fn parse_flat_points(values: &Array) -> Option<Vec<Vec2>> {
    if values.len() < 4 || values.len() % 2 != 0 {
        return None;
    }

    let mut scalars = Vec::with_capacity(values.len());
    for value in values {
        scalars.push(dynamic_to_f32(value.clone())?);
    }

    let mut points = Vec::with_capacity(scalars.len() / 2);
    let mut index = 0;
    while index < scalars.len() {
        points.push(Vec2::new(scalars[index], scalars[index + 1]));
        index += 2;
    }

    Some(points)
}

fn parse_point(value: Dynamic) -> Option<Vec2> {
    if let Some(values) = value.clone().try_cast::<Array>() {
        if values.len() < 2 {
            return None;
        }
        let x = dynamic_to_f32(values[0].clone())?;
        let y = dynamic_to_f32(values[1].clone())?;
        return Some(Vec2::new(x, y));
    }

    if let Some(map) = value.try_cast::<Map>() {
        let x = map
            .get("x")
            .and_then(|value| dynamic_to_f32(value.clone()))?;
        let y = map
            .get("y")
            .and_then(|value| dynamic_to_f32(value.clone()))?;
        return Some(Vec2::new(x, y));
    }

    None
}

fn dynamic_to_f32(value: Dynamic) -> Option<f32> {
    if let Some(number) = value.clone().try_cast::<FLOAT>() {
        return Some(number as f32);
    }

    if let Some(number) = value.try_cast::<INT>() {
        return Some(number as f32);
    }

    None
}
