import "pkg:amigo.std/input" as input;

fn model_entity(world_api, index) {
    world_api.entities.named("playground-npr-model-1-soldier")
}

fn model_entity_name(index) {
    "playground-npr-model-1-soldier"
}

fn hide_all_models(world_api) {
    model_entity(world_api, 1).hide();
    model_entity(world_api, 2).hide();
}

fn active_model_name(world_api, index) {
    let label = world_api.mesh3d.model_label(index - 1);
    if label != "" { return label; }
    "Soldier"
}

fn model_library_count(world_api) {
    let count = world_api.mesh3d.model_count();
    if count > 0 { return count; }
    1
}

fn animation_count(model_index) {
    1
}

fn animation_label(model_index, animation_index) {
    if model_index == 2 {
        return "Static mesh";
    }
    "Static"
}

fn animation_status_label(model_index) {
    if model_index == 2 {
        return "skinning unsupported in V1";
    }
    "no clips";
}

fn npr_preset_count() {
    1
}

fn npr_style_base_id(index) {
    "toriyama_ink_cpu_reference"
}

fn npr_preset_id(index) {
    npr_style_base_id(index)
}

fn npr_preset_label(index) {
    "Toriyama Ink"
}

fn npr_debug_view_label(view_id) {
    if view_id == "camera.final" { return "Final"; }
    if view_id == "npr.line_kinds" { return "LineKinds"; }
    if view_id == "npr.raw_paths" { return "RawPaths"; }
    if view_id == "npr.candidate_importance" { return "Importance"; }
    if view_id == "npr.technical_selection" { return "Technical"; }
    if view_id == "npr.stroke_length_bucket" { return "Length"; }
    if view_id == "npr.source_edge_count" { return "SourceEdges"; }
    if view_id == "npr.stroke_roles" { return "StrokeRoles"; }
    if view_id == "npr.material_roles" { return "MaterialRoles"; }
    if view_id == "npr.dropout" { return "Dropout"; }
    if view_id == "npr.width_alpha" { return "WidthAlpha"; }
    if view_id == "npr.chain_hops" { return "ChainHops"; }
    view_id
}

fn npr_debug_mode_from_view_id(view_id) {
    if view_id == "npr.line_kinds" { return "line_kinds"; }
    if view_id == "npr.raw_paths" { return "raw_paths"; }
    if view_id == "npr.candidate_importance" { return "candidate_importance"; }
    if view_id == "npr.technical_selection" { return "technical_selection"; }
    if view_id == "npr.stroke_length_bucket" { return "stroke_length_bucket"; }
    if view_id == "npr.source_edge_count" { return "source_edge_count"; }
    if view_id == "npr.stroke_roles" { return "stroke_roles"; }
    if view_id == "npr.material_roles" { return "material_roles"; }
    if view_id == "npr.dropout" { return "dropout"; }
    if view_id == "npr.width_alpha" { return "width_alpha"; }
    if view_id == "npr.chain_hops" { return "chain_hops"; }
    "final"
}

fn on_off(value) {
    if value { return "on"; }
    "off"
}

fn camera_mode_label(world_api) {
    if world_api.state.get_bool("npr_camera_freelook") {
        return "Freelook";
    }
    "Orbit"
}

fn frame_debug_line(dt) {
    let frame_ms = (dt * 1000.0).to_int();
    let fps = if dt > 0.00001 { (1.0 / dt).to_int() } else { 0 };
    "Frame    " + frame_ms + " ms | " + fps + " fps"
}

fn preset_index_label(index) {
    let one_based = index + 1;
    let prefix = if one_based < 10 { "0" } else { "" };
    prefix + one_based + "/" + npr_preset_count()
}

fn clamp_value(value, minimum, maximum) {
    if value < minimum { return minimum; }
    if value > maximum { return maximum; }
    value
}

fn default_model_scale() {
    1.55
}

fn active_model_scale(world_api) {
    let scale = world_api.state.get_float("npr_model_scale");
    if scale <= 0.0 { return default_model_scale(); }
    scale
}

fn active_model_scale_percent(world_api) {
    (active_model_scale(world_api) * 100.0).to_int()
}

fn apply_active_model_scale(world_api) {
    let scale = active_model_scale(world_api);
    world_api.entities.set_scale_3d(
        model_entity_name(world_api.state.get_int("active_npr_model")),
        scale,
        scale,
        scale,
    );
}

fn step_model_scale(world_api, multiplier) {
    let next = clamp_value(active_model_scale(world_api) * multiplier, 0.35, 4.0);
    world_api.state.set_float("npr_model_scale", next);
    apply_active_model_scale(world_api);
    world_api.dev.event("playground-npr.model-scale", "" + active_model_scale_percent(world_api) + "%");
    refresh_hud(world_api, 0.0);
}

fn refresh_hud(world_api, dt) {
    let model_index = world_api.state.get_int("active_npr_model");
    let animation_index = world_api.state.get_int("active_npr_animation");
    let preset_index = world_api.state.get_int("active_npr_preset");
    let strategy = world_api.state.get_string("active_npr_strategy");
    let debug_view = world_api.state.get_string("npr_debug_view");
    let input_map = input::active_map(world_api);
    let runtime_backend = world_api.runtime.render_backend();
    let dev_mode = on_off(world_api.runtime.dev_mode());
    let npr_gpu_meshes = world_api.runtime.npr_gpu_meshes();
    let npr_cpu_meshes = world_api.runtime.npr_cpu_meshes();
    let npr_paths = world_api.runtime.npr_paths();
    let npr_gpu_edges = world_api.runtime.npr_gpu_edges();
    let npr_gpu_triangles = world_api.runtime.npr_gpu_triangles();
    let npr_path_states_capacity = world_api.runtime.npr_path_states_capacity();
    let npr_path_segments_capacity = world_api.runtime.npr_path_segments_capacity();
    let npr_stroke_segments_capacity = world_api.runtime.npr_stroke_segments_capacity();
    let npr_debug_mode = world_api.runtime.npr_debug_mode();

    world_api.ui.set_text(
        "playground-npr-hud.root.model",
        "Model    " + active_model_name(world_api, model_index),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.preset",
        "Preset   " + preset_index_label(preset_index) + " " + npr_preset_label(preset_index),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.animation",
        "Library  " + model_index + "/" + model_library_count(world_api) + " | scale " + active_model_scale_percent(world_api) + "% | <> GLB | +/-",
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.strategy",
        "Backend  " + strategy + " | CPU only",
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.camera",
        "Camera   " + camera_mode_label(world_api) + " | auto " + on_off(world_api.state.get_bool("npr_model_autorotate")),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.debug",
        "Debug    " + npr_debug_view_label(debug_view) + " | Temporal " + on_off(world_api.state.get_bool("npr_temporal_path_smoothing")),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.frame",
        frame_debug_line(dt),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.runtime",
        "Runtime  " + runtime_backend + " | dev " + dev_mode + " | map " + input_map,
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.stats_a",
        "NPR      gpu " + npr_gpu_meshes + " cpu " + npr_cpu_meshes + " | paths " + npr_paths + " | mode " + npr_debug_mode,
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.stats_b",
        "GPU      edges " + npr_gpu_edges + " tris " + npr_gpu_triangles + " | buffers " + npr_path_states_capacity + "/" + npr_path_segments_capacity + "/" + npr_stroke_segments_capacity,
    );
}

fn apply_active_npr_runtime_overrides(world_api, entity_name) {
    let strategy = world_api.state.get_string("active_npr_strategy");
    world_api.mesh3d.set_npr_render_strategy(entity_name, strategy);
    world_api.mesh3d.set_npr_temporal_path_smoothing(
        entity_name,
        world_api.state.get_bool("npr_temporal_path_smoothing"),
    );
    world_api.mesh3d.set_npr_gpu_debug_mode(
        entity_name,
        npr_debug_mode_from_view_id(world_api.state.get_string("npr_debug_view")),
    );
}

fn apply_active_npr_preset(world_api) {
    let model_index = world_api.state.get_int("active_npr_model");
    let preset_index = world_api.state.get_int("active_npr_preset");
    let preset_id = npr_preset_id(preset_index);
    let entity_name = model_entity_name(model_index);
    world_api.mesh3d.apply_npr_preset(entity_name, preset_id);
    apply_active_npr_runtime_overrides(world_api, entity_name);
    world_api.dev.event("playground-npr.preset-selected", npr_preset_label(preset_index));
    refresh_hud(world_api, 0.0);
}

fn step_npr_preset(world_api, delta) {
    let count = npr_preset_count();
    let next = (world_api.state.get_int("active_npr_preset") + delta + count) % count;
    world_api.state.set_int("active_npr_preset", next);
    apply_active_npr_preset(world_api);
}

fn step_model_animation(world_api, delta) {
    let model_index = world_api.state.get_int("active_npr_model");
    let count = animation_count(model_index);
    let next = (world_api.state.get_int("active_npr_animation") + delta + count) % count;
    world_api.state.set_int("active_npr_animation", next);
    world_api.dev.event("playground-npr.animation-unavailable", animation_status_label(model_index));
    refresh_hud(world_api, 0.0);
}

fn select_model(world_api, index) {
    let count = model_library_count(world_api);
    let safe_index = ((index - 1 + count) % count) + 1;
    hide_all_models(world_api);
    model_entity(world_api, safe_index).show();
    let mesh_asset = world_api.mesh3d.model_asset(safe_index - 1);
    if mesh_asset != "" {
        world_api.mesh3d.set_mesh(model_entity_name(safe_index), mesh_asset);
    }
    world_api.state.set_int("active_npr_model", safe_index);
    world_api.state.set_int("active_npr_animation", 0);
    apply_active_model_scale(world_api);
    apply_active_npr_preset(world_api);
    world_api.dev.event("playground-npr.model-selected", active_model_name(world_api, safe_index));
    refresh_hud(world_api, 0.0);
}

fn step_model(world_api, delta) {
    select_model(world_api, world_api.state.get_int("active_npr_model") + delta);
}

fn toggle_model_autorotate(world_api) {
    let enabled = !world_api.state.get_bool("npr_model_autorotate");
    world_api.state.set_bool("npr_model_autorotate", enabled);
    if enabled {
        world_api.dev.event("playground-npr.model-autorotate", "on");
    } else {
        world_api.dev.event("playground-npr.model-autorotate", "off");
    }
    refresh_hud(world_api, 0.0);
}

fn toggle_npr_temporal_path_smoothing(world_api) {
    let enabled = !world_api.state.get_bool("npr_temporal_path_smoothing");
    world_api.state.set_bool("npr_temporal_path_smoothing", enabled);
    world_api.mesh3d.set_npr_temporal_path_smoothing(
        model_entity_name(world_api.state.get_int("active_npr_model")),
        enabled,
    );
    if enabled {
        world_api.dev.event("playground-npr.temporal-path-smoothing", "on");
    } else {
        world_api.dev.event("playground-npr.temporal-path-smoothing", "off");
    }
    refresh_hud(world_api, 0.0);
}

fn set_npr_strategy(world_api, strategy) {
    world_api.state.set_string("active_npr_strategy", strategy);
    apply_active_npr_runtime_overrides(
        world_api,
        model_entity_name(world_api.state.get_int("active_npr_model")),
    );
    world_api.dev.event("playground-npr.strategy-selected", strategy);
    refresh_hud(world_api, 0.0);
}

fn toggle_npr_strategy(world_api) {
    set_npr_strategy(world_api, "cpu_reference");
    world_api.dev.event("playground-npr.strategy-locked", "cpu_reference");
}

fn set_npr_debug_view(world_api, view_id, label) {
    world_api.camera.set_main_debug_view(view_id);
    world_api.state.set_string("npr_debug_view", view_id);
    world_api.mesh3d.set_npr_gpu_debug_mode(
        model_entity_name(world_api.state.get_int("active_npr_model")),
        npr_debug_mode_from_view_id(view_id),
    );
    world_api.dev.event("playground-npr.debug-view", label);
    refresh_hud(world_api, 0.0);
}

fn cycle_npr_debug_view(world_api) {
    let current = world_api.state.get_string("npr_debug_view");
    if current == "camera.final" {
        set_npr_debug_view(world_api, "npr.line_kinds", "Line Kinds");
        return;
    }
    if current == "npr.line_kinds" {
        set_npr_debug_view(world_api, "npr.raw_paths", "Raw Paths");
        return;
    }
    if current == "npr.raw_paths" {
        set_npr_debug_view(world_api, "npr.candidate_importance", "Importance");
        return;
    }
    if current == "npr.candidate_importance" {
        set_npr_debug_view(world_api, "npr.technical_selection", "Technical");
        return;
    }
    if current == "npr.technical_selection" {
        set_npr_debug_view(world_api, "npr.stroke_length_bucket", "Length");
        return;
    }
    if current == "npr.stroke_length_bucket" {
        set_npr_debug_view(world_api, "npr.source_edge_count", "Source Edges");
        return;
    }
    if current == "npr.source_edge_count" {
        set_npr_debug_view(world_api, "npr.stroke_roles", "Stroke Roles");
        return;
    }
    if current == "npr.stroke_roles" {
        set_npr_debug_view(world_api, "npr.material_roles", "Material Roles");
        return;
    }
    if current == "npr.material_roles" {
        set_npr_debug_view(world_api, "npr.dropout", "Dropout");
        return;
    }
    if current == "npr.dropout" {
        set_npr_debug_view(world_api, "npr.width_alpha", "Width Alpha");
        return;
    }
    if current == "npr.width_alpha" {
        set_npr_debug_view(world_api, "npr.chain_hops", "Chain Hops");
        return;
    }
    set_npr_debug_view(world_api, "camera.final", "Final");
}

fn rotate_active_model(world_api, dt) {
    if !world_api.state.get_bool("npr_model_autorotate") {
        return;
    }

    model_entity(world_api, world_api.state.get_int("active_npr_model")).rotate_3d(0.0, 0.32 * dt, 0.0);
}

fn on_enter() {
    world.state.reset_scene();
    let scanned = world.mesh3d.scan_models("playground-npr");
    world.dev.event("playground-npr.models-scanned", "" + scanned);
    world.state.set_bool("npr_camera_freelook", false);
    world.state.set_bool("npr_model_autorotate", false);
    world.state.set_bool("npr_temporal_path_smoothing", false);
    world.state.set_string("active_npr_strategy", "cpu_reference");
    world.state.set_int("active_npr_preset", 0);
    world.state.set_int("active_npr_animation", 0);
    world.state.set_float("npr_model_scale", default_model_scale());
    world.state.set_string("npr_debug_view", "camera.final");
    world.camera.set_main_debug_view("camera.final");
    let initial_model = world.mesh3d.model_index_by_label("soldier");
    if initial_model >= 0 {
        select_model(world, initial_model + 1);
    } else {
        select_model(world, 1);
    }
    refresh_hud(world, 0.0);
    let scene_id = world.scene.current_id();
    world.dev.event("playground-npr.scene-ready", scene_id);
    world.dev.log("playground-npr comic lines workbench: CPU reference NPR only, recursive GLB model library under source-models, </> switch models, +/- scale model, ArrowUp/ArrowDown styles, G reasserts CPU, V debug cycle, 7 final, 8 kinds, 9 dropout, 0 width/alpha, LMB orbit, Shift+LMB or MMB pan, wheel zoom, F freelook, WASD/Space/Q fly, Shift fast fly, R autorotate, T temporal smoothing; GLB skinned clips are not executed in V1");
}

fn update(dt) {
    world.state.set_float("npr_last_dt", dt);
    if input::pressed(world, "npr.select_1") { select_model(world, 1); }
    if input::pressed(world, "npr.select_2") { select_model(world, 2); }
    if input::pressed(world, "npr.model_previous") { step_model(world, -1); }
    if input::pressed(world, "npr.model_next") { step_model(world, 1); }
    if input::pressed(world, "npr.model_scale_down") { step_model_scale(world, 0.9090909); }
    if input::pressed(world, "npr.model_scale_up") { step_model_scale(world, 1.1); }
    if input::pressed(world, "npr.camera_toggle") {
        world.state.set_bool("npr_camera_freelook", !world.state.get_bool("npr_camera_freelook"));
        refresh_hud(world, dt);
    }
    if input::pressed(world, "npr.preset_next") { step_npr_preset(world, 1); }
    if input::pressed(world, "npr.preset_previous") { step_npr_preset(world, -1); }
    if input::pressed(world, "npr.model_autorotate_toggle") { toggle_model_autorotate(world); }
    if input::pressed(world, "npr.temporal_smoothing_toggle") { toggle_npr_temporal_path_smoothing(world); }
    if input::pressed(world, "npr.strategy_toggle") { toggle_npr_strategy(world); }
    if input::pressed(world, "npr.debug_cycle") { cycle_npr_debug_view(world); }
    if input::pressed(world, "npr.debug_final") { set_npr_debug_view(world, "camera.final", "Final"); }
    if input::pressed(world, "npr.debug_line_kinds") { set_npr_debug_view(world, "npr.line_kinds", "Line Kinds"); }
    if input::pressed(world, "npr.debug_dropout") { set_npr_debug_view(world, "npr.dropout", "Dropout"); }
    if input::pressed(world, "npr.debug_width_alpha") { set_npr_debug_view(world, "npr.width_alpha", "Width Alpha"); }
    world.state.set_string("npr_last_active_input_map", input::active_map(world));
    rotate_active_model(world, dt);
    refresh_hud(world, dt);
}
