fn on_attach(entity, params) {
    world.state.set_string(entity + ".component.label", params.label);
    world.state.set_float(entity + ".component.elapsed", 0.0);
    world.dev.event("playground-2d.demo.component.attach", entity);
}

fn update(entity, params, dt) {
    let elapsed_key = entity + ".component.elapsed";
    let elapsed = world.state.get_float(elapsed_key);
    world.state.set_float(elapsed_key, elapsed + dt * params.speed);
}

fn on_detach(entity, params) {
    world.dev.event("playground-2d.demo.component.detach", entity);
}
