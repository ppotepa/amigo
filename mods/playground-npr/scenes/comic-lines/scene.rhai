import "pkg:amigo.std/input" as input;

fn model_entity(world_api, index) {
    if index == 1 { return world_api.entities.named("playground-npr-model-1-soldier"); }
    if index == 2 { return world_api.entities.named("playground-npr-model-2-box-textured"); }
    if index == 3 { return world_api.entities.named("playground-npr-model-3-box-interleaved"); }
    if index == 4 { return world_api.entities.named("playground-npr-model-4-box-animated"); }
    if index == 5 { return world_api.entities.named("playground-npr-model-5-fox"); }
    world_api.entities.named("playground-npr-model-6-cesium-man")
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

fn select_model(world_api, index) {
    hide_all_models(world_api);
    model_entity(world_api, index).show();
    world_api.state.set_int("active_npr_model", index);
    world_api.dev.event("playground-npr.model-selected", active_model_name(index));
}

fn toggle_model_autorotate(world_api) {
    let enabled = !world_api.state.get_bool("npr_model_autorotate");
    world_api.state.set_bool("npr_model_autorotate", enabled);
    if enabled {
        world_api.dev.event("playground-npr.model-autorotate", "on");
    } else {
        world_api.dev.event("playground-npr.model-autorotate", "off");
    }
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
    world.state.set_bool("npr_model_autorotate", false);
    select_model(world, 1);
    let scene_id = world.scene.current_id();
    world.dev.event("playground-npr.scene-ready", scene_id);
    world.dev.log("playground-npr comic lines workbench: 1-6 switch models, LMB orbit, wheel zoom, F freelook, R autorotate");
}

fn update(dt) {
    world.state.set_float("npr_last_dt", dt);
    if input::pressed(world, "npr.select_1") { select_model(world, 1); }
    if input::pressed(world, "npr.select_2") { select_model(world, 2); }
    if input::pressed(world, "npr.select_3") { select_model(world, 3); }
    if input::pressed(world, "npr.select_4") { select_model(world, 4); }
    if input::pressed(world, "npr.select_5") { select_model(world, 5); }
    if input::pressed(world, "npr.select_6") { select_model(world, 6); }
    if input::pressed(world, "npr.model_autorotate_toggle") { toggle_model_autorotate(world); }
    world.state.set_string("npr_last_active_input_map", input::active_map(world));
    rotate_active_model(world, dt);
}
