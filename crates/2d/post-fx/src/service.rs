use std::sync::RwLock;

use crate::{LensDroplets2dCertificationReport, PostFx2d, PostFx2dStack, PostFxBlur2d};

#[derive(Debug, Default)]
pub struct PostFx2dService {
    default_blur: RwLock<PostFxBlur2d>,
    scene_stack: RwLock<PostFx2dStack>,
    certification_reports: RwLock<Vec<LensDroplets2dCertificationReport>>,
    renderer_mode: RwLock<String>,
}

impl PostFx2dService {
    pub fn default_blur(&self) -> PostFxBlur2d {
        *self
            .default_blur
            .read()
            .expect("post-fx default blur lock should be readable")
    }

    pub fn set_default_blur(&self, blur: PostFxBlur2d) {
        *self
            .default_blur
            .write()
            .expect("post-fx default blur lock should be writable") = blur.normalized();
    }

    pub fn default_blur_stack(&self) -> PostFx2dStack {
        PostFx2dStack::single(PostFx2d::Blur(self.default_blur()))
    }

    pub fn set_scene_stack(&self, stack: PostFx2dStack) {
        *self
            .scene_stack
            .write()
            .expect("post-fx scene stack lock should be writable") = stack.normalized();
    }

    pub fn scene_stack(&self) -> PostFx2dStack {
        self.scene_stack
            .read()
            .expect("post-fx scene stack lock should be readable")
            .clone()
    }

    pub fn set_lens_certification_reports(&self, reports: Vec<LensDroplets2dCertificationReport>) {
        *self
            .certification_reports
            .write()
            .expect("post-fx cert lock should be writable") = reports;
    }

    pub fn lens_certification_reports(&self) -> Vec<LensDroplets2dCertificationReport> {
        self.certification_reports
            .read()
            .expect("post-fx cert lock should be readable")
            .clone()
    }

    pub fn set_renderer_mode(&self, mode: impl Into<String>) {
        *self
            .renderer_mode
            .write()
            .expect("post-fx renderer mode lock should be writable") = mode.into();
    }

    pub fn renderer_mode(&self) -> String {
        self.renderer_mode
            .read()
            .expect("post-fx renderer mode lock should be readable")
            .clone()
    }

    pub fn supports_lens_droplets_overlay(&self) -> bool {
        true
    }

    pub fn supports_lens_droplets_blur(&self) -> bool {
        false
    }

    pub fn supports_world_offscreen_post_fx(&self) -> bool {
        true
    }
}
