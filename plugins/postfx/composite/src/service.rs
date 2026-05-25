use std::collections::BTreeSet;
use std::sync::RwLock;

use crate::{
    LensDroplets2dCertificationReport, PostFx2dStack, PostFxBlur2d, PostFxDiagnostic2d,
    PostFxScope2d, ScopedPostFx2dStack, diagnose_post_fx_stacks,
};
use amigo_render_api::{PostFx2d, post_fx_blur};

#[derive(Debug, Default)]
pub struct PostFx2dService {
    default_blur: RwLock<PostFxBlur2d>,
    scoped_stacks: RwLock<Vec<ScopedPostFx2dStack>>,
    disabled_frame_effects: RwLock<BTreeSet<usize>>,
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
        PostFx2dStack::single(post_fx_blur(self.default_blur()))
    }

    pub fn set_scoped_stacks(&self, stacks: Vec<ScopedPostFx2dStack>) {
        *self
            .scoped_stacks
            .write()
            .expect("post-fx stacks lock should be writable") = stacks
            .into_iter()
            .map(ScopedPostFx2dStack::normalized)
            .collect();
        self.disabled_frame_effects
            .write()
            .expect("post-fx disabled effects lock should be writable")
            .clear();
    }

    pub fn scoped_stacks(&self) -> Vec<ScopedPostFx2dStack> {
        self.scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .clone()
    }

    pub fn diagnostics(&self) -> Vec<PostFxDiagnostic2d> {
        diagnose_post_fx_stacks(&self.scoped_stacks())
    }

    pub fn frame_stack(&self) -> Option<PostFx2dStack> {
        let disabled = self
            .disabled_frame_effects
            .read()
            .expect("post-fx disabled effects lock should be readable")
            .clone();
        self.scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(|stack| {
                let mut frame_stack = stack.as_frame_stack();
                frame_stack.effects = frame_stack
                    .effects
                    .into_iter()
                    .enumerate()
                    .filter_map(|(index, effect)| (!disabled.contains(&index)).then_some(effect))
                    .collect();
                frame_stack
            })
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

    pub fn frame_effects_raw(&self) -> Vec<PostFx2d> {
        let scoped = match self.scoped_stacks.read() {
            Ok(scoped) => scoped,
            Err(_) => return Vec::new(),
        };
        scoped
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .map(|stack| {
                stack
                    .effects
                    .iter()
                    .map(|instance| instance.effect.clone())
                    .collect()
            })
            .unwrap_or_default()
    }

    pub fn frame_effect_raw(&self, index: usize) -> Option<PostFx2d> {
        let scoped = self.scoped_stacks.read().ok()?;
        scoped
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .and_then(|stack| {
                stack
                    .effects
                    .get(index)
                    .map(|instance| instance.effect.clone())
            })
    }

    pub fn frame_effects(&self) -> Vec<PostFx2d> {
        self.frame_stack()
            .map(|stack| stack.effects)
            .unwrap_or_default()
    }

    pub fn frame_effect(&self, index: usize) -> Option<PostFx2d> {
        self.frame_effects().into_iter().nth(index)
    }

    pub fn frame_effect_enabled(&self, index: usize) -> bool {
        !self
            .disabled_frame_effects
            .read()
            .expect("post-fx disabled effects lock should be readable")
            .contains(&index)
    }

    pub fn set_frame_effect_enabled(&self, index: usize, enabled: bool) -> bool {
        let has_effect = self
            .scoped_stacks
            .read()
            .expect("post-fx stacks lock should be readable")
            .iter()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
            .is_some_and(|stack| index < stack.effects.len());
        if !has_effect {
            return false;
        }

        let mut disabled = self
            .disabled_frame_effects
            .write()
            .expect("post-fx disabled effects lock should be writable");
        if enabled {
            disabled.remove(&index);
        } else {
            disabled.insert(index);
        }
        true
    }

    pub fn update_frame_effect<F>(&self, index: usize, update: F) -> bool
    where
        F: FnOnce(PostFx2d) -> Option<PostFx2d>,
    {
        let mut scoped = match self.scoped_stacks.write() {
            Ok(scoped) => scoped,
            Err(_) => return false,
        };

        let Some(stack) = scoped
            .iter_mut()
            .find(|stack| matches!(stack.scope, PostFxScope2d::Frame))
        else {
            return false;
        };

        let Some(slot) = stack.effects.get_mut(index) else {
            return false;
        };

        let current = slot.effect.clone();
        let Some(updated) = update(current) else {
            return false;
        };

        slot.effect = updated.normalized();
        true
    }

    pub fn clear_scoped_stacks(&self) {
        self.scoped_stacks
            .write()
            .expect("post-fx stacks lock should be writable")
            .clear();
        self.disabled_frame_effects
            .write()
            .expect("post-fx disabled effects lock should be writable")
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
