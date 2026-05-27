pub use amigo_camera_core_plugin::scripting::rhai_api::CameraApi;

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<CameraApi>("WorldCamera")
        .register_fn("set_main_lens_rain", CameraApi::set_main_lens_rain)
        .register_fn("set_lens_rain", CameraApi::set_lens_rain)
        .register_fn(
            "set_main_lens_rain_profile",
            CameraApi::set_main_lens_rain_profile,
        )
        .register_fn("set_lens_rain_profile", CameraApi::set_lens_rain_profile)
        .register_fn(
            "clear_main_lens_rain_profile",
            CameraApi::clear_main_lens_rain_profile,
        )
        .register_fn(
            "clear_lens_rain_profile",
            CameraApi::clear_lens_rain_profile,
        )
        .register_fn(
            "clear_main_lens_rain_override",
            CameraApi::clear_main_lens_rain_override,
        )
        .register_fn(
            "clear_lens_rain_override",
            CameraApi::clear_lens_rain_override,
        )
        .register_fn(
            "set_main_focal_length_mm",
            CameraApi::set_main_focal_length_mm,
        )
        .register_fn("set_main_lens_profile", CameraApi::set_main_lens_profile)
        .register_fn("set_lens_profile", CameraApi::set_lens_profile)
        .register_fn("set_main_preset", CameraApi::set_main_preset)
        .register_fn("set_preset", CameraApi::set_preset)
        .register_fn("set_main_focal_length", CameraApi::set_main_focal_length_mm)
        .register_fn("set_focal_length_mm", CameraApi::set_focal_length_mm)
        .register_fn("set_focal_length", CameraApi::set_focal_length_mm)
        .register_fn(
            "set_main_aperture_f_stop",
            CameraApi::set_main_aperture_f_stop,
        )
        .register_fn("set_main_aperture", CameraApi::set_main_aperture_f_stop)
        .register_fn("set_aperture_f_stop", CameraApi::set_aperture_f_stop)
        .register_fn("set_aperture", CameraApi::set_aperture_f_stop)
        .register_fn(
            "set_main_aperture_rotation_degrees",
            CameraApi::set_main_aperture_rotation_degrees,
        )
        .register_fn(
            "set_aperture_rotation_degrees",
            CameraApi::set_aperture_rotation_degrees,
        )
        .register_fn(
            "set_main_dof_max_blur_px",
            CameraApi::set_main_dof_max_blur_px,
        )
        .register_fn("set_dof_max_blur_px", CameraApi::set_dof_max_blur_px)
        .register_fn(
            "set_main_dof_focus_width",
            CameraApi::set_main_dof_focus_width,
        )
        .register_fn("set_dof_focus_width", CameraApi::set_dof_focus_width)
        .register_fn(
            "set_main_dof_blur_boosts",
            CameraApi::set_main_dof_blur_boosts,
        )
        .register_fn("set_dof_blur_boosts", CameraApi::set_dof_blur_boosts)
        .register_fn(
            "set_main_dof_sample_count",
            CameraApi::set_main_dof_sample_count,
        )
        .register_fn("set_dof_sample_count", CameraApi::set_dof_sample_count)
        .register_fn(
            "set_main_dof_highlights",
            CameraApi::set_main_dof_highlights,
        )
        .register_fn("set_dof_highlights", CameraApi::set_dof_highlights)
        .register_fn(
            "set_main_focus_distance_m",
            CameraApi::set_main_focus_distance_m,
        )
        .register_fn("focus_main", CameraApi::focus_main)
        .register_fn("focus_main_over", CameraApi::focus_main_over)
        .register_fn("focus_over", CameraApi::focus_over)
        .register_fn("has_focus_target", CameraApi::has_focus_target)
        .register_fn("focus_target_summary", CameraApi::focus_target_summary)
        .register_fn("set_focus_distance_m", CameraApi::set_focus_distance_m)
        .register_fn("set_main_focus_depth", CameraApi::set_main_focus_depth)
        .register_fn("set_focus_depth", CameraApi::set_focus_depth)
        .register_fn("set_main_sway_amounts", CameraApi::set_main_sway_amounts)
        .register_fn("set_sway_amounts", CameraApi::set_sway_amounts)
        .register_fn(
            "set_main_sway_frequency",
            CameraApi::set_main_sway_frequency,
        )
        .register_fn("set_sway_frequency", CameraApi::set_sway_frequency)
        .register_fn("set_main_sway_z_offset", CameraApi::set_main_sway_z_offset)
        .register_fn("set_sway_z_offset", CameraApi::set_sway_z_offset)
        .register_fn("set_main_camera_z_m", CameraApi::set_main_camera_z_m)
        .register_fn("set_camera_z_m", CameraApi::set_camera_z_m)
        .register_fn(
            "set_main_focus_residual_m",
            CameraApi::set_main_focus_residual_m,
        )
        .register_fn("set_focus_residual_m", CameraApi::set_focus_residual_m)
        .register_fn("set_main_dolly_signal", CameraApi::set_main_dolly_signal)
        .register_fn("set_dolly_signal", CameraApi::set_dolly_signal)
        .register_fn(
            "set_main_shutter_speed_s",
            CameraApi::set_main_shutter_speed_s,
        )
        .register_fn("set_shutter_speed_s", CameraApi::set_shutter_speed_s)
        .register_fn(
            "set_main_shutter_fraction",
            CameraApi::set_main_shutter_fraction,
        )
        .register_fn(
            "set_main_shutter_enabled",
            CameraApi::set_main_shutter_enabled,
        )
        .register_fn("set_shutter_enabled", CameraApi::set_shutter_enabled)
        .register_fn(
            "set_main_shutter_opacity",
            CameraApi::set_main_shutter_opacity,
        )
        .register_fn("set_shutter_opacity", CameraApi::set_shutter_opacity)
        .register_fn(
            "set_main_sway_affects_focus",
            CameraApi::set_main_sway_affects_focus,
        )
        .register_fn("set_sway_affects_focus", CameraApi::set_sway_affects_focus)
        .register_fn("clear_main_sway", CameraApi::clear_main_sway)
        .register_fn("clear_sway", CameraApi::clear_sway)
        .register_fn("set_main_quality", CameraApi::set_main_quality)
        .register_fn("set_quality", CameraApi::set_quality)
        .register_fn("set_main_debug_view", CameraApi::set_main_debug_view)
        .register_fn("set_debug_view", CameraApi::set_debug_view);
}
