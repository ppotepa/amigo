use amigo_runtime::Runtime;

pub fn update_ui_input_viewport_state(runtime: &Runtime, width: f32, height: f32) {
    let Ok(state) = runtime.required::<amigo_ui::UiInputViewportState>() else {
        return;
    };

    state.set(Some(amigo_overlay_api::UiViewportSize::new(width, height)));
}
