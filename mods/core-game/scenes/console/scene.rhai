fn on_enter() {
    world.dev.event("core-game.scene-ready", world.scene.current_id());
    world.dev.command("help");
}
