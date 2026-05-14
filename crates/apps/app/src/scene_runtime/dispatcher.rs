use super::super::*;

pub(crate) struct SceneCommandRuntimePlugin;

impl RuntimePlugin for SceneCommandRuntimePlugin {
    fn name(&self) -> &'static str {
        "amigo-app-scene-command-registry"
    }

    fn register(&self, _services: &mut ServiceRegistry) -> AmigoResult<()> {
        Ok(())
    }
}
