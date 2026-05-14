use amigo_core::AmigoResult;

use crate::EditorCapabilityRegistry;

pub trait EditorCapabilityProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn register(&self, registry: &EditorCapabilityRegistry) -> AmigoResult<()>;
}
