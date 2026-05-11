import "mod:tar_fx/ids" as ids;
import "mod:tar_fx/menu_state" as menu_state;
import "mod:tar_fx/neon_alley" as neon;
import "mod:tar_fx/rain" as rain;

fn on_enter() {
    let background = ids::background_entity();
    let lightning = world.light2d.get_group(ids::lightning_id());

    world.state.reset_scene();

    menu_state::seed(world);
    neon::reset(world);
    rain::reset(world);

    world.entities.hide(ids::menu_entity());
    world.entities.hide(ids::press_any_key_entity());

    world.layered_image2d.set_base_opacity(background, 0.0);
    neon::set_layer_opacities(world, background, 0.0);

    lightning.set_intensity(0.0);

    world.dev.log("They Are Rotten main menu loaded.");
}

fn update(dt) {
    let time = world.state.get_float("intro.t") + dt;
    world.state.set_float("intro.t", time);

    let lights = neon::update(world, time);
    rain::update(world, time, lights[0], lights[1], lights[2]);

    if neon::intro_done(time) && !world.state.get_bool("intro.prompt_shown") {
        world.entities.show(ids::press_any_key_entity());
        world.state.set_bool("intro.prompt_shown", true);
    }

    if world.state.get_bool("intro.prompt_shown") && !world.state.get_bool("intro.menu_active") {
        if world.actions.pressed("menu.any") {
            world.entities.hide(ids::press_any_key_entity());
            world.entities.show(ids::menu_entity());
            world.state.set_bool("intro.menu_active", true);
        }
    }
}

fn on_event(topic, payload) {
    if topic == "they-are-rotten.menu.start" {
        world.dev.log("Main Menu: START selected.");
        return;
    }

    if topic == "they-are-rotten.menu.options" {
        world.dev.log("Main Menu: OPTIONS selected.");
        return;
    }

    if topic == "they-are-rotten.menu.quit" {
        world.dev.log("Main Menu: QUIT selected.");
        return;
    }
}
