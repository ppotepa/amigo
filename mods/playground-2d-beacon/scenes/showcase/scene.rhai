fn target() { "all" }

fn clamp(value, min_value, max_value) {
    if value < min_value { return min_value; }
    if value > max_value { return max_value; }
    return value;
}

fn percent(value) {
    return (value * 100.0).to_int() + "%";
}

fn sync_labels(world_api) {
    world_api.state.set_string("active_theme", "beacon_dark");
    world_api.state.set_string("intensity_label", "Intensity " + world_api.state.get_float("intensity"));
    world_api.state.set_string("frequency_label", "Frequency " + world_api.state.get_float("frequency") + " Hz");
    world_api.state.set_string("duty_label", "Duty " + percent(world_api.state.get_float("duty")));
    world_api.state.set_string("halo_label", "Halo " + world_api.state.get_float("halo") + " px");
    world_api.state.set_string("aberration_label", "Aberration " + world_api.state.get_float("aberration") + " px");
    world_api.state.set_string("flare_label", "Flare " + world_api.state.get_float("flare"));
}

fn apply_controls(world_api) {
    world_api.beacon2d.set_base_intensity(target(), world_api.state.get_float("intensity"));
    world_api.beacon2d.set_frequency_hz(target(), world_api.state.get_float("frequency"));
    world_api.beacon2d.set_duty_cycle(target(), world_api.state.get_float("duty"));
    world_api.beacon2d.set_halo_radius_px(target(), world_api.state.get_float("halo"));
    world_api.beacon2d.set_aberration_px(target(), world_api.state.get_float("aberration"));
    world_api.beacon2d.set_flare_strength(target(), world_api.state.get_float("flare"));
}

fn set_intensity(world_api, value) {
    world_api.state.set_float("intensity", clamp(value, 0.0, 8.0));
    world_api.beacon2d.set_base_intensity(target(), world_api.state.get_float("intensity"));
    sync_labels(world_api);
}

fn set_frequency(world_api, value) {
    world_api.state.set_float("frequency", clamp(value, 0.1, 2.0));
    world_api.beacon2d.set_frequency_hz(target(), world_api.state.get_float("frequency"));
    sync_labels(world_api);
}

fn set_duty(world_api, value) {
    world_api.state.set_float("duty", clamp(value, 0.05, 0.85));
    world_api.beacon2d.set_duty_cycle(target(), world_api.state.get_float("duty"));
    sync_labels(world_api);
}

fn set_halo(world_api, value) {
    world_api.state.set_float("halo", clamp(value, 4.0, 120.0));
    world_api.beacon2d.set_halo_radius_px(target(), world_api.state.get_float("halo"));
    sync_labels(world_api);
}

fn set_aberration(world_api, value) {
    world_api.state.set_float("aberration", clamp(value, 0.0, 10.0));
    world_api.beacon2d.set_aberration_px(target(), world_api.state.get_float("aberration"));
    sync_labels(world_api);
}

fn set_flare(world_api, value) {
    world_api.state.set_float("flare", clamp(value, 0.0, 3.0));
    world_api.beacon2d.set_flare_strength(target(), world_api.state.get_float("flare"));
    sync_labels(world_api);
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_float("intensity", 3.0);
    world.state.set_float("frequency", 0.5);
    world.state.set_float("duty", 0.34);
    world.state.set_float("halo", 38.0);
    world.state.set_float("aberration", 2.5);
    world.state.set_float("flare", 0.9);
    sync_labels(world);
    apply_controls(world);
}

fn on_event(topic, payload) {
    if payload.len() == 0 { return; }
    let value = payload[0].to_float();

    if topic == "playground-2d-beacon.intensity" {
        set_intensity(world, value);
        return;
    }
    if topic == "playground-2d-beacon.frequency" {
        set_frequency(world, value);
        return;
    }
    if topic == "playground-2d-beacon.duty" {
        set_duty(world, value);
        return;
    }
    if topic == "playground-2d-beacon.halo" {
        set_halo(world, value);
        return;
    }
    if topic == "playground-2d-beacon.aberration" {
        set_aberration(world, value);
        return;
    }
    if topic == "playground-2d-beacon.flare" {
        set_flare(world, value);
        return;
    }
}
