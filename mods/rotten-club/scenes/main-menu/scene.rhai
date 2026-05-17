import "mod:fx/ids" as ids;
import "mod:fx/camera_dof_controls" as camera_dof;
import "mod:fx/camera_presets" as camera_presets;
import "mod:fx/menu_state" as menu_state;
import "mod:fx/neon_alley" as neon;
import "mod:fx/rain" as rain;

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
    if !world_api.state.get_bool("camera_motion.enabled") {
        return;
    }

    let x = (time * world_api.state.get_float("camera_motion.sway.x.primary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.x.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.secondary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.x.detail_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.x.detail_amplitude");
    let y = (time * world_api.state.get_float("camera_motion.sway.y.primary_frequency")).cos()
            * world_api.state.get_float("camera_motion.sway.y.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.y.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.sway.y.secondary_amplitude")
        + (time * world_api.state.get_float("camera_motion.sway.y.detail_frequency")).cos()
            * world_api.state.get_float("camera_motion.sway.y.detail_amplitude");
    let rotation = (time * world_api.state.get_float("camera_motion.rotation.primary_frequency")).sin()
            * world_api.state.get_float("camera_motion.rotation.primary_amplitude")
        + (time * world_api.state.get_float("camera_motion.rotation.secondary_frequency")).sin()
            * world_api.state.get_float("camera_motion.rotation.secondary_amplitude");
    world_api.entities.set_position_2d(ids::camera_entity(), x, y);
    world_api.entities.set_rotation_2d(ids::camera_entity(), rotation);
}

fn on_enter() {
    let background = ids::background_entity();
    let lightning = world.light2d.get_group(ids::lightning_id());

    world.state.reset_scene();

    menu_state::seed(world);
    camera_presets::apply(world);
    camera_dof::apply(world);
    neon::reset(world);
    rain::reset(world);

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
    camera_presets::update(world);
    camera_dof::update(world, dt);

    let lights = neon::update(world, time);
    rain::update(world, time, lights[0], lights[1], lights[2]);

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
            }
        }
    }
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
