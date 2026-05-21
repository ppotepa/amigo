use std::collections::BTreeSet;
use std::sync::RwLock;

#[derive(Default)]
pub struct RuntimeRenderExtractorIdRegistry {
    ids: RwLock<BTreeSet<String>>,
}

impl RuntimeRenderExtractorIdRegistry {
    pub fn register(&self, id: impl Into<String>) {
        self.ids
            .write()
            .expect("render extractor id registry poisoned")
            .insert(id.into());
    }

    pub fn registered_ids(&self) -> Vec<String> {
        self.ids
            .read()
            .expect("render extractor id registry poisoned")
            .iter()
            .cloned()
            .collect()
    }
}
