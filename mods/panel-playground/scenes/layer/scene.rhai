fn on_event(topic, payload) {
    if topic == "layer.reset" {
        world.controls.set("world.demo.RenderLayer2D.opacity", 1.0);
        world.controls.set("world.demo.RenderLayer2D.visible", true);
    }
}
