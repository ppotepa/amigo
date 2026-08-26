fn enabled(provider_namespaces: &[String], namespace: &str) -> bool {
    provider_namespaces.iter().any(|candidate| candidate == namespace)
}

pub(crate) fn default_domain_namespaces() -> Vec<String> {
    [
        "assets", "audio", "camera", "physics", "physics3d", "postfx", "pools",
        "projectiles", "mod", "motion", "particles", "sprite2d", "layered_image2d",
        "beacon2d", "light2d", "render2d", "state", "vector2d", "text2d", "mesh3d",
        "material3d", "text3d", "ui",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect()
}

pub(crate) fn register_all(engine: &mut rhai::Engine, provider_namespaces: &[String]) {
    super::common::register_api(engine);
    super::world_root::register_api(engine);
    super::scene::register_api(engine);
    super::session::register_api(engine);
    super::entities::register_api(engine);
    super::input::register_api(engine);
    super::actions::register_api(engine);
    super::arcade::register_api(engine);
    super::random::register_api(engine);
    super::time::register_api(engine);
    super::timers::register_api(engine);
    super::trace::register_api(engine);
    super::debug::register_api(engine);
    super::runtime::register_api(engine);
    super::handle_refs::register_api(engine);

    if enabled(provider_namespaces, "assets") { super::assets::register_api(engine); }
    if enabled(provider_namespaces, "audio") { super::audio::register_api(engine); }
    if enabled(provider_namespaces, "camera") { super::camera::register_api(engine); }
    if enabled(provider_namespaces, "physics") { super::physics::register_api(engine); }
    if enabled(provider_namespaces, "physics3d") { super::physics3d::register_api(engine); }
    if enabled(provider_namespaces, "postfx") { super::postfx::register_api(engine); }
    if enabled(provider_namespaces, "pools") { super::pools::register_api(engine); }
    if enabled(provider_namespaces, "projectiles") { super::projectiles::register_api(engine); }
    if enabled(provider_namespaces, "mod") { super::mod_api::register_api(engine); }
    if enabled(provider_namespaces, "motion") { super::motion::register_api(engine); }
    if enabled(provider_namespaces, "particles") { super::particles::register_api(engine); }
    if enabled(provider_namespaces, "sprite2d") { super::sprite2d::register_api(engine); }
    if enabled(provider_namespaces, "layered_image2d") { super::layered_image2d::register_api(engine); }
    if enabled(provider_namespaces, "beacon2d") { super::beacon2d::register_api(engine); }
    if enabled(provider_namespaces, "light2d") { super::light2d::register_api(engine); }
    if enabled(provider_namespaces, "render2d") { super::render2d::register_api(engine); }
    if enabled(provider_namespaces, "state") { super::state::register_api(engine); }
    if enabled(provider_namespaces, "vector2d") { super::vector2d::register_api(engine); }
    if enabled(provider_namespaces, "text2d") { super::text2d::register_api(engine); }
    if enabled(provider_namespaces, "mesh3d") { super::mesh3d::register_api(engine); }
    if enabled(provider_namespaces, "material3d") { super::material3d::register_api(engine); }
    if enabled(provider_namespaces, "text3d") { super::text3d::register_api(engine); }
    if enabled(provider_namespaces, "ui") { super::ui::register_api(engine); }
}
