impl Particle2dSceneService {
    pub fn set_intensity(&self, entity_name: &str, intensity: f32) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) || !intensity.is_finite() {
            return false;
        }
        state
            .intensities
            .insert(entity_name.to_owned(), intensity.clamp(0.0, 1.0));
        true
    }

    pub fn set_spawn_rate(&self, entity_name: &str, spawn_rate: f32) -> bool {
        if !spawn_rate.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.spawn_rate = spawn_rate.max(0.0);
        })
    }

    pub fn set_particle_lifetime(&self, entity_name: &str, lifetime: f32) -> bool {
        if !lifetime.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.particle_lifetime = lifetime.max(0.001);
        })
    }

    pub fn set_lifetime_jitter(&self, entity_name: &str, jitter: f32) -> bool {
        if !jitter.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.lifetime_jitter = jitter.max(0.0);
        })
    }

    pub fn set_max_particles(&self, entity_name: &str, max_particles: usize) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.max_particles = max_particles;
        })
    }

    pub fn set_initial_speed(&self, entity_name: &str, speed: f32) -> bool {
        if !speed.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.initial_speed = speed.max(0.0);
        })
    }

    pub fn set_speed_jitter(&self, entity_name: &str, jitter: f32) -> bool {
        if !jitter.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.speed_jitter = jitter.max(0.0);
        })
    }

    pub fn set_spread_radians(&self, entity_name: &str, spread_radians: f32) -> bool {
        if !spread_radians.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.spread_radians = spread_radians.max(0.0);
        })
    }

    pub fn set_local_direction_radians(&self, entity_name: &str, radians: f32) -> bool {
        if !radians.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.local_direction_radians = radians;
        })
    }

    pub fn set_inherit_parent_velocity(&self, entity_name: &str, scale: f32) -> bool {
        if !scale.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.inherit_parent_velocity = scale;
        })
    }

    pub fn set_velocity_mode(
        &self,
        entity_name: &str,
        velocity_mode: ParticleVelocityMode2d,
    ) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.velocity_mode = velocity_mode;
        })
    }

    pub fn set_initial_size(&self, entity_name: &str, size: f32) -> bool {
        if !size.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.initial_size = size.max(0.0);
        })
    }

    pub fn set_final_size(&self, entity_name: &str, size: f32) -> bool {
        if !size.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.final_size = size.max(0.0);
        })
    }

    pub fn set_z_index(&self, entity_name: &str, z_index: f32) -> bool {
        if !z_index.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter.z_index = z_index;
        })
    }

    pub fn set_color(&self, entity_name: &str, color: ColorRgba) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color = color;
            emitter.color_ramp = None;
        })
    }

    pub fn set_color_ramp(&self, entity_name: &str, color_ramp: ColorRamp) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color_ramp = Some(color_ramp);
        })
    }

    pub fn clear_color_ramp(&self, entity_name: &str) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.color_ramp = None;
        })
    }

    pub fn set_emission_rate_curve(&self, entity_name: &str, curve: Curve1d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.emission_rate_curve = curve;
        })
    }

    pub fn set_size_curve(&self, entity_name: &str, curve: Curve1d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.size_curve = curve;
        })
    }

    pub fn set_alpha_curve(&self, entity_name: &str, curve: Curve1d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.alpha_curve = curve;
        })
    }

    pub fn set_speed_curve(&self, entity_name: &str, curve: Curve1d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.speed_curve = curve;
        })
    }

    pub fn set_curve4(
        &self,
        entity_name: &str,
        curve_name: &str,
        v0: f32,
        v1: f32,
        v2: f32,
        v3: f32,
    ) -> bool {
        if !v0.is_finite() || !v1.is_finite() || !v2.is_finite() || !v3.is_finite() {
            return false;
        }
        let curve = Curve1d::Custom {
            points: vec![
                CurvePoint1d {
                    t: 0.0,
                    value: v0.clamp(0.0, 1.0),
                },
                CurvePoint1d {
                    t: 1.0 / 3.0,
                    value: v1.clamp(0.0, 1.0),
                },
                CurvePoint1d {
                    t: 2.0 / 3.0,
                    value: v2.clamp(0.0, 1.0),
                },
                CurvePoint1d {
                    t: 1.0,
                    value: v3.clamp(0.0, 1.0),
                },
            ],
        };
        match curve_name {
            "emission" | "emission_rate" | "rate" => {
                self.set_emission_rate_curve(entity_name, curve)
            }
            "size" => self.set_size_curve(entity_name, curve),
            "alpha" => self.set_alpha_curve(entity_name, curve),
            "speed" => self.set_speed_curve(entity_name, curve),
            _ => false,
        }
    }

}
