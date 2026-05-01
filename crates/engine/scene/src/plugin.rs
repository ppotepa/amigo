use amigo_runtime::{RuntimePlugin, ServiceRegistry};

use crate::*;


pub struct ScenePlugin;

impl RuntimePlugin for ScenePlugin {
    fn name(&self) -> &'static str {
        "amigo-scene"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> amigo_core::AmigoResult<()> {
        registry.register(SceneService::default())?;
        registry.register(HydratedSceneState::default())?;
        registry.register(SceneTransitionService::default())?;
        registry.register(EntityPoolSceneService::default())?;
        registry.register(LifetimeSceneService::default())?;
        registry.register(CameraFollow2dSceneService::default())?;
        registry.register(Parallax2dSceneService::default())?;
        registry.register(ActivationSetSceneService::default())?;
        registry.register(SceneCommandQueue::default())?;
        registry.register(SceneEventQueue::default())
    }
}
