use amigo_core::AmigoError;
use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::{ScriptComponentService, ScriptRuntimeService};

use crate::runtime_context::required;

pub(crate) fn tick_script_components(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
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
                AmigoError::Message(format!(
                    "script component update failed for entity `{}` using `{}`: {error}",
                    component.entity_name,
                    component.script.display()
                ))
            })?;
    }

    Ok(())
}
