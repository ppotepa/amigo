import "pkg:amigo.std/input" as input;

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

fn pipe_join(values) {
    let output = "";
    let index = 0;
    while index < values.len() {
        if index > 0 {
            output += "|";
        }
        output += values[index];
        index += 1;
    }
    return output;
}

fn is_burst_preset(world_api, preset) {
    if preset == "explosion" { return true; }
    if preset == "shockwave" { return true; }
    if preset == "muzzle_flash" { return true; }
    return world_api.particles.preset_category(preset) == "burst";
}

fn sync_model_state(world_api) {
    let preset = selected_preset(world_api);
    let label = world_api.particles.preset_label(preset);
    let category = world_api.particles.preset_category(preset);
    let tags = world_api.particles.preset_tags(preset);
    world_api.state.set_string("active_theme", "space_dark");
    world_api.state.set_string("preset_options", pipe_join(world_api.particles.preset_ids()));
    world_api.state.set_string("selected_preset_label", "Selected: " + preset);
    world_api.state.set_string("intensity_label", "Intensity " + percent(intensity(world_api)));
    world_api.state.set_string("current_preset_detail", label + " / " + category + " / " + tags);
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
    sync_model_state(world_api);
}

fn set_selected_intensity(world_api, value) {
    let clamped = value;
    if clamped < 0.0 { clamped = 0.0; }
    if clamped > 1.0 { clamped = 1.0; }
    world_api.state.set_float("intensity", clamped);
    world_api.particles.set_intensity(preview_emitter(), clamped);
    sync_model_state(world_api);
}

fn set_all_active(world_api, active) {
    world_api.state.set_bool("preview_active", active);
    world_api.particles.set_active(preview_emitter(), active && !is_burst_preset(world_api, selected_preset(world_api)));
}

fn toggle_selected(world_api) {
    set_all_active(world_api, !preview_active(world_api));
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_string("selected_preset", "fire");
    world.state.set_float("intensity", 1.0);
    world.state.set_bool("preview_active", true);

    focus_selected(world);
    sync_model_state(world);
}

fn update(dt) {
    if input::pressed(world, "showcase.select_fire") { set_selected_preset(world, "fire"); }
    if input::pressed(world, "showcase.select_smoke") { set_selected_preset(world, "smoke"); }
    if input::pressed(world, "showcase.select_sparks") { set_selected_preset(world, "sparks"); }
    if input::pressed(world, "showcase.select_magic") { set_selected_preset(world, "magic"); }
    if input::pressed(world, "showcase.select_snow") { set_selected_preset(world, "snow"); }
    if input::pressed(world, "showcase.select_dust") { set_selected_preset(world, "dust"); }
    if input::pressed(world, "showcase.select_engine_plume") { set_selected_preset(world, "engine_plume"); }
    if input::pressed(world, "showcase.select_plasma") { set_selected_preset(world, "plasma"); }
    if input::pressed(world, "showcase.select_portal") { set_selected_preset(world, "portal"); }
    if input::pressed(world, "showcase.select_explosion") { set_selected_preset(world, "explosion"); }

    if input::pressed(world, "showcase.toggle_selected") {
        toggle_selected(world);
    }
    if input::pressed(world, "showcase.start_all") {
        set_all_active(world, true);
    }
    if input::pressed(world, "showcase.stop_all") {
        set_all_active(world, false);
    }
    if input::pressed(world, "showcase.reset") {
        set_selected_intensity(world, 1.0);
    }
    if input::pressed(world, "showcase.burst") {
        world.particles.burst(preview_emitter(), 48);
    }
}

fn on_event(topic, payload) {
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
