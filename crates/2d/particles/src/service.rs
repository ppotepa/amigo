use std::collections::BTreeMap;
use std::sync::Mutex;

use amigo_fx::ColorRamp;
use amigo_math::{ColorRgba, Curve1d, CurvePoint1d, Transform2, Vec2};

use crate::model::*;
use crate::runtime::*;
use crate::scene_bridge::particle_emitter_to_scene_yaml;

#[derive(Debug, Default)]
pub struct Particle2dSceneService {
    state: Mutex<Particle2dState>,
}

#[derive(Debug, Default)]
pub struct ParticlePreset2dService {
    presets: Mutex<BTreeMap<String, ParticlePreset2d>>,
}

#[derive(Debug, Default)]
struct Particle2dState {
    emitters: BTreeMap<String, ParticleEmitter2dCommand>,
    particles: BTreeMap<String, Vec<Particle2d>>,
    source_transforms: BTreeMap<String, Transform2>,
    emission_accumulators: BTreeMap<String, f32>,
    active_overrides: BTreeMap<String, bool>,
    intensities: BTreeMap<String, f32>,
    rng_states: BTreeMap<String, u64>,
    pending_bursts: BTreeMap<String, usize>,
    pending_positioned_bursts: BTreeMap<String, Vec<PositionedParticleBurst2d>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct PositionedParticleBurst2d {
    position: Vec2,
    count: usize,
}


include!("service/emitters.rs");
include!("service/config.rs");
include!("service/forces.rs");
include!("service/bursts.rs");
include!("service/runtime.rs");
include!("service/presets.rs");
