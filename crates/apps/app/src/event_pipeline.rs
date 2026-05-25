use amigo_core::AmigoResult;
use amigo_runtime::Runtime;
use amigo_scripting_api::ScriptEvent;

use crate::runtime_context::RuntimeContext;
use crate::LaunchSelection;

pub(crate) fn run_event_pipelines_for_event(
    runtime: &Runtime,
    event: &ScriptEvent,
) -> AmigoResult<()> {
    let ctx = RuntimeContext::new(runtime);
    let launch_selection = ctx.optional::<LaunchSelection>();

    amigo_runtime_bundles::run_event_pipelines_for_event(
        runtime,
        event,
        |clip| {
            launch_selection
                .as_ref()
                .map(|selection| crate::app_helpers::resolve_mod_audio_asset_key(selection, clip))
                .unwrap_or_else(|| amigo_assets::AssetKey::new(clip.to_owned()))
        },
        |script_runtime, function, event| {
            for script in crate::scripting_runtime::current_executed_scripts(runtime)? {
                script_runtime.call_event_function(
                    &script.source_name,
                    function,
                    &event.topic,
                    &event.payload,
                )?;
            }
            Ok(())
        },
    )
}
