use amigo_math::Vec2;

use crate::{WgpuOffscreenTarget, WgpuSurfaceState};

#[derive(Clone, Copy)]
pub(crate) struct Viewport {
    pub(crate) half_width: f32,
    pub(crate) half_height: f32,
    pub(crate) aspect: f32,
}

impl Viewport {
    pub(crate) fn from_surface(surface: &WgpuSurfaceState) -> Self {
        let width = surface.config.width.max(1) as f32;
        let height = surface.config.height.max(1) as f32;
        Self::from_dimensions(width, height)
    }

    pub(crate) fn from_offscreen(target: &WgpuOffscreenTarget) -> Self {
        Self::from_dimensions(target.width.max(1) as f32, target.height.max(1) as f32)
    }

    pub(crate) fn from_dimensions(width: f32, height: f32) -> Self {
        Self {
            half_width: width * 0.5,
            half_height: height * 0.5,
            aspect: width / height,
        }
    }

    pub(crate) fn size(&self) -> Vec2 {
        Vec2::new(self.half_width * 2.0, self.half_height * 2.0)
    }
}
