use amigo_runtime::Runtime;
use amigo_scene::{ComponentRegistry, component_registry_for_runtime};

pub fn editor_component_registry(runtime: &Runtime) -> ComponentRegistry {
    component_registry_for_runtime(runtime)
}
