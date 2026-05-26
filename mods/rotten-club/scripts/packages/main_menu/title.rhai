import "mod:main_menu/ids" as ids;
import "mod:main_menu/math" as math;
import "mod:main_menu/timeline" as timeline;

fn reset(world_api) {
    world_api.state.set_bool("intro.title_reveal_started", false);
    world_api.state.set_bool("intro.title_shown", false);
    world_api.state.set_float("intro.title_alpha", 0.0);
    world_api.entities.hide(ids::title_entity());
    world_api.render2d.get_layer(ids::title_layer()).set_opacity(0.0);
}

fn update(world_api, time) {
    if time >= timeline::title_reveal_start_s() && !world_api.state.get_bool("intro.title_reveal_started") {
        world_api.entities.show(ids::title_entity());
        world_api.state.set_bool("intro.title_reveal_started", true);
        world_api.state.set_bool("intro.title_shown", true);
    }

    let reveal = math::ramp(time, timeline::title_reveal_start_s(), 2.0);
    let alpha = math::mix(0.0, timeline::title_visibility(), reveal);
    world_api.state.set_float("intro.title_alpha", alpha);
    world_api.render2d.get_layer(ids::title_layer()).set_opacity(alpha);
}
