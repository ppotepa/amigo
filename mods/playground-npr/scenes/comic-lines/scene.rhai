import "pkg:amigo.std/input" as input;

fn model_entity(world_api, index) {
    if index == 1 { return world_api.entities.named("playground-npr-model-1-soldier"); }
    if index == 2 { return world_api.entities.named("playground-npr-model-2-box-textured"); }
    if index == 3 { return world_api.entities.named("playground-npr-model-3-box-interleaved"); }
    if index == 4 { return world_api.entities.named("playground-npr-model-4-box-animated"); }
    if index == 5 { return world_api.entities.named("playground-npr-model-5-fox"); }
    world_api.entities.named("playground-npr-model-6-cesium-man")
}

fn model_entity_name(index) {
    if index == 1 { return "playground-npr-model-1-soldier"; }
    if index == 2 { return "playground-npr-model-2-box-textured"; }
    if index == 3 { return "playground-npr-model-3-box-interleaved"; }
    if index == 4 { return "playground-npr-model-4-box-animated"; }
    if index == 5 { return "playground-npr-model-5-fox"; }
    "playground-npr-model-6-cesium-man"
}

fn hide_all_models(world_api) {
    model_entity(world_api, 1).hide();
    model_entity(world_api, 2).hide();
    model_entity(world_api, 3).hide();
    model_entity(world_api, 4).hide();
    model_entity(world_api, 5).hide();
    model_entity(world_api, 6).hide();
}

fn active_model_name(index) {
    if index == 1 { return "Soldier"; }
    if index == 2 { return "BoxTextured"; }
    if index == 3 { return "BoxInterleaved"; }
    if index == 4 { return "BoxAnimated"; }
    if index == 5 { return "Fox"; }
    if index == 6 { return "CesiumMan"; }
    "Unknown"
}

fn npr_preset_count() {
    15
}

fn npr_style_base_id(index) {
    if index == 0 { return "default_gpu_comic"; }
    if index == 1 { return "clean_comic_ink"; }
    if index == 2 { return "technical_comic_line"; }
    if index == 3 { return "target_60fps"; }
    if index == 4 { return "balanced_30fps"; }
    if index == 5 { return "manga_fine_line"; }
    if index == 6 { return "european_clear_line"; }
    if index == 7 { return "animation_pencil"; }
    if index == 8 { return "loose_pencil"; }
    if index == 9 { return "dry_brush_ink"; }
    if index == 10 { return "heavy_noir_ink"; }
    if index == 11 { return "storyboard_marker"; }
    if index == 12 { return "cinematic_12fps"; }
    if index == 13 { return "low_120fps"; }
    "rough_comic_ink"
}

fn npr_preset_id(index, strategy) {
    let base = npr_style_base_id(index);
    if strategy == "cpu_reference" {
        return base + "_cpu_reference";
    }
    base
}

fn npr_preset_label(index) {
    if index == 0 { return "Default Comic"; }
    if index == 1 { return "Clean Comic Ink"; }
    if index == 2 { return "Technical Comic Line"; }
    if index == 3 { return "Target 60 FPS"; }
    if index == 4 { return "Balanced 30 FPS"; }
    if index == 5 { return "Manga Fine Line"; }
    if index == 6 { return "European Clear Line"; }
    if index == 7 { return "Animation Pencil"; }
    if index == 8 { return "Loose Pencil"; }
    if index == 9 { return "Dry Brush Ink"; }
    if index == 10 { return "Heavy Noir Ink"; }
    if index == 11 { return "Storyboard Marker"; }
    if index == 12 { return "Cinematic 12 FPS"; }
    if index == 13 { return "Low 120 FPS"; }
    "Rough Comic Ink"
}

fn npr_debug_view_label(view_id) {
    if view_id == "camera.final" { return "Final"; }
    if view_id == "npr.line_kinds" { return "LineKinds"; }
    if view_id == "npr.raw_paths" { return "RawPaths"; }
    if view_id == "npr.dropout" { return "Dropout"; }
    if view_id == "npr.width_alpha" { return "WidthAlpha"; }
    view_id
}

fn npr_debug_mode_from_view_id(view_id) {
    if view_id == "npr.line_kinds" { return "line_kinds"; }
    if view_id == "npr.raw_paths" { return "raw_paths"; }
    if view_id == "npr.dropout" { return "dropout"; }
    if view_id == "npr.width_alpha" { return "width_alpha"; }
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

fn refresh_hud(world_api, dt) {
    let model_index = world_api.state.get_int("active_npr_model");
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
        "Model    " + active_model_name(model_index),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.preset",
        "Preset   " + preset_index_label(preset_index) + " " + npr_preset_label(preset_index),
    );
    world_api.ui.set_text(
        "playground-npr-hud.root.strategy",
        "Strategy " + strategy + " | G toggle",
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

fn apply_active_npr_preset(world_api) {
    let model_index = world_api.state.get_int("active_npr_model");
    let preset_index = world_api.state.get_int("active_npr_preset");
    let strategy = world_api.state.get_string("active_npr_strategy");
    let preset_id = npr_preset_id(preset_index, strategy);
    let entity_name = model_entity_name(model_index);
    world_api.mesh3d.apply_npr_preset(entity_name, preset_id);
    world_api.mesh3d.set_npr_temporal_path_smoothing(
        entity_name,
        world_api.state.get_bool("npr_temporal_path_smoothing"),
    );
    world_api.mesh3d.set_npr_gpu_debug_mode(
        entity_name,
        npr_debug_mode_from_view_id(world_api.state.get_string("npr_debug_view")),
    );
    world_api.dev.event("playground-npr.preset-selected", npr_preset_label(preset_index));
    refresh_hud(world_api, 0.0);
}

fn step_npr_preset(world_api, delta) {
    let count = npr_preset_count();
    let next = (world_api.state.get_int("active_npr_preset") + delta + count) % count;
    world_api.state.set_int("active_npr_preset", next);
    apply_active_npr_preset(world_api);
}

fn select_model(world_api, index) {
    hide_all_models(world_api);
    model_entity(world_api, index).show();
    world_api.state.set_int("active_npr_model", index);
    apply_active_npr_preset(world_api);
    world_api.dev.event("playground-npr.model-selected", active_model_name(index));
    refresh_hud(world_api, 0.0);
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
    apply_active_npr_preset(world_api);
    world_api.dev.event("playground-npr.strategy-selected", strategy);
}

fn toggle_npr_strategy(world_api) {
    if world_api.state.get_string("active_npr_strategy") == "gpu_realtime" {
        set_npr_strategy(world_api, "cpu_reference");
    } else {
        set_npr_strategy(world_api, "gpu_realtime");
    }
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
        set_npr_debug_view(world_api, "npr.dropout", "Dropout");
        return;
    }
    if current == "npr.dropout" {
        set_npr_debug_view(world_api, "npr.width_alpha", "Width Alpha");
        return;
    }
    set_npr_debug_view(world_api, "camera.final", "Final");
}

fn rotate_active_model(world_api, dt) {
    if !world_api.state.get_bool("npr_model_autorotate") {
        return;
    }

    let index = world_api.state.get_int("active_npr_model");
    if index == 1 { model_entity(world_api, 1).rotate_3d(0.0, 0.32 * dt, 0.0); }
    if index == 2 { model_entity(world_api, 2).rotate_3d(0.0, 0.28 * dt, 0.0); }
    if index == 3 { model_entity(world_api, 3).rotate_3d(0.0, -0.30 * dt, 0.0); }
    if index == 4 { model_entity(world_api, 4).rotate_3d(0.0, 0.26 * dt, 0.0); }
    if index == 5 { model_entity(world_api, 5).rotate_3d(0.0, -0.18 * dt, 0.0); }
    if index == 6 { model_entity(world_api, 6).rotate_3d(0.0, 0.20 * dt, 0.0); }
}

fn on_enter() {
    world.state.reset_scene();
    world.state.set_bool("npr_camera_freelook", false);
    world.state.set_bool("npr_model_autorotate", false);
    world.state.set_bool("npr_temporal_path_smoothing", false);
    world.state.set_string("active_npr_strategy", "gpu_realtime");
    world.state.set_int("active_npr_preset", 0);
    world.state.set_string("npr_debug_view", "camera.final");
    world.camera.set_main_debug_view("camera.final");
    select_model(world, 1);
    refresh_hud(world, 0.0);
    let scene_id = world.scene.current_id();
    world.dev.event("playground-npr.scene-ready", scene_id);
    world.dev.log("playground-npr comic lines workbench: 1-6 models, ArrowUp/ArrowDown styles, G strategy toggle, V debug cycle, 7 final, 8 kinds, 9 dropout, 0 width/alpha, LMB orbit, wheel zoom, F freelook, R autorotate, T temporal smoothing");
}

fn update(dt) {
    world.state.set_float("npr_last_dt", dt);
    if input::pressed(world, "npr.select_1") { select_model(world, 1); }
    if input::pressed(world, "npr.select_2") { select_model(world, 2); }
    if input::pressed(world, "npr.select_3") { select_model(world, 3); }
    if input::pressed(world, "npr.select_4") { select_model(world, 4); }
    if input::pressed(world, "npr.select_5") { select_model(world, 5); }
    if input::pressed(world, "npr.select_6") { select_model(world, 6); }
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
