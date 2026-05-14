use std::any::type_name;
use std::sync::Arc;

use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::{Runtime, ServiceRegistry};

pub(crate) struct RuntimeContext<'a> {
    runtime: &'a Runtime,
}

pub(crate) type SceneCommandContext<'a> = RuntimeContext<'a>;
#[allow(dead_code)]
pub(crate) type ScriptCommandContext<'a> = RuntimeContext<'a>;
#[allow(dead_code)]
pub(crate) type SystemContext<'a> = RuntimeContext<'a>;
#[allow(dead_code)]
pub(crate) type AssetContext<'a> = RuntimeContext<'a>;

impl<'a> RuntimeContext<'a> {
    pub(crate) fn new(runtime: &'a Runtime) -> Self {
        Self { runtime }
    }

    pub(crate) fn required<T>(&self) -> AmigoResult<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.runtime
            .resolve::<T>()
            .ok_or(AmigoError::MissingService(type_name::<T>()))
    }

    pub(crate) fn optional<T>(&self) -> Option<Arc<T>>
    where
        T: Send + Sync + 'static,
    {
        self.runtime.resolve::<T>()
    }
}

pub(crate) fn required<T>(runtime: &Runtime) -> AmigoResult<Arc<T>>
where
    T: Send + Sync + 'static,
{
    SceneCommandContext::new(runtime).required::<T>()
}

pub(crate) fn required_from_registry<T>(registry: &ServiceRegistry) -> AmigoResult<Arc<T>>
where
    T: Send + Sync + 'static,
{
    registry
        .resolve::<T>()
        .ok_or(AmigoError::MissingService(type_name::<T>()))
}
