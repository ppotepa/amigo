pub(crate) mod assets;
pub(crate) mod commands;
pub(crate) mod common;
pub(crate) mod debug;
pub(crate) mod entities;
pub(crate) mod input;
pub(crate) mod material3d;
pub(crate) mod mesh3d;
pub(crate) mod mod_api;
pub(crate) mod runtime;
pub(crate) mod scene;
pub(crate) mod sprite2d;
pub(crate) mod text2d;
pub(crate) mod text3d;
pub(crate) mod time;
pub(crate) mod ui;
pub(crate) mod world_root;

pub use assets::AssetsApi;
pub use debug::DebugApi;
pub use entities::EntitiesApi;
pub use input::InputApi;
pub use material3d::Material3dApi;
pub use mesh3d::Mesh3dApi;
pub use mod_api::ModApi;
pub use runtime::RuntimeApi;
pub use scene::SceneApi;
pub use sprite2d::Sprite2dApi;
pub use text2d::Text2dApi;
pub use text3d::Text3dApi;
pub use time::{ScriptTimeState, TimeApi};
pub use ui::UiApi;
pub use world_root::WorldApi;

use crate::handles::{AssetRef, EntityRef};

pub fn register_world_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<WorldApi>("World")
        .register_type_with_name::<SceneApi>("WorldScene")
        .register_type_with_name::<EntitiesApi>("WorldEntities")
        .register_type_with_name::<InputApi>("WorldInput")
        .register_type_with_name::<TimeApi>("WorldTime")
        .register_type_with_name::<AssetsApi>("WorldAssets")
        .register_type_with_name::<ModApi>("WorldMod")
        .register_type_with_name::<Sprite2dApi>("WorldSprite2d")
        .register_type_with_name::<Text2dApi>("WorldText2d")
        .register_type_with_name::<Mesh3dApi>("WorldMesh3d")
        .register_type_with_name::<Material3dApi>("WorldMaterial3d")
        .register_type_with_name::<Text3dApi>("WorldText3d")
        .register_type_with_name::<UiApi>("WorldUi")
        .register_type_with_name::<DebugApi>("WorldDebug")
        .register_type_with_name::<RuntimeApi>("WorldRuntime")
        .register_type_with_name::<EntityRef>("EntityRef")
        .register_type_with_name::<AssetRef>("AssetRef")
        .register_get("scene", WorldApi::scene)
        .register_get("entities", WorldApi::entities)
        .register_get("input", WorldApi::input)
        .register_get("time", WorldApi::time)
        .register_get("assets", WorldApi::assets)
        .register_get("mod", WorldApi::game_mod)
        .register_get("sprite2d", WorldApi::sprite2d)
        .register_get("text2d", WorldApi::text2d)
        .register_get("mesh3d", WorldApi::mesh3d)
        .register_get("material3d", WorldApi::material3d)
        .register_get("text3d", WorldApi::text3d)
        .register_get("ui", WorldApi::ui)
        .register_get("dev", WorldApi::dev)
        .register_get("runtime", WorldApi::runtime)
        .register_fn("current_id", SceneApi::current_id)
        .register_fn("available", SceneApi::available)
        .register_fn("has", SceneApi::has)
        .register_fn("select", SceneApi::select)
        .register_fn("reload", SceneApi::reload)
        .register_fn("named", EntitiesApi::named)
        .register_fn("create", EntitiesApi::create)
        .register_fn("exists", EntitiesApi::exists)
        .register_fn("count", EntitiesApi::count)
        .register_fn("names", EntitiesApi::names)
        .register_fn("down", InputApi::down)
        .register_fn("pressed", InputApi::pressed)
        .register_fn("keys", InputApi::keys)
        .register_fn("delta", TimeApi::delta)
        .register_fn("elapsed", TimeApi::elapsed)
        .register_fn("frame", TimeApi::frame)
        .register_fn("get", AssetsApi::get)
        .register_fn("has", AssetsApi::has)
        .register_fn("registered", AssetsApi::registered)
        .register_fn("by_mod", AssetsApi::by_mod)
        .register_fn("reload", AssetsApi::reload)
        .register_fn("pending", AssetsApi::pending)
        .register_fn("loaded", AssetsApi::loaded)
        .register_fn("prepared", AssetsApi::prepared)
        .register_fn("failed", AssetsApi::failed)
        .register_fn("current_id", ModApi::current_id)
        .register_fn("scenes", ModApi::scenes)
        .register_fn("has_scene", ModApi::has_scene)
        .register_fn("capabilities", ModApi::capabilities)
        .register_fn("loaded", ModApi::loaded)
        .register_fn("frame", Sprite2dApi::frame)
        .register_fn("set_frame", Sprite2dApi::set_frame)
        .register_fn("advance", Sprite2dApi::advance)
        .register_fn("queue", Sprite2dApi::queue)
        .register_fn("queue", Text2dApi::queue)
        .register_fn("queue", Mesh3dApi::queue)
        .register_fn("bind", Material3dApi::bind)
        .register_fn("queue", Text3dApi::queue)
        .register_fn("set_text", UiApi::set_text)
        .register_fn("set_value", UiApi::set_value)
        .register_fn("show", UiApi::show)
        .register_fn("hide", UiApi::hide)
        .register_fn("enable", UiApi::enable)
        .register_fn("disable", UiApi::disable)
        .register_fn("event", DebugApi::event)
        .register_fn("event", DebugApi::event_with_payload)
        .register_fn("command", DebugApi::command)
        .register_fn("log", DebugApi::log)
        .register_fn("warn", DebugApi::warn)
        .register_fn("refresh_diagnostics", DebugApi::refresh_diagnostics)
        .register_fn("window_backend", RuntimeApi::window_backend)
        .register_fn("input_backend", RuntimeApi::input_backend)
        .register_fn("render_backend", RuntimeApi::render_backend)
        .register_fn("script_backend", RuntimeApi::script_backend)
        .register_fn("capabilities", RuntimeApi::capabilities)
        .register_fn("plugins", RuntimeApi::plugins)
        .register_fn("services", RuntimeApi::services)
        .register_fn("dev_mode", RuntimeApi::dev_mode)
        .register_fn("name", EntityRef::name)
        .register_fn("exists", EntityRef::exists)
        .register_fn("rotate_2d", EntityRef::rotate_2d)
        .register_fn("rotate_3d", EntityRef::rotate_3d)
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
