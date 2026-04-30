use amigo_core::AmigoResult;

use crate::RuntimeBuilder;

pub trait PluginBundle {
    fn name(&self) -> &'static str;
    fn register(self, builder: RuntimeBuilder) -> AmigoResult<RuntimeBuilder>;
}
