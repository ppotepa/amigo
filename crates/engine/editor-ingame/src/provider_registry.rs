use std::sync::RwLock;

use amigo_editor_api::EditorCommandProvider;

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
