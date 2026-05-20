use amigo_core::AmigoResult;
use amigo_runtime::Runtime;

use crate::EditorCapabilityRegistry;

pub trait EditorCapabilityProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()>;
}

pub trait EditorCommandProvider: Send + Sync {
    fn provider_id(&self) -> &'static str;

    fn register_commands(&self, _runtime: &Runtime, _out: &mut Vec<EditorCommandDescriptor>) {}
}

#[derive(Clone, Debug)]
pub struct EditorCommandDescriptor {
    pub id: String,
    pub label: String,
    pub category: String,
}
