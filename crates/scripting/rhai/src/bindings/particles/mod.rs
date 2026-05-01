use std::sync::Arc;

use amigo_2d_particles::{Particle2dSceneService, ParticlePreset2dService};

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
