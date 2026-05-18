const OFFICE_LIGHT = "office-desk-point-light";
const OFFICE_BACKGROUND = "office-desk-background";
const OFFICE_BACKGROUND_LAYER = "background.room";
const OFFICE_BEACON_LAYER = "lighting.beacons";

fn on_enter() {
    world.state.set_int("office.focus_index", 2);
    world.state.set_float("office.light_intensity", 2.20);
    world.state.set_float("office.light_distance_m", 1.2);
    world.state.set_float("office.light_radius_px", 620.0);
    world.state.set_float("office.albedo_brightness", 1.0);
    world.state.set_float("office.light_target_x", 420.0);
    world.state.set_float("office.light_target_y", -35.0);
    world.state.set_float("office.light_current_x", 420.0);
    world.state.set_float("office.light_current_y", -35.0);
    world.state.set_float("office.light_target_z", 0.49);
    world.state.set_float("office.light_current_z", 0.49);
    world.state.set_int("office.light_follow_mouse", 1);
    world.state.set_int("office.relight_debug_index", 0);
    world.camera.set_main_debug_view("final_output");
    apply_focus_plane(world, 2);
    world.layered_image2d.set_base_opacity(OFFICE_BACKGROUND, 1.0);
    world.render2d.get_layer(OFFICE_BACKGROUND_LAYER).set_opacity(1.0);
    world.render2d.get_layer(OFFICE_BEACON_LAYER).set_opacity(0.0);
    world.beacon2d.set_position_2d(OFFICE_LIGHT, 420.0, -35.0);
    configure_cursor_light(world, 2.20);
    world.beacon2d.set_distance_m(OFFICE_LIGHT, 1.2);
    world.beacon2d.set_z_depth(OFFICE_LIGHT, 0.49);
    world.dev.log("Office Desk: one cursor light; left click pins/unpins follow; WASD moves pinned light; wheel changes light Z; Shift+wheel fine Z; Shift+Alt+wheel focus/blur; Ctrl+wheel intensity; Alt+wheel radius; D relight debug; R final; use plate.relight.status");
}

fn update(dt) {
    handle_cursor_light_input(world, dt);
    apply_cursor_light_smoothing(world, dt);

    if world.input.pressed("D") {
        cycle_relight_debug(world, 1);
    }

    if world.input.pressed("R") {
        set_relight_debug(world, 0);
    }
}

fn handle_cursor_light_input(world_api, dt) {
    let p = world_api.input.mouse_canvas_position(1672.0, 941.0);
    if world_api.input.mouse_pressed("left") {
        if world_api.state.get_int("office.light_follow_mouse") == 1 {
            world_api.state.set_int("office.light_follow_mouse", 0);
            if p["valid"] {
                set_light_target_xy(world_api, p["x"], p["y"]);
            }
        } else {
            world_api.state.set_int("office.light_follow_mouse", 1);
        }
    }

    if world_api.state.get_int("office.light_follow_mouse") == 1 && p["valid"] {
        set_light_target_xy(world_api, p["x"], p["y"]);
    }

    let x_axis = world_api.input.axis("D", "A");
    let y_axis = world_api.input.axis("S", "W");
    if x_axis != 0.0 || y_axis != 0.0 {
        world_api.state.set_int("office.light_follow_mouse", 0);
        let speed = light_xy_key_speed(world_api);
        let x = world_api.state.get_float("office.light_target_x") + x_axis * speed * dt;
        let y = world_api.state.get_float("office.light_target_y") + y_axis * speed * dt;
        set_light_target_xy(world_api, x, y);
    }

    let z_axis = world_api.input.axis("E,PageUp", "Q,PageDown");
    if z_axis != 0.0 {
        let speed = if world_api.input.shift_down() {
            0.18 * 0.25
        } else {
            0.18
        };
        adjust_light_z_target(world_api, z_axis * speed * dt);
    }

    let wheel = world_api.input.wheel_delta();
    if wheel != 0.0 {
        if world_api.input.shift_down() && world_api.input.alt_down() {
            adjust_focus_from_wheel(world_api, wheel);
        } else if world_api.input.ctrl_down() {
            adjust_light_intensity(world_api, wheel);
        } else if world_api.input.alt_down() {
            adjust_light_radius(world_api, wheel);
        } else if world_api.input.shift_down() {
            adjust_light_z_target(world_api, wheel * 0.0015);
        } else {
            adjust_light_z_target(world_api, wheel * 0.006);
        }
    }
}

fn adjust_focus_from_wheel(world_api, wheel) {
    if wheel > 0.0 {
        set_focus_index(world_api, world_api.state.get_int("office.focus_index") + 1);
    }
    if wheel < 0.0 {
        set_focus_index(world_api, world_api.state.get_int("office.focus_index") - 1);
    }
}

fn light_xy_key_speed(world_api) {
    if world_api.input.shift_down() {
        return 130.0;
    }
    if world_api.input.ctrl_down() {
        return 980.0;
    }
    520.0
}

fn apply_cursor_light_smoothing(world_api, dt) {
    let safe_dt = clamp_dt(dt);
    let alpha_xy = clamp_float(safe_dt * 18.0, 0.0, 1.0);
    let alpha_z = clamp_float(safe_dt * 12.0, 0.0, 1.0);
    let current_x = lerp_float(
        world_api.state.get_float("office.light_current_x"),
        world_api.state.get_float("office.light_target_x"),
        alpha_xy,
    );
    let current_y = lerp_float(
        world_api.state.get_float("office.light_current_y"),
        world_api.state.get_float("office.light_target_y"),
        alpha_xy,
    );
    let current_z = lerp_float(
        world_api.state.get_float("office.light_current_z"),
        world_api.state.get_float("office.light_target_z"),
        alpha_z,
    );
    apply_cursor_light_state(world_api, current_x, current_y, current_z);
}

fn set_light_target_xy(world_api, x, y) {
    world_api.state.set_float(
        "office.light_target_x",
        clamp_float(x, -936.0, 936.0),
    );
    world_api.state.set_float(
        "office.light_target_y",
        clamp_float(y, -570.5, 570.5),
    );
}

fn set_light_z_target(world_api, z) {
    world_api.state.set_float(
        "office.light_target_z",
        clamp_float(z, 0.04, 0.96),
    );
}

fn adjust_light_z_target(world_api, delta) {
    set_light_z_target(world_api, world_api.state.get_float("office.light_target_z") + delta);
}

fn apply_cursor_light_state(world_api, x, y, z) {
    world_api.state.set_float("office.light_current_x", x);
    world_api.state.set_float("office.light_current_y", y);
    world_api.state.set_float("office.light_current_z", z);
    world_api.beacon2d.set_position_2d("office-desk-point-light", x, y);
    world_api.beacon2d.set_z_depth("office-desk-point-light", z);
}

fn on_event(topic, payload) {
}

fn set_focus_index(world_api, index) {
    let clamped = clamp_focus_index(index);
    world_api.state.set_int("office.focus_index", clamped);
    apply_focus_plane(world_api, clamped);
}

fn clamp_focus_index(index) {
    if index < 0 {
        return 0;
    }
    if index > 7 {
        return 7;
    }
    index
}

fn apply_focus_plane(world_api, index) {
    world_api.camera.set_main_focus_distance_m(focus_distance_for_index(index));
}

fn cycle_relight_debug(world_api, delta) {
    let current = world_api.state.get_int("office.relight_debug_index");
    let next = current + delta;
    if next > 18 {
        set_relight_debug(world_api, 0);
    } else {
        set_relight_debug(world_api, next);
    }
}

fn set_relight_debug(world_api, index) {
    let clamped = clamp_debug_index(index);
    let view = relight_debug_view_for_index(clamped);
    world_api.state.set_int("office.relight_debug_index", clamped);
    world_api.camera.set_main_debug_view(view);
    world_api.dev.log("Office Desk relight debug: " + view);
}

fn clamp_debug_index(index) {
    if index < 0 {
        return 0;
    }
    if index > 18 {
        return 18;
    }
    index
}

fn relight_debug_view_for_index(index) {
    if index <= 0 {
        return "final_output";
    }
    if index == 1 {
        return "plate_relight_aux_depth";
    }
    if index == 2 {
        return "plate_relight_aux_height";
    }
    if index == 3 {
        return "plate_relight_aux_occluder";
    }
    if index == 4 {
        return "plate_relight_aux_valid";
    }
    if index == 5 {
        return "plate_relight_surface_reflect";
    }
    if index == 6 {
        return "plate_relight_surface_rough";
    }
    if index == 7 {
        return "plate_relight_surface_glass";
    }
    if index == 8 {
        return "plate_relight_surface_mask";
    }
    if index == 9 {
        return "plate_relight_effective_depth";
    }
    if index == 10 {
        return "plate_relight_normal";
    }
    if index == 11 {
        return "plate_relight_occlusion";
    }
    if index == 12 {
        return "plate_relight_contribution";
    }
    if index == 13 {
        return "plate_relight_shadow";
    }
    if index == 14 {
        return "plate_relight_light_mask";
    }
    if index == 15 {
        return "plate_relight_ndl";
    }
    if index == 16 {
        return "plate_relight_specular";
    }
    if index == 17 {
        return "plate_relight_material_gate";
    }
    "plate_relight_lit_raw"
}

fn adjust_light_intensity(world_api, wheel) {
    let next = clamp_float(
        world_api.state.get_float("office.light_intensity") + wheel * 0.04,
        0.05,
        6.0,
    );
    world_api.state.set_float("office.light_intensity", next);
    world_api.beacon2d.set_base_intensity("office-desk-point-light", next);
    world_api.beacon2d.set_glow_strength("office-desk-point-light", 0.0);
    world_api.beacon2d.set_bloom("office-desk-point-light", 0.0);
}

fn adjust_light_radius(world_api, wheel) {
    let next = clamp_float(
        world_api.state.get_float("office.light_radius_px") + wheel * 12.0,
        120.0,
        1100.0,
    );
    world_api.state.set_float("office.light_radius_px", next);
    world_api.beacon2d.set_halo_radius_px("office-desk-point-light", next);
}

fn configure_cursor_light(world_api, intensity) {
    world_api.beacon2d.set_base_intensity("office-desk-point-light", intensity);
    world_api.beacon2d.set_core_radius_px("office-desk-point-light", 18.0);
    world_api.beacon2d.set_halo_radius_px("office-desk-point-light", 620.0);
    world_api.state.set_float("office.light_radius_px", 620.0);
    world_api.beacon2d.set_glow_strength("office-desk-point-light", 0.0);
    world_api.beacon2d.set_bloom("office-desk-point-light", 0.0);
    world_api.beacon2d.set_lens_influence("office-desk-point-light", 1.25);
    world_api.beacon2d.set_flare_strength("office-desk-point-light", 0.0);
}

fn clamp_float(value, min_value, max_value) {
    if value < min_value {
        return min_value;
    }
    if value > max_value {
        return max_value;
    }
    value
}

fn clamp_dt(dt) {
    clamp_float(dt, 0.0, 0.05)
}

fn lerp_float(a, b, t) {
    a + (b - a) * clamp_float(t, 0.0, 1.0)
}

fn focus_distance_for_index(index) {
    if index <= 0 {
        return 0.5;
    }
    if index == 1 {
        return 0.8;
    }
    if index == 2 {
        return 1.2;
    }
    if index == 3 {
        return 2.0;
    }
    if index == 4 {
        return 3.5;
    }
    if index == 5 {
        return 6.0;
    }
    if index == 6 {
        return 12.0;
    }
    24.0
}
