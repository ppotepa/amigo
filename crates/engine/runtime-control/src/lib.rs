mod completion;
mod error;
mod graph;
mod path;
mod provider;
mod registry;
mod service;
mod value;

pub use completion::*;
pub use error::*;
pub use graph::*;
pub use path::*;
pub use provider::*;
pub use registry::*;
pub use service::*;
pub use value::*;

pub struct RuntimeControlPlugin;
impl amigo_runtime::RuntimePlugin for RuntimeControlPlugin {
    fn name(&self) -> &'static str {
        "amigo-runtime-control"
    }
    fn register(
        &self,
        registry: &mut amigo_runtime::ServiceRegistry,
    ) -> amigo_core::AmigoResult<()> {
        registry.register(RuntimeControlService::default())
    }
}
