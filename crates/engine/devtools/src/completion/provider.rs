use std::sync::RwLock;

use amigo_runtime::Runtime;

use super::{ConsoleCompletionContext, ConsoleCompletionSuggestion, ConsoleRhaiSymbol};

pub trait ConsoleCompletionProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn augment_context(&self, _runtime: &Runtime, _context: &mut ConsoleCompletionContext) {}

    fn rhai_symbols(&self, _runtime: &Runtime) -> Vec<ConsoleRhaiSymbol> {
        Vec::new()
    }

    fn rhai_properties(&self, _value_kind: &str) -> Option<Vec<ConsoleCompletionSuggestion>> {
        None
    }

    fn postfx_kinds(&self, _runtime: &Runtime) -> Vec<String> {
        Vec::new()
    }
}

#[derive(Default)]
pub struct ConsoleCompletionProviderRegistry {
    providers: RwLock<Vec<Box<dyn ConsoleCompletionProvider>>>,
}

impl ConsoleCompletionProviderRegistry {
    pub fn register<P>(&self, provider: P)
    where
        P: ConsoleCompletionProvider + 'static,
    {
        self.providers
            .write()
            .expect("console completion provider registry poisoned")
            .push(Box::new(provider));
    }

    pub fn augment_context(&self, runtime: &Runtime, context: &mut ConsoleCompletionContext) {
        let providers = self
            .providers
            .read()
            .expect("console completion provider registry poisoned");
        for provider in providers.iter() {
            provider.augment_context(runtime, context);
        }
    }

    pub fn rhai_symbols(&self, runtime: &Runtime) -> Vec<ConsoleRhaiSymbol> {
        let providers = self
            .providers
            .read()
            .expect("console completion provider registry poisoned");
        providers
            .iter()
            .flat_map(|provider| provider.rhai_symbols(runtime))
            .collect()
    }

    pub fn rhai_properties(&self, value_kind: &str) -> Vec<ConsoleCompletionSuggestion> {
        let providers = self
            .providers
            .read()
            .expect("console completion provider registry poisoned");
        providers
            .iter()
            .filter_map(|provider| provider.rhai_properties(value_kind))
            .flatten()
            .collect()
    }

    pub fn postfx_kinds(&self, runtime: &Runtime) -> Vec<String> {
        let providers = self
            .providers
            .read()
            .expect("console completion provider registry poisoned");
        providers
            .iter()
            .flat_map(|provider| provider.postfx_kinds(runtime))
            .collect()
    }

    pub fn provider_ids(&self) -> Vec<&'static str> {
        self.providers
            .read()
            .expect("console completion provider registry poisoned")
            .iter()
            .map(|provider| provider.provider_id())
            .collect()
    }
}
