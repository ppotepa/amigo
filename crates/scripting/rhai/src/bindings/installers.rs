use std::collections::BTreeSet;

type Installer = fn(&mut rhai::Engine);

fn installer(namespace: &str) -> Option<Installer> {
    match namespace {
        "assets" => Some(super::assets::register_api),
        "audio" => Some(super::audio::register_api),
        "camera" => Some(super::camera::register_api),
        "physics" => Some(super::physics::register_api),
        "physics3d" => Some(super::physics3d::register_api),
        "postfx" => Some(super::postfx::register_api),
        "pools" => Some(super::pools::register_api),
        "projectiles" => Some(super::projectiles::register_api),
        "mod" => Some(super::mod_api::register_api),
        "motion" => Some(super::motion::register_api),
        "particles" => Some(super::particles::register_api),
        "sprite2d" => Some(super::sprite2d::register_api),
        "layered_image2d" => Some(super::layered_image2d::register_api),
        "beacon2d" => Some(super::beacon2d::register_api),
        "light2d" => Some(super::light2d::register_api),
        "render2d" => Some(super::render2d::register_api),
        "state" => Some(super::state::register_api),
        "vector2d" => Some(super::vector2d::register_api),
        "text2d" => Some(super::text2d::register_api),
        "mesh3d" => Some(super::mesh3d::register_api),
        "material3d" => Some(super::material3d::register_api),
        "text3d" => Some(super::text3d::register_api),
        "ui" => Some(super::ui::register_api),
        _ => None,
    }
}

pub(crate) fn install_declared(engine: &mut rhai::Engine, namespaces: &[String]) {
    let mut installed = BTreeSet::new();
    for namespace in namespaces {
        if !installed.insert(namespace.clone()) { continue; }
        let install = installer(namespace).unwrap_or_else(|| {
            panic!("Rhai namespace `{namespace}` has no backend binding installer")
        });
        install(engine);
    }
}
