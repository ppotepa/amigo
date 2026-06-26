use crate::handles::{AssetRef, EntityRef};

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<EntityRef>("EntityRef")
        .register_type_with_name::<AssetRef>("AssetRef")
        .register_fn("name", EntityRef::name)
        .register_fn("exists", EntityRef::exists)
        .register_get("name", EntityRef::name)
        .register_get("exists", EntityRef::exists)
        .register_get_set("opacity", EntityRef::opacity, EntityRef::set_opacity)
        .register_get_set("visible", EntityRef::visible, EntityRef::set_visible)
        .register_get_set("enabled", EntityRef::enabled, EntityRef::set_enabled)
        .register_get_set(
            "collision_enabled",
            EntityRef::collision,
            EntityRef::set_collision,
        )
        .register_fn("rotate_2d", EntityRef::rotate_2d)
        .register_fn("rotate_3d", EntityRef::rotate_3d)
        .register_fn("set_position_2d", EntityRef::set_position_2d)
        .register_fn("set_position_3d", EntityRef::set_position_3d)
        .register_fn("set_rotation_3d", EntityRef::set_rotation_3d)
        .register_fn("hide", EntityRef::hide)
        .register_fn("show", EntityRef::show)
        .register_fn("enable", EntityRef::enable)
        .register_fn("disable", EntityRef::disable)
        .register_fn("set_collision_enabled", EntityRef::set_collision_enabled)
        .register_fn("is_visible", EntityRef::is_visible)
        .register_fn("is_enabled", EntityRef::is_enabled)
        .register_fn("collision_enabled", EntityRef::collision_enabled)
        .register_fn("has_tag", EntityRef::has_tag)
        .register_fn("has_group", EntityRef::has_group)
        .register_fn("property", EntityRef::property)
        .register_fn("property_int", EntityRef::property_int)
        .register_fn("property_float", EntityRef::property_float)
        .register_fn("property_bool", EntityRef::property_bool)
        .register_fn("property_string", EntityRef::property_string)
        .register_fn("key", AssetRef::key)
        .register_fn("exists", AssetRef::exists)
        .register_fn("state", AssetRef::state)
        .register_fn("source", AssetRef::source)
        .register_fn("path", AssetRef::path)
        .register_fn("kind", AssetRef::kind)
        .register_fn("label", AssetRef::label)
        .register_fn("format", AssetRef::format)
        .register_fn("tags", AssetRef::tags)
        .register_fn("reason", AssetRef::reason)
        .register_fn("reload", AssetRef::reload);
}
