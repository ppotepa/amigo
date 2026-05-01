import "pkg:amigo.std/input" as input;

fn root() { "playground-hud-ui-showcase.root" }
fn path(id) { root() + "." + id }
fn controls_path(id) { path("main.controls." + id) }
fn preview_path(id) { path("main.preview." + id) }
fn editor_path(id) { path("main.editor." + id) }

fn percent(value) {
    let whole = (value * 100.0).to_int();
    return whole + "%";
}

fn active_theme(world_api) { world_api.state.get_string("active_theme") }
fn volume(world_api) { world_api.state.get_float("volume") }
fn spawn_rate(world_api) { world_api.state.get_float("spawn_rate") }
fn lifetime(world_api) { world_api.state.get_float("lifetime") }
fn debug_enabled(world_api) { world_api.state.get_bool("debug_enabled") }
fn color_preset(world_api) { world_api.state.get_string("color_preset") }
fn selected_tab(world_api) { world_api.state.get_string("selected_tab") }

fn color_hex(name) {
    if name == "Blue" { return "#236DFFFF"; }
    if name == "Orange" { return "#FF8A22FF"; }
    if name == "White" { return "#F8F8F8FF"; }
    return "#7E5BFFFF";
}

fn sync_model_state(world_api) {
    world_api.state.set_string("active_theme_label", "Theme: " + active_theme(world_api));
    world_api.state.set_string("volume_label", "Volume " + percent(volume(world_api)));
    world_api.state.set_string("debug_label", if debug_enabled(world_api) { "Debug overlay ON" } else { "Debug overlay OFF" });
    world_api.state.set_string("selected_tab_label", "Active tab: " + selected_tab(world_api));
    world_api.state.set_string("spawn_rate_label", "Spawn rate " + percent(spawn_rate(world_api)));
    world_api.state.set_string("lifetime_label", "Lifetime " + percent(lifetime(world_api)));
    world_api.state.set_string("color_preset_label", "Color preset " + color_preset(world_api));
    world_api.state.set_string("color_swatch", color_hex(color_preset(world_api)));
    world_api.state.set_string("debug_text", "selected_theme=" + active_theme(world_api) + " selected_tab=" + selected_tab(world_api) + " volume=" + percent(volume(world_api)) + " spawn_rate=" + percent(spawn_rate(world_api)));
}

fn set_theme(world_api, theme) {
    world_api.state.set_string("active_theme", theme);
    world_api.ui.set_text(path("toast.message"), "Theme changed: " + theme);
    world_api.ui.show(path("toast"));
    sync_model_state(world_api);
}

fn clamp01(value) {
    if value < 0.0 { return 0.0; }
    if value > 1.0 { return 1.0; }
    return value;
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_string("active_theme", "space_dark");
    world.state.set_float("volume", 0.65);
    world.state.set_float("spawn_rate", 0.72);
    world.state.set_float("lifetime", 0.42);
    world.state.set_bool("debug_enabled", true);
    world.state.set_string("color_preset", "Blue");
    world.state.set_string("selected_tab", "Color");
    sync_model_state(world);
}

fn update(dt) {
    if input::pressed(world, "ui.theme.space_dark") {
        set_theme(world, "space_dark");
    }
    if input::pressed(world, "ui.theme.clean_dev") {
        set_theme(world, "clean_dev");
    }
    if input::pressed(world, "ui.theme.cycle") {
        if active_theme(world) == "space_dark" {
            set_theme(world, "clean_dev");
        } else {
            set_theme(world, "space_dark");
        }
    }
}

fn on_event(topic, payload) {
    if topic == "playground-hud-ui.theme.space-dark" {
        set_theme(world, "space_dark");
        return;
    }
    if topic == "playground-hud-ui.theme.clean-dev" {
        set_theme(world, "clean_dev");
        return;
    }
    if topic == "playground-hud-ui.volume.changed" {
        world.state.set_float("volume", payload[0].to_float());
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.debug.changed" {
        world.state.set_bool("debug_enabled", payload[0].to_float() >= 0.5);
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.spawn-rate.changed" {
        world.state.set_float("spawn_rate", payload[0].to_float());
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.lifetime.changed" {
        world.state.set_float("lifetime", payload[0].to_float());
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.color.changed" {
        world.state.set_string("color_preset", payload[0]);
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.tab.changed" {
        world.state.set_string("selected_tab", payload[0]);
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.stepper.minus" {
        world.state.set_float("volume", clamp01(volume(world) - 0.05));
        sync_model_state(world);
        return;
    }
    if topic == "playground-hud-ui.stepper.plus" {
        world.state.set_float("volume", clamp01(volume(world) + 0.05));
        sync_model_state(world);
        return;
    }
}
