use std::collections::BTreeMap;
use std::sync::RwLock;

use crate::{
    SceneComponentPayload, SceneGraphDiagnostic, SceneGraphNodeId, SceneReferenceEdge,
    SceneReferenceKind, SceneReferenceTargetKind, SemanticSceneGraph,
};

pub trait PluginComponentGraphProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;
    fn component_type(&self) -> &'static str;
    fn primary_render_layer(&self, payload: &dyn SceneComponentPayload) -> Option<String>;
    fn add_references(&self, ctx: &mut PluginComponentGraphContext<'_>);
}

pub struct PluginComponentGraphContext<'a> {
    pub payload: &'a dyn SceneComponentPayload,
    pub component_node: SceneGraphNodeId,
    pub graph: &'a mut SemanticSceneGraph,
    pub draw_layers: &'a BTreeMap<String, SceneGraphNodeId>,
    pub scene_objects: &'a BTreeMap<String, SceneGraphNodeId>,
}

impl PluginComponentGraphContext<'_> {
    pub fn add_draw_layer_ref(&mut self, port: &str, raw_target: &str) {
        if let Some(target) = self.draw_layers.get(raw_target) {
            self.graph.add_reference(SceneReferenceEdge::new(
                self.component_node.clone(),
                port,
                SceneReferenceKind::RendersIntoDrawLayer,
                SceneReferenceTargetKind::DrawLayer,
                raw_target,
                true,
                Some(target.clone()),
            ));
            return;
        }

        self.graph.add_reference(SceneReferenceEdge::new(
            self.component_node.clone(),
            port,
            SceneReferenceKind::RendersIntoDrawLayer,
            SceneReferenceTargetKind::DrawLayer,
            raw_target,
            true,
            None,
        ));
        self.graph.add_diagnostic(SceneGraphDiagnostic::error(
            "missing_draw_layer_ref",
            format!("missing `{raw_target}` for `{}`", self.component_node),
            Some(self.component_node.clone()),
        ));
    }

    pub fn add_external_ref(
        &mut self,
        port: &str,
        kind: SceneReferenceKind,
        target_kind: SceneReferenceTargetKind,
        raw_target: &str,
        required: bool,
    ) {
        self.graph.add_reference(SceneReferenceEdge::new(
            self.component_node.clone(),
            port,
            kind,
            target_kind,
            raw_target,
            required,
            None,
        ));
    }

    pub fn add_scene_object_ref(
        &mut self,
        port: &str,
        kind: SceneReferenceKind,
        raw_target: &str,
        missing_code: &'static str,
    ) {
        if let Some(target) = self.scene_objects.get(raw_target) {
            self.graph.add_reference(SceneReferenceEdge::new(
                self.component_node.clone(),
                port,
                kind,
                SceneReferenceTargetKind::SceneObject,
                raw_target,
                true,
                Some(target.clone()),
            ));
            return;
        }

        self.graph.add_reference(SceneReferenceEdge::new(
            self.component_node.clone(),
            port,
            kind,
            SceneReferenceTargetKind::SceneObject,
            raw_target,
            true,
            None,
        ));
        self.graph.add_diagnostic(SceneGraphDiagnostic::error(
            missing_code,
            format!("missing scene object reference `{raw_target}` at `{port}`"),
            Some(self.component_node.clone()),
        ));
    }
}

#[derive(Default)]
pub struct ComponentGraphProviderRegistry {
    providers: RwLock<BTreeMap<String, Box<dyn PluginComponentGraphProvider>>>,
}

impl ComponentGraphProviderRegistry {
    pub fn register<P>(&self, provider: P)
    where
        P: PluginComponentGraphProvider + 'static,
    {
        self.providers
            .write()
            .expect("component graph provider registry poisoned")
            .insert(provider.component_type().to_owned(), Box::new(provider));
    }

    pub fn with_provider<R>(
        &self,
        component_type: &str,
        f: impl FnOnce(&dyn PluginComponentGraphProvider) -> R,
    ) -> Option<R> {
        let providers = self
            .providers
            .read()
            .expect("component graph provider registry poisoned");
        providers
            .get(component_type)
            .map(|provider| f(provider.as_ref()))
    }

    pub fn provider_ids(&self) -> Vec<&'static str> {
        self.providers
            .read()
            .expect("component graph provider registry poisoned")
            .values()
            .map(|provider| provider.provider_id())
            .collect()
    }
}
