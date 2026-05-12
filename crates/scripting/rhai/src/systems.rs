use amigo_core::{AmigoError, AmigoResult};
use amigo_runtime::Runtime;
use amigo_scripting_api::{ScriptComponentService, ScriptRuntimeService};

fn required<T: Send + Sync + 'static>(runtime: &Runtime) -> AmigoResult<std::sync::Arc<T>> {
    runtime.resolve::<T>().ok_or_else(|| {
        AmigoError::Message(format!(
            "required service `{}` is not registered",
            std::any::type_name::<T>()
        ))
    })
}

pub fn tick_script_components(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;
    let components = required::<ScriptComponentService>(runtime)?;

    for component in components.components() {
        script_runtime
            .call_component_update(
                &component.source_name,
                &component.entity_name,
                &component.params,
                delta_seconds,
            )
            .map_err(|error| {
                script_component_lifecycle_error(
                    &component.entity_name,
                    &component.script,
                    &component.source_name,
                    "update",
                    error,
                )
            })?;
    }

    Ok(())
}

fn script_component_lifecycle_error(
    entity_name: &str,
    script: &std::path::Path,
    source_name: &str,
    phase: &str,
    error: impl std::fmt::Display,
) -> AmigoError {
    AmigoError::Message(format!(
        "script component lifecycle phase `{phase}` failed for entity `{entity_name}` (script path `{}`, source name `{source_name}`): {error}",
        script.display()
    ))
}
