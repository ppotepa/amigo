use super::super::*;
use crate::runtime_context::RuntimeContext;

pub(crate) fn process_ui_input(runtime: &Runtime) -> AmigoResult<()> {
    let viewport = RuntimeContext::new(runtime)
        .required::<super::UiInputViewportState>()?
        .get();
    let Some(viewport) = viewport else {
        return Ok(());
    };

    let ctx = RuntimeContext::new(runtime);
    let ui_input = ctx.required::<UiInputService>()?;
    let snapshot = ui_input.snapshot();
    if !snapshot.mouse_left_released {
        return Ok(());
    }

    let Some(mouse_position) = snapshot.mouse_position else {
        return Ok(());
    };

    let ui_scene = ctx.required::<UiSceneService>()?;
    let ui_state = ctx.required::<UiStateService>()?;
    let script_event_queue = ctx.required::<ScriptEventQueue>()?;
    let resolved =
        crate::ui_runtime::resolve_ui_overlay_documents(ui_scene.as_ref(), ui_state.as_ref());
    for document in resolved.iter().rev() {
        let layout = build_ui_layout_tree(viewport, &document.overlay);
        let Some(path) =
            crate::ui_runtime::hit_test_ui_layout(&layout, mouse_position.x, mouse_position.y)
        else {
            continue;
        };

        if !ui_state.is_enabled(&path) {
            continue;
        }

        if let Some(binding) = document.click_bindings.get(&path) {
            script_event_queue.publish(ScriptEvent::new(
                binding.event.clone(),
                binding.payload.clone(),
            ));
            break;
        }
    }

    Ok(())
}
