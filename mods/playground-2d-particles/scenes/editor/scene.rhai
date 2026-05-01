fn emitter() { "playground-2d-particles-editor-preview-emitter" }
fn root() { "playground-2d-particles-editor-ui.root" }
fn path(id) { root() + "." + id }
fn controls_path(id) { path("main.controls." + id) }
fn panel_path(id) { controls_path("tab-panels." + id) }
fn emission_path(id) { panel_path("emission-panel." + id) }
fn color_path(id) { panel_path("color-panel." + id) }
fn size_path(id) { panel_path("size-panel." + id) }
fn shape_path(id) { panel_path("shape-panel." + id) }
fn renderer_path(id) { panel_path("renderer-panel." + id) }
fn motion_path(id) { panel_path("motion-panel." + id) }
fn forces_path(id) { panel_path("forces-panel." + id) }
fn spawn_path(id) { panel_path("spawn-panel." + id) }
fn burst_path(id) { panel_path("burst-panel." + id) }
fn export_path(id) { panel_path("export-panel." + id) }

fn spawn_rate(world_api) { world_api.state.get_float("spawn_rate") }
fn max_particles(world_api) { world_api.state.get_int("max_particles") }
fn lifetime(world_api) { world_api.state.get_float("lifetime") }
fn speed(world_api) { world_api.state.get_float("speed") }
fn spread(world_api) { world_api.state.get_float("spread") }
fn initial_size(world_api) { world_api.state.get_float("initial_size") }
fn final_size(world_api) { world_api.state.get_float("final_size") }
fn gravity_y(world_api) { world_api.state.get_float("gravity_y") }
fn drag(world_api) { world_api.state.get_float("drag") }
fn active(world_api) { world_api.state.get_bool("active") }
fn color_name(world_api) { world_api.state.get_string("color") }
fn ramp_preset(world_api) { world_api.state.get_string("ramp_preset") }
fn ramp_interpolation(world_api) { world_api.state.get_string("ramp_interpolation") }
fn selected_tab(world_api) { world_api.state.get_string("selected_tab") }
fn spawn_kind(world_api) { world_api.state.get_string("spawn_kind") }
fn shape_mode(world_api) { world_api.state.get_string("shape_mode") }
fn shape_kind(world_api) { world_api.state.get_string("shape_kind") }
fn align_kind(world_api) { world_api.state.get_string("align_kind") }
fn blend_kind(world_api) { world_api.state.get_string("blend_kind") }
fn z_index(world_api) { world_api.state.get_float("z_index") }
fn selected_preset(world_api) { world_api.state.get_string("selected_preset") }

fn lerp(min, max, value) {
    return min + (max - min) * value;
}

fn normalize(min, max, value) {
    return (value - min) / (max - min);
}

fn bool_text(value) {
    if value { return "true"; }
    return "false";
}

fn export_id(world_api) {
    let preset = selected_preset(world_api);
    if preset == "" {
        return "editor-export";
    }
    return preset + "-edited";
}

fn export_label(world_api) {
    let label = world_api.particles.preset_label(selected_preset(world_api));
    if label == "" {
        return "Editor Export";
    }
    return label + " Edited";
}

fn export_category(world_api) {
    let category = world_api.particles.preset_category(selected_preset(world_api));
    if category == "" {
        return "custom";
    }
    return category;
}

fn export_tags(world_api) {
    let tags = world_api.particles.preset_tags(selected_preset(world_api));
    if tags == "" {
        return "[editor, custom]";
    }
    return "[" + tags + ", editor, edited]";
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

fn ramp_stop_color(preset, index) {
    if preset == "Fire" {
        if index == 0 { return "#FFFFFFFF"; }
        if index == 1 { return "#FFE680FF"; }
        if index == 2 { return "#FF5A1FCC"; }
        return "#30100000";
    }
    if preset == "Smoke" {
        if index == 0 { return "#40405000"; }
        if index == 1 { return "#A0A0A066"; }
        if index == 2 { return "#C8C8D055"; }
        return "#20202800";
    }
    if preset == "Plasma" {
        if index == 0 { return "#FFFFFFFF"; }
        if index == 1 { return "#39D7FFFF"; }
        if index == 2 { return "#236DFFFF"; }
        return "#0A1A6600";
    }
    if preset == "Portal" {
        if index == 0 { return "#7E5BFF00"; }
        if index == 1 { return "#7E5BFFFF"; }
        if index == 2 { return "#39D7FFFF"; }
        return "#F8F8F800";
    }
    if preset == "Snow" {
        if index == 0 { return "#F8F8F8FF"; }
        if index == 1 { return "#DDEBFFFF"; }
        if index == 2 { return "#BFD7FFFF"; }
        return "#F8F8F800";
    }
    return "#39D7FFFF";
}

fn apply_color_ramp(world_api) {
    let preset = ramp_preset(world_api);
    if preset == "None" || preset == "" {
        world_api.particles.clear_color_ramp(emitter());
        apply_color(world_api, color_name(world_api));
        return;
    }
    world_api.particles.set_color_ramp4(
        emitter(),
        ramp_interpolation(world_api),
        0.0, ramp_stop_color(preset, 0),
        0.25, ramp_stop_color(preset, 1),
        0.70, ramp_stop_color(preset, 2),
        1.0, ramp_stop_color(preset, 3)
    );
}

fn apply_params(world_api) {
    world_api.particles.set_active(emitter(), active(world_api));
    world_api.particles.set_spawn_rate(emitter(), spawn_rate(world_api));
    world_api.particles.set_max_particles(emitter(), max_particles(world_api));
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
    if shape_mode(world_api) == "random_mix" {
        world_api.particles.set_shape_mix(emitter(), 2.0, 1.0, 1.0);
    } else {
        if shape_kind(world_api) == "line" {
            world_api.particles.set_shape_line(emitter(), 14.0);
        } else if shape_kind(world_api) == "quad" {
            world_api.particles.set_shape_quad(emitter());
        } else {
            world_api.particles.set_shape_circle(emitter(), 8);
        }
    }
    world_api.particles.set_align(emitter(), align_kind(world_api));
    world_api.particles.set_blend_mode(emitter(), blend_kind(world_api));
    world_api.particles.set_z_index(emitter(), z_index(world_api));
    apply_color_ramp(world_api);
}

fn sync_ui(world_api) {
    world_api.ui.set_options(controls_path("preset-row.preset-dropdown"), world_api.particles.preset_ids());
    world_api.ui.set_selected(controls_path("preset-row.preset-dropdown"), selected_preset(world_api));
    let preset_label = world_api.particles.preset_label(selected_preset(world_api));
    let preset_category = world_api.particles.preset_category(selected_preset(world_api));
    let preset_tags = world_api.particles.preset_tags(selected_preset(world_api));
    world_api.ui.set_text(controls_path("preset-meta"), "Preset: " + preset_label + " / " + preset_category + " / " + preset_tags);
    world_api.ui.set_selected(controls_path("tabs"), selected_tab(world_api));
    world_api.ui.set_text(controls_path("tab-title"), selected_tab(world_api) + " controls");
    world_api.ui.set_value(emission_path("active-toggle"), if active(world_api) { 1.0 } else { 0.0 });
    world_api.ui.set_value(emission_path("spawn-rate"), normalize(0.0, 200.0, spawn_rate(world_api)));
    world_api.ui.set_value(emission_path("max-particles"), normalize(16.0, 320.0, max_particles(world_api)));
    world_api.ui.set_value(emission_path("lifetime"), normalize(0.1, 1.8, lifetime(world_api)));
    world_api.ui.set_value(motion_path("speed"), normalize(0.0, 300.0, speed(world_api)));
    world_api.ui.set_value(motion_path("spread"), normalize(0.0, 180.0, spread(world_api)));
    world_api.ui.set_value(size_path("sizes.initial-size"), normalize(0.0, 20.0, initial_size(world_api)));
    world_api.ui.set_value(size_path("sizes.final-size"), normalize(0.0, 20.0, final_size(world_api)));
    world_api.ui.set_selected(shape_path("shape-row.shape-kind"), shape_kind(world_api));
    world_api.ui.set_selected(shape_path("shape-row.align-kind"), align_kind(world_api));
    world_api.ui.set_selected(shape_path("shape-mode"), shape_mode(world_api));
    world_api.ui.set_selected(renderer_path("blend-row.blend-kind"), blend_kind(world_api));
    world_api.ui.set_value(renderer_path("z-index"), normalize(-20.0, 80.0, z_index(world_api)));
    world_api.ui.set_value(forces_path("gravity-y"), normalize(-300.0, 300.0, gravity_y(world_api)));
    world_api.ui.set_value(forces_path("drag"), normalize(0.0, 5.0, drag(world_api)));
    world_api.ui.set_selected(spawn_path("spawn-kind"), spawn_kind(world_api));
    world_api.ui.set_selected(color_path("color-row.color-dropdown"), color_name(world_api));
    world_api.ui.set_background(color_path("color-row.color-swatch"), color_hex(color_name(world_api)));
    world_api.ui.set_selected(color_path("ramp-row.ramp-preset"), ramp_preset(world_api));
    world_api.ui.set_selected(color_path("ramp-row.ramp-interpolation"), ramp_interpolation(world_api));
    world_api.ui.set_background(color_path("ramp-preview.stop-0"), ramp_stop_color(ramp_preset(world_api), 0));
    world_api.ui.set_background(color_path("ramp-preview.stop-1"), ramp_stop_color(ramp_preset(world_api), 1));
    world_api.ui.set_background(color_path("ramp-preview.stop-2"), ramp_stop_color(ramp_preset(world_api), 2));
    world_api.ui.set_background(color_path("ramp-preview.stop-3"), ramp_stop_color(ramp_preset(world_api), 3));

    world_api.ui.set_text(emission_path("spawn-rate-label"), "Spawn rate " + spawn_rate(world_api).to_int());
    world_api.ui.set_text(emission_path("max-particles-label"), "Max particles " + max_particles(world_api));
    world_api.ui.set_text(emission_path("lifetime-label"), "Lifetime " + lifetime(world_api));
    world_api.ui.set_text(motion_path("speed-label"), "Speed " + speed(world_api).to_int());
    world_api.ui.set_text(motion_path("spread-label"), "Spread " + spread(world_api).to_int());
    world_api.ui.set_text(size_path("size-label"), "Size " + initial_size(world_api).to_int() + " -> " + final_size(world_api).to_int());
    world_api.ui.set_text(shape_path("shape-label"), "Shape " + shape_kind(world_api) + " / Mode " + shape_mode(world_api) + " / Align " + align_kind(world_api));
    world_api.ui.set_text(renderer_path("renderer-label"), "Blend " + blend_kind(world_api) + " / z-index " + z_index(world_api).to_int());
    world_api.ui.set_text(forces_path("gravity-label"), "Gravity Y " + gravity_y(world_api).to_int());
    world_api.ui.set_text(forces_path("drag-label"), "Drag " + drag(world_api));
    world_api.ui.set_text(spawn_path("spawn-label"), "Spawn area " + spawn_kind(world_api));
    world_api.ui.set_text(color_path("ramp-label"), "ColorRamp " + ramp_preset(world_api) + " / " + ramp_interpolation(world_api));
}

fn sync_tabs(world_api) {
    let tab = selected_tab(world_api);
    if tab == "" {
        tab = "Emission";
        world_api.state.set_string("selected_tab", tab);
    }
    let tabs = ["Emission", "Color", "Size", "Shape", "Renderer", "Motion", "Forces", "Spawn", "Burst", "Export"];
    let panels = ["emission-panel", "color-panel", "size-panel", "shape-panel", "renderer-panel", "motion-panel", "forces-panel", "spawn-panel", "burst-panel", "export-panel"];
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
    world_api.state.set_int("max_particles", 160);
    world_api.state.set_float("lifetime", 0.65);
    world_api.state.set_float("speed", 100.0);
    world_api.state.set_float("spread", 28.0);
    world_api.state.set_float("initial_size", 4.0);
    world_api.state.set_float("final_size", 13.0);
    world_api.state.set_float("gravity_y", 0.0);
    world_api.state.set_float("drag", 0.4);
    world_api.state.set_string("color", "Cyan");
    world_api.state.set_string("ramp_preset", "Plasma");
    world_api.state.set_string("ramp_interpolation", "linear_rgb");
    world_api.state.set_string("spawn_kind", "circle");
    world_api.state.set_string("shape_mode", "single");
    world_api.state.set_string("shape_kind", "circle");
    world_api.state.set_string("align_kind", "velocity");
    world_api.state.set_string("blend_kind", "alpha");
    world_api.state.set_float("z_index", 24.0);
    world_api.state.set_string("selected_preset", "plasma");
    world_api.state.set_string("selected_tab", "Emission");
    apply_params(world_api);
    sync_tabs(world_api);
    sync_ui(world_api);
}

fn export_yaml_text(world_api) {
    let yaml = "";
    yaml += "kind: particle-preset-2d\n";
    yaml += "id: " + export_id(world_api) + "\n";
    yaml += "label: " + export_label(world_api) + "\n";
    yaml += "category: " + export_category(world_api) + "\n";
    yaml += "tags: " + export_tags(world_api) + "\n";
    yaml += "emitter:\n";
    yaml += "  type: ParticleEmitter2D\n";
    yaml += "  active: " + bool_text(active(world_api)) + "\n";
    yaml += "  spawn_rate: " + spawn_rate(world_api) + "\n";
    yaml += "  max_particles: " + max_particles(world_api) + "\n";
    yaml += "  particle_lifetime: " + lifetime(world_api) + "\n";
    yaml += "  lifetime_jitter: 0.08\n";
    yaml += "  initial_speed: " + speed(world_api) + "\n";
    yaml += "  speed_jitter: 24.0\n";
    yaml += "  spread_degrees: " + spread(world_api) + "\n";
    yaml += "  initial_size: " + initial_size(world_api) + "\n";
    yaml += "  final_size: " + final_size(world_api) + "\n";
    yaml += "  color: \"" + color_hex(color_name(world_api)) + "\"\n";
    if ramp_preset(world_api) != "None" {
        yaml += "  color_ramp:\n";
        yaml += "    interpolation: " + ramp_interpolation(world_api) + "\n";
        yaml += "    stops:\n";
        yaml += "      - { t: 0.0, color: \"" + ramp_stop_color(ramp_preset(world_api), 0) + "\" }\n";
        yaml += "      - { t: 0.25, color: \"" + ramp_stop_color(ramp_preset(world_api), 1) + "\" }\n";
        yaml += "      - { t: 0.70, color: \"" + ramp_stop_color(ramp_preset(world_api), 2) + "\" }\n";
        yaml += "      - { t: 1.0, color: \"" + ramp_stop_color(ramp_preset(world_api), 3) + "\" }\n";
    }
    yaml += "  forces:\n";
    yaml += "    - kind: gravity\n";
    yaml += "      acceleration: { x: 0.0, y: " + gravity_y(world_api) + " }\n";
    yaml += "    - kind: drag\n";
    yaml += "      coefficient: " + drag(world_api) + "\n";
    yaml += "  spawn_area:\n";
    yaml += "    kind: " + spawn_kind(world_api) + "\n";
    if spawn_kind(world_api) == "line" {
        yaml += "    length: 120.0\n";
    } else if spawn_kind(world_api) == "ring" {
        yaml += "    inner_radius: 24.0\n";
        yaml += "    outer_radius: 52.0\n";
    } else if spawn_kind(world_api) == "circle" {
        yaml += "    radius: 8.0\n";
    }
    yaml += "  shape:\n";
    yaml += "    kind: " + shape_kind(world_api) + "\n";
    if shape_kind(world_api) == "circle" {
        yaml += "    segments: 8\n";
    } else if shape_kind(world_api) == "line" {
        yaml += "    length: 14.0\n";
    }
    if shape_mode(world_api) == "random_mix" {
        yaml += "  shape_choices:\n";
        yaml += "    - weight: 2.0\n";
        yaml += "      shape: { kind: circle, segments: 8 }\n";
        yaml += "    - weight: 1.0\n";
        yaml += "      shape: { kind: line, length: 14.0 }\n";
        yaml += "    - weight: 1.0\n";
        yaml += "      shape: { kind: quad }\n";
    }
    yaml += "  align: " + align_kind(world_api) + "\n";
    yaml += "  blend_mode: " + blend_kind(world_api) + "\n";
    yaml += "  z_index: " + z_index(world_api) + "\n";
    yaml += "  emission_rate_curve: { kind: constant, value: 1.0 }\n";
    yaml += "  size_curve: { kind: ease_out }\n";
    yaml += "  alpha_curve:\n";
    yaml += "    kind: custom\n";
    yaml += "    points:\n";
    yaml += "      - { t: 0.0, value: 0.90 }\n";
    yaml += "      - { t: 0.70, value: 0.45 }\n";
    yaml += "      - { t: 1.0, value: 0.0 }\n";
    yaml += "  speed_curve: { kind: ease_out }\n";
    return yaml;
}

fn print_yaml(world_api) {
    let yaml = export_yaml_text(world_api);
    world_api.dev.log("--- particle preset export begin ---");
    world_api.dev.log(yaml);
    world_api.dev.write_text("particle-presets/" + export_id(world_api) + ".yml", yaml);
    world_api.dev.log("--- particle preset export end ---");
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
    if topic == "playground-2d-particles.editor.preset" {
        world.state.set_string("selected_preset", payload[0]);
        world.particles.apply_preset(payload[0], emitter());
        world.particles.set_active(emitter(), active(world));
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.spawn-rate" {
        world.state.set_float("spawn_rate", lerp(0.0, 200.0, payload[0].to_float()));
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.max-particles" {
        world.state.set_int("max_particles", lerp(16.0, 320.0, payload[0].to_float()).to_int());
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
    if topic == "playground-2d-particles.editor.shape-kind" {
        world.state.set_string("shape_kind", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.shape-mode" {
        world.state.set_string("shape_mode", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.align-kind" {
        world.state.set_string("align_kind", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.blend-kind" {
        world.state.set_string("blend_kind", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.z-index" {
        world.state.set_float("z_index", lerp(-20.0, 80.0, payload[0].to_float()));
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
        world.state.set_string("ramp_preset", "None");
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.ramp-preset" {
        world.state.set_string("ramp_preset", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.ramp-interpolation" {
        world.state.set_string("ramp_interpolation", payload[0]);
        apply_params(world);
        sync_ui(world);
        return;
    }
    if topic == "playground-2d-particles.editor.rgb-color" {
        world.state.set_string("ramp_preset", "None");
        world.particles.set_color_rgba(
            emitter(),
            payload[0].to_float(),
            payload[1].to_float(),
            payload[2].to_float(),
            1.0
        );
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
