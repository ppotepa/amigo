use std::sync::RwLock;

use crate::{
    SceneCommand, SceneComponentDocument, SceneDocument, SceneDocumentResult, SceneEntityDocument,
};

pub struct ComponentHydrationContext<'a> {
    pub source_mod: &'a str,
    pub document: &'a SceneDocument,
    pub entity: &'a SceneEntityDocument,
    pub entity_name: &'a str,
    pub component_index: usize,
    pub component: &'a SceneComponentDocument,
    pub commands: &'a mut Vec<SceneCommand>,
}

pub trait ComponentHydrator: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn can_hydrate(&self, component: &SceneComponentDocument) -> bool;

    fn hydrate(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<()>;
}

#[derive(Default)]
pub struct ComponentHydratorRegistry {
    hydrators: RwLock<Vec<Box<dyn ComponentHydrator>>>,
}

impl ComponentHydratorRegistry {
    pub fn register<H>(&self, hydrator: H)
    where
        H: ComponentHydrator + 'static,
    {
        self.hydrators
            .write()
            .expect("component hydrator registry poisoned")
            .push(Box::new(hydrator));
    }

    pub fn hydrate_first(
        &self,
        ctx: ComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<bool> {
        let hydrators = self
            .hydrators
            .read()
            .expect("component hydrator registry poisoned");
        let Some(hydrator) = hydrators
            .iter()
            .find(|hydrator| hydrator.can_hydrate(ctx.component))
        else {
            return Ok(false);
        };
        hydrator.hydrate(ctx)?;
        Ok(true)
    }

    pub fn provider_ids(&self) -> Vec<&'static str> {
        self.hydrators
            .read()
            .expect("component hydrator registry poisoned")
            .iter()
            .map(|hydrator| hydrator.provider_id())
            .collect()
    }
}
