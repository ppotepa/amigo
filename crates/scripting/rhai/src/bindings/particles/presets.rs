use super::ParticlesApi;

impl ParticlesApi {
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
}
