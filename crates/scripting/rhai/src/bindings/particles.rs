use std::sync::Arc;

use amigo_2d_particles::{Particle2dSceneService, ParticlePreset2dService};
use amigo_fx::{ColorInterpolation, ColorRamp, ColorStop};
use amigo_math::ColorRgba;

#[derive(Clone)]
pub struct ParticlesApi {
    pub(crate) particles: Option<Arc<Particle2dSceneService>>,
    pub(crate) presets: Option<Arc<ParticlePreset2dService>>,
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

    #[allow(clippy::too_many_arguments)]
    pub fn set_color_ramp4(
        &mut self,
        entity_name: &str,
        interpolation: &str,
        t0: rhai::FLOAT,
        c0: &str,
        t1: rhai::FLOAT,
        c1: &str,
        t2: rhai::FLOAT,
        c2: &str,
        t3: rhai::FLOAT,
        c3: &str,
    ) -> bool {
        let Some(particles) = self.particles.as_ref() else {
            return false;
        };
        let Some(c0) = parse_hex_color(c0) else {
            return false;
        };
        let Some(c1) = parse_hex_color(c1) else {
            return false;
        };
        let Some(c2) = parse_hex_color(c2) else {
            return false;
        };
        let Some(c3) = parse_hex_color(c3) else {
            return false;
        };
        let interpolation = match interpolation {
            "step" => ColorInterpolation::Step,
            _ => ColorInterpolation::LinearRgb,
        };
        particles.set_color_ramp(
            entity_name,
            ColorRamp {
                interpolation,
                stops: vec![
                    ColorStop {
                        t: t0 as f32,
                        color: c0,
                    },
                    ColorStop {
                        t: t1 as f32,
                        color: c1,
                    },
                    ColorStop {
                        t: t2 as f32,
                        color: c2,
                    },
                    ColorStop {
                        t: t3 as f32,
                        color: c3,
                    },
                ],
            },
        )
    }

    pub fn clear_color_ramp(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.clear_color_ramp(entity_name))
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

    pub fn set_wind(
        &mut self,
        entity_name: &str,
        x: rhai::FLOAT,
        y: rhai::FLOAT,
        strength: rhai::FLOAT,
    ) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.set_wind(entity_name, x as f32, y as f32, strength as f32))
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

    pub fn copy_config(&mut self, source_entity_name: &str, target_entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.copy_emitter_config(source_entity_name, target_entity_name))
            .unwrap_or(false)
    }

    pub fn preset_ids(&mut self) -> rhai::Array {
        self.presets
            .as_ref()
            .map(|presets| presets.ids().into_iter().map(rhai::Dynamic::from).collect())
            .unwrap_or_default()
    }

    pub fn preset_label(&mut self, preset_id: &str) -> String {
        self.presets
            .as_ref()
            .and_then(|presets| presets.preset(preset_id))
            .map(|preset| preset.label)
            .unwrap_or_default()
    }

    pub fn preset_category(&mut self, preset_id: &str) -> String {
        self.presets
            .as_ref()
            .and_then(|presets| presets.preset(preset_id))
            .map(|preset| preset.category)
            .unwrap_or_default()
    }

    pub fn preset_tags(&mut self, preset_id: &str) -> String {
        self.presets
            .as_ref()
            .and_then(|presets| presets.preset(preset_id))
            .map(|preset| preset.tags.join(", "))
            .unwrap_or_default()
    }

    pub fn apply_preset(&mut self, preset_id: &str, target_entity_name: &str) -> bool {
        let (Some(particles), Some(presets)) = (self.particles.as_ref(), self.presets.as_ref())
        else {
            return false;
        };
        presets.apply_to_emitter(particles, preset_id, target_entity_name)
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

fn parse_hex_color(raw: &str) -> Option<ColorRgba> {
    let value = raw.strip_prefix('#').unwrap_or(raw);
    if value.len() != 8 || !value.chars().all(|character| character.is_ascii_hexdigit()) {
        return None;
    }
    let r = u8::from_str_radix(&value[0..2], 16).ok()?;
    let g = u8::from_str_radix(&value[2..4], 16).ok()?;
    let b = u8::from_str_radix(&value[4..6], 16).ok()?;
    let a = u8::from_str_radix(&value[6..8], 16).ok()?;
    Some(ColorRgba::new(
        f32::from(r) / 255.0,
        f32::from(g) / 255.0,
        f32::from(b) / 255.0,
        f32::from(a) / 255.0,
    ))
}
