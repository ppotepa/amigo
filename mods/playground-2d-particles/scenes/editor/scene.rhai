fn emitter() { "playground-2d-particles-editor-preview-emitter" }
fn root() { "playground-2d-particles-editor-ui.root" }
fn path(id) { root() + "." + id }
fn controls_path(id) { path("main.controls." + id) }
fn panel_path(id) { controls_path("tab-panels." + id) }
fn emission_path(id) { panel_path("emission-panel." + id) }
fn color_path(id) { panel_path("color-panel." + id) }
fn size_path(id) { panel_path("size-panel." + id) }
fn motion_path(id) { panel_path("motion-panel." + id) }
fn forces_path(id) { panel_path("forces-panel." + id) }
fn spawn_path(id) { panel_path("spawn-panel." + id) }
fn burst_path(id) { panel_path("burst-panel." + id) }
fn export_path(id) { panel_path("export-panel." + id) }

fn spawn_rate(world_api) { world_api.state.get_float("spawn_rate") }
fn lifetime(world_api) { world_api.state.get_float("lifetime") }
fn speed(world_api) { world_api.state.get_float("speed") }
fn spread(world_api) { world_api.state.get_float("spread") }
fn initial_size(world_api) { world_api.state.get_float("initial_size") }
fn final_size(world_api) { world_api.state.get_float("final_size") }
fn gravity_y(world_api) { world_api.state.get_float("gravity_y") }
fn drag(world_api) { world_api.state.get_float("drag") }
fn active(world_api) { world_api.state.get_bool("active") }
fn color_name(world_api) { world_api.state.get_string("color") }
fn selected_tab(world_api) { world_api.state.get_string("selected_tab") }
fn spawn_kind(world_api) { world_api.state.get_string("spawn_kind") }

fn lerp(min, max, value) {
    return min + (max - min) * value;
}

fn normalize(min, max, value) {
    return (value - min) / (max - min);
}

fn color_hex(name) {
    if name == "Orange" { return "#FF8A22FF"; }
    if name == "White" { return "#F8F8F8FF"; }
    if name == "Purple" { return "#7E5BFFFF"; }
    if name == "Smoke" { return "#A0A0A066"; }
    return "#39D7FFFF";
}

fn apply_color(world_api, name) {
    if name == "Orange" {
        world_api.particles.set_color_rgba(emitter(), 1.0, 0.54, 0.13, 0.88);
        return;
    }
    if name == "White" {
        world_api.particles.set_color_rgba(emitter(), 0.97, 0.97, 0.97, 1.0);
        return;
    }
    if name == "Purple" {
        world_api.particles.set_color_rgba(emitter(), 0.49, 0.36, 1.0, 1.0);
        return;
    }
    if name == "Smoke" {
        world_api.particles.set_color_rgba(emitter(), 0.63, 0.63, 0.63, 0.40);
        return;
    }
    world_api.particles.set_color_rgba(emitter(), 0.22, 0.84, 1.0, 1.0);
}

fn apply_params(world_api) {
    world_api.particles.set_active(emitter(), active(world_api));
    world_api.particles.set_spawn_rate(emitter(), spawn_rate(world_api));
    world_api.particles.set_lifetime(emitter(), lifetime(world_api));
    world_api.particles.set_speed(emitter(), speed(world_api));
    world_api.particles.set_spread_degrees(emitter(), spread(world_api));
    world_api.particles.set_initial_size(emitter(), initial_size(world_api));
    world_api.particles.set_final_size(emitter(), final_size(world_api));
    world_api.particles.set_gravity(emitter(), 0.0, gravity_y(world_api));
    world_api.particles.set_drag(emitter(), drag(world_api));
    if spawn_kind(world_api) == "point" {
        world_api.particles.set_spawn_area_point(emitter());
    } else if spawn_kind(world_api) == "line" {
        world_api.particles.set_spawn_area_line(emitter(), 120.0);
    } else if spawn_kind(world_api) == "ring" {
        world_api.particles.set_spawn_area_ring(emitter(), 24.0, 52.0);
    } else {
        world_api.particles.set_spawn_area_circle(emitter(), 8.0);
    }
    apply_color(world_api, color_name(world_api));
}

fn sync_ui(world_api) {
    world_api.ui.set_selected(controls_path("tabs"), selected_tab(world_api));
    world_api.ui.set_text(controls_path("tab-title"), selected_tab(world_api) + " controls");
    world_api.ui.set_value(emission_path("active-toggle"), if active(world_api) { 1.0 } else { 0.0 });
    world_api.ui.set_value(emission_path("spawn-rate"), normalize(0.0, 200.0, spawn_rate(world_api)));
    world_api.ui.set_value(emission_path("lifetime"), normalize(0.1, 1.8, lifetime(world_api)));
    world_api.ui.set_value(motion_path("speed"), normalize(0.0, 300.0, speed(world_api)));
    world_api.ui.set_value(motion_path("spread"), normalize(0.0, 180.0, spread(world_api)));
    world_api.ui.set_value(size_path("sizes.initial-size"), normalize(0.0, 20.0, initial_size(world_api)));
    world_api.ui.set_value(size_path("sizes.final-size"), normalize(0.0, 20.0, final_size(world_api)));
    world_api.ui.set_value(forces_path("gravity-y"), normalize(-300.0, 300.0, gravity_y(world_api)));
    world_api.ui.set_value(forces_path("drag"), normalize(0.0, 5.0, drag(world_api)));
    world_api.ui.set_selected(spawn_path("spawn-kind"), spawn_kind(world_api));
    world_api.ui.set_selected(color_path("color-row.color-dropdown"), color_name(world_api));
    world_api.ui.set_background(color_path("color-row.color-swatch"), color_hex(color_name(world_api)));

    world_api.ui.set_text(emission_path("spawn-rate-label"), "Spawn rate " + spawn_rate(world_api).to_int());
    world_api.ui.set_text(emission_path("lifetime-label"), "Lifetime " + lifetime(world_api));
    world_api.ui.set_text(motion_path("speed-label"), "Speed " + speed(world_api).to_int());
    world_api.ui.set_text(motion_path("spread-label"), "Spread " + spread(world_api).to_int());
    world_api.ui.set_text(size_path("size-label"), "Size " + initial_size(world_api).to_int() + " -> " + final_size(world_api).to_int());
    world_api.ui.set_text(forces_path("gravity-label"), "Gravity Y " + gravity_y(world_api).to_int());
    world_api.ui.set_text(forces_path("drag-label"), "Drag " + drag(world_api));
    world_api.ui.set_text(spawn_path("spawn-label"), "Spawn area " + spawn_kind(world_api));
}

fn sync_tabs(world_api) {
    let tab = selected_tab(world_api);
    if tab == "" {
        tab = "Emission";
        world_api.state.set_string("selected_tab", tab);
    }
    let tabs = ["Emission", "Color", "Size", "Motion", "Forces", "Spawn", "Burst", "Export"];
    let panels = ["emission-panel", "color-panel", "size-panel", "motion-panel", "forces-panel", "spawn-panel", "burst-panel", "export-panel"];
    let i = 0;
    while i < tabs.len() {
        if tabs[i] == tab {
            world_api.ui.show(panel_path(panels[i]));
        } else {
            world_api.ui.hide(panel_path(panels[i]));
        }
        i += 1;
    }
    sync_ui(world_api);
}

fn set_tab(world_api, tab) {
    world_api.state.set_string("selected_tab", tab);
    sync_tabs(world_api);
}

fn reset_params(world_api) {
    world_api.state.set_bool("active", true);
    world_api.state.set_float("spawn_rate", 90.0);
    world_api.state.set_float("lifetime", 0.65);
    world_api.state.set_float("speed", 100.0);
    world_api.state.set_float("spread", 28.0);
    world_api.state.set_float("initial_size", 4.0);
    world_api.state.set_float("final_size", 13.0);
    world_api.state.set_float("gravity_y", 0.0);
    world_api.state.set_float("drag", 0.4);
    world_api.state.set_string("color", "Cyan");
    world_api.state.set_string("spawn_kind", "circle");
    world_api.state.set_string("selected_tab", "Emission");
    apply_params(world_api);
    sync_tabs(world_api);
    sync_ui(world_api);
}

fn print_yaml(world_api) {
    world_api.dev.log("ParticleEmitter2D preset:");
    world_api.dev.log("spawn_rate: " + spawn_rate(world_api));
    world_api.dev.log("particle_lifetime: " + lifetime(world_api));
    world_api.dev.log("initial_speed: " + speed(world_api));
    world_api.dev.log("spread_degrees: " + spread(world_api));
    world_api.dev.log("initial_size: " + initial_size(world_api));
    world_api.dev.log("final_size: " + final_size(world_api));
    world_api.dev.log("color: \"" + color_hex(color_name(world_api)) + "\"");
    world_api.dev.log("forces:");
    world_api.dev.log("  - kind: gravity");
    world_api.dev.log("    acceleration: { x: 0.0, y: " + gravity_y(world_api) + " }");
    world_api.dev.log("  - kind: drag");
    world_api.dev.log("    coefficient: " + drag(world_api));
    world_api.dev.log("spawn_area:");
    world_api.dev.log("  kind: " + spawn_kind(world_api));
    if spawn_kind(world_api) == "line" {
        world_api.dev.log("  length: 120.0");
    } else if spawn_kind(world_api) == "ring" {
        world_api.dev.log("  inner_radius: 24.0");
        world_api.dev.log("  outer_radius: 52.0");
    } else if spawn_kind(world_api) == "circle" {
        world_api.dev.log("  radius: 8.0");
    }
}

fn on_enter() {
    world.ui.set_theme("space_dark");
    world.state.reset_scene();
    reset_params(world);
}

fn update(dt) {
    if world.input.pressed("Escape") {
        world.scene.select("menu");
        return;
    }
}

fn on_event(topic, payload) {
    if topic == "playground-2d-particles.menu" {
        world.scene.select("menu");
        return;
    }
    if topic == "playground-2d-particles.editor.active" {
        world.state.set_bool("active", payload[0].to_float() >= 0.5);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.tab" {
        set_tab(world, payload[0]);
        return;
    }
    if topic == "playground-2d-particles.editor.spawn-rate" {
        world.state.set_float("spawn_rate", lerp(0.0, 200.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.lifetime" {
        world.state.set_float("lifetime", lerp(0.1, 1.8, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.speed" {
        world.state.set_float("speed", lerp(0.0, 300.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.spread" {
        world.state.set_float("spread", lerp(0.0, 180.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.initial-size" {
        world.state.set_float("initial_size", lerp(0.0, 20.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.final-size" {
        world.state.set_float("final_size", lerp(0.0, 20.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.gravity-y" {
        world.state.set_float("gravity_y", lerp(-300.0, 300.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.drag" {
        world.state.set_float("drag", lerp(0.0, 5.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.color" {
        world.state.set_string("color", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.spawn-kind" {
        world.state.set_string("spawn_kind", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.reset" {
        reset_params(world);
        return;
    }
    if topic == "playground-2d-particles.editor.print-yaml" {
        print_yaml(world);
        return;
    }
    if topic == "playground-2d-particles.editor.burst" {
        world.particles.burst(emitter(), 48);
        return;
    }
}
