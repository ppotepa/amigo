use amigo_2d_post_fx::{PostFx2dId, PostFxHost2dId};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) struct PostFxRuntimeKey {
    pub host_id: PostFxHost2dId,
    pub effect_id: PostFx2dId,
}

impl PostFxRuntimeKey {
    pub(crate) fn new(host_id: &PostFxHost2dId, effect_id: &PostFx2dId) -> Self {
        Self {
            host_id: host_id.clone(),
            effect_id: effect_id.clone(),
        }
    }
}
