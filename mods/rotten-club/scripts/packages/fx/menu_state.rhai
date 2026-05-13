fn seed(world_api) {
    world_api.state.set_int("menu_index", 0);

    world_api.state.set_string("menu_color.0", "#39D7FFFF");
    world_api.state.set_string("menu_color.1", "#7D8796FF");
    world_api.state.set_string("menu_color.2", "#7D8796FF");

    world_api.state.set_string("menu_status", "READY");
    world_api.state.set_string("menu_status_color", "#7D8796FF");
}


