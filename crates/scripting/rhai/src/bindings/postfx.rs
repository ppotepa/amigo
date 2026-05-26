pub use amigo_composite_plugin::scripting::rhai_api::{PostFxApi, PostFxItemRef};

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<PostFxApi>("WorldPostFx")
        .register_type_with_name::<PostFxItemRef>("WorldPostFxItem")
        .register_fn("count", PostFxApi::count)
        .register_fn("list", PostFxApi::list)
        .register_fn("item", PostFxApi::item)
        .register_fn("frame_effect_enabled", PostFxApi::frame_effect_enabled)
        .register_fn(
            "set_frame_effect_enabled",
            PostFxApi::set_frame_effect_enabled,
        )
        .register_fn(
            "color_quantize_palette_size",
            PostFxApi::color_quantize_palette_size,
        )
        .register_fn(
            "set_color_quantize_palette_size",
            PostFxApi::set_color_quantize_palette_size,
        )
        .register_fn(
            "adjust_color_quantize_palette_size",
            PostFxApi::adjust_color_quantize_palette_size,
        )
        .register_fn("set_color_quantize", PostFxApi::set_color_quantize)
        .register_fn(
            "color_ramp_palette_size",
            PostFxApi::color_ramp_palette_size,
        )
        .register_fn("set_color_ramp", PostFxApi::set_color_ramp)
        .register_fn(
            "adjust_color_ramp_palette_size",
            PostFxApi::adjust_color_ramp_palette_size,
        )
        .register_fn("set_rain_glass", PostFxApi::set_rain_glass)
        .register_fn(
            "apply_rain_glass_preset",
            PostFxApi::apply_rain_glass_preset,
        )
        .register_fn("set_rain_glass_bool", PostFxApi::set_rain_glass_bool)
        .register_fn("set_rain_glass_int", PostFxApi::set_rain_glass_int)
        .register_fn("set_rain_glass_float", PostFxApi::set_rain_glass_float)
        .register_fn("set_rain_glass_debug", PostFxApi::set_rain_glass_debug)
        .register_fn("set_rain_glass_compose", PostFxApi::set_rain_glass_compose)
        .register_get("exists", PostFxItemRef::exists)
        .register_get("index", PostFxItemRef::index)
        .register_get("name", PostFxItemRef::name)
        .register_get("active", PostFxItemRef::active);
}
