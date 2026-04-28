fn on_enter() {
    let active_mod = world.mod.current_id();

    if !world.entities.exists("core-game-mod") {
        world.entities.create("core-game-mod");
    }

    if world.runtime.dev_mode() && !world.entities.exists("core-game-dev-mode") {
        world.entities.create("core-game-dev-mode");
    }

    for mod_id in world.mod.loaded() {
        let marker = "mod:" + mod_id;

        if !world.entities.exists(marker) {
            world.entities.create(marker);
        }
    }

    world.dev.log("core-game mod runtime entered for mod=" + active_mod);
    world.dev.log("window backend=" + world.runtime.window_backend());
    world.dev.log("input backend=" + world.runtime.input_backend());
    world.dev.log("render backend=" + world.runtime.render_backend());
    world.dev.log("script backend=" + world.runtime.script_backend());
    world.dev.log("runtime capabilities=" + world.runtime.capabilities().len);
    world.dev.log("runtime plugins=" + world.runtime.plugins().len);
    world.dev.log("runtime services=" + world.runtime.services().len);
    world.dev.log("assets registered=" + world.assets.registered().len);
    world.dev.log("assets loaded=" + world.assets.loaded().len);
    world.dev.log("assets prepared=" + world.assets.prepared().len);
    world.dev.log("assets failed=" + world.assets.failed().len);
    world.dev.log("assets pending=" + world.assets.pending().len);
    world.dev.log("active mod assets=" + world.assets.by_mod(active_mod).len);

    for scene_id in world.mod.scenes() {
        world.dev.log("available scene for " + active_mod + ": " + scene_id);
    }
}

fn on_event(topic, payload) {
    if topic == "core-game.scene-ready" && payload.len > 0 {
        let scene_id = payload[0];
        let marker = "scene:" + scene_id;

        if !world.entities.exists(marker) {
            world.entities.create(marker);
        }

        world.dev.event("core-game.bootstrapped", scene_id);
        world.dev.log(
            "core-game dev shell bootstrapped for mod="
            + world.mod.current_id()
            + " scene="
            + scene_id
        );
    }

    if topic == "core-game.refresh-diagnostics" && payload.len > 0 {
        world.dev.refresh_diagnostics(payload[0]);
    }

    ()
}
