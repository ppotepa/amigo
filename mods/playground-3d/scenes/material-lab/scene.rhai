fn on_enter() {
    let scene_id = world.scene.current_id();
    world.dev.event("playground-3d.scene-ready", scene_id);
    world.dev.log("playground-3d scene bootstrapped from scene document for scene=" + scene_id);
}
