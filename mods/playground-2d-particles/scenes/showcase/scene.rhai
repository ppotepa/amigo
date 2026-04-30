fn root() { "playground-2d-particles-showcase-ui.root" }
fn path(id) { root() + "." + id }
fn controls_path(id) { path("body.side." + id) }
fn preview_emitter() { "playground-2d-particles-preview-emitter" }

fn preset_count() { 11 }

fn preset_at(index) {
    if index == 0 { return "fire"; }
    if index == 1 { return "smoke"; }
    if index == 2 { return "sparks"; }
    if index == 3 { return "magic"; }
    if index == 4 { return "snow"; }
    if index == 5 { return "dust"; }
    if index == 6 { return "thruster"; }
    if index == 7 { return "plasma"; }
    if index == 8 { return "portal"; }
    if index == 9 { return "rain"; }
    return "explosion";
}

fn selected_preset(world_api) { world_api.state.get_string("selected_preset") }
fn intensity(world_api) { world_api.state.get_float("intensity") }

fn percent(value) {
    return (value * 100.0).to_int() + "%";
}

fn sync_ui(world_api) {
    world_api.ui.set_text(path("top.selected"), "Selected: " + selected_preset(world_api));
    world_api.ui.set_selected(controls_path("preset-options"), selected_preset(world_api));
    world_api.ui.set_value(controls_path("intensity-slider"), intensity(world_api));
    world_api.ui.set_text(controls_path("intensity-value"), "Intensity " + percent(intensity(world_api)));
}

fn focus_selected(world_api) {
    world_api.particles.apply_preset(selected_preset(world_api), preview_emitter());
    world_api.particles.set_intensity(preview_emitter(), intensity(world_api));
    world_api.particles.set_active(preview_emitter(), selected_preset(world_api) != "explosion");
    if selected_preset(world_api) == "explosion" {
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
    world_api.state.set_bool("active." + selected_preset(world_api), active);
    world_api.particles.set_active(preview_emitter(), active);
}

fn toggle_selected(world_api) {
    let key = "active." + selected_preset(world_api);
    let active = world_api.state.get_bool(key);
    world_api.state.set_bool(key, !active);
    world_api.particles.set_active(preview_emitter(), !active);
}

fn on_enter() {
    world.ui.set_theme("space_dark");
    world.state.reset_scene();
    world.state.set_string("selected_preset", "fire");
    world.state.set_float("intensity", 1.0);

    let i = 0;
    while i < preset_count() {
        world.state.set_bool("active." + preset_at(i), preset_at(i) == "fire");
        i += 1;
    }

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
