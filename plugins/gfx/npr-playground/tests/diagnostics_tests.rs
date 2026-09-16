#[test]
fn indexed_geometry_builds_topology() {
    let geometry = amigo_render_npr::NprGeometry::from_indexed(
        &[[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
        &[0, 1, 2],
    )
    .unwrap();
    assert_eq!(amigo_render_npr::build_topology(&geometry).len(), 3);
}
#[test]
fn arrow_keys_rotate_the_model_without_moving_it() {
    let input = amigo_input_api::InputState::default();
    let state = amigo_npr_playground_plugin::NprPlaygroundState::default();
    input.set_key(amigo_input_api::KeyCode::Left, true);
    input.set_key(amigo_input_api::KeyCode::Up, true);

    let snapshot = state.update(&input, 0.5, 16.0 / 9.0);

    assert!(snapshot.object_yaw > 0.0);
    assert!(snapshot.object_pitch > 0.0);
    assert_eq!(snapshot.object_position, glam::Vec3::ZERO);
}

#[test]
fn suzanne_rotates_without_input_and_space_pauses_it() {
    let input = amigo_input_api::InputState::default();
    let state = amigo_npr_playground_plugin::NprPlaygroundState::default();
    let moving = state.update(&input, 0.5, 1.0);
    assert!(moving.object_yaw > 0.0);

    input.set_key(amigo_input_api::KeyCode::Space, true);
    state.update(&input, 0.0, 1.0);
    input.set_key(amigo_input_api::KeyCode::Space, false);
    let paused = state.update(&input, 0.5, 1.0);
    assert_eq!(paused.object_yaw, moving.object_yaw);
}

#[test]
fn mouse_controls_orbit_zoom_and_translate_the_model() {
    let input = amigo_input_api::InputState::default();
    let state = amigo_npr_playground_plugin::NprPlaygroundState::default();
    input.set_cursor_position(100.0, 100.0);
    input.set_mouse_button(amigo_input_api::MouseButton::Left, true);
    state.update(&input, 0.0, 16.0 / 9.0);
    input.set_cursor_position(140.0, 120.0);
    input.add_mouse_wheel_delta(2.0);

    let orbit = state.update(&input, 0.1, 16.0 / 9.0);
    assert_ne!(orbit.orbit_yaw, 0.0);
    assert!(orbit.camera_distance < 5.0);
    assert!(orbit.camera_distance > 5.0 * (-2.0_f32 * 0.0035).exp());

    input.set_mouse_button(amigo_input_api::MouseButton::Left, false);
    input.set_mouse_button(amigo_input_api::MouseButton::Right, true);
    input.set_cursor_position(180.0, 120.0);
    let moved = state.update(&input, 0.0, 16.0 / 9.0);
    assert_ne!(moved.object_position, glam::Vec3::ZERO);
}

#[test]
fn projection_shortcuts_switch_between_perspective_and_orthographic() {
    let input = amigo_input_api::InputState::default();
    let state = amigo_npr_playground_plugin::NprPlaygroundState::default();
    input.set_key(amigo_input_api::KeyCode::Digit2, true);
    assert_eq!(
        state.update(&input, 0.0, 1.0).projection,
        amigo_npr_playground_plugin::NprPlaygroundProjection::Orthographic,
    );
}

#[test]
fn style_shortcuts_switch_between_composed_npr_pipelines() {
    let input = amigo_input_api::InputState::default();
    let state = amigo_npr_playground_plugin::NprPlaygroundState::default();
    input.set_key(amigo_input_api::KeyCode::Digit4, true);
    assert_eq!(
        state.update(&input, 0.0, 1.0).style_profile,
        amigo_npr_playground_plugin::NprPlaygroundStyleProfile::PencilAnimation,
    );
}
