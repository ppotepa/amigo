use amigo_core::AmigoResult;
use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::AuthoringSceneGraphService;

pub struct EditorAuthoringPlugin {
    enabled: bool,
}

impl EditorAuthoringPlugin {
    pub fn new(enabled: bool) -> Self {
        Self { enabled }
    }
}

impl RuntimePlugin for EditorAuthoringPlugin {
    fn name(&self) -> &'static str {
        "amigo-editor-authoring"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        if self.enabled {
            registry.register(AuthoringSceneGraphService::default())?;
        }
        Ok(())
    }
}
