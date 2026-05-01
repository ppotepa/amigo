impl Particle2dSceneService {
    pub fn burst(&self, entity_name: &str, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        *state
            .pending_bursts
            .entry(entity_name.to_owned())
            .or_default() += count;
        true
    }

    pub fn burst_at(&self, entity_name: &str, position: Vec2, count: usize) -> bool {
        if count == 0 {
            return true;
        }
        if !position.x.is_finite() || !position.y.is_finite() {
            return false;
        }
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        state
            .pending_positioned_bursts
            .entry(entity_name.to_owned())
            .or_default()
            .push(PositionedParticleBurst2d { position, count });
        true
    }

    fn update_emitter(
        &self,
        entity_name: &str,
        update: impl FnOnce(&mut ParticleEmitter2d),
    ) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        let Some(command) = state.emitters.get_mut(entity_name) else {
            return false;
        };
        update(&mut command.emitter);
        true
    }

    pub fn intensity(&self, entity_name: &str) -> f32 {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .intensities
            .get(entity_name)
            .copied()
            .unwrap_or(1.0)
    }

    pub fn particle_count(&self, entity_name: &str) -> usize {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .particles
            .get(entity_name)
            .map(Vec::len)
            .unwrap_or(0)
    }

}
