use amigo_runtime::Runtime;

pub fn update_ui_input_viewport_state(runtime: &Runtime, width: f32, height: f32) {
    let Ok(state) = runtime.required::<amigo_ui::UiInputViewportState>() else {
        return;
    };

    state.set(Some(amigo_overlay_api::UiViewportSize::new(width, height)));
}

pub fn clear_ui_input_frame_transients(runtime: &Runtime) {
    let Some(ui_input) = runtime.resolve::<amigo_ui::UiInputService>() else {
        return;
    };

    ui_input.clear_frame_transients();
}

pub fn set_ui_input_mouse_position(runtime: &Runtime, x: f32, y: f32) {
    let Some(ui_input) = runtime.resolve::<amigo_ui::UiInputService>() else {
        return;
    };

    ui_input.set_mouse_position(x, y);
}

pub fn set_ui_input_left_button(runtime: &Runtime, pressed: bool) {
    let Some(ui_input) = runtime.resolve::<amigo_ui::UiInputService>() else {
        return;
    };

    ui_input.set_left_button(pressed);
}

pub fn add_ui_input_mouse_wheel(runtime: &Runtime, delta_y: f32) {
    let Some(ui_input) = runtime.resolve::<amigo_ui::UiInputService>() else {
        return;
    };

    ui_input.add_mouse_wheel(delta_y);
}
