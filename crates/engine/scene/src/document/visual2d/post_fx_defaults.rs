pub(super) fn default_lens_droplets_max() -> u32 {
    48
}

pub(super) fn default_lens_droplets_spawn_rate() -> f32 {
    0.25
}

pub(super) fn default_lens_droplets_radius_range() -> [f32; 2] {
    [10.0, 42.0]
}

pub(super) fn default_lens_droplets_opacity_range() -> [f32; 2] {
    [0.18, 0.52]
}

pub(super) fn default_lens_droplets_lifetime_range() -> [f32; 2] {
    [4.0, 12.0]
}

pub(super) fn default_lens_droplets_streak_chance() -> f32 {
    0.16
}

pub(super) fn default_lens_droplets_gravity() -> f32 {
    24.0
}

pub(super) fn default_lens_droplets_max_streak_length() -> f32 {
    160.0
}

pub(super) fn default_lens_droplets_wobble() -> f32 {
    0.35
}

pub(super) fn default_wet_blur_px() -> f32 {
    1.5
}

pub(super) fn default_wet_distortion_px() -> f32 {
    0.8
}

pub(super) fn default_wet_shimmer_strength() -> f32 {
    0.04
}

pub(super) fn default_wet_ripple_strength() -> f32 {
    0.02
}

pub(super) fn default_wet_darken() -> f32 {
    0.06
}

pub(super) fn default_wet_specular_boost() -> f32 {
    0.25
}

pub(super) fn default_wet_edge_power() -> f32 {
    1.35
}

pub(super) fn default_wet_light_reflection_strength() -> f32 {
    0.65
}

pub(super) fn default_wet_foreground_strength() -> f32 {
    1.0
}

pub(super) fn default_wet_background_strength() -> f32 {
    0.12
}

pub(super) fn default_wet_horizon_y() -> f32 {
    0.42
}

pub(super) fn default_wet_noise_scale() -> f32 {
    2.5
}

pub(super) fn default_wet_noise_speed() -> f32 {
    0.035
}

pub(super) fn default_wet_ripple_speed() -> f32 {
    0.08
}

pub(super) fn default_film_noise_iso() -> f32 {
    800.0
}

pub(super) fn default_film_noise_grain_size() -> f32 {
    1.0
}

pub(super) fn default_film_noise_chroma_noise() -> f32 {
    0.04
}

pub(super) fn default_film_noise_color_shift() -> f32 {
    0.03
}

pub(super) fn default_film_noise_contrast() -> f32 {
    1.0
}

pub(super) fn default_film_noise_saturation() -> f32 {
    1.0
}

pub(super) fn default_film_noise_flicker() -> f32 {
    0.12
}

pub(super) fn default_film_noise_vignette() -> f32 {
    0.08
}

pub(super) fn default_film_noise_opacity() -> f32 {
    0.35
}

pub(super) fn default_film_noise_seed() -> u32 {
    1337
}

pub(super) fn default_color_quantize_palette_size() -> u32 {
    64
}

pub(super) fn default_color_quantize_dither_strength() -> f32 {
    0.35
}

pub(super) fn default_color_quantize_dither_scale() -> f32 {
    1.0
}

pub(super) fn default_color_quantize_luma_preserve() -> f32 {
    0.2
}

pub(super) fn default_color_quantize_gamma() -> f32 {
    2.2
}

pub(super) fn default_color_quantize_seed() -> u32 {
    911
}

pub(super) fn default_downscale_factor() -> f32 {
    2.0
}

pub(super) fn default_shutter_blur_fps() -> f32 {
    24.0
}

pub(super) fn default_shutter_blur_shutter_angle() -> f32 {
    180.0
}

pub(super) fn default_shutter_blur_opacity() -> f32 {
    0.72
}

pub(super) fn default_shutter_blur_edge_rejection() -> f32 {
    0.35
}

pub(super) fn default_shutter_blur_luma_threshold() -> f32 {
    0.04
}

pub(super) fn default_dirty_bloom_threshold() -> f32 {
    0.62
}

pub(super) fn default_dirty_bloom_strength() -> f32 {
    0.75
}

pub(super) fn default_dirty_bloom_small_radius_px() -> f32 {
    3.0
}

pub(super) fn default_dirty_bloom_medium_radius_px() -> f32 {
    12.0
}

pub(super) fn default_dirty_bloom_large_radius_px() -> f32 {
    32.0
}

pub(super) fn default_dirty_bloom_dirty_noise() -> f32 {
    0.18
}

pub(super) fn default_dirty_bloom_halation_strength() -> f32 {
    0.22
}

pub(super) fn default_dirty_bloom_reflection_smear_x_px() -> f32 {
    6.0
}

pub(super) fn default_dirty_bloom_reflection_smear_y_px() -> f32 {
    28.0
}

pub(super) fn default_dirty_bloom_seed() -> u32 {
    4242
}

pub(super) fn default_crt_scanline_opacity() -> f32 {
    0.12
}

pub(super) fn default_crt_scanline_frequency_px() -> f32 {
    1.5
}

pub(super) fn default_crt_rgb_split_px() -> f32 {
    1.0
}

pub(super) fn default_crt_curvature() -> f32 {
    0.03
}

pub(super) fn default_crt_vignette() -> f32 {
    0.22
}

pub(super) fn default_crt_phosphor_mask() -> f32 {
    0.04
}

pub(super) fn default_crt_brightness_compensation() -> f32 {
    1.05
}

pub(super) fn default_rain_glass_spawn_rate() -> f32 {
    10.0
}

pub(super) fn default_rain_glass_spawn_limit() -> u32 {
    850
}

pub(super) fn default_rain_glass_spawn_size() -> [f32; 2] {
    [45.0, 118.0]
}

pub(super) fn default_rain_glass_refract_base() -> f32 {
    0.34
}

pub(super) fn default_rain_glass_refract_scale() -> f32 {
    0.76
}

pub(super) fn default_rain_glass_light_bump() -> f32 {
    0.78
}

pub(super) fn default_rain_glass_seed() -> u32 {
    121713
}

pub(super) fn default_rain_glass_gravity_px_per_sec2() -> f32 {
    2400.0
}

pub(super) fn default_rain_glass_slip_rate() -> f32 {
    0.34
}

pub(super) fn default_rain_glass_motion_interval() -> [f32; 2] {
    [0.08, 0.32]
}

pub(super) fn default_rain_glass_x_shifting() -> [f32; 2] {
    [0.0, 0.12]
}

pub(super) fn default_rain_glass_collider_scale() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_initial_spread() -> f32 {
    0.52
}

pub(super) fn default_rain_glass_shrink_rate() -> f32 {
    0.014
}

pub(super) fn default_rain_glass_velocity_spread() -> f32 {
    0.34
}

pub(super) fn default_rain_glass_evaporate() -> f32 {
    11.0
}

pub(super) fn default_rain_glass_trail_density() -> f32 {
    0.3224
}

pub(super) fn default_rain_glass_trail_size() -> [f32; 2] {
    [0.2436, 0.5124]
}

pub(super) fn default_rain_glass_trail_distance_px() -> [f32; 2] {
    [10.36, 21.76]
}

pub(super) fn default_rain_glass_trail_spread() -> f32 {
    1.06024
}

pub(super) fn default_rain_glass_trail_shrink_rate() -> f32 {
    0.364
}

pub(super) fn default_rain_glass_trail_evaporate() -> f32 {
    18.0
}

pub(super) fn default_rain_glass_trail_taper() -> f32 {
    0.68
}

pub(super) fn default_rain_glass_streak_boost() -> f32 {
    0.72
}

pub(super) fn default_rain_glass_streak_length() -> f32 {
    1.15
}

pub(super) fn default_rain_glass_micro_droplets_per_second() -> f32 {
    620.0
}

pub(super) fn default_rain_glass_micro_droplet_size() -> [f32; 2] {
    [8.0, 27.0]
}

pub(super) fn default_rain_glass_mist_opacity() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_mist_blur_px() -> f32 {
    4.0
}

pub(super) fn default_rain_glass_mist_accumulation() -> f32 {
    0.012
}

pub(super) fn default_rain_glass_mist_time() -> f32 {
    16.0
}

pub(super) fn default_rain_glass_mist_color_strength() -> f32 {
    0.012
}

pub(super) fn default_rain_glass_mist_blur_step() -> u32 {
    4
}

pub(super) fn default_rain_glass_background_blur_px() -> f32 {
    2.0
}

pub(super) fn default_rain_glass_background_blur_steps() -> u32 {
    2
}

pub(super) fn default_rain_glass_smooth_edge() -> [f32; 2] {
    [0.945, 0.992]
}

pub(super) fn default_rain_glass_distortion_px() -> f32 {
    28.0
}

pub(super) fn default_rain_glass_normal_strength() -> f32 {
    6.0
}

pub(super) fn default_rain_glass_focus_blur_strength() -> f32 {
    0.85
}

pub(super) fn default_rain_glass_body_opacity() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_scene_blend() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_receives_scene_light() -> bool {
    true
}

pub(super) fn default_rain_glass_scene_light_tint_strength() -> f32 {
    0.35
}

pub(super) fn default_rain_glass_scene_shadow_floor() -> f32 {
    0.28
}

pub(super) fn default_rain_glass_blood_tint() -> [f32; 3] {
    [0.55, 0.015, 0.01]
}

pub(super) fn default_rain_glass_blood_amount() -> f32 {
    0.0
}

pub(super) fn default_rain_glass_scene_darken() -> f32 {
    0.0
}

pub(super) fn default_rain_glass_trail_refract_scale() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_trail_opacity() -> f32 {
    1.0
}

pub(super) fn default_rain_glass_reference_mode() -> bool {
    true
}

pub(super) fn default_rain_glass_raindrop_compose() -> String {
    "smoother".to_owned()
}

pub(super) fn default_rain_glass_raindrop_eraser_size() -> [f32; 2] {
    [0.93, 1.0]
}

pub(super) fn default_rain_glass_light_pos() -> [f32; 4] {
    [-1.0, 1.0, 2.0, 0.0]
}

pub(super) fn default_rain_glass_diffuse() -> [f32; 3] {
    [0.22, 0.22, 0.22]
}

pub(super) fn default_rain_glass_shadow_offset() -> f32 {
    0.76
}

pub(super) fn default_rain_glass_specular() -> [f32; 3] {
    [0.025, 0.025, 0.025]
}

pub(super) fn default_rain_glass_specular_shininess() -> f32 {
    300.0
}

pub(super) fn default_rain_glass_scene_light_response() -> f32 {
    0.0
}

pub(super) fn default_rain_glass_rim_strength() -> f32 {
    0.0
}
