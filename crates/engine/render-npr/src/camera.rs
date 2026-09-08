use glam::{Vec2, Vec3};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PerspectiveCamera {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    pub vertical_fov: f32,
    pub near: f32,
    pub aspect: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProjectedPoint {
    pub screen: Vec2,
    pub depth: f32,
}

impl PerspectiveCamera {
    pub fn normalized_depth(&self, distance: f32) -> f32 {
        (1.0 - self.near / distance.max(self.near)).clamp(0.0, 1.0)
    }
    pub fn clip_triangle(&self, points: [Vec3; 3]) -> Vec<[Vec3; 3]> {
        let mut polygon = Vec::new();
        for i in 0..3 {
            let a = points[i];
            let b = points[(i + 1) % 3];
            let da = (a - self.position).dot(self.forward.normalize_or_zero()) - self.near;
            let db = (b - self.position).dot(self.forward.normalize_or_zero()) - self.near;
            if da >= 0.0 {
                polygon.push(a);
            }
            if (da >= 0.0) != (db >= 0.0) {
                polygon.push(a + (b - a) * (da / (da - db)));
            }
        }
        (1..polygon.len().saturating_sub(1))
            .map(|i| [polygon[0], polygon[i], polygon[i + 1]])
            .collect()
    }
    pub fn cube_default(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            vertical_fov: 45.0_f32.to_radians(),
            near: 0.05,
            aspect,
        }
    }
    pub fn project(&self, point: Vec3, viewport: Vec2) -> Option<ProjectedPoint> {
        let f = self.forward.normalize_or_zero();
        let right = f.cross(self.up).normalize_or_zero();
        let up = right.cross(f).normalize_or_zero();
        let relative = point - self.position;
        let depth = relative.dot(f);
        if depth < self.near * (1.0 - 1e-5) {
            return None;
        }
        let tan_half = (self.vertical_fov * 0.5).tan();
        let ndc = Vec2::new(
            relative.dot(right) / (depth * tan_half * self.aspect),
            relative.dot(up) / (depth * tan_half),
        );
        Some(ProjectedPoint {
            screen: Vec2::new(
                (ndc.x * 0.5 + 0.5) * viewport.x,
                (1.0 - (ndc.y * 0.5 + 0.5)) * viewport.y,
            ),
            depth,
        })
    }

    /// Builds a local- or world-space ray from a viewport pixel.
    ///
    /// The returned ray follows the same top-left screen convention as
    /// [`Self::project`]. Its space is the camera's space: callers that pick a
    /// local mesh pass a local camera, while callers that pick world geometry
    /// pass a world camera.
    pub fn ray_from_screen(&self, screen: Vec2, viewport: Vec2) -> Option<(Vec3, Vec3)> {
        if !screen.is_finite()
            || !viewport.is_finite()
            || viewport.x <= 0.0
            || viewport.y <= 0.0
            || !self.vertical_fov.is_finite()
            || !self.aspect.is_finite()
            || self.aspect <= 0.0
        {
            return None;
        }
        let forward = self.forward.normalize_or_zero();
        let right = forward.cross(self.up).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let tangent = (self.vertical_fov * 0.5).tan();
        if forward.length_squared() <= 1e-12
            || right.length_squared() <= 1e-12
            || up.length_squared() <= 1e-12
            || !tangent.is_finite()
            || tangent.abs() <= 1e-12
        {
            return None;
        }
        let ndc = Vec2::new(
            screen.x / viewport.x * 2.0 - 1.0,
            1.0 - screen.y / viewport.y * 2.0,
        );
        let direction =
            (forward + right * (ndc.x * tangent * self.aspect) + up * (ndc.y * tangent))
                .normalize_or_zero();
        (direction.length_squared() > 1e-12).then_some((self.position, direction))
    }

    pub fn project_segment(
        &self,
        a: Vec3,
        b: Vec3,
        viewport: Vec2,
    ) -> Option<(ProjectedPoint, ProjectedPoint)> {
        let da = (a - self.position).dot(self.forward.normalize_or_zero());
        let db = (b - self.position).dot(self.forward.normalize_or_zero());
        if da < self.near && db < self.near {
            return None;
        }
        let (a, b) = if da < self.near {
            (a + (b - a) * ((self.near - da) / (db - da)), b)
        } else if db < self.near {
            (a, a + (b - a) * ((self.near - da) / (db - da)))
        } else {
            (a, b)
        };
        Some((self.project(a, viewport)?, self.project(b, viewport)?))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ray_from_screen_inverts_the_camera_projection() {
        let camera = PerspectiveCamera::cube_default(16.0 / 9.0);
        let viewport = Vec2::new(1920.0, 1080.0);
        let point = Vec3::new(0.7, -0.25, -1.0);
        let projected = camera.project(point, viewport).unwrap();
        let (origin, direction) = camera.ray_from_screen(projected.screen, viewport).unwrap();
        let distance = (point - origin).dot(direction);
        assert!(distance > 0.0);
        assert!((origin + direction * distance - point).length() < 1e-5);
    }

    #[test]
    fn ray_from_screen_rejects_invalid_camera_or_viewport() {
        let camera = PerspectiveCamera::cube_default(1.0);
        assert!(camera.ray_from_screen(Vec2::ZERO, Vec2::ZERO).is_none());
        assert!(camera.ray_from_screen(Vec2::NAN, Vec2::ONE).is_none());
        assert!(PerspectiveCamera {
            forward: Vec3::ZERO,
            ..camera
        }
        .ray_from_screen(Vec2::ZERO, Vec2::ONE)
        .is_none());
    }
}
