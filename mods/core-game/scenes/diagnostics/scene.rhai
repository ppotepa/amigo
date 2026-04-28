fn on_enter() {
    world.dev.event("core-game.scene-ready", world.scene.current_id());
    world.dev.event("core-game.refresh-diagnostics", world.mod.current_id());
}
