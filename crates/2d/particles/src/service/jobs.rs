#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Particle2dFrameJobStats {
    pub updated_emitters: usize,
    pub live_particles: usize,
    pub spawned_particles: usize,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Particle2dFrameJobResult {
    pub stats: Particle2dFrameJobStats,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct Particle2dFrameJobInput {
    pub emitter_inputs: Vec<Particle2dEmitterRuntimeInput>,
    pub delta_seconds: f32,
}

