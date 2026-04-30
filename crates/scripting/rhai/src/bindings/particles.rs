use std::sync::Arc;

use amigo_2d_particles::Particle2dSceneService;

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
}
