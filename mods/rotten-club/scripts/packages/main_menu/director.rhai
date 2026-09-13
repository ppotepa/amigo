fn enter(world_api) {
    world_api.dev.log("Rotten Club main menu loaded.");
}

fn update(_world_api, _dt) {
}

fn event(world_api, topic) {
    if topic == "hot-reload.asset-changed" {
        world_api.dev.log("Rotten Club noticed asset hot reload.");
    }
}
