impl ParticlePreset2dService {
    pub fn register(&self, preset: ParticlePreset2d) -> bool {
        if preset.id.is_empty() {
            return false;
        }
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .insert(preset.id.clone(), preset)
            .is_none()
    }

    pub fn clear(&self) {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .clear();
    }

    pub fn ids(&self) -> Vec<String> {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .keys()
            .cloned()
            .collect()
    }

    pub fn preset(&self, id: &str) -> Option<ParticlePreset2d> {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .get(id)
            .cloned()
    }

    pub fn len(&self) -> usize {
        self.presets
            .lock()
            .expect("particle preset service mutex should not be poisoned")
            .len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn apply_to_emitter(
        &self,
        particle_scene: &Particle2dSceneService,
        preset_id: &str,
        target_entity_name: &str,
    ) -> bool {
        let Some(preset) = self.preset(preset_id) else {
            return false;
        };
        particle_scene.replace_emitter_config(target_entity_name, preset.emitter)
    }
}
