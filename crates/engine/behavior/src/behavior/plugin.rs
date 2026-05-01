pub struct BehaviorPlugin;

impl RuntimePlugin for BehaviorPlugin {
    fn name(&self) -> &'static str {
        "amigo-behavior"
    }

    fn register(&self, registry: &mut ServiceRegistry) -> AmigoResult<()> {
        registry.register(BehaviorSceneService::default())
    }
}
