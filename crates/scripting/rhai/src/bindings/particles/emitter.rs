use amigo_2d_particles::ParticleVelocityMode2d;
use amigo_math::ColorRgba;

use super::ParticlesApi;

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

    pub fn set_lifetime_jitter(&mut self, entity_name: &str, jitter: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_lifetime_jitter(entity_name, jitter as f32))
            .unwrap_or(false)
    }

    pub fn set_max_particles(&mut self, entity_name: &str, max_particles: rhai::INT) -> bool {
        if max_particles < 0 {
            return false;
        }
        self.particles
            .as_ref()
            .map(|particles| particles.set_max_particles(entity_name, max_particles as usize))
            .unwrap_or(false)
    }

    pub fn set_speed(&mut self, entity_name: &str, speed: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_initial_speed(entity_name, speed as f32))
            .unwrap_or(false)
    }

    pub fn set_speed_jitter(&mut self, entity_name: &str, jitter: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_speed_jitter(entity_name, jitter as f32))
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

    pub fn set_local_direction_degrees(&mut self, entity_name: &str, degrees: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_local_direction_radians(entity_name, (degrees as f32).to_radians())
            })
            .unwrap_or(false)
    }

    pub fn set_inherit_parent_velocity(&mut self, entity_name: &str, scale: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_inherit_parent_velocity(entity_name, scale as f32))
            .unwrap_or(false)
    }

    pub fn set_velocity_mode(&mut self, entity_name: &str, mode: &str) -> bool {
        let velocity_mode = match mode {
            "source_inertial" | "inertial" | "space" => ParticleVelocityMode2d::SourceInertial,
            "free" => ParticleVelocityMode2d::Free,
            _ => return false,
        };
        self.particles
            .as_ref()
            .map(|particles| particles.set_velocity_mode(entity_name, velocity_mode))
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

    pub fn set_z_index(&mut self, entity_name: &str, z_index: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_z_index(entity_name, z_index as f32))
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

    pub fn set_align(&mut self, entity_name: &str, align: &str) -> bool {
        let align = match align {
            "none" => amigo_2d_particles::ParticleAlignMode2d::None,
            "emitter" => amigo_2d_particles::ParticleAlignMode2d::Emitter,
            "random" => amigo_2d_particles::ParticleAlignMode2d::Random,
            _ => amigo_2d_particles::ParticleAlignMode2d::Velocity,
        };
        self.particles
            .as_ref()
            .map(|particles| particles.set_align(entity_name, align))
            .unwrap_or(false)
    }

    pub fn set_blend_mode(&mut self, entity_name: &str, blend_mode: &str) -> bool {
        let blend_mode = match blend_mode {
            "additive" => amigo_2d_particles::ParticleBlendMode2d::Additive,
            "multiply" => amigo_2d_particles::ParticleBlendMode2d::Multiply,
            "screen" => amigo_2d_particles::ParticleBlendMode2d::Screen,
            _ => amigo_2d_particles::ParticleBlendMode2d::Alpha,
        };
        self.particles
            .as_ref()
            .map(|particles| particles.set_blend_mode(entity_name, blend_mode))
            .unwrap_or(false)
    }

    pub fn set_spawn_area_point(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spawn_area(entity_name, amigo_2d_particles::ParticleSpawnArea2d::Point)
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

    pub fn set_spawn_area_line(&mut self, entity_name: &str, length: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spawn_area(
                    entity_name,
                    amigo_2d_particles::ParticleSpawnArea2d::Line {
                        length: (length as f32).max(0.0),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn set_spawn_area_ring(
        &mut self,
        entity_name: &str,
        inner_radius: rhai::FLOAT,
        outer_radius: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_spawn_area(
                    entity_name,
                    amigo_2d_particles::ParticleSpawnArea2d::Ring {
                        inner_radius: (inner_radius as f32).max(0.0),
                        outer_radius: (outer_radius as f32).max(0.0),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn set_shape_circle(&mut self, entity_name: &str, segments: rhai::INT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_shape(
                    entity_name,
                    amigo_2d_particles::ParticleShape2d::Circle {
                        segments: (segments as u32).max(3),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn set_shape_line(&mut self, entity_name: &str, length: rhai::FLOAT) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_shape(
                    entity_name,
                    amigo_2d_particles::ParticleShape2d::Line {
                        length: (length as f32).max(0.0),
                    },
                )
            })
            .unwrap_or(false)
    }

    pub fn set_shape_quad(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_shape(entity_name, amigo_2d_particles::ParticleShape2d::Quad)
            })
            .unwrap_or(false)
    }

    pub fn set_shape_mix(
        &mut self,
        entity_name: &str,
        circle_weight: rhai::FLOAT,
        line_weight: rhai::FLOAT,
        quad_weight: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| {
                particles.set_shape_choices(
                    entity_name,
                    vec![
                        amigo_2d_particles::WeightedParticleShape2d {
                            shape: amigo_2d_particles::ParticleShape2d::Circle { segments: 8 },
                            weight: circle_weight as f32,
                        },
                        amigo_2d_particles::WeightedParticleShape2d {
                            shape: amigo_2d_particles::ParticleShape2d::Line { length: 14.0 },
                            weight: line_weight as f32,
                        },
                        amigo_2d_particles::WeightedParticleShape2d {
                            shape: amigo_2d_particles::ParticleShape2d::Quad,
                            weight: quad_weight as f32,
                        },
                    ],
                )
            })
            .unwrap_or(false)
    }

    pub fn copy_config(&mut self, source_entity_name: &str, target_entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.copy_emitter_config(source_entity_name, target_entity_name))
            .unwrap_or(false)
    }

    pub fn export_yaml(&mut self, entity_name: &str) -> String {
        self.particles
            .as_ref()
            .and_then(|particles| particles.emitter_yaml(entity_name))
            .unwrap_or_default()
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

    pub fn burst_at(
        &mut self,
        entity_name: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        count: rhai::INT,
    ) -> bool {
        if count <= 0 {
            return true;
        }
        self.particles
            .as_ref()
            .map(|particles| {
                particles.burst_at(
                    entity_name,
                    amigo_math::Vec2::new(x as f32, y as f32),
                    count as usize,
                )
            })
            .unwrap_or(false)
    }
}
