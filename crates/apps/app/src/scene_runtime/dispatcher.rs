use super::super::*;
use amigo_scene::RuntimeSceneCommandHandlerRegistry;

pub(crate) struct SceneCommandRuntimePlugin;

impl RuntimePlugin for SceneCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-scene-command-registry"
    }

    fn register(&self, services: &mut ServiceRegistry) -> AmigoResult<()> {
        services.register(RuntimeSceneCommandHandlerRegistry::new())
    }
}
