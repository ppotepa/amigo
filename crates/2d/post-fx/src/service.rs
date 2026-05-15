use std::sync::RwLock;

use crate::{
    LensDroplets2dCertificationReport, PostFx2d, PostFx2dStack, PostFxBlur2d, PostFxScope2d,
    ScopedPostFx2dStack,
};

#[derive(Debug, Default)]
pub struct PostFx2dService {
    default_blur: RwLock<PostFxBlur2d>,
    scoped_stacks: RwLock<Vec<ScopedPostFx2dStack>>,
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

    pub fn set_scoped_stacks(&self, stacks: Vec<ScopedPostFx2dStack>) {
        *self
            .scoped_stacks
            .write()
            .expect("post-fx stacks lock should be writable") = stacks
            .into_iter()
            .map(ScopedPostFx2dStack::normalized)
            .collect();
    }

    pub fn scoped_stacks(&self) -> Vec<ScopedPostFx2dStack> {
        self.scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .clone()
    }

    pub fn frame_stack(&self) -> Option<PostFx2dStack> {
        self.scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(ScopedPostFx2dStack::as_frame_stack)
    }

    pub fn frame_effect_count(&self) -> usize {
        self.scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .iter()
            .filter(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(|stack| stack.effects.len())
            .sum()
    }

    pub fn frame_effects(&self) -> Vec<PostFx2d> {
        self.frame_stack()
            .map(|stack| stack.effects)
            .unwrap_or_default()
    }

    pub fn frame_effect(&self, index: usize) -> Option<PostFx2d> {
        self.frame_effects().into_iter().nth(index)
    }

    pub fn clear_scoped_stacks(&self) {
        self.scoped_stacks
            .write()
            .expect("post-fx stacks lock should be writable")
            .clear();
    }

    pub fn push_frame_effect(&self, effect: PostFx2d) {
        let mut stacks = self
            .scoped_stacks
            .write()
            .expect("post-fx stacks lock should be writable");

        if let Some(frame_stack) = stacks
            .iter_mut()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
        {
            frame_stack.push_frame_effect(effect);
            return;
        }

        stacks.push(ScopedPostFx2dStack::from_frame_stack(PostFx2dStack {
            effects: vec![effect],
        }));
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
