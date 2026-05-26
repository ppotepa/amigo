import "mod:main_menu/ids" as ids;
import "mod:main_menu/math" as math;
import "mod:main_menu/timeline" as timeline;

fn prompt_color(alpha) {
    let a = math::clamp01(alpha);
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

fn seed_menu_state(world_api) {
    world_api.state.set_int("menu_index", 0);
    world_api.state.set_int("menu_item_count", 0);
    world_api.state.set_string("menu_color.0", "#39D7FFFF");
    world_api.state.set_string("menu_color.1", "#7D8796FF");
    world_api.state.set_string("menu_color.2", "#7D8796FF");
    world_api.state.set_string("menu_status", "READY");
    world_api.state.set_string("menu_status_color", "#7D8796FF");
}

fn reset(world_api) {
    world_api.state.set_bool("intro.prompt_shown", false);
    world_api.state.set_bool("intro.menu_active", false);
    world_api.state.set_float("intro.prompt_top_space", timeline::prompt_top_base_px());
    world_api.state.set_float("intro.prompt_shown_at", 0.0);
    world_api.state.set_float("intro.prompt_input_after", 0.0);

    world_api.entities.hide(ids::menu_entity());
    world_api.entities.hide(ids::press_any_key_entity());
    world_api.ui.hide(ids::menu_ui_root());
    world_api.ui.hide(ids::press_any_key_ui_root());
    world_api.ui.set_color(ids::press_any_key_prompt_ui(), "#7D879600");
}

fn update(world_api, time) {
    if world_api.state.get_bool("intro.complete") && !world_api.state.get_bool("intro.prompt_shown") {
        world_api.entities.show(ids::press_any_key_entity());
        world_api.ui.show(ids::press_any_key_ui_root());
        world_api.state.set_bool("intro.prompt_shown", true);
        world_api.state.set_float("intro.prompt_shown_at", time);
        world_api.state.set_float("intro.prompt_input_after", time + 0.25);
        world_api.ui.set_color(ids::press_any_key_prompt_ui(), "#7D879600");
        return;
    }

    if world_api.state.get_bool("intro.prompt_shown") && !world_api.state.get_bool("intro.menu_active") {
        let shown_at = world_api.state.get_float("intro.prompt_shown_at");
        let fade = math::clamp01((time - shown_at) / 0.45);
        let prompt_top = timeline::prompt_top_base_px() + (time * 1.35).sin() * 8.0;
        world_api.state.set_float("intro.prompt_top_space", prompt_top);
        world_api.ui.set_color(ids::press_any_key_prompt_ui(), prompt_color(fade));
        if time >= world_api.state.get_float("intro.prompt_input_after") {
            if world_api.actions.pressed("menu.confirm") {
                world_api.entities.hide(ids::press_any_key_entity());
                world_api.ui.hide(ids::press_any_key_ui_root());
                world_api.entities.show(ids::menu_entity());
                world_api.ui.show(ids::menu_ui_root());
                world_api.state.set_bool("intro.menu_active", true);
            }
        }
    }
}

fn event(world_api, topic) {
    if !world_api.state.get_bool("intro.menu_active") {
        return;
    }

    if topic == "menu.start" {
        world_api.dev.log("Main Menu: START selected.");
        return;
    }
    if topic == "menu.options" {
        world_api.dev.log("Main Menu: OPTIONS selected.");
        return;
    }
    if topic == "menu.quit" {
        world_api.dev.log("Main Menu: QUIT selected.");
    }
}
