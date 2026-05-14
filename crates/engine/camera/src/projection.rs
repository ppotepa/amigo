#[derive(Debug, Clone)]
pub enum CameraProjection {
    Perspective(PerspectiveProjection),
    Orthographic(OrthographicProjection),
}

#[derive(Debug, Clone)]
pub struct PerspectiveProjection {
    pub fov_y_degrees: f32,
    pub near: f32,
    pub far: f32,
}

#[derive(Debug, Clone)]
pub struct OrthographicProjection {
    pub width: f32,
    pub height: f32,
    pub near: f32,
    pub far: f32,
}
