use super::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct PostFx2dStack {
    pub effects: Vec<PostFx2d>,
}

impl PostFx2dStack {
    pub fn single(effect: PostFx2d) -> Self {
        Self {
            effects: vec![effect],
        }
    }

    pub fn is_empty(&self) -> bool {
        self.effects.is_empty()
    }

    pub fn normalized(self) -> Self {
        Self {
            effects: self
                .effects
                .into_iter()
                .map(PostFx2d::normalized)
                .filter(PostFx2d::is_active)
                .collect(),
        }
    }
}
