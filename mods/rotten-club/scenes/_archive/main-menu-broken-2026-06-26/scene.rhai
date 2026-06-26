import "mod:main_menu/director" as director;

fn on_enter() {
    director::enter(world);
}

fn update(dt) {
    director::update(world, dt);
}

fn on_event(topic, payload) {
    director::event(world, topic);
}
