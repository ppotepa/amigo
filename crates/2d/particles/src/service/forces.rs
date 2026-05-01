impl Particle2dSceneService {
    pub fn set_gravity(&self, entity_name: &str, x: f32, y: f32) -> bool {
        if !x.is_finite() || !y.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Gravity { .. }));
            emitter.forces.push(ParticleForce2d::Gravity {
                acceleration: Vec2::new(x, y),
            });
        })
    }

    pub fn set_drag(&self, entity_name: &str, coefficient: f32) -> bool {
        if !coefficient.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Drag { .. }));
            emitter.forces.push(ParticleForce2d::Drag {
                coefficient: coefficient.max(0.0),
            });
        })
    }

    pub fn set_wind(&self, entity_name: &str, x: f32, y: f32, strength: f32) -> bool {
        if !x.is_finite() || !y.is_finite() || !strength.is_finite() {
            return false;
        }
        self.update_emitter(entity_name, |emitter| {
            emitter
                .forces
                .retain(|force| !matches!(force, ParticleForce2d::Wind { .. }));
            emitter.forces.push(ParticleForce2d::Wind {
                velocity: Vec2::new(x, y),
                strength: strength.max(0.0),
            });
        })
    }

    pub fn clear_forces(&self, entity_name: &str) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.forces.clear();
        })
    }

    pub fn set_spawn_area(&self, entity_name: &str, spawn_area: ParticleSpawnArea2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.spawn_area = spawn_area;
        })
    }

    pub fn set_shape(&self, entity_name: &str, shape: ParticleShape2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.shape = shape;
            emitter.shape_choices.clear();
        })
    }

    pub fn set_shape_choices(
        &self,
        entity_name: &str,
        choices: Vec<WeightedParticleShape2d>,
    ) -> bool {
        let choices = choices
            .into_iter()
            .filter(|choice| choice.weight.is_finite() && choice.weight > 0.0)
            .collect::<Vec<_>>();
        self.update_emitter(entity_name, |emitter| {
            emitter.shape_choices = choices;
        })
    }

    pub fn set_align(&self, entity_name: &str, align: ParticleAlignMode2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.align = align;
        })
    }

    pub fn set_blend_mode(&self, entity_name: &str, blend_mode: ParticleBlendMode2d) -> bool {
        self.update_emitter(entity_name, |emitter| {
            emitter.blend_mode = blend_mode;
        })
    }

    pub fn copy_emitter_config(&self, source_entity_name: &str, target_entity_name: &str) -> bool {
        let Some(source) = self.emitter(source_entity_name) else {
            return false;
        };
        self.replace_emitter_config(target_entity_name, source.emitter)
    }

    pub fn replace_emitter_config(
        &self,
        target_entity_name: &str,
        emitter: ParticleEmitter2d,
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let Some(target) = state.emitters.get_mut(target_entity_name) else {
            return false;
        };

        target.emitter = emitter;
        state
            .particles
            .entry(target_entity_name.to_owned())
            .or_default()
            .clear();
        state
            .emission_accumulators
            .insert(target_entity_name.to_owned(), 0.0);
        state.active_overrides.remove(target_entity_name);
        state.intensities.remove(target_entity_name);
        true
    }

}
