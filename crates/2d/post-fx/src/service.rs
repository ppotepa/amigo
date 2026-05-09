use std::sync::RwLock;

use crate::{PostFx2d, PostFx2dStack, PostFxBlur2d};

#[derive(Debug, Default)]
pub struct PostFx2dService {
    default_blur: RwLock<PostFxBlur2d>,
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
}
