import "mod:main_menu/camera" as camera;
import "mod:main_menu/lighting" as lighting;
import "mod:main_menu/timeline" as timeline;
import "mod:main_menu/title" as title;
import "mod:main_menu/ui" as ui;
import "mod:main_menu/weather" as weather;

fn enter(world_api) {
    world_api.state.reset_scene();

    world_api.state.set_float("intro.t", 0.0);
    world_api.state.set_bool("intro.complete", false);
    ui::seed_menu_state(world_api);
    camera::apply_presets(world_api);
    camera::apply_dof_controls(world_api);
    lighting::reset(world_api);
    title::reset(world_api);
    weather::reset(world_api);
    camera::reset_intro(world_api);
    ui::reset(world_api);

    world_api.dev.log("Rotten Club main menu loaded.");
}

fn update(world_api, dt) {
    let time = world_api.state.get_float("intro.t") + dt;
    world_api.state.set_float("intro.t", time);

    camera::update_motion(world_api, time);
    camera::update_preset_input(world_api);
    camera::update_dof_controls(world_api, dt);
    camera::update_bokeh(world_api, time);
    camera::update_focus(world_api, time);

    let lights = lighting::update(world_api, time);
    title::update(world_api, time);
    camera::update_title_optics(world_api, time);
    weather::update(world_api, time, lights[0], lights[1], lights[2]);

    if time >= timeline::complete_s() {
        world_api.state.set_bool("intro.complete", true);
    }
    ui::update(world_api, time);
}

fn event(world_api, topic) {
    ui::event(world_api, topic);
}
