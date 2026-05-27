use amigo_camera_optics_plugin::api::CameraOpticalCoverage2d;

use crate::api::ParticleEmitter2dSource;

pub fn particle_emitter_to_camera_optics(
    source: &ParticleEmitter2dSource,
) -> CameraOpticalCoverage2d {
    CameraOpticalCoverage2d::ParticleCoverage {
        emitter_entity_name: source.id.clone(),
    }
}
