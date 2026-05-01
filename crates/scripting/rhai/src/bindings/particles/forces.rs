use super::ParticlesApi;

impl ParticlesApi {
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
            .map(|particles| {
                particles.set_wind(entity_name, x as f32, y as f32, strength as f32)
            })
            .unwrap_or(false)
    }

    pub fn clear_forces(&mut self, entity_name: &str) -> bool {
        self.particles
            .as_ref()
            .map(|particles| particles.clear_forces(entity_name))
            .unwrap_or(false)
    }
}
