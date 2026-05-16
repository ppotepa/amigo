fn clamp(value, min_value, max_value) {
    if value < min_value {
        return min_value;
    }
    if value > max_value {
        return max_value;
    }
    value
}

fn axis_delta(axis, step) {
    if axis > 0 {
        return step;
    }
    if axis < 0 {
        return -step;
    }
    0.0
}

fn apply(world_api) {
    let focal = clamp(world_api.state.get_float("camera_dof_controls.focal_length_mm"), 8.0, 300.0);
    let f_stop = clamp(world_api.state.get_float("camera_dof_controls.aperture_f_stop"), 0.7, 32.0);
    let focus = clamp(world_api.state.get_float("camera_dof_controls.focus_depth"), 0.0, 1.0);

    world_api.state.set_float("camera_dof_controls.focal_length_mm", focal);
    world_api.state.set_float("camera_dof_controls.aperture_f_stop", f_stop);
    world_api.state.set_float("camera_dof_controls.focus_depth", focus);

    world_api.camera.set_main_focal_length_mm(focal);
    world_api.camera.set_main_aperture_f_stop(f_stop);
    world_api.camera.set_main_focus_depth(focus);
}

fn update(world_api, dt) {
    if !world_api.state.get_bool("camera_dof_controls.enabled") {
        return;
    }

    let focal_axis = world_api.input.axis("]", "[");
    let aperture_axis = world_api.input.axis("'", ";");
    let focus_axis = world_api.input.axis("/", ".");
    let changed = focal_axis != 0 || aperture_axis != 0 || focus_axis != 0;

    if !changed {
        return;
    }

    let focal_delta = axis_delta(
        focal_axis,
        world_api.state.get_float("camera_dof_controls.focal_length_speed_mm_per_second") * dt,
    );
    let aperture_delta = axis_delta(
        aperture_axis,
        world_api.state.get_float("camera_dof_controls.aperture_speed_f_stop_per_second") * dt,
    );
    let focus_delta = axis_delta(
        focus_axis,
        world_api.state.get_float("camera_dof_controls.focus_depth_speed_per_second") * dt,
    );

    world_api.state.set_float(
        "camera_dof_controls.focal_length_mm",
        world_api.state.get_float("camera_dof_controls.focal_length_mm") + focal_delta,
    );
    world_api.state.set_float(
        "camera_dof_controls.aperture_f_stop",
        world_api.state.get_float("camera_dof_controls.aperture_f_stop") + aperture_delta,
    );
    world_api.state.set_float(
        "camera_dof_controls.focus_depth",
        world_api.state.get_float("camera_dof_controls.focus_depth") + focus_delta,
    );

    apply(world_api);
}
