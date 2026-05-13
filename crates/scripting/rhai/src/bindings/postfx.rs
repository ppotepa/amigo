use std::sync::Arc;

use amigo_2d_post_fx::{PostFx2d, PostFx2dService};

#[derive(Clone)]
pub struct PostFxApi {
    pub(crate) post_fx: Option<Arc<PostFx2dService>>,
}

impl PostFxApi {
    pub fn count(&mut self) -> rhai::INT {
        self.post_fx
            .as_ref()
            .map(|service| service.scene_effect_count() as rhai::INT)
            .unwrap_or(0)
    }

    pub fn list(&mut self) -> rhai::Array {
        self.post_fx
            .as_ref()
            .map(|service| {
                service
                    .scene_effects()
                    .into_iter()
                    .enumerate()
                    .map(|(index, effect)| item_map(index, effect))
                    .collect::<rhai::Array>()
            })
            .unwrap_or_default()
    }

    pub fn item(&mut self, index: rhai::INT) -> PostFxItemRef {
        PostFxItemRef {
            post_fx: self.post_fx.clone(),
            index: index.max(0) as usize,
        }
    }
}

#[derive(Clone)]
pub struct PostFxItemRef {
    post_fx: Option<Arc<PostFx2dService>>,
    index: usize,
}

impl PostFxItemRef {
    pub fn exists(&mut self) -> bool {
        self.effect().is_some()
    }

    pub fn index(&mut self) -> rhai::INT {
        self.index as rhai::INT
    }

    pub fn name(&mut self) -> String {
        self.effect()
            .map(|effect| effect.kind().to_owned())
            .unwrap_or_default()
    }

    pub fn active(&mut self) -> bool {
        self.effect()
            .map(|effect| effect.is_active())
            .unwrap_or(false)
    }

    fn effect(&self) -> Option<PostFx2d> {
        self.post_fx
            .as_ref()
            .and_then(|service| service.scene_effect(self.index))
    }
}

fn item_map(index: usize, effect: PostFx2d) -> rhai::Dynamic {
    let mut map = rhai::Map::new();
    map.insert("index".into(), (index as rhai::INT).into());
    map.insert("name".into(), effect.clone().kind().to_owned().into());
    map.insert("active".into(), effect.is_active().into());
    map.into()
}
