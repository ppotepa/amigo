use std::sync::Arc;

use amigo_2d_particles::Particle2dSceneService;
use amigo_math::ColorRgba;

#[derive(Clone)]
pub struct ParticlesApi {
    pub(crate) particles: Option<Arc<Particle2dSceneService>>,
}

impl ParticlesApi {
    pub fn start(&mut self, entity_name: &str) -> bool {
        self.set_active(entity_name, true)
    }

    pub fn stop(&mut self, entity_name: &str) -> bool {
        self.set_active(entity_name, false)
    }

    pub fn set_active(&mut self, entity_name: &str, active: bool) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_active(entity_name, active))
            .unwrap_or(false)
    }

    pub fn set_intensity(&mut self, entity_name: &str, intensity: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_intensity(entity_name, intensity as f32))
            .unwrap_or(false)
    }

    pub fn set_intensity_int(&mut self, entity_name: &str, intensity: rhai::INT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_intensity(entity_name, intensity as f32))
            .unwrap_or(false)
    }

    pub fn set_spawn_rate(&mut self, entity_name: &str, spawn_rate: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_spawn_rate(entity_name, spawn_rate as f32))
            .unwrap_or(false)
    }

    pub fn set_lifetime(&mut self, entity_name: &str, lifetime: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_particle_lifetime(entity_name, lifetime as f32))
            .unwrap_or(false)
    }

    pub fn set_speed(&mut self, entity_name: &str, speed: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_initial_speed(entity_name, speed as f32))
            .unwrap_or(false)
    }

    pub fn set_spread_degrees(&mut self, entity_name: &str, spread_degrees: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spread_radians(entity_name, (spread_degrees as f32).to_radians())
            })
            .unwrap_or(false)
    }

    pub fn set_initial_size(&mut self, entity_name: &str, size: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_initial_size(entity_name, size as f32))
            .unwrap_or(false)
    }

    pub fn set_final_size(&mut self, entity_name: &str, size: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_final_size(entity_name, size as f32))
            .unwrap_or(false)
    }

    pub fn set_color_rgba(
        &mut self,
        entity_name: &str,
        r: rhai::FLOAT,
        g: rhai::FLOAT,
        b: rhai::FLOAT,
        a: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_color(
                    entity_name,
                    ColorRgba::new(
                        (r as f32).clamp(0.0, 1.0),
                        (g as f32).clamp(0.0, 1.0),
                        (b as f32).clamp(0.0, 1.0),
                        (a as f32).clamp(0.0, 1.0),
                    ),
                )
            })
            .unwrap_or(false)
    }

    pub fn set_gravity(&mut self, entity_name: &str, x: rhai::FLOAT, y: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_gravity(entity_name, x as f32, y as f32))
            .unwrap_or(false)
    }

    pub fn set_drag(&mut self, entity_name: &str, coefficient: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_drag(entity_name, coefficient as f32))
            .unwrap_or(false)
    }

    pub fn clear_forces(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.clear_forces(entity_name))
            .unwrap_or(false)
    }

    pub fn set_spawn_area_point(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles
                    .set_spawn_area(entity_name, amigo_2d_particles::ParticleSpawnArea2d::Point)
            })
            .unwrap_or(false)
    }

    pub fn set_spawn_area_rect(
        &mut self,
        entity_name: &str,
        width: rhai::FLOAT,
        height: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spawn_area(
                    entity_name,
                    amigo_2d_particles::ParticleSpawnArea2d::Rect {
                        size: amigo_math::Vec2::new(
                            (width as f32).max(0.0),
                            (height as f32).max(0.0),
                        ),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn set_spawn_area_circle(&mut self, entity_name: &str, radius: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spawn_area(
                    entity_name,
                    amigo_2d_particles::ParticleSpawnArea2d::Circle {
                        radius: (radius as f32).max(0.0),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn burst(&mut self, entity_name: &str, count: rhai::INT) -> bool {
        if count <= 0 {
            return true;
        }
        self.particles
            .as_ref()
            .map(|particles| particles.burst(entity_name, count as usize))
            .unwrap_or(false)
    }
}
