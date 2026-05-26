import "mod:main_menu/ids" as ids;
import "mod:main_menu/math" as math;
import "mod:main_menu/timeline" as timeline;

fn slot_key(slot, suffix) {
    "camera_presets.slot_" + slot + "." + suffix
}

fn apply_slot(world_api, slot) {
    let preset_id = world_api.state.get_string(slot_key(slot, "preset_id"));
    if preset_id == "" {
        return false;
    }

    let label = world_api.state.get_string(slot_key(slot, "label"));
    let focal = math::clamp(world_api.state.get_float(slot_key(slot, "focal_length_mm")), 8.0, 300.0);
    let f_stop = math::clamp(world_api.state.get_float(slot_key(slot, "aperture_f_stop")), 0.7, 32.0);
    let focus = math::clamp(world_api.state.get_float(slot_key(slot, "focus_distance_m")), 0.2, 1000.0);
    let shutter_enabled = world_api.state.get_bool(slot_key(slot, "shutter_enabled"));
    let shutter_speed_s = math::clamp(world_api.state.get_float(slot_key(slot, "shutter_speed_s")), 1.0 / 8000.0, 2.0);
    let shutter_opacity = math::clamp(world_api.state.get_float(slot_key(slot, "shutter_opacity")), 0.0, 1.0);

    if !world_api.camera.set_main_preset(preset_id) {
        return false;
    }

    let lens_profile = world_api.state.get_string("camera_presets.lens_profile");
    if lens_profile != "" {
        world_api.camera.set_main_lens_profile(lens_profile);
    }
    let rain_profile = world_api.state.get_string("camera_presets.rain_profile");
    if rain_profile != "" {
        world_api.camera.set_main_lens_rain_profile(rain_profile);
    } else {
        world_api.camera.clear_main_lens_rain_profile();
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
    world_api.camera.set_main_shutter_enabled(shutter_enabled);
    world_api.camera.set_main_shutter_speed_s(shutter_speed_s);
    world_api.camera.set_main_shutter_opacity(shutter_opacity);
    apply_scene_depth_bokeh(world_api);
    world_api.dev.log("Camera preset [" + slot + "] " + label + " -> " + preset_id);
    true
}

fn apply_scene_depth_bokeh(world_api) {
    world_api.camera.set_main_dof_max_blur_px(world_api.state.get_float("camera_dof_controls.max_blur_px"));
    world_api.camera.set_main_dof_focus_width(world_api.state.get_float("camera_dof_controls.focus_width"));
    world_api.camera.set_main_dof_blur_boosts(
        world_api.state.get_float("camera_dof_controls.foreground_blur_boost"),
        world_api.state.get_float("camera_dof_controls.background_blur_boost")
    );
    world_api.camera.set_main_dof_sample_count(world_api.state.get_float("camera_dof_controls.sample_count"));
    world_api.camera.set_main_dof_highlights(
        world_api.state.get_float("camera_dof_controls.highlight_threshold"),
        world_api.state.get_float("camera_dof_controls.highlight_knee"),
        world_api.state.get_float("camera_dof_controls.highlight_gain"),
        world_api.state.get_float("camera_dof_controls.highlight_saturation")
    );
}

fn apply_presets(world_api) {
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

fn update_preset_input(world_api) {
    if !world_api.state.get_bool("camera_presets.enabled") {
        return;
    }

    if world_api.actions.pressed("camera.preset.1") { apply_slot(world_api, "1"); return; }
    if world_api.actions.pressed("camera.preset.2") { apply_slot(world_api, "2"); return; }
    if world_api.actions.pressed("camera.preset.3") { apply_slot(world_api, "3"); return; }
    if world_api.actions.pressed("camera.preset.4") { apply_slot(world_api, "4"); return; }
    if world_api.actions.pressed("camera.preset.5") { apply_slot(world_api, "5"); return; }
    if world_api.actions.pressed("camera.preset.6") { apply_slot(world_api, "6"); return; }
    if world_api.actions.pressed("camera.preset.7") { apply_slot(world_api, "7"); return; }
    if world_api.actions.pressed("camera.preset.8") { apply_slot(world_api, "8"); return; }
    if world_api.actions.pressed("camera.preset.9") { apply_slot(world_api, "9"); return; }
    if world_api.actions.pressed("camera.preset.0") { apply_slot(world_api, "0"); }
}

fn apply_dof_controls(world_api) {
    let focal = math::clamp(world_api.state.get_float("camera_dof_controls.focal_length_mm"), 8.0, 300.0);
    let f_stop = math::clamp(world_api.state.get_float("camera_dof_controls.aperture_f_stop"), 0.7, 32.0);

    world_api.state.set_float("camera_dof_controls.focal_length_mm", focal);
    world_api.state.set_float("camera_dof_controls.aperture_f_stop", f_stop);

    world_api.camera.set_main_focal_length_mm(focal);
    world_api.camera.set_main_aperture_f_stop(f_stop);
    if world_api.state.get_bool("camera_dof_controls.manual_focus_enabled") {
        let focus = math::clamp(world_api.state.get_float("camera_dof_controls.focus_distance_m"), 0.2, 1000.0);
        world_api.state.set_float("camera_dof_controls.focus_distance_m", focus);
        world_api.camera.set_main_focus_distance_m(focus);
    }
}

fn update_dof_controls(world_api, dt) {
    if !world_api.state.get_bool("camera_dof_controls.enabled") {
        return;
    }

    let focal_axis = world_api.input.axis("]", "[");
    let aperture_axis = world_api.input.axis("'", ";");
    let focus_axis = world_api.input.axis("/", ".");
    let changed = focal_axis != 0 || aperture_axis != 0 || focus_axis != 0;

    if focus_axis != 0 {
        world_api.state.set_bool("camera_dof_controls.manual_focus_enabled", true);
    }
    if !changed {
        return;
    }

    world_api.state.set_float(
        "camera_dof_controls.focal_length_mm",
        world_api.state.get_float("camera_dof_controls.focal_length_mm")
            + math::axis_delta(focal_axis, world_api.state.get_float("camera_dof_controls.focal_length_speed_mm_per_second") * dt),
    );
    world_api.state.set_float(
        "camera_dof_controls.aperture_f_stop",
        world_api.state.get_float("camera_dof_controls.aperture_f_stop")
            + math::axis_delta(aperture_axis, world_api.state.get_float("camera_dof_controls.aperture_speed_f_stop_per_second") * dt),
    );
    world_api.state.set_float(
        "camera_dof_controls.focus_distance_m",
        world_api.state.get_float("camera_dof_controls.focus_distance_m")
            + math::axis_delta(focus_axis, world_api.state.get_float("camera_dof_controls.focus_distance_speed_m_per_second") * dt),
    );

    apply_dof_controls(world_api);
}

fn reset_intro(world_api) {
    world_api.state.set_bool("intro.focus.skyline_started", false);
    world_api.state.set_bool("intro.focus.far_street_started", false);
    world_api.state.set_bool("intro.focus.club_started", false);
    world_api.state.set_bool("intro.focus.title_started", false);
    world_api.state.set_bool("intro.focus_hunt.near_started", false);
    world_api.state.set_bool("intro.focus_hunt.far_started", false);
    world_api.state.set_bool("intro.focus_hunt.locked", false);
    world_api.state.set_float("camera_motion.dolly.signal", 0.0);
    world_api.state.set_float("camera_motion.dolly.camera_z_m", 0.0);
    world_api.state.set_float("camera_motion.focus_residual.current_m", 0.0);

    world_api.entities.set_position_2d(ids::camera_entity(), 0.0, 0.0);
    world_api.entities.set_rotation_2d(ids::camera_entity(), 0.0);
    world_api.entities.set_scale_2d(ids::camera_entity(), 1.05, 1.05);
    world_api.camera.set_main_sway_frequency(0.42);
    world_api.camera.set_main_sway_affects_focus(false);
    world_api.camera.set_main_camera_z_m(0.0);
    world_api.camera.set_main_focus_residual_m(0.0);
    world_api.camera.set_main_dolly_signal(0.0);
    world_api.state.set_float("camera_dof_controls.aperture_f_stop", 1.8);
    world_api.camera.set_main_aperture_f_stop(1.8);
    world_api.camera.set_main_focus_distance_m(0.2);
    world_api.camera.set_main_aperture_rotation_degrees(0.0);
    apply_scene_depth_bokeh(world_api);
}

fn update_motion(world_api, time) {
    if !world_api.state.get_bool("camera_motion.enabled") {
        return;
    }

    let intro_breath_t = math::clamp01(time / 6.0);
    let dolly_signal = (time * world_api.state.get_float("camera_motion.dolly.frequency")).sin();
    let zoom_breath_amount = world_api.state.get_float("camera_motion.dolly.zoom_intro_amount")
        + (world_api.state.get_float("camera_motion.dolly.zoom_final_amount")
            - world_api.state.get_float("camera_motion.dolly.zoom_intro_amount"))
            * intro_breath_t;
    let zoom_breath = dolly_signal * zoom_breath_amount;
    let camera_scale = 1.05 * (1.0 + zoom_breath);
    let camera_z_m = dolly_signal * world_api.state.get_float("camera_motion.dolly.amplitude_m");
    world_api.state.set_float("camera_motion.dolly.signal", dolly_signal);
    world_api.state.set_float("camera_motion.dolly.camera_z_m", camera_z_m);
    world_api.camera.set_main_dolly_signal(dolly_signal);
    world_api.camera.set_main_camera_z_m(camera_z_m);
    world_api.camera.set_main_sway_amounts(0.0, 0.0, 0.0, zoom_breath_amount, 0.0);

    let x = (time * world_api.state.get_float("camera_motion.sway.x.primary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.x.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.secondary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.x.detail_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.detail_amplitude");
    let y = (time * world_api.state.get_float("camera_motion.sway.y.primary_frequency")).cos()
            * world_api.state.get_float("camera_motion.sway.y.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.y.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.y.secondary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.y.detail_frequency")).cos()
            * world_api.state.get_float("camera_motion.sway.y.detail_amplitude");
    let rotation = (time * world_api.state.get_float("camera_motion.rotation.primary_frequency")).sin()
            * world_api.state.get_float("camera_motion.rotation.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.rotation.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.rotation.secondary_amplitude");
    world_api.entities.set_position_2d(ids::camera_entity(), x, y);
    world_api.entities.set_rotation_2d(ids::camera_entity(), rotation);
    world_api.entities.set_scale_2d(ids::camera_entity(), camera_scale, camera_scale);
}

fn update_focus(world_api, time) {
    if time < timeline::focus_hunt_end_s() {
        set_focus_distance_once(world_api, "intro.focus_hunt.near_started", 0.2);
        if time >= 1.30 {
            set_focus_once(world_api, "intro.focus_hunt.far_started", ids::background_layer(), 0.70);
        }
    } else {
        set_focus_once(world_api, "intro.focus.skyline_started", ids::background_layer(), 0.45);
        if world_api.state.get_bool("intro.focus.skyline_started") {
            world_api.state.set_bool("intro.focus_hunt.locked", true);
        }
    }

    if time >= timeline::far_street_focus_start_s() {
        set_focus_once(world_api, "intro.focus.far_street_started", ids::background_layer(), 3.0);
    }
    if time >= timeline::club_focus_start_s() {
        set_focus_once(world_api, "intro.focus.club_started", ids::background_layer(), 3.0);
    }
    if time >= timeline::title_focus_start_s() {
        set_focus_once(world_api, "intro.focus.title_started", ids::title_focus_target(), 2.0);
    }
}

fn set_focus_once(world_api, key, selector, seconds) {
    if world_api.state.get_bool(key) {
        return;
    }

    if world_api.camera.focus_main_over(selector, seconds) {
        world_api.state.set_bool(key, true);
    }
}

fn set_focus_distance_once(world_api, key, meters) {
    if world_api.state.get_bool(key) {
        return;
    }

    if world_api.camera.set_main_focus_distance_m(meters) {
        world_api.state.set_bool(key, true);
    }
}

fn update_bokeh(world_api, time) {
    let primary = (time * 0.34).sin() * 10.0;
    let secondary = (time * 0.91).sin() * 3.5;
    world_api.camera.set_main_aperture_rotation_degrees(primary + secondary);
}

fn update_title_optics(world_api, time) {
    if time >= timeline::title_focus_start_s() {
        let t = math::ramp(time, timeline::title_focus_start_s(), 2.0);
        world_api.camera.set_main_aperture_f_stop(math::mix(1.8, 1.4, t));
    }

    if world_api.state.get_bool("intro.focus.title_started") {
        if world_api.state.get_bool("camera_motion.focus_residual.enabled") {
            let freq = world_api.state.get_float("camera_motion.focus_residual.frequency");
            let amp = world_api.state.get_float("camera_motion.focus_residual.amplitude_m");
            let residual = (time * freq).sin() * amp * math::ramp(time, timeline::title_reveal_start_s(), 2.0);
            world_api.state.set_float("camera_motion.focus_residual.current_m", residual);
            world_api.camera.set_main_focus_residual_m(residual);
        }
    } else {
        world_api.state.set_float("camera_motion.focus_residual.current_m", 0.0);
        world_api.camera.set_main_focus_residual_m(0.0);
    }
}
