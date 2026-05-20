use std::sync::RwLock;

use amigo_editor_api::{
    EditorCommandProvider, EditorRuntimeApplyOutcome, EditorRuntimeApplyProvider,
    EditorRuntimeApplyRequest,
};
use amigo_runtime::Runtime;

#[derive(Default)]
pub struct IngameEditorProviderRegistry {
    providers: RwLock<Vec<Box<dyn EditorCommandProvider>>>,
}

impl IngameEditorProviderRegistry {
    pub fn register<P>(&self, provider: P)
    where
        P: EditorCommandProvider + 'static,
    {
        self.providers
            .write()
            .expect("editor provider registry poisoned")
            .push(Box::new(provider));
    }
}

#[derive(Default)]
pub struct IngameEditorRuntimeApplyProviderRegistry {
    providers: RwLock<Vec<Box<dyn EditorRuntimeApplyProvider>>>,
}

impl IngameEditorRuntimeApplyProviderRegistry {
    pub fn register<P>(&self, provider: P)
    where
        P: EditorRuntimeApplyProvider + 'static,
    {
        self.providers
            .write()
            .expect("editor runtime apply provider registry poisoned")
            .push(Box::new(provider));
    }

    pub fn apply_first(
        &self,
        runtime: &Runtime,
        request: EditorRuntimeApplyRequest,
    ) -> amigo_core::AmigoResult<Option<EditorRuntimeApplyOutcome>> {
        let providers = self
            .providers
            .read()
            .expect("editor runtime apply provider registry poisoned");
        for provider in providers.iter() {
            if provider.can_apply(&request) {
                return provider.apply(runtime, request).map(Some);
            }
        }
        Ok(None)
    }
}
