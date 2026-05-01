impl Particle2dSceneService {
    pub fn queue_emitter(&self, command: ParticleEmitter2dCommand) {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        state
            .emitters
            .insert(command.entity_name.clone(), command.clone());
        state
            .particles
            .entry(command.entity_name.clone())
            .or_default();
        state
            .emission_accumulators
            .entry(command.entity_name.clone())
            .or_insert(0.0);
        state
            .rng_states
            .entry(command.entity_name.clone())
            .or_insert_with(|| hash_seed(command.entity_name.as_str()));
    }

    pub fn clear(&self) {
        *self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned") =
            Particle2dState::default();
    }

    pub fn emitter(&self, entity_name: &str) -> Option<ParticleEmitter2dCommand> {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .emitters
            .get(entity_name)
            .cloned()
    }

    pub fn emitter_yaml(&self, entity_name: &str) -> Option<String> {
        self.emitter(entity_name)
            .map(|command| particle_emitter_to_scene_yaml(&command.emitter))
    }

    pub fn emitters(&self) -> Vec<ParticleEmitter2dCommand> {
        self.state
            .lock()
            .expect("particle scene service mutex should not be poisoned")
            .emitters
            .values()
            .cloned()
            .collect()
    }

    pub fn set_active(&self, entity_name: &str, active: bool) -> bool {
        let mut state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        if !state.emitters.contains_key(entity_name) {
            return false;
        }
        state
            .active_overrides
            .insert(entity_name.to_owned(), active);
        true
    }

    pub fn is_active(&self, entity_name: &str) -> bool {
        let state = self
            .state
            .lock()
            .expect("particle scene service mutex should not be poisoned");
        state
            .active_overrides
            .get(entity_name)
            .copied()
            .or_else(|| {
                state
                    .emitters
                    .get(entity_name)
                    .map(|command| command.emitter.active)
            })
            .unwrap_or(false)
    }

}
