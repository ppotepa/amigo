//! Particle-specific Rhai bindings and authored presets.
//! It keeps emitter controls, force helpers, curves, and preset registration together.

use std::sync::Arc;

use amigo_particles_2d_plugin::{Particle2dSceneService, ParticlePreset2dService};

mod curves;
mod emitter;
mod forces;
mod presets;

#[derive(Clone)]
pub struct ParticlesApi {
    pub(crate) particles: Option<Arc<Particle2dSceneService>>,
    pub(crate) presets: Option<Arc<ParticlePreset2dService>>,
}
fn parse_hex_color(raw: &str) -> Option<amigo_math::ColorRgba> {
    let value = raw.strip_prefix('#').unwrap_or(raw);
    if value.len() != 8 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;
    let a = u8::from_str_radix(&value[6..8], 16).ok()?;
    Some(amigo_math::ColorRgba::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ))
}

pub(crate) fn register_api(engine: &mut rhai::Engine) {
    engine
        .register_type_with_name::<ParticlesApi>("WorldParticles")
        .register_fn("start", ParticlesApi::start)
        .register_fn("stop", ParticlesApi::stop)
        .register_fn("set_active", ParticlesApi::set_active)
        .register_fn("set_intensity", ParticlesApi::set_intensity)
        .register_fn("set_intensity", ParticlesApi::set_intensity_int)
        .register_fn("set_spawn_rate", ParticlesApi::set_spawn_rate)
        .register_fn("set_lifetime", ParticlesApi::set_lifetime)
        .register_fn("set_lifetime_jitter", ParticlesApi::set_lifetime_jitter)
        .register_fn("set_max_particles", ParticlesApi::set_max_particles)
        .register_fn("set_speed", ParticlesApi::set_speed)
        .register_fn("set_speed_jitter", ParticlesApi::set_speed_jitter)
        .register_fn("set_spread_degrees", ParticlesApi::set_spread_degrees)
        .register_fn(
            "set_local_direction_degrees",
            ParticlesApi::set_local_direction_degrees,
        )
        .register_fn(
            "set_inherit_parent_velocity",
            ParticlesApi::set_inherit_parent_velocity,
        )
        .register_fn("set_velocity_mode", ParticlesApi::set_velocity_mode)
        .register_fn("set_initial_size", ParticlesApi::set_initial_size)
        .register_fn("set_final_size", ParticlesApi::set_final_size)
        .register_fn("set_z_index", ParticlesApi::set_z_index)
        .register_fn("set_color_rgba", ParticlesApi::set_color_rgba)
        .register_fn("set_color_ramp4", ParticlesApi::set_color_ramp4)
        .register_fn("clear_color_ramp", ParticlesApi::clear_color_ramp)
        .register_fn("set_curve4", ParticlesApi::set_curve4)
        .register_fn("set_gravity", ParticlesApi::set_gravity)
        .register_fn("set_drag", ParticlesApi::set_drag)
        .register_fn("set_wind", ParticlesApi::set_wind)
        .register_fn("clear_forces", ParticlesApi::clear_forces)
        .register_fn("set_spawn_area_point", ParticlesApi::set_spawn_area_point)
        .register_fn("set_spawn_area_rect", ParticlesApi::set_spawn_area_rect)
        .register_fn("set_spawn_area_circle", ParticlesApi::set_spawn_area_circle)
        .register_fn("set_spawn_area_line", ParticlesApi::set_spawn_area_line)
        .register_fn("set_spawn_area_ring", ParticlesApi::set_spawn_area_ring)
        .register_fn("set_shape_circle", ParticlesApi::set_shape_circle)
        .register_fn("set_shape_line", ParticlesApi::set_shape_line)
        .register_fn("set_shape_quad", ParticlesApi::set_shape_quad)
        .register_fn("set_shape_mix", ParticlesApi::set_shape_mix)
        .register_fn("set_align", ParticlesApi::set_align)
        .register_fn("set_blend_mode", ParticlesApi::set_blend_mode)
        .register_fn("copy_config", ParticlesApi::copy_config)
        .register_fn("export_yaml", ParticlesApi::export_yaml)
        .register_fn("preset_ids", ParticlesApi::preset_ids)
        .register_fn("preset_label", ParticlesApi::preset_label)
        .register_fn("preset_category", ParticlesApi::preset_category)
        .register_fn("preset_tags", ParticlesApi::preset_tags)
        .register_fn("apply_preset", ParticlesApi::apply_preset)
        .register_fn("burst", ParticlesApi::burst)
        .register_fn("burst_at", ParticlesApi::burst_at);
}
