fn root() { "playground-2d-particles-showcase-ui.root" }
fn path(id) { root() + "." + id }
fn controls_path(id) { path("body.side." + id) }
fn preview_emitter() { "playground-2d-particles-preview-emitter" }

fn selected_preset(world_api) { world_api.state.get_string("selected_preset") }
fn intensity(world_api) { world_api.state.get_float("intensity") }
fn preview_active(world_api) { world_api.state.get_bool("preview_active") }

fn percent(value) {
    return (value * 100.0).to_int() + "%";
}

fn is_burst_preset(world_api, preset) {
    if preset == "explosion" { return true; }
    if preset == "shockwave" { return true; }
    if preset == "muzzle_flash" { return true; }
    return world_api.particles.preset_category(preset) == "burst";
}

fn sync_ui(world_api) {
    let preset = selected_preset(world_api);
    let label = world_api.particles.preset_label(preset);
    let category = world_api.particles.preset_category(preset);
    let tags = world_api.particles.preset_tags(preset);
    world_api.ui.set_text(path("top.selected"), "Selected: " + preset);
    world_api.ui.set_options(controls_path("preset-options"), world_api.particles.preset_ids());
    world_api.ui.set_selected(controls_path("preset-options"), preset);
    world_api.ui.set_value(controls_path("intensity-slider"), intensity(world_api));
    world_api.ui.set_text(controls_path("intensity-value"), "Intensity " + percent(intensity(world_api)));
    world_api.ui.set_text(controls_path("current-detail"), label + " / " + category + " / " + tags);
}

fn focus_selected(world_api) {
    let preset = selected_preset(world_api);
    world_api.particles.apply_preset(preset, preview_emitter());
    world_api.particles.set_intensity(preview_emitter(), intensity(world_api));
    world_api.particles.set_active(preview_emitter(), preview_active(world_api) && !is_burst_preset(world_api, preset));
    if is_burst_preset(world_api, preset) {
        world_api.particles.burst(preview_emitter(), 96);
    }
}

fn set_selected_preset(world_api, preset) {
    world_api.state.set_string("selected_preset", preset);
    focus_selected(world_api);
    sync_ui(world_api);
}

fn set_selected_intensity(world_api, value) {
    let clamped = value;
    if clamped < 0.0 { clamped = 0.0; }
    if clamped > 1.0 { clamped = 1.0; }
    world_api.state.set_float("intensity", clamped);
    world_api.particles.set_intensity(preview_emitter(), clamped);
    sync_ui(world_api);
}

fn set_all_active(world_api, active) {
    world_api.state.set_bool("preview_active", active);
    world_api.particles.set_active(preview_emitter(), active && !is_burst_preset(world_api, selected_preset(world_api)));
}

fn toggle_selected(world_api) {
    set_all_active(world_api, !preview_active(world_api));
}

fn on_enter() {
    world.ui.set_theme("space_dark");
    world.state.reset_scene();
    world.state.set_string("selected_preset", "fire");
    world.state.set_float("intensity", 1.0);
    world.state.set_bool("preview_active", true);

    focus_selected(world);
    sync_ui(world);
}

fn update(dt) {
    if world.input.pressed("Escape") {
        world.scene.select("menu");
        return;
    }

    if world.input.pressed("1") { set_selected_preset(world, "fire"); }
    if world.input.pressed("2") { set_selected_preset(world, "smoke"); }
    if world.input.pressed("3") { set_selected_preset(world, "sparks"); }
    if world.input.pressed("4") { set_selected_preset(world, "magic"); }
    if world.input.pressed("5") { set_selected_preset(world, "snow"); }
    if world.input.pressed("6") { set_selected_preset(world, "dust"); }
    if world.input.pressed("7") { set_selected_preset(world, "thruster"); }
    if world.input.pressed("8") { set_selected_preset(world, "plasma"); }
    if world.input.pressed("9") { set_selected_preset(world, "portal"); }
    if world.input.pressed("0") { set_selected_preset(world, "explosion"); }

    if world.input.pressed("Space") {
        toggle_selected(world);
    }
    if world.input.pressed("A") {
        set_all_active(world, true);
    }
    if world.input.pressed("S") {
        set_all_active(world, false);
    }
    if world.input.pressed("R") {
        set_selected_intensity(world, 1.0);
    }
    if world.input.pressed("B") {
        world.particles.burst(preview_emitter(), 48);
    }
}

fn on_event(topic, payload) {
    if topic == "playground-2d-particles.menu" {
        world.scene.select("menu");
        return;
    }
    if topic == "playground-2d-particles.showcase.start-all" {
        set_all_active(world, true);
        return;
    }
    if topic == "playground-2d-particles.showcase.stop-all" {
        set_all_active(world, false);
        return;
    }
    if topic == "playground-2d-particles.showcase.select" {
        set_selected_preset(world, payload[0]);
        return;
    }
    if topic == "playground-2d-particles.showcase.intensity" {
        set_selected_intensity(world, payload[0].to_float());
        return;
    }
    if topic == "playground-2d-particles.showcase.intensity-reset" {
        set_selected_intensity(world, 1.0);
        return;
    }
}
