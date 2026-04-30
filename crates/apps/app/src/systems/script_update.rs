use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::ScriptRuntimeService;

use crate::ScriptExecutionRole;
use crate::runtime_context::required;

pub(crate) fn tick_active_scripts(runtime: &Runtime, delta_seconds: f32) -> AmigoResult<()> {
    let script_runtime = required::<ScriptRuntimeService>(runtime)?;

    for script in crate::scripting_runtime::current_executed_scripts(runtime)? {
        match script.role {
            ScriptExecutionRole::ModPersistent | ScriptExecutionRole::Scene => {
                script_runtime.call_update(&script.source_name, delta_seconds)?;
            }
            ScriptExecutionRole::ModBootstrap => {}
        }
    }

    Ok(())
}
