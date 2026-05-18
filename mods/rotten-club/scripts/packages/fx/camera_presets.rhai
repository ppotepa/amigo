fn clamp(value, min_value, max_value) {
    if value < min_value {
        return min_value;
    }
    if value > max_value {
        return max_value;
    }
    value
}

fn slot_key(slot, suffix) {
    "camera_presets.slot_" + slot + "." + suffix
}

fn apply_slot(world_api, slot) {
    let preset_id = world_api.state.get_string(slot_key(slot, "preset_id"));
    if preset_id == "" {
        return false;
    }

    let label = world_api.state.get_string(slot_key(slot, "label"));
    let focal = clamp(world_api.state.get_float(slot_key(slot, "focal_length_mm")), 8.0, 300.0);
    let f_stop = clamp(world_api.state.get_float(slot_key(slot, "aperture_f_stop")), 0.7, 32.0);
    let focus = clamp(world_api.state.get_float(slot_key(slot, "focus_distance_m")), 0.2, 1000.0);

    if !world_api.camera.set_main_preset(preset_id) {
        return false;
    }

    world_api.state.set_string("camera_presets.active_slot", slot);
    world_api.state.set_string("camera_presets.active_label", label);
    world_api.state.set_string("camera_presets.active_preset_id", preset_id);
    world_api.state.set_float("camera_dof_controls.focal_length_mm", focal);
    world_api.state.set_float("camera_dof_controls.aperture_f_stop", f_stop);
    world_api.state.set_float("camera_dof_controls.focus_distance_m", focus);

    world_api.camera.set_main_focal_length_mm(focal);
    world_api.camera.set_main_aperture_f_stop(f_stop);
    world_api.camera.set_main_focus_distance_m(focus);
    world_api.dev.log("Camera preset [" + slot + "] " + label + " -> " + preset_id);
    true
}

fn apply(world_api) {
    if !world_api.state.get_bool("camera_presets.enabled") {
        return;
    }

    let active_slot = world_api.state.get_string("camera_presets.active_slot");
    if active_slot == "" {
        apply_slot(world_api, "1");
        return;
    }

    apply_slot(world_api, active_slot);
}

fn update(world_api) {
    if !world_api.state.get_bool("camera_presets.enabled") {
        return;
    }

    if world_api.actions.pressed("camera.preset.1") {
        apply_slot(world_api, "1");
        return;
    }
    if world_api.actions.pressed("camera.preset.2") {
        apply_slot(world_api, "2");
        return;
    }
    if world_api.actions.pressed("camera.preset.3") {
        apply_slot(world_api, "3");
        return;
    }
    if world_api.actions.pressed("camera.preset.4") {
        apply_slot(world_api, "4");
        return;
    }
    if world_api.actions.pressed("camera.preset.5") {
        apply_slot(world_api, "5");
        return;
    }
    if world_api.actions.pressed("camera.preset.6") {
        apply_slot(world_api, "6");
        return;
    }
    if world_api.actions.pressed("camera.preset.7") {
        apply_slot(world_api, "7");
        return;
    }
    if world_api.actions.pressed("camera.preset.8") {
        apply_slot(world_api, "8");
        return;
    }
    if world_api.actions.pressed("camera.preset.9") {
        apply_slot(world_api, "9");
        return;
    }
    if world_api.actions.pressed("camera.preset.0") {
        apply_slot(world_api, "0");
    }
}
