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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct OrthographicCamera {
    pub position: Vec3,
    pub forward: Vec3,
    pub up: Vec3,
    /// Visible vertical world-space span of the virtual paper camera.
    pub vertical_span: f32,
    pub near: f32,
    pub aspect: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NprCamera {
    Perspective(PerspectiveCamera),
    Orthographic(OrthographicCamera),
}

impl From<PerspectiveCamera> for NprCamera {
    fn from(camera: PerspectiveCamera) -> Self {
        Self::Perspective(camera)
    }
}

impl NprCamera {
    pub fn forward(self) -> Vec3 {
        match self {
            Self::Perspective(camera) => camera.forward,
            Self::Orthographic(camera) => camera.forward,
        }
    }

    pub fn up(self) -> Vec3 {
        match self {
            Self::Perspective(camera) => camera.up,
            Self::Orthographic(camera) => camera.up,
        }
    }

    pub fn project(self, point: Vec3, viewport: Vec2) -> Option<ProjectedPoint> {
        match self {
            Self::Perspective(camera) => camera.project(point, viewport),
            Self::Orthographic(camera) => camera.project(point, viewport),
        }
    }

    pub fn project_segment(self, a: Vec3, b: Vec3, viewport: Vec2) -> Option<(ProjectedPoint, ProjectedPoint)> {
        match self {
            Self::Perspective(camera) => camera.project_segment(a, b, viewport),
            Self::Orthographic(camera) => camera.project_segment(a, b, viewport),
        }
    }
}

impl PerspectiveCamera {
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
        if depth < self.near {
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

impl OrthographicCamera {
    pub fn default_for_viewport(aspect: f32) -> Self {
        Self {
            position: Vec3::new(0.0, 0.0, 5.0),
            forward: Vec3::NEG_Z,
            up: Vec3::Y,
            vertical_span: 4.0,
            near: 0.05,
            aspect,
        }
    }

    pub fn project(&self, point: Vec3, viewport: Vec2) -> Option<ProjectedPoint> {
        let forward = self.forward.normalize_or_zero();
        let right = forward.cross(self.up).normalize_or_zero();
        let up = right.cross(forward).normalize_or_zero();
        let relative = point - self.position;
        let depth = relative.dot(forward);
        if depth < self.near {
            return None;
        }
        let half_height = (self.vertical_span * 0.5).max(f32::EPSILON);
        let half_width = (half_height * self.aspect).max(f32::EPSILON);
        let ndc = Vec2::new(relative.dot(right) / half_width, relative.dot(up) / half_height);
        Some(ProjectedPoint {
            screen: Vec2::new(
                (ndc.x * 0.5 + 0.5) * viewport.x,
                (1.0 - (ndc.y * 0.5 + 0.5)) * viewport.y,
            ),
            depth,
        })
    }

    pub fn project_segment(&self, a: Vec3, b: Vec3, viewport: Vec2) -> Option<(ProjectedPoint, ProjectedPoint)> {
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
    fn orthographic_projection_is_independent_of_depth() {
        let camera = OrthographicCamera::default_for_viewport(1.0);
        let viewport = Vec2::splat(256.0);
        let near = camera.project(Vec3::new(1.0, 0.0, 0.0), viewport).unwrap();
        let far = camera.project(Vec3::new(1.0, 0.0, -2.0), viewport).unwrap();
        assert_eq!(near.screen, far.screen);
        assert!(far.depth > near.depth);
    }
}
