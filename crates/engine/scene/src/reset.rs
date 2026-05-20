use amigo_core::AmigoResult;
use amigo_runtime::{Runtime, ServiceRegistry};
use std::sync::RwLock;

pub trait SceneResetHandler: Send + Sync {
    fn name(&self) -> &'static str;

    fn reset_scene(&self, runtime: &Runtime) -> AmigoResult<()>;
}

#[derive(Default)]
pub struct SceneResetHandlerRegistry {
    handlers: RwLock<Vec<Box<dyn SceneResetHandler>>>,
}

impl SceneResetHandlerRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register<H>(&self, handler: H)
    where
        H: SceneResetHandler + 'static,
    {
        self.handlers
            .write()
            .expect("scene reset handler registry should not be poisoned")
            .push(Box::new(handler));
    }

    pub fn reset_all(&self, runtime: &Runtime) -> AmigoResult<()> {
        let handlers = self
            .handlers
            .read()
            .expect("scene reset handler registry should not be poisoned");
        for handler in handlers.iter() {
            handler.reset_scene(runtime)?;
        }
        Ok(())
    }

    pub fn handler_names(&self) -> Vec<&'static str> {
        self.handlers
            .read()
            .expect("scene reset handler registry should not be poisoned")
            .iter()
            .map(|handler| handler.name())
            .collect()
    }
}

pub fn register_scene_reset_handler<H>(
    registry: &ServiceRegistry,
    handler: H,
) -> AmigoResult<()>
where
    H: SceneResetHandler + 'static,
{
    if let Some(reset_registry) = registry.resolve::<SceneResetHandlerRegistry>() {
        reset_registry.register(handler);
    }
    Ok(())
}
