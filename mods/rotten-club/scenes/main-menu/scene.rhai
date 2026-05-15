import "mod:fx/ids" as ids;
import "mod:fx/menu_state" as menu_state;
import "mod:fx/neon_alley" as neon;
import "mod:fx/rain" as rain;
import "mod:fx/rain_lens" as rain_lens;

fn clamp01(value) {
    if value < 0.0 {
        return 0.0;
    }
    if value > 1.0 {
        return 1.0;
    }
    value
}

fn prompt_color(alpha) {
    let a = clamp01(alpha);
    if a < 0.03125 { return "#7D879600"; }
    if a < 0.09375 { return "#7D879610"; }
    if a < 0.15625 { return "#7D879620"; }
    if a < 0.21875 { return "#7D879630"; }
    if a < 0.28125 { return "#7D879640"; }
    if a < 0.34375 { return "#7D879650"; }
    if a < 0.40625 { return "#7D879660"; }
    if a < 0.46875 { return "#7D879670"; }
    if a < 0.53125 { return "#7D879680"; }
    if a < 0.59375 { return "#7D879690"; }
    if a < 0.65625 { return "#7D8796A0"; }
    if a < 0.71875 { return "#7D8796B0"; }
    if a < 0.78125 { return "#7D8796C0"; }
    if a < 0.84375 { return "#7D8796D0"; }
    if a < 0.90625 { return "#7D8796E0"; }
    if a < 0.96875 { return "#7D8796F0"; }
    "#7D8796FF"
}

fn update_camera_sway(world_api, time) {
    let x = (time * 0.41).sin() * 10.0
        + (time * 1.13).sin() * 3.2
        + (time * 2.70).sin() * 1.1;
    let y = (time * 0.36).cos() * 6.5
        + (time * 0.97).sin() * 2.4
        + (time * 2.35).cos() * 0.9;
    let rotation = (time * 0.31).sin() * 0.0055
        + (time * 1.47).sin() * 0.0018;
    world_api.entities.set_position_2d(ids::camera_entity(), x, y);
    world_api.entities.set_rotation_2d(ids::camera_entity(), rotation);
}

fn on_enter() {
    let background = ids::background_entity();
    let lightning = world.light2d.get_group(ids::lightning_id());

    world.state.reset_scene();

    menu_state::seed(world);
    neon::reset(world);
    rain::reset(world);
    rain_lens::reset(world);

    world.entities.hide(ids::menu_entity());
    world.entities.hide(ids::press_any_key_entity());
    world.ui.hide(ids::menu_ui_root());
    world.ui.hide(ids::press_any_key_ui_root());

    world.layered_image2d.set_base_opacity(background, 0.0);
    neon::set_layer_opacities(world, background, 0.0);

    lightning.set_intensity(0.0);
    world.entities.set_position_2d(ids::camera_entity(), 0.0, 0.0);
    world.entities.set_rotation_2d(ids::camera_entity(), 0.0);
    world.state.set_float("intro.prompt_top_space", 513.0);
    world.ui.set_color(ids::press_any_key_prompt_ui(), "#7D879600");

    world.dev.log("Rotten Club main menu loaded.");
}

fn update(dt) {
    let time = world.state.get_float("intro.t") + dt;
    world.state.set_float("intro.t", time);
    update_camera_sway(world, time);

    let lights = neon::update(world, time);
    rain::update(world, time, lights[0], lights[1], lights[2]);
    rain_lens::update_intro_lens(world, time);

    if neon::intro_done(time) && !world.state.get_bool("intro.prompt_shown") {
        world.entities.show(ids::press_any_key_entity());
        world.ui.show(ids::press_any_key_ui_root());
        world.state.set_bool("intro.prompt_shown", true);
        world.state.set_float("intro.prompt_shown_at", time);
        world.state.set_float("intro.prompt_input_after", time + 0.25);
        world.ui.set_color(ids::press_any_key_prompt_ui(), "#7D879600");
        return;
    }

    if world.state.get_bool("intro.prompt_shown") && !world.state.get_bool("intro.menu_active") {
        let shown_at = world.state.get_float("intro.prompt_shown_at");
        let fade = clamp01((time - shown_at) / 0.45);
        let prompt_top = 513.0 + (time * 1.35).sin() * 8.0;
        world.state.set_float("intro.prompt_top_space", prompt_top);
        world.ui.set_color(ids::press_any_key_prompt_ui(), prompt_color(fade));
        if time >= world.state.get_float("intro.prompt_input_after") {
            if world.actions.pressed("menu.confirm") {
                world.entities.hide(ids::press_any_key_entity());
                world.ui.hide(ids::press_any_key_ui_root());
                world.entities.show(ids::menu_entity());
                world.ui.show(ids::menu_ui_root());
                world.state.set_bool("intro.menu_active", true);
                rain_lens::begin_menu_lens(world);
            }
        }
    }

    rain_lens::update_menu_lens(world, dt);
}

fn on_event(topic, payload) {
    if !world.state.get_bool("intro.menu_active") {
        return;
    }

    if topic == "menu.start" {
        world.dev.log("Main Menu: START selected.");
        return;
    }

    if topic == "menu.options" {
        world.dev.log("Main Menu: OPTIONS selected.");
        return;
    }

    if topic == "menu.quit" {
        world.dev.log("Main Menu: QUIT selected.");
        return;
    }
}
