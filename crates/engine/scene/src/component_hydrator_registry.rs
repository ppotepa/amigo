use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::{
    SceneCommand, SceneComponentDocument, SceneComponentPayload, SceneDocument,
    SceneDocumentResult, SceneEntityDocument,
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

pub struct PluginComponentHydrationContext<'a> {
    pub source_mod: &'a str,
    pub document: &'a SceneDocument,
    pub entity: &'a SceneEntityDocument,
    pub entity_name: &'a str,
    pub component_index: usize,
    pub component_type: &'a str,
    pub payload: &'a dyn SceneComponentPayload,
    pub commands: &'a mut Vec<SceneCommand>,
}

pub trait PluginComponentHydrator: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn component_type(&self) -> &'static str;
    fn hydrate_plugin_payload(
        &self,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<()>;
}

#[derive(Default)]
pub struct ComponentHydratorRegistry {
    hydrators: RwLock<Vec<Box<dyn ComponentHydrator>>>,
    plugin_hydrators: RwLock<BTreeMap<String, Box<dyn PluginComponentHydrator>>>,
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

    pub fn hydrate_first(&self, ctx: ComponentHydrationContext<'_>) -> SceneDocumentResult<bool> {
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

    pub fn register_plugin<H>(&self, hydrator: H)
    where
        H: PluginComponentHydrator + 'static,
    {
        self.plugin_hydrators
            .write()
            .expect("component hydrator registry poisoned")
            .insert(hydrator.component_type().to_owned(), Box::new(hydrator));
    }

    pub fn hydrate_plugin_payload(
        &self,
        component_type: &str,
        ctx: PluginComponentHydrationContext<'_>,
    ) -> SceneDocumentResult<bool> {
        let hydrators = self
            .plugin_hydrators
            .read()
            .expect("component hydrator registry poisoned");
        let Some(hydrator) = hydrators.get(component_type) else {
            return Ok(false);
        };
        hydrator.hydrate_plugin_payload(ctx)?;
        Ok(true)
    }

    pub fn provider_ids(&self) -> Vec<&'static str> {
        let mut ids = self
            .hydrators
            .read()
            .expect("component hydrator registry poisoned")
            .iter()
            .map(|hydrator| hydrator.provider_id())
            .collect::<Vec<_>>();
        ids.extend(
            self.plugin_hydrators
                .read()
                .expect("component hydrator registry poisoned")
                .values()
                .map(|hydrator| hydrator.provider_id()),
        );
        ids.sort_unstable();
        ids.dedup();
        ids
    }
}
